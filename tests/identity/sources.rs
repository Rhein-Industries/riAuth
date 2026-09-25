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
            .source_callback("oauth", callback.clone())
            .await
            .unwrap()["completed"],
        true
    );
    assert!(f.core.source_callback("oauth", callback).await.is_err());
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
