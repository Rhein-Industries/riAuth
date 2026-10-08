//! One transaction writer for bearer, agent and browser session revocation.
//! Account-management browser requests check binding and freshness before receipts.

use crate::{
    core::{Core, audit},
    crypto::digest,
    error::{Error, Result},
    model::Session,
    portal::self_service::Binding,
    saml::Reply,
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(crate) enum RevokeIntent<'a> {
    Logout {
        token: &'a str,
    },
    BrowserSignOut {
        cookie: Option<&'a str>,
        browser_only: bool,
    },
    BearerOne {
        token: &'a str,
        target_id: &'a str,
    },
    BrowserOne {
        cookie: Option<&'a str>,
        binding: &'a Binding,
        target_id: &'a str,
    },
    BrowserAll {
        cookie: Option<&'a str>,
        binding: &'a Binding,
    },
}

pub(crate) struct RevokeOutcome {
    pub(crate) body: Value,
    pub(crate) clear_browser_cookie: bool,
}

impl RevokeOutcome {
    fn plain(body: Value) -> Self {
        Self {
            body,
            clear_browser_cookie: false,
        }
    }
}

struct SessionReceipt {
    key: String,
    fingerprint: String,
    scope: Value,
}

impl SessionReceipt {
    fn current(caller: &str, scope: Value) -> Option<Self> {
        let context = crate::context::current()?;
        let key = context.idempotency_key?;
        Some(Self {
            key: digest(&format!("session.revoke\0{caller}\0{key}")),
            fingerprint: context.fingerprint,
            scope,
        })
    }

    fn replay(&self, tx: &Tx<'_>) -> Result<Option<Value>> {
        crate::context::replay_receipt(tx, &self.key, &self.fingerprint, &self.scope)
    }

    fn save(self, tx: &Tx<'_>, body: &Value) -> Result<()> {
        crate::context::save_receipt(tx, &self.key, self.fingerprint, self.scope, body)
    }
}

/// The adapters provide only their credential or browser binding. This writer
/// rechecks live authority and target ownership inside the mutation transaction.
/// A receipt is available for another session while the caller remains live.
/// Self-revocation and revoke-all deliberately invalidate that caller. Portal
/// sign-out retains its idempotent cookie-clearing response after the session
/// ends; it cannot replay a receipt under a no-longer-live browser session.
pub(crate) fn revoke_sessions(
    core: &Core,
    tx: &Tx<'_>,
    intent: RevokeIntent<'_>,
) -> Result<RevokeOutcome> {
    match intent {
        RevokeIntent::Logout { token } => {
            let (user, current) = core.session(tx, token)?;
            Ok(RevokeOutcome::plain(revoke_one_api(
                core, tx, &user.id, current,
            )?))
        }
        RevokeIntent::BrowserSignOut {
            cookie,
            browser_only,
        } => {
            let session = core.browser_session(tx, cookie)?;
            let body = match session {
                Some(session) if browser_only && crate::signin::bearer_backed(tx, &session)? => {
                    // Only unmap a terminal-backed browser. Its bearer session
                    // and grants remain available to the approving terminal.
                    json!({"revoked":false})
                }
                Some(session) => {
                    let user_id = session.identity.user_id.clone();
                    let revoked = revoke_one_api(core, tx, &user_id, session)?;
                    json!({"revoked":true,"saml_logout_url":revoked["saml_logout_url"]})
                }
                None => json!({"revoked":true}),
            };
            if let Some(cookie) = cookie {
                tx.delete("browser_sessions", &digest(cookie))?;
            }
            Ok(RevokeOutcome {
                body,
                clear_browser_cookie: true,
            })
        }
        RevokeIntent::BearerOne { token, target_id } => {
            if token.starts_with("ri_agent_") {
                // `session.revoke` names one session; `sessions.revoke` covers the
                // sessions of the account it names.
                let actor = core.principal(tx, token)?;
                let by_id = actor.allows("session.revoke", &format!("session/{target_id}"));
                let target = match tx.get::<Session>("sessions", target_id)? {
                    Some(target) => target,
                    None if by_id => return Err(Error::missing("Session not found")),
                    None => return Err(Error::forbidden()),
                };
                if !by_id {
                    let owner = tx
                        .get::<crate::model::User>("users", &target.identity.user_id)?
                        .ok_or_else(Error::forbidden)?;
                    actor.require("sessions.revoke", &format!("user/{}", owner.username))?;
                }
                crate::reconciliation::validate_apply_lease(tx, &actor)?;
                let receipt = SessionReceipt::current(
                    &actor.id,
                    json!({"authority":"agent","actor":actor.id,"permissions":actor.permissions}),
                );
                if let Some(body) = receipt
                    .as_ref()
                    .map(|r| r.replay(tx))
                    .transpose()?
                    .flatten()
                {
                    return Ok(RevokeOutcome::plain(body));
                }
                let body = revoke_one_api(core, tx, &actor.id, target)?;
                if let Some(receipt) = receipt {
                    receipt.save(tx, &body)?;
                }
                Ok(RevokeOutcome::plain(body))
            } else {
                let (user, current) = core.session(tx, token)?;
                let target = session(tx, target_id)?;
                if target.identity.user_id != user.id && !user.admin {
                    return Err(Error::forbidden());
                }
                // The current credential will be invalid after this write.
                let receipt = (target.id != current.id).then(|| {
                    SessionReceipt::current(
                        &format!("human:{}:{}", user.id, current.id),
                        json!({"authority":"bearer","user_id":user.id,"session_id":current.id,"admin":user.admin}),
                    )
                }).flatten();
                if let Some(body) = receipt
                    .as_ref()
                    .map(|r| r.replay(tx))
                    .transpose()?
                    .flatten()
                {
                    return Ok(RevokeOutcome::plain(body));
                }
                let body = revoke_one_api(core, tx, &user.id, target)?;
                if let Some(receipt) = receipt {
                    receipt.save(tx, &body)?;
                }
                Ok(RevokeOutcome::plain(body))
            }
        }
        RevokeIntent::BrowserOne {
            cookie,
            binding,
            target_id,
        } => {
            let cookie = cookie.ok_or_else(Error::unauthorized)?;
            let (user, current) = core.verified_browser(tx, Some(cookie), binding)?;
            let target = tx
                .get::<Session>("sessions", target_id)?
                .filter(|target| target.identity.user_id == user.id)
                .ok_or_else(|| Error::missing("Session not found"))?;
            let signed_out = target.id == current.id;
            let receipt = (!signed_out)
                .then(|| {
                    SessionReceipt::current(
                        &format!("browser:{}:{}", user.id, current.id),
                        json!({"authority":"browser","user_id":user.id,"session_id":current.id}),
                    )
                })
                .flatten();
            if let Some(body) = receipt
                .as_ref()
                .map(|r| r.replay(tx))
                .transpose()?
                .flatten()
            {
                return Ok(RevokeOutcome::plain(body));
            }
            if target.revoked {
                return Err(Error::missing("Session not found"));
            }
            let propagation = revoke_one_browser(core, tx, &user.id, target)?;
            if signed_out {
                tx.delete("browser_sessions", &digest(cookie))?;
            }
            let body = json!({"revoked":true,"signed_out":signed_out,"logout_url":propagation["redirect_uri"],"propagation":propagation});
            if let Some(receipt) = receipt {
                receipt.save(tx, &body)?;
            }
            Ok(RevokeOutcome {
                body,
                clear_browser_cookie: signed_out,
            })
        }
        RevokeIntent::BrowserAll { cookie, binding } => {
            let cookie = cookie.ok_or_else(Error::unauthorized)?;
            let (user, _) = core.verified_browser(tx, Some(cookie), binding)?;
            // Include expired rows: offline refresh grants still validate their
            // originating session until that row is explicitly revoked.
            let mut ids = BTreeSet::new();
            let mut frontchannel = BTreeSet::new();
            for (id, mut target) in tx.list::<Session>("sessions")? {
                if target.identity.user_id != user.id || target.revoked {
                    continue;
                }
                frontchannel.extend(crate::session_protocol::frontchannel_urls(
                    tx,
                    &id,
                    &core.config.issuer,
                )?);
                target.revoked = true;
                tx.put("sessions", &id, &target)?;
                audit(tx, &user.id, "session.revoke", &id)?;
                ids.insert(id);
            }
            crate::logout::queue_user(tx, &user.id)?;
            crate::ssf::enqueue(tx, &user.id, crate::ssf::SESSION_REVOKED, "")?;
            // Application approvals outlive no sign-out, as refresh tokens do not.
            crate::management::agent_applications::revoke_owner_all(tx, &user.id)?;
            audit(tx, &user.id, "session.revoke_all", &user.id)?;
            let propagation = propagate(core, tx, ids.clone(), frontchannel.into_iter().collect())?;
            tx.delete("browser_sessions", &digest(cookie))?;
            Ok(RevokeOutcome {
                body: json!({"revoked":true,"signed_out":true,"sessions_revoked":ids.len(),"logout_url":propagation["redirect_uri"],"propagation":propagation}),
                clear_browser_cookie: true,
            })
        }
    }
}

fn session(tx: &Tx<'_>, id: &str) -> Result<Session> {
    tx.get("sessions", id)?
        .ok_or_else(|| Error::missing("Session not found"))
}

fn revoke_one(tx: &Tx<'_>, actor: &str, mut target: Session) -> Result<()> {
    // Selected-revocation receipts replay before reaching this helper. A fresh
    // attempt against the same row must not queue logout or SSF a second time.
    if target.revoked {
        return Err(Error::missing("Session not found"));
    }
    target.revoked = true;
    tx.put("sessions", &target.id, &target)?;
    crate::logout::queue_session(tx, &target.id)?;
    crate::ssf::enqueue(
        tx,
        &target.identity.user_id,
        crate::ssf::SESSION_REVOKED,
        "",
    )?;
    audit(tx, actor, "session.revoke", &target.id)
}

fn revoke_one_api(core: &Core, tx: &Tx<'_>, actor: &str, target: Session) -> Result<Value> {
    let id = target.id.clone();
    revoke_one(tx, actor, target)?;
    let propagation = crate::saml::logout::redirect(core, tx, &id, None)?;
    Ok(
        json!({"revoked":true,"saml_logout_url":propagation["redirect_uri"],"saml_logout":propagation}),
    )
}

fn revoke_one_browser(core: &Core, tx: &Tx<'_>, actor: &str, target: Session) -> Result<Value> {
    let id = target.id.clone();
    let frontchannel = crate::session_protocol::frontchannel_urls(tx, &id, &core.config.issuer)?;
    revoke_one(tx, actor, target)?;
    propagate(core, tx, BTreeSet::from([id]), frontchannel)
}

fn propagate(
    core: &Core,
    tx: &Tx<'_>,
    ids: BTreeSet<String>,
    frontchannel: Vec<String>,
) -> Result<Value> {
    let finish = crate::saml::logout::Finish {
        frontchannel_urls: frontchannel.into_iter().collect(),
        ..Default::default()
    };
    match crate::saml::logout::begin(core, tx, &ids, finish)? {
        Reply::LogoutPage(value) => Ok(value),
        Reply::Redirect(uri) => Ok(json!({"redirect_uri":uri})),
        _ => Err(Error::internal("Unexpected logout continuation")),
    }
}
