//! Concrete storage capability for a workflow-bound upstream login.
//!
//! The executor supplies its existing transaction. The source adapter decides
//! whether evidence is valid; this port keeps its reads and ordered writes in
//! that same transaction, including one-use browser and poll cleanup.

use crate::{
    error::Result,
    source::{Link, Login, saml::UpstreamSession},
    store::Tx,
};

pub(crate) struct SourceWorkflowTx<'a, 'db> {
    tx: &'a Tx<'db>,
}

impl<'a, 'db> SourceWorkflowTx<'a, 'db> {
    pub(crate) fn new(tx: &'a Tx<'db>) -> Self {
        Self { tx }
    }

    pub(crate) fn link(&self, key: &str) -> Result<Option<Link>> {
        self.tx.get("source_links", key)
    }

    pub(crate) fn login(&self, key: &str) -> Result<Option<Login>> {
        self.tx.get("source_logins", key)
    }

    pub(crate) fn upstream_session(&self, session_id: &str) -> Result<Option<UpstreamSession>> {
        self.tx.get("saml_source_sessions", session_id)
    }

    pub(crate) fn bind_login(&self, key: &str, login: &Login) -> Result<()> {
        self.tx.put("source_logins", key, login)?;
        // The ordinary CLI completion credential is neither returned nor usable.
        self.tx.delete("source_polls", &login.poll_hash)
    }

    pub(crate) fn discard_login(&self, key: &str, login: &Login) -> Result<()> {
        super::clear_browser_return(self.tx, login)?;
        self.tx.delete("source_polls", &login.poll_hash)?;
        self.tx.delete("source_logins", key)
    }
}
