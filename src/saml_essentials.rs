//! Shared SAML record shapes and logout handoff with no XML protocol adapter.
use crate::{
    browser::{BrowserDecision, BrowserReply},
    core::Core,
    error::{Error, Result},
    model::Client,
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use webauthn_rs::prelude::PublicKeyCredential;

pub use crate::model::client_settings::saml::{Attribute, NameIdFormat, Settings};

impl Settings {
    pub fn validate(&self, _client: &Client) -> Result<()> {
        Err(Error::bad("SAML clients require the Platform build"))
    }

    pub(crate) fn scopes(&self, _client: &Client) -> Result<BTreeSet<String>> {
        Err(Error::bad("SAML clients require the Platform build"))
    }
}

pub fn validate_key(_tx: &Tx<'_>, client: &Client) -> Result<()> {
    if client.settings.saml.is_some() {
        return Err(Error::bad("SAML clients require the Platform build"));
    }
    Ok(())
}

pub fn cleanup(_tx: &Tx<'_>, _at: u64) -> Result<()> {
    Ok(())
}

pub fn import_sp_metadata(_xml: &str, _entity_id: &str, _certificate: String) -> Result<Value> {
    Err(Error::bad(
        "SAML metadata import requires the Platform build",
    ))
}

pub(crate) fn consent(_tx: &Tx<'_>, _user_id: &str, _client: &Client) -> Result<Option<Value>> {
    Ok(None)
}

pub(crate) fn revoke_consent(_tx: &Tx<'_>, _user_id: &str, _client_id: &str) -> Result<()> {
    Ok(())
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

impl Core {
    pub(crate) fn is_saml_code(&self, _code: &str) -> Result<bool> {
        Ok(false)
    }

    pub fn saml_details(&self, _token: &str, _code: &str) -> Result<Value> {
        Err(Error::bad("SAML requires the Platform build"))
    }

    pub fn saml_decide(&self, _token: &str, _decision: BrowserDecision) -> Result<Value> {
        Err(Error::bad("SAML requires the Platform build"))
    }

    pub fn saml_state(
        &self,
        _id: &str,
        _binding: Option<&str>,
        _sso: Option<&str>,
    ) -> Result<Value> {
        Err(Error::bad("SAML requires the Platform build"))
    }

    pub fn saml_password(
        &self,
        _id: &str,
        _binding: Option<&str>,
        _sso: Option<&str>,
        _username: String,
        _password: String,
        _otp: Option<String>,
    ) -> Result<BrowserReply> {
        Err(Error::bad("SAML requires the Platform build"))
    }

    pub fn saml_passkey_start(
        &self,
        _id: &str,
        _binding: Option<&str>,
        _sso: Option<&str>,
    ) -> Result<Value> {
        Err(Error::bad("SAML requires the Platform build"))
    }

    pub fn saml_passkey_finish(
        &self,
        _id: &str,
        _binding: Option<&str>,
        _sso: Option<&str>,
        _ceremony: &str,
        _response: PublicKeyCredential,
    ) -> Result<BrowserReply> {
        Err(Error::bad("SAML requires the Platform build"))
    }

    pub fn saml_browser_decide(
        &self,
        _id: &str,
        _binding: Option<&str>,
        _sso: Option<&str>,
        _approve: bool,
        _remember: bool,
        _session_ref: Option<String>,
    ) -> Result<Value> {
        Err(Error::bad("SAML requires the Platform build"))
    }
}

pub mod logout {
    use super::{BTreeSet, Core, Error, Reply, Result, Tx, Value, json};
    use crate::{model::Session, session_protocol::PostLogoutReturn};

    #[derive(Default)]
    pub(crate) struct Finish {
        pub frontchannel_urls: BTreeSet<String>,
    }

    /// OIDC front-channel URLs still propagate after local revocation.
    pub(crate) fn begin(
        _core: &Core,
        tx: &Tx<'_>,
        session_ids: &BTreeSet<String>,
        finish: Finish,
    ) -> Result<Reply> {
        for sid in session_ids {
            if tx
                .get::<Session>("sessions", sid)?
                .is_some_and(|session| !session.revoked)
            {
                return Err(Error::internal(
                    "Logout propagation requires local revocation",
                ));
            }
        }
        Ok(Reply::LogoutPage(json!({
            "logged_out": true,
            "frontchannel_urls": finish.frontchannel_urls,
        })))
    }

    pub(crate) fn redirect(
        _core: &Core,
        _tx: &Tx<'_>,
        _sid: &str,
        original: Option<PostLogoutReturn>,
    ) -> Result<Value> {
        Ok(json!({"redirect_uri": original.map(|target| target.rendered_uri)}))
    }

    pub(crate) fn is_continuation(_core: &Core, _tx: &Tx<'_>, _sid: &str, _uri: &str) -> Result<bool> {
        Ok(false)
    }
}
