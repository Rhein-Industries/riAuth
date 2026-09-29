//! Browser source-login and link transactions over concrete Core storage.

#[cfg(feature = "test-support")]
use crate::portal::sources::FAIL_DELIVERY;
use crate::{
    browser::BrowserReply,
    core::{Core, require_factor_session},
    crypto::{digest, now},
    error::{Error, Result},
    model::{Identity, Session, User},
    portal::{
        self_service::Binding,
        sources::{ID, KIND},
    },
    signin::{FRESH_SECONDS, bearer_backed},
    source::{Finish, Source},
    store::Tx,
};
use axum::http::StatusCode;
use serde_json::{Value, json};

/// A source login lasts ten minutes.
const LOGIN_SECONDS: u64 = 600;

fn expired() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "source_login_expired",
        "This sign-in has expired, was cancelled or has already finished. Start again.",
    )
}

fn session_changed() -> Error {
    Error::new(
        StatusCode::CONFLICT,
        "session_changed",
        "This browser's sign-in changed after linking started. Sign in again, then start linking again.",
    )
}

fn refused() -> Error {
    Error::new(
        StatusCode::FORBIDDEN,
        "access_denied",
        "riAuth didn't accept this provider account here. It may be linked to another riAuth account, the riAuth account may be disabled or an administrator this provider can't sign in, or the sign-in that started linking has expired.",
    )
}

fn reply(body: Value, cookies: Vec<String>) -> BrowserReply {
    BrowserReply {
        form_post: false,
        body,
        location: None,
        refresh: None,
        cookies,
    }
}

/// Enabled sources a browser can use. A SAML source answers with a cross-site POST, which
/// does not carry the browser's Lax login cookie, so SAML sign-in stays in the CLI for now.
fn browser_sources(tx: &Tx<'_>) -> Result<Vec<Source>> {
    let mut sources: Vec<Source> = tx
        .list::<Source>("sources")?
        .into_iter()
        .map(|(_, source)| source)
        .filter(|source| source.enabled && source.saml.is_none())
        .collect();
    sources.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(sources)
}

impl Core {
    /// A browser-started source callback continues on its review page only when
    /// this browser's one-use cookie matches the callback state.
    pub(crate) fn portal_source_continuation(
        &self,
        binding: Option<&str>,
        state: Option<&str>,
    ) -> Option<String> {
        let (_, started) = binding?.split_once('.')?;
        let state = state?;
        crate::crypto::constant_eq(started, &digest(state))
            .then(|| format!("{}account/sources/continue", self.cookie_path()))
    }

    pub fn portal_source_list(&self) -> Result<Value> {
        self.store.read(|tx| {
            let sources = browser_sources(tx)?
                .iter()
                .map(|source| json!({"id": source.id, "name": source.name}))
                .collect::<Vec<_>>();
            Ok(json!({"sources": sources}))
        })
    }

    /// The signed-in account's linked providers and the sources it could link next.
    pub fn portal_source_links(&self, sso: Option<&str>) -> Result<Value> {
        self.store.read(|tx| {
            let (user, session) = self.portal_session(tx, sso)?;
            let sources = browser_sources(tx)?;
            let all = tx.list::<Source>("sources")?;
            let links = crate::source::links_of(tx, &user.id)?
                .into_iter()
                .map(|mut link| {
                    let name = all
                        .iter()
                        .find(|(id, _)| link["source"] == id.as_str())
                        .map(|(_, source)| source.name.clone());
                    link["name"] = json!(name.unwrap_or_else(|| {
                        link["source"].as_str().unwrap_or_default().to_owned()
                    }));
                    link
                })
                .collect::<Vec<_>>();
            let linkable = sources
                .iter()
                .filter(|source| !user.admin || source.allow_admin_login)
                .filter(|source| !links.iter().any(|link| link["source"] == source.id.as_str()))
                .map(|source| json!({"id": source.id, "name": source.name}))
                .collect::<Vec<_>>();
            Ok(json!({
                "user": {"id": user.id, "username": user.username, "display_name": user.display_name},
                "current_session_id": session.id,
                "links": links,
                "linkable": linkable,
                "can_change": self.linking_check(tx, &user, &session).is_ok(),
                "local_session": session.identity.source.is_none(),
            }))
        })
    }

    /// Linking and unlinking change how this account signs in, so they need this browser's
    /// own recent local sign-in, as factor changes do, bound to the account the page shows.
    fn linking_session(
        &self,
        tx: &Tx<'_>,
        sso: Option<&str>,
        binding: &Binding,
    ) -> Result<(User, Session)> {
        let (user, session) = self.portal_session(tx, sso)?;
        if binding.expected_user_id != user.id {
            return Err(Error::new(
                StatusCode::CONFLICT,
                "account_changed",
                "Your signed-in account changed. Reload this page.",
            ));
        }
        if binding.expected_session_id != session.id {
            return Err(Error::new(
                StatusCode::CONFLICT,
                "session_changed",
                "Your browser session changed. Reload this page.",
            ));
        }
        self.linking_check(tx, &user, &session)?;
        Ok((user, session))
    }

    fn linking_check(&self, tx: &Tx<'_>, user: &User, session: &Session) -> Result<()> {
        if session.identity.source.is_some() {
            return Err(Error::new(
                StatusCode::FORBIDDEN,
                "local_sign_in_required",
                "Sign in with your riAuth password or passkey to change linked providers.",
            ));
        }
        if bearer_backed(tx, session)?
            || now().saturating_sub(session.identity.auth_time) > FRESH_SECONDS
        {
            return Err(Error::new(
                StatusCode::FORBIDDEN,
                "reauthentication_required",
                "Sign in again in this browser before changing linked providers.",
            ));
        }
        require_factor_session(user, session)
    }

    /// Starts a browser login. Only the authorization URL reaches the page; the credential
    /// that finishes the login is set as an HttpOnly cookie for this browser alone.
    /// The upstream callback redeems its code only when that same cookie is presented.
    pub fn portal_source_start(
        &self,
        sso: Option<&str>,
        id: &str,
        link: Option<&Binding>,
    ) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let source = browser_sources(tx)?
                .into_iter()
                .find(|source| source.id == id)
                .ok_or_else(|| {
                    Error::missing("This sign-in provider isn't available in the browser")
                })?;
            let started = match link {
                Some(binding) => {
                    let (user, session) = self.linking_session(tx, sso, binding)?;
                    if user.admin && !source.allow_admin_login {
                        return Err(Error::new(
                            StatusCode::FORBIDDEN,
                            "access_denied",
                            "Administrator accounts can't be linked to this provider.",
                        ));
                    }
                    self.source_start_browser(tx, id, Some((&user, &session)))?
                }
                None => self.source_start_browser(tx, id, None)?,
            };
            let credential = started["credential"]["token"]
                .as_str()
                .ok_or_else(|| Error::internal("source login credential missing"))?;
            let authorization = started["authorization_url"]
                .as_str()
                .and_then(|url| url::Url::parse(url).ok())
                .ok_or_else(|| Error::internal("source authorization URL missing"))?;
            let state = authorization
                .query_pairs()
                .find(|(key, _)| key == "state")
                .map(|(_, value)| digest(&value))
                .ok_or_else(|| Error::internal("source login state missing"))?;
            Ok(reply(
                json!({
                    "authorization_url": started["authorization_url"],
                    "expires_at": started["credential"]["expires_at"],
                    "source": {"id": source.id, "name": source.name},
                }),
                vec![self.binding_cookie(
                    KIND,
                    ID,
                    &format!("{credential}.{state}"),
                    &self.cookie_path(),
                    LOGIN_SECONDS,
                )],
            ))
        })
    }

    /// What finishing would do, read without spending the login: the provider account, the
    /// riAuth account it signs in or links, and whether a local code is still needed.
    pub fn portal_source_review(&self, credential: Option<&str>) -> Result<Value> {
        let credential = credential.ok_or_else(expired)?;
        let mut review = self
            .source_finish(Finish {
                credential: credential.into(),
                approve: false,
                otp: None,
            })
            .map_err(|error| match error.status {
                StatusCode::UNAUTHORIZED => expired(),
                // The account rules refuse before a review: the same answer finishing gives.
                StatusCode::FORBIDDEN => refused(),
                _ => error,
            })?;
        if review["status"] == "review" {
            // The review names the upstream by issuer; show the source's own name when only one
            // enabled source uses it.
            let issuer = review["issuer"].as_str().unwrap_or_default().to_owned();
            let names = self.store.read(|tx| {
                Ok(browser_sources(tx)?
                    .into_iter()
                    .filter(|source| source.issuer == issuer)
                    .map(|source| source.name)
                    .collect::<Vec<_>>())
            })?;
            review["provider"] = json!(match names.as_slice() {
                [name] => name.clone(),
                _ => issuer,
            });
        }
        Ok(review)
    }

    /// Cancels, or finishes after the page's review, in one transaction with the source
    /// login: a link is written only while this browser is still on the exact local session
    /// that started it, and a sign-in only together with this browser's session. A sign-in
    /// never exposes a bearer token; a link keeps the local session that started it.
    pub fn portal_source_finish(
        &self,
        credential: Option<&str>,
        sso: Option<&str>,
        approve: bool,
        otp: Option<String>,
    ) -> Result<BrowserReply> {
        let forget = self.binding_cookie(KIND, ID, "", &self.cookie_path(), 0);
        let Some(credential) = credential else {
            return Err(expired());
        };
        if !approve {
            // Without its credential the login is unreachable; it expires on its own.
            return Ok(reply(json!({"cancelled": true}), vec![forget]));
        }
        let bind = |tx: &Tx<'_>, target: Option<&Identity>| {
            let Some(target) = target else {
                return Ok(());
            };
            let (user, session) =
                self.portal_session(tx, sso)
                    .map_err(|error| match error.status {
                        StatusCode::UNAUTHORIZED => session_changed(),
                        _ => error,
                    })?;
            if session.id != target.session_id
                || session.identity.user_id != target.user_id
                || session.identity.epoch != target.epoch
                || user.epoch != target.epoch
            {
                return Err(session_changed());
            }
            self.linking_check(tx, &user, &session)
        };
        let deliver = |tx: &Tx<'_>, linking: bool, finished: &Value| {
            #[cfg(feature = "test-support")]
            if FAIL_DELIVERY.with(std::cell::Cell::get) {
                return Err(Error::internal("injected browser delivery failure"));
            }
            if finished["status"] != "complete" {
                return Err(Error::new(
                    StatusCode::CONFLICT,
                    "source_login_pending",
                    "The provider hasn't finished signing you in yet. Start again.",
                ));
            }
            let token = finished["session_token"]
                .as_str()
                .ok_or_else(|| Error::internal("source session missing"))?;
            let sid = tx
                .get::<String>("session_tokens", &digest(token))?
                .ok_or_else(|| Error::internal("source session missing"))?;
            // The bearer token never leaves this transaction, so the session is reachable only
            // by this browser, as for a source stage.
            tx.delete("session_tokens", &digest(token))?;
            if linking {
                // The link needs no sign-in of its own: the browser keeps its local session.
                let mut session = tx
                    .get::<Session>("sessions", &sid)?
                    .ok_or_else(|| Error::internal("source session missing"))?;
                session.revoked = true;
                tx.put("sessions", &sid, &session)?;
                return Ok(reply(
                    json!({"linked": true, "user": finished["user"]}),
                    vec![forget.clone()],
                ));
            }
            // As for a password sign-in: a session this browser can no longer reach is retired.
            let user = finished["user"]["id"].as_str().unwrap_or_default();
            let mut cookies = self.point_browser(tx, sso, &sid, user)?;
            cookies.push(forget.clone());
            Ok(reply(
                json!({"signed_in": true, "user": finished["user"]}),
                cookies,
            ))
        };
        match self.source_finish_browser(credential, otp.as_deref(), bind, deliver) {
            Ok(finished) => Ok(finished),
            // A wrong code leaves the login open for another try; otherwise it has ended.
            Err(error) if error.status == StatusCode::UNAUTHORIZED => {
                Err(match self.portal_source_review(Some(credential)) {
                    Ok(_) => Error::new(
                        StatusCode::UNAUTHORIZED,
                        "invalid_code",
                        "That code wasn't accepted. Enter the current code from your authenticator app, or a recovery code.",
                    ),
                    Err(error) => error,
                })
            }
            Err(error)
                if error.status == StatusCode::FORBIDDEN && error.code == "access_denied" =>
            {
                Err(refused())
            }
            Err(error) => Err(error),
        }
    }

    pub fn portal_source_unlink(
        &self,
        sso: Option<&str>,
        binding: &Binding,
        link_id: &str,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.linking_session(tx, sso, binding)?;
            crate::management::unlink_source(tx, &user, &session, link_id)
        })
    }
}
