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
}
