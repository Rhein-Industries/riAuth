//! Durable description of a verified passkey enrollment mutation.
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Mutation {
    pub account: String,
    pub from_epoch: u64,
    pub to_epoch: u64,
    pub credential: String,
}

impl VerifiedRegistration {
    pub(crate) fn mutation(&self) -> Result<Mutation> {
        Ok(Mutation {
            account: self.credential.user_id.clone(),
            from_epoch: self.account_epoch,
            to_epoch: self
                .account_epoch
                .checked_add(1)
                .ok_or_else(Error::forbidden)?,
            credential: self.credential.id.clone(),
        })
    }
}
