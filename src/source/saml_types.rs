//! Persisted SAML source shapes shared by both editions for downgrade inspection.
use crate::model::client_settings::saml::NameIdFormat;
use serde::{Deserialize, Serialize};

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub signing_key: String,
    pub sp_certificate_pem: String,
    pub idp_certificates_pem: Vec<String>,
    #[serde(default)]
    pub name_id_format: NameIdFormat,
    pub name_attribute: Option<String>,
    pub email_attribute: Option<String>,
    pub email_verified_attribute: Option<String>,
    #[serde(default)]
    pub require_encrypted_assertions: bool,
    pub slo_redirect_url: Option<String>,
    pub slo_post_url: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct UpstreamSession {
    #[serde(default)]
    pub subject: Option<String>,
    pub index: String,
    pub expires_at: Option<u64>,
}
