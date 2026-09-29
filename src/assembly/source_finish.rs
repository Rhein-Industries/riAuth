//! Complete a source login through the concrete one-use storage transaction.

use crate::{
    core::Core,
    crypto::{digest, now},
    error::{Error, Result},
    model::Identity,
    source::{Finish, Login},
    store::Tx,
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

    /// Finishes a browser's login in one transaction. `bind` first checks a link's target
    /// against the browser, and `deliver` then hands the new session to it. If either fails,
    /// the whole finish rolls back: the proof stays unspent and no link or session is written.
    /// A wrong local code still counts against the login's attempts, as for `source_finish`.
    pub(crate) fn source_finish_browser<T>(
        &self,
        credential: &str,
        otp: Option<&str>,
        bind: impl FnOnce(&Tx<'_>, Option<&Identity>) -> Result<()>,
        deliver: impl FnOnce(&Tx<'_>, bool, &Value) -> Result<T>,
    ) -> Result<T> {
        self.store.write(|tx| {
            let state = tx
                .get::<String>("source_polls", &digest(credential))?
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
            bind(tx, pending.target.as_ref())?;
            let linking = pending.target.is_some();
            match self.complete_source_login(tx, &state, &mut pending, true, otp, None)? {
                Ok(body) => deliver(tx, linking, &body).map(Ok),
                Err(error) => Ok(Err(error)),
            }
        })?
    }
}
