//! Device trust: a local compact-JWT stand-in, or an explicit Chrome Verified Access v2 adapter.
//!
//! The local contract is a nonce-bound JWT under a pinned PEM or JWKS key. The v2
//! adapter asks Google for a challenge and submits a signed challenge response.
//! It does not accept the local JWT. Neither path was executed against a managed
//! Chrome device or `verifiedaccess.googleapis.com`. Replacing the local public
//! key does not establish managed-device compatibility.
use crate::{
    crypto::now,
    error::{Error, Result},
    model::{Client, Identity, Session},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::Value;
use std::{fs::File, io::Read, path::Path};

pub use crate::device_trust_types::{
    CHALLENGE_TTL, Challenge, DEFAULT_FRESHNESS, DeviceVerification, MAX_FRESHNESS, TrustConfig,
};

#[path = "verified_access.rs"]
pub(crate) mod verified_access;

pub(crate) use verified_access::TokenCache;
pub use verified_access::{
    GENERATE_URL, GOOGLE_CHALLENGE_TTL, SCOPE, TOKEN_URL, VERIFY_URL, VerifiedAccessRequest,
    VerifiedAccessResponse, VerifiedAccessTransport,
};

/// End-user verify body. The local provider accepts only `token`. The Verified
/// Access provider accepts only `challenge` and `challenge_response`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceTrustSubmission {
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub challenge: Option<String>,
    #[serde(default)]
    pub challenge_response: Option<String>,
}

/// The pinned verifier configuration supplied by server assembly.
pub trait DeviceTrustContext {
    fn device_trust_config(&self) -> Option<&TrustConfig>;
}

/// Device binding records read or cleaned in the caller's transaction.
pub trait DeviceTrustTx {
    fn stored_session(&self, session_id: &str) -> Result<Option<Session>>;
    fn verification(&self, session_id: &str) -> Result<Option<DeviceVerification>>;
    fn challenge_page(&self) -> Result<Vec<(String, Challenge)>>;
    fn verification_page(&self) -> Result<Vec<(String, DeviceVerification)>>;
    fn delete_challenge(&self, id: &str) -> Result<()>;
    fn delete_verification(&self, id: &str) -> Result<()>;
}

enum Verifier {
    Pem {
        algorithm: Algorithm,
        key: DecodingKey,
        kid: Option<String>,
    },
    Jwks(crate::jose::PublicJwks),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderKind {
    Local,
    GoogleVerifiedAccessV2,
}

impl ProviderKind {
    pub(crate) fn identity(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::GoogleVerifiedAccessV2 => "google_verified_access_v2",
        }
    }
}

pub(crate) fn provider_kind(config: &TrustConfig) -> Result<ProviderKind> {
    match config.provider.as_deref() {
        None | Some("local") => {
            if config.service_account_file.is_some()
                || config.expected_identity.is_some()
                || config.customer_id.is_some()
                || !config.allowed_key_trust_levels.is_empty()
            {
                return Err(Error::bad(
                    "Local device trust rejects Verified Access settings",
                ));
            }
            Ok(ProviderKind::Local)
        }
        Some("google_verified_access_v2") => {
            if config.pem_file.is_some()
                || config.jwks_file.is_some()
                || config.algorithm.is_some()
                || config.kid.is_some()
            {
                return Err(Error::bad("Verified Access rejects the local JWT verifier"));
            }
            Ok(ProviderKind::GoogleVerifiedAccessV2)
        }
        Some(_) => Err(Error::bad(
            "Device trust provider must be local or google_verified_access_v2",
        )),
    }
}

pub(crate) fn provider_ready(config: &TrustConfig) -> Result<()> {
    match provider_kind(config)? {
        ProviderKind::Local => load_verifier(config).map(|_| ()),
        ProviderKind::GoogleVerifiedAccessV2 => verified_access::validate_google_config(config),
    }
}

/// Drop a challenge once its lifetime and any retained response hash have both ended.
pub(crate) fn drop_challenge(challenge: &Challenge, at: u64) -> bool {
    challenge.expires_at <= at && challenge.response_retained_until <= at
}

pub fn validate_config(config: &TrustConfig) -> anyhow::Result<()> {
    provider_ready(config).map_err(|error| anyhow::anyhow!(error.message))
}

fn load_verifier(config: &TrustConfig) -> Result<Verifier> {
    if !(1..=MAX_FRESHNESS).contains(&config.freshness_ttl) {
        return Err(Error::bad(
            "Device trust freshness TTL must be 1..=3600 seconds",
        ));
    }
    match (&config.pem_file, &config.jwks_file) {
        (Some(pem), None) => {
            let algorithm = match config.algorithm.as_deref() {
                Some("RS256") => Algorithm::RS256,
                Some("ES256") => Algorithm::ES256,
                Some("EdDSA") => Algorithm::EdDSA,
                _ => {
                    return Err(Error::bad(
                        "Device trust PEM verifier requires algorithm RS256, ES256, or EdDSA",
                    ));
                }
            };
            let pem = read_bounded(pem, 16_384)?;
            let key = match algorithm {
                Algorithm::RS256 => DecodingKey::from_rsa_pem(pem.as_bytes()),
                Algorithm::ES256 => DecodingKey::from_ec_pem(pem.as_bytes()),
                Algorithm::EdDSA => DecodingKey::from_ed_pem(pem.as_bytes()),
                _ => {
                    return Err(Error::bad(
                        "Device trust PEM verifier requires algorithm RS256, ES256, or EdDSA",
                    ));
                }
            }
            .map_err(|_| Error::bad("Device trust verifier key is invalid"))?;
            Ok(Verifier::Pem {
                algorithm,
                key,
                kid: config.kid.clone(),
            })
        }
        (None, Some(path)) => {
            let text = read_bounded(path, 65_536)?;
            let jwks: crate::jose::PublicJwks = serde_json::from_str(&text)
                .map_err(|_| Error::bad("Device trust verifier key is invalid"))?;
            jwks.validate()?;
            Ok(Verifier::Jwks(jwks))
        }
        _ => Err(Error::bad(
            "Device trust requires exactly one of pem_file or jwks_file",
        )),
    }
}

fn read_bounded(path: &Path, limit: u64) -> Result<String> {
    let file =
        File::open(path).map_err(|_| Error::bad("Device trust verifier key is unavailable"))?;
    let metadata = file
        .metadata()
        .map_err(|_| Error::bad("Device trust verifier key is unavailable"))?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(Error::bad("Device trust verifier key is unavailable"));
    }
    let mut text = String::new();
    file.take(limit + 1)
        .read_to_string(&mut text)
        .map_err(|_| Error::bad("Device trust verifier key is unavailable"))?;
    if text.len() as u64 > limit {
        return Err(Error::bad("Device trust verifier key is unavailable"));
    }
    Ok(text)
}

pub(crate) fn unavailable() -> Error {
    Error::oauth(
        "unmet_authentication_requirements",
        "Device trust verifier is not configured",
    )
}

pub fn policy_reason(
    context: &impl DeviceTrustContext,
    tx: &impl DeviceTrustTx,
    client: &Client,
    identity: Option<&Identity>,
) -> Result<Option<&'static str>> {
    if !client.settings.require_device_trust {
        return Ok(None);
    }
    let Some(config) = context.device_trust_config() else {
        return Ok(Some("device_trust_verifier_unconfigured"));
    };
    if provider_ready(config).is_err() {
        return Ok(Some("device_trust_verifier_unconfigured"));
    }
    let Some(identity) = identity else {
        return Ok(Some("device_trust_session_required"));
    };
    let provider = provider_kind(config)?.identity();
    let at = now();
    let session_active = tx
        .stored_session(&identity.session_id)?
        .is_some_and(|session| {
            !session.revoked
                && session.expires_at > at
                && session.identity.user_id == identity.user_id
                && session.identity.epoch == identity.epoch
        });
    let fresh = tx
        .verification(&identity.session_id)?
        .is_some_and(|record| {
            record.user_id == identity.user_id
                && record.session_id == identity.session_id
                && record.epoch == identity.epoch
                && record.expires_at > at
                && !record.device_id.is_empty()
                && record.provider.as_deref() == Some(provider)
        });
    if session_active && fresh {
        Ok(None)
    } else {
        Ok(Some("device_trust_required"))
    }
}

pub fn require(
    context: &impl DeviceTrustContext,
    tx: &impl DeviceTrustTx,
    client: &Client,
    identity: &Identity,
) -> Result<()> {
    match policy_reason(context, tx, client, Some(identity))? {
        None => Ok(()),
        Some("device_trust_verifier_unconfigured") => Err(unavailable()),
        Some(_) => Err(Error::oauth(
            "unmet_authentication_requirements",
            "Fresh device trust verification is required",
        )),
    }
}

pub(crate) fn verify_token(config: &TrustConfig, token: &str, audience: &str) -> Result<Value> {
    if token.len() > 16_384 || token.matches('.').count() != 2 {
        return Err(Error::bad("Device trust token validation failed"));
    }
    let header = jsonwebtoken::decode_header(token)
        .map_err(|_| Error::bad("Device trust token validation failed"))?;
    if header.jku.is_some()
        || header.jwk.is_some()
        || header.x5u.is_some()
        || header.crit.as_ref().is_some_and(|crit| !crit.is_empty())
    {
        return Err(Error::bad(
            "Device trust token must use the pinned verifier key",
        ));
    }
    let verifier = load_verifier(config).map_err(|_| unavailable())?;
    let (algorithm, key) = match verifier {
        Verifier::Pem {
            algorithm,
            key,
            kid,
        } => {
            if header.alg != algorithm
                || kid
                    .as_ref()
                    .is_some_and(|expected| header.kid.as_ref() != Some(expected))
            {
                return Err(Error::bad("Unknown device trust key"));
            }
            (algorithm, key)
        }
        Verifier::Jwks(jwks) => {
            let pinned = jwks
                .keys
                .iter()
                .find(|key| {
                    header.kid.as_ref() == Some(&key.kid)
                        || (header.kid.is_none() && jwks.keys.len() == 1)
                })
                .ok_or_else(|| Error::bad("Unknown device trust key"))?;
            if pinned.algorithm()? != header.alg {
                return Err(Error::bad(
                    "Device trust algorithm does not match the pinned key",
                ));
            }
            (header.alg, pinned.decoding_key()?)
        }
    };
    let mut validation = Validation::new(algorithm);
    validation.set_audience(&[audience]);
    validation.set_required_spec_claims(&["aud", "exp"]);
    validation.leeway = 0;
    validation.validate_exp = true;
    validation.validate_nbf = true;
    jsonwebtoken::decode::<Value>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|_| Error::bad("Device trust token validation failed"))
}

pub(crate) fn device_identifier(claims: &Value) -> Result<String> {
    for name in [
        "device_permanent_id",
        "devicePermanentId",
        "device_id",
        "device_enrollment_id",
        "sub",
    ] {
        if let Some(value) = claims.get(name).and_then(Value::as_str) {
            if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
                return Err(Error::bad("Invalid device identifier"));
            }
            return Ok(value.to_owned());
        }
    }
    Err(Error::bad(
        "Device trust token is missing a device identifier",
    ))
}

pub fn cleanup(tx: &impl DeviceTrustTx, at: u64) -> Result<()> {
    for (id, challenge) in tx.challenge_page()? {
        if drop_challenge(&challenge, at) {
            tx.delete_challenge(&id)?;
        }
    }
    for (id, verification) in tx.verification_page()? {
        // Retain the device binding throughout the session, even when freshness
        // has expired. Legacy user-keyed rows have no session and are discarded.
        let session_active = tx
            .stored_session(&verification.session_id)?
            .is_some_and(|session| !session.revoked && session.expires_at > at);
        if !session_active {
            tx.delete_verification(&id)?;
        }
    }
    Ok(())
}
