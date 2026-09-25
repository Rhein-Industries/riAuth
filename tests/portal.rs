use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt;
use riauth::{
    config::Config,
    core::Core,
    crypto::{digest, now},
    model::*,
    portal::Settings,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
use tower::ServiceExt;
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

const PASSWORD: &str = "portal-test-password-only";
const ORIGIN: &str = "http://localhost:9000";
type Authenticator = WebauthnAuthenticator<SoftPasskey>;
struct Fixture {
    _dir: tempfile::TempDir,
    core: Core,
    admin: String,
}
impl Fixture {
    fn new(prefix: &str) -> Self {
        Self::with_issuer(&format!("{ORIGIN}{prefix}"))
    }
    fn with_issuer(issuer: &str) -> Self {
        let dir = tempfile::TempDir::new().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                issuer: issuer.into(),
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
    /// Creates an account without any session.
    fn create(&self, name: &str) {
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
    }
    fn user(&self, name: &str) -> String {
        self.create(name);
        self.core.login(name.into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .into()
    }
    fn client(&self, id: &str, settings: ProviderSettings, groups: &[&str]) {
        self.core
            .create_client(
                &self.admin,
                NewClient {
                    client_id: id.into(),
                    name: id.into(),
                    confidential: false,
                    redirect_uris: vec!["https://example.test/callback".into()],
                    scopes: ["openid", "profile", "groups", "email"]
                        .map(String::from)
                        .into(),
                    allowed_groups: groups.iter().map(|s| (*s).into()).collect(),
                    require_mfa: false,
                    service: false,
                    settings,
                },
            )
            .unwrap();
    }
    fn cookie(&self, session: &str) -> String {
        let request = self.core.portal_sign_in().unwrap();
        self.core
            .portal_decide(session, request.body["code"].as_str().unwrap(), true)
            .unwrap();
        let response = self
            .core
            .portal_poll(
                request.body["id"].as_str().unwrap(),
                Some(cookie_value(&request.cookies[0])),
            )
            .unwrap();
        cookie_value(
            response
                .cookies
                .iter()
                .find(|s| s.starts_with("riauth_sso="))
                .unwrap(),
        )
        .into()
    }
    /// Signs a browser in through the portal password form and returns its SSO value.
    fn browser(&self, name: &str, sso: Option<&str>) -> String {
        let reply = self
            .core
            .portal_password(sso, name.into(), PASSWORD.into(), None, false)
            .unwrap();
        sso_value(&reply.cookies)
    }
    /// Usernameless portal passkey sign-in; returns the SSO value.
    fn passkey_browser(
        &self,
        authenticator: &mut Authenticator,
        credential: &str,
        user_id: &str,
    ) -> String {
        let started = self.core.portal_passkey_start(None, false).unwrap();
        let binding = cookie_named(&started.cookies, "riauth_passkey").unwrap();
        let proof = discoverable(self, authenticator, &started.body, credential, user_id);
        let reply = self
            .core
            .portal_passkey_finish(
                None,
                Some(&binding),
                started.body["ceremony"].as_str().unwrap(),
                serde_json::from_value(proof).unwrap(),
            )
            .unwrap();
        sso_value(&reply.cookies)
    }
    /// Enrolls a SoftPasskey from a bearer session; returns it with its credential id.
    fn enroll(&self, token: &str) -> (Authenticator, String) {
        let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
        let started = self
            .core
            .passkey_register_start(token, "Test key".into())
            .unwrap();
        let response = register(self, &mut authenticator, &started);
        let credential = response["rawId"].as_str().unwrap().to_owned();
        self.core
            .passkey_register_finish(
                token,
                started["ceremony"].as_str().unwrap(),
                serde_json::from_value(response).unwrap(),
            )
            .unwrap();
        (authenticator, credential)
    }
    /// The session a cookie maps to, including a rotated mapping's holder.
    fn sid(&self, sso: &str) -> String {
        self.core
            .store
            .get::<Value>("browser_sessions", &digest(sso))
            .unwrap()
            .unwrap()["session_id"]
            .as_str()
            .unwrap()
            .into()
    }
    fn session(&self, sid: &str) -> Session {
        self.core.store.get("sessions", sid).unwrap().unwrap()
    }
    fn age(&self, sid: &str, seconds: u64) {
        self.core
            .store
            .write(|tx| {
                let mut session: Session = tx.get("sessions", sid)?.unwrap();
                session.identity.auth_time -= seconds;
                tx.put("sessions", sid, &session)
            })
            .unwrap();
    }
    fn user_id(&self, name: &str) -> String {
        self.core
            .store
            .get::<String>("usernames", name)
            .unwrap()
            .unwrap()
    }
    fn signed_in_as(&self, sso: &str) -> Option<String> {
        self.core
            .portal_apps(Some(sso))
            .ok()
            .map(|feed| feed["user"]["username"].as_str().unwrap().into())
    }
    fn audited(&self, actor: &str, action: &str, target: &str) -> bool {
        self.core
            .audit_events(&self.admin, 1000)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["actor"] == actor && e["action"] == action && e["target"] == target)
    }
}
fn cookie_value(cookie: &str) -> &str {
    cookie.split(';').next().unwrap().split_once('=').unwrap().1
}
fn cookie_named(cookies: &[String], name: &str) -> Option<String> {
    cookies
        .iter()
        .find_map(|c| c.strip_prefix(&format!("{name}=")))
        .map(|c| c.split(';').next().unwrap().to_owned())
}
fn sso_value(cookies: &[String]) -> String {
    cookie_named(cookies, "riauth_sso")
        .filter(|v| !v.is_empty())
        .expect("SSO cookie")
}
fn origin(f: &Fixture) -> url::Url {
    let issuer = url::Url::parse(&f.core.config.issuer).unwrap();
    url::Url::parse(&issuer.origin().ascii_serialization()).unwrap()
}
/// SoftPasskey keys are not resident, so the test names the credential and supplies the
/// user handle a discoverable key would return.
fn discoverable(
    f: &Fixture,
    authenticator: &mut Authenticator,
    started: &Value,
    credential: &str,
    user_id: &str,
) -> Value {
    let mut options = started["public_key"].clone();
    options["publicKey"]["allowCredentials"] = json!([{"type":"public-key","id":credential}]);
    let proof = authenticator
        .do_authentication(origin(f), serde_json::from_value(options).unwrap())
        .unwrap();
    let mut proof = serde_json::to_value(proof).unwrap();
    let handle = Sha256::digest(format!("riauth.webauthn-user/v1\0{user_id}"));
    proof["response"]["userHandle"] = json!(URL_SAFE_NO_PAD.encode(&handle[..16]));
    proof
}
fn register(f: &Fixture, authenticator: &mut Authenticator, started: &Value) -> Value {
    let mut options = started["public_key"].clone();
    options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
    let response = authenticator
        .do_registration(origin(f), serde_json::from_value(options).unwrap())
        .unwrap();
    serde_json::to_value(response).unwrap()
}
/// A same-origin portal write, as `auth.js` sends it.
fn post(uri: &str, cookie: &str, body: Value) -> Request<Body> {
    let mut request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("origin", ORIGIN)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .header("content-type", "application/json");
    if !cookie.is_empty() {
        request = request.header("cookie", cookie);
    }
    request.body(Body::from(body.to_string())).unwrap()
}
fn get(uri: &str, cookie: &str) -> Request<Body> {
    let mut request = Request::builder().uri(uri);
    if !cookie.is_empty() {
        request = request.header("cookie", cookie);
    }
    request.body(Body::empty()).unwrap()
}
async fn call(app: &Router, request: Request<Body>) -> (StatusCode, Vec<String>, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let cookies = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_owned())
        .collect();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        cookies,
        serde_json::from_slice(&body).unwrap_or(Value::Null),
    )
}
const CLEAR: &str = "riauth_sso=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax";
fn metadata() -> ProviderSettings {
    ProviderSettings {
        app: Some(Settings {
            launch_url: Some("https://apps.example.test/home?view=all#start".into()),
            ..Default::default()
        }),
        ..Default::default()
    }
}
fn ids(feed: &Value) -> Vec<String> {
    feed["apps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|app| app["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn portal_inherits_live_access_without_admin_bypass_or_information_leaks() {
    let f = Fixture::new("");
    let alice = f.user("alice");
    let bob = f.user("bob");
    for group in ["employees", "engineering", "blocked"] {
        f.core.create_group(&f.admin, group).unwrap();
    }
    for group in ["employees", "engineering"] {
        f.core.group_member(&f.admin, group, "alice", true).unwrap();
    }
    f.client("everyone", metadata(), &[]);
    f.client("engineering", metadata(), &["engineering"]);
    f.client("no-launch-url", ProviderSettings::default(), &[]);
    f.client("app@team", metadata(), &[]);
    let mut typed = metadata();
    typed.policy.access.all_groups.insert("employees".into());
    typed.policy.access.users.insert("alice".into());
    typed.policy.access.denied_groups.insert("blocked".into());
    f.client("typed-policy", typed, &[]);
    let mut denied = metadata();
    denied.policy.access.denied_users.insert("alice".into());
    f.client("not-alice", denied, &[]);
    let mut hidden = metadata();
    hidden.app.as_mut().unwrap().hidden = true;
    f.client("hidden", hidden, &[]);
    f.client("disabled", metadata(), &[]);
    f.core
        .update_client(
            &f.admin,
            "disabled",
            ClientPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    f.client("mfa", metadata(), &[]);
    f.core
        .update_client(
            &f.admin,
            "mfa",
            ClientPatch {
                require_mfa: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let mut acr = metadata();
    acr.default_acr_values.push(riauth::assurance::MFA.into());
    f.client("acr", acr, &[]);
    let mut scopes = metadata();
    scopes
        .app
        .as_mut()
        .unwrap()
        .launch_scopes
        .insert("profile".into());
    scopes.policy.scopes.insert(
        "profile".into(),
        riauth::claims::Rule {
            denied_users: ["alice".into()].into(),
            ..Default::default()
        },
    );
    f.client("scope-policy", scopes, &[]);
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "service".into(),
                name: "Service".into(),
                confidential: true,
                redirect_uris: vec![],
                scopes: ["api".into()].into(),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: true,
                settings: Default::default(),
            },
        )
        .unwrap();
    let alice_cookie = f.cookie(&alice);
    let bob_cookie = f.cookie(&bob);
    let admin_cookie = f.cookie(&f.admin);
    let feed = f.core.portal_apps(Some(&alice_cookie)).unwrap();
    assert_eq!(
        ids(&feed),
        vec![
            "app@team",
            "engineering",
            "everyone",
            "no-launch-url",
            "typed-policy"
        ]
    );
    for secret in [
        "secret_hash",
        "redirect_uris",
        "allowed_groups",
        "denied_users",
        "settings",
        "password_hash",
        "scope-policy",
        "not-alice",
        "disabled",
        "hidden",
        "service",
    ] {
        assert!(!feed.to_string().contains(secret), "leaked {secret}");
    }
    assert!(
        feed["apps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == "no-launch-url")
            .unwrap()["launch_path"]
            .is_null()
    );
    assert_eq!(
        feed["apps"][0]["launch_path"],
        "/apps/launch?client_id=app%40team"
    );
    assert!(
        !ids(&f.core.portal_apps(Some(&bob_cookie)).unwrap()).contains(&"engineering".to_owned())
    );
    assert!(
        !ids(&f.core.portal_apps(Some(&admin_cookie)).unwrap()).contains(&"engineering".to_owned()),
        "admin is not an application policy bypass"
    );
    assert_eq!(
        f.core
            .portal_launch(Some(&alice_cookie), "everyone")
            .unwrap(),
        "https://apps.example.test/home?view=all#start"
    );
    for id in [
        "hidden",
        "disabled",
        "mfa",
        "acr",
        "scope-policy",
        "not-alice",
        "service",
        "unknown",
    ] {
        assert!(
            f.core.portal_launch(Some(&alice_cookie), id).is_err(),
            "launched {id}"
        );
    }
    assert!(
        f.core
            .portal_launch(Some(&alice_cookie), "no-launch-url")
            .is_err()
    );
    // MFA applications appear after actual TOTP authentication, without changing assignments.
    let enrollment = f.core.mfa_begin(&bob).unwrap();
    let totp = riauth::crypto::totp(enrollment["secret"].as_str().unwrap(), "bob").unwrap();
    f.core
        .mfa_confirm(&bob, &totp.generate(riauth::crypto::now() - 30))
        .unwrap();
    let mfa_login = f
        .core
        .login(
            "bob".into(),
            PASSWORD.into(),
            Some(totp.generate(riauth::crypto::now())),
        )
        .unwrap();
    let mfa_cookie = f.cookie(mfa_login["session_token"].as_str().unwrap());
    let mfa_apps = ids(&f.core.portal_apps(Some(&mfa_cookie)).unwrap());
    assert!(mfa_apps.contains(&"mfa".to_owned()));
    assert!(mfa_apps.contains(&"acr".to_owned()));
    // Membership changes affect an existing browser session, including already rendered links.
    f.core
        .group_member(&f.admin, "engineering", "alice", false)
        .unwrap();
    f.core
        .group_member(&f.admin, "blocked", "alice", true)
        .unwrap();
    assert!(
        f.core
            .portal_launch(Some(&alice_cookie), "engineering")
            .is_err()
    );
    assert!(
        f.core
            .portal_launch(Some(&alice_cookie), "typed-policy")
            .is_err()
    );
    assert_eq!(
        ids(&f.core.portal_apps(Some(&alice_cookie)).unwrap()),
        vec!["app@team", "everyone", "no-launch-url"]
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
    assert!(f.core.portal_apps(Some(&alice_cookie)).is_err());
    assert!(
        f.core
            .portal_launch(Some(&alice_cookie), "everyone")
            .is_err()
    );
}

#[test]
fn portal_sign_in_is_browser_bound_single_use_recent_and_revocable() {
    let f = Fixture::new("/identity");
    let token = f.user("alice");
    let request = f.core.portal_sign_in().unwrap();
    let id = request.body["id"].as_str().unwrap();
    let code = request.body["code"].as_str().unwrap();
    let binding = cookie_value(&request.cookies[0]);
    assert!(request.cookies[0].contains(&format!("Path=/identity/api/portal/sign-in/{id};")));
    assert!(request.cookies[0].contains("HttpOnly; SameSite=Lax"));
    assert!(f.core.portal_poll(id, Some("wrong-browser")).is_err());
    assert!(f.core.portal_request("not-a-session", code).is_err());
    assert_eq!(
        f.core.portal_request(&token, code).unwrap()["username"],
        "alice"
    );
    assert_eq!(
        f.core.portal_poll(id, Some(binding)).unwrap().body["status"],
        "pending"
    );
    f.core.portal_decide(&token, code, true).unwrap();
    assert!(f.core.portal_decide(&f.admin, code, true).is_err());
    assert!(f.core.portal_poll(id, None).is_err());
    let result = f.core.portal_poll(id, Some(binding)).unwrap();
    assert!(!result.body.to_string().contains("token"));
    let cookie = cookie_value(
        result
            .cookies
            .iter()
            .find(|c| c.starts_with("riauth_sso="))
            .unwrap(),
    );
    assert_eq!(
        f.core.portal_apps(Some(cookie)).unwrap()["user"]["username"],
        "alice"
    );
    assert!(f.core.portal_poll(id, Some(binding)).is_err());
    assert!(f.core.portal_request(&token, code).is_err());
    f.core.portal_sign_out(Some(cookie)).unwrap();
    assert!(f.core.portal_apps(Some(cookie)).is_err());
    assert!(f.core.me(&token).is_err());
    let denied = f.core.portal_sign_in().unwrap();
    let id = denied.body["id"].as_str().unwrap();
    f.core
        .portal_decide(&f.admin, denied.body["code"].as_str().unwrap(), false)
        .unwrap();
    assert_eq!(
        f.core
            .portal_poll(id, Some(cookie_value(&denied.cookies[0])))
            .unwrap()
            .body["status"],
        "denied"
    );
    let cancelled = f.core.portal_sign_in().unwrap();
    let id = cancelled.body["id"].as_str().unwrap();
    assert!(f.core.portal_cancel(id, Some("wrong-browser")).is_err());
    f.core
        .portal_cancel(id, Some(cookie_value(&cancelled.cookies[0])))
        .unwrap();
    assert!(
        f.core
            .portal_decide(&f.admin, cancelled.body["code"].as_str().unwrap(), true)
            .is_err()
    );
    // A saved CLI session still needs fresh user authentication before browser sign-in.
    let sid = f.core.me(&f.admin).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    f.core
        .store
        .write(|tx| {
            let mut s: Session = tx.get("sessions", &sid)?.unwrap();
            s.identity.auth_time = riauth::crypto::now() - 301;
            tx.put("sessions", &sid, &s)
        })
        .unwrap();
    let stale = f.core.portal_sign_in().unwrap();
    let code = stale.body["code"].as_str().unwrap();
    assert_eq!(
        f.core.portal_request(&f.admin, code).unwrap()["reauthentication_required"],
        true
    );
    assert!(f.core.portal_decide(&f.admin, code, true).is_err());
    f.core
        .store
        .write(|tx| riauth::portal::cleanup(tx, riauth::crypto::now() + 601))
        .unwrap();
    assert!(f.core.portal_request(&f.admin, code).is_err());
}

#[tokio::test]
async fn portal_request_warns_at_240_seconds_and_shows_requester() {
    let f = Fixture::new("");
    let app = riauth::api::router(f.core.clone());
    let request = Request::builder()
        .method("POST")
        .uri("/api/portal/sign-in")
        .header("origin", ORIGIN)
        .header("x-riauth-portal", "1")
        .header("user-agent", "RequesterProbe/1.0")
        .body(Body::empty())
        .unwrap();
    let (status, _, started) = call(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    let code = started["code"].as_str().unwrap();
    let sid = f.core.me(&f.admin).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    f.age(&sid, 235);
    let details = f.core.portal_request(&f.admin, code).unwrap();
    assert_eq!(details["reauthentication_required"], false);
    assert_eq!(
        details["requested_from"]["user_agent"],
        "RequesterProbe/1.0"
    );
    assert!(
        details["requested_from"]["ip"].is_string() && details["requested_from"]["at"].is_u64()
    );
    // The CLI re-authenticates from 240 s, before approval stops working at 300 s.
    f.age(&sid, 10);
    assert_eq!(
        f.core.portal_request(&f.admin, code).unwrap()["reauthentication_required"],
        true
    );
    assert!(f.core.portal_decide(&f.admin, code, true).is_ok());
}

#[tokio::test]
async fn portal_http_enforces_origin_and_preserves_prefix_and_cookie_boundaries() {
    let f = Fixture::new("/identity");
    f.client("app", metadata(), &[]);
    let sso = f.cookie(&f.admin);
    let app = riauth::api::router(f.core.clone());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/identity/")
                .header("accept", "text/html")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert!(
        response.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("script-src 'self'")
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
    assert!(body.contains("/identity/portal/assets/app.js"));
    assert!(!body.contains("__BASE__"));
    assert!(!body.contains(&sso));
    for (origin, custom) in [
        (None, false),
        (Some("https://evil.test"), true),
        (Some("null"), true),
        (Some("http://localhost:9000"), false),
    ] {
        let mut req = Request::builder()
            .method("POST")
            .uri("/identity/api/portal/sign-out")
            .header("cookie", format!("riauth_sso={sso}"));
        if let Some(origin) = origin {
            req = req.header("origin", origin);
        }
        if custom {
            req = req.header("x-riauth-portal", "1");
        }
        let res = app
            .clone()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }
    for cookie in [
        None,
        Some("riauth_sso=bad".into()),
        Some(format!("riauth_sso={sso}; riauth_sso={sso}")),
    ] {
        let mut req = Request::builder().uri("/identity/api/portal");
        if let Some(cookie) = cookie {
            req = req.header("cookie", cookie);
        }
        assert_eq!(
            app.clone()
                .oneshot(req.body(Body::empty()).unwrap())
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    let feed = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/identity/api/portal")
                .header("cookie", format!("riauth_sso={sso}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(feed.status(), StatusCode::OK);
    let data: Value =
        serde_json::from_slice(&feed.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(
        data["apps"][0]["launch_path"],
        "/identity/apps/launch?client_id=app"
    );
    let launch = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/identity/apps/launch?client_id=app")
                .header("cookie", format!("riauth_sso={sso}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(launch.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        launch.headers()["location"],
        "https://apps.example.test/home?view=all#start"
    );
    let request = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/identity/api/portal/sign-in")
                .header("origin", "http://localhost:9000")
                .header("x-riauth-portal", "1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(request.status(), StatusCode::OK);
    assert!(
        request.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .contains("HttpOnly")
    );
    let logout = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/identity/api/portal/sign-out")
                .header("origin", "http://localhost:9000")
                .header("x-riauth-portal", "1")
                .header("cookie", format!("riauth_sso={sso}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::OK);
    assert!(
        logout.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );
    assert!(f.core.portal_apps(Some(&sso)).is_err());
    let denied = app
        .oneshot(
            Request::builder()
                .uri("/identity/apps/launch?client_id=app")
                .header("cookie", format!("riauth_sso={sso}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    assert!(
        denied.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/html")
    );
    assert!(
        String::from_utf8(
            denied
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec()
        )
        .unwrap()
        .contains("Back to applications")
    );
}

#[test]
fn portal_metadata_rejects_unsafe_links_and_round_trips_in_agent_schema() {
    let f = Fixture::new("");
    f.client("app", metadata(), &[]);
    for url in [
        "javascript:alert(1)",
        "data:text/html,x",
        "//evil.test",
        "http://evil.test",
        "https://user:password@example.test",
        "https://example.test/\n",
        "https://example.test/%(username)s",
    ] {
        let mut settings = metadata();
        settings.app.as_mut().unwrap().launch_url = Some(url.into());
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
                .is_err(),
            "accepted {url}"
        );
    }
    assert!(
        riauth::schema::schema("provider")
            .unwrap()
            .to_string()
            .contains("launch_scopes")
    );
    let manifest = f.core.export_state(&f.admin).unwrap();
    assert!(manifest.to_string().contains("apps.example.test/home"));
    // Existing providers need no metadata migration and never launch their callback URI.
    f.client("old", ProviderSettings::default(), &[]);
    let cookie = f.cookie(&f.admin);
    assert!(f.core.portal_launch(Some(&cookie), "old").is_err());
    assert_eq!(
        serde_json::from_value::<Settings>(json!({"category":"Engineering"}))
            .unwrap()
            .category,
        "Engineering"
    );
}

#[tokio::test]
async fn portal_password_sign_in_sets_only_an_httponly_cookie() {
    let f = Fixture::new("/identity");
    let cli = f.user("alice");
    let app = riauth::api::router(f.core.clone());
    let (status, cookies, body) = call(
        &app,
        post(
            "/identity/api/portal/login/password",
            "",
            json!({"username":"alice","password":PASSWORD,"otp":null,"reauthenticate":false}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({"status":"signed_in"}));
    assert_eq!(cookies.len(), 1, "{cookies:?}");
    assert!(cookies[0].starts_with("riauth_sso=ri_sso_"), "{cookies:?}");
    for part in ["; Path=/identity/;", "; HttpOnly", "; SameSite=Lax"] {
        assert!(cookies[0].contains(part), "{cookies:?}");
    }
    assert!(!cookies[0].contains("Domain") && !cookies[0].contains("Secure"));
    let sso = sso_value(&cookies);
    let sid = f.sid(&sso);
    let session = f.session(&sid);
    assert_eq!(session.identity.session_id, sid);
    assert!(!session.identity.mfa);
    assert!(
        f.core
            .store
            .get::<String>("session_tokens", &session.token_hash)
            .unwrap()
            .is_none(),
        "no bearer token exists for a browser session"
    );
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );
    let kinds = f.core.sessions(&cli).unwrap();
    let kind = |id: &str| {
        kinds
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == id)
            .map(|s| s["kind"].clone())
    };
    assert_eq!(kind(&sid), Some(json!("browser")));
    assert_eq!(
        kind(f.core.me(&cli).unwrap()["session_id"].as_str().unwrap()),
        Some(json!("terminal"))
    );
    let (status, _, feed) = call(
        &app,
        get("/identity/api/portal", &format!("riauth_sso={sso}")),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(feed["user"]["username"], "alice");
    assert_eq!(
        (&feed["mfa"], &feed["mfa_available"]),
        (&json!(false), &json!(false))
    );
    for text in [body.to_string(), feed.to_string()] {
        for secret in ["ri_session_", "ri_sso_", "ri_auth_", "session_token"] {
            assert!(!text.contains(secret), "{secret} in {text}");
        }
    }
}

type Variant = fn(&mut axum::http::HeaderMap);
#[tokio::test]
async fn portal_login_endpoints_enforce_origin_header_fetch_site_and_json() {
    let f = Fixture::new("/identity");
    f.create("alice");
    let sso = f.browser("alice", None);
    let cookie = format!("riauth_sso={sso}");
    let variants: [(&str, Variant); 11] = [
        ("no origin", |h| {
            h.remove("origin");
        }),
        ("foreign origin", |h| {
            h.insert("origin", "https://evil.test".parse().unwrap());
        }),
        ("opaque origin", |h| {
            h.insert("origin", "null".parse().unwrap());
        }),
        ("two origins", |h| {
            h.append("origin", ORIGIN.parse().unwrap());
        }),
        ("no portal header", |h| {
            h.remove("x-riauth-portal");
        }),
        ("wrong portal header", |h| {
            h.insert("x-riauth-portal", "true".parse().unwrap());
        }),
        ("two portal headers", |h| {
            h.append("x-riauth-portal", "1".parse().unwrap());
        }),
        ("same-site request", |h| {
            h.insert("sec-fetch-site", "same-site".parse().unwrap());
        }),
        ("cross-site request", |h| {
            h.insert("sec-fetch-site", "cross-site".parse().unwrap());
        }),
        ("user-initiated request", |h| {
            h.insert("sec-fetch-site", "none".parse().unwrap());
        }),
        ("two fetch sites", |h| {
            h.append("sec-fetch-site", "same-origin".parse().unwrap());
        }),
    ];
    for (path, body) in [
        (
            "login/password",
            json!({"username":"alice","password":PASSWORD}),
        ),
        ("login/passkey/start", json!({"reauthenticate":false})),
        (
            "login/passkey/finish",
            json!({"ceremony":"ri_passkey_auth_x","credential":{}}),
        ),
        ("passkeys/registration/start", json!({"name":"Laptop"})),
        (
            "passkeys/registration/finish",
            json!({"ceremony":"ri_passkey_enroll_x","credential":{}}),
        ),
        ("passkeys/unknown/remove", json!({})),
        ("sign-out", json!({"scope":"browser"})),
    ] {
        // A fresh router per endpoint keeps these requests within every rate bucket.
        let app = riauth::api::router(f.core.clone());
        let uri = format!("/identity/api/portal/{path}");
        for (name, variant) in variants {
            let mut request = post(&uri, &cookie, body.clone());
            variant(request.headers_mut());
            let (status, cookies, error) = call(&app, request).await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{path}: {name}");
            assert_eq!(error["error"], "access_denied", "{path}: {name}");
            assert!(cookies.is_empty(), "{path}: {name}");
        }
        let mut request = post(&uri, &cookie, body.clone());
        request
            .headers_mut()
            .insert("content-type", "text/plain".parse().unwrap());
        assert_eq!(
            call(&app, request).await.0,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "{path}"
        );
        let mut unknown = body.clone();
        unknown["unexpected"] = json!(true);
        assert_eq!(
            call(&app, post(&uri, &cookie, unknown)).await.0,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{path}"
        );
    }
    assert_eq!(
        f.signed_in_as(&sso).as_deref(),
        Some("alice"),
        "no refused request acted"
    );
    // Input limits are checked before any password work.
    let app = riauth::api::router(f.core.clone());
    for body in [
        json!({"username":"alice","password":"x".repeat(1025)}),
        json!({"username":"alice","password":PASSWORD,"otp":"1".repeat(129)}),
        json!({"username":"not a name","password":PASSWORD}),
    ] {
        let (status, _, error) =
            call(&app, post("/identity/api/portal/login/password", "", body)).await;
        assert_eq!(
            (status, error["error"].as_str()),
            (StatusCode::BAD_REQUEST, Some("invalid_request"))
        );
    }
    let (status, _, error) = call(
        &app,
        post(
            "/identity/api/portal/passkeys/registration/start",
            &cookie,
            json!({"name":""}),
        ),
    )
    .await;
    assert_eq!(
        (status, error["error"].as_str()),
        (StatusCode::BAD_REQUEST, Some("invalid_request"))
    );
    // Browsers without Fetch Metadata still pass the guard.
    let mut request = post("/identity/api/portal/login/passkey/start", "", json!({}));
    request.headers_mut().remove("sec-fetch-site");
    assert_eq!(call(&app, request).await.0, StatusCode::OK);
}

#[tokio::test]
async fn portal_login_errors_are_uniform_including_lockout() {
    let f = Fixture::new("");
    for name in ["alice", "bob", "dave"] {
        f.create(name);
    }
    f.core
        .update_user(
            &f.admin,
            "dave",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let bob = f.browser("bob", None);
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let app = riauth::api::router(f.core.clone());
    let attempt = |username: &str, password: &str, cookie: &str, reauthenticate: bool| {
        let request = post(
            "/api/portal/login/password",
            cookie,
            json!({"username":username,"password":password,"otp":null,"reauthenticate":reauthenticate}),
        );
        let app = app.clone();
        async move {
            let started = Instant::now();
            let (status, cookies, body) = call(&app, request).await;
            assert!(cookies.is_empty(), "{cookies:?}");
            (status, body, started.elapsed())
        }
    };
    let mut failures = vec![
        attempt("nobody", PASSWORD, "", false).await,
        attempt("dave", PASSWORD, "", false).await,
        // Re-authentication pinned to bob refuses alice's valid password alike.
        attempt("alice", PASSWORD, &format!("riauth_sso={bob}"), true).await,
    ];
    for _ in 0..5 {
        failures.push(attempt("alice", "wrong-password", "", false).await);
    }
    // Locked: even the right password gets the same answer.
    failures.push(attempt("alice", PASSWORD, "", false).await);
    let expected = json!({
        "error": "invalid_credentials",
        "error_description": "Check your username, password and code. After several failed attempts, sign-in pauses for 15 minutes."
    });
    for (i, (status, body, elapsed)) in failures.iter().enumerate() {
        assert_eq!(*status, StatusCode::UNAUTHORIZED, "attempt {i}");
        assert_eq!(body, &expected, "attempt {i}");
        assert!(
            *elapsed >= Duration::from_millis(1000),
            "attempt {i} answered after {elapsed:?}"
        );
    }
    assert!(f.audited("anonymous", "login.locked", "alice"));
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions,
        "no failure created a session"
    );
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );
    assert_eq!(f.signed_in_as(&bob).as_deref(), Some("bob"));
}

#[test]
fn portal_totp_and_recovery_code_sign_in() {
    let f = Fixture::new("");
    let bob = f.user("bob");
    let enrollment = f.core.mfa_begin(&bob).unwrap();
    let totp = riauth::crypto::totp(enrollment["secret"].as_str().unwrap(), "bob").unwrap();
    f.core
        .mfa_confirm(&bob, &totp.generate(now() - 30))
        .unwrap();
    let cli = f
        .core
        .login("bob".into(), PASSWORD.into(), Some(totp.generate(now())))
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let recovery = f.core.recovery_codes(&cli).unwrap()["recovery_codes"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let sign_in = |otp: Option<&str>| {
        f.core
            .portal_password(
                None,
                "bob".into(),
                PASSWORD.into(),
                otp.map(String::from),
                false,
            )
            .map(|reply| sso_value(&reply.cookies))
    };
    for otp in [None, Some(""), Some("abcdef")] {
        assert_eq!(
            sign_in(otp).unwrap_err().code,
            "invalid_credentials",
            "{otp:?}"
        );
    }
    let code = totp.generate(now() + 30);
    let sso = sign_in(Some(&code)).unwrap();
    assert!(f.session(&f.sid(&sso)).identity.mfa);
    assert_eq!(f.core.portal_apps(Some(&sso)).unwrap()["mfa"], true);
    assert_eq!(
        sign_in(Some(&code)).unwrap_err().code,
        "invalid_credentials",
        "a code is spent once"
    );
    let sso = sign_in(Some(&recovery)).unwrap();
    let session = f.session(&f.sid(&sso));
    assert!(session.identity.mfa);
    assert_eq!(session.identity.authentication_methods(), ["pwd", "otp"]);
    assert_eq!(
        sign_in(Some(&recovery)).unwrap_err().code,
        "invalid_credentials",
        "recovery codes are single use"
    );
    assert!(f.core.me(&cli).is_ok(), "the terminal session is untouched");
}

#[test]
fn portal_reauthentication_merges_session_and_extends_expiry() {
    let f = Fixture::new("");
    f.create("alice");
    f.create("bob");
    let first = f.browser("alice", None);
    let sid = f.sid(&first);
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", &sid)?.unwrap();
            session.identity.auth_time -= 400;
            session.expires_at -= 1000;
            tx.put("sessions", &sid, &session)
        })
        .unwrap();
    let before = f.session(&sid);
    let reauthenticate = |sso: Option<&str>, name: &str, password: &str| {
        f.core
            .portal_password(sso, name.into(), password.into(), None, true)
    };
    for sso in [None, Some("ri_sso_unknown")] {
        let error = reauthenticate(sso, "alice", PASSWORD).err().unwrap();
        assert_eq!(
            (error.status, error.code),
            (StatusCode::UNAUTHORIZED, "invalid_token")
        );
    }
    for (name, password) in [("bob", PASSWORD), ("alice", "wrong-password")] {
        assert_eq!(
            reauthenticate(Some(&first), name, password)
                .err()
                .unwrap()
                .code,
            "invalid_credentials",
            "{name}"
        );
    }
    assert_eq!(
        f.session(&sid).identity.auth_time,
        before.identity.auth_time
    );
    let second = sso_value(
        &reauthenticate(Some(&first), "alice", PASSWORD)
            .unwrap()
            .cookies,
    );
    assert_ne!(second, first, "every sign-in issues a new cookie");
    assert_eq!(f.sid(&second), sid, "the session id is kept");
    let after = f.session(&sid);
    assert_eq!(after.identity.session_id, sid);
    assert!(after.identity.auth_time + 5 >= now());
    assert!(after.expires_at > before.expires_at);
    assert!(!after.revoked);
    assert_eq!(
        f.signed_in_as(&first),
        None,
        "the rotated cookie never authenticates"
    );
    assert_eq!(f.signed_in_as(&second).as_deref(), Some("alice"));
    let alice = f.user_id("alice");
    assert!(f.audited(&alice, "browser.reauthenticate", &sid));
    assert_eq!(
        f.core
            .store
            .list::<Session>("sessions")
            .unwrap()
            .iter()
            .filter(|(_, s)| s.identity.user_id == alice)
            .count(),
        1
    );
}

#[test]
fn portal_account_switch_revokes_previous_browser_session() {
    let f = Fixture::new("");
    f.create("alice");
    f.create("bob");
    let alice = f.browser("alice", None);
    let alice_sid = f.sid(&alice);
    let bob = f.browser("bob", Some(&alice));
    let bob_sid = f.sid(&bob);
    assert_ne!(bob_sid, alice_sid);
    assert!(f.session(&alice_sid).revoked);
    assert!(!f.session(&bob_sid).revoked);
    let bob_id = f.user_id("bob");
    assert!(f.audited(&bob_id, "session.revoke", &alice_sid));
    assert!(f.audited(&bob_id, "browser.sign_in", &bob_sid));
    assert_eq!(f.signed_in_as(&alice), None);
    assert_eq!(f.signed_in_as(&bob).as_deref(), Some("bob"));
    // Switching back creates a new session; neither earlier session comes back.
    let again = f.sid(&f.browser("alice", Some(&bob)));
    assert!(again != alice_sid && again != bob_sid);
    assert!(f.session(&alice_sid).revoked && f.session(&bob_sid).revoked);
}

#[test]
fn portal_sign_in_over_terminal_session_leaves_terminal_session_valid() {
    let f = Fixture::new("");
    let cli = f.user("alice");
    f.create("bob");
    let cli_sid = f.core.me(&cli).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    for name in ["alice", "bob"] {
        let terminal = f.cookie(&cli);
        assert_eq!(f.sid(&terminal), cli_sid);
        let browser = f.browser(name, Some(&terminal));
        let sid = f.sid(&browser);
        assert_ne!(sid, cli_sid, "{name}: a new browser-owned session");
        assert!(
            f.core
                .store
                .get::<String>("session_tokens", &f.session(&sid).token_hash)
                .unwrap()
                .is_none()
        );
        assert!(
            f.core.me(&cli).is_ok(),
            "{name}: the terminal session is kept"
        );
        assert!(!f.session(&cli_sid).revoked);
        assert_eq!(
            f.signed_in_as(&terminal),
            None,
            "{name}: the old mapping is retired"
        );
        assert_eq!(f.signed_in_as(&browser).as_deref(), Some(name));
    }
}

#[tokio::test]
async fn portal_terminal_approval_revokes_displaced_browser_owned_session() {
    let f = Fixture::new("/identity");
    f.create("alice");
    let bob = f.user("bob");
    let carol = f.user("carol");
    let approve = |token: &str| {
        let request = f.core.portal_sign_in().unwrap();
        f.core
            .portal_decide(token, request.body["code"].as_str().unwrap(), true)
            .unwrap();
        (
            request.body["id"].as_str().unwrap().to_owned(),
            cookie_value(&request.cookies[0]).to_owned(),
        )
    };
    let alice = f.browser("alice", None);
    let alice_sid = f.sid(&alice);
    let (id, binding) = approve(&bob);
    let app = riauth::api::router(f.core.clone());
    let (status, cookies, body) = call(
        &app,
        post(
            &format!("/identity/api/portal/sign-in/{id}"),
            &format!("riauth_portal={binding}; riauth_sso={alice}"),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "approved");
    let pointed = sso_value(&cookies);
    assert_eq!(
        f.sid(&pointed),
        f.core.me(&bob).unwrap()["session_id"].as_str().unwrap()
    );
    assert!(
        f.session(&alice_sid).revoked,
        "the displaced session is revoked"
    );
    assert!(f.audited(&f.user_id("bob"), "session.revoke", &alice_sid));
    assert_eq!(f.signed_in_as(&alice), None);
    assert_eq!(f.signed_in_as(&pointed).as_deref(), Some("bob"));
    // A displaced terminal session is only unmapped.
    let (id, binding) = approve(&carol);
    let reply = f
        .core
        .portal_poll_with(&id, Some(&binding), Some(&pointed))
        .unwrap();
    assert_eq!(reply.body["status"], "approved");
    assert!(f.core.me(&bob).is_ok());
    assert_eq!(f.signed_in_as(&pointed), None);
    assert_eq!(
        f.signed_in_as(&sso_value(&reply.cookies)).as_deref(),
        Some("carol")
    );
}

#[tokio::test]
async fn portal_passkey_sign_in_requires_binding_cookie() {
    let f = Fixture::new("/identity");
    let token = f.user("alice");
    f.create("bob");
    let alice_id = f.user_id("alice");
    let (mut authenticator, credential) = f.enroll(&token);
    let app = riauth::api::router(f.core.clone());
    let start = |cookie: &str, body: Value| {
        call(
            &app,
            post("/identity/api/portal/login/passkey/start", cookie, body),
        )
    };
    let finish = |cookie: &str, ceremony: &Value, proof: &Value| {
        call(
            &app,
            post(
                "/identity/api/portal/login/passkey/finish",
                cookie,
                json!({"ceremony":ceremony,"credential":proof}),
            ),
        )
    };
    let (status, cookies, started) = start("", json!({})).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    let binding = cookie_named(&cookies, "riauth_passkey").unwrap();
    assert!(binding.starts_with("ri_passkey_bind_"));
    assert!(
        cookies[0].ends_with(
            "; Path=/identity/api/portal/login/passkey/; Max-Age=300; HttpOnly; SameSite=Lax"
        ),
        "{cookies:?}"
    );
    assert!(!started.to_string().contains(&binding));
    assert_eq!(started["expires_in"], 300);
    assert_eq!(
        started["public_key"]["publicKey"]["allowCredentials"],
        json!([])
    );
    assert!(started["public_key"].get("mediation").is_none());
    // Without the binding cookie the ceremony fails, and it is spent.
    let proof = discoverable(&f, &mut authenticator, &started, &credential, &alice_id);
    let (status, cookies, error) = finish("", &started["ceremony"], &proof).await;
    assert_eq!(
        (status, error["error"].as_str()),
        (StatusCode::UNAUTHORIZED, Some("invalid_token"))
    );
    assert!(cookies.is_empty());
    let bound = format!("riauth_passkey={binding}");
    assert_eq!(
        finish(&bound, &started["ceremony"], &proof).await.0,
        StatusCode::UNAUTHORIZED
    );
    // Another ceremony's binding is refused.
    let (_, _, started) = start("", json!({})).await;
    let (_, cookies, _) = start("", json!({})).await;
    let other = format!(
        "riauth_passkey={}",
        cookie_named(&cookies, "riauth_passkey").unwrap()
    );
    let proof = discoverable(&f, &mut authenticator, &started, &credential, &alice_id);
    assert_eq!(
        finish(&other, &started["ceremony"], &proof).await.0,
        StatusCode::UNAUTHORIZED
    );
    let (_, cookies, started) = start("", json!({})).await;
    let bound = format!(
        "riauth_passkey={}",
        cookie_named(&cookies, "riauth_passkey").unwrap()
    );
    let proof = discoverable(&f, &mut authenticator, &started, &credential, &alice_id);
    let (status, cookies, body) = finish(&bound, &started["ceremony"], &proof).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({"status":"signed_in"}));
    assert!(
        cookies.contains(
            &"riauth_passkey=; Path=/identity/api/portal/login/passkey/; Max-Age=0; HttpOnly; SameSite=Lax"
                .to_owned()
        ),
        "{cookies:?}"
    );
    let sso = sso_value(&cookies);
    let session = f.session(&f.sid(&sso));
    assert!(session.identity.mfa);
    assert_eq!(session.identity.amr, ["webauthn", "mfa"]);
    assert!(
        f.core
            .store
            .get::<String>("session_tokens", &session.token_hash)
            .unwrap()
            .is_none()
    );
    // Re-authentication lists only the session user's keys.
    assert_eq!(
        start("", json!({"reauthenticate":true})).await.0,
        StatusCode::UNAUTHORIZED
    );
    let bob = f.browser("bob", None);
    let (status, _, error) =
        start(&format!("riauth_sso={bob}"), json!({"reauthenticate":true})).await;
    assert_eq!(
        (status, error["error"].as_str()),
        (StatusCode::CONFLICT, Some("no_passkey"))
    );
    let (status, cookies, started) =
        start(&format!("riauth_sso={sso}"), json!({"reauthenticate":true})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        started["public_key"]["publicKey"]["allowCredentials"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let proof = authenticator
        .do_authentication(
            origin(&f),
            serde_json::from_value(started["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let bound = format!(
        "riauth_passkey={}; riauth_sso={sso}",
        cookie_named(&cookies, "riauth_passkey").unwrap()
    );
    let (status, cookies, _) = finish(
        &bound,
        &started["ceremony"],
        &serde_json::to_value(proof).unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        f.sid(&sso_value(&cookies)),
        session.id,
        "the session is kept"
    );
    let (status, _, _) = finish(&bound, &json!("ri_passkey_auth_x"), &json!({"id":1})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "malformed WebAuthn data");
}

#[tokio::test]
async fn portal_passkey_registration_requires_fresh_mfa_rule_and_signs_out() {
    let f = Fixture::new("");
    f.create("alice");
    let alice_id = f.user_id("alice");
    let app = riauth::api::router(f.core.clone());
    let start = |sso: &str| {
        call(
            &app,
            post(
                "/api/portal/passkeys/registration/start",
                &format!("riauth_sso={sso}"),
                json!({"name":"Laptop"}),
            ),
        )
    };
    assert_eq!(start("ri_sso_unknown").await.0, StatusCode::UNAUTHORIZED);
    let stale = f.browser("alice", None);
    f.age(&f.sid(&stale), 301);
    let (status, _, error) = start(&stale).await;
    assert_eq!(
        (status, error["error"].as_str()),
        (StatusCode::FORBIDDEN, Some("reauthentication_required"))
    );
    let sso = sso_value(
        &f.core
            .portal_password(Some(&stale), "alice".into(), PASSWORD.into(), None, true)
            .unwrap()
            .cookies,
    );
    let (status, _, started) = start(&sso).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    assert!(
        started["ceremony"]
            .as_str()
            .unwrap()
            .starts_with("ri_passkey_enroll_")
    );
    assert_eq!(
        started["public_key"]["publicKey"]["authenticatorSelection"],
        json!({"residentKey":"required","requireResidentKey":true,"userVerification":"required"})
    );
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let response = register(&f, &mut authenticator, &started);
    let credential = response["rawId"].as_str().unwrap().to_owned();
    let finish = |sso: &str| {
        call(
            &app,
            post(
                "/api/portal/passkeys/registration/finish",
                &format!("riauth_sso={sso}"),
                json!({"ceremony":started["ceremony"],"credential":response}),
            ),
        )
    };
    // Only the session that started the ceremony can finish it.
    let other = f.browser("alice", None);
    assert_eq!(finish(&other).await.0, StatusCode::UNAUTHORIZED);
    let (status, cookies, body) = finish(&sso).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["status"], "enrolled");
    assert_eq!(body["sessions_revoked"], true);
    assert_eq!(body["passkey"]["name"], "Laptop");
    assert_eq!(cookies, [CLEAR]);
    assert!(
        f.core
            .store
            .get::<Value>("browser_sessions", &digest(&sso))
            .unwrap()
            .is_none(),
        "the mapping is deleted"
    );
    assert_eq!(
        f.signed_in_as(&other),
        None,
        "the epoch bump ends every session"
    );
    // With a passkey enrolled, a password-only session cannot add another (F15).
    let password = f.browser("alice", None);
    let (status, _, error) = start(&password).await;
    assert_eq!(
        (status, error["error"].as_str()),
        (StatusCode::FORBIDDEN, Some("mfa_required"))
    );
    // The new passkey signs in without a username, and that session may enroll.
    let passkey = f.passkey_browser(&mut authenticator, &credential, &alice_id);
    assert_eq!(start(&passkey).await.0, StatusCode::OK);
}

#[tokio::test]
async fn portal_passkey_remove_requires_fresh_mfa_session() {
    let f = Fixture::new("");
    let token = f.user("alice");
    let alice_id = f.user_id("alice");
    let (mut authenticator, credential) = f.enroll(&token);
    let id = f.core.store.list::<Value>("passkeys").unwrap()[0].0.clone();
    let app = riauth::api::router(f.core.clone());
    let remove = |sso: &str| {
        call(
            &app,
            post(
                &format!("/api/portal/passkeys/{id}/remove"),
                &format!("riauth_sso={sso}"),
                json!({}),
            ),
        )
    };
    assert_eq!(remove("ri_sso_unknown").await.0, StatusCode::UNAUTHORIZED);
    let password = f.browser("alice", None);
    let (status, _, error) = remove(&password).await;
    assert_eq!(
        (status, error["error"].as_str()),
        (StatusCode::FORBIDDEN, Some("mfa_required"))
    );
    let stale = f.passkey_browser(&mut authenticator, &credential, &alice_id);
    f.age(&f.sid(&stale), 301);
    let (status, _, error) = remove(&stale).await;
    assert_eq!(
        (status, error["error"].as_str()),
        (StatusCode::FORBIDDEN, Some("reauthentication_required"))
    );
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
    let fresh = f.passkey_browser(&mut authenticator, &credential, &alice_id);
    let (status, cookies, body) = remove(&fresh).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, json!({"removed":true,"sessions_revoked":true}));
    assert_eq!(cookies, [CLEAR]);
    assert!(f.core.store.list::<Value>("passkeys").unwrap().is_empty());
    assert!(
        f.core
            .store
            .get::<Value>("browser_sessions", &digest(&fresh))
            .unwrap()
            .is_none()
    );
    for sso in [&password, &stale, &fresh] {
        assert_eq!(f.signed_in_as(sso), None);
    }
    let user: User = f.core.store.get("users", &alice_id).unwrap().unwrap();
    assert!(!user.has_passkeys);
}

/// A browser that collected a terminal approval shares the approver's terminal session,
/// which approval keeps fresh and which is MFA after a passkey login. A phished approval
/// must not let that browser plant or remove passkeys: it signs in here first.
#[tokio::test]
async fn terminal_shared_browser_signs_in_before_changing_passkeys() {
    let f = Fixture::new("");
    let bootstrap = f.user("alice");
    let (mut authenticator, _) = f.enroll(&bootstrap);
    let challenge = f.core.passkey_login_start("alice", None).unwrap();
    let proof = authenticator
        .do_authentication(
            origin(&f),
            serde_json::from_value(challenge["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let cli = f
        .core
        .passkey_login_finish(challenge["ceremony"].as_str().unwrap(), proof)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let terminal = f.cookie(&cli);
    let id = f.core.store.list::<Value>("passkeys").unwrap()[0].0.clone();
    let app = riauth::api::router(f.core.clone());
    let cookie = |sso: &str| format!("riauth_sso={sso}");
    let (status, _, data) = call(&app, get("/api/portal/passkeys", &cookie(&terminal))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        [
            &data["terminal"],
            &data["fresh"],
            &data["mfa"],
            &data["can_register"],
            &data["can_remove"]
        ],
        [
            &json!(true),
            &json!(true),
            &json!(true),
            &json!(false),
            &json!(false)
        ]
    );
    for (uri, body) in [
        (
            "/api/portal/passkeys/registration/start".to_owned(),
            json!({"name":"Planted"}),
        ),
        (format!("/api/portal/passkeys/{id}/remove"), json!({})),
    ] {
        let (status, _, error) = call(&app, post(&uri, &cookie(&terminal), body)).await;
        assert_eq!(
            (status, error["error"].as_str()),
            (StatusCode::FORBIDDEN, Some("reauthentication_required")),
            "{uri}"
        );
    }
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
    // Signing in here with Alice's own passkey gives this browser its own session.
    let started = f.core.portal_passkey_start(Some(&terminal), true).unwrap();
    let binding = cookie_named(&started.cookies, "riauth_passkey").unwrap();
    let proof = authenticator
        .do_authentication(
            origin(&f),
            serde_json::from_value(started.body["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let owned = sso_value(
        &f.core
            .portal_passkey_finish(
                Some(&terminal),
                Some(&binding),
                started.body["ceremony"].as_str().unwrap(),
                proof,
            )
            .unwrap()
            .cookies,
    );
    assert_ne!(
        f.sid(&owned),
        f.core.me(&cli).unwrap()["session_id"].as_str().unwrap(),
        "a browser-owned session"
    );
    assert!(!f.session(&f.sid(&owned)).revoked);
    let (_, _, data) = call(&app, get("/api/portal/passkeys", &cookie(&owned))).await;
    assert_eq!(
        [
            &data["terminal"],
            &data["can_register"],
            &data["can_remove"]
        ],
        [&json!(false), &json!(true), &json!(true)]
    );
    let (status, _, started) = call(
        &app,
        post(
            "/api/portal/passkeys/registration/start",
            &cookie(&owned),
            json!({"name":"Laptop"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{started}");
    // Finishing is refused to a terminal-shared browser as well.
    let mut second = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let response = register(&f, &mut second, &started);
    let (status, _, error) = call(
        &app,
        post(
            "/api/portal/passkeys/registration/finish",
            &cookie(&f.cookie(&cli)),
            json!({"ceremony":started["ceremony"],"credential":response}),
        ),
    )
    .await;
    assert_eq!(
        (status, error["error"].as_str()),
        (StatusCode::FORBIDDEN, Some("reauthentication_required"))
    );
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
}

#[tokio::test]
async fn portal_passkey_list_reports_freshness_and_limits() {
    let f = Fixture::new("");
    let token = f.user("alice");
    let alice_id = f.user_id("alice");
    let app = riauth::api::router(f.core.clone());
    let list = |sso: &str| {
        call(
            &app,
            get("/api/portal/passkeys", &format!("riauth_sso={sso}")),
        )
    };
    assert_eq!(list("ri_sso_unknown").await.0, StatusCode::UNAUTHORIZED);
    let bearer = Request::builder()
        .uri("/api/portal/passkeys")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        call(&app, bearer).await.0,
        StatusCode::UNAUTHORIZED,
        "only a browser session counts"
    );
    let sso = f.browser("alice", None);
    let (status, _, data) = list(&sso).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        data,
        json!({"passkeys":[],"fresh":true,"terminal":false,"mfa":false,"can_register":true,"can_remove":false,"limit":16})
    );
    f.age(&f.sid(&sso), 301);
    assert_eq!(list(&sso).await.2["fresh"], false);
    let (mut authenticator, credential) = f.enroll(&token);
    let password = f.browser("alice", None);
    let (_, _, data) = list(&password).await;
    assert_eq!(data["passkeys"].as_array().unwrap().len(), 1);
    let key = &data["passkeys"][0];
    assert_eq!(key["name"], "Test key");
    assert!(key["id"].is_string() && key["created_at"].is_u64() && !key["algorithm"].is_null());
    assert_eq!(
        [
            &data["fresh"],
            &data["mfa"],
            &data["can_register"],
            &data["can_remove"]
        ],
        [&json!(true), &json!(false), &json!(false), &json!(false)]
    );
    let passkey = f.passkey_browser(&mut authenticator, &credential, &alice_id);
    let (_, _, data) = list(&passkey).await;
    assert_eq!(
        [&data["mfa"], &data["can_register"], &data["can_remove"]],
        [&json!(true), &json!(true), &json!(true)]
    );
    // At sixteen keys nobody can enroll another.
    f.core
        .store
        .write(|tx| {
            let (id, row) = tx.list::<Value>("passkeys")?.remove(0);
            for i in 1..16 {
                let mut copy = row.clone();
                copy["id"] = json!(format!("{id}-{i}"));
                tx.put("passkeys", &format!("{id}-{i}"), &copy)?;
            }
            Ok(())
        })
        .unwrap();
    let (_, _, data) = list(&passkey).await;
    assert_eq!(data["passkeys"].as_array().unwrap().len(), 16);
    assert_eq!(
        [&data["can_register"], &data["can_remove"]],
        [&json!(false), &json!(true)]
    );
    let (status, _, _) = call(
        &app,
        post(
            "/api/portal/passkeys/registration/start",
            &format!("riauth_sso={passkey}"),
            json!({"name":"Seventeenth"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

/// The portal page gives a browser without an SSO cookie a placeholder that signs nothing
/// in, and two first sign-ins presenting it at once end on one session.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn portal_page_placeholder_makes_first_sign_ins_converge() {
    let f = Fixture::new("");
    f.create("alice");
    let app = riauth::api::router(f.core.clone());
    let page = |cookie: &str| {
        let mut request = Request::builder().uri("/apps");
        if !cookie.is_empty() {
            request = request.header("cookie", cookie);
        }
        call(&app, request.body(Body::empty()).unwrap())
    };
    let (status, cookies, _) = page("").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cookies.len(), 1, "{cookies:?}");
    assert!(
        cookies[0].ends_with("; Path=/; Max-Age=3600; HttpOnly; SameSite=Lax"),
        "{cookies:?}"
    );
    let placeholder = sso_value(&cookies);
    assert!(placeholder.starts_with("ri_sso_"));
    assert_eq!(f.signed_in_as(&placeholder), None);
    let (_, cookies, _) = page(&format!("riauth_sso={placeholder}")).await;
    assert!(cookies.is_empty(), "a browser with a cookie keeps it");
    let login = || {
        call(
            &app,
            post(
                "/api/portal/login/password",
                &format!("riauth_sso={placeholder}"),
                json!({"username":"alice","password":PASSWORD,"otp":null,"reauthenticate":false}),
            ),
        )
    };
    let ((first, a, _), (second, b, _)) = tokio::join!(login(), login());
    assert_eq!((first, second), (StatusCode::OK, StatusCode::OK));
    let (a, b) = (sso_value(&a), sso_value(&b));
    assert_ne!(a, b);
    assert_eq!(f.sid(&a), f.sid(&b), "both tabs hold the same session");
    assert_eq!(f.signed_in_as(&a).as_deref(), Some("alice"));
}

#[tokio::test]
async fn portal_sign_out_of_browser_session_does_not_touch_cli_session() {
    let f = Fixture::new("");
    let cli = f.user("alice");
    let app = riauth::api::router(f.core.clone());
    let browser = f.browser("alice", None);
    let sid = f.sid(&browser);
    // The portal signs out with no body.
    let request = Request::builder()
        .method("POST")
        .uri("/api/portal/sign-out")
        .header("origin", ORIGIN)
        .header("x-riauth-portal", "1")
        .header("cookie", format!("riauth_sso={browser}"))
        .body(Body::empty())
        .unwrap();
    let (status, cookies, body) = call(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["revoked"], true);
    assert_eq!(cookies, [CLEAR]);
    assert!(f.session(&sid).revoked);
    assert!(f.audited(&f.user_id("alice"), "session.revoke", &sid));
    assert!(f.core.me(&cli).is_ok(), "the CLI session is independent");
    // `riauth logout` in turn leaves a browser-owned session signed in.
    let browser = f.browser("alice", None);
    f.core.logout(&cli).unwrap();
    assert!(f.core.me(&cli).is_err());
    assert_eq!(f.signed_in_as(&browser).as_deref(), Some("alice"));
}

#[tokio::test]
async fn portal_sign_out_browser_scope_keeps_terminal_session() {
    let f = Fixture::new("");
    let cli = f.user("alice");
    let app = riauth::api::router(f.core.clone());
    let sign_out = |sso: &str, body: Value| {
        call(
            &app,
            post("/api/portal/sign-out", &format!("riauth_sso={sso}"), body),
        )
    };
    let terminal = f.cookie(&cli);
    let (status, cookies, body) = sign_out(&terminal, json!({"scope":"browser"})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"revoked":false}));
    assert_eq!(cookies, [CLEAR]);
    assert!(
        f.core.me(&cli).is_ok(),
        "\"Use another account\" keeps the terminal signed in"
    );
    assert_eq!(
        f.signed_in_as(&terminal),
        None,
        "this browser is signed out"
    );
    // A browser-owned session is signed out fully whatever the scope.
    let browser = f.browser("alice", None);
    let sid = f.sid(&browser);
    let (_, _, body) = sign_out(&browser, json!({"scope":"browser"})).await;
    assert_eq!(body["revoked"], true);
    assert!(f.session(&sid).revoked);
    assert!(f.core.me(&cli).is_ok());
    // Unknown scopes are refused; `{}` keeps today's full sign-out of a terminal session.
    let terminal = f.cookie(&cli);
    assert_eq!(
        sign_out(&terminal, json!({"scope":"everywhere"})).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert!(f.core.me(&cli).is_ok());
    let (_, _, body) = sign_out(&terminal, json!({})).await;
    assert_eq!(body["revoked"], true);
    assert!(f.core.me(&cli).is_err());
}

#[test]
fn portal_catalogue_reports_mfa_available() {
    let f = Fixture::new("");
    f.create("alice");
    let bob = f.user("bob");
    let carol = f.user("carol");
    let flags = |sso: &str| {
        let feed = f.core.portal_apps(Some(sso)).unwrap();
        (feed["mfa"].clone(), feed["mfa_available"].clone())
    };
    assert_eq!(
        flags(&f.browser("alice", None)),
        (json!(false), json!(false))
    );
    let enrollment = f.core.mfa_begin(&bob).unwrap();
    let totp = riauth::crypto::totp(enrollment["secret"].as_str().unwrap(), "bob").unwrap();
    f.core
        .mfa_confirm(&bob, &totp.generate(now() - 30))
        .unwrap();
    let reply = f
        .core
        .portal_password(
            None,
            "bob".into(),
            PASSWORD.into(),
            Some(totp.generate(now())),
            false,
        )
        .unwrap();
    assert_eq!(
        flags(&sso_value(&reply.cookies)),
        (json!(true), json!(true))
    );
    // A passkey user signed in with a password learns that MFA is available.
    f.enroll(&carol);
    assert_eq!(
        flags(&f.browser("carol", None)),
        (json!(false), json!(true))
    );
}

#[tokio::test]
async fn https_issuer_uses_host_prefixed_sso_and_ignores_duplicates_and_tossed_names() {
    let f = Fixture::with_issuer("https://auth.example.test/identity");
    f.create("alice");
    f.create("mallory");
    let app = riauth::api::router(f.core.clone());
    let https = |uri: &str, cookie: &str, body: Value| {
        let mut request = post(uri, cookie, body);
        request
            .headers_mut()
            .insert("origin", "https://auth.example.test".parse().unwrap());
        request
    };
    let login = |name: &str| json!({"username":name,"password":PASSWORD});
    let legacy = "riauth_sso=; Path=/identity/; Max-Age=0; HttpOnly; SameSite=Lax; Secure";
    assert_eq!(
        call(
            &app,
            post("/identity/api/portal/login/password", "", login("alice"))
        )
        .await
        .0,
        StatusCode::FORBIDDEN,
        "the http origin is not the issuer's"
    );
    let (status, cookies, _) = call(
        &app,
        https("/identity/api/portal/login/password", "", login("mallory")),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cookies.len(), 2, "{cookies:?}");
    assert!(
        cookies[0].starts_with("__Host-riauth_sso=ri_sso_"),
        "{cookies:?}"
    );
    for part in ["; Path=/;", "; HttpOnly", "; SameSite=None", "; Secure"] {
        assert!(cookies[0].contains(part), "{cookies:?}");
    }
    assert!(!cookies[0].contains("Domain"));
    assert_eq!(cookies[1], legacy);
    let mallory = cookie_named(&cookies, "__Host-riauth_sso").unwrap();
    let mallory_sid = f.sid(&mallory);
    // A sibling subdomain can plant only the unprefixed name, which is never presented:
    // alice's sign-in neither joins nor revokes the planted session.
    let (status, cookies, _) = call(
        &app,
        https(
            "/identity/api/portal/login/password",
            &format!("riauth_sso={mallory}"),
            login("alice"),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let alice = cookie_named(&cookies, "__Host-riauth_sso").unwrap();
    assert_ne!(f.sid(&alice), mallory_sid);
    assert!(!f.session(&mallory_sid).revoked);
    for (cookie, expected) in [
        (format!("__Host-riauth_sso={alice}"), Some("alice")),
        (
            format!("__Host-riauth_sso={alice}; riauth_sso={mallory}"),
            Some("alice"),
        ),
        (format!("riauth_sso={alice}"), None),
        // A duplicate is a denial of service only.
        (
            format!("__Host-riauth_sso={mallory}; __Host-riauth_sso={alice}"),
            None,
        ),
    ] {
        let (status, _, feed) = call(&app, get("/identity/api/portal", &cookie)).await;
        assert_eq!(feed["user"]["username"].as_str(), expected, "{cookie}");
        if expected.is_none() {
            assert_eq!(status, StatusCode::UNAUTHORIZED, "{cookie}");
        }
    }
    let (_, cookies, _) = call(
        &app,
        https("/identity/api/portal/login/passkey/start", "", json!({})),
    )
    .await;
    assert!(
        cookies[0].starts_with("riauth_passkey=ri_passkey_bind_")
            && cookies[0].ends_with("; Secure"),
        "{cookies:?}"
    );
    let (status, cookies, _) = call(
        &app,
        https(
            "/identity/api/portal/sign-out",
            &format!("__Host-riauth_sso={alice}"),
            json!({}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        cookies,
        [
            "__Host-riauth_sso=; Path=/; Max-Age=0; HttpOnly; SameSite=None; Secure",
            legacy
        ]
    );
    assert_eq!(f.signed_in_as(&alice), None);
}

#[tokio::test]
async fn portal_page_and_auth_asset_stay_offline() {
    let f = Fixture::new("/identity");
    let app = riauth::api::router(f.core.clone());
    let fetch = |uri: &str| {
        let request = Request::builder()
            .uri(uri)
            .header("accept", "text/html")
            .body(Body::empty())
            .unwrap();
        let app = app.clone();
        async move {
            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let headers = response.headers().clone();
            let body = response.into_body().collect().await.unwrap().to_bytes();
            (headers, String::from_utf8(body.to_vec()).unwrap())
        }
    };
    let (headers, page) = fetch("/identity/apps").await;
    let policy = headers["content-security-policy"].to_str().unwrap();
    for directive in [
        "default-src 'none'",
        "script-src 'self'",
        "connect-src 'self'",
        "frame-ancestors 'none'",
    ] {
        assert!(policy.contains(directive), "{policy}");
    }
    assert!(!policy.contains("http") && !policy.contains("unsafe"));
    assert_eq!(headers["cross-origin-opener-policy"], "same-origin");
    assert_eq!(headers["x-frame-options"], "DENY");
    assert_eq!(headers["referrer-policy"], "no-referrer");
    assert!(
        headers["permissions-policy"]
            .to_str()
            .unwrap()
            .contains("publickey-credentials-get=(self)")
    );
    let auth = page
        .find(r#"<script src="/identity/portal/assets/auth.js" defer></script>"#)
        .unwrap();
    let script = page
        .find(r#"<script src="/identity/portal/assets/app.js" defer></script>"#)
        .unwrap();
    assert!(auth < script, "auth.js loads before app.js");
    assert_eq!(page.matches("<script").count(), 2, "no inline script");
    assert!(page.contains(r#"src="/identity/portal/assets/riauth-mark.svg""#));
    for id in [
        "auth-error",
        "passkey-login",
        "auth-divider",
        "password-form",
        "login-username",
        "login-password",
        "login-otp",
        "login-otp-hint",
        "password-login",
        "start-login",
        "login-request",
        "account-security",
        "sign-out",
        "mfa-notice",
        "security-dialog",
        "security-title",
        "passkey-list",
        "security-status",
        "reauth-panel",
        "reauth-passkey",
        "reauth-form",
        "reauth-password",
        "reauth-otp",
        "reauth-error",
        "passkey-form",
        "passkey-name",
        "add-passkey",
        "security-close",
    ] {
        assert!(page.contains(&format!("id=\"{id}\"")), "{id}");
    }
    assert!(page.contains(r#"<button class="button secondary" id="start-login""#));
    assert!(page.contains("Sign in with your terminal"));
    assert_eq!(page.matches("http://").count(), 1, "only the SVG namespace");
    for forbidden in ["/events", "event-map", "map.js", "https://", "autofocus"] {
        assert!(!page.contains(forbidden), "{forbidden}");
    }
    let (headers, auth) = fetch("/identity/portal/assets/auth.js").await;
    assert!(
        headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/javascript")
    );
    for needle in [
        "window.RiAuth = Object.freeze(",
        r#""X-Riauth-Portal": "1""#,
        r#"mode: "same-origin""#,
        r#"redirect: "error""#,
        "isSecureContext",
    ] {
        assert!(auth.contains(needle), "{needle}");
    }
    for forbidden in [
        "http://",
        "https://",
        "signalUnknownCredential",
        "toJSON",
        "FromJSON",
        "document.cookie",
        "localStorage",
        "innerHTML",
    ] {
        assert!(!auth.contains(forbidden), "{forbidden} in auth.js");
    }
    let (_, script) = fetch("/identity/portal/assets/app.js").await;
    for forbidden in [
        "https://",
        "signalUnknownCredential",
        "document.cookie",
        "innerHTML",
    ] {
        assert!(!script.contains(forbidden), "{forbidden} in app.js");
    }
    let (_, css) = fetch("/identity/portal/assets/app.css").await;
    let (headers, logo) = fetch("/identity/portal/assets/riauth-mark.svg").await;
    assert!(
        headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("image/svg+xml")
    );
    assert!(logo.starts_with("<svg "));
    assert!(logo.contains("<path "));
    assert!(!css.contains("url(") && !css.contains("http"));
    for class in [
        ".field{",
        ".field-hint{",
        ".form-error{",
        ".notice{",
        ".divider{",
        ".consent-list{",
        ".account-line{",
        ".checkbox{",
        "details.terminal{",
        ".passkey-list{",
        ".passkey-row{",
        "dialog.security{",
        "dialog::backdrop{",
        ".spinner{",
        ".signin-shell{",
        "[aria-busy=true]",
        "summary:focus-visible",
        "min-height:44px",
    ] {
        assert!(css.contains(class), "{class}");
    }
    let reduced = css.find("@media(prefers-reduced-motion:reduce)").unwrap();
    assert!(css[reduced..].contains(".spinner"));
    for body in [&page, &auth, &script, &css] {
        let lower = body.to_ascii_lowercase();
        for host in ["googleapis", "gstatic", "unpkg.com", "jsdelivr", "cdnjs"] {
            assert!(!lower.contains(host), "{host}");
        }
    }
}
