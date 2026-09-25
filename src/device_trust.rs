//! Local stand-in for Chrome Enterprise device trust.
//!
//! A managed Chrome deployment and Google's Verified Access API were not tested.
//! The local compact-JWT contract requires a separate, reviewed adapter for a
//! vendor verifier. Merely replacing the key does not establish compatibility.
use crate::{
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Client, Identity, Session},
    store::Tx,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub const CHALLENGE_TTL: u64 = 120;
pub const DEFAULT_FRESHNESS: u64 = 300;
pub const MAX_FRESHNESS: u64 = 3600;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustConfig {
    /// SPKI or PKCS#1 PEM public key. Mutually exclusive with `jwks_file`.
    #[serde(default)]
    pub pem_file: Option<PathBuf>,
    /// JWKS document containing 1–8 verification keys. Mutually exclusive with `pem_file`.
    #[serde(default)]
    pub jwks_file: Option<PathBuf>,
    /// Required with `pem_file`: RS256, ES256, or EdDSA.
    #[serde(default)]
    pub algorithm: Option<String>,
    /// When set, the device JWT `kid` must equal this value (PEM has a single key).
    #[serde(default)]
    pub kid: Option<String>,
    /// How long a successful verification stays fresh. Default 300, maximum 3600.
    #[serde(default = "default_freshness")]
    pub freshness_ttl: u64,
}

fn default_freshness() -> u64 {
    DEFAULT_FRESHNESS
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            pem_file: None,
            jwks_file: None,
            algorithm: None,
            kid: None,
            freshness_ttl: DEFAULT_FRESHNESS,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub user_id: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub epoch: u64,
    pub expires_at: u64,
    pub used: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceVerification {
    pub device_id: String,
    pub user_id: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub epoch: u64,
    pub verified_at: u64,
    pub expires_at: u64,
}

enum Verifier {
    Pem {
        algorithm: Algorithm,
        key: DecodingKey,
        kid: Option<String>,
    },
    Jwks(crate::jose::PublicJwks),
}

pub fn validate_config(config: &TrustConfig) -> anyhow::Result<()> {
    load_verifier(config)
        .map(|_| ())
        .map_err(|error| anyhow::anyhow!(error.message))
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

fn unavailable() -> Error {
    Error::oauth(
        "unmet_authentication_requirements",
        "Device trust verifier is not configured",
    )
}

pub fn policy_reason(
    core: &Core,
    tx: &Tx<'_>,
    client: &Client,
    identity: Option<&Identity>,
) -> Result<Option<&'static str>> {
    if !client.settings.require_device_trust {
        return Ok(None);
    }
    let Some(config) = &core.config.device_trust else {
        return Ok(Some("device_trust_verifier_unconfigured"));
    };
    if load_verifier(config).is_err() {
        return Ok(Some("device_trust_verifier_unconfigured"));
    }
    let Some(identity) = identity else {
        return Ok(Some("device_trust_session_required"));
    };
    let at = now();
    let session_active = tx
        .get::<Session>("sessions", &identity.session_id)?
        .is_some_and(|session| {
            !session.revoked
                && session.expires_at > at
                && session.identity.user_id == identity.user_id
                && session.identity.epoch == identity.epoch
        });
    let fresh = tx
        .get::<DeviceVerification>("device_verifications", &identity.session_id)?
        .is_some_and(|record| {
            record.user_id == identity.user_id
                && record.session_id == identity.session_id
                && record.epoch == identity.epoch
                && record.expires_at > at
                && !record.device_id.is_empty()
        });
    if session_active && fresh {
        Ok(None)
    } else {
        Ok(Some("device_trust_required"))
    }
}

pub fn require(core: &Core, tx: &Tx<'_>, client: &Client, identity: &Identity) -> Result<()> {
    match policy_reason(core, tx, client, Some(identity))? {
        None => Ok(()),
        Some("device_trust_verifier_unconfigured") => Err(unavailable()),
        Some(_) => Err(Error::oauth(
            "unmet_authentication_requirements",
            "Fresh device trust verification is required",
        )),
    }
}

fn verify_token(config: &TrustConfig, token: &str, audience: &str) -> Result<Value> {
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

fn device_identifier(claims: &Value) -> Result<String> {
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

impl Core {
    pub fn device_challenge(&self, token: &str) -> Result<Value> {
        if self.config.device_trust.is_none() {
            return Err(unavailable());
        }
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            let mut active = 0u32;
            for (key, challenge) in tx.list::<Challenge>("device_challenges")? {
                if challenge.used || challenge.expires_at <= now() {
                    tx.delete("device_challenges", &key)?;
                } else if challenge.user_id == user.id {
                    active += 1;
                }
            }
            if active >= 8 {
                return Err(Error::bad("Too many outstanding device challenges"));
            }
            let issued_at = now();
            let challenge = crypto::random_token("");
            let record = Challenge {
                user_id: user.id.clone(),
                session_id: session.id.clone(),
                epoch: user.epoch,
                expires_at: (issued_at + CHALLENGE_TTL).min(session.expires_at),
                used: false,
            };
            tx.put("device_challenges", &digest(&challenge), &record)?;
            audit(tx, &user.id, "device_trust.challenge", &user.id)?;
            Ok(serde_json::json!({
                "challenge": challenge,
                "expires_in": record.expires_at.saturating_sub(issued_at),
                "expires_at": record.expires_at,
                "audience": self.config.issuer,
            }))
        })
    }

    pub fn device_verify(&self, token: &str, device_token: &str) -> Result<Value> {
        let config = self.config.device_trust.clone().ok_or_else(unavailable)?;
        let claims = verify_token(&config, device_token, &self.config.issuer)?;
        let nonce = claims["nonce"]
            .as_str()
            .filter(|nonce| !nonce.is_empty() && nonce.len() <= 512)
            .ok_or_else(|| {
                Error::bad("Device trust nonce does not match an outstanding challenge")
            })?;
        let device_id = device_identifier(&claims)?;
        let proof_expires_at = claims["exp"]
            .as_u64()
            .ok_or_else(|| Error::bad("Device trust token validation failed"))?;
        let hash = digest(nonce);
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            let mut challenge = tx
                .get::<Challenge>("device_challenges", &hash)?
                .ok_or_else(|| {
                    Error::bad("Device trust nonce does not match an outstanding challenge")
                })?;
            if challenge.user_id != user.id
                || challenge.session_id != session.id
                || challenge.epoch != user.epoch
            {
                return Err(Error::bad(
                    "Device trust nonce does not match an outstanding challenge",
                ));
            }
            if challenge.used {
                return Err(Error::bad("Device challenge already used"));
            }
            if challenge.expires_at <= now() {
                return Err(Error::bad("Device challenge expired"));
            }
            if proof_expires_at <= now() {
                return Err(Error::bad("Device trust token expired"));
            }
            if tx
                .get::<DeviceVerification>("device_verifications", &session.id)?
                .is_some_and(|previous| previous.device_id != device_id)
            {
                return Err(Error::bad("A different device requires a new session"));
            }
            challenge.used = true;
            tx.put("device_challenges", &hash, &challenge)?;
            let verified_at = now();
            let record = DeviceVerification {
                device_id: device_id.clone(),
                user_id: user.id.clone(),
                session_id: session.id.clone(),
                epoch: user.epoch,
                verified_at,
                expires_at: (verified_at + config.freshness_ttl)
                    .min(proof_expires_at)
                    .min(session.expires_at),
            };
            tx.put("device_verifications", &session.id, &record)?;
            audit(tx, &user.id, "device_trust.verified", &device_id)?;
            Ok(serde_json::json!({
                "verified": true,
                "device_id": device_id,
                "verified_at": verified_at,
                "expires_at": record.expires_at,
            }))
        })
    }
}

pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, challenge) in tx.maintenance_page::<Challenge>("device_challenges")? {
        if challenge.used || challenge.expires_at < at {
            tx.delete("device_challenges", &id)?;
        }
    }
    for (id, verification) in tx.maintenance_page::<DeviceVerification>("device_verifications")? {
        // Retain the device binding throughout the session, even when freshness
        // has expired. Legacy user-keyed rows have no session and are discarded.
        let session_active = tx
            .get::<Session>("sessions", &verification.session_id)?
            .is_some_and(|session| !session.revoked && session.expires_at > at);
        if !session_active {
            tx.delete("device_verifications", &id)?;
        }
    }
    Ok(())
}
