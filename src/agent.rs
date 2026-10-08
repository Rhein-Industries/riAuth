use crate::{
    config::Config,
    core::Core,
    crypto::{self, digest, now},
    error::{Error, Result},
    management,
    model::{Session, User},
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use crate::identity::agent_credentials::{Agent, Permission};
pub use crate::management::owner_agents::AgentProposalInput;
pub use crate::management::prepared_changes::{Factor, PrepareChange, SensitiveChange};

/// Prefix of the management credential for a browser session: the SSO cookie value follows.
/// `api::bearer` rejects whitespace, so an Authorization header can never carry one; only the
/// same-origin administration routes (`portal::admin`) build it, behind the browser guards.
const BROWSER_SESSION: &str = "browser-session ";

/// The management credential `principal` accepts for this browser session cookie.
pub(crate) fn browser_credential(cookie: &str) -> String {
    format!("{BROWSER_SESSION}{cookie}")
}

/// The SSO cookie of a browser management credential.
pub(crate) fn browser_cookie(token: &str) -> Option<&str> {
    token.strip_prefix(BROWSER_SESSION)
}

pub const ACTIONS: &[(&str, &str)] = &[
    ("ldap.search", "client"),
    ("certificate.read", "user"),
    ("certificate.write", "user"),
    ("mtls.read", "user"),
    ("mtls.bind", "user"),
    ("radius.enroll", "radius"),
    ("directory.read", "directory"),
    ("directory.sync", "directory"),
    ("provisioner.read", "provisioner"),
    ("provisioner.sync", "provisioner"),
    ("source.read", "source"),
    ("source.write", "source"),
    ("registration.read", "registration"),
    ("registration.write", "registration"),
    ("client.read", "client"),
    ("client.write", "client"),
    ("client.rotate", "client"),
    ("user.read", "user"),
    ("user.write", "user"),
    ("user.offboard", "user"),
    ("group.read", "group"),
    ("group.write", "group"),
    ("group.members", "group"),
    ("access.read", "access"),
    ("audit.read", "audit"),
    ("key.rotate", "key"),
    ("key.read", "key"),
    ("key.write", "key"),
    ("session.revoke", "session"),
    ("device.enroll", "device"),
    ("state.read", "state"),
    ("workflow.read", "workflow"),
    ("workflow.write", "workflow"),
    ("operations.read", "operations"),
    ("operations.backup", "operations"),
    ("ssf.manage", "ssf"),
    ("ssf.configure", "ssf"),
    ("profile.read", "user"),
    ("profile.write", "user"),
    ("sessions.read", "user"),
    ("sessions.revoke", "user"),
    ("consents.read", "user"),
    ("consents.revoke", "user"),
    ("agents.read", "user"),
    ("agents.revoke", "user"),
    ("changes.prepare", "user"),
];

/// Narrow management of one person's own account. Each names `user/<username>`,
/// `*`, or `self` for an owned agent, and none implies `user.read` or `user.write`.
pub const PERSONAL_ACTIONS: &[&str] = &[
    "profile.read",
    "profile.write",
    "sessions.read",
    "sessions.revoke",
    "consents.read",
    "consents.revoke",
    "agents.read",
    "agents.revoke",
    "changes.prepare",
];

/// Permission resource naming the owner's own account. Valid only for personal
/// actions of an owned agent, and resolved to `user/<username>` on every use.
pub const SELF_RESOURCE: &str = "self";

/// The authority an agent's owner holds now. An agent acts with its approved
/// permissions limited to this authority, recomputed on every credential use and
/// before every deferred job runs, so a lost grant takes effect at once.
pub(crate) enum OwnerAuthority {
    /// No owner: the approved permissions are the only ceiling.
    Unowned,
    /// A full administrator adds no authority: the approved permissions stay
    /// the ceiling and every agent restriction still applies.
    Administrator { username: String },
    /// Anyone else holds their exact live delegated grants, plus personal
    /// actions and the configuration revision for their own account.
    Scoped {
        username: String,
        grants: Vec<crate::delegation::HumanGrant>,
    },
}

impl OwnerAuthority {
    pub(crate) fn of(tx: &Tx<'_>, config: &Config, owner: &User) -> Result<Self> {
        Ok(if owner.admin {
            Self::Administrator {
                username: owner.username.clone(),
            }
        } else {
            Self::Scoped {
                username: owner.username.clone(),
                grants: crate::delegation::active(tx, config, &owner.id)?,
            }
        })
    }

    fn own_account(&self) -> Option<String> {
        match self {
            Self::Unowned => None,
            Self::Administrator { username } | Self::Scoped { username, .. } => {
                Some(format!("user/{username}"))
            }
        }
    }

    fn allows(&self, action: &str, resource: &str) -> bool {
        match self {
            Self::Unowned | Self::Administrator { .. } => true,
            Self::Scoped { username, grants } => {
                action == "state.read" && resource == "state/revision"
                    || PERSONAL_ACTIONS.contains(&action)
                        && resource.strip_prefix("user/") == Some(username.as_str())
                    || grants.iter().any(|grant| grant.allows(action, resource))
            }
        }
    }

    /// Every exact resource a scoped owner can hold, to narrow a wildcard.
    fn resources(&self) -> Vec<String> {
        let mut resources = vec!["state/revision".to_owned()];
        resources.extend(self.own_account());
        if let Self::Scoped { grants, .. } = self {
            resources.extend(grants.iter().map(|grant| grant.scope.clone()));
        }
        resources
    }

    /// Whether issuance may approve this permission for this owner: within
    /// the owner's authority now and, for anyone but an administrator, an
    /// exact resource or `self`, so a later change of role cannot widen it.
    pub(crate) fn approves(&self, permission: &Permission) -> bool {
        let effective = self.limit(std::slice::from_ref(permission));
        match self {
            Self::Unowned | Self::Administrator { .. } => !effective.is_empty(),
            Self::Scoped { .. } => permission.resource != "*" && effective.len() == 1,
        }
    }

    /// The approved permissions limited to this authority, with `self` resolved.
    /// A wildcard narrows to the owner's exact resources; nothing is widened.
    pub(crate) fn limit(&self, approved: &[Permission]) -> Vec<Permission> {
        let mut effective = Vec::new();
        for permission in approved {
            let resource = if permission.resource == SELF_RESOURCE {
                match self.own_account() {
                    Some(resource) => resource,
                    None => continue,
                }
            } else {
                permission.resource.clone()
            };
            let resources = match self {
                Self::Unowned | Self::Administrator { .. } => vec![resource],
                Self::Scoped { .. } if resource == "*" => self.resources(),
                Self::Scoped { .. } => vec![resource],
            };
            for resource in resources {
                let candidate = Permission {
                    action: permission.action.clone(),
                    resource,
                };
                if self.allows(&candidate.action, &candidate.resource)
                    && !effective.contains(&candidate)
                {
                    effective.push(candidate);
                }
            }
        }
        effective
    }
}

/// The stored view plus the permissions the agent holds now.
pub(crate) fn effective_view(tx: &Tx<'_>, config: &Config, agent: &Agent) -> Result<Value> {
    let mut view = agent.view();
    view["effective_permissions"] = match live_principal(tx, config, agent)? {
        Some(principal) => json!(principal.permissions),
        None => json!([]),
    };
    Ok(view)
}

/// The management principal of a stored agent, or `None` once it is revoked or
/// expired or its owner is missing or disabled. The credential path and every
/// deferred job that acts for an agent build it here.
pub(crate) fn live_principal(
    tx: &Tx<'_>,
    config: &Config,
    agent: &Agent,
) -> Result<Option<Principal>> {
    if !agent.enabled || agent.expires_at <= now() {
        return Ok(None);
    }
    let authority = match &agent.parent_user {
        None => OwnerAuthority::Unowned,
        Some(id) => match tx.get::<User>("users", id)?.filter(|owner| owner.enabled) {
            Some(owner) => OwnerAuthority::of(tx, config, &owner)?,
            None => return Ok(None),
        },
    };
    Ok(Some(Principal {
        id: format!("agent:{}", agent.id),
        agent: true,
        delegated: false,
        grants: vec![],
        permissions: authority.limit(&agent.permissions),
    }))
}

#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewAgent {
    pub id: String,
    pub permissions: Vec<Permission>,
    pub ttl: u64,
    /// Existing enabled username. Ownership grants no permissions: the agent acts
    /// with these permissions limited to the owner's current authority.
    #[serde(default)]
    pub parent: Option<String>,
}

#[derive(Clone)]
pub struct Principal {
    pub id: String,
    pub agent: bool,
    /// A human with current, exact grants rather than full administrator rights.
    pub delegated: bool,
    pub grants: Vec<crate::delegation::HumanGrant>,
    pub permissions: Vec<Permission>,
}
impl Principal {
    pub fn allows(&self, action: &str, resource: &str) -> bool {
        crate::edition::action_available(action)
            && (if self.delegated {
                self.grants
                    .iter()
                    .any(|grant| grant.allows(action, resource))
            } else if self.agent {
                self.permissions.iter().any(|p| {
                    crate::edition::agent_permission_available(&p.action, &p.resource)
                        && p.action == action
                        && (p.resource == resource || p.resource == "*")
                })
            } else {
                true
            })
    }
    pub fn require(&self, action: &str, resource: &str) -> Result<()> {
        if self.allows(action, resource) {
            Ok(())
        } else {
            Err(Error::forbidden())
        }
    }

    /// A directory operator's write authority is confined to a reviewed
    /// connector sync. Direct user and group APIs still require their own
    /// permissions; agents retain their existing combined permission checks.
    pub(crate) fn require_directory_user(
        &self,
        scope: &str,
        username: &str,
        user_id: Option<&str>,
    ) -> Result<()> {
        if self.delegated {
            self.require("directory.sync", scope)?;
            if user_id == Some(self.id.as_str()) {
                return Err(Error::forbidden());
            }
            Ok(())
        } else {
            self.require("user.write", &format!("user/{username}"))
        }
    }

    pub(crate) fn require_directory_group(&self, scope: &str, name: &str) -> Result<()> {
        if self.delegated {
            self.require("directory.sync", scope)
        } else {
            self.require("group.members", &format!("group/{name}"))
        }
    }
}

impl Core {
    pub fn rotate_agent(&self, token: &str, id: &str, ttl: u64) -> Result<Value> {
        // Retain the existing validation order before receipt lookup; the
        // writer repeats it at its transaction boundary. A generic mutation
        // receipt would persist and replay the plaintext rotated credential.
        management::validate_agent_rotation_ttl(ttl)?;
        self.store
            .write(|tx| management::rotate_agent(self, tx, token, id, ttl))
    }
    pub fn principal(&self, tx: &Tx<'_>, token: &str) -> Result<Principal> {
        if token.starts_with("ri_agent_") {
            let hash = digest(token);
            let id = tx
                .get::<String>("agent_tokens", &hash)?
                .ok_or_else(Error::unauthorized)?;
            let agent = tx
                .get::<Agent>("agents", &id)?
                .ok_or_else(Error::unauthorized)?;
            // Checked on every credential use, not only when a disable path revokes the row.
            if !crypto::constant_eq(&agent.token_hash, &hash) {
                return Err(Error::unauthorized());
            }
            live_principal(tx, &self.config, &agent)?.ok_or_else(Error::unauthorized)
        } else {
            let user = match browser_cookie(token) {
                Some(cookie) => self.browser_user(tx, cookie)?.0,
                None => self.session(tx, token)?.0,
            };
            let grants = if user.admin {
                vec![]
            } else {
                crate::delegation::active(tx, &self.config, &user.id)?
            };
            if !user.admin && grants.is_empty() {
                return Err(Error::forbidden());
            }
            Ok(Principal {
                id: user.id,
                agent: false,
                delegated: !user.admin,
                grants,
                permissions: vec![],
            })
        }
    }
    /// The user and session behind a browser SSO cookie, with the same session and identity
    /// checks as a bearer session.
    pub(crate) fn browser_user(&self, tx: &Tx<'_>, cookie: &str) -> Result<(User, Session)> {
        let session = self
            .browser_session(tx, Some(cookie))?
            .ok_or_else(Error::unauthorized)?;
        Ok((self.identity_user(tx, &session.identity)?, session))
    }
    pub(crate) fn management(
        &self,
        tx: &Tx<'_>,
        token: &str,
        action: &str,
        resource: &str,
    ) -> Result<Principal> {
        let actor = self.principal(tx, token)?;
        actor.require(action, resource)?;
        crate::reconciliation::validate_apply_lease(tx, &actor)?;
        Ok(actor)
    }
    pub fn create_agent(&self, token: &str, input: NewAgent) -> Result<Value> {
        // Retain the existing validation order before receipt lookup; the writer
        // repeats validation at its own transaction boundary.
        management::validate_new_agent(&input)?;
        // This writer records only a redacted issuance marker. The generic
        // mutation receipt would persist and replay the plaintext credential.
        self.store
            .write(|tx| management::create_agent(self, tx, token, input))
    }
    pub fn list_agents(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            self.admin(tx, token)?;
            let mut views = Vec::new();
            for (_, agent) in tx.list::<Agent>("agents")? {
                views.push(effective_view(tx, &self.config, &agent)?);
            }
            Ok(json!(views))
        })
    }
    pub fn revoke_agent(&self, token: &str, id: &str) -> Result<Value> {
        // Remote revocation must bind the exact request to a configuration
        // revision. In-process lifecycle callers have no HTTP context.
        if let Some(context) = crate::context::current()
            && (context.idempotency_key.is_none() || context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "Agent revocation requires Idempotency-Key and If-Match",
            ));
        }
        self.mutation(token, |tx| management::revoke_agent(self, tx, token, id))
    }
}

/// The parent user — when set — still exists and is enabled.
pub(crate) fn parent_active(tx: &Tx<'_>, agent: &Agent) -> Result<bool> {
    let Some(parent) = &agent.parent_user else {
        return Ok(true);
    };
    Ok(tx
        .get::<User>("users", parent)?
        .is_some_and(|user| user.enabled))
}

/// Record the owner (`parent_user`) and the approving human (`authorized_by`)
/// of an agent actor, or of the agent an `agent.*` action targets.
pub(crate) fn audit_attribution(
    tx: &Tx<'_>,
    actor: &str,
    action: &str,
    target: &str,
    details: &mut Value,
) -> Result<()> {
    let acting = actor.strip_prefix("agent:");
    let subject = action.starts_with("agent.").then_some(target);
    // An acting agent is attributed first. When it acts on another agent,
    // that agent's owner is recorded separately as `target_parent_user`.
    let (attributed, other) = match (acting, subject) {
        (Some(acting), Some(subject)) if subject != acting => (acting, Some(subject)),
        (Some(acting), _) => (acting, None),
        (None, Some(subject)) => (subject, None),
        (None, None) => return Ok(()),
    };
    if let Some(agent) = tx.get::<Agent>("agents", attributed)? {
        if let Some(parent) = agent.parent_user {
            details["parent_user"] = json!(parent);
        }
        if let Some(authorizer) = agent.authorized_by {
            details["authorized_by"] = json!(authorizer);
        }
    }
    if let Some(parent) = other
        .map(|id| tx.get::<Agent>("agents", id))
        .transpose()?
        .flatten()
        .and_then(|agent| agent.parent_user)
    {
        details["target_parent_user"] = json!(parent);
    }
    Ok(())
}

pub const FEATURES: &[&str] = &[
    "portal.user_applications",
    "portal.terminal_sign_in",
    "audit.self_hosted_event_map",
    "saml.idp_signed_browser_sso",
    "saml.sp_initiated_logout",
    "saml.logout_fanout",
    "saml.upstream_logout",
    "saml.assertion_encryption",
    "radius.pap",
    "radius.radsec",
    "radius.eap_tls",
    "agents.certificate_bindings",
    "directory.ldap_provider",
    "proxy.forward_auth_sso",
    "proxy.shared_domain_sso",
    "proxy.reverse_proxy",
    "directory.ldap_sync",
    "identity.ldap_authentication",
    "identity.passkeys",
    "identity.email_verification",
    "identity.invitations",
    "identity.email_password_reset",
    "operations.postgresql",
    "operations.shared_rate_limits",
    "operations.native_tls",
    "operations.vault_transit_signing",
    "oidc.par",
    "oidc.jar",
    "oidc.jarm",
    "oidc.claims_requests",
    "oidc.dpop",
    "oidc.bound_key",
    "oidc.pairwise_subjects",
    "oidc.resource_indicators",
    "oidc.key_domains",
    "oidc.jwe",
    "oidc.provider_issuers",
    "oidc.frontchannel_logout",
    "oidc.session_management",
    "identity.oidc_sources",
    "identity.saml_sources",
    "identity.oauth_sources",
    "identity.source_linking",
    "identity.totp_import",
    "agents.source_manifests",
    "directory.scim_inbound",
    "directory.scim_outbound",
    "operations.prometheus",
    "operations.schema_migrations",
    "oidc.private_key_jwt",
    "oidc.federated_machine_grants",
    "oidc.token_exchange",
    "oidc.dynamic_registration",
    "oidc.code.pkce_s256",
    "oidc.device",
    "oidc.refresh_rotation",
    "oidc.native_redirects",
    "oidc.request_bound_reauthentication",
    "agents.scoped_credentials",
    "agents.plan_apply",
    "agents.atomic_idempotency",
    "agents.conditional_mutations",
    "agents.audit_run_id",
    "agents.schema",
    "oidc.browser_terminal_handoff",
    "oidc.rp_logout",
    "oidc.backchannel_logout",
    "oidc.claim_mappings",
    "oidc.scope_policies",
    "oidc.provider_settings",
    "oidc.cors",
    "operations.encrypted_backup_restore",
    "operations.encrypted_storage",
    "identity.recovery_codes",
    "identity.self_password_change",
    "access.temporary_entitlements",
    "identity.scheduled_offboarding",
    "operations.audit_review",
    "operations.csv_export",
    "identity.windows_device_login",
    "agents.parent_ownership",
    "directory.workspace_sync",
    "directory.entra_sync",
    "identity.https_client_certificates",
    "identity.device_trust",
    "ssf.push",
    "workflow.controlled_extensions",
];

pub const PLATFORM_FEATURES: &[&str] = &[
    "audit.self_hosted_event_map",
    "saml.idp_signed_browser_sso",
    "saml.sp_initiated_logout",
    "saml.logout_fanout",
    "saml.upstream_logout",
    "saml.assertion_encryption",
    "radius.pap",
    "radius.radsec",
    "radius.eap_tls",
    "agents.certificate_bindings",
    "directory.ldap_provider",
    "proxy.forward_auth_sso",
    "proxy.shared_domain_sso",
    "proxy.reverse_proxy",
    "operations.vault_transit_signing",
    "identity.saml_sources",
    "directory.scim_inbound",
    "access.temporary_entitlements",
    "identity.scheduled_offboarding",
    "identity.windows_device_login",
    "directory.workspace_sync",
    "directory.entra_sync",
    "identity.https_client_certificates",
    "identity.device_trust",
    "ssf.push",
    "workflow.controlled_extensions",
];

pub fn capabilities() -> Value {
    crate::capability::artifact()
}
