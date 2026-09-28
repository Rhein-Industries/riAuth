//! HTTPS client-certificate login. Anonymous handshakes stay allowed.
//! Enrollment is an explicit fingerprint and/or SAN binding, not "any cert from this CA".
use crate::{
    crypto::{digest, now},
    error::{Error, Result},
    model::Identity,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rustls::pki_types::{CertificateDer, CertificateRevocationListDer, UnixTime, pem::PemObject};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    io::Read,
    net::IpAddr,
    path::Path,
    sync::Arc,
};

pub use crate::mtls_config::{ClientCertAuth, ClientCertMode, TlsClientCerts};

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
pub(crate) fn rejected(message: &str) -> Error {
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

pub(crate) struct Leaf {
    pub(crate) fingerprint: String,
    pub(crate) emails: Vec<String>,
    pub(crate) uris: Vec<String>,
    pub(crate) not_after: u64,
}

pub(crate) fn presented_chain(
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

pub(crate) fn parse_pem(pem: &str) -> Result<Vec<CertificateDer<'static>>> {
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

pub(crate) fn verify_chain(profile: &ClientCertAuth, chain: &[CertificateDer<'static>]) -> Result<Leaf> {
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

pub(crate) fn validate_san_uri(uri: &str) -> Result<()> {
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
pub(crate) struct Binding {
    pub(crate) id: String,
    pub(crate) username: String,
    pub(crate) user_id: String,
    pub(crate) fingerprint: Option<String>,
    pub(crate) san_uri: Option<String>,
    pub(crate) san_email: Option<String>,
    pub(crate) not_after: Option<u64>,
    pub(crate) created_at: u64,
}

impl Binding {
    pub(crate) fn view(&self) -> Value {
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

pub(crate) fn selector(binding: &Binding) -> String {
    digest(&format!(
        "{}\0{}\0{}",
        binding.fingerprint.as_deref().unwrap_or(""),
        binding.san_uri.as_deref().unwrap_or(""),
        binding.san_email.as_deref().unwrap_or("")
    ))
}

pub(crate) fn binding_matches(binding: &Binding, leaf: &Leaf) -> bool {
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
pub(crate) struct LoginLink {
    pub(crate) binding_id: String,
    pub(crate) selector: String,
    pub(crate) expires_at: u64,
}

/// Identity reads and cleanup writes supplied by Platform assembly.
pub(crate) trait MtlsTx {
    fn login_link(&self, session_id: &str) -> Result<Option<LoginLink>>;
    fn binding(&self, binding_id: &str) -> Result<Option<Binding>>;
    fn login_links_for_cleanup(&self) -> Result<Vec<(String, LoginLink)>>;
    fn delete_login_link(&self, session_id: &str) -> Result<()>;
}

pub(crate) fn validate_identity(tx: &impl MtlsTx, identity: &Identity) -> Result<()> {
    if !identity.amr.iter().any(|method| method == "cert") {
        return Ok(());
    }
    let link = tx
        .login_link(&identity.session_id)?
        .filter(|link| link.expires_at > now())
        .ok_or_else(Error::unauthorized)?;
    let binding = tx
        .binding(&link.binding_id)?
        .filter(|binding| binding.user_id == identity.user_id && selector(binding) == link.selector)
        .ok_or_else(Error::unauthorized)?;
    let _ = binding;
    Ok(())
}

pub(crate) fn cleanup(tx: &impl MtlsTx, at: u64) -> Result<()> {
    for (id, link) in tx.login_links_for_cleanup()? {
        if link.expires_at <= at {
            tx.delete_login_link(&id)?;
        }
    }
    Ok(())
}
