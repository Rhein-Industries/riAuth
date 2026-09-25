//! Audit review and CSV reports. Operational control, not a compliance certification.
use crate::{
    core::{AUDIT_RETENTION_SECONDS, Core},
    crypto::{self, now},
    error::{Error, Result},
    model::{Audit, Group, User},
    store::Tx,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const EXAMINE_CAP: usize = 10_000;
const GROUP_CAP: usize = 10_000;
const CURSOR_TTL: u64 = 3600;
const AUDIT_CURSOR: &[u8] = b"riauth.audit.page/v1";
const USER_CURSOR: &[u8] = b"riauth.users.page/v1";
const USER_HEADER: &[&str] = &[
    "id",
    "username",
    "email",
    "display_name",
    "enabled",
    "admin",
    "created_at",
    "groups",
];
const AUDIT_HEADER: &[&str] = &[
    "id",
    "at",
    "actor",
    "action",
    "target",
    "run_id",
    "request_id",
];

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AuditReviewQuery {
    pub action: Option<String>,
    pub actor: Option<String>,
    pub target: Option<String>,
    pub run_id: Option<String>,
    pub from: Option<u64>,
    pub to: Option<u64>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UserReportQuery {
    /// Username substring, matching inventory's user filter.
    pub filter: Option<String>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

#[derive(Debug)]
pub struct ReportPage {
    pub body: String,
    pub next_cursor: Option<String>,
    pub rows: usize,
}

struct AuditFilter {
    action: Option<String>,
    actor: Option<String>,
    target: Option<String>,
    run_id: Option<String>,
    from: Option<u64>,
    to: Option<u64>,
}

impl AuditFilter {
    fn parse(query: &AuditReviewQuery) -> Result<Self> {
        let action = bounded_text(&query.action, 256)?;
        if action.as_ref().is_some_and(|value| {
            matches!(value.as_str(), "*" | "." | "*.")
                || value.chars().all(|c| c == '*' || c == '.')
        }) {
            return Err(Error::bad("Invalid report filter"));
        }
        let filter = Self {
            action,
            actor: bounded_text(&query.actor, 256)?,
            target: bounded_text(&query.target, 512)?,
            run_id: bounded_text(&query.run_id, 256)?,
            from: query.from,
            to: query.to,
        };
        if filter
            .from
            .is_some_and(|from| filter.to.is_some_and(|to| from > to))
        {
            return Err(Error::bad("Invalid report time range"));
        }
        Ok(filter)
    }

    fn value(&self) -> Value {
        json!({
            "action": self.action,
            "actor": self.actor,
            "target": self.target,
            "run_id": self.run_id,
            "from": self.from,
            "to": self.to,
        })
    }

    fn matches(&self, event: &Audit) -> bool {
        if self
            .action
            .as_ref()
            .is_some_and(|filter| !action_matches(&event.action, filter))
        {
            return false;
        }
        if self
            .actor
            .as_ref()
            .is_some_and(|actor| event.actor != *actor)
        {
            return false;
        }
        if self
            .target
            .as_ref()
            .is_some_and(|target| event.target != *target)
        {
            return false;
        }
        if self
            .run_id
            .as_ref()
            .is_some_and(|run| event.run_id.as_ref() != Some(run))
        {
            return false;
        }
        if self.from.is_some_and(|from| event.at < from) || self.to.is_some_and(|to| event.at > to)
        {
            return false;
        }
        true
    }
}

/// `user.create` matches that action and `user.create.*`. `user` also matches
/// dot-separated children. A trailing `.` or `*` is a raw prefix (`user.` / `user*`).
fn action_matches(action: &str, filter: &str) -> bool {
    if let Some(prefix) = filter.strip_suffix('*') {
        return !prefix.is_empty() && action.starts_with(prefix);
    }
    if filter.ends_with('.') {
        return filter.len() > 1 && action.starts_with(filter);
    }
    action == filter
        || action
            .strip_prefix(filter)
            .is_some_and(|rest| rest.starts_with('.'))
}

fn bounded_text(value: &Option<String>, max: usize) -> Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_empty() || value.len() > max || value.chars().any(char::is_control) {
        return Err(Error::bad("Invalid report filter"));
    }
    Ok(Some(value.clone()))
}

fn bound_limit(requested: Option<usize>, default: usize, max: usize) -> usize {
    requested.unwrap_or(default).clamp(1, max)
}

fn cursor_key(tx: &Tx<'_>) -> Result<[u8; 32]> {
    let material = tx
        .get::<String>("meta", "dummy_hash")?
        .ok_or_else(|| Error::internal("Missing cursor key material"))?;
    Ok(Sha256::digest(material.as_bytes()).into())
}

fn open_cursor(
    tx: &Tx<'_>,
    context: &[u8],
    actor: &str,
    filter: &Value,
    revision: Option<u64>,
    encoded: Option<&str>,
) -> Result<Option<String>> {
    let Some(encoded) = encoded else {
        return Ok(None);
    };
    if encoded.is_empty() || encoded.len() > 4096 || encoded.chars().any(char::is_control) {
        return Err(Error::bad("Invalid report cursor"));
    }
    let raw = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| Error::bad("Invalid report cursor"))?;
    let plain = crypto::unseal(&cursor_key(tx)?, context, &raw)
        .map_err(|_| Error::bad("Invalid report cursor"))?;
    let data: Value =
        serde_json::from_slice(&plain).map_err(|_| Error::bad("Invalid report cursor"))?;
    if data["v"] != json!(1) || data["actor"] != actor || data["filter"] != *filter {
        return Err(Error::bad(
            "Report cursor belongs to another query or actor",
        ));
    }
    if revision.is_some_and(|revision| data["revision"] != revision) {
        return Err(Error::conflict(
            "Report changed or cursor expired; restart enumeration",
        ));
    }
    if data["expires_at"]
        .as_u64()
        .is_none_or(|expires| expires <= now())
    {
        return Err(Error::conflict(
            "Report changed or cursor expired; restart enumeration",
        ));
    }
    let after = data["after"].as_str().filter(|after| {
        !after.is_empty() && after.len() <= 256 && !after.chars().any(char::is_control)
    });
    after
        .map(str::to_owned)
        .map(Some)
        .ok_or_else(|| Error::bad("Invalid report cursor"))
}

fn seal_cursor(
    tx: &Tx<'_>,
    context: &[u8],
    actor: &str,
    filter: &Value,
    revision: Option<u64>,
    after: &str,
) -> Result<String> {
    let body = json!({
        "v": 1,
        "actor": actor,
        "filter": filter,
        "revision": revision,
        "expires_at": now() + CURSOR_TTL,
        "after": after,
    });
    let plain = serde_json::to_vec(&body).map_err(Error::internal)?;
    Ok(URL_SAFE_NO_PAD.encode(crypto::seal(&cursor_key(tx)?, context, &plain)?))
}

fn scan_page<T, F>(
    tx: &Tx<'_>,
    bucket: &str,
    cursor: Option<&str>,
    limit: usize,
    reverse: bool,
    mut accept: F,
) -> Result<(Vec<T>, Option<String>)>
where
    T: DeserializeOwned,
    F: FnMut(&T) -> Result<bool>,
{
    let mut items = Vec::new();
    let mut position = cursor.map(str::to_owned);
    let mut examined = 0usize;
    let mut exhausted = false;
    loop {
        let batch = if reverse {
            tx.scan_reverse::<T>(bucket, position.as_deref(), 256)?
        } else {
            tx.scan::<T>(bucket, position.as_deref(), 256)?
        };
        if batch.is_empty() {
            exhausted = true;
            break;
        }
        let len = batch.len();
        let full = len == 256;
        let mut stop = false;
        for (index, (key, value)) in batch.into_iter().enumerate() {
            position = Some(key);
            examined += 1;
            if accept(&value)? {
                items.push(value);
            }
            let reached = items.len() == limit || examined >= EXAMINE_CAP;
            let end = !full && index + 1 == len;
            if reached || end {
                exhausted = end;
                stop = true;
                break;
            }
        }
        if stop || !full {
            break;
        }
    }
    let next = if exhausted { None } else { position };
    Ok((items, next))
}

fn memberships(tx: &Tx<'_>) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut after = None;
    let mut seen = 0usize;
    loop {
        let batch = tx.scan::<Group>("groups", after.as_deref(), 256)?;
        if batch.is_empty() {
            break;
        }
        seen += batch.len();
        if seen > GROUP_CAP {
            return Err(Error::bad("Too many groups to export users"));
        }
        let full = batch.len() == 256;
        after = batch.last().map(|(key, _)| key.clone());
        for (_, group) in batch {
            for member in group.members {
                index.entry(member).or_default().insert(group.name.clone());
            }
        }
        if !full {
            break;
        }
    }
    Ok(index)
}

fn project_event(event: &Audit) -> Value {
    let mut changes = event
        .details
        .get("changes")
        .cloned()
        .unwrap_or_else(|| json!([]));
    crate::store::redact_audit_value(&mut changes);
    json!({
        "id": event.id,
        "at": event.at,
        "actor": event.actor,
        "action": event.action,
        "target": event.target,
        "run_id": event.run_id,
        "request_id": event.details.get("request_id").cloned().filter(|value| !value.is_null()).unwrap_or(Value::Null),
        "changes": changes,
    })
}

fn request_id(event: &Audit) -> String {
    event
        .details
        .get("request_id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

pub(crate) fn csv_cell(value: &str) -> String {
    let mut text = String::with_capacity(value.len() + 1);
    if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        text.push('\'');
    }
    text.push_str(value);
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text
    }
}

fn write_row<'a>(out: &mut String, cells: impl IntoIterator<Item = &'a str>) {
    let mut first = true;
    for cell in cells {
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str(&csv_cell(cell));
    }
    out.push('\n');
}

fn csv_document(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    write_row(&mut out, header.iter().copied());
    for row in rows {
        write_row(&mut out, row.iter().map(String::as_str));
    }
    out
}

impl Core {
    fn audit_page(
        &self,
        token: &str,
        query: &AuditReviewQuery,
        default_limit: usize,
        max_limit: usize,
    ) -> Result<(Vec<Audit>, Option<String>, usize)> {
        let filter = AuditFilter::parse(query)?;
        let limit = bound_limit(query.limit, default_limit, max_limit);
        self.store.read(|tx| {
            let actor = self.management(tx, token, "audit.read", "audit/events")?;
            let filter_value = filter.value();
            let cursor = open_cursor(
                tx,
                AUDIT_CURSOR,
                &actor.id,
                &filter_value,
                None,
                query.cursor.as_deref(),
            )?;
            let (events, next_key) = scan_page(
                tx,
                "audit",
                cursor.as_deref(),
                limit,
                true,
                |event: &Audit| Ok(filter.matches(event)),
            )?;
            let next_cursor = next_key
                .as_deref()
                .map(|after| seal_cursor(tx, AUDIT_CURSOR, &actor.id, &filter_value, None, after))
                .transpose()?;
            Ok((events, next_cursor, limit))
        })
    }

    pub fn audit_review(&self, token: &str, query: AuditReviewQuery) -> Result<Value> {
        let (events, next_cursor, limit) = self.audit_page(token, &query, 100, 500)?;
        Ok(json!({
            "events": events.iter().map(project_event).collect::<Vec<_>>(),
            "next_cursor": next_cursor,
            "limit": limit,
            "retention_seconds": AUDIT_RETENTION_SECONDS,
        }))
    }

    pub fn audit_csv(&self, token: &str, query: AuditReviewQuery) -> Result<ReportPage> {
        let (events, next_cursor, _) = self.audit_page(token, &query, 100, 1000)?;
        let rows = events
            .iter()
            .map(|event| {
                vec![
                    event.id.clone(),
                    event.at.to_string(),
                    event.actor.clone(),
                    event.action.clone(),
                    event.target.clone(),
                    event.run_id.clone().unwrap_or_default(),
                    request_id(event),
                ]
            })
            .collect::<Vec<_>>();
        let count = rows.len();
        Ok(ReportPage {
            body: csv_document(AUDIT_HEADER, &rows),
            next_cursor,
            rows: count,
        })
    }

    pub fn users_csv(&self, token: &str, query: UserReportQuery) -> Result<ReportPage> {
        let filter = bounded_text(&query.filter, 256)?;
        let limit = bound_limit(query.limit, 100, 1000);
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            if actor.agent
                && !actor
                    .permissions
                    .iter()
                    .any(|permission| permission.action == "user.read")
            {
                return Err(Error::forbidden());
            }
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let filter_value = json!(filter);
            let cursor = open_cursor(
                tx,
                USER_CURSOR,
                &actor.id,
                &filter_value,
                Some(revision),
                query.cursor.as_deref(),
            )?;
            let index = memberships(tx)?;
            let (users, next_key) = scan_page(
                tx,
                "users",
                cursor.as_deref(),
                limit,
                false,
                |user: &User| {
                    Ok(
                        actor.allows("user.read", &format!("user/{}", user.username))
                            && filter
                                .as_ref()
                                .is_none_or(|needle| user.username.contains(needle)),
                    )
                },
            )?;
            let rows = users
                .into_iter()
                .map(|user| {
                    let groups = index
                        .get(&user.id)
                        .map(|names| names.iter().cloned().collect::<Vec<_>>().join(";"))
                        .unwrap_or_default();
                    vec![
                        user.id,
                        user.username,
                        user.email.unwrap_or_default(),
                        user.display_name,
                        user.enabled.to_string(),
                        user.admin.to_string(),
                        user.created_at.to_string(),
                        groups,
                    ]
                })
                .collect::<Vec<_>>();
            let count = rows.len();
            let next_cursor = next_key
                .as_deref()
                .map(|after| {
                    seal_cursor(
                        tx,
                        USER_CURSOR,
                        &actor.id,
                        &filter_value,
                        Some(revision),
                        after,
                    )
                })
                .transpose()?;
            Ok(ReportPage {
                body: csv_document(USER_HEADER, &rows),
                next_cursor,
                rows: count,
            })
        })
    }
}
