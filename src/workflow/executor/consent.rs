//! Explicit consent for one prepared OIDC request and one live bearer session.
//! A configured transition alone never supplies the consent receipt or a code.

use super::*;
use crate::{
    model::{AuthenticationTransaction, Client},
    oidc::{Authorization, needs_reauthentication},
};

const CONSENTS: &str = "workflow_consents";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Pin {
    request_hash: String,
    authentication: String,
    client: String,
    client_fingerprint: String,
    expires_at: u64,
    #[serde(default)]
    reauthentication: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bound {
    pin: Pin,
    request: Authorization,
    run: String,
    version: RunBinding,
    workflow_request: String,
    account: String,
    account_epoch: u64,
    session: String,
    completed: bool,
    expires_at: u64,
}

fn fingerprint(client: &Client) -> Result<String> {
    serde_json::to_string(client)
        .map(|value| digest(&value))
        .map_err(Error::internal)
}

pub(super) fn reject_reserved(tx: &Tx<'_>, request: &Authorization) -> Result<()> {
    let Some(transaction) = request.transaction_id.as_deref() else {
        return Ok(());
    };
    let key = digest(transaction);
    let legacy = tx.get::<Bound>(CONSENTS, &request.request_hash()?)?;
    if tx.get::<Bound>(CONSENTS, &key)?.is_some()
        || legacy.is_some_and(|bound| bound.pin.authentication == key)
    {
        return Err(Error::conflict("This authorization belongs to a workflow"));
    }
    Ok(())
}

fn bound_key(tx: &Tx<'_>, pin: &Pin) -> Result<String> {
    if tx.get::<Bound>(CONSENTS, &pin.authentication)?.is_some() {
        return Ok(pin.authentication.clone());
    }
    if tx
        .get::<Bound>(CONSENTS, &pin.request_hash)?
        .is_some_and(|bound| bound.pin.authentication == pin.authentication)
    {
        return Ok(pin.request_hash.clone());
    }
    Err(Error::forbidden())
}

pub(super) fn bind(
    tx: &Tx<'_>,
    run: &StoredRun,
    authority: &mut RequestAuthority,
    session: &Session,
    request: &Authorization,
    reauthentication: bool,
    at: u64,
) -> Result<()> {
    if request.decision.is_some()
        || request.request_binding.is_some()
        || request.has_prompt("none")
        || request.has_prompt("select_account")
        || authority.source.is_some()
        || authority.authorization.is_some()
    {
        return Err(Error::bad(
            "A prepared request needs an explicit consent decision",
        ));
    }
    let (client, _) = crate::oidc::validate_authorization(tx, request)?;
    if client.settings.source_stage.is_some()
        || needs_reauthentication(&client, request, &session.identity) != reauthentication
    {
        return Err(Error::conflict("This request needs a different workflow"));
    }
    let authentication = digest(
        request
            .transaction_id
            .as_deref()
            .ok_or_else(|| Error::bad("Prepare the authorization first"))?,
    );
    let prepared: AuthenticationTransaction = tx
        .get("authentication", &authentication)?
        .ok_or_else(Error::forbidden)?;
    let request_hash = request.request_hash()?;
    if prepared.request_hash != request_hash
        || prepared.user_id.as_deref() != Some(run.account.as_str())
        || prepared.authenticated_session.is_some()
        || prepared.source_stage.is_some()
        || prepared.expires_at <= at
    {
        return Err(Error::forbidden());
    }
    authorization::reject_reserved(tx, request)?;
    let expires_at = authority
        .expires_at
        .min(prepared.expires_at)
        .min(crate::authorization::reference_expiry(tx, request)?.unwrap_or(u64::MAX));
    if expires_at <= at {
        return Err(Error::forbidden());
    }
    let pin = Pin {
        request_hash,
        authentication,
        client: client.id.clone(),
        client_fingerprint: fingerprint(&client)?,
        expires_at,
        reauthentication,
    };
    let mut request = request.clone();
    request.transaction_id = None;
    let bound = Bound {
        pin: pin.clone(),
        request,
        run: run.id.clone(),
        version: run.binding.clone(),
        workflow_request: run.request.clone(),
        account: run.account.clone(),
        account_epoch: run.account_epoch,
        session: session.id.clone(),
        completed: false,
        expires_at,
    };
    tx.put(CONSENTS, &pin.authentication, &bound)?;
    authority.expires_at = expires_at;
    authority.consent = Some(pin);
    Ok(())
}

fn pending(tx: &Tx<'_>, run: &StoredRun, authority: &RequestAuthority, at: u64) -> Result<Bound> {
    let pin = authority.consent.as_ref().ok_or_else(Error::forbidden)?;
    let bound: Bound = tx
        .get(CONSENTS, &bound_key(tx, pin)?)?
        .ok_or_else(Error::forbidden)?;
    if bound.pin != *pin
        || bound.completed
        || pin.expires_at <= at
        || bound.expires_at != pin.expires_at
        || bound.run != run.id
        || bound.version != run.binding
        || bound.workflow_request != run.request
        || bound.account != run.account
        || bound.account_epoch != run.account_epoch
        || run.session.as_deref() != Some(bound.session.as_str())
        || bound.session != authority.session
        || bound.request.request_hash()? != pin.request_hash
        || bound.request.client_id != pin.client
        || bound.request.decision.is_some()
        || bound.request.transaction_id.is_some()
    {
        return Err(Error::forbidden());
    }
    let (client, _) = crate::oidc::validate_authorization(tx, &bound.request)?;
    if fingerprint(&client)? != pin.client_fingerprint || client.settings.source_stage.is_some() {
        return Err(Error::conflict("Authorization client changed"));
    }
    let prepared: AuthenticationTransaction = tx
        .get("authentication", &pin.authentication)?
        .ok_or_else(Error::forbidden)?;
    if prepared.request_hash != pin.request_hash
        || prepared.user_id.as_deref() != Some(run.account.as_str())
        || prepared.authenticated_session.is_some()
        || prepared.source_stage.is_some()
        || prepared.expires_at < pin.expires_at
        || prepared.expires_at <= at
    {
        return Err(Error::forbidden());
    }
    Ok(bound)
}

pub(super) fn check(
    tx: &Tx<'_>,
    run: &StoredRun,
    authority: &RequestAuthority,
    at: u64,
) -> Result<()> {
    if authority.consent.is_some() {
        pending(tx, run, authority, at)?;
    }
    Ok(())
}

/// Missing or changed client material fails the reviewed pin closed.
pub(super) fn client_policy_changed(tx: &Tx<'_>, authority: &RequestAuthority) -> Result<bool> {
    let Some(pin) = authority.consent.as_ref() else {
        return Ok(false);
    };
    let key = match bound_key(tx, pin) {
        Ok(key) => key,
        Err(error) if error.status == axum::http::StatusCode::FORBIDDEN => return Ok(true),
        Err(error) => return Err(error),
    };
    if tx.get::<Bound>(CONSENTS, &key)?.is_none() {
        return Ok(true);
    }
    let Some(client) = tx.get::<Client>("clients", &pin.client)? else {
        return Ok(true);
    };
    if !client.enabled || client.settings.source_stage.is_some() {
        return Ok(true);
    }
    Ok(fingerprint(&client)? != pin.client_fingerprint)
}

pub(super) fn complete(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &StoredRun,
    outcome: super::super::Outcome,
    evidence: &[StoredEvidence],
    at: u64,
) -> Result<Option<String>> {
    if tx
        .get::<RequestAuthority>(REQUESTS, &run.request)?
        .is_none_or(|request| request.consent.is_none())
    {
        return Ok(None);
    }
    let (_, authority) = super::authority(core, tx, run, at)?;
    let approved = outcome == super::super::Outcome::ConsentGranted;
    let reauthentication = super::super::supported_configured_passkey_consent(checked.definition());
    let expected = if reauthentication {
        &[Proof::Session, Proof::Passkey, Proof::Consent][..]
    } else {
        &[Proof::Session, Proof::Consent][..]
    };
    if !matches!(
        outcome,
        super::super::Outcome::ConsentGranted | super::super::Outcome::Denied
    ) || (approved
        && !evidence
            .iter()
            .map(|receipt| receipt.proof)
            .eq(expected.iter().copied()))
    {
        return Err(Error::forbidden());
    }
    let mut bound = pending(tx, run, &authority, at)?;
    if bound.pin.reauthentication != reauthentication {
        return Err(Error::forbidden());
    }
    let mut session: Session = tx
        .get("sessions", &bound.session)?
        .ok_or_else(Error::forbidden)?;
    if approved && reauthentication {
        let proof = &evidence[1];
        if proof.step.as_str() != "passkey"
            || !matches!(proof.action, Action::VerifyPasskey {})
            || proof.verified_at < run.started_at
            || proof.verified_at > at
            || proof.expires_at <= at
        {
            return Err(Error::forbidden());
        }
        // This fresh assurance belongs only to the OIDC grant. Never mutate
        // the bearer session or create a replacement session here.
        session.identity.auth_time = proof.verified_at;
        session.identity.mfa = true;
        session.identity.amr = vec!["webauthn".into(), "mfa".into()];
        session.identity.source = None;
        let mut prepared: AuthenticationTransaction = tx
            .get("authentication", &bound.pin.authentication)?
            .ok_or_else(Error::forbidden)?;
        prepared.authenticated_session = Some(session.id.clone());
        tx.put("authentication", &bound.pin.authentication, &prepared)?;
    }
    let mut request = bound.request.clone();
    request.decision = Some(if approved { "approve" } else { "deny" }.into());
    let response = core.authorize_workflow(
        tx,
        authorization::Accepted::for_consent(
            session.clone(),
            request,
            bound.pin.authentication.clone(),
        ),
    )?;
    if bound.expires_at <= now()
        || session.expires_at <= now()
        || (approved && evidence.iter().any(|receipt| receipt.expires_at <= now()))
    {
        return Err(Error::conflict("Consent expired during completion"));
    }
    tx.delete("authentication", &bound.pin.authentication)?;
    bound.completed = true;
    tx.put(CONSENTS, &bound_key(tx, &bound.pin)?, &bound)?;
    Ok(Some(response))
}

pub(super) fn abandon(tx: &Tx<'_>, run: &StoredRun) -> Result<()> {
    let Some(authority) = tx.get::<RequestAuthority>(REQUESTS, &run.request)? else {
        return Ok(());
    };
    let Some(pin) = authority.consent else {
        return Ok(());
    };
    let key = bound_key(tx, &pin)?;
    let mut bound: Bound = tx.get(CONSENTS, &key)?.ok_or_else(Error::forbidden)?;
    if bound.pin != pin || bound.run != run.id || bound.workflow_request != run.request {
        return Err(Error::forbidden());
    }
    bound.completed = true;
    tx.put(CONSENTS, &key, &bound)?;
    tx.delete("authentication", &pin.authentication)
}

pub(super) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (key, bound) in tx.maintenance_page::<Bound>(CONSENTS)? {
        if bound.expires_at.saturating_add(RETAIN_FINAL_SECONDS) <= at {
            tx.delete(CONSENTS, &key)?;
        }
    }
    Ok(())
}

impl Core {
    /// Consume a prepared request only after this bearer explicitly approves or
    /// denies its pinned consent step. The definition supplies no decision.
    pub fn workflow_consent_decide(&self, token: &str, id: &str, approve: bool) -> Result<View> {
        version::reject_stale_reviewed(self, id)?;
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            if run.record.state.is_final() {
                return Err(Error::conflict("Workflow run is already final"));
            }
            settle_time(self, tx, &checked, &mut run, now())?;
            if run.record.state.is_final() {
                return run.view(&checked);
            }
            let RunState::Active { step, attempt } = &run.record.state else {
                unreachable!()
            };
            let early_denial = !approve
                && super::super::supported_configured_passkey_consent(checked.definition())
                && checked.step(step).map(|s| &s.action) == Some(&Action::VerifyPasskey {});
            if !supported_configured_consent(checked.definition())
                || (!early_denial
                    && checked.step(step).map(|s| &s.action) != Some(&Action::RequestConsent {}))
                || (!early_denial && run.in_flight.is_some())
                || (run.executions >= checked.definition().limits.max_executions
                    && (!early_denial || run.in_flight.is_none()))
            {
                return Err(Error::forbidden());
            }
            let (user, request) = authority(self, tx, &run.record, now())?;
            if request.consent.is_none()
                || request.authorization.is_some()
                || request.source.is_some()
            {
                return Err(Error::forbidden());
            }
            let had_reservation = run.in_flight.is_some();
            if early_denial {
                // An explicit refusal may close the request before the user
                // completes WebAuthn. Discard its reserved challenge in the
                // same write so a late assertion cannot revive the run.
                passkey::discard(tx, &run)?;
                run.in_flight = None;
            }
            let at = now();
            let evidence = approve.then(|| StoredEvidence {
                id: crypto::id(),
                proof: Proof::Consent,
                action: Action::RequestConsent {},
                step: step.clone(),
                attempt: *attempt,
                account: user.id.clone(),
                account_epoch: user.epoch,
                session: run.record.session.clone(),
                request: request.id,
                run: run.record.id.clone(),
                binding: run.record.binding.clone(),
                verified_at: at,
                expires_at: request.expires_at.min(at.saturating_add(RECEIPT_SECONDS)),
                consumed: false,
                source: None,
            });
            run.attempts.push(Attempt {
                step: step.clone(),
                ordinal: *attempt,
                started_at: run.step_started_at,
                finished_at: at,
                result: if approve {
                    AttemptResult::Verified
                } else {
                    AttemptResult::Failed
                },
            });
            if !early_denial || !had_reservation {
                run.executions += 1;
            }
            finish_step(
                self,
                tx,
                &checked,
                &mut run,
                Label::fixed(if approve {
                    "granted"
                } else if early_denial {
                    "failed"
                } else {
                    "denied"
                }),
                evidence,
                at,
            )?;
            run.view(&checked)
        })
    }
}
