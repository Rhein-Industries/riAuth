//! RFC 7591 registration using bounded, revocable initial access tokens.
pub(crate) use crate::assembly::RegistrationAuthority;
use crate::{jose::PublicJwks, model::ProviderSettings};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrationTemplate {
    pub id: String,
    pub redirect_uris: Vec<String>,
    pub scopes: BTreeSet<String>,
    pub grant_types: BTreeSet<String>,
    pub auth_methods: BTreeSet<String>,
    pub settings: ProviderSettings,
    #[serde(default)]
    pub allowed_groups: BTreeSet<String>,
    #[serde(default)]
    pub require_mfa: bool,
    pub ttl: u64,
    pub max_uses: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct InitialAccess {
    pub(crate) template: RegistrationTemplate,
    pub(crate) token_hash: String,
    pub(crate) created_by: String,
    pub(crate) creator_agent: bool,
    pub(crate) expires_at: u64,
    pub(crate) used: u32,
    pub(crate) enabled: bool,
}
impl InitialAccess {
    pub(crate) fn view(&self) -> Value {
        json!({"template": self.template, "created_by": self.created_by, "expires_at": self.expires_at, "used": self.used, "enabled": self.enabled})
    }
}

#[derive(schemars::JsonSchema, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrationRequest {
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    pub scope: Option<String>,
    pub grant_types: Option<BTreeSet<String>>,
    pub response_types: Option<BTreeSet<String>>,
    pub token_endpoint_auth_method: Option<String>,
    pub application_type: Option<String>,
    pub jwks: Option<PublicJwks>,
    pub post_logout_redirect_uris: Option<Vec<String>>,
}
