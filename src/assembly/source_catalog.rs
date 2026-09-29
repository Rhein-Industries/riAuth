//! Authorized source catalog read over concrete storage.

use crate::{
    agent::Principal,
    core::Core,
    crypto::digest,
    error::{Error, Result},
    model::{Group, User},
    source::{Link, LinkSpec, Login, Source, SourceInput, Start},
    store::Tx,
};
use serde_json::{Value, json};

pub(crate) struct SourceWritePrior {
    pub(crate) admin_login_was_allowed: bool,
    pub(crate) identity_binding_changed: bool,
}

pub(crate) fn source_write_prior(tx: &Tx<'_>, source: &Source) -> Result<SourceWritePrior> {
    let old = tx.get::<Source>("sources", &source.id)?;
    Ok(SourceWritePrior {
        admin_login_was_allowed: old.as_ref().is_some_and(|stored| stored.allow_admin_login),
        identity_binding_changed: old.as_ref().is_some_and(|stored| {
            stored.issuer != source.issuer
                || stored.client_id != source.client_id
                || stored.oauth_profile != source.oauth_profile
                || stored
                    .saml
                    .as_ref()
                    .map(|settings| &settings.name_id_format)
                    != source
                        .saml
                        .as_ref()
                        .map(|settings| &settings.name_id_format)
        }),
    })
}

pub(crate) fn require_source_group(tx: &Tx<'_>, group: &str) -> Result<()> {
    if tx.get::<Group>("groups", group)?.is_none() {
        return Err(Error::bad("Source references an unknown group"));
    }
    Ok(())
}

/// The upstream accounts linked to a user, in storage iteration order.
pub(crate) fn source_links_of(tx: &Tx<'_>, user_id: &str) -> Result<Vec<Value>> {
    Ok(tx
        .list::<Link>("source_links")?
        .into_iter()
        .filter(|(_, l)| l.user_id == user_id)
        .map(|(id, l)| json!({"id":id,"source":l.source,"issuer":l.issuer,"subject":l.subject}))
        .collect())
}

pub(crate) fn export_all_links(tx: &Tx<'_>) -> Result<Vec<LinkSpec>> {
    let mut output = Vec::new();
    for (_, link) in tx.list::<Link>("source_links")? {
        let user = tx
            .get::<User>("users", &link.user_id)?
            .ok_or_else(|| Error::internal("Linked user missing"))?;
        output.push(LinkSpec {
            source: link.source,
            subject: link.subject,
            username: user.username,
            issuer: Some(link.issuer),
        });
    }
    Ok(output)
}

pub(crate) fn export_links(tx: &Tx<'_>, actor: &Principal) -> Result<Vec<LinkSpec>> {
    Ok(export_all_links(tx)?
        .into_iter()
        .filter(|link| {
            actor.allows("source.read", &format!("source/{}", link.source))
                && actor.allows("user.read", &format!("user/{}", link.username))
        })
        .collect())
}

pub(crate) fn enabled_source(tx: &Tx<'_>, id: &str) -> Result<Source> {
    tx.get::<Source>("sources", id)?
        .filter(|source| source.enabled)
        .ok_or_else(|| Error::missing("Enabled source not found"))
}

impl Core {
    pub fn source_put(&self, token: &str, input: SourceInput) -> Result<Value> {
        let secret = input.client_secret.map(zeroize::Zeroizing::new);
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            crate::management::write_source(
                tx,
                &actor,
                &input.source,
                crate::management::SourceWrite::Direct {
                    secret: secret.as_deref().map(String::as_str),
                },
            )?;
            Ok(json!(input.source))
        })
    }

    pub fn source_start(&self, id: &str, input: Start, token: Option<&str>) -> Result<Value> {
        self.store.write(|tx| {
            self.source_start_in(tx, id, &input, token, None)
                .map(|started| started.body)
        })
    }

    /// Keep the callback state and its credential lookup in the caller's write transaction.
    /// Browser starts and embedded stages use the same reservation after their binding checks.
    pub(crate) fn persist_source_start(
        &self,
        tx: &Tx<'_>,
        state: &str,
        pending: &Login,
    ) -> Result<()> {
        tx.put("source_logins", &digest(state), pending)?;
        tx.put("source_polls", &pending.poll_hash, &digest(state))?;
        Ok(())
    }

    pub fn source_unlink(&self, token: &str, link_id: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            crate::management::unlink_source(tx, &user, &session, link_id)
        })
    }

    pub fn source_links(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let (user, _) = self.session(tx, token)?;
            Ok(json!(source_links_of(tx, &user.id)?))
        })
    }

    pub fn source_list(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(
                tx.list::<Source>("sources")?
                    .into_iter()
                    .filter(|(_, s)| actor.allows("source.read", &format!("source/{}", s.id)))
                    .map(|(_, s)| s)
                    .collect::<Vec<_>>()
            ))
        })
    }
}
