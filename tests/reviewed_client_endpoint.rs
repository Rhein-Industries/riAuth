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
        Client, ClientEndpointBinding, ClientEndpointInput, ClientPatch, Family, Group, NewUser,
        ProviderSettings, UserPatch,
    },
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
fn binding(change: &Value) -> ClientEndpointBinding {
    ClientEndpointBinding {
        digest: change["digest"].as_str().unwrap().into(),
    }
}
fn input() -> ClientEndpointInput {
    ClientEndpointInput {
        redirect_uris: vec![
            "https://app.example.test/callback?tenant=one".into(),
            "https://backup.example.test/cb".into(),
        ],
        origins: ["https://app.example.test".into()].into(),
    }
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
        .stage_client_endpoint(&f.admin, "app", input())
        .unwrap();
    f.core
        .approve_client_endpoint_change(reviewer, id(&change), binding(&change))
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
    let session = f._dir.path().join("client-endpoint-session.json");
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
async fn endpoints_bind_exact_content_block_all_writers_and_preserve_credentials_and_grants() {
    let built = std::path::Path::new(env!("CARGO_BIN_EXE_riauth"));
    let pinned = tempfile::Builder::new()
        .prefix(".reviewed-endpoint-")
        .tempdir_in(built.parent().unwrap())
        .unwrap();
    let binary = pinned.path().join("riauth");
    std::fs::hard_link(built, &binary).unwrap();
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    let alice = f.user("alice");
    let owner = f.user("owner");
    let secret = f.client("app", true);
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
    let scoped = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "endpoint-operator".into(),
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
    let cookies = [&f.admin, &reviewer, &executor].map(|token| cookie(&f.core, token));
    let auth = cookies
        .each_ref()
        .map(|c| Auth::Browser(c, Some("http://localhost:9000")));
    let app = riauth::api::router(f.core.clone());
    let mut settings = client(&f).settings;
    settings.origins = input().origins;
    for patch in [
        json!({"redirect_uris":input().redirect_uris}),
        json!({"settings":settings}),
    ] {
        for actor in [
            Auth::Bearer(&f.admin),
            Auth::Bearer(&scoped),
            Auth::Bearer(&owner),
            auth[0],
        ] {
            let path = if matches!(actor, Auth::Browser(..)) {
                "/api/admin/clients/app"
            } else {
                "/api/clients/app"
            };
            let before = f.snapshot().unwrap();
            let response = call(
                &app,
                "PATCH",
                path,
                actor,
                patch.clone(),
                Some(revision(&f)),
                None,
            )
            .await;
            assert_eq!(response.0, StatusCode::CONFLICT, "{}", response.1);
            f.assert_http_mutation_snapshot(&before);
        }
        refused(
            &f,
            || {
                f.core
                    .update_client(&f.admin, "app", serde_json::from_value(patch).unwrap())
            },
            409,
        );
    }
    for token in [&owner, &scoped] {
        refused(
            &f,
            || f.core.stage_client_endpoint(token, "app", input()),
            403,
        );
    }
    let mut manifest: Manifest =
        serde_json::from_value(f.core.export_state(&f.admin).unwrap()["manifest"].clone()).unwrap();
    manifest.users.clear();
    manifest.groups.clear();
    manifest.clients[0].redirect_uris = input().redirect_uris;
    refused(&f, || f.core.plan_state(&f.admin, manifest.clone()), 409);
    manifest.clients[0].redirect_uris = client(&f).redirect_uris;
    manifest.clients[0].settings.origins = input().origins;
    refused(&f, || f.core.plan_state(&f.admin, manifest.clone()), 409);
    manifest.clients[0].settings.origins.clear();
    manifest.clients[0].name = "Planned ordinary edit".into();
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    let baseline = client(&f);
    let mut drifted = baseline.clone();
    drifted.settings.origins = input().origins;
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
    f.core
        .update_client(
            &owner,
            "app",
            ClientPatch {
                name: Some("Ordinary owner edit".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let tokens = f.tokens("app", &alice, secret.clone());
    let _pending = f.exchange_request("app", &alice, secret.clone());
    let at = revision(&f);
    let stage = "/api/admin/clients/app/endpoint-changes";
    for body in [
        json!({"redirect_uris":[]}),
        json!({"redirect_uris":[], "origins":[], "enabled":false}),
    ] {
        assert_eq!(
            call(&app, "POST", stage, auth[0], body, None, None).await.0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let staged = call(
        &app,
        "POST",
        stage,
        auth[0],
        json!(input()),
        Some(at),
        Some("endpoint-stage"),
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
            Some("endpoint-stage")
        )
        .await,
        staged
    );
    let change = &staged.1;
    assert_eq!(change["proposal"]["after"], json!(input()));
    assert_eq!(change["proposal"]["effects"]["revoke_families"], 0);
    let endpoint = format!("/api/admin/client-endpoint-changes/{}", id(change));
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
        for origin in [None, Some("http://attacker.example")] {
            let body = if path == stage {
                json!(input())
            } else {
                json!(binding(change))
            };
            assert_eq!(
                call(
                    &app,
                    "POST",
                    &path,
                    Auth::Browser(&cookies[1], origin),
                    body,
                    Some(at),
                    None
                )
                .await
                .0,
                StatusCode::FORBIDDEN
            );
        }
    }
    refused(
        &f,
        || {
            f.core
                .approve_client_endpoint_change(&f.admin, id(change), binding(change))
        },
        403,
    );
    refused(
        &f,
        || {
            f.core
                .execute_client_endpoint_change(&executor, id(change), binding(change))
        },
        403,
    );
    refused(
        &f,
        || {
            f.core.approve_client_endpoint_change(
                &reviewer,
                id(change),
                ClientEndpointBinding {
                    digest: "substituted".into(),
                },
            )
        },
        409,
    );
    f.core
        .approve_client_endpoint_change(&reviewer, id(change), binding(change))
        .unwrap();
    for actor in [&f.admin, &reviewer] {
        refused(
            &f,
            || {
                f.core
                    .execute_client_endpoint_change(actor, id(change), binding(change))
            },
            403,
        );
    }
    // Enabled-client edits retain protocol state even if tokens are issued after approval.
    f.tokens("app", &alice, secret.clone());
    assert_eq!(revision(&f), at);
    let protocol = [
        "access",
        "refresh",
        "families",
        "codes",
        "devices",
        "rp_sessions",
        "logout_deliveries",
    ]
    .map(|bucket| (bucket, f.core.store.list::<Value>(bucket).unwrap()));
    let baseline = serde_json::to_value(client(&f)).unwrap();
    let execute = format!("{endpoint}/execute");
    let result = call(
        &app,
        "POST",
        &execute,
        auth[2],
        json!(binding(change)),
        Some(at),
        Some("endpoint-execute"),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK, "{}", result.1);
    assert_eq!(result.1["proposal"], change["proposal"]);
    let mut expected = baseline;
    expected["redirect_uris"] = json!(input().redirect_uris);
    expected["settings"]["origins"] = json!(input().origins);
    assert_eq!(
        serde_json::to_value(client(&f)).unwrap(),
        expected,
        "Only the approved fields may change"
    );
    assert_eq!(revision(&f), at + 1);
    for (bucket, before) in protocol {
        assert_eq!(
            f.core.store.list::<Value>(bucket).unwrap(),
            before,
            "{bucket}"
        );
    }
    assert!(
        f.core
            .userinfo(tokens["access_token"].as_str().unwrap())
            .is_ok()
    );
    let snapshot = f.snapshot().unwrap();
    assert_eq!(
        call(
            &app,
            "POST",
            &execute,
            auth[2],
            json!(binding(change)),
            Some(at),
            Some("endpoint-execute")
        )
        .await,
        result
    );
    assert_eq!(
        call(
            &app,
            "POST",
            &execute,
            auth[2],
            json!(binding(change)),
            None,
            Some("endpoint-replay")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&snapshot);
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["action"] == "reviewed_client_endpoints.execute"
                && e["details"]["change_id"] == id(change))
            .count(),
        1
    );
    for sensitive in [
        secret.as_deref().unwrap(),
        expected["secret_hash"].as_str().unwrap(),
    ] {
        assert!(!format!("{events}{change}{}", result.1).contains(sensitive));
    }
    let asset = "/portal/assets/client-endpoint-review.js";
    assert!(
        call(&app, "GET", asset, auth[0], Value::Null, None, None)
            .await
            .1
            .as_str()
            .unwrap()
            .contains("RiAuthClientEndpointReview")
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let denied = cli(
        &binary,
        &f,
        &url,
        &f.admin,
        &[
            "client",
            "update",
            "app",
            "--redirect-uri",
            "https://unreviewed.example/cb",
        ],
    )
    .await;
    assert_eq!(denied.1["error"]["http_status"], 409, "{denied:?}");
    let file = f._dir.path().join("endpoints.json");
    std::fs::write(&file, r#"{"redirect_uris":[],"origins":[]}"#).unwrap();
    let (exit, result) = cli(
        &binary,
        &f,
        &url,
        &f.admin,
        &[
            "client",
            "endpoint-review",
            "stage",
            "app",
            "--file",
            file.to_str().unwrap(),
        ],
    )
    .await;
    assert_eq!(exit, 0, "{result}");
    let change = &result["data"];
    for (action, actor) in [("approve", &reviewer), ("execute", &executor)] {
        let result = cli(
            &binary,
            &f,
            &url,
            actor,
            &[
                "client",
                "endpoint-review",
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
    assert!(client(&f).redirect_uris.is_empty() && client(&f).settings.origins.is_empty());
    assert_eq!(
        client(&f).secret_hash.as_deref(),
        expected["secret_hash"].as_str()
    );
}

#[test]
fn endpoint_review_rejects_content_authority_revision_and_oversized_dependencies() {
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    f.client("app", true);
    for after in [
        ClientEndpointInput {
            redirect_uris: vec!["https://app.example.test/*".into()],
            origins: Default::default(),
        },
        ClientEndpointInput {
            redirect_uris: input().redirect_uris,
            origins: ["https://app.example.test/path".into()].into(),
        },
    ] {
        refused(
            &f,
            || f.core.stage_client_endpoint(&f.admin, "app", after),
            400,
        );
    }
    let change = approved(&f, &reviewer);
    let stored: Value = f
        .core
        .store
        .get("reviewed_client_endpoints", id(&change))
        .unwrap()
        .unwrap();
    for field in ["redirect_uris", "origins"] {
        let mut altered = stored.clone();
        altered["proposal"]["after"][field] = json!(["https://substituted.example"]);
        f.core
            .store
            .write(|tx| tx.put("reviewed_client_endpoints", id(&change), &altered))
            .unwrap();
        refused(
            &f,
            || {
                f.core
                    .execute_client_endpoint_change(&executor, id(&change), binding(&change))
            },
            409,
        );
    }
    f.core
        .store
        .write(|tx| tx.put("reviewed_client_endpoints", id(&change), &stored))
        .unwrap();
    let baseline = client(&f);
    let at = revision(&f);
    let mut drifted = baseline.clone();
    drifted.name = "Changed without revision".into();
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &drifted))
        .unwrap();
    assert_eq!(revision(&f), at);
    refused(
        &f,
        || {
            f.core
                .execute_client_endpoint_change(&executor, id(&change), binding(&change))
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
        || config.execute_client_endpoint_change(&executor, id(&change), binding(&change)),
        409,
    );
    #[cfg(feature = "test-support")]
    riauth::crypto::with_test_time(change["proposal"]["expires_at"].as_u64().unwrap(), || {
        refused(
            &f,
            || {
                f.core
                    .execute_client_endpoint_change(&executor, id(&change), binding(&change))
            },
            409,
        )
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
                    .execute_client_endpoint_change(&executor, id(&change), binding(&change))
            },
            if admin { 409 } else { 403 },
        );
    }
    let reviewer = login(&f, "reviewer");
    f.core
        .cancel_client_endpoint_change(&f.admin, id(&change), binding(&change))
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_endpoint_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    let change = approved(&f, &reviewer);
    f.core.rotate_client_secret(&f.admin, "app").unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_endpoint_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    let change = approved(&f, &reviewer);
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                name: Some("Ordinary change".into()),
                ..Default::default()
            },
        )
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_endpoint_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    let oversized = ClientEndpointInput {
        origins: (0..65)
            .map(|n| format!("https://site{n}.example.test"))
            .collect(),
        ..input()
    };
    refused(
        &f,
        || f.core.stage_client_endpoint(&f.admin, "app", oversized),
        409,
    );
    let mut client = client(&f);
    client.allowed_groups.insert("huge".into());
    f.core
        .store
        .write(|tx| {
            tx.put("clients", "app", &client)?;
            tx.put(
                "groups",
                "huge",
                &Group {
                    name: "huge".into(),
                    members: (0..4097).map(|n| format!("member{n}")).collect(),
                },
            )
        })
        .unwrap();
    refused(
        &f,
        || f.core.stage_client_endpoint(&f.admin, "app", input()),
        409,
    );
}

#[test]
fn disabled_client_endpoint_edits_bind_and_preserve_existing_revocation_semantics() {
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    let secret = f.client("app", true);
    let alice = f.user("alice");
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    backchannel_logout_uri: Some("https://app.example.test/logout".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    f.tokens("app", &alice, secret);
    let _pending = f.exchange_request("app", &alice, None);
    // Historical disabled-client state, including still-present grants, must keep
    // the shared writer's cleanup. Only this fixture setup bypasses management.
    let mut baseline = client(&f);
    baseline.enabled = false;
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &baseline))
        .unwrap();
    let change = approved(&f, &reviewer);
    assert_eq!(change["proposal"]["client_enabled"], false);
    assert_eq!(change["proposal"]["effects"]["revoke_families"], 1);
    assert_eq!(
        change["proposal"]["effects"]["queue_backchannel_logouts"],
        1
    );
    let (key, mut rp) = f
        .core
        .store
        .list::<riauth::logout::RpSession>("rp_sessions")
        .unwrap()
        .pop()
        .unwrap();
    rp.expires_at += 1;
    f.core
        .store
        .write(|tx| tx.put("rp_sessions", &key, &rp))
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_endpoint_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    let change = approved(&f, &reviewer);
    f.core
        .execute_client_endpoint_change(&executor, id(&change), binding(&change))
        .unwrap();
    baseline.redirect_uris = input().redirect_uris;
    baseline.settings.origins = input().origins;
    assert_eq!(
        serde_json::to_value(client(&f)).unwrap(),
        serde_json::to_value(baseline).unwrap()
    );
    assert!(
        f.core
            .store
            .list::<Family>("families")
            .unwrap()
            .iter()
            .all(|(_, f)| f.revoked)
    );
    assert!(f.core.store.list::<Value>("codes").unwrap().is_empty());
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
