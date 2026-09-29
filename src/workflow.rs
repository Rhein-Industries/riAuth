//! Canonical typed, versioned workflow definitions and durable execution.
//!
//! The W01 definition model and W03 completion seam describe authentication,
//! enrollment, recovery, consent and sensitive-action journeys. The W02 executor
//! consumes only [`Validated`] definitions and exposes password, passkey and upstream source
//! reauthentication for live bearer sessions, plus configured local verifier and
//! request-bound consent paths. The held W07 host in [`extension`] is an
//! in-process contract and is not called. [`extension_gate`] runs one configured
//! custom stage in a killable Wasmi child process on Platform. See `docs/workflows.md`.

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

#[cfg(feature = "platform")]
pub(crate) mod approval;
mod essentials;
#[cfg(feature = "platform")]
pub(crate) mod evidence;
#[cfg(feature = "platform")]
pub mod executor;
pub mod extension;
pub mod extension_gate;
mod validate;

pub use essentials::{builtin, defaults};
pub use validate::{Code, Environment, Invalid, Profile, Target, Validated, parse, validate};

/// Wire identifier of the only supported definition format.
pub const FORMAT: &str = "riauth.workflow/v1";
pub const MAX_DOCUMENT_BYTES: usize = 64 * 1024;
pub const MAX_STEPS: usize = 32;
pub const MAX_TERMINALS: usize = 8;
pub const MAX_TRANSITIONS: usize = 12;
pub const MAX_ATTEMPTS: u8 = 5;
pub const MAX_STEP_TIMEOUT_SECONDS: u32 = 3_600;
pub const MAX_RUN_SECONDS: u32 = 86_400;
pub const MAX_RUN_EXECUTIONS: u16 = 64;
pub const MAX_CONDITION_NODES: usize = 16;
pub const MAX_CONDITION_DEPTH: usize = 4;
pub const MAX_ALTERNATIVES: usize = 8;
pub const MAX_CUSTOM_OUTPUTS: usize = 8;
pub const MAX_CUSTOM_TIMEOUT_SECONDS: u32 = 30;
pub const MAX_CUSTOM_OUTPUT_BYTES: u32 = 4_096;
/// Upper bound for proof age at enrollment, recovery and sensitive-action success.
pub const MAX_SENSITIVE_PROOF_AGE_SECONDS: u32 = 300;
/// Identifier prefix reserved for shipped Essentials definitions.
pub const BUILTIN_PREFIX: &str = "essentials-";

/// A complete workflow definition. Every string is a bounded identifier; there is
/// no free-form value, display text, script, URL or secret field.
#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub format: Format,
    pub id: Id,
    /// Definition revision, starting at 1. A run binds `id`, `revision` and the
    /// canonical fingerprint.
    pub revision: u32,
    pub category: Category,
    pub origin: Origin,
    /// First step. It must name a step, not a terminal.
    pub entry: Id,
    pub limits: Limits,
    pub steps: Vec<Step>,
    pub terminals: Vec<Terminal>,
}

/// An operator-supplied Platform definition. Only active entries can start new
/// runs; each run stores its validated canonical definition and fingerprint.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredWorkflow {
    #[serde(default)]
    pub active: bool,
    pub definition: Definition,
}

/// The local password shapes the configured executor can safely bind to its
/// existing verifier adapters. Recovery is only a fallback from a reserved
/// TOTP step, so it never bypasses the password proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConfiguredPasswordPath {
    PasswordOnly,
    Totp,
    TotpOrRecovery,
}

impl ConfiguredPasswordPath {
    #[cfg(feature = "platform")]
    pub(crate) fn requires_mfa(self) -> bool {
        self != Self::PasswordOnly
    }
}

pub(crate) fn configured_password_path(definition: &Definition) -> Option<ConfiguredPasswordPath> {
    if definition.origin != Origin::Configured
        || definition.category != Category::Authentication
        || definition.terminals.len() != 2
        || definition
            .steps
            .first()
            .is_none_or(|step| definition.entry != step.id)
    {
        return None;
    }
    let success = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Authenticated);
    let denied = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied);
    let (Some(success), Some(denied)) = (success, denied) else {
        return None;
    };
    let routes = |step: &Step, verified: &Id, failed: &Id| {
        step.transitions.len() == 2
            && step
                .transitions
                .iter()
                .all(|transition| transition.when.is_none())
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed("verified") && &transition.to == verified
            })
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed("failed") && &transition.to == failed
            })
    };
    match definition.steps.as_slice() {
        [password]
            if matches!(password.action, Action::VerifyPassword {})
                && routes(password, &success.id, &denied.id) =>
        {
            Some(ConfiguredPasswordPath::PasswordOnly)
        }
        [password, totp]
            if matches!(password.action, Action::VerifyPassword {})
                && matches!(totp.action, Action::VerifyTotp {})
                && totp.id.as_str() == "totp"
                && routes(password, &totp.id, &denied.id)
                && routes(totp, &success.id, &denied.id)
                && success.requires.len() == 1
                && success.requires[0].len() == 2
                && success.requires[0].contains(&Proof::Password)
                && success.requires[0].contains(&Proof::Totp) =>
        {
            Some(ConfiguredPasswordPath::Totp)
        }
        [password, totp, recovery]
            if matches!(password.action, Action::VerifyPassword {})
                && matches!(totp.action, Action::VerifyTotp {})
                && matches!(recovery.action, Action::VerifyRecoveryCode {})
                && totp.id.as_str() == "totp"
                && recovery.id.as_str() == "recovery-code"
                && routes(password, &totp.id, &denied.id)
                && routes(totp, &success.id, &recovery.id)
                && routes(recovery, &success.id, &denied.id)
                && success.requires.len() == 2
                && [Proof::Totp, Proof::RecoveryCode]
                    .into_iter()
                    .all(|factor| {
                        success.requires.iter().any(|alternative| {
                            alternative.len() == 2
                                && alternative.contains(&Proof::Password)
                                && alternative.contains(&factor)
                        })
                    }) =>
        {
            Some(ConfiguredPasswordPath::TotpOrRecovery)
        }
        _ => None,
    }
}

/// One custom stage, then the local password verifier. The guest label only
/// chooses `password` or `denied`. Success still requires a password proof.
pub(crate) fn supported_configured_extension_password(definition: &Definition) -> bool {
    if definition.origin != Origin::Configured
        || definition.category != Category::Authentication
        || definition.steps.len() != 2
        || definition.terminals.len() != 2
        || definition.entry.as_str() != "extension"
        || definition
            .steps
            .first()
            .is_none_or(|step| step.id != definition.entry)
    {
        return false;
    }
    let Some(success) = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Authenticated)
    else {
        return false;
    };
    let Some(denied) = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied)
    else {
        return false;
    };
    if success.id.as_str() != "success"
        || denied.id.as_str() != "denied"
        || success.max_proof_age_seconds.is_some()
        || denied.max_proof_age_seconds.is_some()
        || !denied.requires.is_empty()
        || success.requires != vec![vec![Proof::Password]]
    {
        return false;
    }
    let [extension, password] = definition.steps.as_slice() else {
        return false;
    };
    let Action::Custom {
        outputs,
        permissions,
        ..
    } = &extension.action
    else {
        return false;
    };
    let mut labels: Vec<_> = outputs.iter().map(Label::as_str).collect();
    labels.sort_unstable();
    let extension_routes = |on: &str, to: &Id| {
        extension.transitions.iter().any(|transition| {
            transition.when.is_none() && transition.on.as_str() == on && &transition.to == to
        })
    };
    let password_routes = |on: &str, to: &Id| {
        password.transitions.iter().any(|transition| {
            transition.when.is_none() && transition.on.as_str() == on && &transition.to == to
        })
    };
    let worst = u16::from(extension.max_attempts) + u16::from(password.max_attempts);
    extension.id.as_str() == "extension"
        && extension.max_attempts == 1
        && (1..=MAX_CUSTOM_TIMEOUT_SECONDS).contains(&extension.timeout_seconds)
        && !extension.cancellable
        && extension.transitions.len() == 3
        && extension
            .transitions
            .iter()
            .all(|transition| transition.when.is_none())
        && labels == ["allow", "block"]
        && !permissions.contains(&StagePermission::Network)
        && extension_routes("allow", &password.id)
        && extension_routes("block", &denied.id)
        && extension_routes("failed", &denied.id)
        && password.id.as_str() == "password"
        && matches!(password.action, Action::VerifyPassword {})
        && (1..=3).contains(&password.max_attempts)
        && (1..=300).contains(&password.timeout_seconds)
        && password.cancellable
        && password.transitions.len() == 2
        && password
            .transitions
            .iter()
            .all(|transition| transition.when.is_none())
        && password_routes("verified", &success.id)
        && password_routes("failed", &denied.id)
        && (1..=600).contains(&definition.limits.max_duration_seconds)
        && (1..=8).contains(&definition.limits.max_executions)
        && definition.limits.max_executions >= worst
}

/// A configured passkey run uses the existing user-verified WebAuthn adapter.
/// Only its built-in verifier can emit the proof required by this terminal.
pub(crate) fn supported_configured_passkey(definition: &Definition) -> bool {
    if definition.origin != Origin::Configured
        || definition.category != Category::Authentication
        || definition.steps.len() != 1
        || definition.terminals.len() != 2
        || definition.entry != definition.steps[0].id
    {
        return false;
    }
    let step = &definition.steps[0];
    let success = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Authenticated);
    let denied = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied);
    let (Some(success), Some(denied)) = (success, denied) else {
        return false;
    };
    matches!(step.action, Action::VerifyPasskey {})
        && success.requires.len() == 1
        && success.requires[0].as_slice() == [Proof::Passkey]
        && step.transitions.len() == 2
        && step
            .transitions
            .iter()
            .all(|transition| transition.when.is_none())
        && step.transitions.iter().any(|transition| {
            transition.on == Label::fixed("verified") && transition.to == success.id
        })
        && step
            .transitions
            .iter()
            .any(|transition| transition.on == Label::fixed("failed") && transition.to == denied.id)
}

/// Add a passkey after existing UV proof. No transition can skip either the
/// live session or fresh factor, or complete without registration capability.
pub(crate) fn supported_configured_passkey_enrollment(definition: &Definition) -> bool {
    supported_configured_passkey_change(definition, Proof::Passkey)
}

/// A local password-only account may add its first passkey only through a
/// live session, fresh password receipt and workflow-owned WebAuthn ceremony.
pub(crate) fn supported_configured_password_passkey_enrollment(definition: &Definition) -> bool {
    supported_configured_passkey_change(definition, Proof::Password)
}

/// A local account with TOTP and no passkey may add its first passkey after
/// consuming a fresh code from its currently enrolled TOTP in this run.
pub(crate) fn supported_configured_totp_first_passkey_enrollment(definition: &Definition) -> bool {
    supported_configured_passkey_change(definition, Proof::Totp)
}

/// This exact shape can name one source. The runtime still pins and checks the
/// enabled source and its existing account link before accepting its proof.
pub(crate) fn configured_source_first_passkey_enrollment(definition: &Definition) -> Option<Id> {
    let Action::VerifySource { source } = &definition.steps.get(1)?.action else {
        return None;
    };
    supported_configured_passkey_change(definition, Proof::Source).then(|| source.clone())
}

/// Static validation may recognize the source named by this one canonical
/// shape. Runtime admission must resolve the source from the live registry.
pub(crate) fn configured_environment(definition: &Definition) -> Environment {
    let mut environment = Environment::platform();
    if let Some(source) = configured_source_first_passkey_enrollment(definition) {
        environment.sources.insert(source);
    }
    environment
}

fn supported_configured_passkey_change(definition: &Definition, factor_proof: Proof) -> bool {
    let existing_passkey = factor_proof == Proof::Passkey;
    let max_duration = if existing_passkey { 1_200 } else { 600 };
    let max_executions = if existing_passkey { 16 } else { 8 };
    if definition.origin != Origin::Configured
        || definition.category != Category::Enrollment
        || definition.steps.len() != 3
        || definition.terminals.len() != 2
        || definition.entry != definition.steps[0].id
        || definition.limits.max_duration_seconds > max_duration
        || definition.limits.max_executions > max_executions
    {
        return false;
    }
    let [session, factor, enroll] = definition.steps.as_slice() else {
        return false;
    };
    let success = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Enrolled);
    let denied = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied);
    let (Some(success), Some(denied)) = (success, denied) else {
        return false;
    };
    let routes = |step: &Step, verified: &'static str, target: &Id, failed: &Id| {
        step.transitions.len() == 2
            && step
                .transitions
                .iter()
                .all(|transition| transition.when.is_none())
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed(verified) && &transition.to == target
            })
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed("failed") && &transition.to == failed
            })
    };
    let (factor_name, factor_action, factor_timeout) = match factor_proof {
        Proof::Passkey => (
            "passkey",
            matches!(factor.action, Action::VerifyPasskey {}),
            300,
        ),
        Proof::Password => (
            "password",
            matches!(factor.action, Action::VerifyPassword {}),
            300,
        ),
        Proof::Totp => ("totp", matches!(factor.action, Action::VerifyTotp {}), 120),
        Proof::Source => (
            "source",
            matches!(factor.action, Action::VerifySource { .. }),
            300,
        ),
        _ => return false,
    };
    session.id.as_str() == "session"
        && factor.id.as_str() == factor_name
        && enroll.id.as_str() == "enroll"
        && success.id.as_str() == "success"
        && denied.id.as_str() == "denied"
        && matches!(session.action, Action::ResumeSession {})
        && factor_action
        && matches!(
            enroll.action,
            Action::EnrollCredential {
                credential: Credential::Passkey
            }
        )
        && session.max_attempts == 1
        && factor.max_attempts <= 3
        && (factor_proof != Proof::Source || factor.max_attempts == 1)
        && enroll.max_attempts == 1
        && session.timeout_seconds <= 60
        && factor.timeout_seconds <= factor_timeout
        && enroll.timeout_seconds <= 300
        && session.cancellable
        && factor.cancellable
        && enroll.cancellable
        && success.max_proof_age_seconds.is_some_and(|age| age <= 120)
        && success.requires.len() == 1
        && success.requires[0].len() == 3
        && [Proof::Session, factor_proof, Proof::Enrolled]
            .into_iter()
            .all(|proof| success.requires[0].contains(&proof))
        && routes(session, "verified", &factor.id, &denied.id)
        && routes(factor, "verified", &enroll.id, &denied.id)
        && routes(enroll, "completed", &success.id, &denied.id)
}

/// Enroll a new local TOTP secret only after a live session and an existing
/// user-verified passkey. The secret is generated by the bound run, not the
/// definition or an ordinary session-scoped enrollment.
pub(crate) fn supported_configured_totp_enrollment(definition: &Definition) -> bool {
    supported_configured_totp_change(definition, false, false)
}

/// A password-only local account may enroll its first TOTP factor after a
/// fresh, request-bound password receipt in a live session.
pub(crate) fn supported_configured_password_totp_enrollment(definition: &Definition) -> bool {
    supported_configured_totp_change(definition, false, true)
}

/// Replace an existing TOTP factor only through the exact session, UV passkey,
/// run-owned secret confirmation chain. The replacement action is distinct
/// from enrollment so definitions cannot select the mutation by account state.
pub(crate) fn supported_configured_totp_replacement(definition: &Definition) -> bool {
    supported_configured_totp_change(definition, true, false)
}

/// A local account with TOTP and no passkey may replace it only after both a
/// fresh password and its current TOTP are verified in the same bound run.
pub(crate) fn supported_configured_password_totp_replacement(definition: &Definition) -> bool {
    if definition.origin != Origin::Configured
        || definition.category != Category::Enrollment
        || definition.steps.len() != 4
        || definition.terminals.len() != 2
        || definition.entry != definition.steps[0].id
        || definition.limits.max_duration_seconds > 600
        || definition.limits.max_executions > 8
    {
        return false;
    }
    let [session, password, totp, replace] = definition.steps.as_slice() else {
        return false;
    };
    let (Some(success), Some(denied)) = (
        definition
            .terminals
            .iter()
            .find(|terminal| terminal.outcome == Outcome::Enrolled),
        definition
            .terminals
            .iter()
            .find(|terminal| terminal.outcome == Outcome::Denied),
    ) else {
        return false;
    };
    let routes = |step: &Step, signal: &'static str, target: &Id| {
        step.transitions.len() == 2
            && step
                .transitions
                .iter()
                .all(|transition| transition.when.is_none())
            && step
                .transitions
                .iter()
                .any(|transition| transition.on == Label::fixed(signal) && &transition.to == target)
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed("failed") && transition.to == denied.id
            })
    };
    session.id.as_str() == "session"
        && password.id.as_str() == "password"
        && totp.id.as_str() == "totp"
        && replace.id.as_str() == "enroll"
        && success.id.as_str() == "success"
        && denied.id.as_str() == "denied"
        && matches!(session.action, Action::ResumeSession {})
        && matches!(password.action, Action::VerifyPassword {})
        && matches!(totp.action, Action::VerifyTotp {})
        && matches!(replace.action, Action::ReplaceTotp {})
        && session.max_attempts == 1
        && password.max_attempts <= 3
        && totp.max_attempts <= 3
        && replace.max_attempts == 1
        && session.timeout_seconds <= 60
        && password.timeout_seconds <= 300
        && totp.timeout_seconds <= 120
        && replace.timeout_seconds <= 120
        && [session, password, totp, replace]
            .into_iter()
            .all(|step| step.cancellable)
        && success.max_proof_age_seconds.is_some_and(|age| age <= 120)
        && success.requires.len() == 1
        && success.requires[0].len() == 4
        && [
            Proof::Session,
            Proof::Password,
            Proof::Totp,
            Proof::Enrolled,
        ]
        .into_iter()
        .all(|proof| success.requires[0].contains(&proof))
        && routes(session, "verified", &password.id)
        && routes(password, "verified", &totp.id)
        && routes(totp, "verified", &replace.id)
        && routes(replace, "completed", &success.id)
}

fn supported_configured_totp_change(
    definition: &Definition,
    replace: bool,
    password: bool,
) -> bool {
    if definition.origin != Origin::Configured
        || definition.category != Category::Enrollment
        || definition.steps.len() != 3
        || definition.terminals.len() != 2
        || definition.entry != definition.steps[0].id
        || definition.limits.max_duration_seconds > 600
        || definition.limits.max_executions > 8
    {
        return false;
    }
    let [session, factor, enroll] = definition.steps.as_slice() else {
        return false;
    };
    let success = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Enrolled);
    let denied = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied);
    let (Some(success), Some(denied)) = (success, denied) else {
        return false;
    };
    let routes = |step: &Step, verified: &'static str, target: &Id| {
        step.transitions.len() == 2
            && step
                .transitions
                .iter()
                .all(|transition| transition.when.is_none())
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed(verified) && &transition.to == target
            })
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed("failed") && transition.to == denied.id
            })
    };
    let action_matches = if replace {
        matches!(enroll.action, Action::ReplaceTotp {})
    } else {
        matches!(
            enroll.action,
            Action::EnrollCredential {
                credential: Credential::Totp
            }
        )
    };
    let factor_proof = if password {
        Proof::Password
    } else {
        Proof::Passkey
    };
    let factor_name = if password { "password" } else { "passkey" };
    let factor_matches = if password {
        matches!(factor.action, Action::VerifyPassword {})
    } else {
        matches!(factor.action, Action::VerifyPasskey {})
    };
    session.id.as_str() == "session"
        && factor.id.as_str() == factor_name
        && enroll.id.as_str() == "enroll"
        && success.id.as_str() == "success"
        && denied.id.as_str() == "denied"
        && matches!(session.action, Action::ResumeSession {})
        && factor_matches
        && action_matches
        && session.max_attempts == 1
        && factor.max_attempts <= 3
        && enroll.max_attempts == 1
        && session.timeout_seconds <= 60
        && factor.timeout_seconds <= 300
        && enroll.timeout_seconds <= 120
        && session.cancellable
        && factor.cancellable
        && enroll.cancellable
        && success.max_proof_age_seconds.is_some_and(|age| age <= 120)
        && success.requires.len() == 1
        && success.requires[0].len() == 3
        && [Proof::Session, factor_proof, Proof::Enrolled]
            .into_iter()
            .all(|proof| success.requires[0].contains(&proof))
        && routes(session, "verified", &factor.id)
        && routes(factor, "verified", &enroll.id)
        && routes(enroll, "completed", &success.id)
}

/// Remove a request-pinned passkey after either exact, fresh factor path.
/// The target is never a definition field.
pub(crate) fn supported_configured_passkey_removal(definition: &Definition) -> bool {
    supported_configured_passkey_removal_uv(definition)
        || supported_configured_password_totp_passkey_removal(definition)
}

fn supported_configured_passkey_removal_uv(definition: &Definition) -> bool {
    if definition.origin != Origin::Configured
        || definition.category != Category::SensitiveAction
        || definition.steps.len() != 3
        || definition.terminals.len() != 2
        || definition.entry != definition.steps[0].id
        || definition.limits.max_duration_seconds > 600
        || definition.limits.max_executions > 8
    {
        return false;
    }
    let [session, passkey, remove] = definition.steps.as_slice() else {
        return false;
    };
    let success = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::ActionAuthorized);
    let denied = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied);
    let (Some(success), Some(denied)) = (success, denied) else {
        return false;
    };
    let routes = |step: &Step, verified: &'static str, target: &Id| {
        step.transitions.len() == 2
            && step
                .transitions
                .iter()
                .all(|transition| transition.when.is_none())
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed(verified) && &transition.to == target
            })
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed("failed") && transition.to == denied.id
            })
    };
    session.id.as_str() == "session"
        && passkey.id.as_str() == "passkey"
        && remove.id.as_str() == "remove"
        && success.id.as_str() == "success"
        && denied.id.as_str() == "denied"
        && matches!(session.action, Action::ResumeSession {})
        && matches!(passkey.action, Action::VerifyPasskey {})
        && matches!(remove.action, Action::RemovePasskey {})
        && session.max_attempts == 1
        && passkey.max_attempts <= 3
        && remove.max_attempts == 1
        && session.timeout_seconds <= 60
        && passkey.timeout_seconds <= 300
        && remove.timeout_seconds <= 120
        && session.cancellable
        && passkey.cancellable
        && remove.cancellable
        && success.max_proof_age_seconds.is_some_and(|age| age <= 120)
        && success.requires.len() == 1
        && success.requires[0].len() == 3
        && [Proof::Session, Proof::Passkey, Proof::PasskeyRemoved]
            .into_iter()
            .all(|proof| success.requires[0].contains(&proof))
        && routes(session, "verified", &passkey.id)
        && routes(passkey, "verified", &remove.id)
        && routes(remove, "completed", &success.id)
}

/// An MFA session may remove its exact pinned passkey after a fresh local
/// password and current TOTP code. Recovery codes are not a substitute.
pub(crate) fn supported_configured_password_totp_passkey_removal(definition: &Definition) -> bool {
    if definition.origin != Origin::Configured
        || definition.category != Category::SensitiveAction
        || definition.steps.len() != 4
        || definition.terminals.len() != 2
        || definition.entry != definition.steps[0].id
        || definition.limits.max_duration_seconds > 600
        || definition.limits.max_executions > 8
    {
        return false;
    }
    let [session, password, totp, remove] = definition.steps.as_slice() else {
        return false;
    };
    let success = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::ActionAuthorized);
    let denied = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied);
    let (Some(success), Some(denied)) = (success, denied) else {
        return false;
    };
    let routes = |step: &Step, verified: &'static str, target: &Id| {
        step.transitions.len() == 2
            && step
                .transitions
                .iter()
                .all(|transition| transition.when.is_none())
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed(verified) && &transition.to == target
            })
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed("failed") && transition.to == denied.id
            })
    };
    session.id.as_str() == "session"
        && password.id.as_str() == "password"
        && totp.id.as_str() == "totp"
        && remove.id.as_str() == "remove"
        && success.id.as_str() == "success"
        && denied.id.as_str() == "denied"
        && matches!(session.action, Action::ResumeSession {})
        && matches!(password.action, Action::VerifyPassword {})
        && matches!(totp.action, Action::VerifyTotp {})
        && matches!(remove.action, Action::RemovePasskey {})
        && session.max_attempts == 1
        && password.max_attempts <= 3
        && totp.max_attempts <= 3
        && remove.max_attempts == 1
        && session.timeout_seconds <= 60
        && password.timeout_seconds <= 300
        && totp.timeout_seconds <= 120
        && remove.timeout_seconds <= 120
        && session.cancellable
        && password.cancellable
        && totp.cancellable
        && remove.cancellable
        && success.max_proof_age_seconds.is_some_and(|age| age <= 120)
        && success.requires.len() == 1
        && success.requires[0].len() == 4
        && [
            Proof::Session,
            Proof::Password,
            Proof::Totp,
            Proof::PasskeyRemoved,
        ]
        .into_iter()
        .all(|proof| success.requires[0].contains(&proof))
        && routes(session, "verified", &password.id)
        && routes(password, "verified", &totp.id)
        && routes(totp, "verified", &remove.id)
        && routes(remove, "completed", &success.id)
}

/// A reset-mail submission can execute this exact recovery path in one writer.
/// The mail verifier alone supplies the reset proof and password mutation.
pub(crate) fn supported_configured_password_reset(definition: &Definition) -> bool {
    if definition.origin != Origin::Configured
        || definition.category != Category::Recovery
        || definition.steps.len() != 3
        || definition.terminals.len() != 2
        || definition.entry != definition.steps[0].id
        || definition.limits.max_duration_seconds > 2_400
        || definition.limits.max_executions > 4
    {
        return false;
    }
    let [identify, email, reset] = definition.steps.as_slice() else {
        return false;
    };
    let success = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Recovered);
    let denied = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied);
    let (Some(success), Some(denied)) = (success, denied) else {
        return false;
    };
    let routes = |step: &Step, verified: &'static str, target: &Id| {
        step.transitions.len() == 2
            && step
                .transitions
                .iter()
                .all(|transition| transition.when.is_none())
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed(verified) && &transition.to == target
            })
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed("failed") && transition.to == denied.id
            })
    };
    identify.id.as_str() == "identify"
        && email.id.as_str() == "email"
        && reset.id.as_str() == "reset"
        && success.id.as_str() == "success"
        && denied.id.as_str() == "denied"
        && matches!(identify.action, Action::Identify {})
        && matches!(
            email.action,
            Action::VerifyEmail {
                purpose: EmailPurpose::Reset
            }
        )
        && matches!(reset.action, Action::ResetPassword {})
        && identify.max_attempts == 1
        && email.max_attempts == 1
        && reset.max_attempts == 1
        && identify.timeout_seconds <= 300
        && email.timeout_seconds <= 1_800
        && reset.timeout_seconds <= 300
        && identify.cancellable
        && email.cancellable
        && !reset.cancellable
        && success.max_proof_age_seconds.is_some_and(|age| age <= 120)
        && success.requires.len() == 1
        && success.requires[0].len() == 2
        && success.requires[0].contains(&Proof::ResetEmail)
        && success.requires[0].contains(&Proof::PasswordReset)
        && identify.transitions.len() == 1
        && identify.transitions[0].when.is_none()
        && identify.transitions[0].on == Label::fixed("completed")
        && identify.transitions[0].to == email.id
        && routes(email, "verified", &reset.id)
        && routes(reset, "completed", &success.id)
}

/// A configured consent decision has one live-session proof and one explicit
/// user decision. Its success terminal cannot be reached by a static signal.
pub(crate) fn supported_configured_consent(definition: &Definition) -> bool {
    supported_configured_session_consent(definition)
        || supported_configured_passkey_consent(definition)
        || supported_configured_password_totp_consent(definition)
}

pub(crate) fn supported_configured_session_consent(definition: &Definition) -> bool {
    if definition.origin != Origin::Configured
        || definition.category != Category::Consent
        || definition.steps.len() != 2
        || definition.terminals.len() != 2
        || definition.entry != definition.steps[0].id
        || definition.limits.max_duration_seconds > 120
    {
        return false;
    }
    let session = &definition.steps[0];
    let consent = &definition.steps[1];
    let success = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::ConsentGranted);
    let denied = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied);
    let (Some(success), Some(denied)) = (success, denied) else {
        return false;
    };
    let routes =
        |step: &Step, granted: &'static str, target: &Id, refused: &'static str, rejected: &Id| {
            step.transitions.len() == 2
                && step
                    .transitions
                    .iter()
                    .all(|transition| transition.when.is_none())
                && step.transitions.iter().any(|transition| {
                    transition.on == Label::fixed(granted) && &transition.to == target
                })
                && step.transitions.iter().any(|transition| {
                    transition.on == Label::fixed(refused) && &transition.to == rejected
                })
        };
    session.id.as_str() == "session"
        && consent.id.as_str() == "consent"
        && matches!(session.action, Action::ResumeSession {})
        && matches!(consent.action, Action::RequestConsent {})
        && session.max_attempts == 1
        && consent.max_attempts == 1
        && consent.timeout_seconds <= 120
        && success.requires.len() == 1
        && success.requires[0].len() == 2
        && success.requires[0].contains(&Proof::Session)
        && success.requires[0].contains(&Proof::Consent)
        && routes(session, "verified", &consent.id, "failed", &denied.id)
        && routes(consent, "granted", &success.id, "denied", &denied.id)
}

/// This passkey consent graph proves the same session
/// with a fresh UV passkey before the explicit user decision.
pub(crate) fn supported_configured_passkey_consent(definition: &Definition) -> bool {
    if definition.origin != Origin::Configured
        || definition.category != Category::Consent
        || definition.steps.len() != 3
        || definition.terminals.len() != 2
        || definition.entry != definition.steps[0].id
        || definition.limits.max_duration_seconds > 120
        || definition.limits.max_executions > 5
    {
        return false;
    }
    let [session, passkey, consent] = definition.steps.as_slice() else {
        return false;
    };
    let success = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::ConsentGranted);
    let denied = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied);
    let (Some(success), Some(denied)) = (success, denied) else {
        return false;
    };
    let routes = |step: &Step, granted: &'static str, target: &Id, refused: &'static str| {
        step.transitions.len() == 2
            && step
                .transitions
                .iter()
                .all(|transition| transition.when.is_none())
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed(granted) && &transition.to == target
            })
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed(refused) && transition.to == denied.id
            })
    };
    session.id.as_str() == "session"
        && passkey.id.as_str() == "passkey"
        && consent.id.as_str() == "consent"
        && success.id.as_str() == "success"
        && denied.id.as_str() == "denied"
        && matches!(session.action, Action::ResumeSession {})
        && matches!(passkey.action, Action::VerifyPasskey {})
        && matches!(consent.action, Action::RequestConsent {})
        && session.max_attempts == 1
        && passkey.max_attempts <= 3
        && consent.max_attempts == 1
        && session.timeout_seconds <= 60
        && passkey.timeout_seconds <= 120
        && consent.timeout_seconds <= 120
        && session.cancellable
        && passkey.cancellable
        && consent.cancellable
        && success.max_proof_age_seconds.is_some_and(|age| age <= 120)
        && success.requires.len() == 1
        && success.requires[0].len() == 3
        && success.requires[0].contains(&Proof::Session)
        && success.requires[0].contains(&Proof::Passkey)
        && success.requires[0].contains(&Proof::Consent)
        && denied.requires.is_empty()
        && routes(session, "verified", &passkey.id, "failed")
        && routes(passkey, "verified", &consent.id, "failed")
        && routes(consent, "granted", &success.id, "denied")
}

/// Consent reauthentication with fresh local primary and current TOTP proofs.
/// A TOTP code alone cannot refresh the OIDC grant's authentication time.
pub(crate) fn supported_configured_password_totp_consent(definition: &Definition) -> bool {
    if definition.origin != Origin::Configured
        || definition.category != Category::Consent
        || definition.steps.len() != 4
        || definition.terminals.len() != 2
        || definition.entry != definition.steps[0].id
        || definition.limits.max_duration_seconds > 120
        || definition.limits.max_executions > 8
    {
        return false;
    }
    let [session, password, totp, consent] = definition.steps.as_slice() else {
        return false;
    };
    let success = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::ConsentGranted);
    let denied = definition
        .terminals
        .iter()
        .find(|terminal| terminal.outcome == Outcome::Denied);
    let (Some(success), Some(denied)) = (success, denied) else {
        return false;
    };
    let routes = |step: &Step, granted: &'static str, target: &Id, refused: &'static str| {
        step.transitions.len() == 2
            && step
                .transitions
                .iter()
                .all(|transition| transition.when.is_none())
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed(granted) && &transition.to == target
            })
            && step.transitions.iter().any(|transition| {
                transition.on == Label::fixed(refused) && transition.to == denied.id
            })
    };
    session.id.as_str() == "session"
        && password.id.as_str() == "password"
        && totp.id.as_str() == "totp"
        && consent.id.as_str() == "consent"
        && success.id.as_str() == "success"
        && denied.id.as_str() == "denied"
        && matches!(session.action, Action::ResumeSession {})
        && matches!(password.action, Action::VerifyPassword {})
        && matches!(totp.action, Action::VerifyTotp {})
        && matches!(consent.action, Action::RequestConsent {})
        && session.max_attempts == 1
        && password.max_attempts <= 3
        && totp.max_attempts <= 3
        && consent.max_attempts == 1
        && session.timeout_seconds <= 60
        && password.timeout_seconds <= 120
        && totp.timeout_seconds <= 120
        && consent.timeout_seconds <= 120
        && definition.steps.iter().all(|step| step.cancellable)
        && success.max_proof_age_seconds.is_some_and(|age| age <= 120)
        && success.requires.len() == 1
        && success.requires[0].len() == 4
        && [Proof::Session, Proof::Password, Proof::Totp, Proof::Consent]
            .into_iter()
            .all(|proof| success.requires[0].contains(&proof))
        && denied.requires.is_empty()
        && routes(session, "verified", &password.id, "failed")
        && routes(password, "verified", &totp.id, "failed")
        && routes(totp, "verified", &consent.id, "failed")
        && routes(consent, "granted", &success.id, "denied")
}

#[derive(JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Format {
    #[serde(rename = "riauth.workflow/v1")]
    V1,
}

#[derive(
    JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Authentication,
    Enrollment,
    Recovery,
    Consent,
    SensitiveAction,
}

/// `builtin` definitions are shipped Essentials defaults and must match one
/// exactly. `configured` definitions are the additive Platform boundary.
#[derive(JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Builtin,
    Configured,
}

#[derive(JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Wall-clock bound for the whole run; expiry ends it as `expired`.
    pub max_duration_seconds: u32,
    /// Bound on step executions including retries. Validation also checks the
    /// longest path, weighted by `max_attempts`, against it.
    pub max_executions: u16,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: Id,
    pub action: Action,
    /// Attempts before the `failed` signal is taken; 1 disables retry.
    pub max_attempts: u8,
    /// Per-attempt bound; exceeding it counts as a failed attempt.
    pub timeout_seconds: u32,
    /// Whether the user may cancel while this step is active. Cancellation and
    /// run expiry are not transitions: they always end as `cancelled`/`expired`.
    pub cancellable: bool,
    /// Ordered per signal; the first matching transition wins and the last one
    /// for each signal must be unconditional.
    pub transitions: Vec<Transition>,
}

/// Built-in step behavior. Only built-in verifiers produce proofs; the set is
/// closed and unknown `type` values are rejected when parsing.
#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Collect an account identifier. Always emits `completed`, proves nothing
    /// and does not reveal whether the account exists.
    Identify {},
    /// Require the live browser session.
    ResumeSession {},
    VerifyPassword {},
    VerifyPasskey {},
    VerifyTotp {},
    VerifyRecoveryCode {},
    /// Consume a purpose-bound single-use mail proof.
    VerifyEmail {
        purpose: EmailPurpose,
    },
    /// Complete a configured upstream source stage bound to this request.
    VerifySource {
        source: Id,
    },
    RequestConsent {},
    EnrollCredential {
        credential: Credential,
    },
    /// Replace an existing TOTP secret after a fresh, bound factor proof.
    ReplaceTotp {},
    /// Delete only the passkey pinned to this run's server-owned request.
    RemovePasskey {},
    /// Replace only the local password; other factors are preserved.
    ResetPassword {},
    /// Platform-only reference to a registered stage. It may route by its
    /// declared outputs but never produces a proof.
    Custom {
        stage: Id,
        outputs: Vec<Label>,
        permissions: Vec<StagePermission>,
        max_output_bytes: u32,
    },
}

#[derive(JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmailPurpose {
    Reset,
    Invitation,
}

#[derive(
    JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum Credential {
    Password,
    Passkey,
    Totp,
    RecoveryCodes,
}

/// Explicit permissions a custom stage requests; each must be granted by the
/// stage's registration in the [`Environment`].
#[derive(
    JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum StagePermission {
    ReadProfile,
    ReadGroups,
    ReadRequest,
    Network,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub on: Label,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<Condition>,
    pub to: Id,
}

/// Closed set of routing conditions. Conditions choose among validated paths;
/// validation treats every branch as possible, so they cannot weaken a proof
/// requirement.
#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    AccountHas { credential: Credential },
    HasProof { proof: Proof },
    RequestRequiresMfa {},
    All { of: Vec<Condition> },
    Any { of: Vec<Condition> },
    Not { condition: Box<Condition> },
}

/// Inputs an executor supplies to evaluate a [`Condition`].
pub trait Facts {
    fn account_has(&self, credential: Credential) -> bool;
    fn has_proof(&self, proof: Proof) -> bool;
    fn request_requires_mfa(&self) -> bool;
}

impl Condition {
    pub fn evaluate(&self, facts: &dyn Facts) -> bool {
        match self {
            Self::AccountHas { credential } => facts.account_has(*credential),
            Self::HasProof { proof } => facts.has_proof(*proof),
            Self::RequestRequiresMfa {} => facts.request_requires_mfa(),
            Self::All { of } => of.iter().all(|c| c.evaluate(facts)),
            Self::Any { of } => of.iter().any(|c| c.evaluate(facts)),
            Self::Not { condition } => !condition.evaluate(facts),
        }
    }
}

/// Kind of evidence a built-in action can produce on its success signal.
/// A kind alone is never sufficient to finish a run: completion loads a
/// store-backed record with provenance and exact run bindings.
#[derive(
    JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "snake_case")]
pub enum Proof {
    Session,
    Password,
    Passkey,
    Totp,
    RecoveryCode,
    ResetEmail,
    Invitation,
    Source,
    Consent,
    Enrolled,
    PasswordReset,
    PasskeyRemoved,
}

impl Proof {
    pub const ALL: [Proof; 12] = [
        Proof::Session,
        Proof::Password,
        Proof::Passkey,
        Proof::Totp,
        Proof::RecoveryCode,
        Proof::ResetEmail,
        Proof::Invitation,
        Proof::Source,
        Proof::Consent,
        Proof::Enrolled,
        Proof::PasswordReset,
        Proof::PasskeyRemoved,
    ];
    pub(crate) fn bit(self) -> u16 {
        1 << self as u16
    }
    /// Required binding for a stored evidence record. All records also bind to
    /// the run's optional session when one exists. Sign-in may have no session
    /// until the credential is attached; session and consent evidence require one.
    pub fn binding(self) -> Binding {
        Binding {
            account: true,
            request: true,
            run: true,
            session: matches!(
                self,
                Proof::Session | Proof::Consent | Proof::PasskeyRemoved
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub account: bool,
    pub request: bool,
    pub run: bool,
    pub session: bool,
}

#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Terminal {
    pub id: Id,
    pub outcome: Outcome,
    /// Additional proof alternatives: every path reaching this terminal must hold
    /// all proofs of at least one alternative. Empty means only the category
    /// floor applies. Denial terminals must leave it empty.
    pub requires: Vec<Vec<Proof>>,
    /// Maximum age of stored evidence at completion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_proof_age_seconds: Option<u32>,
}

/// Terminal results. Assurance values such as ACR/AMR are derived by the
/// executor from recorded proofs, never declared by a definition.
#[derive(JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Authenticated,
    Enrolled,
    Recovered,
    ConsentGranted,
    ActionAuthorized,
    Denied,
}

impl Outcome {
    pub fn is_success(self) -> bool {
        self != Outcome::Denied
    }
}

/// Storage-neutral lifecycle state of one run, for the executor to persist.
#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum RunState {
    Active { step: Id, attempt: u8 },
    Finished { terminal: Id, outcome: Outcome },
    Cancelled {},
    Expired {},
}

impl RunState {
    pub fn is_final(&self) -> bool {
        !matches!(self, RunState::Active { .. })
    }
}

/// Identity of the live source registration used by a source-verifier run.
#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceRegistrationBinding {
    pub source: Id,
    pub fingerprint: String,
}

/// Definition and registered dependency identity a run is bound to.
#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunBinding {
    pub workflow: Id,
    pub revision: u32,
    pub fingerprint: String,
    /// New source runs pin the live registration. Omitted by pre-upgrade runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_registration: Option<SourceRegistrationBinding>,
    /// SHA-256 of the module that started a controlled-extension run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_sha256: Option<String>,
}

macro_rules! name_type {
    ($name:ident, $max:expr, $pattern:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String")]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, String> {
                let value = value.into();
                let mut chars = value.chars();
                let valid = value.len() <= $max
                    && chars.next().is_some_and(|c| c.is_ascii_lowercase())
                    && chars.all(|c| {
                        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'
                    });
                if valid {
                    Ok(Self(value))
                } else {
                    Err(format!(
                        "{} must match {} and be at most {} bytes",
                        stringify!($name),
                        $pattern,
                        $max
                    ))
                }
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl TryFrom<String> for $name {
            type Error = String;
            fn try_from(value: String) -> Result<Self, String> {
                Self::new(value)
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
        impl JsonSchema for $name {
            fn schema_name() -> Cow<'static, str> {
                stringify!($name).into()
            }
            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({"type": "string", "pattern": $pattern, "maxLength": $max})
            }
        }
    };
}

name_type!(
    Id,
    64,
    "^[a-z][a-z0-9_-]*$",
    "Workflow, node, source or stage identifier."
);
name_type!(Label, 32, "^[a-z][a-z0-9_-]*$", "Signal emitted by a step.");

impl Label {
    pub(crate) fn fixed(value: &'static str) -> Self {
        Self(value.to_owned())
    }
}
