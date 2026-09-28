//! Persisted OIDC claims-request data shared with the assurance adapter.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimsRequest {
    #[serde(default)]
    pub id_token: BTreeMap<String, Option<Requirement>>,
    #[serde(default)]
    pub userinfo: BTreeMap<String, Option<Requirement>>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    #[serde(default)]
    pub essential: bool,
    pub value: Option<Value>,
    pub values: Option<Vec<Value>>,
}
