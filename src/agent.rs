use crate::{
    core::{Core, audit, user_by_name, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::User,
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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
    ("operations.read", "operations"),
    ("operations.backup", "operations"),
    ("ssf.manage", "ssf"),
    ("ssf.configure", "ssf"),
];

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Permission {
    pub action: String,
    pub resource: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub permissions: Vec<Permission>,
    pub expires_at: u64,
    pub created_at: u64,
    pub enabled: bool,
    pub token_hash: String,
    /// Owning user id. Absent on rows created before parent ownership.
    #[serde(default)]
    pub parent_user: Option<String>,
}
impl Agent {
    pub fn view(&self) -> Value {
        json!({"id": self.id, "parent_user": self.parent_user, "permissions": self.permissions, "expires_at": self.expires_at, "created_at": self.created_at, "enabled": self.enabled})
    }
}

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
    pub permissions: Vec<Permission>,
}
impl Principal {
    pub fn allows(&self, action: &str, resource: &str) -> bool {
        !self.agent
            || self
                .permissions
                .iter()
                .any(|p| p.action == action && (p.resource == resource || p.resource == "*"))
    }
    pub fn require(&self, action: &str, resource: &str) -> Result<()> {
        if self.allows(action, resource) {
            Ok(())
        } else {
            Err(Error::forbidden())
        }
    }
}

impl Core {
    pub fn rotate_agent(&self, token: &str, id: &str, ttl: u64) -> Result<Value> {
        if !(60..=2_592_000).contains(&ttl) {
            return Err(Error::bad("Agent lifetime must be 60 seconds to 30 days"));
        }
        self.mutation(token, |tx| {
            let actor = self.admin(tx, token)?;
            let mut agent = tx.get::<Agent>("agents", id)?.filter(|a| a.enabled).ok_or_else(|| Error::missing("Enabled agent not found"))?;
            // A disabled or deleted parent blocks rotation even if the row was not revoked yet.
            if !parent_active(tx, &agent)? {
                return Err(Error::conflict("Agent parent is disabled or deleted"));
            }
            tx.delete("agent_tokens", &agent.token_hash)?;
            let credential = crypto::random_token("ri_agent_");
            // Parent, id, and permissions stay as created. Only the token and expiry change.
            agent.token_hash = digest(&credential); agent.expires_at = now() + ttl;
            tx.put("agents", id, &agent)?; tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
            audit(tx, &actor.id, "agent.rotate", id)?;
            Ok(json!({"agent":agent.view(),"credential":{"issuer":self.config.issuer,"agent_id":id,"token":credential,"expires_at":agent.expires_at}}))
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
                permissions: agent.permissions,
            })
        } else {
            let user = self.admin(tx, token)?;
            Ok(Principal {
                id: user.id,
                agent: false,
                permissions: vec![],
            })
        }
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
        Ok(actor)
    }
    pub fn create_agent(&self, token: &str, input: NewAgent) -> Result<Value> {
        validate_name(&input.id)?;
        if let Some(parent) = &input.parent {
            validate_name(parent)?;
        }
        if !(60..=2_592_000).contains(&input.ttl)
            || input.permissions.is_empty()
            || input.permissions.len() > 100
        {
            return Err(Error::bad(
                "Agent requires 1–100 permissions and a lifetime of 60 seconds to 30 days",
            ));
        }
        for permission in &input.permissions {
            let (_, kind) = ACTIONS
                .iter()
                .find(|(a, _)| *a == permission.action)
                .ok_or_else(|| Error::bad("Unknown agent permission"))?;
            if permission.resource != "*" {
                let (prefix, name) = permission
                    .resource
                    .split_once('/')
                    .ok_or_else(|| Error::bad("Resource must be kind/name or *"))?;
                let kind_matches = if *kind == "directory" {
                    // LDAP stays directory/<id>. Cloud sync reuses the same actions
                    // with an exact workspace/<id> or entra/<id> resource.
                    matches!(prefix, "directory" | "workspace" | "entra")
                } else {
                    prefix == *kind
                };
                if !kind_matches {
                    return Err(Error::bad(
                        "Permission action and resource kind do not match",
                    ));
                }
                validate_name(name)?;
            }
        }
        self.mutation(token, |tx| {
            // Bootstrap/delegation is intentionally restricted to human administrators.
            let actor = self.admin(tx, token)?;
            if tx.get::<Agent>("agents", &input.id)?.is_some() { return Err(Error::conflict("Agent already exists")); }
            let parent_user = if let Some(username) = &input.parent {
                let parent = user_by_name(tx, username)?;
                if parent.admin {
                    return Err(Error::forbidden());
                }
                if !parent.enabled {
                    return Err(Error::bad("Parent user is disabled"));
                }
                Some(parent.id)
            } else {
                None
            };
            let credential = crypto::random_token("ri_agent_");
            let agent = Agent { id: input.id, permissions: input.permissions, expires_at: now() + input.ttl,
                created_at: now(), enabled: true, token_hash: digest(&credential), parent_user };
            tx.put("agents", &agent.id, &agent)?;
            tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
            audit(tx, &actor.id, "agent.create", &agent.id)?;
            Ok(json!({"agent": agent.view(), "credential": {"issuer": self.config.issuer, "agent_id": agent.id, "token": credential, "expires_at": agent.expires_at}}))
        })
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
        self.mutation(token, |tx| {
            let actor = self.admin(tx, token)?;
            let mut agent = tx
                .get::<Agent>("agents", id)?
                .ok_or_else(|| Error::missing("Agent not found"))?;
            agent.enabled = false;
            tx.put("agents", id, &agent)?;
            tx.delete("agent_tokens", &agent.token_hash)?;
            audit(tx, &actor.id, "agent.revoke", id)?;
            Ok(agent.view())
        })
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

/// Disable every agent owned by this user and drop its token in the caller's transaction.
pub(crate) fn revoke_owned(tx: &Tx<'_>, user_id: &str) -> Result<()> {
    for (id, mut agent) in tx.list::<Agent>("agents")? {
        if agent.parent_user.as_deref() != Some(user_id) {
            continue;
        }
        tx.delete("agent_tokens", &agent.token_hash)?;
        if agent.enabled {
            agent.enabled = false;
            tx.put("agents", &id, &agent)?;
        }
    }
    Ok(())
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

pub fn capabilities() -> Value {
    json!({"schema_version": "riauth.capabilities/v1", "version": env!("CARGO_PKG_VERSION"),
        "interface": "cli", "permissions": ACTIONS.iter().map(|(action, resource_kind)| json!({"action": action, "resource_kind": resource_kind})).collect::<Vec<_>>(),
        "features": ["portal.user_applications", "portal.terminal_sign_in", "audit.self_hosted_event_map", "saml.idp_signed_browser_sso", "saml.sp_initiated_logout", "saml.logout_fanout", "saml.upstream_logout", "saml.assertion_encryption", "radius.pap", "radius.radsec", "radius.eap_tls", "agents.certificate_bindings", "directory.ldap_provider", "proxy.forward_auth_sso", "proxy.shared_domain_sso", "proxy.reverse_proxy", "directory.ldap_sync", "identity.ldap_authentication", "identity.passkeys", "identity.email_verification", "identity.invitations", "identity.email_password_reset", "operations.postgresql", "operations.shared_rate_limits", "operations.native_tls", "operations.vault_transit_signing", "oidc.par", "oidc.jar", "oidc.jarm", "oidc.claims_requests", "oidc.dpop", "oidc.bound_key", "oidc.pairwise_subjects", "oidc.resource_indicators", "oidc.key_domains", "oidc.jwe", "oidc.provider_issuers", "oidc.frontchannel_logout", "oidc.session_management", "identity.oidc_sources", "identity.saml_sources", "identity.oauth_sources", "identity.source_linking", "identity.totp_import", "agents.source_manifests", "directory.scim_inbound", "directory.scim_outbound", "operations.prometheus", "operations.schema_migrations", "oidc.private_key_jwt", "oidc.federated_machine_grants", "oidc.token_exchange", "oidc.dynamic_registration", "oidc.code.pkce_s256", "oidc.device", "oidc.refresh_rotation", "oidc.native_redirects", "oidc.request_bound_reauthentication", "agents.scoped_credentials", "agents.plan_apply", "agents.atomic_idempotency", "agents.conditional_mutations", "agents.audit_run_id", "agents.schema", "oidc.browser_terminal_handoff", "oidc.rp_logout", "oidc.backchannel_logout", "oidc.claim_mappings", "oidc.scope_policies", "oidc.provider_settings", "oidc.cors", "operations.encrypted_backup_restore", "operations.encrypted_storage", "identity.recovery_codes", "identity.self_password_change", "access.temporary_entitlements", "identity.scheduled_offboarding", "operations.audit_review", "operations.csv_export", "identity.windows_device_login", "agents.parent_ownership", "directory.workspace_sync", "directory.entra_sync", "identity.https_client_certificates", "identity.device_trust", "ssf.push"],
        "schemas": crate::schema::NAMES, "cli_result_schema": "riauth.cli/v1", "error_exit_codes": {"operation_failed": 1, "usage": 2, "authentication": 3, "permission": 4, "conflict": 5, "retryable": 6}})
}
