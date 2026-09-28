//! Local password proofs, without creating an attachable login before MFA.
//! Hashing uses sign-in's real verifier outside the writer; the reserved run,
//! credential and live authority are compared again before evidence is stored.

use super::*;
use crate::{
    core::audit,
    workflow::{Category, Format, Limits, Origin, Outcome, Step, Terminal, Transition},
};
use zeroize::Zeroizing;

pub(super) const TOTP_WORKFLOW: &str = "platform-password-totp-reauthentication";

pub(super) fn definition() -> Result<Validated> {
    definition_at_revision(2)
}

/// Keep revision one's pinned path intact while new runs allow recovery codes.
pub(super) fn definition_at_revision(revision: u32) -> Result<Validated> {
    if !matches!(revision, 1 | 2) {
        return Err(Error::conflict("Workflow revision is unavailable"));
    }
    let id = |value| Id::new(value).map_err(Error::internal);
    let mut definition = Definition {
        format: Format::V1,
        id: id(TOTP_WORKFLOW)?,
        revision: 1,
        category: Category::Authentication,
        origin: Origin::Configured,
        entry: id("password")?,
        limits: Limits {
            max_duration_seconds: 600,
            max_executions: 6,
        },
        steps: [
            ("password", Action::VerifyPassword {}, "totp", 300),
            (
                "totp",
                Action::VerifyTotp {},
                "success",
                RECEIPT_SECONDS as u32,
            ),
        ]
        .into_iter()
        .map(|(name, action, next, timeout_seconds)| {
            Ok(Step {
                id: id(name)?,
                action,
                max_attempts: 3,
                timeout_seconds,
                cancellable: true,
                transitions: vec![
                    Transition {
                        on: Label::fixed("verified"),
                        when: None,
                        to: id(next)?,
                    },
                    Transition {
                        on: Label::fixed("failed"),
                        when: None,
                        to: id("denied")?,
                    },
                ],
            })
        })
        .collect::<Result<Vec<_>>>()?,
        terminals: vec![
            Terminal {
                id: id("success")?,
                outcome: Outcome::Authenticated,
                requires: vec![vec![Proof::Password, Proof::Totp]],
                max_proof_age_seconds: Some(RECEIPT_SECONDS as u32),
            },
            Terminal {
                id: id("denied")?,
                outcome: Outcome::Denied,
                requires: vec![],
                max_proof_age_seconds: None,
            },
        ],
    };
    if revision == 2 {
        recovery::extend(&mut definition)?;
    }
    validate(definition, &Environment::platform()).map_err(invalid_error)
}

struct Reserved {
    run: StoredRun,
    request: RequestAuthority,
    attempt: InFlight,
    username: String,
    hash: Zeroizing<String>,
}

fn local(tx: &Tx<'_>, checked: &Validated, user: &User, request: &RequestAuthority) -> Result<()> {
    let mfa = match checked.definition().id.as_str() {
        PASSWORD_WORKFLOW => false,
        TOTP_WORKFLOW => true,
        _ => return Err(Error::forbidden()),
    };
    crate::password::require_local(tx, user)?;
    if request.source.is_some() || request.requires_mfa != mfa || user.totp_secret.is_some() != mfa
    {
        return Err(Error::forbidden());
    }
    Ok(())
}

impl Core {
    /// Verify only this run's reserved password attempt. Successful verification
    /// produces a W03 receipt; it never produces a browser login or a session.
    pub fn workflow_password(&self, token: &str, id: &str, password: String) -> Result<View> {
        let password = Zeroizing::new(password);
        if password.len() > 1024 {
            return Err(Error::bad("Password is too long"));
        }
        let reserved = self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            settle_time(self, tx, &checked, &mut run, now())?;
            let RunState::Active { step, attempt } = &run.record.state else {
                return Ok(Err(Error::conflict("Workflow run is already final")));
            };
            if !matches!(
                checked.step(step).map(|s| &s.action),
                Some(Action::VerifyPassword {})
            ) || run.in_flight.is_some()
                || run.executions >= checked.definition().limits.max_executions
            {
                return Ok(Err(Error::conflict(
                    "Workflow step cannot accept this password",
                )));
            }
            let (user, request) = authority(self, tx, &run.record, now())?;
            local(tx, &checked, &user, &request)?;
            if let Err(error) = crate::password::unlocked(tx, &user)? {
                return Ok(Err(error));
            }
            let reservation = InFlight {
                nonce: crypto::id(),
                step: step.clone(),
                attempt: *attempt,
                step_started_at: run.step_started_at,
                source: None,
                passkey: None,
                totp: None,
                recovery_code: None,
            };
            run.in_flight = Some(reservation.clone());
            run.executions += 1;
            tx.put(RUNS, id, &run)?;
            Ok(Ok(Reserved {
                run: run.record,
                request,
                attempt: reservation,
                username: user.username,
                hash: Zeroizing::new(user.password_hash),
            }))
        })??;

        let verified = {
            let _timer = self.store.telemetry().password.timer();
            crypto::password_matches(&password, &reserved.hash)
        };
        let verified_at = now();
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            let at = now();
            settle_time(self, tx, &checked, &mut run, at)?;
            if run.record != reserved.run
                || run.in_flight.as_ref() != Some(&reserved.attempt)
                || run.step_started_at != reserved.attempt.step_started_at
            {
                return Ok(Err(Error::conflict(
                    "Workflow changed during password verification",
                )));
            }
            let (user, request) = authority(self, tx, &run.record, at)?;
            local(tx, &checked, &user, &request)?;
            if request != reserved.request
                || user.username != reserved.username
                || user.password_hash != *reserved.hash
            {
                return Err(Error::forbidden());
            }
            // A different sign-in can lock the account while hashing runs.
            if let Err(error) = crate::password::unlocked(tx, &user)? {
                fail_attempt(self, tx, &checked, &mut run, AttemptResult::Failed, at)?;
                return Ok(Err(error));
            }
            if !verified {
                record_credential_failure(tx, &user, at)?;
                audit(tx, &user.id, "workflow.password.failed", id)?;
                fail_attempt(self, tx, &checked, &mut run, AttemptResult::Failed, at)?;
                return Ok(Ok(run.view(&checked)?));
            }
            let expires_at = verified_at
                .saturating_add(RECEIPT_SECONDS)
                .min(request.expires_at);
            if verified_at < run.record.started_at
                || verified_at < run.step_started_at
                || verified_at > at
                || expires_at <= at
            {
                return Err(Error::conflict("Password verification is stale"));
            }
            let receipt = StoredEvidence {
                id: crypto::id(),
                proof: Proof::Password,
                action: Action::VerifyPassword {},
                step: reserved.attempt.step.clone(),
                attempt: reserved.attempt.attempt,
                account: user.id.clone(),
                account_epoch: user.epoch,
                session: run.record.session.clone(),
                request: request.id,
                run: id.to_owned(),
                binding: run.record.binding.clone(),
                verified_at,
                expires_at,
                consumed: false,
                source: None,
            };
            run.attempts.push(Attempt {
                step: reserved.attempt.step.clone(),
                ordinal: reserved.attempt.attempt,
                started_at: run.step_started_at,
                finished_at: at,
                result: AttemptResult::Verified,
            });
            run.in_flight = None;
            finish_step(
                self,
                tx,
                &checked,
                &mut run,
                Label::fixed("verified"),
                Some(receipt),
                at,
            )?;
            // Password alone must not replenish an MFA account's guessing budget.
            // TOTP clears it only when the whole chain commits successfully.
            if matches!(
                run.record.state,
                RunState::Finished {
                    outcome: crate::workflow::Outcome::Authenticated,
                    ..
                }
            ) {
                tx.delete("attempts", &user.username)?;
            }
            audit(tx, &user.id, "workflow.password.verified", id)?;
            Ok(Ok(run.view(&checked)?))
        })?
    }
}
