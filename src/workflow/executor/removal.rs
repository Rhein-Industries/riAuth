//! One exact configured sensitive action: remove a request-pinned passkey only
//! after a live session and fresh UV proof. The ordinary management mutation,
//! receipt consumption and final run share the completion writer.
use super::*;
use crate::{
    model::Identity,
    passkey::{Credential as StoredPasskey, user_keys},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Pin {
    account: String,
    account_epoch: u64,
    session: String,
    request: String,
    run: String,
    binding: RunBinding,
    credential_id: String,
    created_at: u64,
}

fn target(tx: &Tx<'_>, user: &User, id: &str) -> Result<StoredPasskey> {
    if id.is_empty() || id.len() > 128 || !user.has_passkeys {
        return Err(Error::forbidden());
    }
    let credential = tx
        .get::<StoredPasskey>("passkeys", id)?
        .filter(|value| value.id == id && value.user_id == user.id)
        .ok_or_else(|| Error::missing("Passkey not found"))?;
    let count = user_keys(tx, &user.id)?.len();
    let local_password = crate::password::Kind::of(tx, user)? == crate::password::Kind::Local;
    if (!local_password && count <= 1) || (!local_password && user.admin && count <= 2) {
        return Err(Error::conflict(
            "Keep another usable authenticator or an independent local password",
        ));
    }
    Ok(credential)
}

pub(super) fn bind(
    tx: &Tx<'_>,
    run: &StoredRun,
    request: &mut RequestAuthority,
    user: &User,
    credential_id: &str,
    at: u64,
) -> Result<()> {
    if request.removal.is_some()
        || request.source.is_some()
        || request.authorization.is_some()
        || request.consent.is_some()
        || request.recovery.is_some()
        || request.invitation.is_some()
        || request.account != user.id
        || request.account_epoch != user.epoch
        || run.account != user.id
        || run.account_epoch != user.epoch
        || run.request != request.id
        || run.session.as_deref() != Some(request.session.as_str())
        || request.expires_at <= at
    {
        return Err(Error::forbidden());
    }
    let credential = target(tx, user, credential_id)?;
    request.removal = Some(Pin {
        account: user.id.clone(),
        account_epoch: user.epoch,
        session: request.session.clone(),
        request: request.id.clone(),
        run: run.id.clone(),
        binding: run.binding.clone(),
        credential_id: credential.id,
        created_at: credential.created_at,
    });
    Ok(())
}

pub(super) fn check(
    tx: &Tx<'_>,
    run: &StoredRun,
    request: &RequestAuthority,
    _at: u64,
) -> Result<()> {
    let Some(pin) = &request.removal else {
        return Ok(());
    };
    if request.source.is_some()
        || request.authorization.is_some()
        || request.consent.is_some()
        || request.recovery.is_some()
        || request.invitation.is_some()
        || pin.account != run.account
        || pin.account_epoch != run.account_epoch
        || run.session.as_deref() != Some(pin.session.as_str())
        || pin.session != request.session
        || pin.request != run.request
        || pin.request != request.id
        || pin.run != run.id
        || pin.binding != run.binding
    {
        return Err(Error::forbidden());
    }
    let saved = load_runtime(tx, &run.id)?;
    if !supported_configured_passkey_removal(saved.validated()?.definition())
        || saved.record.binding != run.binding
        || saved.record.request != run.request
    {
        return Err(Error::forbidden());
    }
    let user: User = tx
        .get("users", &run.account)?
        .ok_or_else(Error::forbidden)?;
    let credential = target(tx, &user, &pin.credential_id)?;
    if credential.created_at != pin.created_at {
        return Err(Error::forbidden());
    }
    Ok(())
}

fn verified_session(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &RuntimeRun,
    at: u64,
) -> Result<(User, Session, RequestAuthority, u64, bool)> {
    if !supported_configured_passkey_removal(checked.definition()) {
        return Err(Error::forbidden());
    }
    let password_totp = supported_configured_password_totp_passkey_removal(checked.definition());
    let prerequisites: &[Proof] = if password_totp {
        &[Proof::Session, Proof::Password, Proof::Totp]
    } else {
        &[Proof::Session, Proof::Passkey]
    };
    let (user, request) = authority(core, tx, &run.record, at)?;
    let RunState::Active { step, .. } = &run.record.state else {
        return Err(Error::forbidden());
    };
    if request.removal.is_none()
        || run.credential_mutation.is_some()
        || run.record.steps.len() != prerequisites.len()
        || (password_totp
            && (!request.requires_mfa
                || user.totp_secret.is_none()
                || user.totp_pending.is_some()))
        || checked.step(step).map(|value| &value.action) != Some(&Action::RemovePasskey {})
    {
        return Err(Error::forbidden());
    }
    let mut factor_at = None;
    for (index, (recorded, proof)) in run.record.steps.iter().zip(prerequisites).enumerate() {
        let proof = *proof;
        if recorded.step != checked.definition().steps[index].id
            || recorded.signal != Label::fixed("verified")
        {
            return Err(Error::forbidden());
        }
        let reference = recorded.evidence.as_deref().ok_or_else(Error::forbidden)?;
        let evidence: StoredEvidence = tx.get(EVIDENCE, reference)?.ok_or_else(Error::forbidden)?;
        super::super::evidence::check_evidence(
            &run.record,
            recorded,
            checked.step(&recorded.step).ok_or_else(Error::forbidden)?,
            proof,
            reference,
            &evidence,
            at,
            Some(Some(RECEIPT_SECONDS as u32)),
        )
        .map_err(invalid_error)?;
        evidence_authority(core, tx, &run.record, &evidence, at)?;
        if proof == Proof::Passkey || proof == Proof::Totp {
            factor_at = Some(evidence.verified_at);
        }
    }
    let mut session: Session = tx
        .get("sessions", &request.session)?
        .ok_or_else(Error::forbidden)?;
    let factor_at = factor_at.ok_or_else(Error::forbidden)?;
    if password_totp && !session.identity.mfa {
        return Err(Error::forbidden());
    }
    if password_totp {
        crate::password::require_local(tx, &user)?;
        if let Err(error) = crate::password::unlocked(tx, &user)? {
            return Err(error);
        }
    }
    let original_mfa = session.identity.mfa;
    let original_amr = session.identity.amr.clone();
    // The password-and-TOTP path freshens only this local copy, without
    // changing or elevating the stored bearer session.
    session.identity = Identity {
        user_id: user.id.clone(),
        epoch: user.epoch,
        session_id: session.id.clone(),
        auth_time: factor_at,
        mfa: if password_totp { original_mfa } else { true },
        amr: if password_totp {
            original_amr
        } else {
            vec!["webauthn".into(), "mfa".into()]
        },
        source: None,
    };
    crate::passkey::require_fresh_factor(&user, &session)?;
    Ok((user, session, request, factor_at, password_totp))
}

fn receipt(run: &RuntimeRun, request: &RequestAuthority, at: u64) -> Result<StoredEvidence> {
    let RunState::Active { step, attempt } = &run.record.state else {
        return Err(Error::forbidden());
    };
    Ok(StoredEvidence {
        id: crypto::id(),
        proof: Proof::PasskeyRemoved,
        step: step.clone(),
        attempt: *attempt,
        action: Action::RemovePasskey {},
        account: run.record.account.clone(),
        account_epoch: run.record.account_epoch,
        session: run.record.session.clone(),
        request: run.record.request.clone(),
        run: run.record.id.clone(),
        binding: run.record.binding.clone(),
        verified_at: at,
        expires_at: request.expires_at.min(at.saturating_add(RECEIPT_SECONDS)),
        consumed: false,
        source: None,
    })
}

/// This capability is constructed only from the verified server-side target
/// and all live unspent receipts in the writer that finalizes the action.
pub(super) struct Verified {
    before: StoredRun,
    evidence: StoredEvidence,
    pin: Pin,
    factor_at: u64,
    password_totp: bool,
}

impl Verified {
    pub(super) fn matches(&self, run: &StoredRun, terminal: &super::super::Terminal) -> bool {
        let mut expected = self.before.clone();
        expected.steps.push(StoredStep {
            step: self.evidence.step.clone(),
            attempt: self.evidence.attempt,
            signal: Label::fixed("completed"),
            evidence: Some(self.evidence.id.clone()),
        });
        expected == *run
            && run.request == self.pin.request
            && run.binding == self.pin.binding
            && terminal.outcome == super::super::Outcome::ActionAuthorized
    }

    pub(super) fn commit(
        self,
        core: &Core,
        tx: &Tx<'_>,
        run: &StoredRun,
        terminal: &super::super::Terminal,
        evidence: &[StoredEvidence],
    ) -> Result<mutation::Completed> {
        if !self.matches(run, terminal) || !evidence.contains(&self.evidence) {
            return Err(Error::forbidden());
        }
        let (user, request) = authority(core, tx, run, now())?;
        if request.removal.as_ref() != Some(&self.pin) {
            return Err(Error::forbidden());
        }
        let mut session: Session = tx
            .get("sessions", &self.pin.session)?
            .ok_or_else(Error::forbidden)?;
        if self.password_totp
            != supported_configured_password_totp_passkey_removal(
                &load_runtime(tx, &run.id)?.definition,
            )
            || (self.password_totp
                && (!session.identity.mfa
                    || user.totp_secret.is_none()
                    || user.totp_pending.is_some()))
        {
            return Err(Error::forbidden());
        }
        if self.password_totp {
            crate::password::require_local(tx, &user)?;
            if let Err(error) = crate::password::unlocked(tx, &user)? {
                return Err(error);
            }
        }
        let original_mfa = session.identity.mfa;
        let original_amr = session.identity.amr.clone();
        session.identity = Identity {
            user_id: user.id.clone(),
            epoch: user.epoch,
            session_id: session.id.clone(),
            auth_time: self.factor_at,
            mfa: if self.password_totp { original_mfa } else { true },
            amr: if self.password_totp {
                original_amr
            } else {
                vec!["webauthn".into(), "mfa".into()]
            },
            source: None,
        };
        crate::passkey::require_fresh_factor(&user, &session)?;
        let before = user.epoch;
        core.passkey_remove_in(tx, user, &session, &self.pin.credential_id)?;
        let after: User = tx
            .get("users", &self.pin.account)?
            .ok_or_else(Error::forbidden)?;
        if after.epoch != before.checked_add(1).ok_or_else(Error::forbidden)?
            || tx
                .get::<StoredPasskey>("passkeys", &self.pin.credential_id)?
                .is_some()
        {
            return Err(Error::forbidden());
        }
        Ok(mutation::Completed {
            account: self.pin.account,
            from_epoch: before,
            to_epoch: after.epoch,
            credential: "passkey_removed".into(),
            recovery_request: None,
            invitation_request: None,
            factors_reset: false,
        })
    }
}

impl Core {
    /// Explicitly commit the exact target pinned when the configured run began.
    /// This endpoint accepts no caller-selected target or workflow signal.
    pub fn workflow_passkey_remove(&self, token: &str, id: &str) -> Result<View> {
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            settle_time(self, tx, &checked, &mut run, now())?;
            if run.executions >= checked.definition().limits.max_executions {
                return Err(Error::conflict("Workflow execution limit reached"));
            }
            let (_, _, request, factor_at, password_totp) =
                verified_session(self, tx, &checked, &run, now())?;
            let evidence = receipt(&run, &request, now())?;
            let mutation = Verified {
                before: run.record.clone(),
                evidence: evidence.clone(),
                pin: request.removal.ok_or_else(Error::forbidden)?,
                factor_at,
                password_totp,
            };
            run.executions += 1;
            finish_step_with_mutation(
                self,
                tx,
                &checked,
                &mut run,
                Label::fixed("completed"),
                Some(evidence),
                now(),
                Some(mutation::Pending::PasskeyRemoval(mutation)),
            )?;
            run.view(&checked)
        })
    }
}
