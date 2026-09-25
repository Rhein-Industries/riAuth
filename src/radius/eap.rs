//! EAP-TLS 1.2/1.3: pinned certificate identities, bounded fragmentation and MPPE delivery.
use super::{Nas, Packet, WireAttributes};
use crate::{
    core::{Core, audit, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Client, Identity, Session, User},
    store::Tx,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use md5::{Digest, Md5};
use rustls::pki_types::{
    CertificateDer, CertificateRevocationListDer, PrivateKeyDer, UnixTime, pem::PemObject,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub const CERTIFICATE_ACR: &str = "urn:riauth:acr:certificate";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub certificate_file: PathBuf,
    pub key_file: PathBuf,
    pub client_ca_file: PathBuf,
    pub client_crl_file: PathBuf,
    pub ocsp_response_file: Option<PathBuf>,
    #[serde(default)]
    pub tls12: bool,
    #[serde(default = "fragment_size")]
    pub fragment_size: usize,
}
fn fragment_size() -> usize {
    1024
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        if !(256..=1400).contains(&self.fragment_size) {
            return Err(Error::bad("EAP-TLS fragment_size must be 256..1400 bytes"));
        }
        Ok(())
    }
    fn material(&self) -> Result<Material> {
        self.validate()?;
        let read = |file: &PathBuf| -> Result<Vec<CertificateDer<'static>>> {
            let mut bytes = Vec::new();
            std::fs::File::open(file)
                .map_err(Error::internal)?
                .take(65537)
                .read_to_end(&mut bytes)
                .map_err(Error::internal)?;
            if bytes.len() > 65536 {
                return Err(Error::bad("EAP certificate file exceeds 64 KiB"));
            }
            let certs = CertificateDer::pem_slice_iter(&bytes)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(Error::internal)?;
            if certs.is_empty() || certs.len() > 16 {
                return Err(Error::bad("EAP certificate file needs 1..16 certificates"));
            }
            Ok(certs)
        };
        let certs = read(&self.certificate_file)?;
        let roots = read(&self.client_ca_file)?;
        let crl_bytes = read_bounded(&self.client_crl_file, 1024 * 1024)?;
        let crls = CertificateRevocationListDer::pem_slice_iter(&crl_bytes)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::internal)?;
        if crls.is_empty() || crls.len() > 16 {
            return Err(Error::bad(
                "EAP requires 1..16 current PEM CRLs covering the client chain",
            ));
        }
        for crl in &crls {
            let (trailing, parsed) = x509_parser::parse_x509_crl(crl.as_ref())
                .map_err(|_| Error::bad("Invalid EAP CRL"))?;
            let at = now() as i64;
            if !trailing.is_empty()
                || parsed.last_update().timestamp() > at
                || parsed.next_update().is_none_or(|t| t.timestamp() <= at)
            {
                return Err(Error::bad(
                    "EAP CRL is stale, not yet valid or has no nextUpdate",
                ));
            }
        }
        let fp = digest(&format!(
            "{}\0{}\0{}",
            serde_json::to_string(self).map_err(Error::internal)?,
            certs
                .iter()
                .chain(&roots)
                .map(|c| URL_SAFE_NO_PAD.encode(c.as_ref()))
                .collect::<Vec<_>>()
                .join("."),
            digest(&URL_SAFE_NO_PAD.encode(&crl_bytes))
        ));
        Ok(Material {
            certs,
            roots,
            crls,
            fp,
        })
    }
    fn verifier(
        &self,
        roots: Vec<CertificateDer<'static>>,
        crls: Vec<CertificateRevocationListDer<'static>>,
    ) -> Result<Arc<dyn rustls::server::danger::ClientCertVerifier>> {
        let mut store = rustls::RootCertStore::empty();
        for cert in roots {
            store.add(cert).map_err(Error::internal)?;
        }
        rustls::server::WebPkiClientVerifier::builder_with_provider(
            Arc::new(store),
            Arc::new(rustls::crypto::aws_lc_rs::default_provider()),
        )
        .with_crls(crls)
        .enforce_revocation_expiration()
        .build()
        .map_err(Error::internal)
    }
    pub(super) fn load(&self) -> Result<(Arc<rustls::ServerConfig>, String)> {
        let Material {
            certs,
            roots,
            crls,
            fp,
        } = self.material()?;
        let ocsp = self
            .ocsp_response_file
            .as_ref()
            .map(|file| read_bounded(file, 16384))
            .transpose()?
            .unwrap_or_default();
        let private =
            crate::config::read_private_secret(&self.key_file, 16384).map_err(Error::internal)?;
        let key = PrivateKeyDer::from_pem_slice(private.as_bytes()).map_err(Error::internal)?;
        let versions = if self.tls12 {
            vec![&rustls::version::TLS13, &rustls::version::TLS12]
        } else {
            vec![&rustls::version::TLS13]
        };
        let mut config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_protocol_versions(&versions)
        .map_err(Error::internal)?
        .with_client_cert_verifier(self.verifier(roots, crls)?)
        .with_single_cert_with_ocsp(certs, key, ocsp)
        .map_err(Error::internal)?;
        config.send_tls13_tickets = 0;
        config.session_storage = Arc::new(rustls::server::NoServerSessionStorage {});
        config.max_early_data_size = 0;
        config.require_ems = true;
        Ok((Arc::new(config), fp))
    }
}
struct Material {
    certs: Vec<CertificateDer<'static>>,
    roots: Vec<CertificateDer<'static>>,
    crls: Vec<CertificateRevocationListDer<'static>>,
    fp: String,
}
fn read_bounded(file: &PathBuf, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(file)
        .map_err(Error::internal)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(Error::internal)?;
    if bytes.len() > limit || bytes.is_empty() {
        return Err(Error::bad("Invalid EAP trust material size"));
    }
    Ok(bytes)
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CertificateInput {
    pub username: String,
    pub listener: String,
    pub certificate_chain_pem: String,
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
fn certificate_key(listener: &str, fingerprint: &str) -> String {
    digest(&format!("{listener}\0{fingerprint}"))
}
fn profile<'a>(core: &'a Core, listener: &str) -> Result<&'a Config> {
    core.config
        .radius_listeners
        .get(listener)
        .and_then(|c| c.eap_tls.as_ref())
        .ok_or_else(|| Error::bad("RADIUS listener has no EAP-TLS trust profile"))
}
fn certificate_fingerprint(der: &[u8]) -> String {
    use sha2::Digest as ShaDigest;
    URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(der))
}
impl Core {
    pub fn radius_certificate_bind(&self, token: &str, input: CertificateInput) -> Result<Value> {
        validate_name(&input.username)?;
        validate_name(&input.listener)?;
        self.store.read(|tx| {
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
            Ok(())
        })?;
        if input.certificate_chain_pem.len() > 32768 {
            return Err(Error::bad("EAP certificate chain exceeds 32 KiB"));
        }
        let chain = CertificateDer::pem_slice_iter(input.certificate_chain_pem.as_bytes())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| Error::bad("Invalid EAP certificate chain"))?;
        let cert = chain
            .first()
            .ok_or_else(|| Error::bad("Missing EAP client certificate"))?;
        if chain.len() > 8 {
            return Err(Error::bad("EAP client chain exceeds eight certificates"));
        }
        let config = profile(self, &input.listener)?;
        let Material { roots, crls, .. } = config.material()?;
        config
            .verifier(roots, crls)?
            .verify_client_cert(cert, &chain[1..], UnixTime::now())
            .map_err(|_| {
                Error::bad("EAP client certificate is not valid under this listener's private CA")
            })?;
        let (_, parsed) = x509_parser::parse_x509_certificate(cert.as_ref())
            .map_err(|_| Error::bad("Invalid EAP X.509 certificate"))?;
        let expires_at = u64::try_from(parsed.validity().not_after.timestamp())
            .map_err(|_| Error::bad("Invalid certificate expiry"))?;
        let fingerprint = certificate_fingerprint(cert.as_ref());
        let key = certificate_key(&input.listener, &fingerprint);
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
            let certificate = Certificate {
                id: crypto::id(),
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
    fn eap_identity(
        &self,
        client: &Client,
        listener: &str,
        profile_fp: &str,
        der: &[u8],
    ) -> Result<Identity> {
        let fingerprint = certificate_fingerprint(der);
        let key = certificate_key(listener, &fingerprint);
        self.store.write(|tx| {
            let (current, settings) = super::client(tx, &client.id)?;
            if !settings.eap_tls || super::fingerprint(&current)? != super::fingerprint(client)? {
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
            let sid = crypto::id();
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
                token_hash: digest(&crypto::random_token("")),
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
}
pub(crate) fn validate_identity(core: &Core, tx: &Tx<'_>, identity: &Identity) -> Result<()> {
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
            || profile(core, &binding.listener)?.material()?.fp != binding.profile_fingerprint
        {
            return Err(Error::unauthorized());
        }
    }
    Ok(())
}
pub(super) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, b) in tx.maintenance_page::<IdentityBinding>("radius_eap_identities")? {
        if b.expires_at <= at {
            tx.delete("radius_eap_identities", &id)?;
        }
    }
    Ok(())
}

struct Conversation {
    tls: rustls::ServerConnection,
    context: String,
    profile_fingerprint: String,
    expires_at: u64,
    expected: u8,
    rounds: u16,
    received: Vec<u8>,
    total: Option<usize>,
    outbound: VecDeque<Vec<u8>>,
    identity: Option<Identity>,
    await_success: bool,
    fragment_size: usize,
}
#[derive(Default)]
pub(super) struct Engine {
    conversations: Mutex<BTreeMap<String, Conversation>>,
}
pub(super) enum Outcome {
    Challenge(WireAttributes),
    Accept(Identity, WireAttributes),
    Reject(WireAttributes),
}
fn message(packet: &Packet) -> Result<Vec<u8>> {
    let mut seen = false;
    let mut ended = false;
    for (code, _) in &packet.attrs {
        if *code == 79 {
            if ended {
                return Err(Error::bad("EAP-Message attributes must be consecutive"));
            }
            seen = true;
        } else if seen {
            ended = true;
        }
    }
    let bytes = packet
        .attrs
        .iter()
        .filter(|(c, _)| *c == 79)
        .flat_map(|(_, v)| v)
        .copied()
        .collect::<Vec<_>>();
    if bytes.len() < 5
        || bytes[0] != 2
        || u16::from_be_bytes([bytes[2], bytes[3]]) as usize != bytes.len()
    {
        return Err(Error::bad("Invalid EAP Response"));
    }
    Ok(bytes)
}
fn packet(code: u8, id: u8, method: Option<u8>, data: &[u8]) -> Vec<u8> {
    let mut bytes = vec![code, id, 0, 0];
    if let Some(method) = method {
        bytes.push(method);
    }
    bytes.extend(data);
    let length = (bytes.len() as u16).to_be_bytes();
    bytes[2..4].copy_from_slice(&length);
    bytes
}
fn attributes(eap: Vec<u8>, state: Option<&str>) -> WireAttributes {
    let mut attrs = eap
        .chunks(253)
        .map(|chunk| (79, chunk.to_vec()))
        .collect::<WireAttributes>();
    if let Some(state) = state {
        attrs.push((24, state.as_bytes().to_vec()));
    }
    attrs
}
pub(super) fn failure(request: &Packet) -> WireAttributes {
    message(request)
        .map(|m| attributes(packet(4, m[1], None, &[]), None))
        .unwrap_or_default()
}
impl Conversation {
    fn request(&mut self, data: &[u8]) -> Vec<u8> {
        self.expected = self.expected.wrapping_add(1);
        packet(1, self.expected, Some(13), data)
    }
    fn flight(&mut self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        if bytes.is_empty() || bytes.len() > 65536 {
            return Err(Error::bad("Invalid EAP-TLS flight length"));
        }
        let fragmented = bytes.len() > self.fragment_size;
        let chunks = bytes.chunks(self.fragment_size).collect::<Vec<_>>();
        for (i, chunk) in chunks.iter().enumerate() {
            let mut data = vec![if i + 1 < chunks.len() { 0x40 } else { 0 }];
            if fragmented && i == 0 {
                data[0] |= 0x80;
                data.extend((bytes.len() as u32).to_be_bytes());
            }
            data.extend(*chunk);
            self.outbound.push_back(data);
        }
        let next = self.outbound.pop_front().unwrap();
        Ok(self.request(&next))
    }
    fn step(&mut self, core: &Core, client: &Client, listener: &str, input: &[u8]) -> Result<Step> {
        self.rounds += 1;
        if self.rounds > 256 || input[1] != self.expected || input[4] != 13 || input.len() < 6 {
            return Err(Error::bad("Unexpected EAP-TLS response or sequence"));
        }
        let flags = input[5];
        if flags & 0x3f != 0 {
            return Err(Error::bad("Unsupported EAP-TLS flags"));
        }
        let data = &input[6..];
        if !self.outbound.is_empty() {
            if flags != 0 || !data.is_empty() {
                return Err(Error::bad("EAP-TLS fragment acknowledgement must be empty"));
            }
            let next = self.outbound.pop_front().unwrap();
            return Ok(Step::Challenge(self.request(&next)));
        }
        if self.await_success {
            if flags != 0 || !data.is_empty() {
                return Err(Error::bad("EAP-TLS success acknowledgement must be empty"));
            }
            let identity = self.identity.clone().ok_or_else(Error::forbidden)?;
            let material = if self.tls.protocol_version() == Some(rustls::ProtocolVersion::TLSv1_3)
            {
                self.tls.export_keying_material(
                    zeroize::Zeroizing::new(vec![0; 128]),
                    b"EXPORTER_EAP_TLS_Key_Material",
                    Some(&[13]),
                )
            } else {
                self.tls.export_keying_material(
                    zeroize::Zeroizing::new(vec![0; 128]),
                    b"client EAP encryption",
                    None,
                )
            }
            .map_err(Error::internal)?;
            return Ok(Step::Accept(identity, material));
        }
        let data = if flags & 0x80 != 0 {
            if data.len() < 4 || self.total.is_some() || !self.received.is_empty() {
                return Err(Error::bad("Invalid EAP-TLS fragment length field"));
            }
            let total = u32::from_be_bytes(data[..4].try_into().unwrap()) as usize;
            if total == 0 || total > 65536 {
                return Err(Error::bad("EAP-TLS flight exceeds 64 KiB"));
            }
            self.total = Some(total);
            &data[4..]
        } else {
            data
        };
        if data.is_empty() || self.received.len() + data.len() > 65536 {
            return Err(Error::bad("Empty or oversized EAP-TLS input"));
        }
        self.received.extend(data);
        if flags & 0x40 != 0 {
            if self.total.is_none_or(|len| self.received.len() >= len) {
                return Err(Error::bad("Invalid fragmented EAP-TLS input"));
            }
            return Ok(Step::Challenge(self.request(&[0])));
        }
        if self.total.is_some_and(|len| self.received.len() != len) {
            return Err(Error::bad("Incomplete EAP-TLS flight"));
        }
        let bytes = std::mem::take(&mut self.received);
        self.total = None;
        let mut input = std::io::Cursor::new(bytes);
        while input.position() < input.get_ref().len() as u64 {
            if self
                .tls
                .read_tls(&mut input)
                .map_err(|_| Error::forbidden())?
                == 0
            {
                return Err(Error::forbidden());
            }
            let state = self
                .tls
                .process_new_packets()
                .map_err(|_| Error::forbidden())?;
            if state.peer_has_closed() || state.plaintext_bytes_to_read() > 0 {
                return Err(Error::forbidden());
            }
        }
        if !self.tls.is_handshaking() {
            let cert = self
                .tls
                .peer_certificates()
                .and_then(|c| c.first())
                .ok_or_else(Error::forbidden)?;
            self.identity = Some(core.eap_identity(
                client,
                listener,
                &self.profile_fingerprint,
                cert.as_ref(),
            )?);
            if self.tls.protocol_version() == Some(rustls::ProtocolVersion::TLSv1_3) {
                self.tls.writer().write_all(&[0]).map_err(Error::internal)?;
            }
            self.await_success = true;
        }
        let mut outbound = Vec::new();
        while self.tls.wants_write() {
            self.tls.write_tls(&mut outbound).map_err(Error::internal)?;
            if outbound.len() > 65536 {
                return Err(Error::bad("EAP-TLS output flight exceeds 64 KiB"));
            }
        }
        if outbound.is_empty() {
            return Ok(Step::Challenge(self.request(&[0])));
        }
        Ok(Step::Challenge(self.flight(outbound)?))
    }
}
enum Step {
    Challenge(Vec<u8>),
    Accept(Identity, zeroize::Zeroizing<Vec<u8>>),
}
impl Engine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn process(
        &self,
        core: &Core,
        listener: &str,
        nas_id: &str,
        nas: &Nas,
        request: &Packet,
        secret: &[u8],
        client: &Client,
    ) -> Result<Outcome> {
        if request.attr(24).is_none()
            && request.attr(2).is_none()
            && request.attr(3).is_none()
            && request.attrs.iter().filter(|(code, _)| *code == 79).count() == 1
            && request.attr(79) == Some(&[][..])
        {
            return Ok(Outcome::Challenge(attributes(
                packet(1, rand::random(), Some(1), &[]),
                None,
            )));
        }
        let input = match message(request) {
            Ok(m) => m,
            Err(_) => return Ok(Outcome::Reject(failure(request))),
        };
        if request.attr(2).is_some() || request.attr(3).is_some() {
            return Ok(Outcome::Reject(failure(request)));
        }
        let config = profile(core, listener)?;
        let context = digest(&format!(
            "{listener}\0{nas_id}\0{}\0{}\0{}",
            nas.peer,
            super::fingerprint(client)?,
            URL_SAFE_NO_PAD.encode(secret)
        ));
        let mut conversations = self
            .conversations
            .lock()
            .map_err(|_| Error::internal("EAP state unavailable"))?;
        conversations.retain(|_, c| c.expires_at > now());
        if request.attr(24).is_none() {
            if input[4] != 1 || input.len() > 258 || input.len() == 5 || conversations.len() >= 512
            {
                return Ok(Outcome::Reject(failure(request)));
            }
            let identity = std::str::from_utf8(&input[5..])
                .map_err(|_| Error::bad("EAP identity must be UTF-8"))?;
            if identity.chars().any(char::is_control) {
                return Ok(Outcome::Reject(failure(request)));
            }
            // The outer identity is an unauthenticated routing hint. Access control uses the certificate pin only.
            let (tls, profile_fingerprint) = config.load()?;
            let tls = rustls::ServerConnection::new(tls).map_err(Error::internal)?;
            let state = crypto::random_token("ri_eap_");
            let mut c = Conversation {
                tls,
                context,
                profile_fingerprint,
                expires_at: now() + 120,
                expected: input[1],
                rounds: 0,
                received: vec![],
                total: None,
                outbound: VecDeque::new(),
                identity: None,
                await_success: false,
                fragment_size: config.fragment_size,
            };
            let reply = c.request(&[0x20]);
            conversations.insert(state.clone(), c);
            return Ok(Outcome::Challenge(attributes(reply, Some(&state))));
        }
        let state = std::str::from_utf8(request.attr(24).unwrap())
            .map_err(|_| Error::bad("Invalid EAP state"))?;
        if conversations
            .get(state)
            .is_none_or(|c| c.context != context)
        {
            return Ok(Outcome::Reject(failure(request)));
        }
        let mut c = conversations.remove(state).unwrap();
        if c.profile_fingerprint != config.material()?.fp {
            return Ok(Outcome::Reject(failure(request)));
        }
        let step = c.step(core, client, listener, &input);
        match step {
            Ok(Step::Challenge(reply)) => {
                conversations.insert(state.into(), c);
                Ok(Outcome::Challenge(attributes(reply, Some(state))))
            }
            Ok(Step::Accept(identity, material)) => {
                let mut attrs = attributes(packet(3, input[1], None, &[]), None);
                let salt: u16 = rand::random::<u16>() | 0x8000;
                attrs.push((
                    26,
                    mppe(&material[..32], secret, &request.authenticator, salt, 17),
                ));
                attrs.push((
                    26,
                    mppe(
                        &material[32..64],
                        secret,
                        &request.authenticator,
                        salt ^ 1,
                        16,
                    ),
                ));
                Ok(Outcome::Accept(identity, attrs))
            }
            Err(error) => {
                if let Some(identity) = &c.identity {
                    let _ = core.store.write(|tx| super::close(tx, Some(identity)));
                }
                if error.status.is_server_error() {
                    Err(error)
                } else {
                    Ok(Outcome::Reject(failure(request)))
                }
            }
        }
    }
}
fn mppe(key: &[u8], secret: &[u8], auth: &[u8; 16], salt: u16, kind: u8) -> Vec<u8> {
    let mut plaintext = zeroize::Zeroizing::new(vec![key.len() as u8]);
    plaintext.extend(key);
    let size = plaintext.len().div_ceil(16) * 16;
    plaintext.resize(size, 0);
    let salt = salt.to_be_bytes();
    let mut previous = [auth.as_slice(), &salt].concat();
    let mut ciphertext: Vec<u8> = Vec::new();
    for block in plaintext.chunks_exact(16) {
        let mut hash = Md5::new();
        hash.update(secret);
        hash.update(&previous);
        let mask = hash.finalize();
        previous = block.iter().zip(mask).map(|(a, b)| a ^ b).collect();
        ciphertext.extend(&previous);
    }
    let mut value = 311u32.to_be_bytes().to_vec();
    value.extend([kind, (ciphertext.len() + 4) as u8]);
    value.extend(salt);
    value.extend(ciphertext);
    value
}

#[cfg(feature = "fuzzing")]
pub(super) fn fuzz_message(data: &[u8]) {
    let request = Packet {
        identifier: 0,
        authenticator: [0; 16],
        attrs: data.chunks(253).map(|chunk| (79, chunk.to_vec())).collect(),
    };
    let _ = message(&request);
}
