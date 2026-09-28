//! Persisted claim mappings and authorization policy data.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClaimMapping {
    pub scope: String,
    pub claim: String,
    pub source: ClaimSource,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClaimSource {
    Username,
    DisplayName,
    Email,
    EmailVerified,
    Groups,
    Attribute { key: String },
    Literal { value: Value },
}

#[derive(schemars::JsonSchema, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Rule {
    pub all_groups: BTreeSet<String>,
    pub any_groups: BTreeSet<String>,
    pub denied_groups: BTreeSet<String>,
    pub users: BTreeSet<String>,
    pub denied_users: BTreeSet<String>,
    pub require_mfa: bool,
}

#[derive(schemars::JsonSchema, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Policy {
    pub access: Rule,
    pub scopes: BTreeMap<String, Rule>,
    /// Additive Platform checks and claim projections. Absence preserves the
    /// existing Essentials access and claim behavior.
    #[cfg(feature = "platform")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional: Option<ConditionalPolicy>,
}

impl Policy {
    pub fn conditional(&self) -> Option<&ConditionalPolicy> {
        #[cfg(feature = "platform")]
        {
            self.conditional.as_ref()
        }
        #[cfg(not(feature = "platform"))]
        {
            None
        }
    }
}

/// Every predicate in `access` (and in each requested scope) must hold. These
/// checks can only narrow the ordinary client and scope policy. Mappings can
/// only add claims after the ordinary authorization checks have succeeded.
#[derive(schemars::JsonSchema, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ConditionalPolicy {
    pub access: Vec<Predicate>,
    pub scopes: BTreeMap<String, Vec<Predicate>>,
    pub claim_mappings: Vec<ConditionalClaimMapping>,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConditionalClaimMapping {
    pub mapping: ClaimMapping,
    pub when: Predicate,
}

/// Closed, typed signals derived from server-held identity, membership and
/// device records. No predicate reads arbitrary stage output or client fields.
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Predicate {
    Application {
        id: String,
    },
    GroupMember {
        group: String,
    },
    VerifiedSource {
        source: String,
    },
    ProofFresh {
        proof: AuthenticationProof,
        max_age_seconds: u32,
    },
    Assurance {
        level: AssuranceLevel,
    },
    ApprovedDevice {
        max_age_seconds: u32,
    },
    All {
        of: Vec<Predicate>,
    },
    Any {
        of: Vec<Predicate>,
    },
    Not {
        condition: Box<Predicate>,
    },
}

#[derive(schemars::JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationProof {
    Password,
    Passkey,
    Source,
}

#[derive(schemars::JsonSchema, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssuranceLevel {
    Password,
    Mfa,
    Federated,
    Certificate,
}

pub trait PredicateFacts {
    fn application(&self, id: &str) -> bool;
    fn group_member(&self, group: &str) -> bool;
    fn verified_source(&self, source: &str) -> bool;
    fn proof_fresh(&self, proof: AuthenticationProof, max_age_seconds: u32) -> bool;
    fn assurance(&self, level: AssuranceLevel) -> bool;
    fn approved_device(&self, max_age_seconds: u32) -> bool;
}

impl Predicate {
    pub fn evaluate(&self, facts: &dyn PredicateFacts) -> bool {
        match self {
            Self::Application { id } => facts.application(id),
            Self::GroupMember { group } => facts.group_member(group),
            Self::VerifiedSource { source } => facts.verified_source(source),
            Self::ProofFresh {
                proof,
                max_age_seconds,
            } => facts.proof_fresh(*proof, *max_age_seconds),
            Self::Assurance { level } => facts.assurance(*level),
            Self::ApprovedDevice { max_age_seconds } => facts.approved_device(*max_age_seconds),
            Self::All { of } => of.iter().all(|predicate| predicate.evaluate(facts)),
            Self::Any { of } => of.iter().any(|predicate| predicate.evaluate(facts)),
            Self::Not { condition } => !condition.evaluate(facts),
        }
    }
}
