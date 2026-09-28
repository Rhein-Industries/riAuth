//! A pending invite is not a session. Its mail secret and a separate, one-use
//! registration request are both required to attach the first authenticator.
use super::*;
use crate::passkey::{Credential, credential_id, handle, webauthn_for_issuer};
use webauthn_rs::prelude::{PasskeyRegistration, RegisterPublicKeyCredential};

pub(super) const PENDING: &str = "invitation_passkey_registration";
pub(super) const SECONDS: u64 = 120;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Pending {
    pub(super) pin: Pin,
    name: String,
    state: PasskeyRegistration,
    pub(super) expires_at: u64,
}

fn mail_proof(tx: &Tx<'_>, token: &str) -> Result<(String, Proof)> {
    if !token.starts_with("ri_mail_") || token.len() > 128 {
        return Err(proof_error(None));
    }
    let hash = digest(token);
    match tx.get::<Proof>("account_proofs", &hash)? {
        Some(proof) if proof.purpose != Purpose::Invite => Err(proof_error(None)),
        Some(proof) if proof.expires_at <= now() => Err(proof_error(Some(ProofEnd::Expired))),
        Some(proof) => Ok((hash, proof)),
        None => {
            let reason = tx
                .get::<ProofOutcome>("account_proof_outcomes", &hash)?
                .filter(|outcome| {
                    outcome.purpose == Purpose::Invite && outcome.retain_until > now()
                })
                .map(|outcome| outcome.reason);
            Err(proof_error(reason))
        }
    }
}

fn owned(tx: &Tx<'_>, token: &str, ceremony: &str) -> Result<Pending> {
    let (hash, proof) = mail_proof(tx, token)?;
    let pending: Pending = tx.get(PENDING, &hash)?.ok_or_else(Error::forbidden)?;
    if pending.pin.hash != hash
        || pending.pin.proof != proof
        || !pending
            .pin
            .registration
            .as_deref()
            .is_some_and(|request| crypto::constant_eq(request, &digest(ceremony)))
    {
        return Err(Error::forbidden());
    }
    pending.pin.authority(tx, now())?;
    Ok(pending)
}

impl Core {
    /// Starting or replacing a registration never consumes the invitation or
    /// creates login authority. There is at most one pending request per invite.
    pub fn account_invitation_passkey_start(&self, token: String, name: String) -> Result<Value> {
        let token = Zeroizing::new(token);
        crate::core::validate_display(&name)?;
        if name.trim().is_empty() {
            return Err(Error::bad("Passkey name must not be blank"));
        }
        self.store.write(|tx| {
            let (hash, proof) = mail_proof(tx, &token)?;
            let mut pin = Pin::new(tx, hash.clone(), proof)?;
            let user = pin.authority(tx, now())?;
            let (challenge, state) = webauthn_for_issuer(&self.config.issuer)?
                .start_passkey_registration(handle(&user.id), &user.username, &user.display_name, None)
                .map_err(|_| Error::bad("Cannot start invitation passkey enrollment"))?;
            let mut public_key = json!(challenge);
            public_key["publicKey"]["authenticatorSelection"] = json!({
                "residentKey":"required", "requireResidentKey":true, "userVerification":"required"
            });
            let ceremony = crypto::random_token("ri_invite_enroll_");
            pin.registration = Some(digest(&ceremony));
            let expires_at = pin.expires_at();
            tx.put(PENDING, &hash, &Pending { pin, name, state, expires_at })?;
            Ok(json!({"ceremony":ceremony,"public_key":public_key,"expires_in":expires_at.saturating_sub(now())}))
        })
    }

    pub fn account_invitation_passkey_finish(
        &self,
        token: String,
        ceremony: &str,
        response: RegisterPublicKeyCredential,
    ) -> Result<Value> {
        let token = Zeroizing::new(token);
        self.store.write(|tx| {
            let pending = owned(tx, &token, ceremony)?;
            // Spend a correctly bound failed attempt, without spending its mail
            // proof. Another account/request cannot cancel this registration.
            let key = match webauthn_for_issuer(&self.config.issuer)?
                .finish_passkey_registration(&response, &pending.state)
            {
                Ok(key) => key,
                Err(_) => {
                    tx.delete(PENDING, &pending.pin.hash)?;
                    return Ok(Err(Error::bad("Passkey registration verification failed")));
                }
            };
            let id = credential_id(key.cred_id());
            if tx.get::<Credential>("passkeys", &id)?.is_some() {
                tx.delete(PENDING, &pending.pin.hash)?;
                return Ok(Err(Error::conflict("Credential is already enrolled")));
            }
            let credential = Credential {
                id,
                user_id: pending.pin.proof.user_id.clone(),
                name: pending.name,
                created_at: now(),
                counter: 0,
                key,
            };
            let verified = Verified {
                pin: pending.pin,
                credential: InitialCredential::Passkey(credential),
            };
            #[cfg(feature = "platform")]
            let result = self.complete_invitation_workflow(tx, verified)?;
            #[cfg(not(feature = "platform"))]
            let result = {
                verified.commit(self, tx)?;
                json!({"completed":true,"login_required":true})
            };
            Ok(Ok(result))
        })?
    }

    pub fn account_invitation_passkey_cancel(
        &self,
        token: String,
        ceremony: &str,
    ) -> Result<Value> {
        let token = Zeroizing::new(token);
        self.store.write(|tx| {
            let pending = owned(tx, &token, ceremony)?;
            tx.delete(PENDING, &pending.pin.hash)?;
            Ok(json!({"cancelled":true}))
        })
    }
}
