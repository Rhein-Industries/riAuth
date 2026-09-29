//! RADIUS client, identity, certificate, replay and close storage operations.

use crate::{
    core::{Core, audit, validate_name},
    crypto::{digest, now},
    error::{Error, Result},
    model::{Client, Identity, Session, User},
    radius::{Listener, Packet, RadiusClaim, Settings, WireAttributes, eap, fingerprint},
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, net::IpAddr};

pub async fn radius_start(core: Core) -> anyhow::Result<crate::radius::Servers> {
    crate::radius::start_with_port(core).await
}

#[derive(Clone, Serialize, Deserialize)]
struct Cached {
    expires_at: u64,
    client_id: String,
    client_fingerprint: String,
    attributes_fingerprint: Option<String>,
    response: Option<Vec<u8>>,
    identity: Option<Identity>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Certificate {
    id: String,
    username: String,
    user_id: String,
    listener: String,
    fingerprint: String,
    expires_at: u64,
}

#[derive(Clone, Serialize, Deserialize)]
struct IdentityBinding {
    certificate_key: String,
    certificate_id: String,
    listener: String,
    profile_fingerprint: String,
    expires_at: u64,
}

fn close(tx: &Tx<'_>, identity: Option<&Identity>) -> Result<()> {
    if let Some(identity) = identity
        && let Some(mut session) = tx.get::<Session>("sessions", &identity.session_id)?
    {
        session.revoked = true;
        tx.put("sessions", &session.id, &session)?;
        crate::ssf::enqueue(
            tx,
            &session.identity.user_id,
            crate::ssf::SESSION_REVOKED,
            "",
        )?;
    }
    Ok(())
}

pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    Core::radius_eap_cleanup(tx, at)?;
    Core::radius_cleanup_cache(tx, at)
}

impl Core {
    fn require_radius_certificate_retry_binding() -> Result<()> {
        if let Some(context) = crate::context::current()
            && (context.idempotency_key.is_none() || context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "RADIUS certificate writes require Idempotency-Key and If-Match",
            ));
        }
        Ok(())
    }

    fn radius_eap_profile(&self, listener: &str) -> Result<&eap::Config> {
        self.config
            .radius_listeners
            .get(listener)
            .and_then(|config| config.eap_tls.as_ref())
            .ok_or_else(|| Error::bad("RADIUS listener has no EAP-TLS trust profile"))
    }

    pub(crate) fn radius_rate_limited(
        &self,
        peer: IpAddr,
        listener_id: &str,
        nas_id: &str,
    ) -> Result<bool> {
        self.store
            .shared_rate_limit(peer, &format!("radius:{listener_id}/{nas_id}"), 600)
    }

    pub(crate) fn radius_client_profile(
        &self,
        tx: &Tx<'_>,
        id: &str,
    ) -> Result<(Client, Settings)> {
        let client = tx
            .get::<Client>("clients", id)?
            .filter(|c| c.enabled)
            .ok_or_else(Error::forbidden)?;
        let settings = client
            .settings
            .radius
            .clone()
            .ok_or_else(Error::forbidden)?;
        settings.validate(&client)?;
        Ok((client, settings))
    }

    pub(crate) fn radius_identity(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        identity: &Identity,
    ) -> Result<User> {
        let user = self.authorize_identity(tx, client, identity)?;
        if tx
            .get::<Session>("sessions", &identity.session_id)?
            .is_none_or(|s| s.expires_at <= now())
            || crate::assurance::needs_step_up(client, &Default::default(), identity)
        {
            return Err(Error::forbidden());
        }
        crate::claims::enforce(tx, client, &user, identity, &["radius".into()].into())?;
        Ok(user)
    }

    pub(crate) fn radius_eap_client(&self, id: &str) -> Result<Option<Client>> {
        self.store.read(|tx| {
            let (client, settings) = self.radius_client_profile(tx, id)?;
            Ok(settings.eap_tls.then_some(client))
        })
    }

    pub(crate) fn radius_user_requires_mfa(&self, username: &str) -> Result<bool> {
        self.store
            .read(|tx| match crate::core::user_by_name(tx, username) {
                Ok(user) => Ok(user.totp_secret.is_some()),
                Err(e) if e.status.as_u16() == 404 => Ok(false),
                Err(e) => Err(e),
            })
    }

    pub(crate) fn radius_claim(
        &self,
        client_id: &str,
        key: &str,
        packet: &Packet,
        secret: &[u8],
    ) -> Result<RadiusClaim> {
        self.store.write(|tx| {
            let (client, settings) = self.radius_client_profile(tx, client_id)?;
            let fp = fingerprint(&client)?;
            if let Some(mut cached) = tx.get::<Cached>("radius_requests", key)? {
                if cached.expires_at > now() {
                    if cached.client_id != client_id
                        || cached.client_fingerprint != fp
                        || match cached.identity.as_ref() {
                            None => false,
                            Some(identity) => match self.radius_identity(tx, &client, identity) {
                                Ok(user) => {
                                    let mut attrs = settings.attributes(&user)?;
                                    if packet.attr(79).is_some() {
                                        attrs.push((1, user.username.as_bytes().to_vec()));
                                    }
                                    cached.attributes_fingerprint.as_deref()
                                        != Some(&digest(
                                            &serde_json::to_string(&attrs)
                                                .map_err(Error::internal)?,
                                        ))
                                }
                                Err(e) if e.status.is_server_error() => return Err(e),
                                Err(_) => true,
                            },
                        }
                    {
                        close(tx, cached.identity.as_ref())?;
                        cached.identity = None;
                        cached.response = Some(packet.response(3, secret, eap::failure(packet))?);
                        tx.put("radius_requests", key, &cached)?;
                    }
                    return Ok(RadiusClaim::Cached(cached.response));
                }
                close(tx, cached.identity.as_ref())?;
            }
            if tx.list::<Cached>("radius_requests")?.len() >= 10000 {
                return Err(Error::conflict("RADIUS duplicate cache is full"));
            }
            tx.put(
                "radius_requests",
                key,
                &Cached {
                    expires_at: now() + 90,
                    client_id: client_id.to_owned(),
                    client_fingerprint: fp,
                    attributes_fingerprint: None,
                    response: None,
                    identity: None,
                },
            )?;
            Ok(RadiusClaim::New)
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn radius_pap_commit(
        &self,
        listener_id: &str,
        nas_id: &str,
        client_id: &str,
        key: &str,
        packet: &Packet,
        secret: &[u8],
        token: Option<&str>,
    ) -> Result<(Option<Vec<u8>>, bool)> {
        self.store.write(|tx| {
            let mut cached = tx
                .get::<Cached>("radius_requests", key)?
                .filter(|c| c.expires_at > now() && c.response.is_none())
                .ok_or_else(|| Error::conflict("RADIUS request expired"))?;
            let allowed = (|| -> Result<(Identity, WireAttributes)> {
                let (client, settings) = self.radius_client_profile(tx, client_id)?;
                if fingerprint(&client)? != cached.client_fingerprint {
                    return Err(Error::forbidden());
                }
                let (_, session) = self.session(tx, token.ok_or_else(Error::unauthorized)?)?;
                let user = self.radius_identity(tx, &client, &session.identity)?;
                Ok((session.identity, settings.attributes(&user)?))
            })();
            let (code, attrs) = match allowed {
                Ok((identity, attrs)) => {
                    cached.identity = Some(identity);
                    cached.attributes_fingerprint = Some(digest(
                        &serde_json::to_string(&attrs).map_err(Error::internal)?,
                    ));
                    (2, attrs)
                }
                Err(e) if e.status.is_server_error() => return Err(e),
                Err(_) => (3, vec![]),
            };
            let response = packet.response(code, secret, attrs)?;
            cached.response = Some(response.clone());
            tx.put("radius_requests", key, &cached)?;
            audit(
                tx,
                cached
                    .identity
                    .as_ref()
                    .map(|i| i.user_id.as_str())
                    .unwrap_or("anonymous"),
                if code == 2 {
                    "radius.accept"
                } else {
                    "radius.reject"
                },
                &format!("{listener_id}/{nas_id}"),
            )?;
            Ok((Some(response), code == 2))
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn radius_eap_commit(
        &self,
        listener: &str,
        nas_id: &str,
        client_id: &str,
        key: &str,
        packet: &Packet,
        secret: &[u8],
        outcome: eap::Outcome,
    ) -> Result<(Option<Vec<u8>>, bool)> {
        self.store.write(|tx| {
            let mut cached = tx
                .get::<Cached>("radius_requests", key)?
                .filter(|c| c.expires_at > now() && c.response.is_none())
                .ok_or_else(|| Error::conflict("RADIUS request expired"))?;
            let (client, settings) = self.radius_client_profile(tx, client_id)?;
            let unchanged = settings.eap_tls && fingerprint(&client)? == cached.client_fingerprint;
            let (code, attrs) = if !unchanged {
                (3, eap::failure(packet))
            } else {
                match outcome {
                    eap::Outcome::Challenge(attrs) => (11, attrs),
                    eap::Outcome::Reject(attrs) => (3, attrs),
                    eap::Outcome::Accept(identity, protocol_attrs) => {
                        match self.radius_identity(tx, &client, &identity) {
                            Ok(user) => {
                                let mut attrs = settings.attributes(&user)?;
                                attrs.push((1, user.username.as_bytes().to_vec()));
                                cached.attributes_fingerprint = Some(digest(
                                    &serde_json::to_string(&attrs).map_err(Error::internal)?,
                                ));
                                cached.identity = Some(identity);
                                attrs.extend(protocol_attrs);
                                (2, attrs)
                            }
                            Err(e) if e.status.is_server_error() => return Err(e),
                            Err(_) => (3, eap::failure(packet)),
                        }
                    }
                }
            };
            let response = packet.response(code, secret, attrs)?;
            cached.response = Some(response.clone());
            tx.put("radius_requests", key, &cached)?;
            if code != 11 {
                audit(
                    tx,
                    cached
                        .identity
                        .as_ref()
                        .map(|i| i.user_id.as_str())
                        .unwrap_or("anonymous"),
                    if code == 2 {
                        "radius.accept"
                    } else {
                        "radius.reject"
                    },
                    &format!("{listener}/{nas_id}"),
                )?;
            }
            Ok((Some(response), code == 2))
        })
    }

    pub(crate) fn radius_close(&self, identity: Option<&Identity>) -> Result<()> {
        self.store.write(|tx| close(tx, identity))
    }

    pub(crate) fn radius_cleanup_cache(tx: &Tx<'_>, at: u64) -> Result<()> {
        for (id, cached) in tx.maintenance_page::<Cached>("radius_requests")? {
            if cached.expires_at < at {
                close(tx, cached.identity.as_ref())?;
                tx.delete("radius_requests", &id)?;
            }
        }
        Ok(())
    }

    pub fn radius_certificate_bind(
        &self,
        token: &str,
        input: eap::CertificateInput,
    ) -> Result<Value> {
        validate_name(&input.username)?;
        validate_name(&input.listener)?;
        self.radius_eap_authorize_bind(token, &input.username, &input.listener)?;
        Self::require_radius_certificate_retry_binding()?;
        let prepared =
            eap::prepare_certificate_binding(&input, || self.radius_eap_profile(&input.listener))?;
        self.radius_eap_bind_commit(
            token,
            input,
            prepared.key,
            prepared.fingerprint,
            prepared.expires_at,
        )
    }

    pub(crate) fn radius_eap_authorize_bind(
        &self,
        token: &str,
        username: &str,
        listener: &str,
    ) -> Result<()> {
        self.store.read(|tx| {
            let actor =
                self.management(tx, token, "certificate.write", &format!("user/{username}"))?;
            actor.require("radius.enroll", &format!("radius/{listener}"))?;
            let user = crate::core::user_by_name(tx, username)?;
            if actor.agent && user.admin {
                return Err(Error::forbidden());
            }
            Ok(())
        })
    }

    pub(crate) fn radius_eap_bind_commit(
        &self,
        token: &str,
        input: eap::CertificateInput,
        key: String,
        fingerprint: String,
        expires_at: u64,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.management(
                tx,
                token,
                "certificate.write",
                &format!("user/{}", input.username),
            )?;
            actor.require("radius.enroll", &format!("radius/{}", input.listener))?;
            let user = crate::core::user_by_name(tx, &input.username)?;
            if actor.agent && user.admin {
                return Err(Error::forbidden());
            }
            let existing = tx.get::<Certificate>("radius_certificates", &key)?;
            if let Some(existing) = &existing {
                if existing.user_id != user.id {
                    return Err(Error::conflict(
                        "Certificate is already bound to another identity",
                    ));
                }
                return Ok(json!(existing));
            }
            if tx
                .list::<Certificate>("radius_certificates")?
                .iter()
                .filter(|(_, c)| c.user_id == user.id)
                .count()
                >= 32
            {
                return Err(Error::conflict("User has too many EAP certificates"));
            }
            if actor.agent {
                // The agent can hold this certificate's private key and sign in
                // as the target. Fence live temporary access and record prior
                // exposure before any new binding becomes usable.
                crate::delegation::mark_credential_exposure(&self.config, tx, &actor, &user)?;
            }
            let certificate = Certificate {
                id: crate::crypto::id(),
                username: user.username,
                user_id: user.id,
                listener: input.listener.clone(),
                fingerprint,
                expires_at,
            };
            tx.put("radius_certificate_ids", &certificate.id, &key)?;
            tx.put("radius_certificates", &key, &certificate)?;
            audit(tx, &actor.id, "certificate.bind", &certificate.id)?;
            Ok(json!(certificate))
        })
    }

    pub fn radius_certificates(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let mut rows = Vec::new();
            for (_, mut certificate) in tx.list::<Certificate>("radius_certificates")? {
                let Some(user) = tx.get::<User>("users", &certificate.user_id)? else {
                    continue;
                };
                if actor.allows("certificate.read", &format!("user/{}", user.username))
                    && actor.allows("radius.enroll", &format!("radius/{}", certificate.listener))
                {
                    certificate.username = user.username;
                    rows.push(certificate);
                }
            }
            Ok(json!(rows))
        })
    }

    pub fn radius_certificate_revoke(&self, token: &str, id: &str) -> Result<Value> {
        Self::require_radius_certificate_retry_binding()?;
        self.mutation(token, |tx| {
            let key = tx
                .get::<String>("radius_certificate_ids", id)?
                .ok_or_else(|| Error::missing("Certificate not found"))?;
            let certificate = tx
                .get::<Certificate>("radius_certificates", &key)?
                .ok_or_else(|| Error::missing("Certificate not found"))?;
            let user = tx
                .get::<User>("users", &certificate.user_id)?
                .ok_or_else(Error::forbidden)?;
            let actor = self.management(
                tx,
                token,
                "certificate.write",
                &format!("user/{}", user.username),
            )?;
            actor.require("radius.enroll", &format!("radius/{}", certificate.listener))?;
            if actor.agent && user.admin {
                return Err(Error::forbidden());
            }
            tx.delete("radius_certificates", &key)?;
            tx.delete("radius_certificate_ids", id)?;
            audit(tx, &actor.id, "certificate.revoke", id)?;
            Ok(json!({"revoked":true,"id":id}))
        })
    }

    pub(crate) fn radius_eap_create_identity(
        &self,
        client: &Client,
        listener: &str,
        profile_fp: &str,
        key: String,
    ) -> Result<Identity> {
        self.store.write(|tx| {
            let (current, settings) = self.radius_client_profile(tx, &client.id)?;
            if !settings.eap_tls || fingerprint(&current)? != fingerprint(client)? {
                return Err(Error::forbidden());
            }
            let cert = tx
                .get::<Certificate>("radius_certificates", &key)?
                .filter(|c| c.expires_at > now())
                .ok_or_else(Error::forbidden)?;
            let user = tx
                .get::<User>("users", &cert.user_id)?
                .filter(|u| u.enabled)
                .ok_or_else(Error::forbidden)?;
            let sid = crate::crypto::id();
            let identity = Identity {
                user_id: user.id,
                epoch: user.epoch,
                mfa: false,
                auth_time: now(),
                session_id: sid.clone(),
                amr: vec!["x509".into()],
                source: None,
            };
            let expiry = (now() + 180).min(cert.expires_at);
            let session = Session {
                id: sid.clone(),
                token_hash: digest(&crate::crypto::random_token("")),
                identity: identity.clone(),
                expires_at: expiry,
                revoked: false,
            };
            // No session_tokens index: this protocol identity cannot become an HTTP bearer session.
            tx.put("sessions", &sid, &session)?;
            tx.put(
                "radius_eap_identities",
                &sid,
                &IdentityBinding {
                    certificate_key: key,
                    certificate_id: cert.id,
                    listener: listener.into(),
                    profile_fingerprint: profile_fp.into(),
                    expires_at: expiry,
                },
            )?;
            self.radius_identity(tx, &current, &identity)?;
            Ok(identity)
        })
    }

    pub(crate) fn radius_eap_validate_identity(
        &self,
        tx: &Tx<'_>,
        identity: &Identity,
    ) -> Result<()> {
        if identity.source.is_none() && identity.amr.iter().any(|a| a == "x509") {
            let binding = tx
                .get::<IdentityBinding>("radius_eap_identities", &identity.session_id)?
                .filter(|b| b.expires_at > now())
                .ok_or_else(Error::unauthorized)?;
            let cert = tx
                .get::<Certificate>("radius_certificates", &binding.certificate_key)?
                .filter(|c| {
                    c.id == binding.certificate_id
                        && c.user_id == identity.user_id
                        && c.expires_at > now()
                })
                .ok_or_else(Error::unauthorized)?;
            if cert.listener != binding.listener
                || eap::profile_fingerprint(self.radius_eap_profile(&binding.listener)?)?
                    != binding.profile_fingerprint
            {
                return Err(Error::unauthorized());
            }
        }
        Ok(())
    }

    pub(crate) fn radius_eap_cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
        for (id, b) in tx.maintenance_page::<IdentityBinding>("radius_eap_identities")? {
            if b.expires_at <= at {
                tx.delete("radius_eap_identities", &id)?;
            }
        }
        Ok(())
    }
}

impl eap::EapPort for Core {
    fn profile(&self, listener: &str) -> Result<&eap::Config> {
        self.radius_eap_profile(listener)
    }

    fn create_identity(
        &self,
        client: &Client,
        listener: &str,
        profile_fp: &str,
        der: &[u8],
    ) -> Result<Identity> {
        let fingerprint = eap::certificate_fingerprint(der);
        let key = eap::certificate_key(listener, &fingerprint);
        self.radius_eap_create_identity(client, listener, profile_fp, key)
    }

    fn close(&self, identity: Option<&Identity>) -> Result<()> {
        self.radius_close(identity)
    }
}

impl crate::radius::RadiusPort for Core {
    fn listeners(&self) -> &BTreeMap<String, Listener> {
        &self.config.radius_listeners
    }

    fn bind_listener(&self, id: &str, listener: &Listener) -> crate::capability::ListenerLease {
        self.runtime.bind_radius(id, listener)
    }

    fn rate_limited(&self, peer: IpAddr, listener_id: &str, nas_id: &str) -> Result<bool> {
        Core::radius_rate_limited(self, peer, listener_id, nas_id)
    }

    fn claim(
        &self,
        client_id: &str,
        key: &str,
        packet: &Packet,
        secret: &[u8],
    ) -> Result<RadiusClaim> {
        Core::radius_claim(self, client_id, key, packet, secret)
    }

    fn eap_client(&self, client_id: &str) -> Result<Option<Client>> {
        Core::radius_eap_client(self, client_id)
    }

    fn user_requires_mfa(&self, username: &str) -> Result<bool> {
        Core::radius_user_requires_mfa(self, username)
    }

    fn login(&self, username: String, password: String, otp: Option<String>) -> Result<Value> {
        Core::login(self, username, password, otp)
    }

    fn pap_commit(
        &self,
        listener_id: &str,
        nas_id: &str,
        client_id: &str,
        key: &str,
        packet: &Packet,
        secret: &[u8],
        token: Option<&str>,
    ) -> Result<(Option<Vec<u8>>, bool)> {
        Core::radius_pap_commit(
            self,
            listener_id,
            nas_id,
            client_id,
            key,
            packet,
            secret,
            token,
        )
    }

    fn logout(&self, token: &str) -> Result<Value> {
        Core::logout(self, token)
    }

    fn eap_commit(
        &self,
        listener: &str,
        nas_id: &str,
        client_id: &str,
        key: &str,
        packet: &Packet,
        secret: &[u8],
        outcome: eap::Outcome,
    ) -> Result<(Option<Vec<u8>>, bool)> {
        Core::radius_eap_commit(
            self, listener, nas_id, client_id, key, packet, secret, outcome,
        )
    }

    fn close_identity(&self, identity: Option<&Identity>) -> Result<()> {
        Core::radius_close(self, identity)
    }
}
