use crate::{
    core::groups_for,
    error::{Error, Result},
    model::{Client, Identity, User},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClaimMapping {
    pub scope: String,
    pub claim: String,
    pub source: ClaimSource,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClaimSource {
    Username,
    DisplayName,
    Email,
    EmailVerified,
    Groups,
    Attribute { key: String },
    Literal { value: Value },
}

#[derive(schemars::JsonSchema, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Rule {
    pub all_groups: BTreeSet<String>,
    pub any_groups: BTreeSet<String>,
    pub denied_groups: BTreeSet<String>,
    pub users: BTreeSet<String>,
    pub denied_users: BTreeSet<String>,
    pub require_mfa: bool,
}

#[derive(schemars::JsonSchema, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Policy {
    pub access: Rule,
    pub scopes: BTreeMap<String, Rule>,
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
    tx: &Tx<'_>,
    client: &Client,
    user: &User,
    identity: &Identity,
    scopes: &BTreeSet<String>,
) -> Result<()> {
    let groups = groups_for(tx, &user.id)?;
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

pub fn validate_user(tx: &Tx<'_>, user: &User) -> Result<()> {
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
            .get::<Client>("clients", cid)?
            .ok_or_else(|| Error::bad("Subject mapping references an unknown client"))?;
        if tx.list::<User>("users")?.iter().any(|(_, other)| {
            other.id != user.id
                && (other.id == *subject || self::subject(other, &client) == *subject)
        }) {
            return Err(Error::conflict("Subject already belongs to another user"));
        }
    }
    Ok(())
}

pub fn mapped_claims(
    tx: &Tx<'_>,
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
        claims["groups"] = json!(groups_for(tx, &user.id)?);
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
            ClaimSource::Groups => json!(groups_for(tx, &user.id)?),
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
impl crate::core::Core {
    pub fn explain(&self, token: &str, input: Explain) -> Result<Value> {
        self.store.read(|tx| {
            self.management(tx, token, "client.read", &format!("client/{}", input.client_id))?;
            self.management(tx, token, "user.read", &format!("user/{}", input.username))?;
            let client = tx.get::<Client>("clients", &input.client_id)?.ok_or_else(|| Error::missing("Client not found"))?;
            let user = crate::core::user_by_name(tx, &input.username)?;
            let groups = groups_for(tx, &user.id)?;
            let mut reasons = rule_reasons(&client.settings.policy.access, &user.username, &groups, input.mfa);
            if !client.enabled { reasons.push("client_disabled"); }
            if !user.enabled { reasons.push("user_disabled"); }
            if client.service { reasons.push("service_client_has_no_user_identity"); }
            if !input.scope.is_subset(&client.scopes) { reasons.push("unregistered_scope"); }
            if !client.allowed_groups.is_empty() && client.allowed_groups.is_disjoint(&groups) { reasons.push("no_matching_client_group"); }
            if client.require_mfa && !input.mfa { reasons.push("mfa_required"); }
            if let Some(reason) = crate::device_trust::policy_reason(self, tx, &client, None)? { reasons.push(reason); }
            let scopes: std::collections::BTreeMap<_, _> = input.scope.iter().filter_map(|s| client.settings.policy.scopes.get(s).map(|r| (s, rule_reasons(r, &user.username, &groups, input.mfa)))).collect();
            let allowed = reasons.is_empty() && scopes.values().all(Vec::is_empty);
            let mapped = mapped_claims(tx, &user, &client, &input.scope)?;
            Ok(json!({"simulation": true, "token_issued": false, "mfa_assumed": input.mfa, "allowed": allowed, "reasons": reasons, "scope_decisions": scopes,
                "userinfo": mapped, "id_token_identity_claims": if client.settings.userinfo_only { json!({"sub": subject(&user, &client)}) } else { mapped.clone() },
                "access_token_identity_claims": if client.settings.claims_in_access_token { mapped } else { json!({"sub": subject(&user, &client)}) }}))
        })
    }
}
