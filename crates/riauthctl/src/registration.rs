//! Dynamic-registration templates through the server's one management service.
//!
//! The same routes as `riauth registration`: the server owns authority,
//! template validation, the revision precondition, the redacted issuance
//! receipt and the audit record. The initial access token is returned only by
//! the first committed response. It goes to a new owner-only file, in the
//! `{"issuer","token"}` form `riauth registration register` reads, and is
//! never printed.

use crate::{
    admin::{MutationOptions, SecretFile, mutate, segment},
    review::{Shape, read_content},
    transport::Remote,
};
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::path::PathBuf;
use zeroize::Zeroize;

#[derive(Subcommand)]
pub(crate) enum RegistrationCommand {
    /// List registration templates visible to this principal.
    List,
    /// Create a template from a JSON file (32 KiB at most) and save its one-time
    /// initial access token to a new private file.
    Create {
        #[arg(long)]
        file: PathBuf,
        /// New owner-only file for the token; an existing path is refused.
        #[arg(long)]
        out: PathBuf,
    },
    /// Revoke a template's initial access credential.
    Revoke { id: String },
}

pub(crate) async fn run(
    remote: &Remote,
    command: RegistrationCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        RegistrationCommand::List => {
            remote
                .authenticated(Method::GET, "/api/registration", None::<&()>)
                .await
        }
        RegistrationCommand::Create { file, out } => {
            let template = read_content(&file, "Registration template", Shape::Object)?;
            let id = template
                .get("id")
                .and_then(Value::as_str)
                .context("Registration template needs a string id")?
                .to_owned();
            segment(&id)?;
            // Reserve the private destination first. A refused or failed
            // issuance removes it again, so an exact retry can use the same path.
            let mut destination = SecretFile::reserve(out)?;
            let result = mutate(
                remote,
                Method::POST,
                "/api/registration",
                Some(&template),
                options,
            )
            .await?;
            issued(result, &id, &remote.issuer, &mut destination)
        }
        RegistrationCommand::Revoke { id } => {
            let path = format!("/api/registration/{}", segment(&id)?);
            let revoked = mutate(remote, Method::DELETE, &path, None::<&()>, options).await?;
            if revoked.pointer("/template/id").and_then(Value::as_str) != Some(id.as_str()) {
                bail!("Revocation response does not match the requested template");
            }
            Ok(revoked)
        }
    }
}

/// Move the token from the response into the reserved file, zeroizing the
/// in-memory copy. Standard output carries the template view and the file path.
fn issued(
    mut result: Value,
    id: &str,
    issuer: &str,
    destination: &mut SecretFile,
) -> Result<Value> {
    let registration = result
        .get("registration")
        .filter(|view| view.pointer("/template/id").and_then(Value::as_str) == Some(id))
        .cloned();
    let mut token = result
        .get_mut("initial_access_token")
        .map(std::mem::take)
        .unwrap_or(Value::Null);
    let saved = (|| -> Result<()> {
        if registration.is_none() {
            bail!("Server returned a mismatched registration template");
        }
        let token = token
            .as_str()
            .filter(|token| token.starts_with("ri_register_"))
            .context("Server did not return an initial access token")?;
        destination.write(&json!({"issuer": issuer, "token": token}))
    })();
    if let Value::String(token) = &mut token {
        token.zeroize();
    }
    saved?;
    let registration =
        registration.context("Server returned a mismatched registration template")?;
    Ok(json!({"registration": registration, "credential_file": destination.path_text()?}))
}
