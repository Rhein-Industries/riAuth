//! HTTP rate-limit buckets, address grouping and configured overrides.
mod common;

use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use common::Fixture;
use riauth::{api::rate_key, config::Config};
use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
};
use tower::ServiceExt;

const CLIENT: &str = "198.51.100.7";
const START: &str = "/outpost/dashboard/start?rd=https%3A%2F%2Fdashboard.example.test%2F";

fn router(limits: &[(&str, u32)]) -> (Fixture, Router) {
    let mut f = Fixture::new();
    f.core.config.rate_limits = limits.iter().map(|(k, v)| ((*k).into(), *v)).collect();
    f.core.config.validate().unwrap();
    let router = riauth::api::router(f.core.clone());
    (f, router)
}
async fn status(router: &Router, method: &str, path: &str, ip: &str) -> StatusCode {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(ip.parse().unwrap(), 40000)));
    router.clone().oneshot(request).await.unwrap().status()
}
/// Sends `count` requests round-robin over `paths`, none of which may be limited.
async fn exhaust(router: &Router, paths: &[(&str, &str)], count: usize, ip: &str) {
    for n in 0..count {
        let (method, path) = paths[n % paths.len()];
        let status = status(router, method, path, ip).await;
        assert_ne!(
            status,
            StatusCode::TOO_MANY_REQUESTS,
            "{n}: {method} {path}"
        );
    }
}
async fn limited(router: &Router, method: &str, path: &str, ip: &str) -> bool {
    status(router, method, path, ip).await == StatusCode::TOO_MANY_REQUESTS
}

#[tokio::test]
async fn browser_login_endpoints_share_the_login_bucket() {
    let (_f, router) = router(&[]);
    let logins = [
        ("POST", "/api/login"),
        ("POST", "/api/portal/login/password"),
        ("POST", "/oauth/resume/interaction/password"),
        ("POST", "/saml/resume/interaction/password"),
    ];
    exhaust(&router, &logins, 20, CLIENT).await;
    for (method, path) in logins {
        assert!(limited(&router, method, path, CLIENT).await, "{path}");
    }
    for (method, path) in [
        ("POST", "/api/portal/login/passkey/start"),
        ("GET", "/api/portal/passkeys"),
        ("POST", "/oauth/resume/interaction/passkey/finish"),
        ("POST", "/oauth/resume/interaction/decision"),
        ("GET", "/oauth/resume/interaction/state"),
        ("GET", "/api/portal"),
    ] {
        assert!(!limited(&router, method, path, CLIENT).await, "{path}");
    }
    assert!(!limited(&router, "POST", "/api/login", "198.51.100.8").await);
}

#[tokio::test]
async fn interaction_state_has_its_own_bucket() {
    let (_f, router) = router(&[]);
    let states = [
        ("GET", "/oauth/resume/interaction/state"),
        ("GET", "/saml/resume/interaction/state"),
        ("GET", "/oauth/logout/resume/interaction/state"),
    ];
    // 1200 per minute: twice the general limit.
    exhaust(&router, &states, 1200, CLIENT).await;
    for (method, path) in states {
        assert!(limited(&router, method, path, CLIENT).await, "{path}");
    }
    let decisions = [
        ("POST", "/oauth/resume/interaction/decision"),
        ("POST", "/saml/resume/interaction/decision"),
        ("POST", "/oauth/logout/resume/interaction/decision"),
    ];
    exhaust(&router, &decisions, 60, CLIENT).await;
    assert!(
        limited(
            &router,
            "POST",
            "/oauth/resume/interaction/decision",
            CLIENT
        )
        .await
    );
    for (method, path) in [
        ("GET", "/oauth/resume/interaction"),
        ("GET", "/saml/resume/interaction"),
        ("GET", "/oauth/logout/resume/interaction"),
        ("POST", "/saml/resume/interaction/passkey/start"),
    ] {
        assert!(!limited(&router, method, path, CLIENT).await, "{path}");
    }
}

#[tokio::test]
async fn forward_auth_has_its_own_bucket() {
    let (_f, router) = router(&[]);
    let forward = [
        ("GET", "/outpost/dashboard/auth"),
        ("GET", "/outpost/dashboard/traefik"),
    ];
    // 6000 per minute: ten times the general limit.
    exhaust(&router, &forward, 6000, CLIENT).await;
    for (method, path) in forward {
        assert!(limited(&router, method, path, CLIENT).await, "{path}");
    }
    for (method, path) in [
        ("GET", "/outpost/dashboard/callback"),
        ("POST", "/outpost/dashboard/logout"),
        ("GET", START),
        ("GET", "/.well-known/openid-configuration"),
    ] {
        assert!(!limited(&router, method, path, CLIENT).await, "{path}");
    }
}

#[tokio::test]
async fn outpost_start_is_limited_to_30_per_minute() {
    let (_f, router) = router(&[]);
    exhaust(&router, &[("GET", START)], 30, CLIENT).await;
    assert!(limited(&router, "GET", START, CLIENT).await);
    assert!(limited(&router, "GET", "/outpost/other/start", CLIENT).await);
    for (method, path) in [
        ("GET", "/outpost/dashboard/callback"),
        ("GET", "/outpost/dashboard/auth"),
        ("GET", "/outpost/dashboard/traefik"),
    ] {
        assert!(!limited(&router, method, path, CLIENT).await, "{path}");
    }
    assert!(!limited(&router, "GET", START, "198.51.100.8").await);
}

#[tokio::test]
async fn ipv6_addresses_in_one_64_share_a_bucket() {
    let ip = |value: &str| value.parse::<IpAddr>().unwrap();
    assert_eq!(
        rate_key(ip("2001:db8:1:2:aaaa:bbbb:cccc:dddd")),
        ip("2001:db8:1:2::")
    );
    assert_eq!(rate_key(ip("::ffff:198.51.100.7")), ip(CLIENT));
    assert_eq!(rate_key(ip(CLIENT)), ip(CLIENT));
    assert_eq!(rate_key(ip("::1")), ip("::"));
    let (_f, router) = router(&[]);
    for n in 0..30 {
        let address = format!("2001:db8:1:2:{n:x}::1");
        assert!(!limited(&router, "GET", START, &address).await, "{address}");
    }
    assert!(limited(&router, "GET", START, "2001:db8:1:2:ffff:ffff:ffff:ffff").await);
    assert!(!limited(&router, "GET", START, "2001:db8:1:3::1").await);
    // An IPv4-mapped address counts as the IPv4 address.
    exhaust(&router, &[("GET", START)], 30, "::ffff:198.51.100.7").await;
    assert!(limited(&router, "GET", START, CLIENT).await);
    assert!(!limited(&router, "GET", START, "198.51.100.8").await);
}

#[tokio::test]
async fn configured_rate_limit_overrides_apply() {
    let (_f, router) = router(&[("outpost_start", 2), ("general", 3), ("forward_auth", 4)]);
    exhaust(&router, &[("GET", START)], 2, CLIENT).await;
    assert!(limited(&router, "GET", START, CLIENT).await);
    exhaust(
        &router,
        &[("GET", "/outpost/dashboard/callback")],
        3,
        CLIENT,
    )
    .await;
    assert!(limited(&router, "GET", "/api/portal", CLIENT).await);
    exhaust(&router, &[("GET", "/outpost/dashboard/traefik")], 4, CLIENT).await;
    assert!(limited(&router, "GET", "/outpost/dashboard/auth", CLIENT).await);
    // Categories without an override keep their defaults.
    exhaust(&router, &[("POST", "/api/login")], 20, CLIENT).await;
    assert!(limited(&router, "POST", "/api/login", CLIENT).await);
    let loaded: Config = toml::from_str(&format!(
        "{BASE}[rate_limits]\nlogin=5\nbrowser_state=6000\n"
    ))
    .unwrap();
    loaded.validate().unwrap();
    assert_eq!(
        loaded.rate_limits,
        BTreeMap::from([("browser_state".into(), 6000), ("login".into(), 5)])
    );
    let written = toml::to_string(&loaded).unwrap();
    assert_eq!(
        toml::from_str::<Config>(&written).unwrap().rate_limits,
        loaded.rate_limits
    );
    // Without overrides the written configuration is unchanged.
    assert!(
        !toml::to_string(&Config::default())
            .unwrap()
            .contains("rate_limits")
    );
}
const BASE: &str = "issuer='http://127.0.0.1:9000'\nlisten='127.0.0.1:9000'\ndata_dir='data'\naccess_token_ttl=300\nrefresh_token_ttl=2592000\nsession_ttl=28800\n";

#[test]
fn config_rejects_unknown_rate_limit_categories() {
    let categories = [
        "portal_start",
        "portal_approve",
        "login",
        "passkey",
        "account",
        "source_start",
        "source_callback",
        "saml",
        "mfa",
        "device_start",
        "device_verify",
        "browser_decision",
        "browser_state",
        "forward_auth",
        "outpost_start",
        "general",
    ];
    let config = |category: &str, limit: u32| Config {
        rate_limits: BTreeMap::from([(category.into(), limit)]),
        ..Default::default()
    };
    for category in categories {
        for limit in [1, 100_000] {
            assert!(config(category, limit).validate().is_ok(), "{category}");
        }
        for limit in [0, 100_001] {
            assert!(config(category, limit).validate().is_err(), "{category}");
        }
    }
    for category in ["logins", "Login", "", "forward-auth", "workers"] {
        let error = config(category, 10).validate().unwrap_err().to_string();
        assert!(error.contains("rate limit"), "{error}");
    }
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("riauth.toml");
    for (overrides, loads) in [
        ("login=5\n", true),
        ("logins=5\n", false),
        ("login=0\n", false),
    ] {
        std::fs::write(&path, format!("{BASE}[rate_limits]\n{overrides}")).unwrap();
        assert_eq!(Config::load(&path).is_ok(), loads, "{overrides}");
    }
    assert!(toml::from_str::<Config>(&format!("{BASE}[rate_limits]\nlogin=-1\n")).is_err());
}

/// Ten thousand addresses, each far below its own limit, used to fill the counter table;
/// clients the table had not seen were then refused everywhere. Now they are served.
#[tokio::test]
async fn many_addresses_cannot_lock_new_clients_out() {
    let (_f, router) = router(&[]);
    for n in 0..10_000u32 {
        let address = format!("2001:db8:{:x}:{:x}::1", n >> 16, n & 0xffff);
        assert!(
            !limited(&router, "GET", "/outpost/dashboard/traefik", &address).await,
            "{address}"
        );
    }
    let fresh = "198.51.100.77";
    assert!(!limited(&router, "GET", "/outpost/dashboard/traefik", fresh).await);
    assert!(!limited(&router, "POST", "/api/portal/login/password", fresh).await);
    assert!(!limited(&router, "GET", "/.well-known/openid-configuration", fresh).await);
}

/// A full table drops its oldest window. The evicted client starts a new window; nobody
/// is refused for the table being full.
#[test]
fn a_full_rate_table_evicts_its_oldest_window() {
    use riauth::api::RateTable;
    use std::time::{Duration, Instant};
    let ip = |n: u8| IpAddr::from([198, 51, 100, n]);
    let mut table = RateTable::new(3);
    let start = Instant::now();
    assert!(!table.hit((ip(1), "login"), 1, start));
    assert!(table.hit((ip(1), "login"), 1, start), "over its own limit");
    for n in 2..=3 {
        assert!(!table.hit((ip(n), "login"), 1, start + Duration::from_secs(1)));
    }
    assert_eq!(table.len(), 3);
    // A fourth client fits by evicting the oldest window, which was ip(1)'s.
    assert!(!table.hit((ip(4), "login"), 1, start + Duration::from_secs(2)));
    assert_eq!(table.len(), 3);
    assert!(!table.hit((ip(1), "login"), 1, start + Duration::from_secs(3)));
    // Windows still end after 60 s, and repeat hits keep counting within one.
    assert!(table.hit((ip(1), "login"), 1, start + Duration::from_secs(4)));
    assert!(!table.hit((ip(1), "login"), 1, start + Duration::from_secs(64)));
    assert_eq!(table.len(), 1, "every other window expired");
}

/// The shared (PostgreSQL) counter table behaves the same way when full.
#[test]
fn a_full_shared_rate_table_evicts_its_oldest_window() {
    let f = Fixture::new();
    let ip = |n: u8| IpAddr::from([198, 51, 100, n]);
    let hit = |n: u8| {
        f.core
            .store
            .shared_rate_limit_within(ip(n), "login", 1, 2)
            .unwrap()
    };
    assert!(!hit(1));
    assert!(hit(1), "over its own limit");
    assert!(!hit(2));
    assert!(!hit(3), "a new client is counted, not refused");
    assert!(hit(3), "and its window keeps counting");
    let count = f
        .core
        .store
        .read(|tx| tx.collection_count("http_rates"))
        .unwrap();
    assert_eq!(count, 2, "one of the two older windows was evicted");
}
