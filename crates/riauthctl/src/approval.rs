use crate::{
    read_password, read_prompt,
    transport::{AuthorizationPreparation, Remote, VerifiedIssuer},
};
use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use reqwest::Method;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
};
use url::Url;
use zeroize::Zeroizing;

#[derive(Serialize)]
struct PasswordLogin<'a> {
    username: &'a str,
    password: &'a str,
    otp: Option<&'a str>,
    transaction_id: Option<&'a str>,
}

#[derive(Args)]
pub(crate) struct Reauthentication {
    /// Select this account when the request permits it; device approval must keep the current account.
    #[arg(long)]
    username: Option<String>,
    /// Read the password from one line of standard input.
    #[arg(long, conflicts_with = "passkey")]
    password_stdin: bool,
    /// Use a terminal USB passkey for the fresh sign-in.
    #[arg(long, conflicts_with = "password_stdin")]
    passkey: bool,
    /// Prompt for an authenticator code when RIAUTH_OTP is unset.
    #[arg(long, conflicts_with = "passkey")]
    mfa: bool,
}

impl Reauthentication {
    fn requested(&self) -> bool {
        self.username.is_some() || self.password_stdin || self.passkey || self.mfa
    }

    async fn perform(
        &self,
        remote: &Remote,
        verified: &VerifiedIssuer,
        username: &str,
        transaction_id: Option<&str>,
        non_interactive: bool,
    ) -> Result<()> {
        if self.passkey {
            crate::usb::require_support()?;
            if non_interactive {
                bail!("Terminal USB passkeys need touch/PIN input; run interactively");
            }
            let start = remote
                .request_api(
                    verified,
                    Method::POST,
                    "/api/passkey/authentication/start",
                    Some(&json!({"username": username, "transaction_id": transaction_id})),
                    None,
                    None,
                )
                .await?;
            let ceremony = start
                .get("ceremony")
                .filter(|v| v.is_string())
                .context("Passkey start response is missing a ceremony")?;
            let public_key = start
                .get("public_key")
                .filter(|v| v.is_object())
                .context("Passkey start response is missing public_key")?;
            let response = crate::usb::perform(&remote.issuer, public_key.clone(), false).await?;
            let finish = remote
                .request_api(
                    verified,
                    Method::POST,
                    "/api/passkey/authentication/finish",
                    Some(&json!({"ceremony": ceremony, "response": response})),
                    None,
                    None,
                )
                .await?;
            remote.save_login(&finish, verified)?;
        } else {
            let password = read_password(self.password_stdin, non_interactive)?;
            let otp = match std::env::var("RIAUTH_OTP") {
                Ok(value) if !value.is_empty() => Some(Zeroizing::new(value)),
                _ if self.mfa => Some(read_prompt("One-time code: ", non_interactive)?),
                _ => None,
            };
            if otp.as_ref().is_some_and(|code| code.len() > 128) {
                bail!("One-time code exceeds 128 bytes");
            }
            let login = remote
                .request_api(
                    verified,
                    Method::POST,
                    "/api/login",
                    Some(&PasswordLogin {
                        username,
                        password: password.as_str(),
                        otp: otp.as_ref().map(|value| value.as_str()),
                        transaction_id,
                    }),
                    None,
                    None,
                )
                .await?;
            remote.save_login(&login, verified)?;
        }
        Ok(())
    }
}

#[derive(Subcommand)]
pub(crate) enum RequestCommand {
    /// Review the exact application, scopes and browser that requested authorization.
    Inspect { code: String },
    /// Approve a pending browser authorization after review.
    Approve {
        code: String,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        remember: bool,
        #[command(flatten)]
        reauthentication: Reauthentication,
    },
    /// Deny a pending browser authorization after review.
    Deny { code: String },
}

#[derive(Subcommand)]
pub(crate) enum DeviceCommand {
    /// Review the exact application and scopes requested by a device.
    Inspect { code: String },
    /// Approve a device with a fresh human sign-in after review.
    Approve {
        code: String,
        #[arg(long)]
        yes: bool,
        #[command(flatten)]
        reauthentication: Reauthentication,
    },
    /// Deny a pending device request after review.
    Deny { code: String },
}

pub(crate) async fn request(
    remote: &Remote,
    command: RequestCommand,
    non_interactive: bool,
) -> Result<Value> {
    end_user_only(remote)?;
    let verified = remote.verify_issuer().await?;
    let saved = remote.human_session(&verified)?;
    let code = match &command {
        RequestCommand::Inspect { code }
        | RequestCommand::Approve { code, .. }
        | RequestCommand::Deny { code } => code,
    };
    let details = remote
        .request_api(
            &verified,
            Method::GET,
            &format!("/api/authorization/{}", segment(code)?),
            None::<&()>,
            Some(&saved.token),
            None,
        )
        .await?;
    let review = authorization_review(&details)?;
    match command {
        RequestCommand::Inspect { .. } => {
            if details.get("source_stage").is_some_and(Value::is_object) {
                Ok(source_stage_notice(review, &details))
            } else {
                Ok(review)
            }
        }
        RequestCommand::Deny { code } => {
            show_review(&review)?;
            let result = remote
                .request_api(
                    &verified,
                    Method::POST,
                    "/api/authorization/decision",
                    Some(&json!({"code": code, "approve": false})),
                    Some(&saved.token),
                    None,
                )
                .await?;
            Ok(json!({"review": review, "result": result}))
        }
        RequestCommand::Approve {
            code,
            yes,
            remember,
            reauthentication,
        } => {
            show_review(&review)?;
            require_no_source_stage(&details)?;
            let transaction = required_transaction(&details)?;
            let required = details
                .get("reauthentication_required")
                .and_then(Value::as_bool)
                .context("Authorization review is missing reauthentication_required")?;
            let selected = match details.get("select_account").and_then(Value::as_bool) {
                Some(selected) => selected,
                None if details.get("protocol").and_then(Value::as_str) == Some("saml") => false,
                None => bail!("Authorization review is missing select_account"),
            };
            if selected && reauthentication.username.is_none() {
                bail!("Account selection requires --username");
            }
            if !yes
                && !confirm(
                    "Approve this application and return its browser to the callback? [y/N] ",
                    non_interactive,
                )?
            {
                return Ok(json!({"submitted": false, "review": review}));
            }
            let bearer = if required || reauthentication.requested() {
                let username = reauthentication
                    .username
                    .as_deref()
                    .or_else(|| details.get("username").and_then(Value::as_str))
                    .context("Fresh authentication requires --username")?;
                reauthentication
                    .perform(
                        remote,
                        &verified,
                        username,
                        Some(transaction),
                        non_interactive,
                    )
                    .await?;
                remote.human_session(&verified)?
            } else {
                saved
            };
            let result = remote.request_api(
                &verified, Method::POST, "/api/authorization/decision",
                Some(&json!({"code": code, "approve": true, "remember": remember, "transaction_id": transaction})),
                Some(&bearer.token), None,
            ).await?;
            Ok(json!({"review": review, "result": result}))
        }
    }
}

pub(crate) async fn device(
    remote: &Remote,
    command: DeviceCommand,
    non_interactive: bool,
) -> Result<Value> {
    end_user_only(remote)?;
    let verified = remote.verify_issuer().await?;
    let saved = remote.human_session(&verified)?;
    let code = match &command {
        DeviceCommand::Inspect { code }
        | DeviceCommand::Approve { code, .. }
        | DeviceCommand::Deny { code } => code,
    };
    let details = remote
        .request_api(
            &verified,
            Method::GET,
            &format!("/api/device/{}", segment(code)?),
            None::<&()>,
            Some(&saved.token),
            None,
        )
        .await?;
    let review = device_review(&details)?;
    match command {
        DeviceCommand::Inspect { .. } => Ok(review),
        DeviceCommand::Deny { code } => {
            show_review(&review)?;
            let result = remote
                .request_api(
                    &verified,
                    Method::POST,
                    "/api/device/decision",
                    Some(&json!({"user_code": code, "approve": false})),
                    Some(&saved.token),
                    None,
                )
                .await?;
            Ok(json!({"review": review, "result": result}))
        }
        DeviceCommand::Approve {
            code,
            yes,
            reauthentication,
        } => {
            show_review(&review)?;
            if !yes
                && !confirm(
                    "Approve this device and its requested scopes? [y/N] ",
                    non_interactive,
                )?
            {
                return Ok(json!({"submitted": false, "review": review}));
            }
            let before = remote
                .request_api(
                    &verified,
                    Method::GET,
                    "/api/me",
                    None::<&()>,
                    Some(&saved.token),
                    None,
                )
                .await?;
            let username = before
                .pointer("/user/username")
                .and_then(Value::as_str)
                .context("Current session has no username")?;
            if reauthentication
                .username
                .as_deref()
                .is_some_and(|selected| selected != username)
            {
                bail!("Device approval must use the account shown by the current session");
            }
            reauthentication
                .perform(remote, &verified, username, None, non_interactive)
                .await?;
            let fresh = remote.human_session(&verified)?;
            let after = remote
                .request_api(
                    &verified,
                    Method::GET,
                    "/api/me",
                    None::<&()>,
                    Some(&fresh.token),
                    None,
                )
                .await?;
            if after.pointer("/user/username").and_then(Value::as_str) != Some(username) {
                bail!("Fresh sign-in changed the device approval account; review again");
            }
            let latest = remote
                .request_api(
                    &verified,
                    Method::GET,
                    &format!("/api/device/{}", segment(&code)?),
                    None::<&()>,
                    Some(&fresh.token),
                    None,
                )
                .await?;
            if device_review(&latest)? != review {
                bail!("Device request changed after review; inspect it again before approving");
            }
            let result = remote
                .request_api(
                    &verified,
                    Method::POST,
                    "/api/device/decision",
                    Some(&json!({"user_code": code, "approve": true})),
                    Some(&fresh.token),
                    None,
                )
                .await?;
            Ok(json!({"review": review, "result": result}))
        }
    }
}

pub(crate) async fn authorize(
    remote: &Remote,
    url: &str,
    yes: bool,
    deny: bool,
    callback_file: Option<&Path>,
    reauthentication: Reauthentication,
    non_interactive: bool,
) -> Result<Value> {
    end_user_only(remote)?;
    let verified = remote.verify_issuer().await?;
    let parsed = Url::parse(url).context("Invalid authorization URL")?;
    let expected = Url::parse(&format!("{}/oauth/authorize", verified.api_base))?;
    if parsed.origin() != expected.origin()
        || parsed.path() != expected.path()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.fragment().is_some()
        || parsed.query().is_none()
    {
        bail!("Authorization URL must be the complete /oauth/authorize URL at --server");
    }
    let pairs: Vec<(String, String)> = parsed.query_pairs().into_owned().collect();
    if pairs.iter().any(|(key, _)| {
        matches!(
            key.as_str(),
            "decision" | "transaction_id" | "request_binding"
        )
    }) {
        bail!("Authorization URL contains a reserved decision or transaction parameter");
    }
    if pairs.iter().any(|(key, value)| {
        key == "response_mode" && matches!(value.as_str(), "form_post" | "form_post.jwt")
    }) {
        bail!("Terminal authorization cannot deliver form_post; use the browser approval flow");
    }
    let mut destination = callback_file
        .map(|path| CallbackFile::reserve(path.to_owned()))
        .transpose()?;
    let path = format!("/oauth/authorize?{}", parsed.query().unwrap_or_default());
    let saved = remote.human_session_if_present(&verified)?;
    let prepared = remote
        .authorization_prepare(
            &verified,
            &path,
            saved.as_ref().map(|value| value.token.as_str()),
        )
        .await?;
    let details = match prepared {
        AuthorizationPreparation::Redirect(callback) => {
            return callback_result(callback, destination.as_mut(), false);
        }
        AuthorizationPreparation::Details(details) => details,
    };
    let review = authorization_review(&details)?;
    show_review(&review)?;
    if details.get("source_stage").is_some_and(Value::is_object) {
        return Ok(source_stage_notice(review, &details));
    }
    if details
        .get("response_mode")
        .context("Authorization review is missing response_mode; upgrade the server before terminal authorization")?
        .as_str()
        .is_some_and(|mode| matches!(mode, "form_post" | "form_post.jwt"))
    {
        bail!("Terminal authorization cannot deliver form_post; use the browser approval flow");
    }
    let transaction = required_transaction(&details)?;
    let required = details
        .get("reauthentication_required")
        .and_then(Value::as_bool)
        .context("Authorization review is missing reauthentication_required")?;
    let selected = details
        .get("select_account")
        .and_then(Value::as_bool)
        .context("Authorization review is missing select_account")?;
    if !deny && selected && reauthentication.username.is_none() {
        bail!("Account selection requires --username");
    }
    if !deny
        && !yes
        && !confirm(
            "Allow this application to access these scopes? [y/N] ",
            non_interactive,
        )?
    {
        return Ok(json!({"submitted": false, "review": review}));
    }
    let bearer = if !deny && (required || reauthentication.requested()) {
        let username = reauthentication
            .username
            .as_deref()
            .or_else(|| details.get("username").and_then(Value::as_str))
            .context("Fresh authentication requires --username")?;
        reauthentication
            .perform(
                remote,
                &verified,
                username,
                Some(transaction),
                non_interactive,
            )
            .await?;
        remote.human_session(&verified)?
    } else {
        saved.context("Sign in with `riauthctl login` before deciding this authorization")?
    };
    let mut decision_pairs = pairs;
    decision_pairs.push((
        "decision".into(),
        if deny { "deny" } else { "approve" }.into(),
    ));
    decision_pairs.push(("transaction_id".into(), transaction.into()));
    let callback = remote
        .authorization_decide(&verified, &decision_pairs, &bearer.token)
        .await?;
    callback_result(callback, destination.as_mut(), true)
}

fn callback_result(
    callback: String,
    destination: Option<&mut CallbackFile>,
    submitted: bool,
) -> Result<Value> {
    let callback = Zeroizing::new(callback);
    if let Some(destination) = destination {
        let path = destination.write(&callback)?;
        return Ok(json!({"callback_file": path, "submitted": submitted}));
    }
    Ok(json!({"redirect_uri": callback.as_str(), "submitted": submitted}))
}

struct CallbackFile {
    path: PathBuf,
    file: Option<File>,
    complete: bool,
}

impl CallbackFile {
    fn reserve(path: PathBuf) -> Result<Self> {
        if path.file_name().is_none() {
            bail!("Callback file must name a new file");
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
            .context("Cannot reserve new private callback file")?;
        Ok(Self {
            path,
            file: Some(file),
            complete: false,
        })
    }

    fn write(&mut self, callback: &str) -> Result<&Path> {
        let file = self.file.as_mut().context("Callback file is closed")?;
        file.write_all(callback.as_bytes())
            .context("Cannot write callback file")?;
        file.write_all(b"\n")
            .context("Cannot write callback file")?;
        file.sync_all().context("Cannot sync callback file")?;
        self.complete = true;
        Ok(&self.path)
    }
}

impl Drop for CallbackFile {
    fn drop(&mut self) {
        self.file.take();
        if !self.complete {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn authorization_review(details: &Value) -> Result<Value> {
    let mut review = identity_review(details)?;
    let redirect = details
        .get("redirect_uri")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("Authorization review is missing redirect_uri")?;
    review["redirect_uri"] = json!(redirect);
    for field in [
        "resource",
        "response_mode",
        "requested_from",
        "require_mfa",
        "reauthentication_required",
        "select_account",
        "username",
        "delivery",
        "protocol",
        "sp_entity_id",
        "attributes",
        "name_id_format",
        "requested_authn_context",
    ] {
        if let Some(value) = details.get(field) {
            review[field] = value.clone();
        }
    }
    Ok(review)
}

fn source_stage_notice(review: Value, details: &Value) -> Value {
    json!({
        "review": review,
        "source_stage_required": true,
        "expires_at": details.pointer("/source_stage/expires_at"),
        "instruction": "Complete the upstream source stage in the original browser before deciding this request",
        "submitted": false,
    })
}

fn device_review(details: &Value) -> Result<Value> {
    let mut review = identity_review(details)?;
    let expiry = details
        .get("expires_at")
        .and_then(Value::as_u64)
        .context("Device review is missing expires_at")?;
    review["expires_at"] = json!(expiry);
    for field in ["claims", "resource", "require_mfa"] {
        if let Some(value) = details.get(field) {
            review[field] = value.clone();
        }
    }
    Ok(review)
}

fn identity_review(details: &Value) -> Result<Value> {
    let client = details
        .get("client_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("Review is missing client_id")?;
    let application = details
        .get("application")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("Review is missing application")?;
    let scopes = details
        .get("scopes")
        .and_then(Value::as_array)
        .filter(|values| {
            !values.is_empty()
                && values
                    .iter()
                    .all(|value| value.as_str().is_some_and(|s| !s.is_empty()))
        })
        .context("Review is missing requested scopes")?;
    Ok(json!({"client_id": client, "application": application, "scopes": scopes}))
}

fn required_transaction(details: &Value) -> Result<&str> {
    details
        .get("transaction_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("Authorization review is missing its authentication transaction")
}

fn require_no_source_stage(details: &Value) -> Result<()> {
    if details.get("source_stage").is_some_and(Value::is_object) {
        bail!("Complete the embedded source stage in the browser before approving");
    }
    Ok(())
}

fn end_user_only(remote: &Remote) -> Result<()> {
    if remote.is_agent() {
        bail!("Agent credentials cannot authenticate or consent as an end user");
    }
    Ok(())
}

fn segment(value: &str) -> Result<&str> {
    if value.is_empty()
        || value.len() > 256
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        bail!("Invalid approval code");
    }
    Ok(value)
}

fn show_review(review: &Value) -> Result<()> {
    eprintln!("{}", serde_json::to_string_pretty(review)?);
    Ok(())
}

fn confirm(message: &str, non_interactive: bool) -> Result<bool> {
    if non_interactive || !io::stdin().is_terminal() {
        bail!("Use --yes to approve without a terminal prompt");
    }
    eprint!("{message}");
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
