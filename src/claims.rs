pub use crate::model::claims::{
    AssuranceLevel, AuthenticationProof, ClaimMapping, ClaimSource, ConditionalClaimMapping,
    ConditionalPolicy, Policy, Predicate, PredicateFacts, Rule,
};
use crate::{
    error::{Error, Result},
    model::{Client, Identity, Session, User},
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
    fn session(&self, session_id: &str) -> Result<Option<Session>>;
    fn verified_upstream_source(&self, identity: &Identity) -> Result<Option<String>>;
    fn approved_device_at(&self, identity: &Identity, at: u64) -> Result<Option<u64>>;
    fn validate_reference_name(&self, name: &str) -> Result<()>;
    fn group_exists(&self, group: &str) -> Result<bool>;
    fn source_available(&self, source: &str) -> Result<bool>;
}

const MAX_PREDICATE_DEPTH: usize = 4;
const MAX_PREDICATE_NODES: usize = 16;
const MAX_AUTHENTICATION_AGE: u32 = 86_400;

/// The predicate inputs are reconstructed from server-held records for each
/// authorization decision. In particular, the session must still contain the
/// same identity as the grant; a projected claim never becomes a policy fact.
struct TrustedPredicateFacts<'a> {
    client: &'a Client,
    identity: &'a Identity,
    groups: BTreeSet<String>,
    source: Option<String>,
    device_verified_at: Option<u64>,
    at: u64,
}

impl<'a> TrustedPredicateFacts<'a> {
    fn load(
        tx: &impl ClaimsTx,
        client: &'a Client,
        user: &User,
        identity: &'a Identity,
    ) -> Result<Self> {
        let at = crate::crypto::now();
        let session = tx
            .session(&identity.session_id)?
            .ok_or_else(Error::forbidden)?;
        let actual = &session.identity;
        if session.revoked
            || session.expires_at <= at
            || actual.user_id != user.id
            || actual.user_id != identity.user_id
            || actual.epoch != user.epoch
            || actual.epoch != identity.epoch
            || actual.session_id != identity.session_id
            || actual.mfa != identity.mfa
            || actual.auth_time != identity.auth_time
            || actual.amr != identity.amr
            || actual
                .source
                .as_ref()
                .map(|s| (&s.id, &s.fingerprint, &s.link))
                != identity
                    .source
                    .as_ref()
                    .map(|s| (&s.id, &s.fingerprint, &s.link))
        {
            return Err(Error::forbidden());
        }
        let source = tx.verified_upstream_source(identity)?;
        let device_verified_at = tx.approved_device_at(identity, at)?;
        Ok(Self {
            client,
            identity,
            groups: tx.groups_for(&user.id)?,
            source,
            device_verified_at,
            at,
        })
    }
}

impl PredicateFacts for TrustedPredicateFacts<'_> {
    fn application(&self, id: &str) -> bool {
        self.client.id == id
    }
    fn group_member(&self, group: &str) -> bool {
        self.groups.contains(group)
    }
    fn verified_source(&self, source: &str) -> bool {
        self.source.as_deref() == Some(source)
    }
    fn proof_fresh(&self, proof: AuthenticationProof, max_age_seconds: u32) -> bool {
        self.identity.auth_time > 0
            && self.identity.auth_time <= self.at
            && self.at - self.identity.auth_time <= u64::from(max_age_seconds)
            && match proof {
                AuthenticationProof::Password => {
                    self.identity.source.is_none()
                        && self
                            .identity
                            .authentication_methods()
                            .iter()
                            .any(|method| method == "pwd")
                }
                AuthenticationProof::Passkey => {
                    self.identity.source.is_none()
                        && self
                            .identity
                            .authentication_methods()
                            .iter()
                            .any(|method| method == "webauthn")
                }
                AuthenticationProof::Source => self.source.is_some(),
            }
    }
    fn assurance(&self, level: AssuranceLevel) -> bool {
        let required = match level {
            AssuranceLevel::Password => crate::assurance::PASSWORD,
            AssuranceLevel::Mfa => crate::assurance::MFA,
            AssuranceLevel::Federated => crate::assurance::FEDERATED,
            AssuranceLevel::Certificate => crate::radius::eap::CERTIFICATE_ACR,
        };
        crate::assurance::actual(self.identity) == required
    }
    fn approved_device(&self, max_age_seconds: u32) -> bool {
        self.device_verified_at
            .is_some_and(|verified| self.at - verified <= u64::from(max_age_seconds))
    }
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
    if let Some(policy) = client.settings.policy.conditional() {
        if !cfg!(feature = "platform") {
            return Err(Error::forbidden());
        }
        let facts = TrustedPredicateFacts::load(tx, client, user, identity)?;
        if !policy
            .access
            .iter()
            .all(|condition| condition.evaluate(&facts))
            || scopes.iter().any(|scope| {
                policy.scopes.get(scope).is_some_and(|conditions| {
                    !conditions
                        .iter()
                        .all(|condition| condition.evaluate(&facts))
                })
            })
        {
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
        let value = claim_value(tx, user, &mapping.source)?;
        if !value.is_null() {
            claims[&mapping.claim] = value;
        }
    }
    Ok(claims)
}

/// Apply conditional mappings only with a currently authorized identity. The
/// ordinary mapping function remains useful for a simulation with no session.
pub fn mapped_claims_for_identity(
    tx: &impl ClaimsTx,
    user: &User,
    client: &Client,
    scopes: &BTreeSet<String>,
    identity: &Identity,
) -> Result<Value> {
    let mut claims = mapped_claims(tx, user, client, scopes)?;
    if let Some(policy) = client.settings.policy.conditional() {
        if !cfg!(feature = "platform") {
            return Err(Error::forbidden());
        }
        enforce(tx, client, user, identity, scopes)?;
        let facts = TrustedPredicateFacts::load(tx, client, user, identity)?;
        for conditional in &policy.claim_mappings {
            let mapping = &conditional.mapping;
            if scopes.contains(&mapping.scope) && conditional.when.evaluate(&facts) {
                let value = claim_value(tx, user, &mapping.source)?;
                if !value.is_null() {
                    claims[&mapping.claim] = value;
                }
            }
        }
    }
    Ok(claims)
}

fn claim_value(tx: &impl ClaimsTx, user: &User, source: &ClaimSource) -> Result<Value> {
    Ok(match source {
        ClaimSource::Username => json!(user.username),
        ClaimSource::DisplayName => json!(user.display_name),
        ClaimSource::Email => json!(user.email),
        ClaimSource::EmailVerified => json!(user.email_verified),
        ClaimSource::Groups => json!(tx.groups_for(&user.id)?),
        ClaimSource::Attribute { key } => user.attributes.get(key).cloned().unwrap_or(Value::Null),
        ClaimSource::Literal { value } => value.clone(),
    })
}

fn validate_mapping(
    client: &Client,
    mapping: &ClaimMapping,
    names: &mut BTreeSet<String>,
    conditional: bool,
) -> Result<()> {
    if !client.scopes.contains(&mapping.scope)
        || mapping.claim.is_empty()
        || mapping.claim.len() > 128
        || mapping.claim.chars().any(char::is_control)
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
        .contains(&mapping.claim.as_str())
        || conditional && ["name", "preferred_username", "groups"].contains(&mapping.claim.as_str())
        || !names.insert(mapping.claim.clone())
    {
        return Err(Error::bad("Invalid, duplicate or protected claim mapping"));
    }
    Ok(())
}

fn validate_predicate(
    client: &Client,
    predicate: &Predicate,
    depth: usize,
    nodes: &mut usize,
) -> Result<()> {
    *nodes += 1;
    if depth > MAX_PREDICATE_DEPTH || *nodes > MAX_PREDICATE_NODES {
        return Err(Error::bad("Conditional policy exceeds predicate limits"));
    }
    match predicate {
        Predicate::Application { id } if id != &client.id => Err(Error::bad(
            "Conditional policy references another application",
        )),
        Predicate::ProofFresh {
            max_age_seconds, ..
        } if !(1..=MAX_AUTHENTICATION_AGE).contains(max_age_seconds) => Err(Error::bad(
            "Conditional proof age is outside its supported range",
        )),
        Predicate::ApprovedDevice { max_age_seconds }
            if !(1..=crate::device_trust::MAX_FRESHNESS as u32).contains(max_age_seconds) =>
        {
            Err(Error::bad(
                "Conditional device age is outside its supported range",
            ))
        }
        Predicate::All { of } | Predicate::Any { of } => {
            if of.is_empty() || of.len() > 8 {
                return Err(Error::bad("Conditional policy needs 1–8 composite members"));
            }
            for child in of {
                validate_predicate(client, child, depth + 1, nodes)?;
            }
            Ok(())
        }
        Predicate::Not { condition } => validate_predicate(client, condition, depth + 1, nodes),
        _ => Ok(()),
    }
}

fn validate_predicate_references(tx: &impl ClaimsTx, predicate: &Predicate) -> Result<()> {
    match predicate {
        Predicate::GroupMember { group } => {
            tx.validate_reference_name(group)?;
            if !tx.group_exists(group)? {
                return Err(Error::bad("Conditional policy references an unknown group"));
            }
            Ok(())
        }
        Predicate::VerifiedSource { source } => {
            tx.validate_reference_name(source)?;
            if !tx.source_available(source)? {
                return Err(Error::bad(
                    "Conditional policy references an unavailable source",
                ));
            }
            Ok(())
        }
        Predicate::All { of } | Predicate::Any { of } => {
            for child in of {
                validate_predicate_references(tx, child)?;
            }
            Ok(())
        }
        Predicate::Not { condition } => validate_predicate_references(tx, condition),
        _ => Ok(()),
    }
}

pub(crate) fn validate_conditional_references(tx: &impl ClaimsTx, client: &Client) -> Result<()> {
    if let Some(policy) = client.settings.policy.conditional() {
        for predicate in policy
            .access
            .iter()
            .chain(policy.scopes.values().flatten())
            .chain(policy.claim_mappings.iter().map(|mapping| &mapping.when))
        {
            validate_predicate_references(tx, predicate)?;
        }
    }
    Ok(())
}

pub fn validate_mappings(client: &Client) -> Result<()> {
    let mut names = BTreeSet::new();
    for mapping in &client.settings.claim_mappings {
        validate_mapping(client, mapping, &mut names, false)?;
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
    if let Some(policy) = client.settings.policy.conditional() {
        if !cfg!(feature = "platform") {
            return Err(Error::bad("Conditional policy requires the Platform build"));
        }
        if policy.access.len() > 16
            || policy.scopes.len() > 32
            || policy.claim_mappings.len() > 64
            || policy
                .scopes
                .values()
                .any(|conditions| conditions.len() > 16)
            || policy.access.len()
                + policy.scopes.values().map(Vec::len).sum::<usize>()
                + policy.claim_mappings.len()
                > 128
            || policy
                .scopes
                .keys()
                .any(|scope| !client.scopes.contains(scope))
        {
            return Err(Error::bad("Conditional policy exceeds configured limits"));
        }
        for predicate in policy
            .access
            .iter()
            .chain(policy.scopes.values().flatten())
            .chain(policy.claim_mappings.iter().map(|mapping| &mapping.when))
        {
            let mut nodes = 0;
            validate_predicate(client, predicate, 1, &mut nodes)?;
        }
        for conditional in &policy.claim_mappings {
            validate_mapping(client, &conditional.mapping, &mut names, true)?;
        }
    }
    if names.len() > 64
        || serde_json::to_vec(&client.settings)
            .map_err(Error::internal)?
            .len()
            > 16_384
    {
        return Err(Error::bad("Provider mappings exceed configured limits"));
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
    if client.settings.policy.conditional().is_some() {
        reasons.push("conditional_policy_requires_session");
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
