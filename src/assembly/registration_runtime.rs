//! Registration runtime assembled over the caller's concrete transaction.

use crate::{
    config::Config,
    core::Core,
    crypto::{digest, now},
    error::{Error, Result},
    registration::{InitialAccess, RegistrationRequest, RegistrationTemplate},
    store::Tx,
};
use serde_json::{Value, json};

/// Proof that an initial access token is live and its issuing authority still
/// holds registration rights. This is deliberately separate from an agent or
/// management principal: the token never acquires `client.write`.
pub(crate) struct RegistrationAuthority {
    record: InitialAccess,
}

impl RegistrationAuthority {
    pub(crate) fn for_token(tx: &Tx<'_>, config: &Config, token: &str) -> Result<Self> {
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
        check_creator(tx, config, &record)?;
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

impl Core {
    pub fn registration_template(
        &self,
        token: &str,
        template: RegistrationTemplate,
    ) -> Result<Value> {
        // The first response alone carries the initial access token; the
        // generic mutation receipt would persist and replay it.
        self.store.write(|tx| {
            crate::management::create_registration_template_issuing(self, tx, token, template)
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

fn check_creator(tx: &Tx<'_>, config: &Config, record: &InitialAccess) -> Result<()> {
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
        let actor =
            crate::agent::live_principal(tx, config, &agent)?.ok_or_else(Error::unauthorized)?;
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
