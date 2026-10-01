//! Shared cloud-directory configuration shape.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

fn graph_scope() -> String {
    "https://graph.microsoft.com/.default".into()
}
fn empty_path(path: &Path) -> bool {
    path.as_os_str().is_empty()
}
fn workspace_attributes() -> Attributes {
    Attributes {
        email: "primaryEmail".into(),
        display_name: "name.fullName".into(),
        external_id: "id".into(),
    }
}
fn entra_attributes() -> Attributes {
    Attributes {
        email: "mail".into(),
        display_name: "displayName".into(),
        external_id: "id".into(),
    }
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Attributes {
    pub email: String,
    pub display_name: String,
    pub external_id: String,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceDirectory {
    pub customer_id: String,
    pub domain: String,
    /// Broker endpoint. Omit in direct service-account mode.
    #[serde(default)]
    pub token_url: String,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret_file: PathBuf,
    /// Google service-account JSON key and the Workspace user it impersonates.
    #[serde(default)]
    pub direct_auth: Option<WorkspaceDirectAuth>,
    /// Admin SDK origin. Production is `https://admin.googleapis.com`.
    pub directory_url: String,
    /// Local group name to upstream group id or email. Only these groups are reconciled.
    #[serde(default)]
    pub groups: BTreeMap<String, String>,
    #[serde(default = "workspace_attributes")]
    pub attributes: Attributes,
    #[serde(default)]
    pub username_prefix: String,
    /// Optional `scope` on the client-credentials token request. Empty omits it.
    #[serde(default)]
    pub scope: String,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceDirectAuth {
    pub key_file: PathBuf,
    pub delegated_subject: String,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntraDirectory {
    pub tenant_id: String,
    pub token_url: String,
    pub client_id: String,
    /// Shared-secret credential. Omit when using the certificate fields.
    #[serde(default, skip_serializing_if = "empty_path")]
    pub client_secret_file: PathBuf,
    /// PEM X.509 certificate registered on the Entra application.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_file: Option<PathBuf>,
    /// Matching RSA PEM private key, with owner-only file permissions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_key_file: Option<PathBuf>,
    /// Graph origin. Production is `https://graph.microsoft.com`.
    pub graph_url: String,
    #[serde(default = "graph_scope")]
    pub scope: String,
    /// Local group name to upstream group id or mail.
    #[serde(default)]
    pub groups: BTreeMap<String, String>,
    #[serde(default = "entra_attributes")]
    pub attributes: Attributes,
    #[serde(default)]
    pub username_prefix: String,
}
