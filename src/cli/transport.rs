use super::*;

#[derive(Serialize, Deserialize)]
pub(super) struct SavedSession {
    pub(super) issuer: String,
    pub(super) token: String,
    pub(super) expires_at: u64,
}

pub(super) struct Remote {
    pub(super) issuer: String,
    pub(super) http: HttpClient,
    pub(super) session_file: PathBuf,
    pub(super) json: bool,
    pub(super) agent_file: Option<PathBuf>,
    pub(super) output_file: Option<PathBuf>,
    pub(super) run_id: Option<String>,
    pub(super) idempotency_key: Option<String>,
    pub(super) if_revision: Option<u64>,
    pub(super) show_secrets: bool,
}
impl Remote {
    pub(super) async fn reauthenticate(
        &self,
        username: &str,
        transaction: Option<&str>,
        passkey: bool,
        password_stdin: bool,
    ) -> Result<Value> {
        let authenticated = if passkey {
            if NON_INTERACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
                bail!(
                    "USB login needs touch/PIN input; use passkey start/finish with an authenticator client in noninteractive mode"
                );
            }
            let start = self
                .call(
                    Method::POST,
                    "/api/passkey/authentication/start",
                    Some(json!({"username":username,"transaction_id":transaction})),
                    false,
                )
                .await?;
            let response =
                crate::passkey::usb(&self.issuer, start["public_key"].clone(), false).await?;
            self.call(
                Method::POST,
                "/api/passkey/authentication/finish",
                Some(json!({"ceremony":start["ceremony"],"response":response})),
                false,
            )
            .await?
        } else {
            let password = read_password(password_stdin, false)?;
            self.call(Method::POST,"/api/login",Some(json!({"username":username,"password":password.as_str(),"otp":std::env::var("RIAUTH_OTP").ok(),"transaction_id":transaction})),false).await?
        };
        self.save_session(&authenticated)?;
        Ok(authenticated)
    }
    pub(super) fn save_session(&self, output: &Value) -> Result<()> {
        let saved = SavedSession {
            issuer: self.issuer.clone(),
            token: output["session_token"]
                .as_str()
                .context("Missing session token")?
                .into(),
            expires_at: output["expires_at"]
                .as_u64()
                .context("Missing session expiry")?,
        };
        if let Some(parent) = self
            .session_file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            && !parent.exists()
        {
            private_dir(parent)?;
        }
        write_private(&self.session_file, &serde_json::to_vec(&saved)?, true)
    }
    pub(super) fn new(cli: &Cli) -> Result<Self> {
        let issuer = match &cli.server {
            Some(server) => server.to_owned(),
            None if cli.config.exists() => Config::load(&cli.config)?.issuer,
            None => Config::default().issuer,
        };
        validate_server_url(&issuer)?;
        let mut http = HttpClient::builder()
            .timeout(Duration::from_secs(cli.request_timeout))
            .redirect(reqwest::redirect::Policy::none());
        if let Some(path) = &cli.ca_cert {
            http = http.add_root_certificate(reqwest::Certificate::from_pem(&fs::read(path)?)?);
        }
        let session_file = match &cli.session_file {
            Some(path) => path.clone(),
            None => {
                let base = std::env::var_os("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .or_else(|| {
                        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
                    })
                    .context("Use --session-file when HOME and XDG_CONFIG_HOME are unset")?;
                base.join("riauth/session.json")
            }
        };
        Ok(Self {
            issuer,
            http: http.build()?,
            session_file,
            json: cli.json,
            agent_file: cli.agent_file.clone(),
            output_file: cli.output_file.clone(),
            run_id: cli.run_id.clone(),
            idempotency_key: cli.idempotency_key.clone(),
            if_revision: cli.if_revision,
            show_secrets: cli.show_secrets,
        })
    }
    pub(super) fn session(&self) -> Result<SavedSession> {
        let bytes = crate::config::read_private_secret(&self.session_file, 65536)
            .context("No valid private saved session; run `riauth login`")?;
        let session: SavedSession = serde_json::from_str(&bytes)?;
        if session.issuer != self.issuer {
            bail!(
                "Saved session belongs to another issuer; use a separate --session-file or log in here"
            );
        }
        if session.expires_at <= crypto::now() {
            bail!("Session expired; run `riauth login`");
        }
        Ok(session)
    }
    pub(super) fn authentication(&self) -> Result<String> {
        if let Some(path) = &self.agent_file {
            let credential: Value =
                serde_json::from_str(&crate::config::read_private_secret(path, 65536)?)?;
            if credential["issuer"].as_str() != Some(&self.issuer)
                || credential["expires_at"]
                    .as_u64()
                    .is_none_or(|at| at <= crypto::now())
            {
                bail!("Agent credential is expired or belongs to another issuer");
            }
            return Ok(credential["token"]
                .as_str()
                .filter(|v| v.starts_with("ri_agent_"))
                .context("Invalid agent credential")?
                .into());
        }
        Ok(self.session()?.token)
    }
    pub(super) async fn call(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        authenticated: bool,
    ) -> Result<Value> {
        self.call_with_review(method, path, body, authenticated, None)
            .await
    }
    pub(super) async fn call_with_review(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        authenticated: bool,
        reviewed_plan: Option<&str>,
    ) -> Result<Value> {
        let mutation = method != Method::GET
            && (path.starts_with("/api/") || path.starts_with("/scim/"))
            && !path.starts_with("/api/state/");
        let mut req = self.http.request(
            method,
            format!("{}{path}", self.issuer.trim_end_matches('/')),
        );
        if authenticated {
            req = req.bearer_auth(self.authentication()?);
        }
        if let Some(run_id) = &self.run_id {
            req = req.header("x-riauth-run-id", run_id);
        }
        if let Some(plan_id) = reviewed_plan {
            req = req.header("x-riauth-confirm-cloud-removals", plan_id);
        }
        if mutation {
            if let Some(key) = &self.idempotency_key {
                req = req.header("idempotency-key", key);
            }
            if let Some(revision) = self.if_revision {
                req = req.header("if-match", format!("\"{revision}\""));
            }
        }
        if let Some(body) = body {
            req = req.json(&body);
        }
        response_json(req.send().await?).await
    }
    pub(super) async fn call_response(&self, method: Method, path: &str) -> Result<Response> {
        let mut req = self.http.request(
            method,
            format!("{}{path}", self.issuer.trim_end_matches('/')),
        );
        req = req.bearer_auth(self.authentication()?);
        if let Some(run_id) = &self.run_id {
            req = req.header("x-riauth-run-id", run_id);
        }
        Ok(req.send().await?)
    }
    pub(super) async fn oauth_response(
        &self,
        path: &str,
        auth: &ClientAuth,
        mut pairs: Vec<(&str, String)>,
    ) -> Result<Response> {
        pairs.push(("client_id", auth.client_id.clone()));
        if let Some(path) = &auth.assertion_file {
            if std::env::var(&auth.secret_env).is_ok() {
                bail!("Use exactly one client authentication method");
            }
            pairs.push((
                "client_assertion",
                crate::config::read_private_secret(path, 16384)?
                    .trim()
                    .into(),
            ));
            pairs.push(("client_assertion_type", crate::jose::ASSERTION_TYPE.into()));
        }
        if let Ok(secret) = std::env::var(&auth.secret_env) {
            pairs.push(("client_secret", secret));
        }
        let mut request = self
            .http
            .post(format!("{}{path}", self.issuer.trim_end_matches('/')))
            .form(&pairs);
        if let Some(path) = &auth.dpop_proof_file {
            request = request.header(
                "dpop",
                crate::config::read_private_secret(path, 16384)?.trim(),
            );
        }
        Ok(request.send().await?)
    }
    pub(super) async fn oauth(
        &self,
        path: &str,
        auth: &ClientAuth,
        pairs: Vec<(&str, String)>,
    ) -> Result<Value> {
        response_json(self.oauth_response(path, auth, pairs).await?).await
    }
    pub(super) fn secret_destination(&self) -> Result<()> {
        if self.output_file.is_none() && !self.show_secrets {
            bail!(
                "Use --output-file for a private credential destination, or explicitly --show-secrets"
            );
        }
        Ok(())
    }
    pub(super) fn show(&self, value: &Value) -> Result<()> {
        if let Some(path) = &self.output_file {
            write_private(path, &serde_json::to_vec_pretty(value)?, false)?;
            return emit(self.json, &json!({"output_file": path, "written": true}));
        }
        emit(self.json, value)
    }
}

/// Cap bytes while receiving, including responses without Content-Length.
pub(super) async fn bounded_body(mut response: Response, limit: usize) -> Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|size| size > limit as u64)
    {
        bail!("Response body exceeds {} bytes", limit);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if chunk.len() > limit.saturating_sub(bytes.len()) {
            bail!("Response body exceeds {} bytes", limit);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
