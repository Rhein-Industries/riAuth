//! One signed SAML browser request, one live SSO session, one reviewed consent run.
//! The run records the decision; only SAML's existing resume writer signs a response.

use super::*;
use crate::{
    saml::{self, Decision, Pending},
    workflow::{Outcome, supported_configured_session_consent},
};
use webauthn_rs::prelude::PublicKeyCredential;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Pin {
    interaction: String,
    workflow: String,
    client: String,
    client_fingerprint: String,
    request_hash: String,
    browser_hash: String,
    expires_at: u64,
    reauthentication: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Graph {
    Session,
    Passkey,
    PasswordTotp,
}

fn graph(definition: &Definition) -> Result<Graph> {
    if supported_configured_session_consent(definition) {
        Ok(Graph::Session)
    } else if supported_configured_passkey_consent(definition) {
        Ok(Graph::Passkey)
    } else if supported_configured_password_totp_consent(definition) {
        Ok(Graph::PasswordTotp)
    } else {
        Err(Error::conflict("SAML consent graph is unsupported"))
    }
}

/// Resolve the current exact-content approval or active configuration, never
/// falling back from an invalid active approval to a config entry.
pub(crate) fn saml_selected_definition_in(
    core: &Core,
    tx: &Tx<'_>,
    workflow: &str,
) -> Result<Definition> {
    if core.config.browser_consent_workflow.as_deref() != Some(workflow) {
        return Err(Error::forbidden());
    }
    let definition = crate::workflow::approval::configured_definition_in(core, tx, workflow)?;
    if definition.id.as_str() != workflow {
        return Err(Error::forbidden());
    }
    graph(&definition)?;
    Ok(definition)
}

pub(crate) struct BrowserSaml<'a> {
    pub workflow: &'a str,
    pub interaction: &'a str,
    pub cookie: &'a str,
    pub session: &'a Session,
    pub run_id: &'a str,
}

fn pending(tx: &Tx<'_>, run: &StoredRun, authority: &RequestAuthority, at: u64) -> Result<Pending> {
    let pin = authority
        .saml_consent
        .as_ref()
        .ok_or_else(Error::forbidden)?;
    let p: Pending = tx
        .get("saml_requests", &pin.interaction)?
        .ok_or_else(Error::forbidden)?;
    if p.id != pin.interaction
        || p.configured_consent.as_deref() != Some(pin.workflow.as_str())
        || p.configured_run.as_deref() != Some(run.id.as_str())
        || p.request.id.is_none()
        || p.client_id != pin.client
        || p.client_fingerprint != pin.client_fingerprint
        || p.browser_hash != pin.browser_hash
        || p.signed_request_hash()? != pin.request_hash
        || p.expires_at < pin.expires_at
        || pin.expires_at <= at
        || authority.expires_at != pin.expires_at
        || authority.account != run.account
        || authority.account_epoch != run.account_epoch
        || authority.session != run.session.as_deref().ok_or_else(Error::forbidden)?
        || p.configured_account.as_deref() != Some(run.account.as_str())
        || p.configured_session.as_deref() != run.session.as_deref()
        || p.configured_epoch != Some(run.account_epoch)
        || p.cancelled && !run.state.is_final()
        || p.decision.is_some() && !run.state.is_final()
    {
        return Err(Error::forbidden());
    }
    let (client, _) = saml::current_client(tx, &p)?;
    if client.settings.source_stage.is_some() {
        return Err(Error::forbidden());
    }
    saml::validate_key(tx, &client)?;
    Ok(p)
}

pub(super) fn check(
    tx: &Tx<'_>,
    run: &StoredRun,
    authority: &RequestAuthority,
    at: u64,
) -> Result<()> {
    if authority.saml_consent.is_some() {
        pending(tx, run, authority, at)?;
    }
    Ok(())
}

pub(super) fn client_policy_changed(tx: &Tx<'_>, authority: &RequestAuthority) -> Result<bool> {
    let Some(pin) = &authority.saml_consent else {
        return Ok(false);
    };
    let Some(p) = tx.get::<Pending>("saml_requests", &pin.interaction)? else {
        return Ok(true);
    };
    Ok(saml::current_client(tx, &p).is_err()
        || tx
            .get::<crate::model::Client>("clients", &pin.client)?
            .is_some_and(|client| client.settings.source_stage.is_some())
        || p.client_fingerprint != pin.client_fingerprint
        || p.signed_request_hash()? != pin.request_hash)
}

pub(super) fn complete(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &StoredRun,
    outcome: Outcome,
    evidence: &[StoredEvidence],
    at: u64,
) -> Result<()> {
    let Some(request) = tx.get::<RequestAuthority>(REQUESTS, &run.request)? else {
        return Err(Error::forbidden());
    };
    if request.saml_consent.is_none() {
        return Ok(());
    }
    if request.consent.is_some() || request.authorization.is_some() || request.source.is_some() {
        return Err(Error::forbidden());
    }
    let (_, authority) = authority(core, tx, run, at)?;
    let mut p = pending(tx, run, &authority, at)?;
    if p.decided() {
        return Err(Error::forbidden());
    }
    let (session, reauthentication) =
        consent::grant_session(tx, checked, run, &authority.session, outcome, evidence, at)?;
    if authority
        .saml_consent
        .as_ref()
        .is_none_or(|pin| pin.reauthentication != reauthentication)
    {
        return Err(Error::forbidden());
    }
    if outcome == Outcome::ConsentGranted {
        let (client, settings) = saml::current_client(tx, &p)?;
        core.saml_identity(tx, &client, &settings, &p.request, &session.identity)?;
        p.decision = Some(Decision {
            identity: session.identity,
            approve: true,
            remember: false,
        });
        p.approved_by = Some(session.id);
    } else {
        p.cancelled = true;
    }
    if p.expires_at <= now()
        || (outcome == Outcome::ConsentGranted && evidence.iter().any(|r| r.expires_at <= now()))
    {
        return Err(Error::conflict("SAML consent expired during completion"));
    }
    tx.put("saml_requests", &p.id, &p)?;
    crate::core::audit(
        tx,
        &run.account,
        if outcome == Outcome::ConsentGranted {
            "saml.approve"
        } else {
            "saml.deny"
        },
        &p.client_id,
    )
}

pub(super) fn abandon(tx: &Tx<'_>, run: &StoredRun) -> Result<()> {
    let Some(authority) = tx.get::<RequestAuthority>(REQUESTS, &run.request)? else {
        return Ok(());
    };
    let Some(pin) = authority.saml_consent else {
        return Ok(());
    };
    if let Some(mut p) = tx.get::<Pending>("saml_requests", &pin.interaction)? {
        if p.configured_run.as_deref() != Some(run.id.as_str()) {
            return Err(Error::forbidden());
        }
        if !p.decided() {
            p.cancelled = true;
            tx.put("saml_requests", &p.id, &p)?;
        }
    }
    Ok(())
}

/// The exact signed request is still live and its configured decision is final.
/// The caller issues and removes this pending row in the same storage transaction.
pub(crate) fn saml_consent_issuance_check_in(core: &Core, tx: &Tx<'_>, p: &Pending) -> Result<()> {
    let run_id = p.configured_run.as_deref().ok_or_else(Error::forbidden)?;
    let run = load_runtime(tx, run_id)?;
    if !matches!(
        run.record.state,
        RunState::Finished {
            outcome: Outcome::ConsentGranted,
            ..
        }
    ) {
        return Err(Error::forbidden());
    }
    if let Some(failure) = version::reviewed_policy_failure(core, tx, &run)? {
        return Err(Error::conflict(failure.message()));
    }
    let selected = saml_selected_definition_in(
        core,
        tx,
        p.configured_consent
            .as_deref()
            .ok_or_else(Error::forbidden)?,
    )?;
    let checked = validate(selected, &Environment::platform()).map_err(invalid_error)?;
    if checked.definition() != &run.definition || checked.binding() != run.record.binding {
        return Err(Error::conflict("Workflow policy changed"));
    }
    let (_, authority) = authority(core, tx, &run.record, now())?;
    let pin = authority
        .saml_consent
        .as_ref()
        .ok_or_else(Error::forbidden)?;
    if pin.interaction != p.id
        || pin.workflow
            != p.configured_consent
                .as_deref()
                .ok_or_else(Error::forbidden)?
    {
        return Err(Error::forbidden());
    }
    let decision = p
        .decision
        .as_ref()
        .filter(|d| d.approve && !d.remember)
        .ok_or_else(Error::forbidden)?;
    let session: Session = tx
        .get("sessions", &authority.session)?
        .ok_or_else(Error::forbidden)?;
    if decision.identity.user_id != run.record.account
        || decision.identity.epoch != run.record.account_epoch
        || decision.identity.session_id != authority.session
        || (pin.reauthentication
            && (decision.identity.auth_time < run.record.started_at
                || decision.identity.auth_time > now()
                || !decision.identity.mfa))
        || (!pin.reauthentication
            && (decision.identity.auth_time != session.identity.auth_time
                || decision.identity.mfa != session.identity.mfa))
    {
        return Err(Error::forbidden());
    }
    Ok(())
}

/// Begin only after an SP-signed AuthnRequest, live HttpOnly browser SSO,
/// and the exact configured graph have been checked in this same writer.
pub(crate) fn saml_browser_start_in(
    core: &Core,
    tx: &Tx<'_>,
    p: &mut Pending,
    cookie: &str,
    session: &Session,
) -> Result<String> {
    let workflow = p
        .configured_consent
        .as_deref()
        .ok_or_else(Error::forbidden)?;
    if core.config.browser_consent_workflow.as_deref() != Some(workflow)
        || p.request.id.is_none()
        || p.decided()
        || p.configured_run.is_some()
        || p.expires_at <= now()
    {
        return Err(Error::forbidden());
    }
    let definition = saml_selected_definition_in(core, tx, workflow)?;
    let checked = validate(definition, &Environment::platform()).map_err(invalid_error)?;
    let graph = graph(checked.definition())?;
    let (client, _) = saml::current_client(tx, p)?;
    if client.settings.source_stage.is_some() {
        return Err(Error::forbidden());
    }
    saml::validate_key(tx, &client)?;
    let reauthentication =
        core.saml_reauthentication_needed(&client, &p.request, &session.identity);
    if (graph == Graph::Session) == reauthentication {
        return Err(Error::forbidden());
    }
    let user = core.identity_user(tx, &session.identity)?;
    if p.configured_account
        .as_ref()
        .is_some_and(|id| id != &user.id)
        || p.configured_session
            .as_ref()
            .is_some_and(|id| id != &session.id)
        || p.configured_epoch.is_some_and(|epoch| epoch != user.epoch)
    {
        return Err(Error::forbidden());
    }
    if graph == Graph::PasswordTotp {
        crate::password::require_local(tx, &user)?;
        if user.totp_secret.is_none() || user.totp_pending.is_some() {
            return Err(Error::forbidden());
        }
    }
    let live = core
        .browser_session(tx, Some(cookie))?
        .ok_or_else(Error::unauthorized)?;
    if live.id != session.id || live.token_hash != session.token_hash {
        return Err(Error::unauthorized());
    }
    if let Some(active_id) = tx.get::<String>(ACTIVE_SESSIONS, &session.id)? {
        let mut active = load_runtime(tx, &active_id)?;
        if !active.record.state.is_final() {
            let active_checked = active.validated()?;
            settle_time(core, tx, &active_checked, &mut active, now())?;
        }
        if !active.record.state.is_final() {
            return Err(Error::conflict(
                "An authorization workflow is already active",
            ));
        }
        tx.delete(ACTIVE_SESSIONS, &session.id)?;
    }
    let reviewed = version::review_pin(core, tx, &checked, true)?;
    let at = now();
    let expires_at = at
        .saturating_add(u64::from(checked.definition().limits.max_duration_seconds))
        .min(session.expires_at)
        .min(p.expires_at);
    if expires_at <= at {
        return Err(Error::forbidden());
    }
    let run_id = crypto::id();
    let request_id = crypto::id();
    let entry = checked
        .entry()
        .ok_or_else(|| Error::internal("Missing workflow entry"))?;
    let record = StoredRun {
        id: run_id.clone(),
        account: user.id.clone(),
        account_epoch: user.epoch,
        session: Some(session.id.clone()),
        request: request_id.clone(),
        binding: checked.binding(),
        started_at: at,
        state: RunState::Active {
            step: entry.id.clone(),
            attempt: 1,
        },
        steps: vec![],
    };
    let run = RuntimeRun {
        record,
        definition: checked.definition().clone(),
        step_started_at: at,
        executions: 0,
        attempts: vec![],
        in_flight: None,
        authorization_response: None,
        credential_mutation: None,
        reviewed,
        reviewed_failure: None,
    };
    let request = RequestAuthority {
        id: request_id.clone(),
        run: run_id.clone(),
        account: user.id.clone(),
        account_epoch: user.epoch,
        session: session.id.clone(),
        token_hash: session.token_hash.clone(),
        browser_hash: Some(digest(cookie)),
        expires_at,
        requires_mfa: graph == Graph::PasswordTotp,
        source: None,
        authorization: None,
        consent: None,
        saml_consent: Some(Pin {
            interaction: p.id.clone(),
            workflow: workflow.to_owned(),
            client: p.client_id.clone(),
            client_fingerprint: p.client_fingerprint.clone(),
            request_hash: p.signed_request_hash()?,
            browser_hash: p.browser_hash.clone(),
            expires_at,
            reauthentication,
        }),
        recovery: None,
        invitation: None,
        removal: None,
    };
    p.configured_account = Some(user.id.clone());
    p.configured_session = Some(session.id.clone());
    p.configured_epoch = Some(user.epoch);
    p.configured_run = Some(run_id.clone());
    tx.put("saml_requests", &p.id, p)?;
    tx.put(REQUESTS, &request_id, &request)?;
    tx.put(RUNS, &run_id, &run)?;
    tx.put(ACTIVE_SESSIONS, &session.id, &run_id)?;
    version::track_account_run(tx, &user.id, &run_id)?;
    let mut run = run;
    enrollment::resume_session(core, tx, &checked, &mut run, at)?;
    Ok(run_id)
}

fn browser_run(
    core: &Core,
    tx: &Tx<'_>,
    binding: &BrowserSaml<'_>,
) -> Result<(RuntimeRun, Validated)> {
    let run = load_runtime(tx, binding.run_id)?;
    let checked = run.validated()?;
    let selected = saml_selected_definition_in(core, tx, binding.workflow)?;
    if checked.definition() != &selected
        || run.record.account != binding.session.identity.user_id
        || run.record.account_epoch != binding.session.identity.epoch
        || run.record.session.as_deref() != Some(binding.session.id.as_str())
    {
        return Err(Error::forbidden());
    }
    let live = core
        .browser_session(tx, Some(binding.cookie))?
        .ok_or_else(Error::unauthorized)?;
    if live.id != binding.session.id || live.token_hash != binding.session.token_hash {
        return Err(Error::unauthorized());
    }
    let request: RequestAuthority = tx
        .get(REQUESTS, &run.record.request)?
        .ok_or_else(Error::forbidden)?;
    if request.browser_hash.as_deref() != Some(digest(binding.cookie).as_str())
        || request.saml_consent.as_ref().is_none_or(|pin| {
            pin.interaction != binding.interaction || pin.workflow != binding.workflow
        })
    {
        return Err(Error::forbidden());
    }
    authority(core, tx, &run.record, now())?;
    Ok((run, checked))
}

pub(crate) fn saml_browser_owner(
    core: &Core,
    tx: &Tx<'_>,
    binding: &BrowserSaml<'_>,
    record: &StoredRun,
) -> Result<()> {
    let (run, _) = browser_run(core, tx, binding)?;
    if run.record != *record {
        return Err(Error::forbidden());
    }
    Ok(())
}

pub(crate) fn saml_browser_stage_in(
    core: &Core,
    tx: &Tx<'_>,
    binding: &BrowserSaml<'_>,
) -> Result<&'static str> {
    let (run, checked) = browser_run(core, tx, binding)?;
    match &run.record.state {
        RunState::Active { step, .. } => match checked.step(step).map(|step| &step.action) {
            Some(Action::VerifyPassword {}) => Ok("password"),
            Some(Action::VerifyTotp {}) => Ok("totp"),
            Some(Action::VerifyPasskey {}) => Ok("passkey"),
            Some(Action::RequestConsent {}) => Ok("consent"),
            _ => Err(Error::forbidden()),
        },
        _ => Err(Error::forbidden()),
    }
}

pub(crate) fn saml_browser_code_in(
    core: &Core,
    tx: &Tx<'_>,
    binding: &BrowserSaml<'_>,
    code: String,
) -> Result<View> {
    let (mut run, checked) = browser_run(core, tx, binding)?;
    totp::browser_code_in(core, tx, &mut run, &checked, code)
}

pub(crate) fn saml_browser_passkey_start_in(
    core: &Core,
    tx: &Tx<'_>,
    binding: &BrowserSaml<'_>,
) -> Result<serde_json::Value> {
    let (mut run, checked) = browser_run(core, tx, binding)?;
    if graph(checked.definition())? != Graph::Passkey {
        return Err(Error::forbidden());
    }
    let challenge = passkey::challenge_in(core, tx, &mut run)?.ok_or_else(Error::forbidden)?;
    let ceremony = run
        .in_flight
        .as_ref()
        .and_then(|attempt| attempt.passkey.as_deref())
        .ok_or_else(Error::forbidden)?;
    Ok(
        serde_json::json!({"workflow":challenge.workflow,"ceremony":ceremony,"public_key":challenge.public_key}),
    )
}

pub(crate) fn saml_browser_passkey_finish_in(
    core: &Core,
    tx: &Tx<'_>,
    binding: &BrowserSaml<'_>,
    ceremony: &str,
    response: PublicKeyCredential,
) -> Result<View> {
    let (mut run, checked) = browser_run(core, tx, binding)?;
    if graph(checked.definition())? != Graph::Passkey
        || run
            .in_flight
            .as_ref()
            .and_then(|attempt| attempt.passkey.as_deref())
            != Some(ceremony)
    {
        return Err(Error::forbidden());
    }
    passkey::finish_in(core, tx, &mut run, response)
}

pub(crate) fn saml_browser_passkey_cancel_in(
    core: &Core,
    tx: &Tx<'_>,
    binding: &BrowserSaml<'_>,
    ceremony: &str,
) -> Result<View> {
    let (mut run, checked) = browser_run(core, tx, binding)?;
    if graph(checked.definition())? != Graph::Passkey {
        return Err(Error::forbidden());
    }
    passkey::cancel_in(core, tx, &mut run, ceremony)
}

pub(crate) fn saml_browser_decide_in(
    core: &Core,
    tx: &Tx<'_>,
    binding: &BrowserSaml<'_>,
    approve: bool,
) -> Result<View> {
    let (mut run, checked) = browser_run(core, tx, binding)?;
    consent::decide_loaded(core, tx, &checked, &mut run, approve)
}
