//! Transaction writers for Platform temporary access.

use crate::{
    core::{Core, audit},
    crypto::{digest, id, now},
    error::{Error, Result},
    model::{Group, Session, User},
    pam::{AccessGrant, AccessRequest, NewAccessRequest, RETAIN_SECONDS},
    store::Tx,
};
use serde_json::{Value, json};

const PENDING: &str = "pending";
const APPROVED: &str = "approved";
const DENIED: &str = "denied";
const MAX_PENDING: usize = 1_000;

struct AccessReceipt {
    key: String,
    fingerprint: String,
    scope: Value,
}

impl AccessReceipt {
    fn current(
        user: &User,
        session: &Session,
        channel: &str,
        credential: &str,
        intent: Value,
    ) -> Option<Self> {
        let context = crate::context::current()?;
        let key = context.idempotency_key?;
        let credential_hash = digest(credential);
        Some(Self {
            key: digest(&format!(
                "access.mutation\0{channel}\0{}\0{}\0{credential_hash}\0{key}",
                user.id, session.id,
            )),
            fingerprint: context.fingerprint,
            scope: json!({
                "self_service":"access.mutation",
                "channel":channel,
                "user_id":user.id,
                "session_id":session.id,
                "credential_hash":credential_hash,
                "revision":context.revision,
                "intent":intent,
            }),
        })
    }

    fn replay(&self, tx: &Tx<'_>) -> Result<Option<Value>> {
        crate::context::replay_receipt(tx, &self.key, &self.fingerprint, &self.scope)
    }

    fn save(self, tx: &Tx<'_>, result: &Value) -> Result<()> {
        crate::context::save_receipt(tx, &self.key, self.fingerprint, self.scope, result)
    }
}

fn require_revision(tx: &Tx<'_>) -> Result<()> {
    if let Some(expected) = crate::context::current().and_then(|context| context.revision)
        && expected != tx.get::<u64>("meta", "revision")?.unwrap_or(0)
    {
        return Err(Error::conflict("Configuration revision changed"));
    }
    Ok(())
}

pub(crate) fn validate_access_request(input: &NewAccessRequest) -> Result<()> {
    crate::pam::validate_request(input)
}

pub(crate) fn request_access(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    input: NewAccessRequest,
) -> Result<Value> {
    validate_access_request(&input)?;
    let (actor, session, channel) = core.pam_actor(tx, token)?;
    if !actor.enabled {
        return Err(Error::bad("Requester is disabled"));
    }
    if tx.get::<Group>("groups", &input.group)?.is_none() {
        return Err(Error::bad("Unknown group"));
    }
    if !core.config.pam_approvers.contains_key(&input.group) {
        return Err(Error::bad("No approver rule for this group"));
    }
    let receipt = AccessReceipt::current(
        &actor,
        &session,
        channel,
        token,
        json!({"operation":"request","group":input.group,"reason_hash":digest(&input.reason),"ttl":input.ttl}),
    );
    if let Some(result) = receipt
        .as_ref()
        .map(|receipt| receipt.replay(tx))
        .transpose()?
        .flatten()
    {
        return Ok(result);
    }
    require_revision(tx)?;
    let pending = tx
        .list::<AccessRequest>("access_requests")?
        .into_iter()
        .filter(|(_, request)| request.status == PENDING)
        .count();
    if pending >= MAX_PENDING {
        return Err(Error::conflict("Too many pending access requests"));
    }
    let request = AccessRequest {
        id: id(),
        user_id: actor.id.clone(),
        username: actor.username.clone(),
        group: input.group,
        reason: input.reason,
        ttl: input.ttl,
        status: PENDING.into(),
        created_at: now(),
        decided_at: None,
        decided_by: None,
        grant_id: None,
    };
    tx.put("access_requests", &request.id, &request)?;
    audit(tx, &actor.id, "access.request", &request.id)?;
    let result = json!(request);
    if let Some(receipt) = receipt {
        receipt.save(tx, &result)?;
    }
    Ok(result)
}

fn require_decision_authority(
    core: &Core,
    tx: &Tx<'_>,
    actor: &User,
    request: &AccessRequest,
) -> Result<()> {
    let approvers = core
        .config
        .pam_approvers
        .get(&request.group)
        .ok_or_else(|| Error::bad("No approver rule for this group"))?;
    if tx.get::<Group>("groups", &request.group)?.is_none() {
        return Err(Error::bad("Unknown group"));
    }
    let requester = tx
        .get::<User>("users", &request.user_id)?
        .ok_or_else(|| Error::missing("Requester not found"))?;
    if !requester.enabled {
        return Err(Error::bad("Requester is disabled"));
    }
    if actor.id == requester.id || !approvers.contains(&actor.username) {
        return Err(Error::forbidden());
    }
    Ok(())
}

pub(crate) fn decide_access(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    id: &str,
    approve: bool,
) -> Result<Value> {
    let (actor, session, channel) = core.pam_actor(tx, token)?;
    let mut request = tx
        .get::<AccessRequest>("access_requests", id)?
        .ok_or_else(|| Error::missing("Access request not found"))?;
    let receipt = AccessReceipt::current(
        &actor,
        &session,
        channel,
        token,
        json!({"operation":"decide","request_id":id,"approve":approve}),
    );
    // A decided request must remain replayable only to a still-authorized
    // approver holding the same live session and exact request key.
    if let Some(receipt) = &receipt {
        require_decision_authority(core, tx, &actor, &request)?;
        if let Some(result) = receipt.replay(tx)? {
            return Ok(result);
        }
    }
    if request.status != PENDING {
        return Err(Error::conflict("Access request is already decided"));
    }
    if receipt.is_none() {
        require_decision_authority(core, tx, &actor, &request)?;
    }
    if request.created_at.saturating_add(RETAIN_SECONDS) <= now() {
        return Err(Error::missing("Access request expired"));
    }
    require_revision(tx)?;
    // A third-party operator may already control credentials exposed before
    // this request. Do not confer a temporary group to that known credential.
    if approve && crate::delegation::credential_exposure(tx, &request.user_id)?.is_some() {
        return Err(Error::conflict(
            "Requester needs independent credential recovery before temporary access approval",
        ));
    }
    let at = now();
    request.decided_at = Some(at);
    request.decided_by = Some(actor.id.clone());
    let result = if approve {
        let grant = AccessGrant {
            id: crate::crypto::id(),
            user_id: request.user_id.clone(),
            group: request.group.clone(),
            not_before: at,
            expires_at: at.saturating_add(request.ttl),
            request_id: request.id.clone(),
            revoked_at: None,
            revoked_by: None,
        };
        tx.put("access_grants", &grant.id, &grant)?;
        request.status = APPROVED.into();
        request.grant_id = Some(grant.id.clone());
        tx.put("access_requests", id, &request)?;
        audit(tx, &actor.id, "access.approve", id)?;
        json!({"request":request,"grant":grant})
    } else {
        request.status = DENIED.into();
        tx.put("access_requests", id, &request)?;
        audit(tx, &actor.id, "access.deny", id)?;
        json!({"request":request})
    };
    if let Some(receipt) = receipt {
        receipt.save(tx, &result)?;
    }
    Ok(result)
}

fn require_revoke_authority(core: &Core, actor: &User, grant: &AccessGrant) -> Result<()> {
    let approver = core
        .config
        .pam_approvers
        .get(&grant.group)
        .is_some_and(|names| names.contains(&actor.username));
    if !actor.admin && !approver {
        return Err(Error::forbidden());
    }
    Ok(())
}

/// A separate browser review surface for configured human approvers. Read
/// visibility follows the same live resource checks as the transaction
/// writers, so the page does not grant general management read authority.
pub(crate) fn review_access(core: &Core, tx: &Tx<'_>, token: &str) -> Result<Value> {
    let (actor, _, _) = core.pam_actor(tx, token)?;
    let configured = core
        .config
        .pam_approvers
        .values()
        .any(|names| names.contains(&actor.username));
    if !actor.admin && !configured {
        return Err(Error::forbidden());
    }
    let at = now();
    let mut requests = Vec::new();
    for (_, request) in tx.list::<AccessRequest>("access_requests")? {
        if request.status != PENDING || request.created_at.saturating_add(RETAIN_SECONDS) <= at {
            continue;
        }
        match require_decision_authority(core, tx, &actor, &request) {
            Ok(()) => requests.push(request),
            Err(error) if error.status.is_server_error() => return Err(error),
            Err(_) => {}
        }
    }
    requests.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut grants = Vec::new();
    for (_, grant) in tx.list::<AccessGrant>("access_grants")? {
        if grant.revoked_at.is_some() || grant.expires_at <= at {
            continue;
        }
        if require_revoke_authority(core, &actor, &grant).is_err() {
            continue;
        }
        let username = tx
            .get::<User>("users", &grant.user_id)?
            .map(|user| user.username)
            .unwrap_or_else(|| grant.user_id.clone());
        grants.push(json!({
            "id": grant.id,
            "username": username,
            "group": grant.group,
            "expires_at": grant.expires_at,
        }));
    }
    grants.sort_by(|left, right| {
        left["expires_at"]
            .as_u64()
            .cmp(&right["expires_at"].as_u64())
            .then_with(|| left["id"].as_str().cmp(&right["id"].as_str()))
    });
    Ok(json!({
        "user": {"id": actor.id, "username": actor.username},
        "revision": tx.get::<u64>("meta", "revision")?.unwrap_or(0),
        "requests": requests,
        "grants": grants,
    }))
}

pub(crate) fn revoke_access(core: &Core, tx: &Tx<'_>, token: &str, id: &str) -> Result<Value> {
    let (actor, session, channel) = core.pam_actor(tx, token)?;
    let mut grant = tx
        .get::<AccessGrant>("access_grants", id)?
        .ok_or_else(|| Error::missing("Access grant not found"))?;
    let receipt = AccessReceipt::current(
        &actor,
        &session,
        channel,
        token,
        json!({"operation":"revoke","grant_id":id}),
    );
    if let Some(receipt) = &receipt {
        require_revoke_authority(core, &actor, &grant)?;
        if let Some(result) = receipt.replay(tx)? {
            return Ok(result);
        }
    }
    if grant.revoked_at.is_some() {
        return Err(Error::conflict("Access grant is already revoked"));
    }
    if receipt.is_none() {
        require_revoke_authority(core, &actor, &grant)?;
    }
    // An expired grant confers no entitlement. Browser review already omits it;
    // bearer and CLI must not turn retention-only history into a new revocation.
    // Exact-key replay above still returns a revocation committed while live.
    if grant.expires_at <= now() {
        return Err(Error::conflict("Access grant has expired"));
    }
    require_revision(tx)?;
    grant.revoked_at = Some(now());
    grant.revoked_by = Some(actor.id.clone());
    tx.put("access_grants", id, &grant)?;
    audit(tx, &actor.id, "access.revoke", id)?;
    // Token validation and refresh exchange recompute temporary groups, so
    // this grant stops satisfying their group policy in the same transaction.
    let result = json!(grant);
    if let Some(receipt) = receipt {
        receipt.save(tx, &result)?;
    }
    Ok(result)
}
