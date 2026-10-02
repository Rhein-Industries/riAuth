//! Vault Transit signing with version-pinned public keys and verification of every result.
pub use crate::kms_types::{RemoteKey, VaultSigner};
use crate::{
    core::Core,
    crypto::{SigningKey, now},
    error::{Error, Result},
    telemetry::RemoteSigningFailure as Reason,
};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD},
};
use serde_json::{Value, json};
use std::{io::Read, time::Duration};

/// A remote signing failure: its fixed telemetry label and the unchanged public error.
type Failed = (Reason, Error);

fn because(reason: Reason) -> impl FnOnce(Error) -> Failed {
    move |error| (reason, error)
}

/// A Transit `vault:v<digits>:` signature prefix. The caller already found that
/// it is not the pinned version's prefix.
fn other_version(signature: &str) -> bool {
    signature
        .strip_prefix("vault:v")
        .and_then(|rest| rest.split_once(':'))
        .is_some_and(|(digits, _)| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
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
    fn sign(&self, input: &str) -> std::result::Result<Vec<u8>, Failed> {
        self.validate()
            .map_err(because(Reason::SignerConfiguration))?;
        let token = crate::config::read_private_secret(&self.token_file, 4096).map_err(|_| {
            (
                Reason::CredentialRead,
                Error::bad("Vault credential must be a private file of at most 4096 bytes"),
            )
        })?;
        let token = token.trim();
        if token.is_empty() || token.len() > 4096 || !token.bytes().all(|c| c.is_ascii_graphic()) {
            return Err((
                Reason::CredentialShape,
                Error::bad("Invalid Vault credential file"),
            ));
        }
        let mut http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .redirect(reqwest::redirect::Policy::none());
        if let Some(path) = &self.ca_file {
            http = http.add_root_certificate(
                reqwest::Certificate::from_pem(
                    &std::fs::read(path)
                        .map_err(Error::internal)
                        .map_err(because(Reason::CaSetup))?,
                )
                .map_err(Error::internal)
                .map_err(because(Reason::CaSetup))?,
            );
        }
        let http = http
            .build()
            .map_err(|_| (Reason::ClientSetup, unavailable()))?;
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
        let response = request
            .send()
            .map_err(|_| (Reason::Transport, unavailable()))?;
        let status = response.status();
        if !status.is_success() {
            return Err((Reason::status(status), unavailable()));
        }
        if response.content_length().is_some_and(|n| n > 65_536) {
            return Err((Reason::ResponseSize, unavailable()));
        }
        let mut bytes = zeroize::Zeroizing::new(Vec::new());
        response
            .take(65_537)
            .read_to_end(&mut bytes)
            .map_err(|_| (Reason::Transport, unavailable()))?;
        if bytes.len() > 65_536 {
            return Err((Reason::ResponseSize, unavailable()));
        }
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|_| (Reason::ResponseShape, unavailable()))?;
        let signature = value["data"]["signature"]
            .as_str()
            .ok_or_else(|| (Reason::ResponseShape, unavailable()))?;
        let signature = signature
            .strip_prefix(&format!("vault:v{}:", self.key_version))
            .ok_or_else(|| {
                let reason = if other_version(signature) {
                    Reason::ResponseVersion
                } else {
                    Reason::ResponseShape
                };
                (reason, unavailable())
            })?;
        // Vault's jws ECDSA format uses URL-safe Base64; RSA/Ed25519 use standard Base64.
        let signature = STANDARD
            .decode(signature)
            .or_else(|_| URL_SAFE.decode(signature))
            .or_else(|_| URL_SAFE_NO_PAD.decode(signature))
            .map_err(|_| (Reason::ResponseShape, unavailable()))?;
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
        self.sign_jwt_inner(key, claims, typ)
            .map_err(|(reason, error)| {
                let telemetry = self.store.telemetry();
                telemetry
                    .signing_errors
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if let Some(reason) = reason {
                    telemetry.remote_signing_failed(reason);
                }
                error
            })
    }
    /// A local key's error has no label; every remote failure has exactly one.
    fn sign_jwt_inner(
        &self,
        key: &SigningKey,
        claims: &Value,
        typ: &str,
    ) -> std::result::Result<String, (Option<Reason>, Error)> {
        let Some(remote) = &key.remote else {
            return key.sign_type(claims, typ).map_err(|error| (None, error));
        };
        self.sign_remote(key, remote, claims, typ)
            .map_err(|(reason, error)| (Some(reason), error))
    }
    fn sign_remote(
        &self,
        key: &SigningKey,
        remote: &RemoteKey,
        claims: &Value,
        typ: &str,
    ) -> std::result::Result<String, Failed> {
        key.jwk().map_err(because(Reason::StoredKey))?;
        let config = self
            .config
            .signers
            .get(&remote.signer)
            .filter(|s| s.key_version == remote.key_version && s.public_jwk == remote.public_jwk)
            .ok_or_else(|| (Reason::ConfigurationBinding, unavailable()))?;
        let protected = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&json!({"alg":key.algorithm,"kid":key.kid,"typ":typ}))
                .map_err(Error::internal)
                .map_err(because(Reason::Encoding))?,
        );
        let payload = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(claims)
                .map_err(Error::internal)
                .map_err(because(Reason::Encoding))?,
        );
        let input = format!("{protected}.{payload}");
        let signature = config.sign(&input)?;
        let jwt = format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature));
        // Verification checks the returned signature against our public pin, independent of Vault.
        let mut validation = jsonwebtoken::Validation::new(
            remote
                .public_jwk
                .algorithm()
                .map_err(because(Reason::StoredKey))?,
        );
        validation.required_spec_claims.clear();
        validation.validate_exp = false;
        validation.validate_aud = false;
        validation.validate_nbf = false;
        let verified = jsonwebtoken::decode::<Value>(
            &jwt,
            &remote
                .public_jwk
                .decoding_key()
                .map_err(because(Reason::StoredKey))?,
            &validation,
        )
        .map_err(|_| (Reason::SignatureVerification, unavailable()))?;
        if verified.claims != *claims {
            return Err((Reason::SignatureVerification, unavailable()));
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
