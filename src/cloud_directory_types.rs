//! Shared cloud-directory configuration shape.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

fn graph_scope() -> String {
    "https://graph.microsoft.com/.default".into()
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Attributes {
    pub email: String,
    pub display_name: String,
    pub external_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceDirectory {
    pub customer_id: String,
    pub domain: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret_file: PathBuf,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntraDirectory {
    pub tenant_id: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret_file: PathBuf,
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
