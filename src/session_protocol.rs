use crate::{
    browser::BrowserReply,
    crypto::{self, digest, now},
    error::{Error, Result},
    logout::RpSession,
    model::Client,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Serialize, Deserialize)]
#[doc(hidden)]
pub struct Confirmation {
    pub(crate) id: String,
    pub(crate) code: String,
    pub(crate) browser_hash: String,
    pub(crate) expires_at: u64,
    pub(crate) client_id: Option<String>,
    pub(crate) user_id: Option<String>,
    pub(crate) session_id: Option<String>,
    pub(crate) redirect_uri: Option<String>,
    pub(crate) result: Option<Value>,
    /// Who asked for this sign-out, shown to the approving terminal.
    #[serde(default)]
    pub(crate) requested_from: Option<Value>,
}

/// Logout and session-state records from the caller's transaction.
pub trait SessionProtocolTx {
    fn rp_sessions(&self) -> Result<Vec<(String, RpSession)>>;
    fn client(&self, client_id: &str) -> Result<Option<Client>>;
    fn session_secret(&self) -> Result<Option<String>>;
    fn logout_confirmation_id(&self, code_hash: &str) -> Result<Option<String>>;
    fn logout_confirmation(&self, id: &str) -> Result<Option<Confirmation>>;
    fn put_logout_confirmation(&self, confirmation: &Confirmation) -> Result<()>;
    fn confirmation_page(&self) -> Result<Vec<(String, Confirmation)>>;
    fn delete_logout_code(&self, code_hash: &str) -> Result<()>;
    fn delete_logout_confirmation(&self, id: &str) -> Result<()>;
    fn logout_audit(&self, actor: &str, action: &str, target: &str) -> Result<()>;
}

pub fn logout_redirect(
    client: Option<&Client>,
    uri: Option<&str>,
    state: Option<&str>,
) -> Result<Option<String>> {
    if state.is_some_and(|s| s.len() > 512) {
        return Err(Error::bad("Logout state too long"));
    }
    let Some(uri) = uri else {
        return Ok(None);
    };
    if !client.is_some_and(|c| {
        c.settings
            .post_logout_redirect_uris
            .iter()
            .any(|u| u == uri)
    }) {
        return Err(Error::bad("Unregistered post-logout redirect URI"));
    }
    let mut uri = url::Url::parse(uri).map_err(|_| Error::bad("Invalid logout redirect"))?;
    if let Some(state) = state {
        uri.query_pairs_mut().append_pair("state", state);
    }
    Ok(Some(uri.into()))
}
pub fn frontchannel_urls(
    tx: &impl SessionProtocolTx,
    session_id: &str,
    issuer: &str,
) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    for (_, rp) in tx.rp_sessions()? {
        if rp.session_id != session_id || rp.expires_at.saturating_add(3600) < now() {
            continue;
        }
        if let Some(client) = tx.client(&rp.client_id)?
            && let Some(uri) = &client.settings.frontchannel_logout_uri
        {
            let mut uri = url::Url::parse(uri).map_err(Error::internal)?;
            uri.query_pairs_mut()
                .append_pair("iss", crate::issuer::for_client(issuer, &client))
                .append_pair("sid", &rp.sid);
            urls.push(uri.into());
        }
    }
    Ok(urls)
}
pub fn session_state(
    tx: &impl SessionProtocolTx,
    client_id: &str,
    origin: &str,
    session_id: &str,
    salt: Option<&str>,
) -> Result<String> {
    let secret = tx
        .session_secret()?
        .ok_or_else(|| Error::internal("Missing session secret"))?;
    let salt = salt
        .map(String::from)
        .unwrap_or_else(|| crypto::random_token(""));
    Ok(format!(
        "{}.{}",
        digest(&format!(
            "{client_id}\0{origin}\0{session_id}\0{secret}\0{salt}"
        )),
        salt
    ))
}
pub(crate) fn lookup(tx: &impl SessionProtocolTx, code: &str) -> Result<Confirmation> {
    let id = tx
        .logout_confirmation_id(&digest(&crypto::normalize_code(code)?))?
        .ok_or_else(|| Error::missing("Logout request not found"))?;
    tx.logout_confirmation(&id)?
        .filter(|c| c.expires_at > now())
        .ok_or_else(|| Error::missing("Logout request expired"))
}
/// The confirmation behind a logout page, for the browser holding its binding cookie.
pub(crate) fn bound(
    tx: &impl SessionProtocolTx,
    id: &str,
    binding: Option<&str>,
) -> Result<Confirmation> {
    let c = tx
        .logout_confirmation(id)?
        .filter(|c| c.expires_at > now())
        .ok_or_else(|| {
            Error::new(
                StatusCode::NOT_FOUND,
                "interaction_expired",
                "This sign-out request has expired",
            )
        })?;
    if !binding.is_some_and(|b| crypto::constant_eq(&digest(b), &c.browser_hash)) {
        return Err(Error::unauthorized());
    }
    Ok(c)
}
/// Records a decision that ends no session.
pub(crate) fn settle(
    tx: &impl SessionProtocolTx,
    c: &mut Confirmation,
    logged_out: bool,
) -> Result<()> {
    c.result =
        Some(json!({"logged_out":logged_out,"redirect_uri":c.redirect_uri,"frontchannel_urls":[]}));
    tx.put_logout_confirmation(c)
}
pub(crate) fn deny(tx: &impl SessionProtocolTx, c: &mut Confirmation, actor: &str) -> Result<()> {
    settle(tx, c, false)?;
    tx.logout_audit(
        actor,
        "logout.denied",
        c.client_id.as_deref().unwrap_or("session"),
    )
}
pub fn cleanup(tx: &impl SessionProtocolTx, at: u64) -> Result<()> {
    for (id, c) in tx.confirmation_page()? {
        if c.expires_at <= at {
            tx.delete_logout_code(&digest(&crypto::normalize_code(&c.code)?))?;
            tx.delete_logout_confirmation(&id)?;
        }
    }
    Ok(())
}

pub fn waiting_reply(mut value: Value) -> BrowserReply {
    let cookies = value
        .as_object_mut()
        .and_then(|m| m.remove("set_cookie"))
        .and_then(|v| v.as_str().map(String::from))
        .into_iter()
        .collect();
    let refresh = value["resume_uri"]
        .as_str()
        .map(|uri| format!("2; url={uri}"));
    BrowserReply {
        body: value,
        form_post: false,
        location: None,
        refresh,
        cookies,
    }
}
