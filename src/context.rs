use crate::{
    crypto::now,
    error::Result,
    store::{Store, Tx},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{cell::RefCell, net::IpAddr};

#[derive(Clone, Default)]
pub struct RequestContext {
    pub request_id: String,
    pub run_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub fingerprint: String,
    pub revision: Option<u64>,
    /// The exact quoted entity tag supplied for a SCIM resource mutation.
    pub if_match: Option<String>,
    /// Attributed client address, after trusted-proxy processing.
    pub client_ip: Option<IpAddr>,
    /// User-Agent without control characters, at most 256 bytes.
    pub user_agent: Option<String>,
}
tokio::task_local! { pub static HTTP_CONTEXT: RequestContext; }
thread_local! { static CURRENT: RefCell<Option<RequestContext>> = const { RefCell::new(None) }; }
pub fn current() -> Option<RequestContext> {
    CURRENT.with_borrow(Clone::clone)
}
/// Who asked for an interactive request, shown to the approving terminal.
pub(crate) fn requester() -> Option<Value> {
    current().map(|c| json!({"ip": c.client_ip.map(|ip| ip.to_string()), "user_agent": c.user_agent, "at": now()}))
}
pub fn scope<T>(context: Option<RequestContext>, f: impl FnOnce() -> T) -> T {
    struct Reset(Option<RequestContext>);
    impl Drop for Reset {
        fn drop(&mut self) {
            CURRENT.with_borrow_mut(|v| *v = self.0.take());
        }
    }
    let _reset = Reset(CURRENT.replace(context));
    f()
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Receipt {
    pub(crate) fingerprint: String,
    pub(crate) permissions: Value,
    pub(crate) result: Value,
    pub(crate) expires_at: u64,
}

const AGENT_RECEIPT_REDACTION: &str = "agent_issuance_receipts_redacted_v1";
const REDACTION_PAGE: usize = 128;

fn exact_keys(object: &serde_json::Map<String, Value>, keys: &[&str]) -> bool {
    object.len() == keys.len() && keys.iter().all(|key| object.contains_key(*key))
}

/// Generic receipts from older agent create/rotate writers stored this exact
/// response shape. The receipt has no route field, so keep the matcher narrow:
/// DCR and other secret-return receipts must retain their replay contract.
fn legacy_agent_issuance_id(receipt: &Receipt) -> Option<&str> {
    if !receipt.permissions.as_array().is_some_and(Vec::is_empty) {
        return None;
    }
    let result = receipt.result.as_object()?;
    if !exact_keys(result, &["agent", "credential"]) {
        return None;
    }
    let agent = result.get("agent")?.as_object()?;
    if !exact_keys(
        agent,
        &[
            "id",
            "permissions",
            "expires_at",
            "created_at",
            "enabled",
            "parent_user",
        ],
    ) {
        return None;
    }
    let id = agent.get("id")?.as_str()?;
    if id.is_empty() || !agent.get("enabled")?.as_bool()? {
        return None;
    }
    agent.get("created_at")?.as_u64()?;
    let expires_at = agent.get("expires_at")?.as_u64()?;
    if !agent
        .get("parent_user")
        .is_some_and(|parent| parent.is_null() || parent.is_string())
    {
        return None;
    }
    let permissions = agent.get("permissions")?.as_array()?;
    if permissions.is_empty()
        || permissions.iter().any(|permission| {
            let Some(permission) = permission.as_object() else {
                return true;
            };
            !exact_keys(permission, &["action", "resource"])
                || !permission.get("action").is_some_and(Value::is_string)
                || !permission.get("resource").is_some_and(Value::is_string)
        })
    {
        return None;
    }
    let credential = result.get("credential")?.as_object()?;
    if !exact_keys(credential, &["issuer", "agent_id", "token", "expires_at"])
        || !credential.get("issuer").is_some_and(Value::is_string)
        || credential.get("agent_id")?.as_str()? != id
        || credential.get("expires_at")?.as_u64()? != expires_at
        || !credential.get("token")?.as_str()?.starts_with("ri_agent_")
    {
        return None;
    }
    Some(id)
}

fn redact_legacy_agent_issuance(receipt: &mut Receipt) -> bool {
    let Some(id) = legacy_agent_issuance_id(receipt).map(str::to_owned) else {
        return false;
    };
    receipt.result = json!({"agent_id": id, "credential_issued": true});
    true
}

/// Complete the one-time redaction before this Core is returned to a server.
/// Each page commits independently, so interrupted startup resumes safely; the
/// marker is written only with the final page. Retry keys and expiry survive.
pub(crate) fn migrate_legacy_agent_receipts(store: &Store) -> Result<()> {
    if store.get::<bool>("meta", AGENT_RECEIPT_REDACTION)? == Some(true) {
        return Ok(());
    }
    let mut after: Option<String> = None;
    loop {
        let (last, done) = store.write(|tx| {
            let page = tx.scan::<Receipt>("receipts", after.as_deref(), REDACTION_PAGE)?;
            let done = page.len() < REDACTION_PAGE;
            let last = page.last().map(|(key, _)| key.clone());
            for (key, mut receipt) in page {
                if redact_legacy_agent_issuance(&mut receipt) {
                    tx.put("receipts", &key, &receipt)?;
                }
            }
            if done {
                tx.put("meta", AGENT_RECEIPT_REDACTION, &true)?;
            }
            Ok((last, done))
        })?;
        if done {
            return Ok(());
        }
        after = last;
    }
}
pub(crate) fn replay_receipt(
    tx: &Tx<'_>,
    key: &str,
    fingerprint: &str,
    permissions: &Value,
) -> Result<Option<Value>> {
    let Some(receipt) = tx.get::<Receipt>("receipts", key)? else {
        return Ok(None);
    };
    if receipt.expires_at <= now() {
        return Err(crate::error::Error::conflict(
            "Idempotency receipt expired; inspect state before using a new key",
        ));
    }
    if receipt.fingerprint != fingerprint {
        return Err(crate::error::Error::conflict(
            "Idempotency key was used for a different request",
        ));
    }
    if receipt.permissions != *permissions {
        return Err(crate::error::Error::forbidden());
    }
    Ok(Some(receipt.result))
}
pub(crate) fn save_receipt(
    tx: &Tx<'_>,
    key: &str,
    fingerprint: String,
    permissions: Value,
    result: &Value,
) -> Result<()> {
    tx.put(
        "receipts",
        key,
        &Receipt {
            fingerprint,
            permissions,
            result: result.clone(),
            expires_at: now() + 86_400,
        },
    )
}
pub fn cleanup(tx: &Tx<'_>) -> Result<()> {
    // Keep expired receipts as bounded tombstones for seven days, preventing accidental immediate reuse.
    for (id, mut receipt) in tx.maintenance_page::<Receipt>("receipts")? {
        if receipt.expires_at.saturating_add(6 * 86_400) <= now() {
            tx.delete("receipts", &id)?;
        } else if redact_legacy_agent_issuance(&mut receipt) {
            // A receipt introduced by an older writer or restored after the
            // startup marker is also scrubbed in a successful transaction.
            tx.put("receipts", &id, &receipt)?;
        }
    }
    Ok(())
}
