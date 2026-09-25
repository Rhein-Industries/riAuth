use super::*;

#[tokio::test]
async fn http_routes_enforce_protocol_encoding_and_authentication() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new();
    f.client("app", false);
    let app = riauth::api::router(f.core.clone());
    let discovery = app
        .clone()
        .oneshot(
            Request::get("/.well-known/openid-configuration")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(discovery.status(), StatusCode::OK);
    assert_eq!(discovery.headers()["cache-control"], "no-store");
    let bytes = discovery.into_body().collect().await.unwrap().to_bytes();
    let discovery: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(discovery["issuer"], f.core.config.issuer);
    assert_eq!(
        discovery["code_challenge_methods_supported"],
        json!(["S256"])
    );
    let denied = app
        .clone()
        .oneshot(Request::get("/api/users").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let duplicate = app
        .clone()
        .oneshot(
            Request::post("/oauth/device/code")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("client_id=app&client_id=app&scope=openid"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(duplicate.status(), StatusCode::BAD_REQUEST);
    let result = app
        .clone()
        .oneshot(
            Request::post("/oauth/device/code")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("client_id=app&scope=openid"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::OK);
    let value: Value =
        serde_json::from_slice(&result.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(value["user_code"].is_string());
    let too_large = app
        .oneshot(
            Request::post("/api/login")
                .header("content-type", "application/json")
                .body(Body::from("x".repeat(33 * 1024)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(too_large.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn http_authorization_roundtrip_and_basic_auth() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use base64::engine::general_purpose::STANDARD;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new();
    let secret = f.client("app", true).unwrap();
    let app = riauth::api::router(f.core.clone());
    let verifier = crypto::random_token("");
    let request = f.request("app", &verifier);
    let encoded = serde_urlencoded::to_string(request).unwrap();
    let result = app
        .clone()
        .oneshot(
            Request::get(format!("/oauth/authorize?{encoded}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::OK);
    let result = app
        .clone()
        .oneshot(
            Request::post("/oauth/authorize")
                .header("authorization", format!("Bearer {}", f.admin))
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(encoded))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::FOUND);
    let location = url::Url::parse(result.headers()["location"].to_str().unwrap()).unwrap();
    let code = location
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    let body = serde_urlencoded::to_string([
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", "http://localhost:7777/callback?existing=1"),
        ("code_verifier", &verifier),
    ])
    .unwrap();
    let result = app
        .oneshot(
            Request::post("/oauth/token")
                .header(
                    "authorization",
                    format!("Basic {}", STANDARD.encode(format!("app:{secret}"))),
                )
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::OK);
    let value: Value =
        serde_json::from_slice(&result.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(value["id_token"].is_string());
}

#[test]
fn forwarding_headers_cannot_spoof_the_rate_limit_identity() {
    use axum::http::HeaderMap;
    let trusted = ["127.0.0.1".parse().unwrap(), "10.0.0.1".parse().unwrap()];
    let real = "192.0.2.7".parse().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-forwarded-for",
        "198.51.100.10, 192.0.2.7, 10.0.0.1".parse().unwrap(),
    );
    assert_eq!(
        riauth::api::proxy_client_ip(real, &headers, &trusted).unwrap(),
        real
    );
    assert_eq!(
        riauth::api::proxy_client_ip(trusted[0], &headers, &trusted).unwrap(),
        real
    );
    headers.insert("x-forwarded-for", "not-an-ip".parse().unwrap());
    assert_eq!(
        riauth::api::proxy_client_ip(real, &headers, &trusted).unwrap(),
        real
    );
    assert!(riauth::api::proxy_client_ip(trusted[0], &headers, &trusted).is_err());
}

#[tokio::test]
async fn issuer_paths_and_cors_preserve_exact_issuer_and_client_origins() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let mut f = Fixture::new();
    f.client("app", false);
    f.core.config.issuer = "http://localhost:9000/application/o/app/".into();
    f.core.config.validate().unwrap();
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(riauth::model::ProviderSettings {
                    origins: strings(&["https://app.example.test"]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let app = riauth::api::router(f.core.clone());
    let response = app
        .clone()
        .oneshot(
            Request::get("/application/o/app/.well-known/openid-configuration")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let metadata: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(metadata["issuer"], f.core.config.issuer);
    assert_eq!(
        metadata["token_endpoint"],
        "http://localhost:9000/application/o/app/oauth/token"
    );
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/application/o/app/oauth/token")
                .header("origin", "https://app.example.test")
                .header("access-control-request-method", "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        response.headers()["access-control-allow-origin"],
        "https://app.example.test"
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/application/o/app/oauth/token")
                .header("origin", "https://attacker.invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        !response
            .headers()
            .contains_key("access-control-allow-origin")
    );
}

#[tokio::test]
async fn scim_http_provisioning_is_owned_atomic_retriable_and_deprovisions_sessions() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            riauth::agent::NewAgent {
                id: "scim-agent".into(),
                ttl: 600,
                parent: None,
                permissions: [
                    "user.read",
                    "user.write",
                    "group.read",
                    "group.write",
                    "group.members",
                ]
                .map(|action| riauth::agent::Permission {
                    action: action.into(),
                    resource: "*".into(),
                })
                .into(),
            },
        )
        .unwrap();
    let credential = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let rejected = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/scim/v2/Users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(rejected.headers()["content-type"], "application/scim+json");
    let rev = f
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap();
    let input = json!({"schemas":[riauth::scim::USER],"userName":"scim-alice","externalId":"directory-42","displayName":"Directory Alice","password":PASSWORD,"active":true,"emails":[{"value":"scim-alice@example.test","primary":true}]});
    let mut created = Value::Null;
    for _ in 0..2 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/scim/v2/Users")
                    .header("authorization", format!("Bearer {credential}"))
                    .header("if-match", format!("\"{rev}\""))
                    .header("idempotency-key", "scim-create-42")
                    .header("content-type", "application/scim+json")
                    .body(Body::from(input.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        assert!(response.headers().contains_key("etag"));
        assert!(response.headers().contains_key("location"));
        let bytes = axum::body::to_bytes(response.into_body(), 32768)
            .await
            .unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            !String::from_utf8(bytes.to_vec())
                .unwrap()
                .contains(PASSWORD)
        );
        if created.is_null() {
            created = result;
        } else {
            assert_eq!(created, result);
        }
    }
    let id = text(&created, "id");
    let list = f
        .core
        .scim_list(
            &credential,
            "Users",
            riauth::scim::Query {
                filter: Some("userName eq \"SCIM-ALICE\"".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(list["totalResults"], 1);
    assert!(f.core.scim_get(&f.admin, "Users", &id).is_err()); // Another operator cannot seize this record.
    let session = text(
        &f.core
            .login("scim-alice".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let patch = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"active","value":false},{"op":"replace","path":"roles","value":["admin"]}]});
    assert!(
        f.core
            .scim_write(&credential, "Users", Some(&id), patch, true)
            .is_err()
    );
    assert!(f.core.me(&session).is_ok());
    let group=f.core.scim_write(&credential,"Groups",None,json!({"schemas":[riauth::scim::GROUP],"displayName":"provisioned","members":[{"value":id}]}),false).unwrap();
    let gid = text(&group, "id");
    assert_eq!(
        f.core.scim_get(&credential, "Users", &id).unwrap()["groups"][0]["value"],
        gid
    );
    let bad = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"add","path":"members","value":[{"value":"unknown"}]}]});
    assert!(
        f.core
            .scim_write(&credential, "Groups", Some(&gid), bad, true)
            .is_err()
    );
    assert_eq!(
        f.core.scim_get(&credential, "Groups", &gid).unwrap()["members"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let remove = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"remove","path":format!("members[value eq \"{id}\"]")} ]});
    assert!(
        f.core
            .scim_write(&credential, "Groups", Some(&gid), remove, true)
            .unwrap()["members"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        f.core
            .scim_write(
                &credential,
                "Users",
                None,
                json!({"schemas":[riauth::scim::USER],"userName":"admin"}),
                false
            )
            .is_err()
    );
    f.core.scim_delete(&credential, "Users", &id).unwrap();
    assert!(f.core.me(&session).is_err());
    assert!(f.core.scim_get(&credential, "Users", &id).is_err());
    assert_eq!(
        f.core
            .scim_list(&credential, "Users", Default::default())
            .unwrap()["totalResults"],
        0
    );
}

#[tokio::test]
async fn prometheus_metrics_include_rejections_before_routing_and_require_scoped_authentication() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let f = Fixture::new();
    let app = riauth::api::router(f.core.clone());
    let invalid = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/.well-known/openid-configuration")
                .header("x-riauth-run-id", "bad value")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    assert!(invalid.headers().contains_key("x-request-id"));
    let denied = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/operations/prometheus")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/operations/prometheus")
                .header("authorization", format!("Bearer {}", f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/plain")
    );
    let body = String::from_utf8(
        axum::body::to_bytes(response.into_body(), 32768)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(body.contains("riauth_requests_total 3\n"));
    assert!(body.contains("riauth_response_errors_total 2\n"));
    assert!(body.contains("riauth_request_duration_seconds_count 2\n"));
    assert!(!body.contains(&f.admin));
}

#[tokio::test]
async fn native_tls_serves_trusted_https_and_rejects_untrusted_certificates() {
    use openssl::{
        asn1::Asn1Time,
        ec::{EcGroup, EcKey},
        hash::MessageDigest,
        nid::Nid,
        pkey::PKey,
        x509::{X509, X509NameBuilder, extension::SubjectAlternativeName},
    };
    let f = Fixture::new();
    let dir = tempfile::TempDir::new().unwrap();
    let key = PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
    )
    .unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_text("CN", "localhost").unwrap();
    let name = name.build();
    let mut cert = X509::builder().unwrap();
    cert.set_version(2).unwrap();
    cert.set_subject_name(&name).unwrap();
    cert.set_issuer_name(&name).unwrap();
    cert.set_pubkey(&key).unwrap();
    cert.set_not_before(&Asn1Time::days_from_now(0).unwrap())
        .unwrap();
    cert.set_not_after(&Asn1Time::days_from_now(1).unwrap())
        .unwrap();
    let san = SubjectAlternativeName::new()
        .dns("localhost")
        .ip("127.0.0.1")
        .build(&cert.x509v3_context(None, None))
        .unwrap();
    cert.append_extension(san).unwrap();
    cert.sign(&key, MessageDigest::sha256()).unwrap();
    let cert = cert.build().to_pem().unwrap();
    let cert_path = dir.path().join("tls.pem");
    let key_path = dir.path().join("tls-key.pem");
    std::fs::write(&cert_path, &cert).unwrap();
    riauth::config::write_private(&key_path, &key.private_key_to_pem_pkcs8().unwrap(), false)
        .unwrap();
    let mut config = f.core.config.clone();
    config.issuer = "https://localhost:9000".into();
    config.tls_cert_file = Some(cert_path);
    config.tls_key_file = Some(key_path);
    let tls = riauth::api::tls_configuration(&config)
        .await
        .unwrap()
        .unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = axum_server::Handle::new();
    let control = handle.clone();
    let app = riauth::api::router(f.core);
    let server = tokio::spawn(async move {
        axum_server::from_tcp_rustls(listener, tls)
            .unwrap()
            .handle(handle)
            .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
            .unwrap();
    });
    let url = format!("https://localhost:{}/healthz", addr.port());
    let trusted = reqwest::Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(&cert).unwrap())
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap();
    assert_eq!(
        trusted.get(&url).send().await.unwrap().status(),
        reqwest::StatusCode::OK
    );
    assert!(reqwest::Client::new().get(&url).send().await.is_err());
    control.graceful_shutdown(Some(std::time::Duration::from_secs(1)));
    server.await.unwrap();
    config.tls_key_file = None;
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn http_remembered_consent_prompt_none_is_an_httponly_cookie_redirect() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new();
    f.client("app", false);
    let app = riauth::api::router(f.core.clone());
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.decision = None;
    let encoded = serde_urlencoded::to_string(&request).unwrap();
    let pending = app
        .clone()
        .oneshot(
            Request::get(format!("/oauth/authorize?{encoded}"))
                .header("accept", "text/html")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // A browser is handed over to the interaction page; the terminal contract stays JSON.
    assert_eq!(pending.status(), StatusCode::SEE_OTHER);
    let return_cookie = cookie_value(pending.headers(), "riauth_return").to_owned();
    assert!(
        pending
            .headers()
            .get_all("set-cookie")
            .iter()
            .any(|value| value.to_str().unwrap().contains("HttpOnly"))
    );
    let location = pending.headers()["location"].to_str().unwrap().to_owned();
    let id = location
        .strip_prefix("http://localhost:9000/oauth/resume/")
        .unwrap()
        .to_owned();
    let waiting = app
        .clone()
        .oneshot(
            Request::get(format!("/oauth/resume/{id}"))
                .header("cookie", format!("riauth_return={return_cookie}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(waiting.status(), StatusCode::OK);
    assert_eq!(
        waiting.headers()["refresh"].to_str().unwrap(),
        format!("2; url=/oauth/resume/{id}")
    );
    let body: Value =
        serde_json::from_slice(&waiting.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let code = body["user_code"].as_str().unwrap();
    let details = app
        .clone()
        .oneshot(
            Request::get(format!("/api/authorization/{code}"))
                .header("authorization", format!("Bearer {}", f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(details.status(), StatusCode::OK);
    let details: Value =
        serde_json::from_slice(&details.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(details["reauthentication_required"], false);
    assert_eq!(details["select_account"], false);
    let decision = app
        .clone()
        .oneshot(
            Request::post("/api/authorization/decision")
                .header("authorization", format!("Bearer {}", f.admin))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"code": code, "approve": true, "remember": true}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(decision.status(), StatusCode::OK);
    let resumed = app
        .clone()
        .oneshot(
            Request::get(format!("/oauth/resume/{id}"))
                .header("cookie", format!("riauth_return={return_cookie}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resumed.status(), StatusCode::FOUND);
    let sso = cookie_value(resumed.headers(), "riauth_sso").to_owned();
    let sso_header = resumed
        .headers()
        .get_all("set-cookie")
        .iter()
        .find(|value| value.to_str().unwrap().starts_with("riauth_sso="))
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    assert!(sso_header.contains("HttpOnly"));
    let callback = url::Url::parse(resumed.headers()["location"].to_str().unwrap()).unwrap();
    let pairs: std::collections::BTreeMap<_, _> = callback.query_pairs().collect();
    assert_eq!(pairs["state"], "state with & delimiters");
    assert!(!pairs.contains_key("error"));
    let mut silent = f.request("app", &verifier);
    silent.decision = None;
    silent.prompt = Some("none".into());
    let silent_query = serde_urlencoded::to_string(&silent).unwrap();
    let silent = app
        .clone()
        .oneshot(
            Request::get(format!("/oauth/authorize?{silent_query}"))
                .header("accept", "text/html")
                .header("cookie", format!("riauth_sso={sso}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(silent.status(), StatusCode::FOUND);
    let callback = url::Url::parse(silent.headers()["location"].to_str().unwrap()).unwrap();
    let pairs: std::collections::BTreeMap<_, _> = callback.query_pairs().collect();
    assert_eq!(pairs["state"], "state with & delimiters");
    assert!(!pairs.contains_key("error"));
    let code = pairs["code"].to_string();
    let redeemed = app
        .clone()
        .oneshot(
            Request::post("/oauth/token")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    serde_urlencoded::to_string([
                        ("grant_type", "authorization_code"),
                        ("client_id", "app"),
                        ("code", &code),
                        ("redirect_uri", "http://localhost:7777/callback?existing=1"),
                        ("code_verifier", &verifier),
                    ])
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(redeemed.status(), StatusCode::OK);
    let anonymous = app
        .clone()
        .oneshot(
            Request::get(format!("/oauth/authorize?{silent_query}"))
                .header("accept", "text/html")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(anonymous.status(), StatusCode::FOUND);
    let callback = url::Url::parse(anonymous.headers()["location"].to_str().unwrap()).unwrap();
    let pairs: std::collections::BTreeMap<_, _> = callback.query_pairs().collect();
    assert_eq!(pairs["error"], "login_required");
    let revoked = app
        .clone()
        .oneshot(
            Request::delete("/api/consents/app")
                .header("authorization", format!("Bearer {}", f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(revoked.status(), StatusCode::OK);
    let after = app
        .oneshot(
            Request::get(format!("/oauth/authorize?{silent_query}"))
                .header("accept", "text/html")
                .header("cookie", format!("riauth_sso={sso}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(after.status(), StatusCode::FOUND);
    let callback = url::Url::parse(after.headers()["location"].to_str().unwrap()).unwrap();
    let pairs: std::collections::BTreeMap<_, _> = callback.query_pairs().collect();
    assert_eq!(pairs["error"], "consent_required");
}

fn cookie_value<'a>(headers: &'a axum::http::HeaderMap, name: &str) -> &'a str {
    headers
        .get_all("set-cookie")
        .iter()
        .find_map(|value| {
            let pair = value.to_str().unwrap().split_once(';').unwrap().0;
            let (key, cookie) = pair.split_once('=').unwrap();
            (key == name).then_some(cookie)
        })
        .unwrap()
}
