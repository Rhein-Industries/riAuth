use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use riauth::{
    config::Config,
    core::Core,
    crypto::{self, digest, now},
    model::*,
    oidc::{Authorization, DEVICE_GRANT, TokenRequest},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::Barrier};
use tempfile::TempDir;

mod common;
use common::{Fixture, PASSWORD, strings, text};

fn age_session_and_grant_timestamps(core: &Core, seconds: u64) {
    core.store
        .write(|tx| {
            for (key, mut session) in tx.list::<Session>("sessions")? {
                session.expires_at = session.expires_at.saturating_sub(seconds);
                session.identity.auth_time = session.identity.auth_time.saturating_sub(seconds);
                tx.put("sessions", &key, &session)?;
            }
            for bucket in ["access", "refresh"] {
                for (key, mut grant) in tx.list::<Grant>(bucket)? {
                    grant.issued_at = grant.issued_at.saturating_sub(seconds);
                    grant.expires_at = grant.expires_at.saturating_sub(seconds);
                    if let Some(identity) = &mut grant.identity {
                        identity.auth_time = identity.auth_time.saturating_sub(seconds);
                    }
                    tx.put(bucket, &key, &grant)?;
                }
            }
            for (key, mut family) in tx.list::<Family>("families")? {
                family.expires_at = family.expires_at.saturating_sub(seconds);
                tx.put("families", &key, &family)?;
            }
            Ok(())
        })
        .unwrap();
}

fn managed_manifest(client_id: &str) -> riauth::state::Manifest {
    serde_json::from_value(json!({"api_version": "riauth/v1", "clients": [{"client_id": client_id, "name": "Managed application", "scopes": ["openid", "profile"], "redirect_uris": ["https://app.example.test/callback"]}]})).unwrap()
}
fn agent_token(f: &Fixture, permissions: &[(&str, &str)]) -> String {
    let created = f
        .core
        .create_agent(
            &f.admin,
            riauth::agent::NewAgent {
                id: crypto::id(),
                ttl: 3600,
                parent: None,
                permissions: permissions
                    .iter()
                    .map(|(a, r)| riauth::agent::Permission {
                        action: (*a).into(),
                        resource: (*r).into(),
                    })
                    .collect(),
            },
        )
        .unwrap();
    text(&created["credential"], "token")
}

fn fixture_signing_key(f: &Fixture) -> crypto::SigningKey {
    f.core
        .store
        .get::<crypto::Keys>("meta", "keys")
        .unwrap()
        .unwrap()
        .active
}
fn fixture_jwks(f: &Fixture) -> riauth::jose::PublicJwks {
    serde_json::from_value(f.core.jwks().unwrap()).unwrap()
}
fn service_client(f: &Fixture, cid: &str, settings: ProviderSettings) -> Option<String> {
    let out = f
        .core
        .create_client(
            &f.admin,
            NewClient {
                client_id: cid.into(),
                name: cid.into(),
                confidential: true,
                service: true,
                redirect_uris: vec![],
                scopes: strings(&["api.read", "api.write"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                settings,
            },
        )
        .unwrap();
    out["client_secret"].as_str().map(String::from)
}

fn alternate_client_keys() -> (crypto::SigningKey, riauth::jose::PublicJwks) {
    let key = crypto::SigningKey::generate().unwrap();
    let jwk = serde_json::from_value(key.jwk().unwrap()).unwrap();
    (key, riauth::jose::PublicJwks { keys: vec![jwk] })
}

fn dpop_proof(
    key: &crypto::SigningKey,
    method: &str,
    endpoint: &str,
    token: Option<&str>,
) -> String {
    let mut header = jsonwebtoken::Header::new(Algorithm::ES256);
    header.typ = Some("dpop+jwt".into());
    header.jwk = Some(serde_json::from_value(key.jwk().unwrap()).unwrap());
    let mut claims = json!({"jti":crypto::id(),"iat":now(),"htm":method,"htu":endpoint});
    if let Some(token) = token {
        claims["ath"] = json!(digest(token));
    }
    jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_ec_pem(key.pem.as_bytes()).unwrap(),
    )
    .unwrap()
}

type UpstreamCodes =
    std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, (String, Value)>>>;
struct Upstream {
    source: riauth::source::Source,
    key: crypto::SigningKey,
    codes: UpstreamCodes,
    server: tokio::task::JoinHandle<()>,
}
impl Drop for Upstream {
    fn drop(&mut self) {
        self.server.abort();
    }
}
impl Upstream {
    async fn new(f: &Fixture) -> Self {
        use axum::{Form, Json, Router, routing::post};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let issuer = format!("http://{}", listener.local_addr().unwrap());
        let codes: UpstreamCodes = Default::default();
        let records = codes.clone();
        let app = Router::new().route(
            "/token",
            post(
                move |headers: axum::http::HeaderMap,
                      Form(form): Form<std::collections::HashMap<String, String>>| {
                    let records = records.clone();
                    async move {
                        use base64::engine::general_purpose::STANDARD;
                        assert_eq!(
                            headers["authorization"],
                            format!(
                                "Basic {}",
                                STANDARD.encode("upstream-client:source-client-secret")
                            )
                        );
                        assert_eq!(form["grant_type"], "authorization_code");
                        assert_eq!(
                            form["redirect_uri"],
                            "http://localhost:9000/oauth/sources/upstream/callback"
                        );
                        let (challenge, tokens) =
                            records.lock().unwrap().remove(&form["code"]).unwrap();
                        assert_eq!(digest(&form["code_verifier"]), challenge);
                        Json(tokens)
                    }
                },
            ),
        );
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let key = fixture_signing_key(f);
        let source = riauth::source::Source {
            saml: None,
            oauth_profile: None,
            id: "upstream".into(),
            name: "Example upstream".into(),
            issuer: issuer.clone(),
            authorization_endpoint: format!("{issuer}/authorize"),
            token_endpoint: format!("{issuer}/token"),
            client_id: "upstream-client".into(),
            token_endpoint_auth_method: riauth::jose::ClientAuthMethod::ClientSecretBasic,
            jwks: fixture_jwks(f),
            scopes: strings(&["openid", "profile", "email"]),
            enabled: true,
            auto_provision: true,
            groups: Default::default(),
            trusted_mfa_acr: strings(&["urn:upstream:mfa"]),
            allow_admin_login: false,
        };
        f.core
            .source_put(
                &f.admin,
                riauth::source::SourceInput {
                    source: source.clone(),
                    client_secret: Some("source-client-secret".into()),
                },
            )
            .unwrap();
        Self {
            source,
            key,
            codes,
            server,
        }
    }
    fn start(&self, f: &Fixture, link: Option<&str>) -> Value {
        f.core
            .source_start(
                &self.source.id,
                riauth::source::Start {
                    link: link.is_some(),
                    authentication_transaction: None,
                },
                link,
            )
            .unwrap()
    }
    async fn callback(
        &self,
        f: &Fixture,
        start: &Value,
        subject: &str,
        override_claims: Value,
    ) -> Value {
        let url = url::Url::parse(start["authorization_url"].as_str().unwrap()).unwrap();
        let p: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(p["max_age"], "0");
        assert_eq!(p["code_challenge_method"], "S256");
        let mut claims = json!({"iss":self.source.issuer,"sub":subject,"aud":"upstream-client","iat":now(),"exp":now()+300,"auth_time":now(),"nonce":p["nonce"],"email":"alice@example.test","email_verified":true,"name":"Upstream Alice","acr":"urn:upstream:mfa"});
        claims
            .as_object_mut()
            .unwrap()
            .extend(override_claims.as_object().unwrap().clone());
        let code = crypto::random_token("");
        self.codes.lock().unwrap().insert(code.clone(),(p["code_challenge"].clone(),json!({"id_token":self.key.sign(&claims,false).unwrap(),"access_token":"mock-access"})));
        f.core
            .source_callback(
                &self.source.id,
                vec![
                    ("state".into(), p["state"].clone()),
                    ("code".into(), code),
                    ("iss".into(), self.source.issuer.clone()),
                ],
            )
            .await
            .unwrap()
    }
    fn finish(&self, f: &Fixture, start: &Value, approve: bool) -> riauth::error::Result<Value> {
        f.core.source_finish(riauth::source::Finish {
            credential: text(&start["credential"], "token"),
            approve,
            otp: None,
        })
    }
}

fn mail_config(port: u16) -> riauth::lifecycle::MailConfig {
    riauth::lifecycle::MailConfig {
        host: "127.0.0.1".into(),
        port,
        from: "Identity <identity@example.test>".into(),
        security: riauth::lifecycle::MailSecurity::Loopback,
        username: None,
        password_file: None,
    }
}
fn mail_code(core: &Core) -> String {
    let mut deliveries = core.store.list::<Value>("mail_deliveries").unwrap();
    deliveries.sort_by_key(|(_, d)| d["created_at"].as_u64().unwrap());
    deliveries
        .iter()
        .rev()
        .filter(|(_, d)| {
            core.store
                .get::<Value>("account_proofs", d["proof"].as_str().unwrap())
                .unwrap()
                .is_some()
        })
        .find_map(|(_, d)| {
            d["body"]
                .as_str()
                .and_then(|b| b.lines().find(|line| line.starts_with("ri_mail_")))
                .map(String::from)
        })
        .unwrap()
}
#[path = "identity/saml.rs"]
mod saml_tests;

#[path = "identity/radius_eap.rs"]
mod radius_eap_tests;

#[path = "identity/saml_source.rs"]
mod saml_source_tests;

#[path = "identity/saml_logout.rs"]
mod saml_logout_tests;

#[path = "identity/oidc.rs"]
mod oidc_tests;

#[path = "identity/policy.rs"]
mod policy_tests;

#[path = "identity/network.rs"]
mod network_tests;

#[path = "identity/factors.rs"]
mod factors_tests;

#[path = "identity/operations.rs"]
mod operations_tests;

#[path = "identity/http.rs"]
mod http_tests;

#[path = "identity/sources.rs"]
mod sources_tests;
