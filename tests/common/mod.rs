#![allow(dead_code)]

pub mod security;

use riauth::{
    config::Config,
    core::Core,
    crypto::{self, digest},
    model::*,
    oidc::{Authorization, TokenRequest},
};
use serde_json::Value;
use std::{collections::BTreeSet, sync::OnceLock};
use tempfile::TempDir;

pub const PASSWORD: &str = "test-password-for-fixtures-only";
pub struct Fixture {
    pub _dir: TempDir,
    pub core: Core,
    pub admin: String,
}
pub fn strings(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}
pub fn text(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap().into()
}

impl Fixture {
    pub fn new() -> Self {
        static TEMPLATE: OnceLock<TempDir> = OnceLock::new();
        let template = TEMPLATE.get_or_init(|| {
            let dir = TempDir::new().unwrap();
            let config = Config {
                data_dir: dir.path().into(),
                ..Default::default()
            };
            let core = Core::initialize(
                config,
                NewUser {
                    username: "admin".into(),
                    password: PASSWORD.into(),
                    email: None,
                    display_name: "Administrator".into(),
                    admin: true,
                },
            )
            .unwrap();
            drop(core);
            dir
        });
        let dir = TempDir::new().unwrap();
        std::fs::copy(
            template.path().join("riauth.redb"),
            dir.path().join("riauth.redb"),
        )
        .unwrap();
        let config = Config {
            data_dir: dir.path().into(),
            ..Default::default()
        };
        let core = Core::open(config).unwrap();
        let admin = text(
            &core.login("admin".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        );
        Self {
            _dir: dir,
            core,
            admin,
        }
    }
    pub fn client(&self, cid: &str, confidential: bool) -> Option<String> {
        let output = self
            .core
            .create_client(
                &self.admin,
                NewClient {
                    client_id: cid.into(),
                    name: cid.into(),
                    confidential,
                    redirect_uris: vec!["http://localhost:7777/callback?existing=1".into()],
                    scopes: strings(&["openid", "profile", "email", "groups", "offline_access"]),
                    allowed_groups: BTreeSet::new(),
                    require_mfa: false,
                    service: false,
                    settings: Default::default(),
                },
            )
            .unwrap();
        output["client_secret"].as_str().map(String::from)
    }
    pub fn user(&self, username: &str) -> String {
        self.core
            .create_user(
                &self.admin,
                NewUser {
                    username: username.into(),
                    password: PASSWORD.into(),
                    email: Some(format!("{username}@example.test")),
                    display_name: "Test User".into(),
                    admin: false,
                },
            )
            .unwrap();
        text(
            &self
                .core
                .login(username.into(), PASSWORD.into(), None)
                .unwrap(),
            "session_token",
        )
    }
    pub fn request(&self, cid: &str, verifier: &str) -> Authorization {
        Authorization {
            response_type: "code".into(),
            client_id: cid.into(),
            redirect_uri: "http://localhost:7777/callback?existing=1".into(),
            scope: "openid profile email groups offline_access".into(),
            state: Some("state with & delimiters".into()),
            nonce: Some("expected-nonce".into()),
            code_challenge: digest(verifier),
            code_challenge_method: "S256".into(),
            prompt: None,
            max_age: None,
            response_mode: None,
            decision: Some("approve".into()),
            transaction_id: None,
            request_binding: None,
            ..Default::default()
        }
    }
    pub fn exchange_request(&self, cid: &str, token: &str, secret: Option<String>) -> TokenRequest {
        let verifier = crypto::random_token("");
        let redirect = self
            .core
            .authorize(token, self.request(cid, &verifier))
            .unwrap();
        let url = url::Url::parse(&redirect).unwrap();
        let params: std::collections::HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(params["state"], "state with & delimiters");
        let client: Client = self.core.store.get("clients", cid).unwrap().unwrap();
        assert_eq!(
            params["iss"],
            riauth::issuer::for_client(&self.core.config.issuer, &client)
        );
        assert_eq!(params["existing"], "1");
        TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some(cid.into()),
            client_secret: secret,
            code: Some(params["code"].to_string()),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        }
    }
    pub fn tokens(&self, cid: &str, session: &str, secret: Option<String>) -> Value {
        self.core
            .token(self.exchange_request(cid, session, secret))
            .unwrap()
    }
}
