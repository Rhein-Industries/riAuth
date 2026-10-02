//! Upstream federation with explicit account linking and terminal completion.
#[cfg(feature = "platform")]
pub mod saml;
#[cfg(not(feature = "platform"))]
#[path = "source/saml_essentials.rs"]
pub mod saml;
mod saml_types;
pub use crate::assembly::source_validate_identity as validate_identity;
#[cfg(feature = "platform")]
pub(crate) use crate::assembly::source_workflow_adapter as workflow;
use crate::jose::{ClientAuthMethod, PublicJwks};
pub use crate::model::federation::SourceIdentity;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Source {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saml: Option<saml::Settings>,
    /// Set only for OAuth-only providers with a pinned authenticated identity endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_profile: Option<OAuthProfile>,
    pub id: String,
    pub name: String,
    pub issuer: String,
    pub authorization_endpoint: String,
    #[serde(default)]
    pub token_endpoint: String,
    pub client_id: String,
    pub token_endpoint_auth_method: ClientAuthMethod,
    #[serde(default)]
    pub jwks: PublicJwks,
    #[serde(default = "default_scopes")]
    pub scopes: BTreeSet<String>,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub auto_provision: bool,
    #[serde(default)]
    pub groups: BTreeSet<String>,
    /// Only these explicitly trusted upstream ACRs satisfy local MFA policies.
    #[serde(default)]
    pub trusted_mfa_acr: BTreeSet<String>,
    #[serde(default)]
    pub allow_admin_login: bool,
}
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OAuthProfile {
    pub userinfo_endpoint: String,
    pub subject_pointer: String,
    pub name_pointer: Option<String>,
    pub email_pointer: Option<String>,
    pub email_verified_pointer: Option<String>,
}
fn yes() -> bool {
    true
}
fn default_scopes() -> BTreeSet<String> {
    ["openid", "profile", "email"].map(String::from).into()
}

#[derive(schemars::JsonSchema, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceInput {
    pub source: Source,
    pub client_secret: Option<String>,
}

#[derive(schemars::JsonSchema, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpec {
    pub source: Source,
    pub secret_ref: Option<String>,
    pub secret_version: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Link {
    pub(crate) source: String,
    pub(crate) issuer: String,
    pub(crate) subject: String,
    pub(crate) user_id: String,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkSpec {
    pub source: String,
    pub username: String,
    pub subject: String,
    /// Issuer stored on the link. Omitted on older manifests and on a fresh conversion;
    /// exports copy it. When set, it must equal the source's current issuer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}
/// Persisted reservation metadata is shared by both editions. Essentials must
/// recognize and reject a workflow-bound login rather than deserialize it as a
/// standalone login. Only the Platform adapter may create or consume a binding.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkflowBinding {
    pub run: String,
    pub account: String,
    pub account_epoch: u64,
    pub session: String,
    pub request: String,
    pub definition: crate::workflow::RunBinding,
    pub step: crate::workflow::Id,
    pub attempt: u8,
    pub reservation: String,
    pub started_at: u64,
}

#[expect(
    clippy::large_enum_variant,
    reason = "Short lived transaction state remains inline"
)]
pub(crate) enum CallbackClaim {
    Ready(Source, Login, Option<zeroize::Zeroizing<String>>),
    Mismatch,
    Retired,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Start {
    #[serde(default)]
    pub link: bool,
    pub authentication_transaction: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finish {
    pub credential: String,
    #[serde(default)]
    pub approve: bool,
    pub otp: Option<String>,
}

pub(crate) struct StageStart {
    pub authorization_id: String,
    pub request: crate::oidc::Authorization,
    pub user_id: Option<String>,
    pub browser_id: Option<String>,
}
pub(crate) struct StageStarted {
    pub transaction_id: String,
    pub authorization_id: String,
    pub stage_id: String,
    pub authorization_url: String,
    pub expires_at: u64,
    pub nonce: String,
}
impl StageStarted {
    pub(crate) fn public(&self) -> Value {
        json!({
            "stage_id": self.stage_id,
            "authorization_id": self.authorization_id,
            "authorization_url": self.authorization_url,
            "expires_at": self.expires_at,
            "nonce": self.nonce
        })
    }
}
// Compatibility paths for existing source, storage, management and workflow callers.
#[allow(unused_imports)]
pub(crate) use crate::assembly::SourceStartedLogin as StartedLogin;
pub use crate::assembly::source_cleanup as cleanup;
pub(crate) use crate::assembly::{
    SourceLogin as Login, SourceStage, SourceUpstreamIdentity as UpstreamIdentity,
    source_runtime_browser_binding_matches as browser_binding_matches,
    source_runtime_callback_body as callback_body, source_runtime_enabled as enabled,
    source_runtime_export_all_links as export_all_links,
    source_runtime_export_links as export_links, source_runtime_link_key as link_key,
    source_runtime_presented_source_retired as presented_source_retired,
    source_runtime_reconcile_link as reconcile_link,
    source_runtime_stage_authentication_required as stage_authentication_required,
    source_runtime_suspension_hash as suspension_hash,
};
