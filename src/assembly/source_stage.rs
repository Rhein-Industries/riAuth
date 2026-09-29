//! Embedded source stage transaction entrypoints over concrete Core storage.

use crate::{
    core::Core, crypto::digest, error::Result, model::AuthenticationTransaction,
    source::SourceStage, store::Tx,
};
use serde_json::Value;

impl Core {
    /// This write precedes the login-to-stage binding check in the caller. A failed
    /// check rolls it back together with the already reserved source login.
    pub(crate) fn persist_source_stage_authentication(
        &self,
        tx: &Tx<'_>,
        transaction: &str,
        pending: &AuthenticationTransaction,
    ) -> Result<()> {
        tx.put("authentication", &digest(transaction), pending)
    }

    pub(crate) fn persist_source_stage_binding(
        &self,
        tx: &Tx<'_>,
        stage: &SourceStage,
        suspension: &str,
    ) -> Result<()> {
        tx.put("source_stages", &stage.id, stage)?;
        tx.put("source_stage_requests", suspension, &stage.id)
    }

    pub(crate) fn discard_stage_resume_bearer(&self, tx: &Tx<'_>, token: &str) -> Result<()> {
        tx.delete("session_tokens", &digest(token))
    }

    pub(crate) fn persist_stage_resume_use(&self, tx: &Tx<'_>, stage: &SourceStage) -> Result<()> {
        tx.put("source_stages", &stage.id, stage)
    }

    pub fn source_stage_resume(
        &self,
        stage_id: &str,
        authorization_id: &str,
        otp: Option<String>,
    ) -> Result<Value> {
        // An inner failure can charge a wrong local code in the same transaction.
        self.store
            .write(|tx| self.resume_stage(tx, stage_id, authorization_id, otp.as_deref()))?
    }

    pub fn source_stage_cancel(&self, stage_id: &str, authorization_id: &str) -> Result<Value> {
        self.store
            .write(|tx| self.cancel_stage(tx, stage_id, authorization_id))
    }
}
