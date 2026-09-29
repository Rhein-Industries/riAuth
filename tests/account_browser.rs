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
async fn invitation_writes_require_retry_binding_across_browser_and_bearer() {
    let mut f = mail_fixture();
    // This regression sends more requests than the public account throttle allows per minute.
    f.core.config.rate_limits.insert("account".into(), 100);
    let outsider = f.user("outsider");
    let cookie = browser_cookie(&f, "admin", SIGN_IN_PASSWORD);
    let app = api::router(f.core.clone());
    let revision = || {
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let audit_count = |action: &str| {
        f.core
            .audit_events(&f.admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action)
            .count()
    };
    let write = |method: &str,
                 path: &str,
                 browser: bool,
                 actor: &str,
                 key: Option<&str>,
                 expected: Option<u64>,
                 body: Option<Value>| {
        let mut request = Request::builder().method(method).uri(path);
        if browser {
            request = request
                .header("cookie", format!("riauth_sso={cookie}"))
                .header("x-riauth-portal", "1")
                .header("origin", ORIGIN);
        } else {
            request = request.header("authorization", format!("Bearer {actor}"));
        }
        if let Some(key) = key {
            request = request.header("idempotency-key", key);
        }
        if let Some(expected) = expected {
            request = request.header("if-match", format!("\"{expected}\""));
        }
        let body = match body {
            Some(body) => {
                request = request.header("content-type", "application/json");
                Body::from(body.to_string())
            }
            None => Body::empty(),
        };
        request.body(body).unwrap()
    };
    let new_person = || json!(invitation("invited"));
    let at = revision();
    let before = f.snapshot().unwrap();
    for (key, expected) in [(None, None), (Some("key-only"), None), (None, Some(at))] {
        for browser in [true, false] {
            let create_path = if browser {
                "/api/admin/invitations"
            } else {
                "/api/account/invitations"
            };
            let revoke_path = if browser {
                "/api/admin/invitations/invited"
            } else {
                "/api/account/invitations/invited"
            };
            assert_eq!(
                call(
                    &app,
                    write(
                        "POST",
                        create_path,
                        browser,
                        &f.admin,
                        key,
                        expected,
                        Some(new_person())
                    )
                )
                .await
                .status,
                StatusCode::PRECONDITION_REQUIRED
            );
            assert_eq!(
                call(
                    &app,
                    write(
                        "DELETE",
                        revoke_path,
                        browser,
                        &f.admin,
                        key,
                        expected,
                        None
                    )
                )
                .await
                .status,
                StatusCode::PRECONDITION_REQUIRED
            );
        }
    }
    assert_eq!(
        call(
            &app,
            write(
                "POST",
                "/api/account/invitations",
                false,
                &outsider,
                Some("outsider-invite"),
                Some(at),
                Some(new_person())
            )
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    f.assert_http_mutation_snapshot(&before);

    let browser_create = || {
        write(
            "POST",
            "/api/admin/invitations",
            true,
            &f.admin,
            Some("browser-invite"),
            Some(at),
            Some(new_person()),
        )
    };
    let first = call(&app, browser_create()).await;
    assert_eq!(first.status, StatusCode::OK, "{}", first.text);
    let first_code = mail(&f, "invited").1;
    let committed = f.snapshot().unwrap();
    assert_eq!(call(&app, browser_create()).await.body, first.body);
    f.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("user.invite"), 1);
    assert_eq!(mails(&f, "invited").len(), 1);
    assert_eq!(revision(), at + 1);
    let mut changed = new_person();
    changed["email"] = json!("changed@example.test");
    assert_eq!(
        call(
            &app,
            write(
                "POST",
                "/api/admin/invitations",
                true,
                &f.admin,
                Some("browser-invite"),
                Some(at),
                Some(changed)
            )
        )
        .await
        .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(
            &app,
            write(
                "POST",
                "/api/account/invitations",
                false,
                &f.admin,
                Some("stale-invite"),
                Some(at),
                Some(new_person())
            )
        )
        .await
        .status,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&committed);

    let reissue_at = revision();
    let bearer_reissue = || {
        write(
            "POST",
            "/api/account/invitations",
            false,
            &f.admin,
            Some("bearer-reissue"),
            Some(reissue_at),
            Some(new_person()),
        )
    };
    let reissued = call(&app, bearer_reissue()).await;
    assert_eq!(reissued.status, StatusCode::OK, "{}", reissued.text);
    let committed = f.snapshot().unwrap();
    assert_eq!(call(&app, bearer_reissue()).await.body, reissued.body);
    f.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("user.invitation.reissue"), 1);
    assert_eq!(mails(&f, "invited").len(), 2);
    assert!(proof(&f, &first_code).is_none());

    let revoke_at = revision();
    let browser_revoke = || {
        write(
            "DELETE",
            "/api/admin/invitations/invited",
            true,
            &f.admin,
            Some("browser-revoke"),
            Some(revoke_at),
            None,
        )
    };
    let revoked = call(&app, browser_revoke()).await;
    assert_eq!(revoked.status, StatusCode::OK, "{}", revoked.text);
    let committed = f.snapshot().unwrap();
    assert_eq!(call(&app, browser_revoke()).await.body, revoked.body);
    f.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("user.invitation.revoke"), 1);
    assert_eq!(revision(), revoke_at + 1);
    assert_eq!(
        call(
            &app,
            write(
                "DELETE",
                "/api/account/invitations/invited",
                false,
                &f.admin,
                Some("stale-revoke"),
                Some(revoke_at),
                None
            )
        )
        .await
        .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(
            &app,
            write(
                "DELETE",
                "/api/account/invitations/other",
                false,
                &f.admin,
                Some("browser-revoke"),
                Some(revoke_at),
                None
            )
        )
        .await
        .status,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&committed);

    let script = call(&app, page("GET", "/portal/assets/admin.js")).await;
    assert_eq!(script.status, StatusCode::OK);
    assert!(script.text.contains(
        "`admin/invitations/${seg(username)}`, undefined, { revision: data.revision, key }"
    ));
    assert_eq!(
        script
            .text
            .matches("\"admin/invitations\", body, { revision: data.revision, key }")
            .count(),
        2
    );
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

#[cfg(feature = "platform")]
#[tokio::test]
async fn workflow_invitation_password_is_account_bound_and_one_use() {
    let mut f = mail_fixture();
    f.core.config.password_history = 2;
    f.core.create_group(&f.admin, "invite-team").unwrap();
    f.core
        .account_invite(
            &f.admin,
            Invitation {
                groups: common::strings(&["invite-team"]),
                ..invitation("enroll-owner")
            },
        )
        .unwrap();
    let (_, code) = mail(&f, "enroll-owner");
    let other_code = invite(&f, "enroll-other");
    let before = user(&f, "enroll-owner");
    let other = user(&f, "enroll-other");
    let hash = digest(&code);
    let original = proof(&f, &code).unwrap();
    let complete = || {
        f.core
            .account_complete(code.clone(), Purpose::Invite, Some(PASSWORD.into()))
    };

    // Matching account/email/epoch values still cannot move this secret to a
    // second account's invitation request and reservation.
    let mut retargeted = original.clone();
    retargeted["user_id"] = json!(other.id);
    retargeted["email"] = json!(other.email);
    retargeted["epoch"] = json!(other.epoch);
    f.core
        .store
        .write(|tx| tx.put("account_proofs", &hash, &retargeted))
        .unwrap();
    let snapshot = f.snapshot().unwrap();
    assert!(complete().is_err());
    f.assert_snapshot(&snapshot);
    f.core
        .store
        .write(|tx| tx.put("account_proofs", &hash, &original))
        .unwrap();

    let latest = format!("{}:accept", before.id);
    let admin = user(&f, "admin");
    for (bucket, key, pointer, value) in [
        (
            "account_latest",
            latest.as_str(),
            "",
            json!(digest(&other_code)),
        ),
        ("account_proofs", hash.as_str(), "/expires_at", json!(now())),
        (
            "users",
            before.id.as_str(),
            "/epoch",
            json!(before.epoch + 1),
        ),
        (
            "users",
            before.id.as_str(),
            "/totp_secret",
            json!("existing-factor"),
        ),
        (
            "users",
            before.id.as_str(),
            "/recovery_codes",
            json!(["existing-code"]),
        ),
        (
            "invitation_reservations",
            before.id.as_str(),
            "/username",
            json!("enroll-other"),
        ),
        ("users", admin.id.as_str(), "/admin", json!(false)),
    ] {
        let saved: Value = f.core.store.get(bucket, key).unwrap().unwrap();
        let mut changed = saved.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &changed))
            .unwrap();
        let snapshot = f.snapshot().unwrap();
        assert!(complete().is_err(), "{bucket}{pointer}");
        f.assert_snapshot(&snapshot);
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &saved))
            .unwrap();
    }

    // Password-policy rejection happens inside finalization, after receipts have
    // been prepared. All rows, group membership and the mail secret roll back.
    let prior_hash = riauth::crypto::password_hash(PASSWORD).unwrap();
    f.core
        .store
        .write(|tx| tx.put("password_history", &before.id, &vec![prior_hash]))
        .unwrap();
    let snapshot = f.snapshot().unwrap();
    assert!(complete().is_err());
    f.assert_snapshot(&snapshot);
    assert!(
        f.core
            .store
            .list::<Value>("workflow_runs")
            .unwrap()
            .is_empty()
    );
    f.core
        .store
        .write(|tx| tx.delete("password_history", &before.id))
        .unwrap();

    // A server-side M04 exposure marker is pinned but never cleared by an
    // invitation. Only the independent-recovery path may remove that boundary.
    let exposure = json!({"verified_email":null,"actor_id":admin.id,"at":now()});
    f.core
        .store
        .write(|tx| tx.put("support_credential_exposure", &before.id, &exposure))
        .unwrap();
    let sessions = f.core.store.list::<Value>("sessions").unwrap();
    let staged = f.core.store.list::<Value>("browser_logins").unwrap();
    let codes = f.core.store.list::<Value>("codes").unwrap();
    let app = api::router(f.core.clone());
    let snapshot = f.snapshot().unwrap();
    for method in ["GET", "HEAD"] {
        assert_eq!(
            call(&app, page(method, "/account/accept")).await.status,
            StatusCode::OK
        );
    }
    f.assert_http_mutation_snapshot(&snapshot);
    let request = || {
        completion(
            "/api/portal/account/accept",
            json!({"token":code,"password":PASSWORD}),
            true,
        )
    };
    let (first, second) = tokio::join!(call(&app, request()), call(&app, request()));
    let replies = [first, second];
    assert_eq!(
        replies
            .iter()
            .filter(|r| r.status == StatusCode::OK)
            .count(),
        1
    );
    let winner = replies.iter().find(|r| r.status == StatusCode::OK).unwrap();
    let loser = replies.iter().find(|r| r.status != StatusCode::OK).unwrap();
    assert_eq!(winner.body, json!({"completed":true,"login_required":true}));
    assert!(winner.headers.get("set-cookie").is_none());
    assert_eq!(loser.body["error"], "account_code_used");

    let after = user(&f, "enroll-owner");
    assert!(after.enabled && after.email_verified && !after.admin);
    assert_eq!(after.epoch, before.epoch + 1);
    assert!(riauth::crypto::password_matches(
        PASSWORD,
        &after.password_hash
    ));
    assert!(proof(&f, &code).is_none());
    assert!(proof(&f, &other_code).is_some());
    assert_eq!(user(&f, "enroll-other").epoch, other.epoch);
    assert!(!user(&f, "enroll-other").enabled);
    assert!(
        f.core
            .store
            .get::<Value>("account_latest", &latest)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .get::<Value>("invitation_reservations", &before.id)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        f.core
            .store
            .get::<Value>("support_credential_exposure", &before.id)
            .unwrap(),
        Some(exposure)
    );
    let group: riauth::model::Group = f.core.store.get("groups", "invite-team").unwrap().unwrap();
    assert_eq!(group.members, common::strings(&[&before.id]));
    assert_eq!(f.core.store.list::<Value>("sessions").unwrap(), sessions);
    assert_eq!(
        f.core.store.list::<Value>("browser_logins").unwrap(),
        staged
    );
    assert_eq!(f.core.store.list::<Value>("codes").unwrap(), codes);

    let runs = f.core.store.list::<Value>("workflow_runs").unwrap();
    assert_eq!(runs.len(), 1);
    let (run_id, run) = &runs[0];
    let request_id = format!("accept:{hash}");
    assert_eq!(run["record"]["state"]["outcome"], "enrolled");
    assert!(run["record"]["session"].is_null());
    assert_eq!(run["record"]["account"], before.id);
    assert_eq!(run["record"]["account_epoch"], before.epoch);
    assert_eq!(run["record"]["request"], request_id);
    assert_eq!(run["credential_mutation"]["from_epoch"], before.epoch);
    assert_eq!(run["credential_mutation"]["to_epoch"], after.epoch);
    assert_eq!(run["credential_mutation"]["invitation_request"], request_id);
    assert_eq!(run["credential_mutation"]["credential"], "password");
    assert!(run.get("authorization_response").is_none());
    let receipts = f.core.store.list::<Value>("workflow_evidence").unwrap();
    assert_eq!(receipts.len(), 2);
    for (_, receipt) in &receipts {
        assert_eq!(receipt["account"], before.id);
        assert_eq!(receipt["account_epoch"], before.epoch);
        assert_eq!(receipt["request"], request_id);
        assert_eq!(receipt["run"], *run_id);
        assert_eq!(receipt["binding"], run["record"]["binding"]);
        assert_eq!(receipt["attempt"], 1);
        assert_eq!(receipt["consumed"], true);
        assert!(receipt["session"].is_null());
        assert!(
            receipt["verified_at"].as_u64().unwrap()
                >= run["record"]["started_at"].as_u64().unwrap()
        );
    }
    let snapshot = f.snapshot().unwrap();
    assert_eq!(complete().unwrap_err().code, "account_code_used");
    f.assert_snapshot(&snapshot);
    assert!(
        f.core
            .login("enroll-owner".into(), PASSWORD.into(), None)
            .is_ok()
    );
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
    assert!(
        result["recovery"]["unclassified"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let restored = riauth::core::Core::open(
        riauth::config::Config::load(&output.join("riauth.toml")).unwrap(),
    )
    .unwrap();
    let after_id: String = restored
        .store
        .get("usernames", "restored_pending")
        .unwrap()
        .unwrap();
    let after: User = restored.store.get("users", &after_id).unwrap().unwrap();
    assert_eq!(after.id, before.id);
    assert_eq!(after.epoch, before.epoch + riauth::recovery::STRIDE);
    assert!(
        restored
            .store
            .get::<Value>("account_proofs", &digest(&old_code))
            .unwrap()
            .is_none()
    );
    assert!(
        restored
            .store
            .get::<Value>("account_proof_outcomes", &digest(&old_code))
            .unwrap()
            .is_some()
    );

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
