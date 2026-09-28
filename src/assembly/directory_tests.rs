use super::*;
use crate::{config::Config, connector_guard::BACKUP_SAFE_RECORD_BYTES, model::NewUser};

#[test]
fn near_bound_plan_backs_up_and_expired_removed_directory_draft_is_swept() {
    let temp = tempfile::tempdir().unwrap();
    let core = Core::initialize(
        Config { data_dir: temp.path().join("data"), ..Default::default() },
        NewUser {
            username: "admin".into(),
            password: "test-password-for-backup-bound".into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    ).unwrap();
    let admin = core.login("admin".into(), "test-password-for-backup-bound".into(), None)
        .unwrap()["session_token"].as_str().unwrap().to_owned();

    let mut plan = Plan {
        id: crypto::id(), directory: "removed".into(), actor: "admin".into(),
        revision: 0, expires_at: now().saturating_add(300), fingerprint: "test".into(),
        entries: vec![Entry {
            external_id: "stable".into(), dn: "uid=stable,dc=test".into(),
            username: "stable".into(), display_name: String::new(),
            email: None, groups: BTreeSet::new(),
        }],
        changes: Vec::new(), removal_impact: RemovalImpact::default(),
        review: ReviewBinding::default(), applied: false,
    };
    let base = serde_json::to_vec(&plan).unwrap().len();
    plan.entries[0].display_name = "x".repeat(BACKUP_SAFE_RECORD_BYTES - base - 1024);
    let accepted_bytes = serde_json::to_vec(&plan).unwrap().len();
    assert_eq!(accepted_bytes, BACKUP_SAFE_RECORD_BYTES - 1024);
    require_backup_safe_record(&plan, "oversized plan").unwrap();
    core.store.write(|tx| tx.put("directory_plans", &plan.id, &plan)).unwrap();
    let mut archive = Vec::new();
    core.backup_stream(&admin, &crypto::random_token(""), &mut archive,
        crate::operations::stream::StreamOptions::default()).unwrap();
    assert!(!archive.is_empty());
    plan.entries[0].display_name.push_str(&"x".repeat(1025));
    assert_eq!(require_backup_safe_record(&plan, "oversized plan").unwrap_err().code,
        "invalid_request");

    let sweep_at = now();
    let quota = LdapReconciliationQuota::default();
    let mut expired = SnapshotDraft::new("removed".into(), "admin".into(), 0,
        "test".into(), "test".into(), &quota);
    expired.expires_at = sweep_at;
    let mut live = SnapshotDraft::new("live".into(), "admin".into(), 0,
        "test".into(), "test".into(), &quota);
    live.expires_at = sweep_at.saturating_add(60);
    assert!(!core.config.directories.contains_key("removed"));
    let expired_key = digest("removed");
    let live_key = digest("live");
    core.store.write(|tx| {
        tx.put(LDAP_SNAPSHOTS, &expired_key, &expired)?;
        tx.put(LDAP_SNAPSHOTS, &live_key, &live)
    }).unwrap();
    core.store.write(|tx| cleanup(tx, sweep_at)).unwrap();
    assert!(core.store.get::<SnapshotDraft>(LDAP_SNAPSHOTS, &expired_key).unwrap().is_none());
    assert!(core.store.get::<SnapshotDraft>(LDAP_SNAPSHOTS, &live_key).unwrap().is_some());
}