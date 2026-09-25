//! SAML 2.0 browser SSO with signed requests, terminal consent and pinned trust.
pub mod logout;
pub(crate) mod wire;
use crate::{
    browser::{BrowserDecision, BrowserReply},
    core::{Core, audit},
    crypto::{self, SigningKey, digest, now},
    error::{Error, Result},
    model::{AuthenticationTransaction, Client, Identity, Session, User},
    response::escape,
    signin::{self, FRESH_SECONDS, TERMINAL_WARN_SECONDS},
    store::Tx,
};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use webauthn_rs::prelude::PublicKeyCredential;
use wire::{ASSERTION, DSIG, METADATA, POST, PROTOCOL, REDIRECT, RSA256};

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NameIdFormat {
    #[default]
    Persistent,
    Transient,
    Email,
    Unspecified,
}
impl NameIdFormat {
    pub fn uri(&self) -> &'static str {
        match self {
            Self::Persistent => "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent",
            Self::Transient => "urn:oasis:names:tc:SAML:2.0:nameid-format:transient",
            Self::Email => "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress",
            Self::Unspecified => "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified",
        }
    }
}
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    pub name: String,
    pub claim: String,
    pub friendly_name: Option<String>,
    #[serde(default)]
    pub required: bool,
}
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub sp_entity_id: String,
    pub acs_urls: Vec<String>,
    #[serde(default)]
    pub acs_indices: std::collections::BTreeMap<u16, String>,
    pub idp_entity_id: Option<String>,
    pub idp_certificate_pem: String,
    pub sp_certificates_pem: Vec<String>,
    pub encryption_certificate_pem: Option<String>,
    #[serde(default)]
    pub name_id_format: NameIdFormat,
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    pub slo_redirect_url: Option<String>,
    pub slo_post_url: Option<String>,
    #[serde(default)]
    pub idp_initiated: bool,
    pub default_relay_state: Option<String>,
    #[serde(default = "assertion_ttl")]
    pub assertion_ttl: u64,
}
fn assertion_ttl() -> u64 {
    120
}
/// HTTP endpoint base from the configured issuer. Protocol identifiers keep the exact issuer.
pub(crate) fn endpoint_base(issuer: &str) -> &str {
    issuer.trim_end_matches('/')
}
pub(crate) fn endpoint_url(value: &str) -> Result<()> {
    let url = crate::config::validate_server_url(value).map_err(|_| {
        Error::bad(
            "SAML endpoints require canonical HTTPS, or HTTP loopback, without query/fragment",
        )
    })?;
    if url.as_str() != value {
        return Err(Error::bad("SAML endpoint URL must be canonical"));
    }
    Ok(())
}
pub(crate) fn entity_id(value: &str) -> Result<()> {
    wire::bounded_text(value, 1024)?;
    url::Url::parse(value).map_err(|_| Error::bad("SAML entity ID must be an absolute URI"))?;
    Ok(())
}
impl Settings {
    pub fn validate(&self, client: &Client) -> Result<()> {
        if client.service
            || !client.scopes.contains("saml")
            || self.acs_urls.is_empty()
            || self.acs_urls.len() > 10
            || self.sp_certificates_pem.is_empty()
            || self.sp_certificates_pem.len() > 4
            || self.attributes.len() > 32
            || !(30..=300).contains(&self.assertion_ttl)
        {
            return Err(Error::bad(
                "Invalid SAML interactive policy client or profile limits",
            ));
        }
        entity_id(&self.sp_entity_id)?;
        if let Some(id) = &self.idp_entity_id {
            entity_id(id)?;
        }
        let mut acs = BTreeSet::new();
        for url in &self.acs_urls {
            endpoint_url(url)?;
            if !acs.insert(url) {
                return Err(Error::bad("Duplicate SAML ACS"));
            }
        }
        if self.acs_indices.len() > 10
            || self
                .acs_indices
                .values()
                .any(|url| !self.acs_urls.contains(url))
        {
            return Err(Error::bad(
                "SAML ACS indices must refer to configured ACS URLs",
            ));
        }
        for url in self.slo_redirect_url.iter().chain(self.slo_post_url.iter()) {
            endpoint_url(url)?;
        }
        for cert in std::iter::once(&self.idp_certificate_pem)
            .chain(&self.sp_certificates_pem)
            .chain(self.encryption_certificate_pem.iter())
        {
            wire::certificate(cert)?;
        }
        if let Some(relay) = &self.default_relay_state {
            wire::bounded_text(relay, 80)?;
        }
        let mut names = BTreeSet::new();
        for attr in &self.attributes {
            wire::bounded_text(&attr.name, 1024)?;
            wire::bounded_text(&attr.claim, 128)?;
            if !names.insert(&attr.name) {
                return Err(Error::bad("Duplicate SAML attribute name"));
            }
            if let Some(friendly) = &attr.friendly_name {
                wire::bounded_text(friendly, 200)?;
            }
            claim_scope(client, &attr.claim)?;
        }
        if !self.scopes(client)?.is_subset(&client.scopes) {
            return Err(Error::bad(
                "SAML attributes require their registered claim scopes",
            ));
        }
        Ok(())
    }
    pub(crate) fn scopes(&self, client: &Client) -> Result<BTreeSet<String>> {
        let mut scopes = BTreeSet::from(["saml".into()]);
        if self.name_id_format == NameIdFormat::Email {
            scopes.insert("email".into());
        }
        for attr in &self.attributes {
            scopes.insert(claim_scope(client, &attr.claim)?.into());
        }
        Ok(scopes)
    }
    fn issuer(&self, core: &Core, client: &Client) -> String {
        self.idp_entity_id.clone().unwrap_or_else(|| {
            format!(
                "{}/saml/{}/metadata",
                endpoint_base(&core.config.issuer),
                client.id
            )
        })
    }
}
fn claim_scope<'a>(client: &'a Client, claim: &str) -> Result<&'a str> {
    match claim {
        "sub" => Ok("saml"),
        "name" | "preferred_username" => Ok("profile"),
        "email" | "email_verified" => Ok("email"),
        "groups" => Ok("groups"),
        other => client
            .settings
            .claim_mappings
            .iter()
            .find(|m| m.claim == other)
            .map(|m| m.scope.as_str())
            .ok_or_else(|| {
                Error::bad("SAML attribute must select an existing scoped identity claim")
            }),
    }
}
fn client(tx: &Tx<'_>, id: &str) -> Result<(Client, Settings)> {
    let client = tx
        .get::<Client>("clients", id)?
        .filter(|c| c.enabled)
        .ok_or_else(Error::forbidden)?;
    let settings = client.settings.saml.clone().ok_or_else(Error::forbidden)?;
    settings.validate(&client)?;
    Ok((client, settings))
}
fn fingerprint(client: &Client) -> Result<String> {
    Ok(digest(
        &serde_json::to_string(client).map_err(Error::internal)?,
    ))
}
pub fn validate_key(tx: &Tx<'_>, client: &Client) -> Result<()> {
    if let Some(settings) = &client.settings.saml {
        let key = crate::keyring::for_client(tx, client)?.active;
        if key.remote.is_some() || key.algorithm != "RS256" {
            return Err(Error::bad(
                "SAML XML signing currently requires a local RS256 signing domain",
            ));
        }
        let private =
            risaml::crypto::keys::load_private_key(&key.pem, None).map_err(Error::internal)?;
        let public = risaml::crypto::keys::load_certificate(&settings.idp_certificate_pem)
            .map_err(|_| Error::bad("Invalid SAML IdP certificate"))?;
        if private.to_spki_der().is_none() || private.to_spki_der() != public.to_spki_der() {
            return Err(Error::bad(
                "SAML IdP certificate must match the selected signing domain",
            ));
        }
    }
    Ok(())
}
pub(crate) fn sign(xml: &str, key: &SigningKey, cert: &str, whole: bool) -> Result<String> {
    let key = risaml::crypto::keys::load_private_key(&key.pem, None).map_err(Error::internal)?;
    let metadata =
        xml.starts_with("<md:EntityDescriptor")
            .then(|| risaml::entity::SignatureConfig {
                prefix: "ds".into(),
                reference: Some("/*[local-name(.)='EntityDescriptor']".into()),
                action: risaml::entity::SignatureAction::Prepend,
            });
    risaml::crypto::construct_saml_signature(xml, whole, &key, cert, RSA256, &[], metadata.as_ref())
        .map_err(Error::internal)
}
#[derive(Clone, Serialize, Deserialize)]
struct Decision {
    identity: Identity,
    approve: bool,
    remember: bool,
}
#[derive(Clone, Serialize, Deserialize)]
struct Pending {
    id: String,
    code: String,
    browser_hash: String,
    client_id: String,
    client_fingerprint: String,
    request: wire::Authn,
    expires_at: u64,
    decision: Option<Decision>,
    /// Digest key of the proof a browser sign-in in this interaction bound to the request.
    #[serde(default)]
    authentication: Option<String>,
    /// Declined in the browser; resume answers RequestDenied without an identity.
    #[serde(default)]
    cancelled: bool,
    #[serde(default)]
    requested_from: Option<Value>,
    /// The session that approved on the interaction page. Only a browser signed in to it
    /// collects the response; a terminal approval leaves it unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    approved_by: Option<String>,
}
impl Pending {
    fn request_hash(&self) -> String {
        digest(&format!("saml\0{}\0{}", self.id, self.client_fingerprint))
    }
    fn decided(&self) -> bool {
        self.decision.is_some() || self.cancelled
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Consent {
    fingerprint: String,
    expires_at: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct RpSession {
    #[serde(default)]
    index: Option<String>,
    #[serde(default)]
    issuer: String,
    #[serde(default)]
    fingerprint: String,
    client_id: String,
    name_id: String,
    format: NameIdFormat,
    identity: Identity,
    expires_at: u64,
}
pub enum Reply {
    LogoutPage(Value),
    Waiting(BrowserReply),
    Post {
        target: String,
        fields: Vec<(String, String)>,
        cookies: Vec<String>,
    },
    Redirect(String),
}
fn resume_path(core: &Core, id: &str) -> String {
    format!("{}saml/resume/{id}", core.cookie_path())
}
fn waiting(core: &Core, p: &Pending, cookies: Vec<String>) -> Reply {
    let resume = resume_path(core, &p.id);
    Reply::Waiting(BrowserReply {
        form_post: false,
        body: json!({"protocol":"saml","status":"authorization_pending","user_code":p.code,"client_id":p.client_id,"expires_at":p.expires_at,"resume_uri":resume,"instruction":format!("Run riauth request approve {} in your terminal. This browser will return to the application automatically.",p.code)}),
        location: None,
        refresh: Some(format!("2; url={resume}")),
        cookies,
    })
}
fn post(target: String, xml: String, relay: Option<String>, cookies: Vec<String>) -> Reply {
    let mut fields = vec![("SAMLResponse".into(), STANDARD.encode(xml))];
    if let Some(state) = relay {
        fields.push(("RelayState".into(), state));
    }
    Reply::Post {
        target,
        fields,
        cookies,
    }
}
fn pending(tx: &Tx<'_>, code: &str) -> Result<Pending> {
    let hash = digest(&crypto::normalize_code(code)?);
    let id = tx
        .get::<String>("saml_codes", &hash)?
        .ok_or_else(|| Error::missing("SAML request not found"))?;
    tx.get::<Pending>("saml_requests", &id)?
        .filter(|p| p.expires_at > now())
        .ok_or_else(|| Error::missing("SAML request expired"))
}
fn remove(tx: &Tx<'_>, p: &Pending) -> Result<()> {
    if let Some(proof) = &p.authentication {
        tx.delete("authentication", proof)?;
    }
    tx.delete("saml_codes", &digest(&crypto::normalize_code(&p.code)?))?;
    tx.delete("saml_requests", &p.id)
}
/// The browser's own request: its binding cookie proves which browser started it.
fn interaction(tx: &Tx<'_>, id: &str, binding: Option<&str>) -> Result<Pending> {
    let p = tx
        .get::<Pending>("saml_requests", id)?
        .filter(|p| p.expires_at > now())
        .ok_or_else(|| {
            Error::new(
                StatusCode::NOT_FOUND,
                "interaction_expired",
                "This sign-in request has expired or was already completed",
            )
        })?;
    if !binding.is_some_and(|b| crypto::constant_eq(&digest(b), &p.browser_hash)) {
        return Err(Error::unauthorized());
    }
    Ok(p)
}
fn undecided(p: Pending) -> Result<Pending> {
    if p.decided() {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "request_decided",
            "This request was already decided",
        ));
    }
    Ok(p)
}
/// A valid client whose configuration still matches the one the request started with.
fn current_client(tx: &Tx<'_>, p: &Pending) -> Result<(Client, Settings)> {
    let (client, settings) = client(tx, &p.client_id)?;
    if fingerprint(&client)? != p.client_fingerprint {
        return Err(Error::conflict("SAML client changed"));
    }
    Ok((client, settings))
}
fn consent_key(identity: &Identity, client: &Client) -> String {
    digest(&format!("{}\0{}", identity.user_id, client.id))
}
/// Implicit consent, or a remembered consent for this exact client configuration.
fn consented(tx: &Tx<'_>, client: &Client, identity: &Identity) -> Result<bool> {
    if client.settings.implicit_consent {
        return Ok(true);
    }
    let fingerprint = fingerprint(client)?;
    Ok(tx
        .get::<Consent>("saml_consents", &consent_key(identity, client))?
        .is_some_and(|c| c.expires_at > now() && c.fingerprint == fingerprint))
}
/// F13: terminal approval needs a recent sign-in. `auth_time` 0 (upstream OAuth) is exempt.
pub(crate) fn stale(identity: &Identity, limit: u64) -> bool {
    identity.auth_time != 0 && now().saturating_sub(identity.auth_time) > limit
}
/// F8: a staged login that falls short of this client is never attached.
pub(crate) fn insufficient_error(
    client: &Client,
    user: Option<&User>,
    identity: &Identity,
) -> Error {
    let (code, text) = if identity.mfa {
        (
            "unmet_authentication_requirements",
            "does not accept this sign-in method.",
        )
    } else if user.is_some_and(|u| u.totp_secret.is_some() || u.has_passkeys) {
        (
            "unmet_authentication_requirements",
            "requires a passkey or an authenticator code.",
        )
    } else {
        (
            "mfa_setup_required",
            "requires a passkey or an authenticator code. Add a passkey in your applications portal first.",
        )
    };
    Error::new(
        StatusCode::FORBIDDEN,
        code,
        format!("{} {text}", client.name),
    )
}
fn unavailable(mut state: Value, error: &str, message: Option<String>) -> Result<Value> {
    state["status"] = json!("unavailable");
    state["error"] = json!(error);
    state["message"] = json!(message);
    Ok(state)
}
fn authn_context(core: &Core, identity: &Identity) -> &'static str {
    if identity.mfa {
        crate::assurance::MFA
    } else if crate::assurance::actual(identity) == crate::assurance::FEDERATED {
        crate::assurance::FEDERATED
    } else if core.config.issuer.starts_with("https://") {
        wire::PASSWORD_TLS
    } else {
        wire::PASSWORD
    }
}
/// The authentication falls short of the client's assurance or the requested context.
fn insufficient(core: &Core, client: &Client, request: &wire::Authn, identity: &Identity) -> bool {
    crate::assurance::needs_step_up(client, &Default::default(), identity)
        || !matches_context(core, request, identity)
}
fn matches_context(core: &Core, request: &wire::Authn, identity: &Identity) -> bool {
    let actual = authn_context(core, identity);
    request.contexts.is_empty()
        || request.contexts.iter().any(|c| {
            c == actual
                || request.minimum
                    && ((c == wire::PASSWORD
                        && [wire::PASSWORD_TLS, crate::assurance::MFA].contains(&actual))
                        || (c == wire::PASSWORD_TLS
                            && actual == crate::assurance::MFA
                            && core.config.issuer.starts_with("https://")))
        })
}
fn name_id(
    settings: &Settings,
    user: &User,
    client: &Client,
    identity: &Identity,
) -> Result<String> {
    match settings.name_id_format {
        NameIdFormat::Persistent | NameIdFormat::Unspecified => {
            Ok(crate::claims::subject(user, client))
        }
        NameIdFormat::Transient => Ok(digest(&format!(
            "saml-transient\0{}\0{}",
            client.id, identity.session_id
        ))),
        NameIdFormat::Email => user
            .email
            .as_ref()
            .filter(|_| user.email_verified)
            .cloned()
            .ok_or_else(|| Error::bad("SAML email NameID requires a verified email")),
    }
}
impl Core {
    pub fn saml_metadata(&self, cid: &str) -> Result<String> {
        self.store.read(|tx|{let (client,settings)=client(tx,cid)?;validate_key(tx,&client)?;let endpoint=format!("{}/saml/{cid}/sso",endpoint_base(&self.config.issuer));let issuer=settings.issuer(self,&client);let cert=STANDARD.encode(wire::certificate(&settings.idp_certificate_pem)?);
        let key=format!("<md:KeyDescriptor use=\"signing\"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{cert}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>");
        let sso=format!("<md:SingleSignOnService Binding=\"{REDIRECT}\" Location=\"{}\"/><md:SingleSignOnService Binding=\"{POST}\" Location=\"{}\"/>",escape(&endpoint),escape(&endpoint));
        let slo=if settings.slo_redirect_url.is_some()||settings.slo_post_url.is_some(){format!("<md:SingleLogoutService Binding=\"{REDIRECT}\" Location=\"{}\"/><md:SingleLogoutService Binding=\"{POST}\" Location=\"{}\"/>",escape(&endpoint),escape(&endpoint))}else{String::new()};
        let xml=format!("<md:EntityDescriptor xmlns:md=\"{METADATA}\" xmlns:ds=\"{DSIG}\" ID=\"_metadata_{}\" entityID=\"{}\"><md:IDPSSODescriptor protocolSupportEnumeration=\"{PROTOCOL}\" WantAuthnRequestsSigned=\"true\">{key}{slo}<md:NameIDFormat>{}</md:NameIDFormat>{sso}</md:IDPSSODescriptor></md:EntityDescriptor>",client.id,escape(&issuer),settings.name_id_format.uri());
        sign(&xml,&crate::keyring::for_client(tx,&client)?.active,&settings.idp_certificate_pem,true)
    })
    }
    pub fn saml_start(
        &self,
        cid: &str,
        raw: &str,
        is_post: bool,
        cookie: Option<&str>,
    ) -> Result<Reply> {
        self.store.write(|tx| {
            let (client, settings) = client(tx, cid)?;
            validate_key(tx, &client)?;
            if wire::is_response(raw) {
                return self.saml_logout_response_in(
                    tx,
                    logout::Peer::Client {
                        id: cid.into(),
                        fingerprint: fingerprint(&client)?,
                    },
                    raw,
                    is_post,
                );
            }
            let message = wire::receive(
                &settings,
                &format!("{}/saml/{cid}/sso", endpoint_base(&self.config.issuer)),
                &settings.issuer(self, &client),
                raw,
                is_post,
            )?;
            let request_id = match &message {
                wire::Message::Authn(r) => r.id.as_ref().unwrap(),
                wire::Message::Logout(r) => &r.id,
            };
            let replay = digest(&format!("{cid}\0{request_id}"));
            if tx
                .get::<u64>("saml_replays", &replay)?
                .is_some_and(|expiry| expiry > now())
            {
                return Err(Error::conflict("SAML request replay"));
            }
            if tx.list::<u64>("saml_replays")?.len() > 20000 {
                return Err(Error::conflict("Too many SAML requests"));
            }
            tx.put("saml_replays", &replay, &(now() + 630))?;
            match message {
                wire::Message::Logout(logout) => {
                    self.saml_logout_in(tx, &client, &settings, logout, is_post)
                }
                wire::Message::Authn(request) => {
                    self.saml_pending(tx, &client, &settings, request, cookie)
                }
            }
        })
    }
    pub fn saml_initiate(&self, cid: &str, cookie: Option<&str>) -> Result<Reply> {
        self.store.write(|tx| {
            let (client, settings) = client(tx, cid)?;
            if !settings.idp_initiated {
                return Err(Error::forbidden());
            }
            validate_key(tx, &client)?;
            let request = wire::Authn {
                id: None,
                acs: settings.acs_urls[0].clone(),
                relay_state: settings.default_relay_state.clone(),
                force: false,
                passive: false,
                contexts: vec![],
                minimum: false,
                name_id_format: settings.name_id_format.clone(),
                requested_subject: None,
                allow_create: true,
            };
            self.saml_pending(tx, &client, &settings, request, cookie)
        })
    }
    fn saml_pending(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        settings: &Settings,
        request: wire::Authn,
        cookie: Option<&str>,
    ) -> Result<Reply> {
        if let Some(session) = self.browser_session(tx, cookie)?
            && !request.force
            && !insufficient(self, client, &request, &session.identity)
            && consented(tx, client, &session.identity)?
        {
            return self.saml_issue(
                tx,
                client,
                settings,
                &request,
                Some(&session.identity),
                "Success",
                vec![],
            );
        }
        if request.passive {
            return self.saml_issue(tx, client, settings, &request, None, "NoPassive", vec![]);
        }
        if tx.list::<Pending>("saml_requests")?.len() > 10000 {
            return Err(Error::conflict("Too many pending SAML requests"));
        }
        let id = crypto::id();
        let binding = crypto::random_token("ri_saml_browser_");
        let code = loop {
            let code = crypto::user_code();
            let hash = digest(&crypto::normalize_code(&code)?);
            if tx.get::<String>("saml_codes", &hash)?.is_none()
                && tx.get::<String>("authorization_codes", &hash)?.is_none()
            {
                break code;
            }
        };
        let p = Pending {
            id: id.clone(),
            code,
            client_id: client.id.clone(),
            client_fingerprint: fingerprint(client)?,
            browser_hash: digest(&binding),
            request,
            expires_at: now() + 300,
            decision: None,
            authentication: None,
            cancelled: false,
            requested_from: crate::context::requester(),
            approved_by: None,
        };
        tx.put(
            "saml_codes",
            &digest(&crypto::normalize_code(&p.code)?),
            &id,
        )?;
        tx.put("saml_requests", &id, &p)?;
        Ok(waiting(
            self,
            &p,
            vec![self.binding_cookie("saml", &id, &binding, &resume_path(self, &id), 300)],
        ))
    }
    pub(crate) fn is_saml_code(&self, code: &str) -> Result<bool> {
        self.store.read(|tx| {
            Ok(tx
                .get::<String>("saml_codes", &digest(&crypto::normalize_code(code)?))?
                .is_some())
        })
    }
    pub fn saml_details(&self, token: &str, code: &str) -> Result<Value> {
        self.store.write(|tx|{
        let (user,session)=self.session(tx,token)?;let p=pending(tx,code)?;if p.decided(){return Err(Error::conflict("SAML request already decided"));}let(client,settings)=current_client(tx,&p)?;
        let transaction=crypto::random_token("ri_auth_");tx.put("authentication",&digest(&transaction),&AuthenticationTransaction{request_hash:p.request_hash(),user_id:Some(user.id),authenticated_session:None,expires_at:p.expires_at,source_stage:None})?;
        // Ask the CLI to sign in again before an approval could fail the F13 freshness rule.
        let fresh=p.request.force||insufficient(self,&client,&p.request,&session.identity)||stale(&session.identity,TERMINAL_WARN_SECONDS);
        Ok(json!({"protocol":"saml","user_code":p.code,"client_id":client.id,"application":client.name,"sp_entity_id":settings.sp_entity_id,"redirect_uri":p.request.acs,"scopes":settings.scopes(&client)?,"attributes":settings.attributes,"name_id_format":settings.name_id_format.uri(),"username":user.username,"require_mfa":client.require_mfa,"requested_authn_context":p.request.contexts,"reauthentication_required":fresh,"transaction_id":transaction,"requested_from":p.requested_from,"delivery":"original_browser"}))
    })
    }
    pub fn saml_decide(&self, token: &str, input: BrowserDecision) -> Result<Value> {
        self.store.write(|tx| {
            let (_, session) = self.session(tx, token)?;
            let mut p = pending(tx, &input.code)?;
            if p.decided() {
                return Err(Error::conflict("SAML request already decided"));
            }
            if input.approve && stale(&session.identity, FRESH_SECONDS) {
                return Err(Error::oauth(
                    "login_required",
                    "Sign in again in your terminal before approving",
                ));
            }
            let proof = input.transaction_id.as_deref().map(digest);
            self.saml_decide_session(
                tx,
                &session,
                &mut p,
                input.approve,
                input.remember,
                proof.as_deref(),
            )?;
            Ok(json!({"approved":input.approve,"delivery":"original_browser"}))
        })
    }
    /// Records the decision of `session`. An approval that must re-authenticate needs the
    /// proof stored under `proof_key` for this request and session; it is used up.
    fn saml_decide_session(
        &self,
        tx: &Tx<'_>,
        session: &Session,
        p: &mut Pending,
        approve: bool,
        remember: bool,
        proof_key: Option<&str>,
    ) -> Result<()> {
        let (client, settings) = current_client(tx, p)?;
        if approve {
            if p.request.force || insufficient(self, &client, &p.request, &session.identity) {
                let required = || {
                    Error::oauth(
                        "login_required",
                        "Complete request-bound SAML authentication",
                    )
                };
                let key = proof_key.ok_or_else(required)?;
                let proof = tx
                    .get::<AuthenticationTransaction>("authentication", key)?
                    .filter(|a| {
                        a.expires_at > now()
                            && a.authenticated_session.as_deref() == Some(&session.id)
                            && a.request_hash == p.request_hash()
                    })
                    .ok_or_else(required)?;
                if proof
                    .user_id
                    .as_ref()
                    .is_some_and(|u| *u != session.identity.user_id)
                {
                    return Err(Error::forbidden());
                }
                tx.delete("authentication", key)?;
            }
            self.saml_identity(tx, &client, &settings, &p.request, &session.identity)?;
        }
        p.decision = Some(Decision {
            identity: session.identity.clone(),
            approve,
            remember,
        });
        // A decided request needs no browser proof any more.
        if let Some(proof) = p.authentication.take() {
            tx.delete("authentication", &proof)?;
        }
        tx.put("saml_requests", &p.id, &*p)?;
        audit(
            tx,
            &session.identity.user_id,
            if approve { "saml.approve" } else { "saml.deny" },
            &client.id,
        )
    }
    /// The interaction page's view of this request. Read-only.
    pub fn saml_state(&self, id: &str, binding: Option<&str>, sso: Option<&str>) -> Result<Value> {
        self.store.read(|tx| {
            let p = interaction(tx, id, binding)?;
            let session = self.browser_session(tx, sso)?;
            self.saml_state_in(tx, &p, session)
        })
    }
    /// Browser password sign-in for this request. The body is the interaction state.
    pub fn saml_password(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        username: String,
        password: String,
        otp: Option<String>,
    ) -> Result<BrowserReply> {
        let (_, pin) = self.saml_open(id, binding, sso)?;
        let staged = self.browser_password_login(username, password, otp, pin.as_deref())?;
        self.saml_attach(id, binding, sso, &staged, pin.as_deref())
    }
    /// A pinned account signs in with its own passkeys; otherwise the browser offers any.
    pub fn saml_passkey_start(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<Value> {
        let (p, pin) = self.saml_open(id, binding, sso)?;
        self.browser_passkey_start(pin.as_deref(), &format!("saml:{id}"), &p.browser_hash)
    }
    pub fn saml_passkey_finish(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        ceremony: &str,
        response: PublicKeyCredential,
    ) -> Result<BrowserReply> {
        let (_, pin) = self.saml_open(id, binding, sso)?;
        let staged = self.browser_passkey_finish(
            ceremony,
            response,
            &format!("saml:{id}"),
            binding,
            pin.as_deref(),
        )?;
        self.saml_attach(id, binding, sso, &staged, pin.as_deref())
    }
    /// Approves as this browser's session, or cancels without one. The body is the state.
    pub fn saml_browser_decide(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        approve: bool,
        remember: bool,
        session_ref: Option<String>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let mut p = undecided(interaction(tx, id, binding)?)?;
            let session = self.browser_session(tx, sso)?;
            if approve {
                let session = session.as_ref().ok_or_else(Error::unauthorized)?;
                if !session_ref
                    .as_deref()
                    .is_some_and(|r| crypto::constant_eq(r, &signin::session_ref(id, &session.id)))
                {
                    return Err(Error::new(
                        StatusCode::CONFLICT,
                        "account_changed",
                        "The signed-in account changed. Review the request again.",
                    ));
                }
                let implicit = tx
                    .get::<Client>("clients", &p.client_id)?
                    .is_some_and(|c| c.settings.implicit_consent);
                let proof = p.authentication.clone();
                p.approved_by = Some(session.id.clone());
                self.saml_decide_session(
                    tx,
                    session,
                    &mut p,
                    true,
                    remember && !implicit,
                    proof.as_deref(),
                )?;
            } else {
                p.cancelled = true;
                if let Some(proof) = p.authentication.take() {
                    tx.delete("authentication", &proof)?;
                }
                tx.put("saml_requests", &p.id, &p)?;
                let actor = session.as_ref().map(|s| s.identity.user_id.as_str());
                audit(tx, actor.unwrap_or("anonymous"), "saml.deny", &p.client_id)?;
            }
            self.saml_state_in(tx, &p, session)
        })
    }
    /// The undecided request and the account it is pinned to: always the browser
    /// session's user.
    fn saml_open(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<(Pending, Option<String>)> {
        self.store.read(|tx| {
            let p = undecided(interaction(tx, id, binding)?)?;
            let pin = self.browser_session(tx, sso)?.map(|s| s.identity.user_id);
            Ok((p, pin))
        })
    }
    /// Phase 2 of a browser sign-in: sufficiency (F8), attach (F3) and the request-bound
    /// proof (F7) in one write; then auto-continue (F9) in its own write.
    fn saml_attach(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        staged: &str,
        pin: Option<&str>,
    ) -> Result<BrowserReply> {
        let attached = self.store.write(|tx| {
            let checked = interaction(tx, id, binding)
                .and_then(undecided)
                .and_then(|p| Ok((current_client(tx, &p)?.0, p)));
            let (client, mut p) = match checked {
                Ok(checked) => checked,
                Err(error) if error.status.is_server_error() => return Err(error),
                Err(error) => {
                    signin::discard_staged(tx, staged)?;
                    return Ok(Err(error));
                }
            };
            let login = self.staged_login(tx, staged)?;
            if insufficient(self, &client, &p.request, &login.identity) {
                signin::discard_staged(tx, staged)?;
                let user = tx.get::<User>("users", &login.identity.user_id)?;
                return Ok(Err(insufficient_error(
                    &client,
                    user.as_ref(),
                    &login.identity,
                )));
            }
            let attached = match self.attach_browser_login(tx, staged, sso, pin)? {
                Ok(attached) => attached,
                Err(error) => return Ok(Err(error)),
            };
            p.authentication = Some(signin::bind_proof(
                tx,
                p.authentication.as_deref(),
                p.request_hash(),
                &attached.session.identity.user_id,
                &attached.session.id,
                p.expires_at,
            )?);
            tx.put("saml_requests", &p.id, &p)?;
            Ok(Ok(attached))
        })??;
        let holder = &attached.session.id;
        if let Err(error) = self
            .store
            .write(|tx| self.saml_auto_continue(tx, id, holder))
        {
            tracing::warn!(%error, "SAML auto-continue failed");
        }
        let body = self.store.read(|tx| {
            let p = interaction(tx, id, binding)?;
            let session = tx.get::<Session>("sessions", holder)?.filter(|s| {
                !s.revoked && s.expires_at > now() && self.identity_user(tx, &s.identity).is_ok()
            });
            self.saml_state_in(tx, &p, session)
        })?;
        Ok(BrowserReply {
            form_post: false,
            body,
            location: None,
            refresh: None,
            cookies: attached.cookies,
        })
    }
    /// F9: decides at once when consent is implicit or remembered, the proof holds where
    /// one is needed and the policy dry run passes. Otherwise the state stands as it is.
    fn saml_auto_continue(&self, tx: &Tx<'_>, id: &str, holder: &str) -> Result<bool> {
        let Some(mut p) = tx
            .get::<Pending>("saml_requests", id)?
            .filter(|p| p.expires_at > now() && !p.decided())
        else {
            return Ok(false);
        };
        let Some(session) = tx
            .get::<Session>("sessions", holder)?
            .filter(|s| !s.revoked && s.expires_at > now())
        else {
            return Ok(false);
        };
        let (client, settings) = current_client(tx, &p)?;
        let proven = !(p.request.force
            || insufficient(self, &client, &p.request, &session.identity))
            || signin::proof_valid(tx, p.authentication.as_deref(), &p.request_hash(), holder)?;
        if !proven || !consented(tx, &client, &session.identity)? {
            return Ok(false);
        }
        match self.saml_identity(tx, &client, &settings, &p.request, &session.identity) {
            Err(error) if error.status.is_server_error() => return Err(error),
            Err(_) => return Ok(false),
            Ok(_) => {}
        }
        let proof = p.authentication.clone();
        p.approved_by = Some(holder.to_owned());
        self.saml_decide_session(tx, &session, &mut p, true, false, proof.as_deref())?;
        Ok(true)
    }
    /// The §4.5 state for `session`, the browser's live session if any.
    fn saml_state_in(&self, tx: &Tx<'_>, p: &Pending, session: Option<Session>) -> Result<Value> {
        let name = tx
            .get::<Client>("clients", &p.client_id)?
            .map_or_else(|| p.client_id.clone(), |c| c.name);
        let host = url::Url::parse(&p.request.acs)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned));
        let account = match &session {
            Some(s) => Some(signin::account_json(
                &self.identity_user(tx, &s.identity)?,
                s,
            )),
            None => None,
        };
        let mut state = json!({
            "kind": "saml",
            "status": "complete",
            "reason": null,
            "expires_at": p.expires_at,
            "application": {"client_id": p.client_id, "name": name, "host": host},
            "account": account,
            "session_ref": session.as_ref().map(|s| signin::session_ref(&p.id, &s.id)),
            "pinned": session.is_some(),
            "requirements": null,
            "consent": null,
            "logout": null,
            "terminal": null,
            "continue": null,
            "error": null,
            "message": null
        });
        if p.decided() {
            state["continue"] = json!(resume_path(self, &p.id));
            return Ok(state);
        }
        state["terminal"] = json!({"user_code": p.code, "issuer": self.config.issuer});
        let (client, settings) = match current_client(tx, p) {
            Ok(current) => current,
            Err(error) if error.status.is_server_error() => return Err(error),
            Err(_) => return unavailable(state, "invalid_request", None),
        };
        let probe = |mfa: bool| Identity {
            amr: if mfa {
                vec!["webauthn".into(), "mfa".into()]
            } else {
                vec![]
            },
            source: None,
            user_id: String::new(),
            epoch: 0,
            mfa,
            auth_time: now(),
            session_id: String::new(),
        };
        let mfa = insufficient(self, &client, &p.request, &probe(false));
        let browser = !mfa || !insufficient(self, &client, &p.request, &probe(true));
        let required = match &session {
            Some(s) => !consented(tx, &client, &s.identity)?,
            None => !client.settings.implicit_consent,
        };
        state["requirements"] = json!({"mfa": mfa, "browser": browser});
        state["consent"] = json!({"required": required, "scopes": null, "attributes": settings.attributes, "resource": null, "remember_default": true});
        if !browser {
            return unavailable(state, "step_up_unavailable", None);
        }
        let Some(session) = session else {
            state["status"] = json!("authenticate");
            state["reason"] = json!("sign_in");
            return Ok(state);
        };
        // ForceAuthn, step-up and context mismatches need a sign-in bound to this request.
        if (p.request.force || insufficient(self, &client, &p.request, &session.identity))
            && !signin::proof_valid(
                tx,
                p.authentication.as_deref(),
                &p.request_hash(),
                &session.id,
            )?
        {
            state["status"] = json!("authenticate");
            state["reason"] = json!(if p.request.force {
                "force_authn"
            } else {
                "step_up"
            });
            return Ok(state);
        }
        if let Err(error) =
            self.saml_identity(tx, &client, &settings, &p.request, &session.identity)
        {
            if error.status.is_server_error() {
                return Err(error);
            }
            return unavailable(state, "access_denied", Some(error.message));
        }
        state["status"] = json!("consent");
        Ok(state)
    }
    pub fn saml_resume(&self, id: &str, binding: Option<&str>) -> Result<Reply> {
        self.saml_resume_with(id, binding, None)
    }
    /// Delivers the decided request once. An approval points this browser at the deciding
    /// session (F19), but one given on the interaction page goes only to a browser that still
    /// holds that session; a browser cancel answers RequestDenied.
    pub fn saml_resume_with(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<Reply> {
        self.store.write(|tx| {
            let p = tx
                .get::<Pending>("saml_requests", id)?
                .filter(|p| p.expires_at > now())
                .ok_or_else(|| Error::missing("SAML request expired or consumed"))?;
            if !binding.is_some_and(|b| crypto::constant_eq(&digest(b), &p.browser_hash)) {
                return Err(Error::unauthorized());
            }
            if !p.decided() {
                return Ok(waiting(self, &p, vec![]));
            }
            if let Some(sid) = &p.approved_by
                && self.browser_session(tx, sso)?.is_none_or(|s| s.id != *sid)
            {
                return Err(Error::unauthorized());
            }
            let (client, settings) = current_client(tx, &p)?;
            validate_key(tx, &client)?;
            let mut cookies = vec![self.binding_cookie("saml", id, "", &resume_path(self, id), 0)];
            let approved = p.decision.as_ref().filter(|d| d.approve);
            if let Some(decision) = approved {
                self.saml_identity(tx, &client, &settings, &p.request, &decision.identity)?;
                cookies.extend(self.point_browser(
                    tx,
                    sso,
                    &decision.identity.session_id,
                    &decision.identity.user_id,
                )?);
                if decision.remember {
                    tx.put(
                        "saml_consents",
                        &consent_key(&decision.identity, &client),
                        &Consent {
                            fingerprint: p.client_fingerprint.clone(),
                            expires_at: now() + 2_592_000,
                        },
                    )?;
                }
            }
            let reply = self.saml_issue(
                tx,
                &client,
                &settings,
                &p.request,
                approved.map(|d| &d.identity),
                if approved.is_some() {
                    "Success"
                } else {
                    "RequestDenied"
                },
                cookies,
            )?;
            remove(tx, &p)?;
            Ok(reply)
        })
    }
    fn saml_identity(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        settings: &Settings,
        request: &wire::Authn,
        identity: &Identity,
    ) -> Result<User> {
        let user = self.authorize_identity(tx, client, identity)?;
        if tx
            .get::<Session>("sessions", &identity.session_id)?
            .is_none_or(|s| s.expires_at <= now())
            || identity.auth_time == 0
            || !matches_context(self, request, identity)
            || crate::assurance::needs_step_up(client, &Default::default(), identity)
        {
            return Err(Error::forbidden());
        }
        crate::claims::enforce(tx, client, &user, identity, &settings.scopes(client)?)?;
        if let Some(subject) = &request.requested_subject
            && *subject != name_id(settings, &user, client, identity)?
        {
            return Err(Error::forbidden());
        }
        if !request.allow_create
            && matches!(
                settings.name_id_format,
                NameIdFormat::Persistent | NameIdFormat::Transient
            )
        {
            let subject = name_id(settings, &user, client, identity)?;
            let key = digest(&format!("{}\0{}", client.id, user.id));
            if !user.subjects.contains_key(&client.id)
                && tx.get::<String>("saml_subjects", &key)?.as_ref() != Some(&subject)
            {
                return Err(Error::bad(
                    "SAML NameIDPolicy forbids creating this SP association",
                ));
            }
        }
        Ok(user)
    }
    #[allow(clippy::too_many_arguments)] // Explicit trusted issuance context; never supplied directly by callers.
    fn saml_issue(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        settings: &Settings,
        request: &wire::Authn,
        identity: Option<&Identity>,
        status: &str,
        cookies: Vec<String>,
    ) -> Result<Reply> {
        let issuer = settings.issuer(self, client);
        let key = crate::keyring::for_client(tx, client)?.active;
        let assertion = if let Some(identity) = identity {
            let user = self.saml_identity(tx, client, settings, request, identity)?;
            let subject = name_id(settings, &user, client, identity)?;
            tx.put(
                "saml_subjects",
                &digest(&format!("{}\0{}", client.id, user.id)),
                &subject,
            )?;
            let session = tx
                .get::<Session>("sessions", &identity.session_id)?
                .ok_or_else(Error::unauthorized)?;
            let expiry = (now() + settings.assertion_ttl).min(session.expires_at);
            let index = crypto::random_token("ri_saml_");
            tx.put(
                "saml_sessions",
                &digest(&index),
                &RpSession {
                    index: Some(index.clone()),
                    issuer: issuer.clone(),
                    fingerprint: fingerprint(client)?,
                    client_id: client.id.clone(),
                    name_id: subject.clone(),
                    format: settings.name_id_format.clone(),
                    identity: identity.clone(),
                    expires_at: session.expires_at,
                },
            )?;
            let id = format!("_{}", crypto::id());
            let instant = wire::timestamp(now())?;
            let expires = wire::timestamp(expiry)?;
            let before = wire::timestamp(now().saturating_sub(30))?;
            let auth_time = wire::timestamp(identity.auth_time)?;
            let session_expiry = wire::timestamp(session.expires_at)?;
            let correlation = request
                .id
                .as_ref()
                .map(|id| format!(" InResponseTo=\"{}\"", escape(id)))
                .unwrap_or_default();
            let claims =
                crate::claims::mapped_claims(tx, &user, client, &settings.scopes(client)?)?;
            let mut attributes = String::new();
            for attr in &settings.attributes {
                let value = &claims[&attr.claim];
                if value.is_null() {
                    if attr.required {
                        return Err(Error::bad("Required SAML attribute is unavailable"));
                    }
                    continue;
                }
                let values = if let Some(values) = value.as_array() {
                    values.clone()
                } else {
                    vec![value.clone()]
                };
                if values.len() > 256 {
                    return Err(Error::bad("SAML attribute exceeds 256 values"));
                }
                let mut contents = String::new();
                for v in values {
                    let (kind, value) = match v {
                        Value::String(s) => ("string", s),
                        Value::Bool(b) => ("boolean", b.to_string()),
                        Value::Number(n) if n.is_i64() || n.is_u64() => ("integer", n.to_string()),
                        _ => {
                            return Err(Error::bad(
                                "SAML attributes require scalar text, boolean or integer values",
                            ));
                        }
                    };
                    wire::bounded_text(&value, 4096)?;
                    contents.push_str(&format!(
                        "<saml:AttributeValue xsi:type=\"xs:{kind}\">{}</saml:AttributeValue>",
                        escape(&value)
                    ));
                }
                if contents.is_empty() {
                    if attr.required {
                        return Err(Error::bad("Required SAML attribute is empty"));
                    }
                    continue;
                }
                let friendly = attr
                    .friendly_name
                    .as_ref()
                    .map(|s| format!(" FriendlyName=\"{}\"", escape(s)))
                    .unwrap_or_default();
                attributes.push_str(&format!("<saml:Attribute Name=\"{}\" NameFormat=\"urn:oasis:names:tc:SAML:2.0:attrname-format:unspecified\"{friendly}>{contents}</saml:Attribute>",escape(&attr.name)));
            }
            if !attributes.is_empty() {
                attributes =
                    format!("<saml:AttributeStatement>{attributes}</saml:AttributeStatement>");
            }
            let xml = format!(
                "<saml:Assertion xmlns:saml=\"{ASSERTION}\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ID=\"{id}\" Version=\"2.0\" IssueInstant=\"{instant}\"><saml:Issuer>{}</saml:Issuer><saml:Subject><saml:NameID Format=\"{}\" NameQualifier=\"{}\" SPNameQualifier=\"{}\">{}</saml:NameID><saml:SubjectConfirmation Method=\"urn:oasis:names:tc:SAML:2.0:cm:bearer\"><saml:SubjectConfirmationData NotOnOrAfter=\"{expires}\" Recipient=\"{}\"{correlation}/></saml:SubjectConfirmation></saml:Subject><saml:Conditions NotBefore=\"{before}\" NotOnOrAfter=\"{expires}\"><saml:AudienceRestriction><saml:Audience>{}</saml:Audience></saml:AudienceRestriction></saml:Conditions><saml:AuthnStatement AuthnInstant=\"{auth_time}\" SessionIndex=\"{index}\" SessionNotOnOrAfter=\"{session_expiry}\"><saml:AuthnContext><saml:AuthnContextClassRef>{}</saml:AuthnContextClassRef></saml:AuthnContext></saml:AuthnStatement>{attributes}</saml:Assertion>",
                escape(&issuer),
                settings.name_id_format.uri(),
                escape(&issuer),
                escape(&settings.sp_entity_id),
                escape(&subject),
                escape(&request.acs),
                escape(&settings.sp_entity_id),
                authn_context(self, identity)
            );
            if xml.len() > 48 * 1024 {
                return Err(Error::bad("SAML assertion exceeds 48 KiB"));
            }
            Some(xml)
        } else {
            None
        };
        let mut xml = wire::response_xml(
            &issuer,
            &request.acs,
            request.id.as_deref(),
            status,
            assertion.as_deref(),
        )?;
        if assertion.is_some() {
            xml = sign(&xml, &key, &settings.idp_certificate_pem, false)?;
            if let Some(cert) = &settings.encryption_certificate_pem {
                xml = risaml::crypto::encrypt_assertion(
                    &xml,
                    cert,
                    "http://www.w3.org/2009/xmlenc11#aes256-gcm",
                    "http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p",
                    "saml",
                )
                .map_err(Error::internal)?;
            }
        }
        xml = sign(&xml, &key, &settings.idp_certificate_pem, true)?;
        audit(
            tx,
            identity.map(|i| i.user_id.as_str()).unwrap_or("anonymous"),
            "saml.response",
            &client.id,
        )?;
        Ok(post(
            request.acs.clone(),
            xml,
            request.relay_state.clone(),
            cookies,
        ))
    }
    fn saml_logout_in(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        settings: &Settings,
        request: wire::Logout,
        is_post: bool,
    ) -> Result<Reply> {
        if is_post {
            settings.slo_post_url.as_ref()
        } else {
            settings.slo_redirect_url.as_ref()
        }
        .ok_or_else(|| Error::bad("SAML logout response binding is not registered"))?;
        // Resolve every index before changing any session, and require the exact SP and NameID.
        let fingerprint = fingerprint(client)?;
        let issuer = settings.issuer(self, client);
        let mut sessions = BTreeSet::new();
        for index in &request.indices {
            let rp = tx
                .get::<RpSession>("saml_sessions", &digest(index))?
                .filter(|r| {
                    r.expires_at > now()
                        && r.client_id == client.id
                        && r.issuer == issuer
                        && r.fingerprint == fingerprint
                        && r.name_id == request.name_id
                        && request.format.as_ref().is_none_or(|f| *f == r.format.uri())
                })
                .ok_or_else(Error::forbidden)?;
            sessions.insert(rp.identity.session_id);
        }
        let mut fronts = BTreeSet::new();
        for sid in &sessions {
            if let Some(mut session) = tx.get::<Session>("sessions", sid)? {
                fronts.extend(crate::session_protocol::frontchannel_urls(
                    tx,
                    sid,
                    &self.config.issuer,
                )?);
                session.revoked = true;
                tx.put("sessions", &session.id, &session)?;
                crate::logout::queue_session(tx, &session.id)?;
                crate::ssf::enqueue(
                    tx,
                    &session.identity.user_id,
                    crate::ssf::SESSION_REVOKED,
                    "",
                )?;
                audit(tx, &session.identity.user_id, "saml.logout", &client.id)?;
            }
        }
        logout::begin(
            self,
            tx,
            &sessions,
            logout::Finish {
                redirect: None,
                response: Some(logout::ReturnResponse {
                    peer: logout::Peer::Client {
                        id: client.id.clone(),
                        fingerprint: fingerprint.clone(),
                    },
                    request,
                    post: is_post,
                }),
                frontchannel_urls: fronts,
            },
        )
    }
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    logout::cleanup(tx, at)?;
    for (id, p) in tx.maintenance_page::<Pending>("saml_requests")? {
        if p.expires_at <= at {
            remove(tx, &p)?;
            tx.delete("saml_requests", &id)?;
        }
    }
    for (id, expiry) in tx.maintenance_page::<u64>("saml_replays")? {
        if expiry <= at {
            tx.delete("saml_replays", &id)?;
        }
    }
    for (id, s) in tx.maintenance_page::<RpSession>("saml_sessions")? {
        if s.expires_at <= at {
            tx.delete("saml_sessions", &id)?;
        }
    }
    for (id, s) in tx.maintenance_page::<Consent>("saml_consents")? {
        if s.expires_at <= at {
            tx.delete("saml_consents", &id)?;
        }
    }
    Ok(())
}

pub fn import_sp_metadata(
    xml: &str,
    entity_id: &str,
    idp_certificate_pem: String,
) -> Result<Value> {
    wire::import_metadata(xml, entity_id, idp_certificate_pem)
}

pub(crate) fn consent(tx: &Tx<'_>, user_id: &str, client: &Client) -> Result<Option<Value>> {
    let key = digest(&format!("{user_id}\0{}", client.id));
    let Some(consent) = tx
        .get::<Consent>("saml_consents", &key)?
        .filter(|c| c.expires_at > now())
    else {
        return Ok(None);
    };
    let Some(settings) = &client.settings.saml else {
        return Ok(None);
    };
    Ok(Some(
        json!({"protocol":"saml","client_id":client.id,"name":client.name,"scopes":settings.scopes(client)?,"expires_at":consent.expires_at}),
    ))
}
pub(crate) fn revoke_consent(tx: &Tx<'_>, user_id: &str, cid: &str) -> Result<()> {
    tx.delete("saml_consents", &digest(&format!("{user_id}\0{cid}")))?;
    for (_, p) in tx.list::<Pending>("saml_requests")? {
        if p.client_id == cid
            && p.decision
                .as_ref()
                .is_some_and(|d| d.identity.user_id == user_id)
        {
            remove(tx, &p)?;
        }
    }
    Ok(())
}
