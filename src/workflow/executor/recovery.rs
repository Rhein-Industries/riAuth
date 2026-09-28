//! Revision two adds a bounded recovery-code alternative to canonical MFA.

use super::*;
use crate::workflow::{Step, Transition};

pub(super) fn extend(definition: &mut Definition) -> Result<()> {
    let id = |value| Id::new(value).map_err(Error::internal);
    let primary = definition.steps[0]
        .action
        .proof(&Label::fixed("verified"))
        .ok_or_else(Error::forbidden)?;
    definition.revision = 2;
    definition.limits.max_executions += 3;
    definition.steps[1].transitions[1].to = id("recovery-code")?;
    definition.steps.push(Step {
        id: id("recovery-code")?,
        action: Action::VerifyRecoveryCode {},
        max_attempts: 3,
        timeout_seconds: RECEIPT_SECONDS as u32,
        cancellable: true,
        transitions: vec![
            Transition {
                on: Label::fixed("verified"),
                when: None,
                to: id("success")?,
            },
            Transition {
                on: Label::fixed("failed"),
                when: None,
                to: id("denied")?,
            },
        ],
    });
    definition.terminals[0].requires = vec![
        vec![primary, Proof::Totp],
        vec![primary, Proof::RecoveryCode],
    ];
    Ok(())
}
