//! Embedded source stage transaction entrypoints over concrete Core storage.

use crate::{
    core::Core,
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{AuthenticationTransaction, Session},
    source::{Login, SourceStage, suspension_hash},
    store::Tx,
};
use serde_json::Value;

pub(crate) fn ensure_stage_request_available(tx: &Tx<'_>, suspension: &str) -> Result<()> {
    if let Some(existing) = tx.get::<String>("source_stage_requests", suspension)?
        && tx
            .get::<SourceStage>("source_stages", &existing)?
            .is_some_and(|stage| !stage.used && !stage.cancelled && stage.expires_at > now())
    {
        return Err(Error::conflict(
            "An embedded source stage is already pending for this authorization request",
        ));
    }
    Ok(())
}

pub(crate) fn enforce_pending_stage(
    tx: &Tx<'_>,
    request: &crate::oidc::Authorization,
    session: &Session,
) -> Result<()> {
    let key = suspension_hash(request)?;
    let Some(id) = tx.get::<String>("source_stage_requests", &key)? else {
        return Ok(());
    };
    let Some(stage) = tx.get::<SourceStage>("source_stages", &id)? else {
        return Ok(());
    };
    if stage.expires_at <= now()
        || stage.suspension_hash != key
        || stage.request.client_id != request.client_id
    {
        return Ok(());
    }
    if stage.cancelled {
        return Err(Error::oauth(
            "access_denied",
            "The source stage was cancelled",
        ));
    }
    if !stage.used {
        return Err(Error::oauth(
            "login_required",
            "Complete the embedded source stage",
        ));
    }
    let hash = request.request_hash()?;
    let Some(token) = request.transaction_id.as_deref() else {
        return Err(Error::oauth(
            "login_required",
            "Complete the embedded source stage",
        ));
    };
    if !crypto::constant_eq(token, &stage.transaction) {
        return Err(Error::oauth(
            "login_required",
            "Complete the embedded source stage",
        ));
    }
    tx.get::<AuthenticationTransaction>("authentication", &digest(token))?
        .filter(|record| {
            record.expires_at > now()
                && record.source_stage.as_deref() == Some(stage.id.as_str())
                && record.request_hash == hash
                && record.authenticated_session.as_deref() == Some(session.id.as_str())
        })
        .ok_or_else(|| Error::oauth("login_required", "Complete the embedded source stage"))?;
    Ok(())
}

pub(crate) fn stage_resume_session(tx: &Tx<'_>, token: &str) -> Result<Session> {
    let sid = tx
        .get::<String>("session_tokens", &digest(token))?
        .ok_or_else(|| Error::internal("missing session"))?;
    tx.get::<Session>("sessions", &sid)?
        .ok_or_else(|| Error::internal("missing session"))
}

pub(crate) fn verify_stage_start_login(tx: &Tx<'_>, stage: &SourceStage) -> Result<()> {
    let login = tx
        .get::<Login>("source_logins", &stage.login_key)?
        .ok_or_else(|| Error::internal("source login missing"))?;
    if login.nonce != stage.nonce
        || login.stage.as_deref() != Some(stage.id.as_str())
        || login.source != stage.source_id
        || login.expires_at > now() + 600
    {
        return Err(Error::internal("source stage binding failed"));
    }
    Ok(())
}

pub(crate) fn stage_resume_login(tx: &Tx<'_>, stage: &SourceStage) -> Result<Login> {
    tx.get::<Login>("source_logins", &stage.login_key)?
        .filter(|login| {
            login.stage.as_deref() == Some(stage.id.as_str())
                && login.nonce == stage.nonce
                && login.source == stage.source_id
                && login.expires_at > now()
                && !login.failed
        })
        .ok_or_else(|| Error::bad("Source stage login expired or is not bound"))
}

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

    pub(crate) fn persist_stage_rejection(&self, tx: &Tx<'_>, stage: &SourceStage) -> Result<()> {
        tx.put("source_stages", &stage.id, stage)?;
        if let Some(mut pending) = tx.get::<Login>("source_logins", &stage.login_key)? {
            pending.failed = true;
            tx.delete("source_polls", &pending.poll_hash)?;
            tx.put("source_logins", &stage.login_key, &pending)?;
        }
        Ok(())
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

pub(crate) fn cleanup_expired_source_state(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, pending) in tx.maintenance_page::<Login>("source_logins")? {
        if pending.expires_at < at {
            super::clear_browser_return(tx, &pending)?;
            tx.delete("source_polls", &pending.poll_hash)?;
            tx.delete("source_logins", &id)?;
        }
    }
    for (id, stage) in tx.maintenance_page::<SourceStage>("source_stages")? {
        if stage.expires_at < at {
            if tx
                .get::<String>("source_stage_requests", &stage.suspension_hash)?
                .as_deref()
                == Some(id.as_str())
            {
                tx.delete("source_stage_requests", &stage.suspension_hash)?;
            }
            tx.delete("source_stages", &id)?;
        }
    }
    Ok(())
}
