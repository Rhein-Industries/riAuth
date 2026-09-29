//! Complete a source login through the concrete one-use storage transaction.

use crate::{
    core::Core,
    crypto::{digest, now},
    error::{Error, Result},
    source::{Finish, Login},
};
use serde_json::Value;

impl Core {
    pub fn source_finish(&self, input: Finish) -> Result<Value> {
        let credential = zeroize::Zeroizing::new(input.credential);
        self.store.write(|tx| {
            let state = tx
                .get::<String>("source_polls", &digest(&credential))?
                .ok_or_else(Error::unauthorized)?;
            let mut pending = tx
                .get::<Login>("source_logins", &state)?
                .filter(|p| p.expires_at > now() && !p.failed && p.attempts < 5)
                .ok_or_else(Error::unauthorized)?;
            if pending.stage.is_some() || pending.workflow.is_some() {
                return Err(Error::bad(
                    "Resume the bound source stage or workflow for this login",
                ));
            }
            self.complete_source_login(
                tx,
                &state,
                &mut pending,
                input.approve,
                input.otp.as_deref(),
                None,
            )
        })?
    }
}
