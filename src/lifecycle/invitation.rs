//! Shared, sealed authority for an invitation's first credential.
//! Possession of a stored workflow receipt cannot construct this capability.
use super::*;
use zeroize::Zeroizing;

mod passkey;

fn exposure(tx: &Tx<'_>, account: &str) -> Result<Option<String>> {
    crate::delegation::credential_exposure(tx, account)?
        .map(|value| {
            serde_json::to_string(&value)
                .map(|value| digest(&value))
                .map_err(Error::internal)
        })
        .transpose()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pin {
    hash: String,
    proof: Proof,
    reservation: InvitationReservation,
    verified_at: u64,
    exposure: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    registration: Option<String>,
}

impl Pin {
    #[cfg(feature = "platform")]
    pub(crate) fn request(&self) -> String {
        match &self.registration {
            Some(request) => format!("accept:{}:{request}", self.hash),
            None => format!("accept:{}", self.hash),
        }
    }
    #[cfg(feature = "platform")]
    pub(crate) fn hash(&self) -> &str {
        &self.hash
    }
    #[cfg(feature = "platform")]
    pub(crate) fn verified_at(&self) -> u64 {
        self.verified_at
    }
    pub(crate) fn expires_at(&self) -> u64 {
        if self.registration.is_some() {
            self.proof
                .expires_at
                .min(self.verified_at.saturating_add(passkey::SECONDS))
        } else {
            self.proof.expires_at
        }
    }

    pub(crate) fn is_passkey(&self) -> bool {
        self.registration.is_some()
    }

    fn new(tx: &Tx<'_>, hash: String, proof: Proof) -> Result<Self> {
        let user: User = tx
            .get("users", &proof.user_id)?
            .ok_or_else(Error::forbidden)?;
        let pin = Self {
            reservation: pending_invitation_reservation(tx, &user)?.ok_or_else(Error::forbidden)?,
            exposure: exposure(tx, &user.id)?,
            hash,
            proof,
            verified_at: now(),
            registration: None,
        };
        pin.authority(tx, now())?;
        Ok(pin)
    }

    pub(crate) fn authority(&self, tx: &Tx<'_>, at: u64) -> Result<User> {
        let user: User = tx
            .get("users", &self.proof.user_id)?
            .ok_or_else(Error::forbidden)?;
        if self.proof.purpose != Purpose::Invite
            || self.verified_at > at
            || self.expires_at() <= at
            || user.id != self.proof.user_id
            || user.epoch != self.proof.epoch
            || user.email.as_deref() != Some(self.proof.email.as_str())
            || tx.get::<String>("usernames", &user.username)?.as_deref() != Some(&user.id)
            || tx.get::<Proof>("account_proofs", &self.hash)?.as_ref() != Some(&self.proof)
            || tx
                .get::<String>("account_latest", &proof_key(&user, Purpose::Invite))?
                .as_deref()
                != Some(self.hash.as_str())
            || pending_invitation_reservation(tx, &user)?.as_ref() != Some(&self.reservation)
            || crate::password::Kind::of(tx, &user)? != crate::password::Kind::None
            || crate::passkey::passkey_count(tx, &user.id)? != 0
            || user.totp_last_step.is_some()
            || exposure(tx, &user.id)? != self.exposure
            || !acceptance_authorized(tx, &user, &self.proof)?
        {
            return Err(Error::forbidden());
        }
        if self.registration.is_some() {
            let pending: passkey::Pending = tx
                .get(passkey::PENDING, &self.hash)?
                .ok_or_else(Error::forbidden)?;
            if pending.pin != *self || pending.expires_at != self.expires_at() {
                return Err(Error::forbidden());
            }
        }
        Ok(user)
    }
}

pub(crate) struct Verified {
    pin: Pin,
    credential: InitialCredential,
}

#[expect(
    clippy::large_enum_variant,
    reason = "Short lived transaction state remains inline"
)]
enum InitialCredential {
    Password {
        plaintext: Zeroizing<String>,
        hash: Zeroizing<String>,
    },
    Passkey(crate::passkey::Credential),
}

impl Verified {
    /// Only lifecycle's secret/purpose verifier may create an enrollment ticket.
    pub(in crate::lifecycle) fn new(
        tx: &Tx<'_>,
        hash: String,
        proof: Proof,
        plaintext: &str,
        password_hash: &str,
    ) -> Result<Self> {
        Ok(Self {
            pin: Pin::new(tx, hash, proof)?,
            credential: InitialCredential::Password {
                plaintext: Zeroizing::new(plaintext.to_owned()),
                hash: Zeroizing::new(password_hash.to_owned()),
            },
        })
    }

    #[cfg(feature = "platform")]
    pub(crate) fn pin(&self) -> &Pin {
        &self.pin
    }

    #[cfg(feature = "platform")]
    pub(crate) fn credential(&self) -> &str {
        match &self.credential {
            InitialCredential::Password { .. } => "password",
            InitialCredential::Passkey(key) => &key.id,
        }
    }

    pub(crate) fn commit(self, core: &Core, tx: &Tx<'_>) -> Result<(String, u64, u64)> {
        let mut user = self.pin.authority(tx, now())?;
        let from_epoch = user.epoch;
        let to_epoch = from_epoch.checked_add(1).ok_or_else(Error::forbidden)?;
        match &self.credential {
            InitialCredential::Password { plaintext, hash } => {
                if self.pin.is_passkey() {
                    return Err(Error::forbidden());
                }
                crate::identity::password_history::accept(
                    tx,
                    core.config.password_history,
                    &user.id,
                    &user.password_hash,
                    plaintext,
                    hash,
                )?;
            }
            InitialCredential::Passkey(key) => {
                if !self.pin.is_passkey()
                    || key.user_id != user.id
                    || tx
                        .get::<crate::passkey::Credential>("passkeys", &key.id)?
                        .is_some()
                {
                    return Err(Error::forbidden());
                }
            }
        }
        let actor = creator(
            tx,
            self.pin
                .proof
                .creator
                .as_deref()
                .ok_or_else(Error::forbidden)?,
        )?;
        // Keep the existing management writer's live scoped authority and group
        // checks. It activates the account and advances exactly one epoch.
        crate::management::accept_invitation(
            &core.config,
            tx,
            &actor,
            &mut user,
            &self.pin.proof.groups,
        )?;
        if user.epoch != to_epoch {
            return Err(Error::conflict("Invitation credential epoch changed"));
        }
        match self.credential {
            InitialCredential::Password { hash, .. } => user.password_hash = hash.to_string(),
            InitialCredential::Passkey(key) => {
                user.has_passkeys = true;
                tx.put("passkeys", &key.id, &key)?;
                audit(tx, &user.id, "passkey.enroll", &key.id)?;
            }
        }
        retire_proof(tx, &self.pin.hash, ProofEnd::Used)?;
        tx.delete("account_latest", &proof_key(&user, Purpose::Invite))?;
        tx.delete("invitation_reservations", &user.id)?;
        // Publish E+1 before downstream revocation. Unlike independent recovery,
        // invitation acceptance must never clear M04 credential exposure or
        // manufacture independent human elevation provenance.
        tx.put("users", &user.id, &user)?;
        tx.delete("attempts", &user.username)?;
        crate::logout::queue_user(tx, &user.id)?;
        audit(tx, &user.id, "user.account.accept", &user.id)?;
        if self.pin.expires_at() <= now() {
            return Err(proof_error(Some(ProofEnd::Expired)));
        }
        Ok((user.id, from_epoch, to_epoch))
    }
}
