//! OIDC browser sign-in through the interaction page's HTTP endpoints (§3.3, §4.3-§4.8),
//! with terminal approval and relying-party sign-out alongside.
mod common;

use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{HeaderMap, Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use common::{Fixture, PASSWORD, strings, text};
use http_body_util::BodyExt;
use riauth::{
    browser::BrowserDecision,
    crypto::{self, digest, now},
    model::*,
    oidc::{Authorization, TokenRequest},
    signin,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, time::Instant};
use tower::ServiceExt;
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

const ORIGIN: &str = "http://localhost:9000";
const REDIRECT: &str = "http://localhost:7777/callback?existing=1";
const SIGNED_OUT: &str = "https://app.example.test/signed-out";
const MISSING: &str = "00000000-0000-4000-8000-000000000000";

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Value,
    text: String,
}
impl Reply {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(|v| v.to_str().unwrap())
    }
    fn set_cookie(&self, name: &str) -> Option<String> {
        self.headers
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap())
            .find(|c| c.starts_with(&format!("{name}=")))
            .map(str::to_owned)
    }
    /// The value this response sets for `name`; empty when it clears the cookie.
    fn cookie(&self, name: &str) -> Option<String> {
        self.set_cookie(name)
            .map(|c| c.split(';').next().unwrap()[name.len() + 1..].to_owned())
    }
    fn sso(&self) -> String {
        self.cookie("riauth_sso")
            .filter(|v| !v.is_empty())
            .expect("SSO cookie")
    }
    fn location(&self) -> String {
        self.header("location").expect("Location").to_owned()
    }
}
async fn call(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        headers,
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        text: String::from_utf8_lossy(&bytes).into_owned(),
    }
}
fn jar(cookies: &[(&str, &str)]) -> String {
    cookies
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("; ")
}
fn get(uri: &str, accept: &str, cookies: &[(&str, &str)]) -> Request<Body> {
    let mut request = Request::get(uri).header("accept", accept);
    if !cookies.is_empty() {
        request = request.header("cookie", jar(cookies));
    }
    request.body(Body::empty()).unwrap()
}
/// A same-origin page write, as `auth.js` sends it.
fn post(uri: &str, cookies: &[(&str, &str)], body: Value) -> Request<Body> {
    let mut request = Request::post(uri)
        .header("origin", ORIGIN)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .header("content-type", "application/json")
        .header("accept", "application/json");
    if !cookies.is_empty() {
        request = request.header("cookie", jar(cookies));
    }
    request.body(Body::from(body.to_string())).unwrap()
}
fn query(location: &str) -> BTreeMap<String, String> {
    url::Url::parse(location)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect()
}
fn authorize_uri(request: &Authorization) -> String {
    let mut request = request.clone();
    request.decision = None;
    format!(
        "/oauth/authorize?{}",
        serde_urlencoded::to_string(&request).unwrap()
    )
}
fn client(
    f: &Fixture,
    id: &str,
    confidential: bool,
    require_mfa: bool,
    settings: ProviderSettings,
) -> Option<String> {
    let created = f
        .core
        .create_client(
            &f.admin,
            NewClient {
                client_id: id.into(),
                name: format!("{id} app"),
                confidential,
                redirect_uris: vec![REDIRECT.into()],
                scopes: strings(&["openid", "profile", "email", "groups", "offline_access"]),
                allowed_groups: Default::default(),
                require_mfa,
                service: false,
                settings: ProviderSettings {
                    post_logout_redirect_uris: vec![SIGNED_OUT.into()],
                    ..settings
                },
            },
        )
        .unwrap();
    created["client_secret"].as_str().map(String::from)
}

struct Interaction {
    id: String,
    binding: String,
    code: String,
}
impl Interaction {
    fn path(&self, rest: &str) -> String {
        format!("/oauth/resume/{}{rest}", self.id)
    }
    fn cookies<'a>(&'a self, sso: Option<&'a str>) -> Vec<(&'a str, &'a str)> {
        let mut cookies = vec![("riauth_return", self.binding.as_str())];
        cookies.extend(sso.map(|sso| ("riauth_sso", sso)));
        cookies
    }
}
/// A browser navigation to /oauth/authorize that needs interaction.
async fn start(
    f: &Fixture,
    app: &Router,
    request: &Authorization,
    sso: Option<&str>,
) -> Interaction {
    let cookies: Vec<_> = sso.map(|sso| ("riauth_sso", sso)).into_iter().collect();
    let reply = call(app, get(&authorize_uri(request), "text/html", &cookies)).await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text);
    let location = reply.location();
    let id = location
        .strip_prefix(&format!("{ORIGIN}/oauth/resume/"))
        .unwrap()
        .to_owned();
    let pending: Value = f
        .core
        .store
        .get("browser_authorizations", &id)
        .unwrap()
        .unwrap();
    Interaction {
        binding: reply.cookie("riauth_return").unwrap(),
        code: text(&pending, "code"),
        id,
    }
}
async fn get_state(app: &Router, i: &Interaction, sso: Option<&str>) -> Reply {
    call(
        app,
        get(&i.path("/state"), "application/json", &i.cookies(sso)),
    )
    .await
}
async fn password(
    app: &Router,
    i: &Interaction,
    sso: Option<&str>,
    username: &str,
    otp: Option<&str>,
) -> Reply {
    let body = json!({"username": username, "password": PASSWORD, "otp": otp});
    call(app, post(&i.path("/password"), &i.cookies(sso), body)).await
}
async fn decide(
    app: &Router,
    i: &Interaction,
    sso: Option<&str>,
    approve: bool,
    state: &Value,
) -> Reply {
    let body = json!({"approve": approve, "remember": true, "session_ref": state["session_ref"]});
    call(app, post(&i.path("/decision"), &i.cookies(sso), body)).await
}
async fn resume(app: &Router, i: &Interaction, sso: Option<&str>) -> Reply {
    call(app, get(&i.path(""), "text/html", &i.cookies(sso))).await
}
fn redeem(f: &Fixture, cid: &str, location: &str, verifier: &str, secret: Option<String>) -> Value {
    let params = query(location);
    assert_eq!(params["state"], "state with & delimiters");
    assert_eq!(params["iss"], f.core.config.issuer);
    f.core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some(cid.into()),
            client_secret: secret,
            code: Some(params["code"].clone()),
            redirect_uri: Some(REDIRECT.into()),
            code_verifier: Some(verifier.into()),
            ..Default::default()
        })
        .unwrap()
}
fn id_token(f: &Fixture, tokens: &Value, cid: &str) -> Value {
    let jwks: riauth::jose::PublicJwks = serde_json::from_value(f.core.jwks().unwrap()).unwrap();
    jwks.verify(&text(tokens, "id_token"), &f.core.config.issuer, cid)
        .unwrap()
}
/// The code the callback carries, as the grant it will redeem.
fn grant(f: &Fixture, location: &str) -> Code {
    f.core
        .store
        .get("codes", &digest(&query(location)["code"]))
        .unwrap()
        .unwrap()
}
fn sid(f: &Fixture, sso: &str) -> String {
    let mapping: Value = f
        .core
        .store
        .get("browser_sessions", &digest(sso))
        .unwrap()
        .unwrap();
    text(&mapping, "session_id")
}
fn session(f: &Fixture, sid: &str) -> Session {
    f.core.store.get("sessions", sid).unwrap().unwrap()
}
fn user_id(f: &Fixture, username: &str) -> String {
    f.core.store.get("usernames", username).unwrap().unwrap()
}
/// A browser-owned session from the portal password form; returns its SSO value.
fn browser(f: &Fixture, username: &str) -> String {
    let reply = f
        .core
        .portal_password(None, username.into(), PASSWORD.into(), None, false)
        .unwrap();
    reply
        .cookies
        .iter()
        .find_map(|c| c.split(';').next().unwrap().strip_prefix("riauth_sso="))
        .unwrap()
        .to_owned()
}
fn age(f: &Fixture, sid: &str, seconds: u64) {
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", sid)?.unwrap();
            session.identity.auth_time = session.identity.auth_time.saturating_sub(seconds);
            tx.put("sessions", sid, &session)
        })
        .unwrap();
}
fn audited(f: &Fixture, actor: &str, action: &str, target: &str) -> bool {
    f.core
        .audit_events(&f.admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["actor"] == actor && e["action"] == action && e["target"] == target)
}
fn approve_in_terminal(f: &Fixture, token: &str, code: &str) -> riauth::error::Result<Value> {
    let details = f.core.browser_details(token, code)?;
    f.core.browser_decide(
        token,
        BrowserDecision {
            code: code.into(),
            approve: true,
            transaction_id: details["transaction_id"].as_str().map(String::from),
            remember: false,
        },
    )
}
fn setup(f: &Fixture) -> Router {
    client(f, "app", false, false, Default::default());
    riauth::api::router(f.core.clone())
}
type Authenticator = WebauthnAuthenticator<SoftPasskey>;
fn origin() -> url::Url {
    url::Url::parse(ORIGIN).unwrap()
}
/// Enrolls a SoftPasskey from a bearer session; returns it with its credential id.
fn enroll(f: &Fixture, token: &str) -> (Authenticator, String) {
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let started = f
        .core
        .passkey_register_start(token, "Test key".into())
        .unwrap();
    let mut options = started["public_key"].clone();
    options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
    let response = authenticator
        .do_registration(origin(), serde_json::from_value(options).unwrap())
        .unwrap();
    let credential = text(&serde_json::to_value(&response).unwrap(), "rawId");
    f.core
        .passkey_register_finish(token, &text(&started, "ceremony"), response)
        .unwrap();
    (authenticator, credential)
}

#[tokio::test]
async fn html_authorize_redirects_to_resume_page_with_binding_cookie() {
    let f = Fixture::new();
    let app = setup(&f);
    let request = f.request("app", &crypto::random_token(""));
    let reply = call(&app, get(&authorize_uri(&request), "text/html", &[])).await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER);
    assert!(reply.text.is_empty());
    assert_eq!(reply.header("vary"), Some("Accept"));
    assert_eq!(reply.header("referrer-policy"), Some("no-referrer"));
    assert!(reply.header("refresh").is_none());
    let location = reply.location();
    let id = location
        .strip_prefix("http://localhost:9000/oauth/resume/")
        .expect("absolute Location under the issuer origin");
    assert!(uuid::Uuid::parse_str(id).is_ok());
    let binding = reply.set_cookie("riauth_return").unwrap();
    assert!(binding.contains(&format!("Path=/oauth/resume/{id};")));
    assert!(binding.contains("HttpOnly") && binding.contains("SameSite=Lax"));
    assert!(binding.contains("Max-Age=600"));
    let pending: Value = f
        .core
        .store
        .get("browser_authorizations", id)
        .unwrap()
        .unwrap();
    assert!(pending["session_cookie"].is_null());
    assert_eq!(
        pending["browser_hash"],
        digest(&reply.cookie("riauth_return").unwrap())
    );
    // API clients still receive the terminal details as JSON.
    let json = call(&app, get(&authorize_uri(&request), "application/json", &[])).await;
    assert_eq!(json.status, StatusCode::OK);
    assert_eq!(json.header("vary"), Some("Accept"));
    assert_eq!(json.body["client_id"], "app");
    assert!(json.body["transaction_id"].is_string());
    // A path-prefixed issuer keeps its prefix in the handoff.
    let dir = tempfile::TempDir::new().unwrap();
    let core = riauth::core::Core::initialize(
        riauth::config::Config {
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
    let admin = text(
        &core.login("admin".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    core.create_client(
        &admin,
        NewClient {
            client_id: "app".into(),
            name: "app".into(),
            confidential: false,
            redirect_uris: vec![REDIRECT.into()],
            scopes: strings(&["openid", "profile", "email", "groups", "offline_access"]),
            allowed_groups: Default::default(),
            require_mfa: false,
            service: false,
            settings: Default::default(),
        },
    )
    .unwrap();
    let prefixed = riauth::api::router(core);
    let uri = authorize_uri(&request).replacen("/oauth/", "/identity/oauth/", 1);
    let reply = call(&prefixed, get(&uri, "text/html", &[])).await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text);
    let id = reply
        .location()
        .strip_prefix("http://localhost:9000/identity/oauth/resume/")
        .unwrap()
        .to_owned();
    assert!(
        reply
            .set_cookie("riauth_return")
            .unwrap()
            .contains(&format!("Path=/identity/oauth/resume/{id};"))
    );
}

#[tokio::test]
async fn resume_page_has_strict_headers_no_refresh_and_no_coop() {
    let f = Fixture::new();
    let app = setup(&f);
    let i = start(&f, &app, &f.request("app", &crypto::random_token("")), None).await;
    let page = resume(&app, &i, None).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.header("content-type")
            .unwrap()
            .starts_with("text/html")
    );
    let csp = page.header("content-security-policy").unwrap();
    for directive in [
        "default-src 'none'",
        "script-src 'self'",
        "frame-ancestors 'none'",
        "form-action 'none'",
        "base-uri 'none'",
    ] {
        assert!(csp.contains(directive), "{csp}");
    }
    assert!(!csp.contains("unsafe"), "{csp}");
    assert_eq!(page.header("x-frame-options"), Some("DENY"));
    assert_eq!(page.header("referrer-policy"), Some("no-referrer"));
    assert_eq!(page.header("vary"), Some("Accept"));
    assert!(
        page.header("permissions-policy")
            .unwrap()
            .contains("publickey-credentials-get=(self)")
    );
    assert!(
        page.header("refresh").is_none(),
        "HTML pages never carry Refresh"
    );
    assert!(
        page.header("cross-origin-opener-policy").is_none(),
        "popups keep their opener"
    );
    // A browser without an SSO cookie gets a placeholder that signs nothing in; its first
    // sign-ins present it, so concurrent ones from two tabs end on one session.
    let placeholder = page.cookie("riauth_sso").unwrap();
    assert!(placeholder.starts_with("ri_sso_"), "{placeholder}");
    assert!(
        page.set_cookie("riauth_sso")
            .unwrap()
            .ends_with("; Max-Age=3600; HttpOnly; SameSite=Lax")
    );
    assert!(f.core.portal_apps(Some(&placeholder)).is_err());
    assert!(
        resume(&app, &i, Some(&placeholder))
            .await
            .set_cookie("riauth_sso")
            .is_none(),
        "a browser with an SSO cookie keeps it"
    );
    let html = &page.text;
    assert!(html.contains("<script src=\"/portal/assets/auth.js\" defer></script>"));
    assert!(html.contains("<script src=\"/portal/assets/signin.js\" defer></script>"));
    assert_eq!(html.matches("<script").count(), 2, "no inline script");
    assert!(!html.contains("style="), "no inline style");
    for placeholder in ["__BASE__", "__CODE__", "__COMMAND__"] {
        assert!(!html.contains(placeholder));
    }
    assert!(html.contains(&format!("<code>riauth request approve {}</code>", i.code)));
    // The only refresh reloads the page when JavaScript is off.
    assert_eq!(html.matches("http-equiv=\"refresh\"").count(), 1);
    assert!(html.contains("<noscript><meta http-equiv=\"refresh\" content=\"5\"></noscript>"));
    for id in [
        "signin-loading",
        "signin-authenticate",
        "signin-consent",
        "signin-logout",
        "signin-message",
        "signin-expiry",
        "signin-announcement",
        "signin-terminal",
    ] {
        assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
    }
    let script = call(&app, get("/portal/assets/signin.js", "*/*", &[])).await;
    assert_eq!(script.status, StatusCode::OK);
    assert_eq!(
        script.header("content-type"),
        Some("text/javascript; charset=utf-8")
    );
    for forbidden in [
        "innerHTML",
        "outerHTML",
        "insertAdjacentHTML",
        "document.write",
        "eval(",
        "signalUnknownCredential",
        "http:",
        "https:",
        "localStorage",
    ] {
        assert!(!script.text.contains(forbidden), "{forbidden}");
    }
    assert!(script.text.contains("location.replace"));
}

#[tokio::test]
async fn json_resume_keeps_terminal_contract() {
    let f = Fixture::new();
    let app = setup(&f);
    let i = start(&f, &app, &f.request("app", &crypto::random_token("")), None).await;
    let waiting = call(&app, get(&i.path(""), "application/json", &i.cookies(None))).await;
    assert_eq!(waiting.status, StatusCode::OK);
    assert_eq!(waiting.header("vary"), Some("Accept"));
    assert_eq!(
        waiting.header("refresh").unwrap(),
        format!("2; url=/oauth/resume/{}", i.id)
    );
    assert_eq!(waiting.body["status"], "authorization_pending");
    assert_eq!(waiting.body["user_code"], i.code);
    assert_eq!(
        waiting.body["resume_uri"],
        format!("/oauth/resume/{}", i.id)
    );
    assert_eq!(
        waiting.body["instruction"],
        format!(
            "Sign in in this browser, or run riauth request approve {} in your terminal.",
            i.code
        )
    );
    approve_in_terminal(&f, &f.admin, &i.code).unwrap();
    let delivered = call(&app, get(&i.path(""), "application/json", &i.cookies(None))).await;
    assert_eq!(delivered.status, StatusCode::FOUND);
    assert!(
        delivered
            .location()
            .starts_with("http://localhost:7777/callback?existing=1&")
    );
    let sso = delivered.sso();
    assert_eq!(
        sid(&f, &sso),
        text(&f.core.me(&f.admin).unwrap(), "session_id")
    );
    let again = call(&app, get(&i.path(""), "application/json", &i.cookies(None))).await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    assert_eq!(again.body["error"], "not_found");
}

#[tokio::test]
async fn interaction_state_requires_binding_cookie_and_reports_effective_expiry() {
    let f = Fixture::new();
    let app = setup(&f);
    let before = now();
    let i = start(&f, &app, &f.request("app", &crypto::random_token("")), None).await;
    for cookies in [vec![], vec![("riauth_return", "another-browser")]] {
        let refused = call(&app, get(&i.path("/state"), "application/json", &cookies)).await;
        assert_eq!(refused.status, StatusCode::UNAUTHORIZED);
        assert_eq!(refused.body["error"], "invalid_token");
    }
    let missing = call(
        &app,
        get(
            &format!("/oauth/resume/{MISSING}/state"),
            "application/json",
            &i.cookies(None),
        ),
    )
    .await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);
    assert_eq!(missing.body["error"], "interaction_expired");
    let state = get_state(&app, &i, None).await.body;
    assert_eq!(state["kind"], "authorize");
    assert_eq!(state["status"], "authenticate");
    assert_eq!(state["reason"], "sign_in");
    let expires = state["expires_at"].as_u64().unwrap();
    assert!((before + 600..=now() + 600).contains(&expires));
    assert_eq!(
        state["application"],
        json!({"client_id": "app", "name": "app app", "host": "localhost"})
    );
    assert_eq!(
        state["requirements"],
        json!({"mfa": false, "browser": true})
    );
    assert_eq!(state["consent"]["required"], true);
    assert_eq!(
        state["consent"]["scopes"],
        json!(["email", "groups", "offline_access", "openid", "profile"])
    );
    assert_eq!(state["consent"]["remember_default"], true);
    assert_eq!(
        state["terminal"],
        json!({"user_code": i.code, "issuer": ORIGIN})
    );
    assert_eq!(state["pinned"], false);
    for key in [
        "account",
        "session_ref",
        "continue",
        "error",
        "message",
        "logout",
    ] {
        assert!(state[key].is_null(), "{key}");
    }
    // A signed request object that ends sooner shortens the effective deadline.
    f.core
        .store
        .write(|tx| {
            let mut pending: Value = tx.get("browser_authorizations", &i.id)?.unwrap();
            pending["expires_at"] = json!(now() + 42);
            tx.put("browser_authorizations", &i.id, &pending)
        })
        .unwrap();
    let state = get_state(&app, &i, None).await.body;
    assert!(state["expires_at"].as_u64().unwrap() <= now() + 42);
}

#[tokio::test]
async fn interaction_writes_require_origin_header_fetch_site_and_binding() {
    let f = Fixture::new();
    let app = setup(&f);
    let credential = json!({"id": "AAAA", "rawId": "AAAA", "type": "public-key", "response": {"authenticatorData": "AAAA", "clientDataJSON": "AAAA", "signature": "AAAA", "userHandle": null}, "clientExtensionResults": {}});
    let mut writes = Vec::new();
    for base in [
        format!("/oauth/resume/{MISSING}"),
        format!("/saml/resume/{MISSING}"),
    ] {
        writes.push((
            format!("{base}/password"),
            json!({"username": "alice", "password": PASSWORD, "otp": null}),
        ));
        writes.push((format!("{base}/passkey/start"), json!({})));
        writes.push((
            format!("{base}/passkey/finish"),
            json!({"ceremony": "c", "credential": credential}),
        ));
        writes.push((
            format!("{base}/decision"),
            json!({"approve": false, "remember": false, "session_ref": null}),
        ));
    }
    writes.push((
        format!("/oauth/logout/resume/{MISSING}/decision"),
        json!({"approve": false}),
    ));
    for (uri, body) in &writes {
        let strip = |header: &str| {
            let mut request = post(uri, &[], body.clone());
            request.headers_mut().remove(header);
            request
        };
        let mut cross_site = post(uri, &[], body.clone());
        cross_site
            .headers_mut()
            .insert("sec-fetch-site", "same-site".parse().unwrap());
        let mut foreign = post(uri, &[], body.clone());
        foreign
            .headers_mut()
            .insert("origin", "https://attacker.example".parse().unwrap());
        for request in [
            strip("origin"),
            strip("x-riauth-portal"),
            cross_site,
            foreign,
        ] {
            let refused = call(&app, request).await;
            assert_eq!(refused.status, StatusCode::FORBIDDEN, "{uri}");
            assert_eq!(refused.body["error"], "access_denied", "{uri}");
        }
        let mut form = post(uri, &[], body.clone());
        form.headers_mut()
            .insert("content-type", "text/plain".parse().unwrap());
        assert_eq!(
            call(&app, form).await.status,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "{uri}"
        );
        let mut extra = body.clone();
        extra["unexpected"] = json!(true);
        assert_eq!(
            call(&app, post(uri, &[], extra)).await.status.as_u16(),
            422,
            "{uri}"
        );
        // Past the guard, an unknown interaction is simply gone.
        let passed = call(&app, post(uri, &[], body.clone())).await;
        assert_eq!(passed.status, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(passed.body["error"], "interaction_expired", "{uri}");
    }
    // A live interaction still needs this browser's binding cookie.
    let i = start(&f, &app, &f.request("app", &crypto::random_token("")), None).await;
    f.user("alice");
    let stranger = Interaction {
        id: i.id.clone(),
        binding: "another-browser".into(),
        code: i.code.clone(),
    };
    for reply in [
        password(&app, &stranger, None, "alice", None).await,
        call(
            &app,
            post(
                &i.path("/passkey/start"),
                &stranger.cookies(None),
                json!({}),
            ),
        )
        .await,
        decide(&app, &stranger, None, false, &Value::Null).await,
    ] {
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
        assert_eq!(reply.body["error"], "invalid_token");
    }
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );
    assert!(get_state(&app, &i, None).await.body["status"] == "authenticate");
}

#[tokio::test]
async fn password_sign_in_then_consent_issues_code_once() {
    let f = Fixture::new();
    let app = setup(&f);
    let cli = f.user("alice");
    let verifier = crypto::random_token("");
    let started = now();
    let i = start(&f, &app, &f.request("app", &verifier), None).await;
    let signed_in = password(&app, &i, None, "alice", None).await;
    assert_eq!(signed_in.status, StatusCode::OK, "{}", signed_in.text);
    let state = signed_in.body.clone();
    assert_eq!(state["status"], "consent");
    assert_eq!(
        state["account"],
        json!({"username": "alice", "display_name": "Test User", "mfa": false})
    );
    assert_eq!(state["pinned"], true);
    let cookie = signed_in.set_cookie("riauth_sso").unwrap();
    assert!(
        cookie.contains("Path=/;")
            && cookie.contains("HttpOnly")
            && cookie.contains("SameSite=Lax")
    );
    for secret in ["ri_sso_", "ri_session_", "ri_auth_", "session_token"] {
        assert!(!signed_in.text.contains(secret), "{secret}");
    }
    let sso = signed_in.sso();
    let holder = sid(&f, &sso);
    assert_eq!(state["session_ref"], signin::session_ref(&i.id, &holder));
    let owned = session(&f, &holder);
    assert!(
        !f.core
            .store
            .read(|tx| signin::bearer_backed(tx, &owned))
            .unwrap()
    );
    assert!(f.core.me(&cli).is_ok(), "the terminal session is untouched");
    let pending: Value = f
        .core
        .store
        .get("browser_authorizations", &i.id)
        .unwrap()
        .unwrap();
    let proof = text(&pending, "authentication");
    assert!(pending["session_cookie"].is_null());
    let done = decide(&app, &i, Some(&sso), true, &state).await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.text);
    assert_eq!(done.body["status"], "complete");
    assert_eq!(done.body["continue"], format!("/oauth/resume/{}", i.id));
    assert!(
        f.core
            .store
            .get::<Value>("authentication", &proof)
            .unwrap()
            .is_none()
    );
    let delivered = resume(&app, &i, Some(&sso)).await;
    assert_eq!(delivered.status, StatusCode::FOUND);
    assert!(
        delivered.set_cookie("riauth_sso").is_none(),
        "the browser already holds its session"
    );
    let location = delivered.location();
    assert_eq!(grant(&f, &location).identity.session_id, holder);
    // Delivery is one-shot: the row and its code index are gone.
    assert!(
        f.core
            .store
            .get::<Value>("browser_authorizations", &i.id)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .get::<String>(
                "authorization_codes",
                &digest(&crypto::normalize_code(&i.code).unwrap())
            )
            .unwrap()
            .is_none()
    );
    let again = call(
        &app,
        get(&i.path(""), "application/json", &i.cookies(Some(&sso))),
    )
    .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    let tokens = redeem(&f, "app", &location, &verifier, None);
    let claims = id_token(&f, &tokens, "app");
    assert_eq!(claims["amr"], json!(["pwd"]));
    assert_eq!(claims["acr"], riauth::assurance::PASSWORD);
    assert!((started..=now()).contains(&claims["auth_time"].as_u64().unwrap()));
    assert!(claims["sid"].is_string());
    assert!(audited(
        &f,
        &user_id(&f, "alice"),
        "browser.authorization.decided",
        "app"
    ));
    // The remembered consent returns this browser without a page next time.
    let verifier = crypto::random_token("");
    let silent = call(
        &app,
        get(
            &authorize_uri(&f.request("app", &verifier)),
            "text/html",
            &[("riauth_sso", &sso)],
        ),
    )
    .await;
    assert_eq!(silent.status, StatusCode::FOUND, "{}", silent.text);
    assert_eq!(silent.header("vary"), Some("Accept"));
    redeem(&f, "app", &silent.location(), &verifier, None);
}

#[tokio::test]
async fn implicit_consent_auto_continues_after_sign_in() {
    let f = Fixture::new();
    let secret = client(
        &f,
        "first",
        true,
        false,
        ProviderSettings {
            implicit_consent: true,
            ..Default::default()
        },
    );
    let app = riauth::api::router(f.core.clone());
    f.user("alice");
    let verifier = crypto::random_token("");
    let i = start(&f, &app, &f.request("first", &verifier), None).await;
    let before = get_state(&app, &i, None).await.body;
    assert_eq!(before["consent"]["required"], false);
    let signed_in = password(&app, &i, None, "alice", None).await;
    assert_eq!(signed_in.body["status"], "complete", "{}", signed_in.text);
    assert_eq!(
        signed_in.body["continue"],
        format!("/oauth/resume/{}", i.id)
    );
    let sso = signed_in.sso();
    let pending: Value = f
        .core
        .store
        .get("browser_authorizations", &i.id)
        .unwrap()
        .unwrap();
    assert!(pending["authentication"].is_null(), "the proof was spent");
    assert!(pending["session_id"].is_null(), "delivery points nothing");
    let delivered = resume(&app, &i, Some(&sso)).await;
    assert_eq!(delivered.status, StatusCode::FOUND);
    redeem(&f, "first", &delivered.location(), &verifier, secret);
    assert!(f.core.store.list::<Value>("consents").unwrap().is_empty());
    // The next request from this browser is answered at once.
    let silent = call(
        &app,
        get(
            &authorize_uri(&f.request("first", &verifier)),
            "text/html",
            &[("riauth_sso", &sso)],
        ),
    )
    .await;
    assert_eq!(silent.status, StatusCode::FOUND);
}

#[tokio::test]
async fn prompt_consent_overrides_implicit_consent() {
    let f = Fixture::new();
    let secret = client(
        &f,
        "first",
        true,
        false,
        ProviderSettings {
            implicit_consent: true,
            ..Default::default()
        },
    );
    let app = riauth::api::router(f.core.clone());
    f.user("alice");
    let verifier = crypto::random_token("");
    let mut request = f.request("first", &verifier);
    request.prompt = Some("consent".into());
    let i = start(&f, &app, &request, None).await;
    assert_eq!(
        get_state(&app, &i, None).await.body["consent"]["required"],
        true
    );
    let signed_in = password(&app, &i, None, "alice", None).await;
    assert_eq!(signed_in.body["status"], "consent");
    assert_eq!(signed_in.body["consent"]["required"], true);
    let sso = signed_in.sso();
    let done = decide(&app, &i, Some(&sso), true, &signed_in.body).await;
    assert_eq!(done.body["status"], "complete");
    assert!(
        f.core.store.list::<Value>("consents").unwrap().is_empty(),
        "implicit consent is never stored"
    );
    redeem(
        &f,
        "first",
        &resume(&app, &i, Some(&sso)).await.location(),
        &verifier,
        secret,
    );
}

#[tokio::test]
async fn prompt_login_requires_proof_from_this_interaction_and_keeps_sid() {
    let f = Fixture::new();
    let app = setup(&f);
    f.user("alice");
    let sso = browser(&f, "alice");
    let holder = sid(&f, &sso);
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.prompt = Some("login".into());
    let i = start(&f, &app, &request, Some(&sso)).await;
    let state = get_state(&app, &i, Some(&sso)).await.body;
    assert_eq!(state["status"], "authenticate");
    assert_eq!(state["reason"], "prompt_login");
    assert_eq!(state["pinned"], true);
    assert_eq!(state["account"]["username"], "alice");
    let refused = decide(&app, &i, Some(&sso), true, &state).await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(refused.body["error"], "login_required");
    let signed_in = password(&app, &i, Some(&sso), "alice", None).await;
    assert_eq!(signed_in.body["status"], "consent");
    let rotated = signed_in.sso();
    assert_ne!(rotated, sso, "every sign-in issues a new cookie");
    assert_eq!(sid(&f, &rotated), holder, "the same user keeps the session");
    assert_eq!(signed_in.body["session_ref"], state["session_ref"]);
    // The old cookie is a tombstone and no longer authenticates.
    assert!(f.core.portal_apps(Some(&sso)).is_err());
    let done = decide(&app, &i, Some(&rotated), true, &signed_in.body).await;
    assert_eq!(done.body["status"], "complete", "{}", done.text);
    let location = resume(&app, &i, Some(&rotated)).await.location();
    assert_eq!(grant(&f, &location).identity.session_id, holder);
    let claims = id_token(&f, &redeem(&f, "app", &location, &verifier, None), "app");
    assert!(claims["auth_time"].as_u64().unwrap() + 5 >= now());
}

#[tokio::test]
async fn max_age_zero_requires_proof() {
    let f = Fixture::new();
    let app = setup(&f);
    f.user("alice");
    let sso = browser(&f, "alice");
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.max_age = Some(0);
    let i = start(&f, &app, &request, Some(&sso)).await;
    let state = get_state(&app, &i, Some(&sso)).await.body;
    assert_eq!(state["status"], "authenticate");
    assert_eq!(state["reason"], "max_age");
    assert_eq!(
        decide(&app, &i, Some(&sso), true, &state).await.body["error"],
        "login_required"
    );
    let signed_in = password(&app, &i, Some(&sso), "alice", None).await;
    let sso = signed_in.sso();
    assert_eq!(
        decide(&app, &i, Some(&sso), true, &signed_in.body)
            .await
            .body["status"],
        "complete"
    );
    redeem(
        &f,
        "app",
        &resume(&app, &i, Some(&sso)).await.location(),
        &verifier,
        None,
    );
    // With consent remembered, a sign-in bound to the next request continues at once (F9).
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.max_age = Some(0);
    let i = start(&f, &app, &request, Some(&sso)).await;
    let signed_in = password(&app, &i, Some(&sso), "alice", None).await;
    assert_eq!(signed_in.body["status"], "complete", "{}", signed_in.text);
    let sso = signed_in.sso();
    redeem(
        &f,
        "app",
        &resume(&app, &i, Some(&sso)).await.location(),
        &verifier,
        None,
    );
}

#[tokio::test]
async fn proof_cannot_be_moved_between_identical_requests() {
    let f = Fixture::new();
    let app = setup(&f);
    f.user("alice");
    let sso = browser(&f, "alice");
    let mut request = f.request("app", &crypto::random_token(""));
    request.prompt = Some("login".into());
    let first = start(&f, &app, &request, Some(&sso)).await;
    let second = start(&f, &app, &request, Some(&sso)).await;
    let signed_in = password(&app, &first, Some(&sso), "alice", None).await;
    assert_eq!(signed_in.body["status"], "consent");
    let sso = signed_in.sso();
    // The proof names the first request; the second still asks for a sign-in.
    let other = get_state(&app, &second, Some(&sso)).await.body;
    assert_eq!(other["status"], "authenticate");
    assert_eq!(other["reason"], "prompt_login");
    assert_eq!(
        decide(&app, &second, Some(&sso), true, &other).await.body["error"],
        "login_required"
    );
    let proof = text(
        &f.core
            .store
            .get::<Value>("browser_authorizations", &first.id)
            .unwrap()
            .unwrap(),
        "authentication",
    );
    let hash = |id: &str| {
        let pending: Value = f
            .core
            .store
            .get("browser_authorizations", id)
            .unwrap()
            .unwrap();
        let request: Authorization = serde_json::from_value(pending["request"].clone()).unwrap();
        request.request_hash().unwrap()
    };
    let holder = sid(&f, &sso);
    let valid = |hash: &str| {
        f.core
            .store
            .read(|tx| signin::proof_valid(tx, Some(&proof), hash, &holder))
            .unwrap()
    };
    assert!(valid(&hash(&first.id)));
    assert!(!valid(&hash(&second.id)));
    assert_eq!(
        decide(&app, &first, Some(&sso), true, &signed_in.body)
            .await
            .body["status"],
        "complete"
    );
}

#[tokio::test]
async fn require_mfa_client_rejects_password_only_login_without_creating_a_session() {
    let f = Fixture::new();
    client(&f, "secure", false, true, Default::default());
    let app = riauth::api::router(f.core.clone());
    f.user("bob");
    let carol = f.user("carol");
    enroll(&f, &carol);
    let i = start(
        &f,
        &app,
        &f.request("secure", &crypto::random_token("")),
        None,
    )
    .await;
    let state = get_state(&app, &i, None).await.body;
    assert_eq!(state["requirements"], json!({"mfa": true, "browser": true}));
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    for (username, code) in [
        ("carol", "unmet_authentication_requirements"),
        ("bob", "mfa_setup_required"),
    ] {
        let refused = password(&app, &i, None, username, None).await;
        assert_eq!(refused.status, StatusCode::FORBIDDEN, "{username}");
        assert_eq!(refused.body["error"], code, "{username}");
        assert!(
            refused.body["error_description"]
                .as_str()
                .unwrap()
                .starts_with("secure app requires")
        );
        assert!(refused.set_cookie("riauth_sso").is_none());
    }
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );
    let pending: Value = f
        .core
        .store
        .get("browser_authorizations", &i.id)
        .unwrap()
        .unwrap();
    assert!(pending["authentication"].is_null());
}

#[tokio::test]
async fn require_mfa_client_accepts_totp_login() {
    let f = Fixture::new();
    client(&f, "secure", false, true, Default::default());
    let app = riauth::api::router(f.core.clone());
    let alice = f.user("alice");
    let enrollment = f.core.mfa_begin(&alice).unwrap();
    let totp = crypto::totp(enrollment["secret"].as_str().unwrap(), "alice").unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate(now() - 30))
        .unwrap();
    let verifier = crypto::random_token("");
    let i = start(&f, &app, &f.request("secure", &verifier), None).await;
    let missing = password(&app, &i, None, "alice", None).await;
    assert_eq!(missing.status, StatusCode::UNAUTHORIZED);
    assert_eq!(missing.body["error"], "invalid_credentials");
    let signed_in = password(&app, &i, None, "alice", Some(&totp.generate(now()))).await;
    assert_eq!(signed_in.body["status"], "consent", "{}", signed_in.text);
    assert_eq!(signed_in.body["account"]["mfa"], true);
    let sso = signed_in.sso();
    assert!(session(&f, &sid(&f, &sso)).identity.mfa);
    assert_eq!(
        decide(&app, &i, Some(&sso), true, &signed_in.body)
            .await
            .body["status"],
        "complete"
    );
    let tokens = redeem(
        &f,
        "secure",
        &resume(&app, &i, Some(&sso)).await.location(),
        &verifier,
        None,
    );
    let claims = id_token(&f, &tokens, "secure");
    assert_eq!(claims["amr"], json!(["pwd", "otp"]));
    assert_eq!(claims["acr"], riauth::assurance::MFA);
}

#[tokio::test]
async fn select_account_switches_user_and_revokes_previous_browser_session() {
    let f = Fixture::new();
    let app = setup(&f);
    f.user("alice");
    f.user("bob");
    let sso = browser(&f, "alice");
    let alice_sid = sid(&f, &sso);
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.prompt = Some("select_account".into());
    let i = start(&f, &app, &request, Some(&sso)).await;
    let state = get_state(&app, &i, Some(&sso)).await.body;
    assert_eq!(state["status"], "authenticate");
    assert_eq!(state["reason"], "select_account");
    assert_eq!(state["pinned"], false, "another account may sign in");
    assert_eq!(state["account"]["username"], "alice");
    let signed_in = password(&app, &i, Some(&sso), "bob", None).await;
    assert_eq!(signed_in.body["status"], "consent", "{}", signed_in.text);
    assert_eq!(signed_in.body["account"]["username"], "bob");
    let bob_sso = signed_in.sso();
    let bob_sid = sid(&f, &bob_sso);
    assert_ne!(bob_sid, alice_sid);
    assert!(
        session(&f, &alice_sid).revoked,
        "the displaced browser session cannot be reached again"
    );
    assert!(audited(
        &f,
        &user_id(&f, "bob"),
        "session.revoke",
        &alice_sid
    ));
    assert_eq!(
        decide(&app, &i, Some(&bob_sso), true, &signed_in.body)
            .await
            .body["status"],
        "complete"
    );
    let location = resume(&app, &i, Some(&bob_sso)).await.location();
    assert_eq!(grant(&f, &location).identity.user_id, user_id(&f, "bob"));
}

#[tokio::test]
async fn pinned_request_rejects_other_account_generically() {
    let f = Fixture::new();
    let app = setup(&f);
    f.user("alice");
    f.user("bob");
    let sso = browser(&f, "alice");
    let holder = sid(&f, &sso);
    let mut request = f.request("app", &crypto::random_token(""));
    request.prompt = Some("login".into());
    let i = start(&f, &app, &request, Some(&sso)).await;
    let clock = Instant::now();
    let other = password(&app, &i, Some(&sso), "bob", None).await;
    assert!(
        clock.elapsed().as_millis() >= 1000,
        "failures take at least a second"
    );
    let mut wrong = post(
        &i.path("/password"),
        &i.cookies(Some(&sso)),
        json!({"username": "alice", "password": "not the password"}),
    );
    wrong.headers_mut().remove("accept");
    let wrong = call(&app, wrong).await;
    for refused in [&other, &wrong] {
        assert_eq!(refused.status, StatusCode::UNAUTHORIZED);
        assert_eq!(refused.body["error"], "invalid_credentials");
        assert!(refused.set_cookie("riauth_sso").is_none());
    }
    assert_eq!(
        other.body, wrong.body,
        "another account looks like a wrong password"
    );
    assert!(!session(&f, &holder).revoked);
    assert_eq!(sid(&f, &sso), holder);
    let state = get_state(&app, &i, Some(&sso)).await.body;
    assert_eq!(state["status"], "authenticate");
    assert_eq!(state["account"]["username"], "alice");
}

#[tokio::test]
async fn decision_rejects_changed_session_ref() {
    let f = Fixture::new();
    let app = setup(&f);
    f.user("alice");
    let i = start(&f, &app, &f.request("app", &crypto::random_token("")), None).await;
    let signed_in = password(&app, &i, None, "alice", None).await;
    let sso = signed_in.sso();
    let mut moved = signed_in.body.clone();
    moved["session_ref"] = json!(signin::session_ref(&i.id, "another-session"));
    for state in [&moved, &json!({"session_ref": null})] {
        let changed = decide(&app, &i, Some(&sso), true, state).await;
        assert_eq!(changed.status, StatusCode::CONFLICT);
        assert_eq!(changed.body["error"], "account_changed");
    }
    // Approving needs this browser's session; the cookie alone says who approves.
    let anonymous = decide(&app, &i, None, true, &signed_in.body).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous.body["error"], "invalid_token");
    assert_eq!(
        get_state(&app, &i, Some(&sso)).await.body["status"],
        "consent"
    );
    let done = decide(&app, &i, Some(&sso), true, &signed_in.body).await;
    assert_eq!(done.body["status"], "complete");
    let decided = decide(&app, &i, Some(&sso), true, &signed_in.body).await;
    assert_eq!(decided.status, StatusCode::CONFLICT);
    assert_eq!(decided.body["error"], "request_decided");
}

#[tokio::test]
async fn deny_returns_access_denied_in_query_and_form_post() {
    let f = Fixture::new();
    let app = setup(&f);
    for mode in [None, Some("form_post")] {
        let mut request = f.request("app", &crypto::random_token(""));
        request.response_mode = mode.map(String::from);
        let i = start(&f, &app, &request, None).await;
        let denied = decide(&app, &i, None, false, &Value::Null).await;
        assert_eq!(denied.status, StatusCode::OK, "{}", denied.text);
        assert_eq!(denied.body["status"], "complete");
        assert_eq!(denied.body["continue"], format!("/oauth/resume/{}", i.id));
        assert!(audited(&f, "anonymous", "authorization.denied", "app"));
        let delivered = resume(&app, &i, None).await;
        if mode.is_none() {
            assert_eq!(delivered.status, StatusCode::FOUND);
            let params = query(&delivered.location());
            assert_eq!(params["error"], "access_denied");
            assert_eq!(params["state"], "state with & delimiters");
            assert!(!params.contains_key("code"));
        } else {
            assert_eq!(delivered.status, StatusCode::OK);
            assert!(
                delivered
                    .text
                    .contains("name=\"error\" value=\"access_denied\"")
            );
            assert!(!delivered.text.contains("name=\"code\""));
        }
    }
}

#[tokio::test]
async fn terminal_approval_still_completes_an_open_interaction() {
    let f = Fixture::new();
    let app = setup(&f);
    let verifier = crypto::random_token("");
    let i = start(&f, &app, &f.request("app", &verifier), None).await;
    let details = f.core.browser_details(&f.admin, &i.code).unwrap();
    assert_eq!(details["reauthentication_required"], false);
    approve_in_terminal(&f, &f.admin, &i.code).unwrap();
    let state = get_state(&app, &i, None).await.body;
    assert_eq!(state["status"], "complete");
    assert_eq!(state["continue"], format!("/oauth/resume/{}", i.id));
    let delivered = resume(&app, &i, None).await;
    assert_eq!(delivered.status, StatusCode::FOUND);
    let admin_sid = text(&f.core.me(&f.admin).unwrap(), "session_id");
    assert_eq!(sid(&f, &delivered.sso()), admin_sid);
    redeem(&f, "app", &delivered.location(), &verifier, None);
    // The browser shares the terminal session, which keeps working.
    assert!(f.core.me(&f.admin).is_ok());
}

#[tokio::test]
async fn terminal_approval_revokes_displaced_browser_owned_session_and_queues_logout() {
    let f = Fixture::new();
    let app = setup(&f);
    let alice = f.user("alice");
    f.user("bob");
    // Bob signs in to the application in this browser, so revoking must reach it.
    let verifier = crypto::random_token("");
    let i = start(&f, &app, &f.request("app", &verifier), None).await;
    let signed_in = password(&app, &i, None, "bob", None).await;
    let bob_sso = signed_in.sso();
    let bob_sid = sid(&f, &bob_sso);
    decide(&app, &i, Some(&bob_sso), true, &signed_in.body).await;
    redeem(
        &f,
        "app",
        &resume(&app, &i, Some(&bob_sso)).await.location(),
        &verifier,
        None,
    );
    // Alice approves the next request from her terminal.
    let mut request = f.request("app", &crypto::random_token(""));
    request.prompt = Some("consent".into());
    let next = start(&f, &app, &request, Some(&bob_sso)).await;
    approve_in_terminal(&f, &alice, &next.code).unwrap();
    let delivered = resume(&app, &next, Some(&bob_sso)).await;
    assert_eq!(delivered.status, StatusCode::FOUND);
    let alice_sid = text(&f.core.me(&alice).unwrap(), "session_id");
    assert_eq!(sid(&f, &delivered.sso()), alice_sid);
    assert!(session(&f, &bob_sid).revoked);
    assert!(audited(
        &f,
        &user_id(&f, "alice"),
        "session.revoke",
        &bob_sid
    ));
    let rp: Vec<riauth::logout::RpSession> = f
        .core
        .store
        .list::<riauth::logout::RpSession>("rp_sessions")
        .unwrap()
        .into_iter()
        .map(|(_, rp)| rp)
        .filter(|rp| rp.session_id == bob_sid)
        .collect();
    assert!(
        !rp.is_empty() && rp.iter().all(|rp| rp.ended),
        "logout queued"
    );
    assert!(f.core.portal_apps(Some(&bob_sso)).is_err());
    assert!(f.core.me(&alice).is_ok());
}

#[tokio::test]
async fn stale_terminal_session_must_reauthenticate_before_approving() {
    let f = Fixture::new();
    let app = setup(&f);
    let alice = f.user("alice");
    let alice_sid = text(&f.core.me(&alice).unwrap(), "session_id");
    let i = start(&f, &app, &f.request("app", &crypto::random_token("")), None).await;
    assert_eq!(
        f.core.browser_details(&alice, &i.code).unwrap()["reauthentication_required"],
        false
    );
    // Past 240 s the CLI is asked to sign in again; past 300 s an approval fails.
    age(&f, &alice_sid, 241);
    assert_eq!(
        f.core.browser_details(&alice, &i.code).unwrap()["reauthentication_required"],
        true
    );
    age(&f, &alice_sid, 60);
    let stale = approve_in_terminal(&f, &alice, &i.code).unwrap_err();
    assert_eq!((stale.status.as_u16(), stale.code), (400, "login_required"));
    // A fresh terminal sign-in approves.
    let fresh = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    approve_in_terminal(&f, &fresh, &i.code).unwrap();
    assert_eq!(resume(&app, &i, None).await.status, StatusCode::FOUND);
}

#[tokio::test]
async fn oauth_source_session_is_exempt_from_terminal_freshness() {
    let f = Fixture::new();
    let app = setup(&f);
    let alice = f.user("alice");
    let alice_sid = text(&f.core.me(&alice).unwrap(), "session_id");
    // Upstream OAuth sources report no authentication time.
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", &alice_sid)?.unwrap();
            session.identity.auth_time = 0;
            tx.put("sessions", &alice_sid, &session)
        })
        .unwrap();
    let i = start(&f, &app, &f.request("app", &crypto::random_token("")), None).await;
    assert_eq!(
        f.core.browser_details(&alice, &i.code).unwrap()["reauthentication_required"],
        false
    );
    approve_in_terminal(&f, &alice, &i.code).unwrap();
    assert_eq!(resume(&app, &i, None).await.status, StatusCode::FOUND);
}

#[tokio::test]
async fn terminal_details_show_requester() {
    let f = Fixture::new();
    let app = setup(&f);
    let request = f.request("app", &crypto::random_token(""));
    let mut navigation = get(&authorize_uri(&request), "text/html", &[]);
    navigation
        .headers_mut()
        .insert("user-agent", "RequesterProbe/1.0".parse().unwrap());
    navigation
        .extensions_mut()
        .insert(ConnectInfo(std::net::SocketAddr::from((
            [192, 0, 2, 7],
            40000,
        ))));
    let started = call(&app, navigation).await;
    assert_eq!(started.status, StatusCode::SEE_OTHER);
    let id = started.location().rsplit('/').next().unwrap().to_owned();
    let pending: Value = f
        .core
        .store
        .get("browser_authorizations", &id)
        .unwrap()
        .unwrap();
    let details = call(
        &app,
        Request::get(format!("/api/authorization/{}", text(&pending, "code")))
            .header("authorization", format!("Bearer {}", f.admin))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(details.status, StatusCode::OK);
    let requester = &details.body["requested_from"];
    assert_eq!(requester["ip"], "192.0.2.7");
    assert_eq!(requester["user_agent"], "RequesterProbe/1.0");
    assert!(requester["at"].as_u64().unwrap() + 5 >= now());
}

#[tokio::test]
async fn par_request_survives_browser_sign_in_and_state_reports_its_expiry() {
    let f = Fixture::new();
    let app = setup(&f);
    f.user("alice");
    let verifier = crypto::random_token("");
    let template = f.request("app", &verifier);
    let pairs = [
        ("client_id", "app"),
        ("response_type", "code"),
        ("redirect_uri", REDIRECT),
        ("scope", "openid profile"),
        ("state", "state with & delimiters"),
        ("code_challenge", template.code_challenge.as_str()),
        ("code_challenge_method", "S256"),
    ]
    .map(|(k, v)| (k.to_owned(), v.to_owned()));
    let pushed = f
        .core
        .push_authorization(&HeaderMap::new(), pairs.to_vec())
        .unwrap();
    let uri = format!(
        "/oauth/authorize?{}",
        serde_urlencoded::to_string([
            ("client_id", "app"),
            ("request_uri", pushed["request_uri"].as_str().unwrap())
        ])
        .unwrap()
    );
    let reply = call(&app, get(&uri, "text/html", &[])).await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text);
    let id = reply.location().rsplit('/').next().unwrap().to_owned();
    let pending: Value = f
        .core
        .store
        .get("browser_authorizations", &id)
        .unwrap()
        .unwrap();
    let i = Interaction {
        binding: reply.cookie("riauth_return").unwrap(),
        code: text(&pending, "code"),
        id,
    };
    // Starting the request extended the pushed request; a shorter one sets the deadline.
    let key = digest(pushed["request_uri"].as_str().unwrap());
    let deadline = now() + 200;
    f.core
        .store
        .write(|tx| {
            let mut stored: Value = tx.get("pushed_requests", &key)?.unwrap();
            assert!(stored["expires_at"].as_u64().unwrap() + 5 >= now() + 600);
            stored["expires_at"] = json!(deadline);
            tx.put("pushed_requests", &key, &stored)
        })
        .unwrap();
    assert_eq!(get_state(&app, &i, None).await.body["expires_at"], deadline);
    let signed_in = password(&app, &i, None, "alice", None).await;
    assert_eq!(signed_in.body["status"], "consent", "{}", signed_in.text);
    assert_eq!(signed_in.body["expires_at"], deadline);
    let sso = signed_in.sso();
    assert_eq!(
        decide(&app, &i, Some(&sso), true, &signed_in.body)
            .await
            .body["status"],
        "complete"
    );
    let location = resume(&app, &i, Some(&sso)).await.location();
    redeem(&f, "app", &location, &verifier, None);
    let used: Value = f.core.store.get("pushed_requests", &key).unwrap().unwrap();
    assert_eq!(used["used"], true);
}

#[tokio::test]
async fn passkey_interaction_sign_in_pinned_and_discoverable() {
    let f = Fixture::new();
    let app = setup(&f);
    let alice = f.user("alice");
    let (mut authenticator, credential) = enroll(&f, &alice);
    // Discoverable: no account is known, so the browser may offer any passkey.
    let verifier = crypto::random_token("");
    let i = start(&f, &app, &f.request("app", &verifier), None).await;
    let started = call(
        &app,
        post(&i.path("/passkey/start"), &i.cookies(None), json!({})),
    )
    .await;
    assert_eq!(started.status, StatusCode::OK, "{}", started.text);
    assert_eq!(started.body["expires_in"], 300);
    let options = &started.body["public_key"];
    assert_eq!(options["publicKey"]["allowCredentials"], json!([]));
    assert!(options.get("mediation").is_none() && options["publicKey"].get("extensions").is_none());
    let mut request = started.body["public_key"].clone();
    request["publicKey"]["allowCredentials"] = json!([{"type": "public-key", "id": credential}]);
    let proof = authenticator
        .do_authentication(origin(), serde_json::from_value(request).unwrap())
        .unwrap();
    let mut proof = serde_json::to_value(proof).unwrap();
    let handle = Sha256::digest(format!("riauth.webauthn-user/v1\0{}", user_id(&f, "alice")));
    proof["response"]["userHandle"] = json!(URL_SAFE_NO_PAD.encode(&handle[..16]));
    let finished = call(
        &app,
        post(
            &i.path("/passkey/finish"),
            &i.cookies(None),
            json!({"ceremony": started.body["ceremony"], "credential": proof}),
        ),
    )
    .await;
    assert_eq!(finished.status, StatusCode::OK, "{}", finished.text);
    assert_eq!(finished.body["status"], "consent");
    assert_eq!(finished.body["account"]["mfa"], true);
    let sso = finished.sso();
    let holder = sid(&f, &sso);
    assert_eq!(
        decide(&app, &i, Some(&sso), true, &finished.body)
            .await
            .body["status"],
        "complete"
    );
    let tokens = redeem(
        &f,
        "app",
        &resume(&app, &i, Some(&sso)).await.location(),
        &verifier,
        None,
    );
    assert_eq!(
        id_token(&f, &tokens, "app")["amr"],
        json!(["webauthn", "mfa"])
    );
    // Pinned: re-authentication offers only the signed-in account's passkeys.
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.prompt = Some("login consent".into());
    let i = start(&f, &app, &request, Some(&sso)).await;
    let started = call(
        &app,
        post(&i.path("/passkey/start"), &i.cookies(Some(&sso)), json!({})),
    )
    .await;
    let allowed = &started.body["public_key"]["publicKey"]["allowCredentials"];
    assert_eq!(allowed.as_array().unwrap().len(), 1);
    assert_eq!(allowed[0]["id"], credential);
    assert!(allowed[0].get("transports").is_none());
    let proof = authenticator
        .do_authentication(
            origin(),
            serde_json::from_value(started.body["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let finished = call(
        &app,
        post(
            &i.path("/passkey/finish"),
            &i.cookies(Some(&sso)),
            json!({"ceremony": started.body["ceremony"], "credential": proof}),
        ),
    )
    .await;
    assert_eq!(finished.body["status"], "consent", "{}", finished.text);
    let sso = finished.sso();
    assert_eq!(sid(&f, &sso), holder, "re-authentication keeps the session");
    // A ceremony is single use.
    let replay = call(
        &app,
        post(
            &i.path("/passkey/finish"),
            &i.cookies(Some(&sso)),
            json!({"ceremony": started.body["ceremony"], "credential": proof}),
        ),
    )
    .await;
    assert_eq!(replay.status, StatusCode::UNAUTHORIZED);
    let malformed = call(
        &app,
        post(
            &i.path("/passkey/finish"),
            &i.cookies(Some(&sso)),
            json!({"ceremony": "x", "credential": {"id": 1}}),
        ),
    )
    .await;
    assert_eq!(malformed.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        malformed.body["error_description"],
        "Malformed WebAuthn response"
    );
    assert_eq!(
        decide(&app, &i, Some(&sso), true, &finished.body)
            .await
            .body["status"],
        "complete"
    );
}

#[tokio::test]
async fn unavailable_state_for_policy_denied_and_unsatisfiable_acr() {
    let f = Fixture::new();
    let app = setup(&f);
    f.core.create_group(&f.admin, "staff").unwrap();
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "staff-only".into(),
                name: "Staff tools".into(),
                confidential: false,
                redirect_uris: vec![REDIRECT.into()],
                scopes: strings(&["openid", "profile", "email", "groups", "offline_access"]),
                allowed_groups: strings(&["staff"]),
                require_mfa: false,
                service: false,
                settings: Default::default(),
            },
        )
        .unwrap();
    f.user("alice");
    let sso = browser(&f, "alice");
    let i = start(
        &f,
        &app,
        &f.request("staff-only", &crypto::random_token("")),
        Some(&sso),
    )
    .await;
    let denied = get_state(&app, &i, Some(&sso)).await.body;
    assert_eq!(denied["status"], "unavailable");
    assert_eq!(denied["error"], "access_denied");
    assert!(denied["message"].is_string());
    let back = decide(&app, &i, Some(&sso), false, &denied).await;
    assert_eq!(back.body["status"], "complete");
    assert_eq!(
        query(&resume(&app, &i, Some(&sso)).await.location())["error"],
        "access_denied"
    );
    // Neither a password nor a passkey can reach a federated-only ACR in the browser.
    let mut request = f.request("app", &crypto::random_token(""));
    request.acr_values = Some(riauth::assurance::FEDERATED.into());
    let i = start(&f, &app, &request, None).await;
    let state = get_state(&app, &i, None).await.body;
    assert_eq!(state["status"], "unavailable");
    assert_eq!(state["error"], "step_up_unavailable");
    assert_eq!(
        state["requirements"],
        json!({"mfa": true, "browser": false})
    );
    assert_eq!(state["terminal"]["user_code"], i.code);
}

/// A relying-party logout navigation without an ID token hint.
async fn logout_start(app: &Router, sso: &str) -> (String, String) {
    let uri = format!(
        "/oauth/logout?{}",
        serde_urlencoded::to_string([
            ("client_id", "app"),
            ("post_logout_redirect_uri", SIGNED_OUT),
            ("state", "bye")
        ])
        .unwrap()
    );
    let reply = call(app, get(&uri, "text/html", &[("riauth_sso", sso)])).await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text);
    assert_eq!(reply.header("vary"), Some("Accept"));
    let id = reply
        .location()
        .strip_prefix("http://localhost:9000/oauth/logout/resume/")
        .unwrap()
        .to_owned();
    assert!(
        reply
            .set_cookie("riauth_logout")
            .unwrap()
            .contains(&format!("Path=/oauth/logout/resume/{id};"))
    );
    (id, reply.cookie("riauth_logout").unwrap())
}

#[tokio::test]
async fn logout_without_hint_redirects_to_confirmation_and_revokes_only_own_session() {
    let f = Fixture::new();
    let app = setup(&f);
    let cli = f.user("alice");
    let sso = browser(&f, "alice");
    let other = browser(&f, "alice");
    let (own, other_sid) = (sid(&f, &sso), sid(&f, &other));
    let (id, binding) = logout_start(&app, &sso).await;
    let cookies = [
        ("riauth_logout", binding.as_str()),
        ("riauth_sso", sso.as_str()),
    ];
    let page = call(
        &app,
        get(&format!("/oauth/logout/resume/{id}"), "text/html", &cookies),
    )
    .await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.header("refresh").is_none() && page.header("cross-origin-opener-policy").is_none()
    );
    assert!(page.text.contains("<code>riauth logout-request approve "));
    let state = call(
        &app,
        get(
            &format!("/oauth/logout/resume/{id}/state"),
            "application/json",
            &cookies,
        ),
    )
    .await
    .body;
    assert_eq!(state["kind"], "logout");
    assert_eq!(state["status"], "confirm");
    assert_eq!(
        state["logout"],
        json!({"targeted": true, "matches_browser": true, "ended": false})
    );
    assert_eq!(state["account"]["username"], "alice");
    let done = call(
        &app,
        post(
            &format!("/oauth/logout/resume/{id}/decision"),
            &cookies,
            json!({"approve": true}),
        ),
    )
    .await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.text);
    assert_eq!(
        done.body,
        json!({"status": "done", "continue": format!("/oauth/logout/resume/{id}")})
    );
    assert_eq!(
        done.cookie("riauth_sso").as_deref(),
        Some(""),
        "the SSO cookie is cleared"
    );
    assert!(session(&f, &own).revoked);
    assert!(!session(&f, &other_sid).revoked);
    assert!(f.core.portal_apps(Some(&other)).is_ok() && f.core.me(&cli).is_ok());
    let finished = call(
        &app,
        get(
            &format!("/oauth/logout/resume/{id}"),
            "text/html",
            &cookies[..1],
        ),
    )
    .await;
    assert_eq!(finished.status, StatusCode::FOUND);
    assert_eq!(finished.location(), format!("{SIGNED_OUT}?state=bye"));
    // Without the page, the same request still offers terminal confirmation as JSON.
    let json = call(
        &app,
        get(
            &format!(
                "/oauth/logout?client_id=app&post_logout_redirect_uri={}",
                url::form_urlencoded::byte_serialize(SIGNED_OUT.as_bytes()).collect::<String>()
            ),
            "application/json",
            &[("riauth_sso", &other)],
        ),
    )
    .await;
    assert_eq!(json.status, StatusCode::OK);
    assert_eq!(json.body["interaction_required"], true);
    assert!(json.header("refresh").is_some());
}

#[tokio::test]
async fn logout_confirmation_for_another_session_is_refused() {
    let f = Fixture::new();
    let app = setup(&f);
    let cli = f.user("alice");
    f.user("bob");
    let tokens = f.tokens("app", &cli, None);
    let bob = browser(&f, "bob");
    let uri = format!(
        "/oauth/logout?{}",
        serde_urlencoded::to_string([
            ("id_token_hint", text(&tokens, "id_token").as_str()),
            ("post_logout_redirect_uri", SIGNED_OUT)
        ])
        .unwrap()
    );
    let reply = call(&app, get(&uri, "text/html", &[("riauth_sso", &bob)])).await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text);
    let id = reply.location().rsplit('/').next().unwrap().to_owned();
    let binding = reply.cookie("riauth_logout").unwrap();
    let cookies = [
        ("riauth_logout", binding.as_str()),
        ("riauth_sso", bob.as_str()),
    ];
    let state = call(
        &app,
        get(
            &format!("/oauth/logout/resume/{id}/state"),
            "application/json",
            &cookies,
        ),
    )
    .await
    .body;
    assert_eq!(
        state["logout"],
        json!({"targeted": true, "matches_browser": false, "ended": false})
    );
    assert_eq!(state["account"]["username"], "bob");
    let refused = call(
        &app,
        post(
            &format!("/oauth/logout/resume/{id}/decision"),
            &cookies,
            json!({"approve": true}),
        ),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(refused.body["error"], "session_mismatch");
    assert!(
        f.core.me(&cli).is_ok(),
        "another session is never ended from this browser"
    );
    let back = call(
        &app,
        post(
            &format!("/oauth/logout/resume/{id}/decision"),
            &cookies,
            json!({"approve": false}),
        ),
    )
    .await;
    assert_eq!(back.body["status"], "done");
    assert!(back.set_cookie("riauth_sso").is_none());
    assert!(audited(&f, &user_id(&f, "bob"), "logout.denied", "app"));
    assert!(f.core.me(&cli).is_ok() && f.core.portal_apps(Some(&bob)).is_ok());
    let finished = call(
        &app,
        get(
            &format!("/oauth/logout/resume/{id}"),
            "text/html",
            &cookies[..1],
        ),
    )
    .await;
    assert_eq!(finished.location(), SIGNED_OUT);
}

#[tokio::test]
async fn direct_hint_logout_clears_sso_cookie() {
    let f = Fixture::new();
    let app = setup(&f);
    f.user("alice");
    let verifier = crypto::random_token("");
    let i = start(&f, &app, &f.request("app", &verifier), None).await;
    let signed_in = password(&app, &i, None, "alice", None).await;
    let sso = signed_in.sso();
    let holder = sid(&f, &sso);
    decide(&app, &i, Some(&sso), true, &signed_in.body).await;
    let tokens = redeem(
        &f,
        "app",
        &resume(&app, &i, Some(&sso)).await.location(),
        &verifier,
        None,
    );
    let uri = format!(
        "/oauth/logout?{}",
        serde_urlencoded::to_string([("id_token_hint", text(&tokens, "id_token"))]).unwrap()
    );
    let reply = call(&app, get(&uri, "application/json", &[("riauth_sso", &sso)])).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text);
    assert_eq!(reply.body["logged_out"], true);
    assert!(reply.body.get("clear_sso").is_none(), "an internal flag");
    let cleared = reply.set_cookie("riauth_sso").unwrap();
    assert!(cleared.starts_with("riauth_sso=;") && cleared.contains("Max-Age=0"));
    assert!(session(&f, &holder).revoked);
    assert!(
        f.core
            .store
            .get::<Value>("browser_sessions", &digest(&sso))
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn resume_errors_render_html_for_browsers() {
    let f = Fixture::new();
    let app = setup(&f);
    let i = start(&f, &app, &f.request("app", &crypto::random_token("")), None).await;
    let stranger = [("riauth_return", "another-browser")];
    for (uri, cookies, status, title) in [
        (
            i.path(""),
            &stranger[..],
            StatusCode::UNAUTHORIZED,
            "This sign-in belongs to another browser",
        ),
        (
            format!("/oauth/resume/{MISSING}"),
            &[][..],
            StatusCode::NOT_FOUND,
            "This sign-in has expired or was already completed. Return to the application and try again.",
        ),
        (
            format!("/saml/resume/{MISSING}"),
            &[][..],
            StatusCode::NOT_FOUND,
            "This sign-in has ended",
        ),
        (
            format!("/oauth/logout/resume/{MISSING}"),
            &[][..],
            StatusCode::NOT_FOUND,
            "This sign-out request has ended",
        ),
    ] {
        let page = call(&app, get(&uri, "text/html", cookies)).await;
        assert_eq!(page.status, status, "{uri}");
        assert!(
            page.header("content-type")
                .unwrap()
                .starts_with("text/html"),
            "{uri}"
        );
        assert!(page.text.contains(title), "{uri}: {}", page.text);
        assert!(page.text.contains("href=\"/apps\""), "{uri}");
        assert_eq!(page.header("vary"), Some("Accept"));
        assert!(
            !page
                .header("content-security-policy")
                .unwrap()
                .contains("script-src")
        );
        // API clients keep the JSON error.
        let json = call(&app, get(&uri, "application/json", cookies)).await;
        assert_eq!(json.status, status, "{uri}");
        assert!(json.body["error"].is_string(), "{uri}");
    }
}

#[tokio::test]
async fn browser_approval_is_delivered_only_to_the_approving_browser() {
    let f = Fixture::new();
    let app = setup(&f);
    let secret = client(
        &f,
        "implicit",
        true,
        false,
        ProviderSettings {
            implicit_consent: true,
            ..Default::default()
        },
    );
    f.user("alice");
    f.user("bob");
    let bob = browser(&f, "bob");
    let bob_sid = sid(&f, &bob);
    // A planted binding cookie alone, or with another browser's session, collects neither an
    // approval on the consent screen nor an automatic one; the request waits for its browser.
    for (cid, secret) in [("app", None), ("implicit", secret)] {
        let verifier = crypto::random_token("");
        let i = start(&f, &app, &f.request(cid, &verifier), None).await;
        let signed_in = password(&app, &i, None, "alice", None).await;
        let sso = signed_in.sso();
        if cid == "app" {
            let done = decide(&app, &i, Some(&sso), true, &signed_in.body).await;
            assert_eq!(done.body["status"], "complete", "{}", done.text);
        } else {
            assert_eq!(signed_in.body["status"], "complete", "{}", signed_in.text);
        }
        for stranger in [None, Some(bob.as_str())] {
            let refused = resume(&app, &i, stranger).await;
            assert_eq!(refused.status, StatusCode::UNAUTHORIZED, "{cid}");
            assert!(
                refused
                    .text
                    .contains("This sign-in belongs to another browser")
            );
            assert!(refused.header("set-cookie").is_none(), "{cid}");
        }
        assert!(
            !session(&f, &bob_sid).revoked,
            "the stranger's session is untouched"
        );
        let delivered = resume(&app, &i, Some(&sso)).await;
        assert_eq!(
            delivered.status,
            StatusCode::FOUND,
            "{cid}: {}",
            delivered.text
        );
        assert_eq!(
            delivered.cookie("riauth_return").as_deref(),
            Some(""),
            "delivery clears the binding cookie"
        );
        redeem(&f, cid, &delivered.location(), &verifier, secret);
    }
    // A terminal approval still reaches the browser that started the request (F19).
    let cli = f.user("carol");
    let i = start(&f, &app, &f.request("app", &crypto::random_token("")), None).await;
    approve_in_terminal(&f, &cli, &i.code).unwrap();
    assert_eq!(resume(&app, &i, None).await.status, StatusCode::FOUND);
}

#[tokio::test]
async fn https_binding_cookies_are_host_only_and_named_per_interaction() {
    let dir = tempfile::TempDir::new().unwrap();
    let core = riauth::core::Core::initialize(
        riauth::config::Config {
            data_dir: dir.path().into(),
            issuer: "https://auth.example.test/identity".into(),
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
    let admin = text(
        &core.login("admin".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let f = Fixture {
        _dir: dir,
        core,
        admin,
    };
    let app = setup(&f);
    let cli = f.user("alice");
    let uri = authorize_uri(&f.request("app", &crypto::random_token(""))).replacen(
        "/oauth/",
        "/identity/oauth/",
        1,
    );
    let reply = call(&app, get(&uri, "text/html", &[])).await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text);
    let id = reply
        .location()
        .strip_prefix("https://auth.example.test/identity/oauth/resume/")
        .unwrap()
        .to_owned();
    let name = format!("__Host-riauth_return_{:.16}", digest(&id));
    let cookie = reply.set_cookie(&name).expect("a host-only binding cookie");
    for part in [
        "; Path=/;",
        "; Max-Age=600;",
        "; HttpOnly",
        "; SameSite=Lax",
        "; Secure",
    ] {
        assert!(cookie.contains(part), "{cookie}");
    }
    assert!(!cookie.contains("Domain") && reply.set_cookie("riauth_return").is_none());
    let binding = reply.cookie(&name).unwrap();
    // A sibling subdomain can set only names without the __Host- prefix, which are never
    // read, and another interaction's name binds nothing here (§8).
    let other = format!("__Host-riauth_return_{:.16}", digest(MISSING));
    let state = format!("/identity/oauth/resume/{id}/state");
    for (cookie, status) in [
        ("riauth_return", StatusCode::UNAUTHORIZED),
        (other.as_str(), StatusCode::UNAUTHORIZED),
        (name.as_str(), StatusCode::OK),
    ] {
        let reply = call(&app, get(&state, "application/json", &[(cookie, &binding)])).await;
        assert_eq!(reply.status, status, "{cookie}: {}", reply.text);
    }
    let pending: Value = f
        .core
        .store
        .get("browser_authorizations", &id)
        .unwrap()
        .unwrap();
    approve_in_terminal(&f, &cli, &text(&pending, "code")).unwrap();
    let delivered = call(
        &app,
        get(
            &format!("/identity/oauth/resume/{id}"),
            "text/html",
            &[(&name, &binding)],
        ),
    )
    .await;
    assert_eq!(delivered.status, StatusCode::FOUND, "{}", delivered.text);
    assert_eq!(
        delivered.set_cookie(&name).unwrap(),
        format!("{name}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax; Secure")
    );
    assert!(delivered.set_cookie("__Host-riauth_sso").is_some());
    // The sign-out confirmation binds its browser the same way.
    let reply = call(
        &app,
        get("/identity/oauth/logout?client_id=app", "text/html", &[]),
    )
    .await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text);
    let id = reply.location().rsplit('/').next().unwrap().to_owned();
    let name = format!("__Host-riauth_logout_{:.16}", digest(&id));
    assert!(reply.set_cookie(&name).unwrap().contains("; Path=/;"));
    assert!(reply.set_cookie("riauth_logout").is_none());
    let binding = reply.cookie(&name).unwrap();
    let state = format!("/identity/oauth/logout/resume/{id}/state");
    for (cookie, status) in [
        ("riauth_logout", StatusCode::UNAUTHORIZED),
        (name.as_str(), StatusCode::OK),
    ] {
        let reply = call(&app, get(&state, "application/json", &[(cookie, &binding)])).await;
        assert_eq!(reply.status, status, "{cookie}: {}", reply.text);
    }
}

#[tokio::test]
async fn ended_session_logout_page_says_this_browser_is_still_signed_in() {
    let f = Fixture::new();
    let app = setup(&f);
    let cli = f.user("alice");
    f.user("bob");
    let tokens = f.tokens("app", &cli, None);
    let bob = browser(&f, "bob");
    let uri = format!(
        "/oauth/logout?{}",
        serde_urlencoded::to_string([("id_token_hint", text(&tokens, "id_token"))]).unwrap()
    );
    let reply = call(&app, get(&uri, "text/html", &[("riauth_sso", &bob)])).await;
    assert_eq!(reply.status, StatusCode::SEE_OTHER, "{}", reply.text);
    let id = reply.location().rsplit('/').next().unwrap().to_owned();
    let binding = reply.cookie("riauth_logout").unwrap();
    let cookies = [
        ("riauth_logout", binding.as_str()),
        ("riauth_sso", bob.as_str()),
    ];
    // The targeted session ends before this browser answers; its own session stays.
    f.core.logout(&cli).unwrap();
    let done = call(
        &app,
        post(
            &format!("/oauth/logout/resume/{id}/decision"),
            &cookies,
            json!({"approve": true}),
        ),
    )
    .await;
    assert_eq!(done.body["status"], "done", "{}", done.text);
    assert!(done.set_cookie("riauth_sso").is_none());
    for (cookies, title, text) in [
        (
            &cookies[..],
            "That session is signed out",
            "This browser is still signed in to riAuth.",
        ),
        (
            &cookies[..1],
            "You&#39;re signed out",
            "You can close this tab.",
        ),
    ] {
        let page = call(
            &app,
            get(&format!("/oauth/logout/resume/{id}"), "text/html", cookies),
        )
        .await;
        assert_eq!(page.status, StatusCode::OK);
        assert!(
            page.text.contains(title) && page.text.contains(text),
            "{}",
            page.text
        );
    }
    assert!(f.core.portal_apps(Some(&bob)).is_ok());
}
