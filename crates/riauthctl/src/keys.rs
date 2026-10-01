//! Signing-key domains through the server's one management service.
//!
//! The same routes as `riauth keys` and `riauth rotate-key`. The server owns
//! authority, key and signer validation, the revision precondition, the
//! receipt and the audit record, and returns only public key data. Every write
//! sends `If-Match` and an `Idempotency-Key`; an imported private key is read
//! from an owner-only file and never printed.

use crate::{
    admin::{MutationOptions, mutate, segment},
    session::read_private_text,
    transport::Remote,
};
use anyhow::{Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

const MAX_PRIVATE_KEY_BYTES: u64 = 16 * 1024;

#[derive(Subcommand)]
pub(crate) enum KeyCommand {
    /// List key domains the principal may read, without private material.
    List,
    /// Generate a key domain, or rotate its active key keeping verification keys.
    #[command(alias = "create")]
    Generate {
        id: String,
        #[arg(long, default_value = "RS256", value_parser = ["RS256", "ES256", "EdDSA"])]
        algorithm: String,
    },
    /// Bind a server-configured external signer to a key domain.
    Bind {
        id: String,
        #[arg(long)]
        signer: String,
        #[arg(long, value_parser = ["RS256", "ES256", "EdDSA"])]
        algorithm: String,
    },
    /// Import a private signing key from an owner-only PEM file (16 KiB at most).
    Import {
        id: String,
        #[arg(long)]
        file: PathBuf,
        #[arg(long, value_parser = ["RS256", "ES256", "EdDSA"])]
        algorithm: String,
        /// Preserve an existing key id.
        #[arg(long)]
        kid: Option<String>,
    },
    /// Rotate the instance signing key, keeping its verification keys.
    Rotate,
}

#[derive(Serialize)]
struct KeyBody<'a> {
    id: &'a str,
    algorithm: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    remote_signer: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private_key_pem: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kid: Option<&'a str>,
}

pub(crate) async fn run(
    remote: &Remote,
    command: KeyCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        KeyCommand::List => {
            remote
                .authenticated(Method::GET, "/api/keys", None::<&()>)
                .await
        }
        KeyCommand::Generate { id, algorithm } => {
            let body = KeyBody {
                id: segment(&id)?,
                algorithm: &algorithm,
                remote_signer: None,
                private_key_pem: None,
                kid: None,
            };
            mutate(remote, Method::POST, "/api/keys", Some(&body), options).await
        }
        KeyCommand::Bind {
            id,
            signer,
            algorithm,
        } => {
            if signer.is_empty() || signer.len() > 256 || signer.chars().any(char::is_control) {
                bail!("Signer must be a short printable name");
            }
            let body = KeyBody {
                id: segment(&id)?,
                algorithm: &algorithm,
                remote_signer: Some(&signer),
                private_key_pem: None,
                kid: None,
            };
            mutate(remote, Method::POST, "/api/keys", Some(&body), options).await
        }
        KeyCommand::Import {
            id,
            file,
            algorithm,
            kid,
        } => {
            let id = segment(&id)?;
            let pem = read_private_text(&file, MAX_PRIVATE_KEY_BYTES)?;
            let body = KeyBody {
                id,
                algorithm: &algorithm,
                remote_signer: None,
                private_key_pem: Some(pem.as_str()),
                kid: kid.as_deref(),
            };
            mutate(remote, Method::POST, "/api/keys", Some(&body), options).await
        }
        KeyCommand::Rotate => {
            mutate(
                remote,
                Method::POST,
                "/api/keys/rotate",
                None::<&()>,
                options,
            )
            .await
        }
    }
}
