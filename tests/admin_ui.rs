mod common;

use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use common::Fixture;
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
    body: Option<Value>,
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
async fn explicit_headless_mode_keeps_json_and_oidc_routes() {
    let fixture = Fixture::new();
    let mut core = fixture.core.clone();
    core.config.browser_ui = false;
    let app = riauth::api::router(core);
    async fn get(app: &axum::Router, path: &str) -> (StatusCode, Value) {
        let response = app.clone().oneshot(Request::get(path)
            .header("host", "localhost:9000")
            .body(Body::empty()).unwrap()).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, value)
    }
    for path in [
        "/apps", "/admin", "/account/security", "/account/sources/continue", "/device", "/setup",
        "/portal/assets/app.js", "/portal/assets/admin.js",
        "/portal/assets/capabilities.js", "/portal/assets/signin.js",
    ] {
        let (status, _) = get(&app, path).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
    }
    for path in ["/api/capabilities", "/api/portal/sources", "/.well-known/openid-configuration"] {
        let (status, _) = get(&app, path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
    }
    let (status, _) = get(&app, "/api/portal").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = get(&app, "/api/admin/session").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
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
    let write = Call {
        method: "POST",
        cookie: Some(&admin_cookie),
        portal: true,
        origin: Some(&origin),
        ..Default::default()
    };
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
    let (status, _, created) = send(&app, "/api/admin/users", write("POST", Some(new_user))).await;
    assert_eq!(status, StatusCode::OK, "{created}");
    assert!(created.get("password_hash").is_none());
    // Validation is the management API's own.
    let (status, _, body) = send(
        &app,
        "/api/admin/users",
        write(
            "POST",
            Some(json!({"username": "bad name!", "password": "x"})),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let (status, _, body) = send(
        &app,
        "/api/admin/groups",
        write("POST", Some(json!({"name": "engineering"}))),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, _, body) = send(
        &app,
        "/api/admin/groups/engineering/members/grace",
        write("PUT", None),
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
        ..write("PATCH", Some(json!({"display_name": "Stale"})))
    };
    let (status, _, body) = send(&app, "/api/admin/users/grace", stale).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    let current = Call {
        revision: Some(revision),
        ..write("PATCH", Some(json!({"display_name": "Grace Hopper"})))
    };
    let (status, _, body) = send(&app, "/api/admin/users/grace", current).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["display_name"], "Grace Hopper");
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
    let rotate = Call {
        method: "POST",
        cookie: Some(&admin),
        portal: true,
        origin: Some(&origin),
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
    let patch = |body| Call {
        method: "PATCH",
        cookie: Some(&admin),
        portal: true,
        origin: Some(&origin),
        body: Some(body),
        ..Default::default()
    };
    let body = json!({"email": "ada@new.example", "email_verified": true});
    let (status, _, reply) = send(&app, "/api/admin/users/ada", patch(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{reply}");
    assert_eq!(
        stored(&fixture.core),
        (json!("ada@example.test"), json!(true))
    );
    let body = json!({"email": "ada@new.example"});
    let (status, _, reply) = send(&app, "/api/admin/users/ada", patch(body)).await;
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
    let (status, _, created) = send(&app, "/api/admin/clients", post(&admin, spa)).await;
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
    let (status, _, created) = send(&app, "/api/admin/clients", post(&admin, web)).await;
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
    fixture
        .core
        .update_client(
            &fixture.admin,
            "code",
            riauth::model::ClientPatch {
                allowed_groups: Some(["eng".to_owned()].into()),
                require_mfa: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
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
    fixture
        .core
        .update_client(
            &fixture.admin,
            "code",
            riauth::model::ClientPatch {
                require_mfa: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
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
