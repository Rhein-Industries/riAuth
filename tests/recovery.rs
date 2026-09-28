//! R04 restored-state policy over `riauth restore` (RI-STORE-004, Q05-R08 shape).
//! Backend parity for database-native restores is in the shared contracts.
mod common;

use axum::http::StatusCode;
use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{NewAgent, Permission},
    config::Config,
    core::Core,
    crypto,
    model::User,
    oidc::TokenRequest,
    pam::AccessGrant,
    recovery::{self, Cause, Recovery, STRIDE},
};
use serde_json::{Value, json};
use std::path::Path;

fn user(core: &Core, username: &str) -> User {
    let id: String = core.store.get("usernames", username).unwrap().unwrap();
    core.store.get("users", &id).unwrap().unwrap()
}

fn refresh(tokens: &Value) -> TokenRequest {
    TokenRequest {
        grant_type: "refresh_token".into(),
        client_id: Some("app".into()),
        refresh_token: Some(text(tokens, "refresh_token")),
        ..Default::default()
    }
}

fn backup(f: &Fixture, directory: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let key = crypto::random_token("");
    let envelope = f.core.backup(&f.admin, &key).unwrap();
    let file = directory.join("backup.json");
    let key_file = directory.join("backup.key");
    std::fs::write(&file, serde_json::to_vec(&envelope).unwrap()).unwrap();
    riauth::config::write_private(&key_file, key.as_bytes(), false).unwrap();
    (file, key_file)
}

fn restore(directory: &Path, archive: &(std::path::PathBuf, std::path::PathBuf)) -> (Value, Core) {
    let output = directory.join("restored");
    let result = riauth::operations::restore(&archive.0, &archive.1, &output, None).unwrap();
    let core = Core::open(Config::load(&output.join("riauth.toml")).unwrap()).unwrap();
    (result, core)
}

fn temporary_grant(f: &Fixture, user_id: &str) -> AccessGrant {
    let at = crypto::now();
    let grant = AccessGrant {
        id: "restore-grant".into(),
        user_id: user_id.into(),
        group: "operators".into(),
        not_before: at,
        expires_at: at + 3600,
        request_id: "restore-request".into(),
        revoked_at: None,
        revoked_by: None,
    };
    f.core
        .store
        .write(|tx| tx.put("access_grants", &grant.id, &grant))
        .unwrap();
    grant
}

#[test]
fn restore_invalidates_restored_sessions_proofs_and_grants_but_preserves_identity() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let tokens = f.tokens("app", &alice, None);
    let pending_code = f.exchange_request("app", &alice, None);
    let before = user(&f.core, "alice");
    let grant = temporary_grant(&f, &before.id);
    let revision: u64 = f.core.store.get("meta", "revision").unwrap().unwrap();
    f.core
        .store
        .write(|tx| {
            tx.put(
                "dpop_replays",
                "restore-fixture",
                &json!(crypto::now() + 600),
            )
        })
        .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let archive = backup(&f, directory.path());
    let (result, restored) = restore(directory.path(), &archive);

    assert_eq!(result["verified"], true);
    assert_eq!(result["serving_allowed"], false);
    let recovery: Recovery = serde_json::from_value(result["recovery"].clone()).unwrap();
    assert_eq!(recovery.cause, Cause::BackupRestore);
    assert!(recovery.snapshot_created_at.is_some());
    for class in ["passwords", "enabled_accounts", "clients"] {
        assert!(
            recovery.reconcile[class] > 0,
            "{class} not listed for reconciliation"
        );
    }
    assert!(recovery.invalidated["sessions"] >= 2);
    assert_eq!(recovery.invalidated["access_grants"], 1);

    // Identity continuity.
    let after = user(&restored, "alice");
    assert_eq!(after.id, before.id);
    assert_eq!(after.subjects, before.subjects);
    assert_eq!(after.pairwise_seed, before.pairwise_seed);
    assert_eq!(after.password_hash, before.password_hash);
    assert_eq!(after.epoch, before.epoch + STRIDE);
    assert_eq!(restored.jwks().unwrap(), f.core.jwks().unwrap());
    assert!(
        restored
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap()
            >= revision + STRIDE
    );

    // No restored session, grant or pending proof authenticates.
    assert!(restored.me(&alice).is_err());
    assert!(restored.me(&f.admin).is_err());
    assert!(restored.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(restored.token(refresh(&tokens)).is_err());
    assert!(restored.token(pending_code).is_err());
    let revoked: AccessGrant = restored
        .store
        .get("access_grants", &grant.id)
        .unwrap()
        .unwrap();
    assert!(revoked.revoked_at.is_some());
    assert!(
        restored
            .store
            .read(|tx| tx.user_access_grants::<AccessGrant>(&before.id))
            .unwrap()
            .is_empty()
    );
    let snapshot = restored.store.read(|tx| tx.snapshot()).unwrap();
    for bucket in recovery::INVALIDATED {
        let prefix = format!("{bucket}/");
        assert!(
            !snapshot.keys().any(|key| key.starts_with(&prefix)),
            "{bucket} survived restore"
        );
    }
    // Replay caches are never cleared.
    assert!(snapshot.contains_key("dpop_replays/restore-fixture"));

    // The gate blocks readiness until the operator attests reconciliation.
    assert!(restored.store.ready().is_err());
    let session = text(
        &restored
            .login("alice".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    assert_eq!(restored.me(&session).unwrap()["user"]["id"], before.id);
    assert!(recovery::complete(&restored.store, &recovery.id, false).is_err());
    // The attestation names the reviewed recovery, not whichever one is pending.
    assert!(recovery::complete(&restored.store, "another-recovery", true).is_err());
    assert!(restored.store.ready().is_err());
    let completed = recovery::complete(&restored.store, &recovery.id, true).unwrap();
    assert_eq!(completed.id, recovery.id);
    assert!(completed.completed_at.is_some());
    restored.store.ready().unwrap();
    let status = recovery::status(&restored.store).unwrap();
    assert_eq!(status["serving_allowed"], true);
    assert_eq!(status["history"][0]["id"], json!(recovery.id));
    let admin = text(
        &restored
            .login("admin".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let actions: Vec<Value> = restored
        .audit_events(&admin, 100)
        .unwrap()
        .as_array()
        .unwrap()
        .clone();
    for action in ["recovery.invalidate", "recovery.complete"] {
        assert_eq!(
            actions
                .iter()
                .filter(|event| event["action"] == action)
                .count(),
            1,
            "{action}"
        );
    }
    assert!(
        !serde_json::to_string(&actions)
            .unwrap()
            .contains("password_hash")
    );
}

/// Characterize authority revoked or consumed after the snapshot, then restored.
#[test]
fn older_restore_cannot_resurrect_post_snapshot_revocations_or_consumption() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    let tokens = f.tokens("app", &alice, None);
    let pending_code = f.exchange_request("app", &bob, None);
    let directory = tempfile::tempdir().unwrap();
    let archive = backup(&f, directory.path());

    // After the snapshot: rotate the refresh token, redeem the code, log out and
    // disable an account, and change a password.
    let rotated = f.core.token(refresh(&tokens)).unwrap();
    let redeemed = f.core.token(pending_code.clone()).unwrap();
    f.core.logout(&bob).unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            riauth::model::UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());

    let (result, restored) = restore(directory.path(), &archive);
    // One-time and session authority from before and after the snapshot is dead.
    assert!(restored.token(refresh(&tokens)).is_err());
    assert!(restored.token(refresh(&rotated)).is_err());
    assert!(restored.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(restored.userinfo(&text(&rotated, "access_token")).is_err());
    assert!(restored.userinfo(&text(&redeemed, "access_token")).is_err());
    assert!(restored.token(pending_code).is_err());
    assert!(restored.me(&alice).is_err());
    assert!(restored.me(&bob).is_err());

    // The persistent account state is the snapshot's: alice is enabled again and
    // her password verifies. The snapshot cannot prove otherwise, so the report
    // lists these classes and the gate keeps the store from serving.
    assert!(user(&restored, "alice").enabled);
    let recovery: Recovery = serde_json::from_value(result["recovery"].clone()).unwrap();
    assert_eq!(recovery.reconcile["enabled_accounts"], 3);
    assert_eq!(recovery.reconcile["passwords"], 3);
    assert!(restored.store.ready().is_err());
    let error = riauth::recovery::require_serving(&restored.store).unwrap_err();
    assert!(error.to_string().contains("reconciliation"));
}

#[tokio::test]
async fn serve_refuses_unreconciled_restored_state_before_listening() {
    let f = Fixture::new();
    let directory = tempfile::tempdir().unwrap();
    let archive = backup(&f, directory.path());
    let (_, restored) = restore(directory.path(), &archive);
    let mut config = restored.config.clone();
    // A listener would fail differently; the gate must reject first.
    config.listen = "127.0.0.1:1".parse().unwrap();
    drop(restored);
    let restored = Core::open(config).unwrap();
    let error = riauth::api::serve(restored).await.unwrap_err();
    assert!(error.to_string().contains("reconciliation"), "{error}");
}

#[test]
fn in_place_recovery_matches_restore_policy_and_keeps_history() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let tokens = f.tokens("app", &alice, None);
    let epoch = user(&f.core, "alice").epoch;
    let first = recovery::invalidate_restored(&f.core.store).unwrap();
    assert_eq!(first.cause, Cause::DatabaseRestore);
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.store.ready().is_err());
    // Repeating before completion supersedes the gate and keeps the first record.
    let second = recovery::invalidate_restored(&f.core.store).unwrap();
    assert_ne!(first.id, second.id);
    assert_eq!(user(&f.core, "alice").epoch, epoch + 2 * STRIDE);
    let history: Vec<(String, Recovery)> = f.core.store.list(recovery::HISTORY).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].1.id, first.id);
    assert!(history[0].1.completed_at.is_none());
    // The superseded record cannot complete the newer gate.
    assert!(recovery::complete(&f.core.store, &first.id, true).is_err());
    recovery::complete(&f.core.store, &second.id, true).unwrap();
    f.core.store.ready().unwrap();
    assert_eq!(
        f.core
            .store
            .list::<Recovery>(recovery::HISTORY)
            .unwrap()
            .len(),
        2
    );
}

/// Every collection named in the source has an explicit restore classification.
#[test]
fn every_storage_collection_has_a_restore_classification() {
    let mut unclassified = std::collections::BTreeSet::new();
    let mut seen = 0;
    let mut stack = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            stack.extend(
                std::fs::read_dir(&path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        // Unit-test fixture collections are not part of a restored deployment.
        let source = source
            .split_once("\n#[cfg(test)]")
            .map_or(source.as_str(), |(production, _)| production);
        for receiver in ["tx.", "store."] {
            for method in [
                "get",
                "put",
                "delete",
                "list",
                "scan",
                "scan_reverse",
                "maintenance_page",
                "due",
                "import_record",
            ] {
                let call = format!("{receiver}{method}");
                for (index, _) in source.match_indices(&call) {
                    let rest = &source[index + call.len()..];
                    let rest = match rest.strip_prefix("::<") {
                        Some(generic) => match generic.find(">(") {
                            Some(end) => &generic[end + 1..],
                            None => continue,
                        },
                        None => rest,
                    };
                    let Some(rest) = rest.strip_prefix("(") else {
                        continue;
                    };
                    let rest = rest.trim_start();
                    let Some(rest) = rest.strip_prefix('"') else {
                        continue;
                    };
                    let Some(end) = rest.find('"') else { continue };
                    let bucket = &rest[..end];
                    seen += 1;
                    if recovery::classify(bucket).is_none() {
                        unclassified.insert(bucket.to_owned());
                    }
                }
            }
        }
    }
    assert!(seen > 300, "scanner found only {seen} storage calls");
    assert!(
        unclassified.is_empty(),
        "unclassified collections: {unclassified:?}"
    );
    for bucket in recovery::INVALIDATED {
        assert_eq!(
            recovery::classify(bucket),
            Some(recovery::Class::Invalidated)
        );
    }
    assert_eq!(
        recovery::classify("user_listing_generation"),
        Some(recovery::Class::Retained)
    );
}

#[test]
fn repeated_snapshot_restore_cannot_resurrect_user_list_cursor() {
    let f = Fixture::new();
    f.user("alice");
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "restored-user-reader".into(),
                ttl: 600,
                parent: None,
                permissions: vec![Permission {
                    action: "user.read".into(),
                    resource: "*".into(),
                }],
            },
        )
        .unwrap();
    let token = text(&agent["credential"], "token");
    let generation: u64 = f
        .core
        .store
        .get("user_listing_generation", "all")
        .unwrap()
        .unwrap();
    let archive_dir = tempfile::tempdir().unwrap();
    let archive = backup(&f, archive_dir.path());

    let first_dir = tempfile::tempdir().unwrap();
    let (first_result, first) = restore(first_dir.path(), &archive);
    let first_recovery: Recovery =
        serde_json::from_value(first_result["recovery"].clone()).unwrap();
    recovery::complete(&first.store, &first_recovery.id, true).unwrap();
    assert_eq!(
        first
            .store
            .get::<u64>("user_listing_generation", "all")
            .unwrap(),
        Some(generation)
    );
    let first_revision: u64 = first.store.get("meta", "revision").unwrap().unwrap();
    let first_epoch: String = first
        .store
        .get("meta", "user_listing_cursor_epoch")
        .unwrap()
        .unwrap();
    let first_page = first.list_users_page(&token, 1, None).unwrap();
    let old_cursor = text(&first_page, "next_cursor");

    // The same snapshot recreates the same credential, revision and generation.
    // Its recovery must still give a cursor from the earlier timeline HTTP 409.
    let second_dir = tempfile::tempdir().unwrap();
    let (second_result, second) = restore(second_dir.path(), &archive);
    let second_recovery: Recovery =
        serde_json::from_value(second_result["recovery"].clone()).unwrap();
    recovery::complete(&second.store, &second_recovery.id, true).unwrap();
    assert_eq!(
        second.store.get::<u64>("meta", "revision").unwrap(),
        Some(first_revision)
    );
    assert_eq!(
        second
            .store
            .get::<u64>("user_listing_generation", "all")
            .unwrap(),
        Some(generation)
    );
    let second_epoch: String = second
        .store
        .get("meta", "user_listing_cursor_epoch")
        .unwrap()
        .unwrap();
    assert_ne!(first_epoch, second_epoch);
    let error = second
        .list_users_page(&token, 1, Some(old_cursor))
        .unwrap_err();
    assert_eq!(error.status, StatusCode::CONFLICT);
    assert!(error.message.contains("restart pagination"));
    assert!(second.list_users_page(&token, 1, None).is_ok());
}

/// Paged RP-session logout and the in-place MFA enrollment proof. More live
/// sessions than one page, some already ended; rows stay for key retention.
#[test]
fn recovery_ends_every_rp_session_page_and_clears_pending_mfa_enrollment() {
    use riauth::logout::RpSession;
    const LIVE: u64 = 300;
    let f = Fixture::new();
    f.client("app", false);
    f.user("alice");
    let alice = user(&f.core, "alice").id;
    let at = crypto::now();
    f.core
        .store
        .write(|tx| {
            let mut client: riauth::model::Client = tx.get("clients", "app")?.unwrap();
            client.settings.backchannel_logout_uri = Some("https://rp.example/logout".into());
            tx.put("clients", "app", &client)?;
            for index in 0..LIVE + 5 {
                let sid = format!("rp-{index:04}");
                let session = RpSession {
                    sid: sid.clone(),
                    session_id: format!("session-{index}"),
                    user_id: alice.clone(),
                    subject: "subject".into(),
                    client_id: "app".into(),
                    created_at: at,
                    expires_at: at + 3600 + index,
                    ended: index >= LIVE,
                };
                tx.put("rp_sessions", &sid, &session)?;
            }
            let mut enrolling: User = tx.get("users", &alice)?.unwrap();
            enrolling.totp_pending = Some(("restored-enrollment".into(), at + 600));
            tx.put("users", &alice, &enrolling)
        })
        .unwrap();
    let deliveries = f
        .core
        .store
        .list::<Value>("logout_deliveries")
        .unwrap()
        .len() as u64;

    let applied = recovery::invalidate_restored(&f.core.store).unwrap();
    assert_eq!(applied.invalidated["rp_sessions"], LIVE);
    assert_eq!(applied.invalidated[recovery::PENDING_ENROLLMENTS], 1);
    assert!(user(&f.core, "alice").totp_pending.is_none());
    let sessions = f.core.store.list::<RpSession>("rp_sessions").unwrap();
    assert_eq!(sessions.len() as u64, LIVE + 5);
    assert!(sessions.iter().all(|(_, rp)| rp.ended));
    // Signing-key retention reads the latest expiry; recovery must not shorten it.
    assert_eq!(
        sessions.iter().map(|(_, rp)| rp.expires_at).max(),
        Some(at + 3600 + LIVE + 4)
    );
    let queued = f.core.store.list::<Value>("logout_deliveries").unwrap();
    assert_eq!(queued.len() as u64, deliveries + LIVE);
    // Already ended sessions are not queued again.
    assert!(
        !queued
            .iter()
            .any(|(_, d)| d["sid"].as_str() >= Some("rp-0300"))
    );
    let status = recovery::status(&f.core.store).unwrap();
    assert_eq!(
        status["pending"]["invalidated"][recovery::PENDING_ENROLLMENTS],
        1
    );
    // A repeated recovery finds nothing left to end or clear.
    let again = recovery::invalidate_restored(&f.core.store).unwrap();
    assert!(!again.invalidated.contains_key("rp_sessions"));
    assert!(
        !again
            .invalidated
            .contains_key(recovery::PENDING_ENROLLMENTS)
    );
}
