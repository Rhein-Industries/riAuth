//! Sealed mutation capabilities share the existing completion writer. Only
//! supported real verifiers can cross an account epoch; stored receipts cannot.
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Completed {
    pub account: String,
    pub from_epoch: u64,
    pub to_epoch: u64,
    pub credential: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_request: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invitation_request: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub factors_reset: bool,
}

pub(super) enum Pending {
    Enrollment(enrollment::Verified),
    PasskeyRemoval(removal::Verified),
    PasswordReset(reset::Verified),
    Invitation(invitation::Verified),
}

impl Pending {
    pub(super) fn matches(&self, run: &StoredRun, terminal: &super::super::Terminal) -> bool {
        match self {
            Self::Enrollment(verified) => verified.matches(run, terminal),
            Self::PasskeyRemoval(verified) => verified.matches(run, terminal),
            Self::PasswordReset(verified) => verified.matches(run, terminal),
            Self::Invitation(verified) => verified.matches(run, terminal),
        }
    }

    pub(super) fn commit(
        self,
        core: &Core,
        tx: &Tx<'_>,
        run: &StoredRun,
        terminal: &super::super::Terminal,
        evidence: &[StoredEvidence],
    ) -> Result<Completed> {
        match self {
            Self::Enrollment(verified) => {
                let mutation = verified.commit(core, tx, run, terminal, evidence)?;
                Ok(Completed {
                    account: mutation.account,
                    from_epoch: mutation.from_epoch,
                    to_epoch: mutation.to_epoch,
                    credential: mutation.credential,
                    recovery_request: None,
                    invitation_request: None,
                    factors_reset: false,
                })
            }
            Self::PasskeyRemoval(verified) => verified.commit(core, tx, run, terminal, evidence),
            Self::PasswordReset(verified) => verified.commit(core, tx, run, terminal, evidence),
            Self::Invitation(verified) => verified.commit(core, tx, run, terminal, evidence),
        }
    }
}
