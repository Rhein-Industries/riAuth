//! HTTPS client-certificate login. Anonymous handshakes stay allowed.
//! Enrollment is an explicit fingerprint and/or SAN binding, not "any cert from this CA".
use crate::{
    config::Config,
    core::{Core, audit, user_by_name, validate_email, validate_name},
    crypto::{self, digest, id, now},
    error::{Error, Result},
    model::{AuthenticationTransaction, Identity, Session, User, UserView},
    store::Tx,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rustls::pki_types::{CertificateDer, CertificateRevocationListDer, UnixTime, pem::PemObject};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    io::Read,
    net::IpAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

/// `required` does not turn client certificates on for the whole listener.
/// Both modes use rustls 0.23 `WebPkiClientVerifier::allow_unauthenticated`,
/// so password and browser logins still connect. The certificate login route
/// rejects a missing certificate in either mode.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientCertMode {
    #[default]
    Optional,
    Required,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClientCertAuth {
    pub trust_anchors_file: PathBuf,
    #[serde(default)]
    pub mode: ClientCertMode,
    #[serde(default)]
    pub crl_file: Option<PathBuf>,
    /// PEM client certificate header. Ignored unless the immediate peer is in `trusted_proxies`.
    #[serde(default)]
    pub forwarded_header: Option<String>,
}

impl ClientCertAuth {
    pub fn validate(&self, config: &Config) -> Result<()> {
        if self.trust_anchors_file.as_os_str().is_empty()
            || self
                .crl_file
                .as_ref()
                .is_some_and(|path| path.as_os_str().is_empty())
        {
            return Err(Error::bad("Client certificate trust files must be set"));
        }
        if let Some(header) = &self.forwarded_header {
            validate_header_name(header)?;
            if config.trusted_proxies.is_empty() {
                return Err(Error::bad(
                    "forwarded_header requires at least one trusted proxy address",
                ));
            }
        }
        if config.tls_cert_file.is_none() && self.forwarded_header.is_none() {
            return Err(Error::bad(
                "Client certificate authentication without native TLS requires forwarded_header",
            ));
        }
        Ok(())
    }
}

fn validate_header_name(name: &str) -> Result<()> {
    if !(1..=64).contains(&name.len())
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        || name.starts_with('-')
        || name.ends_with('-')
    {
        return Err(Error::bad("forwarded_header must be one HTTP header token"));
    }
    if matches!(
        name.to_ascii_lowercase().as_str(),
        "authorization"
            | "cookie"
            | "host"
            | "content-type"
            | "content-length"
            | "x-forwarded-for"
            | "forwarded"
            | "connection"
            | "transfer-encoding"
    ) {
        return Err(Error::bad(
            "forwarded_header cannot replace an existing request header",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct TlsClientCerts {
    pub ders: Vec<Vec<u8>>,
}

pub(crate) struct Material {
    roots: Vec<CertificateDer<'static>>,
    crls: Vec<CertificateRevocationListDer<'static>>,
}

impl ClientCertAuth {
    pub(crate) fn material(&self) -> Result<Material> {
        let roots = read_certs(&self.trust_anchors_file, 65_536, 16, "trust anchors")?;
        let crls = match &self.crl_file {
            Some(path) => read_crls(path)?,
            None => Vec::new(),
        };
        Ok(Material { roots, crls })
    }
    pub(crate) fn verifier(
        &self,
        material: &Material,
    ) -> Result<Arc<dyn rustls::server::danger::ClientCertVerifier>> {
        let mut store = rustls::RootCertStore::empty();
        for cert in &material.roots {
            store.add(cert.clone()).map_err(|error| {
                tracing::error!(%error, "client certificate trust anchor rejected");
                unavailable("Client certificate trust material is invalid")
            })?;
        }
        // Configured anchors only. System roots and webpki-roots are not consulted.
        // allow_unauthenticated keeps every other route reachable without a certificate.
        let mut builder = rustls::server::WebPkiClientVerifier::builder_with_provider(
            Arc::new(store),
            Arc::new(rustls::crypto::aws_lc_rs::default_provider()),
        )
        .allow_unauthenticated();
        if !material.crls.is_empty() {
            builder = builder
                .with_crls(material.crls.clone())
                .enforce_revocation_expiration();
        }
        builder.build().map_err(|error| {
            tracing::error!(%error, "client certificate verifier rejected");
            unavailable("Client certificate trust material is invalid")
        })
    }
}

fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|error| {
            tracing::error!(%error, path = %path.display(), "client certificate trust material unreadable");
            unavailable("Client certificate trust material is unavailable")
        })?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            tracing::error!(%error, "client certificate trust material unreadable");
            unavailable("Client certificate trust material is unavailable")
        })?;
    if bytes.is_empty() || bytes.len() > limit {
        return Err(unavailable("Client certificate trust material is invalid"));
    }
    Ok(bytes)
}

fn read_certs(
    path: &Path,
    limit: usize,
    max: usize,
    kind: &str,
) -> Result<Vec<CertificateDer<'static>>> {
    let bytes = read_bounded(path, limit)?;
    let certs = CertificateDer::pem_slice_iter(&bytes)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| unavailable("Client certificate trust material is invalid"))?;
    if certs.is_empty() || certs.len() > max {
        return Err(unavailable(format!(
            "Client certificate {kind} must contain 1..={max} PEM certificates"
        )));
    }
    Ok(certs)
}

fn read_crls(path: &Path) -> Result<Vec<CertificateRevocationListDer<'static>>> {
    let bytes = read_bounded(path, 1024 * 1024)?;
    let crls = CertificateRevocationListDer::pem_slice_iter(&bytes)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| unavailable("Client certificate revocation data is unavailable"))?;
    if crls.is_empty() || crls.len() > 16 {
        return Err(unavailable(
            "Client certificate revocation data must contain 1..=16 PEM CRLs",
        ));
    }
    let at = now() as i64;
    for crl in &crls {
        let (trailing, parsed) = x509_parser::parse_x509_crl(crl.as_ref())
            .map_err(|_| unavailable("Client certificate revocation data is unavailable"))?;
        if !trailing.is_empty()
            || parsed.last_update().timestamp() > at
            || parsed
                .next_update()
                .is_none_or(|time| time.timestamp() <= at)
        {
            return Err(unavailable(
                "Client certificate revocation data is stale or not yet valid",
            ));
        }
    }
    Ok(crls)
}

fn unavailable(message: impl Into<String>) -> Error {
    Error::new(
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "temporarily_unavailable",
        message,
    )
}
fn missing_certificate() -> Error {
    Error::new(
        axum::http::StatusCode::UNAUTHORIZED,
        "certificate_required",
        "Client certificate required",
    )
}
fn rejected(message: &str) -> Error {
    Error::new(
        axum::http::StatusCode::UNAUTHORIZED,
        "invalid_credentials",
        message,
    )
}
fn classify_verification(error: &rustls::Error) -> Error {
    tracing::debug!(%error, "client certificate verification failed");
    match error {
        rustls::Error::InvalidCertificate(rustls::CertificateError::Revoked) => {
            rejected("Client certificate is revoked")
        }
        rustls::Error::InvalidCertificate(rustls::CertificateError::Expired)
        | rustls::Error::InvalidCertificate(rustls::CertificateError::ExpiredContext { .. })
        | rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidYet)
        | rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidYetContext {
            ..
        }) => rejected("Client certificate is expired or not yet valid"),
        rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownRevocationStatus)
        | rustls::Error::InvalidCertificate(rustls::CertificateError::ExpiredRevocationList)
        | rustls::Error::InvalidCertificate(
            rustls::CertificateError::ExpiredRevocationListContext { .. },
        ) => unavailable("Client certificate revocation data is unavailable"),
        _ => rejected("Client certificate is not trusted"),
    }
}

/// SHA-256 of the leaf DER, base64url without padding. Subject CN is not an identity.
pub fn fingerprint(der: &[u8]) -> String {
    use sha2::Digest;
    URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(der))
}

struct Leaf {
    fingerprint: String,
    emails: Vec<String>,
    uris: Vec<String>,
    not_after: u64,
}

fn presented_chain(
    profile: &ClientCertAuth,
    trusted: &[IpAddr],
    peer: IpAddr,
    forwarded: &[String],
    tls_ders: &[Vec<u8>],
) -> Result<Vec<CertificateDer<'static>>> {
    if profile.forwarded_header.is_some() && trusted.contains(&peer) {
        if forwarded.len() > 1 {
            return Err(Error::bad("Duplicate forwarded client certificate"));
        }
        let Some(value) = forwarded.first() else {
            return Err(missing_certificate());
        };
        return parse_forwarded(value);
    }
    if tls_ders.is_empty() {
        return Err(missing_certificate());
    }
    parse_ders(tls_ders)
}

fn parse_forwarded(value: &str) -> Result<Vec<CertificateDer<'static>>> {
    if value.len() > 48 * 1024 {
        return Err(Error::bad("Forwarded certificate exceeds 32 KiB"));
    }
    let decoded = if value.contains('%') {
        percent_decode(value)?
    } else {
        value.to_owned()
    };
    if decoded.len() > 32 * 1024 || decoded.contains('\0') {
        return Err(Error::bad("Forwarded certificate exceeds 32 KiB"));
    }
    if decoded.trim().is_empty() {
        return Err(missing_certificate());
    }
    parse_pem(&decoded)
}

fn percent_decode(value: &str) -> Result<String> {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(Error::bad("Invalid forwarded certificate encoding"));
            }
            let high = hex_nibble(bytes[index + 1])?;
            let low = hex_nibble(bytes[index + 2])?;
            out.push((high << 4) | low);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).map_err(|_| Error::bad("Invalid forwarded certificate encoding"))
}

fn hex_nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(Error::bad("Invalid forwarded certificate encoding")),
    }
}

fn parse_pem(pem: &str) -> Result<Vec<CertificateDer<'static>>> {
    if pem.len() > 32 * 1024 {
        return Err(Error::bad("Certificate chain exceeds 32 KiB"));
    }
    let certs = CertificateDer::pem_slice_iter(pem.as_bytes())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| rejected("Client certificate is not trusted"))?;
    check_chain(&certs)?;
    Ok(certs)
}

fn parse_ders(ders: &[Vec<u8>]) -> Result<Vec<CertificateDer<'static>>> {
    if ders.len() > 8
        || ders
            .iter()
            .any(|der| der.is_empty() || der.len() > 16 * 1024)
    {
        return Err(rejected("Client certificate is not trusted"));
    }
    let certs = ders
        .iter()
        .cloned()
        .map(CertificateDer::from)
        .collect::<Vec<_>>();
    check_chain(&certs)?;
    Ok(certs)
}

fn check_chain(certs: &[CertificateDer<'static>]) -> Result<()> {
    if certs.is_empty() || certs.len() > 8 {
        return Err(rejected("Client certificate is not trusted"));
    }
    Ok(())
}

fn verify_chain(profile: &ClientCertAuth, chain: &[CertificateDer<'static>]) -> Result<Leaf> {
    let material = profile.material()?;
    let verifier = profile.verifier(&material)?;
    verifier
        .verify_client_cert(&chain[0], &chain[1..], UnixTime::now())
        .map_err(|error| classify_verification(&error))?;
    leaf_identity(chain[0].as_ref())
}

fn leaf_identity(der: &[u8]) -> Result<Leaf> {
    let (_, parsed) = x509_parser::parse_x509_certificate(der)
        .map_err(|_| rejected("Client certificate is not trusted"))?;
    if parsed.is_ca() {
        return Err(rejected("Client certificate is not trusted"));
    }
    let at = now() as i64;
    if parsed.validity().not_before.timestamp() > at
        || parsed.validity().not_after.timestamp() <= at
    {
        return Err(rejected("Client certificate is expired or not yet valid"));
    }
    let not_after = u64::try_from(parsed.validity().not_after.timestamp())
        .map_err(|_| rejected("Client certificate is not trusted"))?;
    let mut emails = Vec::new();
    let mut uris = Vec::new();
    if let Some(san) = parsed
        .subject_alternative_name()
        .map_err(|_| rejected("Client certificate is not trusted"))?
    {
        for name in &san.value.general_names {
            match name {
                x509_parser::extensions::GeneralName::RFC822Name(value) => {
                    if emails.len() + uris.len() >= 32 {
                        return Err(rejected("Client certificate is not trusted"));
                    }
                    emails.push((*value).to_owned());
                }
                x509_parser::extensions::GeneralName::URI(value) => {
                    if emails.len() + uris.len() >= 32 {
                        return Err(rejected("Client certificate is not trusted"));
                    }
                    uris.push((*value).to_owned());
                }
                _ => {}
            }
        }
    }
    Ok(Leaf {
        fingerprint: fingerprint(der),
        emails,
        uris,
        not_after,
    })
}

fn validate_san_uri(uri: &str) -> Result<()> {
    if !(6..=512).contains(&uri.len())
        || uri.chars().any(|c| c.is_control() || c.is_whitespace())
        || !(uri.starts_with("urn:") || uri.starts_with("https://"))
        || uri.contains('\\')
        || uri.contains('@')
    {
        return Err(Error::bad(
            "SAN URI must be an exact urn: or https:// identifier without userinfo",
        ));
    }
    Ok(())
}

#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BindInput {
    pub username: String,
    #[serde(default)]
    pub certificate_pem: Option<String>,
    #[serde(default)]
    pub san_uri: Option<String>,
    #[serde(default)]
    pub san_email: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Binding {
    id: String,
    username: String,
    user_id: String,
    fingerprint: Option<String>,
    san_uri: Option<String>,
    san_email: Option<String>,
    not_after: Option<u64>,
    created_at: u64,
}

impl Binding {
    fn view(&self) -> Value {
        json!({
            "id": self.id,
            "username": self.username,
            "user_id": self.user_id,
            "fingerprint": self.fingerprint,
            "san_uri": self.san_uri,
            "san_email": self.san_email,
            "not_after": self.not_after,
            "created_at": self.created_at,
        })
    }
}

fn selector(binding: &Binding) -> String {
    digest(&format!(
        "{}\0{}\0{}",
        binding.fingerprint.as_deref().unwrap_or(""),
        binding.san_uri.as_deref().unwrap_or(""),
        binding.san_email.as_deref().unwrap_or("")
    ))
}

fn binding_matches(binding: &Binding, leaf: &Leaf) -> bool {
    let mut constrained = false;
    if let Some(expected) = &binding.fingerprint {
        constrained = true;
        if expected != &leaf.fingerprint {
            return false;
        }
    }
    if let Some(expected) = &binding.san_email {
        constrained = true;
        if !leaf.emails.iter().any(|email| email == expected) {
            return false;
        }
    }
    if let Some(expected) = &binding.san_uri {
        constrained = true;
        if !leaf.uris.iter().any(|uri| uri == expected) {
            return false;
        }
    }
    constrained
}

#[derive(Clone, Serialize, Deserialize)]
struct LoginLink {
    binding_id: String,
    selector: String,
    expires_at: u64,
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

pub(crate) fn validate_identity(tx: &Tx<'_>, identity: &Identity) -> Result<()> {
    if !identity.amr.iter().any(|method| method == "cert") {
        return Ok(());
    }
    let link = tx
        .get::<LoginLink>("mtls_logins", &identity.session_id)?
        .filter(|link| link.expires_at > now())
        .ok_or_else(Error::unauthorized)?;
    let binding = tx
        .get::<Binding>("mtls_bindings", &link.binding_id)?
        .filter(|binding| binding.user_id == identity.user_id && selector(binding) == link.selector)
        .ok_or_else(Error::unauthorized)?;
    let _ = binding;
    Ok(())
}

pub(crate) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, link) in tx.maintenance_page::<LoginLink>("mtls_logins")? {
        if link.expires_at <= at {
            tx.delete("mtls_logins", &id)?;
        }
    }
    Ok(())
}
