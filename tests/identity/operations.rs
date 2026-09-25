use super::*;

#[test]
fn storage_survives_restart_and_issuer_cannot_be_changed_silently() {
    let f = Fixture::new();
    f.client("app", false);
    let tokens = f.tokens("app", &f.admin, None);
    let config = f.core.config.clone();
    let before = f.core.jwks().unwrap();
    drop(f.core);
    let core = Core::open(config.clone()).unwrap();
    assert_eq!(core.jwks().unwrap(), before);
    assert!(core.userinfo(&text(&tokens, "access_token")).is_ok());
    drop(core);
    let wrong = Config {
        issuer: "https://another.example".into(),
        ..config
    };
    assert!(Core::open(wrong).is_err());
}

#[test]
fn cleanup_retains_sessions_for_outstanding_client_refresh_grants() {
    let mut f = Fixture::new();
    f.core.config.refresh_token_ttl = 300;
    f.core.config.session_ttl = 300;
    let admin = text(
        &f.core.login("admin".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    f.client("app", false);
    f.core
        .update_client(
            &admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    refresh_token_ttl: Some(7_200),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let tokens = f.tokens("app", &admin, None);
    age_session_and_grant_timestamps(&f.core, 3_901);
    let refreshed = f
        .core
        .token(TokenRequest {
            grant_type: "refresh_token".into(),
            client_id: Some("app".into()),
            refresh_token: Some(text(&tokens, "refresh_token")),
            ..Default::default()
        })
        .unwrap();
    let replacement_token = text(&refreshed, "refresh_token");
    let remaining = f
        .core
        .store
        .get::<Grant>("refresh", &digest(&replacement_token))
        .unwrap()
        .unwrap()
        .expires_at
        .saturating_sub(now());
    assert!(remaining > 3_000);
    f.core.cleanup().unwrap();
    let replacement = f
        .core
        .token(TokenRequest {
            grant_type: "refresh_token".into(),
            client_id: Some("app".into()),
            refresh_token: Some(replacement_token),
            ..Default::default()
        })
        .unwrap();
    assert!(text(&replacement, "refresh_token").len() > 10);
}

#[test]
fn cleanup_retains_sessions_when_instance_refresh_ttl_drops_after_issuance() {
    let mut f = Fixture::new();
    f.core.config.session_ttl = 300;
    let admin = text(
        &f.core.login("admin".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    f.client("app", false);
    f.core
        .update_client(
            &admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    refresh_token_ttl: Some(7_200),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let tokens = f.tokens("app", &admin, None);
    f.core.config.refresh_token_ttl = 300;
    age_session_and_grant_timestamps(&f.core, 3_901);
    f.core.cleanup().unwrap();
    let refreshed = f
        .core
        .token(TokenRequest {
            grant_type: "refresh_token".into(),
            client_id: Some("app".into()),
            refresh_token: Some(text(&tokens, "refresh_token")),
            ..Default::default()
        })
        .unwrap();
    assert!(text(&refreshed, "refresh_token").len() > 10);
}

#[test]
fn agent_plans_are_immutable_atomic_and_idempotent() {
    let f = Fixture::new();
    let agent = agent_token(
        &f,
        &[
            ("client.write", "client/managed"),
            ("client.read", "client/managed"),
        ],
    );
    let plan = f
        .core
        .plan_state(&agent, managed_manifest("managed"))
        .unwrap();
    assert!(
        f.core
            .store
            .get::<Client>("clients", "managed")
            .unwrap()
            .is_none()
    );
    let mut tampered = plan.clone();
    tampered.manifest.clients[0].redirect_uris = vec!["https://other.example.test/callback".into()];
    assert!(
        f.core
            .apply_state(
                &agent,
                riauth::state::ApplyRequest {
                    plan: tampered,
                    secrets: Default::default(),
                    run_id: None
                }
            )
            .is_err()
    );
    let input = || riauth::state::ApplyRequest {
        plan: plan.clone(),
        secrets: Default::default(),
        run_id: Some("agent-run-123".into()),
    };
    let first = f.core.apply_state(&agent, input()).unwrap();
    let revision = f.core.store.get::<u64>("meta", "revision").unwrap();
    assert_eq!(first["changed"], true);
    assert_eq!(f.core.apply_state(&agent, input()).unwrap(), first);
    assert_eq!(
        f.core.store.get::<u64>("meta", "revision").unwrap(),
        revision
    );
    let plan = f
        .core
        .plan_state(&agent, managed_manifest("managed"))
        .unwrap();
    assert!(plan.changes.is_empty());
    assert_eq!(
        f.core
            .apply_state(
                &agent,
                riauth::state::ApplyRequest {
                    plan,
                    secrets: Default::default(),
                    run_id: None
                }
            )
            .unwrap()["changed"],
        false
    );
    assert_eq!(
        f.core.store.get::<u64>("meta", "revision").unwrap(),
        revision
    );
    assert!(
        f.core
            .plan_state(&agent, managed_manifest("outside"))
            .is_err()
    );
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert!(
        events
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["run_id"] == "agent-run-123"
                && v["actor"].as_str().unwrap().starts_with("agent:"))
    );
}

#[test]
fn claim_mappings_scope_policies_and_subject_import_are_enforced() {
    let f = Fixture::new();
    f.client("app", false);
    f.user("alice");
    f.core.create_group(&f.admin, "engineering").unwrap();
    f.core
        .group_member(&f.admin, "engineering", "alice", true)
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                attributes: Some(std::collections::BTreeMap::from([(
                    "department".into(),
                    json!("engineering"),
                )])),
                subjects: Some(std::collections::BTreeMap::from([(
                    "app".into(),
                    "authentik-subject-123".into(),
                )])),
                ..Default::default()
            },
        )
        .unwrap();
    let login = f.core.login("alice".into(), PASSWORD.into(), None).unwrap();
    let mut settings = riauth::model::ProviderSettings {
        claims_in_access_token: true,
        groups_in_profile: true,
        ..Default::default()
    };
    settings.claim_mappings.push(riauth::claims::ClaimMapping {
        scope: "profile".into(),
        claim: "department".into(),
        source: riauth::claims::ClaimSource::Attribute {
            key: "department".into(),
        },
    });
    settings.policy.scopes.insert(
        "profile".into(),
        riauth::claims::Rule {
            all_groups: strings(&["engineering"]),
            ..Default::default()
        },
    );
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(settings.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    let request = f.exchange_request("app", &text(&login, "session_token"), None);
    let tokens = f.core.token(request).unwrap();
    let info = f.core.userinfo(&text(&tokens, "access_token")).unwrap();
    assert_eq!(info["sub"], "authentik-subject-123");
    assert_eq!(info["department"], "engineering");
    let jwt: Value = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(text(&tokens, "access_token").split('.').nth(1).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(jwt["department"], "engineering");
    assert_eq!(jwt["sub"], info["sub"]);
    settings.claim_mappings[0].claim = "sub".into();
    assert!(
        f.core
            .update_client(
                &f.admin,
                "app",
                ClientPatch {
                    settings: Some(settings),
                    ..Default::default()
                }
            )
            .is_err()
    );
    f.core
        .group_member(&f.admin, "engineering", "alice", false)
        .unwrap();
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(
        f.core
            .token(TokenRequest {
                grant_type: "refresh_token".into(),
                client_id: Some("app".into()),
                refresh_token: Some(text(&tokens, "refresh_token")),
                ..Default::default()
            })
            .is_err()
    );
}

#[test]
fn encrypted_backup_restore_preserves_identity_keys_and_grants() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let tokens = f.tokens("app", &alice, None);
    let expected = f.core.userinfo(&text(&tokens, "access_token")).unwrap();
    let key = crypto::random_token("");
    let envelope = f.core.backup(&f.admin, &key).unwrap();
    assert!(!envelope.to_string().contains("PRIVATE KEY"));
    assert!(!envelope.to_string().contains(PASSWORD));
    let dir = TempDir::new().unwrap();
    let backup = dir.path().join("backup.json");
    let key_file = dir.path().join("key");
    std::fs::write(&backup, serde_json::to_vec(&envelope).unwrap()).unwrap();
    riauth::config::write_private(&key_file, key.as_bytes(), false).unwrap();
    let wrong_key = dir.path().join("wrong-key");
    riauth::config::write_private(&wrong_key, crypto::random_token("").as_bytes(), false).unwrap();
    let output = dir.path().join("restored");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key_file, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(riauth::operations::restore(&backup, &key_file, &output, None).is_err());
        assert!(!output.exists());
        std::fs::set_permissions(&key_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    assert!(riauth::operations::restore(&backup, &wrong_key, &output, None).is_err());
    assert!(!output.exists());
    riauth::operations::restore(&backup, &key_file, &output, Some(key_file.clone())).unwrap();
    assert!(riauth::operations::restore(&backup, &key_file, &output, None).is_err());
    let config = Config::load(&output.join("riauth.toml")).unwrap();
    let restored = Core::open(config.clone()).unwrap();
    assert_eq!(restored.jwks().unwrap(), f.core.jwks().unwrap());
    assert_eq!(
        restored.userinfo(&text(&tokens, "access_token")).unwrap(),
        expected
    );
    assert_eq!(
        restored.doctor(&f.admin).unwrap()["encrypted_at_rest"],
        true
    );
    let bytes = std::fs::read(output.join("data/riauth.redb")).unwrap();
    for marker in [
        b"PRIVATE KEY".as_slice(),
        b"alice@example.test",
        b"$argon2id$",
    ] {
        assert!(!bytes.windows(marker.len()).any(|w| w == marker));
    }
    drop(restored);
    let mut bad_config = config.clone();
    bad_config.database_key_file = Some(wrong_key);
    assert!(Core::open(bad_config).is_err());
    let mut plain_config = config;
    plain_config.database_key_file = None;
    assert!(Core::open(plain_config).is_err());
    let mut corrupt = envelope;
    let encoded = corrupt["chunks"][0].as_str().unwrap().to_owned();
    let mut cipher = URL_SAFE_NO_PAD.decode(encoded).unwrap();
    let last = cipher.len() - 1;
    cipher[last] ^= 1;
    corrupt["chunks"][0] = json!(URL_SAFE_NO_PAD.encode(cipher));
    std::fs::write(&backup, serde_json::to_vec(&corrupt).unwrap()).unwrap();
    assert!(
        riauth::operations::restore(&backup, &key_file, &dir.path().join("corrupt"), None).is_err()
    );
}

#[tokio::test]
async fn http_agent_mutations_are_atomic_retriable_conditional_and_attributed() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new();
    let token = agent_token(
        &f,
        &[
            ("client.write", "client/app"),
            ("client.rotate", "client/app"),
        ],
    );
    let app = riauth::api::router(f.core.clone());
    let revision = f
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap();
    let body = json!({"client_id":"app", "name":"app", "confidential":true,"redirect_uris":[],"scopes":["openid"],"allowed_groups":[],"require_mfa":false,"service":false});
    let request = |key: &str, revision: Option<u64>, body: &Value| {
        let mut builder = Request::post("/api/clients")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .header("idempotency-key", key)
            .header("x-riauth-run-id", "run-fixture");
        if let Some(r) = revision {
            builder = builder.header("if-match", format!("\"{r}\""));
        }
        builder.body(Body::from(body.to_string())).unwrap()
    };
    assert_eq!(
        app.clone()
            .oneshot(request("create-app", None, &body))
            .await
            .unwrap()
            .status(),
        StatusCode::PRECONDITION_REQUIRED
    );
    let first = app
        .clone()
        .oneshot(request("create-app", Some(revision), &body))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    assert!(first.headers().contains_key("x-request-id"));
    let first: Value =
        serde_json::from_slice(&first.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let repeat = app
        .clone()
        .oneshot(request("create-app", Some(revision), &body))
        .await
        .unwrap();
    assert_eq!(repeat.status(), StatusCode::OK);
    let repeat: Value =
        serde_json::from_slice(&repeat.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(first, repeat);
    assert!(first["client_secret"].is_string());
    assert_eq!(
        app.clone()
            .oneshot(request("different", Some(revision), &body))
            .await
            .unwrap()
            .status(),
        StatusCode::CONFLICT
    );
    let mut modified = body;
    modified["name"] = json!("changed");
    assert_eq!(
        app.oneshot(request("create-app", Some(revision), &modified))
            .await
            .unwrap()
            .status(),
        StatusCode::CONFLICT
    );
    let audit = f.core.audit_events(&f.admin, 100).unwrap();
    let changes: Vec<_> = audit
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["action"] == "client.create")
        .collect();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0]["run_id"], "run-fixture");
    assert!(changes[0]["details"]["request_id"].is_string());
    assert_eq!(
        changes[0]["details"]["changes"][0]["after"]["client_id"],
        "app"
    );
    assert!(
        !audit
            .to_string()
            .contains(first["client_secret"].as_str().unwrap())
    );
}

#[test]
fn inventory_cursors_are_opaque_bound_and_complete() {
    let f = Fixture::new();
    f.client("app", false);
    f.client("hidden", false);
    let token = agent_token(&f, &[("client.read", "client/app")]);
    let first = f.core.inventory(&token, "clients", None, 1, None).unwrap();
    assert_eq!(first["items"][0]["client_id"], "app");
    let cursor = text(&first, "next_cursor");
    assert!(!cursor.contains("app"));
    assert!(
        f.core
            .inventory(
                &token,
                "clients",
                Some(cursor.clone()),
                1,
                Some("app".into())
            )
            .is_err()
    );
    assert!(
        f.core
            .inventory(&f.admin, "clients", Some(cursor.clone()), 1, None)
            .is_err()
    );
    let next = f
        .core
        .inventory(&token, "clients", Some(cursor.clone()), 1, None)
        .unwrap();
    assert_eq!(next["items"], json!([]));
    assert_eq!(next["next_cursor"], Value::Null);
    f.client("another", false);
    assert!(
        f.core
            .inventory(&token, "clients", Some(cursor), 1, None)
            .is_err()
    );
}

#[test]
fn authentik_import_preserves_exported_subjects_and_blocks_incomplete_translation() {
    let f = Fixture::new();
    let input = json!({"api_version":"riauth.authentik-import/v1","issuer":f.core.config.issuer,
        "users":[{"pk":42,"uid":"existing-authentik-subject","username":"alice","name":"Alice","email":"alice@example.test","groups":["child"],"attributes":{},"type":"internal","is_active":true,"roles":[]}],
        "groups":[{"pk":"child","name":"engineering","parent":"parent"},{"pk":"parent","name":"employees","parent":null}],
        "providers":[{"pk":1,"name":"app","client_id":"app","client_type":"public","grant_types":["authorization_code","refresh_token"],"redirect_uris":[{"matching_mode":"strict","url":"http://localhost:7777/callback?existing=1"}],"property_mappings":["profile-mapping"],"sub_mode":"hashed_user_id","include_claims_in_id_token":true,"access_code_validity":"minutes=1","access_token_validity":"minutes=5","refresh_token_validity":"days=30"}],
        "applications":[{"slug":"app","provider":1,"name":"Team workspace","meta_launch_url":"https://app.example.test/","meta_description":"The team’s applications","group":"Engineering"}],"policy_bindings":[{"pk":"binding-1"}],"sources":[],
        "passwords":{"alice":{"reference":"env:ALICE_PASSWORD","version":"import-v1"}},
        "clients":{"app":{"issuer":f.core.config.issuer,"scopes":["openid","profile","groups","offline_access"],"settings":{"groups_in_profile":true,"policy":{"access":{"all_groups":["engineering"]}}},"translated_mapping_ids":["profile-mapping"],"translated_binding_ids":["binding-1"],"authentication_flow_reviewed":true,"require_mfa":false}}});
    let report =
        riauth::migration::convert(serde_json::from_value(input.clone()).unwrap()).unwrap();
    assert_eq!(report["ready_for_plan"], true, "{report}");
    assert_eq!(report["manifest"]["clients"][0]["name"], "Team workspace");
    assert_eq!(
        report["manifest"]["clients"][0]["settings"]["app"]["launch_url"],
        "https://app.example.test/"
    );
    assert_eq!(
        report["manifest"]["clients"][0]["settings"]["app"]["category"],
        "Engineering"
    );
    let manifest = serde_json::from_value(report["manifest"].clone()).unwrap();
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    f.core
        .apply_state(
            &f.admin,
            riauth::state::ApplyRequest {
                plan,
                secrets: [("env:ALICE_PASSWORD".into(), PASSWORD.into())].into(),
                run_id: Some("migration-test".into()),
            },
        )
        .unwrap();
    let login = f.core.login("alice".into(), PASSWORD.into(), None).unwrap();
    let mut auth = f.request("app", &crypto::random_token(""));
    auth.scope = "openid profile".into();
    assert!(
        f.core
            .authorize(&text(&login, "session_token"), auth)
            .is_ok()
    );
    let preview = f
        .core
        .explain(
            &f.admin,
            riauth::claims::Explain {
                client_id: "app".into(),
                username: "alice".into(),
                scope: strings(&["openid", "profile"]),
                mfa: false,
            },
        )
        .unwrap();
    assert_eq!(preview["userinfo"]["sub"], "existing-authentik-subject");
    assert_eq!(
        preview["userinfo"]["groups"],
        json!(["employees", "engineering"])
    );
    let portal = f.core.portal_sign_in().unwrap();
    f.core
        .portal_decide(
            &text(&login, "session_token"),
            portal.body["code"].as_str().unwrap(),
            true,
        )
        .unwrap();
    let binding = portal.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    let approved = f
        .core
        .portal_poll(portal.body["id"].as_str().unwrap(), Some(binding))
        .unwrap();
    let cookie = approved
        .cookies
        .iter()
        .find(|c| c.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    assert_eq!(
        f.core.portal_apps(Some(cookie)).unwrap()["apps"][0]["name"],
        "Team workspace"
    );
    assert_eq!(
        f.core.portal_launch(Some(cookie), "app").unwrap(),
        "https://app.example.test/"
    );
    let mut incomplete = input.clone();
    incomplete["clients"]["app"]["translated_binding_ids"] = json!([]);
    let report = riauth::migration::convert(serde_json::from_value(incomplete).unwrap()).unwrap();
    assert_eq!(report["ready_for_plan"], false);
    assert!(report["manifest"].is_null());
    let mut duplicate = input.clone();
    duplicate["users"]
        .as_array_mut()
        .unwrap()
        .push(input["users"][0].clone());
    duplicate["users"][1]["username"] = json!("bob");
    duplicate["users"][1]["pk"] = json!(43);
    assert_eq!(
        riauth::migration::convert(serde_json::from_value(duplicate).unwrap()).unwrap()["ready_for_plan"],
        false
    );
    let mut pages = input.clone();
    pages["users"] = json!({"pagination":{"count":2,"next":2}, "results":input["users"]});
    assert!(riauth::migration::convert(serde_json::from_value(pages).unwrap()).is_err());
}

#[test]
fn authentik_import_flattens_parents_arrays_and_diamonds() {
    // leaf → {left, right} → root: 2025.12 `parents` arrays, a legacy scalar `parent`, one diamond.
    let input = json!({"api_version":"riauth.authentik-import/v1","issuer":"https://id.example.test",
        "users":[{"pk":1,"uid":"a","username":"alice","name":"Alice","groups":["leaf"],"attributes":{},"type":"internal","is_active":true,"roles":[]},
            {"pk":2,"uid":"b","username":"bob","name":"Bob","groups":["right"],"attributes":{},"type":"internal","is_active":true,"roles":[]}],
        "groups":[{"pk":"leaf","name":"leaf","parents":["left","right"]},{"pk":"left","name":"left","parents":["root"]},
            {"pk":"right","name":"right","parents":[],"parent":"root"},{"pk":"root","name":"root","parents":[]},
            {"pk":"other","name":"other","parents":["root"],"parents_obj":null}],
        "providers":[{"pk":1,"name":"app","client_id":"app","client_type":"public","grant_types":["authorization_code"],"redirect_uris":[{"matching_mode":"strict","url":"http://localhost:7777/callback"}],"property_mappings":[],"sub_mode":"user_username","include_claims_in_id_token":true}],
        "applications":[],"policy_bindings":[],"sources":[],
        "passwords":{"alice":{"reference":"env:ALICE_PASSWORD","version":"v1"},"bob":{"reference":"env:BOB_PASSWORD","version":"v1"}},
        "clients":{"app":{"issuer":"https://id.example.test","scopes":["openid","profile"],"settings":{},"translated_mapping_ids":[],"translated_binding_ids":[],"authentication_flow_reviewed":true,"require_mfa":true}}});
    let report =
        riauth::migration::convert(serde_json::from_value(input.clone()).unwrap()).unwrap();
    assert_eq!(report["ready_for_plan"], true, "{report}");
    let members = report["manifest"]["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| (g["name"].as_str().unwrap(), g["members"].clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(members["leaf"], json!(["alice"]));
    assert_eq!(members["left"], json!(["alice"]));
    assert_eq!(members["right"], json!(["alice", "bob"]));
    assert_eq!(members["root"], json!(["alice", "bob"]));
    assert_eq!(members["other"], json!([]));
    let mut unreviewed = input;
    unreviewed["clients"]["app"]["authentication_flow_reviewed"] = json!(false);
    let report = riauth::migration::convert(serde_json::from_value(unreviewed).unwrap()).unwrap();
    assert_eq!(
        report["blockers"],
        json!([
            "app: authentication flow has not been reviewed for browser/terminal login and MFA"
        ])
    );
}

#[test]
fn authentik_import_rejects_group_cycles_in_parents_arrays() {
    let convert = |groups: Value| {
        riauth::migration::convert(
            serde_json::from_value(json!({"api_version":"riauth.authentik-import/v1","issuer":"https://id.example.test",
                "users":[{"pk":1,"uid":"a","username":"alice","name":"Alice","groups":["g0"],"attributes":{},"type":"internal","is_active":true,"roles":[]}],
                "groups":groups,"providers":[],"applications":[],"policy_bindings":[],"sources":[],
                "passwords":{"alice":{"reference":"env:ALICE_PASSWORD","version":"v1"}},"clients":{}}))
            .unwrap(),
        )
    };
    let group = |id: &str, parents: &[&str]| json!({"pk":id,"name":id,"parents":parents});
    let root = || group("root", &[]);
    for groups in [
        // A loop through the member's own group, beside a branch to root.
        json!([
            group("g0", &["a"]),
            group("a", &["b", "root"]),
            group("b", &["g0"]),
            root()
        ]),
        // A loop above the member's group, never returning to it.
        json!([
            group("g0", &["a"]),
            group("a", &["b"]),
            group("b", &["a"]),
            root()
        ]),
        // A loop no user belongs to.
        json!([
            group("g0", &["root"]),
            group("a", &["b"]),
            group("b", &["a"]),
            root()
        ]),
        // A scalar `parent` closing a `parents` loop, and a group that is its own parent.
        json!([group("g0", &["a"]), {"pk":"a","name":"a","parents":[],"parent":"g0"}, root()]),
        json!([group("g0", &["g0"]), root()]),
    ] {
        assert_eq!(
            convert(groups).unwrap_err().message,
            "Cycle in exported group hierarchy"
        );
    }
    let chain = |ancestors: usize| {
        (0..=ancestors)
            .map(|i| json!({"pk":format!("g{i}"),"name":format!("g{i}"),"parents":Vec::from_iter((i < ancestors).then(|| format!("g{}", i + 1)))}))
            .collect::<Value>()
    };
    assert!(convert(chain(1024)).is_ok());
    assert_eq!(
        convert(chain(1025)).unwrap_err().message,
        "Exported group has more than 1024 ancestors"
    );
}

#[test]
fn agent_credential_rotation_preserves_permissions_and_invalidates_the_old_token() {
    let f = Fixture::new();
    f.client("app", false);
    let old = agent_token(&f, &[("client.read", "client/app")]);
    assert!(f.core.rotate_agent(&old, "deployer", 60).is_err());
    let id = f.core.list_agents(&f.admin).unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let rotated = f.core.rotate_agent(&f.admin, &id, 300).unwrap();
    assert!(f.core.list_clients(&old).is_err());
    let new = rotated["credential"]["token"].as_str().unwrap();
    assert_eq!(
        f.core.list_clients(new).unwrap().as_array().unwrap().len(),
        1
    );
    assert!(f.core.create_group(new, "group").is_err());
}

#[test]
fn provider_signing_domains_import_rotate_and_encrypt_identity_tokens() {
    use aws_lc_rs::{
        encoding::{AsDer, PublicKeyX509Der},
        rsa::{OAEP_SHA256_MGF1SHA256, OaepPrivateDecryptingKey, PrivateDecryptingKey},
    };
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    for algorithm in ["ES256", "EdDSA"] {
        let out = f
            .core
            .configure_key(
                &f.admin,
                riauth::keyring::KeyInput {
                    remote_signer: None,
                    id: "app-key".into(),
                    algorithm: algorithm.into(),
                    private_key_pem: None,
                    kid: None,
                },
            )
            .unwrap();
        f.core
            .update_client(
                &f.admin,
                "app",
                ClientPatch {
                    settings: Some(ProviderSettings {
                        signing_key: Some("app-key".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .unwrap();
        let tokens = f.tokens("app", &alice, None);
        let jwks: riauth::jose::PublicJwks =
            serde_json::from_value(f.core.jwks().unwrap()).unwrap();
        assert_eq!(
            jsonwebtoken::decode_header(text(&tokens, "id_token"))
                .unwrap()
                .kid
                .as_deref(),
            out["active"]["kid"].as_str()
        );
        let claims = jwks
            .verify(&text(&tokens, "id_token"), &f.core.config.issuer, "app")
            .unwrap();
        assert_eq!(claims["nonce"], "expected-nonce");
        if algorithm == "EdDSA" {
            assert_eq!(
                claims["at_hash"],
                URL_SAFE_NO_PAD
                    .encode(&sha2::Sha512::digest(text(&tokens, "access_token").as_bytes())[..32])
            );
        }
    }
    let imported = crypto::SigningKey::generate_algorithm("ES256").unwrap();
    f.core
        .configure_key(
            &f.admin,
            riauth::keyring::KeyInput {
                remote_signer: None,
                id: "imported".into(),
                algorithm: "ES256".into(),
                private_key_pem: Some(imported.pem.clone()),
                kid: Some("authentik-original-kid".into()),
            },
        )
        .unwrap();
    assert!(
        !f.core
            .key_domains(&f.admin)
            .unwrap()
            .to_string()
            .contains("PRIVATE KEY")
    );
    let private = PrivateDecryptingKey::from_pkcs8(
        pem::parse(fixture_signing_key(&f).pem).unwrap().contents(),
    )
    .unwrap();
    let public = AsDer::<PublicKeyX509Der>::as_der(&private.public_key()).unwrap();
    let encryption = riauth::encryption::EncryptionKey {
        content_encryption: "A256GCM".into(),
        kid: "rp-encryption".into(),
        public_key_pem: pem::encode(&pem::Pem::new("PUBLIC KEY", public.as_ref())),
    };
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    signing_key: Some("imported".into()),
                    id_token_encryption: Some(encryption),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let tokens = f.tokens("app", &alice, None);
    let encrypted = text(&tokens, "id_token");
    let parts: Vec<_> = encrypted.split('.').collect();
    assert_eq!(parts.len(), 5);
    let header: Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[0]).unwrap()).unwrap();
    assert_eq!(header["alg"], "RSA-OAEP-256");
    assert_eq!(header["enc"], "A256GCM");
    assert_eq!(header["cty"], "JWT");
    let rsa = OaepPrivateDecryptingKey::new(private).unwrap();
    let mut plaintext = vec![0u8; 512];
    let cek = rsa
        .decrypt(
            &OAEP_SHA256_MGF1SHA256,
            &URL_SAFE_NO_PAD.decode(parts[1]).unwrap(),
            &mut plaintext,
            None,
        )
        .unwrap();
    let cek: [u8; 32] = cek.try_into().unwrap();
    let mut envelope = b"RIAUTH-AEAD1".to_vec();
    for i in [2, 3, 4] {
        envelope.extend(URL_SAFE_NO_PAD.decode(parts[i]).unwrap());
    }
    let signed = crypto::unseal(&cek, parts[0].as_bytes(), &envelope).unwrap();
    let jwks: riauth::jose::PublicJwks = serde_json::from_value(f.core.jwks().unwrap()).unwrap();
    assert_eq!(
        jwks.verify(
            std::str::from_utf8(&signed).unwrap(),
            &f.core.config.issuer,
            "app"
        )
        .unwrap()["nonce"],
        "expected-nonce"
    );
    *envelope.last_mut().unwrap() ^= 1;
    assert!(crypto::unseal(&cek, parts[0].as_bytes(), &envelope).is_err());
}

#[test]
fn schema_upgrade_is_atomic_preserves_credentials_and_rejects_future_versions() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let tokens = f.tokens("app", &alice, None);
    let user_id = text(&f.core.me(&alice).unwrap()["user"], "id");
    f.core
        .store
        .write(|tx| {
            let mut user: Value = tx.get("users", &user_id)?.unwrap();
            user.as_object_mut().unwrap().remove("pairwise_seed");
            tx.put("users", &user_id, &user)?;
            tx.put("meta", "schema", &1u32)
        })
        .unwrap();
    riauth::upgrade::migrate(&f.core.store).unwrap();
    assert_eq!(
        f.core.store.get::<u32>("meta", "schema").unwrap(),
        Some(riauth::upgrade::SCHEMA)
    );
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_ok());
    let user: User = f.core.store.get("users", &user_id).unwrap().unwrap();
    assert_eq!(user.pairwise_seed.len(), 43);
    let revision = f.core.store.get::<u64>("meta", "revision").unwrap();
    riauth::upgrade::migrate(&f.core.store).unwrap();
    assert_eq!(
        f.core.store.get::<u64>("meta", "revision").unwrap(),
        revision
    );
    f.core
        .store
        .write(|tx| tx.put("meta", "schema", &999u32))
        .unwrap();
    assert!(riauth::upgrade::migrate(&f.core.store).is_err());
    assert_eq!(
        f.core.store.get::<u32>("meta", "schema").unwrap(),
        Some(999)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn external_vault_signing_keeps_private_keys_out_of_storage_pins_version_and_verifies_signatures()
 {
    use axum::{
        Json, Router,
        extract::{Path, State},
        http::{HeaderMap, StatusCode},
        routing::post,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    };
    #[derive(Clone)]
    struct Vault {
        keys: Arc<std::collections::BTreeMap<String, crypto::SigningKey>>,
        mode: Arc<AtomicU8>,
        entered: Arc<tokio::sync::Notify>,
        resume: Arc<tokio::sync::Notify>,
    }
    async fn sign(
        State(vault): State<Vault>,
        Path(name): Path<String>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> (StatusCode, Json<Value>) {
        if vault.mode.load(Ordering::Relaxed) == 4 {
            vault.entered.notify_one();
            vault.resume.notified().await;
        }
        assert_eq!(headers.get("x-vault-token").unwrap(), "fixture-vault-token");
        assert_eq!(headers.get("x-vault-namespace").unwrap(), "test-team");
        assert_eq!(body["key_version"], 7);
        assert_eq!(body["prehashed"], false);
        assert_eq!(body["marshaling_algorithm"], "jws");
        assert_eq!(body["signature_algorithm"], "pkcs1v15");
        let input = base64::engine::general_purpose::STANDARD
            .decode(body["input"].as_str().unwrap())
            .unwrap();
        let key =
            openssl::pkey::PKey::private_key_from_pem(vault.keys[&name].pem.as_bytes()).unwrap();
        let bytes = if name == "EdDSA" {
            openssl::sign::Signer::new_without_digest(&key)
                .unwrap()
                .sign_oneshot_to_vec(&input)
                .unwrap()
        } else {
            let mut signer =
                openssl::sign::Signer::new(openssl::hash::MessageDigest::sha256(), &key).unwrap();
            signer.update(&input).unwrap();
            signer.sign_to_vec().unwrap()
        };
        let bytes = if name == "ES256" {
            let signature = openssl::ecdsa::EcdsaSig::from_der(&bytes).unwrap();
            [
                signature.r().to_vec_padded(32).unwrap(),
                signature.s().to_vec_padded(32).unwrap(),
            ]
            .concat()
        } else {
            bytes
        };
        let bytes = if vault.mode.load(Ordering::Relaxed) == 1 {
            vec![0; bytes.len()]
        } else {
            bytes
        };
        let version = if vault.mode.load(Ordering::Relaxed) == 2 {
            8
        } else {
            7
        };
        let encoded = if name == "ES256" {
            base64::engine::general_purpose::URL_SAFE.encode(bytes)
        } else {
            base64::engine::general_purpose::STANDARD.encode(bytes)
        };
        if vault.mode.load(Ordering::Relaxed) == 3 {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error":"unavailable"})),
            );
        }
        (
            StatusCode::OK,
            Json(json!({"data":{"signature":format!("vault:v{version}:{encoded}")}})),
        )
    }
    let mut f = Fixture::new();
    let user = f.user("kms-user");
    f.client("kms-rp", false);
    let keys: std::collections::BTreeMap<_, _> = ["RS256", "ES256", "EdDSA"]
        .iter()
        .map(|alg| {
            let key = if *alg == "EdDSA" {
                // OpenSSL 3.0 needs PKCS#8 v1; generate the mock signer's key
                // with OpenSSL instead of importing AWS-LC's v2 encoding.
                let pem = openssl::pkey::PKey::generate_ed25519()
                    .unwrap()
                    .private_key_to_pem_pkcs8()
                    .unwrap();
                crypto::SigningKey::import(alg, std::str::from_utf8(&pem).unwrap(), None).unwrap()
            } else {
                crypto::SigningKey::generate_algorithm(alg).unwrap()
            };
            (alg.to_string(), key)
        })
        .collect();
    let mode = Arc::new(AtomicU8::new(0));
    let entered = Arc::new(tokio::sync::Notify::new());
    let resume = Arc::new(tokio::sync::Notify::new());
    let vault = Vault {
        keys: Arc::new(keys.clone()),
        mode: mode.clone(),
        entered: entered.clone(),
        resume: resume.clone(),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let mock = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new()
                .route("/v1/transit/sign/{name}", post(sign))
                .with_state(vault),
        )
        .await
        .unwrap();
    });
    let dir = tempfile::TempDir::new().unwrap();
    let credential = dir.path().join("vault-token");
    riauth::config::write_private(&credential, b"fixture-vault-token", false).unwrap();
    for (algorithm, key) in &keys {
        let config = riauth::kms::VaultSigner {
            address: address.clone(),
            mount: "transit".into(),
            key_name: algorithm.clone(),
            key_version: 7,
            public_jwk: serde_json::from_value(key.jwk().unwrap()).unwrap(),
            token_file: credential.clone(),
            ca_file: None,
            namespace: Some("test-team".into()),
        };
        f.core.config.signers.insert(algorithm.clone(), config);
        let mut bound = None;
        for retry in [false, true] {
            let core = f.core.clone();
            let admin = f.admin.clone();
            let algorithm = algorithm.clone();
            if retry {
                mode.store(3, Ordering::Relaxed);
            }
            let result = tokio::task::spawn_blocking(move || {
                let context = riauth::context::RequestContext {
                    request_id: format!("vault-{algorithm}"),
                    idempotency_key: Some(format!("vault-bind-{algorithm}")),
                    fingerprint: format!("bind-signing-{algorithm}"),
                    ..Default::default()
                };
                riauth::context::scope(Some(context), || {
                    core.configure_key(
                        &admin,
                        riauth::keyring::KeyInput {
                            id: "signing".into(),
                            algorithm: algorithm.clone(),
                            private_key_pem: None,
                            kid: None,
                            remote_signer: Some(algorithm),
                        },
                    )
                })
            })
            .await
            .unwrap()
            .unwrap();
            if let Some(first) = &bound {
                assert_eq!(first, &result);
            } else {
                bound = Some(result);
            }
            mode.store(0, Ordering::Relaxed);
        }
        let request = f.exchange_request("kms-rp", &user, None);
        let core = f.core.clone();
        let response = tokio::task::spawn_blocking(move || core.token(request))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            jsonwebtoken::decode_header(text(&response, "id_token"))
                .unwrap()
                .kid,
            Some(key.kid.clone())
        );
        let jwks = riauth::jose::PublicJwks {
            keys: vec![serde_json::from_value(key.jwk().unwrap()).unwrap()],
        };
        jwks.verify(
            &text(&response, "id_token"),
            &f.core.config.issuer,
            "kms-rp",
        )
        .unwrap();
        let stored: crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
        assert!(stored.active.pem.is_empty());
        assert!(stored.active.remote.is_some());
    }
    let snapshot = f.core.store.read(|tx| tx.snapshot()).unwrap();
    let serialized = serde_json::to_string(&snapshot).unwrap();
    for key in keys.values() {
        assert!(!serialized.contains(&key.pem));
    }
    assert!(!serialized.contains("fixture-vault-token"));
    // A failed or incorrectly signed KMS response rolls back code consumption.
    for failure in 1..=3 {
        let request = f.exchange_request("kms-rp", &user, None);
        mode.store(failure, Ordering::Relaxed);
        let core = f.core.clone();
        let attempt = request.clone();
        let failure = tokio::task::spawn_blocking(move || core.token(attempt))
            .await
            .unwrap()
            .unwrap_err();
        assert_eq!(failure.code, "signer_unavailable");
        mode.store(0, Ordering::Relaxed);
        let core = f.core.clone();
        assert!(
            tokio::task::spawn_blocking(move || core.token(request))
                .await
                .unwrap()
                .is_ok()
        );
    }
    // Slow external signing must neither block unrelated mutations nor issue
    // credentials from authority revoked while the signature is in flight.
    let request = f.exchange_request("kms-rp", &user, None);
    mode.store(4, Ordering::Relaxed);
    let core = f.core.clone();
    let signing = tokio::task::spawn_blocking(move || core.token(request));
    tokio::time::timeout(std::time::Duration::from_secs(10), entered.notified())
        .await
        .unwrap();
    let core = f.core.clone();
    let admin = f.admin.clone();
    let revoke = tokio::task::spawn_blocking(move || {
        core.update_user(
            &admin,
            "kms-user",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
    });
    tokio::time::timeout(std::time::Duration::from_secs(2), revoke)
        .await
        .expect("Vault held the writer lock")
        .unwrap()
        .unwrap();
    mode.store(0, Ordering::Relaxed);
    resume.notify_one();
    assert!(signing.await.unwrap().is_err());
    mock.abort();
}

#[test]
fn agent_credentials_enforce_action_resource_and_identity_boundaries() {
    let f = Fixture::new();
    f.client("mine", false);
    f.client("other", false);
    let agent = agent_token(
        &f,
        &[
            ("client.read", "client/mine"),
            ("client.write", "client/mine"),
        ],
    );
    let clients = f.core.list_clients(&agent).unwrap();
    assert_eq!(clients.as_array().unwrap().len(), 1);
    assert_eq!(clients[0]["client_id"], "mine");
    f.core
        .update_client(
            &agent,
            "mine",
            ClientPatch {
                name: Some("Updated".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        f.core
            .update_client(&agent, "other", ClientPatch::default())
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    assert!(f.core.rotate_client_secret(&agent, "mine").is_err());
    assert!(f.core.rotate_key(&agent).is_err());
    assert!(f.core.list_agents(&agent).is_err());
    assert!(
        f.core
            .authorize(&agent, f.request("mine", &crypto::random_token("")))
            .is_err()
    );
    let user_agent = agent_token(&f, &[("user.write", "*")]);
    assert!(
        f.core
            .update_user(
                &user_agent,
                "admin",
                UserPatch {
                    password: Some("another-password".into()),
                    ..Default::default()
                }
            )
            .is_err()
    );
    let id = f.core.me(&agent).unwrap()["agent_id"]
        .as_str()
        .unwrap()
        .strip_prefix("agent:")
        .unwrap()
        .to_owned();
    f.core.revoke_agent(&f.admin, &id).unwrap();
    assert!(f.core.list_clients(&agent).is_err());
}
