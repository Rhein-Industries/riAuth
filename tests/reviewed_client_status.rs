mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    core::Core,
    error::Result,
    model::{
        Client, ClientPatch, ClientStatusBinding, ClientStatusInput, Device, Family, Grant,
        NewUser, ProviderSettings, UserPatch,
    },
    oidc::TokenRequest,
    state::Manifest,
};
use serde_json::{Value, json};
use tower::ServiceExt;

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
    login(f, name)
}
fn login(f: &Fixture, name: &str) -> String {
    f.core.login(name.into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .into()
}
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
        .find(|c| c.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .into()
}
fn revision(f: &Fixture) -> u64 {
    f.core.store.get("meta", "revision").unwrap().unwrap_or(0)
}
fn id(change: &Value) -> &str {
    change["proposal"]["id"].as_str().unwrap()
}
fn binding(change: &Value) -> ClientStatusBinding {
    ClientStatusBinding {
        digest: change["digest"].as_str().unwrap().into(),
    }
}
fn input() -> ClientStatusInput {
    ClientStatusInput { enabled: false }
}
fn client(f: &Fixture) -> Client {
    f.core.store.get("clients", "app").unwrap().unwrap()
}
fn refused<T>(f: &Fixture, action: impl FnOnce() -> Result<T>, expected: u16) {
    let before = f.snapshot().unwrap();
    assert_eq!(
        action().err().expect("must refuse").status.as_u16(),
        expected
    );
    f.assert_snapshot(&before);
}
fn approved(f: &Fixture, reviewer: &str) -> Value {
    let change = f
        .core
        .stage_client_status(&f.admin, "app", input())
        .unwrap();
    f.core
        .approve_client_status_change(reviewer, id(&change), binding(&change))
        .unwrap()
}

#[derive(Clone, Copy)]
enum Auth<'a> {
    Bearer(&'a str),
    Browser(&'a str, Option<&'a str>),
}
async fn call(
    app: &axum::Router,
    method: &str,
    path: &str,
    auth: Auth<'_>,
    body: Value,
    revision: Option<u64>,
    key: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost:9000")
        .header("content-type", "application/json");
    request = match auth {
        Auth::Bearer(token) => request.header("authorization", format!("Bearer {token}")),
        Auth::Browser(cookie, origin) => {
            request = request
                .header("cookie", format!("riauth_sso={cookie}"))
                .header("x-riauth-portal", "1");
            if let Some(origin) = origin {
                request = request.header("origin", origin);
            }
            request
        }
    };
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into())),
    )
}

// Exercise the real remote CLI against this same service, with private fixture credentials.
async fn cli(
    binary: &std::path::Path,
    f: &Fixture,
    server: &str,
    token: &str,
    args: &[&str],
) -> (i32, Value) {
    let session = f._dir.path().join("client-status-session.json");
    riauth::config::write_private(
        &session,
        &serde_json::to_vec(&json!({
            "issuer": server, "token": token, "expires_at": riauth::crypto::now() + 600,
        }))
        .unwrap(),
        true,
    )
    .unwrap();
    let mut command = std::process::Command::new(binary);
    command
        .args(["--server", server, "--session-file"])
        .arg(session)
        .args(["--json", "--non-interactive", "--request-timeout", "10"])
        .args(args)
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .current_dir(f._dir.path())
        .stdin(std::process::Stdio::null());
    let output = tokio::task::spawn_blocking(move || command.output().unwrap())
        .await
        .unwrap();
    let value = serde_json::from_slice(&output.stdout).expect("CLI JSON response");
    (output.status.code().unwrap(), value)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exact_status_revokes_bound_grants_once_and_blocks_api_cli_state_bypass() {
    // Hold this build's CLI inode while other worktrees share Cargo's target.
    let built = std::path::Path::new(env!("CARGO_BIN_EXE_riauth"));
    let pinned = tempfile::Builder::new()
        .prefix(".reviewed-status-")
        .tempdir_in(built.parent().unwrap())
        .unwrap();
    let binary = pinned.path().join("riauth");
    std::fs::hard_link(built, &binary).unwrap();
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    let alice = f.user("alice");
    let secret = f.client_with_settings(
        "app",
        true,
        ProviderSettings {
            backchannel_logout_uri: Some("https://app.example.test/logout".into()),
            ..Default::default()
        },
    );
    let scoped = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "operator".into(),
                ttl: 600,
                parent: None,
                permissions: ["client.read", "client.write", "client.rotate"]
                    .map(|action| Permission {
                        action: action.into(),
                        resource: "client/app".into(),
                    })
                    .into(),
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let owner = f.user("owner");
    f.core
        .set_human_grants(
            &f.admin,
            "owner",
            vec![riauth::delegation::GrantInput {
                role: riauth::delegation::HumanRole::ApplicationOwner,
                scope: "client/app".into(),
            }],
        )
        .unwrap();
    let browser_cookie = cookie(&f.core, &f.admin);
    let app = riauth::api::router(f.core.clone());
    for auth in [
        Auth::Bearer(&f.admin),
        Auth::Bearer(&scoped),
        Auth::Browser(&browser_cookie, Some("http://localhost:9000")),
    ] {
        let path = if matches!(auth, Auth::Browser(..)) {
            "/api/admin/clients/app"
        } else {
            "/api/clients/app"
        };
        let before = f.snapshot().unwrap();
        let response = call(
            &app,
            "PATCH",
            path,
            auth,
            json!(input()),
            Some(revision(&f)),
            Some("direct-status-denied"),
        )
        .await;
        assert_eq!(response.0, StatusCode::CONFLICT, "{}", response.1);
        f.assert_http_mutation_snapshot(&before);
    }
    refused(
        &f,
        || {
            f.core.update_client(
                &f.admin,
                "app",
                ClientPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
        },
        409,
    );
    for token in [&owner, &scoped] {
        refused(
            &f,
            || f.core.stage_client_status(token, "app", input()),
            403,
        );
    }
    let mut manifest: Manifest =
        serde_json::from_value(f.core.export_state(&f.admin).unwrap()["manifest"].clone()).unwrap();
    manifest.users.clear();
    manifest.groups.clear();
    manifest.clients[0].enabled = false;
    refused(&f, || f.core.plan_state(&f.admin, manifest.clone()), 409);
    // Final apply checks the same fence even for a previously ordinary plan.
    manifest.clients[0].enabled = true;
    manifest.clients[0].name = "Ordinary edit".into();
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    let baseline = client(&f);
    let mut drifted = baseline.clone();
    drifted.enabled = false;
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &drifted))
        .unwrap();
    assert_eq!(revision(&f), plan.base_revision);
    refused(
        &f,
        || {
            f.core.apply_state(
                &f.admin,
                riauth::state::ApplyRequest {
                    plan,
                    secrets: Default::default(),
                    run_id: None,
                },
            )
        },
        409,
    );
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &baseline))
        .unwrap();
    // Ordinary owner edits, identical enabled values and explicit rotation remain usable.
    f.core
        .update_client(
            &owner,
            "app",
            ClientPatch {
                name: Some("Reviewed app".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let at = revision(&f);
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                enabled: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(revision(&f), at);
    let tokens = f.tokens("app", &alice, secret.clone());
    let pending = f.exchange_request("app", &alice, secret.clone());
    let device = f
        .core
        .device_start(TokenRequest {
            client_id: Some("app".into()),
            client_secret: secret.clone(),
            scope: Some("openid".into()),
            ..Default::default()
        })
        .unwrap();
    let device_key = riauth::crypto::digest(device["device_code"].as_str().unwrap());
    let device_row: Device = f.core.store.get("devices", &device_key).unwrap().unwrap();
    let change = approved(&f, &reviewer);
    assert_eq!(change["proposal"]["before"], json!({"enabled":true}));
    assert_eq!(change["proposal"]["after"], json!(input()));
    assert_eq!(
        change["proposal"]["effects"],
        json!({
            "revoke_families":1, "delete_authorization_codes":2, "delete_device_codes":1,
            "end_rp_sessions":1, "queue_backchannel_logouts":1, "restore_revoked_grants":false,
        })
    );
    let at = revision(&f);
    // Token issuance does not advance management revision. It still invalidates approval.
    let more_tokens = f.tokens("app", &alice, secret.clone());
    assert_eq!(revision(&f), at);
    refused(
        &f,
        || {
            f.core
                .execute_client_status_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    assert!(client(&f).enabled);
    f.core
        .cancel_client_status_change(&f.admin, id(&change), binding(&change))
        .unwrap();
    let change = approved(&f, &reviewer);
    assert_eq!(change["proposal"]["effects"]["revoke_families"], 2);
    let baseline = serde_json::to_value(client(&f)).unwrap();
    // A new cross-client exchange sharing a family is explicitly outside this slice.
    let access_key = riauth::crypto::digest(tokens["access_token"].as_str().unwrap());
    let mut sibling: Grant = f.core.store.get("access", &access_key).unwrap().unwrap();
    sibling.client_id = "another-app".into();
    f.core
        .store
        .write(|tx| tx.put("access", "cross-client-exchange", &sibling))
        .unwrap();
    refused(
        &f,
        || f.core.stage_client_status(&f.admin, "app", input()),
        409,
    );
    refused(
        &f,
        || {
            f.core
                .execute_client_status_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    f.core
        .store
        .write(|tx| tx.delete("access", "cross-client-exchange"))
        .unwrap();

    // Real exchange lineage uses a separate family. Foreign grants depending on
    // this client as requester, subject or actor must also fail closed, including
    // when they arrive after approval without changing management revision.
    let separate = Family {
        expires_at: sibling.expires_at,
        revoked: false,
    };
    sibling.family_id = "separate-exchange-family".into();
    for edge in ["requester", "subject", "actor"] {
        sibling.exchange = Some(riauth::exchange::ExchangeGrant {
            requester_id: if edge == "requester" {
                "app"
            } else {
                "another-app"
            }
            .into(),
            subject_hash: if edge == "subject" {
                access_key.clone()
            } else {
                "another-subject".into()
            },
            actor_hash: (edge == "actor").then(|| access_key.clone()),
            act: None,
            service_subject: None,
        });
        f.core
            .store
            .write(|tx| {
                tx.put("families", &sibling.family_id, &separate)?;
                tx.put("access", "cross-client-exchange", &sibling)
            })
            .unwrap();
        assert_eq!(revision(&f), at);
        refused(
            &f,
            || f.core.stage_client_status(&f.admin, "app", input()),
            409,
        );
        refused(
            &f,
            || {
                f.core
                    .execute_client_status_change(&executor, id(&change), binding(&change))
            },
            409,
        );
        f.core
            .store
            .write(|tx| {
                tx.delete("access", "cross-client-exchange")?;
                tx.delete("families", &sibling.family_id)
            })
            .unwrap();
    }

    let execute = format!("/api/client-status-changes/{}/execute", id(&change));
    let result = call(
        &app,
        "POST",
        &execute,
        Auth::Bearer(&executor),
        json!(binding(&change)),
        Some(at),
        Some("execute-status"),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK, "{}", result.1);
    assert_eq!(result.1["status"], "executed");
    assert_eq!(result.1["proposal"], change["proposal"]);
    let mut expected = baseline;
    expected["enabled"] = json!(false);
    assert_eq!(serde_json::to_value(client(&f)).unwrap(), expected);
    assert_eq!(revision(&f), at + 1);
    assert!(
        f.core
            .store
            .list::<Family>("families")
            .unwrap()
            .iter()
            .all(|(_, x)| x.revoked)
    );
    assert!(f.core.store.list::<Value>("codes").unwrap().is_empty());
    assert!(f.core.store.list::<Value>("devices").unwrap().is_empty());
    assert!(
        f.core
            .store
            .get::<Value>("device_users", &device_row.user_code_hash)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .list::<riauth::logout::RpSession>("rp_sessions")
            .unwrap()
            .iter()
            .all(|(_, x)| x.ended)
    );
    assert_eq!(
        f.core
            .store
            .list::<Value>("logout_deliveries")
            .unwrap()
            .len(),
        1
    );
    assert!(
        f.core.me(&alice).is_ok(),
        "User sign-in session survives application disable"
    );
    let before = f.snapshot().unwrap();
    assert_eq!(
        call(
            &app,
            "POST",
            &execute,
            Auth::Bearer(&executor),
            json!(binding(&change)),
            Some(at),
            Some("execute-status")
        )
        .await,
        result
    );
    assert_eq!(
        call(
            &app,
            "POST",
            &execute,
            Auth::Bearer(&executor),
            json!(binding(&change)),
            None,
            Some("replay-status")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&before);
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    for action in ["stage", "approve", "execute"] {
        let matches: Vec<_> = events
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| {
                e["action"] == format!("reviewed_client_statuses.{action}")
                    && e["details"]["change_id"] == id(&change)
            })
            .collect();
        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches[0]["details"]["effects"],
            change["proposal"]["effects"]
        );
    }
    let serialized = format!("{events}{change}{}\n", result.1);
    for sensitive in [
        secret.as_deref().unwrap(),
        expected["secret_hash"].as_str().unwrap(),
        tokens["access_token"].as_str().unwrap(),
        &access_key,
        &device_row.user_code_hash,
        &sibling.family_id,
    ] {
        assert!(!serialized.contains(sensitive));
    }

    // Real CLI uses the same guarded API for enable/disable and reviewed decisions.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let at = revision(&f).to_string();
    for args in [
        vec![
            "--if-revision",
            &at,
            "--idempotency-key",
            "direct-enable-denied",
            "client",
            "enable",
            "app",
        ],
        vec![
            "--if-revision",
            &at,
            "--idempotency-key",
            "direct-update-denied",
            "client",
            "update",
            "app",
            "--enabled",
            "true",
        ],
    ] {
        let denied = cli(&binary, &f, &server_url, &f.admin, &args).await;
        assert_eq!(denied.1["error"]["http_status"], 409, "{denied:?}");
    }
    let file = f._dir.path().join("status.json");
    std::fs::write(&file, r#"{"enabled":true}"#).unwrap();
    let (exit, result) = cli(
        &binary,
        &f,
        &server_url,
        &f.admin,
        &[
            "client",
            "status-review",
            "stage",
            "app",
            "--file",
            file.to_str().unwrap(),
        ],
    )
    .await;
    assert_eq!(exit, 0, "{result}");
    let change = &result["data"];
    assert_eq!(change["proposal"]["effects"]["revoke_families"], 0);
    for (action, actor) in [("approve", &reviewer), ("execute", &executor)] {
        let result = cli(
            &binary,
            &f,
            &server_url,
            actor,
            &[
                "client",
                "status-review",
                action,
                id(change),
                "--digest",
                change["digest"].as_str().unwrap(),
            ],
        )
        .await;
        assert_eq!(result.0, 0, "{result:?}");
    }
    server.abort();
    assert!(client(&f).enabled);
    assert_eq!(
        client(&f).secret_hash.as_deref(),
        expected["secret_hash"].as_str()
    );
    for token in [&tokens, &more_tokens] {
        assert!(
            f.core
                .userinfo(token["access_token"].as_str().unwrap())
                .is_err()
        );
    }
    assert!(
        f.core.token(pending).is_err(),
        "Removed authorization code cannot return after enable"
    );
    assert_eq!(
        f.core
            .store
            .list::<Value>("logout_deliveries")
            .unwrap()
            .len(),
        1
    );
    assert!(f.core.me(&alice).is_ok());
}

#[test]
fn status_review_rechecks_exact_content_live_authority_expiry_and_private_dependencies() {
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    f.client("app", true);
    let change = f
        .core
        .stage_client_status(&f.admin, "app", input())
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .approve_client_status_change(&f.admin, id(&change), binding(&change))
        },
        403,
    );
    refused(
        &f,
        || {
            f.core
                .execute_client_status_change(&executor, id(&change), binding(&change))
        },
        403,
    );
    refused(
        &f,
        || {
            f.core.approve_client_status_change(
                &reviewer,
                id(&change),
                ClientStatusBinding {
                    digest: "other".into(),
                },
            )
        },
        409,
    );
    let change = f
        .core
        .approve_client_status_change(&reviewer, id(&change), binding(&change))
        .unwrap();
    for actor in [&f.admin, &reviewer] {
        refused(
            &f,
            || {
                f.core
                    .execute_client_status_change(actor, id(&change), binding(&change))
            },
            403,
        );
    }
    let stored: Value = f
        .core
        .store
        .get("reviewed_client_statuses", id(&change))
        .unwrap()
        .unwrap();
    for field in ["after", "effects"] {
        let mut changed = stored.clone();
        if field == "after" {
            changed["proposal"]["after"]["enabled"] = json!(true);
        } else {
            changed["proposal"]["effects"]["revoke_families"] = json!(999);
        }
        f.core
            .store
            .write(|tx| tx.put("reviewed_client_statuses", id(&change), &changed))
            .unwrap();
        refused(
            &f,
            || {
                f.core
                    .execute_client_status_change(&executor, id(&change), binding(&change))
            },
            409,
        );
    }
    f.core
        .store
        .write(|tx| tx.put("reviewed_client_statuses", id(&change), &stored))
        .unwrap();
    let baseline = client(&f);
    let mut drifted = baseline.clone();
    drifted.settings.backchannel_logout_uri = Some("https://other.example.test/logout".into());
    let at = revision(&f);
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &drifted))
        .unwrap();
    assert_eq!(revision(&f), at);
    refused(
        &f,
        || {
            f.core
                .execute_client_status_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &baseline))
        .unwrap();
    let mut config = f.core.clone();
    config
        .config
        .capabilities
        .disabled
        .insert("oidc.userinfo".into());
    refused(
        &f,
        || config.execute_client_status_change(&executor, id(&change), binding(&change)),
        409,
    );
    #[cfg(feature = "test-support")]
    riauth::crypto::with_test_time(change["proposal"]["expires_at"].as_u64().unwrap(), || {
        refused(
            &f,
            || {
                f.core
                    .execute_client_status_change(&executor, id(&change), binding(&change))
            },
            409,
        );
    });
    for admin in [false, true] {
        f.core
            .update_user(
                &f.admin,
                "reviewer",
                UserPatch {
                    admin: Some(admin),
                    ..Default::default()
                },
            )
            .unwrap();
        refused(
            &f,
            || {
                f.core
                    .execute_client_status_change(&executor, id(&change), binding(&change))
            },
            if admin { 409 } else { 403 },
        );
    }
    let reviewer = login(&f, "reviewer");
    f.core
        .cancel_client_status_change(&f.admin, id(&change), binding(&change))
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_status_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    let change = approved(&f, &reviewer);
    f.core.rotate_client_secret(&f.admin, "app").unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_status_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    // The bounded slice refuses unexpectedly large dependency buckets, atomically.
    f.core
        .store
        .write(|tx| {
            for n in 0..4097 {
                let sid = format!("bounded-{n:04}");
                tx.put(
                    "rp_sessions",
                    &sid,
                    &riauth::logout::RpSession {
                        sid: sid.clone(),
                        session_id: "unused".into(),
                        user_id: "unused".into(),
                        subject: "unused".into(),
                        client_id: "other-app".into(),
                        created_at: 0,
                        expires_at: 0,
                        ended: true,
                    },
                )?;
            }
            Ok(())
        })
        .unwrap();
    refused(
        &f,
        || f.core.stage_client_status(&f.admin, "app", input()),
        409,
    );
}

#[tokio::test]
async fn browser_status_routes_preserve_csrf_typed_content_and_exact_receipts() {
    let f = Fixture::new();
    let reviewer = administrator(&f, "browser-reviewer");
    let executor = administrator(&f, "browser-executor");
    f.client("app", true);
    let cookies = [&f.admin, &reviewer, &executor].map(|token| cookie(&f.core, token));
    let auth = cookies
        .each_ref()
        .map(|cookie| Auth::Browser(cookie, Some("http://localhost:9000")));
    let app = riauth::api::router(f.core.clone());
    let asset = "/portal/assets/client-status-review.js";
    assert!(
        call(&app, "GET", asset, auth[0], Value::Null, None, None)
            .await
            .1
            .as_str()
            .unwrap()
            .contains("RiAuthClientStatusReview")
    );
    assert!(
        call(&app, "GET", "/admin", auth[0], Value::Null, None, None)
            .await
            .1
            .as_str()
            .unwrap()
            .contains(asset)
    );
    let mut headless = f.core.clone();
    headless.config.browser_ui = false;
    assert_eq!(
        call(
            &riauth::api::router(headless),
            "GET",
            asset,
            auth[0],
            Value::Null,
            None,
            None
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    let stage = "/api/admin/clients/app/status-changes";
    for body in [
        json!({}),
        json!({"enabled":false, "client_secret":"smuggled"}),
    ] {
        assert_eq!(
            call(&app, "POST", stage, auth[0], body, None, None).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let at = revision(&f);
    let staged = call(
        &app,
        "POST",
        stage,
        auth[0],
        json!(input()),
        Some(at),
        Some("browser-stage"),
    )
    .await;
    assert_eq!(staged.0, StatusCode::OK, "{}", staged.1);
    assert_eq!(
        call(
            &app,
            "POST",
            stage,
            auth[0],
            json!(input()),
            Some(at),
            Some("browser-stage")
        )
        .await,
        staged
    );
    let change = &staged.1;
    let endpoint = format!("/api/admin/client-status-changes/{}", id(change));
    assert_eq!(
        call(&app, "GET", &endpoint, auth[1], Value::Null, None, None)
            .await
            .1,
        *change
    );
    for path in [
        stage.to_owned(),
        format!("{endpoint}/approve"),
        format!("{endpoint}/execute"),
        format!("{endpoint}/cancel"),
    ] {
        let body = if path == stage {
            json!(input())
        } else {
            json!(binding(change))
        };
        for origin in [None, Some("http://attacker.example")] {
            let before = f.snapshot().unwrap();
            assert_eq!(
                call(
                    &app,
                    "POST",
                    &path,
                    Auth::Browser(&cookies[1], origin),
                    body.clone(),
                    Some(at),
                    None
                )
                .await
                .0,
                StatusCode::FORBIDDEN
            );
            f.assert_http_mutation_snapshot(&before);
        }
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&path)
                    .header("host", "localhost:9000")
                    .header("origin", "http://localhost:9000")
                    .header("cookie", format!("riauth_sso={}", cookies[1]))
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "Missing portal header: {path}"
        );
    }
    for (action, actor, status) in [
        ("approve", auth[1], "approved"),
        ("execute", auth[2], "executed"),
    ] {
        let path = format!("{endpoint}/{action}");
        let response = call(
            &app,
            "POST",
            &path,
            actor,
            json!(binding(change)),
            Some(at),
            Some(action),
        )
        .await;
        assert_eq!(response.0, StatusCode::OK, "{}", response.1);
        assert_eq!(response.1["status"], status);
        assert_eq!(response.1["proposal"], change["proposal"]);
        assert_eq!(
            call(
                &app,
                "POST",
                &path,
                actor,
                json!(binding(change)),
                Some(at),
                Some(action)
            )
            .await,
            response
        );
        assert!(response.1.get("client_secret").is_none());
    }
}
