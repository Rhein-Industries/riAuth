//! LDAP, Google Workspace, Microsoft Entra, and outbound SCIM connector
//! definitions in versioned desired state.
//!
//! A definition has exactly the shape of its `riauth.toml` table, with one
//! difference: every file field is a plain name relative to the operator's
//! `connector_secret_dir`. Planning, applying and exporting never read those
//! files. Only a full human administrator may write a definition, and a
//! definition id may not also exist in `riauth.toml`.
//!
//! This module is the one writer shared by desired-state reconcile and the
//! store. A stored row takes effect only at process start: [`merge`] copies the
//! rows into the configuration of that process, and nothing reloads it later.

use crate::cloud_directory::{EntraDirectory, WorkspaceDirectAuth, WorkspaceDirectory};
use crate::{
    agent::Principal,
    config::Config,
    core::audit_with_details,
    crypto::{digest, now},
    directory::Directory,
    error::{Error, Result},
    provisioning::{Oauth, Target},
    state::{Change, Manifest},
    store::{Store, Tx},
    validation::validate_name,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};
use url::Url;

const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

pub(crate) const BUCKET: &str = "connector_definitions";
pub(crate) const FORMAT: &str = "riauth.connector/v1";
/// Credential file name -> the destination and identity it was first bound to.
/// Plan and apply never remove a binding: it is the tombstone for a released name.
pub(crate) const BINDINGS: &str = "connector_credential_bindings";
const BINDING_FORMAT: &str = "riauth.connector-binding/v1";

/// The four existing connector tables. No other kind exists.
#[derive(
    schemars::JsonSchema, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Ldap,
    Workspace,
    Entra,
    Scim,
}

impl Kind {
    pub(crate) const ALL: [Kind; 4] = [Kind::Ldap, Kind::Workspace, Kind::Entra, Kind::Scim];

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Kind::Ldap => "ldap",
            Kind::Workspace => "workspace",
            Kind::Entra => "entra",
            Kind::Scim => "scim",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == value)
    }

    fn label(self) -> &'static str {
        match self {
            Kind::Ldap => "LDAP directory",
            Kind::Workspace => "Google Workspace directory",
            Kind::Entra => "Microsoft Entra directory",
            Kind::Scim => "SCIM target",
        }
    }

    /// The same limits `Config::validate` applies to the `riauth.toml` tables.
    fn cap(self) -> usize {
        match self {
            Kind::Scim => 32,
            _ => 16,
        }
    }
}

/// A stored definition to retire: its row is deleted, its credential bindings
/// stay, and a process drops it at its next start.
#[derive(schemars::JsonSchema, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetiredConnector {
    pub kind: Kind,
    pub id: String,
}

/// One typed definition. Paths are relative names in a stored or manifest
/// definition, and resolved paths in a [`Entry`].
#[derive(Clone)]
pub(crate) enum Definition {
    Ldap(Directory),
    Workspace(WorkspaceDirectory),
    Entra(EntraDirectory),
    Scim(Target),
}

type PathVisitor<'a> = &'a mut dyn FnMut(&'static str, bool, &mut PathBuf);

impl Definition {
    fn from_value(kind: Kind, value: Value) -> Result<Self> {
        let malformed = |_| Error::conflict("Stored connector definition is malformed");
        Ok(match kind {
            Kind::Ldap => Definition::Ldap(serde_json::from_value(value).map_err(malformed)?),
            Kind::Workspace => {
                Definition::Workspace(serde_json::from_value(value).map_err(malformed)?)
            }
            Kind::Entra => Definition::Entra(serde_json::from_value(value).map_err(malformed)?),
            Kind::Scim => Definition::Scim(serde_json::from_value(value).map_err(malformed)?),
        })
    }

    fn to_value(&self) -> Result<Value> {
        match self {
            Definition::Ldap(value) => serde_json::to_value(value),
            Definition::Workspace(value) => serde_json::to_value(value),
            Definition::Entra(value) => serde_json::to_value(value),
            Definition::Scim(value) => serde_json::to_value(value),
        }
        .map_err(Error::internal)
    }

    /// The same checks `riauth.toml` applies. They never read a file.
    fn validate(&self) -> Result<()> {
        match self {
            Definition::Ldap(value) => {
                value.validate()?;
                // The host the pin is compared with must be the one the client dials.
                let url = Url::parse(&value.url)
                    .map_err(|_| Error::bad("An LDAP URL cannot be parsed"))?;
                origin(&url).map(drop)
            }
            Definition::Workspace(value) => value.validate(),
            Definition::Entra(value) => value.validate(),
            Definition::Scim(value) => value.validate(),
        }
    }

    /// Every file field that is set. An unset optional field, and an unset
    /// path that the type defaults to empty, are not visited. A required path
    /// is always visited, so an empty one is refused. Each struct is
    /// destructured with every field named, so a new file field cannot be added
    /// to one of them without this function failing to compile until the field
    /// is classified.
    fn visit(&mut self, f: PathVisitor<'_>) {
        match self {
            Definition::Ldap(value) => {
                let Directory {
                    url: _,
                    transport: _,
                    bind_dn: _,
                    password_file,
                    ca_file,
                    user_base: _,
                    user_filter: _,
                    id_attribute: _,
                    username_attribute: _,
                    display_attribute: _,
                    email_attribute: _,
                    username_prefix: _,
                    group_user_filters: _,
                } = value;
                f("password_file", true, password_file);
                if let Some(path) = ca_file.as_mut() {
                    f("ca_file", false, path);
                }
            }
            Definition::Workspace(value) => {
                let WorkspaceDirectory {
                    customer_id: _,
                    domain: _,
                    token_url: _,
                    client_id: _,
                    client_secret_file,
                    direct_auth,
                    directory_url: _,
                    groups: _,
                    attributes: _,
                    username_prefix: _,
                    scope: _,
                } = value;
                if !client_secret_file.as_os_str().is_empty() {
                    f("client_secret_file", true, client_secret_file);
                }
                if let Some(direct) = direct_auth.as_mut() {
                    let WorkspaceDirectAuth {
                        key_file,
                        delegated_subject: _,
                    } = direct;
                    f("direct_auth.key_file", true, key_file);
                }
            }
            Definition::Entra(value) => {
                let EntraDirectory {
                    tenant_id: _,
                    token_url: _,
                    client_id: _,
                    client_secret_file,
                    certificate_file,
                    private_key_file,
                    graph_url: _,
                    scope: _,
                    groups: _,
                    attributes: _,
                    username_prefix: _,
                } = value;
                if !client_secret_file.as_os_str().is_empty() {
                    f("client_secret_file", true, client_secret_file);
                }
                if let Some(path) = certificate_file.as_mut() {
                    f("certificate_file", false, path);
                }
                if let Some(path) = private_key_file.as_mut() {
                    f("private_key_file", true, path);
                }
            }
            Definition::Scim(value) => {
                let Target {
                    url: _,
                    token_file,
                    oauth,
                    ca_file,
                    groups: _,
                    export_groups: _,
                } = value;
                if let Some(path) = token_file.as_mut() {
                    f("token_file", true, path);
                }
                if let Some(path) = ca_file.as_mut() {
                    f("ca_file", false, path);
                }
                if let Some(oauth) = oauth.as_mut() {
                    let Oauth {
                        token_url: _,
                        grant: _,
                        client_id: _,
                        client_secret_file,
                        refresh_token_file,
                        scope: _,
                        audience: _,
                        ca_file,
                    } = oauth;
                    if let Some(path) = client_secret_file.as_mut() {
                        f("oauth.client_secret_file", true, path);
                    }
                    if let Some(path) = refresh_token_file.as_mut() {
                        f("oauth.refresh_token_file", true, path);
                    }
                    if let Some(path) = ca_file.as_mut() {
                        f("oauth.ca_file", false, path);
                    }
                }
            }
        }
    }

    fn path_fields(&self) -> Vec<(&'static str, bool, PathBuf)> {
        let mut copy = self.clone();
        let mut fields = Vec::new();
        copy.visit(&mut |field, secret, path| fields.push((field, secret, path.clone())));
        fields
    }

    /// Credential-bearing files only. A shared CA or public certificate is not
    /// a credential and may be named by several connectors.
    fn secret_paths(&self) -> BTreeSet<PathBuf> {
        self.path_fields()
            .into_iter()
            .filter(|(_, secret, _)| *secret)
            .map(|(_, _, path)| path)
            .collect()
    }

    fn kind(&self) -> Kind {
        match self {
            Definition::Ldap(_) => Kind::Ldap,
            Definition::Workspace(_) => Kind::Workspace,
            Definition::Entra(_) => Kind::Entra,
            Definition::Scim(_) => Kind::Scim,
        }
    }

    /// Where each credential this definition names is sent, which trust anchor
    /// protects the exchange, and as whom it is presented. Changing any of it
    /// redirects or re-purposes a credential.
    fn binding_value(&self) -> Result<Value> {
        Ok(match self {
            Definition::Ldap(value) => json!({
                "url": value.url,
                "transport": serde_json::to_value(&value.transport).map_err(Error::internal)?,
                "ca_file": value.ca_file,
                "bind_dn": value.bind_dn,
            }),
            Definition::Workspace(value) => json!({
                "token_url": value.token_url,
                "directory_url": value.directory_url,
                "customer_id": value.customer_id,
                "domain": value.domain,
                "client_id": value.client_id,
                "delegated_subject": value.direct_auth.as_ref().map(|direct| &direct.delegated_subject),
            }),
            Definition::Entra(value) => json!({
                "token_url": value.token_url,
                "graph_url": value.graph_url,
                "tenant_id": value.tenant_id,
                "client_id": value.client_id,
            }),
            Definition::Scim(value) => json!({
                "url": value.url,
                "ca_file": value.ca_file,
                "oauth": value.oauth.as_ref().map(|oauth| json!({
                    "token_url": oauth.token_url,
                    "ca_file": oauth.ca_file,
                    "client_id": oauth.client_id,
                    "audience": oauth.audience,
                    "grant": oauth.grant,
                })),
            }),
        })
    }

    /// Digest of [`Self::binding_value`]. A credential file bound to one digest
    /// can never be named by a definition with another.
    fn binding_digest(&self) -> Result<String> {
        let mut canonical = self.binding_value()?;
        canonical.sort_all_objects();
        Ok(digest(&format!(
            "{BINDING_FORMAT}\n{}\n{canonical}",
            self.kind().as_str()
        )))
    }

    /// What a credential is allowed to do at that destination. A change keeps
    /// the binding but is still a credential change.
    fn scopes(&self) -> Value {
        match self {
            Definition::Ldap(_) => Value::Null,
            // A direct-mode token is minted with the account's own Directory
            // access, so the groups it may read are part of what it can do.
            Definition::Workspace(value) if value.direct_auth.is_some() => {
                json!({"scope": value.scope, "groups": value.groups})
            }
            Definition::Workspace(value) => json!(value.scope),
            Definition::Entra(value) => json!(value.scope),
            Definition::Scim(value) => json!(value.oauth.as_ref().map(|oauth| &oauth.scope)),
        }
    }

    /// Lowercased credential file names of an unresolved definition.
    fn credential_names(&self) -> BTreeSet<String> {
        self.secret_paths()
            .iter()
            .map(|path| path.to_string_lossy().to_ascii_lowercase())
            .collect()
    }

    /// For each credential file, every origin it or a token derived from it is
    /// sent to. A static token and an LDAP password go to the target. An OAuth
    /// secret or refresh token goes to the token endpoint, and the access token
    /// it yields goes to the target. A Workspace broker secret goes to the
    /// broker, and the token it yields to the Admin SDK origin; a direct key
    /// signs a request to Google's token endpoint unless a fake peer is set.
    /// An Entra credential goes to the token endpoint, and its token to Graph.
    fn credential_reach(&self) -> Result<Vec<(PathBuf, BTreeSet<String>)>> {
        let of = |text: &str| -> Result<String> {
            origin(&Url::parse(text).map_err(|_| Error::bad("A connector URL cannot be parsed"))?)
        };
        let mut reach = Vec::new();
        match self {
            Definition::Ldap(value) => {
                reach.push((
                    value.password_file.clone(),
                    BTreeSet::from([of(&value.url)?]),
                ));
            }
            Definition::Scim(value) => {
                let target = of(&value.url)?;
                if let Some(path) = &value.token_file {
                    reach.push((path.clone(), BTreeSet::from([target.clone()])));
                }
                if let Some(oauth) = &value.oauth {
                    let both = BTreeSet::from([of(&oauth.token_url)?, target]);
                    for path in [&oauth.client_secret_file, &oauth.refresh_token_file]
                        .into_iter()
                        .flatten()
                    {
                        reach.push((path.clone(), both.clone()));
                    }
                }
            }
            Definition::Workspace(value) => {
                let directory = of(&value.directory_url)?;
                if !value.client_secret_file.as_os_str().is_empty() {
                    reach.push((
                        value.client_secret_file.clone(),
                        BTreeSet::from([of(&value.token_url)?, directory.clone()]),
                    ));
                }
                if let Some(direct) = &value.direct_auth {
                    let token = if value.token_url.is_empty() {
                        GOOGLE_TOKEN_URL
                    } else {
                        value.token_url.as_str()
                    };
                    reach.push((
                        direct.key_file.clone(),
                        BTreeSet::from([of(token)?, directory]),
                    ));
                }
            }
            Definition::Entra(value) => {
                let both = BTreeSet::from([of(&value.token_url)?, of(&value.graph_url)?]);
                let secret =
                    Some(&value.client_secret_file).filter(|path| !path.as_os_str().is_empty());
                for path in [secret, value.private_key_file.as_ref()]
                    .into_iter()
                    .flatten()
                {
                    reach.push((path.clone(), both.clone()));
                }
            }
        }
        Ok(reach)
    }

    /// Join every relative name under the secret directory.
    fn resolve(&mut self, directory: &Path) {
        self.visit(&mut |_, _, path| *path = directory.join(&*path));
    }
}

/// A definition with resolved paths, from `riauth.toml` or a stored row.
pub(crate) struct Entry {
    pub(crate) kind: Kind,
    pub(crate) id: String,
    pub(crate) definition: Definition,
}

/// One stored definition. Never carries secret bytes: the file fields are
/// names under `connector_secret_dir`.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredDefinition {
    pub(crate) format: String,
    pub(crate) kind: Kind,
    pub(crate) id: String,
    pub(crate) revision: u64,
    pub(crate) digest: String,
    pub(crate) definition: Value,
    pub(crate) updated_at: u64,
    pub(crate) updated_by: String,
}

/// A credential file name bound to one destination and identity. Written when
/// a definition first names the file, and never removed or changed.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    format: String,
    name: String,
    binding: String,
    first_kind: Kind,
    first_id: String,
    bound_at: u64,
    bound_by: String,
}

/// The stored definitions merged into this process's configuration at open.
/// Plan and apply never change it: a stored row written later is reported as
/// `restart_required`.
#[derive(Debug, Default)]
pub(crate) struct Loaded {
    entries: BTreeMap<(Kind, String), (u64, String)>,
}

impl Loaded {
    fn contains(&self, kind: Kind, id: &str) -> bool {
        self.entries.contains_key(&(kind, id.to_owned()))
    }
}

fn key(kind: Kind, id: &str) -> String {
    format!("{}/{id}", kind.as_str())
}

fn definition_digest(kind: Kind, definition: &Value) -> String {
    let mut canonical = definition.clone();
    canonical.sort_all_objects();
    digest(&format!("{FORMAT}\n{}\n{canonical}", kind.as_str()))
}

fn view(row: &StoredDefinition) -> Value {
    json!({
        "kind": row.kind.as_str(),
        "id": row.id,
        "revision": row.revision,
        "digest": row.digest,
        "definition": row.definition,
    })
}

fn require_administrator(actor: &Principal) -> Result<()> {
    if actor.agent || actor.delegated {
        Err(Error::forbidden())
    } else {
        Ok(())
    }
}

/// A file field is a canonical relative name: components of ASCII letters,
/// digits, `.`, `_` and `-`, joined by one `/`. A component is not all dots and
/// does not end in a dot, so no spelling aliases another on a case-insensitive
/// or Windows file system. The message does not echo the value.
fn require_relative_name(field: &str, path: &Path) -> Result<()> {
    let refused = || {
        Error::bad(format!(
            "Connector {field} must be a relative name under connector_secret_dir made of ASCII letters, digits, '.', '_' and '-'"
        ))
    };
    let text = path.to_str().ok_or_else(refused)?;
    if text.is_empty() || text.len() > 256 {
        return Err(refused());
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let part = part.to_str().ok_or_else(refused)?;
                if part.len() > 128
                    || part.ends_with('.')
                    || !part
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
                {
                    return Err(refused());
                }
                parts.push(part);
            }
            _ => return Err(refused()),
        }
    }
    if parts.join("/") != text {
        return Err(refused());
    }
    Ok(())
}

fn validate_entry(kind: Kind, id: &str, definition: &Definition) -> Result<()> {
    validate_name(id)?;
    let mut copy = definition.clone();
    let mut refusal = None;
    copy.visit(&mut |field, _, path| {
        if refusal.is_none() {
            refusal = require_relative_name(field, path).err();
        }
    });
    if let Some(error) = refusal {
        return Err(error);
    }
    definition
        .validate()
        .map_err(|error| Error::bad(format!("{} {id}: {}", kind.label(), error.message)))
}

/// The four manifest tables as `(kind, id, definition)`, in a stable order.
fn manifest_definitions(manifest: &Manifest) -> Vec<(Kind, String, Definition)> {
    let mut out = Vec::new();
    for (id, value) in &manifest.directories {
        out.push((Kind::Ldap, id.clone(), Definition::Ldap(value.clone())));
    }
    for (id, value) in &manifest.workspace_directories {
        out.push((
            Kind::Workspace,
            id.clone(),
            Definition::Workspace(value.clone()),
        ));
    }
    for (id, value) in &manifest.entra_directories {
        out.push((Kind::Entra, id.clone(), Definition::Entra(value.clone())));
    }
    for (id, value) in &manifest.scim_targets {
        out.push((Kind::Scim, id.clone(), Definition::Scim(value.clone())));
    }
    out
}

/// Structural checks that need no configuration, so `riauth validate` can run
/// them offline. Authority, `connector_secret_dir`, conflicts with
/// `riauth.toml`, and secret-file ownership are checked when planning.
pub(crate) fn validate_manifest(manifest: &Manifest) -> Result<()> {
    let definitions = manifest_definitions(manifest);
    let mut retired = BTreeSet::new();
    for item in &manifest.retired_connectors {
        validate_name(&item.id)?;
        if !retired.insert((item.kind, item.id.as_str())) {
            return Err(Error::bad("Duplicate resource in manifest"));
        }
    }
    if definitions
        .iter()
        .any(|(kind, id, _)| retired.contains(&(*kind, id.as_str())))
    {
        return Err(Error::bad(
            "A connector id cannot be both defined and retired in one manifest",
        ));
    }
    if definitions.is_empty() {
        return Ok(());
    }
    if !cfg!(feature = "platform")
        && (!manifest.workspace_directories.is_empty() || !manifest.entra_directories.is_empty())
    {
        return Err(Error::bad(
            "Google Workspace and Microsoft Entra connector definitions require Platform",
        ));
    }
    for kind in Kind::ALL {
        let count = definitions.iter().filter(|(k, _, _)| *k == kind).count();
        if count > kind.cap() {
            return Err(Error::bad(format!(
                "Configure at most {} {} entries",
                kind.cap(),
                kind.label()
            )));
        }
    }
    for (kind, id, definition) in &definitions {
        validate_entry(*kind, id, definition)?;
    }
    Ok(())
}

fn retired_keys(manifest: &Manifest) -> BTreeSet<(Kind, String)> {
    manifest
        .retired_connectors
        .iter()
        .map(|item| (item.kind, item.id.clone()))
        .collect()
}

/// Stored rows this manifest would retire. A retirement of an id that is not
/// stored changes nothing and counts for nothing.
pub(crate) fn retiring(tx: &Tx<'_>, manifest: &Manifest) -> Result<usize> {
    let mut count = 0;
    for (kind, id) in retired_keys(manifest) {
        if tx.get::<Value>(BUCKET, &key(kind, &id))?.is_some() {
            count += 1;
        }
    }
    Ok(count)
}

/// Lexical normalization only. Nothing here touches the file system.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// The form two paths are compared in: absolute (so a relative `riauth.toml`
/// path and an absolute secret directory meet), normalized, and lowercased (so
/// a case-insensitive file system cannot alias a name). No file is read.
fn fold(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path)
        .map_err(|_| Error::bad("A configured file path cannot be made absolute"))?;
    Ok(PathBuf::from(
        normalize(&absolute).to_string_lossy().to_lowercase(),
    ))
}

/// Every credential or key file named elsewhere in the process configuration,
/// listener certificates included: a stored name must never reach one.
fn credential_paths(config: &Config) -> Vec<&Path> {
    let mut paths: Vec<&Path> = Vec::new();
    paths.extend(config.database_key_file.as_deref());
    paths.extend(config.tls_cert_file.as_deref());
    paths.extend(config.tls_key_file.as_deref());
    if let Some(mail) = &config.mail {
        paths.extend(mail.password_file.as_deref());
    }
    if let Some(postgres) = &config.postgres {
        paths.push(&postgres.connection_file);
        paths.extend(postgres.ca_file.as_deref());
    }
    for signer in config.signers.values() {
        paths.push(&signer.token_file);
        paths.extend(signer.ca_file.as_deref());
    }
    for controller in config.reconciliation_controllers.values() {
        paths.push(&controller.credential_file);
    }
    if let Some(webhook) = &config.alert_webhook {
        paths.extend(webhook.bearer_file.as_deref());
    }
    if let Some(trust) = &config.device_trust {
        paths.extend(trust.pem_file.as_deref());
        paths.extend(trust.jwks_file.as_deref());
        paths.extend(trust.service_account_file.as_deref());
    }
    if let Some(profile) = &config.client_certificates {
        paths.push(&profile.trust_anchors_file);
        paths.extend(profile.crl_file.as_deref());
    }
    for listener in config.ldap_listeners.values() {
        paths.extend(listener.tls_cert_file.as_deref());
        paths.extend(listener.tls_key_file.as_deref());
    }
    for listener in config.proxy_listeners.values() {
        paths.extend(listener.tls_cert_file.as_deref());
        paths.extend(listener.tls_key_file.as_deref());
        for route in listener.routes.values() {
            paths.extend(route.ca_file.as_deref());
        }
    }
    for listener in config.radius_listeners.values() {
        paths.extend(listener.tls_cert_file.as_deref());
        paths.extend(listener.tls_key_file.as_deref());
        paths.extend(listener.client_ca_file.as_deref());
        for nas in listener.nas.values() {
            paths.extend(nas.shared_secret_file.as_deref());
        }
        if let Some(eap) = &listener.eap_tls {
            paths.push(&eap.certificate_file);
            paths.push(&eap.key_file);
            paths.push(&eap.client_ca_file);
            paths.push(&eap.client_crl_file);
            paths.extend(eap.ocsp_response_file.as_deref());
        }
    }
    paths
}

/// The operator's opt-in. Unset, no definition is accepted. Set, the directory
/// must be a dedicated one: it may not be, contain, or lie inside the data
/// directory, and it may not contain another configured credential file.
pub(crate) fn secret_dir(config: &Config) -> Result<&Path> {
    let directory = config.connector_secret_dir.as_deref().ok_or_else(|| {
        Error::bad(
            "Connector definitions are disabled; the operator must set connector_secret_dir in riauth.toml",
        )
    })?;
    let root = fold(directory)?;
    let data = fold(&config.data_dir)?;
    let mut overlaps = root.starts_with(&data) || data.starts_with(&root);
    for path in credential_paths(config) {
        overlaps |= fold(path)?.starts_with(&root);
    }
    if overlaps {
        return Err(Error::bad(
            "connector_secret_dir must be a dedicated directory outside the data directory and every other configured credential file",
        ));
    }
    Ok(directory)
}

/// The origin of a connector URL: scheme, lowercased host, and an explicit
/// port, so an explicit default port and an omitted one compare equal.
fn origin(url: &Url) -> Result<String> {
    let scheme = url.scheme();
    let default = match scheme {
        "https" => 443,
        "http" => 80,
        "ldaps" => 636,
        "ldap" => 389,
        _ => return Err(Error::bad("A connector URL uses an unsupported scheme")),
    };
    let host = url
        .host_str()
        .filter(|host| !host.is_empty())
        .ok_or_else(|| Error::bad("A connector URL has no host"))?;
    let host = if matches!(scheme, "ldap" | "ldaps") {
        // The LDAP client dials, and names in the TLS handshake, the host
        // exactly as the URL crate hands it over: opaque, not percent-decoded,
        // not IDNA-mapped. So an origin is that string, lowercased. A host the
        // client would read differently from a pin (percent-encoding, non-ASCII)
        // is refused, and a trailing dot or a short IPv4 form stays significant:
        // a pin then does not match it, which fails closed.
        if host.contains('%') || !host.is_ascii() {
            return Err(Error::bad(
                "An LDAP URL host must be plain ASCII without percent-encoding, because the client uses it as written",
            ));
        }
        host.to_ascii_lowercase()
    } else {
        // HTTP hosts go through the one parser that lowercases, applies IDNA, and
        // canonicalizes IP literals. A trailing dot names the same DNS host.
        let parsed = Url::parse(&format!("https://{host}"))
            .ok()
            .and_then(|parsed| parsed.host_str().map(str::to_owned))
            .ok_or_else(|| Error::bad("A connector URL host cannot be normalized"))?;
        let trimmed = parsed.trim_end_matches('.');
        if trimmed.is_empty() {
            return Err(Error::bad("A connector URL has no host"));
        }
        trimmed.to_owned()
    };
    Ok(format!(
        "{scheme}://{host}:{}",
        url.port().unwrap_or(default)
    ))
}

/// One operator-written origin: a bare origin over HTTPS, LDAPS or LDAP, or
/// HTTP on a loopback host only, as `riauth.toml` allows elsewhere.
fn pinned_origin(text: &str) -> Result<String> {
    let invalid = || {
        Error::bad(
            "connector_credentials origins must be scheme://host[:port] over https, ldaps, or ldap (http only on loopback)",
        )
    };
    let url = Url::parse(text.trim()).map_err(|_| invalid())?;
    let loopback = matches!(
        url.host_str().map(str::to_ascii_lowercase).as_deref(),
        Some("localhost" | "127.0.0.1" | "[::1]")
    );
    let scheme_allowed =
        matches!(url.scheme(), "https" | "ldaps" | "ldap") || url.scheme() == "http" && loopback;
    if !scheme_allowed
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.path(), "" | "/")
    {
        return Err(invalid());
    }
    origin(&url)
}

/// The operator's pins. `exact` maps a credential file name, spelled exactly as
/// the operator wrote it, to the origins its content may be sent to; the file is
/// opened case-exact on Linux, so only the exact spelling is pinned. `folded`
/// holds the same names lowercased, for the checks that must not be fooled by
/// a case-insensitive file system.
struct Pins {
    exact: BTreeMap<String, BTreeSet<String>>,
    folded: BTreeSet<String>,
}

fn parse_pins(pins: &BTreeMap<String, String>) -> Result<Pins> {
    let mut parsed = Pins {
        exact: BTreeMap::new(),
        folded: BTreeSet::new(),
    };
    for (name, origins) in pins {
        require_relative_name("connector_credentials", Path::new(name))?;
        let allowed = origins
            .split(',')
            .map(pinned_origin)
            .collect::<Result<BTreeSet<_>>>()?;
        if allowed.is_empty() || allowed.len() > 32 {
            return Err(Error::bad(
                "connector_credentials pins one to 32 origins per credential file",
            ));
        }
        parsed.folded.insert(name.to_ascii_lowercase());
        parsed.exact.insert(name.clone(), allowed);
    }
    Ok(parsed)
}

/// The operator's `connector_credentials`, checked when `riauth.toml` loads.
/// They are inert while `connector_secret_dir` is unset, and stay checked so a
/// typo is found before the operator sets it.
pub(crate) fn validate_pins(pins: &BTreeMap<String, String>) -> Result<()> {
    if pins.len() > 256 {
        return Err(Error::bad("Configure at most 256 connector_credentials"));
    }
    parse_pins(pins)?;
    let mut spellings = BTreeSet::new();
    if !pins
        .keys()
        .all(|name| spellings.insert(name.to_ascii_lowercase()))
    {
        return Err(Error::bad(
            "connector_credentials names must differ by more than case",
        ));
    }
    Ok(())
}

/// A definition may name a credential file only if the operator pinned that
/// exact spelling, and every origin that file or a token derived from it is
/// sent to is among the pinned ones. The message names no pin.
fn check_pins(pins: &Pins, kind: Kind, id: &str, definition: &Definition) -> Result<()> {
    for (path, reach) in definition.credential_reach()? {
        // Exact spelling: `Token` pinned does not pin an unpinned `token` file.
        match pins.exact.get(path.to_string_lossy().as_ref()) {
            None => {
                return Err(Error::conflict(format!(
                    "{} {id}: a credential file it names is not pinned by the operator (connector_credentials in riauth.toml)",
                    kind.label()
                )));
            }
            Some(allowed) if !reach.is_subset(allowed) => {
                return Err(Error::conflict(format!(
                    "{} {id}: a credential file it names is pinned to other origins than the endpoints this definition sends it to",
                    kind.label()
                )));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// A CA or certificate file name must not be a name that is, or may become, a
/// credential: bound, owned by another definition, or pinned.
fn check_non_secret(
    bound: &BTreeMap<String, String>,
    pins: &Pins,
    kind: Kind,
    id: &str,
    definition: &Definition,
) -> Result<()> {
    for (_, secret, path) in definition.path_fields() {
        let name = path.to_string_lossy().to_ascii_lowercase();
        if !secret && (bound.contains_key(&name) || pins.folded.contains(&name)) {
            return Err(Error::conflict(format!(
                "{} {id}: a CA or certificate file name must not be a credential file name",
                kind.label()
            )));
        }
    }
    Ok(())
}

/// The tables written in `riauth.toml`, as resolved entries. An id that this
/// process merged from a stored row is not one of them.
fn toml_entries(config: &Config, loaded: &Loaded) -> Vec<Entry> {
    let mut out = Vec::new();
    for (id, value) in &config.directories {
        out.push(Entry {
            kind: Kind::Ldap,
            id: id.clone(),
            definition: Definition::Ldap(value.clone()),
        });
    }
    for (id, value) in &config.workspace_directories {
        out.push(Entry {
            kind: Kind::Workspace,
            id: id.clone(),
            definition: Definition::Workspace(value.clone()),
        });
    }
    for (id, value) in &config.entra_directories {
        out.push(Entry {
            kind: Kind::Entra,
            id: id.clone(),
            definition: Definition::Entra(value.clone()),
        });
    }
    for (id, value) in &config.scim_targets {
        out.push(Entry {
            kind: Kind::Scim,
            id: id.clone(),
            definition: Definition::Scim(value.clone()),
        });
    }
    out.retain(|entry| !loaded.contains(entry.kind, &entry.id));
    out
}

/// Limits per kind and one owner per credential file, across `riauth.toml` and
/// stored definitions together. Reusing a connector's credential file under a
/// new id is how a credential would be sent to another destination.
fn check_set(entries: &[Entry]) -> Result<()> {
    for kind in Kind::ALL {
        if entries.iter().filter(|entry| entry.kind == kind).count() > kind.cap() {
            return Err(Error::bad(format!(
                "Configure at most {} {} entries",
                kind.cap(),
                kind.label()
            )));
        }
    }
    let mut owners: BTreeMap<PathBuf, (Kind, &str)> = BTreeMap::new();
    for entry in entries {
        for path in entry.definition.secret_paths() {
            let owner = (entry.kind, entry.id.as_str());
            if let Some(other) = owners.insert(fold(&path)?, owner)
                && other != owner
            {
                return Err(Error::conflict(
                    "A connector credential file may belong to one connector only",
                ));
            }
        }
    }
    for entry in entries {
        for (_, secret, path) in entry.definition.path_fields() {
            if !secret && owners.contains_key(&fold(&path)?) {
                return Err(Error::conflict(
                    "A CA or certificate file name must not be a credential file name",
                ));
            }
        }
    }
    Ok(())
}

/// Every stored row except the ones in `skip`, checked against its key, format
/// and digest. A row that fails is a conflict: the store is not trusted to
/// describe a connector. A row being retired is neither read nor validated, so
/// a damaged row can still be retired.
pub(crate) fn load_rows(
    tx: &Tx<'_>,
    skip: &BTreeSet<(Kind, String)>,
) -> Result<BTreeMap<(Kind, String), StoredDefinition>> {
    let mut rows = BTreeMap::new();
    for (stored_key, raw) in tx.list::<Value>(BUCKET)? {
        let named = stored_key
            .split_once('/')
            .and_then(|(kind, id)| Kind::parse(kind).map(|kind| (kind, id.to_owned())));
        if named.as_ref().is_some_and(|named| skip.contains(named)) {
            continue;
        }
        let row: StoredDefinition = serde_json::from_value(raw)
            .map_err(|_| Error::conflict("Stored connector definition is malformed"))?;
        if row.format != FORMAT {
            return Err(Error::conflict(
                "Stored connector definition format requires a newer release",
            ));
        }
        if stored_key != key(row.kind, &row.id)
            || row.digest != definition_digest(row.kind, &row.definition)
        {
            return Err(Error::conflict(
                "Stored connector definition does not match its key or digest",
            ));
        }
        rows.insert((row.kind, row.id.clone()), row);
    }
    Ok(rows)
}

fn resolved_entry(directory: &Path, row: &StoredDefinition) -> Result<Entry> {
    if !cfg!(feature = "platform") && matches!(row.kind, Kind::Workspace | Kind::Entra) {
        return Err(Error::conflict(format!(
            "A stored {} {} exists, and this Essentials build cannot plan or load it. Retire it with retired_connectors, or run the Platform build; the edition transition preflight lists it as a blocker. An Essentials build refuses to start on such a row whatever connector_secret_dir says",
            row.kind.label(),
            row.id
        )));
    }
    let mut definition = Definition::from_value(row.kind, row.definition.clone())?;
    validate_entry(row.kind, &row.id, &definition)?;
    definition.resolve(directory);
    Ok(Entry {
        kind: row.kind,
        id: row.id.clone(),
        definition,
    })
}

/// Credential file names bound to a destination and identity, as stored plus
/// what rows stored before bindings existed still name.
struct Bindings {
    bound: BTreeMap<String, String>,
    persisted: BTreeSet<String>,
    written: BTreeSet<String>,
}

impl Bindings {
    fn load(tx: &Tx<'_>) -> Result<Self> {
        let mut bindings = Self {
            bound: BTreeMap::new(),
            persisted: BTreeSet::new(),
            written: BTreeSet::new(),
        };
        for (name, binding) in tx.list::<Binding>(BINDINGS)? {
            if binding.format != BINDING_FORMAT || binding.name != name {
                return Err(Error::conflict(
                    "Stored connector credential binding does not match its key or format",
                ));
            }
            bindings.persisted.insert(name.clone());
            bindings.bound.insert(name, binding.binding);
        }
        Ok(bindings)
    }

    /// A definition stored before bindings existed keeps what it names today.
    fn overlay(&mut self, definition: &Definition) -> Result<()> {
        let digest = definition.binding_digest()?;
        for name in definition.credential_names() {
            self.bound.entry(name).or_insert_with(|| digest.clone());
        }
        Ok(())
    }

    /// Write the binding of every name `definition` uses that has none yet.
    fn claim(
        &mut self,
        tx: &Tx<'_>,
        actor: &Principal,
        kind: Kind,
        id: &str,
        definition: &Definition,
    ) -> Result<()> {
        let binding = definition.binding_digest()?;
        for name in definition.credential_names() {
            if !self.persisted.contains(&name) && self.written.insert(name.clone()) {
                tx.put(
                    BINDINGS,
                    &name,
                    &Binding {
                        format: BINDING_FORMAT.into(),
                        name: name.clone(),
                        binding: binding.clone(),
                        first_kind: kind,
                        first_id: id.to_owned(),
                        bound_at: now(),
                        bound_by: actor.id.clone(),
                    },
                )?;
            }
        }
        Ok(())
    }
}

/// Write the manifest's definitions and retirements. The only writer. Preview
/// validates and reports and writes nothing. Omitted definitions are untouched.
pub(crate) fn reconcile(
    config: &Config,
    loaded: &Loaded,
    tx: &Tx<'_>,
    actor: &Principal,
    manifest: &Manifest,
    preview: bool,
) -> Result<Vec<Change>> {
    let mut desired = manifest_definitions(manifest);
    let retired = retired_keys(manifest);
    if desired.is_empty() && retired.is_empty() {
        return Ok(Vec::new());
    }
    require_administrator(actor)?;
    validate_manifest(manifest)?;
    desired.sort_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)));
    let configured = toml_entries(config, loaded);
    for (kind, id, _) in &desired {
        if configured
            .iter()
            .any(|entry| entry.kind == *kind && entry.id == *id)
        {
            return Err(Error::conflict(format!(
                "{} {id} is defined in riauth.toml; remove that entry or choose another id",
                kind.label()
            )));
        }
    }
    for (kind, id) in &retired {
        if configured
            .iter()
            .any(|entry| entry.kind == *kind && entry.id == *id)
        {
            return Err(Error::conflict(format!(
                "{} {id} is defined in riauth.toml and cannot be retired through desired state; remove that entry from riauth.toml",
                kind.label()
            )));
        }
    }
    let mut bindings = Bindings::load(tx)?;
    let mut changes = Vec::new();
    if !desired.is_empty() {
        let rows = load_rows(tx, &retired)?;
        for row in rows.values() {
            bindings.overlay(&Definition::from_value(row.kind, row.definition.clone())?)?;
        }
        changes.extend(define(
            config,
            tx,
            actor,
            configured,
            &desired,
            &rows,
            &mut bindings,
            &retired,
            preview,
        )?);
    }
    for (kind, id) in retired {
        if let Some(change) = retire(tx, actor, kind, &id, &mut bindings, preview)? {
            changes.push(change);
        }
    }
    Ok(changes)
}

/// Delete one stored row and keep every credential binding it holds. A row
/// that is not stored is no change. A damaged row can be retired too: it is
/// read as far as it parses and never validated.
fn retire(
    tx: &Tx<'_>,
    actor: &Principal,
    kind: Kind,
    id: &str,
    bindings: &mut Bindings,
    preview: bool,
) -> Result<Option<Change>> {
    let stored = key(kind, id);
    let Some(raw) = tx.get::<Value>(BUCKET, &stored)? else {
        return Ok(None);
    };
    let parsed = serde_json::from_value::<StoredDefinition>(raw.clone())
        .ok()
        .filter(|row| row.kind == kind && row.id == id);
    let before = match &parsed {
        Some(row) => view(row),
        None => json!({
            "kind": kind.as_str(),
            "id": id,
            "revision": raw["revision"],
            "digest": raw["digest"],
        }),
    };
    if !preview {
        // The names this row holds stay bound, including a legacy row's names.
        if let Some(row) = &parsed
            && let Ok(definition) = Definition::from_value(kind, row.definition.clone())
        {
            bindings.claim(tx, actor, kind, id, &definition)?;
        }
        tx.delete(BUCKET, &stored)?;
    }
    Ok(Some(Change {
        resource: format!("connector.{}/{id}", kind.as_str()),
        action: "retire".into(),
        before,
        after: Value::Null,
        credential_change: false,
        secret_references: BTreeSet::new(),
    }))
}

#[allow(clippy::too_many_arguments)]
fn define(
    config: &Config,
    tx: &Tx<'_>,
    actor: &Principal,
    configured: Vec<Entry>,
    desired: &[(Kind, String, Definition)],
    rows: &BTreeMap<(Kind, String), StoredDefinition>,
    bindings: &mut Bindings,
    retired: &BTreeSet<(Kind, String)>,
    preview: bool,
) -> Result<Vec<Change>> {
    let directory = secret_dir(config)?;
    let mut entries = configured;
    for ((kind, id), row) in rows {
        if !desired
            .iter()
            .any(|(wanted, name, _)| wanted == kind && name == id)
            && !retired.contains(&(*kind, id.clone()))
        {
            entries.push(resolved_entry(directory, row)?);
        }
    }
    for (kind, id, definition) in desired {
        let mut resolved = definition.clone();
        resolved.resolve(directory);
        entries.push(Entry {
            kind: *kind,
            id: id.clone(),
            definition: resolved,
        });
    }
    check_set(&entries)?;
    // A credential file keeps the destination and identity it was first bound
    // to, even after the definition that named it stopped naming it or was retired.
    for (kind, id) in retired {
        if let Some(raw) = tx.get::<Value>(BUCKET, &key(*kind, id))?
            && let Ok(row) = serde_json::from_value::<StoredDefinition>(raw)
            && let Ok(definition) = Definition::from_value(*kind, row.definition)
        {
            bindings.overlay(&definition)?;
        }
    }
    let pins = parse_pins(&config.connector_credentials)?;
    for (kind, id, definition) in desired {
        check_pins(&pins, *kind, id, definition)?;
        check_non_secret(&bindings.bound, &pins, *kind, id, definition)?;
        let wanted = definition.binding_digest()?;
        if definition.credential_names().iter().any(|name| {
            bindings
                .bound
                .get(name)
                .is_some_and(|existing| *existing != wanted)
        }) {
            return Err(Error::conflict(format!(
                "{} {id}: a credential file it names is bound to another destination or identity; provision a new file name",
                kind.label()
            )));
        }
    }
    let mut changes = Vec::new();
    for (kind, id, definition) in desired {
        let (kind, id) = (*kind, id.as_str());
        let after = definition.to_value()?;
        let after_digest = definition_digest(kind, &after);
        let existing = rows.get(&(kind, id.to_owned()));
        if existing.is_some_and(|row| row.digest == after_digest) {
            continue;
        }
        let mut credential_change = true;
        let mut previous: Option<Definition> = None;
        if let Some(row) = existing {
            let before = Definition::from_value(kind, row.definition.clone())?;
            let moved = before.binding_digest()? != definition.binding_digest()?;
            let (old, new) = (before.secret_paths(), definition.secret_paths());
            // Keeping any one credential file while moving the destination
            // would send that credential to the new place.
            if moved && !old.is_disjoint(&new) {
                return Err(Error::conflict(format!(
                    "{} {id}: changing where credentials are sent requires new references for every credential file",
                    kind.label()
                )));
            }
            credential_change = moved || old != new || before.scopes() != definition.scopes();
            previous = Some(before);
        }
        let revision = existing.map_or(1, |row| row.revision.saturating_add(1));
        let row = StoredDefinition {
            format: FORMAT.into(),
            kind,
            id: id.to_owned(),
            revision,
            digest: after_digest,
            definition: after,
            updated_at: now(),
            updated_by: actor.id.clone(),
        };
        let change = Change {
            resource: format!("connector.{}/{id}", kind.as_str()),
            action: if existing.is_some() {
                "update"
            } else {
                "create"
            }
            .into(),
            before: existing.map(view).unwrap_or(Value::Null),
            after: view(&row),
            credential_change,
            // File fields are references the server reads itself. The CLI must
            // never read or upload them.
            secret_references: BTreeSet::new(),
        };
        if !preview {
            tx.put(BUCKET, &key(kind, id), &row)?;
            // The previous definition's names first: a legacy row's files are
            // bound now, before this edit releases them.
            if let Some(before) = &previous {
                bindings.claim(tx, actor, kind, id, before)?;
            }
            bindings.claim(tx, actor, kind, id, definition)?;
        }
        changes.push(change);
    }
    Ok(changes)
}

fn refuse(error: Error) -> Error {
    Error::bad(format!(
        "Stored connector definitions cannot be loaded: {}. Remove the conflicting riauth.toml entry or restore the missing connector_credentials pin, or unset connector_secret_dir in riauth.toml to leave stored definitions dormant",
        error.message.trim_end_matches('.')
    ))
}

/// Copy the stored definitions into this process's configuration. Called once
/// by `Core::open_store`, read-only, before any worker or listener starts.
///
/// With `connector_secret_dir` unset nothing is read and nothing is loaded:
/// stored rows stay dormant, which is the operator's way back to `riauth.toml`
/// alone. (An Essentials build refuses to start on a stored Workspace or Entra
/// row before this runs, whatever `connector_secret_dir` says.) With it set, a
/// row that fails the same validation as a `riauth.toml` table, names a
/// credential file the operator has not pinned to the right origins, disagrees
/// with a persisted credential binding, or has an id that `riauth.toml` defines
/// differently, refuses the open. A `riauth.toml` entry identical to its stored
/// row is a configuration that was already merged and is a no-op. The merged
/// configuration passes the same `Config::validate` gates, including edition
/// and capability rules.
pub(crate) fn merge(
    mut config: Config,
    store: &Store,
    admission: &crate::config::ExtensionAdmission,
) -> Result<(Config, Loaded)> {
    if !admission.matches(&config.workflow_extensions) {
        return Err(Error::bad(
            "Workflow extension configuration requires fresh startup validation",
        ));
    }
    let Some(directory) = config.connector_secret_dir.clone() else {
        return Ok((config, Loaded::default()));
    };
    let rows = store
        .read(|tx| load_rows(tx, &BTreeSet::new()))
        .map_err(refuse)?;
    if rows.is_empty() {
        return Ok((config, Loaded::default()));
    }
    secret_dir(&config).map_err(refuse)?;
    let pins = parse_pins(&config.connector_credentials).map_err(refuse)?;
    let persisted = store.read(Bindings::load).map_err(refuse)?;
    for ((kind, id), row) in &rows {
        let definition = Definition::from_value(*kind, row.definition.clone()).map_err(refuse)?;
        validate_entry(*kind, id, &definition).map_err(refuse)?;
        check_pins(&pins, *kind, id, &definition).map_err(refuse)?;
        check_non_secret(&persisted.bound, &pins, *kind, id, &definition).map_err(refuse)?;
        let wanted = definition.binding_digest().map_err(refuse)?;
        if definition.credential_names().iter().any(|name| {
            persisted
                .bound
                .get(name)
                .is_some_and(|existing| *existing != wanted)
        }) {
            return Err(refuse(Error::conflict(format!(
                "{} {id}: a credential file it names is bound to another destination or identity",
                kind.label()
            ))));
        }
    }
    let mut loaded = Loaded::default();
    let mut configured = toml_entries(&config, &loaded);
    let mut stored = Vec::new();
    for ((kind, id), row) in &rows {
        let entry = resolved_entry(&directory, row).map_err(refuse)?;
        let resolved = entry.definition.to_value().map_err(refuse)?;
        let existing = configured
            .iter()
            .find(|candidate| candidate.kind == *kind && candidate.id == *id);
        if let Some(existing) = existing {
            if existing.definition.to_value().map_err(refuse)? != resolved {
                return Err(refuse(Error::conflict(format!(
                    "{} {id} is defined in riauth.toml and differently in a stored definition",
                    kind.label()
                ))));
            }
        } else {
            match &entry.definition {
                Definition::Ldap(value) => {
                    config.directories.insert(id.clone(), value.clone());
                }
                Definition::Workspace(value) => {
                    config
                        .workspace_directories
                        .insert(id.clone(), value.clone());
                }
                Definition::Entra(value) => {
                    config.entra_directories.insert(id.clone(), value.clone());
                }
                Definition::Scim(value) => {
                    config.scim_targets.insert(id.clone(), value.clone());
                }
            }
        }
        loaded
            .entries
            .insert((*kind, id.clone()), (row.revision, row.digest.clone()));
        stored.push(entry);
    }
    configured.retain(|entry| !loaded.contains(entry.kind, &entry.id));
    configured.extend(stored);
    check_set(&configured).map_err(refuse)?;
    config
        .validate_reusing_extension_admission(admission)
        .map_err(|error| refuse(Error::bad(error.to_string())))?;
    Ok((config, loaded))
}

/// Per stored definition: its revision and digest, whether this process loaded
/// exactly that revision, and whether a restart would load it. For a full human
/// administrator only. Absent when no definition is stored and the operator has
/// not opted in.
pub(crate) fn status(
    tx: &Tx<'_>,
    config: &Config,
    loaded: &Loaded,
    actor: &Principal,
) -> Result<Option<Value>> {
    if actor.agent || actor.delegated {
        return Ok(None);
    }
    let enabled = config.connector_secret_dir.is_some();
    let rows = load_rows(tx, &BTreeSet::new())?;
    if rows.is_empty() && loaded.entries.is_empty() && !enabled {
        return Ok(None);
    }
    let mut definitions: Vec<Value> = rows
        .values()
        .map(|row| {
            let here = loaded.entries.get(&(row.kind, row.id.clone()));
            let current = here == Some(&(row.revision, row.digest.clone()));
            json!({
                "kind": row.kind.as_str(),
                "id": row.id,
                "revision": row.revision,
                "digest": row.digest,
                "loaded_in_this_process": current,
                "loaded_revision": here.map(|(revision, _)| revision),
                "restart_required": enabled && !current,
            })
        })
        .collect();
    // Retired while this process still runs it: gone after the next start.
    for ((kind, id), (revision, digest)) in &loaded.entries {
        if !rows.contains_key(&(*kind, id.clone())) {
            definitions.push(json!({
                "kind": kind.as_str(),
                "id": id,
                "revision": null,
                "digest": null,
                "loaded_in_this_process": true,
                "loaded_revision": revision,
                "loaded_digest": digest,
                "retired": true,
                "restart_required": true,
            }));
        }
    }
    Ok(Some(json!({
        "connector_secret_dir": enabled,
        "definitions": definitions,
    })))
}

/// One audit event per applied definition or retirement. It records the kind,
/// id, revision and digest, never a file name or secret.
pub(crate) fn audit_change(tx: &Tx<'_>, actor: &str, change: &Change) -> Result<()> {
    if change.action == "retire" {
        return audit_with_details(
            tx,
            actor,
            "connector.retire",
            &change.resource,
            json!({
                "connector_kind": change.before["kind"],
                "connector_id": change.before["id"],
                "retired_revision": change.before["revision"],
                "digest": change.before["digest"],
            }),
        );
    }
    audit_with_details(
        tx,
        actor,
        "connector.define",
        &change.resource,
        json!({
            "connector_kind": change.after["kind"],
            "connector_id": change.after["id"],
            "revision": change.after["revision"],
            "previous_revision": change.before["revision"],
            "digest": change.after["digest"],
            "credential_change": change.credential_change,
        }),
    )
}

/// A connector change as the aggregate `state.apply` audit row records it: the
/// kind, id, revision and digest only. The applying administrator's own result
/// keeps the full definition; the audit row is readable by `audit.read` holders.
pub(crate) fn audit_view(change: &Change) -> Value {
    json!({
        "resource": change.resource,
        "action": change.action,
        "credential_change": change.credential_change,
        "before": if change.before.is_null() {
            Value::Null
        } else {
            json!({"revision": change.before["revision"], "digest": change.before["digest"]})
        },
        "after": if change.after.is_null() {
            Value::Null
        } else {
            json!({
                "kind": change.after["kind"],
                "id": change.after["id"],
                "revision": change.after["revision"],
                "digest": change.after["digest"],
            })
        },
    })
}

/// Stored definitions for a full human administrator. Agents and delegated
/// humans receive none, as for delegated grants.
pub(crate) fn export_into(tx: &Tx<'_>, actor: &Principal, manifest: &mut Manifest) -> Result<()> {
    if actor.agent || actor.delegated {
        return Ok(());
    }
    for ((kind, id), row) in load_rows(tx, &BTreeSet::new())? {
        match Definition::from_value(kind, row.definition)? {
            Definition::Ldap(value) => {
                manifest.directories.insert(id, value);
            }
            Definition::Workspace(value) => {
                manifest.workspace_directories.insert(id, value);
            }
            Definition::Entra(value) => {
                manifest.entra_directories.insert(id, value);
            }
            Definition::Scim(value) => {
                manifest.scim_targets.insert(id, value);
            }
        }
    }
    Ok(())
}

impl Manifest {
    /// Whether the manifest names a connector definition or retirement.
    pub(crate) fn has_connectors(&self) -> bool {
        !self.directories.is_empty()
            || !self.workspace_directories.is_empty()
            || !self.entra_directories.is_empty()
            || !self.scim_targets.is_empty()
            || !self.retired_connectors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin_of(text: &str) -> String {
        origin(&Url::parse(text).unwrap()).unwrap()
    }

    #[test]
    fn origins_compare_by_scheme_normalized_host_and_effective_port() {
        // Case, path and query are not part of an origin.
        assert_eq!(
            origin_of("HTTPS://Example.COM/path?x=1"),
            "https://example.com:443"
        );
        // An explicit default port is the implied one, for every scheme.
        assert_eq!(
            origin_of("https://example.com:443"),
            origin_of("https://example.com")
        );
        assert_eq!(
            origin_of("http://example.test:80"),
            "http://example.test:80"
        );
        assert_eq!(
            origin_of("ldaps://ldap.example.test:636"),
            origin_of("ldaps://LDAP.example.test")
        );
        assert_eq!(
            origin_of("ldaps://ldap.example.test"),
            "ldaps://ldap.example.test:636"
        );
        assert_eq!(
            origin_of("ldap://ldap.example.test:389"),
            origin_of("ldap://ldap.example.test")
        );
        assert_eq!(
            origin_of("ldap://ldap.example.test"),
            "ldap://ldap.example.test:389"
        );
        // Another port, another scheme, or another host is another origin.
        assert_ne!(
            origin_of("ldaps://h.example.test"),
            origin_of("ldaps://h.example.test:6360")
        );
        assert_ne!(
            origin_of("ldap://h.example.test"),
            origin_of("ldaps://h.example.test")
        );
        assert_ne!(
            origin_of("https://a.example.test"),
            origin_of("https://b.example.test")
        );
        // An HTTP host's trailing dot names the same DNS host, and its Unicode
        // and punycode spellings agree.
        assert_eq!(
            origin_of("https://example.com."),
            origin_of("https://example.com")
        );
        assert_eq!(
            origin_of("https://BÜCHER.example"),
            origin_of("https://xn--bcher-kva.example")
        );
        // An LDAP host is what the client dials, as written: never decoded or
        // IDNA-mapped, so a host written differently from how it would be read
        // is refused rather than matched to a pin.
        for host in [
            "ldaps://BÜCHER.example",
            "ldaps://l%64ap.example.test",
            "ldap://l%64ap.example.test",
            "ldaps://%C3%BCber.example",
        ] {
            assert!(origin(&Url::parse(host).unwrap()).is_err(), "{host}");
        }
        assert_eq!(
            origin_of("ldaps://xn--bcher-kva.example"),
            "ldaps://xn--bcher-kva.example:636"
        );
        // Its trailing dot and short IPv4 forms stay significant, so a pin for
        // the plain form does not match them.
        assert_ne!(
            origin_of("ldaps://example.com."),
            origin_of("ldaps://example.com")
        );
        assert_ne!(origin_of("ldaps://127.1"), origin_of("ldaps://127.0.0.1"));
        assert_eq!(
            origin_of("ldaps://LDAP.Example.Test."),
            "ldaps://ldap.example.test.:636"
        );
        // IP literals are canonical.
        assert_eq!(origin_of("https://[::1]"), "https://[::1]:443");
        assert_eq!(
            origin_of("https://[0:0:0:0:0:0:0:1]:8443"),
            "https://[::1]:8443"
        );
        assert_eq!(origin_of("ldaps://[::1]"), "ldaps://[::1]:636");
        assert_eq!(
            origin_of("https://127.0.0.1:8443"),
            "https://127.0.0.1:8443"
        );
        // Only the four connector schemes have an origin.
        assert!(origin(&Url::parse("ftp://example.test").unwrap()).is_err());
        assert!(origin(&Url::parse("file:///etc/passwd").unwrap()).is_err());
    }

    #[test]
    fn a_pinned_origin_is_a_bare_origin_the_operator_can_trust() {
        for good in [
            "https://idp.example.test",
            "https://idp.example.test/",
            "  https://IDP.example.test:8443 ",
            "ldaps://ldap.example.test:636",
            "ldap://ldap.example.test",
            "http://localhost:8080",
            "http://[::1]:8080",
            "http://LOCALHOST",
            "http://127.0.0.1:9000",
        ] {
            assert!(pinned_origin(good).is_ok(), "{good:?}");
        }
        assert_eq!(
            pinned_origin("  https://IDP.example.test:443 ").unwrap(),
            "https://idp.example.test:443"
        );
        assert_eq!(
            pinned_origin("http://LOCALHOST").unwrap(),
            "http://localhost:80"
        );
        for bad in [
            "",
            "idp.example.test",
            "https://user@idp.example.test",
            "https://user:secret@idp.example.test",
            "https://idp.example.test/token",
            "https://idp.example.test?x=1",
            "https://idp.example.test#frag",
            "http://idp.example.test",
            "http://10.0.0.5",
            "ftp://idp.example.test",
            "file:///etc/passwd",
            "https://",
            "ldaps://l%64ap.example.test",
            "ldaps://BÜCHER.example",
        ] {
            assert!(pinned_origin(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn a_pin_matches_the_credential_name_exactly_and_the_other_checks_fold_case() {
        let pins = parse_pins(&BTreeMap::from([(
            "scim/Token".to_owned(),
            "https://scim.example.test".to_owned(),
        )]))
        .unwrap();
        assert!(pins.exact.contains_key("scim/Token"));
        assert!(!pins.exact.contains_key("scim/token"));
        assert!(pins.folded.contains("scim/token"));
        // Two spellings are one pin only when the operator wrote them both.
        let duplicate = BTreeMap::from([
            ("scim/Token".to_owned(), "https://a.example.test".to_owned()),
            ("scim/token".to_owned(), "https://a.example.test".to_owned()),
        ]);
        assert!(validate_pins(&duplicate).is_err());
        assert!(parse_pins(&duplicate).is_ok());
    }
}
