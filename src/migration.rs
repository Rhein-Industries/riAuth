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
#[derive(schemars::JsonSchema, Serialize, Deserialize, Default)]
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
/// How much of an exported behavior survives conversion.
#[derive(
    schemars::JsonSchema, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    /// Carried over unchanged.
    Exact,
    /// Carried over through a deterministic, documented transformation.
    Convertible,
    /// Needs reviewed operator input or an operator decision; `blocking` says whether it is still missing.
    Manual,
    /// Cannot be represented or transferred; rebuild, re-enroll or retire it.
    Unsupported,
}
#[derive(
    schemars::JsonSchema, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    User,
    Password,
    Totp,
    Passkey,
    Session,
    Subject,
    Group,
    Source,
    Provider,
    AuthenticationFlow,
    PropertyMapping,
    PolicyBinding,
    Federation,
    SigningKey,
    EncryptionKey,
    ClientSecret,
    Grant,
    TokenLifetime,
    RedirectUri,
    Logout,
    Application,
    Manifest,
    /// A declared inventory element of a source-native type with no riAuth kind; see `source_kind`.
    SourceNative,
}
/// One preflight finding. It never contains credential values.
#[derive(
    schemars::JsonSchema, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Finding {
    pub kind: ItemKind,
    /// The source system's own element type, set only for `source_native` findings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<String>,
    /// Exported identifier, scoped as `client/item` where needed.
    pub id: String,
    pub classification: Classification,
    /// True while this finding keeps `ready_for_plan` false.
    pub blocking: bool,
    pub reason: String,
    pub action: String,
    /// The matching entry in the report's `blockers`.
    pub blocker: Option<String>,
}
#[derive(Default)]
struct Preflight {
    items: Vec<Finding>,
}
impl Preflight {
    fn add(
        &mut self,
        kind: ItemKind,
        id: impl Into<String>,
        classification: Classification,
        reason: impl Into<String>,
        action: impl Into<String>,
    ) -> &mut Finding {
        self.items.push(Finding {
            kind,
            source_kind: None,
            id: id.into(),
            classification,
            blocking: false,
            reason: reason.into(),
            action: action.into(),
            blocker: None,
        });
        self.items.last_mut().unwrap()
    }
    /// Sorted findings, the sorted distinct blockers, and a count per classification plus `blocking`.
    fn finish(self) -> (Vec<Finding>, Vec<String>, BTreeMap<&'static str, usize>) {
        let mut items = self.items;
        items.sort();
        items.dedup();
        let mut blockers = items
            .iter()
            .filter_map(|i| i.blocker.clone())
            .collect::<Vec<_>>();
        blockers.sort();
        blockers.dedup();
        let mut summary = BTreeMap::from([
            ("exact", 0),
            ("convertible", 0),
            ("manual", 0),
            ("unsupported", 0),
        ]);
        for item in &items {
            *summary
                .entry(match item.classification {
                    Classification::Exact => "exact",
                    Classification::Convertible => "convertible",
                    Classification::Manual => "manual",
                    Classification::Unsupported => "unsupported",
                })
                .or_default() += 1;
        }
        summary.insert("blocking", items.iter().filter(|i| i.blocking).count());
        (items, blockers, summary)
    }
}
impl Finding {
    fn block(&mut self, blocker: impl Into<String>) {
        self.blocking = true;
        self.blocker = Some(blocker.into());
    }
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

/// Authentik source family from `/sources/all/` metadata; anything unrecognized stays unsupported.
fn source_family(source: &Value) -> &'static str {
    if source["managed"] == "goauthentik.io/sources/inbuilt" {
        return "inbuilt";
    }
    let model = source["meta_model_name"]
        .as_str()
        .or(source["component"].as_str())
        .unwrap_or("");
    [
        "oauth", "saml", "ldap", "plex", "kerberos", "scim", "telegram",
    ]
    .into_iter()
    .find(|family| model.contains(family))
    .unwrap_or("unrecognized")
}
fn source_adapter(source: &crate::source::Source) -> &'static str {
    if source.saml.is_some() {
        "SAML"
    } else if source.oauth_profile.is_some() {
        "OAuth-profile"
    } else {
        "OIDC"
    }
}
fn subject_source(mode: &str) -> Option<&'static str> {
    Some(match mode {
        "hashed_user_id" => "uid",
        "user_id" => "numeric user ID",
        "user_uuid" => "user UUID",
        "user_username" => "username",
        "user_email" => "email",
        "user_upn" => "upn attribute (uid when absent)",
        _ => return None,
    })
}

pub fn convert(input: Import) -> Result<Value> {
    if input.api_version != AUTHENTIK_FORMAT {
        return Err(Error::bad("Unsupported Authentik import format"));
    }
    validate_server_url(&input.issuer).map_err(|_| Error::bad("Invalid target issuer"))?;
    let users = rows(&input.users)?;
    let groups = rows(&input.groups)?;
    let providers = rows(&input.providers)?;
    let applications = rows(&input.applications)?;
    let bindings = rows(&input.policy_bindings)?;
    let mut p = Preflight::default();
    let mut manifest = Manifest {
        api_version: "riauth/v1".into(),
        source_links: input.source_links.clone(),
        ..Default::default()
    };
    let mut exported_sources = BTreeSet::new();
    for source in rows(&input.sources)? {
        let id = identifier(&source["pk"])?;
        exported_sources.insert(id.clone());
        let family = source_family(source);
        let resolution = input.source_resolutions.get(&id);
        match family {
            "inbuilt" => {
                p.add(ItemKind::Source, &id, Classification::Convertible,
                    "The built-in source is Authentik's local login; exported local users become riAuth local users",
                    "Review each user's password entry; the built-in source itself needs no resolution");
                if resolution.is_some() {
                    p.add(ItemKind::Source, &id, Classification::Manual,
                        "The built-in source cannot be replaced by a source resolution; the resolution was not applied",
                        "Remove this source_resolutions entry")
                        .block(format!("Source {id}: the built-in source cannot take a source resolution"));
                }
            }
            "oauth" | "saml" => {
                let adapter = if family == "oauth" {
                    "OIDC or OAuth-profile"
                } else {
                    "SAML"
                };
                let Some(spec) = resolution else {
                    p.add(ItemKind::Source, &id, Classification::Manual,
                        format!("Authentik {family} sources are not translated automatically"),
                        format!("Supply a reviewed {adapter} SourceSpec in source_resolutions and explicit source_links"))
                        .block(format!("Source {id}: provide an explicitly reviewed {adapter} source resolution"));
                    continue;
                };
                let supplied = source_adapter(&spec.source);
                if (family == "saml") != (supplied == "SAML") {
                    p.add(ItemKind::Source, &id, Classification::Manual,
                        format!("The resolution uses the {supplied} adapter, which cannot replace an Authentik {family} source; it was not applied"),
                        format!("Supply a reviewed {adapter} SourceSpec instead"))
                        .block(format!("Source {id}: the source resolution must use the {adapter} adapter"));
                    continue;
                }
                spec.source.validate()?;
                manifest.sources.push(spec.clone());
                let matching = source["user_matching_mode"]
                    .as_str()
                    .unwrap_or("unspecified");
                p.add(ItemKind::Source, &id, Classification::Manual,
                    format!("Exported {family} source is replaced by reviewed riAuth {supplied} source {}; its Authentik flows, policies and user matching mode ({matching}) are not translated", spec.source.id),
                    "Link upstream subjects explicitly in source_links and rehearse sign-in; riAuth never links accounts by email or username");
            }
            _ => {
                if family == "ldap" {
                    p.add(ItemKind::Source, &id, Classification::Unsupported,
                        "Authentik LDAP sources are not accepted by this importer; riAuth reads LDAP through its own directory configuration",
                        "Configure a riAuth LDAP directory (docs/ldap.md) and retire this source before the final export")
                        .block(format!("Source {id}: LDAP sources are not imported; configure a riAuth LDAP directory instead"));
                } else {
                    p.add(
                        ItemKind::Source,
                        &id,
                        Classification::Unsupported,
                        format!("riAuth has no adapter for Authentik {family} sources"),
                        "Replace this sign-in method or retire the source before the final export",
                    )
                    .block(format!(
                        "Source {id}: {family} sources have no riAuth adapter"
                    ));
                }
                if resolution.is_some() {
                    p.add(ItemKind::Source, &id, Classification::Unsupported,
                        format!("A source resolution cannot replace an unsupported {family} source; it was not applied"),
                        "Remove this source_resolutions entry and retire the source before the final export")
                        .block(format!("Source {id}: a source resolution cannot replace an unsupported {family} source"));
                }
            }
        }
    }
    for id in input.source_resolutions.keys() {
        if !exported_sources.contains(id) {
            p.add(
                ItemKind::Source,
                id,
                Classification::Manual,
                "The resolution names a source that is not in the export; it was not applied",
                "Remove the stale entry or export the sources again",
            )
            .block(format!(
                "Source {id}: source resolution does not match an exported source"
            ));
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
        if direct.is_empty() {
            p.add(
                ItemKind::Group,
                &name,
                Classification::Exact,
                "Group name and direct memberships are copied",
                "None",
            );
        } else {
            p.add(ItemKind::Group, &name, Classification::Convertible,
                "Parent groups are flattened into explicit memberships; later hierarchy changes do not propagate",
                "Review flattened memberships and group claims");
            parents.insert(id, direct);
        }
        if group["attributes"]
            .as_object()
            .is_some_and(|a| !a.is_empty())
        {
            p.add(
                ItemKind::Group,
                &name,
                Classification::Manual,
                "riAuth groups have no attributes; exported group attributes are not copied",
                "Move required values into user attributes or reviewed claim settings",
            );
        }
        if group["is_superuser"] == true || group["roles"].as_array().is_some_and(|r| !r.is_empty())
        {
            p.add(ItemKind::Group, &name, Classification::Manual,
                "Authentik superuser status and roles granted through this group are not promoted",
                "Grant riAuth administration or agent permissions explicitly if members still need them");
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
    let mut exported_subjects = BTreeMap::<String, Vec<String>>::new();
    let mut resolved_bindings = BTreeSet::new();
    for provider in providers {
        let cid = field(provider, "client_id")?.to_owned();
        // Without a reviewed resolution the export is still classified item by item against an
        // empty review, so every review-dependent item blocks; no client is converted from it.
        let reviewed = input.clients.get(&cid);
        let unreviewed = ClientResolution::default();
        let resolution = reviewed.unwrap_or(&unreviewed);
        if reviewed.is_some() {
            validate_server_url(&resolution.issuer)
                .map_err(|_| Error::bad("Invalid reviewed provider issuer"))?;
            p.add(ItemKind::Provider, &cid, Classification::Manual,
                if resolution.issuer == input.issuer {
                    "Client uses the target issuer named in the reviewed resolution".to_owned()
                } else {
                    format!("Client keeps the reviewed per-provider issuer {}", resolution.issuer)
                },
                "Verify every relying party's discovery and token validation against this issuer before cutover");
        } else {
            p.add(ItemKind::Provider, &cid, Classification::Manual,
                "Exported OAuth2/OIDC provider has no reviewed client resolution",
                "Add a clients entry with the discovery issuer, scopes, translated mappings, policies and authentication review")
                .block(format!("{cid}: provide reviewed issuer, scopes, mappings, policies and authentication requirements in clients"));
        }
        let flow = p.add(ItemKind::AuthenticationFlow, &cid, Classification::Manual,
            "Authentik authentication, authorization and consent flows are not executed or converted",
            "Review browser and terminal login, consent and MFA; set require_mfa and scope rules, then authentication_flow_reviewed");
        if !resolution.authentication_flow_reviewed {
            flow.block(format!(
                "{cid}: authentication flow has not been reviewed for browser/terminal login and MFA"
            ));
        }
        let exported_mappings = ids(&provider["property_mappings"])?;
        let mapping_blocker = format!(
            "{cid}: every exported property mapping must have a reviewed declarative translation"
        );
        for mapping in exported_mappings.union(&resolution.translated_mapping_ids) {
            let id = format!("{cid}/{mapping}");
            if !exported_mappings.contains(mapping) {
                p.add(
                    ItemKind::PropertyMapping,
                    id,
                    Classification::Manual,
                    "The resolution lists a translated mapping that is not in the export",
                    "Remove the stale ID or export the provider again",
                )
                .block(&mapping_blocker);
                continue;
            }
            let item = p.add(ItemKind::PropertyMapping, id, Classification::Manual,
                "Mapping expressions are never executed or assumed equivalent",
                "Translate the mapping into reviewed declarative claim settings and list its ID in translated_mapping_ids");
            if !resolution.translated_mapping_ids.contains(mapping) {
                item.block(&mapping_blocker);
            }
        }
        resolved_bindings.extend(resolution.translated_binding_ids.clone());
        for name in ["jwt_federation_sources", "jwt_federation_providers"] {
            if provider[name].as_array().is_some_and(|v| !v.is_empty()) {
                let item = p.add(ItemKind::Federation, format!("{cid}/{name}"), Classification::Manual,
                    "Exported JWT federation trust is not converted automatically",
                    "Translate it into pinned machine_trust, exchange and token_managers settings, then set federation_reviewed");
                if !resolution.federation_reviewed {
                    item.block(format!("{cid}: review {name} and translate its trust into pinned machine_trust, exchange and token_managers settings"));
                }
            }
        }
        if !provider["encryption_key"].is_null() {
            let item = p.add(
                ItemKind::EncryptionKey,
                &cid,
                Classification::Manual,
                "Authentik encryption keys are not part of the API export",
                "Configure the reviewed recipient public key for both ID and access tokens",
            );
            if resolution.settings.id_token_encryption.is_none()
                || resolution.settings.access_token_encryption.is_none()
            {
                item.block(format!(
                    "{cid}: configure the reviewed public encryption key for both ID and access tokens"
                ));
            }
        }
        p.add(ItemKind::SigningKey, &cid, Classification::Manual,
            if provider["signing_key"].is_null() {
                "Authentik signs this provider's tokens with the client secret when no signing key is set; riAuth signs with asymmetric keys"
            } else {
                "Authentik private signing keys are not part of the API export"
            },
            if provider["signing_key"].is_null() {
                "Confirm the relying party validates asymmetric ID tokens through riAuth's JWKS instead of the client secret"
            } else if resolution.settings.signing_key.is_some() {
                "Confirm the configured signing domain was imported with riauth keys import and keeps the expected kid"
            } else {
                "Import the key with riauth keys import and set settings.signing_key to keep its kid, or have the relying party accept riAuth's JWKS"
            });
        let mut settings = resolution.settings.clone();
        let provider_id = identifier(&provider["pk"])?;
        let associated = applications
            .iter()
            .filter(|a| identifier(&a["provider"]).ok().as_ref() == Some(&provider_id))
            .collect::<Vec<_>>();
        if associated.len() > 1 {
            for shared in &associated {
                p.add(ItemKind::Application, shared["slug"].as_str().unwrap_or(&cid), Classification::Manual,
                    format!("{} applications share provider {cid}", associated.len()),
                    "Split the applications or choose one application's launch metadata and policies explicitly")
                    .block(format!("{cid}: multiple applications share this provider; resolve their separate launch metadata and policies explicitly"));
            }
        }
        let application = associated.first().copied();
        let slug = application.and_then(|a| a["slug"].as_str()).unwrap_or(&cid);
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
                    || application["meta_hide"] == true
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
            let app = p.add(ItemKind::Application, slug, Classification::Convertible,
                "Name, launch URL, description, category and dashboard visibility become portal settings; other application settings are not copied",
                if reviewed.is_some() {
                    "Review the portal entry".to_owned()
                } else {
                    format!("Add a reviewed clients entry for provider {cid}, then review the portal entry")
                });
            // Nothing is converted for an unreviewed provider, so its application waits on that review.
            if reviewed.is_none() {
                app.block(format!("{cid}: provide reviewed issuer, scopes, mappings, policies and authentication requirements in clients"));
            }
        } else if application.is_some() {
            p.add(ItemKind::Application, slug, Classification::Manual,
                "Portal settings come from the reviewed resolution instead of the exported application metadata",
                "Review the portal entry");
        }
        settings.issuer = (resolution.issuer != input.issuer).then(|| resolution.issuer.clone());
        // Authentik exports `grant_types` from 2026.5; older exports have no grant inventory.
        let legacy_grants = provider["grant_types"].is_null();
        let exported_grants = if legacy_grants {
            BTreeSet::new()
        } else {
            ids(&provider["grant_types"])?
        };
        if settings.allowed_grants.is_empty() {
            settings.allowed_grants = exported_grants.clone();
        }
        let grant = if exported_grants.is_empty() {
            p.add(
                ItemKind::Grant,
                &cid,
                Classification::Manual,
                if legacy_grants {
                    "The export has no grant_types field (Authentik before 2026.5)"
                } else {
                    "The export lists no grant types"
                },
                "Inventory the grants this relying party uses and set settings.allowed_grants",
            )
        } else if settings.allowed_grants == exported_grants {
            p.add(
                ItemKind::Grant,
                &cid,
                Classification::Exact,
                "Exported grant types are carried over",
                "None",
            )
        } else {
            p.add(
                ItemKind::Grant,
                &cid,
                Classification::Manual,
                "The reviewed resolution selects the allowed grants",
                "Confirm the relying party needs only the selected grants",
            )
        };
        if !exported_grants.is_empty() && !settings.allowed_grants.is_subset(&exported_grants) {
            grant.block(format!(
                "{cid}: selected grants were not present in the export"
            ));
        }
        if settings.allowed_grants.is_empty() {
            grant.block(format!("{cid}: explicitly inventory allowed grants; an empty legacy grant list is ambiguous"));
        }
        settings.code_ttl = duration(&provider["access_code_validity"])?;
        settings.access_token_ttl = duration(&provider["access_token_validity"])?;
        settings.refresh_token_ttl = duration(&provider["refresh_token_validity"])?;
        settings.userinfo_only = provider["include_claims_in_id_token"] == false;
        p.add(ItemKind::TokenLifetime, &cid, Classification::Convertible,
            "Code, access and refresh token lifetimes are converted to seconds; absent values use riAuth defaults, and ID-token claim placement is copied",
            "None");
        let mut redirects = Vec::new();
        for redirect in provider["redirect_uris"]
            .as_array()
            .ok_or_else(|| Error::bad("Missing redirect_uris"))?
        {
            if redirect["matching_mode"] != "strict" {
                p.add(
                    ItemKind::RedirectUri,
                    format!("{cid}/{}", redirect["url"].as_str().unwrap_or("")),
                    Classification::Unsupported,
                    "Regex redirect matching is not supported",
                    "Register each exact redirect URI in Authentik and export again",
                )
                .block(format!(
                    "{cid}: regex redirect needs explicit exact registrations"
                ));
                continue;
            }
            let url = field(redirect, "url")?.to_owned();
            match redirect["redirect_uri_type"]
                .as_str()
                .unwrap_or("authorization")
            {
                "authorization" => {
                    p.add(
                        ItemKind::RedirectUri,
                        format!("{cid}/{url}"),
                        Classification::Exact,
                        "Strict redirect URI is registered unchanged",
                        "None",
                    );
                    redirects.push(url);
                }
                "logout" => {
                    p.add(
                        ItemKind::RedirectUri,
                        format!("{cid}/{url}"),
                        Classification::Convertible,
                        "Strict logout redirect becomes a post-logout redirect URI",
                        "None",
                    );
                    if !settings.post_logout_redirect_uris.contains(&url) {
                        settings.post_logout_redirect_uris.push(url);
                    }
                }
                other => {
                    p.add(
                        ItemKind::RedirectUri,
                        format!("{cid}/{url}"),
                        Classification::Unsupported,
                        format!("Redirect type {other} is not supported"),
                        "Remove it or replace it with an exact authorization or logout redirect",
                    )
                    .block(format!("{cid}: unsupported redirect type"));
                }
            }
        }
        if let Some(uri) = provider["logout_uri"].as_str().filter(|s| !s.is_empty()) {
            let method = provider["logout_method"].as_str().unwrap_or("");
            if method == "backchannel" || method == "frontchannel" {
                if method == "backchannel" {
                    settings.backchannel_logout_uri = Some(uri.to_owned());
                } else {
                    settings.frontchannel_logout_uri = Some(uri.to_owned());
                }
                p.add(
                    ItemKind::Logout,
                    &cid,
                    Classification::Exact,
                    format!("The {method} logout URI is carried over"),
                    "Rehearse logout with the relying party",
                );
            } else {
                p.add(
                    ItemKind::Logout,
                    &cid,
                    Classification::Unsupported,
                    "The logout method is unknown",
                    "Choose back-channel or front-channel logout for this client",
                )
                .block(format!("{cid}: unknown logout method"));
            }
        }
        let confidential = match field(provider, "client_type")? {
            "confidential" => true,
            "public" => false,
            _ => return Err(Error::bad("Unknown client type")),
        };
        if confidential {
            let item = p.add(ItemKind::ClientSecret, &cid, Classification::Manual,
                "Exported client secrets are never copied into plans; a reviewed reference is resolved at apply",
                "Reference the existing secret or coordinate a new secret with the relying party");
            if resolution.secret_ref.is_none() || resolution.secret_version.is_none() {
                item.block(format!("{cid}: confidential client requires a secret reference/version; exported secrets are never copied into plans"));
            }
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
        if reviewed.is_some()
            && let Err(error) = crate::provider::validate_settings(&client)
                .and_then(|_| claims::validate_mappings(&client))
        {
            p.add(
                ItemKind::Provider,
                &cid,
                Classification::Manual,
                "The reviewed client settings fail riAuth validation",
                "Correct the settings in this clients entry",
            )
            .block(format!("{cid}: {}", error.message));
        }
        let mode = field(provider, "sub_mode")?.to_owned();
        if let Some(source) = subject_source(&mode) {
            p.add(
                ItemKind::Subject,
                &cid,
                Classification::Exact,
                format!("Subjects are copied from the exported {source}"),
                "Compare sample subjects with the relying party's stored account IDs",
            );
        } else {
            p.add(ItemKind::Subject, &cid, Classification::Unsupported,
                format!("Subject mode {mode} is not supported"),
                "Choose a supported subject mode and export again, or plan an explicit relying-party account migration")
                .block(format!("{cid}: unsupported subject mode {mode}"));
        }
        subject_modes.insert(cid.clone(), (mode, reviewed.is_some()));
        if reviewed.is_none() {
            continue;
        }
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
    for cid in input.clients.keys() {
        if !subject_modes.contains_key(cid) {
            p.add(
                ItemKind::Provider,
                cid,
                Classification::Manual,
                "The clients entry names a provider that is not in the export; it was not applied",
                "Remove the stale entry or export the providers again",
            )
            .block(format!(
                "{cid}: clients entry does not match an exported provider"
            ));
        }
    }
    let exported_bindings = bindings
        .iter()
        .map(|b| identifier(&b["pk"]))
        .collect::<Result<BTreeSet<_>>>()?;
    let binding_blocker = "Every exported policy binding must be inventoried and translated; unresolved or extra binding IDs remain";
    for binding in exported_bindings.union(&resolved_bindings) {
        if !exported_bindings.contains(binding) {
            p.add(
                ItemKind::PolicyBinding,
                binding,
                Classification::Manual,
                "A resolution lists a translated binding that is not in the export",
                "Remove the stale ID or export the bindings again",
            )
            .block(binding_blocker);
            continue;
        }
        let item = p.add(ItemKind::PolicyBinding, binding, Classification::Manual,
            "Policies and their bindings are never executed or assumed equivalent",
            "Translate the binding into reviewed client policy settings and list its ID in translated_binding_ids");
        if !resolved_bindings.contains(binding) {
            item.block(binding_blocker);
        }
    }
    let provider_ids = providers
        .iter()
        .map(|p| identifier(&p["pk"]))
        .collect::<Result<BTreeSet<_>>>()?;
    for application in applications {
        if application["provider"].is_null() {
            p.add(
                ItemKind::Application,
                application["slug"].as_str().unwrap_or_default(),
                Classification::Unsupported,
                "The application has no provider, so there is no sign-in to convert",
                "Recreate its launch link manually if users still need it",
            );
        } else if !provider_ids.contains(&identifier(&application["provider"])?) {
            let slug = field(application, "slug")?;
            p.add(ItemKind::Application, slug, Classification::Unsupported,
                "The application's provider is not an exported OAuth2/OIDC provider, so it cannot be converted",
                "Recreate it with the matching riAuth integration, such as a proxy client (docs/proxy.md), and rehearse it separately")
                .block(format!("Application {slug} uses an unexported or unsupported provider"));
        }
        if application["backchannel_providers"]
            .as_array()
            .is_some_and(|v| !v.is_empty())
        {
            let slug = field(application, "slug")?;
            p.add(ItemKind::Application, slug, Classification::Unsupported,
                "Backchannel provisioning providers are not converted",
                "Configure riAuth provisioning (for example outbound SCIM) separately and rehearse it")
                .block(format!("Application {slug} has provisioning providers requiring separate migration"));
        }
    }
    for user in users {
        let username = field(user, "username")?.to_owned();
        let password = input.passwords.get(&username);
        let external =
            user["type"] == "external" && input.source_links.iter().any(|l| l.username == username);
        p.add(ItemKind::User, &username, Classification::Convertible,
            "The local ID becomes authentik-<pk>; name, email, attributes, active state and flattened memberships are copied; email_verified starts false",
            "Review profile data and email verification");
        match password {
            Some(reference) if reference.hashed => {
                p.add(ItemKind::Password, &username, Classification::Convertible,
                    "The referenced hash is checked at apply: supported Django pbkdf2_sha256 and Argon2 hashes are kept and rehashed after sign-in where needed; other or invalid hashes fail the apply",
                    "Rehearse this user's sign-in; the converter never reads the hash");
            }
            Some(_) => {
                p.add(ItemKind::Password, &username, Classification::Manual,
                    "A newly established password is referenced; the Authentik password is not carried over",
                    "Deliver the new password through the managed reset process");
            }
            None if external => {
                p.add(ItemKind::Password, &username, Classification::Manual,
                    "The local password is disabled; the user signs in through the explicitly linked source",
                    "Rehearse sign-in through the linked source");
            }
            None => {
                p.add(ItemKind::Password, &username, Classification::Manual,
                    "No password or password-hash reference was supplied",
                    "Supply a hash reference from an authorized offline export, or complete a managed password reset")
                    .block(format!("{username}: supply a password/password-hash reference or complete password reset before migration"));
            }
        }
        if let Some(kind) = user["type"].as_str().filter(|v| *v != "internal")
            && !external
        {
            p.add(ItemKind::User, &username, Classification::Manual,
                format!("Authentik {kind} accounts are not converted as ordinary local users"),
                "Link the user to a reviewed source in source_links, or recreate it as a riAuth service client or agent")
                .block(format!("{username}: external/service identity requires explicit source or service-account migration"));
        }
        if user["roles"].as_array().is_some_and(|v| !v.is_empty()) {
            p.add(ItemKind::User, &username, Classification::Manual,
                "Authentik roles are not converted into riAuth permissions",
                "Grant equivalent riAuth administration or agent permissions explicitly, then remove the roles from the export")
                .block(format!("{username}: Authentik administrative roles require explicit agent permission migration"));
        }
        if user["is_superuser"] == true {
            p.add(ItemKind::User, &username, Classification::Manual,
                "Authentik superuser status is not promoted; the riAuth bootstrap administrator remains",
                "Grant riAuth administration explicitly if this user still needs it");
        }
        if input.totp.contains_key(&username) {
            p.add(ItemKind::Totp, &username, Classification::Convertible,
                "The referenced TOTP secret is imported at apply and the current time step is marked spent",
                "Tell the user to wait for a fresh code; the converter never reads the secret");
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
        for (cid, (mode, reviewed)) in &subject_modes {
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
                // Reported once per client by the provider loop.
                _ => continue,
            };
            if subject.is_empty()
                || subject.len() > 255
                || !subject.is_ascii()
                || subject.chars().any(char::is_control)
            {
                p.add(ItemKind::Subject, format!("{username}/{cid}"), Classification::Unsupported,
                    "The exported subject is empty, longer than 255 bytes, non-ASCII or contains control characters",
                    "Choose a subject mode with a supported value, or plan an explicit relying-party account migration")
                    .block(format!(
                        "{username}/{cid}: subject is outside the supported OIDC format"
                    ));
            }
            exported_subjects
                .entry(cid.clone())
                .or_default()
                .push(subject.clone());
            if *reviewed {
                subjects.insert(cid.clone(), subject);
            }
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
    // Checked for every exported provider, reviewed or not.
    for (cid, subjects) in &exported_subjects {
        let mut seen = BTreeSet::new();
        if subjects.iter().any(|subject| !seen.insert(subject)) {
            p.add(
                ItemKind::Subject,
                cid,
                Classification::Unsupported,
                "Two exported users would receive the same subject",
                "Resolve the duplicate in Authentik; riAuth never merges identities",
            )
            .block(format!("{cid}: duplicate subjects would merge identities"));
        }
    }
    p.add(ItemKind::Passkey, "*", Classification::Unsupported,
        "WebAuthn credentials are bound to Authentik's hostname and are not exported",
        "Users add passkeys in the riAuth portal after sign-in; plan this before moving require_mfa applications");
    p.add(
        ItemKind::Session,
        "*",
        Classification::Unsupported,
        "Live sessions, cookies and opaque access or refresh tokens are not imported",
        "Users sign in again; plan relying-party sessions and offline token validators",
    );
    p.add(
        ItemKind::Totp,
        "*",
        Classification::Manual,
        "TOTP devices are not in the API export; only users listed in totp keep their factor",
        "Supply TOTP references from an authorized offline export, or have users enroll again",
    );
    if let Err(error) = manifest.validate() {
        p.add(
            ItemKind::Manifest,
            "manifest",
            Classification::Manual,
            "The converted manifest fails riAuth validation",
            "Correct the input that produced this error",
        )
        .block(error.message);
    }
    let (items, blockers, summary) = p.finish();
    Ok(
        json!({"api_version": "riauth.migration-report/v1", "ready_for_plan": blockers.is_empty(), "issuer": input.issuer, "blockers": blockers,
        "summary": summary, "items": items,
        "manifest": if blockers.is_empty() { json!(manifest) } else { Value::Null }, "draft": manifest,
        "reauthentication_required": true, "old_tokens_and_sessions_imported": false, "signing_keys_imported": false, "administrators_imported": false,
        "source_fingerprint": crypto::digest(&serde_json::to_string(&input).map_err(Error::internal)?)}),
    )
}

pub const AUTHENTIK_FORMAT: &str = "riauth.authentik-import/v1";
pub const INVENTORY_FORMAT: &str = "riauth.migration-inventory/v1";

/// Declared inventory of a source system that has no export converter. It carries no
/// configuration or credential values and can never produce a manifest.
#[derive(schemars::JsonSchema, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    pub api_version: String,
    /// Lowercase source system name, such as `keycloak`, `okta`, `active-directory` or `entra-id`.
    pub system: String,
    /// Every configuration element to account for. `id` is the source identifier, or `*` for all
    /// elements of that kind.
    pub elements: Vec<InventoryElement>,
}
#[derive(schemars::JsonSchema, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryElement {
    /// riAuth finding kind. Set exactly one of `kind` and `source_kind`.
    #[serde(default)]
    pub kind: Option<ItemKind>,
    /// The source system's own element type when no riAuth kind fits, such as a Keycloak `realm`
    /// or `role`: 1-64 lowercase letters, digits, hyphens or underscores, starting with a letter.
    #[serde(default)]
    pub source_kind: Option<String>,
    pub id: String,
}

/// riAuth adapter that can take over part of a declared source system.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Route {
    /// Authentik has an export converter; an inventory is the wrong input.
    Authentik,
    /// A configured riAuth LDAP directory (docs/ldap.md).
    Ldap,
    /// `riauth directory workspace` or `riauth directory entra`, with its guide.
    Cloud(&'static str, &'static str),
    None,
}
fn route(system: &str) -> Route {
    match system {
        "authentik" => Route::Authentik,
        "ldap" | "openldap" | "active-directory" => Route::Ldap,
        "google-workspace" => Route::Cloud("workspace", "docs/enterprise/ENT-03.md"),
        "entra-id" => Route::Cloud("entra", "docs/enterprise/ENT-04.md"),
        _ => Route::None,
    }
}
fn kind_name(kind: ItemKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}
/// Remediation for an element that no riAuth adapter takes over.
fn rebuild_action(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::User => {
            "Recreate the account through a reviewed riauth/v1 manifest or a riAuth directory; no local ID, subject or credential carries over"
        }
        ItemKind::Password => {
            "Have the user set a new password through a managed reset; this system's hashes are not converted"
        }
        ItemKind::Totp => "Have the user enroll a new TOTP factor in riAuth",
        ItemKind::Passkey => "Have the user add a passkey in the riAuth portal after signing in",
        ItemKind::Session => {
            "Users sign in again; plan relying-party sessions and offline token validators"
        }
        ItemKind::Subject => {
            "Record each relying party's subject contract; riAuth never derives another issuer's subjects, and a changed subject needs an explicit relying-party account migration"
        }
        ItemKind::Group => {
            "Recreate the group and its memberships in a reviewed manifest or through a riAuth directory"
        }
        ItemKind::Source => {
            "Configure a reviewed riAuth source (riauth --json schema source) and explicit source links, or retire the sign-in method"
        }
        ItemKind::AuthenticationFlow | ItemKind::PropertyMapping | ItemKind::PolicyBinding => {
            "Translate it by hand into reviewed declarative riAuth settings; scripts and expressions are never executed"
        }
        ItemKind::Federation => {
            "Translate the trust into pinned machine_trust, exchange and token_managers settings"
        }
        ItemKind::SigningKey => {
            "Import a reviewed key with riauth keys import, or have relying parties accept riAuth's JWKS"
        }
        ItemKind::EncryptionKey => {
            "Configure the relying party's reviewed public encryption key in riAuth"
        }
        ItemKind::ClientSecret => {
            "Reference the existing secret privately in a reviewed client, or coordinate a new secret with the relying party"
        }
        _ => {
            "Recreate it as a reviewed riAuth client in a riauth/v1 manifest and rehearse it with its relying party"
        }
    }
}
/// Resolve an element to its finding kind and, for a source-native element, its validated type.
fn element_kind(element: &InventoryElement) -> Result<(ItemKind, Option<&str>)> {
    match (element.kind, element.source_kind.as_deref()) {
        (Some(ItemKind::Manifest), None) => {
            Err(Error::bad("An inventory cannot declare a manifest element"))
        }
        (Some(ItemKind::SourceNative), None) => Err(Error::bad(
            "Declare a source-native element with source_kind instead of kind",
        )),
        (Some(kind), None) => Ok((kind, None)),
        (None, Some(native)) => {
            let bytes = native.as_bytes();
            if bytes.is_empty()
                || bytes.len() > 64
                || !bytes[0].is_ascii_lowercase()
                || !bytes.iter().all(|&b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_'
                })
            {
                return Err(Error::bad(
                    "source_kind must be 1-64 lowercase letters, digits, hyphens or underscores, starting with a letter",
                ));
            }
            // A type riAuth already names must use `kind`, so directory routes are never bypassed.
            if serde_json::from_value::<ItemKind>(json!(native.replace('-', "_"))).is_ok() {
                return Err(Error::bad(
                    "A source_kind that matches a riAuth finding kind must be declared with kind",
                ));
            }
            Ok((ItemKind::SourceNative, Some(native)))
        }
        _ => Err(Error::bad(
            "Each inventory element needs exactly one of kind or source_kind",
        )),
    }
}
fn classify_element(
    p: &mut Preflight,
    system: &str,
    route: Route,
    kind: ItemKind,
    source_kind: Option<&str>,
    id: &str,
) {
    let label = format!(
        "{} {id}",
        source_kind.map_or_else(|| kind_name(kind), String::from)
    );
    if let (Some(native), false) = (source_kind, route == Route::Authentik) {
        let finding = p.add(kind, id, Classification::Unsupported,
            format!("{system} {native} elements have no riAuth equivalent kind and no converter"),
            "Rebuild what it provides as reviewed riAuth configuration and rehearse it, or retire it before cutover");
        finding.source_kind = Some(native.to_owned());
        finding.block(format!(
            "{system} {label}: no riAuth converter; rebuild or retire it"
        ));
        return;
    }
    let unsupported = |p: &mut Preflight| {
        let reason = match route {
            Route::Ldap => format!(
                "riAuth's LDAP directory imports only users, group memberships and password checks, and no converter reads {system} exports for anything else"
            ),
            Route::Cloud(command, _) => format!(
                "riauth directory {command} imports only users and mapped group memberships, and no converter reads {system} exports for anything else"
            ),
            _ => format!(
                "No riAuth converter reads {system} exports, so this element is not carried over"
            ),
        };
        p.add(
            kind,
            id,
            Classification::Unsupported,
            reason,
            rebuild_action(kind),
        )
        .block(format!(
            "{system} {label}: no riAuth converter; rebuild or retire it"
        ));
    };
    match (route, kind) {
        (Route::Authentik, _) => {
            p.add(kind, id, Classification::Manual,
                "Authentik configuration is classified from its API export bundle, not from an inventory",
                format!("Build a {AUTHENTIK_FORMAT} bundle (docs/migration.md) and run the preflight on it"))
                .block(format!("{system} {label}: use the {AUTHENTIK_FORMAT} export bundle"));
            p.items.last_mut().unwrap().source_kind = source_kind.map(String::from);
        }
        (Route::Ldap, ItemKind::User) => {
            p.add(kind, id, Classification::Manual,
                "Directory entries are imported by a configured riAuth LDAP directory, not by this file; its id_attribute becomes the stable identity",
                "Configure [directories.<name>] in riauth.toml (docs/ldap.md, including its size limits) and review riauth directory plan; accounts are never adopted by DN or email")
                .block(format!("{system} {label}: import through a riAuth LDAP directory and review its plan"));
        }
        (Route::Ldap, ItemKind::Group) => {
            p.add(kind, id, Classification::Manual,
                "Memberships are imported into existing riAuth groups through per-group user filters; nested groups need explicit filters",
                "Create the riAuth group, set its group_user_filters entry and review the directory plan")
                .block(format!("{system} {label}: map the group through a riAuth LDAP directory and review its plan"));
        }
        (Route::Ldap, ItemKind::Password) => {
            p.add(kind, id, Classification::Manual,
                "Directory passwords are not copied; riAuth checks them with an LDAP bind at each login",
                "Keep the directory reachable from every riAuth node; an outage never falls back to a local password")
                .block(format!("{system} {label}: keep password checks on the directory through a riAuth LDAP directory"));
        }
        (Route::Cloud(command, guide), ItemKind::User | ItemKind::Group) => {
            p.add(kind, id, Classification::Manual,
                format!("{system} users and the memberships of mapped groups are imported by riauth directory {command}, not by this file; the upstream object ID becomes the stable link"),
                format!("Configure the directory in riauth.toml ({guide}) and review riauth directory {command} plan after creating the local groups; accounts are never adopted by email or username"))
                .block(format!("{system} {label}: import through riauth directory {command} and review its plan"));
        }
        (Route::Cloud(..), ItemKind::Password) => {
            p.add(kind, id, Classification::Unsupported,
                format!("{system} passwords are neither exported nor checked by riAuth; its directory import does not enable password sign-in"),
                "Configure explicit source links to a reviewed upstream source, or have users set a riAuth password")
                .block(format!("{system} {label}: choose a riAuth sign-in method for imported accounts"));
        }
        _ => unsupported(p),
    }
}
/// Classify a declared inventory. Every element blocks, and the report never contains a manifest.
pub fn inventory(input: Inventory) -> Result<Value> {
    if input.api_version != INVENTORY_FORMAT {
        return Err(Error::bad("Unsupported migration inventory format"));
    }
    let system = input.system.as_str();
    if system.is_empty()
        || system.len() > 64
        || !system
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(Error::bad(
            "Inventory system must be 1-64 lowercase letters, digits or hyphens",
        ));
    }
    if input.elements.is_empty() {
        return Err(Error::bad("Declare at least one inventory element"));
    }
    let route = route(system);
    let mut p = Preflight::default();
    let mut declared = BTreeSet::new();
    for element in &input.elements {
        let (kind, source_kind) = element_kind(element)?;
        if element.id.is_empty()
            || element.id.len() > 256
            || element.id.chars().any(char::is_control)
        {
            return Err(Error::bad(
                "Inventory element IDs must be 1-256 characters without control characters",
            ));
        }
        if !declared.insert((kind, source_kind, element.id.as_str())) {
            return Err(Error::bad("Duplicate inventory element"));
        }
        classify_element(&mut p, system, route, kind, source_kind, &element.id);
    }
    let action = match route {
        Route::Authentik => format!("Preflight a {AUTHENTIK_FORMAT} bundle instead"),
        Route::Ldap => "Configure a riAuth LDAP directory for the listed users, groups and password checks; its reviewed plan replaces a manifest".into(),
        Route::Cloud(command, _) => format!("Configure riauth directory {command} for the listed users and groups; its reviewed plan replaces a manifest"),
        Route::None => "Rebuild the listed configuration as reviewed riAuth configuration and rehearse every application; retire what is not rebuilt".into(),
    };
    p.add(
        ItemKind::Manifest,
        "manifest",
        Classification::Unsupported,
        format!("{INVENTORY_FORMAT} only declares what {system} contains; no converter reads its exports"),
        action,
    )
    .block(format!("{system}: an inventory cannot produce an applicable manifest"));
    let (items, blockers, summary) = p.finish();
    Ok(
        json!({"api_version": "riauth.migration-report/v1", "ready_for_plan": false,
        "source": {"system": system, "format": INVENTORY_FORMAT, "converter": Value::Null},
        "blockers": blockers, "summary": summary, "items": items, "manifest": Value::Null}),
    )
}
/// Parse a typed migration input. Deserialization errors can quote input values, such as a
/// misplaced secret, so only the location is reported.
fn parse<T: serde::de::DeserializeOwned>(input: &[u8], what: &str, schema: &str) -> Result<T> {
    serde_json::from_slice(input).map_err(|error| {
        Error::bad(format!(
            "Invalid {what} at line {} column {}; check it against riauth --json schema {schema}",
            error.line(),
            error.column()
        ))
    })
}
/// Source-aware preflight: dispatch on `api_version` and return the classified findings only.
/// It never returns a manifest or draft; `import-authentik --out` is the only manifest writer.
pub fn preflight(input: &[u8]) -> Result<Value> {
    let value: Value = serde_json::from_slice(input).map_err(|error| {
        Error::bad(format!(
            "Migration input is not JSON at line {} column {}",
            error.line(),
            error.column()
        ))
    })?;
    let report = match value["api_version"].as_str() {
        Some(AUTHENTIK_FORMAT) => {
            let mut report = convert(parse(input, "Authentik import bundle", "authentik-import")?)?;
            report["source"] = json!({"system": "authentik", "format": AUTHENTIK_FORMAT, "converter": "authentik"});
            report
        }
        Some(INVENTORY_FORMAT) => {
            inventory(parse(input, "migration inventory", "migration-inventory")?)?
        }
        _ => {
            return Err(Error::bad(format!(
                "Unsupported migration input; api_version must be {AUTHENTIK_FORMAT} or {INVENTORY_FORMAT}"
            )));
        }
    };
    Ok(
        json!({"api_version": report["api_version"], "source": report["source"],
        "ready_for_plan": report["ready_for_plan"], "summary": report["summary"],
        "blockers": report["blockers"], "items": report["items"]}),
    )
}
