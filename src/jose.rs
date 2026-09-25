//! Pinned JOSE trust. No token-controlled URL is ever fetched.
use crate::{
    crypto::{digest, now},
    error::{Error, Result},
    store::Tx,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const ASSERTION_TYPE: &str = "urn:ietf:params:oauth:client-assertion-type:jwt-bearer";
pub const JWT_GRANT: &str = "urn:ietf:params:oauth:grant-type:jwt-bearer";

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientAuthMethod {
    None,
    ClientSecretBasic,
    ClientSecretPost,
    PrivateKeyJwt,
}

#[derive(schemars::JsonSchema, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublicJwks {
    pub keys: Vec<PublicJwk>,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PublicJwk {
    pub kty: String,
    pub kid: String,
    pub alg: String,
    #[serde(rename = "use", skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_ops: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,
}

impl PublicJwk {
    pub fn algorithm(&self) -> Result<Algorithm> {
        match (self.kty.as_str(), self.alg.as_str(), self.crv.as_deref()) {
            ("RSA", "RS256", None) => Ok(Algorithm::RS256),
            ("EC", "ES256", Some("P-256")) => Ok(Algorithm::ES256),
            ("OKP", "EdDSA", Some("Ed25519")) => Ok(Algorithm::EdDSA),
            _ => Err(Error::bad(
                "Supported public signing keys: RS256, ES256/P-256, EdDSA/Ed25519",
            )),
        }
    }
    pub fn decoding_key(&self) -> Result<DecodingKey> {
        let jwk = serde_json::from_value(serde_json::to_value(self).map_err(Error::internal)?)
            .map_err(|_| Error::bad("Invalid public JWK"))?;
        DecodingKey::from_jwk(&jwk).map_err(|_| Error::bad("Invalid public JWK"))
    }
    pub fn validate(&self) -> Result<()> {
        if self.kid.is_empty()
            || self.kid.len() > 128
            || !self.kid.bytes().all(|b| b.is_ascii_graphic())
            || self.usage.as_deref().is_some_and(|u| u != "sig")
            || self.key_ops.iter().any(|op| op != "verify")
            || self.key_ops.len() > 1
        {
            return Err(Error::bad(
                "A public signing JWK requires a unique kid and verification-only usage",
            ));
        }
        let decode = |s: &Option<String>| -> Result<Vec<u8>> {
            let s = s
                .as_deref()
                .ok_or_else(|| Error::bad("Missing JWK component"))?;
            let bytes = URL_SAFE_NO_PAD
                .decode(s)
                .map_err(|_| Error::bad("Invalid JWK component"))?;
            if URL_SAFE_NO_PAD.encode(&bytes) != s {
                return Err(Error::bad("Noncanonical JWK component"));
            }
            Ok(bytes)
        };
        match self.algorithm()? {
            Algorithm::RS256 => {
                let n = decode(&self.n)?;
                let e = decode(&self.e)?;
                if !(256..=512).contains(&n.len())
                    || n[0] < 128
                    || n.last().is_none_or(|n| n & 1 == 0)
                    || e.is_empty()
                    || e.len() > 4
                    || e[0] == 0
                    || e.last().is_none_or(|e| e & 1 == 0)
                    || (e.len() == 1 && e[0] < 3)
                    || self.x.is_some()
                    || self.y.is_some()
                {
                    return Err(Error::bad(
                        "RSA verification keys must have a 2048–4096 bit modulus and valid exponent",
                    ));
                }
            }
            Algorithm::ES256 => {
                if decode(&self.x)?.len() != 32
                    || decode(&self.y)?.len() != 32
                    || self.n.is_some()
                    || self.e.is_some()
                {
                    return Err(Error::bad("Invalid P-256 public key"));
                }
            }
            Algorithm::EdDSA => {
                if decode(&self.x)?.len() != 32
                    || self.y.is_some()
                    || self.n.is_some()
                    || self.e.is_some()
                {
                    return Err(Error::bad("Invalid Ed25519 public key"));
                }
            }
            _ => return Err(Error::bad("Unsupported signing algorithm")),
        }
        self.decoding_key()?;
        Ok(())
    }
}

impl PublicJwks {
    pub fn validate(&self) -> Result<()> {
        if self.keys.is_empty() || self.keys.len() > 8 {
            return Err(Error::bad("Provide 1–8 pinned public signing keys"));
        }
        let mut kids = BTreeSet::new();
        for key in &self.keys {
            key.validate()?;
            if !kids.insert(&key.kid) {
                return Err(Error::bad("Duplicate public key id"));
            }
        }
        Ok(())
    }
    pub fn verify(&self, token: &str, issuer: &str, audience: &str) -> Result<Value> {
        self.verify_claims(token, issuer, audience, false)
    }

    /// SSF SETs deliberately have no `exp` claim. Their required `iat` and
    /// replay lifetime are checked by the SSF receiver after signature checks.
    pub fn verify_set(&self, token: &str, issuer: &str, audience: &str) -> Result<Value> {
        self.verify_claims(token, issuer, audience, true)
    }

    /// Verify a SET's signature without claim validation, only to select the
    /// RFC 8935 delivery error after normal stream validation has failed.
    pub(crate) fn verify_set_signature(&self, token: &str) -> bool {
        let Ok(header) = jsonwebtoken::decode_header(token) else {
            return false;
        };
        let Some(key) = self
            .keys
            .iter()
            .find(|key| Some(&key.kid) == header.kid.as_ref())
        else {
            return false;
        };
        if key.algorithm().ok() != Some(header.alg) {
            return false;
        }
        let Ok(decoding_key) = key.decoding_key() else {
            return false;
        };
        let mut validation = Validation::new(header.alg);
        validation.required_spec_claims.clear();
        validation.validate_aud = false;
        validation.validate_exp = false;
        validation.validate_nbf = false;
        jsonwebtoken::decode::<Value>(token, &decoding_key, &validation).is_ok()
    }

    fn verify_claims(
        &self,
        token: &str,
        issuer: &str,
        audience: &str,
        security_event: bool,
    ) -> Result<Value> {
        if token.len() > 16_384 {
            return Err(Error::bad("JWT exceeds size limit"));
        }
        let header =
            jsonwebtoken::decode_header(token).map_err(|_| Error::bad("Malformed JWT header"))?;
        if header.jku.is_some()
            || header.jwk.is_some()
            || header.x5u.is_some()
            || header.crit.as_ref().is_some_and(|c| !c.is_empty())
        {
            return Err(Error::bad("JWT must use pinned keys and supported headers"));
        }
        let key = self
            .keys
            .iter()
            .find(|k| Some(&k.kid) == header.kid.as_ref())
            .ok_or_else(|| Error::bad("Unknown JWT key"))?;
        if key.algorithm()? != header.alg {
            return Err(Error::bad(
                "JWT algorithm does not match its registered key",
            ));
        }
        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[issuer]);
        validation.set_audience(&[audience]);
        if security_event {
            validation.set_required_spec_claims(&["iss", "aud", "iat", "jti"]);
            validation.validate_exp = false;
        } else {
            validation.set_required_spec_claims(&["iss", "aud", "exp"]);
        }
        validation.leeway = 0;
        validation.validate_nbf = true;
        jsonwebtoken::decode::<Value>(token, &key.decoding_key()?, &validation)
            .map(|t| t.claims)
            .map_err(|_| Error::bad("JWT signature or claims validation failed"))
    }
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MachineTrust {
    pub issuer: String,
    pub subject: String,
    pub jwks: PublicJwks,
    pub scopes: BTreeSet<String>,
}

/// Consume a short-lived assertion inside the same transaction as its result.
pub fn consume_assertion(tx: &Tx<'_>, claims: &Value, namespace: &str) -> Result<()> {
    let at = now();
    let iat = claims["iat"]
        .as_u64()
        .ok_or_else(|| Error::bad("Assertion requires iat"))?;
    let exp = claims["exp"]
        .as_u64()
        .ok_or_else(|| Error::bad("Assertion requires exp"))?;
    let jti = claims["jti"]
        .as_str()
        .filter(|j| !j.is_empty() && j.len() <= 256)
        .ok_or_else(|| Error::bad("Assertion requires a bounded unique jti"))?;
    if iat > at + 30 || exp <= at || exp <= iat || exp - iat > 300 || at.saturating_sub(iat) > 300 {
        return Err(Error::bad(
            "Assertion lifetime must be at most five minutes",
        ));
    }
    let key = digest(&format!("{namespace}\0{}\0{jti}", claims["iss"]));
    if tx
        .get::<u64>("assertion_replays", &key)?
        .is_some_and(|e| e > at)
    {
        return Err(Error::bad("Assertion already used"));
    }
    tx.put("assertion_replays", &key, &exp)
}
