//! The existing mail verifier's sealed recovery authority. No workflow stage,
//! caller-supplied account or persisted receipt can construct a replacement.
use super::*;
use zeroize::Zeroizing;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pin {
    hash: String,
    proof: Proof,
    verified_at: u64,
    exposure: Option<String>,
}

fn exposure(tx: &Tx<'_>, account: &str) -> Result<Option<String>> {
    crate::delegation::support_exposure(tx, account)?
        .map(|value| {
            serde_json::to_string(&value)
                .map(|value| digest(&value))
                .map_err(Error::internal)
        })
        .transpose()
}

impl Pin {
    pub(crate) fn request(&self) -> String {
        format!("reset:{}", self.hash)
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
        if self.verified_at > at
            || exposure(tx, &self.proof.user_id)? != self.exposure
            || tx.get::<Proof>("account_proofs", &self.hash)?.as_ref() != Some(&self.proof)
        {
            return Err(Error::forbidden());
        }
        reset_authority(tx, &self.hash, &self.proof, at)
    }
}

pub(crate) struct VerifiedReset {
    pin: Pin,
    plaintext: Zeroizing<String>,
    password_hash: Zeroizing<String>,
}

impl VerifiedReset {
    /// Called only after account_complete verifies the actual secret and purpose.
    pub(super) fn new(
        tx: &Tx<'_>,
        hash: String,
        proof: Proof,
        plaintext: &str,
        password_hash: &str,
    ) -> Result<Self> {
        let pin = Pin {
            exposure: exposure(tx, &proof.user_id)?,
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

    pub(crate) fn commit(self, core: &Core, tx: &Tx<'_>) -> Result<(String, u64, u64, bool)> {
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
        // Ordinary reset retains factors. Assisted recovery clears exposed
        // factors under the same pinned, independently verified mail authority.
        let factors_reset = reset_exposed_factors(tx, &mut user)?;
        user.password_hash = self.password_hash.to_string();
        user.epoch = to_epoch;
        retire_proof(tx, &self.pin.hash, ProofEnd::Used)?;
        tx.delete("account_latest", &proof_key(&user, Purpose::Reset))?;
        // Publish the new epoch before queuing downstream revocation. The caller
        // commits these writes with proof consumption and recovered run state.
        tx.put("users", &user.id, &user)?;
        tx.delete("attempts", &user.username)?;
        crate::logout::queue_user(tx, &user.id)?;
        audit(tx, &user.id, "user.account.reset", &user.id)?;
        if self.pin.expires_at() <= now() {
            return Err(proof_error(Some(ProofEnd::Expired)));
        }
        Ok((user.id, from_epoch, to_epoch, factors_reset))
    }
}
