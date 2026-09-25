//! Offline Authentik API export conversion. Never execute exported Python policies.
use crate::{
    claims,
    config::validate_server_url,
    crypto,
    error::{Error, Result},
    model::ProviderSettings,
    state::{ClientSpec, GroupSpec, Manifest, UserSpec},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const MAX_GROUP_ANCESTORS: usize = 1024;

#[derive(schemars::JsonSchema, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Import {
    #[serde(default)]
    pub source_links: Vec<crate::source::LinkSpec>,
    #[serde(default)]
    pub totp: BTreeMap<String, PasswordReference>,
    #[serde(default)]
    pub source_resolutions: BTreeMap<String, crate::source::SourceSpec>,
    pub api_version: String,
    pub issuer: String,
    pub users: Value,
    pub groups: Value,
    pub providers: Value,
    pub applications: Value,
    pub policy_bindings: Value,
    pub sources: Value,
    pub passwords: BTreeMap<String, PasswordReference>,
    pub clients: BTreeMap<String, ClientResolution>,
}
#[derive(schemars::JsonSchema, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PasswordReference {
    pub reference: String,
    pub version: String,
    #[serde(default)]
    pub hashed: bool,
}
#[derive(schemars::JsonSchema, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientResolution {
    #[serde(default)]
    pub federation_reviewed: bool,
    /// Exact issuer obtained from this provider's discovery document.
    pub issuer: String,
    pub scopes: BTreeSet<String>,
    /// Reviewed replacement for exported policy and mapping expressions.
    pub settings: ProviderSettings,
    pub translated_mapping_ids: BTreeSet<String>,
    pub translated_binding_ids: BTreeSet<String>,
    pub authentication_flow_reviewed: bool,
    pub require_mfa: bool,
    pub secret_ref: Option<String>,
    pub secret_version: Option<String>,
}
fn rows(value: &Value) -> Result<&[Value]> {
    if let Some(rows) = value.as_array() {
        return Ok(rows);
    }
    let rows = value["results"]
        .as_array()
        .ok_or_else(|| Error::bad("Exports must contain an array or complete API results"))?;
    let pagination = &value["pagination"];
    if pagination["count"].as_u64() != Some(rows.len() as u64)
        || pagination["next"].as_u64().is_some_and(|n| n > 0)
        || value["next"].as_str().is_some()
    {
        return Err(Error::bad(
            "Incomplete paginated export; collect every page into an array first",
        ));
    }
    Ok(rows)
}
fn identifier(value: &Value) -> Result<String> {
    match value {
        Value::String(s) if !s.is_empty() => Ok(s.clone()),
        Value::Number(n) if n.is_u64() => Ok(n.to_string()),
        _ => Err(Error::bad("Missing export identifier")),
    }
}
fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value[name]
        .as_str()
        .ok_or_else(|| Error::bad(format!("Export is missing {name}")))
}
fn ids(value: &Value) -> Result<BTreeSet<String>> {
    value
        .as_array()
        .ok_or_else(|| Error::bad("Export relation must be an array"))?
        .iter()
        .map(identifier)
        .collect()
}
fn duration(value: &Value) -> Result<Option<u64>> {
    let Some(value) = value.as_str() else {
        return Ok(None);
    };
    let mut total = 0u64;
    for pair in value.split(';') {
        let (unit, n) = pair
            .split_once('=')
            .ok_or_else(|| Error::bad("Unsupported Authentik duration"))?;
        let multiplier = match unit {
            "seconds" => 1,
            "minutes" => 60,
            "hours" => 3600,
            "days" => 86400,
            "weeks" => 604800,
            _ => return Err(Error::bad("Unsupported Authentik duration unit")),
        };
        total = total
            .checked_add(
                n.parse::<u64>()
                    .map_err(|_| Error::bad("Invalid duration"))?
                    .checked_mul(multiplier)
                    .ok_or_else(|| Error::bad("Duration overflow"))?,
            )
            .ok_or_else(|| Error::bad("Duration overflow"))?;
    }
    Ok(Some(total))
}
/// Skipping `seen` groups makes diamonds fine; only a path back to `start` is a cycle.
fn group_ancestors(
    parents: &BTreeMap<String, BTreeSet<String>>,
    start: &str,
) -> Result<BTreeSet<String>> {
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([start]);
    while let Some(at) = queue.pop_front() {
        for parent in parents.get(at).into_iter().flatten() {
            if parent == start {
                return Err(Error::bad("Cycle in exported group hierarchy"));
            }
            if seen.insert(parent.clone()) {
                if seen.len() > MAX_GROUP_ANCESTORS {
                    return Err(Error::bad(format!(
                        "Exported group has more than {MAX_GROUP_ANCESTORS} ancestors"
                    )));
                }
                queue.push_back(parent);
            }
        }
    }
    Ok(seen)
}

pub fn convert(input: Import) -> Result<Value> {
    if input.api_version != "riauth.authentik-import/v1" {
        return Err(Error::bad("Unsupported Authentik import format"));
    }
    validate_server_url(&input.issuer).map_err(|_| Error::bad("Invalid target issuer"))?;
    let users = rows(&input.users)?;
    let groups = rows(&input.groups)?;
    let providers = rows(&input.providers)?;
    let applications = rows(&input.applications)?;
    let bindings = rows(&input.policy_bindings)?;
    let mut blockers = Vec::<String>::new();
    let mut manifest = Manifest {
        api_version: "riauth/v1".into(),
        source_links: input.source_links.clone(),
        ..Default::default()
    };
    for source in rows(&input.sources)? {
        let id = identifier(&source["pk"])?;
        match input.source_resolutions.get(&id) {
            Some(spec) => { spec.source.validate()?; manifest.sources.push(spec.clone()); }
            None => blockers.push(format!("Source {id}: provide an explicitly reviewed OIDC source resolution; non-OIDC sources need their own adapter")),
        }
    }
    let mut group_names = BTreeMap::new();
    let mut parents = BTreeMap::<String, BTreeSet<String>>::new();
    for group in groups {
        let id = identifier(&group["pk"])?;
        let name = field(group, "name")?.to_owned();
        if group_names.insert(id.clone(), name.clone()).is_some() {
            return Err(Error::bad("Duplicate exported group ID"));
        }
        // Authentik 2025.12 exports a `parents` array; older exports a scalar `parent`.
        let mut direct = match &group["parents"] {
            Value::Null => BTreeSet::new(),
            list => ids(list)?,
        };
        if !group["parent"].is_null() {
            direct.insert(identifier(&group["parent"])?);
        }
        if !direct.is_empty() {
            parents.insert(id, direct);
        }
        manifest.groups.push(GroupSpec {
            name,
            members: BTreeSet::new(),
        });
    }
    // Every exported group is walked, so a cycle fails even where no user reaches it.
    let ancestors = group_names
        .keys()
        .map(|id| Ok((id.as_str(), group_ancestors(&parents, id)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut subject_modes = BTreeMap::new();
    let mut resolved_bindings = BTreeSet::new();
    for provider in providers {
        let cid = field(provider, "client_id")?.to_owned();
        let Some(resolution) = input.clients.get(&cid) else {
            blockers.push(format!("{cid}: provide reviewed issuer, scopes, mappings, policies and authentication requirements in clients"));
            continue;
        };
        validate_server_url(&resolution.issuer)
            .map_err(|_| Error::bad("Invalid reviewed provider issuer"))?;
        if !resolution.authentication_flow_reviewed {
            blockers.push(format!(
                "{cid}: authentication flow has not been reviewed for browser/terminal login and MFA"
            ));
        }
        let exported_mappings = ids(&provider["property_mappings"])?;
        if exported_mappings != resolution.translated_mapping_ids {
            blockers.push(format!("{cid}: every exported property mapping must have a reviewed declarative translation"));
        }
        resolved_bindings.extend(resolution.translated_binding_ids.clone());
        for name in ["jwt_federation_sources", "jwt_federation_providers"] {
            if provider[name].as_array().is_some_and(|v| !v.is_empty())
                && !resolution.federation_reviewed
            {
                blockers.push(format!("{cid}: review {name} and translate its trust into pinned machine_trust, exchange and token_managers settings"));
            }
        }
        if !provider["encryption_key"].is_null()
            && (resolution.settings.id_token_encryption.is_none()
                || resolution.settings.access_token_encryption.is_none())
        {
            blockers.push(format!(
                "{cid}: configure the reviewed public encryption key for both ID and access tokens"
            ));
        }
        let mut settings = resolution.settings.clone();
        let provider_id = identifier(&provider["pk"])?;
        let associated = applications
            .iter()
            .filter(|a| identifier(&a["provider"]).ok().as_ref() == Some(&provider_id))
            .collect::<Vec<_>>();
        if associated.len() > 1 {
            blockers.push(format!("{cid}: multiple applications share this provider; resolve their separate launch metadata and policies explicitly"));
        }
        let application = associated.first().copied();
        if settings.app.is_none()
            && let Some(application) = application
        {
            settings.app = Some(crate::portal::Settings {
                launch_url: application["meta_launch_url"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(String::from),
                description: application["meta_description"]
                    .as_str()
                    .unwrap_or("")
                    .into(),
                category: application["group"].as_str().unwrap_or("").into(),
                hidden: application["meta_launch_url"] == "blank://blank"
                    || application["hide_from_application_dashboard"]
                        .as_bool()
                        .unwrap_or(false),
                ..Default::default()
            });
            if let Some(app) = &mut settings.app
                && app.launch_url.as_deref() == Some("blank://blank")
            {
                app.launch_url = None;
            }
        }
        settings.issuer = (resolution.issuer != input.issuer).then(|| resolution.issuer.clone());
        let exported_grants = ids(&provider["grant_types"])?;
        if settings.allowed_grants.is_empty() {
            settings.allowed_grants = exported_grants.clone();
        }
        if !exported_grants.is_empty() && !settings.allowed_grants.is_subset(&exported_grants) {
            blockers.push(format!(
                "{cid}: selected grants were not present in the export"
            ));
        }
        if settings.allowed_grants.is_empty() {
            blockers.push(format!("{cid}: explicitly inventory allowed grants; an empty legacy grant list is ambiguous"));
        }
        settings.code_ttl = duration(&provider["access_code_validity"])?;
        settings.access_token_ttl = duration(&provider["access_token_validity"])?;
        settings.refresh_token_ttl = duration(&provider["refresh_token_validity"])?;
        settings.userinfo_only = provider["include_claims_in_id_token"] == false;
        let mut redirects = Vec::new();
        for redirect in provider["redirect_uris"]
            .as_array()
            .ok_or_else(|| Error::bad("Missing redirect_uris"))?
        {
            if redirect["matching_mode"] != "strict" {
                blockers.push(format!(
                    "{cid}: regex redirect needs explicit exact registrations"
                ));
                continue;
            }
            let url = field(redirect, "url")?.to_owned();
            match redirect["redirect_uri_type"]
                .as_str()
                .unwrap_or("authorization")
            {
                "authorization" => redirects.push(url),
                "logout" => {
                    if !settings.post_logout_redirect_uris.contains(&url) {
                        settings.post_logout_redirect_uris.push(url);
                    }
                }
                _ => blockers.push(format!("{cid}: unsupported redirect type")),
            }
        }
        if let Some(uri) = provider["logout_uri"].as_str().filter(|s| !s.is_empty()) {
            if provider["logout_method"] == "backchannel" {
                settings.backchannel_logout_uri = Some(uri.to_owned());
            } else if provider["logout_method"] == "frontchannel" {
                settings.frontchannel_logout_uri = Some(uri.to_owned());
            } else {
                blockers.push(format!("{cid}: unknown logout method"));
            }
        }
        let confidential = match field(provider, "client_type")? {
            "confidential" => true,
            "public" => false,
            _ => return Err(Error::bad("Unknown client type")),
        };
        if confidential && (resolution.secret_ref.is_none() || resolution.secret_version.is_none())
        {
            blockers.push(format!("{cid}: confidential client requires a secret reference/version; exported secrets are never copied into plans"));
        }
        let client = crate::model::Client {
            id: cid.clone(),
            name: application
                .and_then(|a| a["name"].as_str())
                .unwrap_or(field(provider, "name")?)
                .into(),
            secret_hash: confidential.then(|| "import".into()),
            redirect_uris: redirects.clone(),
            scopes: resolution.scopes.clone(),
            allowed_groups: BTreeSet::new(),
            require_mfa: resolution.require_mfa,
            enabled: true,
            service: false,
            settings: settings.clone(),
        };
        if let Err(error) = crate::provider::validate_settings(&client)
            .and_then(|_| claims::validate_mappings(&client))
        {
            blockers.push(format!("{cid}: {}", error.message));
        }
        subject_modes.insert(cid.clone(), field(provider, "sub_mode")?.to_owned());
        manifest.clients.push(ClientSpec {
            client_id: cid,
            name: client.name,
            confidential,
            service: false,
            enabled: true,
            redirect_uris: redirects,
            scopes: resolution.scopes.clone(),
            allowed_groups: BTreeSet::new(),
            require_mfa: resolution.require_mfa,
            settings,
            secret_ref: resolution.secret_ref.clone(),
            secret_version: resolution.secret_version.clone(),
        });
    }
    let exported_bindings = bindings
        .iter()
        .map(|b| identifier(&b["pk"]))
        .collect::<Result<BTreeSet<_>>>()?;
    if exported_bindings != resolved_bindings {
        blockers.push("Every exported policy binding must be inventoried and translated; unresolved or extra binding IDs remain".into());
    }
    let provider_ids = providers
        .iter()
        .map(|p| identifier(&p["pk"]))
        .collect::<Result<BTreeSet<_>>>()?;
    for application in applications {
        if !application["provider"].is_null()
            && !provider_ids.contains(&identifier(&application["provider"])?)
        {
            blockers.push(format!(
                "Application {} uses an unexported or unsupported provider",
                field(application, "slug")?
            ));
        }
        if application["backchannel_providers"]
            .as_array()
            .is_some_and(|v| !v.is_empty())
        {
            blockers.push(format!(
                "Application {} has provisioning providers requiring separate migration",
                field(application, "slug")?
            ));
        }
    }
    for user in users {
        let username = field(user, "username")?.to_owned();
        let password = input.passwords.get(&username);
        let external =
            user["type"] == "external" && input.source_links.iter().any(|l| l.username == username);
        if password.is_none() && !external {
            blockers.push(format!("{username}: supply a password/password-hash reference or complete password reset before migration"));
        }
        if user["type"].as_str().is_some_and(|v| v != "internal") && !external {
            blockers.push(format!("{username}: external/service identity requires explicit source or service-account migration"));
        }
        if user["roles"].as_array().is_some_and(|v| !v.is_empty()) {
            blockers.push(format!("{username}: Authentik administrative roles require explicit agent permission migration"));
        }
        let mut memberships = ids(&user["groups"])?;
        for id in memberships.clone() {
            memberships.extend(ancestors.get(id.as_str()).into_iter().flatten().cloned());
        }
        for id in memberships {
            let name = group_names
                .get(&id)
                .ok_or_else(|| Error::bad("User references unexported group"))?;
            manifest
                .groups
                .iter_mut()
                .find(|g| &g.name == name)
                .unwrap()
                .members
                .insert(username.clone());
        }
        let mut subjects = BTreeMap::new();
        for (cid, mode) in &subject_modes {
            let subject = match mode.as_str() {
                "hashed_user_id" => field(user, "uid")?.to_owned(),
                "user_id" => identifier(&user["pk"])?,
                "user_uuid" => field(user, "uuid")?.to_owned(),
                "user_username" => username.clone(),
                "user_email" => field(user, "email")?.to_owned(),
                "user_upn" => user["attributes"]["upn"]
                    .as_str()
                    .unwrap_or(field(user, "uid")?)
                    .to_owned(),
                _ => {
                    blockers.push(format!("{cid}: unsupported subject mode {mode}"));
                    continue;
                }
            };
            if subject.is_empty()
                || subject.len() > 255
                || !subject.is_ascii()
                || subject.chars().any(char::is_control)
            {
                blockers.push(format!(
                    "{username}/{cid}: subject is outside the supported OIDC format"
                ));
            }
            subjects.insert(cid.clone(), subject);
        }
        let email = user["email"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from);
        manifest.users.push(UserSpec {
            password_disabled: external && password.is_none(),
            totp_ref: input.totp.get(&username).map(|t| t.reference.clone()),
            totp_version: input.totp.get(&username).map(|t| t.version.clone()),
            id: Some(format!("authentik-{}", identifier(&user["pk"])?)),
            username: username.clone(),
            display_name: user["name"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or(&username)
                .into(),
            email,
            email_verified: false,
            enabled: user["is_active"] == true,
            admin: false,
            attributes: serde_json::from_value(user["attributes"].clone())
                .map_err(|_| Error::bad("Invalid user attributes"))?,
            subjects,
            password_ref: password.filter(|p| !p.hashed).map(|p| p.reference.clone()),
            password_hash_ref: password.filter(|p| p.hashed).map(|p| p.reference.clone()),
            password_version: password.map(|p| p.version.clone()),
        });
    }
    for client in &manifest.clients {
        let mut seen = BTreeSet::new();
        for user in &manifest.users {
            if let Some(subject) = user.subjects.get(&client.client_id)
                && !seen.insert(subject)
            {
                blockers.push(format!(
                    "{}: duplicate subjects would merge identities",
                    client.client_id
                ));
            }
        }
    }
    if let Err(error) = manifest.validate() {
        blockers.push(error.message);
    }
    blockers.sort();
    blockers.dedup();
    Ok(
        json!({"api_version": "riauth.migration-report/v1", "ready_for_plan": blockers.is_empty(), "issuer": input.issuer, "blockers": blockers,
        "manifest": if blockers.is_empty() { json!(manifest) } else { Value::Null }, "draft": manifest,
        "reauthentication_required": true, "old_tokens_and_sessions_imported": false, "signing_keys_imported": false, "administrators_imported": false,
        "source_fingerprint": crypto::digest(&serde_json::to_string(&input).map_err(Error::internal)?)}),
    )
}
