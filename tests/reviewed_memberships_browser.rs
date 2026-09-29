mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::{core::Core, model::NewUser};
use serde_json::{Value, json};
use tower::ServiceExt;

fn cookie(core: &Core, token: &str) -> String {
    let request = core.portal_sign_in().unwrap();
    core.portal_decide(token, request.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = request.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    core.portal_poll(request.body["id"].as_str().unwrap(), Some(binding))
        .unwrap()
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
        .to_string()
}

fn administrator(f: &Fixture, name: &str) -> String {
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: name.into(),
                password: PASSWORD.into(),
                email: None,
                display_name: name.into(),
                admin: true,
            },
        )
        .unwrap();
    let token = f.core.login(name.into(), PASSWORD.into(), None).unwrap();
    cookie(&f.core, token["session_token"].as_str().unwrap())
}

async fn request(
    app: &axum::Router,
    method: &str,
    path: &str,
    cookie: Option<&str>,
    origin: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost:9000")
        .header("x-riauth-portal", "1");
    if let Some(cookie) = cookie {
        request = request.header("cookie", format!("riauth_sso={cookie}"));
    }
    if let Some(origin) = origin {
        request = request.header("origin", origin);
    }
    let body = match body {
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
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value =
        serde_json::from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes)));
    (status, value)
}

#[tokio::test]
async fn browser_membership_review_uses_guarded_exact_content_service_and_consumes_once() {
    let mut f = Fixture::new();
    f.core
        .config
        .reviewed_membership_groups
        .insert("protected".into());
    f.core.create_group(&f.admin, "protected").unwrap();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    let author = cookie(&f.core, &f.admin);
    let member = cookie(&f.core, &f.user("recipient"));
    let app = riauth::api::router(f.core.clone());
    let origin = Some("http://localhost:9000");
    let path = "/api/admin/groups/protected/membership-changes";
    let input = json!({"members":["recipient"]});

    let (status, page) = request(&app, "GET", "/admin", None, None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.as_str()
            .unwrap()
            .contains("/portal/assets/membership-review.js")
    );
    let (status, _) = request(
        &app,
        "GET",
        "/portal/assets/membership-review.js",
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let mut headless = f.core.clone();
    headless.config.browser_ui = false;
    let (status, _) = request(
        &riauth::api::router(headless),
        "GET",
        "/portal/assets/membership-review.js",
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    for (session, origin, expected) in [
        (
            Some(author.as_str()),
            Some("https://untrusted.test"),
            StatusCode::FORBIDDEN,
        ),
        (Some(author.as_str()), None, StatusCode::FORBIDDEN),
        (None, origin, StatusCode::UNAUTHORIZED),
        (Some(member.as_str()), origin, StatusCode::FORBIDDEN),
    ] {
        let before = f.snapshot().unwrap();
        let (status, _) = request(&app, "POST", path, session, origin, Some(input.clone())).await;
        assert_eq!(status, expected);
        f.assert_http_mutation_snapshot(&before);
    }
    let (status, change) = request(&app, "POST", path, Some(&author), origin, Some(input)).await;
    assert_eq!(status, StatusCode::OK, "{change}");
    let id = change["proposal"]["id"].as_str().unwrap();
    let read = format!("/api/admin/group-membership-changes/{id}");
    let approve = format!("{read}/approve");
    let execute = format!("{read}/execute");
    let binding = json!({"digest": change["digest"]});
    let (status, exact) = request(&app, "GET", &read, Some(&reviewer), None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(exact, change);

    for (action, actor, payload, expected) in [
        (&approve, &author, binding.clone(), StatusCode::FORBIDDEN),
        (
            &approve,
            &reviewer,
            json!({"digest":"changed"}),
            StatusCode::CONFLICT,
        ),
        (
            &approve,
            &reviewer,
            json!({"digest":change["digest"],"members":[]}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (&execute, &executor, binding.clone(), StatusCode::FORBIDDEN),
    ] {
        let before = f.snapshot().unwrap();
        let (status, _) = request(&app, "POST", action, Some(actor), origin, Some(payload)).await;
        assert_eq!(status, expected);
        f.assert_http_mutation_snapshot(&before);
    }
    let (status, approved) = request(
        &app,
        "POST",
        &approve,
        Some(&reviewer),
        origin,
        Some(binding.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(approved["status"], "approved");
    let before = f.snapshot().unwrap();
    let (status, _) = request(
        &app,
        "POST",
        &execute,
        Some(&reviewer),
        origin,
        Some(binding.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    f.assert_http_mutation_snapshot(&before);
    let (status, result) = request(
        &app,
        "POST",
        &execute,
        Some(&executor),
        origin,
        Some(binding.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result["status"], "executed");
    assert_eq!(result["proposal"], change["proposal"]);
    let before = f.snapshot().unwrap();
    let (status, _) = request(
        &app,
        "POST",
        &execute,
        Some(&executor),
        origin,
        Some(binding),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(
        f.core
            .store
            .get::<riauth::model::Group>("groups", "protected")
            .unwrap()
            .unwrap()
            .members,
        [change["proposal"]["after"][0]["user_id"]
            .as_str()
            .unwrap()
            .to_owned()]
        .into()
    );
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["action"] == "reviewed_memberships.execute"
                && e["details"]["change_id"] == id)
            .count(),
        1
    );
    let (status, cancelled) = request(
        &app,
        "POST",
        path,
        Some(&author),
        origin,
        Some(json!({"members":[]})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let endpoint = format!(
        "/api/admin/group-membership-changes/{}",
        cancelled["proposal"]["id"].as_str().unwrap()
    );
    let binding = json!({"digest": cancelled["digest"]});
    let (status, result) = request(
        &app,
        "POST",
        &format!("{endpoint}/cancel"),
        Some(&author),
        origin,
        Some(binding.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(result["status"], "cancelled");
    let before = f.snapshot().unwrap();
    let (status, _) = request(
        &app,
        "POST",
        &format!("{endpoint}/execute"),
        Some(&executor),
        origin,
        Some(binding),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&before);
}
