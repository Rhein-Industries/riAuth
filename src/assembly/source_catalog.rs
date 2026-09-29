//! Authorized source catalog read over concrete storage.

use crate::{
    core::Core,
    crypto::digest,
    error::Result,
    source::{Link, Login, Source, SourceInput, Start},
    store::Tx,
};
use serde_json::{Value, json};

/// The upstream accounts linked to a user, in storage iteration order.
pub(crate) fn source_links_of(tx: &Tx<'_>, user_id: &str) -> Result<Vec<Value>> {
    Ok(tx
        .list::<Link>("source_links")?
        .into_iter()
        .filter(|(_, l)| l.user_id == user_id)
        .map(|(id, l)| json!({"id":id,"source":l.source,"issuer":l.issuer,"subject":l.subject}))
        .collect())
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
