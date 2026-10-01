use crate::session::{self, AgentCredential, SavedSession};
use anyhow::{Context, Result, bail};
use reqwest::{Client, Method, Response, StatusCode, header};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    fmt, fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
use url::{Host, Url};

const MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const MAX_CA_BYTES: u64 = 1024 * 1024;

pub(crate) struct Remote {
    pub(crate) issuer: String,
    http: Client,
    session_file: PathBuf,
    agent_file: Option<PathBuf>,
    private_default_dir: bool,
}

pub(crate) struct VerifiedIssuer {
    pub(crate) api_base: String,
}

pub(crate) enum AuthorizationPreparation {
    Details(Value),
    Redirect(String),
}

struct MutationHeaders {
    revision: Option<u64>,
    idempotency_key: String,
}

struct RequestHeaders<'a> {
    bearer: Option<&'a str>,
    run_id: Option<&'a str>,
    mutation: Option<&'a MutationHeaders>,
}

pub(crate) enum Credential {
    Session(SavedSession),
    Agent(AgentCredential),
}
impl Credential {
    pub(crate) fn token(&self) -> &str {
        match self {
            Self::Session(saved) => &saved.token,
            Self::Agent(saved) => &saved.token,
        }
    }
}

#[derive(Debug)]
pub(crate) struct HttpFailure {
    pub(crate) status: u16,
    pub(crate) code: String,
}

impl fmt::Display for HttpFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP {} {}", self.status, self.code)
    }
}
impl std::error::Error for HttpFailure {}

impl Remote {
    pub(crate) fn new(
        issuer: &str,
        session_file: Option<PathBuf>,
        agent_file: Option<PathBuf>,
        ca_cert: Option<&Path>,
        request_timeout: u64,
    ) -> Result<Self> {
        validate_server_url(issuer)?;
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(request_timeout))
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy();
        if let Some(path) = ca_cert {
            let file = fs::File::open(path).context("Cannot read CA certificate")?;
            let metadata = file.metadata()?;
            if !metadata.is_file() || metadata.len() > MAX_CA_BYTES {
                bail!("CA certificate must be a bounded regular file");
            }
            let mut pem = Vec::new();
            file.take(MAX_CA_BYTES + 1).read_to_end(&mut pem)?;
            if pem.len() as u64 > MAX_CA_BYTES {
                bail!("CA certificate exceeds its size limit");
            }
            builder = builder.add_root_certificate(reqwest::Certificate::from_pem(&pem)?);
        }
        let private_default_dir = session_file.is_none();
        let session_file = session_file.map(Ok).unwrap_or_else(session::default_path)?;
        Ok(Self {
            issuer: issuer.into(),
            http: builder.build()?,
            session_file,
            agent_file,
            private_default_dir,
        })
    }

    pub(crate) async fn status(&self) -> Result<Value> {
        let verified = self.verify_issuer().await?;
        let status = self
            .request_api(&verified, Method::GET, "/healthz", None::<&()>, None, None)
            .await?;
        if status.get("status").and_then(Value::as_str) != Some("ok")
            || status.get("issuer").and_then(Value::as_str) != Some(&verified.api_base)
        {
            bail!("Health endpoint did not report ready status");
        }
        Ok(status)
    }

    /// Read discovery at the explicitly selected issuer, without credentials.
    pub(crate) async fn discovery(&self) -> Result<Value> {
        let document = self
            .request_at(
                &self.issuer,
                Method::GET,
                "/.well-known/openid-configuration",
                None::<&()>,
                RequestHeaders {
                    bearer: None,
                    run_id: None,
                    mutation: None,
                },
            )
            .await?;
        if document.get("issuer").and_then(Value::as_str) != Some(&self.issuer) {
            bail!("Discovery issuer does not match --server");
        }
        Ok(document)
    }

    /// A provider-only issuer has discovery but no management routes. Never
    /// follow its primary endpoint URLs with a password or saved bearer token.
    pub(crate) async fn verify_issuer(&self) -> Result<VerifiedIssuer> {
        let document = self.discovery().await?;
        let candidate = management_base(&document)?;
        if candidate.trim_end_matches('/') != self.issuer.trim_end_matches('/') {
            bail!(
                "This provider issuer has no management API; use the primary management issuer with --server {candidate}"
            );
        }
        Ok(VerifiedIssuer {
            api_base: self.issuer.clone(),
        })
    }

    pub(crate) fn credential(&self, verified: &VerifiedIssuer) -> Result<Credential> {
        if let Some(path) = &self.agent_file {
            return Ok(Credential::Agent(session::read_agent(
                path,
                &verified.api_base,
            )?));
        }
        Ok(Credential::Session(self.human_session(verified)?))
    }

    pub(crate) fn human_session(&self, verified: &VerifiedIssuer) -> Result<SavedSession> {
        if self.agent_file.is_some() {
            bail!("Agent credentials cannot sign in or manage a human session");
        }
        let saved = session::read(&self.session_file, &self.issuer)?;
        if saved.api_base.as_deref() != Some(&verified.api_base)
            && !(saved.api_base.is_none() && self.issuer == verified.api_base)
        {
            bail!("Saved session belongs to another management API; log in again");
        }
        Ok(saved)
    }

    pub(crate) fn human_session_if_present(
        &self,
        verified: &VerifiedIssuer,
    ) -> Result<Option<SavedSession>> {
        if self.agent_file.is_some() {
            bail!("Agent credentials cannot authorize as an end user");
        }
        match fs::symlink_metadata(&self.session_file) {
            Ok(_) => self.human_session(verified).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error).context("Cannot inspect saved session"),
        }
    }

    pub(crate) fn is_agent(&self) -> bool {
        self.agent_file.is_some()
    }

    pub(crate) fn save_login(&self, output: &Value, verified: &VerifiedIssuer) -> Result<Value> {
        let token = output
            .get("session_token")
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .context("Login response is missing a session token")?;
        let expires_at = output
            .get("expires_at")
            .and_then(Value::as_u64)
            .context("Login response is missing a session expiry")?;
        if expires_at <= session::now()? {
            bail!("Login response contains an expired session");
        }
        let saved = SavedSession {
            issuer: self.issuer.clone(),
            api_base: Some(verified.api_base.clone()),
            token: token.into(),
            expires_at,
        };
        session::write(&self.session_file, &saved, self.private_default_dir)?;
        Ok(json!({
            "user": output.get("user").cloned().unwrap_or(Value::Null),
            "expires_at": expires_at,
            "session_file": self.session_file,
        }))
    }

    pub(crate) fn remove_session(&self) -> Result<()> {
        fs::remove_file(&self.session_file).context("Cannot remove saved session")
    }

    pub(crate) async fn authenticated<T: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
    ) -> Result<Value> {
        let verified = self.verify_issuer().await?;
        let credential = self.credential(&verified)?;
        self.request_api(
            &verified,
            method,
            path,
            body,
            Some(credential.token()),
            None,
        )
        .await
    }

    pub(crate) async fn request_api<T: Serialize + ?Sized>(
        &self,
        verified: &VerifiedIssuer,
        method: Method,
        path: &str,
        body: Option<&T>,
        bearer: Option<&str>,
        run_id: Option<&str>,
    ) -> Result<Value> {
        self.request_at(
            &verified.api_base,
            method,
            path,
            body,
            RequestHeaders {
                bearer,
                run_id,
                mutation: None,
            },
        )
        .await
    }

    /// Session revocation has a caller-bound receipt but no configuration
    /// revision. Preserve the exact selected-session request key across CLI retries.
    pub(crate) async fn revoke_session_receipted(
        &self,
        verified: &VerifiedIssuer,
        path: &str,
        bearer: &str,
        run_id: Option<&str>,
        idempotency_key: &str,
    ) -> Result<Value> {
        let mutation = MutationHeaders {
            revision: None,
            idempotency_key: idempotency_key.to_owned(),
        };
        self.request_at(
            &verified.api_base,
            Method::DELETE,
            path,
            None::<&()>,
            RequestHeaders {
                bearer: Some(bearer),
                run_id,
                mutation: Some(&mutation),
            },
        )
        .await
    }

    /// OAuth authorization deliberately returns a callback redirect instead of JSON.
    /// The HTTP client never follows that redirect with a session bearer.
    pub(crate) async fn authorization_prepare(
        &self,
        verified: &VerifiedIssuer,
        path: &str,
        bearer: Option<&str>,
    ) -> Result<AuthorizationPreparation> {
        if !path.starts_with("/oauth/authorize?") || path.contains('#') {
            bail!("Invalid authorization path");
        }
        let mut request = self.http.get(format!("{}{}", verified.api_base, path));
        if let Some(token) = bearer {
            request = request.bearer_auth(token);
        }
        let response = request
            .send()
            .await
            .context("Authorization preparation request failed")?;
        if response.status() == StatusCode::FOUND {
            return Ok(AuthorizationPreparation::Redirect(callback_location(
                &response,
            )?));
        }
        Ok(AuthorizationPreparation::Details(
            decode_response(response).await?,
        ))
    }

    pub(crate) async fn authorization_decide(
        &self,
        verified: &VerifiedIssuer,
        pairs: &[(String, String)],
        bearer: &str,
    ) -> Result<String> {
        let body = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(pairs)
            .finish();
        let response = self
            .http
            .post(format!("{}/oauth/authorize", verified.api_base))
            .bearer_auth(bearer)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .context("Authorization decision request failed")?;
        if response.status() == StatusCode::FOUND {
            return callback_location(&response);
        }
        // Preserve the server's sanitized failure code, including when the
        // request reached a different response mode than the client supports.
        decode_response(response).await?;
        bail!("Authorization did not return a callback redirect")
    }

    /// Direct management writes use the same conditional and receipt envelope
    /// as the server's other remote adapters. The server remains authoritative
    /// for permissions, validation, auditing, and revision conflicts.
    pub(crate) async fn mutate<T: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
        run_id: Option<&str>,
        if_revision: Option<u64>,
        idempotency_key: Option<&str>,
    ) -> Result<Value> {
        let verified = self.verify_issuer().await?;
        let credential = self.credential(&verified)?;
        let revision = match if_revision {
            Some(revision) => revision,
            None => self
                .request_api(
                    &verified,
                    Method::GET,
                    "/api/state/revision",
                    None::<&()>,
                    Some(credential.token()),
                    run_id,
                )
                .await?
                .get("revision")
                .and_then(Value::as_u64)
                .context("Revision response is missing a numeric revision")?,
        };
        let headers = MutationHeaders {
            revision: Some(revision),
            idempotency_key: idempotency_key
                .map(str::to_owned)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        };
        self.request_at(
            &verified.api_base,
            method,
            path,
            body,
            RequestHeaders {
                bearer: Some(credential.token()),
                run_id,
                mutation: Some(&headers),
            },
        )
        .await
    }

    async fn request_at<T: Serialize + ?Sized>(
        &self,
        base: &str,
        method: Method,
        path: &str,
        body: Option<&T>,
        headers: RequestHeaders<'_>,
    ) -> Result<Value> {
        if !path.starts_with('/') || path.starts_with("//") || path.contains('#') {
            bail!("Invalid API path");
        }
        let url = format!("{}{}", base.trim_end_matches('/'), path);
        let mut request = self.http.request(method, &url);
        if let Some(token) = headers.bearer {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        if let Some(run_id) = headers.run_id {
            request = request.header("x-riauth-run-id", run_id);
        }
        if let Some(mutation) = headers.mutation {
            if let Some(revision) = mutation.revision {
                request = request.header("if-match", format!("\"{revision}\""));
            }
            request = request.header("idempotency-key", &mutation.idempotency_key);
        }
        decode_response(request.send().await?).await
    }
}

fn callback_location(response: &Response) -> Result<String> {
    let location = response
        .headers()
        .get(header::LOCATION)
        .context("Authorization redirect is missing Location")?
        .to_str()
        .context("Authorization redirect has an invalid Location")?;
    let callback = Url::parse(location).context("Authorization callback is not an absolute URL")?;
    let private_scheme = !matches!(
        callback.scheme(),
        "http" | "https" | "file" | "data" | "javascript"
    ) && callback.scheme().contains('.')
        && !callback.path().is_empty();
    if !((matches!(callback.scheme(), "http" | "https") && callback.host_str().is_some())
        || private_scheme)
    {
        bail!("Authorization callback has an unsupported URL scheme");
    }
    Ok(location.to_owned())
}

fn management_base(document: &Value) -> Result<String> {
    let token = document
        .get("token_endpoint")
        .and_then(Value::as_str)
        .context("Discovery is missing token_endpoint")?;
    let base = token
        .strip_suffix("/oauth/token")
        .filter(|base| !base.is_empty())
        .context("Discovery token_endpoint is not a riAuth endpoint")?;
    validate_server_url(base)?;
    for (field, suffix) in [
        ("authorization_endpoint", "/oauth/authorize"),
        ("jwks_uri", "/oauth/jwks"),
        ("userinfo_endpoint", "/oauth/userinfo"),
    ] {
        if document.get(field).and_then(Value::as_str) != Some(format!("{base}{suffix}").as_str()) {
            bail!("Discovery management endpoints disagree");
        }
    }
    Ok(base.to_owned())
}

pub(crate) fn validate_server_url(value: &str) -> Result<Url> {
    let url = Url::parse(value).context("Invalid server URL")?;
    let loopback = match url.host() {
        Some(Host::Domain("localhost")) => true,
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    };
    if !(url.scheme() == "https" || url.scheme() == "http" && loopback)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.host_str().is_none()
        || url.as_str().trim_end_matches('/') != value.trim_end_matches('/')
    {
        bail!(
            "Server URL must be canonical HTTPS (HTTP loopback allowed), without credentials, queries or fragments"
        );
    }
    Ok(url)
}

async fn decode_response(mut response: Response) -> Result<Value> {
    let status = response.status();
    if status.is_redirection() {
        return Err(HttpFailure {
            status: status.as_u16(),
            code: "redirect_refused".into(),
        }
        .into());
    }
    if status == StatusCode::NO_CONTENT {
        return Ok(json!({}));
    }
    if response
        .content_length()
        .is_some_and(|size| size > MAX_RESPONSE_BYTES as u64)
    {
        bail!("Response body exceeds {} bytes", MAX_RESPONSE_BYTES);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if chunk.len() > MAX_RESPONSE_BYTES.saturating_sub(bytes.len()) {
            bail!("Response body exceeds {} bytes", MAX_RESPONSE_BYTES);
        }
        bytes.extend_from_slice(&chunk);
    }
    if !status.is_success() {
        let code = serde_json::from_slice::<Value>(&bytes)
            .ok()
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .filter(|code| {
                matches!(
                    code.as_str(),
                    "invalid_credentials"
                        | "invalid_token"
                        | "invalid_request"
                        | "access_denied"
                        | "mfa_required"
                        | "reauthentication_required"
                        | "rate_limited"
                        | "temporarily_unavailable"
                        | "not_ready"
                        | "not_found"
                        | "conflict"
                        | "credential_already_issued"
                        | "precondition_required"
                        | "permission_denied"
                )
            })
            .unwrap_or_else(|| "http_error".into());
        // Server-derived descriptions and unknown codes can echo credentials.
        return Err(HttpFailure {
            status: status.as_u16(),
            code,
        }
        .into());
    }
    serde_json::from_slice(&bytes)
        .with_context(|| format!("Server returned invalid JSON (HTTP {status})"))
}
