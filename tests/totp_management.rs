//! E07: browser authenticator-app (TOTP) enrollment, confirmation, replacement and
//! removal, and recovery-code rotation, with fresh verification and one-time codes.
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use riauth::{
    browser::BrowserReply,
    config::Config,
    core::Core,
    crypto::{self, digest, now},
    error::Error,
    model::{NewUser, Session, User},
};
use serde_json::{Value, json};
use std::sync::{Arc, Barrier};
use tower::ServiceExt;

const PASSWORD: &str = "totp-management-test-password";
const ORIGIN: &str = "http://localhost:9000";

struct Fixture {
    _dir: tempfile::TempDir,
    core: Core,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::TempDir::new().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                issuer: ORIGIN.into(),
                ..Default::default()
            },
            NewUser {
                username: "alice".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Alice".into(),
                admin: true,
            },
        )
        .unwrap();
        Self { _dir: dir, core }
    }

    fn user(&self) -> User {
        let id: String = self.core.store.get("usernames", "alice").unwrap().unwrap();
        self.core.store.get("users", &id).unwrap().unwrap()
    }

    /// A browser-owned session from the portal password form.
    fn browser(&self, otp: Option<String>) -> String {
        let reply = self
            .core
            .portal_password(None, "alice".into(), PASSWORD.into(), otp, false)
            .unwrap();
        cookie(&reply.cookies, "riauth_sso")
    }

    /// A browser that collected a terminal approval and shares that terminal session.
    fn terminal_browser(&self, otp: String) -> String {
        let token = self
            .core
            .login("alice".into(), PASSWORD.into(), Some(otp))
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let request = self.core.portal_sign_in().unwrap();
        self.core
            .portal_decide(&token, text(&request.body, "code"), true)
            .unwrap();
        let binding = cookie(&request.cookies, "riauth_portal");
        let reply = self
            .core
            .portal_poll(text(&request.body, "id"), Some(&binding))
            .unwrap();
        cookie(&reply.cookies, "riauth_sso")
    }

    fn session(&self, sso: &str, change: impl FnOnce(&mut Session)) {
        let id: Value = self
            .core
            .store
            .get("browser_sessions", &digest(sso))
            .unwrap()
            .unwrap();
        let id = id["session_id"].as_str().unwrap().to_owned();
        self.core
            .store
            .write(|tx| {
                let mut session: Session = tx.get("sessions", &id)?.unwrap();
                change(&mut session);
                tx.put("sessions", &id, &session)
            })
            .unwrap();
    }

    fn start(&self, sso: &str, replace: bool) -> Result<Value, Error> {
        self.core
            .portal_totp_start(Some(sso), &self.user().id, replace)
    }

    fn confirm(&self, sso: &str, code: &str) -> Result<BrowserReply, Error> {
        self.core
            .portal_totp_confirm(Some(sso), &self.user().id, code)
    }

    /// Enables the authenticator app from a fresh password browser. Returns its secret and
    /// the recovery codes shown once.
    fn enable(&self) -> (String, Vec<String>) {
        let sso = self.browser(None);
        let started = self.start(&sso, false).unwrap();
        let secret = text(&started, "secret").to_owned();
        let reply = self.confirm(&sso, &code(&secret, now() - 30)).unwrap();
        (secret, codes(&reply.body))
    }

    fn login(&self, otp: &str) -> Result<Value, Error> {
        self.core
            .login("alice".into(), PASSWORD.into(), Some(otp.into()))
    }

    fn audit(&self) -> String {
        serde_json::to_string(&self.core.store.list::<Value>("audit").unwrap()).unwrap()
    }

    fn snapshot(&self) -> std::collections::BTreeMap<String, Value> {
        self.core.store.read(|tx| tx.snapshot()).unwrap()
    }
}

fn text<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field].as_str().unwrap()
}
fn cookie(cookies: &[String], name: &str) -> String {
    cookies
        .iter()
        .find_map(|cookie| cookie.strip_prefix(&format!("{name}=")))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .into()
}
fn code(secret: &str, at: u64) -> String {
    crypto::totp(secret, "alice")
        .unwrap()
        .generate(at)
        .to_string()
}
fn codes(body: &Value) -> Vec<String> {
    body["recovery_codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|code| code.as_str().unwrap().to_owned())
        .collect()
}
fn error_code<T>(result: Result<T, Error>) -> &'static str {
    result.err().expect("expected an error").code
}
/// The SVG path draws exactly the dark modules of the returned `otpauth://` URI's QR code.
fn assert_qr_draws(started: &Value) {
    let expected = qrcode::QrCode::with_error_correction_level(
        text(started, "otpauth_uri"),
        qrcode::EcLevel::M,
    )
    .unwrap();
    let size = expected.width();
    assert_eq!(started["qr"]["size"], size);
    let mut drawn = vec![vec![false; size]; size];
    for run in text(&started["qr"], "path").split('M').skip(1) {
        let (x, rest) = run.split_once(' ').unwrap();
        let (y, rest) = rest.split_once('h').unwrap();
        let (length, _) = rest.split_once('v').unwrap();
        let (x, y, length): (usize, usize, usize) = (
            x.parse().unwrap(),
            y.parse().unwrap(),
            length.parse().unwrap(),
        );
        for cell in &mut drawn[y][x..x + length] {
            assert!(!*cell, "module drawn twice");
            *cell = true;
        }
    }
    for y in 0..size {
        for x in 0..size {
            assert_eq!(
                drawn[y][x],
                expected[(x, y)] == qrcode::Color::Dark,
                "({x}, {y})"
            );
        }
    }
}
fn clears_sso(reply: &BrowserReply) -> bool {
    reply
        .cookies
        .iter()
        .any(|cookie| cookie.starts_with("riauth_sso=;") && cookie.contains("Max-Age=0"))
}

#[test]
fn enrollment_is_session_bound_and_grants_nothing_until_confirmed() {
    let f = Fixture::new();
    let sso = f.browser(None);
    assert_eq!(
        error_code(f.core.portal_totp_start(Some(&sso), "someone-else", false)),
        "account_mismatch"
    );
    assert!(f.user().totp_pending.is_none());
    let started = f.start(&sso, false).unwrap();
    let secret = text(&started, "secret").to_owned();
    assert!(text(&started, "otpauth_uri").starts_with("otpauth://totp/"));
    assert_qr_draws(&started);
    let status = f.core.portal_mfa(Some(&sso)).unwrap();
    assert_eq!(
        (&status["totp_enabled"], &status["enrollment_pending"]),
        (&json!(false), &json!(true))
    );
    assert!(!status.to_string().contains(&secret));

    // An unconfirmed enrollment is not a factor: the password alone still signs in,
    // without MFA.
    let password = f.core.login("alice".into(), PASSWORD.into(), None).unwrap();
    assert_eq!(
        f.core.me(text(&password, "session_token")).unwrap()["mfa"],
        false
    );
    // Another session of the same account holds no binding to this enrollment.
    let other = f.browser(None);
    assert_eq!(
        error_code(f.confirm(&other, &code(&secret, now() - 30))),
        "enrollment_not_found"
    );
    assert_eq!(
        f.core.portal_mfa(Some(&other)).unwrap()["enrollment_pending"],
        false
    );
    assert_eq!(
        f.core.portal_totp_cancel(Some(&other)).unwrap()["cancelled"],
        false
    );
    let before = f.snapshot();
    assert_eq!(error_code(f.confirm(&sso, "000000x")), "invalid_code");
    assert_eq!(error_code(f.confirm(&sso, "12345")), "invalid_code");
    assert_eq!(f.snapshot(), before, "a rejected code changes nothing");
    // Cancelling ends it for good.
    assert_eq!(
        f.core.portal_totp_cancel(Some(&sso)).unwrap()["cancelled"],
        true
    );
    assert!(f.user().totp_pending.is_none());
    assert_eq!(
        error_code(f.confirm(&sso, &code(&secret, now() - 30))),
        "enrollment_not_found"
    );
    assert!(f.user().totp_secret.is_none());

    let started = f.start(&sso, false).unwrap();
    let secret = text(&started, "secret").to_owned();
    let confirming = code(&secret, now() - 30);
    let epoch = f.user().epoch;
    let reply = f.confirm(&sso, &confirming).unwrap();
    assert_eq!(reply.body["status"], "enabled");
    assert_eq!(reply.body["sessions_revoked"], true);
    let recovery = codes(&reply.body);
    assert_eq!(recovery.len(), 10);
    assert!(clears_sso(&reply));
    let user = f.user();
    assert_eq!(user.totp_secret.as_deref(), Some(secret.as_str()));
    assert!(user.totp_pending.is_none() && user.epoch > epoch);
    assert_eq!(user.recovery_codes.len(), 10);
    // Every session ended, the confirming browser's and the terminal's included.
    for sso in [&sso, &other] {
        assert_eq!(
            f.core.portal_mfa(Some(sso)).unwrap_err().status,
            StatusCode::UNAUTHORIZED
        );
    }
    assert!(f.core.me(text(&password, "session_token")).is_err());
    // The confirming code is spent; a later one signs in with MFA exactly once.
    assert!(f.login(&confirming).is_err());
    assert!(f.core.login("alice".into(), PASSWORD.into(), None).is_err());
    let current = code(&secret, now());
    let signed_in = f.login(&current).unwrap();
    assert_eq!(
        f.core.me(text(&signed_in, "session_token")).unwrap()["mfa"],
        true
    );
    assert!(f.login(&current).is_err());
    // Secrets and codes never reach audit or stored state in clear text.
    let audit = f.audit();
    let stored = serde_json::to_string(&f.snapshot()).unwrap();
    for code in &recovery {
        assert!(!audit.contains(code.as_str()) && !stored.contains(code.as_str()));
    }
    assert!(!audit.contains(&secret));
    for action in [
        "mfa.enroll.begin",
        "mfa.enroll.cancel",
        "mfa.enabled",
        "mfa.recovery_codes.rotate",
    ] {
        assert!(audit.contains(&format!("\"{action}\"")), "{action}");
    }
}

#[test]
fn stale_password_only_and_terminal_sessions_cannot_change_factors() {
    let f = Fixture::new();
    let (secret, recovery) = f.enable();
    let sso = f.browser(Some(code(&secret, now())));
    let terminal = f.terminal_browser(code(&secret, now() + 30));
    let id = f.user().id;
    let attempts = |sso: &str| {
        let before = f.snapshot();
        let refused = [
            error_code(f.core.portal_totp_start(Some(sso), &id, true)),
            error_code(f.core.portal_totp_remove(Some(sso), &id)),
            error_code(f.core.portal_recovery_codes(Some(sso), &id)),
        ];
        assert_eq!(f.snapshot(), before, "a refused change writes nothing");
        refused
    };
    f.session(&sso, |session| session.identity.auth_time = now() - 301);
    assert_eq!(attempts(&sso), ["reauthentication_required"; 3]);
    // Once the account has a factor, a recent sign-in must also have used one.
    f.session(&sso, |session| {
        session.identity.auth_time = now();
        session.identity.mfa = false;
    });
    assert_eq!(attempts(&sso), ["mfa_required"; 3]);
    // A terminal approval, even a fresh MFA one, never authorizes browser factor changes.
    let status = f.core.portal_mfa(Some(&terminal)).unwrap();
    assert_eq!(
        [&status["terminal"], &status["fresh"], &status["mfa"]],
        [&json!(true); 3]
    );
    assert_eq!(attempts(&terminal), ["reauthentication_required"; 3]);
    assert_eq!(f.user().totp_secret.as_deref(), Some(secret.as_str()));
    // Enrolling while an app is enabled needs the explicit replacement flow.
    let fresh = f.browser(Some(recovery[0].clone()));
    assert_eq!(error_code(f.start(&fresh, false)), "conflict");
}

#[test]
fn replacement_keeps_the_old_app_until_confirmed_then_retires_it_and_its_codes() {
    let f = Fixture::new();
    let (old, old_codes) = f.enable();
    let sso = f.browser(Some(code(&old, now())));
    let terminal = text(&f.login(&code(&old, now() + 30)).unwrap(), "session_token").to_owned();
    let started = f.start(&sso, true).unwrap();
    assert_eq!(started["replace"], true);
    let new = text(&started, "secret").to_owned();
    assert_ne!(new, old);
    // Until confirmation the enabled app and its recovery codes keep working.
    assert!(f.login(&old_codes[0]).is_ok());
    assert_eq!(f.user().totp_secret.as_deref(), Some(old.as_str()));
    assert_eq!(
        error_code(f.confirm(&sso, &code(&old, now() - 30))),
        "invalid_code"
    );

    let reply = f.confirm(&sso, &code(&new, now() - 30)).unwrap();
    assert_eq!(reply.body["status"], "replaced");
    assert!(clears_sso(&reply));
    let new_codes = codes(&reply.body);
    assert!(new_codes.iter().all(|code| !old_codes.contains(code)));
    assert!(f.core.me(&terminal).is_err());
    assert!(f.login(&code(&old, now())).is_err());
    assert!(f.login(&old_codes[1]).is_err());
    assert!(f.login(&code(&new, now())).is_ok());
    assert!(f.login(&new_codes[0]).is_ok());
    assert!(f.login(&new_codes[0]).is_err());
    assert!(f.audit().contains("\"mfa.replace\""));
}

#[test]
fn removal_needs_fresh_mfa_and_retires_the_app_codes_and_sessions() {
    let f = Fixture::new();
    let (secret, recovery) = f.enable();
    let sso = f.browser(Some(code(&secret, now())));
    let terminal = text(
        &f.login(&code(&secret, now() + 30)).unwrap(),
        "session_token",
    )
    .to_owned();
    assert_eq!(
        error_code(f.core.portal_totp_remove(Some(&sso), "someone-else")),
        "account_mismatch"
    );
    let epoch = f.user().epoch;
    let reply = f.core.portal_totp_remove(Some(&sso), &f.user().id).unwrap();
    assert_eq!(reply.body["removed"], true);
    assert!(clears_sso(&reply));
    let user = f.user();
    assert!(user.totp_secret.is_none() && user.totp_pending.is_none());
    assert!(user.recovery_codes.is_empty() && user.epoch > epoch);
    assert!(f.core.me(&terminal).is_err());
    assert_eq!(
        f.core.portal_mfa(Some(&sso)).unwrap_err().status,
        StatusCode::UNAUTHORIZED
    );
    // The password alone signs in again, and a former recovery code adds no MFA.
    let session = f.login(&recovery[0]).unwrap();
    assert_eq!(
        f.core.me(text(&session, "session_token")).unwrap()["mfa"],
        false
    );
    let sso = f.browser(None);
    assert_eq!(
        error_code(f.core.portal_totp_remove(Some(&sso), &f.user().id)),
        "conflict"
    );
    assert!(f.audit().contains("\"mfa.disabled\""));
}

#[test]
fn rotation_replaces_every_code_once_and_keeps_sessions() {
    let f = Fixture::new();
    let (secret, first) = f.enable();
    let sso = f.browser(Some(code(&secret, now())));
    let rotated = codes(
        &f.core
            .portal_recovery_codes(Some(&sso), &f.user().id)
            .unwrap(),
    );
    assert_eq!(rotated.len(), 10);
    assert!(rotated.iter().all(|code| !first.contains(code)));
    // Rotation adds no way in, so this session continues.
    let status = f.core.portal_mfa(Some(&sso)).unwrap();
    assert_eq!(status["recovery_codes_remaining"], 10);
    assert!(f.login(&first[0]).is_err());
    assert!(f.login(&rotated[0]).is_ok());
    assert!(f.login(&rotated[0]).is_err());
    assert_eq!(
        f.core.portal_mfa(Some(&sso)).unwrap()["recovery_codes_remaining"],
        9
    );
    let audit = f.audit();
    assert!(
        rotated
            .iter()
            .chain(&first)
            .all(|code| !audit.contains(code.as_str()))
    );
}

#[test]
fn concurrent_confirmations_accept_at_most_one() {
    let f = Fixture::new();
    let sso = f.browser(None);
    let secret = text(&f.start(&sso, false).unwrap(), "secret").to_owned();
    let code = code(&secret, now() - 30);
    let user = f.user().id;
    let barrier = Arc::new(Barrier::new(2));
    let results: Vec<_> = (0..2)
        .map(|_| {
            let (core, barrier) = (f.core.clone(), barrier.clone());
            let (sso, code, user) = (sso.clone(), code.clone(), user.clone());
            std::thread::spawn(move || {
                barrier.wait();
                core.portal_totp_confirm(Some(&sso), &user, &code).is_ok()
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|ok| **ok).count(), 1);
    assert_eq!(f.user().recovery_codes.len(), 10);
}

#[test]
fn expired_enrollments_are_cleared_by_maintenance() {
    let f = Fixture::new();
    let sso = f.browser(None);
    let secret = text(&f.start(&sso, false).unwrap(), "secret").to_owned();
    let id = f.user().id;
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &id)?.unwrap();
            user.totp_pending.as_mut().unwrap().1 = now() - 1;
            tx.put("users", &id, &user)?;
            let mut binding: Value = tx.get("totp_enrollments", &id)?.unwrap();
            binding["expires_at"] = json!(now() - 1);
            tx.put("totp_enrollments", &id, &binding)
        })
        .unwrap();
    assert_eq!(
        error_code(f.confirm(&sso, &code(&secret, now()))),
        "enrollment_expired"
    );
    f.core.cleanup().unwrap();
    assert!(f.user().totp_pending.is_none());
    assert!(
        f.core
            .store
            .list::<Value>("totp_enrollments")
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn portal_routes_need_the_session_cookie_and_the_write_guard() {
    let f = Fixture::new();
    let sso = f.browser(None);
    let app: Router = riauth::api::router(f.core.clone());
    let request = |method: &str, uri: &str, guarded: bool, body: Value| {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("cookie", format!("riauth_sso={sso}"))
            .header("content-type", "application/json");
        if guarded {
            request = request
                .header("origin", ORIGIN)
                .header("x-riauth-portal", "1")
                .header("sec-fetch-site", "same-origin");
        }
        request.body(Body::from(body.to_string())).unwrap()
    };
    let call = |request: Request<Body>| {
        let app = app.clone();
        async move {
            let response = app.oneshot(request).await.unwrap();
            let (status, headers) = (response.status(), response.headers().clone());
            let body = response.into_body().collect().await.unwrap().to_bytes();
            (
                status,
                headers,
                serde_json::from_slice::<Value>(&body).unwrap_or(Value::Null),
            )
        }
    };
    let anonymous = Request::builder()
        .uri("/api/portal/mfa")
        .body(Body::empty())
        .unwrap();
    assert_eq!(call(anonymous).await.0, StatusCode::UNAUTHORIZED);
    let user = json!({"expected_user_id": f.user().id});
    for uri in [
        "/api/portal/mfa/totp/start",
        "/api/portal/mfa/recovery-codes",
        "/api/portal/mfa/totp/remove",
    ] {
        assert_eq!(
            call(request("POST", uri, false, user.clone())).await.0,
            StatusCode::FORBIDDEN,
            "{uri}"
        );
    }
    assert!(f.user().totp_pending.is_none());
    let (status, headers, started) = call(request(
        "POST",
        "/api/portal/mfa/totp/start",
        true,
        user.clone(),
    ))
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers["cache-control"], "no-store");
    let (status, _, body) = call(request("GET", "/api/portal/mfa", false, Value::Null)).await;
    assert_eq!(
        (status, &body["enrollment_pending"]),
        (StatusCode::OK, &json!(true))
    );
    let mut confirm = user.clone();
    confirm["code"] = json!(code(text(&started, "secret"), now() - 30));
    let (status, _, body) = call(request(
        "POST",
        "/api/portal/mfa/totp/confirm",
        true,
        confirm,
    ))
    .await;
    assert_eq!(
        (status, &body["status"]),
        (StatusCode::OK, &json!("enabled"))
    );
    assert_eq!(codes(&body).len(), 10);
}
