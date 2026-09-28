//! Browser invitation and email-verification proof boundaries over HTTP.
mod common;

use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use common::{Fixture, PASSWORD as SIGN_IN_PASSWORD};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    api,
    crypto::{digest, now},
    lifecycle::{Invitation, MailConfig, MailSecurity, Purpose},
    model::{Session, User, UserPatch},
};
use serde_json::{Value, json};
use tower::ServiceExt;

const ORIGIN: &str = "http://localhost:9000";
const PASSWORD: &str = "browser-invitation-password-123";

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Value,
    text: String,
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

fn page(method: &str, path: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header("accept", "text/html")
        .body(Body::empty())
        .unwrap()
}

fn completion(path: &str, body: Value, origin: bool) -> Request<Body> {
    let mut request = Request::post(path)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .header("content-type", "application/json");
    if origin {
        request = request.header("origin", ORIGIN);
    }
    request.body(Body::from(body.to_string())).unwrap()
}

fn verify_request(cookie: Option<&str>, origin: Option<&str>) -> Request<Body> {
    let mut request = Request::post("/api/portal/account/verify-request")
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin");
    if let Some(origin) = origin {
        request = request.header("origin", origin);
    }
    if let Some(cookie) = cookie {
        request = request.header("cookie", format!("riauth_sso={cookie}"));
    }
    request.body(Body::empty()).unwrap()
}

fn mail_fixture() -> Fixture {
    let mut f = Fixture::new();
    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
    f
}

fn mails(f: &Fixture, username: &str) -> Vec<(String, String)> {
    let marker = format!("Account: {username}");
    f.core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .filter_map(|(_, delivery)| {
            let body = delivery["body"].as_str()?;
            let code = body.lines().find(|line| line.starts_with("ri_mail_"))?;
            body.contains(&marker)
                .then(|| (body.to_owned(), code.to_owned()))
        })
        .collect()
}

fn mail(f: &Fixture, username: &str) -> (String, String) {
    mails(f, username)
        .into_iter()
        .next()
        .expect("synthetic undelivered mail")
}

fn new_mail(f: &Fixture, username: &str, old_code: &str) -> (String, String) {
    mails(f, username)
        .into_iter()
        .find(|(_, code)| code != old_code)
        .expect("new synthetic undelivered mail")
}

fn assert_fragment_link(body: &str, code: &str, purpose: &str) {
    let link = body
        .lines()
        .find_map(|line| line.strip_prefix("Open in your browser: "))
        .expect("browser link");
    let url = url::Url::parse(link).unwrap();
    assert_eq!(url.path(), format!("/account/{purpose}"));
    assert_eq!(url.fragment(), Some(format!("token={code}").as_str()));
    assert!(url.query().is_none(), "proof must not appear in a query");
}

fn proof(f: &Fixture, code: &str) -> Option<Value> {
    f.core.store.get("account_proofs", &digest(code)).unwrap()
}

fn user(f: &Fixture, username: &str) -> User {
    let id: String = f.core.store.get("usernames", username).unwrap().unwrap();
    f.core.store.get("users", &id).unwrap().unwrap()
}

fn invitation(username: &str) -> Invitation {
    Invitation {
        username: username.into(),
        email: format!("{username}@example.test"),
        display_name: "Invited Person".into(),
        groups: Default::default(),
    }
}

fn invite(f: &Fixture, username: &str) -> String {
    f.core
        .account_invite(&f.admin, invitation(username))
        .unwrap();
    let (body, code) = mail(f, username);
    assert_fragment_link(&body, &code, "accept");
    code
}

fn browser_cookie(f: &Fixture, username: &str, password: &str) -> String {
    let reply = f
        .core
        .portal_password(None, username.into(), password.into(), None, false)
        .unwrap();
    reply
        .cookies
        .iter()
        .find_map(|cookie| {
            cookie
                .split(';')
                .next()
                .and_then(|pair| pair.strip_prefix("riauth_sso="))
        })
        .expect("browser SSO cookie")
        .to_owned()
}

#[tokio::test]
async fn invitation_scanner_get_and_head_leave_proof_for_explicit_post() {
    let f = mail_fixture();
    let code = invite(&f, "newcomer");
    let before = proof(&f, &code).unwrap();
    let app = api::router(f.core.clone());

    for method in ["GET", "HEAD"] {
        for path in ["/account/accept", "/account/verify"] {
            let reply = call(&app, page(method, path)).await;
            assert_eq!(reply.status, StatusCode::OK);
            assert_eq!(reply.headers["cache-control"], "no-store");
            assert_eq!(reply.headers["referrer-policy"], "no-referrer");
            assert_eq!(proof(&f, &code), Some(before.clone()));
            if method == "GET" {
                assert!(reply.text.contains("account.js"));
                assert!(!reply.text.contains(&code));
            }
        }
    }
    assert!(!user(&f, "newcomer").enabled);

    let body = json!({"token":code,"password":PASSWORD});
    let unguarded = call(
        &app,
        completion("/api/portal/account/accept", body.clone(), false),
    )
    .await;
    assert_eq!(unguarded.status, StatusCode::FORBIDDEN);
    assert_eq!(proof(&f, &code), Some(before.clone()));

    let wrong_purpose = call(
        &app,
        completion("/api/portal/account/verify", json!({"token":code}), true),
    )
    .await;
    assert_eq!(wrong_purpose.body["error"], "account_code_invalid");
    assert_eq!(proof(&f, &code), Some(before));

    let first = call(
        &app,
        completion("/api/portal/account/accept", body.clone(), true),
    )
    .await;
    assert_eq!(first.status, StatusCode::OK);
    assert_eq!(first.body["completed"], true);
    assert!(proof(&f, &code).is_none());
    let accepted = user(&f, "newcomer");
    assert!(accepted.enabled && accepted.email_verified && !accepted.admin);
    let browser = browser_cookie(&f, "newcomer", PASSWORD);
    let signed_in = call(
        &app,
        Request::get("/api/portal")
            .header("cookie", format!("riauth_sso={browser}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(signed_in.status, StatusCode::OK);
    assert_eq!(signed_in.body["user"]["username"], "newcomer");

    let replay = call(&app, completion("/api/portal/account/accept", body, true)).await;
    assert_eq!(replay.status, StatusCode::GONE);
    assert_eq!(replay.body["error"], "account_code_used");
}

#[tokio::test]
async fn verification_scanner_get_leaves_proof_and_post_verifies_once() {
    let f = mail_fixture();
    let session = f.user("alice");
    f.core.account_verify_request(&session).unwrap();
    let (body, code) = mail(&f, "alice");
    assert_fragment_link(&body, &code, "verify");
    let before = proof(&f, &code).unwrap();
    let app = api::router(f.core.clone());

    for method in ["GET", "HEAD"] {
        let reply = call(&app, page(method, "/account/verify")).await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(proof(&f, &code), Some(before.clone()));
    }
    assert!(!user(&f, "alice").email_verified);

    let submitted = json!({"token":code});
    let first = call(
        &app,
        completion("/api/portal/account/verify", submitted.clone(), true),
    )
    .await;
    assert_eq!(first.status, StatusCode::OK);
    assert_eq!(first.body["completed"], true);
    assert!(user(&f, "alice").email_verified);
    assert!(proof(&f, &code).is_none());

    let replay = call(
        &app,
        completion("/api/portal/account/verify", submitted, true),
    )
    .await;
    assert_eq!(replay.status, StatusCode::GONE);
    assert_eq!(replay.body["error"], "account_code_used");
}

#[tokio::test]
async fn signed_in_browser_requests_verification_with_guard_cooldown_and_live_identity() {
    let f = mail_fixture();
    let terminal = f.user("alice");
    let terminal_id = f.core.me(&terminal).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let alice_id = user(&f, "alice").id;
    let browser = browser_cookie(&f, "alice", SIGN_IN_PASSWORD);
    let app = api::router(f.core.clone());
    let delivery_count = || f.core.store.list::<Value>("mail_deliveries").unwrap().len();
    assert_eq!(delivery_count(), 0);

    for method in ["GET", "HEAD"] {
        let page_reply = call(&app, page(method, "/account/verify")).await;
        assert_eq!(page_reply.status, StatusCode::OK);
        let request = Request::builder()
            .method(method)
            .uri("/api/portal/account/verify-request")
            .header("cookie", format!("riauth_sso={browser}"))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            call(&app, request).await.status,
            StatusCode::METHOD_NOT_ALLOWED
        );
    }
    assert_eq!(delivery_count(), 0);

    assert_eq!(
        call(&app, verify_request(None, Some(ORIGIN))).await.status,
        StatusCode::UNAUTHORIZED
    );
    for origin in [None, Some("https://elsewhere.example")] {
        assert_eq!(
            call(&app, verify_request(Some(&browser), origin))
                .await
                .status,
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(delivery_count(), 0);

    let queued = call(&app, verify_request(Some(&browser), Some(ORIGIN))).await;
    assert_eq!(queued.status, StatusCode::OK);
    assert_eq!(queued.body, json!({"accepted":true,"status":"queued"}));
    let (body, code) = mail(&f, "alice");
    assert_fragment_link(&body, &code, "verify");
    assert!(!queued.text.contains(&code));
    assert_eq!(proof(&f, &code).unwrap()["user_id"], alice_id);
    assert_eq!(delivery_count(), 1);

    let cooldown = call(&app, verify_request(Some(&browser), Some(ORIGIN))).await;
    assert_eq!(cooldown.status, StatusCode::OK);
    assert_eq!(cooldown.body, json!({"accepted":true,"status":"cooldown"}));
    assert_eq!(delivery_count(), 1);
    assert!(proof(&f, &code).is_some());

    let verified = call(
        &app,
        completion("/api/portal/account/verify", json!({"token":code}), true),
    )
    .await;
    assert_eq!(verified.status, StatusCode::OK);
    assert_eq!(verified.body["completed"], true);
    assert!(user(&f, "alice").email_verified);
    assert_eq!(
        call(&app, verify_request(Some(&browser), Some(ORIGIN)))
            .await
            .body["status"],
        "already_verified"
    );
    assert_eq!(delivery_count(), 1);

    f.core
        .store
        .write(|tx| {
            let mut browser_session = tx
                .list::<Session>("sessions")?
                .into_iter()
                .map(|(_, session)| session)
                .find(|session| session.identity.user_id == alice_id && session.id != terminal_id)
                .expect("separate browser session");
            browser_session.identity.auth_time = now().saturating_sub(301);
            tx.put("sessions", &browser_session.id, &browser_session)
        })
        .unwrap();
    let stale = call(&app, verify_request(Some(&browser), Some(ORIGIN))).await;
    assert_eq!(stale.status, StatusCode::FORBIDDEN);
    assert_eq!(stale.body["error"], "reauthentication_required");
    assert_eq!(delivery_count(), 1);
}

#[tokio::test]
async fn expired_and_revoked_invitations_show_distinct_errors_without_account_changes() {
    let f = mail_fixture();
    let expired = invite(&f, "expired");
    let revoked = invite(&f, "revoked");
    f.core
        .store
        .write(|tx| {
            let key = digest(&expired);
            let mut record: Value = tx.get("account_proofs", &key)?.unwrap();
            record["expires_at"] = json!(now());
            tx.put("account_proofs", &key, &record)
        })
        .unwrap();
    f.core
        .account_invitation_revoke(&f.admin, "revoked")
        .unwrap();
    let app = api::router(f.core.clone());

    for (code, error) in [
        (&expired, "account_code_expired"),
        (&revoked, "account_code_revoked"),
    ] {
        let scanner = call(&app, page("GET", "/account/accept")).await;
        assert_eq!(scanner.status, StatusCode::OK);
        let result = call(
            &app,
            completion(
                "/api/portal/account/accept",
                json!({"token":code,"password":PASSWORD}),
                true,
            ),
        )
        .await;
        assert_eq!(result.status, StatusCode::GONE);
        assert_eq!(result.body["error"], error);
    }
    for username in ["expired", "revoked"] {
        let pending = user(&f, username);
        assert!(!pending.enabled && !pending.email_verified);
        assert!(pending.password_hash.is_empty());
        assert!(
            f.core
                .login(username.into(), PASSWORD.into(), None)
                .is_err()
        );
    }
}

#[tokio::test]
async fn pending_invitations_reissue_after_expiry_or_revocation_but_not_account_change() {
    let f = mail_fixture();
    let unprivileged = f.user("ordinary");
    let app = api::router(f.core.clone());

    for (username, revoked) in [("expired_reissue", false), ("revoked_reissue", true)] {
        let old_code = invite(&f, username);
        let id = user(&f, username).id;
        if revoked {
            f.core
                .account_invitation_revoke(&f.admin, username)
                .unwrap();
        } else {
            f.core
                .store
                .write(|tx| {
                    let key = digest(&old_code);
                    let mut record: Value = tx.get("account_proofs", &key)?.unwrap();
                    record["expires_at"] = json!(now());
                    tx.put("account_proofs", &key, &record)
                })
                .unwrap();
        }
        let old_result = call(
            &app,
            completion(
                "/api/portal/account/accept",
                json!({"token":old_code,"password":PASSWORD}),
                true,
            ),
        )
        .await;
        assert_eq!(old_result.status, StatusCode::GONE);
        assert_eq!(
            old_result.body["error"],
            if revoked {
                "account_code_revoked"
            } else {
                "account_code_expired"
            }
        );

        assert_eq!(
            f.core
                .account_invite(&unprivileged, invitation(username))
                .unwrap_err()
                .status,
            StatusCode::FORBIDDEN
        );
        let issued = f
            .core
            .account_invite(&f.admin, invitation(username))
            .unwrap();
        assert_eq!(issued["user"]["id"], id);
        assert_eq!(user(&f, username).id, id);
        let (body, new_code) = new_mail(&f, username, &old_code);
        assert_fragment_link(&body, &new_code, "accept");
        assert!(proof(&f, &old_code).is_none());
        assert!(proof(&f, &new_code).is_some());

        let old_replay = call(
            &app,
            completion(
                "/api/portal/account/accept",
                json!({"token":old_code,"password":PASSWORD}),
                true,
            ),
        )
        .await;
        assert_eq!(old_replay.status, StatusCode::GONE);
        if revoked {
            assert_eq!(old_replay.body["error"], "account_code_revoked");
        }
        let accepted = call(
            &app,
            completion(
                "/api/portal/account/accept",
                json!({"token":new_code,"password":PASSWORD}),
                true,
            ),
        )
        .await;
        assert_eq!(accepted.status, StatusCode::OK);
        assert_eq!(accepted.body["completed"], true);
        assert_eq!(user(&f, username).id, id);
        assert!(user(&f, username).enabled);
        assert_eq!(
            call(
                &app,
                completion(
                    "/api/portal/account/accept",
                    json!({"token":new_code,"password":PASSWORD}),
                    true,
                ),
            )
            .await
            .body["error"],
            "account_code_used"
        );
        assert_eq!(
            f.core
                .account_invite(&f.admin, invitation(username))
                .unwrap_err()
                .status,
            StatusCode::CONFLICT
        );
    }

    assert_eq!(
        f.core
            .account_invite(&f.admin, invitation("ordinary"))
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    let changed_code = invite(&f, "credentialed");
    let changed_id = user(&f, "credentialed").id;
    f.core
        .update_user(
            &f.admin,
            "credentialed",
            UserPatch {
                password: Some(SIGN_IN_PASSWORD.into()),
                ..Default::default()
            },
        )
        .unwrap();
    let delivery_count = f.core.store.list::<Value>("mail_deliveries").unwrap().len();
    assert_eq!(
        f.core
            .account_invite(&f.admin, invitation("credentialed"))
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(user(&f, "credentialed").id, changed_id);
    assert_eq!(
        f.core.store.list::<Value>("mail_deliveries").unwrap().len(),
        delivery_count
    );
    assert!(proof(&f, &changed_code).is_some());
    let toggled_code = invite(&f, "toggled");
    for enabled in [true, false] {
        f.core
            .update_user(
                &f.admin,
                "toggled",
                UserPatch {
                    enabled: Some(enabled),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    assert_eq!(
        f.core
            .account_invite(&f.admin, invitation("toggled"))
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    assert!(proof(&f, &toggled_code).is_some());
    assert!(
        f.core
            .audit_events(&f.admin, 1000)
            .unwrap()
            .to_string()
            .contains("user.invitation.reissue")
    );
}

#[test]
fn restored_pending_invitation_keeps_provenance_for_authorized_reissue() {
    let f = mail_fixture();
    let old_code = invite(&f, "restored_pending");
    let before = user(&f, "restored_pending");
    f.core
        .account_invitation_revoke(&f.admin, "restored_pending")
        .unwrap();
    let key = riauth::crypto::random_token("");
    let backup = f.core.backup(&f.admin, &key).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let archive = directory.path().join("backup.json");
    let key_file = directory.path().join("backup.key");
    let output = directory.path().join("restored");
    std::fs::write(&archive, backup.to_string()).unwrap();
    riauth::config::write_private(&key_file, key.as_bytes(), false).unwrap();

    let result = riauth::operations::restore(&archive, &key_file, &output, None).unwrap();
    assert!(result["recovery"]["unclassified"].as_array().unwrap().is_empty());
    let restored = riauth::core::Core::open(
        riauth::config::Config::load(&output.join("riauth.toml")).unwrap(),
    )
    .unwrap();
    let after_id: String = restored.store.get("usernames", "restored_pending").unwrap().unwrap();
    let after: User = restored.store.get("users", &after_id).unwrap().unwrap();
    assert_eq!(after.id, before.id);
    assert_eq!(after.epoch, before.epoch + riauth::recovery::STRIDE);
    assert!(restored
        .store
        .get::<Value>("account_proofs", &digest(&old_code))
        .unwrap()
        .is_none());
    assert!(restored
        .store
        .get::<Value>("account_proof_outcomes", &digest(&old_code))
        .unwrap()
        .is_some());

    let admin = restored
        .login("admin".into(), SIGN_IN_PASSWORD.into(), None)
        .unwrap();
    let admin_token = admin["session_token"].as_str().unwrap();
    let reissued = restored
        .account_invite(admin_token, invitation("restored_pending"))
        .unwrap();
    assert_eq!(reissued["user"]["id"], before.id);
    assert_eq!(reissued["delivery_queued"], true);
}

#[test]
fn invitation_review_shows_only_what_the_reader_may_read() {
    let f = mail_fixture();
    for group in ["engineering", "finance"] {
        f.core.create_group(&f.admin, group).unwrap();
    }
    f.core
        .account_invite(
            &f.admin,
            Invitation {
                groups: ["engineering", "finance"].map(String::from).into(),
                ..invitation("invited")
            },
        )
        .unwrap();
    let scoped = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "reviewer".into(),
                ttl: 3600,
                permissions: [
                    ("user.read", "user/invited"),
                    ("group.read", "group/engineering"),
                ]
                .map(|(action, resource)| Permission {
                    action: action.into(),
                    resource: resource.into(),
                })
                .into(),
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let review = |token: &str| {
        let listed = f.core.account_invitations(token).unwrap();
        let entry = &listed["invitations"][0];
        assert_eq!(entry["user"]["username"], "invited");
        (
            entry["groups"].clone(),
            entry["invited_by"].clone(),
            entry["delivery"]["status"].clone(),
        )
    };
    let admin_id = f.core.me(&f.admin).unwrap()["user"]["id"].clone();
    assert_eq!(
        review(&f.admin),
        (json!(["engineering", "finance"]), admin_id, json!("queued"))
    );
    // A reader scoped to the invitee sees neither the group, the administrator who sent the
    // invitation, nor delivery state that it has no permission to read.
    assert_eq!(
        review(&scoped),
        (json!(["engineering"]), Value::Null, Value::Null)
    );
}

#[test]
fn invitation_review_blocks_a_link_whose_inviter_lost_authority() {
    let f = mail_fixture();
    f.core
        .create_user(
            &f.admin,
            riauth::model::NewUser {
                username: "inviter".into(),
                password: SIGN_IN_PASSWORD.into(),
                email: None,
                display_name: "Inviter".into(),
                admin: true,
            },
        )
        .unwrap();
    let inviter = f
        .core
        .login("inviter".into(), SIGN_IN_PASSWORD.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    f.core
        .account_invite(&inviter, invitation("invited"))
        .unwrap();
    let status =
        || f.core.account_invitations(&f.admin).unwrap()["invitations"][0]["status"].clone();
    assert_eq!(status(), "pending");
    f.core
        .update_user(
            &f.admin,
            "inviter",
            UserPatch {
                admin: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    // Acceptance rechecks the inviter's authority, so review must not report a working link.
    assert_eq!(status(), "blocked");
    let (_, code) = mail(&f, "invited");
    let refused = f
        .core
        .account_complete(
            code,
            Purpose::Invite,
            Some("invited-person-password-2026".into()),
        )
        .unwrap_err();
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
}
