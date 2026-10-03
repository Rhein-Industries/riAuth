//! Assemble device-trust persistence and authenticated Core entry points.

use crate::{
    core::{Core, audit},
    crypto::{self, digest, now},
    device_trust::{
        self, CHALLENGE_TTL, Challenge, DeviceTrustContext, DeviceTrustTx, DeviceVerification,
        device_identifier, unavailable, verify_token,
    },
    error::{Error, Result},
    model::{Session, User},
    store::Tx,
};
use serde_json::{Value, json};
use std::sync::Arc;

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
        let config = self.config.device_trust.clone().ok_or_else(unavailable)?;
        match device_trust::provider_kind(&config)? {
            device_trust::ProviderKind::Local => self.device_challenge_local(token),
            device_trust::ProviderKind::GoogleVerifiedAccessV2 => {
                self.device_challenge_google(token, &config)
            }
        }
    }

    fn device_challenge_local(&self, token: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            self.reap_device_challenges(tx)?;
            if self.outstanding_device_challenges(tx, &user.id)? >= 8 {
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
                response_sha256: String::new(),
                response_retained_until: 0,
                issued_at,
            };
            tx.put("device_challenges", &digest(&challenge), &record)?;
            audit(tx, &user.id, "device_trust.challenge", &user.id)?;
            Ok(json!({
                "challenge": challenge,
                "expires_in": record.expires_at.saturating_sub(issued_at),
                "expires_at": record.expires_at,
                "audience": self.config.issuer,
                "provider": "local",
            }))
        })
    }

    fn device_challenge_google(
        &self,
        token: &str,
        config: &device_trust::TrustConfig,
    ) -> Result<Value> {
        // A read transaction cannot delete. Count outstanding challenges first so
        // Google is not called for a dead session or a full cap. The write reaps.
        self.store.read(|tx| {
            let (user, _) = self.session(tx, token)?;
            if self.outstanding_device_challenges(tx, &user.id)? >= 8 {
                return Err(Error::bad("Too many outstanding device challenges"));
            }
            Ok(())
        })?;
        let challenge = device_trust::verified_access::generate_challenge(
            config,
            &self.verified_access_cache,
            self.verified_access_transport()?.as_ref(),
        )?;
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            self.reap_device_challenges(tx)?;
            if self.outstanding_device_challenges(tx, &user.id)? >= 8 {
                return Err(Error::bad("Too many outstanding device challenges"));
            }
            let hash = digest(&challenge);
            if tx.get::<Challenge>("device_challenges", &hash)?.is_some() {
                return Err(Error::bad(
                    "Verified Access challenge does not match this session",
                ));
            }
            let issued_at = now();
            let record = Challenge {
                user_id: user.id.clone(),
                session_id: session.id.clone(),
                epoch: user.epoch,
                expires_at: (issued_at + device_trust::GOOGLE_CHALLENGE_TTL)
                    .min(session.expires_at),
                used: false,
                response_sha256: String::new(),
                response_retained_until: 0,
                issued_at,
            };
            tx.put("device_challenges", &hash, &record)?;
            audit(tx, &user.id, "device_trust.challenge", &user.id)?;
            Ok(json!({
                "challenge": challenge,
                "expires_in": record.expires_at.saturating_sub(issued_at),
                "expires_at": record.expires_at,
                "provider": "google_verified_access_v2",
            }))
        })
    }

    pub fn device_verify(&self, token: &str, device_token: &str) -> Result<Value> {
        let config = self.config.device_trust.clone().ok_or_else(unavailable)?;
        if device_trust::provider_kind(&config)? != device_trust::ProviderKind::Local {
            return Err(Error::bad(
                "Device trust provider does not accept a local JWT",
            ));
        }
        self.device_verify_local(token, &config, device_token)
    }

    pub fn device_verify_submitted(
        &self,
        token: &str,
        body: device_trust::DeviceTrustSubmission,
    ) -> Result<Value> {
        let config = self.config.device_trust.clone().ok_or_else(unavailable)?;
        match device_trust::provider_kind(&config)? {
            device_trust::ProviderKind::Local => {
                if body.challenge.is_some() || body.challenge_response.is_some() {
                    return Err(Error::bad(
                        "Local device trust does not accept a Verified Access response",
                    ));
                }
                let device_token = body
                    .token
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| Error::bad("Device trust token validation failed"))?;
                self.device_verify_local(token, &config, device_token)
            }
            device_trust::ProviderKind::GoogleVerifiedAccessV2 => {
                if body.token.is_some() {
                    return Err(Error::bad(
                        "Device trust provider does not accept a local JWT",
                    ));
                }
                let challenge = body.challenge.as_deref().ok_or_else(|| {
                    Error::bad("Verified Access challenge does not match this session")
                })?;
                let response = body
                    .challenge_response
                    .as_deref()
                    .ok_or_else(|| Error::bad("Verified Access challenge response is required"))?;
                self.device_verify_google(token, &config, challenge, response)
            }
        }
    }

    fn device_verify_local(
        &self,
        token: &str,
        config: &device_trust::TrustConfig,
        device_token: &str,
    ) -> Result<Value> {
        let claims = verify_token(config, device_token, &self.config.issuer)?;
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
                || challenge.epoch != session.identity.epoch
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
                provider: Some(device_trust::ProviderKind::Local.identity().into()),
                verified_at,
                expires_at: (verified_at + config.freshness_ttl)
                    .min(proof_expires_at)
                    .min(session.expires_at),
            };
            tx.put("device_verifications", &session.id, &record)?;
            audit(tx, &user.id, "device_trust.verified", &device_id)?;
            Ok(json!({
                "verified": true,
                "device_id": device_id,
                "verified_at": verified_at,
                "expires_at": record.expires_at,
            }))
        })
    }

    fn device_verify_google(
        &self,
        token: &str,
        config: &device_trust::TrustConfig,
        challenge: &str,
        challenge_response: &str,
    ) -> Result<Value> {
        let challenge = device_trust::verified_access::require_challenge_material(challenge)?;
        let response =
            device_trust::verified_access::require_response_material(challenge_response)?;
        let response_hash = digest(&response);
        let hash = digest(&challenge);
        self.store.read(|tx| {
            self.google_bound_challenge(tx, token, &hash, &response_hash)?;
            Ok(())
        })?;
        // Session, freshness, and response-hash checks stay ahead of this parse
        // so those failures keep their existing errors. A mismatch does not call
        // verify and does not consume the challenge.
        device_trust::verified_access::response_answers_challenge(&challenge, &response)?;
        let accepted = device_trust::verified_access::verify_challenge_response(
            config,
            &self.verified_access_cache,
            self.verified_access_transport()?.as_ref(),
            &response,
        )?;

        self.store.write(|tx| {
            let (user, session, mut challenge) =
                self.google_bound_challenge(tx, token, &hash, &response_hash)?;
            let verified_at = now();
            challenge.used = true;
            challenge.response_sha256 = response_hash.clone();
            challenge.response_retained_until =
                verified_at.saturating_add(device_trust::GOOGLE_CHALLENGE_TTL);
            if tx
                .get::<DeviceVerification>("device_verifications", &session.id)?
                .is_some_and(|previous| previous.device_id != accepted.device_id)
            {
                tx.put("device_challenges", &hash, &challenge)?;
                return Ok(Err(Error::bad("A different device requires a new session")));
            }
            tx.put("device_challenges", &hash, &challenge)?;
            let record = DeviceVerification {
                device_id: accepted.device_id.clone(),
                user_id: user.id.clone(),
                session_id: session.id.clone(),
                epoch: user.epoch,
                provider: Some(
                    device_trust::ProviderKind::GoogleVerifiedAccessV2
                        .identity()
                        .into(),
                ),
                verified_at,
                expires_at: (verified_at + config.freshness_ttl).min(session.expires_at),
            };
            tx.put("device_verifications", &session.id, &record)?;
            audit(tx, &user.id, "device_trust.verified", &accepted.device_id)?;
            Ok(Ok(json!({
                "verified": true,
                "device_id": accepted.device_id,
                "verified_at": verified_at,
                "expires_at": record.expires_at,
            })))
        })?
    }

    fn google_bound_challenge(
        &self,
        tx: &Tx<'_>,
        token: &str,
        hash: &str,
        response_hash: &str,
    ) -> Result<(User, Session, Challenge)> {
        let (user, session) = self.session(tx, token)?;
        let challenge = tx
            .get::<Challenge>("device_challenges", hash)?
            .ok_or_else(|| Error::bad("Verified Access challenge does not match this session"))?;
        if challenge.user_id != user.id
            || challenge.session_id != session.id
            || challenge.epoch != user.epoch
            || challenge.epoch != session.identity.epoch
        {
            return Err(Error::bad(
                "Verified Access challenge does not match this session",
            ));
        }
        if challenge.used {
            return Err(Error::bad("Verified Access challenge already used"));
        }
        let at = now();
        if challenge.expires_at <= at
            || challenge.issued_at == 0
            || at.saturating_sub(challenge.issued_at) >= device_trust::GOOGLE_CHALLENGE_TTL
        {
            return Err(Error::bad("Verified Access challenge expired"));
        }
        for (_, row) in tx.list::<Challenge>("device_challenges")? {
            if !row.response_sha256.is_empty()
                && crypto::constant_eq(&row.response_sha256, response_hash)
                && row.response_retained_until > at
            {
                return Err(Error::bad("Verified Access response was already accepted"));
            }
        }
        Ok((user, session, challenge))
    }

    fn reap_device_challenges(&self, tx: &Tx<'_>) -> Result<()> {
        let at = now();
        for (key, challenge) in tx.list::<Challenge>("device_challenges")? {
            if device_trust::drop_challenge(&challenge, at) {
                tx.delete("device_challenges", &key)?;
            }
        }
        Ok(())
    }

    fn outstanding_device_challenges(&self, tx: &Tx<'_>, user_id: &str) -> Result<u32> {
        let at = now();
        let mut active = 0u32;
        for (_, challenge) in tx.list::<Challenge>("device_challenges")? {
            if !device_trust::drop_challenge(&challenge, at)
                && challenge.user_id == user_id
                && !challenge.used
                && challenge.expires_at > at
            {
                active += 1;
            }
        }
        Ok(active)
    }

    fn verified_access_transport(&self) -> Result<Arc<dyn device_trust::VerifiedAccessTransport>> {
        #[cfg(feature = "test-support")]
        {
            let guard = self
                .verified_access_transport
                .lock()
                .map_err(|_| device_trust::verified_access::remote_unavailable())?;
            if let Some(transport) = guard.as_ref() {
                return Ok(Arc::clone(transport));
            }
        }
        Ok(Arc::new(device_trust::verified_access::ProductionTransport))
    }

    #[cfg(feature = "test-support")]
    pub fn install_verified_access_transport(
        &self,
        transport: Arc<dyn device_trust::VerifiedAccessTransport>,
    ) {
        *self
            .verified_access_transport
            .lock()
            .expect("verified access transport lock") = Some(transport);
    }
}
