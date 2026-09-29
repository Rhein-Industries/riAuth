//! Reviewed pin for runs loaded from `config.workflows`.
//!
//! The pin sits beside [`super::RuntimeRun`], not inside `RunBinding`. A seal
//! commits only when the writer returns `Ok`: `Store::write` drops the
//! transaction on `Err`.

use super::*;

pub(super) const REVIEWED: &str = "workflow_reviewed";
pub(super) const ACCOUNT_RUNS: &str = "workflow_account_runs";
const MAX_ACCOUNT_RUNS: usize = 32;
const POLICY_PREFIX: &str = "riauth.workflow-reviewed/v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct ReviewedPin {
    pub(super) revision: u32,
    pub(super) fingerprint: String,
    pub(super) policy: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AccountRuns {
    runs: BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReviewedFailure {
    PolicyChanged,
    RolledBack,
    UserDisabled,
}

impl ReviewedFailure {
    pub(super) fn code(self) -> &'static str {
        match self {
            Self::PolicyChanged => "policy_changed",
            Self::RolledBack => "rolled_back",
            Self::UserDisabled => "user_disabled",
        }
    }

    pub(super) fn message(self) -> &'static str {
        match self {
            Self::PolicyChanged => "Workflow policy changed",
            Self::RolledBack => "Workflow version was rolled back",
            Self::UserDisabled => "Workflow account is disabled",
        }
    }
}

fn policy_digest(active: bool, id: &str, revision: u32, fingerprint: &str) -> String {
    digest(&format!(
        "{POLICY_PREFIX}\n{active}\n{id}\n{revision}\n{fingerprint}"
    ))
}

/// Record the high-water when this start's definition is the active configured
/// entry. A same-revision policy edit is not adopted. A lower revision is rejected.
pub(super) fn review_pin(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    required: bool,
) -> Result<Option<ReviewedPin>> {
    if !required {
        return Ok(None);
    }
    let id = checked.definition().id.as_str();
    let Some(entry) = core.config.workflows.get(id) else {
        return Err(Error::conflict("Workflow policy changed"));
    };
    if !entry.active
        || &entry.definition != checked.definition()
        || checked.fingerprint() != entry.definition.fingerprint()
    {
        return Err(Error::conflict("Workflow policy changed"));
    }
    let pin = ReviewedPin {
        revision: checked.definition().revision,
        fingerprint: checked.fingerprint().to_owned(),
        policy: policy_digest(
            true,
            id,
            checked.definition().revision,
            checked.fingerprint(),
        ),
    };
    match tx.get::<ReviewedPin>(REVIEWED, id)? {
        None => tx.put(REVIEWED, id, &pin)?,
        Some(stored) if stored.revision < pin.revision => tx.put(REVIEWED, id, &pin)?,
        Some(stored) if stored == pin => {}
        Some(stored) if stored.revision == pin.revision => {
            return Err(Error::conflict("Workflow policy changed"));
        }
        Some(_) => return Err(Error::conflict("Workflow version was rolled back")),
    }
    Ok(Some(pin))
}

pub(super) fn reviewed_failure(
    core: &Core,
    tx: &Tx<'_>,
    run: &RuntimeRun,
) -> Result<Option<ReviewedFailure>> {
    if run.record.state.is_final() {
        return Ok(None);
    }
    let Some(pin) = &run.reviewed else {
        return Ok(None);
    };
    let id = run.definition.id.as_str();
    if run.definition.revision != pin.revision || run.definition.fingerprint() != pin.fingerprint {
        return Ok(Some(ReviewedFailure::PolicyChanged));
    }
    let user = tx.get::<User>("users", &run.record.account)?;
    if user.as_ref().is_none_or(|user| !user.enabled) {
        return Ok(Some(ReviewedFailure::UserDisabled));
    }
    let Some(entry) = core.config.workflows.get(id) else {
        return Ok(Some(ReviewedFailure::PolicyChanged));
    };
    if !entry.active || entry.definition.id.as_str() != id {
        return Ok(Some(ReviewedFailure::PolicyChanged));
    }
    let live_fingerprint = entry.definition.fingerprint();
    let Some(stored) = tx.get::<ReviewedPin>(REVIEWED, id)? else {
        return Ok(Some(ReviewedFailure::PolicyChanged));
    };
    if entry.definition.revision < pin.revision || entry.definition.revision < stored.revision {
        return Ok(Some(ReviewedFailure::RolledBack));
    }
    let live_policy = policy_digest(
        entry.active,
        id,
        entry.definition.revision,
        &live_fingerprint,
    );
    if entry.definition.revision > pin.revision
        || live_fingerprint != pin.fingerprint
        || live_policy != pin.policy
        || &stored != pin
    {
        return Ok(Some(ReviewedFailure::PolicyChanged));
    }
    let Some(request) = tx.get::<RequestAuthority>(REQUESTS, &run.record.request)? else {
        return Ok(Some(ReviewedFailure::PolicyChanged));
    };
    if authorization::client_policy_changed(tx, &request)?
        || consent::client_policy_changed(tx, &request)?
    {
        return Ok(Some(ReviewedFailure::PolicyChanged));
    }
    Ok(None)
}

fn sealed_state(definition: &Definition) -> RunState {
    definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == crate::workflow::Outcome::Denied)
        .map(|terminal| RunState::Finished {
            terminal: terminal.id.clone(),
            outcome: crate::workflow::Outcome::Denied,
        })
        .unwrap_or(RunState::Cancelled {})
}

pub(super) fn seal_reviewed(
    tx: &Tx<'_>,
    run: &mut RuntimeRun,
    failure: ReviewedFailure,
) -> Result<()> {
    if run.record.state.is_final() {
        return Ok(());
    }
    run.reviewed_failure = Some(failure.code().to_owned());
    close(tx, run, sealed_state(&run.definition))
}

/// Own write so the seal survives a later `Err` from the verifier.
pub(super) fn reviewed_outcome(core: &Core, id: &str) -> Result<Option<ReviewedFailure>> {
    core.store.write(|tx| {
        let mut run = load_runtime(tx, id)?;
        let Some(failure) = reviewed_failure(core, tx, &run)? else {
            return Ok(None);
        };
        seal_reviewed(tx, &mut run, failure)?;
        Ok(Some(failure))
    })
}

pub(super) fn reject_stale_reviewed(core: &Core, id: &str) -> Result<()> {
    if let Some(failure) = reviewed_outcome(core, id)? {
        return Err(Error::conflict(failure.message()));
    }
    Ok(())
}

pub(super) fn reject_if_stale(core: &Core, tx: &Tx<'_>, run_id: &str) -> Result<()> {
    let Some(run) = tx.get::<RuntimeRun>(RUNS, run_id)? else {
        return Ok(());
    };
    if let Some(failure) = reviewed_failure(core, tx, &run)? {
        return Err(Error::conflict(failure.message()));
    }
    Ok(())
}

/// Seal inside a verifier write and commit that write via `Ok(Err)`.
pub(super) fn commit_reviewed_seal(
    core: &Core,
    tx: &Tx<'_>,
    run: &mut RuntimeRun,
) -> Result<Option<Error>> {
    let Some(failure) = reviewed_failure(core, tx, run)? else {
        return Ok(None);
    };
    seal_reviewed(tx, run, failure)?;
    Ok(Some(Error::conflict(failure.message())))
}

pub(super) fn seal_stale_session(core: &Core, token: &str) -> Result<()> {
    core.store.write(|tx| {
        let (user, session) = core.session(tx, token)?;
        let Some(active_id) = tx.get::<String>(ACTIVE_SESSIONS, &session.id)? else {
            return Ok(());
        };
        let Some(mut run) = tx.get::<RuntimeRun>(RUNS, &active_id)? else {
            return Ok(());
        };
        if run.record.id != active_id
            || run.record.account != user.id
            || run.record.session.as_deref() != Some(session.id.as_str())
            || run.reviewed.is_none()
            || run.record.state.is_final()
        {
            return Ok(());
        }
        if let Some(failure) = reviewed_failure(core, tx, &run)? {
            seal_reviewed(tx, &mut run, failure)?;
        }
        Ok(())
    })
}

pub(super) fn track_account_run(tx: &Tx<'_>, account: &str, run_id: &str) -> Result<()> {
    let mut index = tx
        .get::<AccountRuns>(ACCOUNT_RUNS, account)?
        .unwrap_or_default();
    if index.runs.contains(run_id) {
        return Ok(());
    }
    if index.runs.len() >= MAX_ACCOUNT_RUNS {
        return Err(Error::conflict("Too many active workflow runs"));
    }
    index.runs.insert(run_id.to_owned());
    tx.put(ACCOUNT_RUNS, account, &index)
}

pub(super) fn untrack_account_run(tx: &Tx<'_>, account: &str, run_id: &str) -> Result<()> {
    let Some(mut index) = tx.get::<AccountRuns>(ACCOUNT_RUNS, account)? else {
        return Ok(());
    };
    if !index.runs.remove(run_id) {
        return Ok(());
    }
    if index.runs.is_empty() {
        tx.delete(ACCOUNT_RUNS, account)
    } else {
        tx.put(ACCOUNT_RUNS, account, &index)
    }
}

/// Seal every open pinned run for an account whose user row just became disabled.
pub(crate) fn seal_disabled_account(tx: &Tx<'_>, user_id: &str) -> Result<()> {
    let Some(index) = tx.get::<AccountRuns>(ACCOUNT_RUNS, user_id)? else {
        return Ok(());
    };
    for id in index.runs.iter() {
        let Some(mut run) = tx.get::<RuntimeRun>(RUNS, id)? else {
            continue;
        };
        if run.reviewed.is_some() && !run.record.state.is_final() && run.record.account == user_id {
            seal_reviewed(tx, &mut run, ReviewedFailure::UserDisabled)?;
        }
    }
    if tx.get::<AccountRuns>(ACCOUNT_RUNS, user_id)?.is_some() {
        tx.delete(ACCOUNT_RUNS, user_id)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config,
        model::{NewUser, UserPatch},
        workflow::Outcome,
    };

    const PASSWORD: &str = "configured-workflow-fixture-password";

    fn definition(revision: u32) -> Definition {
        crate::workflow::parse(
            serde_json::json!({
                "format": "riauth.workflow/v1",
                "id": "local-password",
                "revision": revision,
                "category": "authentication",
                "origin": "configured",
                "entry": "password",
                "limits": {"max_duration_seconds": 600, "max_executions": 3},
                "steps": [{
                    "id": "password",
                    "action": {"type": "verify_password"},
                    "max_attempts": 3,
                    "timeout_seconds": 120,
                    "cancellable": true,
                    "transitions": [
                        {"on": "verified", "to": "success"},
                        {"on": "failed", "to": "denied"}
                    ]
                }],
                "terminals": [
                    {"id": "success", "outcome": "authenticated", "requires": [["password"]]},
                    {"id": "denied", "outcome": "denied", "requires": []}
                ]
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
    }

    fn start_core(revision: u32) -> (tempfile::TempDir, Core, String) {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config {
            data_dir: dir.path().to_owned(),
            ..Default::default()
        };
        config.workflows.insert(
            "local-password".into(),
            crate::workflow::ConfiguredWorkflow {
                active: true,
                definition: definition(revision),
            },
        );
        config.validate().unwrap();
        let core = Core::initialize(
            config,
            NewUser {
                username: "admin".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let token = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        (dir, core, token)
    }

    fn saved(core: &Core, id: &str) -> RuntimeRun {
        core.store.write(|tx| load_runtime(tx, id)).unwrap()
    }

    fn assert_denied(run: &RuntimeRun, code: &str) {
        assert!(
            matches!(
                run.record.state,
                RunState::Finished {
                    outcome: Outcome::Denied,
                    ..
                }
            ),
            "{:?}",
            run.record.state
        );
        assert_eq!(run.reviewed_failure.as_deref(), Some(code));
        assert!(run.record.steps.is_empty());
        assert!(run.authorization_response.is_none());
        assert_eq!(run.executions, 0);
    }

    #[test]
    fn configured_policy_change_seals_and_rejects_replay_and_a_new_start() {
        let (_dir, mut core, token) = start_core(1);
        let started = core
            .workflow_configured_start(&token, "local-password")
            .unwrap();
        let original_policy = started.reviewed_policy.clone().unwrap();
        let original_fingerprint = started.binding.fingerprint.clone();
        assert_eq!(started.reviewed_revision, Some(1));
        assert_eq!(started.reviewed_failure, None);

        core.config
            .workflows
            .get_mut("local-password")
            .unwrap()
            .definition
            .steps[0]
            .timeout_seconds = 30;

        let rejected = core
            .workflow_password(&token, &started.id, PASSWORD.into())
            .unwrap_err();
        assert_eq!(rejected.message, "Workflow policy changed");
        assert_denied(&saved(&core, &started.id), "policy_changed");
        assert_eq!(
            saved(&core, &started.id).record.binding.fingerprint,
            original_fingerprint
        );

        let resumed = core.workflow_resume(&token, &started.id).unwrap();
        assert!(matches!(
            resumed.state,
            RunState::Finished {
                outcome: Outcome::Denied,
                ..
            }
        ));
        assert_eq!(resumed.reviewed_failure.as_deref(), Some("policy_changed"));
        assert!(
            core.workflow_password(&token, &started.id, PASSWORD.into())
                .is_err()
        );
        assert_denied(&saved(&core, &started.id), "policy_changed");

        let restart = core
            .workflow_configured_start(&token, "local-password")
            .unwrap_err();
        assert_eq!(restart.message, "Workflow policy changed");
        let stored = core
            .store
            .get::<ReviewedPin>(REVIEWED, "local-password")
            .unwrap()
            .unwrap();
        assert_eq!(stored.revision, 1);
        assert_eq!(stored.fingerprint, original_fingerprint);
        assert_eq!(stored.policy, original_policy);

        core.config
            .workflows
            .get_mut("local-password")
            .unwrap()
            .definition
            .revision = 2;
        let replacement = core
            .workflow_configured_start(&token, "local-password")
            .unwrap();
        assert_ne!(replacement.id, started.id);
        assert_eq!(replacement.reviewed_revision, Some(2));
        core.config
            .workflows
            .get_mut("local-password")
            .unwrap()
            .active = false;
        let withdrawn = core.workflow_resume(&token, &replacement.id).unwrap();
        assert_eq!(
            withdrawn.reviewed_failure.as_deref(),
            Some("policy_changed")
        );
        assert_eq!(
            core.workflow_configured_start(&token, "local-password")
                .unwrap_err()
                .message,
            "Configured workflow is unavailable"
        );
        assert_denied(&saved(&core, &replacement.id), "policy_changed");

        core.config
            .workflows
            .get_mut("local-password")
            .unwrap()
            .active = true;
        let again = core.workflow_resume(&token, &replacement.id).unwrap();
        assert!(matches!(
            again.state,
            RunState::Finished {
                outcome: Outcome::Denied,
                ..
            }
        ));
        let fresh = core
            .workflow_configured_start(&token, "local-password")
            .unwrap();
        assert_ne!(fresh.id, replacement.id);
        let authenticated = core
            .workflow_password(&token, &fresh.id, PASSWORD.into())
            .unwrap();
        assert!(matches!(
            authenticated.state,
            RunState::Finished {
                outcome: Outcome::Authenticated,
                ..
            }
        ));
        assert!(
            core.workflow_password(&token, &replacement.id, PASSWORD.into())
                .is_err()
        );
        assert_denied(&saved(&core, &replacement.id), "policy_changed");
    }

    #[test]
    fn configured_higher_revision_restarts_a_new_run_and_denies_the_old_one() {
        let (_dir, mut core, token) = start_core(1);
        let started = core
            .workflow_configured_start(&token, "local-password")
            .unwrap();
        core.config
            .workflows
            .get_mut("local-password")
            .unwrap()
            .definition
            .revision = 3;
        let rejected = core
            .workflow_password(&token, &started.id, PASSWORD.into())
            .unwrap_err();
        assert_eq!(rejected.message, "Workflow policy changed");
        let resumed = core.workflow_resume(&token, &started.id).unwrap();
        assert_eq!(resumed.reviewed_failure.as_deref(), Some("policy_changed"));
        assert_denied(&saved(&core, &started.id), "policy_changed");

        let restarted = core
            .workflow_configured_start(&token, "local-password")
            .unwrap();
        assert_ne!(restarted.id, started.id);
        assert_eq!(restarted.reviewed_revision, Some(3));
        assert_ne!(restarted.binding.fingerprint, started.binding.fingerprint);
        let authenticated = core
            .workflow_password(&token, &restarted.id, PASSWORD.into())
            .unwrap();
        assert!(matches!(
            authenticated.state,
            RunState::Finished {
                outcome: Outcome::Authenticated,
                ..
            }
        ));
        assert!(
            core.workflow_password(&token, &started.id, PASSWORD.into())
                .is_err()
        );
        assert_denied(&saved(&core, &started.id), "policy_changed");
        assert_eq!(
            core.store
                .get::<ReviewedPin>(REVIEWED, "local-password")
                .unwrap()
                .unwrap()
                .revision,
            3
        );
    }

    #[test]
    fn configured_rollback_seals_and_stays_denied_after_a_later_revision() {
        let (_dir, mut core, token) = start_core(2);
        let started = core
            .workflow_configured_start(&token, "local-password")
            .unwrap();
        assert_eq!(started.reviewed_revision, Some(2));
        core.config
            .workflows
            .get_mut("local-password")
            .unwrap()
            .definition = definition(1);
        let rejected = core
            .workflow_password(&token, &started.id, PASSWORD.into())
            .unwrap_err();
        assert_eq!(rejected.message, "Workflow version was rolled back");
        let resumed = core.workflow_resume(&token, &started.id).unwrap();
        assert_eq!(resumed.reviewed_failure.as_deref(), Some("rolled_back"));
        assert_denied(&saved(&core, &started.id), "rolled_back");
        assert!(
            core.workflow_password(&token, &started.id, PASSWORD.into())
                .is_err()
        );

        let blocked = core
            .workflow_configured_start(&token, "local-password")
            .unwrap_err();
        assert_eq!(blocked.message, "Workflow version was rolled back");
        assert_eq!(
            core.store
                .get::<ReviewedPin>(REVIEWED, "local-password")
                .unwrap()
                .unwrap()
                .revision,
            2
        );

        core.config
            .workflows
            .get_mut("local-password")
            .unwrap()
            .definition = definition(3);
        let restarted = core
            .workflow_configured_start(&token, "local-password")
            .unwrap();
        assert_ne!(restarted.id, started.id);
        assert_eq!(restarted.reviewed_revision, Some(3));
        let authenticated = core
            .workflow_password(&token, &restarted.id, PASSWORD.into())
            .unwrap();
        assert!(matches!(
            authenticated.state,
            RunState::Finished {
                outcome: Outcome::Authenticated,
                ..
            }
        ));
        let old = core.workflow_resume(&token, &started.id).unwrap();
        assert_eq!(old.reviewed_failure.as_deref(), Some("rolled_back"));
        assert_denied(&saved(&core, &started.id), "rolled_back");
        assert!(
            core.workflow_password(&token, &started.id, PASSWORD.into())
                .is_err()
        );
    }

    #[test]
    fn disabled_account_seals_pinned_runs_and_a_new_login_cannot_resume_them() {
        let (_dir, core, admin) = start_core(1);
        core.create_user(
            &admin,
            NewUser {
                username: "alice".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Alice".into(),
                admin: false,
            },
        )
        .unwrap();
        let alice = core.login("alice".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let started = core
            .workflow_configured_start(&alice, "local-password")
            .unwrap();
        assert_eq!(started.reviewed_revision, Some(1));
        let account = saved(&core, &started.id).record.account.clone();
        assert!(
            core.store
                .get::<AccountRuns>(ACCOUNT_RUNS, &account)
                .unwrap()
                .is_some_and(|index| index.runs.contains(&started.id))
        );

        core.update_user(
            &admin,
            "alice",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
        assert_denied(&saved(&core, &started.id), "user_disabled");
        assert!(
            core.store
                .get::<AccountRuns>(ACCOUNT_RUNS, &account)
                .unwrap()
                .is_none()
        );
        assert!(
            core.workflow_password(&alice, &started.id, PASSWORD.into())
                .is_err()
        );
        assert!(core.workflow_resume(&alice, &started.id).is_err());
        assert_denied(&saved(&core, &started.id), "user_disabled");

        core.update_user(
            &admin,
            "alice",
            UserPatch {
                enabled: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        let alice = core.login("alice".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(core.workflow_resume(&alice, &started.id).is_err());
        assert!(
            core.workflow_password(&alice, &started.id, PASSWORD.into())
                .is_err()
        );
        assert_denied(&saved(&core, &started.id), "user_disabled");
        let fresh = core
            .workflow_configured_start(&alice, "local-password")
            .unwrap();
        assert_ne!(fresh.id, started.id);
        assert_eq!(fresh.reviewed_revision, Some(1));
        let authenticated = core
            .workflow_password(&alice, &fresh.id, PASSWORD.into())
            .unwrap();
        assert!(matches!(
            authenticated.state,
            RunState::Finished {
                outcome: Outcome::Authenticated,
                ..
            }
        ));
        assert_eq!(
            core.store
                .get::<ReviewedPin>(REVIEWED, "local-password")
                .unwrap()
                .unwrap()
                .revision,
            1
        );
        assert_denied(&saved(&core, &started.id), "user_disabled");
    }

    #[test]
    fn missing_retained_pin_seals_the_open_run() {
        let (_dir, core, token) = start_core(1);
        let started = core
            .workflow_configured_start(&token, "local-password")
            .unwrap();
        assert!(matches!(
            saved(&core, &started.id).record.state,
            RunState::Active { .. }
        ));
        core.store
            .write(|tx| tx.delete(REVIEWED, "local-password"))
            .unwrap();
        assert!(
            core.store
                .get::<ReviewedPin>(REVIEWED, "local-password")
                .unwrap()
                .is_none()
        );
        let rejected = core
            .workflow_password(&token, &started.id, PASSWORD.into())
            .unwrap_err();
        assert_eq!(rejected.message, "Workflow policy changed");
        assert_denied(&saved(&core, &started.id), "policy_changed");
        let resumed = core.workflow_resume(&token, &started.id).unwrap();
        assert_eq!(resumed.reviewed_failure.as_deref(), Some("policy_changed"));
        assert!(
            core.workflow_password(&token, &started.id, PASSWORD.into())
                .is_err()
        );
        assert_denied(&saved(&core, &started.id), "policy_changed");
    }

    #[test]
    fn reviewed_pin_and_denial_survive_core_reopen() {
        let (dir, core, token) = start_core(1);
        let started = core
            .workflow_configured_start(&token, "local-password")
            .unwrap();
        let original_policy = started.reviewed_policy.clone().unwrap();
        let original_fingerprint = started.binding.fingerprint.clone();
        let mut config = core.config.clone();
        config
            .workflows
            .get_mut("local-password")
            .unwrap()
            .definition
            .steps[0]
            .timeout_seconds = 30;
        drop(core);
        assert!(dir.path().join("riauth.redb").is_file());

        let core = Core::open(config.clone()).unwrap();
        let stored = core
            .store
            .get::<ReviewedPin>(REVIEWED, "local-password")
            .unwrap()
            .unwrap();
        assert_eq!(stored.revision, 1);
        assert_eq!(stored.fingerprint, original_fingerprint);
        assert_eq!(stored.policy, original_policy);
        assert!(matches!(
            saved(&core, &started.id).record.state,
            RunState::Active { .. }
        ));
        let rejected = core
            .workflow_password(&token, &started.id, PASSWORD.into())
            .unwrap_err();
        assert_eq!(rejected.message, "Workflow policy changed");
        assert_denied(&saved(&core, &started.id), "policy_changed");
        drop(core);

        let core = Core::open(config).unwrap();
        let stored = core
            .store
            .get::<ReviewedPin>(REVIEWED, "local-password")
            .unwrap()
            .unwrap();
        assert_eq!(stored.revision, 1);
        assert_eq!(stored.fingerprint, original_fingerprint);
        assert_eq!(stored.policy, original_policy);
        let resumed = core.workflow_resume(&token, &started.id).unwrap();
        assert_eq!(resumed.reviewed_failure.as_deref(), Some("policy_changed"));
        assert!(
            core.workflow_password(&token, &started.id, PASSWORD.into())
                .is_err()
        );
        assert_denied(&saved(&core, &started.id), "policy_changed");
        let blocked = core
            .workflow_configured_start(&token, "local-password")
            .unwrap_err();
        assert_eq!(blocked.message, "Workflow policy changed");
        let stored = core
            .store
            .get::<ReviewedPin>(REVIEWED, "local-password")
            .unwrap()
            .unwrap();
        assert_eq!(stored.revision, 1);
        assert_eq!(stored.fingerprint, original_fingerprint);
        assert_eq!(stored.policy, original_policy);
    }
}
