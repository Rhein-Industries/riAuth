//! Routine remote administration. The API handlers call the server's shared
//! management service; this module only shapes requests and protects replies.

use crate::{read_password, transport::Remote};
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zeroize::{Zeroize, Zeroizing};

const MAX_SETTINGS_BYTES: u64 = 32 * 1024;

pub(crate) struct MutationOptions<'a> {
    pub(crate) run_id: Option<&'a str>,
    pub(crate) if_revision: Option<u64>,
    pub(crate) idempotency_key: Option<&'a str>,
    pub(crate) non_interactive: bool,
}

#[derive(Subcommand)]
pub(crate) enum UserCommand {
    /// List users visible to this principal.
    List,
    /// Create a password user; password is read without echo or from stdin.
    Create {
        username: String,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        admin: bool,
        #[arg(long)]
        password_stdin: bool,
    },
    /// Change only the supplied user fields.
    Update {
        username: String,
        #[arg(long)]
        enabled: Option<bool>,
        #[arg(long)]
        admin: Option<bool>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        password_stdin: bool,
        #[arg(long)]
        reset_mfa: bool,
        #[arg(long)]
        revoke_sessions: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum GroupCommand {
    /// List groups visible to this principal.
    List,
    Create {
        name: String,
    },
    AddMember {
        group: String,
        username: String,
    },
    RemoveMember {
        group: String,
        username: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum ClientCommand {
    /// List applications visible to this principal.
    List,
    /// Create an OAuth application. Confidential clients need --secret-file.
    Create {
        client_id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        confidential: bool,
        #[arg(long)]
        service: bool,
        #[arg(long)]
        native: bool,
        /// Complete ProviderSettings JSON object.
        #[arg(long)]
        settings_file: Option<PathBuf>,
        #[arg(long = "redirect-uri")]
        redirect_uris: Vec<String>,
        #[arg(long = "scope", value_delimiter = ',')]
        scopes: Vec<String>,
        #[arg(long = "group", value_delimiter = ',')]
        groups: Vec<String>,
        #[arg(long)]
        require_mfa: bool,
        /// New owner-only file for a one-time client secret.
        #[arg(long)]
        secret_file: Option<PathBuf>,
    },
    /// Change only the supplied application fields.
    Update {
        client_id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        enabled: Option<bool>,
        #[arg(long = "redirect-uri")]
        redirect_uris: Option<Vec<String>>,
        #[arg(long = "scope", value_delimiter = ',')]
        scopes: Option<Vec<String>>,
        #[arg(long = "group", value_delimiter = ',', conflicts_with = "allow_all")]
        groups: Option<Vec<String>>,
        /// Clear group restrictions for this application.
        #[arg(long)]
        allow_all: bool,
        #[arg(long)]
        require_mfa: Option<bool>,
        /// Complete ProviderSettings replacement JSON object.
        #[arg(long)]
        settings_file: Option<PathBuf>,
    },
    /// Rotate a confidential application's secret into a new private file.
    RotateSecret {
        client_id: String,
        #[arg(long)]
        secret_file: PathBuf,
    },
}

#[derive(Serialize)]
struct NewUser<'a> {
    username: &'a str,
    password: &'a str,
    email: Option<&'a str>,
    display_name: &'a str,
    admin: bool,
}

#[derive(Serialize)]
struct UserPatch<'a> {
    enabled: Option<bool>,
    admin: Option<bool>,
    password: Option<&'a str>,
    email: Option<&'a str>,
    display_name: Option<&'a str>,
    reset_mfa: bool,
    revoke_sessions: bool,
}

#[derive(Serialize)]
struct NewClient {
    client_id: String,
    name: String,
    confidential: bool,
    service: bool,
    redirect_uris: Vec<String>,
    scopes: BTreeSet<String>,
    allowed_groups: BTreeSet<String>,
    require_mfa: bool,
    settings: Value,
}

#[derive(Serialize)]
struct ClientPatch {
    name: Option<String>,
    enabled: Option<bool>,
    redirect_uris: Option<Vec<String>>,
    scopes: Option<BTreeSet<String>>,
    allowed_groups: Option<BTreeSet<String>>,
    require_mfa: Option<bool>,
    settings: Option<Value>,
}

pub(crate) async fn user(
    remote: &Remote,
    command: UserCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        UserCommand::List => {
            remote
                .authenticated(Method::GET, "/api/users", None::<&()>)
                .await
        }
        UserCommand::Create {
            username,
            email,
            name,
            admin,
            password_stdin,
        } => {
            let password = read_password(password_stdin, options.non_interactive)?;
            let name = name.as_deref().unwrap_or(&username);
            let input = NewUser {
                username: &username,
                password: &password,
                email: email.as_deref(),
                display_name: name,
                admin,
            };
            mutate(remote, Method::POST, "/api/users", Some(&input), options).await
        }
        UserCommand::Update {
            username,
            enabled,
            admin,
            email,
            name,
            password_stdin,
            reset_mfa,
            revoke_sessions,
        } => {
            if enabled.is_none()
                && admin.is_none()
                && email.is_none()
                && name.is_none()
                && !password_stdin
                && !reset_mfa
                && !revoke_sessions
            {
                bail!("Supply at least one user change");
            }
            let path = format!("/api/users/{}", segment(&username)?);
            let password = password_stdin
                .then(|| read_password(true, options.non_interactive))
                .transpose()?;
            let patch = UserPatch {
                enabled,
                admin,
                password: password.as_deref().map(String::as_str),
                email: email.as_deref(),
                display_name: name.as_deref(),
                reset_mfa,
                revoke_sessions,
            };
            mutate(remote, Method::PATCH, &path, Some(&patch), options).await
        }
    }
}

pub(crate) async fn group(
    remote: &Remote,
    command: GroupCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        GroupCommand::List => {
            remote
                .authenticated(Method::GET, "/api/groups", None::<&()>)
                .await
        }
        GroupCommand::Create { name } => {
            mutate(
                remote,
                Method::POST,
                "/api/groups",
                Some(&json!({"name":name})),
                options,
            )
            .await
        }
        GroupCommand::AddMember { group, username } => {
            let path = member_path(&group, &username)?;
            mutate(remote, Method::PUT, &path, None::<&()>, options).await
        }
        GroupCommand::RemoveMember { group, username } => {
            let path = member_path(&group, &username)?;
            mutate(remote, Method::DELETE, &path, None::<&()>, options).await
        }
    }
}

pub(crate) async fn client(
    remote: &Remote,
    command: ClientCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        ClientCommand::List => {
            remote
                .authenticated(Method::GET, "/api/clients", None::<&()>)
                .await
        }
        ClientCommand::Create {
            client_id,
            name,
            confidential,
            service,
            native,
            settings_file,
            redirect_uris,
            scopes,
            groups,
            require_mfa,
            secret_file,
        } => {
            let mut settings = match settings_file {
                Some(path) => read_settings(&path)?,
                None => json!({}),
            };
            if native {
                settings
                    .as_object_mut()
                    .context("Provider settings must be a JSON object")?
                    .insert("native".into(), Value::Bool(true));
            }
            let shared_secret = (confidential || service)
                && settings
                    .get("token_endpoint_auth_method")
                    .and_then(Value::as_str)
                    != Some("private_key_jwt");
            if shared_secret && secret_file.is_none() {
                bail!("Use --secret-file to save the one-time client secret");
            }
            let mut destination = secret_file.map(SecretFile::reserve).transpose()?;
            let scopes = if scopes.is_empty() {
                if service {
                    vec!["api".into()]
                } else {
                    ["openid", "profile", "email", "offline_access"]
                        .map(str::to_owned)
                        .to_vec()
                }
            } else {
                scopes
            };
            let input = NewClient {
                name: name.unwrap_or_else(|| client_id.clone()),
                client_id,
                confidential,
                service,
                redirect_uris,
                scopes: scopes.into_iter().collect(),
                allowed_groups: groups.into_iter().collect(),
                require_mfa,
                settings,
            };
            let result =
                mutate(remote, Method::POST, "/api/clients", Some(&input), options).await?;
            protect_secret(result, destination.as_mut(), shared_secret)
        }
        ClientCommand::Update {
            client_id,
            name,
            enabled,
            redirect_uris,
            scopes,
            groups,
            allow_all,
            require_mfa,
            settings_file,
        } => {
            if name.is_none()
                && enabled.is_none()
                && redirect_uris.is_none()
                && scopes.is_none()
                && groups.is_none()
                && !allow_all
                && require_mfa.is_none()
                && settings_file.is_none()
            {
                bail!("Supply at least one application change");
            }
            let path = format!("/api/clients/{}", segment(&client_id)?);
            let settings = settings_file.map(|path| read_settings(&path)).transpose()?;
            let patch = ClientPatch {
                name,
                enabled,
                redirect_uris,
                scopes: scopes.map(|values| values.into_iter().collect()),
                allowed_groups: if allow_all {
                    Some(BTreeSet::new())
                } else {
                    groups.map(|values| values.into_iter().collect())
                },
                require_mfa,
                settings,
            };
            mutate(remote, Method::PATCH, &path, Some(&patch), options).await
        }
        ClientCommand::RotateSecret {
            client_id,
            secret_file,
        } => {
            let path = format!("/api/clients/{}/rotate-secret", segment(&client_id)?);
            let mut destination = SecretFile::reserve(secret_file)?;
            let result = mutate(remote, Method::POST, &path, None::<&()>, options).await?;
            protect_secret(result, Some(&mut destination), true)
        }
    }
}

async fn mutate<T: Serialize + ?Sized>(
    remote: &Remote,
    method: Method,
    path: &str,
    body: Option<&T>,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    remote
        .mutate(
            method,
            path,
            body,
            options.run_id,
            options.if_revision,
            options.idempotency_key,
        )
        .await
}

fn member_path(group: &str, username: &str) -> Result<String> {
    Ok(format!(
        "/api/groups/{}/members/{}",
        segment(group)?,
        segment(username)?
    ))
}

// Path arguments use the server's name alphabet, which contains no path or
// query delimiters. The server still performs its own resource validation.
fn segment(value: &str) -> Result<&str> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.@".contains(&byte))
    {
        bail!("Invalid resource name in API path");
    }
    Ok(value)
}

fn read_settings(path: &Path) -> Result<Value> {
    let file = File::open(path).context("Cannot read provider settings")?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_SETTINGS_BYTES {
        bail!("Provider settings must be a regular JSON file of at most 32 KiB");
    }
    let mut bytes = Vec::new();
    file.take(MAX_SETTINGS_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_SETTINGS_BYTES {
        bail!("Provider settings exceed 32 KiB");
    }
    let settings: Value =
        serde_json::from_slice(&bytes).context("Invalid provider settings JSON")?;
    if !settings.is_object() {
        bail!("Provider settings must be a JSON object");
    }
    Ok(settings)
}

struct SecretFile {
    path: PathBuf,
    file: Option<File>,
    complete: bool,
}

impl SecretFile {
    fn reserve(path: PathBuf) -> Result<Self> {
        if path.file_name().is_none() {
            bail!("Secret file must name a new file");
        }
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options
            .open(&path)
            .context("Cannot reserve new private secret file")?;
        Ok(Self {
            path,
            file: Some(file),
            complete: false,
        })
    }

    fn write(&mut self, value: &Value) -> Result<()> {
        let mut bytes = Zeroizing::new(serde_json::to_vec_pretty(value)?);
        bytes.push(b'\n');
        let file = self.file.as_mut().context("Client secret file is closed")?;
        file.write_all(&bytes)
            .context("Cannot write client secret file")?;
        file.sync_all().context("Cannot sync client secret file")?;
        bytes.zeroize();
        self.complete = true;
        Ok(())
    }
}

impl Drop for SecretFile {
    fn drop(&mut self) {
        self.file.take();
        if !self.complete {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn protect_secret(
    mut result: Value,
    destination: Option<&mut SecretFile>,
    expected: bool,
) -> Result<Value> {
    let saved = (|| -> Result<Option<PathBuf>> {
        let secret = match result.get("client_secret") {
            Some(Value::String(secret)) if !secret.is_empty() => true,
            Some(Value::Null) | None => false,
            _ => bail!("Server returned an invalid client secret response"),
        };
        if expected && !secret {
            bail!("Server did not return the expected one-time client secret");
        }
        if secret {
            let destination =
                destination.context("Server issued a secret without a private destination")?;
            destination.write(&result)?;
            Ok(Some(destination.path.clone()))
        } else {
            Ok(None)
        }
    })();
    if let Some(fields) = result.as_object_mut() {
        if let Some(Value::String(secret)) = fields.get_mut("client_secret") {
            secret.zeroize();
        }
        fields.remove("client_secret");
    }
    if let Some(path) = saved? {
        result
            .as_object_mut()
            .context("Invalid client response")?
            .insert("secret_file".into(), json!(path));
    }
    Ok(result)
}
