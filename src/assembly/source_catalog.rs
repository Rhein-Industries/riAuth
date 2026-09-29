//! Authorized source catalog read over concrete storage.

use crate::{
    core::Core,
    error::Result,
    source::{Source, SourceInput, Start},
};
use serde_json::{Value, json};

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

    pub fn source_unlink(&self, token: &str, link_id: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            crate::management::unlink_source(tx, &user, &session, link_id)
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
