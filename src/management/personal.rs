//! Narrow management of one account: its profile, sessions, application
//! consent and owned agents. Each route checks its own personal action on
//! `user/<username>`; none implies `user.read` or `user.write`. Session and
//! agent revocation reuse their existing writers.

use crate::{
    browser::consents_for_user,
    core::{Core, user_by_name},
    error::Result,
    model::{Session, User},
    store::Tx,
};
use serde_json::{Value, json};

/// Unrevoked sessions of one account, newest information first as stored.
pub(crate) fn session_list(tx: &Tx<'_>, user_id: &str) -> Result<Value> {
    let mut list = Vec::new();
    for session in tx
        .list::<Session>("sessions")?
        .into_iter()
        .map(|(_, session)| session)
        .filter(|session| session.identity.user_id == user_id && !session.revoked)
    {
        let kind = if crate::signin::bearer_backed(tx, &session)? {
            "terminal"
        } else {
            "browser"
        };
        list.push(json!({"id": session.id, "auth_time": session.identity.auth_time, "expires_at": session.expires_at, "mfa": session.identity.mfa, "kind": kind}));
    }
    Ok(json!(list))
}

impl Core {
    fn personal_target(
        &self,
        tx: &Tx<'_>,
        token: &str,
        action: &str,
        username: &str,
    ) -> Result<User> {
        self.management(tx, token, action, &format!("user/{username}"))?;
        user_by_name(tx, username)
    }

    /// Permitted profile fields of one account (`profile.read`).
    pub fn user_profile(&self, token: &str, username: &str) -> Result<Value> {
        self.store.read(|tx| {
            let user = self.personal_target(tx, token, "profile.read", username)?;
            Ok(json!({"username": user.username, "display_name": user.display_name, "email": user.email, "email_verified": user.email_verified}))
        })
    }

    /// Unrevoked sessions of one account (`sessions.read`). Revoke one with
    /// `DELETE /api/sessions/{id}` and `sessions.revoke` on the same account.
    pub fn user_sessions(&self, token: &str, username: &str) -> Result<Value> {
        self.store.read(|tx| {
            let user = self.personal_target(tx, token, "sessions.read", username)?;
            session_list(tx, &user.id)
        })
    }

    /// Remembered application consent of one account (`consents.read`).
    pub fn user_consents(&self, token: &str, username: &str) -> Result<Value> {
        self.store.read(|tx| {
            let user = self.personal_target(tx, token, "consents.read", username)?;
            Ok(json!(consents_for_user(tx, &user.id)?))
        })
    }

    /// Withdraw one application's consent and revoke that account's
    /// outstanding grants for it (`consents.revoke`), as the owner's own
    /// bearer withdrawal does, without first revealing whether the client exists.
    pub fn revoke_user_consent(
        &self,
        token: &str,
        username: &str,
        client_id: &str,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor =
                self.management(tx, token, "consents.revoke", &format!("user/{username}"))?;
            let user = user_by_name(tx, username)?;
            super::consents::revoke_consent_for_user(tx, &actor.id, &user.id, client_id)?;
            Ok(json!({"revoked": true, "username": user.username, "client_id": client_id}))
        })
    }

    /// Agents owned by one account, with the permissions each holds now
    /// (`agents.read`). Revoke one with `DELETE /api/agents/{id}` and
    /// `agents.revoke` on the same account.
    pub fn user_agents(&self, token: &str, username: &str) -> Result<Value> {
        self.store.read(|tx| {
            let user = self.personal_target(tx, token, "agents.read", username)?;
            let mut agents = Vec::new();
            for (_, agent) in tx.list::<crate::agent::Agent>("agents")? {
                if agent.parent_user.as_deref() == Some(user.id.as_str()) {
                    agents.push(crate::agent::effective_view(tx, &self.config, &agent)?);
                }
            }
            Ok(json!(agents))
        })
    }
}
