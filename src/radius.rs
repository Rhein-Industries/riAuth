//! RADIUS PAP/EAP-TLS with mandatory Message-Authenticator, duplicate handling and RadSec.
pub mod eap;
use crate::{
    core::{Core, audit, validate_name},
    crypto::{digest, now},
    error::{Error, Result},
    model::{Client, Identity, Session, User},
    store::Tx,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, UdpSocket},
    sync::Semaphore,
    task::{JoinHandle, JoinSet},
};

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ReplyValue {
    Text(String),
    Integer(u32),
    Ipv4(Ipv4Addr),
    Attribute(String),
}
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Attribute {
    Standard {
        code: u8,
        value: ReplyValue,
    },
    Vendor {
        vendor: u32,
        code: u8,
        value: ReplyValue,
    },
}
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(default)]
    pub eap_tls: bool,
    #[serde(default)]
    pub reply: Vec<Attribute>,
}
impl Settings {
    pub fn validate(&self, client: &Client) -> Result<()> {
        if client.service || !client.scopes.contains("radius") || self.reply.len() > 32 {
            return Err(Error::bad(
                "RADIUS requires an interactive policy client, radius scope and at most 32 reply attributes",
            ));
        }
        for a in &self.reply {
            let value = match a {
                Attribute::Standard { code, value } => {
                    if ![
                        6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 19, 20, 22, 23, 25, 27, 28, 29,
                        34, 35, 37, 38, 39, 64, 65, 81,
                    ]
                    .contains(code)
                    {
                        return Err(Error::bad(
                            "RADIUS attribute is reserved or outside this reply profile",
                        ));
                    }
                    if self.eap_tls && *code == 18 {
                        return Err(Error::bad("EAP replies cannot carry Reply-Message"));
                    }
                    validate_type(*code, value)?;
                    value
                }
                Attribute::Vendor {
                    vendor,
                    code,
                    value,
                } => {
                    if *vendor == 0 || *code == 0 || *vendor == 311 && [16, 17].contains(code) {
                        return Err(Error::bad("Invalid or reserved RADIUS vendor attribute"));
                    }
                    value
                }
            };
            match value {
                ReplyValue::Text(v)
                    if v.is_empty() || v.len() > 200 || v.chars().any(char::is_control) =>
                {
                    return Err(Error::bad("Invalid RADIUS text value"));
                }
                ReplyValue::Attribute(name) => {
                    validate_name(name)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn attributes(&self, user: &User) -> Result<Vec<(u8, Vec<u8>)>> {
        let mut attrs = Vec::new();
        for attr in &self.reply {
            let (vendor, code, value) = match attr {
                Attribute::Standard { code, value } => (None, *code, value),
                Attribute::Vendor {
                    vendor,
                    code,
                    value,
                } => (Some(*vendor), *code, value),
            };
            let resolved = match value {
                ReplyValue::Attribute(name) => match user.attributes.get(name) {
                    Some(serde_json::Value::String(v))
                        if !v.is_empty() && v.len() <= 200 && !v.chars().any(char::is_control) =>
                    {
                        ReplyValue::Text(v.clone())
                    }
                    Some(serde_json::Value::Number(v)) => ReplyValue::Integer(
                        u32::try_from(
                            v.as_u64()
                                .ok_or_else(|| Error::bad("RADIUS attribute must be unsigned"))?,
                        )
                        .map_err(|_| Error::bad("RADIUS attribute is too large"))?,
                    ),
                    _ => {
                        return Err(Error::bad(
                            "RADIUS reply attribute requires a configured text or integer user attribute",
                        ));
                    }
                },
                value => value.clone(),
            };
            if vendor.is_none() {
                validate_type(code, &resolved)?;
            }
            let mut bytes = match resolved {
                ReplyValue::Text(s) => s.into_bytes(),
                ReplyValue::Integer(n) => n.to_be_bytes().to_vec(),
                ReplyValue::Ipv4(ip) => ip.octets().to_vec(),
                ReplyValue::Attribute(_) => unreachable!(),
            };
            // One untagged tunnel. Integer attributes encode tag zero in their high byte;
            // Tunnel-Private-Group-ID requires a separate tag octet before the string.
            if vendor.is_none() && code == 81 {
                bytes.insert(0, 0);
            }
            if let Some(vendor) = vendor {
                let mut encoded = vendor.to_be_bytes().to_vec();
                encoded.extend([code, (bytes.len() + 2) as u8]);
                encoded.extend(bytes);
                attrs.push((26, encoded));
            } else {
                attrs.push((code, bytes));
            }
        }
        Ok(attrs)
    }
}
fn validate_type(code: u8, value: &ReplyValue) -> Result<()> {
    if matches!(value, ReplyValue::Attribute(_)) {
        return Ok(());
    }
    let valid = match code {
        8 | 9 | 14 => matches!(value, ReplyValue::Ipv4(_)),
        64 | 65 => matches!(value, ReplyValue::Integer(n) if *n <= 0x00ff_ffff),
        6 | 7 | 10 | 12 | 13 | 15 | 16 | 19 | 23 | 27 | 28 | 29 | 37 | 38 => {
            matches!(value, ReplyValue::Integer(_))
        }
        _ => matches!(value, ReplyValue::Text(_)),
    };
    if !valid {
        return Err(Error::bad(
            "RADIUS reply value has the wrong wire type for this attribute",
        ));
    }
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Udp,
    Tls,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Nas {
    pub peer: IpAddr,
    pub client_id: String,
    pub shared_secret_file: Option<PathBuf>,
    pub certificate_sha256: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Listener {
    #[serde(default)]
    pub eap_tls: Option<eap::Config>,
    pub listen: SocketAddr,
    pub transport: Transport,
    pub nas: BTreeMap<String, Nas>,
    pub tls_cert_file: Option<PathBuf>,
    pub tls_key_file: Option<PathBuf>,
    pub client_ca_file: Option<PathBuf>,
}
impl Listener {
    pub fn validate(&self) -> Result<()> {
        if let Some(eap) = &self.eap_tls {
            eap.validate()?;
        }
        if self.nas.is_empty() || self.nas.len() > 128 {
            return Err(Error::bad(
                "RADIUS requires one to 128 explicit NAS entries",
            ));
        }
        let mut peers = BTreeSet::new();
        for (id, nas) in &self.nas {
            validate_name(id)?;
            validate_name(&nas.client_id)?;
            if nas.peer.is_unspecified() || nas.peer.is_multicast() {
                return Err(Error::bad("RADIUS NAS needs an explicit unicast peer IP"));
            }
            match self.transport {
                Transport::Udp => {
                    if nas.shared_secret_file.is_none()
                        || nas.certificate_sha256.is_some()
                        || !peers.insert(nas.peer.to_string())
                    {
                        return Err(Error::bad(
                            "UDP RADIUS requires one shared-secret NAS per peer IP",
                        ));
                    }
                }
                Transport::Tls => {
                    let pin = nas.certificate_sha256.as_deref().unwrap_or("");
                    if nas.shared_secret_file.is_some()
                        || !URL_SAFE_NO_PAD.decode(pin).is_ok_and(|b| b.len() == 32)
                        || pin.len() != 43
                        || !peers.insert(format!("{}:{pin}", nas.peer))
                    {
                        return Err(Error::bad(
                            "RadSec requires a unique peer/certificate pin and uses the fixed radsec protocol secret",
                        ));
                    }
                }
            }
        }
        let files = [
            &self.tls_cert_file,
            &self.tls_key_file,
            &self.client_ca_file,
        ];
        if (self.transport == Transport::Tls && files.iter().any(|f| f.is_none()))
            || (self.transport == Transport::Udp && files.iter().any(|f| f.is_some()))
        {
            return Err(Error::bad(
                "RadSec requires server certificate/key and private client CA files; UDP has no TLS files",
            ));
        }
        Ok(())
    }
}
fn secret(nas: &Nas, tls: bool) -> Result<zeroize::Zeroizing<String>> {
    if tls {
        return Ok(zeroize::Zeroizing::new("radsec".into()));
    }
    let value = crate::config::read_private_secret(
        nas.shared_secret_file
            .as_ref()
            .ok_or_else(Error::forbidden)?,
        1024,
    )
    .map_err(Error::internal)?;
    let value = value.trim_end_matches(['\r', '\n']);
    if value.len() < 32 || !value.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(Error::bad(
            "RADIUS shared secret requires 32..1024 non-whitespace ASCII bytes in a private file",
        ));
    }
    Ok(zeroize::Zeroizing::new(value.into()))
}
type WireAttributes = Vec<(u8, Vec<u8>)>;
struct Packet {
    identifier: u8,
    authenticator: [u8; 16],
    attrs: Vec<(u8, Vec<u8>)>,
}
fn decode(bytes: &[u8], secret: &[u8]) -> Result<Packet> {
    if bytes.len() < 20
        || bytes.len() > 4096
        || bytes[0] != 1
        || u16::from_be_bytes([bytes[2], bytes[3]]) as usize != bytes.len()
    {
        return Err(Error::bad("Invalid RADIUS packet"));
    }
    let mut attrs = Vec::new();
    let mut offset = 20;
    let mut ma = None;
    let mut singleton = BTreeSet::new();
    while offset < bytes.len() {
        if offset + 2 > bytes.len() {
            return Err(Error::bad("Truncated RADIUS attribute"));
        }
        let code = bytes[offset];
        let len = bytes[offset + 1] as usize;
        if code == 0 || len < 2 || offset + len > bytes.len() {
            return Err(Error::bad("Invalid RADIUS attribute length"));
        }
        if [1, 2, 3, 24, 80].contains(&code) && !singleton.insert(code) {
            return Err(Error::bad("Duplicate RADIUS credential or authenticator"));
        }
        if code == 80 {
            if len != 18 {
                return Err(Error::bad("Invalid Message-Authenticator"));
            }
            ma = Some(offset + 2);
        }
        attrs.push((code, bytes[offset + 2..offset + len].to_vec()));
        offset += len;
    }
    let ma = ma.ok_or_else(|| Error::bad("Message-Authenticator is mandatory"))?;
    let mut signed = bytes.to_vec();
    signed[ma..ma + 16].fill(0);
    let mut mac = Hmac::<Md5>::new_from_slice(secret).map_err(Error::internal)?;
    mac.update(&signed);
    mac.verify_slice(&bytes[ma..ma + 16])
        .map_err(|_| Error::forbidden())?;
    Ok(Packet {
        identifier: bytes[1],
        authenticator: bytes[4..20].try_into().unwrap(),
        attrs,
    })
}
impl Packet {
    fn attr(&self, code: u8) -> Option<&[u8]> {
        self.attrs
            .iter()
            .find(|(t, _)| *t == code)
            .map(|(_, v)| v.as_slice())
    }
    fn password(&self, secret: &[u8]) -> Result<zeroize::Zeroizing<String>> {
        let ciphertext = self
            .attr(2)
            .ok_or_else(|| Error::bad("PAP User-Password is required"))?;
        if ciphertext.is_empty() || ciphertext.len() > 128 || ciphertext.len() % 16 != 0 {
            return Err(Error::bad("Invalid PAP password length"));
        }
        let mut plain = zeroize::Zeroizing::new(Vec::new());
        let mut previous = self.authenticator.as_slice();
        for chunk in ciphertext.chunks_exact(16) {
            let mut md5 = Md5::new();
            md5.update(secret);
            md5.update(previous);
            let key = md5.finalize();
            plain.extend(chunk.iter().zip(key).map(|(a, b)| a ^ b));
            previous = chunk;
        }
        let end = plain.iter().position(|b| *b == 0).unwrap_or(plain.len());
        if end == 0 || plain[end..].iter().any(|b| *b != 0) {
            return Err(Error::bad("Invalid PAP password padding"));
        }
        Ok(zeroize::Zeroizing::new(
            std::str::from_utf8(&plain[..end])
                .map_err(|_| Error::bad("PAP passwords must be UTF-8"))?
                .to_owned(),
        ))
    }
    fn response(&self, code: u8, secret: &[u8], attributes: Vec<(u8, Vec<u8>)>) -> Result<Vec<u8>> {
        let mut bytes = vec![code, self.identifier, 0, 0];
        bytes.extend(self.authenticator);
        bytes.extend([80, 18]);
        bytes.extend([0; 16]);
        for (code, value) in attributes
            .into_iter()
            .chain(self.attrs.iter().filter(|(code, _)| *code == 33).cloned())
        {
            if value.len() > 253 {
                return Err(Error::bad("RADIUS response attribute is too long"));
            }
            bytes.extend([code, (value.len() + 2) as u8]);
            bytes.extend(value);
        }
        if bytes.len() > 4096 {
            return Err(Error::bad("RADIUS response exceeds packet limit"));
        }
        let len = (bytes.len() as u16).to_be_bytes();
        bytes[2..4].copy_from_slice(&len);
        let mut mac = Hmac::<Md5>::new_from_slice(secret).map_err(Error::internal)?;
        mac.update(&bytes);
        bytes[22..38].copy_from_slice(&mac.finalize().into_bytes());
        let mut md5 = Md5::new();
        md5.update(&bytes);
        md5.update(secret);
        bytes[4..20].copy_from_slice(&md5.finalize());
        Ok(bytes)
    }
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
fn client(tx: &Tx<'_>, id: &str) -> Result<(Client, Settings)> {
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
fn fingerprint(client: &Client) -> Result<String> {
    Ok(digest(
        &serde_json::to_string(client).map_err(Error::internal)?,
    ))
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
    fn radius_identity(&self, tx: &Tx<'_>, client: &Client, identity: &Identity) -> Result<User> {
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

    fn radius_packet(
        &self,
        listener_id: &str,
        nas_id: &str,
        nas: &Nas,
        bytes: &[u8],
        tls: bool,
        engine: &eap::Engine,
    ) -> Result<Option<Vec<u8>>> {
        let secret = secret(nas, tls)?;
        let packet = decode(bytes, secret.as_bytes())?;
        let key = digest(&format!(
            "{listener_id}\0{nas_id}\0{}\0{}",
            digest(&secret),
            URL_SAFE_NO_PAD.encode(bytes)
        ));
        enum Claim {
            Cached(Option<Vec<u8>>),
            New,
        }
        let claim = self.store.write(|tx| {
            let (client, settings) = client(tx, &nas.client_id)?;
            let fp = fingerprint(&client)?;
            if let Some(mut cached) = tx.get::<Cached>("radius_requests", &key)? {
                if cached.expires_at > now() {
                    if cached.client_id != nas.client_id
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
                        cached.response =
                            Some(packet.response(3, secret.as_bytes(), eap::failure(&packet))?);
                        tx.put("radius_requests", &key, &cached)?;
                    }
                    return Ok(Claim::Cached(cached.response));
                }
                close(tx, cached.identity.as_ref())?;
            }
            if tx.list::<Cached>("radius_requests")?.len() >= 10000 {
                return Err(Error::conflict("RADIUS duplicate cache is full"));
            }
            tx.put(
                "radius_requests",
                &key,
                &Cached {
                    expires_at: now() + 90,
                    client_id: nas.client_id.clone(),
                    client_fingerprint: fp,
                    attributes_fingerprint: None,
                    response: None,
                    identity: None,
                },
            )?;
            Ok(Claim::New)
        })?;
        if let Claim::Cached(reply) = claim {
            return Ok(reply);
        }
        if packet.attr(79).is_some() {
            let client = self.store.read(|tx| {
                let (client, settings) = client(tx, &nas.client_id)?;
                Ok(settings.eap_tls.then_some(client))
            })?;
            let outcome = match client {
                Some(client) if self.config.radius_listeners[listener_id].eap_tls.is_some() => {
                    engine.process(
                        self,
                        listener_id,
                        nas_id,
                        nas,
                        &packet,
                        secret.as_bytes(),
                        &client,
                    )?
                }
                _ => eap::Outcome::Reject(eap::failure(&packet)),
            };
            return self.radius_eap_finish(
                listener_id,
                nas_id,
                nas,
                &key,
                &packet,
                secret.as_bytes(),
                outcome,
            );
        }
        let authenticated = (|| -> Result<String> {
            if packet.attr(3).is_some() || packet.attr(24).is_some() || packet.attr(79).is_some() {
                return Err(Error::bad("This RADIUS profile requires PAP"));
            }
            let username = std::str::from_utf8(
                packet
                    .attr(1)
                    .ok_or_else(|| Error::bad("Missing RADIUS username"))?,
            )
            .map_err(|_| Error::bad("Invalid RADIUS username"))?;
            validate_name(username)?;
            let password = packet.password(secret.as_bytes())?;
            let mfa = self
                .store
                .read(|tx| match crate::core::user_by_name(tx, username) {
                    Ok(user) => Ok(user.totp_secret.is_some()),
                    Err(e) if e.status.as_u16() == 404 => Ok(false),
                    Err(e) => Err(e),
                })?;
            let (password, otp) = if mfa {
                password
                    .rsplit_once(';')
                    .map(|(p, o)| (p, Some(o.to_owned())))
                    .unwrap_or((&password, None))
            } else {
                (password.as_str(), None)
            };
            let result = self.login(username.into(), password.into(), otp)?;
            Ok(result["session_token"]
                .as_str()
                .ok_or_else(Error::unauthorized)?
                .to_owned())
        })();
        if authenticated
            .as_ref()
            .is_err_and(|e| e.status.as_u16() >= 500 || e.status.as_u16() == 429)
        {
            return Ok(None);
        }
        let token = authenticated.ok().map(zeroize::Zeroizing::new);
        let result = self.store.write(|tx| {
            let mut cached = tx
                .get::<Cached>("radius_requests", &key)?
                .filter(|c| c.expires_at > now() && c.response.is_none())
                .ok_or_else(|| Error::conflict("RADIUS request expired"))?;
            let allowed = (|| -> Result<(Identity, WireAttributes)> {
                let (client, settings) = client(tx, &nas.client_id)?;
                if fingerprint(&client)? != cached.client_fingerprint {
                    return Err(Error::forbidden());
                }
                let (_, session) = self.session(
                    tx,
                    token
                        .as_deref()
                        .map(|s| s.as_str())
                        .ok_or_else(Error::unauthorized)?,
                )?;
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
            let response = packet.response(code, secret.as_bytes(), attrs)?;
            cached.response = Some(response.clone());
            tx.put("radius_requests", &key, &cached)?;
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
        });
        if !result.as_ref().is_ok_and(|(_, ok)| *ok)
            && let Some(token) = token
        {
            let _ = self.logout(&token);
        }
        result.map(|(v, _)| v)
    }

    #[allow(clippy::too_many_arguments)]
    fn radius_eap_finish(
        &self,
        listener: &str,
        nas_id: &str,
        nas: &Nas,
        key: &str,
        packet: &Packet,
        secret: &[u8],
        outcome: eap::Outcome,
    ) -> Result<Option<Vec<u8>>> {
        let identity = match &outcome {
            eap::Outcome::Accept(identity, _) => Some(identity.clone()),
            _ => None,
        };
        let result = self.store.write(|tx| {
            let mut cached = tx
                .get::<Cached>("radius_requests", key)?
                .filter(|c| c.expires_at > now() && c.response.is_none())
                .ok_or_else(|| Error::conflict("RADIUS request expired"))?;
            let (client, settings) = client(tx, &nas.client_id)?;
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
        });
        if !result.as_ref().is_ok_and(|(_, accepted)| *accepted) && identity.is_some() {
            let _ = self.store.write(|tx| close(tx, identity.as_ref()));
        }
        result.map(|(response, _)| response)
    }
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    eap::cleanup(tx, at)?;
    for (id, cached) in tx.maintenance_page::<Cached>("radius_requests")? {
        if cached.expires_at < at {
            close(tx, cached.identity.as_ref())?;
            tx.delete("radius_requests", &id)?;
        }
    }
    Ok(())
}
pub struct Servers {
    pub addresses: Vec<SocketAddr>,
    tasks: Vec<JoinHandle<()>>,
}
impl Drop for Servers {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}
async fn tls_config(config: &Listener) -> anyhow::Result<Arc<rustls::ServerConfig>> {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
    let config = config.clone();
    tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        let mut roots = rustls::RootCertStore::empty();
        for cert in CertificateDer::pem_file_iter(config.client_ca_file.unwrap())? {
            roots.add(cert?)?;
        }
        let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(
            Arc::new(roots),
            provider.clone(),
        )
        .build()?;
        let certs = CertificateDer::pem_file_iter(config.tls_cert_file.unwrap())?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let key = PrivateKeyDer::from_pem_file(config.tls_key_file.unwrap())?;
        Ok(Arc::new(
            rustls::ServerConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()?
                .with_client_cert_verifier(verifier)
                .with_single_cert(certs, key)?,
        ))
    })
    .await?
}
enum Bound {
    Udp(UdpSocket),
    Tls(TcpListener, Arc<rustls::ServerConfig>),
}
pub async fn start(core: Core) -> anyhow::Result<Servers> {
    let mut bound = Vec::new();
    for (id, config) in &core.config.radius_listeners {
        config.validate()?;
        if let Some(config) = config.eap_tls.clone() {
            tokio::task::spawn_blocking(move || config.load()).await??;
        }
        if config.transport == Transport::Udp {
            for nas in config.nas.values() {
                secret(nas, false)?;
            }
        }
        let socket = match config.transport {
            Transport::Udp => Bound::Udp(UdpSocket::bind(config.listen).await?),
            Transport::Tls => Bound::Tls(
                TcpListener::bind(config.listen).await?,
                tls_config(config).await?,
            ),
        };
        bound.push((id.clone(), config.clone(), socket));
    }
    let mut servers = Servers {
        addresses: Vec::new(),
        tasks: Vec::new(),
    };
    for (id, config, socket) in bound {
        let core = core.clone();
        let address = match &socket {
            Bound::Udp(s) => s.local_addr()?,
            Bound::Tls(s, _) => s.local_addr()?,
        };
        servers.addresses.push(address);
        servers.tasks.push(tokio::spawn(async move {
            match socket {
                Bound::Udp(socket) => udp(core, id, config, socket).await,
                Bound::Tls(socket, tls) => tcp(core, id, config, socket, tls).await,
            }
        }));
    }
    Ok(servers)
}
async fn process(
    core: Core,
    id: String,
    nas_id: String,
    nas: Nas,
    bytes: Vec<u8>,
    tls: bool,
    engine: Arc<eap::Engine>,
) -> Option<Vec<u8>> {
    tokio::task::spawn_blocking(move || {
        if core
            .store
            .shared_rate_limit(nas.peer, &format!("radius:{id}/{nas_id}"), 600)
            .ok()?
        {
            return None;
        }
        core.radius_packet(&id, &nas_id, &nas, &bytes, tls, &engine)
            .ok()
            .flatten()
    })
    .await
    .ok()
    .flatten()
}
async fn udp(core: Core, id: String, config: Listener, socket: UdpSocket) {
    let engine = Arc::new(eap::Engine::default());
    let socket = Arc::new(socket);
    let slots = Arc::new(Semaphore::new(8));
    let mut jobs = JoinSet::new();
    let mut bytes = [0u8; 4097];
    loop {
        tokio::select! {
            received=socket.recv_from(&mut bytes)=>{let Ok((len,peer))=received else{break};if len>4096{continue;}let Some((nas_id,nas))=config.nas.iter().find(|(_,n)|n.peer==peer.ip())else{continue};let Ok(permit)=slots.clone().try_acquire_owned()else{continue};let (nas_id,nas,core,id,socket,bytes)=(nas_id.clone(),nas.clone(),core.clone(),id.clone(),socket.clone(),bytes[..len].to_vec());let engine=engine.clone();jobs.spawn(async move{let _permit=permit;if let Some(response)=process(core,id,nas_id,nas,bytes,false,engine).await{let _=socket.send_to(&response,peer).await;}});},
            _=jobs.join_next(),if !jobs.is_empty()=>{},
        }
    }
}
async fn tcp(
    core: Core,
    id: String,
    config: Listener,
    socket: TcpListener,
    mut tls: Arc<rustls::ServerConfig>,
) {
    let engine = Arc::new(eap::Engine::default());
    let slots = Arc::new(Semaphore::new(64));
    let mut jobs = JoinSet::new();
    let mut reload = tokio::time::interval(Duration::from_secs(60));
    loop {
        tokio::select! {
            accepted=socket.accept()=>{let Ok((stream,peer))=accepted else{break};if !config.nas.values().any(|n|n.peer==peer.ip()){continue;}let Ok(permit)=slots.clone().try_acquire_owned()else{continue};let(core,id,config,tls)=(core.clone(),id.clone(),config.clone(),tls.clone());let engine=engine.clone();jobs.spawn(async move{let _permit=permit;let _=async{
                let mut stream=tokio::time::timeout(Duration::from_secs(5),tokio_rustls::TlsAcceptor::from(tls).accept(stream)).await??;
                let certificates=stream.get_ref().1.peer_certificates().ok_or_else(||anyhow::anyhow!("Missing RadSec certificate"))?;let certificate=certificates.first().ok_or_else(||anyhow::anyhow!("Missing RadSec certificate"))?;
                use sha2::Digest as ShaDigest;let pin=URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(certificate.as_ref()));let (nas_id,nas)=config.nas.iter().find(|(_,n)|n.peer==peer.ip()&&n.certificate_sha256.as_deref()==Some(&pin)).ok_or_else(||anyhow::anyhow!("Unregistered RadSec certificate"))?;
                for _ in 0..1000 {let mut header=[0;4];tokio::time::timeout(Duration::from_secs(30),stream.read_exact(&mut header)).await??;let len=u16::from_be_bytes([header[2],header[3]]) as usize;if !(20..=4096).contains(&len){break;}let mut bytes=vec![0;len];bytes[..4].copy_from_slice(&header);tokio::time::timeout(Duration::from_secs(5),stream.read_exact(&mut bytes[4..])).await??;if let Some(reply)=process(core.clone(),id.clone(),nas_id.clone(),nas.clone(),bytes,true,engine.clone()).await{tokio::time::timeout(Duration::from_secs(5),stream.write_all(&reply)).await??;}}
                anyhow::Ok(())
            }.await;});},
            _=reload.tick()=>{match tls_config(&config).await{Ok(next)=>tls=next,Err(_)=>tracing::warn!("RadSec certificate reload failed; retaining current configuration")}},
            _=jobs.join_next(),if !jobs.is_empty()=>{},
        }
    }
}

#[cfg(feature = "fuzzing")]
pub(crate) fn fuzz_packet(data: &[u8]) {
    let _ = decode(data, b"fuzz-fixture-secret-not-a-credential");
    // Exercise EAP framing even when the outer RADIUS authenticator is invalid.
    eap::fuzz_message(data);
}
