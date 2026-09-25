//! Derived indexes commit with their source records. Timestamp/hash keys expose
//! scheduling metadata; values retain the configured record encryption.
use super::*;

pub const PAGE: usize = 128;
pub const INDEX_VERSION: u32 = 2;
pub const QUEUES: [&str; 5] = [
    "logout_deliveries",
    "mail_deliveries",
    "provisioning_jobs",
    "ssf_deliveries",
    "offboard_jobs",
];
const COUNTED: [&str; 2] = ["http_rates", "mail_limits"];
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct QueueStats {
    pub pending: u64,
    pub failed: u64,
    pub oldest_pending_seconds: u64,
}
fn queue_state(bucket: &str, value: &Value) -> (bool, bool, u64, u64) {
    if bucket == "offboard_jobs" {
        let running = value["status"] == "running";
        let pending = running || value["status"] == "scheduled";
        let due = value["next_attempt"].as_u64().unwrap_or(0).max(
            value[if running { "lease_until" } else { "execute_at" }]
                .as_u64()
                .unwrap_or(0),
        );
        return (
            pending,
            value["status"] == "failed",
            due,
            value["created_at"].as_u64().unwrap_or(0),
        );
    }
    let delivered = !value["delivered_at"].is_null();
    let stopped = value["stopped"] == true || value["stale"] == true;
    let pending = !delivered && !stopped && value["completed"] != true;
    let failed = stopped
        || value["last_failed"] == true
        || value["next_attempt"].as_u64() == Some(u64::MAX)
        || !value["error"].is_null()
        || value["last_status"]
            .as_u64()
            .is_some_and(|s| !(200..300).contains(&s))
        || bucket == "mail_deliveries"
            && !delivered
            && value["attempts"].as_u64().unwrap_or(0) > 0
            && value["lease"].is_null();
    let created = value["created_at"].as_u64().unwrap_or_else(|| {
        value["plan"]["expires_at"]
            .as_u64()
            .unwrap_or(3600)
            .saturating_sub(3600)
    });
    (
        pending,
        failed,
        value["next_attempt"].as_u64().unwrap_or(0),
        created,
    )
}
fn index_key(at: u64, id: &str) -> String {
    format!("{at:020}/{}", crypto::digest(id))
}
fn counter_change(value: u64, before: bool, after: bool) -> u64 {
    match (before, after) {
        (false, true) => value.saturating_add(1),
        (true, false) => value.saturating_sub(1),
        _ => value,
    }
}
impl Tx<'_> {
    pub(super) fn update_indexes(
        &self,
        bucket: &str,
        id: &str,
        after: Option<&Value>,
    ) -> Result<()> {
        if bucket == "access_grants" {
            let before = self.get::<Value>(bucket, id)?;
            self.update_grant_index(id, before.as_ref(), after)?;
        }
        if COUNTED.contains(&bucket) {
            let previous = self.get::<Value>(bucket, id)?;
            let before = previous.is_some();
            let expiry_bucket = format!("index_expiry_{bucket}");
            let ttl = if bucket == "http_rates" { 60 } else { 3600 };
            if let Some(at) = previous.as_ref().and_then(|v| v[0].as_u64()) {
                self.delete(&expiry_bucket, &index_key(at.saturating_add(ttl), id))?;
            }
            if let Some(at) = after.and_then(|v| v[0].as_u64()) {
                self.put(&expiry_bucket, &index_key(at.saturating_add(ttl), id), &id)?;
            }
            let count = self.get::<u64>("index_counts", bucket)?.unwrap_or(0);
            let next = counter_change(count, before, after.is_some());
            if next != count {
                self.put("index_counts", bucket, &next)?;
            }
        }
        if matches!(bucket, "access" | "refresh")
            && let Some(value) = after
            && let (Some(session), Some(expiry)) = (
                value["identity"]["session_id"].as_str(),
                value["expires_at"].as_u64(),
            )
        {
            let previous = self.get::<u64>("session_retention", session)?.unwrap_or(0);
            if expiry > previous {
                self.put("session_retention", session, &expiry)?;
            }
        }
        if !QUEUES.contains(&bucket) {
            return Ok(());
        }
        let before = self.get::<Value>(bucket, id)?;
        self.update_queue_indexes(bucket, id, before.as_ref(), after)
    }
    fn update_grant_index(
        &self,
        id: &str,
        before: Option<&Value>,
        after: Option<&Value>,
    ) -> Result<()> {
        let index = |value: &Value| {
            value["user_id"]
                .as_str()
                .filter(|_| value["revoked_at"].is_null())
                .map(|user| format!("{}/{}", crypto::digest(user), crypto::digest(id)))
        };
        if let Some(key) = before.and_then(index) {
            self.delete("index_user_access_grants", &key)?;
        }
        if let Some(key) = after.and_then(index) {
            self.put("index_user_access_grants", &key, &id)?;
        }
        Ok(())
    }
    /// Only grants for this user are visited; revoked grants are absent from the index.
    pub fn user_access_grants<T: DeserializeOwned>(&self, user_id: &str) -> Result<Vec<T>> {
        let prefix = format!("index_user_access_grants/{}/", crypto::digest(user_id));
        let records = self.scan_records(Range {
            start: prefix.clone(),
            end: format!("{}0", prefix.trim_end_matches('/')),
            limit: usize::MAX,
            reverse: false,
        })?;
        let mut grants = Vec::new();
        for (name, bytes) in records {
            let id: String = decode(self.key, &name, &bytes)?;
            if let Some(grant) = self.get("access_grants", &id)? {
                grants.push(grant);
            }
        }
        Ok(grants)
    }
    fn update_queue_indexes(
        &self,
        bucket: &str,
        id: &str,
        before: Option<&Value>,
        after: Option<&Value>,
    ) -> Result<()> {
        let old = before.map(|v| queue_state(bucket, v)).unwrap_or_default();
        let new = after.map(|v| queue_state(bucket, v)).unwrap_or_default();
        let due = format!("index_due_{bucket}");
        let age = format!("index_age_{bucket}");
        if old.0 {
            self.delete(&due, &index_key(old.2, id))?;
            self.delete(&age, &index_key(old.3, id))?;
        }
        if new.0 {
            self.put(&due, &index_key(new.2, id), &id)?;
            self.put(&age, &index_key(new.3, id), &new.3)?;
        }
        if old.0 == new.0 && old.1 == new.1 {
            return Ok(());
        }
        let mut stats = self
            .get::<QueueStats>("index_queues", bucket)?
            .unwrap_or_default();
        stats.pending = counter_change(stats.pending, old.0, new.0);
        stats.failed = counter_change(stats.failed, old.1, new.1);
        self.put("index_queues", bucket, &stats)
    }
    pub fn reclaim_expired_limits(&self, bucket: &str, at: u64) -> Result<()> {
        if !COUNTED.contains(&bucket) {
            return Err(Error::internal("Collection has no expiry index"));
        }
        for (key, id) in self.scan::<String>(&format!("index_expiry_{bucket}"), None, 64)? {
            let expiry = key
                .split('/')
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| Error::internal("Invalid expiry index"))?;
            if expiry > at {
                break;
            }
            self.delete(bucket, &id)?;
        }
        Ok(())
    }
    /// Deletes the record whose expiry comes first.
    pub fn evict_oldest_limit(&self, bucket: &str) -> Result<()> {
        if !COUNTED.contains(&bucket) {
            return Err(Error::internal("Collection has no expiry index"));
        }
        if let Some((_, id)) = self
            .scan::<String>(&format!("index_expiry_{bucket}"), None, 1)?
            .into_iter()
            .next()
        {
            self.delete(bucket, &id)?;
        }
        Ok(())
    }
    pub fn collection_count(&self, bucket: &str) -> Result<u64> {
        if !COUNTED.contains(&bucket) {
            return Err(Error::internal("Collection has no count index"));
        }
        Ok(self.get("index_counts", bucket)?.unwrap_or(0))
    }
    pub fn due<T: DeserializeOwned>(
        &self,
        bucket: &str,
        at: u64,
        limit: usize,
    ) -> Result<Vec<(String, T)>> {
        if !QUEUES.contains(&bucket) {
            return Err(Error::internal("Collection has no due index"));
        }
        let mut output = Vec::new();
        for (key, id) in
            self.scan::<String>(&format!("index_due_{bucket}"), None, limit.min(PAGE))?
        {
            let due = key
                .split('/')
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| Error::internal("Invalid due index"))?;
            if due > at {
                break;
            }
            if let Some(record) = self.get(bucket, &id)? {
                output.push((id, record));
            }
        }
        Ok(output)
    }
    pub fn queue_stats(&self, bucket: &str, at: u64) -> Result<QueueStats> {
        if !QUEUES.contains(&bucket) {
            return Err(Error::internal("Collection has no queue index"));
        }
        let mut stats = self
            .get::<QueueStats>("index_queues", bucket)?
            .unwrap_or_default();
        stats.oldest_pending_seconds = self
            .scan::<u64>(&format!("index_age_{bucket}"), None, 1)?
            .first()
            .map(|(_, created)| at.saturating_sub(*created))
            .unwrap_or(0);
        Ok(stats)
    }
    /// A durable cursor advances at most PAGE records per collection each pass.
    /// Freeze the sweep's upper bound so continuous appends cannot prevent wraparound.
    pub fn maintenance_page<T: DeserializeOwned>(&self, bucket: &str) -> Result<Vec<(String, T)>> {
        let cursor = self.get::<String>("maintenance_cursors", bucket)?;
        let end = match self.get::<String>("maintenance_bounds", bucket)? {
            Some(end) => Some(end),
            None => self
                .scan_reverse::<Value>(bucket, None, 1)?
                .into_iter()
                .next()
                .map(|(key, _)| key),
        };
        let Some(end) = end else {
            self.delete("maintenance_cursors", bucket)?;
            self.delete("maintenance_bounds", bucket)?;
            return Ok(Vec::new());
        };
        let records: Vec<_> = self
            .scan(bucket, cursor.as_deref(), PAGE)?
            .into_iter()
            .take_while(|(key, _)| key <= &end)
            .collect();
        if records.len() < PAGE || records.last().is_some_and(|(key, _)| key >= &end) {
            self.delete("maintenance_cursors", bucket)?;
            self.delete("maintenance_bounds", bucket)?;
        } else {
            self.put("maintenance_cursors", bucket, &records.last().unwrap().0)?;
            self.put("maintenance_bounds", bucket, &end)?;
        }
        Ok(records)
    }
    /// Used for offline upgrades and restore, never for ordinary requests.
    pub fn rebuild_indexes(&self) -> Result<()> {
        let mut indexes = vec![
            "index_counts".to_owned(),
            "index_queues".into(),
            "session_retention".into(),
            "index_user_access_grants".into(),
        ];
        for bucket in COUNTED {
            indexes.push(format!("index_expiry_{bucket}"));
        }
        for queue in QUEUES {
            indexes.push(format!("index_due_{queue}"));
            indexes.push(format!("index_age_{queue}"));
        }
        for bucket in indexes {
            for (key, _) in self.list::<Value>(&bucket)? {
                self.delete(&bucket, &key)?;
            }
        }
        for bucket in COUNTED {
            let records = self.list::<Value>(bucket)?;
            self.put("index_counts", bucket, &(records.len() as u64))?;
            for (id, value) in records {
                self.update_indexes(bucket, &id, Some(&value))?;
            }
        }
        for bucket in ["access", "refresh"] {
            for (id, value) in self.list::<Value>(bucket)? {
                self.update_indexes(bucket, &id, Some(&value))?;
            }
        }
        for bucket in QUEUES {
            for (id, value) in self.list::<Value>(bucket)? {
                self.update_queue_indexes(bucket, &id, None, Some(&value))?;
            }
        }
        for (id, value) in self.list::<Value>("access_grants")? {
            self.update_grant_index(&id, None, Some(&value))?;
        }
        self.put("meta", "index_version", &INDEX_VERSION)?;
        Ok(())
    }
}
