//! One proof-bound device decision writer for bearer, CLI and browser callers.

use crate::{
    core::{Core, audit},
    crypto::{self, digest},
    error::{Error, Result},
    model::{Client, Device, DeviceStatus, Session},
    oidc::{device_authentication_stale, get_client, lookup_device},
    store::Tx,
};
use axum::http::StatusCode;
use serde_json::{Value, json};

pub(crate) enum DeviceDecisionAuthority<'a> {
    Bearer {
        token: &'a str,
    },
    Browser {
        cookie: Option<&'a str>,
        session_ref: &'a str,
    },
}

struct DeviceReceipt {
    key: String,
    fingerprint: String,
    channel: &'static str,
    user_id: String,
    session_id: String,
    code_hash: String,
    approve: bool,
    review: Option<String>,
}

impl DeviceReceipt {
    fn current(
        channel: &'static str,
        session: &Session,
        user_code: &str,
        approve: bool,
        review: Option<&str>,
    ) -> Result<Option<Self>> {
        let Some(context) = crate::context::current() else {
            return Ok(None);
        };
        let Some(key) = context.idempotency_key else {
            return Ok(None);
        };
        Ok(Some(Self {
            key: digest(&format!(
                "device.decision\0{channel}\0{}\0{}\0{key}",
                session.identity.user_id, session.id
            )),
            fingerprint: context.fingerprint,
            channel,
            user_id: session.identity.user_id.clone(),
            session_id: session.id.clone(),
            code_hash: digest(&crypto::normalize_code(user_code)?),
            approve,
            review: review.map(str::to_owned),
        }))
    }

    fn scope(&self, device_key: &str) -> Value {
        json!({
            "self_service":"device.decision",
            "channel":self.channel,
            "user_id":self.user_id,
            "session_id":self.session_id,
            "code_hash":self.code_hash,
            "device_key":device_key,
            "approve":self.approve,
            "review":self.review,
        })
    }

    fn replay(&self, tx: &Tx<'_>, session: &Session) -> Result<Option<Value>> {
        let Some(saved) = tx.get::<crate::context::Receipt>("receipts", &self.key)? else {
            return Ok(None);
        };
        let device_key = saved.permissions["device_key"]
            .as_str()
            .ok_or_else(|| Error::conflict("Idempotency receipt does not match device request"))?;
        let result = crate::context::replay_receipt(
            tx,
            &self.key,
            &self.fingerprint,
            &self.scope(device_key),
        )?;
        // A consumed code has no mapping. If a later request reuses that user
        // code, its new device must never inherit this decision's receipt.
        if tx
            .get::<String>("device_users", &self.code_hash)?
            .is_some_and(|current| current != device_key)
        {
            return Err(Error::conflict(
                "Idempotency key was used for another device request",
            ));
        }
        if let Some(review) = &self.review {
            validate_review(review, device_key, session)?;
        }
        if self.approve {
            require_fresh_authentication(session)?;
        }
        Ok(result)
    }

    fn save(self, tx: &Tx<'_>, device_key: &str, result: &Value) -> Result<()> {
        let scope = self.scope(device_key);
        crate::context::save_receipt(tx, &self.key, self.fingerprint, scope, result)
    }
}

fn validate_review(review: &str, device_key: &str, session: &Session) -> Result<()> {
    if !crypto::constant_eq(review, &crate::signin::session_ref(device_key, &session.id)) {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "account_changed",
            "Review this device request again with the current account",
        ));
    }
    Ok(())
}

fn require_fresh_authentication(session: &Session) -> Result<()> {
    if device_authentication_stale(session) {
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "reauthentication_required",
            "Sign in again before approving this device request",
        ));
    }
    Ok(())
}

/// Revalidate the live caller and, for a new decision, the current application
/// inside the transaction. A browser must present the reference from its review;
/// the bearer/API/CLI path keeps its existing user-code authority. An exact
/// keyed retry may replay after redemption removes the device, but only with
/// the same live caller, review proof and request.
pub(crate) fn decide_device(
    core: &Core,
    tx: &Tx<'_>,
    authority: DeviceDecisionAuthority<'_>,
    user_code: &str,
    approve: bool,
) -> Result<Value> {
    let (session, review, channel) = match authority {
        DeviceDecisionAuthority::Bearer { token } => {
            let (_, session) = core.session(tx, token)?;
            (session, None, "bearer")
        }
        DeviceDecisionAuthority::Browser {
            cookie,
            session_ref,
        } => {
            let session = core
                .browser_session(tx, cookie)?
                .ok_or_else(Error::unauthorized)?;
            (session, Some(session_ref), "browser")
        }
    };
    let receipt = DeviceReceipt::current(channel, &session, user_code, approve, review)?;
    if let Some(result) = receipt
        .as_ref()
        .map(|receipt| receipt.replay(tx, &session))
        .transpose()?
        .flatten()
    {
        return Ok(result);
    }
    let (key, mut device) = lookup_device(tx, user_code)?;
    let pending = matches!(device.status, DeviceStatus::Pending);
    // New decisions always retain the one-use conflict before review or
    // application validation, whether or not the caller supplied a key.
    if !pending {
        return Err(Error::conflict("Device request already decided"));
    }
    if let Some(review) = review {
        validate_review(review, &key, &session)?;
    }
    let client = get_client(tx, &device.client_id)?;
    if approve {
        require_fresh_authentication(&session)?;
        device_approval_policy(core, tx, &client, &device, &session)?;
    }
    device.status = if approve {
        DeviceStatus::Approved(session.identity.clone())
    } else {
        DeviceStatus::Denied
    };
    tx.put("devices", &key, &device)?;
    audit(
        tx,
        &session.identity.user_id,
        if approve {
            "device.approved"
        } else {
            "device.denied"
        },
        &device.client_id,
    )?;
    let result = json!({"approved": approve});
    if let Some(receipt) = receipt {
        receipt.save(tx, &key, &result)?;
    }
    Ok(result)
}

/// The same live policy check is used for the browser's review and the write.
pub(crate) fn device_approval_policy(
    core: &Core,
    tx: &Tx<'_>,
    client: &Client,
    device: &Device,
    session: &Session,
) -> Result<()> {
    let user = core.authorize_identity(tx, client, &session.identity)?;
    if !device.scopes.is_subset(&client.scopes) {
        return Err(Error::forbidden());
    }
    crate::claims::enforce(tx, client, &user, &session.identity, &device.scopes)?;
    let claims = crate::claims::mapped_claims_for_identity(
        tx,
        &user,
        client,
        &device.scopes,
        &session.identity,
    )?;
    crate::assurance::enforce(
        client,
        &session.identity,
        None,
        &Default::default(),
        &claims,
    )
}
