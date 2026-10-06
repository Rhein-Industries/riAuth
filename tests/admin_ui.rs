mod common;

use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::core::Core;
use serde_json::{Value, json};
use tower::ServiceExt;

fn sso_cookie(core: &Core, session: &str) -> String {
    let request = core.portal_sign_in().unwrap();
    core.portal_decide(session, request.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = request.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    let response = core
        .portal_poll(request.body["id"].as_str().unwrap(), Some(binding))
        .unwrap();
    response
        .cookies
        .iter()
        .find(|cookie| cookie.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_owned()
}

fn origin(core: &Core) -> String {
    url::Url::parse(&core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization()
}

#[derive(Default)]
struct Call<'a> {
    method: &'a str,
    cookie: Option<&'a str>,
    bearer: Option<&'a str>,
    portal: bool,
    origin: Option<&'a str>,
    revision: Option<u64>,
    key: Option<&'a str>,
    body: Option<Value>,
}

#[cfg(feature = "platform")]
fn access_write<'a>(
    cookie: &'a str,
    origin: &'a str,
    key: Option<&'a str>,
    revision: Option<u64>,
) -> Call<'a> {
    Call {
        method: "POST",
        cookie: Some(cookie),
        portal: true,
        origin: Some(origin),
        key,
        revision,
        ..Default::default()
    }
}

async fn send(app: &axum::Router, uri: &str, call: Call<'_>) -> (StatusCode, HeaderMap, Value) {
    let mut request = Request::builder()
        .method(if call.method.is_empty() {
            "GET"
        } else {
            call.method
        })
        .uri(uri);
    if let Some(cookie) = call.cookie {
        request = request.header("cookie", format!("riauth_sso={cookie}"));
    }
    if let Some(token) = call.bearer {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    if call.portal {
        request = request.header("x-riauth-portal", "1");
    }
    if let Some(origin) = call.origin {
        request = request.header("origin", origin);
    }
    if let Some(revision) = call.revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    if let Some(key) = call.key {
        request = request.header("idempotency-key", key);
    }
    let body = match call.body {
        Some(body) => {
            request = request.header("content-type", "application/json");
            Body::from(body.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into()));
    (status, headers, value)
}

#[tokio::test]
async fn delivery_browser_contract_preserves_ambiguity_and_audits_scoped_dismissal() {
    use riauth::{crypto, identity::downstream, model::UserPatch};

    let fixture = Fixture::new();
    fixture.user("ada");
    let member_session = fixture.user("viewer");
    let member = sso_cookie(&fixture.core, &member_session);
    let admin = sso_cookie(&fixture.core, &fixture.admin);
    let users = fixture.core.list_users(&fixture.admin).unwrap();
    let ada = users
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["username"] == "ada")
        .unwrap();
    let user_id = ada["id"].as_str().unwrap();
    // A real local disable commits the intent for a previously delivered active link.
    let link = crypto::digest(&format!("payroll\0Users\0{user_id}"));
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "provisioning_links",
                &link,
                &json!({
                    "target": "payroll", "url": "http://127.0.0.1:9/scim/v2", "kind": "Users",
                    "local_id": user_id, "remote_id": "remote-ada", "external_id": "external-ada",
                    "body": {"active": true},
                }),
            )
        })
        .unwrap();
    fixture
        .core
        .update_user(
            &fixture.admin,
            "ada",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let mut row = fixture
        .core
        .provisioning_deactivations(&fixture.admin)
        .unwrap()[0]
        .clone();
    row["status"] = json!("stale");
    row["uncertain"] = json!(true);
    row["last_error"] = json!("Remote identity cannot be verified");
    let id = row["id"].as_str().unwrap().to_owned();
    fixture
        .core
        .store
        .write(|tx| tx.put(downstream::BUCKET, &id, &row))
        .unwrap();
    let app = riauth::api::router(fixture.core.clone());
    let path = "/api/admin/provisioning/deactivations";
    for (cookie, portal, expected) in [
        (None, true, StatusCode::UNAUTHORIZED),
        (Some(member.as_str()), true, StatusCode::FORBIDDEN),
        (Some(admin.as_str()), false, StatusCode::FORBIDDEN),
    ] {
        assert_eq!(
            send(
                &app,
                path,
                Call {
                    cookie,
                    portal,
                    ..Default::default()
                }
            )
            .await
            .0,
            expected
        );
    }
    let read = || Call {
        cookie: Some(&admin),
        portal: true,
        ..Default::default()
    };
    let (status, _, rows) = send(&app, path, read()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        rows,
        fixture
            .core
            .provisioning_deactivations(&fixture.admin)
            .unwrap()
    );
    let reviewed = rows[0].clone();
    assert_eq!(reviewed["delivery_state"], "ambiguous");
    let revision = send(&app, "/api/admin/session", read()).await.2["revision"]
        .as_u64()
        .unwrap();
    let origin = origin(&fixture.core);
    let input = json!({"revision": reviewed["revision"], "reason": "permanently_unverifiable", "evidence": "Payroll console unavailable permanently; OPS-42"});
    let write = || Call {
        method: "POST",
        cookie: Some(&admin),
        portal: true,
        origin: Some(&origin),
        revision: Some(revision),
        key: Some("delivery-browser-dismiss"),
        body: Some(input.clone()),
        ..Default::default()
    };
    let path = format!("{path}/{id}/dismiss");
    let before = fixture.snapshot().unwrap();
    for (call, expected) in [
        (
            Call {
                origin: Some("https://evil.example"),
                ..write()
            },
            StatusCode::FORBIDDEN,
        ),
        (
            Call {
                cookie: Some(&member),
                ..write()
            },
            StatusCode::FORBIDDEN,
        ),
        (
            Call {
                key: None,
                ..write()
            },
            StatusCode::PRECONDITION_REQUIRED,
        ),
        (
            Call {
                body: Some(
                    json!({"revision": "old", "reason": "remote_absent", "evidence": "OPS-42"}),
                ),
                ..write()
            },
            StatusCode::CONFLICT,
        ),
        (
            Call {
                body: Some(
                    json!({"revision": reviewed["revision"], "reason": "remote_absent", "evidence": "  "}),
                ),
                ..write()
            },
            StatusCode::BAD_REQUEST,
        ),
    ] {
        let (status, _, body) = send(&app, &path, call).await;
        assert_eq!(status, expected, "{body}");
        fixture.assert_http_mutation_snapshot(&before);
    }
    let (status, _, dismissed) = send(&app, &path, write()).await;
    assert_eq!(status, StatusCode::OK, "{dismissed}");
    assert_eq!(dismissed["status"], "dismissed");
    assert_eq!(dismissed["delivery_state"], "ambiguous");
    assert_eq!(dismissed["uncertain"], true);
    assert!(dismissed["delivered_at"].is_null());
    for field in ["user_id", "remote_id", "epoch", "last_error", "attempts"] {
        assert_eq!(dismissed[field], reviewed[field], "{field}");
    }
    assert_eq!(dismissed["dismissal"]["evidence"], input["evidence"]);
    assert_eq!(dismissed["dismissal"]["revision"], reviewed["revision"]);
    assert_eq!(dismissed["dismissal"]["previous_status"], "stale");
    let after = fixture.snapshot().unwrap();
    let replay = send(&app, &path, write()).await;
    assert_eq!(replay.0, StatusCode::OK, "{}", replay.2);
    assert_eq!(replay.2, dismissed);
    fixture.assert_http_mutation_snapshot(&after);
    assert_eq!(
        send(&app, "/api/admin/provisioning/deactivations", read())
            .await
            .2[0],
        dismissed
    );
    let audit = fixture.core.audit_events(&fixture.admin, 50).unwrap();
    let events: Vec<_> = audit
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == "provisioner.deactivate.dismiss")
        .collect();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0]["details"]["context"]["dismissal"],
        dismissed["dismissal"]
    );
}

#[tokio::test]
async fn page_is_self_contained() {
    let fixture = Fixture::new();
    let app = riauth::api::router(fixture.core.clone());
    let (status, headers, page) = send(&app, "/admin", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    let csp = headers["content-security-policy"].to_str().unwrap();
    assert!(csp.contains("script-src 'self'") && csp.contains("frame-ancestors 'none'"));
    let page = page.as_str().unwrap();
    assert!(page.contains("portal/assets/admin.js") && !page.contains("__BASE__"));
    for asset in ["/portal/assets/admin.js", "/portal/assets/admin.css"] {
        let (status, _, body) = send(&app, asset, Call::default()).await;
        assert_eq!(status, StatusCode::OK, "{asset}");
        // Nothing is fetched from elsewhere: the only URLs are the SVG namespace and example
        // placeholders and hints.
        let body = body
            .as_str()
            .unwrap()
            .replace("http://www.w3.org/2000/svg", "")
            .replace("https://app.example.com", "")
            .replace("http://127.0.0.1/callback", "");
        assert!(
            !body.contains("http://") && !body.contains("https://"),
            "{asset}"
        );
        assert!(
            !body.contains("url(") && !body.contains("@import"),
            "{asset}"
        );
    }
}

#[tokio::test]
async fn temporary_access_controls_open_the_scoped_review_page() {
    let fixture = Fixture::new();
    let app = riauth::api::router(fixture.core.clone());
    let (status, _, page) = send(&app, "/apps", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    let page = page.as_str().unwrap();
    let nav = page
        .split_once("<nav>")
        .unwrap()
        .1
        .split_once("</nav>")
        .unwrap()
        .0;
    assert!(nav.contains(
        "<a class=\"nav-item\" id=\"access-review-link\" href=\"/access/review\" hidden>"
    ));
    assert_eq!(page.matches("id=\"access-review-link\"").count(), 1);

    let (status, _, app_script) = send(&app, "/portal/assets/app.js", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    let app_script = app_script.as_str().unwrap();
    assert!(app_script.contains("$(\"access-review-link\").hidden = true"));
    assert!(app_script.contains("$(\"access-review-link\").hidden = data.access_review_available !== true || !RiAuthCapabilities.compiled(\"access.temporary_entitlements\")"));

    let (status, _, admin_script) = send(&app, "/portal/assets/admin.js", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    let admin_script = admin_script.as_str().unwrap();
    assert!(admin_script.contains("href: `${base}access/review`"));
    assert!(admin_script.contains("Review request") && admin_script.contains("Review grant"));
    assert!(!admin_script.contains("admin/access/requests/${"));
    assert!(!admin_script.contains("admin/access/grants/${"));
    assert!(!admin_script.contains("keyed: false"));
}

#[tokio::test]
async fn explicit_headless_mode_keeps_json_and_oidc_routes() {
    let fixture = Fixture::new();
    let browser = riauth::api::router(fixture.core.clone());
    for path in ["/account/reset", "/device"] {
        let (status, _, page) = send(&browser, path, Call::default()).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        let page = page.as_str().unwrap();
        assert!(page.contains("/portal/assets/capabilities.js"), "{path}");
    }
    let (_, _, account_script) = send(&browser, "/portal/assets/account.js", Call::default()).await;
    assert!(
        account_script
            .as_str()
            .unwrap()
            .contains("RiAuthCapabilities.usable(feature)")
    );
    let mut core = fixture.core.clone();
    core.config.browser_ui = false;
    let app = riauth::api::router(core);
    async fn get(app: &axum::Router, path: &str) -> (StatusCode, Value) {
        let response = app
            .clone()
            .oneshot(
                Request::get(path)
                    .header("host", "localhost:9000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, value)
    }
    for path in [
        "/apps",
        "/admin",
        "/account/security",
        "/account/sources/continue",
        "/device",
        "/setup",
        "/account/accept",
        "/account/verify",
        "/account/reset",
        "/portal/assets/app.js",
        "/portal/assets/admin.js",
        "/portal/assets/account.js",
        "/portal/assets/device.js",
        "/portal/assets/capabilities.js",
        "/portal/assets/signin.js",
    ] {
        let (status, _) = get(&app, path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
    }
    for path in [
        "/api/capabilities",
        "/api/portal/sources",
        "/.well-known/openid-configuration",
    ] {
        let (status, _) = get(&app, path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
    }
    let (status, _) = get(&app, "/api/portal").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = get(&app, "/api/admin/session").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    for path in [
        "/api/portal/account/accept",
        "/api/portal/account/verify",
        "/api/portal/account/reset",
        "/api/portal/account/reset-request",
        "/api/device/browser/decision",
    ] {
        let (status, _) = get(&app, path).await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED, "{path}");
    }
    let (status, root) = get(&app, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(root["service"], "riAuth");
}

#[tokio::test]
async fn reads_require_an_administrator_browser_session_and_the_portal_header() {
    let fixture = Fixture::new();
    let user = fixture.user("ada");
    let admin = sso_cookie(&fixture.core, &fixture.admin);
    let member = sso_cookie(&fixture.core, &user);
    let app = riauth::api::router(fixture.core.clone());
    for (cookie, portal, status) in [
        (None, true, StatusCode::UNAUTHORIZED),
        (Some("ri_sso_unknown"), true, StatusCode::UNAUTHORIZED),
        (Some(admin.as_str()), false, StatusCode::FORBIDDEN),
        (Some(member.as_str()), true, StatusCode::FORBIDDEN),
    ] {
        for path in [
            "/api/admin/session",
            "/api/admin/users",
            "/api/admin/access/requests",
        ] {
            let call = Call {
                cookie,
                portal,
                ..Default::default()
            };
            let (got, _, body) = send(&app, path, call).await;
            assert_eq!(got, status, "{path}: {body}");
        }
    }
    let call = Call {
        cookie: Some(&admin),
        portal: true,
        ..Default::default()
    };
    let (status, _, session) = send(&app, "/api/admin/session", call).await;
    assert_eq!(status, StatusCode::OK, "{session}");
    assert_eq!(session["user"]["username"], "admin");
    assert!(session["revision"].is_u64());
    assert!(session.get("password_hash").is_none());
}

#[cfg(feature = "platform")]
#[tokio::test]
async fn retained_grant_is_visible_and_revocable_after_approver_rules_are_removed() {
    use riauth::{config::Config, pam::NewAccessRequest};

    let fixture = Fixture::new();
    fixture.core.create_group(&fixture.admin, "ops").unwrap();
    let requester = fixture.user("ada");
    let approver = fixture.user("reviewer");
    let mut config: Config = fixture.core.config.clone();
    config
        .pam_approvers
        .insert("ops".into(), ["reviewer".into()].into());
    let Fixture { _dir, core, admin } = fixture;
    drop(core);
    let core = Core::open(config.clone()).unwrap();
    let request = core
        .request_access(
            &requester,
            NewAccessRequest {
                group: "ops".into(),
                reason: "Release support".into(),
                ttl: 3600,
            },
        )
        .unwrap();
    let decision = core
        .decide_access(&approver, request["id"].as_str().unwrap(), true)
        .unwrap();
    let grant_id = decision["grant"]["id"].as_str().unwrap().to_owned();
    drop(core);

    config.pam_approvers.clear();
    let core = Core::open(config).unwrap();
    assert_eq!(core.me(&requester).unwrap()["groups"], json!(["ops"]));
    let admin_cookie = sso_cookie(&core, &admin);
    let app = riauth::api::router(core.clone());
    let (status, _, capabilities) = send(&app, "/api/capabilities", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    let feature = &capabilities["feature_states"]["access.temporary_entitlements"];
    assert_eq!(feature["compiled"], true);
    assert_eq!(feature["usable"], false);

    let read = Call {
        cookie: Some(&admin_cookie),
        portal: true,
        ..Default::default()
    };
    let (status, _, grants) = send(&app, "/api/admin/access/grants", read).await;
    assert_eq!(status, StatusCode::OK, "{grants}");
    assert!(
        grants
            .as_array()
            .unwrap()
            .iter()
            .any(|grant| grant["id"] == grant_id)
    );
    let (status, _, script) = send(&app, "/portal/assets/admin.js", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        script
            .as_str()
            .unwrap()
            .contains("RiAuthCapabilities.compiled(\"access.temporary_entitlements\")")
    );

    let origin = origin(&core);
    let revision = core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let write = access_write(
        &admin_cookie,
        &origin,
        Some("retained-grant-revoke"),
        Some(revision),
    );
    let (status, _, revoked) = send(
        &app,
        &format!("/api/admin/access/grants/{grant_id}/revoke"),
        write,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{revoked}");
    assert!(revoked["revoked_at"].is_u64());
    assert_eq!(core.me(&requester).unwrap()["groups"], json!([]));
}

#[cfg(feature = "platform")]
#[tokio::test]
async fn legacy_access_decisions_require_preconditions_and_replay_once() {
    use riauth::{config::Config, pam::NewAccessRequest};

    let fixture = Fixture::new();
    fixture.core.create_group(&fixture.admin, "ops").unwrap();
    let alice = fixture.user("alice");
    let bob = fixture.user("bob");
    let mut config: Config = fixture.core.config.clone();
    config
        .pam_approvers
        .insert("ops".into(), ["admin".into()].into());
    let Fixture {
        _dir,
        core: original,
        admin,
    } = fixture;
    drop(original);
    let core = Core::open(config).unwrap();
    let request = |token: &str| {
        core.request_access(
            token,
            NewAccessRequest {
                group: "ops".into(),
                reason: "Release support".into(),
                ttl: 3600,
            },
        )
        .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let approve_id = request(&alice);
    let deny_id = request(&bob);
    let cookie = sso_cookie(&core, &admin);
    let origin = origin(&core);
    let app = riauth::api::router(core.clone());
    let revision = || {
        core.store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let audit_count = |action: &str, target: &str| {
        core.audit_events(&admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action && event["target"] == target)
            .count()
    };
    let approve_path = format!("/api/admin/access/requests/{approve_id}/approve");
    let deny_path = format!("/api/admin/access/requests/{deny_id}/deny");
    let decision_at = revision();
    for path in [&approve_path, &deny_path] {
        for (key, expected) in [
            (None, None),
            (Some("missing-revision"), None),
            (None, Some(decision_at)),
        ] {
            let (status, _, _) =
                send(&app, path, access_write(&cookie, &origin, key, expected)).await;
            assert_eq!(status, StatusCode::PRECONDITION_REQUIRED, "{path}");
        }
        let (status, _, _) = send(
            &app,
            path,
            access_write(
                &cookie,
                &origin,
                Some("stale-decision"),
                Some(decision_at - 1),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "{path}");
    }
    assert_eq!(revision(), decision_at);
    assert_eq!(audit_count("access.approve", &approve_id), 0);
    assert_eq!(audit_count("access.deny", &deny_id), 0);

    let approved = send(
        &app,
        &approve_path,
        access_write(&cookie, &origin, Some("legacy-approve"), Some(decision_at)),
    )
    .await;
    assert_eq!(approved.0, StatusCode::OK, "{}", approved.2);
    let grant_id = approved.2["grant"]["id"].as_str().unwrap().to_owned();
    assert_eq!(
        send(
            &app,
            &approve_path,
            access_write(&cookie, &origin, Some("legacy-approve"), Some(decision_at))
        )
        .await
        .2,
        approved.2
    );
    assert_eq!(audit_count("access.approve", &approve_id), 1);
    assert_eq!(revision(), decision_at + 1);

    let deny_at = revision();
    let denied = send(
        &app,
        &deny_path,
        access_write(&cookie, &origin, Some("legacy-deny"), Some(deny_at)),
    )
    .await;
    assert_eq!(denied.0, StatusCode::OK, "{}", denied.2);
    assert_eq!(
        send(
            &app,
            &deny_path,
            access_write(&cookie, &origin, Some("legacy-deny"), Some(deny_at))
        )
        .await
        .2,
        denied.2
    );
    assert_eq!(audit_count("access.deny", &deny_id), 1);
    assert_eq!(revision(), deny_at + 1);

    let revoke_path = format!("/api/admin/access/grants/{grant_id}/revoke");
    let revoke_at = revision();
    for (key, expected) in [
        (None, None),
        (Some("missing-revision"), None),
        (None, Some(revoke_at)),
    ] {
        let (status, _, _) = send(
            &app,
            &revoke_path,
            access_write(&cookie, &origin, key, expected),
        )
        .await;
        assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    }
    assert_eq!(
        send(
            &app,
            &revoke_path,
            access_write(&cookie, &origin, Some("stale-revoke"), Some(revoke_at - 1))
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(revision(), revoke_at);
    assert_eq!(audit_count("access.revoke", &grant_id), 0);
    let revoked = send(
        &app,
        &revoke_path,
        access_write(&cookie, &origin, Some("legacy-revoke"), Some(revoke_at)),
    )
    .await;
    assert_eq!(revoked.0, StatusCode::OK, "{}", revoked.2);
    assert_eq!(
        send(
            &app,
            &revoke_path,
            access_write(&cookie, &origin, Some("legacy-revoke"), Some(revoke_at))
        )
        .await
        .2,
        revoked.2
    );
    assert_eq!(audit_count("access.revoke", &grant_id), 1);
    assert_eq!(revision(), revoke_at + 1);
    assert_eq!(core.me(&alice).unwrap()["groups"], json!([]));
    assert_eq!(
        send(
            &app,
            &revoke_path,
            access_write(&cookie, &origin, Some("new-revoke"), Some(revoke_at))
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(audit_count("access.revoke", &grant_id), 1);
}

#[tokio::test]
async fn browser_credential_cannot_be_presented_as_a_bearer_token() {
    let fixture = Fixture::new();
    let admin = sso_cookie(&fixture.core, &fixture.admin);
    let app = riauth::api::router(fixture.core.clone());
    for token in [format!("browser-session {admin}"), admin.clone()] {
        let call = Call {
            bearer: Some(&token),
            ..Default::default()
        };
        let (status, _, body) = send(&app, "/api/users", call).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    }
}

#[tokio::test]
async fn writes_use_the_management_path_and_guards() {
    let fixture = Fixture::new();
    let user = fixture.user("ada");
    let admin = sso_cookie(&fixture.core, &fixture.admin);
    let member = sso_cookie(&fixture.core, &user);
    let app = riauth::api::router(fixture.core.clone());
    let origin = origin(&fixture.core);
    let new_user = json!({"username": "grace", "password": "correct horse battery staple 42", "display_name": "Grace"});
    let blocked_revision = fixture
        .core
        .store
        .get("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    // Cross-site shapes and non-administrators change nothing.
    for (cookie, portal, sent_origin, status) in [
        (Some(admin.as_str()), true, None, StatusCode::FORBIDDEN),
        (
            Some(admin.as_str()),
            false,
            Some(origin.as_str()),
            StatusCode::FORBIDDEN,
        ),
        (
            Some(admin.as_str()),
            true,
            Some("https://evil.example"),
            StatusCode::FORBIDDEN,
        ),
        (
            Some(member.as_str()),
            true,
            Some(origin.as_str()),
            StatusCode::FORBIDDEN,
        ),
        (None, true, Some(origin.as_str()), StatusCode::UNAUTHORIZED),
    ] {
        let call = Call {
            method: "POST",
            cookie,
            portal,
            origin: sent_origin,
            revision: Some(blocked_revision),
            key: Some("blocked-user-create"),
            body: Some(new_user.clone()),
            ..Default::default()
        };
        let (got, _, body) = send(&app, "/api/admin/users", call).await;
        assert_eq!(got, status, "{body}");
    }
    assert!(
        !fixture
            .core
            .list_users(&fixture.admin)
            .unwrap()
            .to_string()
            .contains("grace")
    );

    let write = |method, body| Call {
        method,
        cookie: Some(&admin),
        portal: true,
        origin: Some(&origin),
        body,
        ..Default::default()
    };
    let create = Call {
        revision: Some(
            fixture
                .core
                .store
                .get("meta", "revision")
                .unwrap()
                .unwrap_or(0),
        ),
        key: Some("create-grace"),
        ..write("POST", Some(new_user))
    };
    let (status, _, created) = send(&app, "/api/admin/users", create).await;
    assert_eq!(status, StatusCode::OK, "{created}");
    assert!(created.get("password_hash").is_none());
    // Validation is the management API's own.
    let (status, _, body) = send(
        &app,
        "/api/admin/users",
        Call {
            revision: Some(
                fixture
                    .core
                    .store
                    .get("meta", "revision")
                    .unwrap()
                    .unwrap_or(0),
            ),
            key: Some("invalid-user-create"),
            ..write(
                "POST",
                Some(json!({"username": "bad name!", "password": "x"})),
            )
        },
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let group_revision = fixture
        .core
        .store
        .get("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let (status, _, body) = send(
        &app,
        "/api/admin/groups",
        Call {
            revision: Some(group_revision),
            key: Some("create-engineering"),
            ..write("POST", Some(json!({"name": "engineering"})))
        },
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let member_revision = fixture
        .core
        .store
        .get("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let (status, _, body) = send(
        &app,
        "/api/admin/groups/engineering/members/grace",
        Call {
            revision: Some(member_revision),
            key: Some("add-grace-to-engineering"),
            ..write("PUT", None)
        },
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // The audit actor is the administrator, as for the bearer API.
    let admin_id = fixture.core.me(&fixture.admin).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let events = fixture.core.audit_events(&fixture.admin, 20).unwrap();
    for action in ["user.create", "group.create", "group.member.add"] {
        let event = events
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["action"] == action)
            .unwrap();
        assert_eq!(event["actor"], admin_id.as_str(), "{action}");
    }

    // A stale If-Match revision is rejected without a change.
    let session = Call {
        cookie: Some(&admin),
        portal: true,
        ..Default::default()
    };
    let revision = send(&app, "/api/admin/session", session).await.2["revision"]
        .as_u64()
        .unwrap();
    let stale = Call {
        revision: Some(revision - 1),
        key: Some("stale-user-edit"),
        ..write("PATCH", Some(json!({"display_name": "Stale"})))
    };
    let (status, _, body) = send(&app, "/api/admin/users/grace", stale).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    let current = Call {
        revision: Some(revision),
        key: Some("current-user-edit"),
        ..write("PATCH", Some(json!({"display_name": "Grace Hopper"})))
    };
    let (status, _, body) = send(&app, "/api/admin/users/grace", current).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["display_name"], "Grace Hopper");
}

#[tokio::test]
async fn user_creation_requires_receipt_and_revision_across_browser_and_bearer() {
    let fixture = Fixture::new();
    let outsider = fixture.user("outsider");
    let cookie = sso_cookie(&fixture.core, &fixture.admin);
    let origin = origin(&fixture.core);
    let app = riauth::api::router(fixture.core.clone());
    let revision = || {
        fixture
            .core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let audit_count = || {
        fixture
            .core
            .audit_events(&fixture.admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "user.create")
            .count()
    };
    let input =
        |username| json!({"username": username, "password": PASSWORD, "display_name": username});
    let browser = |key: Option<&'static str>, version, username| Call {
        method: "POST",
        cookie: Some(&cookie),
        portal: true,
        origin: Some(&origin),
        key,
        revision: version,
        body: Some(input(username)),
        ..Default::default()
    };
    let bearer = |key: Option<&'static str>, version, username| Call {
        method: "POST",
        bearer: Some(&fixture.admin),
        key,
        revision: version,
        body: Some(input(username)),
        ..Default::default()
    };
    let browser_path = "/api/admin/users";
    let bearer_path = "/api/users";
    let (status, _, script) = send(&app, "/portal/assets/admin.js", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        script
            .as_str()
            .unwrap()
            .contains("api(\"POST\", \"admin/users\", body, { revision: data.revision, key })")
    );
    let at = revision();
    let baseline_audit = audit_count();
    let before = fixture.snapshot().unwrap();
    for (key, version) in [(None, None), (Some("key-only"), None), (None, Some(at))] {
        assert_eq!(
            send(&app, browser_path, browser(key, version, "missing"))
                .await
                .0,
            StatusCode::PRECONDITION_REQUIRED
        );
        assert_eq!(
            send(&app, bearer_path, bearer(key, version, "missing"))
                .await
                .0,
            StatusCode::PRECONDITION_REQUIRED
        );
    }
    assert_eq!(
        send(
            &app,
            bearer_path,
            Call {
                method: "POST",
                bearer: Some(&outsider),
                key: Some("unauthorized-user-create"),
                revision: Some(at),
                body: Some(input("forbidden")),
                ..Default::default()
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    fixture.assert_http_mutation_snapshot(&before);

    let first = send(
        &app,
        browser_path,
        browser(Some("browser-user-create"), Some(at), "browser-created"),
    )
    .await;
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(first.2["username"], "browser-created");
    assert!(first.2.get("password_hash").is_none());
    let committed = fixture.snapshot().unwrap();
    let replay = send(
        &app,
        browser_path,
        browser(Some("browser-user-create"), Some(at), "browser-created"),
    )
    .await;
    assert_eq!(replay.0, StatusCode::OK);
    assert_eq!(replay.2, first.2);
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count(), baseline_audit + 1);
    assert_eq!(revision(), at + 1);
    assert_eq!(
        send(
            &app,
            browser_path,
            browser(Some("browser-user-create"), Some(at), "different")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            bearer_path,
            bearer(Some("stale-user-create"), Some(at), "stale")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    fixture.assert_http_mutation_snapshot(&committed);

    let current = revision();
    let second = send(
        &app,
        bearer_path,
        bearer(Some("bearer-user-create"), Some(current), "api-created"),
    )
    .await;
    assert_eq!(second.0, StatusCode::OK);
    assert_eq!(second.2["username"], "api-created");
    let replay = send(
        &app,
        bearer_path,
        bearer(Some("bearer-user-create"), Some(current), "api-created"),
    )
    .await;
    assert_eq!(replay.0, StatusCode::OK);
    assert_eq!(replay.2, second.2);
    assert_eq!(audit_count(), baseline_audit + 2);
    assert_eq!(revision(), current + 1);
}

#[tokio::test]
async fn group_writes_require_retry_binding_across_browser_and_bearer() {
    use riauth::agent::{NewAgent, Permission};

    let fixture = Fixture::new();
    let member_session = fixture.user("member");
    let member_id = fixture.core.me(&member_session).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let outsider = fixture.user("outsider");
    let scoped = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: "scoped-group-writer".into(),
                ttl: 600,
                parent: None,
                permissions: ["group.write", "group.members"]
                    .into_iter()
                    .map(|action| Permission {
                        action: action.into(),
                        resource: "group/scoped".into(),
                    })
                    .collect(),
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let cookie = sso_cookie(&fixture.core, &fixture.admin);
    let origin = origin(&fixture.core);
    let app = riauth::api::router(fixture.core.clone());
    let revision = || {
        fixture
            .core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let audit_count = |action: &str| {
        fixture
            .core
            .audit_events(&fixture.admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action)
            .count()
    };
    let create = |browser: bool, key: Option<&'static str>, version, name| Call {
        method: "POST",
        cookie: browser.then_some(cookie.as_str()),
        bearer: (!browser).then_some(fixture.admin.as_str()),
        portal: browser,
        origin: browser.then_some(origin.as_str()),
        key,
        revision: version,
        body: Some(json!({"name": name})),
    };
    let membership =
        |browser: bool, method: &'static str, key: Option<&'static str>, version, body| Call {
            method,
            cookie: browser.then_some(cookie.as_str()),
            bearer: (!browser).then_some(fixture.admin.as_str()),
            portal: browser,
            origin: browser.then_some(origin.as_str()),
            key,
            revision: version,
            body,
        };
    let browser_groups = "/api/admin/groups";
    let bearer_groups = "/api/groups";
    let browser_member = "/api/admin/groups/engineering/members/member";
    let bearer_member = "/api/groups/engineering/members/member";
    let (status, _, script) = send(&app, "/portal/assets/admin.js", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    let script = script.as_str().unwrap();
    assert!(
        script.contains(
            "api(\"POST\", \"admin/groups\", { name }, { revision: data.revision, key })"
        )
    );
    assert!(script.contains("memberRetries.get(button)"));
    assert!(script.contains("{ revision: retry.revision, key: retry.key }"));

    let at = revision();
    let before = fixture.snapshot().unwrap();
    for (key, version) in [(None, None), (Some("key-only"), None), (None, Some(at))] {
        assert_eq!(
            send(
                &app,
                browser_groups,
                create(true, key, version, "engineering")
            )
            .await
            .0,
            StatusCode::PRECONDITION_REQUIRED
        );
        assert_eq!(
            send(
                &app,
                bearer_groups,
                create(false, key, version, "engineering")
            )
            .await
            .0,
            StatusCode::PRECONDITION_REQUIRED
        );
        assert_eq!(
            send(
                &app,
                browser_member,
                membership(true, "PUT", key, version, None)
            )
            .await
            .0,
            StatusCode::PRECONDITION_REQUIRED
        );
        assert_eq!(
            send(
                &app,
                bearer_member,
                membership(false, "DELETE", key, version, None)
            )
            .await
            .0,
            StatusCode::PRECONDITION_REQUIRED
        );
    }
    assert_eq!(
        send(
            &app,
            bearer_groups,
            Call {
                bearer: Some(&outsider),
                ..create(false, Some("unauthorized-create"), Some(at), "denied")
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    fixture.assert_http_mutation_snapshot(&before);

    let first = send(
        &app,
        browser_groups,
        create(true, Some("browser-group-create"), Some(at), "engineering"),
    )
    .await;
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(first.2["name"], "engineering");
    let committed = fixture.snapshot().unwrap();
    let replay = send(
        &app,
        browser_groups,
        create(true, Some("browser-group-create"), Some(at), "engineering"),
    )
    .await;
    assert_eq!(replay.0, StatusCode::OK);
    assert_eq!(replay.2, first.2);
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("group.create"), 1);
    assert_eq!(revision(), at + 1);
    assert_eq!(
        send(
            &app,
            browser_groups,
            create(true, Some("browser-group-create"), Some(at), "different")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            bearer_groups,
            create(false, Some("stale-group-create"), Some(at), "research")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    fixture.assert_http_mutation_snapshot(&committed);

    let current = revision();
    let second = send(
        &app,
        bearer_groups,
        create(
            false,
            Some("bearer-group-create"),
            Some(current),
            "research",
        ),
    )
    .await;
    assert_eq!(second.0, StatusCode::OK);
    assert_eq!(second.2["name"], "research");
    let committed = fixture.snapshot().unwrap();
    let replay = send(
        &app,
        bearer_groups,
        create(
            false,
            Some("bearer-group-create"),
            Some(current),
            "research",
        ),
    )
    .await;
    assert_eq!(replay.0, StatusCode::OK);
    assert_eq!(replay.2, second.2);
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("group.create"), 2);
    assert_eq!(revision(), current + 1);

    let current = revision();
    let added = send(
        &app,
        browser_member,
        membership(true, "PUT", Some("browser-member-add"), Some(current), None),
    )
    .await;
    assert_eq!(added.0, StatusCode::OK);
    assert!(
        added.2["members"]
            .as_array()
            .unwrap()
            .contains(&json!(member_id))
    );
    let committed = fixture.snapshot().unwrap();
    let replay = send(
        &app,
        browser_member,
        membership(true, "PUT", Some("browser-member-add"), Some(current), None),
    )
    .await;
    assert_eq!(replay.0, StatusCode::OK);
    assert_eq!(replay.2, added.2);
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("group.member.add"), 1);
    assert_eq!(revision(), current + 1);
    assert_eq!(
        send(
            &app,
            browser_member,
            membership(
                true,
                "PUT",
                Some("browser-member-add"),
                Some(current),
                Some(json!({"ignored": true})),
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            bearer_member,
            membership(
                false,
                "DELETE",
                Some("stale-member-remove"),
                Some(current),
                None
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    fixture.assert_http_mutation_snapshot(&committed);

    let current = revision();
    let removed = send(
        &app,
        bearer_member,
        membership(
            false,
            "DELETE",
            Some("bearer-member-remove"),
            Some(current),
            None,
        ),
    )
    .await;
    assert_eq!(removed.0, StatusCode::OK);
    assert_eq!(removed.2["members"], json!([]));
    let committed = fixture.snapshot().unwrap();
    let replay = send(
        &app,
        bearer_member,
        membership(
            false,
            "DELETE",
            Some("bearer-member-remove"),
            Some(current),
            None,
        ),
    )
    .await;
    assert_eq!(replay.0, StatusCode::OK);
    assert_eq!(replay.2, removed.2);
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("group.member.remove"), 1);
    assert_eq!(revision(), current + 1);

    let scoped_at = revision();
    let before = fixture.snapshot().unwrap();
    assert_eq!(
        send(
            &app,
            bearer_groups,
            Call {
                bearer: Some(&scoped),
                ..create(false, Some("scoped-wrong-group"), Some(scoped_at), "denied")
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    fixture.assert_http_mutation_snapshot(&before);
    let scoped_create = || Call {
        bearer: Some(&scoped),
        ..create(false, Some("scoped-create"), Some(scoped_at), "scoped")
    };
    let created = send(&app, bearer_groups, scoped_create()).await;
    assert_eq!(created.0, StatusCode::OK);
    assert_eq!(created.2["name"], "scoped");
    let committed = fixture.snapshot().unwrap();
    assert_eq!(
        send(&app, bearer_groups, scoped_create()).await.2,
        created.2
    );
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(revision(), scoped_at + 1);
    let member_at = revision();
    let before = fixture.snapshot().unwrap();
    assert_eq!(
        send(
            &app,
            bearer_member,
            Call {
                bearer: Some(&scoped),
                ..membership(
                    false,
                    "PUT",
                    Some("scoped-wrong-member"),
                    Some(member_at),
                    None
                )
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    fixture.assert_http_mutation_snapshot(&before);
    let scoped_member = "/api/groups/scoped/members/member";
    let scoped_add = || Call {
        bearer: Some(&scoped),
        ..membership(
            false,
            "PUT",
            Some("scoped-member-add"),
            Some(member_at),
            None,
        )
    };
    let added = send(&app, scoped_member, scoped_add()).await;
    assert_eq!(added.0, StatusCode::OK);
    let committed = fixture.snapshot().unwrap();
    assert_eq!(send(&app, scoped_member, scoped_add()).await.2, added.2);
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(revision(), member_at + 1);
    assert_eq!(audit_count("group.create"), 3);
    assert_eq!(audit_count("group.member.add"), 2);
}

#[tokio::test]
async fn user_update_requires_receipt_and_revision_across_browser_and_bearer() {
    let fixture = Fixture::new();
    fixture.user("managed");
    let outsider = fixture.user("outsider");
    let cookie = sso_cookie(&fixture.core, &fixture.admin);
    let origin = origin(&fixture.core);
    let app = riauth::api::router(fixture.core.clone());
    let revision = || {
        fixture
            .core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let audit_count = || {
        fixture
            .core
            .audit_events(&fixture.admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "user.update")
            .count()
    };
    let browser = |key: Option<&'static str>, version, name| Call {
        method: "PATCH",
        cookie: Some(&cookie),
        portal: true,
        origin: Some(&origin),
        key,
        revision: version,
        body: Some(json!({"display_name": name})),
        ..Default::default()
    };
    let bearer = |key: Option<&'static str>, version, name| Call {
        method: "PATCH",
        bearer: Some(&fixture.admin),
        key,
        revision: version,
        body: Some(json!({"display_name": name})),
        ..Default::default()
    };
    let browser_path = "/api/admin/users/managed";
    let bearer_path = "/api/users/managed";
    let (status, _, script) = send(&app, "/portal/assets/admin.js", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        script.as_str().unwrap().contains(
            "`admin/users/${seg(user.username)}`, patch, { revision: data.revision, key }"
        )
    );
    let at = revision();
    let before = fixture.snapshot().unwrap();
    for (key, version) in [(None, None), (Some("key-only"), None), (None, Some(at))] {
        assert_eq!(
            send(&app, browser_path, browser(key, version, "First"))
                .await
                .0,
            StatusCode::PRECONDITION_REQUIRED
        );
        assert_eq!(
            send(&app, bearer_path, bearer(key, version, "First"))
                .await
                .0,
            StatusCode::PRECONDITION_REQUIRED
        );
    }
    assert_eq!(
        send(
            &app,
            bearer_path,
            Call {
                method: "PATCH",
                bearer: Some(&outsider),
                key: Some("unauthorized-user-update"),
                revision: Some(at),
                body: Some(json!({"display_name": "Denied"})),
                ..Default::default()
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    fixture.assert_http_mutation_snapshot(&before);
    assert_eq!(audit_count(), 0);

    let first = send(
        &app,
        browser_path,
        browser(Some("browser-user-update"), Some(at), "First"),
    )
    .await;
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(first.2["display_name"], "First");
    let committed = fixture.snapshot().unwrap();
    let replay = send(
        &app,
        browser_path,
        browser(Some("browser-user-update"), Some(at), "First"),
    )
    .await;
    assert_eq!(replay.0, StatusCode::OK);
    assert_eq!(replay.2, first.2);
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count(), 1);
    assert_eq!(revision(), at + 1);
    assert_eq!(
        send(
            &app,
            browser_path,
            browser(Some("browser-user-update"), Some(at), "Changed body")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            bearer_path,
            bearer(Some("stale-user-update"), Some(at), "Second")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    fixture.assert_http_mutation_snapshot(&committed);

    let current = revision();
    let second = send(
        &app,
        bearer_path,
        bearer(Some("bearer-user-update"), Some(current), "Second"),
    )
    .await;
    assert_eq!(second.0, StatusCode::OK);
    assert_eq!(second.2["display_name"], "Second");
    let replay = send(
        &app,
        bearer_path,
        bearer(Some("bearer-user-update"), Some(current), "Second"),
    )
    .await;
    assert_eq!(replay.0, StatusCode::OK);
    assert_eq!(replay.2, second.2);
    assert_eq!(audit_count(), 2);
    assert_eq!(revision(), current + 1);
}

#[tokio::test]
async fn application_presentation_keeps_the_other_settings() {
    let fixture = Fixture::new();
    fixture.client("code", true);
    let admin = sso_cookie(&fixture.core, &fixture.admin);
    let app = riauth::api::router(fixture.core.clone());
    let origin = origin(&fixture.core);
    let read = Call {
        cookie: Some(&admin),
        portal: true,
        ..Default::default()
    };
    let (_, _, clients) = send(&app, "/api/admin/clients", read).await;
    let before = clients
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["client_id"] == "code")
        .unwrap()
        .clone();
    let mut settings = before["settings"].clone();
    settings["app"] = json!({"description": "Code review", "category": "Engineering", "launch_url": "https://code.example.com/", "icon": "code", "accent": "violet", "hidden": false, "launch_scopes": []});
    let patch = Call {
        method: "PATCH",
        cookie: Some(&admin),
        portal: true,
        origin: Some(&origin),
        revision: Some(
            fixture
                .core
                .store
                .get("meta", "revision")
                .unwrap()
                .unwrap_or(0),
        ),
        key: Some("application-presentation-update"),
        body: Some(json!({"settings": settings})),
        ..Default::default()
    };
    let (status, _, after) = send(&app, "/api/admin/clients/code", patch).await;
    assert_eq!(status, StatusCode::OK, "{after}");
    assert_eq!(
        after["settings"]["app"]["launch_url"],
        "https://code.example.com/"
    );
    let mut rest = after["settings"].clone();
    rest.as_object_mut().unwrap().remove("app");
    assert_eq!(rest, before["settings"]);
    // Rotation returns the new secret once, through the same authorization.
    let revision = fixture
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let rotate = Call {
        method: "POST",
        cookie: Some(&admin),
        portal: true,
        origin: Some(&origin),
        revision: Some(revision),
        key: Some("application-presentation-rotation"),
        ..Default::default()
    };
    let (status, _, body) = send(&app, "/api/admin/clients/code/rotate-secret", rotate).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body["client_secret"]
            .as_str()
            .unwrap()
            .starts_with("ri_client_")
    );
}

#[tokio::test]
async fn client_create_and_update_require_retry_binding_across_browser_and_bearer() {
    use riauth::agent::{NewAgent, Permission};

    let fixture = Fixture::new();
    let outsider = fixture.user("outsider");
    let scoped = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: "client-name-writer".into(),
                ttl: 600,
                parent: None,
                permissions: [Permission {
                    action: "client.write".into(),
                    resource: "client/browser-app".into(),
                }]
                .into(),
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let cookie = sso_cookie(&fixture.core, &fixture.admin);
    let origin = origin(&fixture.core);
    let app = riauth::api::router(fixture.core.clone());
    let revision = || {
        fixture
            .core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let audit_count = |action: &str| {
        fixture
            .core
            .audit_events(&fixture.admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action)
            .count()
    };
    let create_body = |id: &str, name: &str, confidential| {
        json!({
            "client_id": id, "name": name, "confidential": confidential,
            "redirect_uris": ["https://app.example.test/callback"], "scopes": ["openid"],
        })
    };
    let write =
        |browser: bool, method: &'static str, key: Option<&'static str>, version, body| Call {
            method,
            cookie: browser.then_some(cookie.as_str()),
            bearer: (!browser).then_some(fixture.admin.as_str()),
            portal: browser,
            origin: browser.then_some(origin.as_str()),
            revision: version,
            key,
            body: Some(body),
        };
    let browser_collection = "/api/admin/clients";
    let bearer_collection = "/api/clients";
    let browser_client = "/api/admin/clients/browser-app";
    let bearer_client = "/api/clients/bearer-app";
    let (status, _, script) = send(&app, "/portal/assets/admin.js", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    let script = script.as_str().unwrap();
    assert!(
        script.contains("api(\"POST\", \"admin/clients\", body, { revision: data.revision, key })")
    );
    assert!(script.contains("patch, { revision: data.revision, key }"));

    let at = revision();
    let before = fixture.snapshot().unwrap();
    for (key, version) in [(None, None), (Some("key-only"), None), (None, Some(at))] {
        for (browser, path) in [(true, browser_collection), (false, bearer_collection)] {
            assert_eq!(
                send(
                    &app,
                    path,
                    write(
                        browser,
                        "POST",
                        key,
                        version,
                        create_body("missing-app", "Missing", false)
                    )
                )
                .await
                .0,
                StatusCode::PRECONDITION_REQUIRED
            );
        }
        for (browser, path) in [(true, browser_client), (false, "/api/clients/browser-app")] {
            assert_eq!(
                send(
                    &app,
                    path,
                    write(browser, "PATCH", key, version, json!({"name":"Missing"}))
                )
                .await
                .0,
                StatusCode::PRECONDITION_REQUIRED
            );
        }
    }
    assert_eq!(
        send(
            &app,
            bearer_collection,
            Call {
                bearer: Some(&outsider),
                ..write(
                    false,
                    "POST",
                    Some("outsider-create"),
                    Some(at),
                    create_body("forbidden-app", "Forbidden", false)
                )
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    fixture.assert_http_mutation_snapshot(&before);

    let browser_create = || {
        write(
            true,
            "POST",
            Some("browser-client-create"),
            Some(at),
            create_body("browser-app", "Browser app", true),
        )
    };
    let first = send(&app, browser_collection, browser_create()).await;
    assert_eq!(first.0, StatusCode::OK, "{}", first.2);
    assert_eq!(first.2["client"]["client_id"], "browser-app");
    assert!(
        first.2["client_secret"]
            .as_str()
            .unwrap()
            .starts_with("ri_client_")
    );
    let committed = fixture.snapshot().unwrap();
    let repeated = send(&app, browser_collection, browser_create()).await;
    assert_eq!(repeated.0, StatusCode::CONFLICT, "{}", repeated.2);
    assert_eq!(repeated.2["error"], "credential_already_issued");
    assert!(repeated.2.get("client_secret").is_none());
    assert!(
        !repeated
            .2
            .to_string()
            .contains(first.2["client_secret"].as_str().unwrap())
    );
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("client.create"), 1);
    assert_eq!(revision(), at + 1);
    assert_eq!(
        send(
            &app,
            browser_collection,
            write(
                true,
                "POST",
                Some("browser-client-create"),
                Some(at),
                create_body("changed-app", "Changed", true)
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            bearer_collection,
            write(
                false,
                "POST",
                Some("stale-client-create"),
                Some(at),
                create_body("stale-app", "Stale", false)
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    fixture.assert_http_mutation_snapshot(&committed);

    let current = revision();
    let bearer_create = || {
        write(
            false,
            "POST",
            Some("bearer-client-create"),
            Some(current),
            create_body("bearer-app", "Bearer app", false),
        )
    };
    let second = send(&app, bearer_collection, bearer_create()).await;
    assert_eq!(second.0, StatusCode::OK, "{}", second.2);
    assert!(second.2["client_secret"].is_null());
    let committed = fixture.snapshot().unwrap();
    assert_eq!(
        send(&app, bearer_collection, bearer_create()).await.2,
        second.2
    );
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("client.create"), 2);
    assert_eq!(revision(), current + 1);

    let at = revision();
    let before = fixture.snapshot().unwrap();
    assert_eq!(
        send(
            &app,
            "/api/clients/browser-app",
            Call {
                bearer: Some(&outsider),
                ..write(
                    false,
                    "PATCH",
                    Some("outsider-update"),
                    Some(at),
                    json!({"name":"Denied"})
                )
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        send(
            &app,
            bearer_client,
            Call {
                bearer: Some(&scoped),
                ..write(
                    false,
                    "PATCH",
                    Some("scoped-wrong-client"),
                    Some(at),
                    json!({"name":"Denied"})
                )
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    fixture.assert_http_mutation_snapshot(&before);
    let browser_update = || {
        write(
            true,
            "PATCH",
            Some("browser-client-update"),
            Some(at),
            json!({"name":"Browser renamed"}),
        )
    };
    let updated = send(&app, browser_client, browser_update()).await;
    assert_eq!(updated.0, StatusCode::OK, "{}", updated.2);
    assert_eq!(updated.2["name"], "Browser renamed");
    let committed = fixture.snapshot().unwrap();
    assert_eq!(
        send(&app, browser_client, browser_update()).await.2,
        updated.2
    );
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("client.update"), 1);
    assert_eq!(revision(), at + 1);
    assert_eq!(
        send(
            &app,
            browser_client,
            write(
                true,
                "PATCH",
                Some("browser-client-update"),
                Some(at),
                json!({"name":"Changed again"})
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            bearer_client,
            write(
                false,
                "PATCH",
                Some("stale-client-update"),
                Some(at),
                json!({"name":"Stale"})
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    fixture.assert_http_mutation_snapshot(&committed);

    let current = revision();
    let bearer_update = || {
        write(
            false,
            "PATCH",
            Some("bearer-client-update"),
            Some(current),
            json!({"name":"Bearer renamed"}),
        )
    };
    let updated = send(&app, bearer_client, bearer_update()).await;
    assert_eq!(updated.0, StatusCode::OK, "{}", updated.2);
    let committed = fixture.snapshot().unwrap();
    assert_eq!(
        send(&app, bearer_client, bearer_update()).await.2,
        updated.2
    );
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("client.update"), 2);
    assert_eq!(revision(), current + 1);

    let scoped_at = revision();
    let scoped_update = || Call {
        bearer: Some(&scoped),
        ..write(
            false,
            "PATCH",
            Some("scoped-client-update"),
            Some(scoped_at),
            json!({"name":"Scoped rename"}),
        )
    };
    let updated = send(&app, "/api/clients/browser-app", scoped_update()).await;
    assert_eq!(updated.0, StatusCode::OK, "{}", updated.2);
    let committed = fixture.snapshot().unwrap();
    assert_eq!(
        send(&app, "/api/clients/browser-app", scoped_update())
            .await
            .2,
        updated.2
    );
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count("client.update"), 3);
    assert_eq!(revision(), scoped_at + 1);
}

#[tokio::test]
async fn client_secret_rotation_requires_retry_binding_across_browser_and_bearer() {
    let fixture = Fixture::new();
    let original = fixture.client("rotation", true).unwrap();
    let outsider = fixture.user("outsider");
    let cookie = sso_cookie(&fixture.core, &fixture.admin);
    let origin = origin(&fixture.core);
    let app = riauth::api::router(fixture.core.clone());
    let revision = || {
        fixture
            .core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let audit_count = || {
        fixture
            .core
            .audit_events(&fixture.admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| {
                event["action"] == "client.secret.rotate" && event["target"] == "rotation"
            })
            .count()
    };
    let browser = |key: Option<&'static str>, version| Call {
        method: "POST",
        cookie: Some(&cookie),
        portal: true,
        origin: Some(&origin),
        key,
        revision: version,
        ..Default::default()
    };
    let bearer = |key: Option<&'static str>, version| Call {
        method: "POST",
        bearer: Some(&fixture.admin),
        key,
        revision: version,
        ..Default::default()
    };
    let browser_path = "/api/admin/clients/rotation/rotate-secret";
    let bearer_path = "/api/clients/rotation/rotate-secret";
    let (status, _, script) = send(&app, "/portal/assets/admin.js", Call::default()).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        script
            .as_str()
            .unwrap()
            .contains("rotate-secret`, undefined, { revision: data.revision, key }")
    );
    let at = revision();
    let before = fixture.snapshot().unwrap();
    for (key, version) in [(None, None), (Some("key-only"), None), (None, Some(at))] {
        assert_eq!(
            send(&app, browser_path, browser(key, version)).await.0,
            StatusCode::PRECONDITION_REQUIRED
        );
        assert_eq!(
            send(&app, bearer_path, bearer(key, version)).await.0,
            StatusCode::PRECONDITION_REQUIRED
        );
    }
    assert_eq!(
        send(
            &app,
            bearer_path,
            Call {
                method: "POST",
                bearer: Some(&outsider),
                key: Some("unauthorized-rotation"),
                revision: Some(at),
                ..Default::default()
            }
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    fixture.assert_http_mutation_snapshot(&before);
    assert_eq!(audit_count(), 0);

    let first = send(
        &app,
        browser_path,
        browser(Some("browser-rotation"), Some(at)),
    )
    .await;
    assert_eq!(first.0, StatusCode::OK);
    assert!(first.2["client_secret"].as_str().unwrap() != original);
    let committed = fixture.snapshot().unwrap();
    let replay = send(
        &app,
        browser_path,
        browser(Some("browser-rotation"), Some(at)),
    )
    .await;
    assert_eq!(replay.0, StatusCode::CONFLICT);
    assert_eq!(replay.2["error"], "credential_already_issued");
    assert!(
        !replay
            .2
            .to_string()
            .contains(first.2["client_secret"].as_str().unwrap())
    );
    fixture.assert_http_mutation_snapshot(&committed);
    assert_eq!(audit_count(), 1);
    assert_eq!(revision(), at + 1);

    assert_eq!(
        send(&app, bearer_path, bearer(Some("stale-rotation"), Some(at)))
            .await
            .0,
        StatusCode::CONFLICT
    );
    fixture.assert_http_mutation_snapshot(&committed);
    let current = revision();
    let second = send(
        &app,
        bearer_path,
        bearer(Some("bearer-rotation"), Some(current)),
    )
    .await;
    assert_eq!(second.0, StatusCode::OK);
    assert!(second.2["client_secret"] != first.2["client_secret"]);
    let replay = send(
        &app,
        bearer_path,
        bearer(Some("bearer-rotation"), Some(current)),
    )
    .await;
    assert_eq!(replay.0, StatusCode::CONFLICT);
    assert_eq!(replay.2["error"], "credential_already_issued");
    assert!(
        !replay
            .2
            .to_string()
            .contains(second.2["client_secret"].as_str().unwrap())
    );
    assert_eq!(audit_count(), 2);
    assert_eq!(revision(), current + 1);
}

#[tokio::test]
async fn a_changed_email_cannot_be_marked_verified() {
    let fixture = Fixture::new();
    fixture.user("ada");
    fixture
        .core
        .update_user(
            &fixture.admin,
            "ada",
            riauth::model::UserPatch {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let admin = sso_cookie(&fixture.core, &fixture.admin);
    let app = riauth::api::router(fixture.core.clone());
    let origin = origin(&fixture.core);
    let stored = |core: &Core| {
        let users = core.list_users(&fixture.admin).unwrap();
        let ada = users
            .as_array()
            .unwrap()
            .iter()
            .find(|u| u["username"] == "ada")
            .unwrap()
            .clone();
        (ada["email"].clone(), ada["email_verified"].clone())
    };
    let patch = |body, key| Call {
        method: "PATCH",
        cookie: Some(&admin),
        portal: true,
        origin: Some(&origin),
        revision: Some(
            fixture
                .core
                .store
                .get("meta", "revision")
                .unwrap()
                .unwrap_or(0),
        ),
        key: Some(key),
        body: Some(body),
        ..Default::default()
    };
    let body = json!({"email": "ada@new.example", "email_verified": true});
    let (status, _, reply) = send(
        &app,
        "/api/admin/users/ada",
        patch(body, "invalid-email-edit"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{reply}");
    assert_eq!(
        stored(&fixture.core),
        (json!("ada@example.test"), json!(true))
    );
    let body = json!({"email": "ada@new.example"});
    let (status, _, reply) = send(
        &app,
        "/api/admin/users/ada",
        patch(body, "valid-email-edit"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{reply}");
    assert_eq!(
        stored(&fixture.core),
        (json!("ada@new.example"), json!(false))
    );
}

fn checks_named<'a>(report: &'a Value, id: &str) -> Vec<&'a Value> {
    report["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["id"] == id)
        .collect()
}

#[tokio::test]
async fn setup_wizard_checks_a_draft_with_the_create_path_without_writing() {
    let fixture = Fixture::new();
    let user = fixture.user("ada");
    fixture.client("code", true);
    let admin = sso_cookie(&fixture.core, &fixture.admin);
    let member = sso_cookie(&fixture.core, &user);
    let app = riauth::api::router(fixture.core.clone());
    let origin = origin(&fixture.core);
    let post = |cookie, body| Call {
        method: "POST",
        cookie: Some(cookie),
        portal: true,
        origin: Some(&origin),
        body: Some(body),
        ..Default::default()
    };
    let spa = json!({
        "client_id": "dashboard", "name": "Dashboard", "confidential": false,
        "redirect_uris": ["https://dash.example.com/callback"],
        "scopes": ["openid", "profile", "email"],
        "settings": {
            "token_endpoint_auth_method": "none",
            "post_logout_redirect_uris": ["https://dash.example.com/"],
            "claim_mappings": [{"scope": "profile", "claim": "department", "source": {"type": "attribute", "key": "department"}}],
        },
    });
    let revision = |app: axum::Router| {
        let admin = admin.clone();
        async move {
            let call = Call {
                cookie: Some(&admin),
                portal: true,
                ..Default::default()
            };
            send(&app, "/api/admin/session", call).await.2["revision"]
                .as_u64()
                .unwrap()
        }
    };
    let before = revision(app.clone()).await;
    let (status, _, report) =
        send(&app, "/api/admin/client-checks", post(&admin, spa.clone())).await;
    assert_eq!(status, StatusCode::OK, "{report}");
    assert_eq!(report["valid"], true);
    assert_eq!(
        report["connection"]["token_endpoint_auth_methods"],
        json!(["none"])
    );
    assert_eq!(report["connection"]["client_id"], "dashboard");
    assert!(
        report["claims"]["by_scope"]["profile"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["claim"] == "department")
    );
    // A browser app without an allowed origin is told which one to add.
    let origins = checks_named(&report, "origins");
    assert_eq!(origins[0]["level"], "warn", "{report}");
    assert!(
        origins[0]["detail"]
            .as_str()
            .unwrap()
            .contains("https://dash.example.com")
    );
    assert!(report.to_string().find("client_secret\"").is_none());
    // Checking writes nothing: no client, no audit record, same revision.
    assert_eq!(revision(app.clone()).await, before);
    assert!(
        !fixture
            .core
            .list_clients(&fixture.admin)
            .unwrap()
            .to_string()
            .contains("dashboard")
    );
    let events = fixture.core.audit_events(&fixture.admin, 50).unwrap();
    assert!(
        !events
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["target"] == "dashboard")
    );

    // Rejections are the management path's own.
    let mut bad = spa.clone();
    bad["redirect_uris"] = json!(["http://dash.example.com/callback"]);
    let (status, _, body) = send(&app, "/api/admin/client-checks", post(&admin, bad)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(
        body["error_description"]
            .as_str()
            .unwrap()
            .contains("Redirect URIs")
    );
    let mut bad = spa.clone();
    bad["settings"]["claim_mappings"][0]["claim"] = json!("email");
    let (status, _, body) = send(&app, "/api/admin/client-checks", post(&admin, bad)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let mut bad = spa.clone();
    bad["settings"]["origins"] = json!(["https://dash.example.com/"]);
    let (status, _, body) = send(&app, "/api/admin/client-checks", post(&admin, bad)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    // A private key is not a public JWKS; the draft is explained, not a bare 422.
    let mut bad = spa.clone();
    bad["confidential"] = json!(true);
    bad["settings"]["token_endpoint_auth_method"] = json!("private_key_jwt");
    bad["settings"]["jwks"] = json!({"keys": [{"kty": "EC", "kid": "k", "alg": "ES256", "crv": "P-256", "x": "x", "y": "y", "d": "secret"}]});
    let (status, _, body) = send(&app, "/api/admin/client-checks", post(&admin, bad)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(
        body["error_description"]
            .as_str()
            .unwrap()
            .contains("couldn't read this configuration")
    );
    let mut taken = spa.clone();
    taken["client_id"] = json!("code");
    let (status, _, body) = send(&app, "/api/admin/client-checks", post(&admin, taken)).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    // Same guards and authorization as a create.
    let (status, _, body) =
        send(&app, "/api/admin/client-checks", post(&member, spa.clone())).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let cross_site = Call {
        origin: Some("https://evil.example"),
        ..post(&admin, spa.clone())
    };
    let (status, _, body) = send(&app, "/api/admin/client-checks", cross_site).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    // The checked draft creates as checked; a confidential web app gets its secret once.
    let (status, _, created) = send(
        &app,
        "/api/admin/clients",
        Call {
            revision: Some(revision(app.clone()).await),
            key: Some("wizard-create-dashboard"),
            ..post(&admin, spa)
        },
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{created}");
    assert!(created["client_secret"].is_null());
    let web = json!({
        "client_id": "portal-web", "name": "Portal", "confidential": true,
        "redirect_uris": ["https://portal.example.com/auth/callback"],
        "scopes": ["openid", "profile"],
        "settings": {"token_endpoint_auth_method": "client_secret_basic"},
    });
    let (status, _, report) =
        send(&app, "/api/admin/client-checks", post(&admin, web.clone())).await;
    assert_eq!(status, StatusCode::OK, "{report}");
    assert_eq!(
        report["connection"]["token_endpoint_auth_methods"],
        json!(["client_secret_basic"])
    );
    assert!(checks_named(&report, "origins").is_empty(), "{report}");
    let (status, _, created) = send(
        &app,
        "/api/admin/clients",
        Call {
            revision: Some(revision(app.clone()).await),
            key: Some("wizard-create-portal-web"),
            ..post(&admin, web)
        },
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{created}");
    assert!(
        created["client_secret"]
            .as_str()
            .unwrap()
            .starts_with("ri_client_")
    );
}

#[tokio::test]
async fn application_diagnostics_explain_what_blocks_sign_in() {
    let fixture = Fixture::new();
    let ada = fixture.user("ada");
    let secret = fixture.client("code", true);
    let admin = sso_cookie(&fixture.core, &fixture.admin);
    let member = sso_cookie(&fixture.core, &ada);
    let app = riauth::api::router(fixture.core.clone());
    let origin = origin(&fixture.core);
    fixture.core.create_group(&fixture.admin, "eng").unwrap();
    crate::common::client_policy::replace(
        &fixture.core,
        &fixture.admin,
        "code",
        Some(["eng".to_owned()].into()),
        Some(true),
    );
    let read = |cookie| Call {
        cookie: Some(cookie),
        portal: true,
        ..Default::default()
    };
    let diagnostics = "/api/admin/clients/code/diagnostics";
    let (status, _, report) = send(&app, diagnostics, read(&admin)).await;
    assert_eq!(status, StatusCode::OK, "{report}");
    let connection = &report["connection"];
    assert_eq!(connection["issuer"], fixture.core.config.issuer.as_str());
    assert!(
        connection["discovery_url"]
            .as_str()
            .unwrap()
            .ends_with("/.well-known/openid-configuration")
    );
    assert_eq!(
        connection["token_endpoint_auth_methods"],
        json!(["client_secret_basic", "client_secret_post"])
    );
    assert!(report.to_string().find("ri_client_").is_none());
    assert_eq!(checks_named(&report, "enabled")[0]["level"], "ok");
    assert_eq!(checks_named(&report, "signing")[0]["level"], "ok");
    assert_eq!(
        checks_named(&report, "access")[0]["level"],
        "warn",
        "{report}"
    );
    assert_eq!(checks_named(&report, "launch")[0]["level"], "warn");
    assert_eq!(checks_named(&report, "activity")[0]["level"], "info");

    // With a member, access passes and the missing second factor is named.
    fixture
        .core
        .group_member(&fixture.admin, "eng", "ada", true)
        .unwrap();
    let (_, _, report) = send(&app, diagnostics, read(&admin)).await;
    assert_eq!(
        checks_named(&report, "access")[0]["level"],
        "ok",
        "{report}"
    );
    assert_eq!(checks_named(&report, "mfa")[0]["level"], "warn", "{report}");

    // The sign-in test is the policy engine's simulation.
    let test = |mfa| Call {
        method: "POST",
        cookie: Some(&admin),
        portal: true,
        origin: Some(&origin),
        body: Some(json!({"username": "ada", "scope": ["openid", "profile"], "mfa": mfa})),
        ..Default::default()
    };
    let (status, _, refused) = send(&app, "/api/admin/clients/code/explain", test(false)).await;
    assert_eq!(status, StatusCode::OK, "{refused}");
    assert_eq!(refused["allowed"], false);
    assert!(
        refused["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("mfa_required"))
    );
    let (_, _, allowed) = send(&app, "/api/admin/clients/code/explain", test(true)).await;
    assert_eq!(allowed["allowed"], true, "{allowed}");
    assert_eq!(allowed["token_issued"], false);
    assert_eq!(
        allowed["id_token_identity_claims"]["preferred_username"],
        "ada"
    );

    // A completed token exchange shows up as activity.
    crate::common::client_policy::replace(&fixture.core, &fixture.admin, "code", None, Some(false));
    fixture.tokens("code", &ada, secret);
    let (_, _, report) = send(&app, diagnostics, read(&admin)).await;
    let activity = checks_named(&report, "activity")[0];
    assert_eq!(activity["level"], "ok", "{report}");
    assert!(activity["last_issued_at"].is_u64());

    // Administrators only, through the portal read guard.
    let (status, _, body) = send(&app, diagnostics, read(&member)).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let no_header = Call {
        cookie: Some(&admin),
        ..Default::default()
    };
    let (status, _, body) = send(&app, diagnostics, no_header).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    let (status, _, body) =
        send(&app, "/api/admin/clients/missing/diagnostics", read(&admin)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}
