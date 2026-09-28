//! Platform client-certificate binding and session persistence.

use crate::{
    core::{Core, audit, user_by_name, validate_email, validate_name},
    crypto::{self, digest, id, now},
    error::{Error, Result},
    model::{AuthenticationTransaction, Identity, Session, User, UserView},
    mtls::{
        BindInput, Binding, ClientCertAuth, ClientCertMode, LoginLink, MtlsTx,
        binding_matches, parse_pem, presented_chain, rejected, selector, validate_san_uri,
        verify_chain,
    },
    store::Tx,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, net::IpAddr};

impl MtlsTx for Tx<'_> {
    fn login_link(&self, session_id: &str) -> Result<Option<LoginLink>> {
        self.get("mtls_logins", session_id)
    }

    fn binding(&self, binding_id: &str) -> Result<Option<Binding>> {
        self.get("mtls_bindings", binding_id)
    }

    fn login_links_for_cleanup(&self) -> Result<Vec<(String, LoginLink)>> {
        self.maintenance_page("mtls_logins")
    }

    fn delete_login_link(&self, session_id: &str) -> Result<()> {
        self.delete("mtls_logins", session_id)
    }
}

fn profile(core: &Core) -> Result<&ClientCertAuth> {
    core.config
        .client_certificates
        .as_ref()
        .ok_or_else(|| Error::missing("Client certificate authentication is not configured"))
}

fn index_conflict(tx: &Tx<'_>, bucket: &str, key: &str, user_id: &str) -> Result<()> {
    let Some(id) = tx.get::<String>(bucket, key)? else {
        return Ok(());
    };
    if tx
        .get::<Binding>("mtls_bindings", &id)?
        .is_some_and(|binding| binding.user_id != user_id)
    {
        return Err(Error::conflict(
            "Certificate identity is already bound to another user",
        ));
    }
    Ok(())
}

fn clear_binding(tx: &Tx<'_>, binding: &Binding) -> Result<()> {
    if let Some(value) = &binding.fingerprint {
        tx.delete("mtls_fingerprints", value)?;
    }
    if let Some(value) = &binding.san_email {
        tx.delete("mtls_san_emails", &digest(value))?;
    }
    if let Some(value) = &binding.san_uri {
        tx.delete("mtls_san_uris", &digest(value))?;
    }
    tx.delete("mtls_users", &binding.user_id)?;
    tx.delete("mtls_bindings", &binding.id)?;
    Ok(())
}

/// Assisted and offline recovery remove certificate sign-in authority too.
pub(crate) fn clear_user_binding(tx: &Tx<'_>, user_id: &str) -> Result<()> {
    if let Some(binding_id) = tx.get::<String>("mtls_users", user_id)?
        && let Some(binding) = tx.get::<Binding>("mtls_bindings", &binding_id)?
    {
        end_binding_sessions(tx, &binding.id)?;
        clear_binding(tx, &binding)?;
    }
    Ok(())
}

fn end_binding_sessions(tx: &Tx<'_>, binding_id: &str) -> Result<()> {
    for (id, mut session) in tx.list::<Session>("sessions")? {
        if session.revoked {
            continue;
        }
        let linked = tx
            .get::<LoginLink>("mtls_logins", &id)?
            .is_some_and(|link| link.binding_id == binding_id);
        if !linked {
            continue;
        }
        session.revoked = true;
        tx.put("sessions", &id, &session)?;
        tx.delete("mtls_logins", &id)?;
        crate::logout::queue_session(tx, &id)?;
    }
    Ok(())
}

impl Core {
    pub fn client_certificate_bind(&self, token: &str, input: BindInput) -> Result<Value> {
        validate_name(&input.username)?;
        let profile = profile(self)?.clone();
        self.store.read(|tx| {
            let actor =
                self.management(tx, token, "mtls.bind", &format!("user/{}", input.username))?;
            let user = user_by_name(tx, &input.username)?;
            if actor.agent && user.admin {
                return Err(Error::forbidden());
            }
            Ok(())
        })?;
        let certificate_pem = input
            .certificate_pem
            .as_ref()
            .map(|pem| pem.trim())
            .filter(|pem| !pem.is_empty());
        let san_email = match input
            .san_email
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            Some(email) => {
                validate_email(email)?;
                Some(email.to_owned())
            }
            None => None,
        };
        let san_uri = match input
            .san_uri
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            Some(uri) => {
                validate_san_uri(uri)?;
                Some(uri.to_owned())
            }
            None => None,
        };
        if certificate_pem.is_none() && san_email.is_none() && san_uri.is_none() {
            return Err(Error::bad(
                "Bind a certificate PEM, SAN URI, SAN email, or a combination",
            ));
        }
        let (fingerprint, not_after) = if let Some(pem) = certificate_pem {
            let chain = parse_pem(pem)?;
            let leaf = verify_chain(&profile, &chain)?;
            if san_email
                .as_ref()
                .is_some_and(|email| !leaf.emails.iter().any(|value| value == email))
                || san_uri
                    .as_ref()
                    .is_some_and(|uri| !leaf.uris.iter().any(|value| value == uri))
            {
                return Err(Error::bad(
                    "Certificate does not contain the enrolled SAN URI or email",
                ));
            }
            (Some(leaf.fingerprint), Some(leaf.not_after))
        } else {
            (None, None)
        };
        let username = input.username;
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "mtls.bind", &format!("user/{username}"))?;
            let user = user_by_name(tx, &username)?;
            if actor.agent && user.admin {
                return Err(Error::forbidden());
            }
            if let Some(value) = &fingerprint {
                index_conflict(tx, "mtls_fingerprints", value, &user.id)?;
            }
            if let Some(value) = &san_email {
                index_conflict(tx, "mtls_san_emails", &digest(value), &user.id)?;
            }
            if let Some(value) = &san_uri {
                index_conflict(tx, "mtls_san_uris", &digest(value), &user.id)?;
            }
            let existing = tx
                .get::<String>("mtls_users", &user.id)?
                .and_then(|binding_id| tx.get::<Binding>("mtls_bindings", &binding_id).transpose())
                .transpose()?;
            if let Some(existing) = &existing
                && existing.fingerprint == fingerprint
                && existing.san_email == san_email
                && existing.san_uri == san_uri
                && existing.not_after == not_after
            {
                return Ok(existing.view());
            }
            if actor.agent {
                crate::delegation::mark_credential_exposure(tx, &actor, &user)?;
            }
            if let Some(existing) = &existing {
                end_binding_sessions(tx, &existing.id)?;
                clear_binding(tx, existing)?;
            }
            let binding = Binding {
                id: id(),
                username: user.username.clone(),
                user_id: user.id.clone(),
                fingerprint: fingerprint.clone(),
                san_uri: san_uri.clone(),
                san_email: san_email.clone(),
                not_after,
                created_at: now(),
            };
            if let Some(value) = &binding.fingerprint {
                tx.put("mtls_fingerprints", value, &binding.id)?;
            }
            if let Some(value) = &binding.san_email {
                tx.put("mtls_san_emails", &digest(value), &binding.id)?;
            }
            if let Some(value) = &binding.san_uri {
                tx.put("mtls_san_uris", &digest(value), &binding.id)?;
            }
            tx.put("mtls_users", &binding.user_id, &binding.id)?;
            tx.put("mtls_bindings", &binding.id, &binding)?;
            audit(tx, &actor.id, "mtls.bind", &binding.id)?;
            Ok(binding.view())
        })
    }
    pub fn client_certificate_list(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let mut rows = Vec::new();
            for (_, mut binding) in tx.list::<Binding>("mtls_bindings")? {
                let Some(user) = tx.get::<User>("users", &binding.user_id)? else {
                    continue;
                };
                if actor.allows("mtls.read", &format!("user/{}", user.username)) {
                    binding.username = user.username;
                    rows.push(binding.view());
                }
            }
            Ok(json!(rows))
        })
    }
    pub fn client_certificate_revoke(&self, token: &str, id: &str) -> Result<Value> {
        self.mutation(token, |tx| {
            let binding = tx
                .get::<Binding>("mtls_bindings", id)?
                .ok_or_else(|| Error::missing("Certificate binding not found"))?;
            let user = tx
                .get::<User>("users", &binding.user_id)?
                .ok_or_else(|| Error::missing("Certificate binding not found"))?;
            let actor =
                self.management(tx, token, "mtls.bind", &format!("user/{}", user.username))?;
            if actor.agent && user.admin {
                return Err(Error::forbidden());
            }
            if actor.agent {
                crate::delegation::mark_credential_exposure(tx, &actor, &user)?;
            }
            end_binding_sessions(tx, &binding.id)?;
            clear_binding(tx, &binding)?;
            audit(tx, &actor.id, "mtls.revoke", id)?;
            Ok(json!({"revoked": true, "id": id}))
        })
    }
    pub fn login_with_client_certificate(
        &self,
        peer: IpAddr,
        forwarded: Vec<String>,
        tls_ders: Vec<Vec<u8>>,
        transaction: Option<String>,
    ) -> Result<Value> {
        let profile = profile(self)?.clone();
        // Both modes reject a missing certificate here. Neither mode changes other routes.
        match profile.mode {
            ClientCertMode::Optional | ClientCertMode::Required => {}
        }
        let chain = presented_chain(
            &profile,
            &self.config.trusted_proxies,
            peer,
            &forwarded,
            &tls_ders,
        )?;
        let leaf = verify_chain(&profile, &chain)?;
        let session_ttl = self.config.session_ttl;
        self.store.write(|tx| {
            let mut matched = Vec::new();
            let mut ids = BTreeSet::new();
            if let Some(binding_id) = tx.get::<String>("mtls_fingerprints", &leaf.fingerprint)? {
                ids.insert(binding_id);
            }
            for email in &leaf.emails {
                if let Some(binding_id) = tx.get::<String>("mtls_san_emails", &digest(email))? {
                    ids.insert(binding_id);
                }
            }
            for uri in &leaf.uris {
                if let Some(binding_id) = tx.get::<String>("mtls_san_uris", &digest(uri))? {
                    ids.insert(binding_id);
                }
            }
            for binding_id in ids {
                if let Some(binding) = tx.get::<Binding>("mtls_bindings", &binding_id)?
                    && binding_matches(&binding, &leaf)
                {
                    matched.push(binding);
                }
            }
            let users: BTreeSet<_> = matched
                .iter()
                .map(|binding| binding.user_id.clone())
                .collect();
            if users.len() != 1 {
                audit(tx, "anonymous", "login.failed", "client-certificate")?;
                return Ok(Err(rejected("Client certificate is not enrolled")));
            }
            let Some(binding) = matched.pop() else {
                audit(tx, "anonymous", "login.failed", "client-certificate")?;
                return Ok(Err(rejected("Client certificate is not enrolled")));
            };
            let Some(user) = tx.get::<User>("users", &binding.user_id)? else {
                audit(tx, "anonymous", "login.failed", "client-certificate")?;
                return Ok(Err(rejected("Client certificate is not enrolled")));
            };
            if !user.enabled {
                audit(tx, "anonymous", "login.failed", &user.username)?;
                return Ok(Err(rejected("Account is disabled")));
            }
            let at = now();
            let expires_at = (at + session_ttl).min(leaf.not_after);
            if expires_at <= at {
                audit(tx, "anonymous", "login.failed", &user.username)?;
                return Ok(Err(rejected(
                    "Client certificate is expired or not yet valid",
                )));
            }
            let challenge = transaction
                .as_ref()
                .map(|token| {
                    let challenge = tx
                        .get::<AuthenticationTransaction>("authentication", &digest(token))?
                        .filter(|item| item.expires_at > at && item.authenticated_session.is_none())
                        .ok_or_else(|| Error::bad("Authentication transaction expired or used"))?;
                    crate::oidc::reject_embedded_stage(&challenge)?;
                    if challenge
                        .user_id
                        .as_ref()
                        .is_some_and(|uid| uid != &user.id)
                    {
                        return Err(Error::forbidden());
                    }
                    Ok(challenge)
                })
                .transpose()?;
            let token = crypto::random_token("ri_session_");
            let sid = id();
            let identity = Identity {
                user_id: user.id.clone(),
                epoch: user.epoch,
                mfa: false,
                auth_time: at,
                session_id: sid.clone(),
                amr: vec!["cert".into()],
                source: None,
            };
            let session = Session {
                id: sid.clone(),
                token_hash: digest(&token),
                identity,
                expires_at,
                revoked: false,
            };
            tx.put("sessions", &sid, &session)?;
            tx.put("session_tokens", &session.token_hash, &sid)?;
            let binding_selector = selector(&binding);
            tx.put(
                "mtls_logins",
                &sid,
                &LoginLink {
                    binding_id: binding.id,
                    selector: binding_selector,
                    expires_at,
                },
            )?;
            if let Some(mut challenge) = challenge {
                challenge.authenticated_session = Some(sid.clone());
                let Some(token) = transaction.as_deref() else {
                    return Err(Error::internal("missing authentication transaction"));
                };
                tx.put("authentication", &digest(token), &challenge)?;
            }
            audit(tx, &user.id, "login.succeeded", &sid)?;
            Ok(Ok(json!({
                "session_token": token,
                "expires_at": session.expires_at,
                "user": UserView::from(&user),
            })))
        })?
    }
}
