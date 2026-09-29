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
        Client, ClientPatch, ClientPolicyBinding, ClientPolicyInput, Group, NewUser, UserPatch,
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
fn binding(change: &Value) -> ClientPolicyBinding {
    ClientPolicyBinding {
        digest: change["digest"].as_str().unwrap().into(),
    }
}
fn input() -> ClientPolicyInput {
    ClientPolicyInput {
        allowed_groups: ["staff".into()].into(),
        require_mfa: true,
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
        .stage_client_policy(&f.admin, "app", input())
        .unwrap();
    f.core
        .approve_client_policy_change(reviewer, id(&change), binding(&change))
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
async fn cli(f: &Fixture, server: &str, token: &str, args: &[&str]) -> (i32, Value) {
    let session = f._dir.path().join("client-policy-session.json");
    riauth::config::write_private(
        &session,
        &serde_json::to_vec(&json!({
            "issuer": server, "token": token, "expires_at": riauth::crypto::now() + 600,
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
        .current_dir(f._dir.path())
        .stdin(std::process::Stdio::null());
    let output = tokio::task::spawn_blocking(move || command.output().unwrap())
        .await
        .unwrap();
    let value = serde_json::from_slice(&output.stdout).expect("CLI JSON response");
    (output.status.code().unwrap(), value)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_policy_review_keeps_csrf_exact_receipts_and_credentials() {
    let f = Fixture::new();
    let reviewer = administrator(&f, "browser-reviewer");
    let executor = administrator(&f, "browser-executor");
    f.core.create_group(&f.admin, "staff").unwrap();
    f.client("app", true);
    let before = serde_json::to_value(client(&f)).unwrap();
    let cookies = [&f.admin, &reviewer, &executor].map(|token| cookie(&f.core, token));
    let auth = cookies
        .each_ref()
        .map(|cookie| Auth::Browser(cookie, Some("http://localhost:9000")));
    let app = riauth::api::router(f.core.clone());
    let asset = "/portal/assets/client-policy-review.js";
    let script = call(&app, "GET", asset, auth[0], Value::Null, None, None).await;
    assert_eq!(script.0, StatusCode::OK);
    assert!(
        script
            .1
            .as_str()
            .unwrap()
            .contains("RiAuthClientPolicyReview")
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

    let at = revision(&f);
    let stage = "/api/admin/clients/app/policy-changes";
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
    assert_eq!(staged.0, StatusCode::OK);
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
    assert_eq!(serde_json::to_value(client(&f)).unwrap(), before);
    let change = &staged.1;
    let endpoint = format!("/api/admin/client-policy-changes/{}", id(change));
    let digest = json!(binding(change));
    // The new browser UI must use the same portal write guard at every transition.
    for path in [
        stage.to_owned(),
        format!("{endpoint}/approve"),
        format!("{endpoint}/execute"),
        format!("{endpoint}/cancel"),
    ] {
        let body = if path == stage {
            json!(input())
        } else {
            digest.clone()
        };
        for origin in [None, Some("http://attacker.example")] {
            let snapshot = f.snapshot().unwrap();
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
            f.assert_http_mutation_snapshot(&snapshot);
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
            "missing portal header: {path}"
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
            digest.clone(),
            Some(at),
            Some(action),
        )
        .await;
        assert_eq!(response.0, StatusCode::OK, "{}", response.1);
        assert_eq!(response.1["status"], status);
        assert_eq!(response.1["proposal"], change["proposal"]);
        assert_eq!(response.1["digest"], change["digest"]);
        assert_eq!(
            call(
                &app,
                "POST",
                &path,
                actor,
                digest.clone(),
                Some(at),
                Some(action)
            )
            .await,
            response
        );
        assert!(response.1.get("client_secret").is_none());
    }
    let after = serde_json::to_value(client(&f)).unwrap();
    let mut expected = before;
    expected["allowed_groups"] = json!(["staff"]);
    expected["require_mfa"] = json!(true);
    assert_eq!(
        after, expected,
        "all credentials and non-policy settings are preserved"
    );
    assert_eq!(revision(&f), at + 1);
    let snapshot = f.snapshot().unwrap();
    assert_eq!(
        call(
            &app,
            "POST",
            &format!("{endpoint}/execute"),
            auth[2],
            digest,
            None,
            Some("fresh-replay")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&snapshot);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn client_policy_review_binds_exact_content_authority_dependencies_and_blocks_bypass_replay()
{
    let f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    f.user("member");
    let owner = f.user("owner");
    f.core.create_group(&f.admin, "staff").unwrap();
    f.client("app", true);
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
                id: "client-operator".into(),
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
    let author_cookie = cookie(&f.core, &f.admin);
    let reviewer_cookie = cookie(&f.core, &reviewer);
    let app = riauth::api::router(f.core.clone());
    let browser = Auth::Browser(&author_cookie, Some("http://localhost:9000"));

    // Each field is protected even when requested alone; retained identical fields are ordinary.
    for patch in [
        json!({"allowed_groups":["staff"]}),
        json!({"require_mfa":true}),
    ] {
        for auth in [Auth::Bearer(&f.admin), Auth::Bearer(&scoped), browser] {
            let path = if matches!(auth, Auth::Browser(..)) {
                "/api/admin/clients/app"
            } else {
                "/api/clients/app"
            };
            let before = f.snapshot().unwrap();
            let (status, error) = call(
                &app,
                "PATCH",
                path,
                auth,
                patch.clone(),
                Some(revision(&f)),
                Some("direct-policy-denied"),
            )
            .await;
            assert_eq!(status, StatusCode::CONFLICT, "{error}");
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
            || f.core.stage_client_policy(token, "app", input()),
            403,
        );
    }
    refused(
        &f,
        || {
            f.core.update_client(
                &owner,
                "app",
                ClientPatch {
                    require_mfa: Some(true),
                    ..Default::default()
                },
            )
        },
        403,
    );
    let mut manifest: Manifest =
        serde_json::from_value(f.core.export_state(&f.admin).unwrap()["manifest"].clone()).unwrap();
    manifest.users.clear();
    manifest.groups.clear();
    manifest.clients[0].require_mfa = true;
    refused(&f, || f.core.plan_state(&f.admin, manifest.clone()), 409);
    // An already-stored ordinary plan cannot overwrite a policy that drifted
    // without a revision bump: final apply must still reach the shared fence.
    manifest.clients[0].require_mfa = false;
    manifest.clients[0].name = "Planned ordinary edit".into();
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    let baseline = client(&f);
    let mut drifted = baseline.clone();
    drifted.require_mfa = true;
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &drifted))
        .unwrap();
    assert_eq!(revision(&f), plan.base_revision);
    refused(
        &f,
        || {
            let result = f.core.apply_state(
                &f.admin,
                riauth::state::ApplyRequest {
                    plan,
                    secrets: Default::default(),
                    run_id: None,
                },
            );
            assert!(
                result
                    .as_ref()
                    .err()
                    .unwrap()
                    .message
                    .contains("reviewed client policy")
            );
            result
        },
        409,
    );
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &baseline))
        .unwrap();
    // A legacy private-key client with a leftover shared secret would normally
    // normalize credentials in the writer. A two-field policy review must not
    // silently authorize that extra effect; require separate authentication repair.
    let mut legacy = baseline.clone();
    legacy.settings.token_endpoint_auth_method =
        Some(riauth::jose::ClientAuthMethod::PrivateKeyJwt);
    legacy.settings.jwks = Some(serde_json::from_value(f.core.jwks().unwrap()).unwrap());
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &legacy))
        .unwrap();
    refused(
        &f,
        || f.core.stage_client_policy(&f.admin, "app", input()),
        409,
    );
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &baseline))
        .unwrap();

    // Existing scoped preconditions and ordinary owner edits keep their accepted contracts.
    assert_eq!(
        call(
            &app,
            "PATCH",
            "/api/clients/app",
            Auth::Bearer(&scoped),
            json!({"name":"Renamed"}),
            None,
            None
        )
        .await
        .0,
        StatusCode::PRECONDITION_REQUIRED
    );
    assert_eq!(
        call(
            &app,
            "PATCH",
            "/api/clients/app",
            Auth::Bearer(&scoped),
            json!({"name":"Renamed", "allowed_groups":[], "require_mfa":false}),
            Some(revision(&f)),
            Some("scoped-client-name-update")
        )
        .await
        .0,
        StatusCode::OK
    );
    f.core
        .update_client(
            &owner,
            "app",
            ClientPatch {
                name: Some("Owner edit".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let endpoint = "/api/clients/app/policy-changes";
    let before = f.snapshot().unwrap();
    for body in [
        json!({"require_mfa":true}),
        json!({"allowed_groups":["staff"],"require_mfa":true,"name":"Smuggled"}),
    ] {
        assert_eq!(
            call(
                &app,
                "POST",
                endpoint,
                Auth::Bearer(&f.admin),
                body,
                None,
                None
            )
            .await
            .0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/admin/clients/app/policy-changes",
            Auth::Browser(&author_cookie, None),
            json!(input()),
            None,
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    f.assert_http_mutation_snapshot(&before);
    let at = revision(&f);
    let (status, change) = call(
        &app,
        "POST",
        endpoint,
        Auth::Bearer(&f.admin),
        json!(input()),
        Some(at),
        Some("stage-policy"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{change}");
    assert_eq!(
        change["proposal"]["before"],
        json!({"allowed_groups":[],"require_mfa":false})
    );
    assert_eq!(change["proposal"]["after"], json!(input()));
    assert_eq!(
        change["proposal"]["expires_at"].as_u64().unwrap()
            - change["proposal"]["created_at"].as_u64().unwrap(),
        900
    );
    assert_eq!(revision(&f), at);
    let before = f.snapshot().unwrap();
    assert_eq!(
        call(
            &app,
            "POST",
            endpoint,
            Auth::Bearer(&f.admin),
            json!(input()),
            Some(at),
            Some("stage-policy")
        )
        .await,
        (status, change.clone())
    );
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("/api/admin/client-policy-changes/{}", id(&change)),
            browser,
            Value::Null,
            None,
            None
        )
        .await
        .1,
        change
    );
    let approve = format!("/api/admin/client-policy-changes/{}/approve", id(&change));
    assert_eq!(
        call(
            &app,
            "POST",
            &approve,
            Auth::Browser(&reviewer_cookie, Some("http://localhost:9000")),
            json!({"digest":change["digest"],"require_mfa":false}),
            None,
            None
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    refused(
        &f,
        || {
            f.core
                .approve_client_policy_change(&f.admin, id(&change), binding(&change))
        },
        403,
    );
    refused(
        &f,
        || {
            f.core
                .execute_client_policy_change(&executor, id(&change), binding(&change))
        },
        403,
    );
    refused(
        &f,
        || {
            f.core.approve_client_policy_change(
                &reviewer,
                id(&change),
                ClientPolicyBinding {
                    digest: "substituted".into(),
                },
            )
        },
        409,
    );
    assert_eq!(
        call(
            &app,
            "POST",
            &approve,
            Auth::Browser(&reviewer_cookie, Some("http://localhost:9000")),
            json!(binding(&change)),
            Some(at),
            None
        )
        .await
        .0,
        StatusCode::OK
    );
    for token in [&f.admin, &reviewer] {
        refused(
            &f,
            || {
                f.core
                    .execute_client_policy_change(token, id(&change), binding(&change))
            },
            403,
        );
    }

    // Stored-content substitution, including a loosened MFA requirement, cannot reuse approval.
    let stored: Value = f
        .core
        .store
        .get("reviewed_client_policies", id(&change))
        .unwrap()
        .unwrap();
    let mut tampered = stored.clone();
    tampered["proposal"]["after"]["require_mfa"] = json!(false);
    f.core
        .store
        .write(|tx| tx.put("reviewed_client_policies", id(&change), &tampered))
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_policy_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    f.core
        .store
        .write(|tx| tx.put("reviewed_client_policies", id(&change), &stored))
        .unwrap();

    // Independently bound dependencies catch drift even without a management revision change.
    let baseline = client(&f);
    let mut drifted = baseline.clone();
    drifted.name = "Out of band change".into();
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &drifted))
        .unwrap();
    assert_eq!(revision(&f), at);
    refused(
        &f,
        || {
            f.core
                .execute_client_policy_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    f.core
        .store
        .write(|tx| tx.put("clients", "app", &baseline))
        .unwrap();
    let group: Group = f.core.store.get("groups", "staff").unwrap().unwrap();
    let mut drifted_group = group.clone();
    drifted_group.members.insert(
        f.core
            .store
            .get::<String>("usernames", "member")
            .unwrap()
            .unwrap(),
    );
    f.core
        .store
        .write(|tx| tx.put("groups", "staff", &drifted_group))
        .unwrap();
    assert_eq!(revision(&f), at);
    refused(
        &f,
        || {
            f.core
                .execute_client_policy_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    f.core
        .store
        .write(|tx| tx.put("groups", "staff", &group))
        .unwrap();
    let mut config_drift = f.core.clone();
    config_drift
        .config
        .capabilities
        .disabled
        .insert("oidc.userinfo".into());
    refused(
        &f,
        || config_drift.execute_client_policy_change(&executor, id(&change), binding(&change)),
        409,
    );
    #[cfg(feature = "test-support")]
    riauth::crypto::with_test_time(change["proposal"]["expires_at"].as_u64().unwrap(), || {
        refused(
            &f,
            || {
                f.core
                    .execute_client_policy_change(&executor, id(&change), binding(&change))
            },
            409,
        );
    });

    // Authority is checked live; restoring a demoted reviewer cannot revive their old approval.
    f.core
        .update_user(
            &f.admin,
            "reviewer",
            UserPatch {
                admin: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_policy_change(&executor, id(&change), binding(&change))
        },
        403,
    );
    f.core
        .update_user(
            &f.admin,
            "reviewer",
            UserPatch {
                admin: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let reviewer = login(&f, "reviewer");
    refused(
        &f,
        || {
            f.core
                .execute_client_policy_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    assert_eq!(
        call(
            &app,
            "POST",
            &format!("/api/admin/client-policy-changes/{}/cancel", id(&change)),
            browser,
            json!(binding(&change)),
            None,
            None
        )
        .await
        .0,
        StatusCode::OK
    );
    refused(
        &f,
        || {
            f.core
                .execute_client_policy_change(&executor, id(&change), binding(&change))
        },
        409,
    );

    let change = approved(&f, &reviewer);
    let old_secret = client(&f).secret_hash;
    f.core.rotate_client_secret(&scoped, "app").unwrap();
    assert_ne!(client(&f).secret_hash, old_secret);
    refused(
        &f,
        || {
            f.core
                .execute_client_policy_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    let change = approved(&f, &reviewer);
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                name: Some("Still immediate".into()),
                ..Default::default()
            },
        )
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_policy_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    let change = approved(&f, &reviewer);
    let baseline = client(&f);
    let at = revision(&f);
    let execute = format!("/api/client-policy-changes/{}/execute", id(&change));
    let result = call(
        &app,
        "POST",
        &execute,
        Auth::Bearer(&executor),
        json!(binding(&change)),
        Some(at),
        Some("execute-policy"),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK, "{}", result.1);
    assert_eq!(result.1["status"], "executed");
    assert_eq!(result.1["proposal"], change["proposal"]);
    assert_eq!(revision(&f), at + 1);
    let mut expected = baseline;
    expected.allowed_groups = input().allowed_groups;
    expected.require_mfa = true;
    assert!(
        serde_json::to_value(client(&f)).unwrap() == serde_json::to_value(&expected).unwrap(),
        "Only the two approved policy fields may change"
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
            Some("execute-policy")
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
            Some(revision(&f)),
            Some("fresh-replay")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&before);

    // Real CLI direct writes hit the same fence, and review can explicitly loosen both fields.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let before = f.snapshot().unwrap();
    let at = revision(&f).to_string();
    let denied = cli(
        &f,
        &server_url,
        &f.admin,
        &[
            "--if-revision",
            &at,
            "--idempotency-key",
            "direct-policy-cli-denied",
            "client",
            "update",
            "app",
            "--allow-all",
            "--require-mfa",
            "false",
        ],
    )
    .await;
    assert_eq!(denied.0, 5, "{}", denied.1);
    assert_eq!(denied.1["error"]["http_status"], 409);
    f.assert_http_mutation_snapshot(&before);
    let file = f._dir.path().join("policy.json");
    std::fs::write(&file, r#"{"allowed_groups":[],"require_mfa":false}"#).unwrap();
    let (exit, change) = cli(
        &f,
        &server_url,
        &f.admin,
        &[
            "client",
            "review",
            "stage",
            "app",
            "--file",
            file.to_str().unwrap(),
        ],
    )
    .await;
    assert_eq!(exit, 0, "{change}");
    let change = &change["data"];
    let digest = change["digest"].as_str().unwrap();
    assert_eq!(
        cli(
            &f,
            &server_url,
            &reviewer,
            &[
                "client",
                "review",
                "approve",
                id(change),
                "--digest",
                digest
            ]
        )
        .await
        .0,
        0
    );
    assert_eq!(
        cli(
            &f,
            &server_url,
            &executor,
            &[
                "client",
                "review",
                "execute",
                id(change),
                "--digest",
                digest
            ]
        )
        .await
        .0,
        0
    );
    server.abort();
    assert!(client(&f).allowed_groups.is_empty());
    assert!(!client(&f).require_mfa);
    assert_eq!(client(&f).secret_hash, expected.secret_hash);
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    for action in [
        "reviewed_client_policies.stage",
        "reviewed_client_policies.approve",
        "reviewed_client_policies.execute",
    ] {
        assert_eq!(
            events
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["action"] == action && e["details"]["change_id"] == id(change))
                .count(),
            1
        );
    }
    let serialized = serde_json::to_string(&events).unwrap();
    assert!(!serialized.contains(expected.secret_hash.as_deref().unwrap()));
}
