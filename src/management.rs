//! Shared management writes (M03).
//!
//! A resource moved here has one mutation implementation that every adapter
//! reaches: the `Core` methods behind `Core::mutation` (HTTP API, and therefore
//! the CLI, which manages remotely over HTTP) and desired-state reconcile.
//! Adapters keep only their envelope: direct writes use receipts and
//! `If-Match`; plans use their own immutable binding. Authorization, validation,
//! credential handling, dependent revocation, persistence and the direct audit
//! record are decided here, inside the caller's transaction.
//!
//! Applications (OAuth/OIDC/SAML/proxy client records) and groups use this seam.
//! RFC 7591 registration reaches the same write path with its own bounded
//! authority, not a management principal.

use crate::{
    agent::Principal,
    core::{audit, revoke_client_grants, validate_client, validate_display, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    jose::ClientAuthMethod,
    model::{Client, Group, ProviderSettings, User},
    registration::{RegistrationAuthority, RegistrationRequest},
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// The shared secret an application write asks for.
pub(crate) enum Secret<'a> {
    /// Keep the stored secret, if any.
    Keep,
    /// Generate a new secret, returned once to the authorized caller.
    Issue,
    /// Store a secret the caller resolved (desired-state references).
    Supplied(&'a str),
}

/// The audit record an application write produces.
pub(crate) enum Record<'a> {
    /// A direct write records its own action, e.g. `client.create`.
    Direct(&'a str),
    /// Plan apply records `<kind>.reconcile` and `state.apply` for the plan.
    Plan,
}

enum Authority<'a> {
    Management(&'a Principal, Record<'a>),
    Registration(&'a RegistrationAuthority),
}

pub(crate) struct ClientWrite {
    pub(crate) client: Client,
    /// A newly generated secret; never persisted in plaintext.
    pub(crate) secret: Option<String>,
}

/// Adapters supply intent, while the service reads the current group and
/// applies the change in their transaction.
pub(crate) enum GroupIntent<'a> {
    Create(&'a BTreeSet<String>),
    ReplaceMembers(&'a BTreeSet<String>),
    Member {
        user_id: &'a str,
        present: bool,
    },
    /// Dependent cleanup for an already disabled identity. This only removes
    /// that identity and is authorized by its exact user.write scope.
    OffboardMember {
        user_id: &'a str,
        username: &'a str,
    },
}

/// Some adapters finish other records before emitting one enclosing audit.
pub(crate) enum GroupAudit<'a> {
    OnChange { action: &'a str, target: &'a str },
    Deferred,
}

pub(crate) struct GroupWrite {
    pub(crate) group: Group,
    pub(crate) changed: bool,
}

/// Create or change group membership. Every call rechecks the relevant right
/// at the write boundary; unchanged membership needs the right but emits no
/// group write or audit. Creation conflicts even when a caller repeats a name
/// without an idempotency receipt, including a SCIM tombstone's local row.
pub(crate) fn write_group(
    tx: &Tx<'_>,
    actor: &Principal,
    name: &str,
    intent: GroupIntent<'_>,
    record: GroupAudit<'_>,
) -> Result<GroupWrite> {
    let resource = format!("group/{name}");
    match intent {
        GroupIntent::Create(members) => {
            actor.require("group.write", &resource)?;
            if !members.is_empty() {
                actor.require("group.members", &resource)?;
            }
            validate_name(name)?;
            if tx.get::<Group>("groups", name)?.is_some() {
                return Err(Error::conflict("Group already exists"));
            }
            validate_group_members(tx, members)?;
            let group = Group {
                name: name.into(),
                members: members.clone(),
            };
            tx.put("groups", name, &group)?;
            audit_group(tx, actor, record)?;
            Ok(GroupWrite {
                group,
                changed: true,
            })
        }
        GroupIntent::ReplaceMembers(members) => {
            actor.require("group.members", &resource)?;
            validate_name(name)?;
            let mut group = existing_group(tx, name)?;
            validate_group_members(tx, members)?;
            let changed = group.members != *members;
            if changed {
                group.members = members.clone();
                tx.put("groups", name, &group)?;
                audit_group(tx, actor, record)?;
            }
            Ok(GroupWrite { group, changed })
        }
        GroupIntent::Member { user_id, present } => {
            actor.require("group.members", &resource)?;
            validate_name(name)?;
            let mut group = existing_group(tx, name)?;
            if present {
                validate_group_member(tx, user_id)?;
            }
            let changed = if present {
                group.members.insert(user_id.into())
            } else {
                group.members.remove(user_id)
            };
            if changed {
                tx.put("groups", name, &group)?;
                audit_group(tx, actor, record)?;
            }
            Ok(GroupWrite { group, changed })
        }
        GroupIntent::OffboardMember { user_id, username } => {
            actor.require("user.write", &format!("user/{username}"))?;
            validate_name(username)?;
            let user = tx
                .get::<User>("users", user_id)?
                .ok_or_else(|| Error::missing("User not found"))?;
            if user.id != user_id || user.username != username || user.enabled {
                return Err(Error::conflict("Offboarded user identity does not match"));
            }
            validate_name(name)?;
            let mut group = existing_group(tx, name)?;
            let changed = group.members.remove(user_id);
            if changed {
                tx.put("groups", name, &group)?;
                audit_group(tx, actor, record)?;
            }
            Ok(GroupWrite { group, changed })
        }
    }
}

fn existing_group(tx: &Tx<'_>, name: &str) -> Result<Group> {
    let group = tx
        .get::<Group>("groups", name)?
        .ok_or_else(|| Error::missing("Group not found"))?;
    if group.name != name {
        return Err(Error::conflict("Group identity does not match its key"));
    }
    Ok(group)
}

fn validate_group_member(tx: &Tx<'_>, user_id: &str) -> Result<()> {
    if tx.get::<User>("users", user_id)?.is_none() {
        return Err(Error::bad("Group references unknown user"));
    }
    Ok(())
}

fn validate_group_members(tx: &Tx<'_>, members: &BTreeSet<String>) -> Result<()> {
    for member in members {
        validate_group_member(tx, member)?;
    }
    Ok(())
}

fn audit_group(tx: &Tx<'_>, actor: &Principal, record: GroupAudit<'_>) -> Result<()> {
    if let GroupAudit::OnChange { action, target } = record {
        audit(tx, &actor.id, action, target)?;
    }
    Ok(())
}

/// The direct and manifest adapters interpret `service` and private-key
/// authentication as confidential even when their input flag is omitted.
pub(crate) fn effective_confidential(
    declared: bool,
    service: bool,
    settings: &ProviderSettings,
) -> bool {
    declared
        || service
        || settings.token_endpoint_auth_method == Some(ClientAuthMethod::PrivateKeyJwt)
}

/// The record and secret request a direct create (`NewClient`) asks for.
pub(crate) fn new_client(input: crate::model::NewClient) -> (Client, Secret<'static>) {
    let secret = if effective_confidential(input.confidential, input.service, &input.settings) {
        Secret::Issue
    } else {
        Secret::Keep
    };
    let client = Client {
        id: input.client_id,
        name: input.name,
        secret_hash: None,
        redirect_uris: input.redirect_uris,
        scopes: input.scopes,
        allowed_groups: input.allowed_groups,
        require_mfa: input.require_mfa,
        enabled: true,
        service: input.service,
        settings: input.settings,
    };
    (client, secret)
}

/// The one write path for an application record.
///
/// `existing` is the record read in this transaction, or `None` to create.
/// Private-key clients never keep a shared secret. The client type
/// (confidential/public, service) is immutable. Authorization is checked
/// before validation: any change except a pure credential change needs
/// `client.write`; a change to the secret or to authentication settings of an
/// existing client needs `client.rotate`. A pure credential rotation does not
/// revalidate unrelated configuration, so drift elsewhere (for example a renamed
/// policy user) cannot block an emergency secret rotation.
/// Disabling, credential changes and issuer/sector changes revoke dependent
/// grants before the record is replaced, so logout uses the endpoint the
/// sessions were established with.
pub(crate) fn write_client(
    tx: &Tx<'_>,
    actor: &Principal,
    existing: Option<&Client>,
    next: Client,
    secret: Secret<'_>,
    record: Record<'_>,
) -> Result<ClientWrite> {
    write_client_as(
        tx,
        Authority::Management(actor, record),
        existing,
        next,
        secret,
    )
}

/// Authorize and validate an application write exactly as `write_client` would, without
/// persisting it, revoking anything or returning a secret. The browser setup wizard checks
/// its draft with this, so its answers are the write path's own.
pub(crate) fn check_client(
    tx: &Tx<'_>,
    actor: &Principal,
    existing: Option<&Client>,
    next: Client,
    secret: Secret<'_>,
) -> Result<Client> {
    let authority = Authority::Management(actor, Record::Direct("client.check"));
    Ok(check_client_as(tx, &authority, existing, next, secret)?.client)
}

/// An authorized, validated record that `write_client_as` persists.
struct Checked {
    client: Client,
    issued: Option<String>,
    credential_change: bool,
}

fn write_client_as(
    tx: &Tx<'_>,
    authority: Authority<'_>,
    existing: Option<&Client>,
    next: Client,
    secret: Secret<'_>,
) -> Result<ClientWrite> {
    let Checked {
        client: next,
        issued,
        credential_change,
    } = check_client_as(tx, &authority, existing, next, secret)?;
    if let Some(c) = existing
        && (!next.enabled
            || credential_change
            || c.settings.issuer != next.settings.issuer
            || c.settings.pairwise_sector != next.settings.pairwise_sector)
    {
        revoke_client_grants(tx, &next.id)?;
    }
    tx.put("clients", &next.id, &next)?;
    match authority {
        Authority::Management(actor, Record::Direct(action)) => {
            audit(tx, &actor.id, action, &next.id)?;
        }
        Authority::Management(_, Record::Plan) => {}
        Authority::Registration(grant) => {
            audit(
                tx,
                &format!("registration:{}", grant.id()),
                "client.register",
                &next.id,
            )?;
        }
    }
    Ok(ClientWrite {
        client: next,
        secret: issued,
    })
}

fn check_client_as(
    tx: &Tx<'_>,
    authority: &Authority<'_>,
    existing: Option<&Client>,
    mut next: Client,
    secret: Secret<'_>,
) -> Result<Checked> {
    let resource = format!("client/{}", next.id);
    let requested = !matches!(secret, Secret::Keep);
    // A requested secret is a rotation even if the value happens to repeat.
    // A switch to private_key_jwt also discards a stored shared secret.
    let credential_change = existing.is_some_and(|c| {
        requested
            || c.settings.authentication_credentials_differ(&next.settings)
            || (c.secret_hash.is_some()
                && next.settings.token_endpoint_auth_method
                    == Some(ClientAuthMethod::PrivateKeyJwt))
    });
    let other_change = match existing {
        Some(c) => without_secret(c)? != without_secret(&next)?,
        None => true,
    };
    match authority {
        Authority::Management(actor, _) => {
            if other_change || !credential_change {
                actor.require("client.write", &resource)?;
            }
            if credential_change {
                actor.require("client.rotate", &resource)?;
            }
        }
        Authority::Registration(grant) => {
            if existing.is_some() || matches!(secret, Secret::Supplied(_)) {
                return Err(Error::forbidden());
            }
            require_registration_bounds(grant, &next, &secret)?;
        }
    }
    let registration_error = |error: Error| match authority {
        Authority::Registration(_) => Error::oauth(
            "invalid_client_metadata",
            "Client metadata conflicts with provider policy",
        ),
        Authority::Management(_, _) => error,
    };
    validate_name(&next.id).map_err(registration_error)?;
    validate_display(&next.name).map_err(registration_error)?;
    if existing.is_none() && tx.get::<Client>("clients", &next.id)?.is_some() {
        return Err(Error::conflict("Client already exists"));
    }
    let mut issued = None;
    next.secret_hash =
        if next.settings.token_endpoint_auth_method == Some(ClientAuthMethod::PrivateKeyJwt) {
            None
        } else {
            match secret {
                Secret::Keep => existing.and_then(|c| c.secret_hash.clone()),
                Secret::Issue => {
                    let value = crypto::random_token("ri_client_");
                    let hash = digest(&value);
                    issued = Some(value);
                    Some(hash)
                }
                Secret::Supplied(value) => Some(digest(value)),
            }
        };
    if let Some(c) = existing
        && (c.confidential() != next.confidential() || c.service != next.service)
    {
        return Err(Error::bad("Existing client type is immutable"));
    }
    // Authentication-setting changes are record changes and are validated.
    if other_change {
        validate_client(tx, &next).map_err(registration_error)?;
    }
    Ok(Checked {
        client: next,
        issued,
        credential_change,
    })
}

fn metadata(message: &str) -> Error {
    Error::oauth("invalid_client_metadata", message)
}

/// Enforce the template at the persistence boundary as well as while parsing
/// the request. A registration authority can only create its own bounded
/// interactive client, even if another caller is added to this module later.
fn require_registration_bounds(
    authority: &RegistrationAuthority,
    client: &Client,
    secret: &Secret<'_>,
) -> Result<()> {
    let template = authority.template();
    let (method, issue_secret) = match client.settings.token_endpoint_auth_method.as_ref() {
        Some(ClientAuthMethod::None) => ("none", false),
        Some(ClientAuthMethod::ClientSecretBasic) => ("client_secret_basic", true),
        Some(ClientAuthMethod::ClientSecretPost) => ("client_secret_post", true),
        Some(ClientAuthMethod::PrivateKeyJwt) => ("private_key_jwt", false),
        None => return Err(metadata("Unsupported authentication method")),
    };
    let mut expected_settings = template.settings.clone();
    expected_settings.implicit_consent = false;
    expected_settings.allowed_grants = client.settings.allowed_grants.clone();
    expected_settings.token_endpoint_auth_method =
        client.settings.token_endpoint_auth_method.clone();
    expected_settings.jwks = client.settings.jwks.clone();
    expected_settings.post_logout_redirect_uris = client.settings.post_logout_redirect_uris.clone();
    if !client.id.starts_with(&format!("{}-", template.id))
        || client.secret_hash.is_some()
        || client.service
        || !client.enabled
        || client.redirect_uris.is_empty()
        || client
            .redirect_uris
            .iter()
            .any(|uri| !template.redirect_uris.contains(uri))
        || client.scopes.is_empty()
        || !client.scopes.is_subset(&template.scopes)
        || client.allowed_groups != template.allowed_groups
        || client.require_mfa != template.require_mfa
        || client.settings != expected_settings
        || client.settings.allowed_grants.is_empty()
        || !client
            .settings
            .allowed_grants
            .is_subset(&template.grant_types)
        || !template.auth_methods.contains(method)
        || issue_secret != matches!(secret, Secret::Issue)
        || client
            .settings
            .post_logout_redirect_uris
            .iter()
            .any(|uri| !template.settings.post_logout_redirect_uris.contains(uri))
        || client.settings.exchange.is_some()
        || !client.settings.exchange_from.is_empty()
        || !client.settings.machine_trust.is_empty()
        || client.settings.implicit_consent
    {
        return Err(metadata(
            "Client metadata is outside the registration template",
        ));
    }
    Ok(())
}

/// RFC 7591 registration uses an initial access token only. Its template,
/// current creator rights, use count, client write and audit are checked and
/// committed in the caller's single store transaction.
pub(crate) fn register_client(
    tx: &Tx<'_>,
    initial_token: &str,
    request: RegistrationRequest,
) -> Result<Value> {
    let mut authority = RegistrationAuthority::for_token(tx, initial_token)?;
    let template = authority.template().clone();
    if request.redirect_uris.is_empty()
        || request
            .redirect_uris
            .iter()
            .any(|uri| !template.redirect_uris.contains(uri))
    {
        return Err(Error::oauth(
            "invalid_redirect_uri",
            "Redirects must be an exact subset of the registration template",
        ));
    }
    let method = request
        .token_endpoint_auth_method
        .as_deref()
        .unwrap_or("client_secret_basic");
    if !template.auth_methods.contains(method) {
        return Err(metadata("Authentication method is outside the template"));
    }
    let grants = request
        .grant_types
        .unwrap_or_else(|| BTreeSet::from(["authorization_code".into()]));
    if grants.is_empty()
        || !grants.is_subset(&template.grant_types)
        || request
            .response_types
            .as_ref()
            .is_some_and(|r| r != &BTreeSet::from(["code".into()]))
    {
        return Err(metadata("Unsupported grant or response type"));
    }
    if request.application_type.as_deref().is_some_and(|kind| {
        kind != if template.settings.native {
            "native"
        } else {
            "web"
        }
    }) {
        return Err(metadata("Application type is fixed by the template"));
    }
    let secret = if ["client_secret_basic", "client_secret_post"].contains(&method) {
        Secret::Issue
    } else {
        Secret::Keep
    };
    let mut settings = template.settings.clone();
    // Registration never turns an initial access token into consent authority.
    settings.implicit_consent = false;
    settings.allowed_grants = grants;
    settings.token_endpoint_auth_method = Some(match method {
        "none" => ClientAuthMethod::None,
        "client_secret_basic" => ClientAuthMethod::ClientSecretBasic,
        "client_secret_post" => ClientAuthMethod::ClientSecretPost,
        "private_key_jwt" => ClientAuthMethod::PrivateKeyJwt,
        _ => return Err(metadata("Unsupported authentication method")),
    });
    if let Some(uris) = request.post_logout_redirect_uris {
        if uris
            .iter()
            .any(|uri| !settings.post_logout_redirect_uris.contains(uri))
        {
            return Err(metadata("Logout redirects are outside the template"));
        }
        settings.post_logout_redirect_uris = uris;
    }
    settings.jwks = request.jwks;
    let cid = format!("{}-{}", template.id, crypto::id());
    validate_name(&cid)
        .map_err(|_| metadata("Template id leaves insufficient space for a generated client id"))?;
    let mut client = Client {
        id: cid,
        name: request.client_name.unwrap_or_else(|| template.id.clone()),
        secret_hash: None,
        redirect_uris: request.redirect_uris,
        scopes: template.scopes.clone(),
        allowed_groups: template.allowed_groups.clone(),
        require_mfa: template.require_mfa,
        enabled: true,
        service: false,
        settings,
    };
    if let Some(scope) = request.scope {
        client.scopes = crate::oidc::scope_request(&scope, &client)
            .map_err(|_| metadata("Scopes are outside the template"))?;
    }
    validate_display(&client.name).map_err(|_| metadata("Invalid client name"))?;
    authority.consume(tx)?;
    let ClientWrite { client, secret } = write_client_as(
        tx,
        Authority::Registration(&authority),
        None,
        client,
        secret,
    )?;
    let mut response = json!({
        "client_id": client.id,
        "client_id_issued_at": now(),
        "client_name": client.name,
        "redirect_uris": client.redirect_uris,
        "scope": client.scopes.iter().cloned().collect::<Vec<_>>().join(" "),
        "grant_types": client.settings.allowed_grants,
        "response_types": ["code"],
        "token_endpoint_auth_method": method,
        "application_type": if client.settings.native { "native" } else { "web" },
        "post_logout_redirect_uris": client.settings.post_logout_redirect_uris,
    });
    if let Some(jwks) = &client.settings.jwks {
        response["jwks"] = json!(jwks);
    }
    if let Some(secret) = secret {
        response["client_secret"] = json!(secret);
        response["client_secret_expires_at"] = json!(0);
    }
    Ok(response)
}

fn without_secret(client: &Client) -> Result<Value> {
    let mut value = serde_json::to_value(client).map_err(Error::internal)?;
    value["secret_hash"] = Value::Null;
    Ok(value)
}
