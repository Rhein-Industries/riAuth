//! Assemble device-trust persistence and authenticated Core entry points.

use crate::{
    core::{Core, audit},
    crypto::{self, digest, now},
    device_trust::{
        self, CHALLENGE_TTL, Challenge, DeviceTrustContext, DeviceTrustTx, DeviceVerification,
        device_identifier, unavailable, verify_token,
    },
    error::{Error, Result},
    model::Session,
    store::Tx,
};
use serde_json::Value;

impl DeviceTrustContext for Core {
    fn device_trust_config(&self) -> Option<&device_trust::TrustConfig> {
        self.config.device_trust.as_ref()
    }
}

impl DeviceTrustTx for Tx<'_> {
    fn stored_session(&self, session_id: &str) -> Result<Option<Session>> {
        self.get("sessions", session_id)
    }

    fn verification(&self, session_id: &str) -> Result<Option<DeviceVerification>> {
        self.get("device_verifications", session_id)
    }

    fn challenge_page(&self) -> Result<Vec<(String, Challenge)>> {
        self.maintenance_page("device_challenges")
    }

    fn verification_page(&self) -> Result<Vec<(String, DeviceVerification)>> {
        self.maintenance_page("device_verifications")
    }

    fn delete_challenge(&self, id: &str) -> Result<()> {
        self.delete("device_challenges", id)
    }

    fn delete_verification(&self, id: &str) -> Result<()> {
        self.delete("device_verifications", id)
    }
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
