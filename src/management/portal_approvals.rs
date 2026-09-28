//! Terminal approval or denial of a browser-bound portal sign-in request.

use crate::{
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::Session,
    portal::{Pending, pending_by_code},
    store::Tx,
};
use serde_json::{Value, json};

struct PortalDecisionReceipt {
    key: String,
    fingerprint: String,
    user_id: String,
    session_id: String,
    code_hash: String,
    approve: bool,
}

impl PortalDecisionReceipt {
    fn current(session: &Session, code: &str, approve: bool) -> Result<Option<Self>> {
        let Some(context) = crate::context::current() else {
            return Ok(None);
        };
        let Some(key) = context.idempotency_key else {
            return Ok(None);
        };
        Ok(Some(Self {
            key: digest(&format!(
                "portal.sign_in.decision\0{}\0{}\0{key}",
                session.identity.user_id, session.id
            )),
            fingerprint: context.fingerprint,
            user_id: session.identity.user_id.clone(),
            session_id: session.id.clone(),
            code_hash: digest(&crypto::normalize_code(code)?),
            approve,
        }))
    }

    fn scope(&self, pending_id: &str, binding_hash: &str) -> Value {
        json!({
            "self_service":"portal.sign_in.decision",
            "user_id":self.user_id,
            "session_id":self.session_id,
            "code_hash":self.code_hash,
            "request_id":pending_id,
            "browser_binding_hash":binding_hash,
            "approve":self.approve,
        })
    }

    fn replay(&self, tx: &Tx<'_>, session: &Session) -> Result<Option<Value>> {
        let Some(saved) = tx.get::<crate::context::Receipt>("receipts", &self.key)? else {
            return Ok(None);
        };
        let request_id = saved.permissions["request_id"]
            .as_str()
            .ok_or_else(|| Error::conflict("Idempotency receipt does not match sign-in request"))?;
        let binding_hash = saved.permissions["browser_binding_hash"]
            .as_str()
            .ok_or_else(|| Error::conflict("Idempotency receipt does not match sign-in request"))?;
        let result = crate::context::replay_receipt(
            tx,
            &self.key,
            &self.fingerprint,
            &self.scope(request_id, binding_hash),
        )?;
        if self.approve {
            require_fresh_approver(session)?;
        }
        // Poll and cancel remove both rows. If a later browser obtains this
        // code, its distinct request and delivery binding cannot use this receipt.
        if let Some(current_id) = tx.get::<String>("portal_codes", &self.code_hash)? {
            if current_id != request_id {
                return Err(Error::conflict(
                    "Idempotency key was used for another sign-in request",
                ));
            }
            let current = tx
                .get::<Pending>("portal_requests", request_id)?
                .ok_or_else(|| Error::conflict("Sign-in request changed after decision"))?;
            validate_binding(&current, &self.code_hash)?;
            if current.binding_hash != binding_hash
                || (self.approve
                    && (current.denied
                        || current.session_id.as_deref() != Some(self.session_id.as_str())))
                || (!self.approve && (!current.denied || current.session_id.is_some()))
            {
                return Err(Error::conflict("Sign-in request changed after decision"));
            }
        }
        Ok(result)
    }

    fn save(self, tx: &Tx<'_>, pending: &Pending, result: &Value) -> Result<()> {
        let scope = self.scope(&pending.id, &pending.binding_hash);
        crate::context::save_receipt(tx, &self.key, self.fingerprint, scope, result)
    }
}

fn require_fresh_approver(session: &Session) -> Result<()> {
    if now().saturating_sub(session.identity.auth_time) > 300 {
        return Err(Error::forbidden());
    }
    Ok(())
}

fn validate_binding(pending: &Pending, code_hash: &str) -> Result<()> {
    // `digest` is an unpadded base64url SHA-256 value, including for the
    // original browser's random delivery cookie.
    if digest(&crypto::normalize_code(&pending.code)?) != code_hash
        || pending.binding_hash.len() != 43
        || !pending
            .binding_hash
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(Error::conflict("Sign-in request binding changed"));
    }
    Ok(())
}

/// The approving terminal supplies only its live bearer session and displayed
/// code. The writer binds that decision to the request's original browser-cookie
/// hash; `portal_poll_with` still requires the cookie before delivering the
/// terminal session. A receipt acknowledges only this committed decision.
pub(crate) fn decide_portal_sign_in(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    code: &str,
    approve: bool,
) -> Result<Value> {
    let (user, session) = core.session(tx, token)?;
    let receipt = PortalDecisionReceipt::current(&session, code, approve)?;
    if let Some(result) = receipt
        .as_ref()
        .map(|receipt| receipt.replay(tx, &session))
        .transpose()?
        .flatten()
    {
        return Ok(result);
    }
    let mut pending = pending_by_code(tx, code)?;
    if pending.session_id.is_some() || pending.denied {
        return Err(Error::conflict("Request already decided"));
    }
    validate_binding(&pending, &digest(&crypto::normalize_code(code)?))?;
    if approve {
        require_fresh_approver(&session)?;
    }
    pending.session_id = approve.then_some(session.id);
    pending.denied = !approve;
    tx.put("portal_requests", &pending.id, &pending)?;
    audit(
        tx,
        &user.id,
        if approve {
            "portal.sign_in.approve"
        } else {
            "portal.sign_in.deny"
        },
        &pending.id,
    )?;
    let result = json!({"approved":approve,"delivery":"original_browser"});
    if let Some(receipt) = receipt {
        receipt.save(tx, &pending, &result)?;
    }
    Ok(result)
}
