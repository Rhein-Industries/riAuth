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
        post_logout_redirect_uris: vec![
            "https://app.example.test/signed-out?tenant=one".into(),
            "http://127.0.0.1:7777/signed-out".into(),
        ],
        frontchannel_logout_uri: Some("https://app.example.test/new-front?tenant=one".into()),
        backchannel_logout_uri: Some("https://app.example.test/new-back?tenant=one".into()),
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
    let mut logout_settings = client(&f).settings;
    logout_settings.post_logout_redirect_uris = input().post_logout_redirect_uris;
    let mut front_settings = client(&f).settings;
    front_settings.frontchannel_logout_uri = input().frontchannel_logout_uri;
    let mut back_settings = client(&f).settings;
    back_settings.backchannel_logout_uri = input().backchannel_logout_uri;
    for patch in [
        json!({"redirect_uris":input().redirect_uris}),
        json!({"settings":settings}),
        json!({"settings":logout_settings}),
        json!({"settings":front_settings}),
        json!({"settings":back_settings}),
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
                Some("direct-endpoint-denied"),
            )
            .await;
            let denied_owner_channel = matches!(actor, Auth::Bearer(token) if token == owner)
                && (patch["settings"]["frontchannel_logout_uri"].is_string()
                    || patch["settings"]["backchannel_logout_uri"].is_string());
            assert_eq!(
                response.0,
                if denied_owner_channel {
                    StatusCode::FORBIDDEN
                } else {
                    StatusCode::CONFLICT
                },
                "{}",
                response.1
            );
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
    manifest.clients[0].settings.post_logout_redirect_uris = input().post_logout_redirect_uris;
    refused(&f, || f.core.plan_state(&f.admin, manifest.clone()), 409);
    manifest.clients[0]
        .settings
        .post_logout_redirect_uris
        .clear();
    manifest.clients[0].settings.frontchannel_logout_uri = input().frontchannel_logout_uri;
    refused(&f, || f.core.plan_state(&f.admin, manifest.clone()), 409);
    manifest.clients[0].settings.frontchannel_logout_uri = None;
    manifest.clients[0].settings.backchannel_logout_uri = input().backchannel_logout_uri;
    refused(&f, || f.core.plan_state(&f.admin, manifest.clone()), 409);
    manifest.clients[0].settings.backchannel_logout_uri = None;
    manifest.clients[0].name = "Planned ordinary edit".into();
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    let baseline = client(&f);
    let mut drifted = baseline.clone();
    drifted.settings.backchannel_logout_uri = input().backchannel_logout_uri;
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
        json!({"redirect_uris":[], "origins":[]}),
        json!({"redirect_uris":[], "origins":[], "post_logout_redirect_uris":[]}),
        json!({"redirect_uris":[], "origins":[], "post_logout_redirect_uris":[], "frontchannel_logout_uri":null}),
        json!({"redirect_uris":[], "origins":[], "post_logout_redirect_uris":[], "backchannel_logout_uri":null}),
        json!({"redirect_uris":[], "origins":[], "post_logout_redirect_uris":[], "frontchannel_logout_uri":null, "backchannel_logout_uri":null, "enabled":false}),
        json!({"redirect_uris":[], "origins":[], "post_logout_redirect_uris":[], "frontchannel_logout_uri":false, "backchannel_logout_uri":null}),
        json!({"redirect_uris":[], "origins":[], "post_logout_redirect_uris":[], "frontchannel_logout_uri":null, "backchannel_logout_uri":[], "enabled":false}),
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
    expected["settings"]["post_logout_redirect_uris"] = json!(input().post_logout_redirect_uris);
    expected["settings"]["frontchannel_logout_uri"] = json!(input().frontchannel_logout_uri);
    expected["settings"]["backchannel_logout_uri"] = json!(input().backchannel_logout_uri);
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
    let at = revision(&f).to_string();
    let denied = cli(
        &binary,
        &f,
        &url,
        &f.admin,
        &[
            "--if-revision",
            &at,
            "--idempotency-key",
            "direct-redirect-denied",
            "client",
            "update",
            "app",
            "--redirect-uri",
            "https://unreviewed.example/cb",
        ],
    )
    .await;
    assert_eq!(denied.1["error"]["http_status"], 409, "{denied:?}");
    let file = f._dir.path().join("settings.json");
    for field in [
        "post_logout_redirect_uris",
        "frontchannel_logout_uri",
        "backchannel_logout_uri",
    ] {
        let mut direct = serde_json::to_value(client(&f).settings).unwrap();
        direct[field] = if field.ends_with("_uris") {
            json!([])
        } else {
            Value::Null
        };
        std::fs::write(&file, serde_json::to_vec(&direct).unwrap()).unwrap();
        let denied = cli(
            &binary,
            &f,
            &url,
            &f.admin,
            &[
                "--if-revision",
                &at,
                "--idempotency-key",
                "direct-logout-denied",
                "client",
                "update",
                "app",
                "--settings-file",
                file.to_str().unwrap(),
            ],
        )
        .await;
        assert_eq!(denied.1["error"]["http_status"], 409, "{denied:?}");
    }
    let file = f._dir.path().join("endpoints.json");
    std::fs::write(
        &file,
        r#"{"redirect_uris":[],"origins":[],"post_logout_redirect_uris":[],"frontchannel_logout_uri":null,"backchannel_logout_uri":null}"#,
    )
    .unwrap();
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
    assert!(client(&f).settings.post_logout_redirect_uris.is_empty());
    assert!(client(&f).settings.frontchannel_logout_uri.is_none());
    assert!(client(&f).settings.backchannel_logout_uri.is_none());
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
            ..input()
        },
        ClientEndpointInput {
            origins: ["https://app.example.test/path".into()].into(),
            ..input()
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
    for field in [
        "redirect_uris",
        "origins",
        "post_logout_redirect_uris",
        "frontchannel_logout_uri",
        "backchannel_logout_uri",
    ] {
        let mut altered = stored.clone();
        altered["proposal"]["after"][field] = if field.ends_with("_uri") {
            json!("https://substituted.example")
        } else {
            json!(["https://substituted.example"])
        };
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
    drifted.settings.frontchannel_logout_uri = Some("https://changed.example/front".into());
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
    for oversized in [
        ClientEndpointInput {
            origins: (0..65)
                .map(|n| format!("https://site{n}.example.test"))
                .collect(),
            ..input()
        },
        ClientEndpointInput {
            post_logout_redirect_uris: (0..33)
                .map(|n| format!("https://app.example.test/logout/{n}"))
                .collect(),
            ..input()
        },
        ClientEndpointInput {
            post_logout_redirect_uris: vec![format!(
                "https://app.example.test/{}",
                "a".repeat(2048)
            )],
            ..input()
        },
    ] {
        refused(
            &f,
            || f.core.stage_client_endpoint(&f.admin, "app", oversized),
            409,
        );
    }
    for after in [
        ClientEndpointInput {
            frontchannel_logout_uri: Some(format!("https://app.example.test/{}", "a".repeat(2048))),
            ..input()
        },
        ClientEndpointInput {
            backchannel_logout_uri: Some(format!("https://app.example.test/{}", "a".repeat(2048))),
            ..input()
        },
    ] {
        refused(
            &f,
            || f.core.stage_client_endpoint(&f.admin, "app", after),
            409,
        );
    }
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
fn post_logout_only_review_retires_legacy_approvals_and_preserves_callback_and_logout_rules() {
    const OLD: &str = "https://app.example.test/old-signed-out";
    const NEW: &str = "https://app.example.test/signed-out?tenant=one";
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    let alice = f.user("alice");
    let secret = f.client_with_settings(
        "app",
        true,
        ProviderSettings {
            origins: ["https://app.example.test".into()].into(),
            post_logout_redirect_uris: vec![OLD.into()],
            backchannel_logout_uri: Some("https://app.example.test/backchannel".into()),
            frontchannel_logout_uri: Some("https://app.example.test/frontchannel".into()),
            ..Default::default()
        },
    );
    let original = client(&f);
    let after = ClientEndpointInput {
        redirect_uris: original.redirect_uris.clone(),
        origins: original.settings.origins.clone(),
        post_logout_redirect_uris: vec![NEW.into(), "http://127.0.0.1:7777/signed-out".into()],
        frontchannel_logout_uri: original.settings.frontchannel_logout_uri.clone(),
        backchannel_logout_uri: original.settings.backchannel_logout_uri.clone(),
    };
    for uri in [
        "http://public.example/signed-out",
        "https://app.example.test/*",
        "https://app.example.test/signed-out#fragment",
        "https://user:pass@app.example.test/",
        "https://app.example.test/?%73tate=reserved",
        "https://app.example.test/?iss=reserved",
        "https://app.example.test/?sid=reserved",
        "com.example.app:/signed-out",
    ] {
        refused(
            &f,
            || {
                f.core.stage_client_endpoint(
                    &f.admin,
                    "app",
                    ClientEndpointInput {
                        post_logout_redirect_uris: vec![uri.into()],
                        ..after.clone()
                    },
                )
            },
            400,
        );
    }
    let staged = f
        .core
        .stage_client_endpoint(&f.admin, "app", after.clone())
        .unwrap();
    let current = f
        .core
        .approve_client_endpoint_change(&reviewer, id(&staged), binding(&staged))
        .unwrap();
    let mut legacy = current.clone();
    for version in [1, 2] {
        legacy = current.clone();
        for field in ["before", "after"] {
            let fields = legacy["proposal"][field].as_object_mut().unwrap();
            fields.remove("frontchannel_logout_uri");
            fields.remove("backchannel_logout_uri");
            if version == 1 {
                fields.remove("post_logout_redirect_uris");
            }
        }
        let mut proposal = legacy["proposal"].clone();
        proposal.sort_all_objects();
        let legacy_digest = riauth::crypto::digest(&format!(
            "riauth/reviewed-client-endpoint/v{version}\n{proposal}"
        ));
        legacy["digest"] = json!(legacy_digest);
        legacy["approvals"][0]["digest"] = json!(legacy_digest);
        f.core
            .store
            .write(|tx| tx.put("reviewed_client_endpoints", id(&legacy), &legacy))
            .unwrap();
        refused(
            &f,
            || f.core.client_endpoint_change(&f.admin, id(&legacy)),
            409,
        );
        refused(
            &f,
            || {
                f.core
                    .execute_client_endpoint_change(&executor, id(&legacy), binding(&legacy))
            },
            409,
        );
    }
    // A legacy record must not make the shared staging scan fail. All five
    // fields must be freshly reviewed; neither old approvals nor omitted fields carry over.
    let change = f
        .core
        .stage_client_endpoint(&f.admin, "app", after.clone())
        .unwrap();
    assert_eq!(
        change["proposal"]["before"]["post_logout_redirect_uris"],
        json!([OLD])
    );
    assert_eq!(change["proposal"]["after"], json!(after));
    assert!(change["approvals"].as_array().unwrap().is_empty());
    let mut proposal = change["proposal"].clone();
    proposal.sort_all_objects();
    assert_eq!(
        change["digest"],
        riauth::crypto::digest(&format!("riauth/reviewed-client-endpoint/v3\n{proposal}"))
    );
    f.core
        .approve_client_endpoint_change(&reviewer, id(&change), binding(&change))
        .unwrap();
    // Issuance on the unchanged callback remains live; endpoint review itself
    // must not revoke the resulting tokens or normalize credentials/settings.
    let tokens = f.tokens("app", &alice, secret.clone());
    let _pending = f.exchange_request("app", &alice, secret.clone());
    let protocol = [
        "access",
        "refresh",
        "families",
        "codes",
        "rp_sessions",
        "logout_deliveries",
    ]
    .map(|bucket| (bucket, f.core.store.list::<Value>(bucket).unwrap()));
    f.core
        .execute_client_endpoint_change(&executor, id(&change), binding(&change))
        .unwrap();
    let mut expected = serde_json::to_value(&original).unwrap();
    expected["settings"]["post_logout_redirect_uris"] = json!(after.post_logout_redirect_uris);
    assert_eq!(serde_json::to_value(client(&f)).unwrap(), expected);
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
    f.tokens("app", &alice, secret);
    refused(
        &f,
        || {
            f.core
                .execute_client_endpoint_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    let logout = |uri: &str| riauth::logout::LogoutRequest {
        client_id: Some("app".into()),
        id_token_hint: Some(tokens["id_token"].as_str().unwrap().into()),
        post_logout_redirect_uri: Some(uri.into()),
        state: Some("returned".into()),
    };
    for uri in [
        OLD,
        "https://app.example.test/signed-out?tenant=two",
        "https://app.example.test/signed-out",
    ] {
        refused(
            &f,
            || f.core.end_session(logout(uri), None, Some(&alice)),
            400,
        );
    }
    let result = f.core.end_session(logout(NEW), None, Some(&alice)).unwrap();
    assert_eq!(result["logged_out"], true);
    assert_eq!(result["redirect_uri"], format!("{NEW}&state=returned"));
    // Expiry cleanup also understands legacy envelopes and audits their original
    // original content without inventing approved empty/null logout fields.
    #[cfg(feature = "test-support")]
    riauth::crypto::with_test_time(
        legacy["proposal"]["expires_at"].as_u64().unwrap() + 1,
        || {
            f.core
                .stage_client_endpoint(
                    &f.admin,
                    "app",
                    ClientEndpointInput {
                        post_logout_redirect_uris: vec![OLD.into()],
                        ..after
                    },
                )
                .unwrap();
            assert!(
                f.core
                    .store
                    .get::<Value>("reviewed_client_endpoints", id(&legacy))
                    .unwrap()
                    .is_none()
            );
            let events = f.core.audit_events(&f.admin, 100).unwrap();
            let event = events
                .as_array()
                .unwrap()
                .iter()
                .find(|event| {
                    event["action"] == "reviewed_client_endpoints.expire"
                        && event["details"]["change_id"] == id(&legacy)
                })
                .unwrap();
            assert_eq!(event["details"]["after"], legacy["proposal"]["after"]);
        },
    );
}

#[test]
fn pending_logout_confirmations_recheck_reviewed_post_logout_removal() {
    use riauth::logout::LogoutRequest;
    // Serialization normalizes the host and port. A currently registered
    // equivalent URL must not stand in for the exact registration that was removed.
    const REMOVED: &str = "https://APP.example.test:443/signed-out?tenant=one%20two";
    const EQUIVALENT: &str = "https://app.example.test/signed-out?tenant=one%20two";
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    f.client_with_settings(
        "app",
        false,
        ProviderSettings {
            post_logout_redirect_uris: vec![REMOVED.into()],
            frontchannel_logout_uri: Some("https://app.example.test/front".into()),
            backchannel_logout_uri: Some("https://app.example.test/back".into()),
            ..Default::default()
        },
    );
    let cases = [
        ("terminal-approve", true, true, true, false),
        ("browser-approve", false, true, true, false),
        ("no-session", false, true, false, false),
        ("browser-deny", false, false, true, false),
        ("terminal-deny", true, false, true, false),
        ("already-ended", false, true, true, true),
    ]
    .map(|(name, terminal, approve, has_browser, ended)| {
        let token = f.user(name);
        let sso = cookie(&f.core, &token);
        f.tokens("app", &token, None);
        let pending = f
            .core
            .end_session(
                LogoutRequest {
                    client_id: Some("app".into()),
                    post_logout_redirect_uri: Some(REMOVED.into()),
                    state: Some("a + &?".into()),
                    ..Default::default()
                },
                has_browser.then_some(sso.as_str()),
                None,
            )
            .unwrap();
        assert_eq!(pending["interaction_required"], true);
        if ended {
            f.core.logout(&token).unwrap();
        }
        (token, sso, pending, terminal, approve, has_browser, ended)
    });
    replace_post_logout_redirects(&f, &reviewer, &executor, &[EQUIVALENT]);
    let mut expected_deliveries = 1; // The already-ended session was logged out above.
    for (token, sso, pending, terminal, approve, has_browser, ended) in cases {
        let (id, binding) = logout_handle(&pending);
        let code = pending["user_code"].as_str().unwrap();
        assert!(
            f.core.logout_state(id, Some(binding), None).unwrap()["application"]["host"].is_null()
        );
        if !ended {
            assert!(f.core.logout_request_details(&token, code).unwrap()["redirect_uri"].is_null());
        }
        if terminal {
            f.core.logout_request_decide(&token, code, approve).unwrap();
        } else {
            let reply = f
                .core
                .logout_browser_decide(
                    id,
                    Some(binding),
                    has_browser.then_some(sso.as_str()),
                    approve,
                )
                .unwrap();
            assert_eq!(reply.body["status"], "done");
            assert_eq!(!reply.cookies.is_empty(), approve && has_browser && !ended);
        }
        // Check the committed decision as well as delivery: a resume-only filter
        // must not conceal stale redirects being written by approval or settle.
        let stored: Value = f
            .core
            .store
            .get("logout_confirmations", id)
            .unwrap()
            .unwrap();
        assert!(stored["result"]["redirect_uri"].is_null());
        let result = f.core.logout_request_resume(id, Some(binding)).unwrap();
        assert_eq!(result, stored["result"]);
        assert_eq!(result["logged_out"], approve && has_browser);
        assert_eq!(f.core.me(&token).is_err(), approve && has_browser);
        if approve && has_browser && !ended {
            assert_eq!(result["frontchannel_urls"].as_array().unwrap().len(), 1);
            expected_deliveries += 1;
        } else {
            assert_eq!(result["frontchannel_urls"], json!([]));
        }
    }
    let deliveries = f
        .core
        .store
        .list::<riauth::logout::Delivery>("logout_deliveries")
        .unwrap();
    assert_eq!(deliveries.len(), expected_deliveries);
    assert!(
        deliveries
            .iter()
            .all(|(_, d)| d.uri == "https://app.example.test/back")
    );
}

fn logout_handle(pending: &Value) -> (&str, &str) {
    let id = pending["resume_uri"]
        .as_str()
        .unwrap()
        .rsplit('/')
        .next()
        .unwrap();
    let binding = pending["set_cookie"]
        .as_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    (id, binding)
}

fn replace_post_logout_redirects(f: &Fixture, reviewer: &str, executor: &str, uris: &[&str]) {
    let before = client(f);
    let change = f
        .core
        .stage_client_endpoint(
            &f.admin,
            "app",
            ClientEndpointInput {
                redirect_uris: before.redirect_uris,
                origins: before.settings.origins,
                post_logout_redirect_uris: uris.iter().map(|uri| (*uri).into()).collect(),
                frontchannel_logout_uri: before.settings.frontchannel_logout_uri,
                backchannel_logout_uri: before.settings.backchannel_logout_uri,
            },
        )
        .unwrap();
    f.core
        .approve_client_endpoint_change(reviewer, id(&change), binding(&change))
        .unwrap();
    f.core
        .execute_client_endpoint_change(executor, id(&change), binding(&change))
        .unwrap();
}

#[tokio::test]
async fn decided_logout_confirmations_recheck_removal_at_browser_resume() {
    use riauth::logout::LogoutRequest;
    const REMOVED: &str = "https://app.example.test/removed?tenant=one%20two";
    const KEPT: &str = "https://app.example.test/kept?tenant=one%20two";
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    f.client_with_settings(
        "app",
        false,
        ProviderSettings {
            post_logout_redirect_uris: vec![REMOVED.into(), KEPT.into()],
            ..Default::default()
        },
    );
    let cases = [
        ("removed", REMOVED, false),
        ("kept", KEPT, false),
        ("legacy", KEPT, true),
    ]
    .map(|(name, uri, legacy)| {
        let token = f.user(name);
        let sso = cookie(&f.core, &token);
        let pending = f
            .core
            .end_session(
                LogoutRequest {
                    client_id: Some("app".into()),
                    post_logout_redirect_uri: Some(uri.into()),
                    state: Some("a + &?".into()),
                    ..Default::default()
                },
                Some(&sso),
                None,
            )
            .unwrap();
        let (id, _) = logout_handle(&pending);
        f.core
            .logout_request_decide(&token, pending["user_code"].as_str().unwrap(), true)
            .unwrap();
        assert!(f.core.me(&token).is_err());
        let mut stored: Value = f
            .core
            .store
            .get("logout_confirmations", id)
            .unwrap()
            .unwrap();
        assert_eq!(
            stored["result"]["redirect_uri"],
            format!(
                "https://app.example.test/{}?tenant=one%20two&state=a+%2B+%26%3F",
                if uri == KEPT { "kept" } else { "removed" },
            )
        );
        if legacy {
            // A confirmation persisted by the previous build must fail closed,
            // even if normalization would let us guess a registered base URL.
            stored
                .as_object_mut()
                .unwrap()
                .remove("post_logout_redirect_uri");
            f.core
                .store
                .write(|tx| tx.put("logout_confirmations", id, &stored))
                .unwrap();
        }
        (pending, stored["result"].clone(), uri == KEPT && !legacy)
    });
    replace_post_logout_redirects(&f, &reviewer, &executor, &[KEPT]);
    let app = riauth::api::router(f.core.clone());
    for (pending, stored_result, allowed) in cases {
        let (id, binding) = logout_handle(&pending);
        assert!(
            f.core
                .logout_request_resume(id, Some("wrong-browser"))
                .is_err()
        );
        // Execution did not rewrite confirmations. Delivery must consult live
        // policy even for a previously approved, immutable logout result.
        let stored: Value = f
            .core
            .store
            .get("logout_confirmations", id)
            .unwrap()
            .unwrap();
        assert_eq!(stored["result"], stored_result);
        let result = f.core.logout_request_resume(id, Some(binding)).unwrap();
        assert_eq!(result["logged_out"], true);
        assert_eq!(result["redirect_uri"].is_string(), allowed);
        if allowed {
            assert_eq!(result, stored_result);
        }
        for accept in ["application/json", "text/html"] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(pending["resume_uri"].as_str().unwrap())
                        .header("host", "localhost:9000")
                        .header("accept", accept)
                        .header(
                            "cookie",
                            pending["set_cookie"]
                                .as_str()
                                .unwrap()
                                .split(';')
                                .next()
                                .unwrap(),
                        )
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            if allowed {
                assert!(response.status().is_redirection());
                assert_eq!(
                    response.headers()["location"],
                    stored_result["redirect_uri"].as_str().unwrap()
                );
            } else {
                assert_eq!(response.status(), StatusCode::OK);
                assert!(!response.headers().contains_key("location"));
                let bytes = response.into_body().collect().await.unwrap().to_bytes();
                let body = String::from_utf8(bytes.to_vec()).unwrap();
                assert!(!body.contains("app.example.test"));
            }
        }
    }
}

#[test]
fn logout_channel_reviews_preserve_live_sessions_and_queued_destinations() {
    use riauth::logout::{Delivery, LogoutRequest};
    const OLD_FRONT: &str = "https://rp.example.test/front-old";
    const OLD_BACK: &str = "https://rp.example.test/back-old";
    const NEW_FRONT: &str = "https://rp.example.test/front-new?tenant=one";
    const NEW_BACK: &str = "https://rp.example.test/back-new?tenant=one";
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    let queued = f.user("queued");
    let live = f.user("live");
    let removed = f.user("removed");
    let secret = f.client_with_settings(
        "app",
        true,
        ProviderSettings {
            frontchannel_logout_uri: Some(OLD_FRONT.into()),
            backchannel_logout_uri: Some(OLD_BACK.into()),
            ..Default::default()
        },
    );
    let baseline = client(&f);
    let after = ClientEndpointInput {
        redirect_uris: baseline.redirect_uris.clone(),
        origins: baseline.settings.origins.clone(),
        post_logout_redirect_uris: baseline.settings.post_logout_redirect_uris.clone(),
        frontchannel_logout_uri: Some(NEW_FRONT.into()),
        backchannel_logout_uri: Some(NEW_BACK.into()),
    };
    // Both fields retain the provider's exact HTTPS/loopback validation. Neither
    // empty strings, reserved parameters, fragments nor custom schemes mean removal.
    for uri in [
        "",
        "http://public.example/logout",
        "https://rp.example.test/*",
        "https://rp.example.test/#fragment",
        "https://user:pass@rp.example.test/",
        "https://rp.example.test/?%73id=reserved",
        "https://rp.example.test/?state=x",
        "app:/logout",
    ] {
        for front in [false, true] {
            let mut invalid = after.clone();
            if front {
                invalid.frontchannel_logout_uri = Some(uri.into());
            } else {
                invalid.backchannel_logout_uri = Some(uri.into());
            }
            refused(
                &f,
                || f.core.stage_client_endpoint(&f.admin, "app", invalid),
                400,
            );
        }
    }
    let tokens = [&queued, &live, &removed].map(|session| f.tokens("app", session, secret.clone()));
    let logout = |tokens: &Value| LogoutRequest {
        id_token_hint: Some(tokens["id_token"].as_str().unwrap().into()),
        client_id: Some("app".into()),
        ..Default::default()
    };
    let change = f
        .core
        .stage_client_endpoint(&f.admin, "app", after.clone())
        .unwrap();
    f.core
        .approve_client_endpoint_change(&reviewer, id(&change), binding(&change))
        .unwrap();
    // Logout activity after approval need not stale an enabled-client edit: the
    // writer does not touch any session/outbox row or rewrite queued destinations.
    f.core
        .end_session(logout(&tokens[0]), None, Some(&queued))
        .unwrap();
    assert_eq!(
        revision(&f),
        change["proposal"]["base_revision"].as_u64().unwrap()
    );
    let protocol = [
        "access",
        "refresh",
        "families",
        "rp_sessions",
        "sessions",
        "logout_deliveries",
    ]
    .map(|bucket| (bucket, f.core.store.list::<Value>(bucket).unwrap()));
    f.core
        .execute_client_endpoint_change(&executor, id(&change), binding(&change))
        .unwrap();
    for (bucket, before) in protocol {
        assert_eq!(
            f.core.store.list::<Value>(bucket).unwrap(),
            before,
            "{bucket}"
        );
    }
    let mut expected = serde_json::to_value(&baseline).unwrap();
    expected["settings"]["frontchannel_logout_uri"] = json!(NEW_FRONT);
    expected["settings"]["backchannel_logout_uri"] = json!(NEW_BACK);
    assert_eq!(serde_json::to_value(client(&f)).unwrap(), expected);
    assert!(f.core.me(&live).is_ok());
    assert!(
        f.core
            .userinfo(tokens[1]["access_token"].as_str().unwrap())
            .is_ok()
    );
    f.tokens("app", &live, secret);
    let claimed = f.core.claim_logout_deliveries().unwrap();
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].0.uri, OLD_BACK);
    f.core
        .finish_logout_delivery(&claimed[0].0.id, claimed[0].0.attempts, Some(204))
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_endpoint_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    // An existing RP session uses the reviewed current front channel and creates
    // a new back-channel job at the new URL; no new sign-in is required.
    let result = f
        .core
        .end_session(logout(&tokens[1]), None, Some(&live))
        .unwrap();
    assert_eq!(result["logged_out"], true);
    let fronts = result["frontchannel_urls"].as_array().unwrap();
    assert_eq!(fronts.len(), 1);
    let front = url::Url::parse(fronts[0].as_str().unwrap()).unwrap();
    assert_eq!(front.path(), "/front-new");
    let query: std::collections::BTreeMap<_, _> = front.query_pairs().collect();
    assert_eq!(query.get("tenant").unwrap(), "one");
    assert_eq!(query.get("iss").unwrap(), &f.core.config.issuer);
    assert!(query.get("sid").is_some_and(|sid| !sid.is_empty()));
    let deliveries = f.core.store.list::<Delivery>("logout_deliveries").unwrap();
    assert_eq!(deliveries.len(), 2);
    assert_eq!(
        deliveries.iter().filter(|(_, d)| d.uri == NEW_BACK).count(),
        1
    );
    let removal = f
        .core
        .stage_client_endpoint(
            &f.admin,
            "app",
            ClientEndpointInput {
                frontchannel_logout_uri: None,
                backchannel_logout_uri: None,
                ..after
            },
        )
        .unwrap();
    assert!(removal["proposal"]["after"]["frontchannel_logout_uri"].is_null());
    assert!(removal["proposal"]["after"]["backchannel_logout_uri"].is_null());
    f.core
        .approve_client_endpoint_change(&reviewer, id(&removal), binding(&removal))
        .unwrap();
    f.core
        .execute_client_endpoint_change(&executor, id(&removal), binding(&removal))
        .unwrap();
    let result = f
        .core
        .end_session(logout(&tokens[2]), None, Some(&removed))
        .unwrap();
    assert_eq!(result["frontchannel_urls"], json!([]));
    assert_eq!(
        f.core
            .store
            .list::<Value>("logout_deliveries")
            .unwrap()
            .len(),
        2
    );
    let claimed = f.core.claim_logout_deliveries().unwrap();
    assert_eq!(claimed.len(), 1);
    assert_eq!(
        claimed[0].0.uri, NEW_BACK,
        "Removal must not retarget or discard an already queued job"
    );
}

#[tokio::test]
async fn decided_logout_frontchannel_uses_current_reviewed_registration() {
    use riauth::logout::LogoutRequest;
    const OLD: &str = "https://rp.example.test/old-front";
    const NEW: &str = "https://rp.example.test/new-front?tenant=one";
    const BACK: &str = "https://rp.example.test/back";
    const RETURN: &str = "https://app.example.test/signed-out";
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    f.client_with_settings(
        "app",
        false,
        ProviderSettings {
            frontchannel_logout_uri: Some(OLD.into()),
            backchannel_logout_uri: Some(BACK.into()),
            post_logout_redirect_uris: vec![RETURN.into()],
            ..Default::default()
        },
    );
    #[derive(Clone, Copy)]
    enum Decision {
        Targeted,
        Browser,
        Terminal,
    }
    let cases = [
        ("targeted", Decision::Targeted, false),
        ("nohint-browser-legacy", Decision::Browser, true),
        ("nohint-terminal", Decision::Terminal, false),
        ("nohint-terminal-legacy", Decision::Terminal, true),
    ]
    .map(|(name, decision, legacy)| {
        let token = f.user(name);
        let rp = f.tokens("app", &token, None);
        let sso = matches!(decision, Decision::Browser).then(|| cookie(&f.core, &token));
        let pending = f
            .core
            .end_session(
                LogoutRequest {
                    id_token_hint: matches!(decision, Decision::Targeted)
                        .then(|| rp["id_token"].as_str().unwrap().into()),
                    client_id: Some("app".into()),
                    post_logout_redirect_uri: Some(RETURN.into()),
                    state: Some("unchanged".into()),
                },
                sso.as_deref(),
                None,
            )
            .unwrap();
        let (id, binding) = logout_handle(&pending);
        if let Some(sso) = sso.as_deref() {
            f.core
                .logout_browser_decide(id, Some(binding), Some(sso), true)
                .unwrap();
        } else {
            f.core
                .logout_request_decide(&token, pending["user_code"].as_str().unwrap(), true)
                .unwrap();
        }
        assert!(f.core.me(&token).is_err());
        let mut stored: Value = f
            .core
            .store
            .get("logout_confirmations", id)
            .unwrap()
            .unwrap();
        assert_eq!(
            stored["result"]["frontchannel_urls"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(
            stored["result"]["frontchannel_urls"][0]
                .as_str()
                .unwrap()
                .contains("old-front")
        );
        if legacy {
            stored
                .as_object_mut()
                .unwrap()
                .remove("completed_session_id");
            f.core
                .store
                .write(|tx| tx.put("logout_confirmations", id, &stored))
                .unwrap();
        }
        (pending, legacy && matches!(decision, Decision::Terminal))
    });
    let queued = f.core.store.list::<Value>("logout_deliveries").unwrap();
    assert_eq!(queued.len(), cases.len());
    assert!(queued.iter().all(|(_, delivery)| delivery["uri"] == BACK));
    let update = |front: Option<&str>| {
        let before = client(&f);
        let change = f
            .core
            .stage_client_endpoint(
                &f.admin,
                "app",
                ClientEndpointInput {
                    redirect_uris: before.redirect_uris,
                    origins: before.settings.origins,
                    post_logout_redirect_uris: before.settings.post_logout_redirect_uris,
                    frontchannel_logout_uri: front.map(str::to_owned),
                    backchannel_logout_uri: before.settings.backchannel_logout_uri,
                },
            )
            .unwrap();
        f.core
            .approve_client_endpoint_change(&reviewer, id(&change), binding(&change))
            .unwrap();
        f.core
            .execute_client_endpoint_change(&executor, id(&change), binding(&change))
            .unwrap();
        assert_eq!(
            f.core.store.list::<Value>("logout_deliveries").unwrap(),
            queued
        );
    };
    let app = riauth::api::router(f.core.clone());
    for front in [Some(NEW), None] {
        update(front);
        for (pending, legacy_without_session) in &cases {
            let (id, binding) = logout_handle(pending);
            let stored: Value = f
                .core
                .store
                .get("logout_confirmations", id)
                .unwrap()
                .unwrap();
            assert!(
                stored["result"]["frontchannel_urls"][0]
                    .as_str()
                    .unwrap()
                    .contains("old-front")
            );
            let result = f.core.logout_request_resume(id, Some(binding)).unwrap();
            assert_eq!(result["logged_out"], true);
            assert_eq!(result["redirect_uri"], format!("{RETURN}?state=unchanged"));
            let expected_front = front.filter(|_| !*legacy_without_session);
            let urls = result["frontchannel_urls"].as_array().unwrap();
            assert_eq!(urls.len(), usize::from(expected_front.is_some()));
            if let Some(expected) = expected_front {
                let actual = url::Url::parse(urls[0].as_str().unwrap()).unwrap();
                let expected = url::Url::parse(expected).unwrap();
                assert_eq!(actual.path(), expected.path());
                assert!(
                    actual
                        .query_pairs()
                        .any(|(k, v)| k == "iss" && v == f.core.config.issuer)
                );
                assert!(
                    actual
                        .query_pairs()
                        .any(|(k, v)| k == "sid" && !v.is_empty())
                );
            }
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(pending["resume_uri"].as_str().unwrap())
                        .header("host", "localhost:9000")
                        .header("accept", "text/html")
                        .header(
                            "cookie",
                            pending["set_cookie"]
                                .as_str()
                                .unwrap()
                                .split(';')
                                .next()
                                .unwrap(),
                        )
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            if expected_front.is_some() {
                assert_eq!(response.status(), StatusCode::OK);
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
                assert!(body.contains("<iframe") && body.contains("new-front"));
                assert!(!body.contains("old-front"));
            } else {
                assert!(response.status().is_redirection());
                assert_eq!(
                    response.headers()["location"],
                    format!("{RETURN}?state=unchanged")
                );
            }
        }
    }
}

#[test]
fn disabled_client_endpoint_edits_bind_and_preserve_existing_revocation_semantics() {
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
    baseline.settings.post_logout_redirect_uris = input().post_logout_redirect_uris;
    baseline.settings.frontchannel_logout_uri = input().frontchannel_logout_uri;
    baseline.settings.backchannel_logout_uri = input().backchannel_logout_uri;
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
    let deliveries = f.core.claim_logout_deliveries().unwrap();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(
        deliveries[0].0.uri, "https://app.example.test/logout",
        "Disabled cleanup queues to the prior URL before storing the reviewed replacement"
    );
}
