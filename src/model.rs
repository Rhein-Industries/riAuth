use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(default)]
    pub has_passkeys: bool,
    #[serde(default)]
    pub totp_settings: crate::authenticator::TotpSettings,
    #[serde(default)]
    pub pairwise_seed: String,
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub display_name: String,
    pub password_hash: String,
    pub enabled: bool,
    pub admin: bool,
    pub epoch: u64,
    pub totp_secret: Option<String>,
    pub totp_pending: Option<(String, u64)>,
    pub totp_last_step: Option<u64>,
    pub created_at: u64,
    #[serde(default)]
    pub attributes: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default)]
    pub subjects: BTreeMap<String, String>,
    #[serde(default)]
    pub recovery_codes: BTreeSet<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserView {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub display_name: String,
    pub enabled: bool,
    pub admin: bool,
    pub mfa_enabled: bool,
    pub created_at: u64,
    pub attributes: BTreeMap<String, serde_json::Value>,
    pub email_verified: bool,
    pub subjects: BTreeMap<String, String>,
}
impl From<&User> for UserView {
    fn from(u: &User) -> Self {
        Self {
            id: u.id.clone(),
            username: u.username.clone(),
            email: u.email.clone(),
            display_name: u.display_name.clone(),
            enabled: u.enabled,
            admin: u.admin,
            mfa_enabled: u.totp_secret.is_some() || u.has_passkeys,
            created_at: u.created_at,
            attributes: u.attributes.clone(),
            email_verified: u.email_verified,
            subjects: u.subjects.clone(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Group {
    pub name: String,
    pub members: BTreeSet<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub secret_hash: Option<String>,
    pub redirect_uris: Vec<String>,
    pub scopes: BTreeSet<String>,
    pub allowed_groups: BTreeSet<String>,
    pub require_mfa: bool,
    pub enabled: bool,
    pub service: bool,
    #[serde(default)]
    pub settings: ProviderSettings,
}

#[derive(schemars::JsonSchema, Clone, Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ProviderSettings {
    /// User-facing application metadata. Access always inherits the client policies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<crate::portal::Settings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saml: Option<crate::saml::Settings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<crate::radius::Settings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldap: Option<crate::ldap_server::Settings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<crate::outpost::Settings>,
    pub resources: BTreeMap<String, BTreeSet<String>>,
    pub issuer: Option<String>,
    pub token_endpoint_auth_method: Option<crate::jose::ClientAuthMethod>,
    pub jwks: Option<crate::jose::PublicJwks>,
    pub machine_trust: Vec<crate::jose::MachineTrust>,
    pub exchange: Option<crate::exchange::ExchangePolicy>,
    pub exchange_from: BTreeSet<String>,
    pub signing_key: Option<String>,
    pub id_token_encryption: Option<crate::encryption::EncryptionKey>,
    pub access_token_encryption: Option<crate::encryption::EncryptionKey>,
    pub userinfo_encryption: Option<crate::encryption::EncryptionKey>,
    pub userinfo_signed_response: bool,
    pub authorization_encryption: Option<crate::encryption::EncryptionKey>,
    pub default_acr_values: Vec<String>,
    pub require_pushed_authorization_requests: bool,
    pub require_signed_request: bool,
    pub pairwise_sector: Option<String>,
    pub token_managers: BTreeSet<String>,
    pub dpop_bound_access_tokens: bool,
    pub native: bool,
    pub allowed_grants: BTreeSet<String>,
    pub access_token_ttl: Option<u64>,
    pub refresh_token_ttl: Option<u64>,
    pub code_ttl: Option<u64>,
    pub device_ttl: Option<u64>,
    pub origins: BTreeSet<String>,
    pub post_logout_redirect_uris: Vec<String>,
    pub backchannel_logout_uri: Option<String>,
    pub frontchannel_logout_uri: Option<String>,
    pub claim_mappings: Vec<crate::claims::ClaimMapping>,
    pub groups_in_profile: bool,
    pub claims_in_access_token: bool,
    pub userinfo_only: bool,
    pub policy: crate::claims::Policy,
    /// Upstream source embedded in interactive authorization. One source id.
    /// Authentication is suspended until that source returns; it is not a new login stack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_stage: Option<String>,
    /// Fail closed on user token issuance unless this user has a fresh device-trust verification.
    #[serde(default)]
    pub require_device_trust: bool,
    /// First-party client: skip the browser consent screen. prompt=consent still asks.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub implicit_consent: bool,
}

impl ProviderSettings {
    /// Authentication keys, method, or workload trust — changes require `client.rotate`.
    pub(crate) fn authentication_credentials_differ(&self, other: &Self) -> bool {
        self.jwks != other.jwks
            || self.token_endpoint_auth_method != other.token_endpoint_auth_method
            || self.machine_trust != other.machine_trust
    }
}

impl Client {
    pub fn confidential(&self) -> bool {
        self.secret_hash.is_some()
            || self.settings.token_endpoint_auth_method
                == Some(crate::jose::ClientAuthMethod::PrivateKeyJwt)
    }
    pub fn view(&self) -> serde_json::Value {
        serde_json::json!({"client_id": self.id, "name": self.name, "confidential": self.confidential(), "redirect_uris": self.redirect_uris, "scopes": self.scopes, "allowed_groups": self.allowed_groups, "require_mfa": self.require_mfa, "enabled": self.enabled, "service": self.service, "settings": self.settings})
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Identity {
    #[serde(default)]
    pub amr: Vec<String>,
    #[serde(default)]
    pub source: Option<crate::source::SourceIdentity>,
    pub user_id: String,
    pub epoch: u64,
    pub mfa: bool,
    pub auth_time: u64,
    pub session_id: String,
}

impl Identity {
    pub fn authentication_methods(&self) -> Vec<String> {
        if !self.amr.is_empty() {
            self.amr.clone()
        } else if self.mfa {
            vec!["pwd".into(), "otp".into()]
        } else {
            vec!["pwd".into()]
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub token_hash: String,
    pub identity: Identity,
    pub expires_at: u64,
    pub revoked: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Code {
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub dpop_jkt: Option<String>,
    #[serde(default)]
    pub claims_request: crate::assurance::ClaimsRequest,
    #[serde(default)]
    pub acr_values: Option<String>,
    pub client_id: String,
    pub identity: Identity,
    pub redirect_uri: String,
    pub challenge: String,
    pub scopes: BTreeSet<String>,
    pub nonce: Option<String>,
    pub expires_at: u64,
    #[serde(default)]
    pub issued_family: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthenticationTransaction {
    pub request_hash: String,
    pub user_id: Option<String>,
    pub authenticated_session: Option<String>,
    pub expires_at: u64,
    /// Set when only the matching embedded source stage may complete this transaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_stage: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum DeviceStatus {
    Pending,
    Approved(Identity),
    Denied,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Device {
    #[serde(default)]
    pub resource: Option<String>,
    pub client_id: String,
    pub user_code_hash: String,
    pub scopes: BTreeSet<String>,
    pub expires_at: u64,
    pub last_poll_at: Option<u64>,
    pub interval: u64,
    pub status: DeviceStatus,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Grant {
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub confirmation_jkt: Option<String>,
    #[serde(default)]
    pub id_token_jkt: Option<String>,
    #[serde(default)]
    pub claims_request: crate::assurance::ClaimsRequest,
    #[serde(default)]
    pub acr_values: Option<String>,
    #[serde(default)]
    pub machine_trust_hash: Option<String>,
    #[serde(default)]
    pub exchange: Option<crate::exchange::ExchangeGrant>,
    pub client_id: String,
    pub identity: Option<Identity>,
    pub scopes: BTreeSet<String>,
    pub nonce: Option<String>,
    pub family_id: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub used: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Family {
    pub expires_at: u64,
    pub revoked: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Attempts {
    pub failures: u32,
    pub window_start: u64,
    pub locked_until: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Audit {
    pub id: String,
    pub at: u64,
    pub actor: String,
    pub action: String,
    pub target: String,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub details: serde_json::Value,
}

#[derive(schemars::JsonSchema, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub admin: bool,
}

#[derive(schemars::JsonSchema, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct UserPatch {
    pub enabled: Option<bool>,
    pub admin: Option<bool>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub display_name: Option<String>,
    #[serde(default)]
    pub reset_mfa: bool,
    #[serde(default)]
    pub revoke_sessions: bool,
    pub attributes: Option<BTreeMap<String, serde_json::Value>>,
    pub email_verified: Option<bool>,
    pub subjects: Option<BTreeMap<String, String>>,
}

#[derive(schemars::JsonSchema, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NewClient {
    pub client_id: String,
    pub name: String,
    #[serde(default)]
    pub confidential: bool,
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    pub scopes: BTreeSet<String>,
    #[serde(default)]
    pub allowed_groups: BTreeSet<String>,
    #[serde(default)]
    pub require_mfa: bool,
    #[serde(default)]
    pub service: bool,
    #[serde(default)]
    pub settings: ProviderSettings,
}

#[derive(schemars::JsonSchema, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ClientPatch {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub allowed_groups: Option<BTreeSet<String>>,
    pub require_mfa: Option<bool>,
    pub redirect_uris: Option<Vec<String>>,
    pub scopes: Option<BTreeSet<String>>,
    pub settings: Option<ProviderSettings>,
}
