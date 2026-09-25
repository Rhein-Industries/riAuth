mod common;

use base64::Engine;
use common::{Fixture, PASSWORD};
use riauth::{
    lifecycle::{Invitation, MailConfig, MailSecurity, Purpose},
    model::{NewUser, User, UserPatch},
    state::{ApplyRequest, Manifest},
};
use serde_json::{Value, json};
use std::{
    sync::{Arc, Barrier},
    thread,
};

fn reused(error: &riauth::error::Error) -> bool {
    error.code == "invalid_request" && error.message == "Password was used recently"
}

fn patch(password: &str) -> UserPatch {
    UserPatch {
        password: Some(password.into()),
        ..Default::default()
    }
}

fn user_id(core: &riauth::core::Core, username: &str) -> String {
    core.store
        .get::<String>("usernames", username)
        .unwrap()
        .unwrap()
}

fn history(core: &riauth::core::Core, username: &str) -> Option<Vec<String>> {
    core.store
        .get("password_history", &user_id(core, username))
        .unwrap()
}

fn create(core: &riauth::core::Core, admin: &str, username: &str, password: &str) {
    core.create_user(
        admin,
        NewUser {
            username: username.into(),
            password: password.into(),
            email: Some(format!("{username}@example.test")),
            display_name: username.into(),
            admin: false,
        },
    )
    .unwrap();
}

fn mail(core: &mut riauth::core::Core) {
    core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
}

fn mail_code_for(core: &riauth::core::Core, username: &str) -> String {
    let account = format!("Account: {username}");
    core.store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .find_map(|(_, delivery)| {
            let body = delivery["body"].as_str()?;
            if !body.contains(&account) {
                return None;
            }
            body.lines()
                .find(|line| line.starts_with("ri_mail_"))
                .map(str::to_owned)
        })
        .unwrap()
}

fn apply_manifest(
    f: &Fixture,
    manifest: Manifest,
    secrets: Vec<(&str, &str)>,
) -> riauth::error::Result<Value> {
    let plan = f.core.plan_state(&f.admin, manifest)?;
    f.core.apply_state(
        &f.admin,
        ApplyRequest {
            plan,
            secrets: secrets
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect(),
            run_id: None,
        },
    )
}

fn hidden(value: &Value, secrets: &[&str]) {
    let text = value.to_string();
    for secret in secrets {
        assert!(!text.contains(secret), "{text}");
    }
    assert!(!text.contains("password_hash"));
    assert!(!text.contains("password_history"));
    assert!(!text.contains("$argon2"));
    assert!(!text.contains("pbkdf2_sha256$"));
}

#[test]
fn reuse_is_rejected_on_every_password_write() {
    let mut f = Fixture::new();
    mail(&mut f.core);
    let alice = f.user("alice");
    assert_eq!(history(&f.core, "alice").unwrap().len(), 1);
    assert!(reused(
        &f.core.recover_admin("alice", PASSWORD, false).unwrap_err()
    ));
    assert!(reused(
        &f.core
            .change_password(&alice, PASSWORD.into(), PASSWORD.into(), None)
            .unwrap_err()
    ));
    let changed = "changed-history-password";
    f.core
        .change_password(&alice, PASSWORD.into(), changed.into(), None)
        .unwrap();
    assert!(f.core.me(&alice).is_err());
    assert!(reused(
        &f.core
            .update_user(&f.admin, "alice", patch(changed))
            .unwrap_err()
    ));
    let manifest = serde_json::from_value(json!({
        "api_version": "riauth/v1",
        "users": [{
            "username": "alice",
            "display_name": "Test User",
            "email": "alice@example.test",
            "password_ref": "env:PW",
            "password_version": "v1"
        }]
    }))
    .unwrap();
    assert!(reused(
        &apply_manifest(&f, manifest, vec![("env:PW", changed)]).unwrap_err()
    ));
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
    f.core.account_reset_request("alice").unwrap();
    assert!(reused(
        &f.core
            .account_complete(
                mail_code_for(&f.core, "alice"),
                Purpose::Reset,
                Some(changed.into())
            )
            .unwrap_err()
    ));
    let scim_password = "scim-history-password";
    let created = f
        .core
        .scim_write(
            &f.admin,
            "Users",
            None,
            json!({"schemas":[riauth::scim::USER],"userName":"scim-user","displayName":"SCIM","password":scim_password,"active":true}),
            false,
        )
        .unwrap();
    hidden(&created, &[scim_password]);
    let scim_id = created["id"].as_str().unwrap();
    assert!(reused(
        &f.core
            .scim_write(
                &f.admin,
                "Users",
                Some(scim_id),
                json!({"schemas":[riauth::scim::USER],"userName":"scim-user","displayName":"SCIM","password":scim_password,"active":true}),
                false,
            )
            .unwrap_err()
    ));
    let empty = f
        .core
        .scim_write(
            &f.admin,
            "Users",
            None,
            json!({"schemas":[riauth::scim::USER],"userName":"scim-empty","displayName":"Empty"}),
            false,
        )
        .unwrap();
    hidden(&empty, &[]);
    let empty_user: User = f
        .core
        .store
        .get("users", &user_id(&f.core, "scim-empty"))
        .unwrap()
        .unwrap();
    assert!(empty_user.password_hash.is_empty());
    assert!(history(&f.core, "scim-empty").is_none());
    f.core
        .account_invite(
            &f.admin,
            Invitation {
                username: "invited".into(),
                email: "invited@example.test".into(),
                display_name: "Invited".into(),
                groups: Default::default(),
            },
        )
        .unwrap();
    assert!(history(&f.core, "invited").is_none());
    let invited: User = f
        .core
        .store
        .get("users", &user_id(&f.core, "invited"))
        .unwrap()
        .unwrap();
    assert!(invited.password_hash.is_empty());
    let invite_password = "invite-history-password";
    f.core
        .account_complete(
            mail_code_for(&f.core, "invited"),
            Purpose::Invite,
            Some(invite_password.into()),
        )
        .unwrap();
    assert_eq!(history(&f.core, "invited").unwrap().len(), 1);
    assert!(reused(
        &f.core
            .update_user(&f.admin, "invited", patch(invite_password))
            .unwrap_err()
    ));
    let users = f.core.list_users(&f.admin).unwrap();
    let audit = f.core.audit_events(&f.admin, 1000).unwrap();
    let hashes: Vec<String> = f
        .core
        .store
        .list::<Vec<String>>("password_history")
        .unwrap()
        .into_iter()
        .flat_map(|(_, values)| values)
        .collect();
    let mut secrets = vec![PASSWORD, changed, scim_password, invite_password];
    secrets.extend(hashes.iter().map(String::as_str));
    hidden(&users, &secrets);
    hidden(&audit, &secrets);
    hidden(
        &f.core.get_resource(&f.admin, "user", "alice").unwrap(),
        &secrets,
    );
}

#[test]
fn history_retention_import_preview_and_concurrent_writes() {
    let mut f = Fixture::new();
    f.core.config.password_history = 2;
    let p0 = "cap-history-password-0";
    let p1 = "cap-history-password-1";
    let p2 = "cap-history-password-2";
    create(&f.core, &f.admin, "capped", p0);
    assert_eq!(history(&f.core, "capped").unwrap().len(), 1);
    f.core.update_user(&f.admin, "capped", patch(p1)).unwrap();
    assert!(reused(
        &f.core
            .update_user(&f.admin, "capped", patch(p0))
            .unwrap_err()
    ));
    f.core.update_user(&f.admin, "capped", patch(p2)).unwrap();
    assert_eq!(history(&f.core, "capped").unwrap().len(), 2);
    assert!(reused(
        &f.core
            .update_user(&f.admin, "capped", patch(p1))
            .unwrap_err()
    ));
    f.core.update_user(&f.admin, "capped", patch(p0)).unwrap();
    assert_eq!(history(&f.core, "capped").unwrap().len(), 2);
    let disabled: Manifest = serde_json::from_value(json!({
        "api_version": "riauth/v1",
        "users": [{
            "username": "capped",
            "display_name": "capped",
            "email": "capped@example.test",
            "password_disabled": true
        }]
    }))
    .unwrap();
    apply_manifest(&f, disabled, vec![]).unwrap();
    let cleared: User = f
        .core
        .store
        .get("users", &user_id(&f.core, "capped"))
        .unwrap()
        .unwrap();
    assert!(cleared.password_hash.is_empty());
    assert_eq!(history(&f.core, "capped").unwrap().len(), 2);
    assert!(reused(
        &f.core
            .update_user(&f.admin, "capped", patch(p0))
            .unwrap_err()
    ));
    f.core.config.password_history = 0;
    f.core.update_user(&f.admin, "capped", patch(p0)).unwrap();
    assert!(f.core.login("capped".into(), p0.into(), None).is_ok());
    f.core.config.password_history = 5;
    let imported_password = "imported-history-secret";
    let mut digest = [0u8; 32];
    aws_lc_rs::pbkdf2::derive(
        aws_lc_rs::pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(10_000).unwrap(),
        b"history-salt",
        imported_password.as_bytes(),
        &mut digest,
    );
    let imported_hash = format!(
        "pbkdf2_sha256$10000$history-salt${}",
        base64::engine::general_purpose::STANDARD.encode(digest)
    );
    let preview: Manifest = serde_json::from_value(json!({
        "api_version": "riauth/v1",
        "users": [{
            "username": "preview-user",
            "display_name": "Preview",
            "password_ref": "env:NOT_LOADED",
            "password_version": "v1"
        }]
    }))
    .unwrap();
    let plan = f.core.plan_state(&f.admin, preview).unwrap();
    assert!(
        plan.changes
            .iter()
            .any(|change| { change.resource == "user/preview-user" && change.credential_change })
    );
    let rendered = serde_json::to_string(&plan).unwrap();
    assert!(!rendered.contains(imported_password));
    assert!(!rendered.contains(&imported_hash));
    assert!(
        f.core
            .store
            .get::<String>("usernames", "preview-user")
            .unwrap()
            .is_none()
    );
    let manifest: Manifest = serde_json::from_value(json!({
        "api_version": "riauth/v1",
        "users": [{
            "username": "imported",
            "display_name": "Imported",
            "password_hash_ref": "env:HASH",
            "password_version": "import-v1"
        }]
    }))
    .unwrap();
    apply_manifest(&f, manifest, vec![("env:HASH", &imported_hash)]).unwrap();
    common::security::subscribe(&f, "imported");
    let first = f
        .core
        .login("imported".into(), imported_password.into(), None)
        .unwrap();
    hidden(&first, &[imported_password, imported_hash.as_str()]);
    assert!(
        f.core
            .login("imported".into(), imported_password.into(), None)
            .is_ok()
    );
    let imported: User = f
        .core
        .store
        .get("users", &user_id(&f.core, "imported"))
        .unwrap()
        .unwrap();
    assert!(imported.password_hash.starts_with("$argon2id$"));
    assert!(
        common::security::events(&f, "imported").is_empty(),
        "transparent rehash is not a credential replacement"
    );
    let imported_history = history(&f.core, "imported").unwrap();
    assert!(
        imported_history
            .iter()
            .any(|hash| hash.starts_with("$argon2id$"))
    );
    assert!(
        imported_history
            .iter()
            .all(|hash| !hash.contains(imported_password))
    );
    assert!(reused(
        &f.core
            .update_user(&f.admin, "imported", patch(imported_password))
            .unwrap_err()
    ));
    create(&f.core, &f.admin, "racer", "racer-initial-password");
    let barrier = Arc::new(Barrier::new(2));
    let mut joins = Vec::new();
    for _ in 0..2 {
        let core = f.core.clone();
        let admin = f.admin.clone();
        let barrier = Arc::clone(&barrier);
        joins.push(thread::spawn(move || {
            barrier.wait();
            core.update_user(&admin, "racer", patch("racer-shared-password"))
        }));
    }
    let results: Vec<_> = joins.into_iter().map(|join| join.join().unwrap()).collect();
    assert_eq!(
        results.iter().filter(|result| result.is_ok()).count(),
        1,
        "{results:?}"
    );
    assert!(
        results
            .iter()
            .filter_map(|result| result.as_ref().err())
            .all(reused),
        "{results:?}"
    );
    assert_eq!(history(&f.core, "racer").unwrap().len(), 2);
}
