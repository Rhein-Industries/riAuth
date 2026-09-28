use super::*;

#[tokio::test]
async fn email_capabilities_require_local_smtp_credential_before_serving() {
    let mut f = Fixture::new();
    let password_file = f._dir.path().join("smtp-password");
    assert!(!password_file.exists());
    f.core.config.mail = Some(riauth::lifecycle::MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: riauth::lifecycle::MailSecurity::Loopback,
        username: Some("sender".into()),
        password_file: Some(password_file.clone()),
    });
    let states = riauth::capability::runtime(&f.core).unwrap();
    for name in [
        "identity.email_verification",
        "identity.invitations",
        "identity.email_password_reset",
    ] {
        assert_eq!(states["feature_states"][name]["compiled"], true);
        assert_eq!(states["feature_states"][name]["enabled"], true);
        assert_eq!(states["feature_states"][name]["configured"], false, "{name}");
        assert_eq!(states["feature_states"][name]["usable"], false);
    }
    let error = riauth::api::serve(f.core.clone()).await.unwrap_err();
    assert!(error.to_string().contains("SMTP configuration unusable"));

    riauth::config::write_private(&password_file, b"local-test-password", false).unwrap();
    let states = riauth::capability::runtime(&f.core).unwrap();
    for name in [
        "identity.email_verification",
        "identity.invitations",
        "identity.email_password_reset",
    ] {
        assert_eq!(states["feature_states"][name]["configured"], true);
        assert_eq!(states["feature_states"][name]["usable"], true);
    }
    std::fs::write(&password_file, b"\n").unwrap();
    let states = riauth::capability::runtime(&f.core).unwrap();
    assert_eq!(
        states["feature_states"]["identity.email_verification"]["usable"],
        false
    );
}

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
fn encrypted_backup_restore_preserves_identity_and_keys_and_invalidates_grants() {
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
    // R04: the restored grant is invalidated; the identity and subject continue.
    assert!(restored.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(restored.doctor(&f.admin).is_err());
    let alice = text(
        &restored
            .login("alice".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let verifier = crypto::random_token("");
    let redirect = restored
        .authorize(&alice, f.request("app", &verifier))
        .unwrap();
    let code = url::Url::parse(&redirect)
        .unwrap()
        .query_pairs()
        .find(|(name, _)| name == "code")
        .unwrap()
        .1
        .into_owned();
    let fresh = restored
        .token(riauth::oidc::TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(code),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        restored.userinfo(&text(&fresh, "access_token")).unwrap()["sub"],
        expected["sub"]
    );
    let admin = text(
        &restored
            .login("admin".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    assert_eq!(restored.doctor(&admin).unwrap()["encrypted_at_rest"], true);
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
        "providers":[{"pk":1,"name":"app","client_id":"app","client_type":"public","grant_types":["authorization_code","refresh_token"],"redirect_uris":[{"matching_mode":"strict","url":"http://localhost:7777/callback?existing=1"}],"property_mappings":["profile-mapping"],"sub_mode":"hashed_user_id","issuer_mode":"per_provider","include_claims_in_id_token":true,"access_code_validity":"minutes=1","access_token_validity":"minutes=5","refresh_token_validity":"days=30"}],
        "applications":[{"slug":"app","provider":1,"name":"Team workspace","meta_launch_url":"https://app.example.test/","meta_description":"The team’s applications","group":"Engineering"}],"policy_bindings":[{"pk":"binding-1"}],"sources":[],
        "passwords":{"alice":{"reference":"env:ALICE_PASSWORD","version":"import-v1"}},
        "clients":{"app":{"issuer":format!("{}/application/o/app/", f.core.config.issuer),"scopes":["openid","profile","groups","offline_access"],"settings":{"groups_in_profile":true,"policy":{"access":{"all_groups":["engineering"]}}},"translated_mapping_ids":["profile-mapping"],"translated_binding_ids":["binding-1"],"authentication_flow_reviewed":true,"require_mfa":false}}});
    let report =
        riauth::migration::convert(serde_json::from_value(input.clone()).unwrap()).unwrap();
    assert_eq!(report["ready_for_plan"], true, "{report}");
    assert_eq!(report["summary"]["blocking"], 0);
    let classified = |kind: &str, id: &str| {
        report["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|i| i["kind"] == kind && i["id"] == id)
            .map(|i| i["classification"].as_str().unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(classified("group", "employees"), ["exact"]);
    assert_eq!(classified("group", "engineering"), ["convertible"]);
    assert_eq!(classified("subject", "app"), ["exact"]);
    assert_eq!(classified("issuer", "app"), ["exact"]);
    assert_eq!(
        report["manifest"]["clients"][0]["settings"]["issuer"],
        format!("{}/application/o/app/", f.core.config.issuer)
    );
    assert_eq!(classified("grant", "app"), ["exact"]);
    assert_eq!(classified("password", "alice"), ["manual"]);
    assert_eq!(classified("passkey", "*"), ["unsupported"]);
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
        "providers":[{"pk":1,"name":"app","client_id":"app","client_type":"public","grant_types":["authorization_code"],"redirect_uris":[{"matching_mode":"strict","url":"http://localhost:7777/callback"}],"property_mappings":[],"sub_mode":"user_username","issuer_mode":"per_provider","include_claims_in_id_token":true}],
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
fn authentik_preflight_classifies_every_exported_item() {
    let secret = "exported-client-secret-must-not-appear";
    let input = json!({"api_version":"riauth.authentik-import/v1","issuer":"https://id.example.test",
        "users":[
            {"pk":1,"uid":"a","username":"alice","name":"Alice","groups":["child"],"attributes":{},"type":"internal","is_active":true,"roles":[],"is_superuser":true},
            {"pk":2,"uid":"b","username":"bob","name":"Bob","groups":[],"attributes":{},"type":"internal","is_active":true,"roles":["role-uuid"]},
            {"pk":3,"uid":"c","username":"robot","name":"Robot","groups":[],"attributes":{},"type":"service_account","is_active":true,"roles":[]}],
        "groups":[
            {"pk":"child","name":"engineering","parents":["parent"]},
            {"pk":"parent","name":"admins","parents":[],"is_superuser":true,"attributes":{"cost_center":"42"}}],
        "providers":[
            {"pk":1,"name":"legacy","client_id":"legacy","client_type":"confidential","client_secret":secret,"signing_key":null,
             "redirect_uris":[{"matching_mode":"strict","url":"https://legacy.example.test/cb"},{"matching_mode":"regex","url":"https://.*\\.example\\.test/cb"},
                {"matching_mode":"strict","url":"https://legacy.example.test/bye","redirect_uri_type":"logout"}],
             "property_mappings":["mapped","unmapped"],"sub_mode":"user_upn","issuer_mode":"per_provider","include_claims_in_id_token":true,
             "logout_uri":"https://legacy.example.test/logout","logout_method":"backchannel"}],
        "applications":[
            {"slug":"legacy","provider":1,"name":"Legacy","meta_launch_url":"https://legacy.example.test/","meta_hide":true},
            {"slug":"wiki","provider":7,"name":"Wiki"},
            {"slug":"bookmark","provider":null,"name":"Bookmark"}],
        "policy_bindings":[{"pk":"bound"},{"pk":"loose"}],
        "sources":[
            {"pk":"inbuilt-uuid","slug":"authentik-built-in","managed":"goauthentik.io/sources/inbuilt","meta_model_name":"authentik_core.source","component":""},
            {"pk":"saml-uuid","slug":"corp","meta_model_name":"authentik_sources_saml.samlsource","user_matching_mode":"email_link"},
            {"pk":"ldap-uuid","slug":"dir","meta_model_name":"authentik_sources_ldap.ldapsource"},
            {"pk":"plex-uuid","slug":"plex","meta_model_name":"authentik_sources_plex.plexsource"}],
        "passwords":{"alice":{"reference":"file:alice-hash","version":"v1","hashed":true}},
        "totp":{"alice":{"reference":"file:alice-totp","version":"v1"}},
        "clients":{"legacy":{"issuer":"https://id.example.test/application/o/legacy/","scopes":["openid"],"settings":{"allowed_grants":["authorization_code"]},
            "translated_mapping_ids":["mapped"],"translated_binding_ids":["bound"],"authentication_flow_reviewed":true,"require_mfa":false}}});
    let report = riauth::migration::convert(serde_json::from_value(input).unwrap()).unwrap();
    assert_eq!(report["ready_for_plan"], false);
    assert!(!report.to_string().contains(secret));
    let items: Vec<riauth::migration::Finding> =
        serde_json::from_value(report["items"].clone()).unwrap();
    // Every blocker is traceable to a blocking finding and vice versa.
    let mut from_items = items
        .iter()
        .filter_map(|i| {
            assert_eq!(i.blocking, i.blocker.is_some(), "{i:?}");
            assert!(!i.reason.is_empty() && !i.action.is_empty(), "{i:?}");
            i.blocker.clone()
        })
        .collect::<Vec<_>>();
    from_items.sort();
    from_items.dedup();
    assert_eq!(json!(from_items), report["blockers"]);
    for (class, name) in [
        (Classification::Exact, "exact"),
        (Classification::Convertible, "convertible"),
        (Classification::Manual, "manual"),
        (Classification::Unsupported, "unsupported"),
    ] {
        let count = items.iter().filter(|i| i.classification == class).count();
        assert_eq!(report["summary"][name], count, "{name}");
    }
    assert_eq!(
        report["summary"]["blocking"],
        items.iter().filter(|i| i.blocking).count()
    );
    let find = |kind: ItemKind, id: &str| {
        let found = items
            .iter()
            .filter(|i| i.kind == kind && i.id == id)
            .map(|i| (i.classification, i.blocking))
            .collect::<Vec<_>>();
        assert!(!found.is_empty(), "{kind:?} {id} missing");
        found
    };
    use Classification::*;
    use riauth::migration::{Classification, ItemKind, ItemKind::*};
    for (kind, id, expected) in [
        (Source, "inbuilt-uuid", vec![(Convertible, false)]),
        (Source, "saml-uuid", vec![(Manual, true)]),
        (Source, "ldap-uuid", vec![(Unsupported, true)]),
        (Source, "plex-uuid", vec![(Unsupported, true)]),
        (Group, "engineering", vec![(Convertible, false)]),
        (
            Group,
            "admins",
            vec![(Exact, false), (Manual, false), (Manual, false)],
        ),
        (User, "alice", vec![(Convertible, false), (Manual, false)]),
        (User, "bob", vec![(Convertible, false), (Manual, true)]),
        (User, "robot", vec![(Convertible, false), (Manual, true)]),
        (Password, "alice", vec![(Convertible, false)]),
        (Password, "bob", vec![(Manual, true)]),
        (Totp, "alice", vec![(Convertible, false)]),
        (Totp, "*", vec![(Manual, false)]),
        (Passkey, "*", vec![(Unsupported, false)]),
        (Session, "*", vec![(Unsupported, false)]),
        (Provider, "legacy", vec![(Manual, false)]),
        (AuthenticationFlow, "legacy", vec![(Manual, false)]),
        (PropertyMapping, "legacy/mapped", vec![(Manual, false)]),
        (PropertyMapping, "legacy/unmapped", vec![(Manual, true)]),
        (PolicyBinding, "bound", vec![(Manual, false)]),
        (PolicyBinding, "loose", vec![(Manual, true)]),
        (SigningKey, "legacy", vec![(Manual, false)]),
        (ClientSecret, "legacy", vec![(Manual, true)]),
        (Grant, "legacy", vec![(Manual, false)]),
        (TokenLifetime, "legacy", vec![(Convertible, false)]),
        (
            RedirectUri,
            "legacy/https://legacy.example.test/cb",
            vec![(Exact, false)],
        ),
        (
            RedirectUri,
            "legacy/https://legacy.example.test/bye",
            vec![(Convertible, false)],
        ),
        (
            RedirectUri,
            "legacy/https://.*\\.example\\.test/cb",
            vec![(Unsupported, true)],
        ),
        (Logout, "legacy", vec![(Exact, false)]),
        (Subject, "legacy", vec![(Convertible, false)]),
        (Issuer, "legacy", vec![(Exact, false)]),
        (Application, "legacy", vec![(Convertible, false)]),
        (Application, "wiki", vec![(Unsupported, true)]),
        (Application, "bookmark", vec![(Unsupported, false)]),
    ] {
        let mut found = find(kind, id);
        found.sort();
        assert_eq!(found, expected, "{kind:?} {id}");
    }
    // A pre-2026.5 export without grant_types asks for an inventory instead of failing.
    assert!(
        report["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| !b.as_str().unwrap().contains("grants"))
    );
    assert_eq!(
        report["draft"]["clients"][0]["settings"]["app"]["hidden"],
        true
    );
}

#[test]
fn authentik_preflight_fails_closed_on_missing_or_mismatched_resolutions() {
    use riauth::migration::{Classification::*, Finding, ItemKind::*};
    let secret = "unresolved-client-secret-must-not-appear";
    // A valid reviewed OAuth-profile SourceSpec; only the OAuth source may accept it.
    let oauth_spec = |id: &str| {
        json!({"source":{"id":id,"name":"Replacement","issuer":"https://idp.example.test","authorization_endpoint":"https://idp.example.test/authorize",
            "token_endpoint":"https://idp.example.test/token","client_id":"riauth","token_endpoint_auth_method":"client_secret_post","scopes":["read:user"],
            "oauth_profile":{"userinfo_endpoint":"https://idp.example.test/me","subject_pointer":"/id"}},
            "secret_ref":"env:SOURCE_SECRET","secret_version":"v1"})
    };
    let input = json!({"api_version":"riauth.authentik-import/v1","issuer":"https://id.example.test",
        "users":[
            {"pk":1,"uid":"a","username":"alice","name":"Alice","email":"shared@example.test","groups":[],"attributes":{},"type":"internal","is_active":true,"roles":[]},
            {"pk":2,"uid":"b","username":"bob","name":"Bob","email":"shared@example.test","groups":[],"attributes":{},"type":"internal","is_active":true,"roles":[]}],
        "groups":[],
        "providers":[{"pk":1,"name":"orphan","client_id":"orphan","client_type":"confidential","client_secret":secret,
            "signing_key":"key-uuid","encryption_key":"enc-uuid","jwt_federation_sources":["fed-uuid"],"grant_types":["authorization_code"],
            "redirect_uris":[{"matching_mode":"strict","url":"https://orphan.example.test/cb"},{"matching_mode":"regex","url":"https://.*\\.orphan\\.test/cb"}],
            "property_mappings":["m1"],"sub_mode":"user_email","include_claims_in_id_token":true,
            "logout_uri":"https://orphan.example.test/logout","logout_method":"backchannel"}],
        "applications":[
            {"slug":"orphan-app","provider":1,"name":"Orphan","meta_launch_url":"https://orphan.example.test/"},
            {"slug":"orphan-twin","provider":1,"name":"Twin"}],
        "policy_bindings":[{"pk":"b1"}],
        "sources":[
            {"pk":"inbuilt-uuid","managed":"goauthentik.io/sources/inbuilt","meta_model_name":"authentik_core.source","component":""},
            {"pk":"oauth-uuid","meta_model_name":"authentik_sources_oauth.oauthsource","user_matching_mode":"email_link"},
            {"pk":"saml-uuid","meta_model_name":"authentik_sources_saml.samlsource"},
            {"pk":"ldap-uuid","meta_model_name":"authentik_sources_ldap.ldapsource"},
            {"pk":"plex-uuid","meta_model_name":"authentik_sources_plex.plexsource"}],
        "source_resolutions":{"inbuilt-uuid":oauth_spec("as-inbuilt"),"oauth-uuid":oauth_spec("corp-oauth"),"saml-uuid":oauth_spec("as-saml"),
            "ldap-uuid":oauth_spec("as-ldap"),"plex-uuid":oauth_spec("as-plex"),"gone-uuid":oauth_spec("as-gone")},
        "passwords":{"alice":{"reference":"env:ALICE","version":"v1"},"bob":{"reference":"env:BOB","version":"v1"}},
        "clients":{"ghost":{"issuer":"https://id.example.test","scopes":["openid"],"settings":{},"translated_mapping_ids":[],
            "translated_binding_ids":["b1"],"authentication_flow_reviewed":true,"require_mfa":false}}});
    let report =
        riauth::migration::convert(serde_json::from_value(input.clone()).unwrap()).unwrap();
    assert_eq!(report["ready_for_plan"], false);
    assert!(report["manifest"].is_null());
    assert!(!report.to_string().contains(secret));
    // Nothing unreviewed or rejected is converted: no client, no unreviewed subjects, and
    // only the matching OAuth resolution becomes a source.
    assert_eq!(report["draft"]["clients"], json!([]));
    assert!(
        report["draft"]["users"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["subjects"] == json!({}))
    );
    let sources = report["draft"]["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0]["source"]["id"], "corp-oauth");
    let items: Vec<Finding> = serde_json::from_value(report["items"].clone()).unwrap();
    let mut from_items = items
        .iter()
        .filter_map(|i| {
            assert_eq!(i.blocking, i.blocker.is_some(), "{i:?}");
            assert!(!i.reason.is_empty() && !i.action.is_empty(), "{i:?}");
            i.blocker.clone()
        })
        .collect::<Vec<_>>();
    from_items.sort();
    from_items.dedup();
    assert_eq!(json!(from_items), report["blockers"]);
    for (class, name) in [
        (Exact, "exact"),
        (Convertible, "convertible"),
        (Manual, "manual"),
        (Unsupported, "unsupported"),
    ] {
        assert_eq!(
            report["summary"][name],
            items.iter().filter(|i| i.classification == class).count(),
            "{name}"
        );
    }
    assert_eq!(
        report["summary"]["blocking"],
        items.iter().filter(|i| i.blocking).count()
    );
    let found = |kind, id: &str| {
        let mut found = items
            .iter()
            .filter(|i| i.kind == kind && i.id == id)
            .map(|i| (i.classification, i.blocking))
            .collect::<Vec<_>>();
        found.sort();
        found
    };
    // Every exported item has at least one finding.
    let exported = |collection: &str, key: &str| {
        input[collection]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v[key].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    for (kind, ids) in [
        (User, exported("users", "username")),
        (Password, exported("users", "username")),
        (Application, exported("applications", "slug")),
        (Source, exported("sources", "pk")),
        (PolicyBinding, exported("policy_bindings", "pk")),
        (PropertyMapping, vec!["orphan/m1".into()]),
        (
            RedirectUri,
            input["providers"][0]["redirect_uris"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| format!("orphan/{}", r["url"].as_str().unwrap()))
                .collect(),
        ),
    ] {
        for id in ids {
            assert!(!found(kind, &id).is_empty(), "{kind:?} {id} has no finding");
        }
    }
    for (kind, id, expected) in [
        (Provider, "orphan", vec![(Manual, true)]),
        (AuthenticationFlow, "orphan", vec![(Manual, true)]),
        (PropertyMapping, "orphan/m1", vec![(Manual, true)]),
        (
            Federation,
            "orphan/jwt_federation_sources",
            vec![(Manual, true)],
        ),
        (EncryptionKey, "orphan", vec![(Manual, true)]),
        (SigningKey, "orphan", vec![(Manual, false)]),
        (ClientSecret, "orphan", vec![(Manual, true)]),
        (Grant, "orphan", vec![(Exact, false)]),
        (TokenLifetime, "orphan", vec![(Convertible, false)]),
        (
            RedirectUri,
            "orphan/https://orphan.example.test/cb",
            vec![(Exact, false)],
        ),
        (
            RedirectUri,
            "orphan/https://.*\\.orphan\\.test/cb",
            vec![(Unsupported, true)],
        ),
        (Logout, "orphan", vec![(Exact, false)]),
        // Duplicate subjects are detected even though the client is not reviewed yet.
        (
            Subject,
            "orphan",
            vec![(Convertible, false), (Unsupported, true)],
        ),
        (Issuer, "orphan", vec![(Manual, true)]),
        (
            Application,
            "orphan-app",
            vec![(Convertible, true), (Manual, true)],
        ),
        (Application, "orphan-twin", vec![(Manual, true)]),
        // Only the stale `ghost` entry translates b1, and stale entries are never applied.
        (PolicyBinding, "b1", vec![(Manual, true)]),
        (Provider, "ghost", vec![(Manual, true)]),
        (
            Source,
            "inbuilt-uuid",
            vec![(Convertible, false), (Manual, true)],
        ),
        (Source, "oauth-uuid", vec![(Manual, false)]),
        (Source, "saml-uuid", vec![(Manual, true)]),
        (
            Source,
            "ldap-uuid",
            vec![(Unsupported, true), (Unsupported, true)],
        ),
        (
            Source,
            "plex-uuid",
            vec![(Unsupported, true), (Unsupported, true)],
        ),
        (Source, "gone-uuid", vec![(Manual, true)]),
    ] {
        assert_eq!(found(kind, id), expected, "{kind:?} {id}");
    }
    for blocker in [
        "orphan: provide reviewed issuer, scopes, mappings, policies and authentication requirements in clients",
        "orphan: duplicate subjects would merge identities",
        "ghost: clients entry does not match an exported provider",
        "Source plex-uuid: a source resolution cannot replace an unsupported plex source",
        "Source ldap-uuid: a source resolution cannot replace an unsupported ldap source",
        "Source inbuilt-uuid: the built-in source cannot take a source resolution",
        "Source saml-uuid: the source resolution must use the SAML adapter",
        "Source gone-uuid: source resolution does not match an exported source",
    ] {
        assert!(
            report["blockers"]
                .as_array()
                .unwrap()
                .contains(&json!(blocker)),
            "{blocker}"
        );
    }
    let oauth = items
        .iter()
        .find(|i| i.kind == Source && i.id == "oauth-uuid")
        .unwrap();
    assert!(
        oauth.reason.contains("email_link")
            && oauth.action.contains("never links accounts by email")
    );

    // The source-aware entry point returns the same findings for the same stale and mismatched
    // resolutions, and never a manifest or draft.
    let entry = riauth::migration::preflight(&serde_json::to_vec(&input).unwrap()).unwrap();
    assert_eq!(entry["source"]["converter"], "authentik");
    for field in [
        "api_version",
        "ready_for_plan",
        "blockers",
        "summary",
        "items",
    ] {
        assert_eq!(entry[field], report[field], "{field}");
    }
    assert!(entry.get("manifest").is_none() && entry.get("draft").is_none());
    assert!(!entry.to_string().contains(secret));
}

/// Sorted (classification, blocking) pairs of a report's findings for one kind and ID.
fn findings(
    report: &Value,
    kind: riauth::migration::ItemKind,
    id: &str,
) -> Vec<(riauth::migration::Classification, bool)> {
    let items: Vec<riauth::migration::Finding> =
        serde_json::from_value(report["items"].clone()).unwrap();
    let mut found = items
        .iter()
        .filter(|i| i.kind == kind && i.id == id)
        .map(|i| (i.classification, i.blocking))
        .collect::<Vec<_>>();
    found.sort();
    found
}

#[test]
fn authentik_preflight_keeps_or_blocks_each_exported_issuer() {
    use riauth::migration::{Classification::*, ItemKind::*};
    let provider = |pk: u64, cid: &str, mode: Value| {
        json!({"pk":pk,"name":cid,"client_id":cid,"client_type":"public","grant_types":["authorization_code"],
            "redirect_uris":[{"matching_mode":"strict","url":format!("https://{cid}.example.test/cb")}],
            "property_mappings":[],"sub_mode":"user_uuid","issuer_mode":mode,"include_claims_in_id_token":true})
    };
    let client = |issuer: &str| {
        json!({"issuer":issuer,"scopes":["openid"],"settings":{},"translated_mapping_ids":[],"translated_binding_ids":[],
            "authentication_flow_reviewed":true,"require_mfa":false})
    };
    let providers = [
        ("wiki", json!("per_provider")),
        ("chat", json!("per_provider")),
        ("grafana", json!("global")),
        ("vault", json!("global")),
        ("docs", json!("global")),
        ("git", json!("global")),
        ("legacy", Value::Null),
        ("odd", json!("per_tenant")),
    ];
    let input = json!({"api_version":"riauth.authentik-import/v1","issuer":"https://id.example.test",
        "users":[{"pk":1,"uuid":"user-1","uid":"a","username":"alice","name":"Alice","groups":[],"attributes":{},"type":"internal","is_active":true,"roles":[]}],
        "groups":[],"policy_bindings":[],"sources":[],
        "providers":providers.iter().enumerate().map(|(i, (cid, mode))| provider(i as u64 + 1, cid, mode.clone())).collect::<Vec<_>>(),
        "applications":providers.iter().enumerate().map(|(i, (cid, _))| json!({"slug":cid,"provider":i + 1,"name":cid})).collect::<Vec<_>>(),
        "passwords":{"alice":{"reference":"env:ALICE_PASSWORD","version":"v1"}},
        "clients":{
            "wiki":client("https://auth.example.test/application/o/wiki/"),
            // Another application's issuer: its relying party would see a different issuer.
            "chat":client("https://auth.example.test/application/o/wiki-chat/"),
            // One global Authentik issuer, but riAuth serves a non-primary issuer for one client.
            "grafana":client("https://auth.example.test/"),
            "vault":client("https://auth.example.test/"),
            // Authentik's global issuer ends in a slash; this one would differ from it.
            "docs":client("https://auth.example.test"),
            // Shares riAuth's discovery URL without being riAuth's issuer.
            "git":client("https://id.example.test/"),
            "legacy":client("https://auth.example.test/application/o/legacy/"),
            "odd":client("https://auth.example.test/application/o/odd/")}});
    let report =
        riauth::migration::convert(serde_json::from_value(input.clone()).unwrap()).unwrap();
    assert_eq!(report["ready_for_plan"], false);
    for (cid, expected) in [
        ("wiki", vec![(Exact, false)]),
        ("chat", vec![(Manual, true)]),
        ("grafana", vec![(Exact, false), (Unsupported, true)]),
        ("vault", vec![(Exact, false), (Unsupported, true)]),
        ("docs", vec![(Manual, true)]),
        ("git", vec![(Exact, false), (Unsupported, true)]),
        ("legacy", vec![(Manual, true)]),
        ("odd", vec![(Unsupported, true)]),
    ] {
        assert_eq!(findings(&report, Issuer, cid), expected, "{cid}");
    }
    for blocker in [
        "chat: reviewed issuer does not match the exported per_provider issuer",
        "docs: reviewed issuer does not match the exported global issuer",
        "Issuer https://auth.example.test/: several providers share it, but it is not riAuth's issuer",
        "git: issuer collides with riAuth's own issuer",
        "legacy: export issuer_mode so the reviewed issuer can be checked",
        "odd: unknown issuer mode per_tenant",
    ] {
        assert!(
            report["blockers"]
                .as_array()
                .unwrap()
                .contains(&json!(blocker)),
            "{blocker}: {}",
            report["blockers"]
        );
    }
    // Serving the shared global issuer as riAuth's own issuer keeps it for every provider.
    let mut shared = input.clone();
    shared["issuer"] = json!("https://auth.example.test/");
    shared["providers"] = json!([
        provider(3, "grafana", json!("global")),
        provider(4, "vault", json!("global"))
    ]);
    shared["applications"] = json!([{"slug":"grafana","provider":3,"name":"grafana"},{"slug":"vault","provider":4,"name":"vault"}]);
    for cid in ["wiki", "chat", "docs", "git", "legacy", "odd"] {
        shared["clients"].as_object_mut().unwrap().remove(cid);
    }
    let report = riauth::migration::convert(serde_json::from_value(shared).unwrap()).unwrap();
    assert_eq!(report["ready_for_plan"], true, "{}", report["blockers"]);
    for (index, cid) in ["grafana", "vault"].into_iter().enumerate() {
        assert_eq!(findings(&report, Issuer, cid), [(Exact, false)], "{cid}");
        assert_eq!(report["manifest"]["clients"][index]["client_id"], cid);
        assert!(report["manifest"]["clients"][index]["settings"]["issuer"].is_null());
    }
}

#[test]
fn authentik_application_bindings_convert_exactly_or_block() {
    use riauth::migration::{Classification::*, ItemKind::*};
    let f = Fixture::new();
    let issuer = f.core.config.issuer.clone();
    let user = |pk: u64, username: &str, kind: &str, groups: &[&str]| {
        json!({"pk":pk,"uid":format!("uid-{pk}"),"username":username,"name":username,"groups":groups,
            "attributes":{},"type":kind,"is_active":true,"roles":[]})
    };
    // (slug, policy_engine_mode, acknowledged binding IDs, reviewed settings.policy.access)
    let apps: [(&str, Value, &[&str], Value); 15] = [
        ("wiki", json!("any"), &[], json!({})),
        ("vault", json!("all"), &[], json!({})),
        ("lab", json!("all"), &[], json!({})),
        ("payroll", json!("any"), &[], json!({})),
        ("kiosk", json!("any"), &[], json!({})),
        ("desk", json!("any"), &[], json!({})),
        // Acknowledging an any-mode alternative accepts narrower access.
        ("chat", json!("any"), &["chat-bob", "chat-ops"], json!({})),
        ("admin", json!("all"), &[], json!({})),
        ("docs", json!("any"), &[], json!({})),
        ("hr", Value::Null, &[], json!({})),
        // Adversarial: acknowledgements and reviewed restrictions never clear a condition
        // riAuth cannot keep: disjoint user lists, several users in all mode, an unknown mode,
        // and required expression or expiring conditions.
        (
            "roster",
            json!("any"),
            &["roster-alice"],
            json!({"users":["bob"]}),
        ),
        ("pair", json!("all"), &["pair-alice", "pair-bob"], json!({})),
        (
            "mystery",
            json!("first_match"),
            &["mystery-staff"],
            json!({"any_groups":["staff"]}),
        ),
        ("notes", json!("all"), &["notes-policy"], json!({})),
        (
            "timed",
            json!("any"),
            &["timed-staff"],
            json!({"all_groups":["staff"]}),
        ),
    ];
    let binding = |pk: &str, fields: Value| {
        let slug = pk.split('-').next().unwrap();
        let mut binding = json!({"pk":pk,"target":format!("app-{slug}"),"policy":null,"group":null,"user":null,
            "negate":false,"enabled":true,"order":0});
        binding
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        binding
    };
    let bindings = [
        // Any mode: positive groups become any-of allowed groups; disabled bindings do nothing.
        binding("wiki-staff", json!({"group":"g-staff"})),
        binding("wiki-ops", json!({"group":"g-ops","enabled":false})),
        // All mode: each static binding becomes one required condition.
        binding("vault-staff", json!({"group":"g-staff"})),
        binding("vault-engineering", json!({"group":"g-eng"})),
        binding("lab-staff", json!({"group":"g-staff"})),
        binding("lab-ops", json!({"group":"g-ops","negate":true})),
        binding("lab-carol", json!({"user":3,"negate":true})),
        // A single binding is one required condition whatever the mode.
        binding("payroll-dave", json!({"user":4})),
        binding("kiosk-ops", json!({"group":"g-ops","negate":true})),
        // Any mode without groups: positive users become the users allow-list.
        binding("desk-carol", json!({"user":3})),
        binding("desk-dave", json!({"user":4})),
        // Alternatives riAuth cannot OR with the converted groups narrow access once acknowledged.
        binding("chat-staff", json!({"group":"g-staff"})),
        binding("chat-bob", json!({"user":2})),
        binding("chat-ops", json!({"group":"g-ops","negate":true})),
        // Nobody is two users at once, so Authentik admits no one.
        binding("admin-alice", json!({"user":1})),
        binding("admin-bob", json!({"user":2})),
        // No alternative converts, so leaving these out could not narrow access.
        binding(
            "docs-expiring",
            json!({"group":"g-staff","expiring":true,"expires":"2030-01-01T00:00:00Z"}),
        ),
        binding("docs-admins", json!({"group":"g-admins"})),
        binding("docs-outpost", json!({"user":5})),
        // Without a mode, several bindings cannot be combined.
        binding("hr-staff", json!({"group":"g-staff"})),
        binding("hr-ops", json!({"group":"g-ops"})),
        binding("roster-alice", json!({"user":1})),
        binding("pair-staff", json!({"group":"g-staff"})),
        binding("pair-alice", json!({"user":1})),
        binding("pair-bob", json!({"user":2})),
        binding("mystery-staff", json!({"group":"g-staff"})),
        binding("notes-staff", json!({"group":"g-staff"})),
        binding("notes-policy", json!({"policy":"policy-uuid"})),
        binding(
            "timed-staff",
            json!({"group":"g-staff","expiring":true,"expires":"2030-01-01T00:00:00Z"}),
        ),
    ];
    let bundle = |slugs: &[&str]| {
        let chosen = apps
            .iter()
            .enumerate()
            .filter(|(_, (slug, ..))| slugs.contains(slug));
        json!({"api_version":"riauth.authentik-import/v1","issuer":issuer,
            "users":[user(1, "alice", "internal", &["g-eng"]), user(2, "bob", "internal", &["g-staff", "g-ops"]),
                user(3, "carol", "internal", &["g-staff"]), user(4, "dave", "internal", &[]),
                user(5, "ak-outpost-x", "internal_service_account", &[])],
            "groups":[{"pk":"g-staff","name":"staff","parents":[]},{"pk":"g-eng","name":"engineering","parents":["g-staff"]},
                {"pk":"g-ops","name":"ops","parents":[]},{"pk":"g-admins","name":"authentik Admins","parents":[]}],
            "excluded_groups":["g-admins"],
            "providers":chosen.clone().map(|(i, (slug, ..))| json!({"pk":i + 1,"name":slug,"client_id":slug,"client_type":"public",
                "grant_types":["authorization_code"],"redirect_uris":[{"matching_mode":"strict","url":"http://localhost:7777/callback?existing=1"}],
                "property_mappings":[],"sub_mode":"hashed_user_id","issuer_mode":"per_provider","include_claims_in_id_token":true})).collect::<Vec<_>>(),
            "applications":chosen.clone().map(|(i, (slug, mode, ..))| json!({"pk":format!("app-{slug}"),"slug":slug,"provider":i + 1,
                "name":slug,"policy_engine_mode":mode})).collect::<Vec<_>>(),
            "policy_bindings":bindings.iter().filter(|b| slugs.contains(&b["target"].as_str().unwrap().trim_start_matches("app-"))).collect::<Vec<_>>(),
            "sources":[],
            "passwords":{"alice":{"reference":"env:ALICE","version":"v1"},"bob":{"reference":"env:BOB","version":"v1"},
                "carol":{"reference":"env:CAROL","version":"v1"},"dave":{"reference":"env:DAVE","version":"v1"}},
            "clients":chosen.map(|(_, (slug, _, acknowledged, access))| (slug.to_string(), json!({"issuer":format!("{issuer}/application/o/{slug}/"),
                "scopes":["openid","profile"],"settings":{"policy":{"access":access}},"translated_mapping_ids":[],
                "translated_binding_ids":acknowledged,"authentication_flow_reviewed":true,"require_mfa":false}))).collect::<serde_json::Map<_, _>>()})
    };
    let convert =
        |input: Value| riauth::migration::convert(serde_json::from_value(input).unwrap()).unwrap();
    let report = convert(bundle(
        &apps.iter().map(|(slug, ..)| *slug).collect::<Vec<_>>(),
    ));
    assert_eq!(report["ready_for_plan"], false);
    for (id, expected) in [
        ("wiki-staff", (Convertible, false)),
        ("wiki-ops", (Exact, false)),
        ("vault-staff", (Convertible, false)),
        ("vault-engineering", (Convertible, false)),
        ("lab-staff", (Convertible, false)),
        ("lab-ops", (Convertible, false)),
        ("lab-carol", (Convertible, false)),
        ("payroll-dave", (Convertible, false)),
        ("kiosk-ops", (Convertible, false)),
        ("desk-carol", (Convertible, false)),
        ("desk-dave", (Convertible, false)),
        ("chat-staff", (Convertible, false)),
        ("chat-bob", (Manual, false)),
        ("chat-ops", (Manual, false)),
        ("admin-alice", (Unsupported, true)),
        ("admin-bob", (Unsupported, true)),
        ("docs-expiring", (Unsupported, true)),
        ("docs-admins", (Unsupported, true)),
        ("docs-outpost", (Unsupported, true)),
        ("hr-staff", (Unsupported, true)),
        ("hr-ops", (Unsupported, true)),
        ("roster-alice", (Unsupported, true)),
        ("pair-staff", (Convertible, false)),
        ("pair-alice", (Unsupported, true)),
        ("pair-bob", (Unsupported, true)),
        ("mystery-staff", (Unsupported, true)),
        ("notes-staff", (Convertible, false)),
        ("notes-policy", (Unsupported, true)),
        ("timed-staff", (Unsupported, true)),
    ] {
        assert_eq!(findings(&report, PolicyBinding, id), [expected], "{id}");
    }
    // Acknowledged or not, a condition riAuth cannot keep stays a blocker.
    for id in [
        "roster-alice",
        "pair-alice",
        "pair-bob",
        "mystery-staff",
        "notes-policy",
        "timed-staff",
    ] {
        let slug = id.split('-').next().unwrap();
        let blocker = format!(
            "{slug}: application binding {id} cannot be kept exactly and blocks until changed in Authentik"
        );
        assert!(
            report["blockers"]
                .as_array()
                .unwrap()
                .contains(&json!(blocker)),
            "{blocker}"
        );
    }
    // A client its bindings restricted, but which would admit every user, blocks as well.
    for (slug, ..) in &apps {
        let guarded = findings(&report, Application, slug).contains(&(Manual, true));
        assert_eq!(guarded, ["admin", "docs", "hr"].contains(slug), "{slug}");
    }
    let access = |report: &Value, cid: &str| {
        let client = report["draft"]["clients"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["client_id"] == cid)
            .unwrap()
            .clone();
        let rule = &client["settings"]["policy"]["access"];
        [
            client["allowed_groups"].clone(),
            rule["all_groups"].clone(),
            rule["denied_groups"].clone(),
            rule["users"].clone(),
            rule["denied_users"].clone(),
        ]
    };
    for (cid, expected) in [
        (
            "wiki",
            [json!(["staff"]), json!([]), json!([]), json!([]), json!([])],
        ),
        (
            "vault",
            [
                json!([]),
                json!(["engineering", "staff"]),
                json!([]),
                json!([]),
                json!([]),
            ],
        ),
        (
            "lab",
            [
                json!([]),
                json!(["staff"]),
                json!(["ops"]),
                json!([]),
                json!(["carol"]),
            ],
        ),
        (
            "payroll",
            [json!([]), json!([]), json!([]), json!(["dave"]), json!([])],
        ),
        (
            "kiosk",
            [json!([]), json!([]), json!(["ops"]), json!([]), json!([])],
        ),
        (
            "desk",
            [
                json!([]),
                json!([]),
                json!([]),
                json!(["carol", "dave"]),
                json!([]),
            ],
        ),
        (
            "chat",
            [json!(["staff"]), json!([]), json!([]), json!([]), json!([])],
        ),
        (
            "admin",
            [json!([]), json!([]), json!([]), json!([]), json!([])],
        ),
    ] {
        assert_eq!(access(&report, cid), expected, "{cid}");
    }

    // The exactly converted applications plan and apply, and admit exactly whom Authentik did;
    // the acknowledged chat alternatives only narrow it to staff.
    let report = convert(bundle(&[
        "wiki", "vault", "lab", "payroll", "kiosk", "desk", "chat",
    ]));
    assert_eq!(report["ready_for_plan"], true, "{}", report["blockers"]);
    let plan = f
        .core
        .plan_state(
            &f.admin,
            serde_json::from_value(report["manifest"].clone()).unwrap(),
        )
        .unwrap();
    let secrets =
        ["ALICE", "BOB", "CAROL", "DAVE"].map(|name| (format!("env:{name}"), PASSWORD.to_owned()));
    f.core
        .apply_state(
            &f.admin,
            riauth::state::ApplyRequest {
                plan,
                secrets: secrets.into(),
                run_id: None,
            },
        )
        .unwrap();
    let sessions = ["alice", "bob", "carol", "dave"].map(|username| {
        (
            username,
            text(
                &f.core
                    .login(username.into(), PASSWORD.into(), None)
                    .unwrap(),
                "session_token",
            ),
        )
    });
    for (cid, admitted) in [
        ("wiki", ["alice", "bob", "carol"].as_slice()),
        ("vault", &["alice"]),
        ("lab", &["alice"]),
        ("payroll", &["dave"]),
        ("kiosk", &["alice", "carol", "dave"]),
        ("desk", &["carol", "dave"]),
        // Authentik also admitted Dave, who is not in ops; the narrowed client refuses him.
        ("chat", &["alice", "bob", "carol"]),
    ] {
        for (username, session) in &sessions {
            let mut request = f.request(cid, &crypto::random_token(""));
            request.scope = "openid profile".into();
            assert_eq!(
                f.core.authorize(session, request).is_ok(),
                admitted.contains(username),
                "{cid} {username}"
            );
        }
    }
}

#[test]
fn authentik_membership_expressions_factor_into_two_lists_or_block() {
    use riauth::migration::{Classification::*, ItemKind::*};
    let f = Fixture::new();
    let issuer = f.core.config.issuer.clone();
    let check = |group: &str| format!("ak_is_group_member(request.user, name=\"{group}\")");
    let (staff, ops, lab, research) = (
        check("staff"),
        check("ops"),
        check("lab"),
        check("research"),
    );
    let chain = |groups: std::ops::RangeInclusive<u32>| {
        groups
            .map(|i| check(&format!("t{i}")))
            .collect::<Vec<_>>()
            .join(" or ")
    };
    let expressions = [
        ("both", format!("return {staff} and not {ops}")),
        ("either", format!("return {staff} or {lab}")),
        ("notboth", format!("return not {staff} and not {lab}")),
        // `not` binds tighter than `and`, and `and` tighter than `or`.
        ("precedence", format!("return not {ops} and {staff}")),
        (
            "notops",
            "return not ak_is_group_member(request.user, name='ops')".to_owned(),
        ),
        (
            "spaced",
            "\n\t return\tak_is_group_member( request.user ,\tname = \"ops\" )\n".to_owned(),
        ),
        (
            "factored",
            format!("return {staff} and {lab} or {staff} and {ops}"),
        ),
        (
            "factoredneg",
            format!("return {lab} and not {ops} or {staff} and not {ops}"),
        ),
        (
            "notstaffchain",
            format!("return not {staff} or not {lab} and not {ops}"),
        ),
        ("redundant", format!("return {staff} or {staff} and {ops}")),
        (
            "absorbneg",
            format!("return {staff} and not {ops} or {staff} and not {ops} and {lab}"),
        ),
        ("twelve", format!("return {}", chain(1..=12))),
        // Two any-of lists, written with and without parentheses.
        ("mixed", format!("return {staff} and {ops} or {lab}")),
        ("mixed2", format!("return {staff} or {ops} and {lab}")),
        ("paren", format!("return ({staff} or {ops})")),
        (
            "twolist",
            format!("return ({staff} or {lab}) and ({ops} or {research})"),
        ),
        (
            "twolistneg",
            format!(
                "return ({staff} or {lab}) and ({ops} or {research}) and not {}",
                check("engineering")
            ),
        ),
        (
            "twolistdnf",
            format!(
                "return {staff} and {ops} or {staff} and {research} or {lab} and {ops} or {lab} and {research}"
            ),
        ),
        (
            "negtwolistsrc",
            format!("return not {staff} and not {lab} or not {ops} and not {research}"),
        ),
        (
            "overlap",
            format!("return ({staff} or {lab}) and ({staff} or {ops})"),
        ),
        (
            "parensnot",
            format!("return ({staff}) and not ({ops} or {lab})"),
        ),
        (
            "nested",
            format!("return ((({staff}) or {lab}) and (({ops}) or {research}))"),
        ),
        (
            "tight",
            format!("return ({staff} or {lab})and({ops} or {research})"),
        ),
        // These do not factor.
        (
            "threelists",
            format!(
                "return ({staff} or {lab}) and ({ops} or {research}) and ({})",
                chain(1..=2)
            ),
        ),
        (
            "nonmono",
            format!("return ({staff} or not {lab}) and ({ops} or {research})"),
        ),
        ("negor", format!("return {staff} or not {ops}")),
        ("tautology", format!("return {staff} or not {staff}")),
        ("toomany", format!("return {}", chain(1..=13))),
        ("contradiction", format!("return {staff} and not {staff}")),
        // Outside the grammar.
        (
            "deep",
            format!("return {}{staff}{}", "(".repeat(9), ")".repeat(9)),
        ),
        ("unbalanced", format!("return ({staff} or {ops}")),
        ("nbsp", format!("return\u{a0}{staff}")),
        ("formfeed", format!("return {staff}\u{c} or {ops}")),
        ("newline", format!("return {staff} or\n{ops}")),
        (
            "call",
            format!("return {staff} or request.user.is_superuser"),
        ),
        ("comment", format!("return {staff}  # or {ops}")),
        // Python would decode this escape to "staff"; riAuth never evaluates escapes.
        (
            "escape",
            "return ak_is_group_member(request.user, name=\"st\\u0061ff\")".to_owned(),
        ),
        ("doubleeq", format!("return {staff} == True")),
        (
            "long",
            format!("return {}", vec![staff.clone(); 33].join(" or ")),
        ),
        ("ghost", format!("return {staff} or {}", check("ghosts"))),
        (
            "admins",
            format!("return {staff} and not {}", check("authentik Admins")),
        ),
        ("twin", format!("return {} or {staff}", check("archive"))),
        ("list-b", format!("return {ops} or {lab}")),
        (
            "secret",
            r#"return request.context.get("token") == "s3cr3t-token""#.to_owned(),
        ),
    ];
    let policies = expressions
        .iter()
        .map(|(pk, source)| json!({"pk":format!("p-{pk}"),"name":format!("policy {pk}"),"expression":source}))
        .collect::<Vec<_>>();
    let binding = |slug: &str, suffix: &str, fields: Value| {
        let mut binding = json!({"pk":format!("{slug}-{suffix}"),"target":format!("app-{slug}"),"policy":null,
            "group":null,"user":null,"negate":false,"enabled":true,"order":0});
        binding
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        binding
    };
    let policy =
        |slug: &str, pk: &str| binding(slug, "policy", json!({"policy":format!("p-{pk}")}));
    let negated = |slug: &str, pk: &str| {
        binding(
            slug,
            "policy",
            json!({"policy":format!("p-{pk}"),"negate":true}),
        )
    };
    let extra = |slug: &str, suffix: &str, pk: &str| {
        binding(slug, suffix, json!({"policy":format!("p-{pk}")}))
    };
    // (slug, policy_engine_mode, bindings, acknowledged binding IDs, reviewed settings.policy.access)
    let app =
        |slug: &str, mode: &'static str, bindings: Vec<Value>, acknowledged: Vec<&'static str>| {
            (slug.to_owned(), mode, bindings, acknowledged, json!({}))
        };
    let single = |slug: &str| app(slug, "any", vec![policy(slug, slug)], vec![]);
    let mut apps = vec![
        single("both"),
        single("either"),
        // A negated binding of a disjunction refuses every named group, and the reverse.
        app("neither", "any", vec![negated("neither", "either")], vec![]),
        app("anyof", "any", vec![negated("anyof", "notboth")], vec![]),
        single("precedence"),
        app("opsonly", "any", vec![negated("opsonly", "notops")], vec![]),
        single("spaced"),
        // Any mode: a disjunction joins the other alternatives' allowed groups.
        app(
            "anymix",
            "any",
            vec![
                binding("anymix", "lab", json!({"group":"g-lab"})),
                policy("anymix", "either"),
            ],
            vec![],
        ),
        // All mode: any-of lists sit beside the other required conditions.
        app(
            "allpair",
            "all",
            vec![
                policy("allpair", "either"),
                binding("allpair", "ops", json!({"group":"g-ops","negate":true})),
            ],
            vec![],
        ),
        single("factored"),
        single("factoredneg"),
        // `not staff or (not lab and not ops)`, negated, is `staff and (lab or ops)`.
        app(
            "negfactored",
            "any",
            vec![negated("negfactored", "notstaffchain")],
            vec![],
        ),
        single("redundant"),
        single("absorbneg"),
        single("twelve"),
        single("mixed"),
        single("mixed2"),
        single("paren"),
        single("twolist"),
        single("twolistneg"),
        single("twolistdnf"),
        // `(not staff and not lab) or (not ops and not research)`, negated, is two lists.
        app(
            "negtwolist",
            "any",
            vec![negated("negtwolist", "negtwolistsrc")],
            vec![],
        ),
        single("overlap"),
        single("parensnot"),
        single("nested"),
        single("tight"),
        // All mode: two bindings fill allowed_groups and settings.policy.access.any_groups.
        app(
            "twolists",
            "all",
            vec![
                policy("twolists", "either"),
                extra("twolists", "second", "list-b"),
            ],
            vec![],
        ),
        app(
            "twomixed",
            "all",
            vec![
                policy("twomixed", "factored"),
                extra("twomixed", "second", "list-b"),
            ],
            vec![],
        ),
        // Acknowledging an any-mode alternative that does not convert accepts narrower access.
        app(
            "narrowacked",
            "any",
            vec![
                binding("narrowacked", "lab", json!({"group":"g-lab"})),
                policy("narrowacked", "twolist"),
            ],
            vec!["narrowacked-policy"],
        ),
    ];
    let ready = apps
        .iter()
        .map(|(slug, ..)| slug.clone())
        .collect::<Vec<_>>();
    for slug in [
        "threelists",
        "nonmono",
        "negor",
        "tautology",
        "toomany",
        "contradiction",
        "deep",
        "unbalanced",
        "nbsp",
        "formfeed",
        "newline",
        "call",
        "comment",
        "escape",
        "doubleeq",
        "long",
        "ghost",
        "admins",
        "twin",
        "secret",
    ] {
        apps.push(single(slug));
    }
    apps.extend([
        // `staff and not ops`, negated, admits users outside every named group.
        app("negand", "any", vec![negated("negand", "both")], vec![]),
        // Acknowledgement never clears a required formula that does not factor.
        app(
            "acked",
            "any",
            vec![policy("acked", "negor")],
            vec!["acked-policy"],
        ),
        // riAuth ANDs at most two any-of lists per client.
        app(
            "threebind",
            "all",
            vec![
                policy("threebind", "either"),
                extra("threebind", "second", "list-b"),
                extra("threebind", "third", "paren"),
            ],
            vec![],
        ),
        app(
            "listsplus",
            "all",
            vec![
                policy("listsplus", "twolist"),
                extra("listsplus", "second", "either"),
            ],
            vec![],
        ),
        // In any mode an unconverted alternative only narrows access, after review.
        app(
            "narrow",
            "any",
            vec![
                binding("narrow", "lab", json!({"group":"g-lab"})),
                policy("narrow", "twolist"),
            ],
            vec![],
        ),
    ]);
    // Reviewed settings that already use any_groups leave room for one list only.
    apps.push((
        "reviewedany".to_owned(),
        "any",
        vec![policy("reviewedany", "twolist")],
        vec![],
        json!({"any_groups":["t1"]}),
    ));
    let user = |pk: u64, name: &str, groups: &[&str]| {
        json!({"pk":pk,"uid":format!("uid-{pk}"),"username":name,"name":name,"groups":groups,
            "attributes":{},"type":"internal","is_active":true,"roles":[]})
    };
    let users = json!([
        user(1, "alice", &["g-eng"]),
        user(2, "bob", &["g-staff", "g-ops"]),
        user(3, "carol", &["g-lab"]),
        user(4, "dave", &[]),
        user(5, "erin", &["g-lab", "g-research"])
    ]);
    let bundle = |slugs: &[String], twin: bool, extra: Vec<Value>| {
        let chosen = apps
            .iter()
            .enumerate()
            .filter(|(_, (slug, ..))| slugs.contains(slug))
            .collect::<Vec<_>>();
        let mut groups = vec![
            json!({"pk":"g-staff","name":"staff","parents":[]}),
            json!({"pk":"g-eng","name":"engineering","parents":["g-staff"]}),
            json!({"pk":"g-ops","name":"ops","parents":[]}),
            json!({"pk":"g-lab","name":"lab","parents":[]}),
            json!({"pk":"g-research","name":"research","parents":[]}),
            json!({"pk":"g-archive","name":"archive","parents":[]}),
            json!({"pk":"g-admins","name":"authentik Admins","parents":[]}),
        ];
        groups.extend(
            (1..=13).map(|i| json!({"pk":format!("g-t{i}"),"name":format!("t{i}"),"parents":[]})),
        );
        let mut excluded = vec!["g-admins"];
        // Authentik's check also matches an excluded group of the same name.
        if twin {
            groups.push(json!({"pk":"g-archive-old","name":"archive","parents":[]}));
            excluded.push("g-archive-old");
        }
        let mut bindings = chosen
            .iter()
            .flat_map(|(_, (_, _, bindings, ..))| bindings.clone())
            .collect::<Vec<_>>();
        bindings.extend(extra);
        let passwords = ["alice", "bob", "carol", "dave", "erin"]
            .map(|name| {
                (
                    name.to_owned(),
                    json!({"reference":format!("env:{}", name.to_uppercase()),"version":"v1"}),
                )
            })
            .into_iter()
            .collect::<serde_json::Map<_, _>>();
        json!({"api_version":"riauth.authentik-import/v1","issuer":issuer,
            "users":users,"groups":groups,"excluded_groups":excluded,"expression_policies":policies,
            "providers":chosen.iter().map(|(i, (slug, ..))| json!({"pk":i + 1,"name":slug,"client_id":slug,"client_type":"public",
                "grant_types":["authorization_code"],"redirect_uris":[{"matching_mode":"strict","url":"http://localhost:7777/callback?existing=1"}],
                "property_mappings":[],"sub_mode":"hashed_user_id","issuer_mode":"per_provider","include_claims_in_id_token":true})).collect::<Vec<_>>(),
            "applications":chosen.iter().map(|(i, (slug, mode, ..))| json!({"pk":format!("app-{slug}"),"slug":slug,"provider":i + 1,
                "name":slug,"policy_engine_mode":mode})).collect::<Vec<_>>(),
            "policy_bindings":bindings,"sources":[],"passwords":passwords,
            "clients":chosen.iter().map(|(_, (slug, _, _, acknowledged, access))| (slug.to_string(), json!({"issuer":format!("{issuer}/application/o/{slug}/"),
                "scopes":["openid","profile"],"settings":{"policy":{"access":access}},"translated_mapping_ids":[],"translated_binding_ids":acknowledged,
                "authentication_flow_reviewed":true,"require_mfa":false}))).collect::<serde_json::Map<_, _>>()})
    };
    let convert = |input: Value| riauth::migration::convert(serde_json::from_value(input).unwrap());
    let all = apps
        .iter()
        .map(|(slug, ..)| slug.clone())
        .collect::<Vec<_>>();
    let report = convert(bundle(&all, true, vec![])).unwrap();
    assert_eq!(report["ready_for_plan"], false);
    let policy_id = |slug: &str| format!("{slug}-policy");
    let mut expected = vec![
        ("anymix-lab".to_owned(), (Convertible, false)),
        ("allpair-ops".to_owned(), (Convertible, false)),
        ("twolists-second".to_owned(), (Convertible, false)),
        ("twomixed-second".to_owned(), (Convertible, false)),
        ("narrowacked-lab".to_owned(), (Convertible, false)),
        ("narrowacked-policy".to_owned(), (Manual, false)),
        ("narrow-lab".to_owned(), (Convertible, false)),
        ("narrow-policy".to_owned(), (Manual, true)),
        ("threebind-second".to_owned(), (Unsupported, true)),
        ("threebind-third".to_owned(), (Unsupported, true)),
        ("listsplus-second".to_owned(), (Unsupported, true)),
    ];
    for slug in [
        "both",
        "either",
        "neither",
        "anyof",
        "precedence",
        "opsonly",
        "spaced",
        "anymix",
        "allpair",
        "factored",
        "factoredneg",
        "negfactored",
        "redundant",
        "absorbneg",
        "twelve",
        "mixed",
        "mixed2",
        "paren",
        "twolist",
        "twolistneg",
        "twolistdnf",
        "negtwolist",
        "overlap",
        "parensnot",
        "nested",
        "tight",
        "twolists",
        "twomixed",
    ] {
        expected.push((policy_id(slug), (Convertible, false)));
    }
    for slug in [
        "contradiction",
        "deep",
        "unbalanced",
        "nbsp",
        "formfeed",
        "newline",
        "call",
        "comment",
        "escape",
        "doubleeq",
        "long",
        "ghost",
        "admins",
        "twin",
        "secret",
        "threebind",
        "listsplus",
        "reviewedany",
    ] {
        expected.push((policy_id(slug), (Unsupported, true)));
    }
    // Formulas that do not factor are manual rewrites that no acknowledgement clears.
    for slug in [
        "threelists",
        "nonmono",
        "negor",
        "negand",
        "acked",
        "tautology",
        "toomany",
    ] {
        expected.push((policy_id(slug), (Manual, true)));
    }
    for (id, classification) in expected {
        assert_eq!(
            findings(&report, PolicyBinding, &id),
            [classification],
            "{id}"
        );
    }
    // Expressions are never quoted in the report.
    let rendered = report.to_string();
    for fragment in ["s3cr3t-token", "is_superuser", "st\\u0061ff"] {
        assert!(!rendered.contains(fragment), "{fragment}");
    }
    let access = |report: &Value, cid: &str| {
        let client = report["draft"]["clients"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["client_id"] == cid)
            .unwrap()
            .clone();
        let rule = &client["settings"]["policy"]["access"];
        [
            client["allowed_groups"].clone(),
            rule["any_groups"].clone(),
            rule["all_groups"].clone(),
            rule["denied_groups"].clone(),
        ]
    };
    let twelve = (1..=12)
        .map(|i| format!("t{i}"))
        .collect::<std::collections::BTreeSet<_>>();
    let (ls, or, lo, os) = (
        json!(["lab", "staff"]),
        json!(["ops", "research"]),
        json!(["lab", "ops"]),
        json!(["ops", "staff"]),
    );
    let none = json!([]);
    for (cid, expected) in [
        (
            "both",
            [none.clone(), none.clone(), json!(["staff"]), json!(["ops"])],
        ),
        (
            "either",
            [ls.clone(), none.clone(), none.clone(), none.clone()],
        ),
        (
            "neither",
            [none.clone(), none.clone(), none.clone(), ls.clone()],
        ),
        (
            "anyof",
            [ls.clone(), none.clone(), none.clone(), none.clone()],
        ),
        (
            "precedence",
            [none.clone(), none.clone(), json!(["staff"]), json!(["ops"])],
        ),
        (
            "opsonly",
            [json!(["ops"]), none.clone(), none.clone(), none.clone()],
        ),
        (
            "spaced",
            [json!(["ops"]), none.clone(), none.clone(), none.clone()],
        ),
        (
            "anymix",
            [ls.clone(), none.clone(), none.clone(), none.clone()],
        ),
        (
            "allpair",
            [ls.clone(), none.clone(), none.clone(), json!(["ops"])],
        ),
        (
            "factored",
            [lo.clone(), none.clone(), json!(["staff"]), none.clone()],
        ),
        (
            "factoredneg",
            [ls.clone(), none.clone(), none.clone(), json!(["ops"])],
        ),
        (
            "negfactored",
            [lo.clone(), none.clone(), json!(["staff"]), none.clone()],
        ),
        (
            "redundant",
            [json!(["staff"]), none.clone(), none.clone(), none.clone()],
        ),
        (
            "absorbneg",
            [none.clone(), none.clone(), json!(["staff"]), json!(["ops"])],
        ),
        (
            "twelve",
            [json!(twelve), none.clone(), none.clone(), none.clone()],
        ),
        (
            "mixed",
            [lo.clone(), ls.clone(), none.clone(), none.clone()],
        ),
        (
            "mixed2",
            [ls.clone(), os.clone(), none.clone(), none.clone()],
        ),
        (
            "paren",
            [os.clone(), none.clone(), none.clone(), none.clone()],
        ),
        (
            "twolist",
            [ls.clone(), or.clone(), none.clone(), none.clone()],
        ),
        (
            "twolistneg",
            [ls.clone(), or.clone(), none.clone(), json!(["engineering"])],
        ),
        (
            "twolistdnf",
            [ls.clone(), or.clone(), none.clone(), none.clone()],
        ),
        (
            "negtwolist",
            [ls.clone(), or.clone(), none.clone(), none.clone()],
        ),
        (
            "overlap",
            [ls.clone(), os.clone(), none.clone(), none.clone()],
        ),
        (
            "parensnot",
            [none.clone(), none.clone(), json!(["staff"]), lo.clone()],
        ),
        (
            "nested",
            [ls.clone(), or.clone(), none.clone(), none.clone()],
        ),
        (
            "tight",
            [ls.clone(), or.clone(), none.clone(), none.clone()],
        ),
        (
            "twolists",
            [ls.clone(), lo.clone(), none.clone(), none.clone()],
        ),
        (
            "twomixed",
            [lo.clone(), lo.clone(), json!(["staff"]), none.clone()],
        ),
        (
            "narrowacked",
            [json!(["lab"]), none.clone(), none.clone(), none.clone()],
        ),
    ] {
        assert_eq!(access(&report, cid), expected, "{cid}");
    }
    // A binding flag that is not a boolean fails the conversion instead of flipping its meaning.
    for field in ["negate", "enabled", "expiring"] {
        let malformed = binding(
            "both",
            "malformed",
            json!({"policy":"p-both", field:"true"}),
        );
        let error = convert(bundle(&["both".to_owned()], false, vec![malformed])).unwrap_err();
        assert_eq!(
            error.message,
            format!("Policy binding field {field} must be a boolean")
        );
    }

    // Converted formulas admit exactly whom Authentik admitted, through the group hierarchy.
    let report = convert(bundle(&ready, false, vec![])).unwrap();
    assert_eq!(report["ready_for_plan"], true, "{}", report["blockers"]);
    let plan = f
        .core
        .plan_state(
            &f.admin,
            serde_json::from_value(report["manifest"].clone()).unwrap(),
        )
        .unwrap();
    let secrets = ["ALICE", "BOB", "CAROL", "DAVE", "ERIN"]
        .map(|name| (format!("env:{name}"), PASSWORD.to_owned()));
    f.core
        .apply_state(
            &f.admin,
            riauth::state::ApplyRequest {
                plan,
                secrets: secrets.into(),
                run_id: None,
            },
        )
        .unwrap();
    let sessions = ["alice", "bob", "carol", "dave", "erin"].map(|username| {
        (
            username,
            text(
                &f.core
                    .login(username.into(), PASSWORD.into(), None)
                    .unwrap(),
                "session_token",
            ),
        )
    });
    for (cid, admitted) in [
        ("both", ["alice"].as_slice()),
        ("either", &["alice", "bob", "carol", "erin"]),
        ("neither", &["dave"]),
        ("anyof", &["alice", "bob", "carol", "erin"]),
        ("precedence", &["alice"]),
        ("opsonly", &["bob"]),
        ("spaced", &["bob"]),
        ("anymix", &["alice", "bob", "carol", "erin"]),
        ("allpair", &["alice", "carol", "erin"]),
        ("factored", &["bob"]),
        ("factoredneg", &["alice", "carol", "erin"]),
        ("negfactored", &["bob"]),
        ("redundant", &["alice", "bob"]),
        ("absorbneg", &["alice"]),
        ("twelve", &[]),
        ("mixed", &["bob", "carol", "erin"]),
        ("mixed2", &["alice", "bob"]),
        ("paren", &["alice", "bob"]),
        ("twolist", &["bob", "erin"]),
        ("twolistneg", &["bob", "erin"]),
        ("twolistdnf", &["bob", "erin"]),
        ("negtwolist", &["bob", "erin"]),
        ("overlap", &["alice", "bob"]),
        ("parensnot", &["alice"]),
        ("nested", &["bob", "erin"]),
        ("tight", &["bob", "erin"]),
        ("twolists", &["bob", "carol", "erin"]),
        ("twomixed", &["bob"]),
        // Authentik also admitted Bob through the acknowledged alternative.
        ("narrowacked", &["carol", "erin"]),
    ] {
        for (username, session) in &sessions {
            let mut request = f.request(cid, &crypto::random_token(""));
            request.scope = "openid profile".into();
            assert_eq!(
                f.core.authorize(session, request).is_ok(),
                admitted.contains(username),
                "{cid} {username}"
            );
        }
    }
}

#[test]
fn authentik_scope_mappings_convert_exact_claims_or_block() {
    use riauth::migration::{Classification::*, ItemKind::*};
    let f = Fixture::new();
    let issuer = f.core.config.issuer.clone();
    // Authentik 2025.10's default mappings, verbatim.
    let openid = "# This scope is required by the OpenID-spec, and must as such exist in authentik.\n# The scope by itself does not grant any information\nreturn {}\n";
    let email = "return {\n    \"email\": request.user.email,\n    \"email_verified\": True\n}\n";
    let profile = r#"return {
    # Because authentik only saves the user's full name, and has no concept of first and last names,
    # the full name is used as given name.
    # You can override this behaviour in custom mappings, i.e. `request.user.name.split(" ")`
    "name": request.user.name,
    "given_name": request.user.name,
    "preferred_username": request.user.username,
    "nickname": request.user.username,
    "groups": [group.name for group in request.user.ak_groups.all()],
}
"#;
    let offline = "# This scope grants the application a refresh token that can be used to refresh user data\n# and let the application access authentik without the users interaction\nreturn {}\n";
    let api = "# This scope grants the application the ability to access the authentik API\n# on behalf of the authorizing user\nreturn {}\n";
    let entitlements = "entitlements = [entitlement.name for entitlement in request.user.app_entitlements(provider.application)]\nreturn {\n    \"entitlements\": entitlements,\n    \"roles\": entitlements,\n}\n";
    // Authentik's current profile mapping calls helpers riAuth never evaluates.
    let profile_main = "avatar = request.user.avatar\nreturn delete_none_values({\n    \"name\": request.user.name,\n    \"given_name\": ak_obj_attr(request.user, \"given_name\", \"name\"),\n    \"preferred_username\": request.user.username,\n})\n";
    let mapping = |pk: &str, scope: &str, expression: &str| json!({"pk":pk,"managed":null,"name":format!("mapping {pk}"),"scope_name":scope,"expression":expression});
    let mappings = json!([
        mapping("m-openid", "openid", openid),
        mapping("m-email", "email", email),
        mapping("m-profile", "profile", profile),
        mapping("m-profile2", "profile", profile),
        mapping("m-offline", "offline_access", offline),
        mapping("m-api", "goauthentik.io/api", api),
        mapping("m-entitlements", "entitlements", entitlements),
        mapping("m-profile-main", "profile", profile_main),
        mapping(
            "m-custom",
            "department",
            "return {\"department_admin\": True, \"login\": request.user.username}"
        ),
        mapping(
            "m-contact",
            "contact",
            "return {'contact': request.user.email}"
        ),
        mapping("m-sub", "legacy", "return {\"sub\": request.user.username}"),
        mapping("m-secret", "extra", "return {\"token\": \"s3cr3t-token\"}"),
        mapping(
            "m-reserved",
            "extra",
            "return {\"email\": request.user.email}"
        ),
        mapping(
            "m-name-only",
            "profile",
            "return {\"name\": request.user.name}"
        ),
    ]);
    // (client, mappings, scopes, acknowledged mapping IDs, reviewed claim mappings)
    let clients: Vec<(&str, Vec<&str>, Vec<&str>, Vec<&str>, Value)> = vec![
        (
            "app",
            vec!["m-openid", "m-profile", "m-offline"],
            vec!["openid", "profile", "offline_access"],
            vec![],
            json!([]),
        ),
        (
            "custom",
            vec!["m-openid", "m-custom"],
            vec!["openid", "department"],
            vec![],
            json!([]),
        ),
        (
            "contact",
            vec!["m-contact"],
            vec!["openid", "contact"],
            vec![],
            json!([]),
        ),
        // riAuth reports its own verification state, so acknowledging the email mapping is a choice.
        (
            "mailack",
            vec!["m-openid", "m-email"],
            vec!["openid", "email"],
            vec!["m-email"],
            json!([]),
        ),
        (
            "mail",
            vec!["m-email"],
            vec!["openid", "email"],
            vec![],
            json!([]),
        ),
        (
            "subject",
            vec!["m-sub"],
            vec!["openid", "legacy"],
            vec!["m-sub"],
            json!([]),
        ),
        (
            "authentik",
            vec!["m-api", "m-entitlements", "m-profile-main"],
            vec!["openid", "profile", "entitlements"],
            vec![],
            json!([]),
        ),
        (
            "noscope",
            vec!["m-custom"],
            vec!["openid"],
            vec![],
            json!([]),
        ),
        (
            "dup",
            vec!["m-profile", "m-profile2"],
            vec!["openid", "profile"],
            vec![],
            json!([]),
        ),
        (
            "secret",
            vec!["m-secret"],
            vec!["openid", "extra"],
            vec![],
            json!([]),
        ),
        (
            "reserved",
            vec!["m-reserved"],
            vec!["openid", "extra"],
            vec![],
            json!([]),
        ),
        (
            "nameonly",
            vec!["m-name-only"],
            vec!["openid", "profile"],
            vec![],
            json!([]),
        ),
        (
            "clash",
            vec!["m-custom"],
            vec!["openid", "department"],
            vec![],
            json!([{"scope":"openid","claim":"login","source":{"type":"display_name"}}]),
        ),
        (
            "unknown",
            vec!["m-missing"],
            vec!["openid"],
            vec!["m-missing"],
            json!([]),
        ),
    ];
    let user = |pk: u64, name: &str, display: &str, email: &str, groups: &[&str]| {
        json!({"pk":pk,"uid":format!("uid-{pk}"),"username":name,"name":display,"email":email,"groups":groups,
            "attributes":{},"type":"internal","is_active":true,"roles":[]})
    };
    let bundle = |names: &[&str], users: Value| {
        let chosen = clients
            .iter()
            .enumerate()
            .filter(|(_, (cid, ..))| names.contains(cid))
            .collect::<Vec<_>>();
        let passwords = users
            .as_array()
            .unwrap()
            .iter()
            .map(|u| {
                (
                    u["username"].as_str().unwrap().to_owned(),
                    json!({"reference":"env:PASSWORD","version":"v1"}),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        json!({"api_version":"riauth.authentik-import/v1","issuer":issuer,"users":users,
            "groups":[{"pk":"g-staff","name":"staff","parents":[]},{"pk":"g-ops","name":"ops","parents":[]},
                {"pk":"g-child","name":"child","parents":["g-staff"]}],
            "scope_mappings":mappings,"applications":[],"policy_bindings":[],"sources":[],"passwords":passwords,
            "providers":chosen.iter().map(|(i, (cid, mappings, ..))| json!({"pk":i + 1,"name":cid,"client_id":cid,"client_type":"public",
                "grant_types":["authorization_code"],"redirect_uris":[{"matching_mode":"strict","url":"http://localhost:7777/callback?existing=1"}],
                "property_mappings":mappings,"sub_mode":"hashed_user_id","issuer_mode":"per_provider","include_claims_in_id_token":true})).collect::<Vec<_>>(),
            "clients":chosen.iter().map(|(_, (cid, _, scopes, acknowledged, reviewed))| (cid.to_string(), json!({"issuer":format!("{issuer}/application/o/{cid}/"),
                "scopes":scopes,"settings":{"claim_mappings":reviewed},"translated_mapping_ids":acknowledged,"translated_binding_ids":[],
                "authentication_flow_reviewed":true,"require_mfa":false}))).collect::<serde_json::Map<_, _>>()})
    };
    let convert =
        |input: Value| riauth::migration::convert(serde_json::from_value(input).unwrap()).unwrap();
    // Every account has a name and an email, keeps its username, and has flat groups.
    let exact_users = json!([
        user(1, "alice", "Alice", "alice@example.test", &["g-staff"]),
        user(2, "bob", "Bob", "bob@example.test", &["g-ops"])
    ]);
    let all = clients.iter().map(|(cid, ..)| *cid).collect::<Vec<_>>();
    let report = convert(bundle(&all, exact_users.clone()));
    assert_eq!(report["ready_for_plan"], false);
    for (id, expected) in [
        ("app/m-openid", vec![(Exact, false)]),
        ("app/m-profile", vec![(Convertible, false)]),
        ("app/m-offline", vec![(Exact, false)]),
        ("custom/m-custom", vec![(Convertible, false)]),
        ("contact/m-contact", vec![(Convertible, false)]),
        ("mailack/m-email", vec![(Manual, false)]),
        ("mail/m-email", vec![(Manual, true)]),
        // A mapping that changes subjects blocks even when acknowledged.
        ("subject/m-sub", vec![(Unsupported, true)]),
        ("authentik/m-api", vec![(Manual, true)]),
        ("authentik/m-entitlements", vec![(Manual, true)]),
        ("authentik/m-profile-main", vec![(Unsupported, true)]),
        ("noscope/m-custom", vec![(Manual, true)]),
        ("dup/m-profile", vec![(Manual, true)]),
        ("dup/m-profile2", vec![(Manual, true)]),
        ("secret/m-secret", vec![(Manual, true)]),
        ("reserved/m-reserved", vec![(Manual, true)]),
        ("nameonly/m-name-only", vec![(Manual, true)]),
        ("clash/m-custom", vec![(Manual, true)]),
        ("unknown/m-missing", vec![(Unsupported, true)]),
    ] {
        assert_eq!(findings(&report, PropertyMapping, id), expected, "{id}");
    }
    assert!(report["blockers"].as_array().unwrap().contains(&json!(
        "unknown: property mapping m-missing is missing from scope_mappings; subject continuity cannot be proved"
    )));
    // Mapping sources are never quoted in the report.
    let rendered = report.to_string();
    for fragment in ["s3cr3t-token", "app_entitlements", "delete_none_values"] {
        assert!(!rendered.contains(fragment), "{fragment}");
    }
    let claim_mappings = |report: &Value, cid: &str| {
        report["draft"]["clients"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["client_id"] == cid)
            .unwrap()["settings"]["claim_mappings"]
            .clone()
    };
    assert_eq!(
        claim_mappings(&report, "app"),
        json!([
        {"scope":"profile","claim":"given_name","source":{"type":"display_name"}},
        {"scope":"profile","claim":"nickname","source":{"type":"username"}},
        {"scope":"profile","claim":"groups","source":{"type":"groups"}}])
    );
    assert_eq!(
        claim_mappings(&report, "custom"),
        json!([
        {"scope":"department","claim":"department_admin","source":{"type":"literal","value":true}},
        {"scope":"department","claim":"login","source":{"type":"username"}}])
    );

    // An empty name, a missing email or an ancestor group changes the value riAuth would return.
    let inexact = json!([
        user(1, "alice", "Alice", "alice@example.test", &["g-staff"]),
        user(3, "carol", "", "", &["g-child"])
    ]);
    let report = convert(bundle(&["app", "contact"], inexact));
    for id in ["app/m-profile", "contact/m-contact"] {
        assert_eq!(
            findings(&report, PropertyMapping, id),
            [(Manual, true)],
            "{id}"
        );
    }

    // Converted claims are exactly what Authentik's mappings returned.
    let report = convert(bundle(&["app", "custom", "mailack"], exact_users));
    assert_eq!(report["ready_for_plan"], true, "{}", report["blockers"]);
    let plan = f
        .core
        .plan_state(
            &f.admin,
            serde_json::from_value(report["manifest"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .apply_state(
            &f.admin,
            riauth::state::ApplyRequest {
                plan,
                secrets: [("env:PASSWORD".into(), PASSWORD.into())].into(),
                run_id: None,
            },
        )
        .unwrap();
    let userinfo = |cid: &str, username: &str, scopes: &[&str]| {
        let mut claims = f
            .core
            .explain(
                &f.admin,
                riauth::claims::Explain {
                    client_id: cid.into(),
                    username: username.into(),
                    scope: strings(scopes),
                    mfa: false,
                },
            )
            .unwrap()["userinfo"]
            .clone();
        claims.as_object_mut().unwrap().remove("sub");
        claims
    };
    assert_eq!(
        userinfo("app", "alice", &["openid", "profile", "offline_access"]),
        json!({"name":"Alice","given_name":"Alice","preferred_username":"alice","nickname":"alice","groups":["staff"]})
    );
    assert_eq!(
        userinfo("custom", "bob", &["openid", "department"]),
        json!({"department_admin":true,"login":"bob"})
    );
}

#[test]
fn authentik_scope_mapping_exactness_rejects_escaped_keys_extra_groups_and_absent_scopes() {
    use riauth::migration::{Classification::*, ItemKind::*};
    let issuer = "https://identity.example.test";
    let profile =
        "return {\"name\": request.user.name, \"preferred_username\": request.user.username}";
    let profile_with_groups = "return {\"name\": request.user.name, \"preferred_username\": request.user.username, \"groups\": [group.name for group in request.user.ak_groups.all()]}";
    // (client, scope, Python expression, reviewed scopes, groups_in_profile, expected finding)
    let cases = [
        (
            "escaped-sub",
            "legacy",
            r#"return {"\u0073ub": True}"#,
            &["openid", "legacy"][..],
            false,
            Unsupported,
            true,
        ),
        (
            "opaque-sub",
            "legacy",
            "return {\"sub\": request.user.username.upper()}",
            &["openid", "legacy"],
            false,
            Unsupported,
            true,
        ),
        (
            "parenthesized-sub",
            "legacy",
            r#"return {("sub"): True}"#,
            &["openid", "legacy"],
            false,
            Unsupported,
            true,
        ),
        (
            "triple-sub",
            "legacy",
            r#"return {"""\u0073ub""": True}"#,
            &["openid", "legacy"],
            false,
            Unsupported,
            true,
        ),
        (
            "joined-sub",
            "legacy",
            r#"return {"s" "ub": True}"#,
            &["openid", "legacy"],
            false,
            Unsupported,
            true,
        ),
        (
            "formatted-sub",
            "legacy",
            r#"return {f"{'s'}ub": True}"#,
            &["openid", "legacy"],
            false,
            Unsupported,
            true,
        ),
        (
            "keyword-sub",
            "legacy",
            "return dict(sub=True)",
            &["openid", "legacy"],
            false,
            Unsupported,
            true,
        ),
        (
            "assigned-sub",
            "legacy",
            "claims = {}; claims[\"sub\"] = True; return claims",
            &["openid", "legacy"],
            false,
            Unsupported,
            true,
        ),
        (
            "ordinary-custom",
            "legacy",
            "return {\"region\": request.user.attributes.get(\"region\")}",
            &["openid", "legacy"],
            false,
            Manual,
            false,
        ),
        (
            "extra-groups",
            "profile",
            profile,
            &["openid", "profile"],
            true,
            Manual,
            true,
        ),
        (
            "matched-groups",
            "profile",
            profile_with_groups,
            &["openid", "profile"],
            true,
            Exact,
            false,
        ),
        (
            "absent-empty",
            "offline_access",
            "return {}",
            &["openid"],
            false,
            Manual,
            true,
        ),
        (
            "absent-builtin",
            "profile",
            profile,
            &["openid"],
            false,
            Manual,
            true,
        ),
        (
            "present-empty",
            "openid",
            "return {}",
            &["openid"],
            false,
            Exact,
            false,
        ),
        (
            "unmatched-scopes",
            "openid",
            "return {}",
            &["openid", "profile", "groups", "email"],
            true,
            Exact,
            false,
        ),
    ];
    let mut input = json!({
        "api_version":"riauth.authentik-import/v1",
        "issuer":issuer,
        "users":[{"pk":1,"uid":"uid-alice","username":"alice","name":"Alice",
            "email":"alice@example.test","groups":[],"attributes":{},"type":"internal",
            "is_active":true,"roles":[]}],
        "groups":[],
        "passwords":{"alice":{"reference":"env:PASSWORD","version":"v1"}},
        "scope_mappings":cases.iter().map(|(cid, scope, expression, ..)| json!({
            "pk":format!("m-{cid}"),"managed":null,"name":format!("mapping {cid}"),
            "scope_name":scope,"expression":expression
        })).collect::<Vec<_>>(),
        "applications":[],"policy_bindings":[],"sources":[],
        "providers":cases.iter().enumerate().map(|(i, (cid, ..))| json!({
            "pk":i + 1,"name":cid,"client_id":cid,"client_type":"public",
            "grant_types":["authorization_code"],
            "redirect_uris":[{"matching_mode":"strict","url":"http://localhost:7777/callback"}],
            "property_mappings":[format!("m-{cid}")],"sub_mode":"hashed_user_id",
            "issuer_mode":"per_provider","include_claims_in_id_token":true
        })).collect::<Vec<_>>(),
        "clients":cases.iter().map(|(cid, _, _, scopes, groups_in_profile, ..)| (
            (*cid).to_owned(), json!({
                "issuer":format!("{issuer}/application/o/{cid}/"),"scopes":scopes,
                "settings":{"groups_in_profile":groups_in_profile},
                "translated_mapping_ids":[],"translated_binding_ids":[],
                "authentication_flow_reviewed":true,"require_mfa":false
            })
        )).collect::<serde_json::Map<_, _>>()
    });
    for cid in [
        "escaped-sub",
        "opaque-sub",
        "parenthesized-sub",
        "triple-sub",
        "joined-sub",
        "formatted-sub",
        "keyword-sub",
        "assigned-sub",
        "ordinary-custom",
    ] {
        input["clients"][cid]["translated_mapping_ids"] = json!([format!("m-{cid}")]);
    }
    input["scope_mappings"].as_array_mut().unwrap().extend([
        json!({"pk":"m-grouped-sub","managed":null,"name":"grouped sub","scope_name":"legacy",
            "expression":r#"return {"sub": True}"#}),
        json!({"pk":"m-grouped-safe","managed":null,"name":"grouped safe","scope_name":"legacy",
            "expression":"return {\"region\": request.user.attributes.get(\"region\")}"}),
    ]);
    input["providers"].as_array_mut().unwrap().push(json!({
        "pk":cases.len() + 1,"name":"grouped","client_id":"grouped","client_type":"public",
        "grant_types":["authorization_code"],
        "redirect_uris":[{"matching_mode":"strict","url":"http://localhost:7777/callback"}],
        "property_mappings":["m-grouped-sub","m-grouped-safe"],"sub_mode":"hashed_user_id",
        "issuer_mode":"per_provider","include_claims_in_id_token":true
    }));
    input["clients"]["grouped"] = json!({
        "issuer":format!("{issuer}/application/o/grouped/"),"scopes":["openid","legacy"],
        "settings":{},"translated_mapping_ids":["m-grouped-sub","m-grouped-safe"],
        "translated_binding_ids":[],"authentication_flow_reviewed":true,"require_mfa":false
    });
    let report = riauth::migration::convert(serde_json::from_value(input).unwrap()).unwrap();
    for (cid, _, _, _, _, classification, blocking) in cases {
        let id = format!("{cid}/m-{cid}");
        assert_eq!(
            findings(&report, PropertyMapping, &id),
            [(classification, blocking)],
            "{id}"
        );
    }
    assert_eq!(
        findings(&report, PropertyMapping, "grouped/m-grouped-sub"),
        [(Unsupported, true)]
    );
    assert_eq!(
        findings(&report, PropertyMapping, "grouped/m-grouped-safe"),
        [(Manual, false)]
    );
    for scope in ["profile", "groups", "email"] {
        let id = format!("unmatched-scopes/scope/{scope}");
        assert_eq!(findings(&report, PropertyMapping, &id), [(Manual, false)]);
    }
    let profile_expansion = report["items"].as_array().unwrap().iter().find(|item| {
        item["kind"] == "property_mapping" && item["id"] == "unmatched-scopes/scope/profile"
    }).unwrap();
    assert!(profile_expansion["reason"].as_str().unwrap().contains(
        "name, preferred_username and groups"
    ));
    assert!(report["blockers"].as_array().unwrap().iter().any(|blocker| {
        blocker == "escaped-sub: property mapping m-escaped-sub may change subjects"
    }));
    assert!(report["blockers"].as_array().unwrap().iter().any(|blocker| {
        blocker == "parenthesized-sub: property mapping m-parenthesized-sub may change subjects"
    }));
    assert!(report["blockers"].as_array().unwrap().iter().any(|blocker| {
        blocker == "grouped: property mapping m-grouped-sub may change subjects"
    }));
}

#[test]
fn authentik_reimport_keeps_verified_accounts_and_never_moves_identities() {
    use riauth::migration::{Classification::*, ItemKind::*};
    let f = Fixture::new();
    let issuer = f.core.config.issuer.clone();
    let user = |pk: u64, username: &str, email: &str, extra: Value| {
        let mut user = json!({"pk":pk,"uid":format!("uid-{pk}"),"uuid":format!("uuid-{pk}"),"username":username,
            "name":username,"email":email,"groups":["g-staff"],"attributes":{},"type":"internal","is_active":true,"roles":[]});
        user.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        user
    };
    let provider = |pk: u64, cid: &str, mode: &str| {
        json!({"pk":pk,"name":cid,"client_id":cid,"client_type":"public","grant_types":["authorization_code"],
            "redirect_uris":[{"matching_mode":"strict","url":"http://localhost:7777/callback?existing=1"}],
            "property_mappings":[],"sub_mode":mode,"issuer_mode":"per_provider","include_claims_in_id_token":true})
    };
    let client = |cid: &str| {
        json!({"issuer":format!("{issuer}/application/o/{cid}/"),"scopes":["openid","profile"],"settings":{},
            "translated_mapping_ids":[],"translated_binding_ids":[],"authentication_flow_reviewed":true,"require_mfa":false})
    };
    let bundle = |users: Value, state: Option<&Value>| {
        // Alice's password follows her Authentik account across the rename.
        let passwords = users
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|u| {
                let reference = match (u["pk"].as_u64(), u["username"].as_str()) {
                    (Some(42), Some(name)) => (name, "env:ALICE"),
                    (Some(43), Some(name)) => (name, "env:BOB"),
                    _ => return None,
                };
                Some((
                    reference.0.to_owned(),
                    json!({"reference":reference.1,"version":"v1"}),
                ))
            })
            .collect::<serde_json::Map<_, _>>();
        let mut bundle = json!({"api_version":"riauth.authentik-import/v1","issuer":issuer,"users":users,
            "groups":[{"pk":"g-staff","name":"staff","parents":[]}],
            "providers":[provider(1, "wiki", "hashed_user_id"), provider(2, "mail", "user_email")],
            "applications":[{"pk":"app-wiki","slug":"wiki","provider":1,"name":"wiki"},
                {"pk":"app-mail","slug":"mail","provider":2,"name":"mail"}],
            "policy_bindings":[],"sources":[],"passwords":passwords,
            "clients":{"wiki":client("wiki"),"mail":client("mail")}});
        if let Some(state) = state {
            bundle["target_state"] = state.clone();
        }
        bundle
    };
    let convert =
        |input: Value| riauth::migration::convert(serde_json::from_value(input).unwrap()).unwrap();
    let apply = |report: &Value| {
        assert_eq!(report["ready_for_plan"], true, "{}", report["blockers"]);
        let plan = f
            .core
            .plan_state(
                &f.admin,
                serde_json::from_value(report["manifest"].clone()).unwrap(),
            )
            .unwrap();
        let secrets = ["ALICE", "BOB"].map(|name| (format!("env:{name}"), PASSWORD.to_owned()));
        f.core
            .apply_state(
                &f.admin,
                riauth::state::ApplyRequest {
                    plan,
                    secrets: secrets.into(),
                    run_id: None,
                },
            )
            .unwrap();
    };
    let blocks = |report: &Value, blocker: &str| {
        report["blockers"]
            .as_array()
            .unwrap()
            .contains(&json!(blocker))
    };
    let draft_user = |report: &Value, id: &str| {
        report["draft"]["users"]
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["id"] == id)
            .cloned()
    };
    let bob = user(43, "bob", "bob@example.test", json!({}));
    // An inactive account keeps its identity without a credential; Authentik's temporary
    // accounts are never converted.
    let dave = user(45, "dave", "dave@example.test", json!({"is_active":false}));
    let temporary = user(
        46,
        "ak-temp",
        "temp@example.test",
        json!({"attributes":{"goauthentik.io/user/generated":true,"goauthentik.io/user/expires":1893456000}}),
    );
    let first = convert(bundle(
        json!([
            user(42, "alice", "alice@example.test", json!({})),
            bob,
            dave,
            temporary
        ]),
        None,
    ));
    assert_eq!(findings(&first, User, "ak-temp"), [(Unsupported, false)]);
    assert_eq!(findings(&first, Password, "dave"), [(Manual, false)]);
    let draft = draft_user(&first, "authentik-45").unwrap();
    assert_eq!(
        (&draft["enabled"], &draft["password_disabled"]),
        (&json!(false), &json!(true))
    );
    assert!(draft_user(&first, "authentik-46").is_none());
    assert_eq!(
        draft_user(&first, "authentik-42").unwrap()["attributes"]["riauth.migration.authentik"],
        json!({"pk":"42","uuid":"uuid-42"})
    );
    apply(&first);
    let state = f.core.export_state(&f.admin).unwrap()["manifest"].clone();

    // Authentik renamed Alice and gave her old name to a new account. The verified account keeps
    // its immutable riAuth username; the new account may not take it.
    let renamed = || user(42, "alice.smith", "alice@example.test", json!({}));
    let report = convert(bundle(
        json!([
            renamed(),
            user(44, "alice", "newcomer@example.test", json!({})),
            bob,
            dave
        ]),
        Some(&state),
    ));
    assert!(findings(&report, User, "alice.smith").contains(&(Convertible, false)));
    assert_eq!(findings(&report, User, "alice"), [(Unsupported, true)]);
    assert!(blocks(
        &report,
        "alice: username already belongs to another riAuth account"
    ));
    assert!(draft_user(&report, "authentik-44").is_none());
    assert_eq!(
        draft_user(&report, "authentik-42").unwrap()["username"],
        "alice"
    );
    let report = convert(bundle(json!([renamed(), bob, dave]), Some(&state)));
    let staff = report["draft"]["groups"][0]["members"].clone();
    assert_eq!(staff, json!(["alice", "bob", "dave"]));
    apply(&report);
    let exported = f.core.export_state(&f.admin).unwrap()["manifest"].clone();
    let alice = exported["users"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["id"] == "authentik-42")
        .unwrap()
        .clone();
    assert_eq!(
        (&alice["username"], &alice["display_name"]),
        (&json!("alice"), &json!("alice.smith"))
    );
    assert_eq!(
        alice["subjects"],
        json!({"mail":"alice@example.test","wiki":"uid-42"})
    );
    assert!(f.core.login("alice".into(), PASSWORD.into(), None).is_ok());

    // Without recorded evidence, or with evidence of another account, nothing is adopted.
    let mut unproven = exported.clone();
    let mut other = exported.clone();
    for (state, evidence) in [
        (&mut unproven, None),
        (&mut other, Some(json!({"pk":"42","uuid":"uuid-elsewhere"}))),
    ] {
        let account = state["users"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|u| u["id"] == "authentik-42")
            .unwrap();
        let attributes = account["attributes"].as_object_mut().unwrap();
        match evidence {
            Some(evidence) => attributes.insert("riauth.migration.authentik".into(), evidence),
            None => attributes.remove("riauth.migration.authentik"),
        };
    }
    let report = convert(bundle(json!([renamed(), bob]), Some(&unproven)));
    assert_eq!(
        findings(&report, User, "alice.smith"),
        [(Unsupported, true)]
    );
    assert!(blocks(
        &report,
        "alice.smith: rename of riAuth account authentik-42 is not backed by a recorded Authentik UUID"
    ));
    assert!(draft_user(&report, "authentik-42").is_none());
    let report = convert(bundle(json!([renamed(), bob]), Some(&other)));
    assert!(blocks(
        &report,
        "alice.smith: riAuth account authentik-42 records a different Authentik account"
    ));

    // A subject the target issued never changes or moves to another account.
    let report = convert(bundle(
        json!([
            user(42, "alice.smith", "alice.new@example.test", json!({})),
            user(43, "bob", "alice@example.test", json!({})),
            dave
        ]),
        Some(&exported),
    ));
    assert!(blocks(
        &report,
        "alice.smith/mail: subject of an existing riAuth account would change"
    ));
    assert!(blocks(
        &report,
        "bob/mail: subject belongs to another riAuth account"
    ));

    // An upstream identity linked to one account never moves to another.
    let mut linked = exported.clone();
    linked["source_links"] = json!([{"source":"corp","username":"alice","subject":"sub-1"}]);
    let mut moved = bundle(json!([renamed(), bob, dave]), Some(&linked));
    moved["sources"] = json!([{"pk":"oauth-uuid","meta_model_name":"authentik_sources_oauth.oauthsource","user_matching_mode":"identifier"}]);
    moved["source_resolutions"] = json!({"oauth-uuid":{"source":{"id":"corp","name":"Corp","issuer":"https://idp.example.test",
        "authorization_endpoint":"https://idp.example.test/authorize","token_endpoint":"https://idp.example.test/token",
        "client_id":"riauth","token_endpoint_auth_method":"client_secret_post","scopes":["read:user"],
        "oauth_profile":{"userinfo_endpoint":"https://idp.example.test/me","subject_pointer":"/id"}},
        "secret_ref":"env:SOURCE_SECRET","secret_version":"v1"}});
    moved["user_source_connections"] =
        json!([{"pk":1,"user":43,"source":"oauth-uuid","identifier":"sub-1"}]);
    moved["source_links"] = json!([{"source":"corp","username":"bob","subject":"sub-1"}]);
    let report = convert(moved);
    assert!(findings(&report, SourceLink, "bob/corp").contains(&(Unsupported, true)));
    assert!(blocks(
        &report,
        "bob/corp: source link belongs to another riAuth account"
    ));
    assert_eq!(report["draft"]["source_links"], json!([]));
}

#[test]
fn authentik_manifest_plans_only_on_its_exact_target_issuer() {
    use riauth::migration::{Classification::*, ItemKind::*};
    let f = Fixture::new();
    let convert = |issuer: &str| {
        riauth::migration::convert(serde_json::from_value(json!({"api_version":"riauth.authentik-import/v1","issuer":issuer,
            "users":[{"pk":5,"uid":"uid-5","username":"dana","name":"Dana","groups":[],"attributes":{},"type":"internal","is_active":true,"roles":[]}],
            "groups":[],"providers":[],"applications":[],"policy_bindings":[],"sources":[],
            "passwords":{"dana":{"reference":"env:DANA_PASSWORD","version":"v1"}},"clients":{}})).unwrap())
        .unwrap()
    };
    let bound = |issuer: &str| -> riauth::state::Manifest {
        serde_json::from_value(convert(issuer)["manifest"].clone()).unwrap()
    };
    // Offline the instance's issuer is unknown, so the binding is a manual, non-blocking
    // finding that planning enforces.
    let report = convert(&f.core.config.issuer);
    assert_eq!(report["ready_for_plan"], true, "{}", report["blockers"]);
    assert_eq!(findings(&report, Issuer, "*"), [(Manual, false)]);
    assert_eq!(report["manifest"]["issuer"], f.core.config.issuer);
    // Another host, or the same issuer with a trailing slash, never reaches a plan, and a
    // non-canonical form is rejected before it is compared.
    for (issuer, error) in [
        (format!("{}/", f.core.config.issuer), "bound to issuer"),
        ("https://id.example.test".to_owned(), "bound to issuer"),
        (f.core.config.issuer.to_uppercase(), "canonical HTTPS URL"),
    ] {
        let mut manifest = bound(&f.core.config.issuer);
        manifest.issuer = Some(issuer.clone());
        let rejected = f.core.plan_state(&f.admin, manifest).err().unwrap();
        assert!(
            rejected.message.contains(error),
            "{issuer}: {}",
            rejected.message
        );
    }
    assert!(
        f.core
            .plan_state(&f.admin, bound("https://id.example.test"))
            .err()
            .unwrap()
            .message
            .contains("bound to issuer")
    );
    // Unbound manifests, such as exported state, plan as before.
    let mut unbound = bound(&f.core.config.issuer);
    unbound.issuer = None;
    f.core.plan_state(&f.admin, unbound).unwrap();
    let plan = f
        .core
        .plan_state(&f.admin, bound(&f.core.config.issuer))
        .unwrap();
    f.core
        .apply_state(
            &f.admin,
            riauth::state::ApplyRequest {
                plan,
                secrets: [("env:DANA_PASSWORD".into(), PASSWORD.into())].into(),
                run_id: None,
            },
        )
        .unwrap();
    assert!(f.core.login("dana".into(), PASSWORD.into(), None).is_ok());
    assert!(
        f.core.export_state(&f.admin).unwrap()["manifest"]
            .get("issuer")
            .is_none()
    );
}

#[tokio::test]
async fn authentik_import_links_only_exported_source_connections() {
    use riauth::migration::{Classification::*, ItemKind::*};
    let f = Fixture::new();
    let upstream = Upstream::new(&f).await;
    assert!(upstream.source.auto_provision);
    let link = |username: &str, subject: &str| json!({"source":"upstream","username":username,"subject":subject});
    let connection = |user: u64, identifier: &str| json!({"pk":user + 100,"user":user,"source":"oauth-uuid","identifier":identifier});
    let user = |pk: u64, username: &str, kind: &str| {
        json!({"pk":pk,"uid":format!("uid-{pk}"),"username":username,"name":username,"email":"shared@example.test",
            "groups":[],"attributes":{},"type":kind,"is_active":true,"roles":[]})
    };
    let input = json!({"api_version":"riauth.authentik-import/v1","issuer":f.core.config.issuer,
        "users":[user(7, "alice", "external"), user(8, "bob", "internal"), user(9, "carol", "internal")],
        "groups":[],"providers":[],"applications":[],"policy_bindings":[],
        "sources":[{"pk":"oauth-uuid","slug":"corp","meta_model_name":"authentik_sources_oauth.oauthsource","user_matching_mode":"identifier"}],
        "source_resolutions":{"oauth-uuid":{"source":upstream.source,"secret_ref":"env:UPSTREAM_SECRET","secret_version":"v1"}},
        "user_source_connections":[connection(7, "subject-1"), connection(8, "subject-2")],
        "source_links":[link("alice", "subject-1")],
        "passwords":{"bob":{"reference":"env:BOB_PASSWORD","version":"v1"},"carol":{"reference":"env:CAROL_PASSWORD","version":"v1"}},
        "clients":{}});
    let convert = |input: &Value| {
        riauth::migration::convert(serde_json::from_value(input.clone()).unwrap()).unwrap()
    };
    // Bob's connection is not carried over, and the auto-provisioning source would give him a
    // second account on his next upstream sign-in.
    let report = convert(&input);
    assert_eq!(
        report["blockers"],
        json!([
            "bob/upstream: exported connection is not linked and the source provisions accounts"
        ])
    );
    assert_eq!(
        findings(&report, SourceLink, "alice/upstream"),
        [(Exact, false)]
    );
    assert_eq!(findings(&report, Password, "alice"), [(Manual, false)]);
    assert_eq!(
        report["draft"]["source_links"],
        json!([link("alice", "subject-1")])
    );

    // Links the export does not establish are never applied, and never stand in for a password.
    let mut forged = input.clone();
    forged["source_links"] = json!([link("alice", "subject-2"), link("bob", "subject-2"),
        link("carol", "subject-3"), link("mallory", "subject-4"),
        {"source":"elsewhere","username":"bob","subject":"subject-2"}]);
    let report = convert(&forged);
    assert_eq!(
        report["draft"]["source_links"],
        json!([link("bob", "subject-2")])
    );
    for (id, blocker) in [
        (
            "alice/upstream",
            "alice/upstream: source link subject differs from the exported connection",
        ),
        (
            "carol/upstream",
            "carol/upstream: source link is not backed by an exported connection",
        ),
        (
            "mallory/upstream",
            "mallory/upstream: source link names no converted account",
        ),
        (
            "bob/elsewhere",
            "bob/elsewhere: source link names no applied source resolution",
        ),
    ] {
        let found = findings(&report, SourceLink, id);
        assert!(
            !found.is_empty() && found.iter().all(|f| *f == (Manual, true)),
            "{id}"
        );
        assert!(
            report["blockers"]
                .as_array()
                .unwrap()
                .contains(&json!(blocker)),
            "{blocker}"
        );
    }
    assert_eq!(findings(&report, Password, "alice"), [(Manual, true)]);
    assert!(report["blockers"].as_array().unwrap().contains(&json!(
        "alice: external/service identity requires explicit source or service-account migration"
    )));
    let mut unverified = input.clone();
    unverified
        .as_object_mut()
        .unwrap()
        .remove("user_source_connections");
    let report = convert(&unverified);
    assert_eq!(report["draft"]["source_links"], json!([]));
    assert_eq!(findings(&report, SourceLink, "*"), [(Manual, true)]);
    assert_eq!(
        findings(&report, SourceLink, "alice/upstream"),
        [(Manual, true)]
    );
    let mut merged = input.clone();
    merged["user_source_connections"] =
        json!([connection(7, "subject-1"), connection(8, "subject-1")]);
    merged["source_links"] = json!([link("alice", "subject-1"), link("bob", "subject-1")]);
    let report = convert(&merged);
    assert_eq!(report["draft"]["source_links"], json!([]));
    for id in ["alice/upstream", "bob/upstream"] {
        assert!(
            findings(&report, SourceLink, id).contains(&(Unsupported, true)),
            "{id}"
        );
    }

    // With every exported connection carried over, each upstream identity signs in to the
    // migrated account it had in Authentik instead of provisioning a new one.
    let mut ready = input;
    ready["source_links"] = json!([link("alice", "subject-1"), link("bob", "subject-2")]);
    let report = convert(&ready);
    assert_eq!(report["ready_for_plan"], true, "{}", report["blockers"]);
    let manifest = serde_json::from_value(report["manifest"].clone()).unwrap();
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    f.core
        .apply_state(
            &f.admin,
            riauth::state::ApplyRequest {
                plan,
                secrets: [
                    ("env:BOB_PASSWORD".into(), PASSWORD.into()),
                    ("env:CAROL_PASSWORD".into(), PASSWORD.into()),
                    ("env:UPSTREAM_SECRET".into(), "source-client-secret".into()),
                ]
                .into(),
                run_id: Some("migration-links".into()),
            },
        )
        .unwrap();
    assert!(f.core.login("alice".into(), PASSWORD.into(), None).is_err());
    for (subject, id) in [("subject-1", "authentik-7"), ("subject-2", "authentik-8")] {
        let start = upstream.start(&f, None);
        upstream.callback(&f, &start, subject, json!({})).await;
        assert_eq!(
            upstream.finish(&f, &start, true).unwrap()["user"]["id"],
            id,
            "{subject}"
        );
    }
}

#[test]
fn authentik_preflight_never_renames_merges_or_invents_identities() {
    use riauth::migration::{Classification::*, ItemKind::*};
    let user = |pk: u64, username: &str, kind: &str, groups: &[&str]| {
        json!({"pk":pk,"uid":format!("uid-{pk}"),"username":username,"name":username,"groups":groups,"attributes":{},
            "type":kind,"is_active":true,"roles":[]})
    };
    let input = json!({"api_version":"riauth.authentik-import/v1","issuer":"https://id.example.test",
        "users":[user(1, "alice", "internal", &["eng-a", "staff"]), user(2, "bob", "internal", &["eng-b"]),
            user(3, "ak-outpost-1", "internal_service_account", &[]), user(4, "Jane Doe", "internal", &["staff"])],
        "groups":[{"pk":"admins","name":"authentik Admins","parents":[],"is_superuser":true},
            {"pk":"staff","name":"staff","parents":["admins"]},
            {"pk":"eng-a","name":"engineering","parents":[]},{"pk":"eng-b","name":"engineering","parents":["staff"]}],
        "providers":[],"applications":[],"policy_bindings":[],"sources":[],
        "excluded_groups":["admins","gone"],
        "passwords":{"alice":{"reference":"env:ALICE","version":"v1"},"bob":{"reference":"env:BOB","version":"v1"},
            "ghost":{"reference":"env:GHOST","version":"v1"}},
        "totp":{"Jane Doe":{"reference":"file:jane-totp","version":"v1"}},
        "clients":{}});
    let report =
        riauth::migration::convert(serde_json::from_value(input.clone()).unwrap()).unwrap();
    for (kind, id, expected) in [
        // Deliberately excluded, and a stale exclusion that is not applied.
        (Group, "authentik Admins", vec![(Manual, false)]),
        (Group, "gone", vec![(Manual, true)]),
        (Group, "staff", vec![(Convertible, false)]),
        // Two exported groups share a name; neither absorbs the other's members.
        (Group, "engineering", vec![(Unsupported, true)]),
        // Authentik's outpost account is reported but never converted.
        (User, "ak-outpost-1", vec![(Unsupported, false)]),
        (User, "Jane Doe", vec![(Unsupported, true)]),
        // Credentials for accounts that are not converted are never applied elsewhere.
        (Password, "ghost", vec![(Manual, true)]),
        (Totp, "Jane Doe", vec![(Manual, true)]),
        (Credential, "static_tokens", vec![(Unsupported, false)]),
        (Credential, "authenticators", vec![(Unsupported, false)]),
        (Credential, "tokens", vec![(Unsupported, false)]),
        // Every problem is named by its own finding, not by a whole-manifest failure.
        (Manifest, "manifest", vec![]),
    ] {
        assert_eq!(findings(&report, kind, id), expected, "{kind:?} {id}");
    }
    assert!(findings(&report, Password, "ak-outpost-1").is_empty());
    let draft = &report["draft"];
    assert_eq!(
        draft["users"]
            .as_array()
            .unwrap()
            .iter()
            .map(|u| u["username"].clone())
            .collect::<Vec<_>>(),
        [json!("alice"), json!("bob")]
    );
    // Only staff survives: Bob reaches it through his unconverted child group, and the excluded
    // ancestor is not flattened into anyone.
    assert_eq!(
        draft["groups"],
        json!([{"name":"staff","members":["alice","bob"]}])
    );
    let mut unexcluded = input.clone();
    unexcluded["excluded_groups"] = json!([]);
    let report = riauth::migration::convert(serde_json::from_value(unexcluded).unwrap()).unwrap();
    assert_eq!(
        findings(&report, Group, "authentik Admins"),
        [(Unsupported, true)]
    );
    assert!(report["blockers"].as_array().unwrap().contains(&json!(
        "Group authentik Admins: name cannot be represented unchanged"
    )));
    let mut duplicate = input;
    duplicate["users"][1]["pk"] = json!(1);
    assert_eq!(
        riauth::migration::convert(serde_json::from_value(duplicate).unwrap())
            .unwrap_err()
            .message,
        "Duplicate exported user"
    );
}

#[test]
fn migration_inventory_classifies_every_declared_element_and_never_yields_a_manifest() {
    use riauth::migration::{Classification::*, Finding, ItemKind, ItemKind::*};
    let kinds = [
        User,
        Password,
        Totp,
        Passkey,
        Session,
        Subject,
        Group,
        Source,
        Provider,
        AuthenticationFlow,
        PropertyMapping,
        PolicyBinding,
        Federation,
        SigningKey,
        EncryptionKey,
        ClientSecret,
        Grant,
        TokenLifetime,
        RedirectUri,
        Logout,
        Application,
    ];
    let inventory = |system: &str| {
        let elements = kinds
            .iter()
            .flat_map(|kind| {
                [
                    json!({"kind": kind, "id": "*"}),
                    json!({"kind": kind, "id": "grafana"}),
                ]
            })
            .collect::<Vec<_>>();
        json!({"api_version": "riauth.migration-inventory/v1", "system": system, "elements": elements})
    };
    // Directory systems keep users, groups and (for LDAP) password checks on riAuth's live
    // directory adapters; nothing else, and nothing from other systems, has a route.
    let routed = |system: &str, kind: ItemKind| match (system, kind) {
        ("authentik", _) => Some(Manual),
        ("active-directory", User | Group | Password) => Some(Manual),
        ("entra-id", User | Group) => Some(Manual),
        _ => None,
    };
    for system in [
        "keycloak",
        "okta",
        "active-directory",
        "entra-id",
        "authentik",
    ] {
        let input = inventory(system);
        let report =
            riauth::migration::inventory(serde_json::from_value(input.clone()).unwrap()).unwrap();
        assert_eq!(report["ready_for_plan"], false, "{system}");
        assert!(report["manifest"].is_null(), "{system}");
        assert!(report.get("draft").is_none(), "{system}");
        assert_eq!(report["source"]["system"], system);
        assert!(report["source"]["converter"].is_null());
        let items: Vec<Finding> = serde_json::from_value(report["items"].clone()).unwrap();
        // One finding per declared element, plus the manifest finding; every one blocks.
        assert_eq!(items.len(), kinds.len() * 2 + 1, "{system}");
        let mut blockers = items
            .iter()
            .map(|i| {
                assert!(
                    i.blocking && !i.reason.is_empty() && !i.action.is_empty(),
                    "{i:?}"
                );
                i.blocker.clone().unwrap()
            })
            .collect::<Vec<_>>();
        blockers.sort();
        blockers.dedup();
        assert_eq!(json!(blockers), report["blockers"], "{system}");
        assert_eq!(report["summary"]["blocking"], items.len());
        for (class, name) in [
            (Exact, "exact"),
            (Convertible, "convertible"),
            (Manual, "manual"),
            (Unsupported, "unsupported"),
        ] {
            assert_eq!(
                report["summary"][name],
                items.iter().filter(|i| i.classification == class).count(),
                "{system} {name}"
            );
        }
        for element in input["elements"].as_array().unwrap() {
            let kind: ItemKind = serde_json::from_value(element["kind"].clone()).unwrap();
            let id = element["id"].as_str().unwrap();
            let found = items
                .iter()
                .filter(|i| i.kind == kind && i.id == id)
                .collect::<Vec<_>>();
            assert_eq!(found.len(), 1, "{system} {kind:?} {id}");
            assert_eq!(
                found[0].classification,
                routed(system, kind).unwrap_or(Unsupported),
                "{system} {kind:?} {id}"
            );
        }
        let manifest = items.iter().find(|i| i.kind == Manifest).unwrap();
        assert_eq!(
            (manifest.id.as_str(), manifest.classification),
            ("manifest", Unsupported)
        );
        assert_eq!(
            manifest.blocker.as_deref(),
            Some(format!("{system}: an inventory cannot produce an applicable manifest").as_str())
        );
        // The entry point dispatches on api_version and returns the same findings.
        let entry = riauth::migration::preflight(&serde_json::to_vec(&input).unwrap()).unwrap();
        for field in [
            "api_version",
            "source",
            "ready_for_plan",
            "blockers",
            "summary",
            "items",
        ] {
            assert_eq!(entry[field], report[field], "{system} {field}");
        }
        assert!(entry.get("manifest").is_none());
    }
    let ldap = riauth::migration::inventory(serde_json::from_value(inventory("openldap")).unwrap())
        .unwrap();
    let action = |kind: &str| {
        ldap["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["kind"] == kind && i["id"] == "*")
            .unwrap()["action"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    assert!(
        action("user").contains("docs/ldap.md")
            && action("user").contains("never adopted by DN or email")
    );
    assert!(action("password").contains("never falls back to a local password"));

    // Unknown formats, unsupported shapes and anything carrying extra data are rejected outright.
    let secret = "inventory-secret-must-not-appear";
    let base = json!({"api_version": "riauth.migration-inventory/v1", "system": "keycloak",
        "elements": [{"kind": "provider", "id": "grafana"}]});
    let with = |patch: Value| {
        let mut input = base.clone();
        for (k, v) in patch.as_object().unwrap() {
            input[k] = v.clone();
        }
        input
    };
    for (input, message) in [
        (
            with(json!({"api_version": "riauth.keycloak-import/v1"})),
            "Unsupported migration input",
        ),
        (
            json!({"system": "keycloak", "elements": []}),
            "Unsupported migration input",
        ),
        (
            with(json!({"client_secret": secret})),
            "Invalid migration inventory",
        ),
        (
            with(json!({"elements": [{"kind": secret, "id": "grafana"}]})),
            "Invalid migration inventory",
        ),
        (
            with(json!({"elements": [{"kind": "provider", "id": "grafana", "secret": secret}]})),
            "Invalid migration inventory",
        ),
        (
            with(json!({"elements": []})),
            "Declare at least one inventory element",
        ),
        (
            with(json!({"elements": [{"kind": "manifest", "id": "manifest"}]})),
            "cannot declare a manifest",
        ),
        (
            with(json!({"elements": [{"kind": "user", "id": "*"}, {"kind": "user", "id": "*"}]})),
            "Duplicate inventory element",
        ),
        (
            with(json!({"elements": [{"kind": "user", "id": ""}]})),
            "Inventory element IDs",
        ),
        (
            with(json!({"elements": [{"kind": "user", "id": "a\nb"}]})),
            "Inventory element IDs",
        ),
        (with(json!({"system": "Keycloak"})), "Inventory system"),
        (with(json!({"system": ""})), "Inventory system"),
        // An inventory cannot stand in for the Authentik bundle and vice versa.
        (
            with(json!({"api_version": "riauth.authentik-import/v1"})),
            "Invalid Authentik import bundle",
        ),
        // A misplaced secret in a typed Authentik field is not quoted back.
        (
            json!({"api_version": "riauth.authentik-import/v1", "issuer": "https://id.example.test",
                "users": [], "groups": [], "providers": [], "applications": [], "policy_bindings": [],
                "sources": [], "clients": {}, "passwords": {"alice": secret}}),
            "Invalid Authentik import bundle at line 1",
        ),
    ] {
        let error = riauth::migration::preflight(&serde_json::to_vec(&input).unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains(message), "{input}: {error}");
        assert!(!error.contains(secret), "{error}");
    }
    let error = riauth::migration::preflight(b"not json")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("Migration input is not JSON at line 1"),
        "{error}"
    );
}

#[test]
fn migration_inventory_classifies_source_native_kinds_as_unsupported() {
    use riauth::migration::{Classification::*, Finding, ItemKind::*};
    let inventory = |system: &str, elements: Value| json!({"api_version": "riauth.migration-inventory/v1", "system": system, "elements": elements});
    // `admin` is declared as both a realm and a role; uniqueness is by type and ID together.
    let keycloak = inventory(
        "keycloak",
        json!([
            {"source_kind": "realm", "id": "master"},
            {"source_kind": "realm", "id": "admin"},
            {"source_kind": "role", "id": "admin"},
            {"source_kind": "client-scope", "id": "profile"},
            {"source_kind": "required_action", "id": "*"},
            {"kind": "provider", "id": "grafana"}
        ]),
    );
    let report =
        riauth::migration::inventory(serde_json::from_value(keycloak.clone()).unwrap()).unwrap();
    assert_eq!(report["ready_for_plan"], false);
    assert!(report["manifest"].is_null());
    let items: Vec<Finding> = serde_json::from_value(report["items"].clone()).unwrap();
    assert_eq!(items.len(), 7);
    assert!(items.iter().all(|i| i.blocking && i.blocker.is_some()));
    for element in keycloak["elements"].as_array().unwrap() {
        let id = element["id"].as_str().unwrap();
        let found = items
            .iter()
            .filter(|i| {
                i.id == id
                    && match element["source_kind"].as_str() {
                        Some(native) => {
                            i.kind == SourceNative && i.source_kind.as_deref() == Some(native)
                        }
                        None => i.kind == Provider && i.source_kind.is_none(),
                    }
            })
            .collect::<Vec<_>>();
        assert_eq!(found.len(), 1, "{element}");
        assert_eq!(found[0].classification, Unsupported, "{element}");
    }
    let role = items
        .iter()
        .find(|i| i.source_kind.as_deref() == Some("role"))
        .unwrap();
    assert_eq!(
        role.blocker.as_deref(),
        Some("keycloak role admin: no riAuth converter; rebuild or retire it")
    );
    assert!(role.reason.contains("keycloak role elements") && role.action.contains("retire it"));
    // Only source-native findings carry the field, so Authentik reports are unchanged.
    let serialized = report["items"].as_array().unwrap();
    assert_eq!(
        serialized
            .iter()
            .filter(|i| i.get("source_kind").is_some())
            .count(),
        5
    );
    assert_eq!(
        serialized
            .iter()
            .find(|i| i["kind"] == "source_native")
            .unwrap()["kind"],
        "source_native"
    );

    // Directory routes never apply to a source-native type; Authentik still points at the bundle.
    for (system, expected) in [
        ("active-directory", Unsupported),
        ("entra-id", Unsupported),
        ("authentik", Manual),
    ] {
        let report = riauth::migration::inventory(
            serde_json::from_value(inventory(
                system,
                json!([{"source_kind": "gpo", "id": "default"}]),
            ))
            .unwrap(),
        )
        .unwrap();
        let item = &report["items"].as_array().unwrap()[1];
        assert_eq!(
            (item["kind"].as_str(), item["source_kind"].as_str()),
            (Some("source_native"), Some("gpo"))
        );
        assert_eq!(item["classification"], json!(expected), "{system}");
        assert_eq!(item["blocking"], true);
    }

    let secret = "native-kind-secret";
    for (elements, message) in [
        (
            json!([{"source_kind": "role", "id": "admin"}, {"source_kind": "role", "id": "admin"}]),
            "Duplicate inventory element",
        ),
        (
            json!([{"kind": "provider", "source_kind": "client", "id": "a"}]),
            "exactly one of kind or source_kind",
        ),
        (json!([{"id": "a"}]), "exactly one of kind or source_kind"),
        (
            json!([{"kind": "source_native", "id": "a"}]),
            "with source_kind instead of kind",
        ),
        (
            json!([{"source_kind": "user", "id": "a"}]),
            "must be declared with kind",
        ),
        (
            json!([{"source_kind": "authentication-flow", "id": "a"}]),
            "must be declared with kind",
        ),
        (
            json!([{"source_kind": "manifest", "id": "a"}]),
            "must be declared with kind",
        ),
        (
            json!([{"source_kind": "Realm", "id": "a"}]),
            "source_kind must be",
        ),
        (
            json!([{"source_kind": "1realm", "id": "a"}]),
            "source_kind must be",
        ),
        (
            json!([{"source_kind": "", "id": "a"}]),
            "source_kind must be",
        ),
        (
            json!([{"source_kind": "r".repeat(65), "id": "a"}]),
            "source_kind must be",
        ),
        (
            json!([{"source_kind": format!("role:{secret}"), "id": "a"}]),
            "source_kind must be",
        ),
        (
            json!([{"source_kind": "role", "id": "a", "secret": secret}]),
            "Invalid migration inventory",
        ),
        (
            json!([{"source_kind": 7, "id": secret}]),
            "Invalid migration inventory",
        ),
        // `kind` stays the closed riAuth taxonomy.
        (
            json!([{"kind": "realm", "id": "master"}]),
            "Invalid migration inventory",
        ),
    ] {
        let input = inventory("keycloak", elements);
        let error = riauth::migration::preflight(&serde_json::to_vec(&input).unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains(message), "{input}: {error}");
        assert!(!error.contains(secret), "{error}");
    }
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
            // This fixture represents a store written before activation records existed.
            tx.delete("meta", "version_activation")?;
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
