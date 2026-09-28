//! RFC 7591 registration using bounded, revocable initial access tokens.
use crate::{
    core::Core,
    crypto::{digest, now},
    error::{Error, Result},
    jose::PublicJwks,
    model::ProviderSettings,
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrationTemplate {
    pub id: String,
    pub redirect_uris: Vec<String>,
    pub scopes: BTreeSet<String>,
    pub grant_types: BTreeSet<String>,
    pub auth_methods: BTreeSet<String>,
    pub settings: ProviderSettings,
    #[serde(default)]
    pub allowed_groups: BTreeSet<String>,
    #[serde(default)]
    pub require_mfa: bool,
    pub ttl: u64,
    pub max_uses: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct InitialAccess {
    pub(crate) template: RegistrationTemplate,
    pub(crate) token_hash: String,
    pub(crate) created_by: String,
    pub(crate) creator_agent: bool,
    pub(crate) expires_at: u64,
    pub(crate) used: u32,
    pub(crate) enabled: bool,
}
impl InitialAccess {
    pub(crate) fn view(&self) -> Value {
        json!({"template": self.template, "created_by": self.created_by, "expires_at": self.expires_at, "used": self.used, "enabled": self.enabled})
    }
}

/// Proof that an initial access token is live and its issuing authority still
/// holds registration rights. This is deliberately separate from an agent or
/// management principal: the token never acquires `client.write`.
pub(crate) struct RegistrationAuthority {
    record: InitialAccess,
}

impl RegistrationAuthority {
    pub(crate) fn for_token(tx: &Tx<'_>, token: &str) -> Result<Self> {
        let hash = digest(token);
        let id = tx
            .get::<String>("registration_tokens", &hash)?
            .ok_or_else(Error::unauthorized)?;
        let record = tx
            .get::<InitialAccess>("registrations", &id)?
            .filter(|r| {
                r.template.id == id && r.token_hash == hash && r.enabled && r.expires_at > now()
            })
            .ok_or_else(Error::unauthorized)?;
        check_creator(tx, &record)?;
        Ok(Self { record })
    }

    pub(crate) fn template(&self) -> &RegistrationTemplate {
        &self.record.template
    }

    pub(crate) fn id(&self) -> &str {
        &self.record.template.id
    }

    /// A completed request may replay its receipt after the final use; new
    /// registrations still require an available use.
    pub(crate) fn require_available(&self) -> Result<()> {
        if self.record.used >= self.record.template.max_uses {
            return Err(Error::unauthorized());
        }
        Ok(())
    }

    /// Persist use in the same transaction as the client and its audit event.
    pub(crate) fn consume(&mut self, tx: &Tx<'_>) -> Result<()> {
        self.require_available()?;
        self.record.used += 1;
        tx.put("registrations", self.id(), &self.record)
    }
}

#[derive(schemars::JsonSchema, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrationRequest {
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    pub scope: Option<String>,
    pub grant_types: Option<BTreeSet<String>>,
    pub response_types: Option<BTreeSet<String>>,
    pub token_endpoint_auth_method: Option<String>,
    pub application_type: Option<String>,
    pub jwks: Option<PublicJwks>,
    pub post_logout_redirect_uris: Option<Vec<String>>,
}

impl Core {
    pub fn registration_template(
        &self,
        token: &str,
        template: RegistrationTemplate,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            crate::management::create_registration_template(tx, &actor, template)
        })
    }
    pub fn registration_templates(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(
                tx.list::<InitialAccess>("registrations")?
                    .iter()
                    .filter(|(_, r)| actor.allows(
                        "registration.read",
                        &format!("registration/{}", r.template.id)
                    ))
                    .map(|(_, r)| r.view())
                    .collect::<Vec<_>>()
            ))
        })
    }
    pub fn revoke_registration(&self, token: &str, id: &str) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            crate::management::revoke_registration_template(tx, &actor, id)
        })
    }
    pub fn dynamic_register(
        &self,
        initial_token: &str,
        request: RegistrationRequest,
    ) -> Result<Value> {
        self.store.write(|tx| {
            crate::management::register_client(tx, &self.config, initial_token, request)
        })
    }
}

fn check_creator(tx: &Tx<'_>, record: &InitialAccess) -> Result<()> {
    if record.creator_agent {
        let agent = tx
            .get::<crate::agent::Agent>(
                "agents",
                record
                    .created_by
                    .strip_prefix("agent:")
                    .unwrap_or(&record.created_by),
            )?
            .ok_or_else(Error::unauthorized)?;
        if !crate::agent::authority_active(tx, &agent)? {
            return Err(Error::unauthorized());
        }
        let actor = crate::agent::Principal {
            id: record.created_by.clone(),
            agent: true,
            permissions: agent.permissions,
        };
        actor.require("client.write", "*")?;
        actor.require(
            "registration.write",
            &format!("registration/{}", record.template.id),
        )?;
    } else {
        tx.get::<crate::model::User>("users", &record.created_by)?
            .filter(|u| u.enabled && u.admin)
            .ok_or_else(Error::unauthorized)?;
    }
    Ok(())
}
