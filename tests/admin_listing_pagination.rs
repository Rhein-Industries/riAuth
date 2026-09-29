mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use common::{Fixture, text};
use riauth::{
    agent::{Agent, NewAgent, Permission},
    model::User,
    telemetry::ReadContext,
};
use serde_json::Value;
use tower::ServiceExt;

fn reader(fixture: &Fixture, id: &str, resources: &[&str]) -> String {
    let created = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: id.into(),
                ttl: 600,
                parent: None,
                permissions: resources
                    .iter()
                    .map(|resource| Permission {
                        action: "user.read".into(),
                        resource: (*resource).into(),
                    })
                    .collect(),
            },
        )
        .unwrap();
    text(&created["credential"], "token")
}

#[test]
fn user_listing_pages_bucket_and_keeps_order_and_per_row_permissions() {
    let fixture = Fixture::new();
    let template = fixture
        .core
        .store
        .read(|tx| Ok(tx.list::<User>("users")?.pop().unwrap().1))
        .unwrap();
    let mut expected = vec![(template.id.clone(), template.username.clone())];
    fixture
        .core
        .store
        .write(|tx| {
            for index in 0..130 {
                let mut user = template.clone();
                user.id = format!("u{index:03}");
                user.username = format!("reader-{index:03}");
                user.display_name = user.username.clone();
                user.admin = false;
                tx.put("users", &user.id, &user)?;
                tx.put("usernames", &user.username, &user.id)?;
                expected.push((user.id, user.username));
            }
            Ok(())
        })
        .unwrap();
    expected.sort_by(|left, right| left.0.cmp(&right.0));
    let all_reader = reader(&fixture, "all-users-reader", &["*"]);
    let scoped_reader = reader(
        &fixture,
        "two-users-reader",
        &["user/reader-000", "user/reader-129"],
    );
    let scans = &fixture.core.store.telemetry().reads;
    let counts = || {
        (
            scans.scans(ReadContext::Read, false).count(),
            scans.scans(ReadContext::Read, true).count(),
            scans.scans(ReadContext::Read, true).sum(),
        )
    };

    let before = counts();
    let all = fixture.core.list_users(&all_reader).unwrap();
    let after = counts();
    assert_eq!(after.0, before.0, "listing must not load the full bucket");
    assert_eq!(after.1 - before.1, 2);
    assert_eq!(after.2 - before.2, 131);
    let usernames: Vec<_> = all
        .as_array()
        .unwrap()
        .iter()
        .map(|user| text(user, "username"))
        .collect();
    assert_eq!(
        usernames,
        expected
            .into_iter()
            .map(|(_, username)| username)
            .collect::<Vec<_>>()
    );
    assert!(
        all.as_array()
            .unwrap()
            .iter()
            .all(|user| user.get("password_hash").is_none())
    );

    let before = counts();
    let scoped = fixture.core.list_users(&scoped_reader).unwrap();
    let after = counts();
    assert_eq!(after.0, before.0);
    assert_eq!(after.1 - before.1, 2);
    assert_eq!(after.2 - before.2, 131);
    assert_eq!(
        scoped
            .as_array()
            .unwrap()
            .iter()
            .map(|user| text(user, "username"))
            .collect::<Vec<_>>(),
        ["reader-000", "reader-129"]
    );
    assert_eq!(fixture.core.list_users(&fixture.admin).unwrap(), all);
    fixture
        .core
        .revoke_agent(&fixture.admin, "two-users-reader")
        .unwrap();
    assert!(fixture.core.list_users(&scoped_reader).is_err());
}

async fn get_users(app: &axum::Router, path: &str, token: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn external_user_pages_preserve_array_contract_and_reject_stale_cursors() {
    let fixture = Fixture::new();
    let template = fixture
        .core
        .store
        .read(|tx| Ok(tx.list::<User>("users")?.pop().unwrap().1))
        .unwrap();
    fixture
        .core
        .store
        .write(|tx| {
            for index in 0..130 {
                let mut user = template.clone();
                user.id = format!("u{index:03}");
                user.username = format!("reader-{index:03}");
                user.admin = false;
                tx.put("users", &user.id, &user)?;
                tx.put("usernames", &user.username, &user.id)?;
            }
            Ok(())
        })
        .unwrap();
    let all_reader = reader(&fixture, "page-all", &["*"]);
    let scoped_reader = reader(
        &fixture,
        "page-scoped",
        &["user/reader-000", "user/reader-129"],
    );
    let app = riauth::api::router(fixture.core.clone());

    let (status, legacy) = get_users(&app, "/api/users", &all_reader).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(legacy.as_array().unwrap().len(), 131);
    let mut cursor = None;
    let mut collected = Vec::new();
    let mut first_cursor = None;
    let scans = &fixture.core.store.telemetry().reads;
    loop {
        let path = cursor.as_ref().map_or_else(
            || "/api/users?limit=17".to_owned(),
            |cursor: &String| format!("/api/users?limit=17&cursor={cursor}"),
        );
        let unbounded = scans.scans(ReadContext::Read, false).count();
        let bounded = scans.scans(ReadContext::Read, true).sum();
        let (status, page) = get_users(&app, &path, &all_reader).await;
        assert_eq!(status, StatusCode::OK, "{page}");
        assert_eq!(scans.scans(ReadContext::Read, false).count(), unbounded);
        assert!(scans.scans(ReadContext::Read, true).sum() - bounded <= 128);
        assert_eq!(page["limit"], 17);
        let items = page["items"].as_array().unwrap();
        assert!(items.len() <= 17);
        collected.extend(items.iter().cloned());
        cursor = page["next_cursor"].as_str().map(str::to_owned);
        if first_cursor.is_none() {
            first_cursor = cursor.clone();
        }
        if cursor.is_none() {
            break;
        }
        assert!(collected.len() <= 131, "page walk did not advance");
    }
    assert_eq!(Value::Array(collected), legacy);
    let first_cursor = first_cursor.unwrap();
    assert_eq!(
        get_users(&app, "/api/users?limit=101", &all_reader).await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        get_users(
            &app,
            &format!("/api/users?limit=18&cursor={first_cursor}"),
            &all_reader
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        get_users(
            &app,
            &format!("/api/users?limit=17&cursor={first_cursor}"),
            &scoped_reader
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );

    let (status, first) = get_users(&app, "/api/users?limit=1", &scoped_reader).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["items"][0]["username"], "reader-000");
    let scoped_cursor = text(&first, "next_cursor");
    let (status, second) = get_users(
        &app,
        &format!("/api/users?limit=1&cursor={scoped_cursor}"),
        &scoped_reader,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(second["items"][0]["username"], "reader-129");

    // A UserView field outside the provisioning projection still invalidates
    // an earlier page, even when the management revision has not changed.
    let (status, before_change) = get_users(&app, "/api/users?limit=17", &all_reader).await;
    assert_eq!(status, StatusCode::OK);
    let cursor_before_change = text(&before_change, "next_cursor");
    let revision: u64 = fixture.core.store.get("meta", "revision").unwrap().unwrap();
    let generation: u64 = fixture
        .core
        .store
        .get("user_listing_generation", "all")
        .unwrap()
        .unwrap();
    fixture
        .core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", "u020")?.unwrap();
            user.email_verified = !user.email_verified;
            tx.put("users", "u020", &user)
        })
        .unwrap();
    assert_eq!(
        fixture
            .core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap(),
        revision
    );
    assert!(
        fixture
            .core
            .store
            .get::<u64>("user_listing_generation", "all")
            .unwrap()
            .unwrap()
            > generation
    );
    let (status, stale) = get_users(
        &app,
        &format!("/api/users?limit=17&cursor={cursor_before_change}"),
        &all_reader,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{stale}");

    let (status, before_auth_change) = get_users(&app, "/api/users?limit=1", &scoped_reader).await;
    assert_eq!(status, StatusCode::OK);
    let cursor_before_auth_change = text(&before_auth_change, "next_cursor");
    fixture
        .core
        .store
        .write(|tx| {
            let mut agent: Agent = tx.get("agents", "page-scoped")?.unwrap();
            agent.permissions.pop();
            tx.put("agents", "page-scoped", &agent)
        })
        .unwrap();
    let (status, stale) = get_users(
        &app,
        &format!("/api/users?limit=1&cursor={cursor_before_auth_change}"),
        &scoped_reader,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{stale}");

    let (status, before_revision_change) =
        get_users(&app, "/api/users?limit=17", &all_reader).await;
    assert_eq!(status, StatusCode::OK);
    let cursor_before_revision_change = text(&before_revision_change, "next_cursor");
    fixture
        .core
        .create_group(&fixture.admin, "page-revision-change")
        .unwrap();
    let (status, stale) = get_users(
        &app,
        &format!("/api/users?limit=17&cursor={cursor_before_revision_change}"),
        &all_reader,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{stale}");
}
