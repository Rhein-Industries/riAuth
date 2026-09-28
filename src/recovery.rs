//! Restored-state policy (R04, RI-STORE-004).
//!
//! An authenticated snapshot proves integrity, not freshness. Whenever older
//! state may have replaced newer state (`riauth restore`, a database-native
//! restore, or a changed PostgreSQL lineage), one transaction:
//!
//! - deletes every session, pending proof, one-time code, consent and
//!   OAuth/SAML/proxy grant, ends RP sessions with a queued back-channel logout,
//!   and revokes temporary access;
//! - advances every account epoch and the configuration revision by [`STRIDE`],
//!   so authority minted on the lost timeline cannot match the restored counters;
//! - keeps replay caches, identities, subjects, keys and persistent credentials;
//! - records a serving gate. `serve` and readiness refuse the store until an
//!   operator attests that persistent credentials were reconciled or rotated.
//!
//! redb and PostgreSQL run the same function inside their ordinary writer.
use crate::{
    crypto::{self, now},
    error::{Error, Result},
    model::Audit,
    store::{Store, Tx, maintenance::PAGE},
};
use serde::{Deserialize, Serialize, de::IgnoredAny};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const POLICY: &str = "riauth.recovery/v1";
/// Far beyond the epoch or revision changes a lost timeline can make, and small
/// enough that a counter stays exact in JSON for millions of recoveries.
pub const STRIDE: u64 = 1 << 32;
const GATE: &str = "recovery";
const LINEAGE: &str = "storage_lineage";
pub const HISTORY: &str = "recovery_history";
/// `invalidated` key counting `users[*].totp_pending` proofs cleared in place.
pub const PENDING_ENROLLMENTS: &str = "users.totp_pending";

/// Deleted on recovery. Each record is live authority, a pending proof, an
/// in-flight protocol exchange or its reverse index; losing one costs a new
/// sign-in, consent prompt or restarted flow.
pub const INVALIDATED: &[&str] = &[
    // Bearer, browser and protocol sessions.
    "sessions",
    "session_tokens",
    "session_retention",
    "browser_sessions",
    "browser_logins",
    "browser_authorizations",
    "authorization_codes",
    "saml_sessions",
    "saml_logout_sessions",
    "saml_source_sessions",
    "proxy_sessions",
    "proxy_pending",
    "mtls_logins",
    "radius_eap_identities",
    "device_verifications",
    // A restored snapshot cannot prove a human grant was not revoked later.
    "human_grants",
    // OAuth grants, families and one-time codes.
    "access",
    "refresh",
    "families",
    "codes",
    "devices",
    "device_users",
    "saml_codes",
    "portal_codes",
    "logout_codes",
    // Pending interactions, ceremonies and proofs.
    "authentication",
    "pushed_requests",
    "saml_requests",
    "saml_logout_flows",
    "logout_confirmations",
    "portal_requests",
    "account_proofs",
    "account_latest",
    "passkey_authentication",
    "passkey_registration",
    "admin_passkey_registration",
    "totp_enrollments",
    "device_challenges",
    "radius_requests",
    "source_logins",
    "source_polls",
    "source_stages",
    "source_stage_requests",
    "windows_tickets",
    // Platform workflow runs, verifier requests, evidence, and session bindings
    // cannot survive a restore of potentially stale authority.
    "workflow_runs",
    "workflow_requests",
    "workflow_evidence",
    "workflow_active_sessions",
    // A restored controller cursor can schedule work from an older timeline;
    // pending or uncertain jobs must not resume after the recovery gate opens.
    // Fresh cursors are derived from current config on the first worker tick.
    "reconciliation_schedules",
    "reconciliation_jobs",
    // Queued mail bodies carry the plaintext proofs deleted above.
    "mail_deliveries",
    // Consent withdrawn after the snapshot must not return; users are asked again.
    "consents",
    "saml_consents",
];

/// Kept: deleting a record that remembers consumption would enable a replay.
/// Entries accepted only on the lost timeline are absent and cannot be rebuilt;
/// each artifact's own lifetime is the remaining bound.
pub const REPLAY_CACHES: &[&str] = &[
    // Used/revoked account-proof tombstones preserve useful replay outcomes.
    "account_proof_outcomes",
    "assertion_replays",
    "dpop_replays",
    "saml_replays",
    "saml_source_replays",
    "signed_requests",
    "ssf_jti",
];

/// Persistent credentials and bindings restored from the snapshot. Their
/// post-snapshot rotation or revocation cannot be derived from the snapshot.
const RECONCILE: &[(&str, &str)] = &[
    ("agents", "agent_credentials"),
    ("clients", "clients"),
    ("windows_devices", "windows_device_credentials"),
    ("passkeys", "passkeys"),
    ("mtls_bindings", "certificate_bindings"),
    ("radius_certificates", "radius_certificates"),
    ("source_secrets", "source_secrets"),
    ("source_links", "source_links"),
    ("key_domains", "signing_keys"),
    ("registrations", "registration_access_tokens"),
    ("ssf_streams", "ssf_stream_credentials"),
];

/// Kept unchanged: identity, configuration, ownership, audit, jobs, receipts,
/// counters and indexes of the reconciled credentials. Restored plans and jobs
/// carry the old revision and fail their equality check; queued revocation is idempotent.
const RETAINED: &[&str] = &[
    "meta",
    "users",
    "usernames",
    "invitation_reservations",
    "groups",
    "sources",
    "saml_subjects",
    "directory_users",
    "directory_bindings",
    "cloud_directory_users",
    "cloud_directory_bindings",
    "scim_users",
    "scim_groups",
    "provisioning_links",
    "provisioning_user_generation",
    "provisioning_link_generations",
    "audit",
    "schema_migrations",
    "logout_deliveries",
    "ssf_deliveries",
    "provisioning_jobs",
    "provisioning_plans",
    "provisioning_snapshots",
    "offboard_jobs",
    // Deactivation intent revalidates the account, link, target and scoped
    // controller authority before any dispatch; delivery is idempotent.
    "provisioning_deactivations",
    "plans",
    "directory_plans",
    "cloud_directory_plans",
    "receipts",
    "attempts",
    "directory_attempts",
    "cloud_directory_runs",
    "http_rates",
    "mail_limits",
    "maintenance_cursors",
    "maintenance_bounds",
    "connector_due_cursors",
    "password_history",
    "credential_versions",
    "human_grant_generations",
    // A restored support exposure must keep blocking later privilege elevation.
    "support_credential_exposure",
    "scim_oauth_cache",
    "agent_tokens",
    "registration_tokens",
    "mtls_fingerprints",
    "mtls_san_emails",
    "mtls_san_uris",
    "mtls_users",
    "radius_certificate_ids",
    HISTORY,
];

/// How recovery treats one storage collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Invalidated,
    ReplayCache,
    /// Restored temporary access grants are revoked; pending requests denied.
    Revoked,
    /// Restored RP sessions are ended and queued for back-channel logout.
    LoggedOut,
    Reconcile,
    Retained,
}

pub fn classify(bucket: &str) -> Option<Class> {
    if INVALIDATED.contains(&bucket) {
        Some(Class::Invalidated)
    } else if REPLAY_CACHES.contains(&bucket) {
        Some(Class::ReplayCache)
    } else if matches!(bucket, "access_grants" | "access_requests") {
        Some(Class::Revoked)
    } else if bucket == "rp_sessions" {
        Some(Class::LoggedOut)
    } else if RECONCILE.iter().any(|(name, _)| *name == bucket) {
        Some(Class::Reconcile)
    } else if RETAINED.contains(&bucket) || bucket.starts_with("index_") {
        Some(Class::Retained)
    } else {
        None
    }
}

/// Why older state may be in use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cause {
    BackupRestore,
    DatabaseRestore,
    StorageLineageChanged,
}

/// The PostgreSQL objects holding riAuth records. A logical restore creates new ones.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    pub system_identifier: Option<String>,
    pub database_oid: u32,
    pub records_oid: u32,
}

impl Lineage {
    /// Roles differ in whether they may read the system identifier, so it is
    /// compared only when both sides have it.
    pub fn same_store(&self, other: &Self) -> bool {
        self.database_oid == other.database_oid
            && self.records_oid == other.records_oid
            && match (&self.system_identifier, &other.system_identifier) {
                (Some(left), Some(right)) => left == right,
                _ => true,
            }
    }
}

fn unchanged(recorded: Option<&Lineage>, observed: Option<&Lineage>) -> bool {
    match (recorded, observed) {
        (_, None) => true,
        (Some(recorded), Some(observed)) => recorded.same_store(observed),
        (None, Some(_)) => false,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageChange {
    pub recorded: Lineage,
    pub observed: Lineage,
}

/// Pending reconciliation. Present in `meta/recovery` until completed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recovery {
    pub id: String,
    pub policy: String,
    pub cause: Cause,
    pub invalidated_at: u64,
    #[serde(default)]
    pub snapshot_created_at: Option<u64>,
    /// Records removed per collection, plus ended RP sessions, revoked temporary
    /// access grants, denied requests and cleared pending MFA enrollments.
    pub invalidated: BTreeMap<String, u64>,
    pub epoch_advanced_users: u64,
    /// Restored persistent credentials the operator must reconcile or rotate.
    pub reconcile: BTreeMap<String, u64>,
    /// Collections present in the store that this policy does not know.
    #[serde(default)]
    pub unclassified: Vec<String>,
    #[serde(default)]
    pub lineage: Option<LineageChange>,
    #[serde(default)]
    pub completed_at: Option<u64>,
}

/// Apply the restored-state policy in the caller's writer. The caller rebuilds
/// derived indexes afterwards when the store is offline.
pub(crate) fn invalidate(
    tx: &Tx<'_>,
    cause: Cause,
    snapshot_created_at: Option<u64>,
    lineage: Option<LineageChange>,
) -> Result<Recovery> {
    let at = now();
    let unclassified = collections(tx)?
        .into_iter()
        .filter(|bucket| classify(bucket).is_none())
        .collect();
    let mut invalidated = BTreeMap::new();
    // Tell RPs that restored sessions ended; rows stay for key retention.
    let ended = crate::identity::logout_queue::queue_all(tx)?;
    if ended > 0 {
        invalidated.insert("rp_sessions".into(), ended);
    }
    for bucket in INVALIDATED {
        let removed = clear(tx, bucket)?;
        if removed > 0 {
            invalidated.insert((*bucket).to_owned(), removed);
        }
    }
    let revoked = revoke_access_grants(tx, at)?;
    if revoked > 0 {
        invalidated.insert("access_grants".into(), revoked);
    }
    let denied = deny_access_requests(tx, at)?;
    if denied > 0 {
        invalidated.insert("access_requests".into(), denied);
    }
    let mut users = 0;
    let mut pending_enrollments = 0;
    let mut reconcile = BTreeMap::new();
    let mut after = None;
    loop {
        let page = tx.scan::<Value>("users", after.as_deref(), PAGE)?;
        let Some((last, _)) = page.last() else { break };
        after = Some(last.clone());
        for (id, mut user) in page {
            // Only the epoch and the pending MFA enrollment proof change. Users
            // have no derived index, and the raw value keeps fields this version
            // does not model.
            let epoch = user["epoch"]
                .as_u64()
                .ok_or_else(|| Error::internal("User record has no epoch"))?;
            let new_epoch = epoch.saturating_add(STRIDE);
            crate::lifecycle::rebase_invitation_reservation(
                tx,
                &id,
                user["username"]
                    .as_str()
                    .ok_or_else(|| Error::internal("User record has no username"))?,
                epoch,
                new_epoch,
            )?;
            user["epoch"] = Value::from(new_epoch);
            // A restored enrollment secret could confirm TOTP for its remaining window.
            if user
                .get("totp_pending")
                .is_some_and(|pending| !pending.is_null())
            {
                user["totp_pending"] = Value::Null;
                pending_enrollments += 1;
            }
            count_user_credentials(&user, &mut reconcile);
            tx.import_record("users", &id, &user)?;
            users += 1;
        }
    }
    if pending_enrollments > 0 {
        invalidated.insert(PENDING_ENROLLMENTS.into(), pending_enrollments);
    }
    if tx.get::<IgnoredAny>("meta", "keys")?.is_some() {
        *reconcile.entry("signing_keys".to_owned()).or_default() += 1;
    }
    for (bucket, class) in RECONCILE {
        let count = count(tx, bucket)?;
        if count > 0 {
            *reconcile.entry((*class).to_owned()).or_default() += count;
        }
    }
    let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
    tx.put("meta", "revision", &revision.saturating_add(STRIDE))?;
    // A snapshot's lineage describes where it was taken, not where it now lives.
    tx.delete("meta", LINEAGE)?;
    let recovery = Recovery {
        id: crypto::id(),
        policy: POLICY.into(),
        cause,
        invalidated_at: at,
        snapshot_created_at,
        invalidated,
        epoch_advanced_users: users,
        reconcile,
        unclassified,
        lineage,
        completed_at: None,
    };
    // An earlier unfinished recovery stays in history; this one supersedes its gate.
    if let Some(previous) = tx.get::<Recovery>("meta", GATE)? {
        tx.put(HISTORY, &history_key(&previous), &previous)?;
    }
    tx.put("meta", GATE, &recovery)?;
    record(tx, "recovery.invalidate", &recovery, json!({}))?;
    Ok(recovery)
}

/// Pending reconciliation, if any.
pub fn pending(tx: &Tx<'_>) -> Result<Option<Recovery>> {
    tx.get("meta", GATE)
}

/// Refuse traffic while restored state is unreconciled or PostgreSQL lineage differs.
pub fn require_serving(store: &Store) -> Result<()> {
    store.read(|tx| {
        if pending(tx)?.is_some() {
            return Err(Error::conflict(
                "Restored state requires reconciliation; run `riauth recovery status`",
            ));
        }
        if !unchanged(
            tx.get::<Lineage>("meta", LINEAGE)?.as_ref(),
            tx.postgres_lineage()?.as_ref(),
        ) {
            return Err(Error::conflict(
                "PostgreSQL storage lineage changed; reopen the store to apply the recovery policy",
            ));
        }
        Ok(())
    })
}

/// Record the lineage of a new PostgreSQL store. No-op on redb.
pub(crate) fn stamp_lineage(tx: &Tx<'_>) -> Result<()> {
    if let Some(observed) = tx.postgres_lineage()? {
        tx.put("meta", LINEAGE, &observed)?;
    }
    Ok(())
}

/// On open: a PostgreSQL store whose records now live in another cluster,
/// database or table is treated as restored older state.
pub(crate) fn verify_lineage(store: &Store) -> Result<Option<Recovery>> {
    if store.backend() != "postgresql" {
        return Ok(None);
    }
    let current = store.read(|tx| {
        Ok(unchanged(
            tx.get::<Lineage>("meta", LINEAGE)?.as_ref(),
            tx.postgres_lineage()?.as_ref(),
        ))
    })?;
    if current {
        return Ok(None);
    }
    store.write(|tx| {
        let Some(observed) = tx.postgres_lineage()? else {
            return Ok(None);
        };
        let recovery = match tx.get::<Lineage>("meta", LINEAGE)? {
            // Another node already recorded this store; keep its identifier.
            Some(recorded) if recorded.same_store(&observed) => return Ok(None),
            // A store created before lineage stamping has nothing to compare.
            None => None,
            Some(recorded) => {
                let change = LineageChange {
                    recorded,
                    observed: observed.clone(),
                };
                let recovery = invalidate(tx, Cause::StorageLineageChanged, None, Some(change))?;
                tx.rebuild_indexes()?;
                Some(recovery)
            }
        };
        tx.put("meta", LINEAGE, &observed)?;
        Ok(recovery)
    })
}

/// Operator-invoked policy after a database-native restore. Every server must
/// be stopped: redb holds a file lock and PostgreSQL clients are counted.
pub fn invalidate_restored(store: &Store) -> Result<Recovery> {
    store.write(|tx| {
        if tx.postgres_other_clients()?.is_some_and(|count| count > 0) {
            return Err(Error::conflict(
                "Stop every riAuth process connected to this database before recovery",
            ));
        }
        let recovery = invalidate(tx, Cause::DatabaseRestore, None, None)?;
        tx.rebuild_indexes()?;
        stamp_lineage(tx)?;
        Ok(recovery)
    })
}

/// Clear the serving gate. The attestation is the operator's reviewed statement
/// that restored persistent credentials were reconciled or rotated.
/// `recovery_id` binds the attestation to the reviewed record, so a recovery
/// applied after the review cannot be cleared by it.
pub fn complete(store: &Store, recovery_id: &str, attested: bool) -> Result<Recovery> {
    if !attested {
        return Err(Error::bad(
            "Reconcile or rotate restored persistent credentials, then confirm with --persistent-credentials-reconciled",
        ));
    }
    store.write(|tx| {
        let mut recovery =
            pending(tx)?.ok_or_else(|| Error::missing("No restored-state recovery is pending"))?;
        if recovery.id != recovery_id {
            return Err(Error::conflict(
                "Another recovery is pending; review `riauth recovery status` again",
            ));
        }
        recovery.completed_at = Some(now());
        tx.put(HISTORY, &history_key(&recovery), &recovery)?;
        tx.delete("meta", GATE)?;
        record(
            tx,
            "recovery.complete",
            &recovery,
            json!({"persistent_credentials_reconciled": true}),
        )?;
        Ok(recovery)
    })
}

pub fn status(store: &Store) -> Result<Value> {
    store.read(|tx| describe(store.backend(), Some(tx)))
}

/// `status` for the configured store without opening it: nothing is created,
/// migrated, stamped or recovered, and a missing store reports not serving.
pub fn inspect(config: &crate::config::Config) -> Result<Value> {
    Store::inspect(config, describe)
}

fn describe(backend: &str, tx: Option<&Tx<'_>>) -> Result<Value> {
    let schema = tx
        .map(|tx| tx.get::<u32>("meta", "schema"))
        .transpose()?
        .flatten();
    let (Some(tx), Some(schema)) = (tx, schema) else {
        return Ok(json!({
            "policy": POLICY,
            "backend": backend,
            "initialized": false,
            "pending": null,
            "serving_allowed": false,
            "history": [],
        }));
    };
    let observed = tx.postgres_lineage()?;
    let recorded = tx.get::<Lineage>("meta", LINEAGE)?;
    let pending = pending(tx)?;
    Ok(json!({
        "policy": POLICY,
        "backend": backend,
        "initialized": true,
        "schema": schema,
        "serving_allowed": pending.is_none()
            && unchanged(recorded.as_ref(), observed.as_ref()),
        "pending": pending,
        "lineage": {"recorded": recorded, "observed": observed},
        // Completed or superseded recoveries, newest first.
        "history": tx.scan_reverse::<Recovery>(HISTORY, None, 8)?
            .into_iter().map(|(_, recovery)| recovery).collect::<Vec<_>>(),
    }))
}

fn history_key(recovery: &Recovery) -> String {
    format!("{:020}-{}", recovery.invalidated_at, recovery.id)
}

/// The cursor keeps PostgreSQL from revisiting rows deleted earlier in this transaction.
fn clear(tx: &Tx<'_>, bucket: &str) -> Result<u64> {
    let mut removed = 0;
    let mut after = None;
    loop {
        let page = tx.scan::<IgnoredAny>(bucket, after.as_deref(), PAGE)?;
        let Some((last, _)) = page.last() else {
            return Ok(removed);
        };
        after = Some(last.clone());
        for (key, _) in page {
            tx.delete(bucket, &key)?;
            removed += 1;
        }
    }
}

fn count(tx: &Tx<'_>, bucket: &str) -> Result<u64> {
    let mut total = 0;
    let mut after = None;
    loop {
        let page = tx.scan::<IgnoredAny>(bucket, after.as_deref(), PAGE)?;
        let Some((last, _)) = page.last() else {
            return Ok(total);
        };
        after = Some(last.clone());
        total += page.len() as u64;
    }
}

fn revoke_access_grants(tx: &Tx<'_>, at: u64) -> Result<u64> {
    let mut revoked = 0;
    let mut after = None;
    loop {
        let page = tx.scan::<crate::pam::AccessGrant>("access_grants", after.as_deref(), PAGE)?;
        let Some((last, _)) = page.last() else {
            return Ok(revoked);
        };
        after = Some(last.clone());
        for (id, mut grant) in page {
            if grant.revoked_at.is_none() && at < grant.expires_at {
                grant.revoked_at = Some(at);
                grant.revoked_by = Some("recovery".into());
                tx.put("access_grants", &id, &grant)?;
                revoked += 1;
            }
        }
    }
}

fn deny_access_requests(tx: &Tx<'_>, at: u64) -> Result<u64> {
    let mut denied = 0;
    let mut after = None;
    loop {
        let page =
            tx.scan::<crate::pam::AccessRequest>("access_requests", after.as_deref(), PAGE)?;
        let Some((last, _)) = page.last() else {
            return Ok(denied);
        };
        after = Some(last.clone());
        for (id, mut request) in page {
            // A request denied after the snapshot must not become approvable again.
            if request.status == "pending" {
                request.status = "denied".into();
                request.decided_at = Some(at);
                request.decided_by = Some("recovery".into());
                tx.put("access_requests", &id, &request)?;
                denied += 1;
            }
        }
    }
}

/// Distinct top-level collections, one key read per collection.
fn collections(tx: &Tx<'_>) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let mut after: Option<String> = None;
    while let Some(key) = tx.snapshot_next_key(after.as_deref())? {
        let bucket = key
            .split_once('/')
            .map_or(key.as_str(), |(bucket, _)| bucket);
        // '0' follows '/', so this skips the rest of the collection.
        after = Some(format!("{bucket}0"));
        names.push(bucket.to_owned());
    }
    Ok(names)
}

fn count_user_credentials(user: &Value, reconcile: &mut BTreeMap<String, u64>) {
    let mut add = |class: &str, present: bool| {
        if present {
            *reconcile.entry(class.to_owned()).or_default() += 1;
        }
    };
    add(
        "passwords",
        user["password_hash"]
            .as_str()
            .is_some_and(|hash| !hash.is_empty()),
    );
    add("totp", !user["totp_secret"].is_null());
    add(
        "recovery_codes",
        user["recovery_codes"]
            .as_array()
            .is_some_and(|codes| !codes.is_empty()),
    );
    add("enabled_accounts", user["enabled"] == true);
}

/// Summary only: per-user epoch changes would otherwise copy every account view.
fn record(tx: &Tx<'_>, action: &str, recovery: &Recovery, extra: Value) -> Result<()> {
    let mut details = json!({
        "recovery_id": recovery.id,
        "cause": recovery.cause,
        "policy": recovery.policy,
        "invalidated": recovery.invalidated,
        "epoch_advanced_users": recovery.epoch_advanced_users,
        "reconcile": recovery.reconcile,
        "unclassified": recovery.unclassified,
    });
    if let (Value::Object(details), Value::Object(extra)) = (&mut details, extra) {
        details.extend(extra);
    }
    let event = Audit {
        id: crypto::id(),
        at: now(),
        actor: "local-recovery".into(),
        action: action.into(),
        target: format!("recovery/{}", recovery.id),
        run_id: None,
        details,
    };
    tx.put("audit", &format!("{:020}-{}", event.at, event.id), &event)
}

#[cfg(test)]
mod tests {
    use super::{Lineage, unchanged};

    fn lineage(system_identifier: Option<&str>, database_oid: u32) -> Lineage {
        Lineage {
            system_identifier: system_identifier.map(str::to_owned),
            database_oid,
            records_oid: 16390,
        }
    }

    #[test]
    fn lineage_ignores_an_unreadable_identifier_but_not_a_different_one() {
        let recorded = lineage(Some("7"), 16384);
        assert!(unchanged(Some(&recorded), Some(&lineage(None, 16384))));
        assert!(unchanged(Some(&lineage(None, 16384)), Some(&recorded)));
        assert!(!unchanged(
            Some(&recorded),
            Some(&lineage(Some("8"), 16384))
        ));
        assert!(!unchanged(
            Some(&recorded),
            Some(&lineage(Some("7"), 16385))
        ));
        // redb has no lineage; an unstamped PostgreSQL store is not yet verified.
        assert!(unchanged(None, None));
        assert!(!unchanged(None, Some(&recorded)));
    }
}
