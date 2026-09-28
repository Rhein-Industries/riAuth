//! RADIUS client, identity, replay and close operations over concrete storage.

use crate::{
    core::{Core, audit},
    crypto::{digest, now},
    error::{Error, Result},
    model::{Client, Identity, Session, User},
    radius::{Packet, RadiusClaim, Settings, WireAttributes, eap, fingerprint},
    store::Tx,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Cached {
    expires_at: u64,
    client_id: String,
    client_fingerprint: String,
    attributes_fingerprint: Option<String>,
    response: Option<Vec<u8>>,
    identity: Option<Identity>,
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

impl Core {
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
}
