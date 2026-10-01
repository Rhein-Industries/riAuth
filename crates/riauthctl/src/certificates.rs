//! Client-certificate and RADIUS EAP-TLS certificate bindings through the
//! server's one management service.
//!
//! The same routes as `riauth certificate` and `riauth radius`. Certificates
//! are public material; the server owns authority, chain and identity
//! validation, the revision precondition, the receipt and the audit record.
//! Each PEM file must be a regular file of at most 32 KiB and the request body
//! the server actually receives must fit its 32 KiB limit. Every write sends
//! `If-Match` and an `Idempotency-Key`.

use crate::{
    admin::{MutationOptions, mutate, segment},
    transport::Remote,
};
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::{fs::File, io::Read, path::Path, path::PathBuf};

const MAX_BODY_BYTES: usize = 32 * 1024;

#[derive(Subcommand)]
pub(crate) enum CertificateCommand {
    /// List client-certificate bindings visible to this principal.
    List,
    /// Bind a leaf fingerprint and/or an exact SAN URI or email to a user.
    Bind {
        username: String,
        /// PEM certificate file; the first certificate is the leaf.
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        san_uri: Option<String>,
        #[arg(long)]
        san_email: Option<String>,
    },
    /// Revoke one binding.
    Revoke { id: String },
}

#[derive(Subcommand)]
pub(crate) enum RadiusCommand {
    /// List RADIUS EAP-TLS certificate bindings.
    Certificates,
    /// Bind a user's certificate chain to one RADIUS listener.
    BindCertificate {
        username: String,
        #[arg(long)]
        listener: String,
        /// PEM certificate chain file.
        #[arg(long)]
        file: PathBuf,
    },
    /// Revoke one RADIUS certificate binding.
    RevokeCertificate { id: String },
}

pub(crate) async fn certificate(
    remote: &Remote,
    command: CertificateCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        CertificateCommand::List => {
            remote
                .authenticated(Method::GET, "/api/certificates", None::<&()>)
                .await
        }
        CertificateCommand::Bind {
            username,
            file,
            san_uri,
            san_email,
        } => {
            segment(&username)?;
            let pem = file
                .as_deref()
                .map(|file| read_pem(file, "Certificate"))
                .transpose()?;
            if pem.is_none() && san_uri.is_none() && san_email.is_none() {
                bail!("Provide --file, --san-uri, or --san-email");
            }
            let body = json!({"username": username, "certificate_pem": pem,
                              "san_uri": san_uri, "san_email": san_email});
            fit(&body)?;
            mutate(
                remote,
                Method::POST,
                "/api/certificates",
                Some(&body),
                options,
            )
            .await
        }
        CertificateCommand::Revoke { id } => {
            let path = format!("/api/certificates/{}", segment(&id)?);
            mutate(remote, Method::DELETE, &path, None::<&()>, options).await
        }
    }
}

pub(crate) async fn radius(
    remote: &Remote,
    command: RadiusCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        RadiusCommand::Certificates => {
            remote
                .authenticated(Method::GET, "/api/radius/certificates", None::<&()>)
                .await
        }
        RadiusCommand::BindCertificate {
            username,
            listener,
            file,
        } => {
            segment(&username)?;
            // The listener is a configured name the server resolves, not a path segment.
            if listener.is_empty() || listener.len() > 128 || listener.chars().any(char::is_control)
            {
                bail!("Listener must be a short printable name");
            }
            let chain = read_pem(&file, "Certificate chain")?;
            let body = json!({"username": username, "listener": listener,
                              "certificate_chain_pem": chain});
            fit(&body)?;
            mutate(
                remote,
                Method::POST,
                "/api/radius/certificates",
                Some(&body),
                options,
            )
            .await
        }
        RadiusCommand::RevokeCertificate { id } => {
            let path = format!("/api/radius/certificates/{}", segment(&id)?);
            mutate(remote, Method::DELETE, &path, None::<&()>, options).await
        }
    }
}

/// A bounded regular file of UTF-8 PEM text.
fn read_pem(path: &Path, what: &str) -> Result<String> {
    let file = File::open(path).with_context(|| format!("Cannot read {what} file"))?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        bail!("{what} file must be a regular file");
    }
    let mut pem = String::new();
    file.take(MAX_BODY_BYTES as u64 + 1)
        .read_to_string(&mut pem)
        .with_context(|| format!("{what} file must be UTF-8 PEM text"))?;
    if pem.len() > MAX_BODY_BYTES {
        bail!("{what} file exceeds 32 KiB");
    }
    Ok(pem)
}

/// The JSON body re-escapes PEM newlines, so a file just under the limit can
/// still exceed the server's request limit once it is sent.
fn fit(body: &Value) -> Result<()> {
    if serde_json::to_vec(body)?.len() > MAX_BODY_BYTES {
        bail!("Certificate request exceeds the server's 32 KiB request limit");
    }
    Ok(())
}
