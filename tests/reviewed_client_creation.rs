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
    model::{Client, ClientCreationBinding, ClientPatch, Group, NewClient, NewUser, UserPatch},
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
fn binding(change: &Value) -> ClientCreationBinding {
    ClientCreationBinding {
        digest: change["digest"].as_str().unwrap().into(),
    }
}
fn input() -> NewClient {
    NewClient {
        client_id: "reviewed-app".into(),
        name: "Reviewed application".into(),
        confidential: true,
        service: false,
        redirect_uris: vec!["https://app.example.test/callback".into()],
        scopes: ["openid".into(), "profile".into()].into(),
        allowed_groups: ["staff".into()].into(),
        require_mfa: true,
        settings: Default::default(),
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
    let change = f.core.stage_client_creation(&f.admin, input()).unwrap();
    f.core
        .approve_client_creation_change(reviewer, id(&change), binding(&change))
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
        serde_json::from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes))),
    )
}

// Exercise the real remote CLI against this same service, with private fixture credentials.
async fn cli(
    f: &Fixture,
    binary: &std::path::Path,
    server: &str,
    token: &str,
    args: &[&str],
) -> (i32, Value) {
    let session = f._dir.path().join("client-creation-session.json");
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
async fn configured_creation_review_binds_content_authority_dependencies_and_issues_secret_once() {
    // Cargo releases its shared target lock before running tests. Another worktree
    // can replace the CLI while the API checks run; hold this build's inode without
    // copying a large executable or retaining it after the test.
    let built_cli = std::path::Path::new(env!("CARGO_BIN_EXE_riauth"));
    let pinned = tempfile::Builder::new()
        .prefix(".reviewed-client-creation-")
        .tempdir_in(built_cli.parent().unwrap())
        .unwrap();
    let binary = pinned.path().join("riauth");
    std::fs::hard_link(built_cli, &binary).unwrap();
    let mut f = Fixture::new();
    assert!(!f.core.config.reviewed_client_creation);
    f.client("app", true); // Ordinary creation is unchanged with the policy omitted.
    refused(&f, || f.core.stage_client_creation(&f.admin, input()), 400);
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    f.user("member");
    f.core.create_group(&f.admin, "staff").unwrap();
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
    let scoped = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "client-writer".into(),
                ttl: 600,
                parent: None,
                permissions: ["client.read", "client.write", "client.rotate"]
                    .map(|action| Permission {
                        action: action.into(),
                        resource: "*".into(),
                    })
                    .into(),
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let registration = f
        .core
        .registration_template(
            &f.admin,
            riauth::registration::RegistrationTemplate {
                id: "before-policy".into(),
                redirect_uris: vec!["https://app.example.test/callback".into()],
                scopes: ["openid".into()].into(),
                grant_types: ["authorization_code".into()].into(),
                auth_methods: ["client_secret_basic".into()].into(),
                settings: Default::default(),
                allowed_groups: Default::default(),
                require_mfa: false,
                ttl: 600,
                max_uses: 1,
            },
        )
        .unwrap();
    // This valid stored plan predates activation. Apply must gate the final writer
    // and roll back earlier speculative user creation, even at the same revision.
    let manifest: Manifest = serde_json::from_value(json!({"api_version":"riauth/v1",
        "users":[{"username":"planned-user","display_name":"Planned","password_disabled":true}],
        "clients":[{"client_id":"planned-app","name":"Planned app","scopes":["openid"]}]}))
    .unwrap();
    let plan = f.core.plan_state(&f.admin, manifest.clone()).unwrap();
    let at = revision(&f);
    f.core.config.reviewed_client_creation = true;
    f.core.config.validate().unwrap();
    assert_eq!(revision(&f), at);
    refused(&f, || f.core.plan_state(&f.admin, manifest), 409);
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
                    .contains("reviewed client creation")
            );
            result
        },
        409,
    );
    refused(&f, || f.core.create_client(&f.admin, input()), 409);
    refused(&f, || f.core.check_new_client(&f.admin, input()), 409);
    for token in [&owner, &scoped] {
        refused(&f, || f.core.stage_client_creation(token, input()), 403);
    }
    let author_cookie = cookie(&f.core, &f.admin);
    let reviewer_cookie = cookie(&f.core, &reviewer);
    let app = riauth::api::router(f.core.clone());
    let browser = Auth::Browser(&author_cookie, Some("http://localhost:9000"));
    let session = call(
        &app,
        "GET",
        "/api/admin/session",
        browser,
        Value::Null,
        None,
        None,
    )
    .await;
    assert_eq!(session.0, StatusCode::OK);
    assert_eq!(session.1["reviewed_client_creation"], true);
    let marker = session.1["session_marker"].as_str().unwrap();
    assert_eq!(marker.len(), 43);
    assert!(
        marker
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
    );
    assert_ne!(marker, riauth::crypto::digest(&author_cookie));
    let again = call(
        &app,
        "GET",
        "/api/admin/session",
        browser,
        Value::Null,
        None,
        None,
    )
    .await;
    assert_eq!(again.1["session_marker"], marker);
    // Both an SSO binding rotation and a fresh login as the same administrator
    // change the marker. It is neither a browser cookie nor a bearer credential.
    for next_cookie in [
        cookie(&f.core, &f.admin),
        cookie(&f.core, &login(&f, "admin")),
    ] {
        let next = call(
            &app,
            "GET",
            "/api/admin/session",
            Auth::Browser(&next_cookie, Some("http://localhost:9000")),
            Value::Null,
            None,
            None,
        )
        .await;
        assert_eq!(next.0, StatusCode::OK);
        assert_eq!(next.1["user"]["id"], session.1["user"]["id"]);
        assert_ne!(next.1["session_marker"], marker);
    }
    for (path, auth) in [
        (
            "/api/admin/session",
            Auth::Browser(marker, Some("http://localhost:9000")),
        ),
        ("/api/users", Auth::Bearer(marker)),
    ] {
        assert_eq!(
            call(&app, "GET", path, auth, Value::Null, None, None)
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
    }
    let asset = "/portal/assets/client-creation-review.js";
    let script = call(&app, "GET", asset, browser, Value::Null, None, None).await;
    assert_eq!(script.0, StatusCode::OK);
    assert!(
        script
            .1
            .as_str()
            .unwrap()
            .contains("RiAuthClientCreationReview")
    );
    let page = call(&app, "GET", "/admin", browser, Value::Null, None, None).await;
    assert!(page.1.as_str().unwrap().contains(asset));
    let mut headless = f.core.clone();
    headless.config.browser_ui = false;
    let headless = riauth::api::router(headless);
    assert_eq!(
        call(&headless, "GET", asset, browser, Value::Null, None, None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
    let mut ordinary = f.core.clone();
    ordinary.config.reviewed_client_creation = false;
    let ordinary = riauth::api::router(ordinary);
    assert_eq!(
        call(
            &ordinary,
            "GET",
            "/api/admin/session",
            browser,
            Value::Null,
            None,
            None
        )
        .await
        .1["reviewed_client_creation"],
        false
    );
    for (path, auth) in [
        ("/api/clients", Auth::Bearer(&f.admin)),
        ("/api/clients", Auth::Bearer(&scoped)),
        ("/api/admin/clients", browser),
        ("/api/admin/client-checks", browser),
    ] {
        let before = f.snapshot().unwrap();
        assert_eq!(
            call(
                &app,
                "POST",
                path,
                auth,
                json!(input()),
                Some(at),
                Some("denied-create")
            )
            .await
            .0,
            StatusCode::CONFLICT
        );
        f.assert_http_mutation_snapshot(&before);
    }
    let before = f.snapshot().unwrap();
    assert_eq!(
        call(
            &app,
            "POST",
            "/oauth/register",
            Auth::Bearer(registration["initial_access_token"].as_str().unwrap()),
            json!({"redirect_uris":["https://app.example.test/callback"]}),
            None,
            Some("denied-registration")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(
        f.core.registration_templates(&f.admin).unwrap()[0]["used"],
        0
    );
    let endpoint = "/api/admin/client-creation-changes";
    let mut extra = json!(input());
    extra["client_secret"] = json!("caller-chosen-secret-must-not-be-staged");
    assert_eq!(
        call(&app, "POST", endpoint, browser, extra, None, None)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(
            &app,
            "POST",
            endpoint,
            Auth::Browser(&author_cookie, None),
            json!(input()),
            None,
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut advanced = input();
    advanced.settings.source_stage = Some("upstream".into());
    refused(&f, || f.core.stage_client_creation(&f.admin, advanced), 400);
    let mut consent = input();
    consent.settings.implicit_consent = true;
    refused(&f, || f.core.stage_client_creation(&f.admin, consent), 400);
    let mut keys = input();
    keys.settings.token_endpoint_auth_method = Some(riauth::jose::ClientAuthMethod::PrivateKeyJwt);
    refused(&f, || f.core.stage_client_creation(&f.admin, keys), 400);
    let mut certificate = input();
    certificate.settings.default_acr_values = vec![riauth::radius::eap::CERTIFICATE_ACR.into()];
    refused(
        &f,
        || f.core.stage_client_creation(&f.admin, certificate),
        400,
    );

    // Configuring creation review leaves ordinary updates and emergency rotation available.
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
    let old_secret = client(&f).secret_hash;
    f.core.rotate_client_secret(&scoped, "app").unwrap();
    assert!(client(&f).secret_hash != old_secret);
    let at = revision(&f);
    let (status, change) = call(
        &app,
        "POST",
        endpoint,
        browser,
        json!(input()),
        Some(at),
        Some("stage-create"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{change}");
    assert_eq!(revision(&f), at);
    assert_eq!(change["proposal"]["before"], Value::Null);
    assert_eq!(change["proposal"]["after"], json!(input()));
    assert_eq!(change["proposal"]["enabled"], true);
    assert_eq!(change["proposal"]["generate_client_secret"], true);
    assert!(
        f.core
            .store
            .get::<Client>("clients", "reviewed-app")
            .unwrap()
            .is_none()
    );
    let encoded = change.to_string();
    for forbidden in [
        "ri_client_",
        "\"client_secret\"",
        "\"secret_hash\"",
        "validation-only",
    ] {
        assert!(!encoded.contains(forbidden));
    }
    let mut proposal = change["proposal"].clone();
    proposal.sort_all_objects();
    assert_eq!(
        change["digest"],
        riauth::crypto::digest(&format!("riauth/reviewed-client-creation/v1\n{proposal}"))
    );
    let before = f.snapshot().unwrap();
    assert_eq!(
        call(
            &app,
            "POST",
            endpoint,
            browser,
            json!(input()),
            Some(at),
            Some("stage-create")
        )
        .await,
        (status, change.clone())
    );
    f.assert_http_mutation_snapshot(&before);
    let approve = format!("/api/admin/client-creation-changes/{}/approve", id(&change));
    assert_eq!(
        call(
            &app,
            "POST",
            &approve,
            Auth::Browser(&reviewer_cookie, Some("http://localhost:9000")),
            json!({"digest":change["digest"],"after":{}}),
            Some(at),
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
                .approve_client_creation_change(&f.admin, id(&change), binding(&change))
        },
        403,
    );
    refused(
        &f,
        || {
            f.core
                .execute_client_creation_change(&executor, id(&change), binding(&change))
        },
        403,
    );
    refused(
        &f,
        || {
            f.core.approve_client_creation_change(
                &reviewer,
                id(&change),
                ClientCreationBinding {
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
    for token in [&f.admin, &reviewer, &scoped, &owner] {
        refused(
            &f,
            || {
                f.core
                    .execute_client_creation_change(token, id(&change), binding(&change))
            },
            403,
        );
    }
    let stored: Value = f
        .core
        .store
        .get("reviewed_client_creations", id(&change))
        .unwrap()
        .unwrap();
    let mut tampered = stored.clone();
    tampered["proposal"]["after"]["redirect_uris"] =
        json!(["https://attacker.example.test/callback"]);
    f.core
        .store
        .write(|tx| tx.put("reviewed_client_creations", id(&change), &tampered))
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_client_creation_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    f.core
        .store
        .write(|tx| tx.put("reviewed_client_creations", id(&change), &stored))
        .unwrap();
    let group: Group = f.core.store.get("groups", "staff").unwrap().unwrap();
    let mut changed_group = group.clone();
    changed_group.members.insert(
        f.core
            .store
            .get::<String>("usernames", "member")
            .unwrap()
            .unwrap(),
    );
    f.core
        .store
        .write(|tx| tx.put("groups", "staff", &changed_group))
        .unwrap();
    assert_eq!(revision(&f), at);
    refused(
        &f,
        || {
            f.core
                .execute_client_creation_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    f.core
        .store
        .write(|tx| tx.put("groups", "staff", &group))
        .unwrap();
    let mut policy_drift = f.core.clone();
    policy_drift.config.reviewed_client_creation = false;
    refused(
        &f,
        || policy_drift.execute_client_creation_change(&executor, id(&change), binding(&change)),
        409,
    );
    #[cfg(feature = "test-support")]
    riauth::crypto::with_test_time(change["proposal"]["expires_at"].as_u64().unwrap(), || {
        refused(
            &f,
            || {
                f.core
                    .execute_client_creation_change(&executor, id(&change), binding(&change))
            },
            409,
        );
    });
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
                .execute_client_creation_change(&executor, id(&change), binding(&change))
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
                .execute_client_creation_change(&executor, id(&change), binding(&change))
        },
        409,
    );
    assert_eq!(
        call(
            &app,
            "POST",
            &format!("/api/admin/client-creation-changes/{}/cancel", id(&change)),
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
                .execute_client_creation_change(&executor, id(&change), binding(&change))
        },
        409,
    );

    let change = approved(&f, &reviewer);
    // Competing proposals for the same absent identifier can be approved, but
    // only one can create it. There is no reservation that bypasses the writer.
    let competing = approved(&f, &reviewer);
    let at = revision(&f);
    let execute = format!("/api/client-creation-changes/{}/execute", id(&change));
    let result = call(
        &app,
        "POST",
        &execute,
        Auth::Bearer(&executor),
        json!(binding(&change)),
        Some(at),
        Some("execute-create"),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["change"]["status"], "executed");
    assert_eq!(result.1["change"]["proposal"], change["proposal"]);
    let secret = result.1["client_secret"].as_str().unwrap();
    let client: Client = f
        .core
        .store
        .get("clients", "reviewed-app")
        .unwrap()
        .unwrap();
    assert!(client.secret_hash.as_deref() == Some(riauth::crypto::digest(secret).as_str()));
    let mut expected = change["proposal"]["after"].clone();
    expected["enabled"] = json!(true);
    assert_eq!(client.view(), expected);
    assert_eq!(revision(&f), at + 1);
    let read = call(
        &app,
        "GET",
        &format!("/api/admin/client-creation-changes/{}", id(&change)),
        browser,
        Value::Null,
        None,
        None,
    )
    .await;
    assert_eq!(read.1, result.1["change"]);
    let before = f.snapshot().unwrap();
    assert_eq!(
        call(
            &app,
            "POST",
            &execute,
            Auth::Bearer(&executor),
            json!(binding(&change)),
            Some(at),
            Some("execute-create")
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
            Some("fresh-create-replay")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&before);
    refused(
        &f,
        || {
            f.core
                .execute_client_creation_change(&executor, id(&competing), binding(&competing))
        },
        409,
    );
    refused(&f, || f.core.stage_client_creation(&f.admin, input()), 409);
    for action in [
        "reviewed_client_creations.stage",
        "reviewed_client_creations.approve",
        "reviewed_client_creations.execute",
    ] {
        let events = f.core.audit_events(&f.admin, 100).unwrap();
        assert_eq!(
            events
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| e["action"] == action && e["details"]["change_id"] == id(&change))
                .count(),
            1
        );
    }
    let snapshot = f.snapshot().unwrap();
    for (key, value) in snapshot {
        if !key.starts_with("receipts/") {
            assert!(
                !value.to_string().contains(secret),
                "Generated credential leaked outside execution receipt: {key}"
            );
        }
    }

    // The real CLI shares the fence and requires an explicit secret destination
    // before execution; its draft contains only canonical content and intent.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_url = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let before = f.snapshot().unwrap();
    let denied = cli(
        &f,
        &binary,
        &server_url,
        &f.admin,
        &["client", "create", "direct-cli", "--scope", "openid"],
    )
    .await;
    assert_eq!(denied.0, 5);
    assert_eq!(denied.1["error"]["http_status"], 409);
    f.assert_http_mutation_snapshot(&before);
    let file = f._dir.path().join("new-client.json");
    let mut service = input();
    service.client_id = "service-app".into();
    service.service = true;
    service.confidential = false; // Canonical effective type is confidential.
    service.scopes = ["api".into()].into();
    service.allowed_groups.clear();
    service.require_mfa = false;
    service.redirect_uris.clear();
    std::fs::write(&file, serde_json::to_vec(&service).unwrap()).unwrap();
    let (exit, staged) = cli(
        &f,
        &binary,
        &server_url,
        &f.admin,
        &[
            "client",
            "creation-review",
            "stage",
            "--file",
            file.to_str().unwrap(),
        ],
    )
    .await;
    assert_eq!(exit, 0, "{staged}");
    let staged = &staged["data"];
    assert_eq!(staged["proposal"]["after"]["confidential"], true);
    assert_eq!(staged["proposal"]["generate_client_secret"], true);
    let digest = staged["digest"].as_str().unwrap();
    assert_eq!(
        cli(
            &f,
            &binary,
            &server_url,
            &reviewer,
            &[
                "client",
                "creation-review",
                "approve",
                id(staged),
                "--digest",
                digest
            ]
        )
        .await
        .0,
        0
    );
    let before = f.snapshot().unwrap();
    assert_ne!(
        cli(
            &f,
            &binary,
            &server_url,
            &executor,
            &[
                "client",
                "creation-review",
                "execute",
                id(staged),
                "--digest",
                digest
            ]
        )
        .await
        .0,
        0
    );
    f.assert_snapshot(&before);
    let output = f._dir.path().join("created-client.json");
    let (exit, response) = cli(
        &f,
        &binary,
        &server_url,
        &executor,
        &[
            "--output-file",
            output.to_str().unwrap(),
            "client",
            "creation-review",
            "execute",
            id(staged),
            "--digest",
            digest,
        ],
    )
    .await;
    assert_eq!(exit, 0);
    assert!(response["data"]["client_secret"].is_null());
    let written: Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    assert!(
        written["client_secret"]
            .as_str()
            .unwrap()
            .starts_with("ri_client_")
    );
    assert_eq!(written["change"]["status"], "executed");
    server.abort();
    // Public creation can also consume a review, with no credential at all.
    let mut public = input();
    public.client_id = "public-app".into();
    public.confidential = false;
    let public = f.core.stage_client_creation(&f.admin, public).unwrap();
    assert_eq!(public["proposal"]["generate_client_secret"], false);
    f.core
        .approve_client_creation_change(&reviewer, id(&public), binding(&public))
        .unwrap();
    let public = f
        .core
        .execute_client_creation_change(&executor, id(&public), binding(&public))
        .unwrap();
    assert!(public["client_secret"].is_null());
    assert_eq!(public["client"]["confidential"], false);
}
