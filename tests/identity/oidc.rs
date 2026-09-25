use super::*;

#[test]
fn oidc_claims_verify_against_public_jwks() {
    let f = Fixture::new();
    f.client("app", false);
    let user = f.user("alice");
    let tokens = f.tokens("app", &user, None);
    let jwks = f.core.jwks().unwrap();
    let key = &jwks["keys"][0];
    assert!(key.get("d").is_none());
    let decoding =
        DecodingKey::from_rsa_components(key["n"].as_str().unwrap(), key["e"].as_str().unwrap())
            .unwrap();
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&["app"]);
    validation.set_issuer(&[f.core.config.issuer.as_str()]);
    let jwt =
        jsonwebtoken::decode::<Value>(tokens["id_token"].as_str().unwrap(), &decoding, &validation)
            .unwrap();
    assert_eq!(jwt.header.kid.as_deref(), key["kid"].as_str());
    assert_eq!(jwt.claims["nonce"], "expected-nonce");
    assert_eq!(jwt.claims["email"], "alice@example.test");
    assert_eq!(jwt.claims["email_verified"], false);
    assert_eq!(jwt.claims["preferred_username"], "alice");
    assert_eq!(jwt.claims["amr"], json!(["pwd"]));
    assert_eq!(
        jwt.claims["at_hash"],
        URL_SAFE_NO_PAD.encode(&Sha256::digest(text(&tokens, "access_token").as_bytes())[..16])
    );
    let access = jsonwebtoken::decode::<Value>(
        tokens["access_token"].as_str().unwrap(),
        &decoding,
        &validation,
    )
    .unwrap();
    assert_eq!(access.header.typ.as_deref(), Some("at+jwt"));
    let info = f
        .core
        .userinfo(tokens["access_token"].as_str().unwrap())
        .unwrap();
    assert_eq!(info["sub"], jwt.claims["sub"]);
    assert_eq!(info["name"], "Test User");
    assert!(
        f.core
            .userinfo(tokens["id_token"].as_str().unwrap())
            .is_err()
    );
    assert!(f.core.me(tokens["access_token"].as_str().unwrap()).is_err());
}

#[test]
fn redirect_pkce_and_code_replay_are_enforced() {
    let f = Fixture::new();
    f.client("app", false);
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.redirect_uri = "https://attacker.example/callback".into();
    assert!(f.core.authorize(&f.admin, request).is_err());
    let mut request = f.request("app", &verifier);
    request.code_challenge_method = "plain".into();
    assert!(f.core.authorize(&f.admin, request).is_err());
    let request = f.exchange_request("app", &f.admin, None);
    let mut wrong = request.clone();
    wrong.redirect_uri = Some("http://localhost:7777/other".into());
    assert_eq!(f.core.token(wrong).unwrap_err().code, "invalid_grant");
    let mut wrong = request.clone();
    wrong.code_verifier = Some(crypto::random_token(""));
    assert_eq!(f.core.token(wrong).unwrap_err().code, "invalid_grant");
    assert!(f.core.token(request.clone()).is_ok());
    assert_eq!(f.core.token(request).unwrap_err().code, "invalid_grant");
}

#[test]
fn concurrent_code_redemption_has_exactly_one_winner() {
    let f = Fixture::new();
    f.client("app", false);
    let request = f.exchange_request("app", &f.admin, None);
    let barrier = Barrier::new(2);
    std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..2)
            .map(|_| {
                let request = request.clone();
                let core = &f.core;
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    core.token(request).is_ok()
                })
            })
            .collect();
        assert_eq!(
            jobs.into_iter()
                .map(|j| j.join().unwrap())
                .filter(|ok| *ok)
                .count(),
            1
        );
    });
}

#[test]
fn grants_are_bound_to_the_original_client() {
    let f = Fixture::new();
    let secret = f.client("app", true);
    f.client("other", false);
    let request = f.exchange_request("app", &f.admin, secret.clone());
    let mut stolen = request.clone();
    stolen.client_id = Some("other".into());
    stolen.client_secret = None;
    assert_eq!(f.core.token(stolen).unwrap_err().code, "invalid_grant");
    let mut missing = request.clone();
    missing.client_secret = None;
    assert_eq!(f.core.token(missing).unwrap_err().code, "invalid_client");
    let tokens = f.core.token(request).unwrap();
    let refresh = TokenRequest {
        grant_type: "refresh_token".into(),
        client_id: Some("other".into()),
        refresh_token: Some(text(&tokens, "refresh_token")),
        ..Default::default()
    };
    assert_eq!(f.core.token(refresh).unwrap_err().code, "invalid_grant");
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_ok());
}

#[test]
fn refresh_rotation_replay_revokes_the_entire_family() {
    let f = Fixture::new();
    f.client("app", false);
    let tokens = f.tokens("app", &f.admin, None);
    let request = TokenRequest {
        grant_type: "refresh_token".into(),
        client_id: Some("app".into()),
        refresh_token: Some(text(&tokens, "refresh_token")),
        ..Default::default()
    };
    let rotated = f.core.token(request.clone()).unwrap();
    assert_ne!(tokens["refresh_token"], rotated["refresh_token"]);
    assert!(f.core.userinfo(&text(&rotated, "access_token")).is_ok());
    assert_eq!(f.core.token(request).unwrap_err().code, "invalid_grant");
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(f.core.userinfo(&text(&rotated, "access_token")).is_err());
    let replacement = TokenRequest {
        grant_type: "refresh_token".into(),
        client_id: Some("app".into()),
        refresh_token: Some(text(&rotated, "refresh_token")),
        ..Default::default()
    };
    assert_eq!(f.core.token(replacement).unwrap_err().code, "invalid_grant");
}

#[test]
fn refresh_scopes_can_only_shrink_and_expiry_is_absolute() {
    let f = Fixture::new();
    f.client("app", false);
    let tokens = f.tokens("app", &f.admin, None);
    let request = TokenRequest {
        grant_type: "refresh_token".into(),
        client_id: Some("app".into()),
        refresh_token: Some(text(&tokens, "refresh_token")),
        scope: Some("openid admin".into()),
        ..Default::default()
    };
    assert_eq!(
        f.core.token(request.clone()).unwrap_err().code,
        "invalid_scope"
    );
    let old: Grant = f
        .core
        .store
        .get("refresh", &digest(&text(&tokens, "refresh_token")))
        .unwrap()
        .unwrap();
    let mut request = request;
    request.scope = Some("openid offline_access".into());
    let rotated = f.core.token(request).unwrap();
    let new: Grant = f
        .core
        .store
        .get("refresh", &digest(&text(&rotated, "refresh_token")))
        .unwrap()
        .unwrap();
    assert_eq!(old.expires_at, new.expires_at);
    assert_eq!(
        f.core
            .userinfo(&text(&rotated, "access_token"))
            .unwrap()
            .as_object()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn device_pending_slowdown_approval_and_single_use() {
    let f = Fixture::new();
    f.client("app", false);
    let start = f
        .core
        .device_start(TokenRequest {
            client_id: Some("app".into()),
            scope: Some("openid offline_access".into()),
            ..Default::default()
        })
        .unwrap();
    let request = TokenRequest {
        grant_type: DEVICE_GRANT.into(),
        client_id: Some("app".into()),
        device_code: Some(text(&start, "device_code")),
        ..Default::default()
    };
    assert_eq!(
        f.core.token(request.clone()).unwrap_err().code,
        "authorization_pending"
    );
    assert_eq!(f.core.token(request.clone()).unwrap_err().code, "slow_down");
    let key = digest(&text(&start, "device_code"));
    f.core
        .store
        .write(|tx| {
            let mut d: Device = tx.get("devices", &key)?.unwrap();
            assert_eq!(d.interval, 10);
            d.last_poll_at = Some(now() - 20);
            tx.put("devices", &key, &d)
        })
        .unwrap();
    let details = f
        .core
        .device_details(&f.admin, &text(&start, "user_code"))
        .unwrap();
    assert_eq!(details["client_id"], "app");
    f.core
        .device_decide(&f.admin, &text(&start, "user_code"), true)
        .unwrap();
    assert!(
        f.core
            .device_decide(&f.admin, &text(&start, "user_code"), true)
            .is_err()
    );
    let tokens = f.core.token(request.clone()).unwrap();
    assert!(tokens["id_token"].is_string());
    assert_eq!(f.core.token(request).unwrap_err().code, "invalid_grant");
}

#[test]
fn device_denial_expiry_and_client_binding() {
    let f = Fixture::new();
    f.client("app", false);
    f.client("other", false);
    let start = f
        .core
        .device_start(TokenRequest {
            client_id: Some("app".into()),
            ..Default::default()
        })
        .unwrap();
    let request = TokenRequest {
        grant_type: DEVICE_GRANT.into(),
        client_id: Some("other".into()),
        device_code: Some(text(&start, "device_code")),
        ..Default::default()
    };
    assert_eq!(
        f.core.token(request.clone()).unwrap_err().code,
        "invalid_grant"
    );
    f.core
        .device_decide(&f.admin, &text(&start, "user_code"), false)
        .unwrap();
    let mut request = request;
    request.client_id = Some("app".into());
    assert_eq!(
        f.core.token(request.clone()).unwrap_err().code,
        "access_denied"
    );
    f.core
        .store
        .write(|tx| {
            let key = digest(&text(&start, "device_code"));
            let mut d: Device = tx.get("devices", &key)?.unwrap();
            d.expires_at = now() - 1;
            tx.put("devices", &key, &d)
        })
        .unwrap();
    assert_eq!(f.core.token(request).unwrap_err().code, "expired_token");
}

#[test]
fn logout_revokes_grants_from_that_session_only() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let second = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let tokens = f.tokens("app", &alice, None);
    let other = f.tokens("app", &second, None);
    f.core.logout(&alice).unwrap();
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(f.core.userinfo(&text(&other, "access_token")).is_ok());
}

#[test]
fn introspection_and_revocation_are_confined_to_the_client() {
    let f = Fixture::new();
    let secret = f.client("app", true);
    let other_secret = f.client("other", true);
    let tokens = f.tokens("app", &f.admin, secret.clone());
    let foreign = TokenRequest {
        client_id: Some("other".into()),
        client_secret: other_secret,
        token: Some(text(&tokens, "access_token")),
        ..Default::default()
    };
    assert_eq!(
        f.core.introspect(foreign.clone()).unwrap(),
        json!({"active": false})
    );
    f.core.revoke(foreign).unwrap();
    let own = TokenRequest {
        client_id: Some("app".into()),
        client_secret: secret,
        token: Some(text(&tokens, "access_token")),
        ..Default::default()
    };
    assert_eq!(f.core.introspect(own.clone()).unwrap()["active"], true);
    f.core.revoke(own.clone()).unwrap();
    assert_eq!(f.core.introspect(own).unwrap()["active"], false);
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
}

#[test]
fn service_credentials_have_no_user_identity_or_refresh_token() {
    let f = Fixture::new();
    let created = f
        .core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "worker".into(),
                name: "Worker".into(),
                confidential: true,
                redirect_uris: vec![],
                scopes: strings(&["api.read"]),
                allowed_groups: BTreeSet::new(),
                require_mfa: false,
                service: true,
                settings: Default::default(),
            },
        )
        .unwrap();
    let request = TokenRequest {
        grant_type: "client_credentials".into(),
        client_id: Some("worker".into()),
        client_secret: Some(text(&created, "client_secret")),
        ..Default::default()
    };
    let tokens = f.core.token(request.clone()).unwrap();
    assert!(tokens.get("id_token").is_none());
    assert!(tokens.get("refresh_token").is_none());
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(f.core.device_start(request.clone()).is_err());
    f.core.rotate_client_secret(&f.admin, "worker").unwrap();
    assert!(f.core.token(request).is_err());
}

#[test]
fn signing_key_rotation_keeps_previous_public_key() {
    let f = Fixture::new();
    f.client("app", false);
    let tokens = f.tokens("app", &f.admin, None);
    let before = f.core.jwks().unwrap();
    let rotated = f.core.rotate_key(&f.admin).unwrap();
    let after = f.core.jwks().unwrap();
    assert_ne!(before["keys"][0]["kid"], rotated["kid"]);
    assert_eq!(after["keys"].as_array().unwrap().len(), 2);
    assert!(
        after["keys"]
            .as_array()
            .unwrap()
            .contains(&before["keys"][0])
    );
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_ok());
}

#[test]
fn expired_codes_access_and_refresh_tokens_are_rejected() {
    let f = Fixture::new();
    f.client("app", false);
    let request = f.exchange_request("app", &f.admin, None);
    f.core
        .store
        .write(|tx| {
            let key = digest(request.code.as_ref().unwrap());
            let mut code: Code = tx.get("codes", &key)?.unwrap();
            code.expires_at = now() - 1;
            tx.put("codes", &key, &code)
        })
        .unwrap();
    assert_eq!(f.core.token(request).unwrap_err().code, "invalid_grant");
    let tokens = f.tokens("app", &f.admin, None);
    f.core
        .store
        .write(|tx| {
            for (bucket, token) in [
                ("access", text(&tokens, "access_token")),
                ("refresh", text(&tokens, "refresh_token")),
            ] {
                let key = digest(&token);
                let mut grant: Grant = tx.get(bucket, &key)?.unwrap();
                grant.expires_at = now() - 1;
                tx.put(bucket, &key, &grant)?;
            }
            Ok(())
        })
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
    f.core.cleanup().unwrap();
    assert!(
        f.core
            .store
            .get::<Grant>("refresh", &digest(&text(&tokens, "refresh_token")))
            .unwrap()
            .is_none()
    );
}

#[test]
fn cli_expiry_does_not_end_an_explicit_offline_grant() {
    let f = Fixture::new();
    f.client("app", false);
    let tokens = f.tokens("app", &f.admin, None);
    let sid = text(&f.core.me(&f.admin).unwrap(), "session_id");
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", &sid)?.unwrap();
            session.expires_at = now() - 1;
            tx.put("sessions", &sid, &session)
        })
        .unwrap();
    assert!(f.core.me(&f.admin).is_err());
    assert!(
        f.core
            .token(TokenRequest {
                grant_type: "refresh_token".into(),
                client_id: Some("app".into()),
                refresh_token: Some(text(&tokens, "refresh_token")),
                ..Default::default()
            })
            .is_ok()
    );
}

#[test]
fn reauthentication_is_request_bound_and_combined_prompts_work() {
    let f = Fixture::new();
    f.client("app", false);
    let mut request = f.request("app", &crypto::random_token(""));
    request.prompt = Some("login consent".into());
    assert_eq!(
        f.core
            .authorize(&f.admin, request.clone())
            .unwrap_err()
            .code,
        "login_required"
    );
    let prepared = f
        .core
        .authorization_prepare(Some(&f.admin), request.clone())
        .unwrap();
    assert_eq!(prepared["reauthentication_required"], true);
    let transaction = text(&prepared, "transaction_id");
    let new_login = f
        .core
        .login_for(
            "admin".into(),
            PASSWORD.into(),
            None,
            Some(transaction.clone()),
        )
        .unwrap();
    request.transaction_id = Some(transaction);
    let mut wrong = request.clone();
    wrong.nonce = Some("other-request".into());
    assert!(
        f.core
            .authorize(&text(&new_login, "session_token"), wrong)
            .is_err()
    );
    assert!(f.core.authorize(&f.admin, request.clone()).is_err());
    assert!(
        f.core
            .authorize(&text(&new_login, "session_token"), request.clone())
            .is_ok()
    );
    assert!(
        f.core
            .authorize(&text(&new_login, "session_token"), request)
            .is_err()
    );
    let mut zero = f.request("app", &crypto::random_token(""));
    zero.max_age = Some(0);
    assert_eq!(
        f.core
            .authorize(&text(&new_login, "session_token"), zero)
            .unwrap_err()
            .code,
        "login_required"
    );
}

#[test]
fn code_replay_revocation_requires_original_client_and_verifier() {
    let f = Fixture::new();
    f.client("app", false);
    f.client("other", false);
    let request = f.exchange_request("app", &f.admin, None);
    let tokens = f.core.token(request.clone()).unwrap();
    let mut wrong = request.clone();
    wrong.code_verifier = Some(crypto::random_token(""));
    assert!(f.core.token(wrong).is_err());
    let mut wrong = request.clone();
    wrong.client_id = Some("other".into());
    assert!(f.core.token(wrong).is_err());
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_ok());
    assert!(f.core.token(request).is_err());
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
fn empty_optional_parameters_and_native_callback_ports() {
    let f = Fixture::new();
    f.client("app", false);
    let parsed: TokenRequest = riauth::oidc::parse_form(vec![
        ("client_id".into(), "app".into()),
        ("scope".into(), "".into()),
    ])
    .unwrap();
    assert!(parsed.scope.is_none());
    assert!(f.core.device_start(parsed).is_ok());
    assert!(
        riauth::oidc::parse_form::<TokenRequest>(vec![
            ("scope".into(), "".into()),
            ("scope".into(), "openid".into())
        ])
        .is_err()
    );
    let patch = ClientPatch {
        settings: Some(riauth::model::ProviderSettings {
            native: true,
            ..Default::default()
        }),
        redirect_uris: Some(vec![
            "http://127.0.0.1:12345/callback".into(),
            "com.example.app:/callback".into(),
        ]),
        ..Default::default()
    };
    f.core.update_client(&f.admin, "app", patch).unwrap();
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.redirect_uri = "http://127.0.0.1:54321/callback".into();
    let callback = f.core.authorize(&f.admin, request.clone()).unwrap();
    let code = url::Url::parse(&callback)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .to_string();
    let mut token = TokenRequest {
        grant_type: "authorization_code".into(),
        client_id: Some("app".into()),
        code: Some(code),
        code_verifier: Some(verifier),
        redirect_uri: Some("http://127.0.0.1:12345/callback".into()),
        ..Default::default()
    };
    assert!(f.core.token(token.clone()).is_err());
    token.redirect_uri = Some(request.redirect_uri.clone());
    assert!(f.core.token(token).is_ok());
    request.redirect_uri = "http://127.0.0.1:54321/other".into();
    assert!(f.core.authorize(&f.admin, request).is_err());
}

#[tokio::test]
async fn authorization_errors_redirect_only_to_trusted_callbacks_and_bearer_errors_are_specific() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let f = Fixture::new();
    f.client("app", false);
    let app = riauth::api::router(f.core.clone());
    let mut request = f.request("app", &crypto::random_token(""));
    request.prompt = Some("none".into());
    let response = app
        .clone()
        .oneshot(
            Request::get(format!(
                "/oauth/authorize?{}",
                serde_urlencoded::to_string(&request).unwrap()
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FOUND);
    let location = url::Url::parse(response.headers()["location"].to_str().unwrap()).unwrap();
    let params: std::collections::BTreeMap<_, _> = location.query_pairs().collect();
    assert_eq!(params["error"], "login_required");
    assert_eq!(params["state"], request.state.as_deref().unwrap());
    request.redirect_uri = "https://attacker.invalid/callback".into();
    let response = app
        .clone()
        .oneshot(
            Request::get(format!(
                "/oauth/authorize?{}",
                serde_urlencoded::to_string(&request).unwrap()
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(!response.headers().contains_key("location"));
    let response = app
        .clone()
        .oneshot(
            Request::get("/oauth/userinfo")
                .header("authorization", "Bearer invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.headers()["www-authenticate"],
        "Bearer realm=\"riauth\", error=\"invalid_token\""
    );
    let response = app
        .oneshot(Request::get("/oauth/userinfo").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        response.headers()["www-authenticate"],
        "Bearer realm=\"riauth\""
    );
}

#[test]
fn stale_plans_and_partial_secret_failures_do_not_change_configuration() {
    let f = Fixture::new();
    let stale = f
        .core
        .plan_state(&f.admin, managed_manifest("stale"))
        .unwrap();
    f.core.create_group(&f.admin, "concurrent-change").unwrap();
    assert_eq!(
        f.core
            .apply_state(
                &f.admin,
                riauth::state::ApplyRequest {
                    plan: stale,
                    secrets: Default::default(),
                    run_id: None
                }
            )
            .unwrap_err()
            .status
            .as_u16(),
        409
    );
    let manifest: riauth::state::Manifest = serde_json::from_value(json!({"api_version": "riauth/v1", "users": [{"username": "planned-user", "display_name": "Planned", "password_ref": "env:TEST_PASSWORD", "password_version": "1"}], "clients": [{"client_id": "confidential", "name": "Confidential", "confidential": true, "scopes": ["openid"], "secret_ref": "env:TEST_CLIENT_SECRET", "secret_version": "1"}]})).unwrap();
    let plan = f.core.plan_state(&f.admin, manifest.clone()).unwrap();
    let mut secrets = std::collections::BTreeMap::from([(
        "env:TEST_PASSWORD".into(),
        "new-user-strong-password".into(),
    )]);
    assert!(
        f.core
            .apply_state(
                &f.admin,
                riauth::state::ApplyRequest {
                    plan: plan.clone(),
                    secrets: secrets.clone(),
                    run_id: None
                }
            )
            .is_err()
    );
    assert!(
        f.core
            .store
            .get::<String>("usernames", "planned-user")
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .get::<Client>("clients", "confidential")
            .unwrap()
            .is_none()
    );
    secrets.insert(
        "env:TEST_CLIENT_SECRET".into(),
        "client-secret-with-more-than-thirty-two-bytes".into(),
    );
    f.core
        .apply_state(
            &f.admin,
            riauth::state::ApplyRequest {
                plan,
                secrets,
                run_id: None,
            },
        )
        .unwrap();
    assert!(
        f.core
            .login(
                "planned-user".into(),
                "new-user-strong-password".into(),
                None
            )
            .is_ok()
    );
    assert!(
        f.core
            .plan_state(&f.admin, manifest)
            .unwrap()
            .changes
            .is_empty()
    );
}

#[test]
fn browser_reauthentication_cannot_be_moved_to_an_identical_request() {
    let f = Fixture::new();
    f.client("app", false);
    let mut request = f.request("app", &crypto::random_token(""));
    request.prompt = Some("login".into());
    let a = f.core.browser_start(request.clone(), None).unwrap();
    let b = f.core.browser_start(request, None).unwrap();
    let details = f
        .core
        .browser_details(&f.admin, &text(&a.body, "user_code"))
        .unwrap();
    let transaction = text(&details, "transaction_id");
    let session = f
        .core
        .login_for(
            "admin".into(),
            PASSWORD.into(),
            None,
            Some(transaction.clone()),
        )
        .unwrap();
    assert!(
        f.core
            .browser_decide(
                &text(&session, "session_token"),
                riauth::browser::BrowserDecision {
                    code: text(&b.body, "user_code"),
                    approve: true,
                    transaction_id: Some(transaction.clone()),
                    remember: false
                }
            )
            .is_err()
    );
    f.core
        .browser_decide(
            &text(&session, "session_token"),
            riauth::browser::BrowserDecision {
                code: text(&a.body, "user_code"),
                approve: true,
                transaction_id: Some(transaction),
                remember: false,
            },
        )
        .unwrap();
}

#[test]
fn private_key_jwt_binds_issuer_subject_audience_and_consumes_assertions_atomically() {
    use riauth::jose::{ASSERTION_TYPE, ClientAuthMethod};
    let f = Fixture::new();
    assert!(
        service_client(
            &f,
            "workload",
            ProviderSettings {
                token_endpoint_auth_method: Some(ClientAuthMethod::PrivateKeyJwt),
                jwks: Some(fixture_jwks(&f)),
                ..Default::default()
            }
        )
        .is_none()
    );
    let claims = json!({"iss":"workload", "sub":"workload", "aud":format!("{}/oauth/token", f.core.config.issuer), "iat":now(), "exp":now()+120, "jti":crypto::id()});
    let key = fixture_signing_key(&f);
    let request = |claims: &Value| TokenRequest {
        client_id: Some("workload".into()),
        grant_type: "client_credentials".into(),
        scope: Some("api.read".into()),
        client_assertion_type: Some(ASSERTION_TYPE.into()),
        client_assertion: Some(key.sign(claims, false).unwrap()),
        ..Default::default()
    };
    for field in ["iss", "sub", "aud"] {
        let mut bad = claims.clone();
        bad[field] = json!("other");
        assert_eq!(
            f.core.token(request(&bad)).unwrap_err().code,
            "invalid_client"
        );
    }
    let mut bad = claims.clone();
    bad["exp"] = json!(now() + 1000);
    assert_eq!(
        f.core.token(request(&bad)).unwrap_err().code,
        "invalid_client"
    );
    let shared = request(&claims);
    let results: Vec<_> = std::thread::scope(|scope| {
        (0..8)
            .map(|_| {
                let req = shared.clone();
                let core = &f.core;
                scope.spawn(move || core.token(req).is_ok())
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|t| t.join().unwrap())
            .collect()
    });
    assert_eq!(results.iter().filter(|ok| **ok).count(), 1);
    assert_eq!(f.core.token(shared).unwrap_err().code, "invalid_client");
    assert_eq!(
        f.core
            .token(TokenRequest {
                client_id: Some("workload".into()),
                client_secret: Some("fallback".into()),
                grant_type: "client_credentials".into(),
                ..Default::default()
            })
            .unwrap_err()
            .code,
        "invalid_client"
    );
    let mut claims = claims;
    claims["jti"] = json!(crypto::id());
    let out = f.core.token(request(&claims)).unwrap();
    claims["jti"] = json!(crypto::id());
    let mut inspect = request(&claims);
    inspect.token = Some(text(&out, "access_token"));
    assert_eq!(f.core.introspect(inspect.clone()).unwrap()["active"], true);
    assert_eq!(
        f.core.introspect(inspect).unwrap_err().code,
        "invalid_client"
    );
}

#[test]
fn client_write_cannot_replace_authentication_credentials_without_rotate() {
    use riauth::jose::{ASSERTION_TYPE, ClientAuthMethod};
    let f = Fixture::new();
    let original = fixture_signing_key(&f);
    let original_jwks = fixture_jwks(&f);
    service_client(
        &f,
        "workload",
        ProviderSettings {
            token_endpoint_auth_method: Some(ClientAuthMethod::PrivateKeyJwt),
            jwks: Some(original_jwks.clone()),
            ..Default::default()
        },
    );
    let writer = agent_token(
        &f,
        &[
            ("client.read", "client/workload"),
            ("client.write", "client/workload"),
        ],
    );
    let rotator = agent_token(
        &f,
        &[
            ("client.read", "client/workload"),
            ("client.write", "client/workload"),
            ("client.rotate", "client/workload"),
        ],
    );
    let (replacement_key, replacement_jwks) = alternate_client_keys();
    let mut settings = ProviderSettings {
        token_endpoint_auth_method: Some(ClientAuthMethod::PrivateKeyJwt),
        jwks: Some(replacement_jwks.clone()),
        ..Default::default()
    };
    assert_eq!(
        f.core
            .update_client(
                &writer,
                "workload",
                ClientPatch {
                    settings: Some(settings.clone()),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    let assertion = |key: &crypto::SigningKey| {
        let claims = json!({"iss":"workload","sub":"workload","aud":format!("{}/oauth/token", f.core.config.issuer),"iat":now(),"exp":now()+120,"jti":crypto::id()});
        TokenRequest {
            client_id: Some("workload".into()),
            grant_type: "client_credentials".into(),
            scope: Some("api.read".into()),
            client_assertion_type: Some(ASSERTION_TYPE.into()),
            client_assertion: Some(key.sign(&claims, false).unwrap()),
            ..Default::default()
        }
    };
    assert!(f.core.token(assertion(&original)).is_ok());
    assert_eq!(
        f.core.token(assertion(&replacement_key)).unwrap_err().code,
        "invalid_client"
    );

    let prior = f.core.token(assertion(&original)).unwrap();
    f.core
        .update_client(
            &rotator,
            "workload",
            ClientPatch {
                settings: Some(settings.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        f.core.token(assertion(&original)).unwrap_err().code,
        "invalid_client"
    );
    assert!(f.core.token(assertion(&replacement_key)).is_ok());
    assert_eq!(
        f.core
            .introspect(TokenRequest {
                client_id: Some("workload".into()),
                client_assertion_type: Some(ASSERTION_TYPE.into()),
                client_assertion: Some({
                    let claims = json!({"iss":"workload","sub":"workload","aud":format!("{}/oauth/token", f.core.config.issuer),"iat":now(),"exp":now()+120,"jti":crypto::id()});
                    replacement_key.sign(&claims, false).unwrap()
                }),
                token: Some(text(&prior, "access_token")),
                ..Default::default()
            })
            .unwrap()["active"],
        false
    );

    settings.token_endpoint_auth_method = Some(ClientAuthMethod::ClientSecretPost);
    settings.jwks = None;
    assert_eq!(
        f.core
            .update_client(
                &writer,
                "workload",
                ClientPatch {
                    settings: Some(settings.clone()),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    settings.token_endpoint_auth_method = Some(ClientAuthMethod::PrivateKeyJwt);
    settings.jwks = Some(replacement_jwks.clone());
    settings.machine_trust = vec![riauth::jose::MachineTrust {
        issuer: "https://ci.example.test".into(),
        subject: "repo:example/service:environment:production".into(),
        jwks: original_jwks,
        scopes: strings(&["api.read"]),
    }];
    assert_eq!(
        f.core
            .update_client(
                &writer,
                "workload",
                ClientPatch {
                    settings: Some(settings.clone()),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    f.core
        .update_client(
            &rotator,
            "workload",
            ClientPatch {
                settings: Some(settings.clone()),
                ..Default::default()
            },
        )
        .unwrap();

    let (_, next_jwks) = alternate_client_keys();
    settings.machine_trust.clear();
    settings.jwks = Some(next_jwks.clone());
    let mut manifest: riauth::state::Manifest =
        serde_json::from_value(f.core.export_state(&rotator).unwrap()["manifest"].clone()).unwrap();
    let client = manifest
        .clients
        .iter_mut()
        .find(|c| c.client_id == "workload")
        .unwrap();
    client.settings.jwks = Some(next_jwks);
    assert_eq!(
        f.core
            .plan_state(&writer, manifest.clone())
            .map(|_| ())
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    let plan = f.core.plan_state(&rotator, manifest).unwrap();
    assert!(plan.changes.iter().any(|c| c.credential_change));
    f.core
        .apply_state(
            &rotator,
            riauth::state::ApplyRequest {
                plan,
                secrets: Default::default(),
                run_id: None,
            },
        )
        .unwrap();
}

#[test]
fn workload_jwt_grants_require_pinned_subject_scope_and_one_use() {
    let f = Fixture::new();
    service_client(
        &f,
        "workload",
        ProviderSettings {
            allowed_grants: strings(&[riauth::jose::JWT_GRANT]),
            machine_trust: vec![riauth::jose::MachineTrust {
                issuer: "https://ci.example.test".into(),
                subject: "repo:example/service:environment:production".into(),
                jwks: fixture_jwks(&f),
                scopes: strings(&["api.read"]),
            }],
            ..Default::default()
        },
    );
    let key = fixture_signing_key(&f);
    let claims = json!({"iss":"https://ci.example.test", "sub":"repo:example/service:environment:production", "aud":format!("{}/oauth/token", f.core.config.issuer), "iat":now(),"exp":now()+120,"jti":crypto::id()});
    let mut req = TokenRequest {
        grant_type: riauth::jose::JWT_GRANT.into(),
        client_id: Some("workload".into()),
        scope: Some("api.read".into()),
        assertion: Some(key.sign(&claims, false).unwrap()),
        ..Default::default()
    };
    let mut wrong = claims.clone();
    wrong["sub"] = json!("repo:attacker/fork");
    req.assertion = Some(key.sign(&wrong, false).unwrap());
    assert_eq!(f.core.token(req.clone()).unwrap_err().code, "invalid_grant");
    req.assertion = Some(key.sign(&claims, false).unwrap());
    req.scope = Some("api.write".into());
    assert_eq!(f.core.token(req.clone()).unwrap_err().code, "invalid_scope");
    req.scope = Some("api.read".into());
    assert!(f.core.token(req.clone()).unwrap()["access_token"].is_string());
    assert_eq!(f.core.token(req).unwrap_err().code, "invalid_grant");
}

#[test]
fn token_exchange_requires_bilateral_trust_actor_proof_and_revokes_with_parent() {
    use riauth::exchange::{ACCESS_TOKEN, ExchangePolicy, TOKEN_EXCHANGE};
    let f = Fixture::new();
    f.client("frontend", false);
    let mut frontend: Client = f.core.store.get("clients", "frontend").unwrap().unwrap();
    frontend.scopes.insert("api.read".into());
    f.core
        .update_client(
            &f.admin,
            "frontend",
            ClientPatch {
                scopes: Some(frontend.scopes),
                ..Default::default()
            },
        )
        .unwrap();
    let target_secret = service_client(
        &f,
        "api",
        ProviderSettings {
            exchange_from: strings(&["worker"]),
            ..Default::default()
        },
    )
    .unwrap();
    let worker_secret = service_client(
        &f,
        "worker",
        ProviderSettings {
            allowed_grants: strings(&["client_credentials", TOKEN_EXCHANGE]),
            exchange: Some(ExchangePolicy {
                subject_clients: strings(&["frontend"]),
                target_clients: strings(&["api"]),
                scopes: strings(&["api.read"]),
                allow_delegation: true,
                allow_impersonation: false,
            }),
            ..Default::default()
        },
    )
    .unwrap();
    let alice = f.user("alice");
    let verifier = crypto::random_token("");
    let mut authorize = f.request("frontend", &verifier);
    authorize.scope.push_str(" api.read");
    let redirect = f.core.authorize(&alice, authorize).unwrap();
    let code = url::Url::parse(&redirect)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    let source = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("frontend".into()),
            code: Some(code),
            code_verifier: Some(verifier),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            ..Default::default()
        })
        .unwrap();
    let actor = f
        .core
        .token(TokenRequest {
            grant_type: "client_credentials".into(),
            client_id: Some("worker".into()),
            client_secret: Some(worker_secret.clone()),
            scope: Some("api.read".into()),
            ..Default::default()
        })
        .unwrap();
    let mut req = TokenRequest {
        grant_type: TOKEN_EXCHANGE.into(),
        client_id: Some("worker".into()),
        client_secret: Some(worker_secret),
        subject_token: Some(text(&source, "access_token")),
        subject_token_type: Some(ACCESS_TOKEN.into()),
        audience: Some("api".into()),
        scope: Some("api.read".into()),
        ..Default::default()
    };
    assert_eq!(
        f.core.token(req.clone()).unwrap_err().code,
        "unauthorized_client"
    );
    req.actor_token = Some(text(&actor, "access_token"));
    req.actor_token_type = Some(ACCESS_TOKEN.into());
    let mut wrong = req.clone();
    wrong.actor_token = Some(text(&source, "access_token"));
    assert_eq!(f.core.token(wrong).unwrap_err().code, "invalid_grant");
    let mut wrong = req.clone();
    wrong.scope = Some("api.write".into());
    assert_eq!(f.core.token(wrong).unwrap_err().code, "invalid_scope");
    let out = f.core.token(req).unwrap();
    assert_eq!(out["issued_token_type"], ACCESS_TOKEN);
    assert!(out.get("refresh_token").is_none() && out.get("id_token").is_none());
    let verified = fixture_jwks(&f)
        .verify(&text(&out, "access_token"), &f.core.config.issuer, "api")
        .unwrap();
    assert_eq!(verified["act"]["sub"], "service:worker");
    assert_eq!(verified["client_id"], "worker");
    let inspect = TokenRequest {
        client_id: Some("api".into()),
        client_secret: Some(target_secret),
        token: Some(text(&out, "access_token")),
        ..Default::default()
    };
    assert_eq!(f.core.introspect(inspect.clone()).unwrap()["active"], true);
    f.core
        .revoke(TokenRequest {
            client_id: Some("frontend".into()),
            token: Some(text(&source, "access_token")),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(f.core.introspect(inspect).unwrap()["active"], false);
}

#[test]
fn token_exchange_enforces_target_dpop_binding_for_impersonation_and_delegation() {
    use riauth::exchange::{ACCESS_TOKEN, ExchangePolicy, TOKEN_EXCHANGE};
    let f = Fixture::new();
    f.client("frontend", false);
    let mut frontend: Client = f.core.store.get("clients", "frontend").unwrap().unwrap();
    frontend.scopes.insert("api.read".into());
    f.core
        .update_client(
            &f.admin,
            "frontend",
            ClientPatch {
                scopes: Some(frontend.scopes),
                ..Default::default()
            },
        )
        .unwrap();
    let target_secret = service_client(
        &f,
        "api",
        ProviderSettings {
            exchange_from: strings(&["worker"]),
            dpop_bound_access_tokens: true,
            allowed_grants: strings(&["client_credentials"]),
            ..Default::default()
        },
    )
    .unwrap();
    let worker_secret = service_client(
        &f,
        "worker",
        ProviderSettings {
            allowed_grants: strings(&["client_credentials", TOKEN_EXCHANGE]),
            exchange: Some(ExchangePolicy {
                subject_clients: strings(&["frontend", "worker"]),
                target_clients: strings(&["api"]),
                scopes: strings(&["api.read"]),
                allow_delegation: true,
                allow_impersonation: true,
            }),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        f.core
            .token(TokenRequest {
                grant_type: "client_credentials".into(),
                client_id: Some("api".into()),
                client_secret: Some(target_secret.clone()),
                scope: Some("api.read".into()),
                ..Default::default()
            })
            .unwrap_err()
            .code,
        "invalid_dpop_proof"
    );
    let alice = f.user("alice");
    let verifier = crypto::random_token("");
    let mut authorize = f.request("frontend", &verifier);
    authorize.scope.push_str(" api.read");
    let redirect = f.core.authorize(&alice, authorize).unwrap();
    let code = url::Url::parse(&redirect)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    let source = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("frontend".into()),
            code: Some(code),
            code_verifier: Some(verifier),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            ..Default::default()
        })
        .unwrap();
    let endpoint = format!("{}/oauth/token", f.core.config.issuer);
    let key = crypto::SigningKey::generate_algorithm("ES256").unwrap();
    let mut impersonation = TokenRequest {
        grant_type: TOKEN_EXCHANGE.into(),
        client_id: Some("worker".into()),
        client_secret: Some(worker_secret.clone()),
        subject_token: Some(text(&source, "access_token")),
        subject_token_type: Some(ACCESS_TOKEN.into()),
        audience: Some("api".into()),
        scope: Some("api.read".into()),
        ..Default::default()
    };
    assert_eq!(
        f.core.token(impersonation.clone()).unwrap_err().code,
        "invalid_dpop_proof"
    );
    impersonation.dpop_proof = Some(dpop_proof(&key, "POST", &endpoint, None));
    let out = f.core.token(impersonation).unwrap();
    assert_eq!(out["token_type"], "DPoP");
    let inspect = f
        .core
        .introspect(TokenRequest {
            client_id: Some("api".into()),
            client_secret: Some(target_secret.clone()),
            token: Some(text(&out, "access_token")),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(inspect["active"], true);
    assert_eq!(
        inspect["cnf"]["jkt"],
        riauth::dpop::thumbprint(&serde_json::from_value(key.jwk().unwrap()).unwrap()).unwrap()
    );
    let actor = f
        .core
        .token(TokenRequest {
            grant_type: "client_credentials".into(),
            client_id: Some("worker".into()),
            client_secret: Some(worker_secret.clone()),
            scope: Some("api.read".into()),
            ..Default::default()
        })
        .unwrap();
    let mut delegation = TokenRequest {
        grant_type: TOKEN_EXCHANGE.into(),
        client_id: Some("worker".into()),
        client_secret: Some(worker_secret),
        subject_token: Some(text(&source, "access_token")),
        subject_token_type: Some(ACCESS_TOKEN.into()),
        actor_token: Some(text(&actor, "access_token")),
        actor_token_type: Some(ACCESS_TOKEN.into()),
        audience: Some("api".into()),
        scope: Some("api.read".into()),
        ..Default::default()
    };
    assert_eq!(
        f.core.token(delegation.clone()).unwrap_err().code,
        "invalid_dpop_proof"
    );
    let key = crypto::SigningKey::generate_algorithm("ES256").unwrap();
    delegation.dpop_proof = Some(dpop_proof(&key, "POST", &endpoint, None));
    let out = f.core.token(delegation).unwrap();
    assert_eq!(out["token_type"], "DPoP");
    let inspect = f
        .core
        .introspect(TokenRequest {
            client_id: Some("api".into()),
            client_secret: Some(target_secret),
            token: Some(text(&out, "access_token")),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(inspect["active"], true);
    assert_eq!(
        inspect["cnf"]["jkt"],
        riauth::dpop::thumbprint(&serde_json::from_value(key.jwk().unwrap()).unwrap()).unwrap()
    );
}

#[test]
fn dynamic_registration_constrains_metadata_uses_and_revocation() {
    use riauth::registration::{RegistrationRequest, RegistrationTemplate};
    let f = Fixture::new();
    let result = f
        .core
        .registration_template(
            &f.admin,
            RegistrationTemplate {
                id: "test".into(),
                redirect_uris: vec!["https://app.example.test/callback".into()],
                scopes: strings(&["openid", "profile"]),
                grant_types: strings(&["authorization_code"]),
                auth_methods: strings(&["private_key_jwt", "client_secret_basic"]),
                settings: ProviderSettings::default(),
                allowed_groups: Default::default(),
                require_mfa: false,
                ttl: 300,
                max_uses: 2,
            },
        )
        .unwrap();
    let credential = text(&result, "initial_access_token");
    let mut request = RegistrationRequest {
        redirect_uris: vec!["https://evil.example.test/callback".into()],
        token_endpoint_auth_method: Some("private_key_jwt".into()),
        jwks: Some(fixture_jwks(&f)),
        ..Default::default()
    };
    assert_eq!(
        f.core
            .dynamic_register(&credential, request.clone())
            .unwrap_err()
            .code,
        "invalid_redirect_uri"
    );
    request.redirect_uris = vec!["https://app.example.test/callback".into()];
    let mut wrong = request.clone();
    wrong.scope = Some("openid admin".into());
    assert_eq!(
        f.core
            .dynamic_register(&credential, wrong)
            .unwrap_err()
            .code,
        "invalid_client_metadata"
    );
    let out = f
        .core
        .dynamic_register(&credential, request.clone())
        .unwrap();
    assert!(out.get("client_secret").is_none());
    assert_eq!(out["token_endpoint_auth_method"], "private_key_jwt");
    let c: Client = f
        .core
        .store
        .get("clients", &text(&out, "client_id"))
        .unwrap()
        .unwrap();
    assert!(c.confidential());
    assert!(
        f.core
            .dynamic_register(&credential, request.clone())
            .is_ok()
    );
    assert_eq!(
        f.core
            .dynamic_register(&credential, request.clone())
            .unwrap_err()
            .status,
        401
    );
    f.core.revoke_registration(&f.admin, "test").unwrap();
    assert_eq!(
        f.core
            .dynamic_register(&credential, request)
            .unwrap_err()
            .status,
        401
    );
    let plans = f.core.registration_templates(&f.admin).unwrap();
    assert!(!plans.to_string().contains(&credential));
}

#[test]
fn signed_and_pushed_requests_are_bound_immutable_expiring_and_one_use() {
    let f = Fixture::new();
    let secret = f.client("app", true).unwrap();
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    jwks: Some(fixture_jwks(&f)),
                    require_signed_request: true,
                    require_pushed_authorization_requests: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let alice = f.user("alice");
    let verifier = crypto::random_token("");
    let mut claims = serde_json::to_value(f.request("app", &verifier)).unwrap();
    claims.as_object_mut().unwrap().retain(|_, v| !v.is_null());
    claims.as_object_mut().unwrap().remove("decision");
    claims["iss"] = json!("app");
    claims["aud"] = json!(f.core.config.issuer);
    claims["iat"] = json!(now());
    claims["exp"] = json!(now() + 120);
    claims["jti"] = json!(crypto::id());
    let signed = fixture_signing_key(&f)
        .sign_type(&claims, "oauth-authz-req+jwt")
        .unwrap();
    let pairs = vec![
        ("client_id".into(), "app".into()),
        ("client_secret".into(), secret.clone()),
        ("request".into(), signed.clone()),
    ];
    let pushed = f
        .core
        .push_authorization(&axum::http::HeaderMap::new(), pairs.clone())
        .unwrap();
    let resolve = vec![
        ("client_id".into(), "app".into()),
        ("request_uri".into(), text(&pushed, "request_uri")),
    ];
    let mut tamper = resolve.clone();
    tamper.push(("redirect_uri".into(), "https://evil.test".into()));
    assert_eq!(
        f.core.resolve_authorization(tamper).unwrap_err().code,
        "invalid_request_uri"
    );
    let mut wrong = resolve.clone();
    wrong[0].1 = "other".into();
    assert_eq!(
        f.core.resolve_authorization(wrong).unwrap_err().code,
        "invalid_request_uri"
    );
    let mut request = f.core.resolve_authorization(resolve.clone()).unwrap();
    request.decision = Some("approve".into());
    let callback = f.core.authorize(&alice, request).unwrap();
    assert!(callback.contains("code="));
    assert_eq!(
        f.core.resolve_authorization(resolve).unwrap_err().code,
        "invalid_request_uri"
    );
    assert_eq!(
        f.core
            .push_authorization(&axum::http::HeaderMap::new(), pairs)
            .unwrap_err()
            .code,
        "invalid_request_object"
    );
    let mut outer = vec![
        ("client_id".into(), "app".into()),
        ("request".into(), signed),
    ];
    outer.push(("scope".into(), "openid admin".into()));
    assert_eq!(
        f.core.resolve_authorization(outer).unwrap_err().code,
        "invalid_request_object"
    );
}

#[tokio::test]
async fn signed_authorization_responses_and_form_post_escape_callback_values() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let mut request = f.request("app", &crypto::random_token(""));
    request.response_mode = Some("query.jwt".into());
    let url = url::Url::parse(&f.core.authorize(&alice, request.clone()).unwrap()).unwrap();
    let response = url
        .query_pairs()
        .find(|(k, _)| k == "response")
        .unwrap()
        .1
        .into_owned();
    let payload = fixture_jwks(&f)
        .verify(&response, &f.core.config.issuer, "app")
        .unwrap();
    assert_eq!(payload["state"], "state with & delimiters");
    assert!(payload["code"].is_string());
    assert!(url.query_pairs().all(|(k, _)| k != "code" && k != "state"));
    request.response_mode = Some("form_post.jwt".into());
    request.state = Some("\"><script>alert(1)</script>".into());
    let value = serde_json::to_value(&request).unwrap();
    let pairs: Vec<_> = value
        .as_object()
        .unwrap()
        .iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k.as_str(), s)))
        .collect();
    let response = riauth::api::router(f.core.clone())
        .oneshot(
            Request::post("/oauth/authorize")
                .header("authorization", format!("Bearer {alice}"))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(serde_urlencoded::to_string(pairs).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert!(
        response.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("script-src 'nonce-")
    );
    let body = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("method=\"post\"") && body.contains("name=\"response\""));
    assert!(!body.contains("alert(1)"));
}

#[test]
fn bound_key_profile_binds_only_id_tokens_unless_access_binding_is_required() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let mut client: Client = f.core.store.get("clients", "app").unwrap().unwrap();
    client.scopes.insert("bound_key".into());
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                scopes: Some(client.scopes),
                ..Default::default()
            },
        )
        .unwrap();
    let verifier = crypto::random_token("");
    let mut authorization = f.request("app", &verifier);
    authorization.scope.push_str(" bound_key");
    let url = url::Url::parse(&f.core.authorize(&alice, authorization).unwrap()).unwrap();
    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    let key = crypto::SigningKey::generate_algorithm("ES256").unwrap();
    let mut request = TokenRequest {
        client_id: Some("app".into()),
        grant_type: "authorization_code".into(),
        code: Some(code),
        code_verifier: Some(verifier),
        redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
        ..Default::default()
    };
    assert_eq!(
        f.core.token(request.clone()).unwrap_err().code,
        "invalid_dpop_proof"
    );
    request.dpop_proof = Some(dpop_proof(
        &key,
        "POST",
        &format!("{}/oauth/token", f.core.config.issuer),
        None,
    ));
    let out = f.core.token(request).unwrap();
    assert_eq!(out["token_type"], "Bearer");
    assert!(f.core.userinfo(&text(&out, "access_token")).is_ok());
    let header = jsonwebtoken::decode_header(text(&out, "id_token")).unwrap();
    assert_eq!(header.typ.as_deref(), Some("dpop+id_token"));
    let claims = fixture_jwks(&f)
        .verify(&text(&out, "id_token"), &f.core.config.issuer, "app")
        .unwrap();
    let jwk = serde_json::from_value(key.jwk().unwrap()).unwrap();
    assert_eq!(
        claims["cnf"]["jkt"],
        riauth::dpop::thumbprint(&jwk).unwrap()
    );
}

#[test]
fn pairwise_subjects_are_stable_by_sector_and_token_management_is_directional() {
    let f = Fixture::new();
    f.client("a", false);
    let bsecret = f.client("b", true).unwrap();
    f.client("c", false);
    let alice = f.user("alice");
    for (cid, sector) in [
        ("a", "one.example.test"),
        ("b", "one.example.test"),
        ("c", "two.example.test"),
    ] {
        f.core
            .update_client(
                &f.admin,
                cid,
                ClientPatch {
                    settings: Some(ProviderSettings {
                        pairwise_sector: Some(sector.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    let a = f.tokens("a", &alice, None);
    let b = f.tokens("b", &alice, Some(bsecret.clone()));
    let c = f.tokens("c", &alice, None);
    let asub = f.core.userinfo(&text(&a, "access_token")).unwrap()["sub"].clone();
    assert_eq!(
        asub,
        f.core.userinfo(&text(&b, "access_token")).unwrap()["sub"]
    );
    assert_ne!(
        asub,
        f.core.userinfo(&text(&c, "access_token")).unwrap()["sub"]
    );
    let inspect = TokenRequest {
        client_id: Some("b".into()),
        client_secret: Some(bsecret),
        token: Some(text(&a, "access_token")),
        ..Default::default()
    };
    assert_eq!(f.core.introspect(inspect.clone()).unwrap()["active"], false);
    f.core
        .update_client(
            &f.admin,
            "a",
            ClientPatch {
                settings: Some(ProviderSettings {
                    pairwise_sector: Some("one.example.test".into()),
                    token_managers: strings(&["b"]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let inspected = f.core.introspect(inspect.clone()).unwrap();
    assert_eq!(inspected["active"], true);
    assert_eq!(inspected["sub"], asub);
    f.core.revoke(inspect).unwrap();
    assert!(f.core.userinfo(&text(&a, "access_token")).is_err());
    assert!(f.core.userinfo(&text(&b, "access_token")).is_ok());
}

#[tokio::test]
async fn one_store_preserves_distinct_provider_issuers_and_discovery_paths() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new();
    f.client("a", false);
    f.client("b", false);
    let alice = f.user("alice");
    let issuer = "http://localhost:9000/application/o/a/";
    f.core
        .update_client(
            &f.admin,
            "a",
            ClientPatch {
                settings: Some(ProviderSettings {
                    issuer: Some(issuer.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let a = f.tokens("a", &alice, None);
    let b = f.tokens("b", &alice, None);
    let jwks: riauth::jose::PublicJwks = serde_json::from_value(f.core.jwks().unwrap()).unwrap();
    assert!(jwks.verify(&text(&a, "id_token"), issuer, "a").is_ok());
    assert!(
        jwks.verify(&text(&a, "id_token"), &f.core.config.issuer, "a")
            .is_err()
    );
    assert!(
        jwks.verify(&text(&b, "id_token"), &f.core.config.issuer, "b")
            .is_ok()
    );
    let router = riauth::api::router(f.core.clone());
    let response = router
        .oneshot(
            Request::get("/application/o/a/.well-known/openid-configuration")
                .header("host", "localhost:9000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let metadata: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(metadata["issuer"], issuer);
    assert_eq!(
        metadata["token_endpoint"],
        "http://localhost:9000/oauth/token"
    );
    assert!(
        f.core
            .end_session(
                riauth::logout::LogoutRequest {
                    id_token_hint: Some(text(&a, "id_token")),
                    ..Default::default()
                },
                None,
                Some(&alice)
            )
            .is_ok()
    );
    assert!(f.core.userinfo(&text(&b, "access_token")).is_err()); // Shared OP session was ended.
}

#[test]
fn cbc_hmac_jwe_roundtrips_with_an_independent_openssl_implementation() {
    use openssl::{
        encrypt::Decrypter,
        hash::MessageDigest,
        pkey::PKey,
        rsa::{Padding, Rsa},
        sign::Signer,
    };
    let private = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let key = riauth::encryption::EncryptionKey {
        kid: "cbc-fixture".into(),
        content_encryption: "A256CBC-HS512".into(),
        public_key_pem: String::from_utf8(private.public_key_to_pem().unwrap()).unwrap(),
    };
    let plaintext = "a.nested.jwt";
    let token = key.encrypt(plaintext).unwrap();
    let parts: Vec<_> = token.split('.').collect();
    assert_eq!(parts.len(), 5);
    let wrapped = URL_SAFE_NO_PAD.decode(parts[1]).unwrap();
    let mut decryptor = Decrypter::new(&private).unwrap();
    decryptor.set_rsa_padding(Padding::PKCS1_OAEP).unwrap();
    decryptor.set_rsa_oaep_md(MessageDigest::sha256()).unwrap();
    decryptor.set_rsa_mgf1_md(MessageDigest::sha256()).unwrap();
    let mut cek = vec![0; decryptor.decrypt_len(&wrapped).unwrap()];
    let n = decryptor.decrypt(&wrapped, &mut cek).unwrap();
    cek.truncate(n);
    assert_eq!(cek.len(), 64);
    let iv = URL_SAFE_NO_PAD.decode(parts[2]).unwrap();
    let ciphertext = URL_SAFE_NO_PAD.decode(parts[3]).unwrap();
    let tag = URL_SAFE_NO_PAD.decode(parts[4]).unwrap();
    let hmac = PKey::hmac(&cek[..32]).unwrap();
    let mut signer = Signer::new(MessageDigest::sha512(), &hmac).unwrap();
    signer.update(parts[0].as_bytes()).unwrap();
    signer.update(&iv).unwrap();
    signer.update(&ciphertext).unwrap();
    signer
        .update(&((parts[0].len() as u64) * 8).to_be_bytes())
        .unwrap();
    assert_eq!(tag, signer.sign_to_vec().unwrap()[..32]);
    let actual = openssl::symm::decrypt(
        openssl::symm::Cipher::aes_256_cbc(),
        &cek[32..],
        Some(&iv),
        &ciphertext,
    )
    .unwrap();
    assert_eq!(actual, plaintext.as_bytes());
}

#[test]
fn dpop_binds_code_refresh_resource_and_replay_revocation_to_the_key() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let key = crypto::SigningKey::generate_algorithm("ES256").unwrap();
    let attacker = crypto::SigningKey::generate_algorithm("ES256").unwrap();
    let endpoint = format!("{}/oauth/token", f.core.config.issuer);
    let mut request = f.exchange_request("app", &alice, None);
    request.dpop_proof = Some(dpop_proof(&key, "POST", &endpoint, None));
    let out = f.core.token(request.clone()).unwrap();
    assert_eq!(out["token_type"], "DPoP");
    let access = text(&out, "access_token");
    assert_eq!(f.core.userinfo(&access).unwrap_err().status, 401);
    let uri = format!("{}/oauth/userinfo", f.core.config.issuer);
    let proof = dpop_proof(&key, "GET", &uri, Some(&access));
    assert_eq!(
        f.core
            .userinfo_with_proof(&access, Some(&proof), "GET")
            .unwrap()["preferred_username"],
        "alice"
    );
    assert_eq!(
        f.core
            .userinfo_with_proof(&access, Some(&proof), "GET")
            .unwrap_err()
            .code,
        "invalid_dpop_proof"
    );
    request.dpop_proof = Some(dpop_proof(&attacker, "POST", &endpoint, None));
    assert_eq!(
        f.core.token(request).unwrap_err().code,
        "invalid_dpop_proof"
    );
    let mut refresh = TokenRequest {
        client_id: Some("app".into()),
        grant_type: "refresh_token".into(),
        refresh_token: Some(text(&out, "refresh_token")),
        ..Default::default()
    };
    assert_eq!(
        f.core.token(refresh.clone()).unwrap_err().code,
        "invalid_dpop_proof"
    );
    refresh.dpop_proof = Some(dpop_proof(&key, "POST", &endpoint, None));
    let rotated = f.core.token(refresh.clone()).unwrap();
    refresh.dpop_proof = Some(dpop_proof(&attacker, "POST", &endpoint, None));
    assert_eq!(
        f.core.token(refresh.clone()).unwrap_err().code,
        "invalid_dpop_proof"
    );
    let access = text(&rotated, "access_token");
    let proof = dpop_proof(&key, "GET", &uri, Some(&access));
    assert!(
        f.core
            .userinfo_with_proof(&access, Some(&proof), "GET")
            .is_ok()
    );
    refresh.dpop_proof = Some(dpop_proof(&key, "POST", &endpoint, None));
    assert_eq!(f.core.token(refresh).unwrap_err().code, "invalid_grant");
    let proof = dpop_proof(&key, "GET", &uri, Some(&access));
    assert!(
        f.core
            .userinfo_with_proof(&access, Some(&proof), "GET")
            .is_err()
    );
}

#[test]
fn resource_indicators_bind_consent_code_refresh_audience_and_online_policy() {
    let f = Fixture::new();
    let secret = f.client("app", true);
    let alice = f.user("alice");
    let scopes = strings(&["openid", "profile", "email", "groups", "offline_access"]);
    let resources = [
        ("https://api.example/a".into(), scopes.clone()),
        ("https://api.example/b".into(), scopes),
    ]
    .into();
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    resources,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let verifier = crypto::random_token("");
    let mut authorize = f.request("app", &verifier);
    authorize.resource = Some("https://api.example/a".into());
    let details = f.core.authorization_details(authorize.clone()).unwrap();
    assert_eq!(details["resource"], "https://api.example/a");
    let uri = url::Url::parse(&f.core.authorize(&alice, authorize).unwrap()).unwrap();
    let code = uri
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    let mut request = TokenRequest {
        grant_type: "authorization_code".into(),
        client_id: Some("app".into()),
        client_secret: secret.clone(),
        code: Some(code),
        code_verifier: Some(verifier),
        redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
        resource: Some("https://api.example/b".into()),
        ..Default::default()
    };
    assert_eq!(
        f.core.token(request.clone()).unwrap_err().code,
        "invalid_target"
    );
    request.resource = None;
    let tokens = f.core.token(request).unwrap();
    let access = text(&tokens, "access_token");
    assert_eq!(
        fixture_jwks(&f)
            .verify(&access, &f.core.config.issuer, "https://api.example/a")
            .unwrap()["aud"],
        "https://api.example/a"
    );
    assert!(
        fixture_jwks(&f)
            .verify(&text(&tokens, "id_token"), &f.core.config.issuer, "app")
            .is_ok()
    );
    assert!(f.core.userinfo(&access).is_err());
    assert!(f.core.proxy_auth(&access, "app").is_err());
    assert!(f.core.proxy_auth(&access, "https://api.example/a").is_ok());
    let inspected = f
        .core
        .introspect(TokenRequest {
            client_id: Some("app".into()),
            client_secret: secret.clone(),
            token: Some(access.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(inspected["aud"], "https://api.example/a");
    let mut refresh = TokenRequest {
        grant_type: "refresh_token".into(),
        client_id: Some("app".into()),
        client_secret: secret.clone(),
        refresh_token: Some(text(&tokens, "refresh_token")),
        resource: Some("https://api.example/b".into()),
        ..Default::default()
    };
    assert_eq!(
        f.core.token(refresh.clone()).unwrap_err().code,
        "invalid_target"
    );
    refresh.resource = None;
    let refreshed = f.core.token(refresh).unwrap();
    assert!(
        fixture_jwks(&f)
            .verify(
                &text(&refreshed, "access_token"),
                &f.core.config.issuer,
                "https://api.example/a"
            )
            .is_ok()
    );
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(Default::default()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        f.core
            .introspect(TokenRequest {
                client_id: Some("app".into()),
                client_secret: secret,
                token: Some(access),
                ..Default::default()
            })
            .unwrap()["active"],
        false
    );
}

#[test]
fn logout_confirmation_session_checks_and_frontchannel_are_account_bound() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    frontchannel_logout_uri: Some("https://app.example.test/front-logout".into()),
                    post_logout_redirect_uris: vec!["https://app.example.test/signed-out".into()],
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let sid: String = f
        .core
        .store
        .get("session_tokens", &digest(&alice))
        .unwrap()
        .unwrap();
    let cookie = crypto::random_token("fixture_");
    f.core
        .store
        .write(|tx| {
            tx.put(
                "browser_sessions",
                &digest(&cookie),
                &json!({"session_id":sid,"expires_at":now()+600}),
            )
        })
        .unwrap();
    let verifier = crypto::random_token("");
    let url = url::Url::parse(
        &f.core
            .authorize(&alice, f.request("app", &verifier))
            .unwrap(),
    )
    .unwrap();
    let state = url
        .query_pairs()
        .find(|(k, _)| k == "session_state")
        .unwrap()
        .1
        .into_owned();
    assert_eq!(
        f.core
            .session_check("app", "http://localhost:7777", &state, Some(&cookie))
            .unwrap()["status"],
        "unchanged"
    );
    assert!(
        f.core
            .session_check("app", "https://evil.example.test", &state, Some(&cookie))
            .is_err()
    );
    let tokens = f.tokens("app", &alice, None);
    let request = riauth::logout::LogoutRequest {
        client_id: Some("app".into()),
        post_logout_redirect_uri: Some("https://app.example.test/signed-out".into()),
        state: Some("state".into()),
        ..Default::default()
    };
    let pending = f
        .core
        .end_session(request.clone(), Some(&cookie), None)
        .unwrap();
    assert_eq!(pending["interaction_required"], true);
    let code = text(&pending, "user_code");
    assert!(f.core.logout_request_details(&bob, &code).is_err());
    assert_eq!(
        f.core.logout_request_details(&alice, &code).unwrap()["client_id"],
        "app"
    );
    let id = pending["resume_uri"]
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap();
    let binding = pending["set_cookie"]
        .as_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    assert!(
        f.core
            .logout_request_resume(id, Some("wrong-browser"))
            .is_err()
    );
    f.core.logout_request_decide(&alice, &code, true).unwrap();
    let result = f.core.logout_request_resume(id, Some(binding)).unwrap();
    assert_eq!(result["logged_out"], true);
    let front = url::Url::parse(result["frontchannel_urls"][0].as_str().unwrap()).unwrap();
    assert!(
        front
            .query_pairs()
            .any(|(k, v)| k == "iss" && v == f.core.config.issuer)
    );
    assert!(front.query_pairs().any(|(k, _)| k == "sid"));
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert_eq!(
        f.core
            .session_check("app", "http://localhost:7777", &state, Some(&cookie))
            .unwrap()["status"],
        "changed"
    );
    let repeated = f
        .core
        .end_session(
            riauth::logout::LogoutRequest {
                id_token_hint: Some(text(&tokens, "id_token")),
                ..request
            },
            Some(&cookie),
            None,
        )
        .unwrap();
    assert_eq!(repeated["logged_out"], true);
}

const SIGNED_OUT: &str = "https://app.example.test/signed-out";
fn logout_app(f: &Fixture) {
    f.client("app", false);
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    post_logout_redirect_uris: vec![SIGNED_OUT.into()],
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
}
fn rp_logout(hint: Option<&Value>) -> riauth::logout::LogoutRequest {
    riauth::logout::LogoutRequest {
        id_token_hint: hint.map(|tokens| text(tokens, "id_token")),
        client_id: Some("app".into()),
        post_logout_redirect_uri: Some(SIGNED_OUT.into()),
        state: Some("state".into()),
    }
}
/// Signs `username` in through the browser password path; returns (sid, SSO cookie value).
fn browser_sign_in(f: &Fixture, username: &str) -> (String, String) {
    let staged = f
        .core
        .browser_password_login(username.into(), PASSWORD.into(), None, None)
        .unwrap();
    let attached = f
        .core
        .store
        .write(|tx| f.core.attach_browser_login(tx, &staged, None, None))
        .unwrap()
        .unwrap();
    (attached.session.id, sso_value(&attached.cookies))
}
fn sso_value(cookies: &[String]) -> String {
    cookies
        .iter()
        .find_map(|c| c.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap()
        .into()
}
fn clears_sso(cookies: &[String]) -> bool {
    cookies
        .iter()
        .any(|c| c.starts_with("riauth_sso=;") && c.contains("Max-Age=0"))
}
fn browser_tokens(f: &Fixture, sid: &str) -> Value {
    let session: Session = f.core.store.get("sessions", sid).unwrap().unwrap();
    let verifier = crypto::random_token("");
    let location = f
        .core
        .store
        .write(|tx| {
            f.core
                .authorize_session_proof(tx, session, f.request("app", &verifier), false, None)
        })
        .unwrap();
    let code = url::Url::parse(&location)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    f.core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(code),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap()
}
/// (interaction id, `riauth_logout` binding) of a pending confirmation.
fn logout_interaction(pending: &Value) -> (String, String) {
    assert_eq!(pending["interaction_required"], true);
    let id = pending["resume_uri"]
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap();
    let binding = pending["set_cookie"]
        .as_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    (id.into(), binding.into())
}
fn error_code<T>(result: riauth::error::Result<T>) -> &'static str {
    result.err().unwrap().code
}
fn logout_audited(f: &Fixture, username: &str, action: &str) -> bool {
    let actor: String = f.core.store.get("usernames", username).unwrap().unwrap();
    f.core
        .audit_events(&f.admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["actor"] == actor && e["action"] == action && e["target"] == "app")
}
fn noop_result(logged_out: bool) -> Value {
    json!({"logged_out": logged_out, "redirect_uri": format!("{SIGNED_OUT}?state=state"), "frontchannel_urls": []})
}

#[test]
fn browser_logout_decision_revokes_only_the_cookie_session() {
    let f = Fixture::new();
    logout_app(&f);
    let cli = f.user("alice");
    let bob = f.user("bob");
    let (sid, sso) = browser_sign_in(&f, "alice");
    let (other_sid, other_sso) = browser_sign_in(&f, "alice");
    let pending = f
        .core
        .end_session(rp_logout(None), Some(&sso), None)
        .unwrap();
    let (id, binding) = logout_interaction(&pending);
    assert_eq!(
        error_code(f.core.logout_state("missing", Some(&binding), Some(&sso))),
        "interaction_expired"
    );
    assert_eq!(
        error_code(f.core.logout_state(&id, Some("wrong-browser"), Some(&sso))),
        "invalid_token"
    );
    assert_eq!(
        error_code(f.core.logout_browser_decide(&id, None, Some(&sso), true)),
        "invalid_token"
    );
    let state = f
        .core
        .logout_state(&id, Some(&binding), Some(&sso))
        .unwrap();
    assert_eq!(state["kind"], "logout");
    assert_eq!(state["status"], "confirm");
    assert_eq!(state["expires_at"], pending["expires_at"]);
    assert_eq!(
        state["application"],
        json!({"client_id": "app", "name": "app", "host": "app.example.test"})
    );
    assert_eq!(state["account"]["username"], "alice");
    assert_eq!(state["session_ref"], riauth::signin::session_ref(&id, &sid));
    assert_eq!(
        state["logout"],
        json!({"targeted": true, "matches_browser": true, "ended": false})
    );
    assert_eq!(state["terminal"]["user_code"], pending["user_code"]);
    assert!(state["consent"].is_null() && state["continue"].is_null());

    // An hour-old sign-in still approves: unlike the terminal, the browser needs no fresh one.
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", &sid)?.unwrap();
            session.identity.auth_time -= 3600;
            tx.put("sessions", &sid, &session)
        })
        .unwrap();
    let reply = f
        .core
        .logout_browser_decide(&id, Some(&binding), Some(&sso), true)
        .unwrap();
    assert_eq!(
        reply.body,
        json!({"status": "done", "continue": pending["resume_uri"]})
    );
    assert!(clears_sso(&reply.cookies));
    assert!(
        f.core
            .store
            .get::<Value>("browser_sessions", &digest(&sso))
            .unwrap()
            .is_none()
    );
    let revoked = |sid: &str| {
        f.core
            .store
            .get::<Session>("sessions", sid)
            .unwrap()
            .unwrap()
            .revoked
    };
    assert!(revoked(&sid));
    assert!(!revoked(&other_sid));
    assert!(f.core.portal_apps(Some(&other_sso)).is_ok());
    assert!(f.core.me(&cli).is_ok() && f.core.me(&bob).is_ok());
    assert!(logout_audited(&f, "alice", "logout.confirmed"));
    let result = f.core.logout_request_resume(&id, Some(&binding)).unwrap();
    assert_eq!(result["logged_out"], true);
    assert_eq!(result["redirect_uri"], format!("{SIGNED_OUT}?state=state"));
    let state = f.core.logout_state(&id, Some(&binding), None).unwrap();
    assert_eq!(state["status"], "done");
    assert_eq!(state["continue"], pending["resume_uri"]);
    assert_eq!(
        error_code(
            f.core
                .logout_browser_decide(&id, Some(&binding), Some(&other_sso), true)
        ),
        "request_decided"
    );
    assert!(!revoked(&other_sid));

    // An untargeted request ends whichever session this browser holds when it decides.
    let pending = f.core.end_session(rp_logout(None), None, None).unwrap();
    let (id, binding) = logout_interaction(&pending);
    assert_eq!(
        f.core
            .logout_state(&id, Some(&binding), Some(&other_sso))
            .unwrap()["logout"],
        json!({"targeted": false, "matches_browser": true, "ended": false})
    );
    let reply = f
        .core
        .logout_browser_decide(&id, Some(&binding), Some(&other_sso), true)
        .unwrap();
    assert!(clears_sso(&reply.cookies));
    assert!(revoked(&other_sid));
    assert!(f.core.me(&cli).is_ok());
}

#[test]
fn browser_logout_decision_refuses_a_different_live_hinted_session() {
    let f = Fixture::new();
    logout_app(&f);
    let cli = f.user("alice");
    let tokens = f.tokens("app", &cli, None);
    let (sid, sso) = browser_sign_in(&f, "alice");
    // The hint names the terminal session, so this browser's own session is not the target.
    let pending = f
        .core
        .end_session(rp_logout(Some(&tokens)), Some(&sso), None)
        .unwrap();
    let (id, binding) = logout_interaction(&pending);
    let state = f
        .core
        .logout_state(&id, Some(&binding), Some(&sso))
        .unwrap();
    assert_eq!(
        state["logout"],
        json!({"targeted": true, "matches_browser": false, "ended": false})
    );
    assert_eq!(state["session_ref"], riauth::signin::session_ref(&id, &sid));
    for browser in [Some(sso.as_str()), None] {
        assert_eq!(
            error_code(
                f.core
                    .logout_browser_decide(&id, Some(&binding), browser, true)
            ),
            "session_mismatch"
        );
    }
    assert!(f.core.me(&cli).is_ok());
    assert!(f.core.portal_apps(Some(&sso)).is_ok());
    assert_eq!(
        f.core
            .logout_state(&id, Some(&binding), Some(&sso))
            .unwrap()["status"],
        "confirm"
    );
    let reply = f
        .core
        .logout_browser_decide(&id, Some(&binding), Some(&sso), false)
        .unwrap();
    assert_eq!(reply.body["status"], "done");
    assert!(reply.cookies.is_empty());
    assert_eq!(
        f.core.logout_request_resume(&id, Some(&binding)).unwrap(),
        noop_result(false)
    );
    assert!(logout_audited(&f, "alice", "logout.denied"));
    assert!(f.core.me(&cli).is_ok());
    assert!(f.core.portal_apps(Some(&sso)).is_ok());
}

#[test]
fn browser_logout_decision_for_an_already_ended_session_is_done() {
    let f = Fixture::new();
    logout_app(&f);
    let cli = f.user("alice");
    let tokens = f.tokens("app", &cli, None);
    let (sid, sso) = browser_sign_in(&f, "alice");
    let pending = f
        .core
        .end_session(rp_logout(Some(&tokens)), Some(&sso), None)
        .unwrap();
    let (id, binding) = logout_interaction(&pending);
    f.core.logout(&cli).unwrap();
    assert_eq!(
        f.core
            .logout_state(&id, Some(&binding), Some(&sso))
            .unwrap()["logout"],
        json!({"targeted": true, "matches_browser": false, "ended": true})
    );
    let reply = f
        .core
        .logout_browser_decide(&id, Some(&binding), Some(&sso), true)
        .unwrap();
    assert_eq!(reply.body["status"], "done");
    assert!(reply.cookies.is_empty());
    assert_eq!(
        f.core.logout_request_resume(&id, Some(&binding)).unwrap(),
        noop_result(true)
    );
    assert!(f.core.portal_apps(Some(&sso)).is_ok());

    // An account-wide revocation keeps the session row but it can never authenticate
    // again, so it counts as ended too.
    let pending = f
        .core
        .end_session(rp_logout(None), Some(&sso), None)
        .unwrap();
    let (id, binding) = logout_interaction(&pending);
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                revoke_sessions: true,
                ..Default::default()
            },
        )
        .unwrap();
    let state = f
        .core
        .logout_state(&id, Some(&binding), Some(&sso))
        .unwrap();
    assert_eq!(
        state["logout"],
        json!({"targeted": true, "matches_browser": false, "ended": true})
    );
    assert!(state["account"].is_null());
    let reply = f
        .core
        .logout_browser_decide(&id, Some(&binding), Some(&sso), true)
        .unwrap();
    assert!(reply.cookies.is_empty());
    assert_eq!(
        f.core.logout_request_resume(&id, Some(&binding)).unwrap(),
        noop_result(true)
    );
    let session: Session = f.core.store.get("sessions", &sid).unwrap().unwrap();
    assert!(!session.revoked);
}

#[test]
fn browser_logout_without_session_records_redirect_only() {
    let f = Fixture::new();
    logout_app(&f);
    let cli = f.user("alice");
    let pending = f.core.end_session(rp_logout(None), None, None).unwrap();
    let (id, binding) = logout_interaction(&pending);
    let state = f.core.logout_state(&id, Some(&binding), None).unwrap();
    assert_eq!(
        state["logout"],
        json!({"targeted": false, "matches_browser": false, "ended": false})
    );
    assert!(state["account"].is_null() && state["session_ref"].is_null());
    let reply = f
        .core
        .logout_browser_decide(&id, Some(&binding), Some("ri_sso_unknown"), true)
        .unwrap();
    assert_eq!(
        reply.body,
        json!({"status": "done", "continue": pending["resume_uri"]})
    );
    assert!(reply.cookies.is_empty());
    assert_eq!(
        f.core.logout_request_resume(&id, Some(&binding)).unwrap(),
        noop_result(false)
    );
    assert!(f.core.me(&cli).is_ok());
    assert!(!logout_audited(&f, "alice", "logout.confirmed"));
}

#[test]
fn direct_logout_reports_clear_sso_and_drops_mapping() {
    let f = Fixture::new();
    logout_app(&f);
    f.user("alice");
    let (sid, sso) = browser_sign_in(&f, "alice");
    let (other_sid, other_sso) = browser_sign_in(&f, "alice");
    let mapping = |sso: &str| {
        f.core
            .store
            .get::<Value>("browser_sessions", &digest(sso))
            .unwrap()
    };
    let tokens = browser_tokens(&f, &sid);
    let result = f
        .core
        .end_session(rp_logout(Some(&tokens)), Some(&sso), None)
        .unwrap();
    assert_eq!(result["logged_out"], true);
    assert_eq!(result["clear_sso"], true);
    assert!(mapping(&sso).is_none());
    assert!(mapping(&other_sso).is_some());
    assert!(f.core.portal_apps(Some(&other_sso)).is_ok());

    // A bearer logout leaves a browser mapped to another session alone.
    let cli = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let tokens = f.tokens("app", &cli, None);
    let result = f
        .core
        .end_session(rp_logout(Some(&tokens)), Some(&other_sso), Some(&cli))
        .unwrap();
    assert_eq!(result["logged_out"], true);
    assert_eq!(result["clear_sso"], false);
    assert!(mapping(&other_sso).is_some());

    // The terminal already ended a session this browser shares: the RP logout clears it.
    let cli = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let cli_sid = text(&f.core.me(&cli).unwrap(), "session_id");
    let shared = sso_value(
        &f.core
            .store
            .write(|tx| f.core.point_browser(tx, None, &cli_sid, "test"))
            .unwrap(),
    );
    let tokens = f.tokens("app", &cli, None);
    f.core.logout(&cli).unwrap();
    let result = f
        .core
        .end_session(rp_logout(Some(&tokens)), Some(&shared), None)
        .unwrap();
    assert_eq!(result["logged_out"], true);
    assert_eq!(result["clear_sso"], true);
    assert!(mapping(&shared).is_none());
    assert!(mapping(&other_sso).is_some());
    let other: Session = f.core.store.get("sessions", &other_sid).unwrap().unwrap();
    assert!(!other.revoked);
}

#[test]
fn logout_request_details_show_requester() {
    let f = Fixture::new();
    logout_app(&f);
    let alice = f.user("alice");
    let context = riauth::context::RequestContext {
        client_ip: Some("203.0.113.9".parse().unwrap()),
        user_agent: Some("ExampleBrowser/1.0".into()),
        ..Default::default()
    };
    let pending = riauth::context::scope(Some(context), || {
        f.core.end_session(rp_logout(None), None, None)
    })
    .unwrap();
    let details = f
        .core
        .logout_request_details(&alice, &text(&pending, "user_code"))
        .unwrap();
    let from = &details["requested_from"];
    assert_eq!(from["ip"], "203.0.113.9");
    assert_eq!(from["user_agent"], "ExampleBrowser/1.0");
    assert!(from["at"].as_u64().unwrap().abs_diff(now()) < 5);
    let plain = f.core.end_session(rp_logout(None), None, None).unwrap();
    assert!(
        f.core
            .logout_request_details(&alice, &text(&plain, "user_code"))
            .unwrap()["requested_from"]
            .is_null()
    );
}
