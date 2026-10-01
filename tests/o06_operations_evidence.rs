//! O06 evidence over the real HTTP router and the real `riauth` command:
//! the offboarding aggregate and the Prometheus queue gauges, each with a
//! planted failure. A caller holding only `operations/metrics` reads neither
//! the offboarding aggregate nor any stored error text.
#![cfg(feature = "test-support")]

#[path = "common/mod.rs"]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use clap::Parser;
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    cli::{Cli, Command, OffboardCommand},
    crypto,
    model::NewUser,
    offboarding::{BUCKET, BeforeCommit, ExecuteAt, Job, ScheduleRequest, Status},
};
use serde_json::{Value, json};
use tower::ServiceExt;

const SECRET_ERROR: &str = "https://hooks.secret.example/offboard?access_token=SECRET-ALICE-TOKEN";

fn agent(fixture: &Fixture, id: &str, permissions: &[(&str, &str)]) -> String {
    let created = fixture
        .core
        .create_agent(
            &fixture.admin,
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
    created["credential"]["token"].as_str().unwrap().to_owned()
}

fn schedule(fixture: &Fixture, username: &str) -> String {
    fixture
        .core
        .create_user(
            &fixture.admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
                email: None,
                display_name: username.into(),
                admin: false,
            },
        )
        .unwrap();
    fixture
        .core
        .offboard_schedule(
            &fixture.admin,
            ScheduleRequest {
                username: username.into(),
                execute_at: ExecuteAt::Unix(crypto::now() + 3600),
                timezone: "UTC".into(),
            },
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// Alice's job fails five times and ends `failed`; Bob's stays `scheduled`.
/// The stored error of Alice's job carries a URL and token text.
fn plant_offboarding(fixture: &Fixture) -> String {
    let alice = schedule(fixture, "alice");
    schedule(fixture, "bob");
    for _ in 0..5 {
        fixture
            .core
            .store
            .write(|tx| {
                let mut job = tx.get::<Job>(BUCKET, &alice)?.unwrap();
                job.execute_at = 1;
                job.next_attempt = 1;
                tx.put(BUCKET, &alice, &job)?;
                Ok(())
            })
            .unwrap();
        assert!(
            fixture
                .core
                .offboard_process("alice-worker", |_| BeforeCommit::RetryableFailure)
                .unwrap()
        );
    }
    fixture
        .core
        .store
        .write(|tx| {
            let mut job = tx.get::<Job>(BUCKET, &alice)?.unwrap();
            assert_eq!(job.status, Status::Failed);
            job.last_error = Some(SECRET_ERROR.into());
            tx.put(BUCKET, &alice, &job)?;
            Ok(())
        })
        .unwrap();
    alice
}

async fn get(fixture: &Fixture, token: Option<&str>, path: &str) -> (StatusCode, String) {
    let mut request = Request::builder().method("GET").uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = riauth::api::router(fixture.core.clone())
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

async fn get_json(fixture: &Fixture, token: Option<&str>, path: &str) -> (StatusCode, Value) {
    let (status, body) = get(fixture, token, path).await;
    (status, serde_json::from_str(&body).unwrap())
}

fn assert_no_stored_text(text: &str) {
    for needle in [
        "https://",
        "secret.example",
        "access_token",
        "SECRET-",
        "last_error",
    ] {
        assert!(!text.contains(needle), "{needle} in {text}");
    }
}

fn count(report: &Value, key: &str) -> u64 {
    report["counts"][key].as_u64().unwrap()
}

/// The remote `riauth` command, with the caller's token in a private session file.
async fn cli(fixture: &Fixture, server: &str, token: &str, args: &[&str]) -> (i32, Value) {
    let session = fixture._dir.path().join("o06-session.json");
    riauth::config::write_private(
        &session,
        &serde_json::to_vec(&json!({
            "issuer": server, "token": token, "expires_at": crypto::now() + 600,
        }))
        .unwrap(),
        true,
    )
    .unwrap();
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_riauth"));
    command
        .args(["--server", server, "--session-file"])
        .arg(session)
        .args(["--json", "--non-interactive", "--request-timeout", "10"])
        .args(args)
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .current_dir(fixture._dir.path())
        .stdin(std::process::Stdio::null());
    let output = tokio::task::spawn_blocking(move || command.output().unwrap())
        .await
        .unwrap();
    let value = serde_json::from_slice(&output.stdout).expect("CLI JSON response");
    (output.status.code().unwrap(), value)
}

#[test]
fn offboard_diagnostics_command_parses_to_the_aggregate_read() {
    let cli =
        Cli::try_parse_from(["riauth", "--non-interactive", "offboard", "diagnostics"]).unwrap();
    assert!(matches!(
        cli.command,
        Command::Offboard {
            command: OffboardCommand::Diagnostics
        }
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn offboarding_aggregate_http_and_cli_enforce_the_permission_and_leak_nothing() {
    let fixture = Fixture::new();
    let alice = plant_offboarding(&fixture);
    let path = "/api/operations/offboarding";

    // Authorized read: one failed job with a planted stored error, one scheduled.
    let (status, report) = get_json(&fixture, Some(&fixture.admin), path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        report["schema_version"],
        "riauth.offboarding-diagnostics/v1"
    );
    assert_eq!(report["affects_readiness"], false);
    assert_eq!(count(&report, "jobs"), 2);
    assert_eq!(count(&report, "failed"), 1);
    assert_eq!(count(&report, "scheduled"), 1);
    assert_eq!(count(&report, "attention"), 1);
    assert_eq!(count(&report, "withheld"), 0);
    assert_eq!(report["listed"], 1);
    let item = &report["items"][0];
    assert_eq!(item["id"], alice);
    assert_eq!(item["status"], "failed");
    assert_eq!(item["has_error"], true);
    assert_eq!(item["attempts"], 5);
    assert_no_stored_text(&report.to_string());
    // The job read still holds the text an operator needs.
    let stored = fixture.core.offboard_get(&fixture.admin, &alice).unwrap();
    assert!(stored.to_string().contains("SECRET-ALICE-TOKEN"));

    // An operations reader without user.offboard sees counts, not the rows.
    let reader = agent(
        &fixture,
        "offboard-ops",
        &[("operations.read", "operations/offboarding")],
    );
    let (status, withheld) = get_json(&fixture, Some(&reader), path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(count(&withheld, "jobs"), 2);
    assert_eq!(count(&withheld, "withheld"), 2);
    assert_eq!(count(&withheld, "withheld_attention"), 1);
    assert_eq!(withheld["listed"], 0);
    assert_no_stored_text(&withheld.to_string());
    assert!(!withheld.to_string().contains("alice"));

    // A metrics-only token, a lifecycle-only token and no token are refused.
    let metrics = agent(
        &fixture,
        "metrics-only",
        &[("operations.read", "operations/metrics")],
    );
    let offboard_only = agent(
        &fixture,
        "offboard-only",
        &[("user.offboard", "user/alice")],
    );
    for token in [&metrics, &offboard_only] {
        let (status, body) = get_json(&fixture, Some(token), path).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"], "access_denied");
        assert_no_stored_text(&body.to_string());
    }
    let (status, body) = get(&fixture, None, path).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_no_stored_text(&body);

    // The real command reads the same route. A refusal exits 4 and prints no text.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_url = format!("http://{}", listener.local_addr().unwrap());
    let app = riauth::api::router(fixture.core.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let (code, output) = cli(
        &fixture,
        &server_url,
        &fixture.admin,
        &["offboard", "diagnostics"],
    )
    .await;
    assert_eq!(code, 0, "{output}");
    assert_eq!(output["ok"], true);
    let mut expected = report.clone();
    let mut observed = output["data"].clone();
    expected.as_object_mut().unwrap().remove("checked_at");
    observed.as_object_mut().unwrap().remove("checked_at");
    assert_eq!(observed, expected);
    assert_no_stored_text(&output.to_string());
    let (code, refused) = cli(
        &fixture,
        &server_url,
        &metrics,
        &["offboard", "diagnostics"],
    )
    .await;
    assert_eq!(code, 4, "{refused}");
    assert_eq!(refused["ok"], false);
    assert_eq!(refused["error"]["code"], "access_denied");
    assert_eq!(refused["error"]["http_status"], 403);
    assert_no_stored_text(&refused.to_string());
    server.abort();
}

fn series(text: &str, name: &str, queue: &str) -> f64 {
    let prefix = format!("{name}{{queue=\"{queue}\"}} ");
    text.lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("no {name} series for {queue}"))
        .parse()
        .unwrap()
}

#[tokio::test]
async fn prometheus_queue_failed_gauges_count_planted_failed_rows() {
    let fixture = Fixture::new();
    let metrics = agent(
        &fixture,
        "metrics-only",
        &[("operations.read", "operations/metrics")],
    );
    let path = "/api/operations/prometheus";
    let (status, before) = get(&fixture, Some(&metrics), path).await;
    assert_eq!(status, StatusCode::OK);
    for queue in riauth::store::maintenance::QUEUES {
        assert_eq!(
            series(&before, "riauth_queue_failed", queue),
            0.0,
            "{queue}"
        );
    }

    // One failed offboarding job (written through the worker), two failed
    // logout deliveries (one still pending, one stopped), one failed SSF
    // delivery. Every row carries stored error text that must stay out.
    plant_offboarding(&fixture);
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "logout_deliveries",
                "pending-error",
                &json!({"created_at": 100, "next_attempt": 200, "delivered_at": null,
                        "error": SECRET_ERROR}),
            )?;
            tx.put(
                "logout_deliveries",
                "stopped",
                &json!({"created_at": 101, "next_attempt": 201, "delivered_at": null,
                        "stopped": true, "error": SECRET_ERROR}),
            )?;
            tx.put(
                "logout_deliveries",
                "delivered",
                &json!({"created_at": 102, "next_attempt": 202, "delivered_at": 300}),
            )?;
            tx.put(
                "ssf_deliveries",
                "failed",
                &json!({"created_at": 100, "next_attempt": 200, "delivered_at": null,
                        "last_failed": true, "error": SECRET_ERROR}),
            )
        })
        .unwrap();

    let (status, text) = get(&fixture, Some(&metrics), path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(series(&text, "riauth_queue_failed", "offboard_jobs"), 1.0);
    assert_eq!(series(&text, "riauth_queue_pending", "offboard_jobs"), 1.0);
    assert_eq!(
        series(&text, "riauth_queue_failed", "logout_deliveries"),
        2.0
    );
    assert_eq!(
        series(&text, "riauth_queue_pending", "logout_deliveries"),
        1.0
    );
    assert_eq!(series(&text, "riauth_queue_failed", "ssf_deliveries"), 1.0);
    assert_eq!(series(&text, "riauth_queue_failed", "mail_deliveries"), 0.0);
    assert_eq!(
        series(&text, "riauth_queue_failed", "provisioning_jobs"),
        0.0
    );
    assert_eq!(
        series(&text, "riauth_queue_failed", "provisioning_deactivations"),
        0.0
    );
    assert!(text.contains("# TYPE riauth_queue_failed gauge"));
    assert_no_stored_text(&text);

    // The route is its own permission: the offboarding reader does not get it.
    let reader = agent(
        &fixture,
        "offboard-ops",
        &[("operations.read", "operations/offboarding")],
    );
    let (status, body) = get_json(&fixture, Some(&reader), path).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"], "access_denied");
    // Administrators read it too, with the same values.
    let (status, admin) = get(&fixture, Some(&fixture.admin), path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(series(&admin, "riauth_queue_failed", "offboard_jobs"), 1.0);
    assert_eq!(
        series(&admin, "riauth_queue_failed", "logout_deliveries"),
        2.0
    );
}
