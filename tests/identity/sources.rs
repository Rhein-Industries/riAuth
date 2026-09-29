use super::*;

#[tokio::test]
async fn upstream_oidc_pkce_pinned_keys_claim_validation_and_terminal_completion() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let upstream = Upstream::new(&f).await;
    let start = upstream.start(&f, None);
    assert_eq!(
        upstream.finish(&f, &start, false).unwrap()["status"],
        "pending"
    );
    let callback = upstream.callback(&f, &start, "subject-1", json!({})).await;
    assert_eq!(callback["completed"], true);
    assert!(!callback.to_string().contains("ri_session_"));
    let review = upstream.finish(&f, &start, false).unwrap();
    assert_eq!(review["status"], "review");
    assert_eq!(review["mfa"], true);
    assert!(review["local_user"].is_null()); // An identical email does not link local Alice.
    let result = upstream.finish(&f, &start, true).unwrap();
    assert_ne!(
        result["user"]["id"],
        f.core.me(&alice).unwrap()["user"]["id"]
    );
    assert!(upstream.finish(&f, &start, true).is_err());
    let session = text(&result, "session_token");
    let tokens = f.tokens("app", &session, None);
    let claims = fixture_jwks(&f)
        .verify(&text(&tokens, "id_token"), &f.core.config.issuer, "app")
        .unwrap();
    assert_eq!(claims["amr"], json!(["federated", "mfa"]));
    let next = upstream.start(&f, None);
    upstream.callback(&f, &next, "subject-1", json!({})).await;
    assert_eq!(
        upstream.finish(&f, &next, true).unwrap()["user"]["id"],
        result["user"]["id"]
    );
    for invalid in [
        json!({"nonce":"wrong"}),
        json!({"iss":"https://foreign.example"}),
        json!({"azp":"other-client"}),
        json!({"aud":["upstream-client","foreign"]}),
        json!({"at_hash":"wrong"}),
        json!({"auth_time":now()-600}),
        json!({"exp":now()-1}),
    ] {
        let request = upstream.start(&f, None);
        assert_eq!(
            upstream.callback(&f, &request, "subject-1", invalid).await["completed"],
            false
        );
        assert!(upstream.finish(&f, &request, true).is_err());
    }
    let pending = upstream.start(&f, None);
    upstream
        .callback(&f, &pending, "subject-1", json!({}))
        .await;
    let mut changed = upstream.source.clone();
    changed.enabled = false;
    f.core
        .source_put(
            &f.admin,
            riauth::source::SourceInput {
                source: changed,
                client_secret: None,
            },
        )
        .unwrap();
    assert!(upstream.finish(&f, &pending, true).is_err());
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(f.core.me(&alice).is_ok());
}

#[tokio::test]
async fn source_account_linking_requires_fresh_local_identity_and_unlink_revokes_only_its_sessions()
{
    let f = Fixture::new();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let mut upstream = Upstream::new(&f).await;
    upstream.source.auto_provision = false;
    f.core
        .source_put(
            &f.admin,
            riauth::source::SourceInput {
                source: upstream.source.clone(),
                client_secret: None,
            },
        )
        .unwrap();
    assert!(
        f.core
            .source_start(
                "upstream",
                riauth::source::Start {
                    link: true,
                    authentication_transaction: None
                },
                Some(&f.admin)
            )
            .is_err()
    );
    let request = upstream.start(&f, None);
    upstream
        .callback(&f, &request, "subject-1", json!({}))
        .await;
    assert!(upstream.finish(&f, &request, true).is_err());
    let linked = upstream.start(&f, Some(&alice));
    upstream.callback(&f, &linked, "subject-1", json!({})).await;
    let result = upstream.finish(&f, &linked, true).unwrap();
    assert_eq!(result["user"]["username"], "alice");
    let collision = upstream.start(&f, Some(&bob));
    upstream
        .callback(&f, &collision, "subject-1", json!({}))
        .await;
    assert!(upstream.finish(&f, &collision, true).is_err());
    let links = f.core.source_links(&alice).unwrap();
    let link_id = text(&links[0], "id");
    assert!(f.core.source_unlink(&bob, &link_id).is_err());
    let remote_session = text(&result, "session_token");
    assert!(f.core.source_unlink(&remote_session, &link_id).is_err());
    f.core.source_unlink(&alice, &link_id).unwrap();
    assert!(f.core.me(&remote_session).is_err());
    assert!(f.core.me(&alice).is_ok());
}

#[tokio::test]
async fn desired_state_and_verified_login_share_source_link_ownership_without_reassignment() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let link_count = |token: &str| {
        f.core
            .source_links(token)
            .unwrap()
            .as_array()
            .unwrap()
            .len()
    };
    let mut upstream = Upstream::new(&f).await;
    upstream.source.auto_provision = false;
    f.core
        .source_put(
            &f.admin,
            riauth::source::SourceInput {
                source: upstream.source.clone(),
                client_secret: None,
            },
        )
        .unwrap();

    let spec = json!({"source":"upstream","username":"alice","subject":"planned"});
    let manifest: riauth::state::Manifest = serde_json::from_value(json!({
        "api_version":"riauth/v1", "source_links":[spec]
    }))
    .unwrap();
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    let request = || riauth::state::ApplyRequest {
        plan: plan.clone(),
        secrets: Default::default(),
        run_id: None,
    };
    let applied = f
        .core
        .apply_state_confirmed(&f.admin, request(), Some(&plan.plan_id))
        .unwrap();
    assert_eq!(f.core.apply_state(&f.admin, request()).unwrap(), applied);
    assert_eq!(link_count(&alice), 1);

    // A real verified upstream response uses the planned link without moving
    // it or creating another row. Completion is still one-use.
    let existing = upstream.start(&f, None);
    upstream.callback(&f, &existing, "planned", json!({})).await;
    assert_eq!(
        upstream.finish(&f, &existing, false).unwrap()["status"],
        "review"
    );
    assert_eq!(
        upstream.finish(&f, &existing, true).unwrap()["user"]["id"],
        f.core.me(&alice).unwrap()["user"]["id"]
    );
    assert!(upstream.finish(&f, &existing, true).is_err());
    assert_eq!(link_count(&alice), 1);

    let collision = upstream.start(&f, Some(&bob));
    upstream
        .callback(&f, &collision, "planned", json!({}))
        .await;
    assert!(upstream.finish(&f, &collision, true).is_err());
    assert_eq!(link_count(&bob), 0);

    let new_link = upstream.start(&f, Some(&alice));
    upstream
        .callback(&f, &new_link, "consented", json!({}))
        .await;
    assert_eq!(
        upstream.finish(&f, &new_link, false).unwrap()["status"],
        "review"
    );
    assert_eq!(link_count(&alice), 1);
    upstream.finish(&f, &new_link, true).unwrap();
    assert_eq!(link_count(&alice), 2);
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    let count = |action: &str| {
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action && event["target"] == "upstream")
            .count()
    };
    assert_eq!(count("source.login"), 1);
    assert_eq!(count("source.link"), 1);
    let action_count = |action: &str| {
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action)
            .count()
    };
    assert_eq!(action_count("source_link.reconcile"), 1);
    assert_eq!(action_count("state.apply"), 1);
}

#[tokio::test]
async fn source_manifests_are_redacted_atomic_idempotent_and_permission_checked() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f).await;
    let mut profile = upstream.source.clone();
    profile.id = "declarative".into();
    profile.auto_provision = false;
    let manifest = riauth::state::Manifest {
        api_version: "riauth/v1".into(),
        sources: vec![riauth::source::SourceSpec {
            source: profile.clone(),
            secret_ref: Some("env:UPSTREAM_SECRET".into()),
            secret_version: Some("v1".into()),
        }],
        ..Default::default()
    };
    let plan = f.core.plan_state(&f.admin, manifest.clone()).unwrap();
    let encoded = serde_json::to_string(&plan).unwrap();
    assert!(!encoded.contains("source-client-secret"));
    assert!(
        f.core
            .apply_state(
                &f.admin,
                riauth::state::ApplyRequest {
                    plan: plan.clone(),
                    secrets: Default::default(),
                    run_id: None
                }
            )
            .is_err()
    );
    assert!(
        f.core
            .store
            .get::<Value>("sources", "declarative")
            .unwrap()
            .is_none()
    );
    f.core
        .apply_state(
            &f.admin,
            riauth::state::ApplyRequest {
                plan: plan.clone(),
                secrets: [(
                    "env:UPSTREAM_SECRET".into(),
                    "supplied-upstream-credential".into(),
                )]
                .into(),
                run_id: None,
            },
        )
        .unwrap();
    assert!(
        f.core
            .plan_state(&f.admin, manifest)
            .unwrap()
            .changes
            .is_empty()
    );
    assert!(
        !f.core
            .export_state(&f.admin)
            .unwrap()
            .to_string()
            .contains("supplied-upstream-credential")
    );
    assert!(
        !f.core
            .audit_events(&f.admin, 100)
            .unwrap()
            .to_string()
            .contains("supplied-upstream-credential")
    );
    let agent = f
        .core
        .create_agent(
            &f.admin,
            riauth::agent::NewAgent {
                id: "source-agent".into(),
                permissions: vec![
                    riauth::agent::Permission {
                        action: "source.write".into(),
                        resource: "source/declarative".into(),
                    },
                    riauth::agent::Permission {
                        action: "source.read".into(),
                        resource: "source/declarative".into(),
                    },
                ],
                ttl: 600,
                parent: None,
            },
        )
        .unwrap();
    let token = text(&agent["credential"], "token");
    assert_eq!(
        f.core
            .source_list(&token)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    profile.allow_admin_login = true;
    assert!(
        f.core
            .source_put(
                &token,
                riauth::source::SourceInput {
                    source: profile,
                    client_secret: None
                }
            )
            .is_err()
    );
}

#[tokio::test]
async fn oauth_only_sources_use_pinned_userinfo_and_do_not_invent_oidc_assurance_or_link_by_email()
{
    use axum::{
        Form, Json, Router,
        extract::State,
        http::{HeaderMap, StatusCode},
        routing::{get, post},
    };
    #[derive(Clone)]
    struct OAuthMock {
        challenge: std::sync::Arc<std::sync::Mutex<String>>,
        identity: std::sync::Arc<std::sync::Mutex<Value>>,
    }
    async fn token(
        State(state): State<OAuthMock>,
        Form(body): Form<std::collections::HashMap<String, String>>,
    ) -> Json<Value> {
        assert_eq!(body["client_id"], "cli-source");
        assert_eq!(body["grant_type"], "authorization_code");
        assert_eq!(
            digest(&body["code_verifier"]),
            *state.challenge.lock().unwrap()
        );
        Json(json!({"access_token":"upstream-test-access","token_type":"Bearer"}))
    }
    async fn userinfo(
        State(state): State<OAuthMock>,
        headers: HeaderMap,
    ) -> (StatusCode, Json<Value>) {
        assert_eq!(headers["authorization"], "Bearer upstream-test-access");
        (StatusCode::OK, Json(state.identity.lock().unwrap().clone()))
    }
    let f = Fixture::new();
    let local = f.user("same-email");
    let _ = local;
    let state = OAuthMock {
        challenge: Default::default(),
        identity: std::sync::Arc::new(std::sync::Mutex::new(
            json!({"id":456,"name":"OAuth User","email":"same-email@example.test","verified":true,"acr":"urn:riauth:acr:mfa"}),
        )),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/token", post(token))
        .route("/user", get(userinfo))
        .with_state(state.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let source = riauth::source::Source {
        saml: None,
        id: "oauth".into(),
        name: "OAuth-only source".into(),
        issuer: base.clone(),
        authorization_endpoint: format!("{base}/authorize"),
        token_endpoint: format!("{base}/token"),
        client_id: "cli-source".into(),
        token_endpoint_auth_method: riauth::jose::ClientAuthMethod::None,
        jwks: Default::default(),
        scopes: strings(&["profile"]),
        enabled: true,
        auto_provision: true,
        groups: Default::default(),
        trusted_mfa_acr: Default::default(),
        allow_admin_login: false,
        oauth_profile: Some(riauth::source::OAuthProfile {
            userinfo_endpoint: format!("{base}/user"),
            subject_pointer: "/id".into(),
            name_pointer: Some("/name".into()),
            email_pointer: Some("/email".into()),
            email_verified_pointer: None,
        }),
    };
    f.core
        .source_put(
            &f.admin,
            riauth::source::SourceInput {
                source: source.clone(),
                client_secret: None,
            },
        )
        .unwrap();
    assert!(
        f.core
            .source_start(
                "oauth",
                riauth::source::Start {
                    link: false,
                    authentication_transaction: Some("fresh-login".into())
                },
                None
            )
            .is_err()
    );
    let start = f
        .core
        .source_start(
            "oauth",
            riauth::source::Start {
                link: false,
                authentication_transaction: None,
            },
            None,
        )
        .unwrap();
    let authorize = url::Url::parse(start["authorization_url"].as_str().unwrap()).unwrap();
    let query: std::collections::HashMap<_, _> = authorize.query_pairs().into_owned().collect();
    assert!(!query.contains_key("nonce"));
    assert!(!query.contains_key("max_age"));
    *state.challenge.lock().unwrap() = query["code_challenge"].clone();
    let callback = vec![
        ("state".into(), query["state"].clone()),
        ("code".into(), "fixture-code".into()),
    ];
    assert_eq!(
        f.core
            .source_callback("oauth", callback.clone(), None)
            .await
            .unwrap()["completed"],
        true
    );
    assert!(
        f.core
            .source_callback("oauth", callback, None)
            .await
            .is_err()
    );
    let login = f
        .core
        .source_finish(riauth::source::Finish {
            credential: text(&start["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();
    assert_ne!(login["user"]["username"], "same-email");
    assert_eq!(login["user"]["email_verified"], false);
    let session = f
        .core
        .store
        .list::<Session>("sessions")
        .unwrap()
        .into_iter()
        .map(|(_, s)| s)
        .find(|s| s.token_hash == digest(&text(&login, "session_token")))
        .unwrap();
    assert_eq!(session.identity.auth_time, 0);
    assert!(!session.identity.mfa);
    let mut changed = source.clone();
    changed.oauth_profile.as_mut().unwrap().subject_pointer = "/name".into();
    assert!(
        f.core
            .source_put(
                &f.admin,
                riauth::source::SourceInput {
                    source: changed,
                    client_secret: None
                }
            )
            .is_err()
    );
    let mut disabled = source;
    disabled.enabled = false;
    f.core
        .source_put(
            &f.admin,
            riauth::source::SourceInput {
                source: disabled,
                client_secret: None,
            },
        )
        .unwrap();
    assert!(f.core.me(&text(&login, "session_token")).is_err());
    server.abort();
}

/// The value of cookie `name` set by a browser reply.
fn reply_cookie(reply: &riauth::browser::BrowserReply, name: &str) -> String {
    let prefix = format!("{name}=");
    reply
        .cookies
        .iter()
        .find_map(|cookie| {
            cookie
                .split(';')
                .next()
                .and_then(|pair| pair.strip_prefix(&prefix))
        })
        .expect(name)
        .to_owned()
}

#[cfg(feature = "test-support")]
#[tokio::test]
async fn browser_link_finish_needs_the_original_fresh_local_session_and_rolls_back_whole() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let upstream = Upstream::new(&f).await;
    let browser = |f: &Fixture| {
        reply_cookie(
            &f.core
                .portal_password(None, "alice".into(), PASSWORD.into(), None, false)
                .unwrap(),
            "riauth_sso",
        )
    };
    let original = browser(&f);
    let page = f.core.portal_source_links(Some(&original)).unwrap();
    let binding = riauth::portal::self_service::Binding {
        expected_user_id: text(&page["user"], "id"),
        expected_session_id: text(&page, "current_session_id"),
    };
    let started = f
        .core
        .portal_source_start(Some(&original), "upstream", Some(&binding))
        .unwrap();
    let cookie = reply_cookie(&started, "riauth_source");
    let credential = cookie.split_once('.').unwrap().0.to_owned();
    upstream
        .callback_with(
            &f,
            &started.body,
            "subject-browser",
            json!({}),
            Some(&cookie),
        )
        .await;
    let finish = |sso: &str| {
        f.core
            .portal_source_finish(Some(&credential), Some(sso), true, None)
    };
    let links = || {
        f.core
            .source_links(&alice)
            .unwrap()
            .as_array()
            .unwrap()
            .len()
    };
    let sessions = || f.core.store.list::<Session>("sessions").unwrap().len();
    let age = |seconds: u64| {
        f.core
            .store
            .write(|tx| {
                let mut session: Session =
                    tx.get("sessions", &binding.expected_session_id)?.unwrap();
                session.identity.auth_time = now() - seconds;
                tx.put("sessions", &binding.expected_session_id, &session)
            })
            .unwrap()
    };

    // Another fresh browser session of the same account did not start this link.
    let other = browser(&f);
    assert_eq!(finish(&other).err().unwrap().code, "session_changed");
    // The original session must still be a fresh local sign-in when the link is written.
    age(riauth::signin::FRESH_SECONDS + 5);
    assert_eq!(
        finish(&original).err().unwrap().code,
        "reauthentication_required"
    );
    assert_eq!(links(), 0);

    // A failure after the proof is spent rolls back the link and the new session with it.
    age(0);
    let before = sessions();
    assert!(riauth::portal::sources::with_failed_delivery(|| finish(&original)).is_err());
    assert_eq!((links(), sessions()), (0, before));
    assert_eq!(
        f.core.portal_source_review(Some(&credential)).unwrap()["status"],
        "review"
    );

    // The same login then finishes once, from the original session, which stays signed in.
    assert_eq!(finish(&original).unwrap().body["linked"], true);
    assert_eq!(links(), 1);
    assert!(f.core.portal_source_links(Some(&original)).is_ok());
    assert_eq!(
        f.core
            .portal_source_review(Some(&credential))
            .unwrap_err()
            .code,
        "source_login_expired"
    );
}

#[cfg(feature = "test-support")]
#[tokio::test]
async fn browser_source_sign_in_delivery_failure_rolls_back_and_retries_once() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f).await;
    let started = f.core.portal_source_start(None, "upstream", None).unwrap();
    let cookie = reply_cookie(&started, "riauth_source");
    let credential = cookie.split_once('.').unwrap().0.to_owned();
    upstream
        .callback_with(
            &f,
            &started.body,
            "new-browser-user",
            json!({}),
            Some(&cookie),
        )
        .await;
    assert_eq!(
        f.core.portal_source_review(Some(&credential)).unwrap()["status"],
        "review"
    );
    let before = f.snapshot().unwrap();
    let users = f.core.store.list::<User>("users").unwrap().len();
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let bearer_tokens = f.core.store.list::<String>("session_tokens").unwrap().len();
    let finish = || {
        f.core
            .portal_source_finish(Some(&credential), None, true, None)
    };

    assert_eq!(
        riauth::portal::sources::with_failed_delivery(finish)
            .err()
            .unwrap()
            .code,
        "server_error"
    );
    // The proof, new identity, link, session and browser pointer share one transaction.
    f.assert_snapshot(&before);
    assert_eq!(
        f.core.portal_source_review(Some(&credential)).unwrap()["status"],
        "review"
    );

    let finished = finish().unwrap();
    assert_eq!(finished.body["signed_in"], true);
    let sso = reply_cookie(&finished, "riauth_sso");
    let page = f.core.portal_source_links(Some(&sso)).unwrap();
    assert_eq!(page["user"]["id"], finished.body["user"]["id"]);
    assert_eq!(page["links"].as_array().unwrap().len(), 1);
    assert_eq!(f.core.store.list::<User>("users").unwrap().len(), users + 1);
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions + 1
    );
    assert_eq!(
        f.core.store.list::<String>("session_tokens").unwrap().len(),
        bearer_tokens
    );
    let committed = f.snapshot().unwrap();
    assert_eq!(finish().err().unwrap().code, "source_login_expired");
    f.assert_snapshot(&committed);
}

#[cfg(feature = "test-support")]
#[tokio::test]
async fn browser_and_bearer_source_links_require_an_enrolled_factor_session() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let upstream = Upstream::new(&f).await;
    let browser = |otp| {
        reply_cookie(
            &f.core
                .portal_password(None, "alice".into(), PASSWORD.into(), otp, false)
                .unwrap(),
            "riauth_sso",
        )
    };
    let binding = |sso: &str| {
        let page = f.core.portal_source_links(Some(sso)).unwrap();
        riauth::portal::self_service::Binding {
            expected_user_id: text(&page["user"], "id"),
            expected_session_id: text(&page, "current_session_id"),
        }
    };
    let password_only = browser(None);
    let bound = binding(&password_only);
    let pending = f
        .core
        .portal_source_start(Some(&password_only), "upstream", Some(&bound))
        .unwrap();
    let cookie = reply_cookie(&pending, "riauth_source");
    let credential = cookie.split_once('.').unwrap().0.to_owned();
    upstream
        .callback_with(&f, &pending.body, "pending", json!({}), Some(&cookie))
        .await;

    // A factor imported while a link is pending must be checked again at finish.
    let secret = crypto::totp_secret();
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &bound.expected_user_id)?.unwrap();
            user.totp_secret = Some(secret.clone());
            tx.put("users", &user.id, &user)
        })
        .unwrap();
    assert_eq!(
        f.core.portal_source_links(Some(&password_only)).unwrap()["can_change"],
        false
    );
    assert_eq!(
        f.core
            .portal_source_start(Some(&password_only), "upstream", Some(&bound))
            .err()
            .unwrap()
            .code,
        "mfa_required"
    );
    assert_eq!(
        f.core
            .portal_source_finish(Some(&credential), Some(&password_only), true, None)
            .err()
            .unwrap()
            .code,
        "mfa_required"
    );
    assert_eq!(
        f.core.portal_source_review(Some(&credential)).unwrap()["status"],
        "review"
    );
    let link_start = || riauth::source::Start {
        link: true,
        authentication_transaction: None,
    };
    assert_eq!(
        f.core
            .source_start("upstream", link_start(), Some(&alice))
            .unwrap_err()
            .code,
        "mfa_required"
    );

    let code = crypto::totp(&secret, "alice")
        .unwrap()
        .generate(now())
        .to_string();
    let verified = browser(Some(code));
    let verified_bound = binding(&verified);
    let started = f
        .core
        .portal_source_start(Some(&verified), "upstream", Some(&verified_bound))
        .unwrap();
    let verified_cookie = reply_cookie(&started, "riauth_source");
    let verified_credential = verified_cookie.split_once('.').unwrap().0.to_owned();
    upstream
        .callback_with(
            &f,
            &started.body,
            "verified",
            json!({}),
            Some(&verified_cookie),
        )
        .await;
    assert_eq!(
        f.core
            .portal_source_finish(Some(&verified_credential), Some(&verified), true, None)
            .unwrap()
            .body["linked"],
        true
    );
    let link_id = text(&f.core.source_links(&alice).unwrap()[0], "id");
    assert_eq!(
        f.core
            .portal_source_unlink(Some(&password_only), &bound, &link_id)
            .unwrap_err()
            .code,
        "mfa_required"
    );
    assert_eq!(
        f.core.source_unlink(&alice, &link_id).unwrap_err().code,
        "mfa_required"
    );
    assert_eq!(
        f.core
            .portal_source_unlink(Some(&verified), &verified_bound, &link_id)
            .unwrap()["unlinked"],
        true
    );

    let p = Fixture::new();
    let bootstrap = p.user("alice");
    let upstream = Upstream::new(&p).await;
    let (mut authenticator, _) = super::factors_tests::enroll_passkey(&p, &bootstrap);
    let password_bearer = text(
        &p.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let password_browser = reply_cookie(
        &p.core
            .portal_password(None, "alice".into(), PASSWORD.into(), None, false)
            .unwrap(),
        "riauth_sso",
    );
    let page = p.core.portal_source_links(Some(&password_browser)).unwrap();
    let bound = riauth::portal::self_service::Binding {
        expected_user_id: text(&page["user"], "id"),
        expected_session_id: text(&page, "current_session_id"),
    };
    assert_eq!(page["can_change"], false);
    assert_eq!(
        p.core
            .portal_source_start(Some(&password_browser), "upstream", Some(&bound))
            .err()
            .unwrap()
            .code,
        "mfa_required"
    );
    assert_eq!(
        p.core
            .source_start("upstream", link_start(), Some(&password_bearer))
            .unwrap_err()
            .code,
        "mfa_required"
    );
    let passkey = super::factors_tests::passkey_session(&p, "alice", &mut authenticator);
    let linked = upstream.start(&p, Some(&passkey));
    upstream.callback(&p, &linked, "passkey", json!({})).await;
    assert_eq!(
        upstream.finish(&p, &linked, true).unwrap()["user"]["username"],
        "alice"
    );
    let link_id = text(&p.core.source_links(&passkey).unwrap()[0], "id");
    assert_eq!(
        p.core
            .portal_source_unlink(Some(&password_browser), &bound, &link_id)
            .unwrap_err()
            .code,
        "mfa_required"
    );
    assert_eq!(
        p.core
            .source_unlink(&password_bearer, &link_id)
            .unwrap_err()
            .code,
        "mfa_required"
    );
    assert_eq!(
        p.core.source_unlink(&passkey, &link_id).unwrap()["unlinked"],
        true
    );
}

#[tokio::test]
async fn browser_source_callback_is_redeemed_only_by_the_starting_browser() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f).await;
    let accounts = || {
        (
            f.core.store.list::<User>("users").unwrap().len(),
            f.core.store.list::<Session>("sessions").unwrap().len(),
            f.core.store.list::<Value>("source_links").unwrap().len(),
        )
    };
    let before = accounts();
    let login_row = |body: &Value| {
        let url = url::Url::parse(body["authorization_url"].as_str().unwrap()).unwrap();
        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .unwrap()
            .1
            .to_string();
        f.core
            .store
            .get::<Value>("source_logins", &digest(&state))
            .unwrap()
            .unwrap()
    };
    let hides = |error: &riauth::error::Error, secret: &str| {
        let shown = error.to_string();
        assert!(!shown.contains(secret), "{shown}");
    };

    let started = f.core.portal_source_start(None, "upstream", None).unwrap();
    let cookie = reply_cookie(&started, "riauth_source");
    let credential = cookie.split_once('.').unwrap().0.to_owned();
    let (mismatch, code) = upstream
        .redeem(&f, &started.body, "phished", json!({}), None)
        .await;
    let mismatch = mismatch.unwrap_err();
    assert_eq!(mismatch.code, "source_browser_mismatch");
    assert_eq!(mismatch.status.as_u16(), 403);
    hides(&mismatch, &code);
    hides(&mismatch, &cookie);
    assert!(upstream.retains(&code));
    let burned = login_row(&started.body);
    assert_eq!(burned["claimed"], true);
    assert_eq!(burned["failed"], true);
    assert!(burned["result"].is_null());
    assert_eq!(burned["browser_binding"], digest(&cookie));
    assert!(
        f.core
            .store
            .list::<Value>("audit")
            .unwrap()
            .iter()
            .any(|(_, event)| {
                event["action"] == "source.login_failed"
                    && event["actor"] == "upstream"
                    && event["target"] == "upstream"
            })
    );
    assert_eq!(accounts(), before);
    assert_eq!(
        f.core
            .portal_source_review(Some(&credential))
            .unwrap_err()
            .code,
        "source_login_expired"
    );

    let replay = upstream
        .complete(&f, &started.body, &code, Some(&cookie))
        .await
        .unwrap_err();
    assert!(upstream.retains(&code));
    hides(&replay, &code);
    hides(&replay, &cookie);
    assert_eq!(accounts(), before);

    let other = f.core.portal_source_start(None, "upstream", None).unwrap();
    let other_cookie = reply_cookie(&other, "riauth_source");
    let foreign = upstream
        .complete(&f, &started.body, &code, Some(&other_cookie))
        .await
        .unwrap_err();
    assert!(upstream.retains(&code));
    hides(&foreign, &code);
    hides(&foreign, &cookie);
    hides(&foreign, &other_cookie);

    // A cookie from the login above is a different browser for this new login.
    let wrong = f.core.portal_source_start(None, "upstream", None).unwrap();
    let wrong_cookie = reply_cookie(&wrong, "riauth_source");
    let wrong_credential = wrong_cookie.split_once('.').unwrap().0.to_owned();
    let (rejected, wrong_code) = upstream
        .redeem(
            &f,
            &wrong.body,
            "other-browser",
            json!({}),
            Some(&other_cookie),
        )
        .await;
    let rejected = rejected.unwrap_err();
    assert_eq!(rejected.code, "source_browser_mismatch");
    assert_eq!(rejected.status.as_u16(), 403);
    assert!(upstream.retains(&wrong_code));
    hides(&rejected, &wrong_code);
    hides(&rejected, &other_cookie);
    assert_eq!(login_row(&wrong.body)["failed"], true);
    assert!(login_row(&wrong.body)["result"].is_null());
    assert_eq!(
        f.core
            .portal_source_review(Some(&wrong_credential))
            .unwrap_err()
            .code,
        "source_login_expired"
    );
    assert!(
        upstream
            .complete(&f, &wrong.body, &wrong_code, Some(&wrong_cookie))
            .await
            .is_err()
    );
    assert!(upstream.retains(&wrong_code));

    let bulky = f.core.portal_source_start(None, "upstream", None).unwrap();
    let bulky_cookie = "x".repeat(257);
    let (oversized, bulky_code) = upstream
        .redeem(&f, &bulky.body, "oversized", json!({}), Some(&bulky_cookie))
        .await;
    let oversized = oversized.unwrap_err();
    assert_eq!(oversized.code, "source_browser_mismatch");
    assert_eq!(oversized.status.as_u16(), 403);
    assert!(upstream.retains(&bulky_code));
    hides(&oversized, &bulky_code);
    hides(&oversized, &bulky_cookie);

    let (completed, other_code) = upstream
        .redeem(
            &f,
            &other.body,
            "same-browser",
            json!({}),
            Some(&other_cookie),
        )
        .await;
    assert_eq!(completed.unwrap()["completed"], true);
    assert!(!upstream.retains(&other_code));
    let stored = login_row(&other.body);
    assert_eq!(stored["failed"], false);
    assert_eq!(stored["result"]["subject"], "same-browser");
    assert_eq!(stored["browser_binding"], digest(&other_cookie));
    let other_credential = other_cookie.split_once('.').unwrap().0.to_owned();
    assert_eq!(
        f.core
            .portal_source_review(Some(&other_credential))
            .unwrap()["status"],
        "review"
    );
    assert_eq!(accounts(), before);
    assert!(
        upstream
            .complete(&f, &other.body, &other_code, Some(&other_cookie))
            .await
            .is_err()
    );
    assert!(!upstream.retains(&other_code));
}
