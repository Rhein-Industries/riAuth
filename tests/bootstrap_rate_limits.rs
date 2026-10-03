use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use riauth::{
    bootstrap::{Bootstrap, router},
    config::Config,
};
use serde_json::{Value, json};
use std::net::{IpAddr, SocketAddr};
use tower::ServiceExt;

struct Fixture {
    _dir: tempfile::TempDir,
    routes: Router,
    proof: String,
}
impl Fixture {
    fn new(trusted: bool) -> Self {
        let dir = tempfile::TempDir::new().unwrap();
        let config = Config {
            issuer: "http://localhost:9000/identity".into(),
            data_dir: dir.path().join("data"),
            trusted_proxies: if trusted {
                vec![IpAddr::from([127, 0, 0, 1])]
            } else {
                vec![]
            },
            rate_limits: [("account".into(), 2)].into(),
            ..Default::default()
        };
        let proof_file = dir.path().join("ownership-proof");
        Bootstrap::prepare(config.clone(), &proof_file, 900).unwrap();
        Self {
            proof: std::fs::read_to_string(proof_file).unwrap(),
            routes: router(Bootstrap::open(config).unwrap()),
            _dir: dir,
        }
    }
    async fn post(&self, path: &str, forwarded: &[&str], value: Value) -> StatusCode {
        let mut request = Request::builder()
            .method("POST")
            .uri(format!("/identity/api/setup{path}"))
            .header("host", "localhost:9000")
            .header("origin", "http://localhost:9000")
            .header("x-riauth-portal", "1")
            .header("sec-fetch-site", "same-origin")
            .header("content-type", "application/json");
        for value in forwarded {
            request = request.header("x-forwarded-for", *value);
        }
        let mut request = request
            .body(Body::from(serde_json::to_vec(&value).unwrap()))
            .unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))));
        self.routes.clone().oneshot(request).await.unwrap().status()
    }
    fn input(&self, valid: bool) -> Value {
        json!({"proof": if valid { &self.proof } else { "invalid-proof" },
            "username":"admin", "password":"bootstrap-rate-test-password",
            "display_name":"Administrator", "email":null})
    }
}

#[tokio::test]
async fn trusted_proxy_setup_attempts_are_counted_for_each_client() {
    let f = Fixture::new(true);
    for _ in 0..2 {
        assert_eq!(
            f.post("", &["192.0.2.1"], f.input(false)).await,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        f.post("", &["192.0.2.1"], f.input(true)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        f.post("", &["192.0.2.2"], f.input(true)).await,
        StatusCode::OK
    );
}

#[tokio::test]
async fn untrusted_forwarding_cannot_reset_setup_limit() {
    let f = Fixture::new(false);
    for ip in ["192.0.2.1", "192.0.2.2"] {
        assert_eq!(
            f.post("", &[ip], f.input(false)).await,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        f.post("", &["192.0.2.3"], f.input(true)).await,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn passkey_setup_uses_validated_forwarding_and_ipv6_rate_groups() {
    let f = Fixture::new(true);
    let invalid = json!({"proof":"invalid-proof", "ceremony":"invalid-ceremony"});
    assert_eq!(
        f.post("/passkey/cancel", &["not-an-address"], invalid.clone())
            .await,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        f.post(
            "/passkey/cancel",
            &["192.0.2.1", "192.0.2.2"],
            invalid.clone()
        )
        .await,
        StatusCode::BAD_REQUEST
    );
    for ip in ["2001:db8:1::1", "2001:db8:1::2"] {
        assert_eq!(
            f.post("/passkey/cancel", &[ip], invalid.clone()).await,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        f.post("/passkey/cancel", &["2001:db8:1::3"], invalid.clone())
            .await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        f.post("/passkey/cancel", &["2001:db8:2::1"], invalid).await,
        StatusCode::UNAUTHORIZED
    );
}
