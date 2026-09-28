//! One consent-withdrawal transaction for bearer, CLI and browser self-service.

use crate::{
    browser::{consent_key, consents_for_user},
    core::{Core, audit},
    crypto::digest,
    error::{Error, Result},
    portal::self_service::Binding,
    store::Tx,
};
use serde_json::{Value, json};

pub(crate) enum ConsentWithdraw<'a> {
    Bearer {
        token: &'a str,
    },
    Browser {
        cookie: Option<&'a str>,
        binding: &'a Binding,
    },
}

struct ConsentReceipt {
    key: String,
    fingerprint: String,
    scope: Value,
}

impl ConsentReceipt {
    fn current(channel: &str, user_id: &str, session_id: &str, client_id: &str) -> Option<Self> {
        let context = crate::context::current()?;
        let key = context.idempotency_key?;
        Some(Self {
            key: digest(&format!(
                "consent.withdraw\0{channel}\0{user_id}\0{session_id}\0{key}"
            )),
            fingerprint: context.fingerprint,
            scope: json!({"self_service":"consent.withdraw","channel":channel,"user_id":user_id,"session_id":session_id,"client_id":client_id}),
        })
    }

    fn replay(&self, tx: &Tx<'_>) -> Result<Option<Value>> {
        crate::context::replay_receipt(tx, &self.key, &self.fingerprint, &self.scope)
    }

    fn save(self, tx: &Tx<'_>, result: &Value) -> Result<()> {
        crate::context::save_receipt(tx, &self.key, self.fingerprint, self.scope, result)
    }
}

/// Browser account/session binding and fresh-factor checks run before replay.
/// The bearer path deliberately retains its broader target semantics: it may
/// revoke outstanding grants for this caller and client without a visible
/// remembered-consent row. Neither path can act for a different account.
pub(crate) fn withdraw_consent(
    core: &Core,
    tx: &Tx<'_>,
    authority: ConsentWithdraw<'_>,
    client_id: &str,
) -> Result<Value> {
    let (user, session, browser) = match authority {
        ConsentWithdraw::Bearer { token } => {
            let (user, session) = core.session(tx, token)?;
            (user, session, false)
        }
        ConsentWithdraw::Browser { cookie, binding } => {
            let (user, session) = core.verified_browser(tx, cookie, binding)?;
            (user, session, true)
        }
    };
    let receipt = ConsentReceipt::current(
        if browser { "browser" } else { "bearer" },
        &user.id,
        &session.id,
        client_id,
    );
    if let Some(result) = receipt
        .as_ref()
        .map(|receipt| receipt.replay(tx))
        .transpose()?
        .flatten()
    {
        return Ok(result);
    }
    if browser
        && !consents_for_user(tx, &user.id)?
            .iter()
            .any(|consent| consent["client_id"] == client_id)
    {
        return Err(Error::missing("Remembered consent not found"));
    }
    revoke_consent_for_user(tx, &user.id, client_id)?;
    let result = if browser {
        json!({"withdrawn":true,"client_id":client_id})
    } else {
        json!({"revoked":true,"client_id":client_id})
    };
    if let Some(receipt) = receipt {
        receipt.save(tx, &result)?;
    }
    Ok(result)
}

/// Remove approval and all dependent authorization in the same transaction,
/// then record the one withdrawal audit before saving any replay receipt.
fn revoke_consent_for_user(tx: &Tx<'_>, user_id: &str, cid: &str) -> Result<()> {
    tx.delete("consents", &consent_key(user_id, cid))?;
    crate::saml::revoke_consent(tx, user_id, cid)?;
    crate::outpost::revoke_sessions(tx, user_id, cid)?;
    for (_, grant) in tx
        .list::<crate::model::Grant>("access")?
        .into_iter()
        .chain(tx.list("refresh")?)
    {
        if grant.client_id == cid
            && grant.identity.is_some_and(|i| i.user_id == user_id)
            && let Some(mut family) =
                tx.get::<crate::model::Family>("families", &grant.family_id)?
        {
            family.revoked = true;
            tx.put("families", &grant.family_id, &family)?;
        }
    }
    // Unredeemed authorizations for this user/client must not resurrect consent.
    for (id, code) in tx.list::<crate::model::Code>("codes")? {
        if code.client_id == cid && code.identity.user_id == user_id {
            tx.delete("codes", &id)?;
        }
    }
    for (id, mut device) in tx.list::<crate::model::Device>("devices")? {
        if device.client_id == cid
            && matches!(&device.status, crate::model::DeviceStatus::Approved(i) if i.user_id == user_id)
        {
            device.status = crate::model::DeviceStatus::Denied;
            tx.put("devices", &id, &device)?;
        }
    }
    crate::logout::queue_user_client(tx, user_id, cid)?;
    crate::ssf::enqueue(tx, user_id, crate::ssf::SESSION_REVOKED, "")?;
    audit(tx, user_id, "consent.revoke", cid)
}
