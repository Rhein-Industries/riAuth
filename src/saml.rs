//! SAML 2.0 browser SSO with signed requests, terminal consent and pinned trust.
pub mod logout;
pub(crate) mod wire;
use crate::{
    browser::BrowserReply,
    crypto::{self, SigningKey, digest, now},
    error::{Error, Result},
    model::{Client, Identity, User},
};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use wire::RSA256;

pub use crate::model::client_settings::saml::{Attribute, NameIdFormat, Settings};

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
pub(crate) fn client(tx: &impl SamlTx, id: &str) -> Result<(Client, Settings)> {
    let client = tx
        .client_record(id)?
        .filter(|c| c.enabled)
        .ok_or_else(Error::forbidden)?;
    let settings = client.settings.saml.clone().ok_or_else(Error::forbidden)?;
    settings.validate(&client)?;
    Ok((client, settings))
}
pub(crate) fn fingerprint(client: &Client) -> Result<String> {
    Ok(digest(
        &serde_json::to_string(client).map_err(Error::internal)?,
    ))
}
pub fn validate_key(tx: &impl SamlTx, client: &Client) -> Result<()> {
    if let Some(settings) = &client.settings.saml {
        let key = tx.signing_key(client)?;
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
pub struct Decision {
    pub(crate) identity: Identity,
    pub(crate) approve: bool,
    pub(crate) remember: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Pending {
    pub(crate) id: String,
    pub(crate) code: String,
    pub(crate) browser_hash: String,
    pub(crate) client_id: String,
    pub(crate) client_fingerprint: String,
    pub(crate) request: wire::Authn,
    pub(crate) expires_at: u64,
    pub(crate) decision: Option<Decision>,
    /// Digest key of the proof a browser sign-in in this interaction bound to the request.
    #[serde(default)]
    pub(crate) authentication: Option<String>,
    /// Declined in the browser; resume answers RequestDenied without an identity.
    #[serde(default)]
    pub(crate) cancelled: bool,
    #[serde(default)]
    pub(crate) requested_from: Option<Value>,
    /// The session that approved on the interaction page. Only a browser signed in to it
    /// collects the response; a terminal approval leaves it unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) approved_by: Option<String>,
}
impl Pending {
    pub(crate) fn request_hash(&self) -> String {
        digest(&format!("saml\0{}\0{}", self.id, self.client_fingerprint))
    }
    pub(crate) fn decided(&self) -> bool {
        self.decision.is_some() || self.cancelled
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Consent {
    pub(crate) fingerprint: String,
    pub(crate) expires_at: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct RpSession {
    #[serde(default)]
    pub(crate) index: Option<String>,
    #[serde(default)]
    pub(crate) issuer: String,
    #[serde(default)]
    pub(crate) fingerprint: String,
    pub(crate) client_id: String,
    pub(crate) name_id: String,
    pub(crate) format: NameIdFormat,
    pub(crate) identity: Identity,
    pub(crate) expires_at: u64,
}

/// SAML request, replay and consent records use the caller's transaction.
/// Assembly maps these operations to the existing collections and signing key.
pub trait SamlTx {
    fn client_record(&self, id: &str) -> Result<Option<Client>>;
    fn signing_key(&self, client: &Client) -> Result<SigningKey>;
    fn code_request_id(&self, hash: &str) -> Result<Option<String>>;
    fn request(&self, id: &str) -> Result<Option<Pending>>;
    fn requests(&self) -> Result<Vec<(String, Pending)>>;
    fn delete_proof(&self, key: &str) -> Result<()>;
    fn delete_code(&self, hash: &str) -> Result<()>;
    fn delete_request(&self, id: &str) -> Result<()>;
    fn consent_record(&self, key: &str) -> Result<Option<Consent>>;
    fn delete_consent(&self, key: &str) -> Result<()>;
    fn cleanup_logout(&self, at: u64) -> Result<()>;
    fn request_page(&self) -> Result<Vec<(String, Pending)>>;
    fn replay_page(&self) -> Result<Vec<(String, u64)>>;
    fn delete_replay(&self, id: &str) -> Result<()>;
    fn session_page(&self) -> Result<Vec<(String, RpSession)>>;
    fn delete_session(&self, id: &str) -> Result<()>;
    fn consent_page(&self) -> Result<Vec<(String, Consent)>>;
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
pub(crate) fn post(
    target: String,
    xml: String,
    relay: Option<String>,
    cookies: Vec<String>,
) -> Reply {
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
pub(crate) fn pending(tx: &impl SamlTx, code: &str) -> Result<Pending> {
    let hash = digest(&crypto::normalize_code(code)?);
    let id = tx
        .code_request_id(&hash)?
        .ok_or_else(|| Error::missing("SAML request not found"))?;
    tx.request(&id)?
        .filter(|p| p.expires_at > now())
        .ok_or_else(|| Error::missing("SAML request expired"))
}
pub(crate) fn remove(tx: &impl SamlTx, p: &Pending) -> Result<()> {
    if let Some(proof) = &p.authentication {
        tx.delete_proof(proof)?;
    }
    tx.delete_code(&digest(&crypto::normalize_code(&p.code)?))?;
    tx.delete_request(&p.id)
}
/// The browser's own request: its binding cookie proves which browser started it.
pub(crate) fn interaction(tx: &impl SamlTx, id: &str, binding: Option<&str>) -> Result<Pending> {
    let p = tx
        .request(id)?
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
pub(crate) fn undecided(p: Pending) -> Result<Pending> {
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
pub(crate) fn current_client(tx: &impl SamlTx, p: &Pending) -> Result<(Client, Settings)> {
    let (client, settings) = client(tx, &p.client_id)?;
    if fingerprint(&client)? != p.client_fingerprint {
        return Err(Error::conflict("SAML client changed"));
    }
    Ok((client, settings))
}
pub(crate) fn consent_key(identity: &Identity, client: &Client) -> String {
    digest(&format!("{}\0{}", identity.user_id, client.id))
}
/// Implicit consent, or a remembered consent for this exact client configuration.
pub(crate) fn consented(tx: &impl SamlTx, client: &Client, identity: &Identity) -> Result<bool> {
    if client.settings.implicit_consent {
        return Ok(true);
    }
    let fingerprint = fingerprint(client)?;
    Ok(tx
        .consent_record(&consent_key(identity, client))?
        .is_some_and(|c| c.expires_at > now() && c.fingerprint == fingerprint))
}
pub(crate) fn unavailable(mut state: Value, error: &str, message: Option<String>) -> Result<Value> {
    state["status"] = json!("unavailable");
    state["error"] = json!(error);
    state["message"] = json!(message);
    Ok(state)
}
pub(crate) fn name_id(
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
pub fn cleanup(tx: &impl SamlTx, at: u64) -> Result<()> {
    tx.cleanup_logout(at)?;
    for (id, p) in tx.request_page()? {
        if p.expires_at <= at {
            remove(tx, &p)?;
            tx.delete_request(&id)?;
        }
    }
    for (id, expiry) in tx.replay_page()? {
        if expiry <= at {
            tx.delete_replay(&id)?;
        }
    }
    for (id, s) in tx.session_page()? {
        if s.expires_at <= at {
            tx.delete_session(&id)?;
        }
    }
    for (id, s) in tx.consent_page()? {
        if s.expires_at <= at {
            tx.delete_consent(&id)?;
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

pub(crate) fn consent(tx: &impl SamlTx, user_id: &str, client: &Client) -> Result<Option<Value>> {
    let key = digest(&format!("{user_id}\0{}", client.id));
    let Some(consent) = tx.consent_record(&key)?.filter(|c| c.expires_at > now()) else {
        return Ok(None);
    };
    let Some(settings) = &client.settings.saml else {
        return Ok(None);
    };
    Ok(Some(
        json!({"protocol":"saml","client_id":client.id,"name":client.name,"scopes":settings.scopes(client)?,"expires_at":consent.expires_at}),
    ))
}
pub(crate) fn revoke_consent(tx: &impl SamlTx, user_id: &str, cid: &str) -> Result<()> {
    tx.delete_consent(&digest(&format!("{user_id}\0{cid}")))?;
    for (_, p) in tx.requests()? {
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
