//! Canonical typed, versioned workflow definitions.
//!
//! This module is the W01 model: a storage- and protocol-neutral description of
//! authentication, enrollment, recovery, consent and sensitive-action journeys.
//! It imports no Core, storage or protocol module and executes nothing. A later
//! executor consumes only [`Validated`] definitions; see `docs/workflows.md`.

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

mod essentials;
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

/// Evidence gained inside one run. Each proof is produced only by the listed
/// built-in action on its success signal and is bound to the run's account,
/// request and run instance (see [`Proof::binding`]).
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
}

impl Proof {
    pub const ALL: [Proof; 11] = [
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
    ];
    pub(crate) fn bit(self) -> u16 {
        1 << self as u16
    }
    /// Binding that an executor must record with the proof. It is intrinsic to
    /// the proof type, not a definition field, so a definition cannot declare or
    /// accept an unbound proof.
    pub fn binding(self) -> Binding {
        Binding {
            account: true,
            request: true,
            run: true,
            session: matches!(self, Proof::Session | Proof::Consent),
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
    /// Maximum age of proofs at completion, enforced by the executor.
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

/// Definition identity a run is bound to.
#[derive(JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunBinding {
    pub workflow: Id,
    pub revision: u32,
    pub fingerprint: String,
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
