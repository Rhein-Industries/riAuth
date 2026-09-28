//! RFC 7591 registration using bounded, revocable initial access tokens.
use crate::{
    core::{Core, audit, validate_client, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    jose::PublicJwks,
    model::{Client, ProviderSettings},
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
struct InitialAccess {
    template: RegistrationTemplate,
    token_hash: String,
    created_by: String,
    creator_agent: bool,
    expires_at: u64,
    used: u32,
    enabled: bool,
}
impl InitialAccess {
    fn view(&self) -> Value {
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
                r.template.id == id
                    && r.token_hash == hash
                    && r.enabled
                    && r.expires_at > now()
                    && r.used < r.template.max_uses
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

    /// Persist use in the same transaction as the client and its audit event.
    pub(crate) fn consume(&mut self, tx: &Tx<'_>) -> Result<()> {
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
            let actor = self.management(
                tx,
                token,
                "registration.write",
                &format!("registration/{}", template.id),
            )?;
            actor.require("client.write", "*")?;
            validate_name(&template.id)?;
            if !(60..=2_592_000).contains(&template.ttl)
                || !(1..=1000).contains(&template.max_uses)
                || template.redirect_uris.is_empty()
                || template.auth_methods.is_empty()
                || template.auth_methods.iter().any(|m| {
                    ![
                        "none",
                        "client_secret_basic",
                        "client_secret_post",
                        "private_key_jwt",
                    ]
                    .contains(&m.as_str())
                })
                || template.grant_types.is_empty()
                || template.grant_types.iter().any(|g| {
                    ![
                        "authorization_code",
                        "refresh_token",
                        crate::oidc::DEVICE_GRANT,
                    ]
                    .contains(&g.as_str())
                })
            {
                return Err(Error::bad(
                    "Invalid registration template limits, grants or authentication methods",
                ));
            }
            // Registration cannot create machine trusts, exchange permissions or inherit keys.
            if template.settings.exchange.is_some()
                || !template.settings.exchange_from.is_empty()
                || !template.settings.machine_trust.is_empty()
                || template.settings.jwks.is_some()
                || template.settings.token_endpoint_auth_method.is_some()
            {
                return Err(Error::bad(
                    "Registration templates cannot delegate machine/exchange trust or client keys",
                ));
            }
            if template.settings.implicit_consent {
                return Err(Error::bad(
                    "Registration templates cannot skip browser consent",
                ));
            }
            let mut sample = Client {
                id: "registration-validation".into(),
                name: "Registration validation".into(),
                secret_hash: None,
                redirect_uris: template.redirect_uris.clone(),
                scopes: template.scopes.clone(),
                allowed_groups: template.allowed_groups.clone(),
                require_mfa: template.require_mfa,
                enabled: true,
                service: false,
                settings: template.settings.clone(),
            };
            sample.settings.allowed_grants = template.grant_types.clone();
            validate_client(tx, &sample)?;
            if tx
                .get::<InitialAccess>("registrations", &template.id)?
                .is_some()
            {
                return Err(Error::conflict("Registration template already exists"));
            }
            let credential = crypto::random_token("ri_register_");
            let record = InitialAccess {
                token_hash: digest(&credential),
                created_by: actor.id.clone(),
                creator_agent: actor.agent,
                expires_at: now() + template.ttl,
                used: 0,
                enabled: true,
                template,
            };
            tx.put("registrations", &record.template.id, &record)?;
            tx.put(
                "registration_tokens",
                &record.token_hash,
                &record.template.id,
            )?;
            audit(tx, &actor.id, "registration.create", &record.template.id)?;
            Ok(json!({"registration": record.view(), "initial_access_token": credential}))
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
            let actor = self.management(
                tx,
                token,
                "registration.write",
                &format!("registration/{id}"),
            )?;
            let mut record = tx
                .get::<InitialAccess>("registrations", id)?
                .ok_or_else(|| Error::missing("Template not found"))?;
            record.enabled = false;
            tx.put("registrations", id, &record)?;
            audit(tx, &actor.id, "registration.revoke", id)?;
            Ok(record.view())
        })
    }
    pub fn dynamic_register(
        &self,
        initial_token: &str,
        request: RegistrationRequest,
    ) -> Result<Value> {
        self.store
            .write(|tx| crate::management::register_client(tx, initial_token, request))
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
