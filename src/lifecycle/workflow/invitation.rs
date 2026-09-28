//! The real invitation mail verifier's capability for first-password enrollment.
//! Possession of a stored workflow receipt cannot construct this capability.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pin {
    hash: String,
    proof: Proof,
    reservation: InvitationReservation,
    verified_at: u64,
    exposure: Option<String>,
}

impl Pin {
    pub(crate) fn request(&self) -> String {
        format!("accept:{}", self.hash)
    }
    pub(crate) fn hash(&self) -> &str {
        &self.hash
    }
    pub(crate) fn verified_at(&self) -> u64 {
        self.verified_at
    }
    pub(crate) fn expires_at(&self) -> u64 {
        self.proof.expires_at
    }

    pub(crate) fn authority(&self, tx: &Tx<'_>, at: u64) -> Result<User> {
        let user: User = tx
            .get("users", &self.proof.user_id)?
            .ok_or_else(Error::forbidden)?;
        if self.proof.purpose != Purpose::Invite
            || self.verified_at > at
            || self.proof.expires_at <= at
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
        Ok(user)
    }
}

pub(crate) struct Verified {
    pin: Pin,
    plaintext: Zeroizing<String>,
    password_hash: Zeroizing<String>,
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
        let user: User = tx
            .get("users", &proof.user_id)?
            .ok_or_else(Error::forbidden)?;
        let pin = Pin {
            reservation: pending_invitation_reservation(tx, &user)?.ok_or_else(Error::forbidden)?,
            exposure: exposure(tx, &user.id)?,
            hash,
            proof,
            verified_at: now(),
        };
        pin.authority(tx, now())?;
        Ok(Self {
            pin,
            plaintext: Zeroizing::new(plaintext.to_owned()),
            password_hash: Zeroizing::new(password_hash.to_owned()),
        })
    }

    pub(crate) fn pin(&self) -> &Pin {
        &self.pin
    }

    pub(crate) fn commit(self, core: &Core, tx: &Tx<'_>) -> Result<(String, u64, u64)> {
        let mut user = self.pin.authority(tx, now())?;
        let from_epoch = user.epoch;
        let to_epoch = from_epoch.checked_add(1).ok_or_else(Error::forbidden)?;
        crate::identity::password_history::accept(
            tx,
            core.config.password_history,
            &user.id,
            &user.password_hash,
            &self.plaintext,
            &self.password_hash,
        )?;
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
        crate::management::accept_invitation(tx, &actor, &mut user, &self.pin.proof.groups)?;
        if user.epoch != to_epoch {
            return Err(Error::conflict("Invitation credential epoch changed"));
        }
        user.password_hash = self.password_hash.to_string();
        retire_proof(tx, &self.pin.hash, ProofEnd::Used)?;
        tx.delete("account_latest", &proof_key(&user, Purpose::Invite))?;
        tx.delete("invitation_reservations", &user.id)?;
        // Publish E+1 before downstream revocation. Unlike independent recovery,
        // invitation acceptance must never clear an M04 support-exposure marker.
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
