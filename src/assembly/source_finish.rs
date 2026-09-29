//! Complete a source login through the concrete one-use storage transaction.

use crate::{
    core::{Core, audit, make_user},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{AuthenticationTransaction, Identity, NewUser, Session, User, UserView},
    source::{Finish, Link, Login, SourceIdentity, enabled, link_key},
    store::Tx,
};
use serde_json::{Value, json};

pub(crate) fn clear_browser_return(tx: &Tx<'_>, pending: &Login) -> Result<()> {
    if let Some(token) = &pending.browser_return {
        tx.delete("source_returns", token)?;
    }
    Ok(())
}

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
    pub(crate) fn complete_source_login(
        &self,
        tx: &Tx<'_>,
        state: &str,
        pending: &mut Login,
        approve: bool,
        otp: Option<&str>,
        expected_user: Option<&str>,
    ) -> Result<Result<Value>> {
        // Browser-started SAML is unfinished until the same-site return. OIDC already
        // checked its cookie, and rows written before the handoff default confirmed.
        if pending.browser_binding.is_some() && !pending.browser_return_confirmed {
            return Err(Error::unauthorized());
        }
        if pending.workflow.is_some() {
            return Err(Error::forbidden());
        }
        let source = enabled(tx, &pending.source)?;
        if pending.fingerprint != source.fingerprint()? {
            return Err(Error::bad("Source configuration changed; restart login"));
        }
        let Some(identity) = pending.result.clone() else {
            return Ok(Ok(json!({"status":"pending"})));
        };
        let link_id = link_key(&source.id, &source.issuer, &identity.subject);
        let link = tx.get::<Link>("source_links", &link_id)?;
        let mut user = if let Some(target) = &pending.target {
            if expected_user.is_some() {
                return Err(Error::forbidden());
            }
            let user = self.identity_user(tx, target)?;
            if tx
                .get::<Session>("sessions", &target.session_id)?
                .is_none_or(|s| s.expires_at <= now())
                || target.auth_time + 900 < now()
                || link.as_ref().is_some_and(|l| l.user_id != user.id)
            {
                return Err(Error::forbidden());
            }
            Some(user)
        } else {
            link.as_ref()
                .map(|l| tx.get::<User>("users", &l.user_id))
                .transpose()?
                .flatten()
        };
        if expected_user.is_some_and(|id| user.as_ref().map(|u| u.id.as_str()) != Some(id)) {
            // A bound authorization may use only the explicit link. Do not provision or reattach.
            return Err(Error::forbidden());
        }
        if user
            .as_ref()
            .is_some_and(|u| !u.enabled || u.admin && !source.allow_admin_login)
        {
            return Err(Error::forbidden());
        }
        if !approve {
            return Ok(Ok(
                json!({"status":"review", "issuer":source.issuer,"subject":identity.subject,"name":identity.name,"email":identity.email,"email_verified":identity.email_verified,"mfa":identity.mfa,"linking":pending.target.is_some(),"local_user":user.as_ref().map(UserView::from),"local_otp_required":user.as_ref().is_some_and(|u| u.totp_secret.is_some()) && !identity.mfa,"auto_provision":source.auto_provision}),
            ));
        }
        if user.is_none() {
            if !source.auto_provision || link.is_some() {
                return Err(Error::forbidden());
            }
            let mut created = make_user(NewUser {
                username: format!("oidc-{}", digest(&link_id)),
                password: crypto::random_token(""),
                email: identity.email.clone(),
                display_name: identity.name.clone(),
                admin: false,
            })?;
            // Generated password is discarded: this account authenticates at its source.
            created.password_hash.clear();
            created.email_verified = identity.email_verified;
            if tx.get::<String>("usernames", &created.username)?.is_some() {
                return Err(Error::conflict(
                    "Provisioned username already exists; explicit linking is required",
                ));
            }
            crate::management::write_source_memberships(
                &self.config,
                tx,
                &source.groups,
                &created.id,
            )?;
            audit(
                tx,
                &format!("source:{}", source.id),
                "user.provision",
                &created.username,
            )?;
            user = Some(created);
        }
        let mut user = user.unwrap();
        let local_mfa = user.totp_secret.is_some() && !identity.mfa;
        if local_mfa {
            let valid = if let Some(code) = otp.filter(|c| c.starts_with("ri_recovery_")) {
                user.recovery_codes.remove(&digest(code))
            } else {
                let step = crypto::totp_step_with(
                    user.totp_secret.as_deref().unwrap(),
                    &user.username,
                    otp.unwrap_or(""),
                    now(),
                    user.totp_last_step,
                    &user.totp_settings,
                )?;
                if let Some(step) = step {
                    user.totp_last_step = Some(step);
                    true
                } else {
                    false
                }
            };
            if !valid {
                pending.attempts += 1;
                tx.put("source_logins", state, &pending)?;
                return Ok(Err(Error::unauthorized()));
            }
        }
        tx.put("users", &user.id, &user)?;
        tx.put("usernames", &user.username, &user.id)?;
        let link_id = crate::management::write_source_link(
            tx,
            crate::management::SourceLinkAuthority::VerifiedLogin {
                source_id: &source.id,
                source_fingerprint: &pending.fingerprint,
                user_id: &user.id,
                subject: &identity.subject,
                approved: approve,
            },
        )?
        .id;
        let session_token = crypto::random_token("ri_session_");
        let sid = crypto::id();
        let mut amr = vec!["federated".into()];
        if identity.mfa {
            amr.push("mfa".into());
        }
        if local_mfa {
            amr.push("otp".into());
        }
        let session = Session {
            id: sid.clone(),
            token_hash: digest(&session_token),
            identity: Identity {
                user_id: user.id.clone(),
                epoch: user.epoch,
                mfa: identity.mfa || local_mfa,
                auth_time: identity.auth_time,
                session_id: sid.clone(),
                amr,
                source: Some(SourceIdentity {
                    id: source.id.clone(),
                    fingerprint: pending.fingerprint.clone(),
                    link: link_id,
                    pin_retired: false,
                }),
            },
            expires_at: (now() + self.config.session_ttl).min(
                identity
                    .saml_session
                    .as_ref()
                    .and_then(|s| s.expires_at)
                    .unwrap_or(u64::MAX),
            ),
            revoked: false,
        };
        if let Some(challenge) = &pending.authentication {
            let mut transaction = tx
                .get::<AuthenticationTransaction>("authentication", &digest(challenge))?
                .filter(|c| {
                    c.expires_at > now()
                        && c.authenticated_session.is_none()
                        && c.user_id.as_ref().is_none_or(|id| id == &user.id)
                })
                .ok_or_else(Error::forbidden)?;
            if transaction.source_stage.as_deref() != pending.stage.as_deref()
                && transaction.source_stage.is_some()
            {
                return Err(Error::forbidden());
            }
            transaction.authenticated_session = Some(sid.clone());
            tx.put("authentication", &digest(challenge), &transaction)?;
        }
        if let Some(upstream) = &identity.saml_session {
            if upstream.expires_at.is_some_and(|at| at <= now()) {
                return Err(Error::forbidden());
            }
            tx.put("saml_source_sessions", &sid, upstream)?;
        }
        tx.put("sessions", &sid, &session)?;
        tx.put("session_tokens", &session.token_hash, &sid)?;
        clear_browser_return(tx, pending)?;
        tx.delete("source_polls", &pending.poll_hash)?;
        tx.delete("source_logins", state)?;
        audit(
            tx,
            &user.id,
            if pending.target.is_some() {
                "source.link"
            } else {
                "source.login"
            },
            &source.id,
        )?;
        Ok(Ok(
            json!({"status":"complete","session_token":session_token,"expires_at":session.expires_at,"user":UserView::from(&user)}),
        ))
    }
}
