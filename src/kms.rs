//! Vault Transit signing with version-pinned public keys and verification of every result.
use crate::{
    core::Core,
    crypto::{SigningKey, now},
    error::{Error, Result},
    jose::PublicJwk,
};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{io::Read, path::PathBuf, time::Duration};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultSigner {
    pub address: String,
    #[serde(default = "transit")]
    pub mount: String,
    pub key_name: String,
    pub key_version: u32,
    pub public_jwk: PublicJwk,
    pub token_file: PathBuf,
    pub ca_file: Option<PathBuf>,
    pub namespace: Option<String>,
}
fn transit() -> String {
    "transit".into()
}
#[derive(Clone, Serialize, Deserialize)]
pub struct RemoteKey {
    pub signer: String,
    pub key_version: u32,
    pub public_jwk: PublicJwk,
}
impl VaultSigner {
    pub fn validate(&self) -> Result<()> {
        crate::config::validate_server_url(&self.address)
            .map_err(|_| Error::bad("Vault address must use HTTPS or HTTP loopback"))?;
        crate::core::validate_name(&self.mount)?;
        crate::core::validate_name(&self.key_name)?;
        self.public_jwk.validate()?;
        if self.key_version == 0
            || self
                .namespace
                .as_ref()
                .is_some_and(|n| n.is_empty() || n.len() > 256 || n.chars().any(char::is_control))
        {
            return Err(Error::bad(
                "Vault requires an explicit key_version and valid namespace",
            ));
        }
        Ok(())
    }
    fn sign(&self, input: &str) -> Result<Vec<u8>> {
        self.validate()?;
        let token = crate::config::read_private_secret(&self.token_file, 4096).map_err(|_| {
            Error::bad("Vault credential must be a private file of at most 4096 bytes")
        })?;
        let token = token.trim();
        if token.is_empty() || token.len() > 4096 || !token.bytes().all(|c| c.is_ascii_graphic()) {
            return Err(Error::bad("Invalid Vault credential file"));
        }
        let mut http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .redirect(reqwest::redirect::Policy::none());
        if let Some(path) = &self.ca_file {
            http = http.add_root_certificate(
                reqwest::Certificate::from_pem(&std::fs::read(path).map_err(Error::internal)?)
                    .map_err(Error::internal)?,
            );
        }
        let http = http.build().map_err(|_| unavailable())?;
        let uri = format!(
            "{}/v1/{}/sign/{}",
            self.address.trim_end_matches('/'),
            self.mount,
            self.key_name
        );
        let mut request=http.post(uri).header("x-vault-token",token).json(&json!({"input":STANDARD.encode(input),"key_version":self.key_version,"hash_algorithm":"sha2-256","prehashed":false,"signature_algorithm":"pkcs1v15","marshaling_algorithm":"jws"}));
        if let Some(namespace) = &self.namespace {
            request = request.header("x-vault-namespace", namespace);
        }
        let response = request.send().map_err(|_| unavailable())?;
        if !response.status().is_success() || response.content_length().is_some_and(|n| n > 65_536)
        {
            return Err(unavailable());
        }
        let mut bytes = zeroize::Zeroizing::new(Vec::new());
        response
            .take(65_537)
            .read_to_end(&mut bytes)
            .map_err(|_| unavailable())?;
        if bytes.len() > 65_536 {
            return Err(unavailable());
        }
        let value: Value = serde_json::from_slice(&bytes).map_err(|_| unavailable())?;
        let signature = value["data"]["signature"]
            .as_str()
            .and_then(|s| s.strip_prefix(&format!("vault:v{}:", self.key_version)))
            .ok_or_else(unavailable)?;
        // Vault's jws ECDSA format uses URL-safe Base64; RSA/Ed25519 use standard Base64.
        let signature = STANDARD
            .decode(signature)
            .or_else(|_| URL_SAFE.decode(signature))
            .or_else(|_| URL_SAFE_NO_PAD.decode(signature))
            .map_err(|_| unavailable())?;
        Ok(signature)
    }
}
fn unavailable() -> Error {
    Error::new(
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "signer_unavailable",
        "Configured signing service failed; no token was issued",
    )
}
impl Core {
    pub(crate) fn sign_jwt(&self, key: &SigningKey, claims: &Value, typ: &str) -> Result<String> {
        let _timer = self.store.telemetry().signing.timer();
        let result = self.sign_jwt_inner(key, claims, typ);
        if result.is_err() {
            self.store
                .telemetry()
                .signing_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        result
    }
    fn sign_jwt_inner(&self, key: &SigningKey, claims: &Value, typ: &str) -> Result<String> {
        let Some(remote) = &key.remote else {
            return key.sign_type(claims, typ);
        };
        key.jwk()?;
        let config = self
            .config
            .signers
            .get(&remote.signer)
            .filter(|s| s.key_version == remote.key_version && s.public_jwk == remote.public_jwk)
            .ok_or_else(unavailable)?;
        let protected = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&json!({"alg":key.algorithm,"kid":key.kid,"typ":typ}))
                .map_err(Error::internal)?,
        );
        let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).map_err(Error::internal)?);
        let input = format!("{protected}.{payload}");
        let signature = config.sign(&input)?;
        let jwt = format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature));
        // Verification checks the returned signature against our public pin, independent of Vault.
        let mut validation = jsonwebtoken::Validation::new(remote.public_jwk.algorithm()?);
        validation.required_spec_claims.clear();
        validation.validate_exp = false;
        validation.validate_aud = false;
        validation.validate_nbf = false;
        let verified =
            jsonwebtoken::decode::<Value>(&jwt, &remote.public_jwk.decoding_key()?, &validation)
                .map_err(|_| unavailable())?;
        if verified.claims != *claims {
            return Err(unavailable());
        }
        Ok(jwt)
    }
    pub(crate) fn external_signing_key(&self, name: &str) -> Result<SigningKey> {
        let signer = self
            .config
            .signers
            .get(name)
            .ok_or_else(|| Error::bad("Unknown configured external signer"))?;
        signer.validate()?;
        let key = SigningKey {
            algorithm: signer.public_jwk.alg.clone(),
            kid: signer.public_jwk.kid.clone(),
            pem: String::new(),
            created_at: now(),
            remote: Some(RemoteKey {
                signer: name.into(),
                key_version: signer.key_version,
                public_jwk: signer.public_jwk.clone(),
            }),
        };
        self.sign_jwt(&key,&json!({"iss":"riauth-signer-check","aud":"riauth-signer-check","exp":now()+30,"jti":crate::crypto::random_token("")}),"JWT")?;
        Ok(key)
    }
}
