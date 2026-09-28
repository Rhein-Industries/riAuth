pub use crate::model::claims::{ClaimMapping, ClaimSource, Policy, Rule};
use crate::{
    error::{Error, Result},
    model::{Client, Identity, User},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Read-only facts needed by claim policy and subject validation.
pub trait ClaimsTx {
    /// Keep the ordinary subject scan aligned with storage's maintenance page.
    const SUBJECT_SCAN_PAGE: usize;

    fn groups_for(&self, user_id: &str) -> Result<BTreeSet<String>>;
    fn client(&self, client_id: &str) -> Result<Option<Client>>;
    fn scan_users(&self, after: Option<&str>, limit: usize) -> Result<Vec<(String, User)>>;
}

pub fn rule_reasons(
    rule: &Rule,
    username: &str,
    groups: &BTreeSet<String>,
    mfa: bool,
) -> Vec<&'static str> {
    let mut reasons = Vec::new();
    if !rule.all_groups.is_subset(groups) {
        reasons.push("required_groups_missing");
    }
    if !rule.any_groups.is_empty() && rule.any_groups.is_disjoint(groups) {
        reasons.push("no_matching_group");
    }
    if !rule.denied_groups.is_disjoint(groups) {
        reasons.push("denied_group");
    }
    if !rule.users.is_empty() && !rule.users.contains(username) {
        reasons.push("user_not_allowed");
    }
    if rule.denied_users.contains(username) {
        reasons.push("user_denied");
    }
    if rule.require_mfa && !mfa {
        reasons.push("mfa_required");
    }
    reasons
}

pub fn enforce(
    tx: &impl ClaimsTx,
    client: &Client,
    user: &User,
    identity: &Identity,
    scopes: &BTreeSet<String>,
) -> Result<()> {
    let groups = tx.groups_for(&user.id)?;
    for rule in std::iter::once(&client.settings.policy.access).chain(
        scopes
            .iter()
            .filter_map(|s| client.settings.policy.scopes.get(s)),
    ) {
        if !rule_reasons(rule, &user.username, &groups, identity.mfa).is_empty() {
            return Err(Error::forbidden());
        }
    }
    Ok(())
}

pub fn subject(user: &User, client: &Client) -> String {
    if let Some(subject) = user.subjects.get(&client.id) {
        return subject.clone();
    }
    if let Some(sector) = &client.settings.pairwise_sector {
        return crate::crypto::digest(&format!("{}\0{sector}\0{}", user.pairwise_seed, user.id));
    }
    user.id.clone()
}

pub fn validate_user<T: ClaimsTx>(tx: &T, user: &User) -> Result<()> {
    validate_user_paged(tx, user, T::SUBJECT_SCAN_PAGE, &|| Ok(()))
}

/// Validate a user while allowing offline restore to abort a long subject scan.
pub fn validate_user_checked(
    tx: &impl ClaimsTx,
    user: &User,
    check: &dyn Fn() -> Result<()>,
) -> Result<()> {
    validate_user_paged(tx, user, 1, check)
}

fn validate_user_paged(
    tx: &impl ClaimsTx,
    user: &User,
    page_size: usize,
    check: &dyn Fn() -> Result<()>,
) -> Result<()> {
    if serde_json::to_vec(&user.attributes)
        .map_err(Error::internal)?
        .len()
        > 16_384
    {
        return Err(Error::bad("User attributes exceed 16 KiB"));
    }
    if user.email_verified && user.email.is_none() {
        return Err(Error::bad("A verified email requires an address"));
    }
    for (cid, subject) in &user.subjects {
        if subject.is_empty()
            || subject.len() > 255
            || !subject.is_ascii()
            || subject.chars().any(char::is_control)
        {
            return Err(Error::bad(
                "Subjects must be 1–255 ASCII bytes without control characters",
            ));
        }
        let client = tx
            .client(cid)?
            .ok_or_else(|| Error::bad("Subject mapping references an unknown client"))?;
        let mut after = None;
        loop {
            // The checked restore path decodes just one imported user at a time;
            // an imported user can approach the archive's per-frame limit.
            check()?;
            let users = tx.scan_users(after.as_deref(), page_size)?;
            if users.is_empty() {
                break;
            }
            for (id, other) in users {
                if other.id != user.id
                    && (other.id == *subject || self::subject(&other, &client) == *subject)
                {
                    return Err(Error::conflict("Subject already belongs to another user"));
                }
                after = Some(id);
            }
        }
    }
    Ok(())
}

pub fn mapped_claims(
    tx: &impl ClaimsTx,
    user: &User,
    client: &Client,
    scopes: &BTreeSet<String>,
) -> Result<Value> {
    let mut claims = json!({"sub": subject(user, client)});
    if scopes.contains("profile") {
        claims["name"] = json!(user.display_name);
        claims["preferred_username"] = json!(user.username);
    }
    if scopes.contains("email")
        && let Some(email) = &user.email
    {
        claims["email"] = json!(email);
        claims["email_verified"] = json!(user.email_verified);
    }
    if scopes.contains("groups") || client.settings.groups_in_profile && scopes.contains("profile")
    {
        claims["groups"] = json!(tx.groups_for(&user.id)?);
    }
    for mapping in &client.settings.claim_mappings {
        if !scopes.contains(&mapping.scope) {
            continue;
        }
        let value = match &mapping.source {
            ClaimSource::Username => json!(user.username),
            ClaimSource::DisplayName => json!(user.display_name),
            ClaimSource::Email => json!(user.email),
            ClaimSource::EmailVerified => json!(user.email_verified),
            ClaimSource::Groups => json!(tx.groups_for(&user.id)?),
            ClaimSource::Attribute { key } => {
                user.attributes.get(key).cloned().unwrap_or(Value::Null)
            }
            ClaimSource::Literal { value } => value.clone(),
        };
        if !value.is_null() {
            claims[&mapping.claim] = value;
        }
    }
    Ok(claims)
}

pub fn validate_mappings(client: &Client) -> Result<()> {
    let mut names = BTreeSet::new();
    for m in &client.settings.claim_mappings {
        if !client.scopes.contains(&m.scope)
            || m.claim.is_empty()
            || m.claim.len() > 128
            || m.claim.chars().any(char::is_control)
            || [
                "iss",
                "sub",
                "aud",
                "exp",
                "iat",
                "nbf",
                "jti",
                "nonce",
                "auth_time",
                "amr",
                "acr",
                "at_hash",
                "c_hash",
                "sid",
                "client_id",
                "scope",
                "cnf",
                "act",
                "email",
                "email_verified",
            ]
            .contains(&m.claim.as_str())
            || !names.insert(&m.claim)
        {
            return Err(Error::bad("Invalid, duplicate or protected claim mapping"));
        }
    }
    if client.settings.claim_mappings.len() > 64
        || serde_json::to_vec(&client.settings)
            .map_err(Error::internal)?
            .len()
            > 16_384
    {
        return Err(Error::bad("Provider mappings exceed configured limits"));
    }
    if client
        .settings
        .policy
        .scopes
        .keys()
        .any(|s| !client.scopes.contains(s))
    {
        return Err(Error::bad("Scope policy references an unregistered scope"));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Explain {
    pub client_id: String,
    pub username: String,
    pub scope: BTreeSet<String>,
    #[serde(default)]
    pub mfa: bool,
}
/// Build the dry-run response after assembly has authorized and loaded its records.
pub(crate) fn explain_decision(
    tx: &impl ClaimsTx,
    input: &Explain,
    client: &Client,
    user: &User,
    device_reason: impl FnOnce() -> Result<Option<&'static str>>,
) -> Result<Value> {
    let groups = tx.groups_for(&user.id)?;
    let mut reasons = rule_reasons(
        &client.settings.policy.access,
        &user.username,
        &groups,
        input.mfa,
    );
    if !client.enabled {
        reasons.push("client_disabled");
    }
    if !user.enabled {
        reasons.push("user_disabled");
    }
    if client.service {
        reasons.push("service_client_has_no_user_identity");
    }
    if !input.scope.is_subset(&client.scopes) {
        reasons.push("unregistered_scope");
    }
    if !client.allowed_groups.is_empty() && client.allowed_groups.is_disjoint(&groups) {
        reasons.push("no_matching_client_group");
    }
    if client.require_mfa && !input.mfa {
        reasons.push("mfa_required");
    }
    if let Some(reason) = device_reason()? {
        reasons.push(reason);
    }
    let scopes: std::collections::BTreeMap<_, _> = input
        .scope
        .iter()
        .filter_map(|scope| {
            client.settings.policy.scopes.get(scope).map(|rule| {
                (
                    scope,
                    rule_reasons(rule, &user.username, &groups, input.mfa),
                )
            })
        })
        .collect();
    let allowed = reasons.is_empty() && scopes.values().all(Vec::is_empty);
    let mapped = mapped_claims(tx, user, client, &input.scope)?;
    Ok(json!({
        "simulation": true,
        "token_issued": false,
        "mfa_assumed": input.mfa,
        "allowed": allowed,
        "reasons": reasons,
        "scope_decisions": scopes,
        "userinfo": mapped,
        "id_token_identity_claims": if client.settings.userinfo_only { json!({"sub": subject(user, client)}) } else { mapped.clone() },
        "access_token_identity_claims": if client.settings.claims_in_access_token { mapped } else { json!({"sub": subject(user, client)}) }
    }))
}
