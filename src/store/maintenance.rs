//! Derived indexes commit with their source records. Timestamp/hash keys expose
//! scheduling metadata; values retain the configured record encryption.
use super::*;

pub const PAGE: usize = 128;
pub const INDEX_VERSION: u32 = 7;
pub const QUEUES: [&str; 6] = [
    "logout_deliveries",
    "mail_deliveries",
    "provisioning_jobs",
    "ssf_deliveries",
    "offboard_jobs",
    "provisioning_deactivations",
];
// Outbound SCIM user links, grouped by local user for the disable transition.
const USER_LINKS: &str = "index_user_provisioning_links";
const GROUP_DN_FOLDS: &str = "index_group_dn_folds";
pub(super) const GROUP_BINDINGS: &str = "index_group_bindings";
pub(super) const GROUP_SOURCE_DIGESTS: &str = "index_group_source_digests";
const COUNTED: [&str; 2] = ["http_rates", "mail_limits"];
// Imported records can approach the archive's per-frame limit. The ordinary
// maintenance PAGE would decode 128 such records before returning to rebuild.
const REBUILD_PAGE: usize = 1;
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct QueueStats {
    pub pending: u64,
    pub failed: u64,
    pub oldest_pending_seconds: u64,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct GroupBinding {
    pub name: String,
    pub source_digest: String,
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
    if bucket == "provisioning_deactivations" {
        let running = value["status"] == "running";
        let due = value["next_attempt"].as_u64().unwrap_or(0).max(if running {
            value["lease_until"].as_u64().unwrap_or(0)
        } else {
            0
        });
        return (
            running || value["status"] == "pending",
            value["status"] == "failed" || value["status"] == "stale",
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
        if bucket == "users" {
            // Authentication can rewrite factors or rehash a password without
            // changing the outbound SCIM projection. Only source changes that
            // could alter a plan invalidate a saved scan cursor.
            let before = self.get::<Value>("users", id)?;
            let changed = match (before.as_ref(), after) {
                (None, None) => false,
                (None, Some(_)) | (Some(_), None) => true,
                (Some(before), Some(after)) => [
                    "id",
                    "username",
                    "display_name",
                    "email",
                    "enabled",
                    "admin",
                ]
                .iter()
                .any(|field| before.get(*field) != after.get(*field)),
            };
            if changed {
                let generation = self
                    .get::<u64>("provisioning_user_generation", "all")?
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or_else(|| Error::internal("SCIM user generation exhausted"))?;
                self.put("provisioning_user_generation", "all", &generation)?;
            }
            // JSON user-list cursors cover every UserView field, including
            // factors and attributes omitted from the provisioning projection.
            if before.as_ref() != after {
                let generation = self
                    .get::<u64>("user_listing_generation", "all")?
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or_else(|| Error::internal("User listing generation exhausted"))?;
                self.put("user_listing_generation", "all", &generation)?;
            }
        }
        if bucket == "groups" {
            let before = self.get::<crate::model::Group>(bucket, id)?;
            let after = after
                .map(|value| serde_json::from_value::<crate::model::Group>(value.clone()))
                .transpose()
                .map_err(Error::internal)?;
            self.update_group_index(id, before.as_ref(), after.as_ref())?;
        }
        if bucket == "access_grants" {
            let before = self.get::<Value>(bucket, id)?;
            self.update_grant_index(id, before.as_ref(), after)?;
        }
        if bucket == "provisioning_links" {
            let before = self.get::<Value>(bucket, id)?;
            self.update_link_index(id, before.as_ref(), after)?;
            // A planning cursor spans transactions. Any ownership change for
            // its target invalidates that cursor, including an insertion that
            // sorts before the cursor or an offboarding link update.
            let targets: std::collections::BTreeSet<_> = [before.as_ref(), after]
                .into_iter()
                .flatten()
                .filter_map(|link| link.get("target").and_then(Value::as_str))
                .collect();
            for target in targets {
                let generation = self
                    .get::<u64>("provisioning_link_generations", target)?
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or_else(|| Error::internal("SCIM link generation exhausted"))?;
                self.put("provisioning_link_generations", target, &generation)?;
            }
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
    fn update_link_index(
        &self,
        id: &str,
        before: Option<&Value>,
        after: Option<&Value>,
    ) -> Result<()> {
        let entry = |value: &Value| {
            value["local_id"]
                .as_str()
                .filter(|_| value["kind"] == "Users")
                .map(|user| {
                    (
                        format!("{USER_LINKS}/{}", crypto::digest(user)),
                        value["target"].clone(),
                    )
                })
        };
        let old = before.and_then(entry);
        let new = after.and_then(entry);
        if old == new {
            return Ok(());
        }
        if let Some((index, _)) = old {
            self.delete(&index, id)?;
        }
        if let Some((index, target)) = new {
            self.put(&index, id, &target)?;
        }
        Ok(())
    }
    fn update_group_index(
        &self,
        id: &str,
        before: Option<&crate::model::Group>,
        after: Option<&crate::model::Group>,
    ) -> Result<()> {
        // The source digest is written independently by import_record after
        // the raw Group value. A bypassing raw import changes that digest but
        // not this binding, so LDAP fails closed without fetching Group.members.
        if let Some(group) = after {
            self.put(
                GROUP_BINDINGS,
                id,
                &GroupBinding {
                    name: group.name.clone(),
                    source_digest: group_source_digest(group)?,
                },
            )?;
        } else if before.is_some() {
            self.delete(GROUP_BINDINGS, id)?;
            self.delete(GROUP_SOURCE_DIGESTS, id)?;
        }
        let fold_bucket = format!(
            "{GROUP_DN_FOLDS}/{}",
            crypto::digest(&id.to_ascii_lowercase())
        );
        if before.is_none() && after.is_some() {
            self.put(&fold_bucket, id, &true)?;
        } else if before.is_some() && after.is_none() {
            self.delete(&fold_bucket, id)?;
        }
        let key = crypto::digest(id);
        if let Some(old) = before {
            for user in &old.members {
                if after.is_none_or(|new| new.name != old.name || !new.members.contains(user)) {
                    self.delete(&format!("index_user_groups/{}", crypto::digest(user)), &key)?;
                }
            }
        }
        if let Some(new) = after {
            for user in &new.members {
                if before.is_none_or(|old| old.name != new.name || !old.members.contains(user)) {
                    self.put(
                        &format!("index_user_groups/{}", crypto::digest(user)),
                        &key,
                        &new.name,
                    )?;
                }
            }
        }
        Ok(())
    }
    /// Validate the Group key/name and its last indexed source content using
    /// only two small records from the caller's snapshot. The source digest
    /// is refreshed on every raw Group import, including imports that skip indexes.
    #[cfg(feature = "platform")]
    pub(crate) fn group_binding(&self, name: &str) -> Result<bool> {
        let binding = self.get::<GroupBinding>(GROUP_BINDINGS, name)?;
        let source_digest = self.get::<String>(GROUP_SOURCE_DIGESTS, name)?;
        match (binding, source_digest) {
            (None, None) => Ok(false),
            (Some(binding), Some(source_digest))
                if binding.name == name && binding.source_digest == source_digest =>
            {
                Ok(true)
            }
            _ => Err(Error::internal("LDAP Group record binding mismatch")),
        }
    }
    /// Read only this user's durable memberships. Each storage range is bounded;
    /// the result can still contain every group when the user belongs to all of them.
    pub fn user_group_names(&self, user_id: &str) -> Result<BTreeSet<String>> {
        let bucket = format!("index_user_groups/{}", crypto::digest(user_id));
        let mut names = BTreeSet::new();
        let mut after = None;
        loop {
            let page = self.scan::<String>(&bucket, after.as_deref(), PAGE)?;
            if page.is_empty() {
                break;
            }
            let full = page.len() == PAGE;
            after = page.last().map(|(key, _)| key.clone());
            names.extend(page.into_iter().map(|(_, name)| name));
            if !full {
                break;
            }
        }
        Ok(names)
    }
    /// One bounded page of durable Group memberships for an LDAP user. The
    /// cursor is the index key (a Group-key digest), not the Group name.
    #[cfg(feature = "platform")]
    pub(crate) fn user_group_index_page(
        &self,
        user_id: &str,
        after: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, String)>> {
        self.scan(
            &format!("index_user_groups/{}", crypto::digest(user_id)),
            after,
            limit.min(PAGE),
        )
    }
    /// Check one durable membership without decoding the Group's member set.
    #[cfg(feature = "platform")]
    pub(crate) fn user_has_group_index(&self, user_id: &str, group_name: &str) -> Result<bool> {
        let bucket = format!("index_user_groups/{}", crypto::digest(user_id));
        match self.get::<String>(&bucket, &crypto::digest(group_name))? {
            Some(name) if name == group_name => Ok(true),
            Some(_) => Err(Error::internal(
                "Group membership index has a mismatched name",
            )),
            None => Ok(false),
        }
    }
    /// Group storage keys whose LDAP DNs have the same ASCII case fold.
    /// A page is enough for the read side to detect visible collisions without
    /// retaining every projected Group DN.
    #[cfg(feature = "platform")]
    pub(crate) fn group_dn_fold_page(
        &self,
        name: &str,
        after: Option<&str>,
    ) -> Result<Vec<String>> {
        let bucket = format!(
            "{GROUP_DN_FOLDS}/{}",
            crypto::digest(&name.to_ascii_lowercase())
        );
        Ok(self
            .scan::<bool>(&bucket, after, PAGE)?
            .into_iter()
            .map(|(key, _)| key)
            .collect())
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
    /// Bounded round-robin selection for connector claims. Advance only past
    /// inspected entries, so a busy target at the head cannot hide later due
    /// jobs. Freeze the due cutoff until wraparound; new retries/appends cannot
    /// keep the cursor chasing the tail forever. Only cursor metadata changes
    /// on a refused claim, never a job's retry time, lease or attempt count.
    pub(crate) fn connector_due<T: DeserializeOwned, R>(
        &self,
        bucket: &str,
        at: u64,
        mut claim: impl FnMut(String, T) -> Result<Option<R>>,
    ) -> Result<Option<R>> {
        const CURSORS: &str = "connector_due_cursors";
        const LIMIT: usize = 16;
        if !matches!(bucket, "provisioning_jobs" | "provisioning_deactivations") {
            return Err(Error::internal("Collection has no connector cursor"));
        }
        let cursor = self.get::<(String, u64)>(CURSORS, bucket)?;
        let cutoff = cursor.as_ref().map_or(at, |(_, cutoff)| (*cutoff).min(at));
        let entries = self.scan::<String>(
            &format!("index_due_{bucket}"),
            cursor.as_ref().map(|(key, _)| key.as_str()),
            LIMIT,
        )?;
        let mut exhausted = entries.len() < LIMIT;
        let mut last = None;
        let mut selected = None;
        for (key, id) in entries {
            let due = key
                .split('/')
                .next()
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| Error::internal("Invalid due index"))?;
            if due > cutoff {
                exhausted = true;
                break;
            }
            last = Some(key);
            if let Some(record) = self.get(bucket, &id)?
                && let Some(result) = claim(id, record)?
            {
                selected = Some(result);
                // Remaining entries in this page have not been inspected.
                exhausted = false;
                break;
            }
        }
        if let Some(last) = last.filter(|_| !exhausted) {
            self.put(CURSORS, bucket, &(last, cutoff))?;
        } else {
            self.delete(CURSORS, bucket)?;
        }
        Ok(selected)
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
        self.rebuild_indexes_checked(&|| Ok(()))
    }
    /// Rebuild derived indexes in pages, checking for cancellation between pages.
    /// The caller's write transaction keeps the source records and rebuilt indexes
    /// atomic even if a check aborts partway through.
    pub fn rebuild_indexes_checked(&self, check: &dyn Fn() -> Result<()>) -> Result<()> {
        let mut indexes = vec![
            "index_counts".to_owned(),
            "index_queues".into(),
            "session_retention".into(),
            "index_user_access_grants".into(),
            "index_user_groups".into(),
            GROUP_DN_FOLDS.into(),
            GROUP_BINDINGS.into(),
            GROUP_SOURCE_DIGESTS.into(),
            USER_LINKS.into(),
        ];
        for bucket in COUNTED {
            indexes.push(format!("index_expiry_{bucket}"));
        }
        for queue in QUEUES {
            indexes.push(format!("index_due_{queue}"));
            indexes.push(format!("index_age_{queue}"));
        }
        for bucket in indexes {
            self.for_each_rebuild_page::<Value>(&bucket, check, |key, _| {
                self.delete(&bucket, &key)
            })?;
        }
        for bucket in COUNTED {
            let mut count = 0u64;
            self.for_each_rebuild_page::<Value>(bucket, check, |id, value| {
                count = count
                    .checked_add(1)
                    .ok_or_else(|| Error::internal("Collection count overflow"))?;
                self.update_indexes(bucket, &id, Some(&value))
            })?;
            self.put("index_counts", bucket, &count)?;
        }
        for bucket in ["access", "refresh"] {
            self.for_each_rebuild_page::<Value>(bucket, check, |id, value| {
                self.update_indexes(bucket, &id, Some(&value))
            })?;
        }
        for bucket in QUEUES {
            self.for_each_rebuild_page::<Value>(bucket, check, |id, value| {
                self.update_queue_indexes(bucket, &id, None, Some(&value))
            })?;
        }
        self.for_each_rebuild_page::<Value>("access_grants", check, |id, value| {
            self.update_grant_index(&id, None, Some(&value))
        })?;
        self.for_each_rebuild_page::<crate::model::Group>("groups", check, |id, group| {
            self.update_group_index(&id, None, Some(&group))?;
            self.put(GROUP_SOURCE_DIGESTS, &id, &group_source_digest(&group)?)
        })?;
        self.for_each_rebuild_page::<Value>("provisioning_links", check, |id, link| {
            self.update_link_index(&id, None, Some(&link))
        })?;
        check()?;
        self.put("meta", "index_version", &INDEX_VERSION)?;
        Ok(())
    }
    fn for_each_rebuild_page<T: DeserializeOwned>(
        &self,
        bucket: &str,
        check: &dyn Fn() -> Result<()>,
        mut visit: impl FnMut(String, T) -> Result<()>,
    ) -> Result<()> {
        let mut after = None;
        loop {
            check()?;
            let page = self.scan::<T>(bucket, after.as_deref(), REBUILD_PAGE)?;
            if page.is_empty() {
                return Ok(());
            }
            let last = page.last().unwrap().0.clone();
            let complete = page.len() < REBUILD_PAGE;
            for (id, value) in page {
                visit(id, value)?;
            }
            if complete {
                return Ok(());
            }
            after = Some(last);
        }
    }
}
