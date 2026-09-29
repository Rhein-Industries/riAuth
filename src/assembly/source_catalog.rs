//! Authorized source catalog read over concrete storage.

use crate::{
    core::Core,
    error::Result,
    source::{Source, SourceInput},
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
