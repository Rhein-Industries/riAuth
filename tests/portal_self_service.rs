use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use riauth::{
    api,
    config::Config,
    core::Core,
    crypto::{digest, now},
    model::{NewClient, NewUser, ProviderSettings},
    portal::self_service::Binding,
};
use serde_json::{Value, json};
use tower::ServiceExt;

const PASSWORD: &str = "portal-self-service-test-password";
const ORIGIN: &str = "http://localhost:9000";

struct Fixture {
    _dir: tempfile::TempDir,
    core: Core,
    admin: String,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::TempDir::new().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                issuer: format!("{ORIGIN}/identity"),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Admin".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        Self {
            _dir: dir,
            core,
            admin,
        }
    }

    fn user(&self, name: &str) -> String {
        self.core
            .create_user(
                &self.admin,
                NewUser {
                    username: name.into(),
                    password: PASSWORD.into(),
                    email: None,
                    display_name: name.into(),
                    admin: false,
                },
            )
            .unwrap();
        self.core.login(name.into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn browser(&self, name: &str) -> String {
        let reply = self
            .core
            .portal_password(None, name.into(), PASSWORD.into(), None, false)
            .unwrap();
        reply
            .cookies
            .iter()
            .find_map(|cookie| cookie.strip_prefix("riauth_sso="))
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned()
    }

    fn terminal_browser(&self, token: &str) -> String {
        let request = self.core.portal_sign_in().unwrap();
        self.core
            .portal_decide(token, request.body["code"].as_str().unwrap(), true)
            .unwrap();
        let binding = request.cookies[0]
            .split('=')
            .nth(1)
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        let reply = self
            .core
            .portal_poll(request.body["id"].as_str().unwrap(), Some(binding))
            .unwrap();
        reply
            .cookies
            .iter()
            .find_map(|cookie| cookie.strip_prefix("riauth_sso="))
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned()
    }

    fn client(&self) {
        self.core
            .create_client(
                &self.admin,
                NewClient {
                    client_id: "sample".into(),
                    name: "Sample application".into(),
                    confidential: false,
                    redirect_uris: vec!["https://sample.example/callback".into()],
                    scopes: ["openid", "profile"].map(String::from).into(),
                    allowed_groups: Default::default(),
                    require_mfa: false,
                    service: false,
                    settings: ProviderSettings::default(),
                },
            )
            .unwrap();
    }
}

fn binding(view: &Value) -> Binding {
    Binding {
        expected_user_id: view["user"]["id"].as_str().unwrap().into(),
        expected_session_id: view["current_session_id"].as_str().unwrap().into(),
    }
}

#[test]
fn browser_revocation_is_owned_and_sign_out_everywhere_ends_terminal_grants() {
    let fixture = Fixture::new();
    let alice_token = fixture.user("alice");
    let bob_token = fixture.user("bob");
    let terminal_browser = fixture.terminal_browser(&alice_token);
    let terminal_view = fixture
        .core
        .portal_security(Some(&terminal_browser))
        .unwrap();
    assert_eq!(terminal_view["browser_owned"], false);
    assert_eq!(
        fixture
            .core
            .portal_revoke_all_sessions(Some(&terminal_browser), &binding(&terminal_view))
            .err()
            .unwrap()
            .code,
        "reauthentication_required"
    );
    assert!(fixture.core.me(&alice_token).is_ok());
    let alice = fixture.browser("alice");
    let another = fixture.browser("alice");
    let bob = fixture.browser("bob");
    let view = fixture.core.portal_security(Some(&alice)).unwrap();
    let binding = binding(&view);
    assert_eq!(view["sessions"].as_array().unwrap().len(), 3);
    assert!(
        view["sessions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["kind"] == "terminal")
    );
    let another_id = fixture.core.portal_security(Some(&another)).unwrap()["current_session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let bob_id = fixture.core.portal_security(Some(&bob)).unwrap()["current_session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        fixture
            .core
            .portal_revoke_selected_session(Some(&alice), &binding, &bob_id)
            .err()
            .unwrap()
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        fixture
            .core
            .portal_revoke_selected_session(Some(&bob), &binding, &another_id)
            .err()
            .unwrap()
            .status,
        StatusCode::CONFLICT
    );
    fixture
        .core
        .portal_revoke_selected_session(Some(&alice), &binding, &another_id)
        .unwrap();
    assert!(fixture.core.portal_security(Some(&another)).is_err());
    assert!(fixture.core.portal_security(Some(&alice)).is_ok());
    assert!(fixture.core.portal_security(Some(&bob)).is_ok());
    let all = fixture
        .core
        .portal_revoke_all_sessions(Some(&alice), &binding)
        .unwrap();
    assert_eq!(all.body["sessions_revoked"], 2);
    assert_eq!(all.body["signed_out"], true);
    assert!(fixture.core.portal_security(Some(&alice)).is_err());
    assert!(fixture.core.me(&alice_token).is_err());
    assert!(fixture.core.portal_security(Some(&bob)).is_ok());
    assert!(fixture.core.me(&bob_token).is_ok());
}

#[test]
fn consent_withdrawal_is_account_bound_and_requires_fresh_browser_verification() {
    let fixture = Fixture::new();
    let alice_token = fixture.user("alice");
    let bob_token = fixture.user("bob");
    fixture.client();
    let alice_id = fixture.core.me(&alice_token).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let bob_id = fixture.core.me(&bob_token).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    for user in [&alice_id, &bob_id] {
        fixture
            .core
            .store
            .write(|tx| {
                tx.put(
                    "consents",
                    &digest(&format!("{user}\0sample")),
                    &json!({"scopes":["openid"],"resource":null,"expires_at":now()+3600}),
                )
            })
            .unwrap();
    }
    let alice = fixture.browser("alice");
    let bob = fixture.browser("bob");
    let view = fixture.core.portal_security(Some(&alice)).unwrap();
    let initial_binding = binding(&view);
    assert_eq!(view["consents"][0]["client_id"], "sample");
    assert_eq!(
        fixture
            .core
            .portal_withdraw_consent(Some(&bob), &initial_binding, "sample")
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    let sid = initial_binding.expected_session_id.clone();
    fixture
        .core
        .store
        .write(|tx| {
            let mut session = tx.get::<riauth::model::Session>("sessions", &sid)?.unwrap();
            session.identity.auth_time = now() - 301;
            tx.put("sessions", &sid, &session)
        })
        .unwrap();
    assert_eq!(
        fixture
            .core
            .portal_withdraw_consent(Some(&alice), &initial_binding, "sample")
            .unwrap_err()
            .code,
        "reauthentication_required"
    );
    let fresh = fixture
        .core
        .portal_password(Some(&alice), "alice".into(), PASSWORD.into(), None, true)
        .unwrap();
    let fresh_cookie = fresh
        .cookies
        .iter()
        .find_map(|c| c.strip_prefix("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let fresh_view = fixture.core.portal_security(Some(fresh_cookie)).unwrap();
    fixture
        .core
        .portal_withdraw_consent(Some(fresh_cookie), &binding(&fresh_view), "sample")
        .unwrap();
    assert!(
        fixture
            .core
            .consents(&alice_token)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        fixture
            .core
            .consents(&bob_token)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn browser_mutation_rejects_requests_without_portal_csrf_headers() {
    let fixture = Fixture::new();
    fixture.user("alice");
    let cookie = fixture.browser("alice");
    let view = fixture.core.portal_security(Some(&cookie)).unwrap();
    let app = api::router(fixture.core.clone());
    let body = serde_json::to_string(&binding(&view)).unwrap();
    let request = Request::post("/identity/api/portal/security/sessions/revoke-all")
        .header("cookie", format!("riauth_sso={cookie}"))
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert!(fixture.core.portal_security(Some(&cookie)).is_ok());
    let current = view["current_session_id"].as_str().unwrap();
    let request = Request::post(format!(
        "/identity/api/portal/security/sessions/{current}/revoke"
    ))
    .header("cookie", format!("riauth_sso={cookie}"))
    .header("origin", ORIGIN)
    .header("sec-fetch-site", "same-origin")
    .header("x-riauth-portal", "1")
    .header("content-type", "application/json")
    .body(Body::from(serde_json::to_string(&binding(&view)).unwrap()))
    .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response
            .headers()
            .get_all("set-cookie")
            .iter()
            .any(|value| value.to_str().unwrap().contains("Max-Age=0"))
    );
    assert!(fixture.core.portal_security(Some(&cookie)).is_err());
}
