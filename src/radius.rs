//! RADIUS PAP/EAP-TLS with mandatory Message-Authenticator, duplicate handling and RadSec.
pub mod eap;
use crate::{
    crypto::digest,
    error::{Error, Result},
    model::{Client, Identity, User},
    validation::validate_name,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use md5::{Digest, Md5};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, UdpSocket},
    sync::{OwnedSemaphorePermit, Semaphore},
    task::{JoinHandle, JoinSet},
};

pub use crate::assembly::radius_cleanup as cleanup;
pub use crate::assembly::radius_start as start;
pub use crate::model::client_settings::radius::{Attribute, ReplyValue, Settings};
pub use crate::radius_listener::{Listener, Nas, Transport};

pub(crate) trait RadiusPort: eap::EapPort + Clone + Send + Sync + 'static {
    fn listeners(&self) -> &BTreeMap<String, Listener>;
    fn bind_listener(&self, id: &str, listener: &Listener) -> crate::capability::ListenerLease;
    fn rate_limited(&self, peer: IpAddr, listener_id: &str, nas_id: &str) -> Result<bool>;
    fn claim(
        &self,
        client_id: &str,
        key: &str,
        packet: &Packet,
        secret: &[u8],
    ) -> Result<RadiusClaim>;
    fn eap_client(&self, client_id: &str) -> Result<Option<Client>>;
    fn user_requires_mfa(&self, username: &str) -> Result<bool>;
    fn login(&self, username: String, password: String, otp: Option<String>) -> Result<Value>;
    #[allow(clippy::too_many_arguments)]
    fn pap_commit(
        &self,
        listener_id: &str,
        nas_id: &str,
        client_id: &str,
        key: &str,
        packet: &Packet,
        secret: &[u8],
        token: Option<&str>,
    ) -> Result<(Option<Vec<u8>>, bool)>;
    fn logout(&self, token: &str) -> Result<Value>;
    #[allow(clippy::too_many_arguments)]
    fn eap_commit(
        &self,
        listener: &str,
        nas_id: &str,
        client_id: &str,
        key: &str,
        packet: &Packet,
        secret: &[u8],
        outcome: eap::Outcome,
    ) -> Result<(Option<Vec<u8>>, bool)>;
    fn close_identity(&self, identity: Option<&Identity>) -> Result<()>;
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
    pub(crate) fn attributes(&self, user: &User) -> Result<Vec<(u8, Vec<u8>)>> {
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
pub(crate) fn secret(nas: &Nas, tls: bool) -> Result<zeroize::Zeroizing<String>> {
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
pub(crate) type WireAttributes = Vec<(u8, Vec<u8>)>;
pub(crate) struct Packet {
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
    pub(crate) fn attr(&self, code: u8) -> Option<&[u8]> {
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
        for chunk in ciphertext.as_chunks::<16>().0 {
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
    pub(crate) fn response(
        &self,
        code: u8,
        secret: &[u8],
        attributes: Vec<(u8, Vec<u8>)>,
    ) -> Result<Vec<u8>> {
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
pub(crate) enum RadiusClaim {
    Cached(Option<Vec<u8>>),
    New,
}
pub(crate) fn fingerprint(client: &Client) -> Result<String> {
    Ok(digest(
        &serde_json::to_string(client).map_err(Error::internal)?,
    ))
}
fn radius_packet<P: RadiusPort>(
    port: &P,
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
    let claim = port.claim(&nas.client_id, &key, &packet, secret.as_bytes())?;
    if let RadiusClaim::Cached(reply) = claim {
        return Ok(reply);
    }
    if packet.attr(79).is_some() {
        let client = port.eap_client(&nas.client_id)?;
        let outcome = match client {
            Some(client) if port.listeners()[listener_id].eap_tls.is_some() => engine.process(
                port,
                listener_id,
                nas_id,
                nas,
                &packet,
                secret.as_bytes(),
                &client,
            )?,
            _ => eap::Outcome::Reject(eap::failure(&packet)),
        };
        return radius_eap_finish(
            port,
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
        let mfa = port.user_requires_mfa(username)?;
        let (password, otp) = if mfa {
            password
                .rsplit_once(';')
                .map(|(p, o)| (p, Some(o.to_owned())))
                .unwrap_or((&password, None))
        } else {
            (password.as_str(), None)
        };
        let result = port.login(username.into(), password.into(), otp)?;
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
    let result = port.pap_commit(
        listener_id,
        nas_id,
        &nas.client_id,
        &key,
        &packet,
        secret.as_bytes(),
        token.as_deref().map(|s| s.as_str()),
    );
    if !result.as_ref().is_ok_and(|(_, ok)| *ok)
        && let Some(token) = token
    {
        let _ = port.logout(&token);
    }
    result.map(|(v, _)| v)
}

#[allow(clippy::too_many_arguments)]
fn radius_eap_finish<P: RadiusPort>(
    port: &P,
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
    let result = port.eap_commit(
        listener,
        nas_id,
        &nas.client_id,
        key,
        packet,
        secret,
        outcome,
    );
    if !result.as_ref().is_ok_and(|(_, accepted)| *accepted) && identity.is_some() {
        let _ = port.close_identity(identity.as_ref());
    }
    result.map(|(response, _)| response)
}
pub struct Servers {
    pub addresses: Vec<SocketAddr>,
    tasks: Vec<JoinHandle<()>>,
    leases: Vec<crate::capability::ListenerLease>,
}
impl Drop for Servers {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}
async fn tls_config(config: &Listener) -> anyhow::Result<Arc<rustls::ServerConfig>> {
    let config = config.clone();
    tokio::task::spawn_blocking(move || tls_material(&config)).await?
}

/// Build the same TLS verifier and server identity used by the live RadSec
/// listener. Capability preflight must reject material that cannot be served.
pub(crate) fn tls_material(config: &Listener) -> anyhow::Result<Arc<rustls::ServerConfig>> {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let mut roots = rustls::RootCertStore::empty();
    for cert in CertificateDer::pem_file_iter(config.client_ca_file.as_ref().unwrap())? {
        roots.add(cert?)?;
    }
    let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(
        Arc::new(roots),
        provider.clone(),
    )
    .build()?;
    let certs = CertificateDer::pem_file_iter(config.tls_cert_file.as_ref().unwrap())?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(config.tls_key_file.as_ref().unwrap())?;
    Ok(Arc::new(
        rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()?
            .with_client_cert_verifier(verifier)
            .with_single_cert(certs, key)?,
    ))
}
enum Bound {
    Udp(UdpSocket),
    Tls(TcpListener, Arc<rustls::ServerConfig>),
}
pub(crate) async fn start_with_port<P: RadiusPort>(core: P) -> anyhow::Result<Servers> {
    let mut bound = Vec::new();
    for (id, config) in core.listeners() {
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
        leases: Vec::new(),
    };
    for (id, config, socket) in bound {
        let core = core.clone();
        let address = match &socket {
            Bound::Udp(s) => s.local_addr()?,
            Bound::Tls(s, _) => s.local_addr()?,
        };
        servers.addresses.push(address);
        let lease = core.bind_listener(&id, &config);
        let worker_lease = lease.clone();
        servers.leases.push(lease);
        servers.tasks.push(tokio::spawn(async move {
            let _lease = worker_lease;
            _lease.running();
            match socket {
                Bound::Udp(socket) => udp(core, id, config, socket).await,
                Bound::Tls(socket, tls) => tcp(core, id, config, socket, tls).await,
            }
        }));
    }
    Ok(servers)
}
async fn process<P: RadiusPort>(
    core: P,
    id: String,
    nas_id: String,
    nas: Nas,
    bytes: Vec<u8>,
    tls: bool,
    engine: Arc<eap::Engine>,
) -> Option<Vec<u8>> {
    tokio::task::spawn_blocking(move || {
        if core.rate_limited(nas.peer, &id, &nas_id).ok()? {
            return None;
        }
        radius_packet(&core, &id, &nas_id, &nas, &bytes, tls, &engine)
            .ok()
            .flatten()
    })
    .await
    .ok()
    .flatten()
}
async fn udp<P: RadiusPort>(core: P, id: String, config: Listener, socket: UdpSocket) {
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
const RADSEC_HANDSHAKES: usize = 64;
const RADSEC_HANDSHAKES_PER_PEER: usize = 4;
const RADSEC_ACCEPTS_PER_WINDOW: u32 = 16;
const RADSEC_ACCEPT_WINDOW: Duration = Duration::from_secs(5);
const RADSEC_SESSIONS: usize = 64;
const RADSEC_SESSIONS_PER_NAS: usize = 8;

struct RadsecPeerBudget {
    slots: Arc<Semaphore>,
    accepts: Mutex<(Instant, u32)>,
}
struct RadsecBudget {
    handshakes: Arc<Semaphore>,
    sessions: Arc<Semaphore>,
    peers: BTreeMap<IpAddr, RadsecPeerBudget>,
    nas: BTreeMap<String, Arc<Semaphore>>,
}
impl RadsecBudget {
    fn new(nas: &BTreeMap<String, Nas>) -> Self {
        Self {
            handshakes: Arc::new(Semaphore::new(RADSEC_HANDSHAKES)),
            sessions: Arc::new(Semaphore::new(RADSEC_SESSIONS)),
            peers: nas
                .values()
                .map(|nas| {
                    (
                        nas.peer,
                        RadsecPeerBudget {
                            slots: Arc::new(Semaphore::new(RADSEC_HANDSHAKES_PER_PEER)),
                            accepts: Mutex::new((Instant::now(), 0)),
                        },
                    )
                })
                .collect(),
            nas: nas
                .keys()
                .map(|id| {
                    (
                        id.clone(),
                        Arc::new(Semaphore::new(RADSEC_SESSIONS_PER_NAS)),
                    )
                })
                .collect(),
        }
    }
    fn handshake(&self, peer: IpAddr) -> Option<(OwnedSemaphorePermit, OwnedSemaphorePermit)> {
        let peer = self.peers.get(&peer)?;
        let mut accepts = peer
            .accepts
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let now = Instant::now();
        if now.duration_since(accepts.0) >= RADSEC_ACCEPT_WINDOW {
            *accepts = (now, 0);
        }
        if accepts.1 >= RADSEC_ACCEPTS_PER_WINDOW {
            return None;
        }
        accepts.1 += 1;
        // Per-source admission precedes the shared pool and TLS processing.
        let peer_permit = peer.slots.clone().try_acquire_owned().ok()?;
        let global_permit = self.handshakes.clone().try_acquire_owned().ok()?;
        Some((peer_permit, global_permit))
    }
    fn session(&self, nas: &str) -> Option<(OwnedSemaphorePermit, OwnedSemaphorePermit)> {
        let nas_permit = self.nas.get(nas)?.clone().try_acquire_owned().ok()?;
        let global_permit = self.sessions.clone().try_acquire_owned().ok()?;
        Some((nas_permit, global_permit))
    }
}

#[allow(clippy::too_many_arguments)]
async fn radsec_connection<P: RadiusPort>(
    core: P,
    id: String,
    config: Listener,
    tls: Arc<rustls::ServerConfig>,
    stream: tokio::net::TcpStream,
    peer: IpAddr,
    budgets: Arc<RadsecBudget>,
    handshake_permits: (OwnedSemaphorePermit, OwnedSemaphorePermit),
    engine: Arc<eap::Engine>,
) -> anyhow::Result<()> {
    let mut stream = tokio::time::timeout(
        Duration::from_secs(5),
        tokio_rustls::TlsAcceptor::from(tls).accept(stream),
    )
    .await??;
    let certificates = stream
        .get_ref()
        .1
        .peer_certificates()
        .ok_or_else(|| anyhow::anyhow!("Missing RadSec certificate"))?;
    let certificate = certificates
        .first()
        .ok_or_else(|| anyhow::anyhow!("Missing RadSec certificate"))?;
    use sha2::Digest as ShaDigest;
    let pin = URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(certificate.as_ref()));
    let (nas_id, nas) = config
        .nas
        .iter()
        .find(|(_, nas)| nas.peer == peer && nas.certificate_sha256.as_deref() == Some(&pin))
        .ok_or_else(|| anyhow::anyhow!("Unregistered RadSec certificate"))?;
    // Only a certificate-pinned NAS can consume authenticated capacity. Keep
    // these budgets independent so stalled handshakes cannot displace sessions.
    let _session_permits = budgets
        .session(nas_id)
        .ok_or_else(|| anyhow::anyhow!("RadSec authenticated capacity is busy"))?;
    drop(handshake_permits);
    for _ in 0..1000 {
        let mut header = [0; 4];
        tokio::time::timeout(Duration::from_secs(30), stream.read_exact(&mut header)).await??;
        let len = u16::from_be_bytes([header[2], header[3]]) as usize;
        if !(20..=4096).contains(&len) {
            break;
        }
        let mut bytes = vec![0; len];
        bytes[..4].copy_from_slice(&header);
        tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut bytes[4..])).await??;
        if let Some(reply) = process(
            core.clone(),
            id.clone(),
            nas_id.clone(),
            nas.clone(),
            bytes,
            true,
            engine.clone(),
        )
        .await
        {
            tokio::time::timeout(Duration::from_secs(5), stream.write_all(&reply)).await??;
        }
    }
    Ok(())
}

async fn tcp<P: RadiusPort>(
    core: P,
    id: String,
    config: Listener,
    socket: TcpListener,
    mut tls: Arc<rustls::ServerConfig>,
) {
    let engine = Arc::new(eap::Engine::default());
    let budgets = Arc::new(RadsecBudget::new(&config.nas));
    let mut jobs = JoinSet::new();
    let mut reload = tokio::time::interval(Duration::from_secs(60));
    loop {
        tokio::select! {
            accepted = socket.accept() => {
                let Ok((stream, peer)) = accepted else { break };
                let Some(handshake_permits) = budgets.handshake(peer.ip()) else { continue };
                let (core, id, config, tls, budgets, engine) = (
                    core.clone(), id.clone(), config.clone(), tls.clone(), budgets.clone(), engine.clone(),
                );
                jobs.spawn(async move {
                    let _ = radsec_connection(core, id, config, tls, stream, peer.ip(), budgets, handshake_permits, engine).await;
                });
            },
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

#[cfg(test)]
mod admission_tests {
    use super::*;

    fn budget() -> RadsecBudget {
        RadsecBudget::new(&BTreeMap::from([
            (
                "nas-a".into(),
                Nas {
                    peer: "192.0.2.1".parse().unwrap(),
                    client_id: "network".into(),
                    shared_secret_file: None,
                    certificate_sha256: None,
                },
            ),
            (
                "nas-b".into(),
                Nas {
                    peer: "192.0.2.2".parse().unwrap(),
                    client_id: "network".into(),
                    shared_secret_file: None,
                    certificate_sha256: None,
                },
            ),
        ]))
    }

    #[test]
    fn one_radsec_peer_cannot_exhaust_handshake_or_authenticated_capacity() {
        let budgets = budget();
        let peer_a = "192.0.2.1".parse().unwrap();
        let peer_b = "192.0.2.2".parse().unwrap();
        let pending = (0..RADSEC_HANDSHAKES_PER_PEER)
            .map(|_| budgets.handshake(peer_a).unwrap())
            .collect::<Vec<_>>();
        assert!(budgets.handshake(peer_a).is_none());
        let other_peer = budgets.handshake(peer_b).unwrap();
        assert_eq!(budgets.sessions.available_permits(), RADSEC_SESSIONS);
        assert!(budgets.handshake("192.0.2.3".parse().unwrap()).is_none());
        drop((pending, other_peer));
        assert_eq!(budgets.handshakes.available_permits(), RADSEC_HANDSHAKES);

        let authenticated = (0..RADSEC_SESSIONS_PER_NAS)
            .map(|_| budgets.session("nas-a").unwrap())
            .collect::<Vec<_>>();
        assert!(budgets.session("nas-a").is_none());
        let other_nas = budgets.session("nas-b").unwrap();
        assert!(budgets.handshake(peer_b).is_some());
        drop((authenticated, other_nas));
        assert_eq!(budgets.sessions.available_permits(), RADSEC_SESSIONS);
    }

    #[test]
    fn radsec_accept_rate_is_per_peer_and_recovers_after_its_window() {
        let budgets = budget();
        let peer_a = "192.0.2.1".parse().unwrap();
        let peer_b = "192.0.2.2".parse().unwrap();
        for _ in 0..RADSEC_ACCEPTS_PER_WINDOW {
            drop(budgets.handshake(peer_a).unwrap());
        }
        assert!(budgets.handshake(peer_a).is_none());
        assert!(budgets.handshake(peer_b).is_some());
        budgets.peers[&peer_a].accepts.lock().unwrap().0 = Instant::now() - RADSEC_ACCEPT_WINDOW;
        assert!(budgets.handshake(peer_a).is_some());
    }
}
