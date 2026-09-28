//! Cursor paging for the management User list. The legacy array remains in Core.

use crate::{
    core::Core,
    crypto::{self, now},
    error::{Error, Result},
    model::{User, UserView},
    store::{Tx, maintenance::PAGE},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const MAX_LIMIT: usize = 100;
const MAX_EXAMINED: usize = 10_000;
const CURSOR_TTL: u64 = 3600;
const CONTEXT: &[u8] = b"riauth.users.json-page/v1";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    version: u8,
    actor: String,
    credential: String,
    authority: String,
    revision: u64,
    users_generation: u64,
    #[serde(default)]
    cursor_epoch: String,
    limit: usize,
    expires_at: u64,
    after: String,
}

fn key(tx: &Tx<'_>) -> Result<[u8; 32]> {
    let material = tx
        .get::<String>("meta", "dummy_hash")?
        .ok_or_else(|| Error::internal("Missing cursor key material"))?;
    Ok(Sha256::digest(material.as_bytes()).into())
}

impl Core {
    /// A page contains at most `limit` visible users. Sparse permissions may
    /// produce an empty page with a cursor; callers continue until it is null.
    pub fn list_users_page(
        &self,
        token: &str,
        limit: usize,
        encoded: Option<String>,
    ) -> Result<Value> {
        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad("User list limit must be 1–100"));
        }
        if encoded.as_ref().is_some_and(|cursor| {
            cursor.is_empty() || cursor.len() > 4096 || cursor.chars().any(char::is_control)
        }) {
            return Err(Error::bad("Invalid user list cursor"));
        }
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let users_generation = tx
                .get::<u64>("user_listing_generation", "all")?
                .unwrap_or(0);
            let cursor_epoch = tx
                .get::<String>("meta", "user_listing_cursor_epoch")?
                .unwrap_or_default();
            let grant_generation = if actor.delegated {
                tx.get::<u64>("human_grant_generations", &actor.id)?
                    .unwrap_or(0)
            } else {
                0
            };
            let authority = crypto::digest(
                &serde_json::to_string(&json!({
                    "agent": actor.agent,
                    "delegated": actor.delegated,
                    "permissions": &actor.permissions,
                    "grants": &actor.grants,
                    "grant_generation": grant_generation,
                }))
                .map_err(Error::internal)?,
            );
            let credential = crypto::digest(token);
            let key = key(tx)?;
            let mut after = if let Some(encoded) = encoded {
                let raw = URL_SAFE_NO_PAD
                    .decode(encoded)
                    .map_err(|_| Error::bad("Invalid user list cursor"))?;
                let plain = crypto::unseal(&key, CONTEXT, &raw)
                    .map_err(|_| Error::bad("Invalid user list cursor"))?;
                let cursor: Cursor = serde_json::from_slice(&plain)
                    .map_err(|_| Error::bad("Invalid user list cursor"))?;
                if cursor.version != 1
                    || cursor.actor != actor.id
                    || cursor.credential != credential
                    || cursor.limit != limit
                    || cursor.after.is_empty()
                    || cursor.after.len() > 256
                    || cursor.after.chars().any(char::is_control)
                {
                    return Err(Error::bad(
                        "User list cursor belongs to another query or actor",
                    ));
                }
                if cursor.revision != revision
                    || cursor.users_generation != users_generation
                    || cursor.cursor_epoch != cursor_epoch
                    || cursor.authority != authority
                    || cursor.expires_at <= now()
                {
                    return Err(Error::conflict(
                        "User list or authorization changed; restart pagination",
                    ));
                }
                Some(cursor.after)
            } else {
                None
            };

            let mut items = Vec::new();
            let mut examined = 0usize;
            let exhausted = 'pages: loop {
                let batch_limit = PAGE.min(MAX_EXAMINED - examined);
                let batch = tx.scan::<User>("users", after.as_deref(), batch_limit)?;
                if batch.is_empty() {
                    break true;
                }
                let length = batch.len();
                let full = length == batch_limit;
                for (index, (key, user)) in batch.into_iter().enumerate() {
                    after = Some(key);
                    examined += 1;
                    if actor.allows("user.read", &format!("user/{}", user.username)) {
                        items.push(json!(UserView::from(&user)));
                    }
                    let end = !full && index + 1 == length;
                    if items.len() == limit || examined == MAX_EXAMINED || end {
                        break 'pages end;
                    }
                }
                if !full {
                    break true;
                }
            };
            let next_cursor = if exhausted {
                None
            } else {
                let cursor = Cursor {
                    version: 1,
                    actor: actor.id,
                    credential,
                    authority,
                    revision,
                    users_generation,
                    cursor_epoch,
                    limit,
                    expires_at: now() + CURSOR_TTL,
                    after: after.expect("a non-exhausted page examined a record"),
                };
                let plain = serde_json::to_vec(&cursor).map_err(Error::internal)?;
                Some(URL_SAFE_NO_PAD.encode(crypto::seal(&key, CONTEXT, &plain)?))
            };
            Ok(json!({
                "items": items,
                "next_cursor": next_cursor,
                "limit": limit,
                "revision": revision,
            }))
        })
    }
}
