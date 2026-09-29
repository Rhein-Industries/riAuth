//! Contract bodies have no backend-specific branches or replacement enforcement.
//! IDs refer to the reviewed Q01 catalog supplied in target/riwork/prerequisites.
#[path = "cloud_mock.rs"]
mod cloud_mock;

use crate::common::{
    Fixture, PASSWORD,
    backend::Backend,
    security::{Dependents, events, subscribe},
    strings, text,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use riauth::{
    agent::{Agent, NewAgent, Permission},
    cloud_directory::{Attributes, WorkspaceDirectory},
    config::write_private,
    core::Core,
    crypto::{self, digest, now},
    error::Error,
    lifecycle::{Invitation, MailConfig, MailSecurity, Purpose},
    model::{Attempts, Client, ClientPatch, Group, NewUser, Session, User, UserPatch},
    offboarding::{self, ExecuteAt, Job, ScheduleRequest, Status},
    oidc::{Authorization, TokenRequest},
    signin,
    state::{ApplyRequest, Manifest},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use tower::ServiceExt;

#[cfg(feature = "test-support")]
use riauth::offboarding::BeforeCommit;

fn user(f: &Fixture, username: &str) -> User {
    let id: String = f.core.store.get("usernames", username).unwrap().unwrap();
    f.core.store.get("users", &id).unwrap().unwrap()
}

fn session(f: &Fixture, token: &str) -> Session {
    let id = text(&f.core.me(token).unwrap(), "session_id");
    f.core.store.get("sessions", &id).unwrap().unwrap()
}

fn agent(f: &Fixture, id: &str, permissions: &[(&str, &str)]) -> String {
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.into(),
                ttl: 3600,
                parent: None,
                permissions: permissions
                    .iter()
                    .map(|(action, resource)| Permission {
                        action: (*action).into(),
                        resource: (*resource).into(),
                    })
                    .collect(),
            },
        )
        .unwrap();
    text(&created["credential"], "token")
}

fn refresh(client: &str, tokens: &Value) -> TokenRequest {
    TokenRequest {
        grant_type: "refresh_token".into(),
        client_id: Some(client.into()),
        refresh_token: Some(text(tokens, "refresh_token")),
        ..Default::default()
    }
}

fn audit(f: &Fixture) -> Vec<Value> {
    f.core
        .audit_events(&f.admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

fn audit_count(f: &Fixture, action: &str) -> usize {
    audit(f)
        .iter()
        .filter(|event| event["action"] == action)
        .count()
}

/// R02: both the v2 envelope and R01 stream import directly into either
/// selected backend with the same identity continuity and R04 serving gate.
pub fn direct_restore_selected_backend(backend: Backend) {
    use riauth::{
        operations::{self, RestoreTarget, stream::StreamOptions},
        recovery::{self, STRIDE},
    };

    let source = Fixture::new();
    source.client("app", false);
    let old_alice = source.user("alice");
    let old_access = text(&source.tokens("app", &old_alice, None), "access_token");
    let before = source.snapshot().unwrap();
    let source_keys = source.core.jwks().unwrap();
    let key = crypto::random_token("");
    let dir = tempfile::tempdir().unwrap();
    let key_file = dir.path().join("backup.key");
    write_private(&key_file, key.as_bytes(), false).unwrap();

    for stream in [false, true] {
        let target = backend.uninitialized();
        assert!(target.untouched());
        let selected = target
            .config
            .postgres
            .clone()
            .map(RestoreTarget::Postgres)
            .unwrap_or(RestoreTarget::Redb);
        let archive = dir
            .path()
            .join(if stream { "backup.v3" } else { "backup.v2" });
        if stream {
            let mut file = std::fs::File::create(&archive).unwrap();
            source
                .core
                .backup_stream(&source.admin, &key, &mut file, StreamOptions::default())
                .unwrap();
        } else {
            let backup = source.core.backup(&source.admin, &key).unwrap();
            std::fs::write(&archive, serde_json::to_vec(&backup).unwrap()).unwrap();
        }
        let output = target.config.data_dir.clone();
        let result = operations::restore_into(
            &archive,
            &key_file,
            &output,
            target.config.database_key_file.clone(),
            selected,
        )
        .unwrap();
        assert_eq!(result["restored"], true);
        assert_eq!(result["verified"], true);
        assert_eq!(result["serving_allowed"], false);
        assert_eq!(
            result["storage"],
            if target.config.postgres.is_some() {
                "postgresql"
            } else {
                "redb"
            }
        );
        let restored =
            Core::open(riauth::config::Config::load(&output.join("riauth.toml")).unwrap()).unwrap();
        assert_eq!(
            restored.store.backend(),
            result["storage"].as_str().unwrap()
        );
        assert_eq!(restored.jwks().unwrap(), source_keys);
        let after = restored.store.read(|tx| tx.snapshot()).unwrap();
        for name in [
            "meta/issuer",
            "meta/keys",
            "clients/app",
            "usernames/admin",
            "usernames/alice",
        ] {
            assert_eq!(after.get(name), before.get(name), "{name} changed");
        }
        let alice_id = before["usernames/alice"].as_str().unwrap();
        let alice_key = format!("users/{alice_id}");
        for field in [
            "id",
            "username",
            "password_hash",
            "subjects",
            "pairwise_seed",
            "enabled",
        ] {
            assert_eq!(
                after[&alice_key][field], before[&alice_key][field],
                "{field} changed"
            );
        }
        assert_eq!(
            after[&alice_key]["epoch"].as_u64().unwrap(),
            before[&alice_key]["epoch"].as_u64().unwrap() + STRIDE
        );
        assert!(restored.store.ready().is_err());
        assert!(restored.me(&old_alice).is_err());
        assert!(restored.me(&source.admin).is_err());
        assert!(restored.userinfo(&old_access).is_err());
        let admin = text(
            &restored
                .login("admin".into(), PASSWORD.into(), None)
                .unwrap(),
            "session_token",
        );
        assert!(restored.get_resource(&admin, "client", "app").is_ok());
        assert!(
            restored
                .login("alice".into(), "wrong-password".into(), None)
                .is_err()
        );
        let alice = text(
            &restored
                .login("alice".into(), PASSWORD.into(), None)
                .unwrap(),
            "session_token",
        );
        let verifier = crypto::random_token("");
        assert!(
            restored
                .authorize(&old_alice, source.request("app", &verifier))
                .is_err()
        );
        let callback = restored
            .authorize(&alice, source.request("app", &verifier))
            .unwrap();
        let code = url::Url::parse(&callback)
            .unwrap()
            .query_pairs()
            .find(|(name, _)| name == "code")
            .unwrap()
            .1
            .into_owned();
        assert!(
            restored
                .token(TokenRequest {
                    grant_type: "authorization_code".into(),
                    client_id: Some("app".into()),
                    code: Some(code),
                    redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
                    code_verifier: Some(verifier),
                    ..Default::default()
                })
                .is_ok()
        );
        let pending = restored.store.read(recovery::pending).unwrap().unwrap();
        assert_eq!(pending.id, result["recovery"]["id"]);
        assert!(recovery::complete(&restored.store, &pending.id, false).is_err());
        assert!(restored.store.ready().is_err());
        assert_eq!(source.snapshot().unwrap(), before);
    }
}

/// Authentication failures leave the selected target untouched; an occupied
/// target cannot be reused even when the archive and key are valid.
pub fn direct_restore_failure_preserves_original(backend: Backend) {
    use riauth::operations::{self, RestoreTarget, stream::StreamOptions};

    let source = Fixture::new();
    let before = source.snapshot().unwrap();
    let key = crypto::random_token("");
    let dir = tempfile::tempdir().unwrap();
    let archive = dir.path().join("backup.v3");
    let mut file = std::fs::File::create(&archive).unwrap();
    source
        .core
        .backup_stream(&source.admin, &key, &mut file, StreamOptions::default())
        .unwrap();
    drop(file);
    let key_file = dir.path().join("backup.key");
    write_private(&key_file, key.as_bytes(), false).unwrap();
    let wrong_key = dir.path().join("wrong.key");
    write_private(&wrong_key, crypto::random_token("").as_bytes(), false).unwrap();

    let empty = backend.uninitialized();
    let selected = empty
        .config
        .postgres
        .clone()
        .map(RestoreTarget::Postgres)
        .unwrap_or(RestoreTarget::Redb);
    let output = empty.config.data_dir.clone();
    assert!(
        operations::restore_into(
            &archive,
            &wrong_key,
            &output,
            empty.config.database_key_file.clone(),
            selected.clone()
        )
        .is_err()
    );
    assert!(empty.untouched());
    assert!(!output.exists());
    let mut corrupt = std::fs::read(&archive).unwrap();
    *corrupt.last_mut().unwrap() ^= 1;
    let corrupt_archive = dir.path().join("corrupt.v3");
    std::fs::write(&corrupt_archive, corrupt).unwrap();
    assert!(
        operations::restore_into(
            &corrupt_archive,
            &key_file,
            &output,
            empty.config.database_key_file.clone(),
            selected
        )
        .is_err()
    );
    assert!(empty.untouched());
    assert!(!output.exists());

    let occupied = backend.fixture();
    let occupied_before = occupied.snapshot().unwrap();
    let selected = occupied
        .core
        .config
        .postgres
        .clone()
        .map(RestoreTarget::Postgres)
        .unwrap_or(RestoreTarget::Redb);
    let output = if occupied.core.config.postgres.is_some() {
        dir.path().join("occupied-output")
    } else {
        occupied.core.config.data_dir.clone()
    };
    let error = operations::restore_into(
        &archive,
        &key_file,
        &output,
        occupied.core.config.database_key_file.clone(),
        selected,
    )
    .unwrap_err();
    assert_eq!(error.status, StatusCode::CONFLICT, "{}", error.message);
    assert!(!output.join("riauth.toml").exists());
    assert_eq!(occupied.snapshot().unwrap(), occupied_before);
    assert_eq!(source.snapshot().unwrap(), before);
}

/// A PostgreSQL archive carries a source lineage marker. A direct redb
/// restore drops that marker while keeping the identity and recovery policy.
pub fn direct_restore_postgres_archive_into_redb() {
    use riauth::{
        operations::{self, RestoreTarget, stream::StreamOptions},
        recovery::STRIDE,
    };

    let source = Backend::Postgres.fixture();
    let old_admin = source.admin.clone();
    let before = source.snapshot().unwrap();
    assert!(before.contains_key("meta/storage_lineage"));
    let key = crypto::random_token("");
    let dir = tempfile::tempdir().unwrap();
    let archive = dir.path().join("postgres.v3");
    let key_file = dir.path().join("backup.key");
    write_private(&key_file, key.as_bytes(), false).unwrap();
    source
        .core
        .backup_stream(
            &source.admin,
            &key,
            &mut std::fs::File::create(&archive).unwrap(),
            StreamOptions::default(),
        )
        .unwrap();
    let output = dir.path().join("redb-restore");
    let result =
        operations::restore_into(&archive, &key_file, &output, None, RestoreTarget::Redb).unwrap();
    assert_eq!(result["storage"], "redb");
    let restored =
        Core::open(riauth::config::Config::load(&output.join("riauth.toml")).unwrap()).unwrap();
    let after = restored.store.read(|tx| tx.snapshot()).unwrap();
    assert!(!after.contains_key("meta/storage_lineage"));
    assert_eq!(after.get("meta/keys"), before.get("meta/keys"));
    let admin_id = before["usernames/admin"].as_str().unwrap();
    let admin_key = format!("users/{admin_id}");
    assert_eq!(
        after[&admin_key]["epoch"].as_u64().unwrap(),
        before[&admin_key]["epoch"].as_u64().unwrap() + STRIDE
    );
    assert!(restored.me(&old_admin).is_err());
    assert!(restored.store.ready().is_err());
    assert_eq!(source.snapshot().unwrap(), before);
}

/// The name/content binding and source digest commit together on both backends.
pub fn group_binding_metadata(backend: Backend) {
    let f = backend.fixture();
    let name = "large-binding-group";
    let stale = format!("stale-{}", "x".repeat(200_000));
    let member_bucket = format!("index_group_members/{}", digest(name));
    let overflow_bucket = format!("index_group_member_overflow/{}", digest(name));
    let mut group = Group {
        name: name.into(),
        members: BTreeSet::from([stale.clone()]),
    };
    f.core.store.write(|tx| tx.put("groups", name, &group)).unwrap();
    let binding: Value = f.core.store.get("index_group_bindings", name).unwrap().unwrap();
    let source: String = f.core.store.get("index_group_source_digests", name).unwrap().unwrap();
    assert_eq!(binding["name"], name);
    assert_eq!(binding["source_digest"], source);
    assert_eq!(source.len(), 43);
    assert_eq!(
        f.core.store.get::<Option<String>>(&member_bucket, &digest(&stale)).unwrap(),
        Some(None)
    );
    assert_eq!(
        f.core.store.get::<String>(&overflow_bucket, &digest(&stale)).unwrap(),
        Some(stale.clone())
    );

    group.members.insert("visible-user".into());
    group.members.extend((0..129).map(|n| format!("short-{n:03}")));
    f.core.store.write(|tx| tx.put("groups", name, &group)).unwrap();
    let changed: String = f.core.store.get("index_group_source_digests", name).unwrap().unwrap();
    let binding: Value = f.core.store.get("index_group_bindings", name).unwrap().unwrap();
    assert_ne!(source, changed);
    assert_eq!(binding["source_digest"], changed);
    let first = f.core.store.read(|tx| tx.scan::<Option<String>>(&member_bucket, None, 128)).unwrap();
    let second = f.core.store.read(|tx| tx.scan::<Option<String>>(&member_bucket, Some(&first.last().unwrap().0), 128)).unwrap();
    assert_eq!(first.len(), 128);
    assert_eq!(first.len() + second.len(), 131);
    assert_eq!(
        f.core.store.get::<Option<String>>(&member_bucket, &digest("visible-user")).unwrap(),
        Some(Some("visible-user".into()))
    );

    // A coherent v7 snapshot has neither Group-to-member collection. Opening
    // it must backfill both the short page and long-ID overflow on each backend.
    f.core.store.write(|tx| {
        let mut activation: Value = tx.get("meta", "version_activation")?.unwrap();
        activation["index_version"] = json!(7);
        tx.put("meta", "version_activation", &activation)?;
        tx.put("meta", "index_version", &7u32)?;
        for bucket in ["index_group_members", "index_group_member_overflow"] {
            for (key, _) in tx.list::<Value>(bucket)? {
                tx.delete(bucket, &key)?;
            }
        }
        Ok(())
    }).unwrap();
    let f = f.reopen_with(|_| {});
    assert_eq!(
        f.core.store.get::<u32>("meta", "index_version").unwrap(),
        Some(riauth::store::maintenance::INDEX_VERSION)
    );
    let backfilled = f.core.store.read(|tx| tx.scan::<Option<String>>(&member_bucket, None, 128)).unwrap();
    assert_eq!(backfilled.len(), 128);
    assert_eq!(
        f.core.store.get::<String>(&overflow_bucket, &digest(&stale)).unwrap(),
        Some(stale.clone())
    );

    f.core.store.write(|tx| tx.rebuild_indexes()).unwrap();
    let rebuilt: String = f.core.store.get("index_group_source_digests", name).unwrap().unwrap();
    let binding: Value = f.core.store.get("index_group_bindings", name).unwrap().unwrap();
    assert_eq!(rebuilt, changed);
    assert_eq!(binding["source_digest"], changed);
    assert_eq!(
        f.core.store.get::<Option<String>>(&member_bucket, &digest(&stale)).unwrap(),
        Some(None)
    );
    assert_eq!(
        f.core.store.get::<String>(&overflow_bucket, &digest(&stale)).unwrap(),
        Some(stale.clone())
    );

    f.core.store.write(|tx| tx.delete("groups", name)).unwrap();
    assert!(f.core.store.get::<Value>("index_group_bindings", name).unwrap().is_none());
    assert!(f.core.store.get::<String>("index_group_source_digests", name).unwrap().is_none());
    assert!(f.core.store.get::<Option<String>>(&member_bucket, &digest("visible-user")).unwrap().is_none());
    assert!(f.core.store.get::<String>(&overflow_bucket, &digest(&stale)).unwrap().is_none());
}

/// Updating a Group reads its large source value once. Index and audit work
/// share that snapshot rather than fetching the same value again.
pub fn group_write_source_read_amplification(backend: Backend) {
    use riauth::telemetry::ReadContext;

    let f = backend.fixture();
    let name = "large-update-group";
    let stale = format!("stale-{}", "x".repeat(200_000));
    let mut group = Group {
        name: name.into(),
        members: BTreeSet::from([stale.clone()]),
    };
    f.core.store.write(|tx| tx.put("groups", name, &group)).unwrap();
    let source_bytes = serde_json::to_vec(&group).unwrap().len() as u64;
    let before_bytes = f.core.store.telemetry().reads.bytes(ReadContext::Writer);
    group.members.insert("second-member".into());
    f.core.store.write(|tx| tx.put("groups", name, &group)).unwrap();
    let read_bytes = f.core.store.telemetry().reads.bytes(ReadContext::Writer) - before_bytes;
    assert!(read_bytes >= source_bytes, "old Group source was not measured: {read_bytes}");
    assert!(
        read_bytes < source_bytes * 3 / 2,
        "Group update fetched more than one large source value: {read_bytes} bytes for {source_bytes}-byte source"
    );
    eprintln!("Group update source={source_bytes} writer_read={read_bytes}");
    let stored: Group = f.core.store.get("groups", name).unwrap().unwrap();
    assert_eq!(stored.members, group.members);
    let binding: Value = f.core.store.get("index_group_bindings", name).unwrap().unwrap();
    let source: String = f.core.store.get("index_group_source_digests", name).unwrap().unwrap();
    assert_eq!(binding["source_digest"], source);

    let source_bytes = serde_json::to_vec(&group).unwrap().len() as u64;
    let before_bytes = f.core.store.telemetry().reads.bytes(ReadContext::Writer);
    f.core.store.write(|tx| tx.delete("groups", name)).unwrap();
    let read_bytes = f.core.store.telemetry().reads.bytes(ReadContext::Writer) - before_bytes;
    assert!(read_bytes >= source_bytes, "old Group source was not measured: {read_bytes}");
    assert!(
        read_bytes < source_bytes * 3 / 2,
        "Group deletion fetched more than one large source value: {read_bytes} bytes for {source_bytes}-byte source"
    );
    eprintln!("Group delete source={source_bytes} writer_read={read_bytes}");
    assert!(f.core.store.get::<Value>("index_group_bindings", name).unwrap().is_none());
}

/// S02: a large group directory has the same live membership and snapshot
/// behavior on both storage backends, including the 128-row index page edge.
pub fn indexed_user_group_membership(backend: Backend) {
    use riauth::telemetry::ReadContext;
    use std::sync::mpsc;
    use std::time::Duration;

    let mut f = backend.fixture();
    let alice_token = f.user("alice");
    let alice = user(&f, "alice");
    f.core
        .store
        .write(|tx| {
            for n in 0..300 {
                let name = format!("group-{n:03}");
                tx.put(
                    "groups",
                    &name,
                    &Group {
                        name: name.clone(),
                        members: if n < 130 {
                            BTreeSet::from([alice.id.clone()])
                        } else {
                            BTreeSet::new()
                        },
                    },
                )?;
            }
            Ok(())
        })
        .unwrap();
    let expected: Vec<_> = (0..130).map(|n| format!("group-{n:03}")).collect();
    let unbounded = f
        .core
        .store
        .telemetry()
        .reads
        .scans(ReadContext::Read, false)
        .sum();
    assert_eq!(f.core.me(&alice_token).unwrap()["groups"], json!(expected));
    assert_eq!(
        f.core
            .store
            .telemetry()
            .reads
            .scans(ReadContext::Read, false)
            .sum(),
        unbounded,
        "me must not materialize the whole groups collection"
    );

    // An ordinary management mutation updates the index atomically and advances
    // the management revision used by paginated inventory and user reports.
    let revision: u64 = f.core.store.get("meta", "revision").unwrap().unwrap();
    let store = f.core.store.clone();
    let core = f.core.clone();
    let admin = f.admin.clone();
    let (committed, receiver) = mpsc::channel();
    f.core
        .store
        .read(|tx| {
            assert!(tx.user_group_names(&alice.id)?.contains("group-000"));
            let writer = std::thread::spawn(move || {
                core.group_member(&admin, "group-000", "alice", false)
                    .unwrap();
                committed.send(()).unwrap();
            });
            receiver.recv_timeout(Duration::from_secs(10)).unwrap();
            assert!(tx.user_group_names(&alice.id)?.contains("group-000"));
            writer.join().unwrap();
            Ok(())
        })
        .unwrap();
    assert!(
        !store
            .read(|tx| tx.user_group_names(&alice.id))
            .unwrap()
            .contains("group-000")
    );
    assert!(
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap()
            > revision
    );

    // A replacement, a deletion, and an aborted transaction cannot leave stale
    // membership entries even when they bypass the public group handler.
    f.core
        .store
        .write(|tx| {
            let mut group: Group = tx.get("groups", "group-001")?.unwrap();
            group.members.clear();
            tx.put("groups", "group-001", &group)?;
            tx.delete("groups", "group-002")?;
            Ok(())
        })
        .unwrap();
    let aborted: riauth::error::Result<()> = f.core.store.write(|tx| {
        let mut group: Group = tx.get("groups", "group-003")?.unwrap();
        group.members.clear();
        tx.put("groups", "group-003", &group)?;
        Err(Error::bad("abort"))
    });
    assert!(aborted.is_err());
    let names = f
        .core
        .store
        .read(|tx| tx.user_group_names(&alice.id))
        .unwrap();
    assert!(!names.contains("group-001") && !names.contains("group-002"));
    assert!(names.contains("group-003"));

    // A restored-state or upgrade rebuild reconstructs derived membership
    // entries from durable Group records without rolling back activation.
    f.core
        .store
        .write(|tx| {
            for (key, _) in tx.list::<Value>("index_user_groups")? {
                tx.delete("index_user_groups", &key)?;
            }
            assert!(tx.user_group_names(&alice.id)?.is_empty());
            tx.rebuild_indexes()
        })
        .unwrap();
    drop(store);
    f = f.reopen_with(|_| {});
    assert_eq!(f.core.me(&alice_token).unwrap()["groups"], json!(names));
    assert_eq!(
        f.core.store.get::<u32>("meta", "index_version").unwrap(),
        Some(riauth::store::maintenance::INDEX_VERSION)
    );
}

// Only synthetic, undelivered test messages are inspected for a proof token.
fn configure_mail(f: &mut Fixture) {
    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
}

fn mail_codes_for(f: &Fixture, username: &str) -> Vec<String> {
    let account = format!("Account: {username}");
    f.core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .filter_map(|(_, delivery)| {
            let body = delivery["body"].as_str()?;
            body.contains(&account).then(|| {
                body.lines()
                    .find(|line| line.starts_with("ri_mail_"))
                    .unwrap()
                    .to_owned()
            })
        })
        .collect()
}

fn mail_code_for(f: &Fixture, username: &str) -> String {
    mail_codes_for(f, username).into_iter().next().unwrap()
}

// RI-ACC-001, RI-DIST-001, Q02-C01: reopening the same instance, not a
// migration/product-parity claim. An issuer mismatch must fail for its own reason.
pub fn identity_and_issuer_continuity(backend: Backend) {
    let f = backend.fixture();
    f.client("app", false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    let id = user(&f, "alice").id;
    assert_ne!(id, user(&f, "bob").id);
    let tokens = f.tokens("app", &alice, None);
    let subject = text(
        &f.core.userinfo(&text(&tokens, "access_token")).unwrap(),
        "sub",
    );
    assert_eq!(
        subject, id,
        "the default subject is the stable local identity"
    );
    let issuer = f.core.config.issuer.clone();
    let keys = f.core.jwks().unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                display_name: Some("Alice renamed".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let before = f.snapshot().unwrap();
    let f = f.reopen_with(|config| {
        let mut wrong = config.clone();
        wrong.issuer = "https://different-issuer.example.test".into();
        let error = Core::open(wrong)
            .err()
            .expect("changed issuer must be rejected");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert!(error.to_string().contains("issuer"));
    });
    f.assert_snapshot(&before);
    assert_eq!(user(&f, "alice").id, id);
    assert_eq!(f.core.config.issuer, issuer);
    assert_eq!(f.core.jwks().unwrap(), keys);
    assert_eq!(f.core.me(&alice).unwrap()["user"]["id"], id);
    assert!(f.core.me(&bob).is_ok());
    assert_eq!(
        f.core.userinfo(&text(&tokens, "access_token")).unwrap()["sub"],
        subject
    );
    let rotated = f.core.token(refresh("app", &tokens)).unwrap();
    assert_eq!(
        f.core.userinfo(&text(&rotated, "access_token")).unwrap()["sub"],
        subject
    );
}

// A01 INV-1, RI-SES-004, RI-MGT-004, Q02-C01/C04/C08.
// Shared existing dependent/SSF assertions are reused across three real writers.
pub fn disable_reenable_revokes_dependents(backend: Backend) {
    for writer in ["core", "manifest", "scim-patch"] {
        let f = backend.fixture();
        f.client_with_settings(
            "app",
            false,
            riauth::model::ProviderSettings {
                backchannel_logout_uri: Some("https://app.example.test/logout".into()),
                ..Default::default()
            },
        );
        let (alice, scim_id) =
            if writer == "scim-patch" {
                let created = f.core.scim_write(&f.admin, "Users", None, json!({
                "schemas":[riauth::scim::USER],"userName":"alice", "displayName":"Alice",
                "password":PASSWORD,"active":true
            }), false).unwrap();
                let alice = text(
                    &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
                    "session_token",
                );
                (alice, Some(text(&created, "id")))
            } else {
                (f.user("alice"), None)
            };
        let original = user(&f, "alice");
        let tokens = f.tokens("app", &alice, None);
        let unspent = f.exchange_request("app", &alice, None);
        let code_key = digest(unspent.code.as_deref().unwrap());
        assert_eq!(
            f.core
                .store
                .get::<Value>("codes", &code_key)
                .unwrap()
                .unwrap()["issued_family"],
            Value::Null,
            "the code must be unspent before disabling the account"
        );
        subscribe(&f, "alice");
        let children = Dependents::create(&f, "alice");
        assert!(
            f.core
                .store
                .get::<Agent>("agents", "child-alice")
                .unwrap()
                .unwrap()
                .enabled
        );
        assert_eq!(
            f.core
                .store
                .get::<Value>("windows_devices", "device-alice")
                .unwrap()
                .unwrap()["revoked"],
            false
        );
        assert!(events(&f, "alice").is_empty());
        match writer {
            "core" => {
                f.core
                    .update_user(
                        &f.admin,
                        "alice",
                        UserPatch {
                            enabled: Some(false),
                            ..Default::default()
                        },
                    )
                    .unwrap();
            }
            "manifest" => {
                let manifest = serde_json::from_value(json!({"api_version":"riauth/v1","users":[{"username":"alice","display_name":"Alice","enabled":false}]})).unwrap();
                let plan = f.core.plan_state(&f.admin, manifest).unwrap();
                assert!(events(&f, "alice").is_empty());
                assert!(user(&f, "alice").enabled, "preview is not revocation");
                let input = || ApplyRequest {
                    plan: plan.clone(),
                    secrets: Default::default(),
                    run_id: None,
                };
                f.core.apply_state(&f.admin, input()).unwrap();
                let committed = f.snapshot().unwrap();
                f.core.apply_state(&f.admin, input()).unwrap();
                f.assert_snapshot(&committed);
            }
            "scim-patch" => {
                f.core
                    .scim_write(
                        &f.admin,
                        "Users",
                        scim_id.as_deref(),
                        json!({
                            "schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
                            "Operations":[{"op":"replace","path":"active","value":false}]
                        }),
                        true,
                    )
                    .unwrap();
            }
            _ => unreachable!(),
        }
        assert!(!user(&f, "alice").enabled, "{writer}");
        assert!(user(&f, "alice").epoch > original.epoch);
        assert!(f.core.me(&alice).is_err());
        assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
        assert!(f.core.token(refresh("app", &tokens)).is_err());
        let disabled = f.snapshot().unwrap();
        assert_eq!(
            f.core.token(unspent.clone()).unwrap_err().code,
            "invalid_grant"
        );
        f.assert_snapshot(&disabled);
        children.assert_revoked(&f);
        assert_eq!(
            events(&f, "alice"),
            vec![(riauth::ssf::ACCOUNT_DISABLED.into(), "".into())]
        );
        assert_eq!(
            f.core
                .store
                .list::<Value>("logout_deliveries")
                .unwrap()
                .len(),
            1
        );
        f.core
            .update_user(
                &f.admin,
                "alice",
                UserPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .unwrap();
        children.assert_revoked_after_reenable(&f);
        assert!(
            user(&f, "alice").enabled,
            "{writer}: account was not re-enabled"
        );
        assert_eq!(user(&f, "alice").id, original.id);
        assert!(
            f.core.me(&alice).is_err(),
            "re-enable must not restore the old session"
        );
        assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
        assert!(f.core.token(refresh("app", &tokens)).is_err());
        let reenabled = f.snapshot().unwrap();
        assert_eq!(
            f.core.token(unspent).unwrap_err().code,
            "invalid_grant",
            "{writer}: an old unspent code must not mint tokens after re-enable"
        );
        f.assert_snapshot(&reenabled);
        assert_eq!(audit_count(&f, "authorization_code.replay"), 0);
        assert_eq!(events(&f, "alice").len(), 1, "{writer}: duplicate signal");
        assert_eq!(
            f.core
                .store
                .list::<Value>("logout_deliveries")
                .unwrap()
                .len(),
            1
        );
        assert!(f.core.login("alice".into(), PASSWORD.into(), None).is_ok());
    }
}

// A01 INV-2, RI-SES-002/003, Q02-C03: real session/proof and authorization path.
pub fn proof_account_session_request_binding(backend: Backend) {
    let f = backend.fixture();
    f.client("app", false);
    let alice = session(&f, &f.user("alice"));
    let bob = session(&f, &f.user("bob"));
    let second = session(
        &f,
        &text(
            &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        ),
    );
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.prompt = Some("login".into());
    request.request_binding = Some("interaction-1".into());
    let hash = request.request_hash().unwrap();
    let key = f
        .core
        .store
        .write(|tx| {
            signin::bind_proof(
                tx,
                None,
                hash.clone(),
                &alice.identity.user_id,
                &alice.id,
                now() + 600,
            )
        })
        .unwrap();
    let decide = |s: &Session, r: &Authorization, key: Option<&str>| {
        f.core.store.write(|tx| {
            f.core
                .authorize_session_proof(tx, s.clone(), r.clone(), false, key)
        })
    };
    let before = f.snapshot().unwrap();
    assert_eq!(
        decide(&alice, &request, None).unwrap_err().code,
        "login_required"
    );
    for wrong in [&bob, &second] {
        assert_eq!(
            decide(wrong, &request, Some(&key)).unwrap_err().code,
            "login_required"
        );
    }
    for field in ["nonce", "interaction"] {
        let mut wrong = request.clone();
        if field == "nonce" {
            wrong.nonce = Some("another-request".into());
        } else {
            wrong.request_binding = Some("interaction-2".into());
        }
        assert!(decide(&alice, &wrong, Some(&key)).is_err());
    }
    f.assert_snapshot(&before);
    let callback = decide(&alice, &request, Some(&key)).unwrap();
    let code = url::Url::parse(&callback)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    assert!(
        f.core
            .store
            .get::<Value>("authentication", &key)
            .unwrap()
            .is_none()
    );
    let committed = f.snapshot().unwrap();
    assert_eq!(
        decide(&alice, &request, Some(&key)).unwrap_err().code,
        "login_required"
    );
    f.assert_snapshot(&committed);
    assert_eq!(f.core.store.list::<Value>("codes").unwrap().len(), 1);
    let tokens = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(code),
            redirect_uri: Some(request.redirect_uri),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    let info = f.core.userinfo(&text(&tokens, "access_token")).unwrap();
    assert_eq!(info["preferred_username"], "alice");
}

// A01 INV-3, RI-SES-003, Q02-C03: invalid binding cannot spend or kill a grant.
pub fn code_binding_and_verified_replay(backend: Backend) {
    let f = backend.fixture();
    f.client("app", false);
    f.client("other", false);
    let alice = f.user("alice");
    let request = f.exchange_request("app", &alice, None);
    let before = f.snapshot().unwrap();
    let wrong_requests = |request: &TokenRequest| {
        let mut client = request.clone();
        client.client_id = Some("other".into());
        let mut redirect = request.clone();
        redirect.redirect_uri = Some("https://attacker.example.test/callback".into());
        let mut pkce = request.clone();
        pkce.code_verifier = Some(crypto::random_token(""));
        [client, redirect, pkce]
    };
    for wrong in wrong_requests(&request) {
        assert_eq!(f.core.token(wrong).unwrap_err().code, "invalid_grant");
        f.assert_snapshot(&before);
    }
    let tokens = f.core.token(request.clone()).unwrap();
    let unrelated = f.tokens("other", &alice, None);
    let committed = f.snapshot().unwrap();
    for wrong in wrong_requests(&request) {
        assert_eq!(f.core.token(wrong).unwrap_err().code, "invalid_grant");
        f.assert_snapshot(&committed);
    }
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_ok());
    assert_eq!(f.core.token(request).unwrap_err().code, "invalid_grant");
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(f.core.token(refresh("app", &tokens)).is_err());
    assert!(f.core.userinfo(&text(&unrelated, "access_token")).is_ok());
    assert!(f.core.me(&alice).is_ok());
}

// RI-SES-003/005, Q02-C03: rotated refresh handles are single-use; bound replay
// revokes that family, rather than the unrelated family or the login session.
pub fn refresh_rotation_and_verified_replay(backend: Backend) {
    let f = backend.fixture();
    f.client("app", false);
    f.client("other", false);
    let alice = f.user("alice");
    let tokens = f.tokens("app", &alice, None);
    let unrelated = f.tokens("other", &alice, None);
    let before = f.snapshot().unwrap();
    assert_eq!(
        f.core.token(refresh("other", &tokens)).unwrap_err().code,
        "invalid_grant"
    );
    f.assert_snapshot(&before);
    let rotated = f.core.token(refresh("app", &tokens)).unwrap();
    assert_ne!(tokens["refresh_token"], rotated["refresh_token"]);
    assert!(f.core.userinfo(&text(&rotated, "access_token")).is_ok());
    let committed = f.snapshot().unwrap();
    assert_eq!(
        f.core.token(refresh("other", &tokens)).unwrap_err().code,
        "invalid_grant"
    );
    f.assert_snapshot(&committed);
    assert_eq!(
        f.core.token(refresh("app", &tokens)).unwrap_err().code,
        "invalid_grant"
    );
    for old in [&tokens, &rotated] {
        assert!(f.core.userinfo(&text(old, "access_token")).is_err());
        assert_eq!(
            f.core.token(refresh("app", old)).unwrap_err().code,
            "invalid_grant"
        );
    }
    assert!(f.core.userinfo(&text(&unrelated, "access_token")).is_ok());
    assert!(f.core.me(&alice).is_ok());
}

// A01 INV-8, RI-AUTH-001, Q02-C05: existing tokens use live group policy.
pub fn live_group_policy_revalidation(backend: Backend) {
    let f = backend.fixture();
    f.client("app", false);
    f.client("open", false);
    let alice = f.user("alice");
    f.core.create_group(&f.admin, "developers").unwrap();
    crate::common::client_policy::replace(
        &f.core,
        &f.admin,
        "app",
        Some(strings(&["developers"])),
        None,
    );
    assert!(
        f.core
            .authorize(&alice, f.request("app", &crypto::random_token("")))
            .is_err()
    );
    f.core
        .group_member(&f.admin, "developers", "alice", true)
        .unwrap();
    let tokens = f.tokens("app", &alice, None);
    let unrelated = f.tokens("open", &alice, None);
    let access = text(&tokens, "access_token");
    assert_eq!(
        f.core.userinfo(&access).unwrap()["groups"],
        json!(["developers"])
    );
    assert!(f.core.proxy_auth(&access, "app").is_ok());
    assert!(f.core.proxy_auth(&access, "wrong-audience").is_err());
    f.core
        .group_member(&f.admin, "developers", "alice", false)
        .unwrap();
    assert!(f.core.userinfo(&access).is_err());
    assert!(f.core.proxy_auth(&access, "app").is_err());
    assert!(f.core.token(refresh("app", &tokens)).is_err());
    assert!(
        f.core
            .authorize(&alice, f.request("app", &crypto::random_token("")))
            .is_err()
    );
    assert!(f.core.me(&alice).is_ok());
    assert!(f.core.userinfo(&text(&unrelated, "access_token")).is_ok());
    assert!(
        f.core
            .store
            .get::<Group>("groups", "developers")
            .unwrap()
            .unwrap()
            .members
            .is_empty()
    );
}

// A01 INV-4, RI-STORE-001, Q02-C08. This lifts the prepared-store primitive
// regression; it does not claim an actual password/signing-operation race.
pub fn prepared_authority_revalidation(backend: Backend) {
    let f = backend.fixture();
    let alice = f.user("alice");
    let s = session(&f, &alice);
    let mut revoke = true;
    let mut after_revocation = None;
    let result = f.core.store.prepared_write(|tx| {
        let user: User = tx.get("users", &s.identity.user_id)?.unwrap();
        let session: Session = tx.get("sessions", &s.id)?.unwrap();
        if !user.enabled || session.revoked || user.epoch != session.identity.epoch {
            return Err(Error::forbidden());
        }
        if std::mem::take(&mut revoke) {
            // A real authority change can commit while preparation holds reads.
            f.core.update_user(
                &f.admin,
                "alice",
                UserPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )?;
            after_revocation = Some(f.snapshot()?);
        }
        tx.put(
            "contract_effects",
            "issued",
            &json!({"credential":"must-not-escape"}),
        )
    });
    assert_eq!(result.unwrap_err().status, StatusCode::FORBIDDEN);
    f.assert_snapshot(&after_revocation.unwrap());
    assert!(
        f.core
            .store
            .get::<Value>("contract_effects", "issued")
            .unwrap()
            .is_none()
    );
    assert!(f.core.me(&alice).is_err());
}

// RI-STORE-001/RI-SES-005, Q02-C08. Controlled clock; no timing sleep or
// database mutation stands in for expiry. This is a store deadline contract.
#[cfg(feature = "test-support")]
pub fn prepared_deadline_revalidation(backend: Backend) {
    // Key initialization validates JWTs against the library's real wall clock.
    // Scope the controlled clock only around the store operation under test.
    let f = backend.fixture();
    let at = now();
    crypto::with_test_time(at, || {
        f.core
            .store
            .write(|tx| {
                tx.put(
                    "contract_authority",
                    "grant",
                    &json!({"expires_at":now()+1}),
                )
            })
            .unwrap();
        let before = f.snapshot().unwrap();
        let result = f.core.store.prepared_write(|tx| {
            let grant: Value = tx.get("contract_authority", "grant")?.unwrap();
            if grant["expires_at"].as_u64().unwrap() <= now() {
                return Err(Error::forbidden());
            }
            crypto::set_test_time(at + 1);
            tx.put("contract_effects", "issued", &true)
        });
        assert_eq!(result.unwrap_err().status, StatusCode::FORBIDDEN);
        f.assert_snapshot(&before);
    });
}

// RI-ACC-002, RI-STORE-001, Q02-C04/C08: reject after a real password/history
// change has been staged; every record, index and audit must roll back.
pub fn last_admin_failure_is_atomic(backend: Backend) {
    let f = backend.fixture();
    let before = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .update_user(
                &f.admin,
                "admin",
                UserPatch {
                    password: Some("replacement-password-for-contract-only".into()),
                    enabled: Some(false),
                    ..Default::default()
                }
            )
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    f.assert_snapshot(&before);
    assert!(f.core.me(&f.admin).is_ok());
    assert!(f.core.login("admin".into(), PASSWORD.into(), None).is_ok());
}

// RI-MGT-001/003/004, RI-STORE-001/002, Q02-C04/C08/C09. HTTP supplies real
// fingerprints/preconditions; snapshots include receipt, secret, revision/audit.
pub fn http_mutation_receipts_and_audit(backend: Backend) {
    let f = backend.fixture();
    let ordinary = f.user("ordinary");
    let token = agent(
        &f,
        "writer",
        &[
            ("client.write", "client/app"),
            ("client.rotate", "client/app"),
        ],
    );
    let other = agent(&f, "other", &[("client.write", "client/other")]);
    let app = riauth::api::router(f.core.clone());
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let call = |token: &str, key: &str, revision: Option<u64>, body: &Value| {
        let mut request = Request::post("/api/clients")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .header("idempotency-key", key)
            .header("x-riauth-run-id", "contract-run");
        if let Some(revision) = revision {
            request = request.header("if-match", format!("\"{revision}\""));
        }
        runtime.block_on(async {
            let response = app
                .clone()
                .oneshot(request.body(Body::from(body.to_string())).unwrap())
                .await
                .unwrap();
            let status = response.status();
            assert!(response.headers().contains_key("x-request-id"));
            let body: Value =
                serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                    .unwrap();
            (status, body)
        })
    };
    let revision = f
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap();
    let body = json!({"client_id":"app","name":"app","confidential":true,"redirect_uris":[],"scopes":["openid"],"allowed_groups":[],"require_mfa":false,"service":false});
    let before = f.snapshot().unwrap();
    let mut denied = body.clone();
    denied["client_id"] = json!("denied");
    for credential in [&token, &ordinary] {
        assert_eq!(
            call(credential, "denied", Some(revision), &denied).0,
            StatusCode::FORBIDDEN
        );
        f.assert_http_mutation_snapshot(&before);
    }
    let counters = f.core.store.list::<(u64, u32)>("http_rates").unwrap();
    assert_eq!(
        counters.len(),
        usize::from(f.core.config.postgres.is_some())
    );
    for (_, (_, count)) in counters {
        assert_eq!(count, 2, "both denied requests must still be counted");
    }
    assert_eq!(
        call(&token, "create", None, &body).0,
        StatusCode::PRECONDITION_REQUIRED
    );
    assert_eq!(
        call(&token, "create", Some(revision + 1), &body).0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&before);
    let (status, first) = call(&token, "create", Some(revision), &body);
    assert_eq!(status, StatusCode::OK);
    let secret = text(&first, "client_secret");
    let committed = f.snapshot().unwrap();
    assert_eq!(
        call(&token, "create", Some(revision), &body),
        (StatusCode::OK, first)
    );
    f.assert_http_mutation_snapshot(&committed);
    let mut modified = body.clone();
    modified["name"] = json!("changed");
    assert_eq!(
        call(&token, "create", Some(revision), &modified).0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(&token, "different", Some(revision), &body).0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&committed);
    let mut other_body = body.clone();
    other_body["client_id"] = json!("other");
    let next = f
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap();
    let (status, other_result) = call(&other, "create", Some(next), &other_body);
    assert_eq!(
        status,
        StatusCode::OK,
        "receipt keys are scoped to the actor"
    );
    let other_secret = text(&other_result, "client_secret");
    assert_ne!(secret, other_secret);
    assert_eq!(f.core.store.list::<Value>("receipts").unwrap().len(), 2);
    let events = audit(&f);
    let created: Vec<_> = events
        .iter()
        .filter(|e| e["action"] == "client.create")
        .collect();
    assert_eq!(created.len(), 2);
    assert!(
        created
            .iter()
            .any(|e| e["actor"] == "agent:writer" && e["target"] == "app")
    );
    assert!(
        created
            .iter()
            .any(|e| e["actor"] == "agent:other" && e["target"] == "other")
    );
    for event in created {
        assert_eq!(event["run_id"], "contract-run");
        assert!(event["details"]["request_id"].is_string());
        assert!(
            event["details"]["changes"]
                .as_array()
                .is_some_and(|c| !c.is_empty())
        );
    }
    let serialized = serde_json::to_string(&events).unwrap();
    for sensitive in [&secret, &other_secret, &token, &other, &digest(&secret)] {
        assert!(!serialized.contains(sensitive));
    }
    // There is no agent permission-edit endpoint yet; fixture mutation models
    // an authorized policy change. The real receipt path must revalidate it.
    f.core
        .store
        .write(|tx| {
            let mut agent: Agent = tx.get("agents", "writer")?.unwrap();
            agent.permissions.retain(|p| p.action != "client.rotate");
            tx.put("agents", "writer", &agent)
        })
        .unwrap();
    let changed = f.snapshot().unwrap();
    assert_eq!(
        call(&token, "create", Some(revision), &body).0,
        StatusCode::FORBIDDEN
    );
    f.assert_http_mutation_snapshot(&changed);
    f.core.revoke_agent(&f.admin, "writer").unwrap();
    let revoked = f.snapshot().unwrap();
    assert!(!call(&token, "create", Some(revision), &body).0.is_success());
    f.assert_http_mutation_snapshot(&revoked);
}

// RI-MGT-002/004, RI-STORE-001/002, Q02-C04/C08/C09. Exact plan fields and
// references are tested. Resolved-secret byte commitment is still a Q01 gap.
pub fn plan_binding_atomicity_and_retry(backend: Backend) {
    let f = backend.fixture();
    let permissions = [
        ("group.write", "group/managed"),
        ("group.read", "group/managed"),
        ("group.members", "group/managed"),
        ("client.write", "client/managed"),
        ("client.read", "client/managed"),
    ];
    let token = agent(&f, "planner", &permissions);
    let other = agent(&f, "other-planner", &permissions);
    let manifest: Manifest = serde_json::from_value(json!({
        "api_version":"riauth/v1", "groups":[{"name":"managed"}],
        "clients":[{"client_id":"managed","name":"Managed","confidential":true,"scopes":["openid"],"secret_ref":"env:CONTRACT_SECRET","secret_version":"v1"}]
    })).unwrap();
    let original_audit = audit(&f);
    let plan = f.core.plan_state(&token, manifest.clone()).unwrap();
    assert!(
        f.core
            .store
            .get::<Group>("groups", "managed")
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .get::<Client>("clients", "managed")
            .unwrap()
            .is_none()
    );
    assert_eq!(audit(&f), original_audit);
    let input = |plan, secrets| ApplyRequest {
        plan,
        secrets,
        run_id: Some("plan-contract-run".into()),
    };
    let before = f.snapshot().unwrap();
    for field in ["content", "reference", "version"] {
        let mut tampered = plan.clone();
        match field {
            "content" => tampered.manifest.clients[0].name = "Tampered".into(),
            "reference" => {
                tampered.manifest.clients[0].secret_ref = Some("env:SUBSTITUTED_SECRET".into())
            }
            "version" => tampered.manifest.clients[0].secret_version = Some("v2".into()),
            _ => unreachable!(),
        }
        assert_eq!(
            f.core
                .apply_state(&token, input(tampered, Default::default()))
                .unwrap_err()
                .status,
            StatusCode::CONFLICT
        );
    }
    assert_eq!(
        f.core
            .apply_state(&other, input(plan.clone(), Default::default()))
            .unwrap_err()
            .status,
        StatusCode::FORBIDDEN
    );
    let mut tampered = plan.clone();
    tampered.issuer = "https://other-issuer.example.test".into();
    assert_eq!(
        f.core
            .apply_state(&token, input(tampered, Default::default()))
            .unwrap_err()
            .status,
        StatusCode::FORBIDDEN
    );
    let missing = f
        .core
        .apply_state(&token, input(plan.clone(), Default::default()))
        .unwrap_err();
    assert_eq!(missing.status, StatusCode::BAD_REQUEST);
    assert_eq!(missing.message, "Required secret value was not supplied");
    f.assert_snapshot(&before);
    f.core.create_group(&f.admin, "dependency-change").unwrap();
    let changed = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .apply_state(&token, input(plan, Default::default()))
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    f.assert_snapshot(&changed);
    let plan = f.core.plan_state(&token, manifest).unwrap();
    let secret = "resolved-secret-for-contract-only-0123456789";
    let secrets = || [("env:CONTRACT_SECRET".into(), secret.into())].into();
    let first = f
        .core
        .apply_state(&token, input(plan.clone(), secrets()))
        .unwrap();
    assert_eq!(first["changed"], true);
    let committed = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .apply_state(&token, input(plan.clone(), secrets()))
            .unwrap(),
        first
    );
    f.assert_snapshot(&committed);
    let client: Client = f.core.store.get("clients", "managed").unwrap().unwrap();
    assert_eq!(client.secret_hash.as_deref(), Some(digest(secret).as_str()));
    assert!(
        f.core
            .store
            .get::<Group>("groups", "managed")
            .unwrap()
            .is_some()
    );
    let events = audit(&f);
    let applied: Vec<_> = events
        .iter()
        .filter(|e| e["action"] == "state.apply" && e["target"] == plan.plan_id)
        .collect();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0]["actor"], "agent:planner");
    assert_eq!(applied[0]["run_id"], "plan-contract-run");
    for view in [
        serde_json::to_value(plan).unwrap(),
        json!(events),
        f.core.export_state(&f.admin).unwrap(),
    ] {
        assert!(!view.to_string().contains(secret));
        assert!(!view.to_string().contains(&digest(secret)));
        assert!(!view.to_string().contains(&token));
    }
}

// M03 first slice; RI-MGT-001/002/003/004, RI-STORE-001/002, Q02-C04/C08/C09.
// One application intent reaches the same management seam from HTTP (the CLI's
// transport) and desired state, while an ambient browser cookie cannot reach it.
// Oracles are the documented rules: client type is immutable, credential changes
// need client.rotate, direct retries replay exactly, stale revisions change
// nothing, each committed write is audited once and secrets never leave the
// authorized response.
pub fn application_writers_share_management_seam(backend: Backend) {
    let f = backend.fixture();
    let jwks = f.core.jwks().unwrap();
    let mut managed = Vec::new();
    for id in ["signed", "public", "shared"] {
        for action in ["client.write", "client.read", "client.rotate"] {
            managed.push((action, format!("client/{id}")));
        }
    }
    let managed: Vec<_> = managed.iter().map(|(a, r)| (*a, r.as_str())).collect();
    let token = agent(&f, "app-manager", &managed);
    let writer = agent(
        &f,
        "app-writer",
        &[
            ("client.write", "client/shared"),
            ("client.read", "client/shared"),
        ],
    );
    f.core
        .create_client(
            &f.admin,
            riauth::model::NewClient {
                client_id: "shared".into(),
                name: "shared".into(),
                confidential: true,
                redirect_uris: vec![],
                scopes: strings(&["openid"]),
                allowed_groups: BTreeSet::new(),
                require_mfa: false,
                service: false,
                settings: Default::default(),
            },
        )
        .unwrap();
    let app = riauth::api::router(f.core.clone());
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let revision = || {
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let send = |method: &str, path: &str, headers: Vec<(&str, String)>, body: Option<&Value>| {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .header("x-riauth-run-id", "m03-run");
        for (name, value) in headers {
            request = request.header(name, value);
        }
        let body = body.map_or_else(Body::empty, |b| Body::from(b.to_string()));
        runtime.block_on(async {
            let response = app
                .clone()
                .oneshot(request.body(body).unwrap())
                .await
                .unwrap();
            let status = response.status();
            let bytes = response.into_body().collect().await.unwrap().to_bytes();
            (
                status,
                serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null),
            )
        })
    };
    let direct = |token: &str, key: &str, at: u64| {
        vec![
            ("authorization", format!("Bearer {token}")),
            ("idempotency-key", key.to_owned()),
            ("if-match", format!("\"{at}\"")),
        ]
    };
    let plan = |token: &str, clients: Value| {
        f.core.plan_state(
            token,
            serde_json::from_value(json!({"api_version": "riauth/v1", "clients": clients}))
                .unwrap(),
        )
    };
    let refused = |result: Result<riauth::state::Plan, Error>| match result {
        Ok(_) => panic!("desired state accepted a refused application change"),
        Err(error) => error,
    };
    let audited = |action: &str, target: &str| {
        audit(&f)
            .into_iter()
            .filter(|e| e["action"] == action && e["target"] == target)
            .collect::<Vec<_>>()
    };

    // Authorized create and exact retry through the direct adapter.
    let signed = json!({"client_id":"signed","name":"signed","confidential":true,"scopes":["openid"],
        "settings":{"token_endpoint_auth_method":"private_key_jwt","jwks":jwks}});
    let at = revision();
    let created = send(
        "POST",
        "/api/clients",
        direct(&token, "create-signed", at),
        Some(&signed),
    );
    assert_eq!(created.0, StatusCode::OK, "{}", created.1);
    assert_eq!(created.1["client"]["confidential"], true);
    assert!(created.1["client_secret"].is_null());
    assert_eq!(revision(), at + 1);
    let committed = f.snapshot().unwrap();
    assert_eq!(
        send(
            "POST",
            "/api/clients",
            direct(&token, "create-signed", at),
            Some(&signed)
        ),
        created
    );
    f.assert_http_mutation_snapshot(&committed);
    let public = json!({"client_id":"public","name":"public","scopes":["openid"]});
    let at = revision();
    let (status, body) = send(
        "POST",
        "/api/clients",
        direct(&token, "create-public", at),
        Some(&public),
    );
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["client"]["confidential"], false);

    // Denied type changes: both writers reject the same intent with the same
    // rule, and neither changes records, receipts, revision or audit.
    let before = f.snapshot().unwrap();
    let at = revision();
    for (id, key, settings) in [
        ("signed", "downgrade-unset", json!({})),
        (
            "signed",
            "downgrade-none",
            json!({"token_endpoint_auth_method":"none"}),
        ),
        (
            "public",
            "upgrade",
            json!({"token_endpoint_auth_method":"private_key_jwt","jwks":jwks}),
        ),
    ] {
        let (status, body) = send(
            "PATCH",
            &format!("/api/clients/{id}"),
            direct(&token, key, at),
            Some(&json!({"settings": settings})),
        );
        assert_eq!(status, StatusCode::BAD_REQUEST, "{id}/{key}: {body}");
        assert_eq!(
            body["error_description"],
            "Existing client type is immutable"
        );
        f.assert_http_mutation_snapshot(&before);
    }
    for (id, confidential) in [("signed", false), ("public", true)] {
        let error = refused(plan(
            &token,
            json!([{"client_id":id,"name":id,"confidential":confidential,"scopes":["openid"]}]),
        ));
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Existing client type is immutable");
        f.assert_http_mutation_snapshot(&before);
    }

    // Authorization precedes validation in both writers: without client.rotate
    // a type-changing authentication edit is forbidden, not merely invalid.
    let (status, _) = send(
        "PATCH",
        "/api/clients/shared",
        direct(&writer, "writer-downgrade", at),
        Some(&json!({"settings":{"token_endpoint_auth_method":"none"}})),
    );
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(
        refused(plan(
            &writer,
            json!([{"client_id":"shared","name":"shared","confidential":false,"scopes":["openid"],
                "settings":{"token_endpoint_auth_method":"none"}}]),
        ))
        .status,
        StatusCode::FORBIDDEN
    );
    f.assert_http_mutation_snapshot(&before);

    // Denied credential changes: client.write alone cannot rotate or change
    // authentication through either writer.
    let rotate_v2 = json!([{"client_id":"shared","name":"shared","confidential":true,"scopes":["openid"],
            "secret_ref":"env:M03_SECRET","secret_version":"v2"}]);
    let (status, _) = send(
        "POST",
        "/api/clients/shared/rotate-secret",
        direct(&writer, "writer-rotate", at),
        None,
    );
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = send(
        "PATCH",
        "/api/clients/shared",
        direct(&writer, "writer-auth", at),
        Some(&json!({"settings":{"token_endpoint_auth_method":"client_secret_post"}})),
    );
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(
        refused(plan(&writer, rotate_v2.clone())).status,
        StatusCode::FORBIDDEN
    );
    f.assert_http_mutation_snapshot(&before);

    // Browser boundary: an ambient SSO cookie, even one holding an
    // administrator session, is not a management credential.
    let cookie = format!(
        "{}={}",
        riauth::signin::sso_cookie_name(&f.core.config.issuer),
        f.admin
    );
    for (method, path, body) in [
        (
            "POST",
            "/api/clients",
            Some(json!({"client_id":"browser","name":"browser","scopes":["openid"]})),
        ),
        (
            "PATCH",
            "/api/clients/shared",
            Some(json!({"enabled": false})),
        ),
        ("POST", "/api/clients/shared/rotate-secret", None),
    ] {
        let (status, _) = send(
            method,
            path,
            vec![
                ("cookie", cookie.clone()),
                ("x-riauth-portal", "1".into()),
                ("origin", f.core.config.issuer.trim_end_matches('/').into()),
                ("sec-fetch-site", "same-origin".into()),
            ],
            body.as_ref(),
        );
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {path}");
        f.assert_http_mutation_snapshot(&before);
    }

    // Authorized rotation: direct retry replays the one returned secret; a
    // stale revision is rejected before any effect.
    let (status, rotated) = send(
        "POST",
        "/api/clients/shared/rotate-secret",
        direct(&token, "rotate-shared", at),
        None,
    );
    assert_eq!(status, StatusCode::OK, "{rotated}");
    let first = text(&rotated, "client_secret");
    assert_eq!(revision(), at + 1);
    let committed = f.snapshot().unwrap();
    assert_eq!(
        send(
            "POST",
            "/api/clients/shared/rotate-secret",
            direct(&token, "rotate-shared", at),
            None
        ),
        (StatusCode::OK, rotated)
    );
    let (status, _) = send(
        "POST",
        "/api/clients/shared/rotate-secret",
        direct(&token, "rotate-stale", at),
        None,
    );
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&committed);

    // Desired state rotates through the same seam; a stale plan changes nothing.
    let second = "m03-desired-state-secret-0123456789abcdef";
    let secrets =
        || -> BTreeMap<String, String> { [("env:M03_SECRET".into(), second.into())].into() };
    let stale = plan(&token, rotate_v2.clone()).unwrap();
    f.core.create_group(&f.admin, "unrelated-change").unwrap();
    let changed = f.snapshot().unwrap();
    let error = f
        .core
        .apply_state(
            &token,
            ApplyRequest {
                plan: stale,
                secrets: secrets(),
                run_id: None,
            },
        )
        .unwrap_err();
    assert_eq!(error.status, StatusCode::CONFLICT);
    f.assert_snapshot(&changed);
    let fresh = plan(&token, rotate_v2).unwrap();
    let at = revision();
    let applied = f
        .core
        .apply_state(
            &token,
            ApplyRequest {
                plan: fresh.clone(),
                secrets: secrets(),
                run_id: Some("m03-run".into()),
            },
        )
        .unwrap();
    assert_eq!(applied["changed"], true);
    assert_eq!(revision(), at + 1);
    let stored: Client = f.core.store.get("clients", "shared").unwrap().unwrap();
    assert_eq!(stored.secret_hash.as_deref(), Some(digest(second).as_str()));

    // Pure rotation does not revalidate drifted, unrelated configuration (a
    // policy user renamed by directory sync), so an emergency rotation still
    // works; a record change is fully validated and refused atomically.
    f.user("policy-user");
    let mut drifted = f
        .core
        .store
        .get::<Client>("clients", "shared")
        .unwrap()
        .unwrap();
    drifted.settings.policy.access.users = strings(&["policy-user"]);
    let drifted_view = f
        .core
        .update_client(
            &f.admin,
            "shared",
            ClientPatch {
                settings: Some(drifted.settings.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(drifted_view["client_id"], "shared");
    f.core
        .store
        .write(|tx| tx.delete("usernames", "policy-user"))
        .unwrap();
    let before_rotation = revision();
    let emergency = f.core.rotate_client_secret(&f.admin, "shared").unwrap();
    assert_eq!(revision(), before_rotation + 1);
    let emergency = text(&emergency, "client_secret");
    let unchanged = f.snapshot().unwrap();
    let error = f
        .core
        .update_client(
            &f.admin,
            "shared",
            ClientPatch {
                name: Some("Renamed".into()),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(error.status, StatusCode::BAD_REQUEST);
    assert_eq!(error.message, "Unknown policy user: policy-user");
    f.assert_snapshot(&unchanged);

    // One audit per committed write, with correlation, redacted changes and
    // no secret, digest or agent credential anywhere in audit, plan or export.
    for (action, target, expected) in [
        ("client.create", "signed", 1),
        ("client.create", "public", 1),
        ("client.update", "signed", 0),
        ("client.update", "public", 0),
        ("client.update", "shared", 1),
        ("client.secret.rotate", "shared", 2),
        ("client.reconcile", "client/shared", 1),
    ] {
        let events = audited(action, target);
        assert_eq!(events.len(), expected, "{action} {target}");
        for event in events {
            // HTTP writes carry correlation; plan correlation is on state.apply
            // and the administrator's in-process calls have no request context.
            if action != "client.reconcile" && event["actor"] == "agent:app-manager" {
                assert_eq!(event["run_id"], "m03-run");
                assert!(event["details"]["request_id"].is_string());
            }
            assert!(
                event["details"]["changes"]
                    .as_array()
                    .is_some_and(|c| !c.is_empty())
            );
        }
    }
    let applied_events = audited("state.apply", &fresh.plan_id);
    assert_eq!(applied_events.len(), 1);
    assert_eq!(applied_events[0]["run_id"], "m03-run");
    let events = json!(audit(&f)).to_string();
    for view in [
        events,
        serde_json::to_value(&fresh).unwrap().to_string(),
        f.core.export_state(&f.admin).unwrap().to_string(),
    ] {
        for sensitive in [
            first.clone(),
            digest(&first),
            emergency.clone(),
            digest(&emergency),
            second.to_owned(),
            digest(second),
            token.clone(),
            writer.clone(),
        ] {
            assert!(!view.contains(&sensitive));
        }
    }
}

// RI-CRED-001/002, RI-STORE-001, Q02-C02: a failed password is accounted for,
// while rejected writes leave the credential and existing authority intact.
pub fn password_attempts_and_change(backend: Backend) {
    let f = backend.fixture();
    f.client("app", false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    let original = user(&f, "alice");
    let access = f.tokens("app", &alice, None);
    for failures in 1..=5 {
        assert_eq!(
            f.core
                .login("alice".into(), "incorrect-password".into(), None)
                .unwrap_err()
                .code,
            "invalid_credentials"
        );
        let attempts: Attempts = f.core.store.get("attempts", "alice").unwrap().unwrap();
        assert_eq!(attempts.failures, failures);
        assert_eq!(user(&f, "alice").password_hash, original.password_hash);
        assert!(f.core.me(&alice).is_ok());
    }
    assert_eq!(audit_count(&f, "login.failed"), 5);
    assert_eq!(
        f.core
            .login("alice".into(), PASSWORD.into(), None)
            .unwrap_err()
            .code,
        "rate_limited"
    );
    assert_eq!(audit_count(&f, "login.locked"), 1);
    assert_eq!(
        f.core
            .store
            .get::<Attempts>("attempts", "alice")
            .unwrap()
            .unwrap()
            .failures,
        5
    );
    assert!(f.core.me(&bob).is_ok());

    let reset = "admin-cleared-owner-password";
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                password: Some(reset.into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        f.core
            .store
            .get::<Attempts>("attempts", "alice")
            .unwrap()
            .is_none()
    );
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.userinfo(&text(&access, "access_token")).is_err());
    assert_eq!(user(&f, "alice").id, original.id);
    let before_reuse = f.snapshot().unwrap();
    assert!(
        f.core
            .update_user(
                &f.admin,
                "alice",
                UserPatch {
                    password: Some(reset.into()),
                    ..Default::default()
                }
            )
            .is_err()
    );
    f.assert_snapshot(&before_reuse);

    let fresh = text(
        &f.core.login("alice".into(), reset.into(), None).unwrap(),
        "session_token",
    );
    let before_wrong = user(&f, "alice");
    assert!(
        f.core
            .change_password(
                &fresh,
                "incorrect-password".into(),
                "new-owner-only-password".into(),
                None
            )
            .is_err()
    );
    assert_eq!(user(&f, "alice").password_hash, before_wrong.password_hash);
    assert_eq!(user(&f, "alice").epoch, before_wrong.epoch);
    assert_eq!(audit_count(&f, "user.password.change"), 0);
    assert!(f.core.me(&fresh).is_ok());
    assert!(f.core.me(&bob).is_ok());
    assert_eq!(
        f.core
            .change_password(&fresh, reset.into(), "new-owner-only-password".into(), None)
            .unwrap()["sessions_revoked"],
        true
    );
    assert_eq!(audit_count(&f, "user.password.change"), 1);
    assert!(f.core.me(&fresh).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(f.core.login("alice".into(), reset.into(), None).is_err());
    assert!(
        f.core
            .login("alice".into(), "new-owner-only-password".into(), None)
            .is_ok()
    );
    assert!(
        !serde_json::to_string(&audit(&f))
            .unwrap()
            .contains("new-owner-only-password")
    );
}

// RI-CRED-001/002, Q02-C02: enrollment is account-bound, a TOTP step cannot
// be replayed, and a recovery code is spent even when request binding fails.
pub fn totp_and_recovery_code_binding(backend: Backend) {
    let f = backend.fixture();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let previous = user(&f, "alice");
    let pending = f.core.mfa_begin(&alice).unwrap();
    let secret = text(&pending, "secret");
    let totp = crypto::totp(&secret, "alice").unwrap();
    let before_rejection = f.snapshot().unwrap();
    assert!(
        f.core
            .mfa_confirm(&bob, &totp.generate(now() - 30).to_string())
            .is_err()
    );
    assert!(f.core.mfa_confirm(&alice, "invalid-code").is_err());
    f.assert_snapshot(&before_rejection);
    let prior = totp.generate(now() - 30).to_string();
    f.core.mfa_confirm(&alice, &prior).unwrap();
    let enrolled = user(&f, "alice");
    assert_eq!(enrolled.id, previous.id);
    assert!(enrolled.epoch > previous.epoch);
    assert_eq!(enrolled.totp_secret.as_deref(), Some(secret.as_str()));
    assert!(enrolled.totp_pending.is_none());
    assert_eq!(audit_count(&f, "mfa.enabled"), 1);
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(prior))
            .is_err()
    );
    let current = totp.generate(now()).to_string();
    let fresh = text(
        &f.core
            .login("alice".into(), PASSWORD.into(), Some(current.clone()))
            .unwrap(),
        "session_token",
    );
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(current))
            .is_err()
    );
    let before_wrong_session = f.snapshot().unwrap();
    assert!(f.core.recovery_codes(&alice).is_err());
    assert!(f.core.recovery_codes(&bob).is_err());
    f.assert_snapshot(&before_wrong_session);
    let codes = f.core.recovery_codes(&fresh).unwrap();
    let first = codes["recovery_codes"][0].as_str().unwrap().to_owned();
    let second = codes["recovery_codes"][1].as_str().unwrap().to_owned();
    assert!(user(&f, "alice").recovery_codes.contains(&digest(&first)));
    assert!(
        !serde_json::to_string(&f.snapshot().unwrap())
            .unwrap()
            .contains(&first)
    );
    assert!(
        f.core
            .login(
                "alice".into(),
                "incorrect-password".into(),
                Some(first.clone())
            )
            .is_err()
    );
    assert!(user(&f, "alice").recovery_codes.contains(&digest(&first)));
    let before_sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    assert_eq!(
        f.core
            .login_for(
                "alice".into(),
                PASSWORD.into(),
                Some(first.clone()),
                Some("ri_auth_nonexistent".into())
            )
            .unwrap_err()
            .code,
        "invalid_request"
    );
    assert!(!user(&f, "alice").recovery_codes.contains(&digest(&first)));
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        before_sessions
    );
    assert_eq!(audit_count(&f, "login.transaction_rejected"), 1);
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(first.clone()))
            .is_err()
    );
    let recovered = f
        .core
        .login("alice".into(), PASSWORD.into(), Some(second.clone()))
        .unwrap();
    assert_eq!(
        f.core.me(&text(&recovered, "session_token")).unwrap()["mfa"],
        true
    );
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(second.clone()))
            .is_err()
    );
    assert!(f.core.me(&bob).is_ok());
    let public_audit = serde_json::to_string(&audit(&f)).unwrap();
    assert!(!public_audit.contains(&secret));
    assert!(!public_audit.contains(&first));
    assert!(!public_audit.contains(&second));
    assert_eq!(audit_count(&f, "mfa.recovery_codes.rotate"), 1);
}

fn pending_passkey_authentication(f: &Fixture, challenge: &Value) -> Option<Value> {
    f.core
        .store
        .get(
            "passkey_authentication",
            &digest(&text(challenge, "ceremony")),
        )
        .unwrap()
}

// RI-CRED-001/002, RI-STORE-001, Q02-C02: registration belongs to one live
// session, and authentication is bound to origin, transaction and live counter.
pub fn passkey_ceremony_binding_and_replay(backend: Backend) {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

    let f = backend.fixture();
    f.client("app", false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    let origin = url::Url::parse(&f.core.config.issuer).unwrap();
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let registration = f
        .core
        .passkey_register_start(&alice, "Contract passkey".into())
        .unwrap();
    let response = authenticator
        .do_registration(
            origin.clone(),
            serde_json::from_value(registration["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let other_alice = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let before_wrong_session = f.snapshot().unwrap();
    assert!(
        f.core
            .passkey_register_finish(&bob, &text(&registration, "ceremony"), response.clone())
            .is_err()
    );
    assert!(
        f.core
            .passkey_register_finish(
                &other_alice,
                &text(&registration, "ceremony"),
                response.clone()
            )
            .is_err()
    );
    f.assert_snapshot(&before_wrong_session);
    let enrolled = f
        .core
        .passkey_register_finish(&alice, &text(&registration, "ceremony"), response.clone())
        .unwrap();
    let passkey_id = text(&enrolled["passkey"], "id");
    let enrolled_state = f.snapshot().unwrap();
    assert!(
        f.core
            .passkey_register_finish(&alice, &text(&registration, "ceremony"), response)
            .is_err()
    );
    f.assert_snapshot(&enrolled_state);
    assert!(user(&f, "alice").has_passkeys);
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&other_alice).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert_eq!(audit_count(&f, "passkey.enroll"), 1);

    let transaction = f
        .core
        .authorization_prepare(Some(&bob), f.request("app", &crypto::random_token("")))
        .unwrap();
    let transaction_id = text(&transaction, "transaction_id");
    let bound = f
        .core
        .passkey_login_start("alice", Some(transaction_id.clone()))
        .unwrap();
    let bound_proof = authenticator
        .do_authentication(
            origin.clone(),
            serde_json::from_value(bound["public_key"].clone()).unwrap(),
        )
        .unwrap();
    assert!(pending_passkey_authentication(&f, &bound).is_some());
    let before_sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    assert!(
        f.core
            .passkey_login_finish(&text(&bound, "ceremony"), bound_proof)
            .is_err()
    );
    assert!(pending_passkey_authentication(&f, &bound).is_none());
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        before_sessions
    );
    let transaction_record: Value = f
        .core
        .store
        .get("authentication", &digest(&transaction_id))
        .unwrap()
        .unwrap();
    assert!(transaction_record["authenticated_session"].is_null());
    assert_eq!(audit_count(&f, "passkey.login_failed"), 1);

    let wrong_origin = f.core.passkey_login_start("alice", None).unwrap();
    let wrong_proof = authenticator
        .do_authentication(
            url::Url::parse("http://localhost:9001").unwrap(),
            serde_json::from_value(wrong_origin["public_key"].clone()).unwrap(),
        )
        .unwrap();
    assert!(pending_passkey_authentication(&f, &wrong_origin).is_some());
    assert!(
        f.core
            .passkey_login_finish(&text(&wrong_origin, "ceremony"), wrong_proof.clone())
            .is_err()
    );
    assert!(pending_passkey_authentication(&f, &wrong_origin).is_none());
    let after_wrong_origin = f.snapshot().unwrap();
    assert!(
        f.core
            .passkey_login_finish(&text(&wrong_origin, "ceremony"), wrong_proof)
            .is_err()
    );
    f.assert_snapshot(&after_wrong_origin);
    assert_eq!(audit_count(&f, "passkey.login_failed"), 2);

    let older = f.core.passkey_login_start("alice", None).unwrap();
    let newer = f.core.passkey_login_start("alice", None).unwrap();
    let older_proof = authenticator
        .do_authentication(
            origin.clone(),
            serde_json::from_value(older["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let newer_proof = authenticator
        .do_authentication(
            origin.clone(),
            serde_json::from_value(newer["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let signed_in = f
        .core
        .passkey_login_finish(&text(&newer, "ceremony"), newer_proof.clone())
        .unwrap();
    let passkey_session = text(&signed_in, "session_token");
    assert_eq!(f.core.me(&passkey_session).unwrap()["mfa"], true);
    assert!(pending_passkey_authentication(&f, &older).is_some());
    assert!(
        f.core
            .passkey_login_finish(&text(&older, "ceremony"), older_proof)
            .is_err()
    );
    assert!(pending_passkey_authentication(&f, &older).is_none());
    let after_stale_counter = f.snapshot().unwrap();
    assert!(
        f.core
            .passkey_login_finish(&text(&newer, "ceremony"), newer_proof)
            .is_err()
    );
    f.assert_snapshot(&after_stale_counter);
    assert_eq!(audit_count(&f, "passkey.login"), 1);
    assert_eq!(audit_count(&f, "passkey.login_failed"), 3);

    let before_remove = f.core.passkey_login_start("alice", None).unwrap();
    let before_remove_proof = authenticator
        .do_authentication(
            origin,
            serde_json::from_value(before_remove["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let before_wrong_remove = f.snapshot().unwrap();
    assert!(f.core.passkey_remove(&bob, &passkey_id).is_err());
    assert!(f.core.passkey_remove(&alice, &passkey_id).is_err());
    f.assert_snapshot(&before_wrong_remove);
    f.core
        .passkey_remove(&passkey_session, &passkey_id)
        .unwrap();
    assert!(f.core.me(&passkey_session).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(!user(&f, "alice").has_passkeys);
    assert_eq!(audit_count(&f, "passkey.remove"), 1);
    assert!(
        f.core
            .passkey_login_finish(&text(&before_remove, "ceremony"), before_remove_proof)
            .is_err()
    );
    assert_eq!(audit_count(&f, "passkey.login_failed"), 4);
}

// RI-CRED-003, RI-SES-004, RI-STORE-001, Q02-C02/C03: reset proofs are
// purpose/email-bound and single-use; a failed completion changes no state.
pub fn account_reset_binding_and_atomicity(backend: Backend) {
    let mut f = backend.fixture();
    configure_mail(&mut f);
    f.client("app", false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    f.user("unverified");
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let pending = f.core.mfa_begin(&alice).unwrap();
    let totp = crypto::totp(&text(&pending, "secret"), "alice").unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate(now() - 30).to_string())
        .unwrap();
    let fresh = text(
        &f.core
            .login(
                "alice".into(),
                PASSWORD.into(),
                Some(totp.generate(now()).to_string()),
            )
            .unwrap(),
        "session_token",
    );
    let recovery = f.core.recovery_codes(&fresh).unwrap()["recovery_codes"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let before = user(&f, "alice");
    let access = f.tokens("app", &fresh, None);
    let delivery_count = f.core.store.list::<Value>("mail_deliveries").unwrap().len();
    assert_eq!(
        f.core.account_reset_request("absent").unwrap()["accepted"],
        true
    );
    assert_eq!(
        f.core.account_reset_request("unverified").unwrap()["accepted"],
        true
    );
    assert_eq!(
        f.core.store.list::<Value>("mail_deliveries").unwrap().len(),
        delivery_count
    );

    f.core
        .update_user(
            &f.admin,
            "bob",
            UserPatch {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    f.core.account_reset_request("bob").unwrap();
    let bob_code = mail_code_for(&f, "bob");
    f.core
        .update_user(
            &f.admin,
            "bob",
            UserPatch {
                email: Some("bob-changed@example.test".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let before_email_rejection = f.snapshot().unwrap();
    assert!(
        f.core
            .account_complete(
                bob_code,
                Purpose::Reset,
                Some("new-password-for-bob".into())
            )
            .is_err()
    );
    f.assert_snapshot(&before_email_rejection);
    assert!(f.core.me(&bob).is_ok());

    f.core.account_reset_request("alice").unwrap();
    let code = mail_code_for(&f, "alice");
    let before_invalid = f.snapshot().unwrap();
    assert!(
        f.core
            .account_complete(code.clone(), Purpose::Verify, None)
            .is_err()
    );
    assert!(
        f.core
            .account_complete(code.clone(), Purpose::Reset, Some(PASSWORD.into()))
            .is_err()
    );
    f.assert_snapshot(&before_invalid);
    let replacement = "new-password-from-account-proof";
    assert_eq!(
        f.core
            .account_complete(code.clone(), Purpose::Reset, Some(replacement.into()))
            .unwrap()["completed"],
        true
    );
    assert_eq!(audit_count(&f, "user.account.reset"), 1);
    let after_completion = f.snapshot().unwrap();
    assert!(
        f.core
            .account_complete(code.clone(), Purpose::Reset, Some(replacement.into()))
            .is_err()
    );
    f.assert_snapshot(&after_completion);
    let reset = user(&f, "alice");
    assert_eq!(reset.id, before.id);
    assert!(reset.epoch > before.epoch);
    assert_eq!(reset.totp_secret, before.totp_secret);
    assert_eq!(reset.recovery_codes, before.recovery_codes);
    assert!(f.core.me(&fresh).is_err());
    assert!(f.core.userinfo(&text(&access, "access_token")).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(f.core.login("alice".into(), PASSWORD.into(), None).is_err());
    assert!(
        f.core
            .login("alice".into(), replacement.into(), None)
            .is_err()
    );
    let recovered = f
        .core
        .login("alice".into(), replacement.into(), Some(recovery.clone()))
        .unwrap();
    assert_eq!(
        f.core.me(&text(&recovered, "session_token")).unwrap()["mfa"],
        true
    );
    assert!(
        f.core
            .login("alice".into(), replacement.into(), Some(recovery.clone()))
            .is_err()
    );
    let published = json!({
        "audit": audit(&f),
        "deliveries": f.core.mail_deliveries(&f.admin).unwrap(),
    })
    .to_string();
    assert!(!published.contains(&code));
    assert!(!published.contains(&recovery));
    assert!(!published.contains(replacement));
}

// RI-CRED-003, RI-SES-003/005, Q02-C02/C03: a newer reset replaces the
// earlier proof, while deadline rejection cannot change credential state.
#[cfg(feature = "test-support")]
pub fn account_proof_supersession_and_expiry(backend: Backend) {
    let start = now();
    crypto::with_test_time(start, || {
        let mut f = backend.fixture();
        configure_mail(&mut f);
        f.user("alice");
        let bob = f.user("bob");
        f.core
            .update_user(
                &f.admin,
                "alice",
                UserPatch {
                    email_verified: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        let alice = text(
            &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        );
        let original = user(&f, "alice");
        f.core.account_reset_request("alice").unwrap();
        let first = mail_code_for(&f, "alice");
        let before_throttled = f.snapshot().unwrap();
        assert_eq!(
            f.core.account_reset_request("alice").unwrap()["accepted"],
            true
        );
        f.assert_snapshot(&before_throttled);

        crypto::set_test_time(start + 61);
        f.core.account_reset_request("alice").unwrap();
        let codes = mail_codes_for(&f, "alice");
        assert_eq!(codes.len(), 2);
        let second = codes.into_iter().find(|code| code != &first).unwrap();
        assert!(
            f.core
                .store
                .get::<Value>("account_proofs", &digest(&first))
                .unwrap()
                .is_none()
        );
        assert!(
            f.core
                .store
                .get::<Value>("account_proofs", &digest(&second))
                .unwrap()
                .is_some()
        );
        assert_eq!(
            f.core
                .store
                .get::<String>("account_latest", &format!("{}:reset", original.id))
                .unwrap(),
            Some(digest(&second))
        );
        let before_old = f.snapshot().unwrap();
        assert!(
            f.core
                .account_complete(
                    first.clone(),
                    Purpose::Reset,
                    Some("superseded-password".into())
                )
                .is_err()
        );
        f.assert_snapshot(&before_old);
        assert_eq!(user(&f, "alice").epoch, original.epoch);
        assert!(f.core.me(&alice).is_ok());
        assert!(f.core.me(&bob).is_ok());

        let replacement = "second-reset-proof-password";
        assert_eq!(
            f.core
                .account_complete(second.clone(), Purpose::Reset, Some(replacement.into()))
                .unwrap()["completed"],
            true
        );
        assert_eq!(audit_count(&f, "user.account.reset"), 1);
        assert!(f.core.me(&alice).is_err());
        assert!(f.core.me(&bob).is_ok());
        assert!(user(&f, "alice").epoch > original.epoch);

        crypto::set_test_time(start + 122);
        f.core.account_reset_request("alice").unwrap();
        let third = mail_codes_for(&f, "alice")
            .into_iter()
            .find(|code| code != &first && code != &second)
            .unwrap();
        let expiry = f
            .core
            .store
            .get::<Value>("account_proofs", &digest(&third))
            .unwrap()
            .unwrap()["expires_at"]
            .as_u64()
            .unwrap();
        let after_reset = user(&f, "alice");
        let before_expiry = f.snapshot().unwrap();
        crypto::set_test_time(expiry);
        assert!(
            f.core
                .account_complete(third, Purpose::Reset, Some("expired-password".into()))
                .is_err()
        );
        f.assert_snapshot(&before_expiry);
        assert_eq!(user(&f, "alice").epoch, after_reset.epoch);
        assert!(user(&f, "alice").password_hash == after_reset.password_hash);
        assert_eq!(audit_count(&f, "user.account.reset"), 1);
        assert!(f.core.me(&bob).is_ok());
    });
}

// RI-CRED-003, RI-CRED-002, Q02-C02: a verification request needs a recent
// account session, and its proof affects only that account and current email.
#[cfg(feature = "test-support")]
pub fn verification_proof_binding_and_replay(backend: Backend) {
    let start = now();
    crypto::with_test_time(start, || {
        let mut f = backend.fixture();
        configure_mail(&mut f);
        let alice = f.user("alice");
        let bob = f.user("bob");
        crypto::set_test_time(start + 301);
        let before_stale = f.snapshot().unwrap();
        assert_eq!(
            f.core.account_verify_request(&alice).unwrap_err().code,
            "access_denied"
        );
        f.assert_snapshot(&before_stale);
        let fresh = text(
            &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        );
        f.core.account_verify_request(&fresh).unwrap();
        let first = mail_code_for(&f, "alice");
        let alice_id = user(&f, "alice").id;
        assert_eq!(
            f.core
                .store
                .get::<Value>("account_proofs", &digest(&first))
                .unwrap()
                .unwrap()["user_id"],
            alice_id
        );
        let before_wrong_purpose = f.snapshot().unwrap();
        assert!(
            f.core
                .account_complete(first.clone(), Purpose::Reset, Some("wrong-purpose".into()))
                .is_err()
        );
        f.assert_snapshot(&before_wrong_purpose);

        f.core
            .update_user(
                &f.admin,
                "alice",
                UserPatch {
                    email: Some("alice-new@example.test".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        let before_old = f.snapshot().unwrap();
        assert!(
            f.core
                .account_complete(first.clone(), Purpose::Verify, None)
                .is_err()
        );
        f.assert_snapshot(&before_old);
        assert!(!user(&f, "alice").email_verified);
        assert!(!user(&f, "bob").email_verified);
        assert!(f.core.me(&bob).is_ok());

        crypto::set_test_time(start + 362);
        let fresh = text(
            &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        );
        f.core.account_verify_request(&fresh).unwrap();
        let codes = mail_codes_for(&f, "alice");
        assert_eq!(codes.len(), 2);
        let current = codes.into_iter().find(|code| code != &first).unwrap();
        let current_proof: Value = f
            .core
            .store
            .get("account_proofs", &digest(&current))
            .unwrap()
            .unwrap();
        assert_eq!(current_proof["user_id"], alice_id);
        assert_eq!(current_proof["email"], "alice-new@example.test");
        assert_eq!(
            f.core
                .account_complete(current.clone(), Purpose::Verify, None)
                .unwrap()["completed"],
            true
        );
        let verified = user(&f, "alice");
        assert_eq!(verified.id, alice_id);
        assert!(verified.email_verified);
        assert!(!user(&f, "bob").email_verified);
        assert_eq!(audit_count(&f, "user.account.verify"), 1);
        let after_completion = f.snapshot().unwrap();
        assert!(
            f.core
                .account_complete(current, Purpose::Verify, None)
                .is_err()
        );
        f.assert_snapshot(&after_completion);
        assert!(f.core.me(&bob).is_ok());
    });
}

/// Real initial-credential proofs cross the workflow completion writer once.
#[cfg(feature = "platform")]
pub fn invitation_passkey_bound_competing_completion(backend: Backend) {
    use std::sync::Barrier;
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

    let mut f = backend.fixture();
    configure_mail(&mut f);
    for name in ["invited", "other"] {
        f.core
            .account_invite(
                &f.admin,
                Invitation {
                    username: name.into(),
                    email: format!("{name}@example.test"),
                    display_name: name.into(),
                    groups: BTreeSet::new(),
                },
            )
            .unwrap();
    }
    let code = mail_code_for(&f, "invited");
    let other = mail_code_for(&f, "other");
    let before = user(&f, "invited");
    let sessions = f.core.store.list::<Value>("sessions").unwrap();
    let start = |token: &str| {
        f.core
            .account_invitation_passkey_start(token.into(), "First passkey".into())
            .unwrap()
    };
    let response = |challenge: &Value| {
        let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
        // This fixture has no resident storage. Only the client's resident-key
        // preference is relaxed; the server still requires user verification.
        let mut options = challenge["public_key"].clone();
        options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
        authenticator
            .do_registration(
                "http://localhost:9000".parse().unwrap(),
                serde_json::from_value(options).unwrap(),
            )
            .unwrap()
    };
    let replaced = start(&code);
    let replaced_proof = response(&replaced);
    let other_challenge = start(&other);
    let challenge = start(&code);
    let ceremony = text(&challenge, "ceremony");
    let proof = response(&challenge);
    let replaced_ceremony = text(&replaced, "ceremony");
    let other_ceremony = text(&other_challenge, "ceremony");

    // Neither another account nor a previous request for this same account can
    // consume the current invitation, registration, or any workflow evidence.
    let pending = f.snapshot().unwrap();
    for (token, request, credential) in [
        (&other, &ceremony, &proof),
        (&code, &other_ceremony, &proof),
        (&code, &replaced_ceremony, &replaced_proof),
        (&f.admin, &ceremony, &proof),
    ] {
        assert!(
            f.core
                .account_invitation_passkey_finish(token.clone(), request, credential.clone())
                .is_err()
        );
        f.assert_snapshot(&pending);
    }
    assert!(
        f.core
            .passkey_register_finish(&f.admin, &ceremony, proof.clone())
            .is_err()
    );
    f.assert_snapshot(&pending);

    // Keep the disposable database's administrative connection on this thread;
    // competing callers share only Core and its real two-connection PG pool.
    let core = &f.core;
    let finish = || core.account_invitation_passkey_finish(code.clone(), &ceremony, proof.clone());
    let gate = Barrier::new(2);
    let results = std::thread::scope(|scope| {
        let complete = || {
            gate.wait();
            finish()
        };
        let a = scope.spawn(complete);
        let b = scope.spawn(complete);
        [a.join().unwrap(), b.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    for result in results {
        match result {
            Ok(value) => assert_eq!(value, json!({"completed":true,"login_required":true})),
            Err(error) => assert_eq!(error.code, "account_code_used"),
        }
    }

    let after = user(&f, "invited");
    assert!(after.enabled && after.email_verified && after.has_passkeys && !after.admin);
    assert!(after.password_hash.is_empty());
    assert_eq!(after.id, before.id);
    assert_eq!(after.epoch, before.epoch + 1);
    assert_eq!(f.core.store.list::<Value>("sessions").unwrap(), sessions);
    assert_eq!(audit_count(&f, "user.account.accept"), 1);
    assert_eq!(audit_count(&f, "passkey.enroll"), 1);
    for (bucket, key) in [
        ("invitation_passkey_registration", digest(&code)),
        ("account_proofs", digest(&code)),
        ("account_latest", format!("{}:accept", before.id)),
        ("invitation_reservations", before.id.clone()),
    ] {
        assert!(f.core.store.get::<Value>(bucket, &key).unwrap().is_none());
    }
    assert_eq!(
        f.core
            .store
            .get::<Value>("account_proof_outcomes", &digest(&code))
            .unwrap()
            .unwrap()["reason"],
        "used"
    );
    assert!(!user(&f, "other").enabled);
    assert!(
        f.core
            .store
            .get::<Value>("account_proofs", &digest(&other))
            .unwrap()
            .is_some()
    );
    let keys = f.core.store.list::<Value>("passkeys").unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].1["user_id"], before.id);
    let runs = f.core.store.list::<Value>("workflow_runs").unwrap();
    assert_eq!(runs.len(), 1);
    let run = &runs[0].1;
    let request = format!("accept:{}:{}", digest(&code), digest(&ceremony));
    assert_eq!(
        run["record"]["binding"]["workflow"],
        "essentials-invitation"
    );
    assert_eq!(run["record"]["account"], before.id);
    assert_eq!(run["record"]["account_epoch"], before.epoch);
    assert!(run["record"]["session"].is_null());
    assert_eq!(run["record"]["request"], request);
    assert_eq!(run["record"]["state"]["outcome"], "enrolled");
    assert_eq!(run["credential_mutation"]["from_epoch"], before.epoch);
    assert_eq!(run["credential_mutation"]["to_epoch"], after.epoch);
    assert_eq!(run["credential_mutation"]["credential"], keys[0].0);
    assert_eq!(run["credential_mutation"]["invitation_request"], request);
    let receipts = f.core.store.list::<Value>("workflow_evidence").unwrap();
    assert_eq!(receipts.len(), 2);
    assert_eq!(
        receipts
            .iter()
            .map(|(_, receipt)| text(receipt, "proof"))
            .collect::<BTreeSet<_>>(),
        strings(&["invitation", "enrolled"])
    );
    for (_, receipt) in receipts {
        assert_eq!(receipt["consumed"], true);
        assert_eq!(receipt["account"], before.id);
        assert_eq!(receipt["account_epoch"], before.epoch);
        assert_eq!(receipt["request"], request);
        assert_eq!(receipt["run"], runs[0].0);
        assert_eq!(receipt["binding"], run["record"]["binding"]);
        assert!(receipt["session"].is_null());
    }
    let completed = f.snapshot().unwrap();
    assert_eq!(finish().unwrap_err().code, "account_code_used");
    assert_eq!(
        f.core
            .account_complete(code.clone(), Purpose::Invite, Some(PASSWORD.into()))
            .unwrap_err()
            .code,
        "account_code_used"
    );
    f.assert_snapshot(&completed);
}

// RI-CRED-003, RI-MGT-004, RI-STORE-001, Q02-C02/C04: invitation completion
// rechecks the creator's live authority before it changes a user or group.
pub fn invitation_acceptance_revalidates_creator(backend: Backend) {
    let mut f = backend.fixture();
    configure_mail(&mut f);
    f.core.create_group(&f.admin, "invited").unwrap();
    let bob = f.user("bob");
    let inviter = agent(
        &f,
        "inviter",
        &[
            ("user.write", "user/pending"),
            ("group.members", "group/invited"),
        ],
    );
    let pending = Invitation {
        username: "pending".into(),
        email: "pending@example.test".into(),
        display_name: "Pending account".into(),
        groups: strings(&["invited"]),
    };
    f.core.account_invite(&inviter, pending.clone()).unwrap();
    let code = mail_code_for(&f, "pending");
    let pending_user = user(&f, "pending");
    assert!(!pending_user.enabled);
    assert!(pending_user.password_hash.is_empty());
    let renewed = f.core.account_invite(&inviter, pending).unwrap();
    assert_eq!(renewed["user"]["id"], pending_user.id);
    assert_eq!(audit_count(&f, "user.invitation.reissue"), 1);
    let renewed_code = mail_codes_for(&f, "pending")
        .into_iter()
        .find(|renewed| renewed != &code)
        .unwrap();
    let after_reissue = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .account_complete(code.clone(), Purpose::Invite, Some(PASSWORD.into()))
            .unwrap_err()
            .code,
        "account_code_replaced"
    );
    assert!(
        f.core
            .account_complete(renewed_code.clone(), Purpose::Verify, None)
            .is_err()
    );
    f.assert_snapshot(&after_reissue);

    f.core.revoke_agent(&f.admin, "inviter").unwrap();
    let before_rejected_acceptance = f.snapshot().unwrap();
    assert!(
        f.core
            .account_complete(renewed_code.clone(), Purpose::Invite, Some(PASSWORD.into()))
            .is_err()
    );
    f.assert_snapshot(&before_rejected_acceptance);
    assert!(!user(&f, "pending").enabled);
    assert!(
        !f.core
            .store
            .get::<Group>("groups", "invited")
            .unwrap()
            .unwrap()
            .members
            .contains(&pending_user.id)
    );
    assert_eq!(audit_count(&f, "user.account.accept"), 0);
    assert!(f.core.me(&bob).is_ok());

    f.core
        .account_invite(
            &f.admin,
            Invitation {
                username: "accepted".into(),
                email: "accepted@example.test".into(),
                display_name: "Accepted account".into(),
                groups: strings(&["invited"]),
            },
        )
        .unwrap();
    let accepted_code = mail_code_for(&f, "accepted");
    let accepted_id = user(&f, "accepted").id;
    assert_eq!(
        f.core
            .account_complete(
                accepted_code.clone(),
                Purpose::Invite,
                Some(PASSWORD.into())
            )
            .unwrap()["completed"],
        true
    );
    let accepted = user(&f, "accepted");
    assert_eq!(accepted.id, accepted_id);
    assert!(accepted.enabled && accepted.email_verified);
    let members = &f
        .core
        .store
        .get::<Group>("groups", "invited")
        .unwrap()
        .unwrap()
        .members;
    assert!(members.contains(&accepted_id));
    assert!(!members.contains(&pending_user.id));
    assert_eq!(audit_count(&f, "user.account.accept"), 1);
    let after_acceptance = f.snapshot().unwrap();
    assert!(
        f.core
            .account_complete(accepted_code, Purpose::Invite, Some(PASSWORD.into()))
            .is_err()
    );
    assert!(
        f.core
            .account_complete(code, Purpose::Invite, Some(PASSWORD.into()))
            .is_err()
    );
    f.assert_snapshot(&after_acceptance);
    assert!(
        f.core
            .login("accepted".into(), PASSWORD.into(), None)
            .is_ok()
    );
    assert!(f.core.me(&bob).is_ok());
}

// M03 user writer: direct mutations and invitation management share scoped
// authority and commit their dependent credential, group and audit effects.
pub fn user_writers_share_management_seam(backend: Backend) {
    let mut f = backend.fixture();
    configure_mail(&mut f);
    f.core.create_group(&f.admin, "team").unwrap();
    let writer = agent(
        &f,
        "user-writer",
        &[
            ("user.write", "user/managed"),
            ("user.write", "user/pending"),
            ("group.members", "group/team"),
            ("group.members", "group/missing"),
        ],
    );
    let new_user = |admin| NewUser {
        username: "managed".into(),
        password: PASSWORD.into(),
        email: Some("managed@example.test".into()),
        display_name: "Managed".into(),
        admin,
    };
    let before = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .create_user(&writer, new_user(true))
            .unwrap_err()
            .status,
        StatusCode::FORBIDDEN
    );
    f.assert_snapshot(&before);
    let created = f.core.create_user(&writer, new_user(false)).unwrap();
    let managed_id = text(&created, "id");
    let login = text(
        &f.core
            .login("managed".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let initial_epoch = user(&f, "managed").epoch;
    f.core
        .update_user(
            &writer,
            "managed",
            UserPatch {
                display_name: Some("Managed account".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(user(&f, "managed").display_name, "Managed account");
    assert!(f.core.me(&login).is_ok());
    f.core
        .update_user(
            &writer,
            "managed",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(!user(&f, "managed").enabled);
    assert!(user(&f, "managed").epoch > initial_epoch);
    assert!(f.core.me(&login).is_err());
    assert_eq!(audit_count(&f, "user.create"), 1);
    assert_eq!(audit_count(&f, "user.update"), 2);
    assert!(
        audit(&f)
            .iter()
            .any(|event| { event["action"] == "user.create" && event["target"] == managed_id })
    );

    let invitation = |username: &str, group: &str| Invitation {
        username: username.into(),
        email: format!("{username}@example.test"),
        display_name: "Pending account".into(),
        groups: strings(&[group]),
    };
    let before = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .account_invite(&writer, invitation("outside", "team"))
            .unwrap_err()
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.core
            .account_invite(&writer, invitation("pending", "missing"))
            .unwrap_err()
            .status,
        StatusCode::NOT_FOUND
    );
    f.assert_snapshot(&before);

    let pending = invitation("pending", "team");
    let first = f.core.account_invite(&writer, pending.clone()).unwrap();
    let pending_id = text(&first["user"], "id");
    let first_code = mail_code_for(&f, "pending");
    assert!(!user(&f, "pending").enabled);
    let reissued = f.core.account_invite(&writer, pending.clone()).unwrap();
    assert_eq!(reissued["user"]["id"], pending_id);
    let second_code = mail_codes_for(&f, "pending")
        .into_iter()
        .find(|code| code != &first_code)
        .unwrap();
    assert_eq!(
        f.core
            .account_complete(first_code.clone(), Purpose::Invite, Some(PASSWORD.into()))
            .unwrap_err()
            .code,
        "account_code_replaced"
    );
    f.core
        .account_invitation_revoke(&writer, "pending")
        .unwrap();
    assert_eq!(
        f.core
            .account_complete(second_code.clone(), Purpose::Invite, Some(PASSWORD.into()))
            .unwrap_err()
            .code,
        "account_code_revoked"
    );
    let renewed = f.core.account_invite(&writer, pending).unwrap();
    assert_eq!(renewed["user"]["id"], pending_id);
    let final_code = mail_codes_for(&f, "pending")
        .into_iter()
        .find(|code| code != &second_code && code != &first_code)
        .unwrap();
    assert_eq!(
        f.core
            .account_complete(final_code, Purpose::Invite, Some(PASSWORD.into()))
            .unwrap()["completed"],
        true
    );
    assert!(user(&f, "pending").enabled);
    assert!(
        f.core
            .store
            .get::<Group>("groups", "team")
            .unwrap()
            .unwrap()
            .members
            .contains(&pending_id)
    );
    assert_eq!(audit_count(&f, "user.invite"), 1);
    assert_eq!(audit_count(&f, "user.invitation.reissue"), 2);
    assert_eq!(audit_count(&f, "user.invitation.revoke"), 1);
    assert_eq!(audit_count(&f, "user.account.accept"), 1);
}

// A desired-state user write keeps its plan binding while reaching the same
// authority, revocation and audit boundary as direct management writes.
pub fn desired_state_user_writes_share_management_seam(backend: Backend) {
    let f = backend.fixture();
    let session = f.user("managed");
    let original_user = user(&f, "managed");
    let replacement_password = "m03-managed-password-v2-2026";
    let writer = agent(&f, "state-user-writer", &[("user.write", "user/managed")]);
    let manifest: Manifest = serde_json::from_value(json!({
        "api_version": "riauth/v1",
        "users": [{
            "username": "managed",
            "display_name": "Managed by plan",
            "email": "managed@example.test",
            "enabled": false,
            "password_ref": "env:M03_MANAGED_PASSWORD",
            "password_version": "v2"
        }]
    }))
    .unwrap();
    let pending = f.core.plan_state(&writer, manifest.clone()).unwrap();
    assert_eq!(pending.changes.len(), 1);
    assert_eq!(pending.changes[0].resource, "user/managed");
    assert!(pending.changes[0].credential_change);
    assert_eq!(audit_count(&f, "user.reconcile"), 0);

    let original_agent: Agent = f
        .core
        .store
        .get("agents", "state-user-writer")
        .unwrap()
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut revoked = original_agent.clone();
            revoked.permissions.clear();
            tx.put("agents", "state-user-writer", &revoked)
        })
        .unwrap();
    let denied_state = f.snapshot().unwrap();
    let request = |plan| ApplyRequest {
        plan,
        secrets: BTreeMap::from([(
            "env:M03_MANAGED_PASSWORD".into(),
            replacement_password.into(),
        )]),
        run_id: Some("m03-state-user".into()),
    };
    assert!(f.core.apply_state(&writer, request(pending)).is_err());
    assert!(f.core.plan_state(&writer, manifest.clone()).is_err());
    f.assert_snapshot(&denied_state);

    f.core
        .store
        .write(|tx| tx.put("agents", "state-user-writer", &original_agent))
        .unwrap();
    let plan = f.core.plan_state(&writer, manifest).unwrap();
    let applied = f.core.apply_state(&writer, request(plan.clone())).unwrap();
    assert_eq!(applied["changed"], true);
    let managed = user(&f, "managed");
    assert_eq!(managed.display_name, "Managed by plan");
    assert!(!managed.enabled);
    assert!(managed.epoch > original_user.epoch);
    assert_ne!(managed.password_hash, original_user.password_hash);
    assert!(crypto::password_matches(
        replacement_password,
        &managed.password_hash
    ));
    assert_eq!(
        f.core
            .store
            .get::<String>("credential_versions", "user/managed")
            .unwrap()
            .as_deref(),
        Some("v2")
    );
    assert!(f.core.me(&session).is_err());

    let events = audit(&f);
    let reconciled: Vec<_> = events
        .iter()
        .filter(|event| event["action"] == "user.reconcile")
        .collect();
    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0]["actor"], "agent:state-user-writer");
    assert_eq!(reconciled[0]["target"], "user/managed");
    assert_eq!(audit_count(&f, "user.update"), 0);
    let state_applies: Vec<_> = events
        .iter()
        .filter(|event| event["action"] == "state.apply")
        .collect();
    assert_eq!(state_applies.len(), 1);
    assert_eq!(state_applies[0]["target"], plan.plan_id);
    assert_eq!(state_applies[0]["run_id"], "m03-state-user");
    assert!(
        !serde_json::to_string(&plan)
            .unwrap()
            .contains(replacement_password)
    );
    assert!(
        !serde_json::to_string(&events)
            .unwrap()
            .contains(replacement_password)
    );

    let committed = f.snapshot().unwrap();
    assert_eq!(f.core.apply_state(&writer, request(plan)).unwrap(), applied);
    f.assert_snapshot(&committed);
}

// RI-CRED-001/002, Q02-C02: a registration response lacking user verification
// consumes only its ceremony and cannot enroll or revoke account authority.
pub fn passkey_registration_requires_user_verification(backend: Backend) {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

    let f = backend.fixture();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let before_user = user(&f, "alice");
    let origin = url::Url::parse(&f.core.config.issuer).unwrap();
    let mut weak = WebauthnAuthenticator::new(SoftPasskey::new(false));
    let registration = f
        .core
        .passkey_register_start(&alice, "Unverified key".into())
        .unwrap();
    let mut options = registration["public_key"].clone();
    options["publicKey"]["authenticatorSelection"]["userVerification"] = json!("discouraged");
    let response = weak
        .do_registration(origin.clone(), serde_json::from_value(options).unwrap())
        .unwrap();
    let ceremony = text(&registration, "ceremony");
    assert!(
        f.core
            .passkey_register_finish(&alice, &ceremony, response.clone())
            .is_err()
    );
    assert!(
        f.core
            .store
            .get::<Value>("passkey_registration", &digest(&ceremony))
            .unwrap()
            .is_none()
    );
    let after_rejection = f.snapshot().unwrap();
    assert!(
        f.core
            .passkey_register_finish(&alice, &ceremony, response)
            .is_err()
    );
    f.assert_snapshot(&after_rejection);
    let unchanged = user(&f, "alice");
    assert_eq!(unchanged.epoch, before_user.epoch);
    assert!(!unchanged.has_passkeys);
    assert!(unchanged.password_hash == before_user.password_hash);
    assert!(f.core.store.list::<Value>("passkeys").unwrap().is_empty());
    assert_eq!(audit_count(&f, "passkey.enroll"), 0);
    assert!(f.core.me(&alice).is_ok());
    assert!(f.core.me(&bob).is_ok());

    let mut strong = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let verified = f
        .core
        .passkey_register_start(&alice, "Verified key".into())
        .unwrap();
    let response = strong
        .do_registration(
            origin,
            serde_json::from_value(verified["public_key"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .passkey_register_finish(&alice, &text(&verified, "ceremony"), response)
        .unwrap();
    assert!(user(&f, "alice").has_passkeys);
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
    assert_eq!(audit_count(&f, "passkey.enroll"), 1);
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&bob).is_ok());
}

// RI-CRED-002, RI-SES-005, Q02-C02: existing-factor MFA and a fresh auth_time
// gate key management; expired challenges cannot create a new session.
#[cfg(feature = "test-support")]
pub fn passkey_management_requires_fresh_mfa(backend: Backend) {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

    let start = now();
    crypto::with_test_time(start, || {
        let f = backend.fixture();
        let alice = f.user("alice");
        let bob = f.user("bob");
        let origin = url::Url::parse(&f.core.config.issuer).unwrap();
        let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
        let registration = f
            .core
            .passkey_register_start(&alice, "Managed key".into())
            .unwrap();
        let response = authenticator
            .do_registration(
                origin.clone(),
                serde_json::from_value(registration["public_key"].clone()).unwrap(),
            )
            .unwrap();
        let enrolled = f
            .core
            .passkey_register_finish(&alice, &text(&registration, "ceremony"), response)
            .unwrap();
        let key_id = text(&enrolled["passkey"], "id");
        let password_only = text(
            &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        );
        let before_weak_session = f.snapshot().unwrap();
        assert_eq!(
            f.core
                .passkey_register_start(&password_only, "Second key".into())
                .unwrap_err()
                .code,
            "mfa_required"
        );
        assert_eq!(
            f.core
                .passkey_remove(&password_only, &key_id)
                .unwrap_err()
                .code,
            "mfa_required"
        );
        f.assert_snapshot(&before_weak_session);

        let first_login = f.core.passkey_login_start("alice", None).unwrap();
        let first_proof = authenticator
            .do_authentication(
                origin.clone(),
                serde_json::from_value(first_login["public_key"].clone()).unwrap(),
            )
            .unwrap();
        let mfa = text(
            &f.core
                .passkey_login_finish(&text(&first_login, "ceremony"), first_proof)
                .unwrap(),
            "session_token",
        );
        let expired_login = f.core.passkey_login_start("alice", None).unwrap();
        let expired_proof = authenticator
            .do_authentication(
                origin.clone(),
                serde_json::from_value(expired_login["public_key"].clone()).unwrap(),
            )
            .unwrap();
        crypto::set_test_time(start + 301);
        let before_stale = f.snapshot().unwrap();
        assert_eq!(
            f.core
                .passkey_register_start(&mfa, "Too late".into())
                .unwrap_err()
                .code,
            "reauthentication_required"
        );
        assert_eq!(
            f.core.passkey_remove(&mfa, &key_id).unwrap_err().code,
            "reauthentication_required"
        );
        assert!(
            f.core
                .passkey_login_finish(&text(&expired_login, "ceremony"), expired_proof)
                .is_err()
        );
        f.assert_snapshot(&before_stale);
        assert!(f.core.me(&mfa).is_ok());
        assert!(f.core.me(&bob).is_ok());
        assert_eq!(audit_count(&f, "passkey.remove"), 0);

        let second_login = f.core.passkey_login_start("alice", None).unwrap();
        let second_proof = authenticator
            .do_authentication(
                origin,
                serde_json::from_value(second_login["public_key"].clone()).unwrap(),
            )
            .unwrap();
        let fresh_mfa = text(
            &f.core
                .passkey_login_finish(&text(&second_login, "ceremony"), second_proof)
                .unwrap(),
            "session_token",
        );
        assert_eq!(
            f.core.passkey_remove(&fresh_mfa, &key_id).unwrap()["removed"],
            true
        );
        assert!(!user(&f, "alice").has_passkeys);
        assert!(f.core.me(&mfa).is_err());
        assert!(f.core.me(&fresh_mfa).is_err());
        assert!(f.core.me(&bob).is_ok());
        assert_eq!(audit_count(&f, "passkey.remove"), 1);
    });
}

fn offboard_job(f: &Fixture, id: &str) -> Job {
    f.core.store.get(offboarding::BUCKET, id).unwrap().unwrap()
}

fn offboard_schedule(f: &Fixture, token: &str, username: &str, execute_at: u64) -> Value {
    f.core
        .offboard_schedule(
            token,
            ScheduleRequest {
                username: username.into(),
                execute_at: ExecuteAt::Unix(execute_at),
                timezone: "UTC".into(),
            },
        )
        .unwrap()
}

// RI-CON-004, RI-MGT-004, RI-STORE-001, Q02-C06/C08: one scheduled intent
// survives reopening; duplicate and unauthorized requests cannot add jobs or
// audits, and cancellation has an exact idempotent retry result.
pub fn offboard_intent_durable_cancel(backend: Backend) {
    let f = backend.fixture();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let allowed = agent(&f, "scheduler", &[("user.offboard", "user/alice")]);
    let denied = agent(&f, "other-scheduler", &[("user.offboard", "user/bob")]);
    let due_at = now() + 3600;
    let scheduled = offboard_schedule(&f, &allowed, "alice", due_at);
    let id = text(&scheduled, "id");
    let alice_id = user(&f, "alice").id;
    assert_eq!(scheduled["user_id"], alice_id);
    assert_eq!(scheduled["created_by"], "agent:scheduler");
    assert_eq!(scheduled["status"], "scheduled");
    assert_eq!(
        f.core.store.list::<Job>(offboarding::BUCKET).unwrap().len(),
        1
    );
    assert_eq!(audit_count(&f, "offboard.schedule"), 1);
    assert!(audit(&f).iter().any(|event| {
        event["action"] == "offboard.schedule"
            && event["actor"] == "agent:scheduler"
            && event["target"] == format!("{id}/alice")
    }));
    let before_rejection = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .offboard_schedule(
                &allowed,
                ScheduleRequest {
                    username: "alice".into(),
                    execute_at: ExecuteAt::Unix(due_at),
                    timezone: "UTC".into(),
                }
            )
            .unwrap_err()
            .code,
        "conflict"
    );
    assert_eq!(
        f.core.offboard_cancel(&denied, &id).unwrap_err().code,
        "access_denied"
    );
    assert_eq!(
        f.core.offboard_get(&denied, &id).unwrap_err().code,
        "access_denied"
    );
    f.assert_snapshot(&before_rejection);
    assert!(f.core.me(&alice).is_ok());
    assert!(f.core.me(&bob).is_ok());

    let f = f.reopen_with(|_| {});
    f.assert_snapshot(&before_rejection);
    assert_eq!(f.core.offboard_get(&allowed, &id).unwrap(), scheduled);
    let cancelled = f.core.offboard_cancel(&allowed, &id).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(offboard_job(&f, &id).status, Status::Cancelled);
    assert_eq!(audit_count(&f, "offboard.cancel"), 1);
    let after_cancel = f.snapshot().unwrap();
    assert_eq!(f.core.offboard_cancel(&allowed, &id).unwrap(), cancelled);
    assert!(f.core.offboard_claim("worker").unwrap().is_none());
    f.assert_snapshot(&after_cancel);
    assert_eq!(
        f.core.store.list::<Job>(offboarding::BUCKET).unwrap().len(),
        1
    );
    assert_eq!(audit_count(&f, "offboard.schedule"), 1);
    assert_eq!(audit_count(&f, "offboard.cancel"), 1);
    assert_eq!(audit_count(&f, "offboard.execute"), 0);
    assert!(user(&f, "alice").enabled);
    assert!(f.core.me(&alice).is_ok());
    assert!(f.core.me(&bob).is_ok());
}

// RI-CON-004, RI-MGT-004, RI-STORE-001, Q02-C06/C08: a precommit fault is
// retried without disabling the user; authority loss during the later lease
// yields one terminal result and one matching audit, never a user mutation.
#[cfg(feature = "test-support")]
pub fn offboard_retry_rechecks_authority(backend: Backend) {
    let start = now();
    crypto::with_test_time(start, || {
        let f = backend.fixture();
        f.user("owner");
        let alice = f.user("alice");
        let bob = f.user("bob");
        let created = f
            .core
            .create_agent(
                &f.admin,
                NewAgent {
                    id: "owned-scheduler".into(),
                    ttl: 3600,
                    parent: Some("owner".into()),
                    permissions: vec![Permission {
                        action: "user.offboard".into(),
                        resource: "user/alice".into(),
                    }],
                },
            )
            .unwrap();
        let scheduler = text(&created["credential"], "token");
        let id = text(&offboard_schedule(&f, &scheduler, "alice", start + 1), "id");
        let before_user = user(&f, "alice");
        crypto::set_test_time(start + 1);
        assert!(
            f.core
                .offboard_process("worker-a", |_| BeforeCommit::RetryableFailure)
                .unwrap()
        );
        let retry = offboard_job(&f, &id);
        assert_eq!(retry.status, Status::Scheduled);
        assert_eq!(retry.attempts, 1);
        assert!(retry.next_attempt > now());
        assert!(retry.last_error.is_some());
        assert_eq!(audit_count(&f, "offboard.execute"), 0);
        assert_eq!(user(&f, "alice").epoch, before_user.epoch);
        assert!(user(&f, "alice").enabled);
        assert!(f.core.me(&alice).is_ok());
        let before_not_due = f.snapshot().unwrap();
        assert!(
            !f.core
                .offboard_process("worker-b", |_| BeforeCommit::Proceed)
                .unwrap()
        );
        f.assert_snapshot(&before_not_due);

        crypto::set_test_time(retry.next_attempt);
        let claimed = f.core.offboard_claim("worker-a").unwrap().unwrap();
        assert_eq!(claimed["id"], id);
        assert_eq!(claimed["attempts"], 2);
        let before_wrong_owner = f.snapshot().unwrap();
        assert!(f.core.offboard_claim("worker-b").unwrap().is_none());
        assert_eq!(
            f.core
                .offboard_commit("worker-b", &id, BeforeCommit::Proceed)
                .unwrap_err()
                .code,
            "lease_lost"
        );
        f.assert_snapshot(&before_wrong_owner);

        f.core.revoke_agent(&f.admin, "owned-scheduler").unwrap();
        let before_authority_rejection = f.snapshot().unwrap();
        let failed = f
            .core
            .offboard_commit("worker-a", &id, BeforeCommit::Proceed)
            .unwrap();
        assert_eq!(failed["status"], "failed");
        assert_eq!(failed["attempts"], 2);
        assert_eq!(offboard_job(&f, &id).status, Status::Failed);
        f.assert_snapshot_except(&before_authority_rejection, |key| {
            key.starts_with("offboard_jobs/")
                || key.starts_with("audit/")
                || (key.starts_with("index_") && key.contains("offboard_jobs"))
                || (key.starts_with("index_") && key.contains("audit"))
                || key == "meta/revision"
        });
        let revision_before = before_authority_rejection["meta/revision"]
            .as_u64()
            .unwrap();
        let revision_after: u64 = f.core.store.get("meta", "revision").unwrap().unwrap();
        assert_eq!(revision_after, revision_before + 1);
        let after_user = user(&f, "alice");
        assert!(after_user.enabled);
        assert_eq!(after_user.epoch, before_user.epoch);
        assert!(after_user.password_hash == before_user.password_hash);
        assert!(f.core.me(&alice).is_ok());
        assert!(f.core.me(&bob).is_ok());
        assert_eq!(audit_count(&f, "offboard.execute"), 1);
        assert!(audit(&f).iter().any(|event| {
            event["action"] == "offboard.execute"
                && event["actor"] == "agent:owned-scheduler"
                && event["target"] == format!("{id}/alice")
        }));
        let terminal = f.snapshot().unwrap();
        assert_eq!(
            f.core
                .offboard_commit("worker-a", &id, BeforeCommit::Proceed)
                .unwrap(),
            failed
        );
        assert!(f.core.offboard_claim("worker-c").unwrap().is_none());
        f.assert_snapshot(&terminal);
        assert_eq!(audit_count(&f, "offboard.execute"), 1);
    });
}

fn configure_cloud(f: &mut Fixture, remote: &cloud_mock::Mock) {
    let secret_file = f._dir.path().join("contract-cloud.secret");
    write_private(&secret_file, cloud_mock::SECRET.as_bytes(), true).unwrap();
    f.core.config.workspace_directories.insert(
        "corp".into(),
        WorkspaceDirectory {
            customer_id: "C01234567".into(),
            domain: "example.test".into(),
            token_url: remote.token_url.clone(),
            client_id: cloud_mock::CLIENT_ID.into(),
            client_secret_file: secret_file,
            direct_auth: None,
            directory_url: remote.base.clone(),
            groups: BTreeMap::new(),
            attributes: Attributes {
                email: "primaryEmail".into(),
                display_name: "name.fullName".into(),
                external_id: "id".into(),
            },
            username_prefix: String::new(),
            scope: String::new(),
        },
    );
}

// RI-CON-001/002, RI-MGT-004, RI-STORE-001, Q02-C06/C08: a real loopback
// connector produces a durable plan. Malformed, partial and changed snapshots
// cannot apply it. Permission loss denies before the fetch; an independent
// second-entry conflict rolls back the first staged account on every backend.
pub fn cloud_snapshot_apply_atomic_retry(backend: Backend) {
    let remote = cloud_mock::Mock::new();
    let mut f = backend.fixture();
    configure_cloud(&mut f, &remote);
    let local = f.user("local");
    let syncer = agent(
        &f,
        "syncer",
        &[
            ("directory.sync", "workspace/corp"),
            ("directory.read", "workspace/corp"),
            ("user.write", "*"),
        ],
    );
    let before_plan = f.snapshot().unwrap();
    let plan = f.core.cloud_plan(&syncer, "workspace", "corp").unwrap();
    let plan_id = text(&plan, "id");
    assert_eq!(plan["actor"], "agent:syncer");
    assert_eq!(plan["changes"].as_array().unwrap().len(), 2);
    assert_eq!(plan["entries"][0]["username"], "cloud-alice");
    assert_eq!(plan["entries"][1]["username"], "cloud-bob");
    assert_eq!(plan["applied"], false);
    assert_eq!(
        f.core
            .store
            .list::<Value>("cloud_directory_plans")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(audit_count(&f, "cloud_directory.plan"), 1);
    assert!(
        f.core
            .store
            .get::<String>("usernames", "cloud-alice")
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .get::<String>("usernames", "cloud-bob")
            .unwrap()
            .is_none()
    );
    assert!(
        before_plan
            .keys()
            .all(|key| !key.starts_with("cloud_directory_plans/"))
    );
    let before_wrong_actor = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .cloud_apply(&f.admin, "workspace", &plan_id)
            .unwrap_err()
            .code,
        "access_denied"
    );
    f.assert_snapshot(&before_wrong_actor);

    for mode in [
        cloud_mock::Mode::MissingUsers,
        cloud_mock::Mode::PartialFailure,
    ] {
        remote.set_mode(mode);
        let before = f.snapshot().unwrap();
        assert_eq!(
            f.core
                .cloud_apply(&syncer, "workspace", &plan_id)
                .unwrap_err()
                .code,
            "directory_unavailable"
        );
        // Failed fetches advance only the connector's retry ledger.
        f.assert_snapshot_except(&before, |key| key.starts_with("cloud_directory_runs/"));
        assert_eq!(audit_count(&f, "cloud_directory.apply"), 0);
    }
    remote.set_mode(cloud_mock::Mode::Changed);
    let before_stale = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .cloud_apply(&syncer, "workspace", &plan_id)
            .unwrap_err()
            .code,
        "conflict"
    );
    f.assert_snapshot_except(&before_stale, |key| {
        key.starts_with("cloud_directory_runs/")
    });

    remote.set_mode(cloud_mock::Mode::Complete);
    let original_permissions = f
        .core
        .store
        .get::<Agent>("agents", "syncer")
        .unwrap()
        .unwrap()
        .permissions;
    // There is no agent permission-edit endpoint. Model a live reduction at
    // the existing authority record. The plan remains readable, but apply
    // must deny the missing user.write scope before consulting the feed.
    f.core
        .store
        .write(|tx| {
            let mut actor: Agent = tx.get("agents", "syncer")?.unwrap();
            actor.permissions.retain(|p| p.action != "user.write");
            actor.permissions.push(Permission {
                action: "user.write".into(),
                resource: "user/cloud-alice".into(),
            });
            tx.put("agents", "syncer", &actor)
        })
        .unwrap();
    let before_denied = f.snapshot().unwrap();
    let hits_before_denied = remote.users_hits();
    let denied = f
        .core
        .cloud_apply(&syncer, "workspace", &plan_id)
        .unwrap_err();
    assert_eq!(denied.code, "access_denied");
    assert_eq!(denied.message, "Access denied");
    assert_eq!(remote.users_hits(), hits_before_denied);
    f.assert_snapshot(&before_denied);
    assert!(
        f.core
            .store
            .list::<Value>("cloud_directory_bindings")
            .unwrap()
            .is_empty()
    );
    for username in ["cloud-alice", "cloud-bob"] {
        assert!(
            f.core
                .store
                .get::<String>("usernames", username)
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(audit_count(&f, "user.cloud_directory_sync"), 0);
    assert_eq!(audit_count(&f, "cloud_directory.apply"), 0);
    assert_eq!(
        f.core
            .cloud_plan_get(&syncer, "workspace", &plan_id)
            .unwrap()["applied"],
        false
    );
    assert!(f.core.me(&local).is_ok());

    f.core
        .store
        .write(|tx| {
            let mut actor: Agent = tx.get("agents", "syncer")?.unwrap();
            actor.permissions = original_permissions.clone();
            tx.put("agents", "syncer", &actor)
        })
        .unwrap();
    // Even a permission addition that preserves all required scopes changes
    // the reviewed authority. A new plan is required before any write.
    f.core
        .store
        .write(|tx| {
            let mut actor: Agent = tx.get("agents", "syncer")?.unwrap();
            actor.permissions.push(Permission {
                action: "audit.read".into(),
                resource: "*".into(),
            });
            tx.put("agents", "syncer", &actor)
        })
        .unwrap();
    let before_changed_authority = f.snapshot().unwrap();
    let changed_authority = f
        .core
        .cloud_apply(&syncer, "workspace", &plan_id)
        .unwrap_err();
    assert_eq!(changed_authority.code, "conflict");
    assert_eq!(
        changed_authority.message,
        "Connector plan content or authority changed; create and review a new plan"
    );
    f.assert_snapshot_except(&before_changed_authority, |key| {
        key.starts_with("cloud_directory_runs/")
    });
    f.core
        .store
        .write(|tx| {
            let mut actor: Agent = tx.get("agents", "syncer")?.unwrap();
            actor.permissions = original_permissions.clone();
            tx.put("agents", "syncer", &actor)
        })
        .unwrap();

    // Reserve the second name directly without advancing the plan's global
    // revision. Reconciliation stages Alice, then detects Bob's collision.
    let mut occupied_bob = user(&f, "local");
    occupied_bob.id = crypto::id();
    occupied_bob.username = "cloud-bob".into();
    f.core
        .store
        .write(|tx| {
            tx.put("users", &occupied_bob.id, &occupied_bob)?;
            tx.put("usernames", "cloud-bob", &occupied_bob.id)
        })
        .unwrap();
    let before_collision = f.snapshot().unwrap();
    let collision = f
        .core
        .cloud_apply(&syncer, "workspace", &plan_id)
        .unwrap_err();
    assert_eq!(collision.code, "conflict");
    assert_eq!(
        collision.message,
        "Cloud directory username collides with an existing account; accounts are never automatically linked"
    );
    f.assert_snapshot_except(&before_collision, |key| {
        key.starts_with("cloud_directory_runs/")
    });
    assert!(
        f.core
            .store
            .get::<String>("usernames", "cloud-alice")
            .unwrap()
            .is_none()
    );
    assert_eq!(user(&f, "cloud-bob").id, occupied_bob.id);
    assert_eq!(audit_count(&f, "user.cloud_directory_sync"), 0);
    assert_eq!(audit_count(&f, "cloud_directory.apply"), 0);
    f.core
        .store
        .write(|tx| {
            tx.delete("usernames", "cloud-bob")?;
            tx.delete("users", &occupied_bob.id)
        })
        .unwrap();
    let applied = f.core.cloud_apply(&syncer, "workspace", &plan_id).unwrap();
    assert_eq!(applied["applied"], true);
    assert_eq!(audit_count(&f, "user.cloud_directory_sync"), 2);
    assert_eq!(audit_count(&f, "cloud_directory.apply"), 1);
    assert_eq!(
        f.core
            .cloud_plan_get(&syncer, "workspace", &plan_id)
            .unwrap()["applied"],
        true
    );
    assert_eq!(
        f.core
            .store
            .list::<Value>("cloud_directory_bindings")
            .unwrap()
            .len(),
        2
    );
    for username in ["cloud-alice", "cloud-bob"] {
        let account = user(&f, username);
        assert!(account.enabled);
        assert!(!account.admin);
        assert!(account.password_hash.is_empty());
    }
    assert!(f.core.me(&local).is_ok());
    let after_apply = f.snapshot().unwrap();
    let hits = remote.users_hits();
    assert_eq!(
        f.core.cloud_apply(&syncer, "workspace", &plan_id).unwrap(),
        applied
    );
    assert_eq!(remote.users_hits(), hits);
    f.assert_snapshot(&after_apply);
    f.core.revoke_agent(&f.admin, "syncer").unwrap();
    let after_revoke = f.snapshot().unwrap();
    assert!(f.core.cloud_apply(&syncer, "workspace", &plan_id).is_err());
    f.assert_snapshot(&after_revoke);
    assert_eq!(audit_count(&f, "cloud_directory.apply"), 1);
}

// R04, RI-STORE-004, Q02-C10: older database state returned by a database-native
// restore (PostgreSQL database clone, copied redb file). PostgreSQL detects the
// new database lineage when it opens; redb has no lineage and relies on the
// operator's `recovery invalidate`. Both must end in the same security state.
pub fn database_native_restore_policy(backend: Backend) {
    use riauth::recovery;
    let f = backend.fixture();
    f.client("app", false);
    let alice = f.user("alice");
    let tokens = f.tokens("app", &alice, None);
    let pending_code = f.exchange_request("app", &alice, None);
    let before = user(&f, "alice");
    let keys = f.core.jwks().unwrap();
    let revision: u64 = f.core.store.get("meta", "revision").unwrap().unwrap();
    f.core
        .store
        .write(|tx| {
            tx.put("assertion_replays", "restore-contract", &json!(now() + 600))?;
            let mut enrolling = before.clone();
            enrolling.totp_pending = Some(("restored-enrollment".into(), now() + 600));
            tx.put("users", &enrolling.id, &enrolling)
        })
        .unwrap();
    let f = f.restored_copy();
    let detected = f.core.store.read(recovery::pending).unwrap();
    assert_eq!(
        detected.as_ref().map(|r| r.cause),
        (f.core.store.backend() == "postgresql").then_some(recovery::Cause::StorageLineageChanged)
    );
    if detected.is_some() {
        // Opening the copy already applied the policy, before any operator step.
        assert!(f.core.me(&alice).is_err());
        assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
        assert_eq!(user(&f, "alice").epoch, before.epoch + recovery::STRIDE);
        assert!(f.core.store.ready().is_err());
    }
    // A second store handle means another server may still be writing: redb's
    // file lock refuses it, PostgreSQL recovery refuses while it is connected.
    let config = f.core.config.clone();
    if f.core.store.backend() == "postgresql" {
        let other = Core::open(config.clone()).unwrap();
        let error = recovery::invalidate_restored(&other.store).unwrap_err();
        assert_eq!(error.status, StatusCode::CONFLICT);
        drop(other);
    } else {
        assert!(Core::open(config.clone()).is_err());
    }
    // A closed client's backend leaves pg_stat_activity asynchronously.
    let mut attempts = 0;
    let applied = loop {
        match recovery::invalidate_restored(&f.core.store) {
            Err(error) if error.status == StatusCode::CONFLICT && attempts < 50 => {
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            result => break result.unwrap(),
        }
    };
    assert_eq!(applied.cause, recovery::Cause::DatabaseRestore);
    let snapshot = f.snapshot().unwrap();
    for bucket in recovery::INVALIDATED {
        let prefix = format!("{bucket}/");
        assert!(
            !snapshot.keys().any(|key| key.starts_with(&prefix)),
            "{bucket} survived recovery"
        );
    }
    assert!(snapshot.contains_key("assertion_replays/restore-contract"));
    let restored = user(&f, "alice");
    assert_eq!(restored.id, before.id);
    assert!(restored.totp_pending.is_none());
    assert_eq!(restored.subjects, before.subjects);
    assert!(restored.epoch >= before.epoch + recovery::STRIDE);
    assert!(
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap()
            >= revision + recovery::STRIDE
    );
    assert_eq!(f.core.jwks().unwrap(), keys);
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&f.admin).is_err());
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(f.core.token(refresh("app", &tokens)).is_err());
    assert!(f.core.token(pending_code).is_err());
    assert!(f.core.store.ready().is_err());
    // Reopening neither re-applies the policy nor clears the gate.
    let at_rest = f.snapshot().unwrap();
    let f = f.reopen_with(|_| {});
    f.assert_snapshot(&at_rest);
    assert!(f.core.store.ready().is_err());
    // New authentication works; traffic needs the operator's attestation.
    let session = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    assert_eq!(f.core.me(&session).unwrap()["user"]["id"], before.id);
    assert!(recovery::complete(&f.core.store, &applied.id, false).is_err());
    assert!(f.core.store.ready().is_err());
    if let Some(superseded) = &detected {
        assert!(recovery::complete(&f.core.store, &superseded.id, true).is_err());
    }
    let completed = recovery::complete(&f.core.store, &applied.id, true).unwrap();
    assert_eq!(completed.id, applied.id);
    f.core.store.ready().unwrap();
    assert!(recovery::complete(&f.core.store, &applied.id, true).is_err());
    let fresh = f.tokens("app", &session, None);
    assert!(f.core.userinfo(&text(&fresh, "access_token")).is_ok());
}

// R04, RI-STORE-004: `riauth recovery status` inspects without opening. A missing
// store stays missing and reports not serving; an existing store, including one
// with a pending gate, is byte-for-byte (redb) or record-for-record unchanged.
pub fn recovery_status_never_creates_or_writes_a_store(backend: Backend) {
    use riauth::recovery;
    let empty = backend.uninitialized();
    for _ in 0..2 {
        let status = recovery::inspect(&empty.config).unwrap();
        assert_eq!(status["initialized"], false);
        assert_eq!(status["serving_allowed"], false);
        assert!(status["pending"].is_null());
        assert!(empty.untouched());
    }
    drop(empty);
    let f = backend.fixture();
    let applied = recovery::invalidate_restored(&f.core.store).unwrap();
    let at_rest = f.snapshot().unwrap();
    let f = f.reopen_with(|config| {
        let file = config.data_dir.join("riauth.redb");
        let bytes = config
            .postgres
            .is_none()
            .then(|| std::fs::read(&file).unwrap());
        let status = recovery::inspect(config).unwrap();
        assert_eq!(status["initialized"], true);
        assert_eq!(status["serving_allowed"], false);
        assert_eq!(status["pending"]["id"], json!(applied.id));
        if let Some(bytes) = bytes {
            assert_eq!(std::fs::read(&file).unwrap(), bytes);
        }
    });
    f.assert_snapshot(&at_rest);
    recovery::complete(&f.core.store, &applied.id, true).unwrap();
    let f = f.reopen_with(|config| {
        assert_eq!(recovery::inspect(config).unwrap()["serving_allowed"], true);
    });
    f.core.store.ready().unwrap();
}
