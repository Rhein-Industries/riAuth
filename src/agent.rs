use crate::{
    core::Core,
    crypto::{self, digest, now},
    error::{Error, Result},
    management,
    model::{Session, User},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use crate::identity::agent_credentials::{Agent, Permission};

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
];

#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewAgent {
    pub id: String,
    pub permissions: Vec<Permission>,
    pub ttl: u64,
    /// Existing enabled non-administrator username. Ownership grants no permissions.
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
        // writer repeats it at its transaction boundary.
        management::validate_agent_rotation_ttl(ttl)?;
        self.mutation(token, |tx| {
            management::rotate_agent(self, tx, token, id, ttl)
        })
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
            if !crypto::constant_eq(&agent.token_hash, &hash) || !authority_active(tx, &agent)? {
                return Err(Error::unauthorized());
            }
            Ok(Principal {
                id: format!("agent:{}", agent.id),
                agent: true,
                delegated: false,
                grants: vec![],
                permissions: agent.permissions,
            })
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
        self.mutation(token, |tx| management::create_agent(self, tx, token, input))
    }
    pub fn list_agents(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            self.admin(tx, token)?;
            Ok(json!(
                tx.list::<Agent>("agents")?
                    .into_iter()
                    .map(|(_, a)| a.view())
                    .collect::<Vec<_>>()
            ))
        })
    }
    pub fn revoke_agent(&self, token: &str, id: &str) -> Result<Value> {
        self.mutation(token, |tx| management::revoke_agent(self, tx, token, id))
    }
}

/// Enabled and unexpired, and the parent user — when set — still exists and is enabled.
pub(crate) fn authority_active(tx: &Tx<'_>, agent: &Agent) -> Result<bool> {
    Ok(agent.enabled && agent.expires_at > now() && parent_active(tx, agent)?)
}

pub(crate) fn parent_active(tx: &Tx<'_>, agent: &Agent) -> Result<bool> {
    let Some(parent) = &agent.parent_user else {
        return Ok(true);
    };
    Ok(tx
        .get::<User>("users", parent)?
        .is_some_and(|user| user.enabled))
}

/// Parent user id for an agent actor, or for agent create/rotate/revoke of that target.
pub(crate) fn audit_parent(
    tx: &Tx<'_>,
    actor: &str,
    action: &str,
    target: &str,
) -> Result<Option<String>> {
    let id = if let Some(id) = actor.strip_prefix("agent:") {
        id
    } else if action.starts_with("agent.") {
        target
    } else {
        return Ok(None);
    };
    Ok(tx
        .get::<Agent>("agents", id)?
        .and_then(|agent| agent.parent_user))
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
    "agents.parent_ownership",
    "directory.workspace_sync",
    "directory.entra_sync",
    "identity.https_client_certificates",
    "identity.device_trust",
    "ssf.push",
];

pub fn capabilities() -> Value {
    crate::capability::artifact()
}
