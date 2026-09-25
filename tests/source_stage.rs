//! Embedded source stages suspend and resume the original authorization.
#[path = "common/mod.rs"]
mod common;

use common::{Fixture, strings, text};
use riauth::{
    crypto::{self, digest, now},
    model::*,
    oidc::{Authorization, TokenRequest},
    source::SourceInput,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

type UpstreamCodes = Arc<Mutex<std::collections::HashMap<String, (String, Value)>>>;

struct Upstream {
    source: riauth::source::Source,
    key: crypto::SigningKey,
    codes: UpstreamCodes,
    server: tokio::task::JoinHandle<()>,
}
impl Drop for Upstream {
    fn drop(&mut self) {
        self.server.abort();
    }
}
impl Upstream {
    async fn new(f: &Fixture, auto_provision: bool) -> Self {
        use axum::{Form, Json, Router, routing::post};
        use base64::{Engine, engine::general_purpose::STANDARD};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let issuer = format!("http://{}", listener.local_addr().unwrap());
        let codes: UpstreamCodes = Default::default();
        let records = codes.clone();
        let app = Router::new().route(
            "/token",
            post(
                move |headers: axum::http::HeaderMap,
                      Form(form): Form<std::collections::HashMap<String, String>>| {
                    let records = records.clone();
                    async move {
                        assert_eq!(
                            headers["authorization"],
                            format!(
                                "Basic {}",
                                STANDARD.encode("upstream-client:source-client-secret")
                            )
                        );
                        assert_eq!(form["grant_type"], "authorization_code");
                        assert_eq!(
                            form["redirect_uri"],
                            "http://localhost:9000/oauth/sources/upstream/callback"
                        );
                        let (challenge, tokens) =
                            records.lock().unwrap().remove(&form["code"]).unwrap();
                        assert_eq!(digest(&form["code_verifier"]), challenge);
                        Json(tokens)
                    }
                },
            ),
        );
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let key = f
            .core
            .store
            .get::<crypto::Keys>("meta", "keys")
            .unwrap()
            .unwrap()
            .active;
        let jwks: riauth::jose::PublicJwks =
            serde_json::from_value(f.core.jwks().unwrap()).unwrap();
        let source = riauth::source::Source {
            saml: None,
            oauth_profile: None,
            id: "upstream".into(),
            name: "Example upstream".into(),
            issuer: issuer.clone(),
            authorization_endpoint: format!("{issuer}/authorize"),
            token_endpoint: format!("{issuer}/token"),
            client_id: "upstream-client".into(),
            token_endpoint_auth_method: riauth::jose::ClientAuthMethod::ClientSecretBasic,
            jwks,
            scopes: strings(&["openid", "profile", "email"]),
            enabled: true,
            auto_provision,
            groups: Default::default(),
            trusted_mfa_acr: Default::default(),
            allow_admin_login: false,
        };
        f.core
            .source_put(
                &f.admin,
                SourceInput {
                    source: source.clone(),
                    client_secret: Some("source-client-secret".into()),
                },
            )
            .unwrap();
        Self {
            source,
            key,
            codes,
            server,
        }
    }
    async fn callback(&self, f: &Fixture, start: &Value, subject: &str) -> Value {
        let url = url::Url::parse(start["authorization_url"].as_str().unwrap()).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["max_age"], "0");
        assert_eq!(pairs["code_challenge_method"], "S256");
        if let Some(nonce) = start["nonce"].as_str() {
            assert_eq!(pairs["nonce"], nonce);
        }
        let claims = json!({
            "iss": self.source.issuer,
            "sub": subject,
            "aud": "upstream-client",
            "iat": now(),
            "exp": now() + 300,
            "auth_time": now(),
            "nonce": pairs["nonce"],
            "email": "alice@example.test",
            "email_verified": true,
            "name": "Upstream Alice",
            "acr": "urn:upstream:mfa"
        });
        let code = crypto::random_token("");
        self.codes.lock().unwrap().insert(
            code.clone(),
            (
                pairs["code_challenge"].clone(),
                json!({"id_token": self.key.sign(&claims, false).unwrap(), "access_token": "mock-access"}),
            ),
        );
        f.core
            .source_callback(
                &self.source.id,
                vec![
                    ("state".into(), pairs["state"].clone()),
                    ("code".into(), code),
                    ("iss".into(), self.source.issuer.clone()),
                ],
            )
            .await
            .unwrap()
    }
}

fn stage_client(f: &Fixture, require_mfa: bool) {
    f.client("app", false);
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                require_mfa: Some(require_mfa),
                settings: Some(ProviderSettings {
                    source_stage: Some("upstream".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
}
fn authorization(
    f: &Fixture,
    prompt: Option<&str>,
    max_age: Option<u64>,
) -> (Authorization, String) {
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.prompt = prompt.map(str::to_owned);
    request.max_age = max_age;
    request.decision = None;
    (request, verifier)
}
fn codes(f: &Fixture) -> Vec<Code> {
    f.core
        .store
        .list::<Code>("codes")
        .unwrap()
        .into_iter()
        .map(|(_, code)| code)
        .collect()
}
fn query(redirect: &str) -> std::collections::HashMap<String, String> {
    url::Url::parse(redirect)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect()
}
#[tokio::test]
async fn suspend_then_resume_completes_the_original_request_and_replay_fails() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let alice = f.user("alice");
    let (request, verifier) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request.clone()).unwrap();
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    let transaction = text(&prepared, "transaction_id");
    assert_eq!(
        prepared["source_stage"]["nonce"].as_str().unwrap().len(),
        43
    );
    let pending: Value = f
        .core
        .store
        .get("authentication", &digest(&transaction))
        .unwrap()
        .unwrap();
    assert!(pending["authenticated_session"].is_null());
    assert_eq!(pending["source_stage"], stage_id);
    assert!(codes(&f).is_empty());
    // Even a filled proof record cannot authorize while the upstream stage is pending.
    let session_id = f
        .core
        .store
        .get::<String>("session_tokens", &digest(&alice))
        .unwrap()
        .unwrap();
    let transaction_key = digest(&transaction);
    let mut forged: AuthenticationTransaction = f
        .core
        .store
        .get("authentication", &transaction_key)
        .unwrap()
        .unwrap();
    forged.authenticated_session = Some(session_id);
    f.core
        .store
        .write(|tx| tx.put("authentication", &transaction_key, &forged))
        .unwrap();
    let mut attempted = request.clone();
    attempted.decision = Some("approve".into());
    attempted.transaction_id = Some(transaction.clone());
    assert_eq!(
        f.core.authorize(&alice, attempted).unwrap_err().code,
        "login_required"
    );
    assert!(codes(&f).is_empty());
    forged.authenticated_session = None;
    f.core
        .store
        .write(|tx| tx.put("authentication", &transaction_key, &forged))
        .unwrap();
    let mut bypass = request.clone();
    bypass.decision = Some("approve".into());
    assert_eq!(
        f.core.authorize(&alice, bypass).unwrap_err().code,
        "login_required"
    );
    assert!(
        f.core
            .login_for(
                "alice".into(),
                common::PASSWORD.into(),
                None,
                Some(transaction.clone())
            )
            .is_err()
    );
    assert!(codes(&f).is_empty());
    let callback = upstream
        .callback(&f, &prepared["source_stage"], "subject-1")
        .await;
    assert_eq!(callback["completed"], true);
    assert_eq!(callback["source_stage"]["stage_id"], stage_id);
    assert!(
        f.core
            .source_callback(
                "upstream",
                vec![
                    ("state".into(), "replay".into()),
                    ("code".into(), "replay".into()),
                ],
            )
            .await
            .is_err()
    );
    let url = url::Url::parse(
        prepared["source_stage"]["authorization_url"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .to_string();
    assert!(
        f.core
            .source_callback(
                "upstream",
                vec![
                    ("state".into(), state),
                    ("code".into(), "second".into()),
                    ("iss".into(), upstream.source.issuer.clone()),
                ],
            )
            .await
            .is_err()
    );
    let resume = f
        .core
        .source_stage_resume(&stage_id, &authorization_id, None)
        .unwrap();
    assert_eq!(resume["status"], "complete");
    assert_eq!(resume["code_issued"], true);
    assert!(!resume.to_string().contains("ri_session_"));
    let params = query(resume["redirect_uri"].as_str().unwrap());
    assert!(params["code"].starts_with("ri_code_"));
    assert_eq!(params["state"], "state with & delimiters");
    assert!(
        f.core
            .source_stage_resume(&stage_id, &authorization_id, None)
            .is_err()
    );
    assert_eq!(codes(&f).len(), 1);
    let tokens = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(params["code"].clone()),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    let jwks: riauth::jose::PublicJwks = serde_json::from_value(f.core.jwks().unwrap()).unwrap();
    let claims = jwks
        .verify(&text(&tokens, "id_token"), &f.core.config.issuer, "app")
        .unwrap();
    assert_eq!(claims["amr"], json!(["federated"]));
    assert_eq!(claims["acr"], "urn:riauth:acr:federated");
    assert_ne!(claims["sub"], f.core.me(&alice).unwrap()["user"]["id"]);
    let browser = f
        .core
        .browser_start(
            {
                let (mut request, _) = authorization(&f, None, None);
                request.nonce = Some("browser-nonce".into());
                request
            },
            None,
        )
        .unwrap();
    assert_eq!(browser.body["status"], "source_stage");
    assert!(
        browser
            .location
            .unwrap()
            .contains(&upstream.source.authorization_endpoint)
    );
}

#[tokio::test]
async fn expired_stage_is_rejected_without_a_code() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    upstream
        .callback(&f, &prepared["source_stage"], "subject-1")
        .await;
    f.core
        .store
        .write(|tx| {
            let mut stage: Value = tx.get("source_stages", &stage_id)?.unwrap();
            stage["expires_at"] = json!(1u64);
            tx.put("source_stages", &stage_id, &stage)
        })
        .unwrap();
    let error = f
        .core
        .source_stage_resume(&stage_id, &authorization_id, None)
        .unwrap_err();
    assert!(error.to_string().contains("expired"));
    assert!(codes(&f).is_empty());
}

#[tokio::test]
async fn cancel_fails_the_authorization_without_a_code() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let alice = f.user("alice");
    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request.clone()).unwrap();
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    let cancelled = f
        .core
        .source_stage_cancel(&stage_id, &authorization_id)
        .unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["code_issued"], false);
    assert_eq!(cancelled["error"], "access_denied");
    let params = query(cancelled["redirect_uri"].as_str().unwrap());
    assert_eq!(params["error"], "access_denied");
    assert!(!params.contains_key("code"));
    assert!(codes(&f).is_empty());
    assert!(
        f.core
            .source_stage_resume(&stage_id, &authorization_id, None)
            .is_err()
    );
    upstream
        .callback(&f, &prepared["source_stage"], "subject-1")
        .await;
    assert!(
        f.core
            .source_stage_resume(&stage_id, &authorization_id, None)
            .is_err()
    );
    let mut again = request.clone();
    again.decision = Some("approve".into());
    assert_eq!(
        f.core.authorize(&alice, again).unwrap_err().code,
        "access_denied"
    );
    assert!(codes(&f).is_empty());
}

#[tokio::test]
async fn upstream_account_must_match_the_bound_user_and_link_table() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    let link = f
        .core
        .source_start(
            "upstream",
            riauth::source::Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&alice),
        )
        .unwrap();
    upstream.callback(&f, &link, "subject-alice").await;
    assert_eq!(
        f.core
            .source_finish(riauth::source::Finish {
                credential: text(&link["credential"], "token"),
                approve: true,
                otp: None,
            })
            .unwrap()["user"]["username"],
        "alice"
    );
    let (request, _) = authorization(&f, Some("login"), None);
    let prepared = f.core.authorization_prepare(Some(&alice), request).unwrap();
    assert_eq!(
        f.core
            .store
            .get::<Value>(
                "authentication",
                &digest(&text(&prepared, "transaction_id"))
            )
            .unwrap()
            .unwrap()["user_id"],
        f.core.me(&alice).unwrap()["user"]["id"]
    );
    let users_before = f.core.store.list::<User>("users").unwrap().len();
    upstream
        .callback(&f, &prepared["source_stage"], "subject-bob")
        .await;
    let resume = f
        .core
        .source_stage_resume(
            &text(&prepared["source_stage"], "stage_id"),
            &text(&prepared["source_stage"], "authorization_id"),
            None,
        )
        .unwrap();
    assert_eq!(resume["code_issued"], false);
    assert_eq!(resume["error"], "access_denied");
    assert!(!query(resume["redirect_uri"].as_str().unwrap()).contains_key("code"));
    assert!(codes(&f).is_empty());
    assert_eq!(
        f.core.store.list::<User>("users").unwrap().len(),
        users_before
    );
    assert!(f.core.me(&bob).is_ok());
    let (unbound, _) = authorization(&f, None, None);
    let open = f.core.authorization_prepare(None, unbound).unwrap();
    upstream
        .callback(&f, &open["source_stage"], "subject-alice")
        .await;
    let matched = f
        .core
        .source_stage_resume(
            &text(&open["source_stage"], "stage_id"),
            &text(&open["source_stage"], "authorization_id"),
            None,
        )
        .unwrap();
    assert_eq!(matched["code_issued"], true);
    let code = query(matched["redirect_uri"].as_str().unwrap())
        .remove("code")
        .unwrap();
    let grant: Code = f.core.store.get("codes", &digest(&code)).unwrap().unwrap();
    assert_eq!(
        grant.identity.user_id,
        f.core.me(&alice).unwrap()["user"]["id"]
    );
}

#[tokio::test]
async fn prompt_and_max_age_still_require_a_fresh_transaction() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, false).await;
    stage_client(&f, false);
    let alice = f.user("alice");
    let link = f
        .core
        .source_start(
            "upstream",
            riauth::source::Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&alice),
        )
        .unwrap();
    upstream.callback(&f, &link, "subject-alice").await;
    f.core
        .source_finish(riauth::source::Finish {
            credential: text(&link["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();
    let (request, _) = authorization(&f, Some("login"), None);
    assert_eq!(
        f.core.authorize(&alice, request.clone()).unwrap_err().code,
        "login_required"
    );
    let prepared = f
        .core
        .authorization_prepare(Some(&alice), request.clone())
        .unwrap();
    let transaction = text(&prepared, "transaction_id");
    assert!(
        f.core
            .login_for(
                "alice".into(),
                common::PASSWORD.into(),
                None,
                Some(transaction)
            )
            .is_err()
    );
    upstream
        .callback(&f, &prepared["source_stage"], "subject-alice")
        .await;
    assert_eq!(
        f.core
            .source_stage_resume(
                &text(&prepared["source_stage"], "stage_id"),
                &text(&prepared["source_stage"], "authorization_id"),
                None,
            )
            .unwrap()["code_issued"],
        true
    );
    let (mut zero, _) = authorization(&f, None, Some(0));
    zero.nonce = Some("max-age-request".into());
    assert_eq!(
        f.core.authorize(&alice, zero.clone()).unwrap_err().code,
        "login_required"
    );
    let second = f.core.authorization_prepare(Some(&alice), zero).unwrap();
    assert_ne!(
        second["source_stage"]["stage_id"],
        prepared["source_stage"]["stage_id"]
    );
    assert_ne!(second["transaction_id"], prepared["transaction_id"]);
    assert!(
        f.core
            .login_for(
                "alice".into(),
                common::PASSWORD.into(),
                None,
                Some(text(&second, "transaction_id"))
            )
            .is_err()
    );
    assert!(!codes(&f).is_empty());
}

#[tokio::test]
async fn local_totp_is_still_required_when_upstream_is_not_mfa() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, true);
    let alice = f.user("alice");
    let link = f
        .core
        .source_start(
            "upstream",
            riauth::source::Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&alice),
        )
        .unwrap();
    upstream.callback(&f, &link, "subject-alice").await;
    f.core
        .source_finish(riauth::source::Finish {
            credential: text(&link["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();
    let enrollment = f.core.mfa_begin(&alice).unwrap();
    let totp = crypto::totp(&text(&enrollment, "secret"), "alice").unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate(now() - 30))
        .unwrap();
    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    upstream
        .callback(&f, &prepared["source_stage"], "subject-alice")
        .await;
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    let route = format!("/oauth/source-stages/{stage_id}/resume");
    let app = riauth::api::router(f.core.clone());
    // Reject even empty/encoded factor parameters before touching the stage.
    // A rejected GET must not consume the OTP or prevent a later JSON POST.
    for secret in ["otp=123456", "otp=", "%6ftp=recovery-code"] {
        let rejected = app
            .clone()
            .oneshot(
                Request::get(format!(
                    "{route}?authorization_id={authorization_id}&{secret}"
                ))
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
        assert!(codes(&f).is_empty());
    }
    let waiting = app
        .clone()
        .oneshot(
            Request::get(format!("{route}?authorization_id={authorization_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(waiting.status(), StatusCode::OK);
    let waiting: Value =
        serde_json::from_slice(&waiting.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(waiting["status"], "local_factor_required");
    assert_eq!(waiting["code_issued"], false);
    assert!(codes(&f).is_empty());
    let resume = app
        .oneshot(
            Request::post(route)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"authorization_id": authorization_id,
                "otp": totp.generate(now())})
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resume.status().is_redirection());
    let code = query(resume.headers()["location"].to_str().unwrap())
        .remove("code")
        .unwrap();
    let grant: Code = f.core.store.get("codes", &digest(&code)).unwrap().unwrap();
    assert!(grant.identity.mfa);
    assert_eq!(
        grant.identity.amr,
        vec!["federated".to_owned(), "otp".into()]
    );
    assert!(
        !grant.identity.amr.iter().any(|method| {
            matches!(method.as_str(), "hwk" | "pop" | "phrh" | "webauthn" | "swk")
        })
    );
}

#[tokio::test]
async fn oauth_only_stage_does_not_invent_authentication_assurance() {
    use axum::{
        Form, Json, Router,
        extract::State,
        http::{HeaderMap, StatusCode},
        routing::{get, post},
    };
    #[derive(Clone)]
    struct OAuthMock {
        challenge: Arc<Mutex<String>>,
    }
    let f = Fixture::new();
    f.client("app", false);
    let state = OAuthMock {
        challenge: Default::default(),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route(
            "/token",
            post(
                |State(state): State<OAuthMock>,
                 Form(body): Form<std::collections::HashMap<String, String>>| async move {
                    assert_eq!(digest(&body["code_verifier"]), *state.challenge.lock().unwrap());
                    Json(json!({"access_token":"upstream-test-access","token_type":"Bearer"}))
                },
            ),
        )
        .route(
            "/user",
            get(|_headers: HeaderMap| async {
                (
                    StatusCode::OK,
                    Json(json!({"id":"oauth-subject","name":"OAuth User","acr":"urn:riauth:acr:mfa"})),
                )
            }),
        )
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
            email_pointer: None,
            email_verified_pointer: None,
        }),
    };
    f.core
        .source_put(
            &f.admin,
            SourceInput {
                source,
                client_secret: None,
            },
        )
        .unwrap();
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    source_stage: Some("oauth".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let (prompt, _) = authorization(&f, Some("login"), None);
    assert!(f.core.authorization_prepare(None, prompt).is_err());
    let (max_age, _) = authorization(&f, None, Some(0));
    assert!(f.core.authorization_prepare(None, max_age).is_err());
    assert!(codes(&f).is_empty());
    let (request, verifier) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    let authorize = url::Url::parse(
        prepared["source_stage"]["authorization_url"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let pairs: std::collections::HashMap<_, _> = authorize.query_pairs().into_owned().collect();
    assert!(!pairs.contains_key("nonce"));
    assert!(!pairs.contains_key("max_age"));
    *state.challenge.lock().unwrap() = pairs["code_challenge"].clone();
    assert_eq!(
        f.core
            .source_callback(
                "oauth",
                vec![
                    ("state".into(), pairs["state"].clone()),
                    ("code".into(), "fixture-code".into()),
                ],
            )
            .await
            .unwrap()["completed"],
        true
    );
    let resume = f
        .core
        .source_stage_resume(
            &text(&prepared["source_stage"], "stage_id"),
            &text(&prepared["source_stage"], "authorization_id"),
            None,
        )
        .unwrap();
    assert_eq!(resume["code_issued"], true);
    let code = query(resume["redirect_uri"].as_str().unwrap())
        .remove("code")
        .unwrap();
    let tokens = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(code),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    let jwks: riauth::jose::PublicJwks = serde_json::from_value(f.core.jwks().unwrap()).unwrap();
    let claims = jwks
        .verify(&text(&tokens, "id_token"), &f.core.config.issuer, "app")
        .unwrap();
    assert_eq!(claims["amr"], json!(["federated"]));
    assert_eq!(claims["acr"], "urn:riauth:acr:federated");
    assert!(claims.get("auth_time").is_none());
    let session = f
        .core
        .store
        .list::<Session>("sessions")
        .unwrap()
        .into_iter()
        .map(|(_, session)| session)
        .find(|session| {
            session
                .identity
                .source
                .as_ref()
                .is_some_and(|source| source.id == "oauth")
        })
        .unwrap();
    assert_eq!(session.identity.auth_time, 0);
    assert!(!session.identity.mfa);
    let record: Value = f
        .core
        .store
        .get(
            "authentication",
            &digest(&text(&prepared, "transaction_id")),
        )
        .unwrap()
        .unwrap();
    assert!(record["authenticated_session"].is_string());
    server.abort();
}

#[tokio::test]
async fn source_stage_browser_session_is_browser_owned() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let (request, _) = authorization(&f, None, None);
    let browser = f.core.browser_start(request, None).unwrap();
    assert_eq!(browser.body["status"], "source_stage");
    let binding = browser.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("riauth_return=")
        .unwrap()
        .to_owned();
    let stage_id = text(&browser.body, "stage_id");
    let authorization_id = text(&browser.body, "authorization_id");
    upstream.callback(&f, &browser.body, "subject-1").await;
    let resume = f
        .core
        .source_stage_resume(&stage_id, &authorization_id, None)
        .unwrap();
    assert_eq!(resume["status"], "complete");
    assert!(!resume.to_string().contains("ri_session_"));
    let code = codes(&f).pop().unwrap();
    let session: Session = f
        .core
        .store
        .get("sessions", &code.identity.session_id)
        .unwrap()
        .unwrap();
    assert!(
        f.core
            .store
            .get::<String>("session_tokens", &session.token_hash)
            .unwrap()
            .is_none()
    );
    assert!(
        !f.core
            .store
            .read(|tx| riauth::signin::bearer_backed(tx, &session))
            .unwrap()
    );
    // The browser reaches the session only through its SSO cookie.
    let delivered = f
        .core
        .browser_resume(&authorization_id, Some(&binding))
        .unwrap();
    let sso = delivered
        .cookies
        .iter()
        .find_map(|c| c.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap();
    let catalogue = f.core.portal_apps(Some(sso)).unwrap();
    assert_eq!(catalogue["user"]["id"], session.identity.user_id);
}
