mod common;

use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use common::Fixture;
use http_body_util::BodyExt;
use riauth::{
    config::Config,
    core::Core,
    model::{Audit, NewUser, UserPatch},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use tower::ServiceExt;

const SECRETS: &[&str] = &[
    "target-should-not-leak",
    "actor-should-not-leak",
    "run-should-not-leak",
    "s3cret-password-value",
    "ri_agent_should_not_leak",
    "client-secret-value-xyz",
    "argon2-should-not-leak",
    "203.0.113.50",
    "198.51.100.10",
    "8.8.8.8",
    "password_hash",
    "client_secret",
];

fn audit(at: u64, id: &str, actor: &str, action: &str, details: Value) -> Audit {
    Audit {
        id: id.into(),
        at,
        actor: actor.into(),
        action: action.into(),
        target: "target-should-not-leak".into(),
        run_id: Some("run-should-not-leak".into()),
        details,
    }
}

fn insert(core: &Core, events: &[Audit]) {
    core.store
        .write(|tx| {
            for event in events {
                tx.put("audit", &format!("{:020}-{}", event.at, event.id), event)?;
            }
            Ok(())
        })
        .unwrap();
}

fn located(latitude: f64, longitude: f64, label: &str) -> Value {
    json!({
        "location": {"latitude": latitude, "longitude": longitude, "label": label},
        "password": "s3cret-password-value",
        "token": "ri_agent_should_not_leak",
        "client_secret": "client-secret-value-xyz",
        "password_hash": "argon2-should-not-leak",
        "ip": "203.0.113.50",
        "remote_addr": "198.51.100.10"
    })
}

async fn send(
    app: &axum::Router,
    uri: &str,
    token: Option<&str>,
    cookie: Option<&str>,
    accept: Option<&str>,
) -> (StatusCode, HeaderMap, String) {
    let mut request = Request::builder().uri(uri);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    if let Some(cookie) = cookie {
        request = request.header("cookie", format!("riauth_sso={cookie}"));
    }
    if let Some(accept) = accept {
        request = request.header("accept", accept);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
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
    (status, headers, body)
}

fn json_body(body: &str) -> Value {
    serde_json::from_str(body).unwrap_or_else(|error| panic!("{error}: {body}"))
}

fn tenth(value: &Value) -> i32 {
    (value.as_f64().expect("coordinate") * 10.0).round() as i32
}

fn find_point(body: &Value, latitude: i32, longitude: i32) -> &Value {
    body["points"]
        .as_array()
        .unwrap()
        .iter()
        .find(|point| {
            tenth(&point["latitude"]) == latitude && tenth(&point["longitude"]) == longitude
        })
        .unwrap_or_else(|| panic!("missing cell {latitude},{longitude} in {body}"))
}

fn assert_public(body: &str) {
    for secret in SECRETS {
        assert!(!body.contains(secret), "{secret} leaked: {body}");
    }
}

fn agent(fixture: &Fixture, id: &str, action: &str, resource: &str) -> String {
    fixture
        .core
        .create_agent(
            &fixture.admin,
            riauth::agent::NewAgent {
                id: id.into(),
                ttl: 3600,
                permissions: vec![riauth::agent::Permission {
                    action: action.into(),
                    resource: resource.into(),
                }],
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

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

fn assert_offline(body: &str) {
    let lower = body.to_ascii_lowercase();
    for host in [
        "openstreetmap",
        "mapbox",
        "arcgis",
        "googleapis",
        "gstatic",
        "cartocdn",
        "maptiler",
        "thunderforest",
        "stamen",
        "unpkg.com",
        "jsdelivr",
        "cdnjs",
        "nominatim",
        "geoip",
        "navigator.geolocation",
    ] {
        assert!(!lower.contains(host), "{host} in portal response");
    }
    assert!(!lower.contains("https://"), "{body}");
    assert!(!lower.contains("http://"), "{body}");
    assert!(!lower.contains("src=\"//"));
    assert!(!lower.contains("href=\"//"));
    assert!(!lower.contains("url("));
}

#[tokio::test]
async fn permissions_match_the_audit_api() {
    let fixture = Fixture::new();
    let user = fixture.user("ada");
    let allowed = agent(&fixture, "map-exact", "audit.read", "audit/events");
    let wildcard = agent(&fixture, "map-star", "audit.read", "*");
    let wrong_resource = agent(&fixture, "map-other", "audit.read", "audit/other");
    let reader = agent(&fixture, "map-user", "user.read", "*");
    let base = riauth::crypto::now() + 86_400;
    insert(
        &fixture.core,
        &[audit(
            base,
            "perm",
            "actor-should-not-leak",
            "map.perm",
            located(12.0, 34.0, "HiddenLabel"),
        )],
    );
    let admin_cookie = sso_cookie(&fixture.core, &fixture.admin);
    let user_cookie = sso_cookie(&fixture.core, &user);
    let app = riauth::api::router(fixture.core.clone());
    let path = format!("/api/audit/map?since={base}&until={base}&action=map.perm");
    for (token, cookie, status) in [
        (None, None, StatusCode::UNAUTHORIZED),
        (
            Some("not-a-token"),
            Some(admin_cookie.as_str()),
            StatusCode::UNAUTHORIZED,
        ),
        (Some(user.as_str()), None, StatusCode::FORBIDDEN),
        (Some(wrong_resource.as_str()), None, StatusCode::FORBIDDEN),
        (Some(reader.as_str()), None, StatusCode::FORBIDDEN),
        (None, Some(user_cookie.as_str()), StatusCode::FORBIDDEN),
    ] {
        let (got, _, body) = send(&app, &path, token, cookie, None).await;
        assert_eq!(got, status, "{body}");
        assert!(!body.contains("HiddenLabel"), "{body}");
        assert_public(&body);
    }
    let before = fixture.core.store.list::<Value>("audit").unwrap().len();
    for token in [fixture.admin.as_str(), allowed.as_str(), wildcard.as_str()] {
        let (status, headers, body) = send(&app, &path, Some(token), None, None).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert!(headers.get("access-control-allow-origin").is_none());
        let parsed = json_body(&body);
        assert_eq!(find_point(&parsed, 120, 340)["count"].as_u64(), Some(1));
        assert_eq!(find_point(&parsed, 120, 340)["label"], "HiddenLabel");
        assert_eq!(parsed["unknown"].as_u64(), Some(0));
        assert_public(&body);
    }
    let (status, _, body) = send(&app, &path, None, Some(&admin_cookie), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(json_body(&body)["matched"].as_u64(), Some(1));
    assert_public(&body);
    assert_eq!(
        fixture.core.store.list::<Value>("audit").unwrap().len(),
        before
    );
    let (status, _, body) =
        send(&app, "/api/audit?limit=1", Some(&fixture.admin), None, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body.starts_with('['));
}

#[tokio::test]
async fn aggregates_rounded_cells_counts_unknown_and_omits_secrets() {
    let fixture = Fixture::new();
    let base = riauth::crypto::now() + 86_400;
    let rows = [
        (10.04, 20.01, "Lab"),
        (10.02, 20.04, "Lab"),
        (10.06, 20.01, "Other"),
        (1.0, 179.96, "East"),
        (1.0, -180.0, "East"),
        (2.0, 3.0, "One"),
        (2.04, 3.04, "Two"),
        (90.0, 180.0, "Pole"),
    ];
    let mut events: Vec<_> = rows
        .iter()
        .enumerate()
        .map(|(index, (latitude, longitude, label))| {
            audit(
                base,
                &format!("cell-{index}"),
                "actor-should-not-leak",
                "map.cells",
                located(*latitude, *longitude, label),
            )
        })
        .collect();
    events.push(audit(
        base,
        "bad-lat",
        "actor-should-not-leak",
        "map.cells",
        json!({"location": {"latitude": 95, "longitude": 0, "label": "Nope"}}),
    ));
    events.push(audit(
        base,
        "label-only",
        "actor-should-not-leak",
        "map.cells",
        json!({"location": {"label": "Nope"}}),
    ));
    events.push(audit(
        base,
        "ip-only",
        "actor-should-not-leak",
        "map.cells",
        json!({"ip": "8.8.8.8", "remote_addr": "203.0.113.50"}),
    ));
    insert(&fixture.core, &events);
    let app = riauth::api::router(fixture.core.clone());
    let path = format!("/api/audit/map?since={base}&until={base}&action=map.cells");
    let (status, _, body) = send(&app, &path, Some(&fixture.admin), None, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let parsed = json_body(&body);
    assert_eq!(parsed["points"].as_array().unwrap().len(), 5);
    assert_eq!(parsed["unknown"].as_u64(), Some(3));
    assert_eq!(parsed["matched"].as_u64(), Some(11));
    assert_eq!(parsed["scanned"].as_u64(), Some(12));
    assert_eq!(parsed["truncated"].as_bool(), Some(false));
    assert_eq!(parsed["omitted_cells"].as_u64(), Some(0));
    let lab = find_point(&parsed, 100, 200);
    assert_eq!(lab["count"].as_u64(), Some(2));
    assert_eq!(lab["label"], "Lab");
    assert_eq!(find_point(&parsed, 101, 200)["label"], "Other");
    assert_eq!(find_point(&parsed, 10, -1800)["count"].as_u64(), Some(2));
    assert_eq!(find_point(&parsed, 10, -1800)["label"], "East");
    let mixed = find_point(&parsed, 20, 30);
    assert_eq!(mixed["count"].as_u64(), Some(2));
    assert!(mixed.get("label").is_none());
    assert_eq!(find_point(&parsed, 900, -1800)["label"], "Pole");
    assert_public(&body);
    let echoed = format!("{path}&ip=8.8.8.8");
    let (again, _, echoed_body) = send(&app, &echoed, Some(&fixture.admin), None, None).await;
    assert_eq!(again, StatusCode::OK);
    assert_eq!(json_body(&echoed_body), parsed);
    assert_public(&echoed_body);
}

#[tokio::test]
async fn user_attribute_is_used_only_when_the_event_has_no_location() {
    let fixture = Fixture::new();
    let created = fixture
        .core
        .create_user(
            &fixture.admin,
            NewUser {
                username: "ada".into(),
                password: common::PASSWORD.into(),
                email: None,
                display_name: "Ada".into(),
                admin: false,
            },
        )
        .unwrap();
    let user_id = created["id"].as_str().unwrap().to_owned();
    fixture
        .core
        .update_user(
            &fixture.admin,
            "ada",
            UserPatch {
                attributes: Some(BTreeMap::from([
                    (
                        "location".into(),
                        json!({"latitude": 51.51, "longitude": -0.12, "label": "London"}),
                    ),
                    ("note".into(), json!("s3cret-password-value")),
                ])),
                ..Default::default()
            },
        )
        .unwrap();
    let base = riauth::crypto::now() + 86_400;
    insert(
        &fixture.core,
        &[
            audit(
                base,
                "attr",
                &user_id,
                "map.attr",
                json!({"ip": "203.0.113.50"}),
            ),
            audit(
                base,
                "null-loc",
                &user_id,
                "map.attr",
                json!({"location": null}),
            ),
            audit(
                base,
                "invalid",
                &user_id,
                "map.attr",
                json!({"location": {"latitude": 200, "longitude": 1}}),
            ),
            audit(
                base,
                "override",
                &user_id,
                "map.attr",
                located(40.0, 10.0, "Desk"),
            ),
        ],
    );
    let app = riauth::api::router(fixture.core);
    let (status, _, body) = send(
        &app,
        &format!("/api/audit/map?since={base}&until={base}&action=map.attr"),
        Some(&fixture.admin),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let parsed = json_body(&body);
    assert_eq!(parsed["unknown"].as_u64(), Some(2));
    assert_eq!(parsed["matched"].as_u64(), Some(4));
    assert_eq!(parsed["points"].as_array().unwrap().len(), 2);
    assert_eq!(find_point(&parsed, 515, -1)["count"].as_u64(), Some(1));
    assert_eq!(find_point(&parsed, 515, -1)["label"], "London");
    assert_eq!(find_point(&parsed, 400, 100)["label"], "Desk");
    assert!(!body.contains(&user_id));
    assert!(!body.contains("ada"));
    assert_public(&body);
}

#[tokio::test]
async fn time_range_is_inclusive_and_action_filter_is_a_prefix() {
    let fixture = Fixture::new();
    let base = riauth::crypto::now() + 86_400;
    insert(
        &fixture.core,
        &[
            audit(
                base + 10,
                "t10",
                "actor-should-not-leak",
                "map.filter.keep",
                located(5.0, 5.0, "Keep"),
            ),
            audit(
                base + 20,
                "t20",
                "actor-should-not-leak",
                "map.filter.keep",
                located(5.0, 5.0, "Keep"),
            ),
            audit(
                base + 30,
                "t30",
                "actor-should-not-leak",
                "map.filter.drop",
                located(6.0, 6.0, "Drop"),
            ),
            audit(
                base + 40,
                "t40",
                "actor-should-not-leak",
                "map.filter.keep",
                located(5.0, 5.0, "Keep"),
            ),
        ],
    );
    let app = riauth::api::router(fixture.core);
    let (status, _, body) = send(
        &app,
        &format!(
            "/api/audit/map?since={}&until={}&action=map.filter.keep",
            base + 10,
            base + 20
        ),
        Some(&fixture.admin),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let parsed = json_body(&body);
    assert_eq!(parsed["matched"].as_u64(), Some(2));
    assert_eq!(parsed["unknown"].as_u64(), Some(0));
    assert_eq!(find_point(&parsed, 50, 50)["count"].as_u64(), Some(2));
    assert!(
        parsed["points"]
            .as_array()
            .unwrap()
            .iter()
            .all(|point| tenth(&point["latitude"]) != 60)
    );

    let (status, _, body) = send(
        &app,
        &format!(
            "/api/audit/map?since={base}&until={}&action=map.filter.",
            base + 100
        ),
        Some(&fixture.admin),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let parsed = json_body(&body);
    assert_eq!(parsed["matched"].as_u64(), Some(4));
    assert_eq!(find_point(&parsed, 50, 50)["count"].as_u64(), Some(3));
    assert_eq!(find_point(&parsed, 60, 60)["count"].as_u64(), Some(1));

    let (status, _, body) = send(
        &app,
        &format!("/api/audit/map?since={base}&until={}&action=.", base + 100),
        Some(&fixture.admin),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(json_body(&body)["matched"].as_u64(), Some(0));

    for path in [
        format!("/api/audit/map?since={}&until={}", base + 20, base + 10),
        "/api/audit/map?action=%0Amap".into(),
        format!("/api/audit/map?action={}", "a".repeat(129)),
        "/api/audit/map?since=no".into(),
    ] {
        let (status, _, body) = send(&app, &path, Some(&fixture.admin), None, None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}: {body}");
    }
}

#[tokio::test]
async fn point_cap_keeps_the_highest_counts_then_stable_coordinates() {
    let fixture = Fixture::new();
    let base = riauth::crypto::now() + 86_400;
    let mut events = Vec::new();
    for latitude in 0..21 {
        for longitude in 0..25 {
            let index = events.len();
            events.push(audit(
                base + index as u64,
                &format!("c{index:04}"),
                "actor-should-not-leak",
                "map.grid",
                json!({"location": {"latitude": latitude, "longitude": longitude}}),
            ));
        }
    }
    assert_eq!(events.len(), 525);
    insert(&fixture.core, &events);
    let app = riauth::api::router(fixture.core);
    let (status, _, body) = send(
        &app,
        &format!(
            "/api/audit/map?since={base}&until={}&action=map.grid",
            base + 524
        ),
        Some(&fixture.admin),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let parsed = json_body(&body);
    let points = parsed["points"].as_array().unwrap();
    assert_eq!(points.len(), riauth::event_map::MAX_POINTS);
    assert_eq!(parsed["omitted_cells"].as_u64(), Some(25));
    assert_eq!(parsed["matched"].as_u64(), Some(525));
    assert_eq!(parsed["unknown"].as_u64(), Some(0));
    assert_eq!(parsed["truncated"].as_bool(), Some(false));
    assert_eq!(parsed["scanned"].as_u64(), Some(526));
    assert!(
        points
            .iter()
            .all(|point| point["count"].as_u64() == Some(1))
    );
    assert!(points.iter().any(|point| tenth(&point["latitude"]) == 0));
    assert!(points.iter().all(|point| tenth(&point["latitude"]) != 200));
    assert_public(&body);
}

#[tokio::test]
async fn scan_stops_at_the_event_ceiling() {
    let fixture = Fixture::new();
    let base = riauth::crypto::now() + 86_400;
    let total = riauth::event_map::MAX_SCAN + 50;
    let details = json!({"location": {"latitude": 12.0, "longitude": 34.0, "label": "Scan"}});
    fixture
        .core
        .store
        .write(|tx| {
            for index in 0..total {
                let at = base + index as u64;
                let id = format!("s{index:06}");
                tx.put(
                    "audit",
                    &format!("{at:020}-{id}"),
                    &audit(
                        at,
                        &id,
                        "actor-should-not-leak",
                        "map.scan",
                        details.clone(),
                    ),
                )?;
            }
            Ok(())
        })
        .unwrap();
    let app = riauth::api::router(fixture.core);
    let (status, _, body) = send(
        &app,
        &format!(
            "/api/audit/map?since={base}&until={}&action=map.scan",
            base + total as u64
        ),
        Some(&fixture.admin),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let parsed = json_body(&body);
    let ceiling = riauth::event_map::MAX_SCAN as u64;
    assert_eq!(parsed["truncated"].as_bool(), Some(true));
    assert_eq!(parsed["scanned"].as_u64(), Some(ceiling));
    assert_eq!(parsed["matched"].as_u64(), Some(ceiling));
    assert_eq!(parsed["unknown"].as_u64(), Some(0));
    assert_eq!(parsed["omitted_cells"].as_u64(), Some(0));
    assert_eq!(
        find_point(&parsed, 120, 340)["count"].as_u64(),
        Some(ceiling)
    );
    assert_public(&body);
}

#[tokio::test]
async fn portal_map_is_inline_and_makes_no_external_request() {
    let dir = tempfile::TempDir::new().unwrap();
    let core = Core::initialize(
        Config {
            data_dir: dir.path().into(),
            issuer: "http://localhost:9000/identity".into(),
            ..Default::default()
        },
        NewUser {
            username: "admin".into(),
            password: common::PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let app = riauth::api::router(core);
    let (status, headers, page) = send(&app, "/identity/events", None, None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        headers["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/html")
    );
    let policy = headers["content-security-policy"].to_str().unwrap();
    assert!(policy.contains("default-src 'none'"));
    assert!(policy.contains("script-src 'self'"));
    assert!(policy.contains("connect-src 'self'"));
    assert!(!policy.contains("http:") && !policy.contains("https:"));
    assert!(!policy.contains("unsafe-inline"));
    assert_eq!(headers["referrer-policy"], "no-referrer");
    assert!(page.contains("id=\"event-map\""));
    assert!(page.contains("id=\"graticule\""));
    assert!(page.contains("<svg"));
    assert!(page.contains("does not add tracking"));
    assert!(page.contains("Unknown locations"));
    assert!(page.contains("/identity/portal/assets/map.js"));
    assert!(page.contains("/identity/portal/assets/map.css"));
    assert!(!page.contains("__BASE__"));
    assert_offline(&page);
    let (status, _, trailing) = send(&app, "/identity/events/", None, None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(trailing.contains("id=\"event-map\""));
    for path in [
        "/identity/portal/assets/map.js",
        "/identity/portal/assets/map.css",
        "/identity/portal/assets/app.css",
    ] {
        let (status, headers, body) = send(&app, path, None, None, None).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        let content_type = headers["content-type"].to_str().unwrap();
        assert!(content_type.contains("text/"), "{path}: {content_type}");
        assert_offline(&body);
        if path.ends_with(".js") {
            assert!(body.contains("api/audit/map"));
            assert!(body.contains("same-origin"));
            assert!(!body.contains("Authorization"));
            assert!(!body.contains("Bearer"));
        }
    }
    let (status, _, apps) = send(&app, "/identity/apps", None, None, Some("text/html")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(apps.contains("Your applications"));
    assert!(!apps.contains("event-map"));
    assert!(!apps.contains("api/audit/map"));
    assert!(!apps.contains("map.js"));
    assert!(!apps.contains("/events"));
    let (status, _, root) = send(&app, "/identity/", None, None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(root.contains("\"portal\""));
    assert!(!root.contains("/events"));
    assert!(!root.contains("event-map"));
    let (status, _, html) = send(&app, "/identity/", None, None, Some("text/html")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("Your applications"));
    assert!(!html.contains("event-map"));
    assert!(!html.contains("/events"));
}
