//! Source reauthentication and exact configured first-passkey enrollment use
//! the same signed verifier, executor and completion store.

use super::*;
use crate::workflow::{Category, Format, Limits, Origin, Outcome, Step, Terminal, Transition};

#[derive(Serialize)]
pub struct SourceStart {
    pub workflow: View,
    pub authorization_url: String,
}

pub(super) const TOTP_WORKFLOW: &str = "platform-source-totp-reauthentication";
pub(super) const SOURCE_WORKFLOW: &str = "platform-source-reauthentication";

fn bind_registration(checked: Validated, pin: &upstream::Pin) -> Result<Validated> {
    checked
        .with_source_registration(SourceRegistrationBinding {
            source: pin.source.clone(),
            fingerprint: pin.fingerprint.clone(),
        })
        .map_err(invalid_error)
}

pub(super) fn session_matches(
    session: &Session,
    pin: &upstream::Pin,
    link: Option<&str>,
) -> bool {
    session.identity.source.as_ref().is_some_and(|source| {
        source.id == pin.source.as_str()
            && source.fingerprint == pin.fingerprint
            && link.is_none_or(|expected| source.link == expected)
    })
}

pub(super) fn definition(source: &Id, totp: bool) -> Result<Validated> {
    definition_at_revision(source, totp, if totp { 2 } else { 1 })
}

pub(super) fn definition_at_revision(source: &Id, totp: bool, revision: u32) -> Result<Validated> {
    if revision != 1 && !(totp && revision == 2) {
        return Err(Error::conflict("Workflow revision is unavailable"));
    }
    if !cfg!(feature = "platform") {
        return Err(Error::forbidden());
    }
    let id = |value| Id::new(value).map_err(Error::internal);
    let mut definition = Definition {
        format: Format::V1,
        id: id(SOURCE_WORKFLOW)?,
        revision: 1,
        category: Category::Authentication,
        origin: Origin::Configured,
        entry: id("source")?,
        limits: Limits {
            max_duration_seconds: 600,
            max_executions: 1,
        },
        steps: vec![Step {
            id: id("source")?,
            action: Action::VerifySource {
                source: source.clone(),
            },
            max_attempts: 1,
            timeout_seconds: 600,
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
        }],
        terminals: vec![
            Terminal {
                id: id("success")?,
                outcome: Outcome::Authenticated,
                requires: vec![vec![Proof::Source]],
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
    // Preserve the exact original source-only definition for pinned runs.
    // Accounts with a local factor get a distinct server-owned path whose
    // success requires both proofs, regardless of any upstream ACR/AMR.
    if totp {
        definition.id = id(TOTP_WORKFLOW)?;
        definition.limits.max_executions = 4;
        definition.steps[0].transitions[0].to = id("totp")?;
        definition.steps.push(Step {
            id: id("totp")?,
            action: Action::VerifyTotp {},
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
        definition.terminals[0].requires = vec![vec![Proof::Source, Proof::Totp]];
        if revision == 2 {
            recovery::extend(&mut definition)?;
        }
    }
    let mut environment = Environment::platform();
    environment.sources.insert(source.clone());
    validate(definition, &environment).map_err(invalid_error)
}

fn binding(run: &RuntimeRun, reservation: &InFlight) -> Result<upstream::Binding> {
    Ok(upstream::Binding {
        run: run.record.id.clone(),
        account: run.record.account.clone(),
        account_epoch: run.record.account_epoch,
        session: run.record.session.clone().ok_or_else(Error::forbidden)?,
        request: run.record.request.clone(),
        definition: run.record.binding.clone(),
        step: reservation.step.clone(),
        attempt: reservation.attempt,
        reservation: reservation.nonce.clone(),
        started_at: reservation.step_started_at,
    })
}

pub(super) fn discard(tx: &Tx<'_>, run: &RuntimeRun) -> Result<()> {
    if let Some(reservation) = &run.in_flight
        && let Some(attempt) = &reservation.source
    {
        upstream::discard(tx, attempt, &binding(run, reservation)?)?;
    }
    Ok(())
}

impl Core {
    /// The session receipt and upstream reservation are created in one writer.
    /// Any failure leaves no active run that could block this bearer.
    pub fn workflow_configured_source_passkey_start(
        &self,
        token: &str,
        workflow: &str,
    ) -> Result<SourceStart> {
        let configured = self
            .config
            .workflows
            .get(workflow)
            .filter(|entry| entry.active)
            .ok_or_else(|| Error::missing("Configured workflow is unavailable"))?;
        let source = configured_source_first_passkey_enrollment(&configured.definition)
            .ok_or_else(|| Error::conflict("Configured workflow is unavailable"))?;
        let mut environment = Environment::platform();
        environment.sources.insert(source.clone());
        let checked = validate(configured.definition.clone(), &environment).map_err(invalid_error)?;
        if checked.definition().id.as_str() != workflow {
            return Err(Error::conflict("Configured workflow is unavailable"));
        }
        version::seal_stale_session(self, token)?;
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            let pin = upstream::pin(tx, &source)?;
            upstream::authority(tx, &pin, &user)?;
            let checked = bind_registration(checked.clone(), &pin)?;
            if user.has_passkeys
                || crate::passkey::passkey_count(tx, &user.id)? != 0
                || user.totp_secret.is_some()
                || user.totp_pending.is_some()
                || crate::password::Kind::of(tx, &user)? != crate::password::Kind::None
                || !session_matches(&session, &pin, None)
            {
                return Err(Error::forbidden());
            }
            let reviewed = version::review_pin(self, tx, &checked, true)?;
            let at = now();
            if let Some(active_id) = tx.get::<String>(ACTIVE_SESSIONS, &session.id)? {
                if let Some(mut active) = tx.get::<RuntimeRun>(RUNS, &active_id)? {
                    owned(self, tx, token, &active.record)?;
                    let pinned = active.validated()?;
                    settle_time(self, tx, &pinned, &mut active, at)?;
                    if !active.record.state.is_final() {
                        return Err(Error::conflict("A workflow is already active for this session"));
                    }
                }
                tx.delete(ACTIVE_SESSIONS, &session.id)?;
            }
            let run_id = crypto::id();
            let request_id = crypto::id();
            let expires_at = at
                .saturating_add(u64::from(checked.definition().limits.max_duration_seconds))
                .min(session.expires_at);
            let mut run = RuntimeRun {
                record: StoredRun {
                    id: run_id.clone(),
                    account: user.id.clone(),
                    account_epoch: user.epoch,
                    session: Some(session.id.clone()),
                    request: request_id.clone(),
                    binding: checked.binding(),
                    started_at: at,
                    state: RunState::Active {
                        step: checked.definition().entry.clone(),
                        attempt: 1,
                    },
                    steps: vec![],
                },
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
                account: user.id,
                account_epoch: user.epoch,
                session: session.id.clone(),
                token_hash: digest(token),
                expires_at,
                requires_mfa: false,
                source: Some(pin.clone()),
                authorization: None,
                consent: None,
                recovery: None,
                invitation: None,
                removal: None,
            };
            tx.put(REQUESTS, &request_id, &request)?;
            tx.put(RUNS, &run_id, &run)?;
            tx.put(ACTIVE_SESSIONS, &session.id, &run_id)?;
            if run.reviewed.is_some() {
                version::track_account_run(tx, &run.record.account, &run_id)?;
            }
            enrollment::resume_session(self, tx, &checked, &mut run, at)?;
            let RunState::Active { step, attempt } = &run.record.state else {
                return Err(Error::forbidden());
            };
            if step.as_str() != "source" || *attempt != 1 {
                return Err(Error::forbidden());
            }
            let mut reservation = InFlight {
                nonce: crypto::id(),
                step: step.clone(),
                attempt: *attempt,
                step_started_at: run.step_started_at,
                source: None,
                passkey: None,
                totp: None,
                recovery_code: None,
                enrollment: None,
                totp_enrollment: None,
            };
            let (attempt, authorization_url) = self.begin_workflow_source(
                tx,
                &pin,
                binding(&run, &reservation)?,
                expires_at,
            )?;
            reservation.source = Some(attempt);
            run.in_flight = Some(reservation);
            run.executions += 1;
            tx.put(RUNS, &run_id, &run)?;
            Ok(SourceStart {
                workflow: run.view(&checked)?,
                authorization_url,
            })
        })
    }

    /// Start one bounded OIDC or SAML reauthentication for the exact live bearer
    /// session. The caller chooses an enabled source, never a workflow or proof.
    pub fn workflow_source_start(&self, token: &str, source: &str) -> Result<SourceStart> {
        self.start_source_authorization_workflow(token, source, None)
    }

    /// Bind the downstream request before reserving the upstream verifier.
    pub fn workflow_source_authorization_start(
        &self,
        token: &str,
        source: &str,
        request: crate::oidc::Authorization,
    ) -> Result<SourceStart> {
        self.start_source_authorization_workflow(token, source, Some(request))
    }

    fn start_source_authorization_workflow(
        &self,
        token: &str,
        source: &str,
        authorization: Option<crate::oidc::Authorization>,
    ) -> Result<SourceStart> {
        let source = Id::new(source).map_err(Error::bad)?;
        version::seal_stale_session(self, token)?;
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            let pin = upstream::pin(tx, &source)?;
            upstream::authority(tx, &pin, &user)?;
            let requires_mfa = user.totp_secret.is_some();
            let checked = bind_registration(definition(&source, requires_mfa)?, &pin)?;
            let at = now();
            if let Some(active_id) = tx.get::<String>(ACTIVE_SESSIONS, &session.id)? {
                if let Some(mut active) = tx.get::<RuntimeRun>(RUNS, &active_id)? {
                    owned(self, tx, token, &active.record)?;
                    let validated = active.validated()?;
                    settle_time(self, tx, &validated, &mut active, at)?;
                    if !active.record.state.is_final() {
                        return Err(Error::conflict(
                            "A workflow is already active for this session",
                        ));
                    }
                }
                tx.delete(ACTIVE_SESSIONS, &session.id)?;
            }
            let run_id = crypto::id();
            let request_id = crypto::id();
            let expires_at = at.saturating_add(600).min(session.expires_at);
            let mut run = RuntimeRun {
                record: StoredRun {
                    id: run_id.clone(),
                    account: user.id.clone(),
                    account_epoch: user.epoch,
                    session: Some(session.id.clone()),
                    request: request_id.clone(),
                    binding: checked.binding(),
                    started_at: at,
                    state: RunState::Active {
                        step: checked.definition().entry.clone(),
                        attempt: 1,
                    },
                    steps: vec![],
                },
                definition: checked.definition().clone(),
                step_started_at: at,
                executions: 1,
                attempts: vec![],
                in_flight: None,
                authorization_response: None,
                credential_mutation: None,
                reviewed: None,
                reviewed_failure: None,
            };
            let mut reservation = InFlight {
                nonce: crypto::id(),
                step: checked.definition().entry.clone(),
                attempt: 1,
                step_started_at: at,
                source: None,
                passkey: None,
                totp: None,
                recovery_code: None,
                enrollment: None,
                totp_enrollment: None,
            };
            let mut request = RequestAuthority {
                id: request_id.clone(),
                run: run_id.clone(),
                account: user.id,
                account_epoch: user.epoch,
                session: session.id.clone(),
                token_hash: digest(token),
                expires_at,
                requires_mfa,
                source: Some(pin.clone()),
                authorization: None,
                consent: None,
                recovery: None,
                invitation: None,
                removal: None,
            };
            if let Some(authorization) = &authorization {
                authorization::bind(tx, &run.record, &mut request, authorization, at)?;
            }
            let (attempt, authorization_url) = self.begin_workflow_source(
                tx,
                &pin,
                binding(&run, &reservation)?,
                request.expires_at,
            )?;
            reservation.source = Some(attempt);
            run.in_flight = Some(reservation);
            tx.put(REQUESTS, &request_id, &request)?;
            tx.put(RUNS, &run_id, &run)?;
            tx.put(ACTIVE_SESSIONS, &session.id, &run_id)?;
            Ok(SourceStart {
                workflow: run.view(&checked)?,
                authorization_url,
            })
        })
    }

    /// Poll only the verifier transaction reserved by this run. Consumption,
    /// evidence creation, live-authority checks and finalization share a writer.
    pub fn workflow_source_finish(&self, token: &str, id: &str) -> Result<View> {
        version::reject_stale_reviewed(self, id)?;
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            if run.record.state.is_final() {
                return Err(Error::conflict("Workflow run is already final"));
            }
            settle_time(self, tx, &checked, &mut run, now())?;
            let RunState::Active { step, attempt } = &run.record.state else {
                return run.view(&checked);
            };
            let reservation = run.in_flight.clone().ok_or_else(Error::forbidden)?;
            let source_attempt = reservation.source.as_ref().ok_or_else(Error::forbidden)?;
            let (user, request) = authority(self, tx, &run.record, now())?;
            let pin = request.source.as_ref().ok_or_else(Error::forbidden)?;
            if reservation.step != *step
                || reservation.passkey.is_some()
                || reservation.totp.is_some()
                || reservation.recovery_code.is_some()
                || reservation.enrollment.is_some()
                || reservation.totp_enrollment.is_some()
                || reservation.attempt != *attempt
                || reservation.step_started_at != run.step_started_at
                || checked.step(step).map(|s| &s.action)
                    != Some(&Action::VerifySource {
                        source: pin.source.clone(),
                    })
                || ((user.totp_secret.is_some() || request.requires_mfa)
                    && checked.definition().id.as_str() != TOTP_WORKFLOW)
                || (checked.definition().id.as_str() == TOTP_WORKFLOW
                    && (user.totp_secret.is_none() || !request.requires_mfa))
                || (configured_source_first_passkey_enrollment(checked.definition()).is_some()
                    && (user.has_passkeys
                        || crate::passkey::passkey_count(tx, &user.id)? != 0
                        || user.totp_pending.is_some()
                        || crate::password::Kind::of(tx, &user)? != crate::password::Kind::None))
            {
                return Err(Error::forbidden());
            }
            let at = now();
            let verified = upstream::consume(
                tx,
                pin,
                source_attempt,
                &binding(&run, &reservation)?,
                &user,
                at,
            )?;
            let (source, auth_time, expires_at) = match verified {
                upstream::Verification::Pending => return run.view(&checked),
                upstream::Verification::Failed => {
                    fail_attempt(self, tx, &checked, &mut run, AttemptResult::Failed, at)?;
                    return run.view(&checked);
                }
                upstream::Verification::Verified {
                    authority,
                    auth_time,
                    expires_at,
                } => (authority, auth_time, expires_at),
            };
            if configured_source_first_passkey_enrollment(checked.definition()).is_some() {
                let session: Session = tx
                    .get("sessions", &request.session)?
                    .ok_or_else(Error::forbidden)?;
                if !session_matches(&session, pin, Some(&source.link)) {
                    return Err(Error::forbidden());
                }
            }
            let receipt = StoredEvidence {
                id: crypto::id(),
                proof: Proof::Source,
                step: step.clone(),
                attempt: *attempt,
                action: Action::VerifySource {
                    source: pin.source.clone(),
                },
                account: run.record.account.clone(),
                account_epoch: run.record.account_epoch,
                session: run.record.session.clone(),
                request: run.record.request.clone(),
                run: run.record.id.clone(),
                binding: run.record.binding.clone(),
                verified_at: auth_time,
                expires_at: expires_at
                    .min(request.expires_at)
                    .min(at.saturating_add(RECEIPT_SECONDS)),
                consumed: false,
                source: Some(source),
            };
            run.attempts.push(Attempt {
                step: step.clone(),
                ordinal: *attempt,
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
            run.view(&checked)
        })
    }
}
