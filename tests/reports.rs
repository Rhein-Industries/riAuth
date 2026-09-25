mod common;
use common::{Fixture, PASSWORD, text};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    context::{self, RequestContext},
    model::*,
    reports::{AuditReviewQuery, UserReportQuery},
};
use serde_json::{Value, json};
use tower::ServiceExt;

const NEW_PASSWORD: &str = "replacement-password";

async fn run_report_cli(
    app: axum::Router,
    directory: &std::path::Path,
    timeout: u64,
) -> std::process::Output {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let session = directory.join("session.json");
    riauth::config::write_private(
        &session,
        &serde_json::to_vec(&json!({
            "issuer": issuer, "token": "fixture-token", "expires_at": riauth::crypto::now() + 3600,
        }))
        .unwrap(),
        false,
    )
    .unwrap();
    let directory = directory.to_owned();
    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new(env!("CARGO_BIN_EXE_riauth"))
            .current_dir(&directory)
            .args([
                "--server",
                &issuer,
                "--session-file",
                session.to_str().unwrap(),
                "--config",
                "absent.toml",
                "--json",
                "--non-interactive",
                "--request-timeout",
                &timeout.to_string(),
                "report",
                "users",
                "--out",
                "users.csv",
            ])
            .env_remove("RIAUTH_AGENT_FILE")
            .env_remove("RIAUTH_RUN_ID")
            .env_remove("RIAUTH_CONFIG")
            .env_remove("RIAUTH_SERVER")
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    server.abort();
    output
}

#[tokio::test]
async fn cli_streams_large_multiline_report_and_counts_logical_records() {
    use axum::{extract::Query, response::IntoResponse, routing::get};
    let directory = tempfile::tempdir().unwrap();
    let app = axum::Router::new().route(
        "/api/reports/users.csv",
        get(
            |Query(query): Query<std::collections::HashMap<String, String>>| async move {
                let page: usize = query
                    .get("cursor")
                    .map(|value| value.parse().unwrap())
                    .unwrap_or(0);
                let body = format!(
                    "id,display_name\n{page},\"Unicode Ελληνικά, first\n\"\"quoted\"\" {}\"\n",
                    "x".repeat(256 * 1024)
                );
                let mut response = body.into_response();
                if page < 95 {
                    response
                        .headers_mut()
                        .insert("x-next-cursor", (page + 1).to_string().parse().unwrap());
                }
                response
            },
        ),
    );
    let output = run_report_cli(app, directory.path(), 30).await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let acknowledgement: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(acknowledgement["data"]["rows"], 96);
    assert_eq!(acknowledgement["data"]["pages"], 96);
    let path = directory.path().join("users.csv");
    assert!(std::fs::metadata(&path).unwrap().len() > 24 * 1024 * 1024);
    let csv = std::fs::read_to_string(&path).unwrap();
    assert_eq!(csv.matches("id,display_name\n").count(), 1);
    assert_eq!(
        csv.matches("Unicode Ελληνικά, first\n\"\"quoted\"\"")
            .count(),
        96
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
}

#[tokio::test]
async fn cli_report_failure_removes_partial_file_and_rejects_oversized_body() {
    use axum::{extract::Query, response::IntoResponse, routing::get};
    let directory = tempfile::tempdir().unwrap();
    let app = axum::Router::new().route(
        "/api/reports/users.csv",
        get(
            |Query(query): Query<std::collections::HashMap<String, String>>| async move {
                let mut response = if query.contains_key("cursor") {
                    axum::response::Response::new(Body::from_stream(futures_util::stream::iter(
                        (0..129).map(|_| Ok::<_, std::io::Error>(vec![b'x'; 65536])),
                    )))
                } else {
                    "id,name\n1,first\n".into_response()
                };
                response
                    .headers_mut()
                    .insert("x-next-cursor", "next".parse().unwrap());
                response
            },
        ),
    );
    let output = run_report_cli(app, directory.path(), 30).await;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("8388608 bytes"));
    assert!(!directory.path().join("users.csv").exists());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[tokio::test]
async fn cli_report_honors_configurable_request_timeout() {
    let directory = tempfile::tempdir().unwrap();
    let app = axum::Router::new().route(
        "/api/reports/users.csv",
        axum::routing::get(|| async {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            "id,name\n1,test\n"
        }),
    );
    let output = run_report_cli(app.clone(), directory.path(), 1).await;
    assert!(!output.status.success());
    assert!(!directory.path().join("users.csv").exists());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    std::fs::remove_file(directory.path().join("session.json")).unwrap();
    let output = run_report_cli(app, directory.path(), 3).await;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn cli_report_rejects_stalled_cursor_without_publishing_a_partial_export() {
    use axum::response::IntoResponse;
    let directory = tempfile::tempdir().unwrap();
    let app = axum::Router::new().route(
        "/api/reports/users.csv",
        axum::routing::get(|| async {
            let mut response = "id,name\n1,first\n".into_response();
            response
                .headers_mut()
                .insert("x-next-cursor", "same".parse().unwrap());
            response
        }),
    );
    let output = run_report_cli(app, directory.path(), 30).await;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cursor did not advance"));
    assert!(!directory.path().join("users.csv").exists());
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}

fn new_user(username: &str, display_name: &str) -> NewUser {
    NewUser {
        username: username.into(),
        password: PASSWORD.into(),
        email: Some(format!("{username}@example.test")),
        display_name: display_name.into(),
        admin: false,
    }
}

fn agent_token(f: &Fixture, id: &str, permissions: &[(&str, &str)]) -> String {
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.into(),
                ttl: 3600,
                permissions: permissions
                    .iter()
                    .map(|(action, resource)| Permission {
                        action: (*action).into(),
                        resource: (*resource).into(),
                    })
                    .collect(),
                parent: None,
            },
        )
        .unwrap();
    text(&created["credential"], "token")
}

fn audit_blob(f: &Fixture) -> String {
    serde_json::to_string(&f.core.store.list::<Value>("audit").unwrap()).unwrap()
}

fn parse_csv(input: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut chars = input.chars().peekable();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
        } else if c == '"' && field.is_empty() {
            quoted = true;
        } else if c == ',' {
            row.push(std::mem::take(&mut field));
        } else if c == '\n' {
            row.push(std::mem::take(&mut field));
            rows.push(std::mem::take(&mut row));
        } else if c != '\r' {
            field.push(c);
        }
    }
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        rows.push(row);
    }
    rows
}

fn usernames(csv: &str) -> Vec<String> {
    parse_csv(csv)
        .into_iter()
        .skip(1)
        .map(|row| row[1].clone())
        .collect()
}

#[test]
fn normal_user_cannot_review_or_export_audit() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let review = f
        .core
        .audit_review(&alice, AuditReviewQuery::default())
        .unwrap_err();
    assert_eq!(review.code, "access_denied");
    assert_eq!(review.status, StatusCode::FORBIDDEN);
    let audit_csv = f
        .core
        .audit_csv(&alice, AuditReviewQuery::default())
        .unwrap_err();
    assert_eq!(audit_csv.code, "access_denied");
    let users_csv = f
        .core
        .users_csv(&alice, UserReportQuery::default())
        .unwrap_err();
    assert_eq!(users_csv.code, "access_denied");
    let agent = agent_token(&f, "no-audit", &[("user.read", "user/alice")]);
    assert_eq!(
        f.core
            .audit_review(&agent, AuditReviewQuery::default())
            .unwrap_err()
            .code,
        "access_denied"
    );
}

#[test]
fn admin_can_filter_paginate_and_attribute_changes() {
    let f = Fixture::new();
    let admin_id = text(&f.core.me(&f.admin).unwrap()["user"], "id");
    let alice = context::scope(
        Some(RequestContext {
            request_id: "req-report".into(),
            run_id: Some("run-report".into()),
            ..Default::default()
        }),
        || {
            f.core
                .create_user(&f.admin, new_user("alice", "Alice"))
                .unwrap()
        },
    );
    let alice_id = text(&alice, "id");
    f.core
        .create_user(&f.admin, new_user("cara", "Cara"))
        .unwrap();
    f.core.create_group(&f.admin, "staff").unwrap();
    f.core
        .group_member(&f.admin, "staff", "alice", true)
        .unwrap();

    let unbounded = f
        .core
        .audit_review(&f.admin, AuditReviewQuery::default())
        .unwrap();
    assert_eq!(unbounded["limit"], 100);
    assert_eq!(unbounded["retention_seconds"], 90 * 24 * 60 * 60);
    assert_eq!(
        f.core
            .audit_review(
                &f.admin,
                AuditReviewQuery {
                    limit: Some(0),
                    ..Default::default()
                }
            )
            .unwrap()["limit"],
        1
    );
    assert_eq!(
        f.core
            .audit_review(
                &f.admin,
                AuditReviewQuery {
                    limit: Some(900),
                    ..Default::default()
                }
            )
            .unwrap()["limit"],
        500
    );

    let mut seen = Vec::new();
    let mut cursor = None;
    loop {
        let page = f
            .core
            .audit_review(
                &f.admin,
                AuditReviewQuery {
                    action: Some("user.create".into()),
                    limit: Some(1),
                    cursor: cursor.clone(),
                    ..Default::default()
                },
            )
            .unwrap();
        let events = page["events"].as_array().unwrap();
        if events.is_empty() {
            assert!(page["next_cursor"].is_null());
            break;
        }
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["action"], "user.create");
        assert_eq!(events[0]["actor"], admin_id);
        seen.push(events[0]["target"].as_str().unwrap().to_owned());
        match page["next_cursor"].as_str() {
            Some(next) => cursor = Some(next.to_owned()),
            None => break,
        }
    }
    seen.sort();
    assert_eq!(seen.len(), 2, "user.create pages: {seen:?}");
    assert!(seen.contains(&alice_id));

    let attributed = f
        .core
        .audit_review(
            &f.admin,
            AuditReviewQuery {
                run_id: Some("run-report".into()),
                limit: Some(10),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(attributed["events"].as_array().unwrap().len(), 1);
    let event = &attributed["events"][0];
    assert_eq!(event["actor"], admin_id);
    assert_eq!(event["action"], "user.create");
    assert_eq!(event["target"], alice_id);
    assert_eq!(event["run_id"], "run-report");
    assert_eq!(event["request_id"], "req-report");
    assert!(event["changes"][0]["before"].is_null());
    assert_eq!(event["changes"][0]["after"]["username"], "alice");
    assert_eq!(event["changes"][0]["after"]["password"], "[changed]");
    assert!(event.get("details").is_none());

    let prefixed = f
        .core
        .audit_review(
            &f.admin,
            AuditReviewQuery {
                action: Some("user.".into()),
                target: Some(alice_id.clone()),
                limit: Some(20),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        prefixed["events"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["action"].as_str().unwrap().starts_with("user."))
    );
    assert!(
        prefixed["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["action"] == "user.create")
    );

    let membership = f
        .core
        .audit_review(
            &f.admin,
            AuditReviewQuery {
                action: Some("group.member.add".into()),
                limit: Some(5),
                ..Default::default()
            },
        )
        .unwrap();
    let change = &membership["events"][0]["changes"][0];
    assert_eq!(change["before"]["members"], json!([]));
    assert_eq!(change["after"]["members"], json!([alice_id]));

    let first = f
        .core
        .audit_review(
            &f.admin,
            AuditReviewQuery {
                limit: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
    let cursor = text(&first, "next_cursor");
    let agent = agent_token(&f, "audit-agent", &[("audit.read", "audit/events")]);
    assert_eq!(
        f.core
            .audit_review(
                &agent,
                AuditReviewQuery {
                    limit: Some(1),
                    cursor: Some(cursor.clone()),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .code,
        "invalid_request"
    );
    assert_eq!(
        f.core
            .audit_review(
                &f.admin,
                AuditReviewQuery {
                    limit: Some(1),
                    action: Some("user.create".into()),
                    cursor: Some(cursor),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .code,
        "invalid_request"
    );
}

#[test]
fn password_change_audit_has_no_hash_and_secrets_stay_out() {
    let f = Fixture::new();
    let admin_id = text(&f.core.me(&f.admin).unwrap()["user"], "id");
    let before = f
        .core
        .store
        .get::<User>("users", &admin_id)
        .unwrap()
        .unwrap();
    let secret = f.client("app", true).unwrap();
    let rotated = f.core.rotate_client_secret(&f.admin, "app").unwrap();
    let new_secret = rotated["client_secret"].as_str().unwrap().to_owned();
    let agent_token = agent_token(&f, "secret-agent", &[("audit.read", "audit/events")]);
    f.core
        .change_password(&f.admin, PASSWORD.into(), NEW_PASSWORD.into(), None)
        .unwrap();
    let after = f
        .core
        .store
        .get::<User>("users", &admin_id)
        .unwrap()
        .unwrap();
    assert_ne!(before.password_hash, after.password_hash);
    assert!(before.password_hash.starts_with("$argon2"));
    let admin = text(
        &f.core
            .login("admin".into(), NEW_PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let blob = audit_blob(&f);
    for secret in [
        before.password_hash.as_str(),
        after.password_hash.as_str(),
        before.pairwise_seed.as_str(),
        PASSWORD,
        NEW_PASSWORD,
        secret.as_str(),
        new_secret.as_str(),
        agent_token.as_str(),
        f.admin.as_str(),
        admin.as_str(),
    ] {
        assert!(!blob.contains(secret), "audit stored {secret}");
    }
    for name in ["password_hash", "secret_hash", "token_hash", "totp_secret"] {
        assert!(!blob.contains(name), "audit stored field {name}");
    }
    let review = f
        .core
        .audit_review(
            &admin,
            AuditReviewQuery {
                action: Some("user.password.change".into()),
                limit: Some(10),
                ..Default::default()
            },
        )
        .unwrap();
    let event = &review["events"][0];
    assert_eq!(event["actor"], admin_id);
    assert_eq!(event["target"], admin_id);
    let changes = event["changes"].to_string();
    assert!(changes.contains("\"password\":\"[changed]\""));
    assert!(changes.contains("\"password\":\"[redacted]\""));
    assert!(
        event["changes"][0]["changed_credentials"]
            .as_array()
            .unwrap()
            .iter()
            .any(|label| label == "password")
    );
    let csv = f
        .core
        .audit_csv(
            &admin,
            AuditReviewQuery {
                limit: Some(1000),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        parse_csv(&csv.body)[0],
        [
            "id",
            "at",
            "actor",
            "action",
            "target",
            "run_id",
            "request_id"
        ]
    );
    assert!(!csv.body.contains('\r'));
    assert!(!csv.body.contains("$argon2"));
    assert!(!csv.body.contains("password_hash"));
    assert!(!csv.body.contains(NEW_PASSWORD));
    assert!(!csv.body.contains(&new_secret));
    assert!(!csv.body.contains(&agent_token));
}

#[test]
fn user_csv_quotes_formulas_and_round_trips_unicode() {
    let f = Fixture::new();
    f.core
        .create_user(&f.admin, new_user("alice", "=cmd"))
        .unwrap();
    f.core.create_group(&f.admin, "staff").unwrap();
    f.core
        .group_member(&f.admin, "staff", "alice", true)
        .unwrap();
    let page = f
        .core
        .users_csv(
            &f.admin,
            UserReportQuery {
                limit: Some(100),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        page.body
            .starts_with("id,username,email,display_name,enabled,admin,created_at,groups\n")
    );
    assert!(!page.body.starts_with('\u{feff}'));
    assert!(!page.body.contains('\r'));
    let rows = parse_csv(&page.body);
    let alice = rows.iter().find(|row| row[1] == "alice").unwrap();
    assert_eq!(alice[3], "'=cmd");
    assert_eq!(alice[2], "alice@example.test");
    assert_eq!(alice[4], "true");
    assert_eq!(alice[5], "false");
    assert_eq!(alice[7], "staff");
    assert!(page.body.contains("'=cmd"));
    assert!(!page.body.contains(",=cmd"));

    for (raw, expect) in [("+1", "'+1"), ("-1", "'-1"), ("@cmd", "'@cmd")] {
        f.core
            .update_user(
                &f.admin,
                "alice",
                UserPatch {
                    display_name: Some(raw.into()),
                    ..Default::default()
                },
            )
            .unwrap();
        let body = f
            .core
            .users_csv(&f.admin, UserReportQuery::default())
            .unwrap()
            .body;
        let alice = parse_csv(&body)
            .into_iter()
            .find(|row| row[1] == "alice")
            .unwrap();
        assert_eq!(alice[3], expect);
    }

    let display = "名字 \"café\", ok";
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                display_name: Some(display.into()),
                ..Default::default()
            },
        )
        .unwrap();
    let body = f
        .core
        .users_csv(&f.admin, UserReportQuery::default())
        .unwrap()
        .body;
    assert!(body.contains("名字"));
    assert!(body.contains("café"));
    let alice = parse_csv(&body)
        .into_iter()
        .find(|row| row[1] == "alice")
        .unwrap();
    assert_eq!(alice[3], display);

    let (id, mut stored) = f
        .core
        .store
        .list::<User>("users")
        .unwrap()
        .into_iter()
        .find(|(_, user)| user.username == "alice")
        .unwrap();
    stored.display_name = "\tcmd\r".into();
    f.core
        .store
        .write(|tx| tx.put("users", &id, &stored))
        .unwrap();
    let body = f
        .core
        .users_csv(&f.admin, UserReportQuery::default())
        .unwrap()
        .body;
    let alice = parse_csv(&body)
        .into_iter()
        .find(|row| row.get(1).is_some_and(|name| name == "alice"))
        .unwrap();
    assert_eq!(alice[3], "'\tcmd\r");
}

#[test]
fn agent_csv_hides_users_outside_user_read() {
    let f = Fixture::new();
    let alice = f
        .core
        .create_user(&f.admin, new_user("alice", "Alice"))
        .unwrap();
    let bob = f
        .core
        .create_user(&f.admin, new_user("bob", "Bob"))
        .unwrap();
    let admin_id = text(&f.core.me(&f.admin).unwrap()["user"], "id");
    let token = agent_token(&f, "reader", &[("user.read", "user/alice")]);
    let page = f
        .core
        .users_csv(
            &token,
            UserReportQuery {
                limit: Some(100),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(usernames(&page.body), vec!["alice".to_string()]);
    assert!(!page.body.contains(&text(&bob, "id")));
    assert!(!page.body.contains(&admin_id));
    assert!(!page.body.contains("password_hash"));
    assert!(!page.body.contains(PASSWORD));
    assert!(page.body.contains(&text(&alice, "id")));
    let filtered = f
        .core
        .users_csv(
            &f.admin,
            UserReportQuery {
                filter: Some("alice".into()),
                limit: Some(100),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(usernames(&filtered.body), vec!["alice".to_string()]);
}

#[test]
fn complete_pagination_returns_every_row() {
    let f = Fixture::new();
    f.core
        .create_user(&f.admin, new_user("alice", "Alice"))
        .unwrap();
    f.core
        .create_user(&f.admin, new_user("bob", "Bob"))
        .unwrap();
    let mut names = Vec::new();
    let mut cursor = None;
    let mut pages = 0;
    loop {
        let page = f
            .core
            .users_csv(
                &f.admin,
                UserReportQuery {
                    limit: Some(1),
                    cursor,
                    ..Default::default()
                },
            )
            .unwrap();
        pages += 1;
        let parsed = usernames(&page.body);
        assert!(parsed.len() <= 1);
        names.extend(parsed);
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
        assert!(pages < 10);
    }
    names.sort();
    assert_eq!(
        names,
        vec!["admin".to_string(), "alice".into(), "bob".into()]
    );

    let listed = f.core.audit_events(&f.admin, 100).unwrap();
    let mut seen = Vec::new();
    let mut cursor = None;
    loop {
        let page = f
            .core
            .audit_review(
                &f.admin,
                AuditReviewQuery {
                    limit: Some(1),
                    cursor,
                    ..Default::default()
                },
            )
            .unwrap();
        let events = page["events"].as_array().unwrap();
        if events.is_empty() {
            break;
        }
        seen.push(events[0]["id"].as_str().unwrap().to_owned());
        match page["next_cursor"].as_str() {
            Some(next) => cursor = Some(next.to_owned()),
            None => break,
        }
    }
    let expected: Vec<_> = listed
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(seen, expected);
    assert!(seen.len() > 3);
}

#[test]
fn audit_retention_is_ninety_days() {
    let f = Fixture::new();
    let at = riauth::crypto::now();
    let retention = 90 * 24 * 60 * 60;
    let kept_at = at.saturating_sub(retention);
    let dropped_at = at.saturating_sub(retention + 1);
    f.core
        .store
        .write(|tx| {
            for (id, stamp) in [("kept-boundary", kept_at), ("dropped-old", dropped_at)] {
                tx.put(
                    "audit",
                    &format!("{stamp:020}-{id}"),
                    &Audit {
                        id: id.into(),
                        at: stamp,
                        actor: "retention".into(),
                        action: "retention.probe".into(),
                        target: "audit".into(),
                        run_id: None,
                        details: json!({}),
                    },
                )?;
            }
            Ok(())
        })
        .unwrap();
    f.core.cleanup().unwrap();
    let events = f.core.store.list::<Audit>("audit").unwrap();
    assert!(events.iter().any(|(_, event)| event.id == "kept-boundary"));
    assert!(events.iter().all(|(_, event)| event.id != "dropped-old"));
}

#[tokio::test]
async fn http_reports_enforce_permission_and_csv_cursor() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let app = riauth::api::router(f.core.clone());
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/reports/audit.csv")
                .header("authorization", format!("Bearer {alice}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    let denied_review = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/audit/review")
                .header("authorization", format!("Bearer {alice}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied_review.status(), StatusCode::FORBIDDEN);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/reports/users.csv?limit=1")
                .header("authorization", format!("Bearer {}", f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/csv; charset=utf-8"
    );
    let cursor = response
        .headers()
        .get("x-next-cursor")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(usernames(&text).len(), 1);

    let mut names = usernames(&text);
    let next = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/reports/users.csv?limit=1&cursor={cursor}"))
                .header("authorization", format!("Bearer {}", f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(next.status(), StatusCode::OK);
    let more = next.into_body().collect().await.unwrap().to_bytes();
    names.extend(usernames(std::str::from_utf8(&more).unwrap()));
    names.sort();
    assert_eq!(names, vec!["admin".to_string(), "alice".into()]);
}
