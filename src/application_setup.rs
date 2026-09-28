//! Application setup checks for the browser wizard and the application page.
//!
//! `check_new_client` answers "would this create succeed?" with the management write path's
//! own authorization and validation (`management::check_client`), in a read transaction, so
//! the wizard never relies on a copy of the rules. `client_diagnostics` reports how an
//! existing application is connected: the endpoints and credentials its app needs and the
//! configuration that stops people from signing in. Neither writes, issues a secret or
//! contacts the application.
use crate::{
    agent::Principal,
    claims::ClaimSource,
    core::Core,
    crypto::now,
    error::{Error, Result},
    jose::ClientAuthMethod,
    model::{Client, Family, Grant, Group, NewClient, User},
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use url::Url;

impl Core {
    /// Authorize and validate a new application exactly as `create_client` would, without
    /// writing it. Returns the record as it would be stored and its setup checks.
    pub fn check_new_client(&self, token: &str, input: NewClient) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.management(
                tx,
                token,
                "client.write",
                &format!("client/{}", input.client_id),
            )?;
            let (client, secret) = crate::management::new_client(input);
            let client =
                crate::management::check_client(tx, &self.config, &actor, None, client, secret)?;
            Ok(json!({
                "valid": true,
                "client": client.view(),
                "connection": self.connection(&client),
                "claims": claims(&client),
                "checks": checks(tx, &actor, &client, false)?,
            }))
        })
    }

    /// Connection details and configuration checks for an existing application.
    pub fn client_diagnostics(&self, token: &str, cid: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.management(tx, token, "client.read", &format!("client/{cid}"))?;
            let client = tx
                .get::<Client>("clients", cid)?
                .ok_or_else(|| Error::missing("Client not found"))?;
            Ok(json!({
                "client_id": client.id,
                "checked_at": now(),
                "connection": self.connection(&client),
                "claims": claims(&client),
                "checks": checks(tx, &actor, &client, true)?,
            }))
        })
    }

    /// What the application's own configuration needs: issuer, endpoints and how it
    /// authenticates. Endpoints are shared; a per-client issuer changes only `iss`.
    fn connection(&self, client: &Client) -> Value {
        let base = self.config.issuer.trim_end_matches('/');
        let issuer = crate::issuer::for_client(&self.config.issuer, client);
        let token_endpoint = format!("{base}/oauth/token");
        json!({
            "issuer": issuer,
            "discovery_url": format!("{}/.well-known/openid-configuration", issuer.trim_end_matches('/')),
            "authorization_endpoint": format!("{base}/oauth/authorize"),
            "token_endpoint": token_endpoint,
            "userinfo_endpoint": format!("{base}/oauth/userinfo"),
            "jwks_uri": format!("{base}/oauth/jwks"),
            "end_session_endpoint": format!("{base}/oauth/logout"),
            "client_id": client.id,
            "token_endpoint_auth_methods": auth_methods(client),
            // A private key JWT's audience is the token endpoint of the primary issuer.
            "client_assertion_audience": (client.settings.token_endpoint_auth_method == Some(ClientAuthMethod::PrivateKeyJwt)).then_some(token_endpoint),
            "pkce": !client.service,
            "scopes": client.scopes,
            "redirect_uris": client.redirect_uris,
            "post_logout_redirect_uris": client.settings.post_logout_redirect_uris,
            "origins": client.settings.origins,
        })
    }
}

/// The token endpoint methods `oidc::authenticate_client` accepts for this client.
fn auth_methods(client: &Client) -> Vec<&'static str> {
    let method = |m: &ClientAuthMethod| match m {
        ClientAuthMethod::None => "none",
        ClientAuthMethod::ClientSecretBasic => "client_secret_basic",
        ClientAuthMethod::ClientSecretPost => "client_secret_post",
        ClientAuthMethod::PrivateKeyJwt => "private_key_jwt",
    };
    match (
        &client.settings.token_endpoint_auth_method,
        &client.secret_hash,
    ) {
        (Some(m), _) => vec![method(m)],
        (None, Some(_)) => vec!["client_secret_basic", "client_secret_post"],
        (None, None) => vec!["none"],
    }
}

/// OIDC sign-in applications: not services, SAML or proxy records.
fn oidc(client: &Client) -> bool {
    !client.service && client.settings.saml.is_none() && client.settings.proxy.is_none()
}

/// The claims each scope adds and where they are delivered, as `claims::mapped_claims`
/// builds them.
fn claims(client: &Client) -> Value {
    let s = &client.settings;
    let mut by_scope: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut add = |scope: &str, claim: &str, source: String| {
        if client.scopes.contains(scope) {
            by_scope
                .entry(scope.to_owned())
                .or_default()
                .push(json!({"claim": claim, "source": source}));
        }
    };
    if !client.service {
        add(
            "openid",
            "sub",
            if s.pairwise_sector.is_some() {
                "pairwise subject".into()
            } else {
                "user id".into()
            },
        );
        add("profile", "name", "display name".into());
        add("profile", "preferred_username", "username".into());
        add("email", "email", "email address".into());
        add("email", "email_verified", "email verified".into());
        add("groups", "groups", "group names".into());
        if s.groups_in_profile && !client.scopes.contains("groups") {
            add("profile", "groups", "group names".into());
        }
    }
    for m in &s.claim_mappings {
        let source = match &m.source {
            ClaimSource::Username => "username".into(),
            ClaimSource::DisplayName => "display name".into(),
            ClaimSource::Email => "email address".into(),
            ClaimSource::EmailVerified => "email verified".into(),
            ClaimSource::Groups => "group names".into(),
            ClaimSource::Attribute { key } => format!("attribute {key}"),
            ClaimSource::Literal { value } => format!("fixed value {value}"),
        };
        add(&m.scope, &m.claim, source);
    }
    json!({
        "by_scope": by_scope,
        "id_token": !s.userinfo_only,
        "userinfo": true,
        "access_token": s.claims_in_access_token,
    })
}

fn check(level: &str, id: &str, title: impl Into<String>, detail: impl Into<String>) -> Value {
    json!({"level": level, "id": id, "title": title.into(), "detail": detail.into()})
}

fn origin(uri: &str) -> Option<String> {
    let url = Url::parse(uri).ok()?;
    matches!(url.scheme(), "http" | "https").then(|| url.origin().ascii_serialization())
}

fn loopback(url: &Url) -> bool {
    matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
}

/// Ordered findings, most important first within each topic. `existing` adds checks that
/// only mean something once the record is stored (signing keys, issued tokens).
fn checks(tx: &Tx<'_>, actor: &Principal, client: &Client, existing: bool) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    let s = &client.settings;
    if existing {
        out.push(if client.enabled {
            check("ok", "enabled", "Enabled", "Sign-in and token requests are accepted.")
        } else {
            check("error", "enabled", "Disabled", "riAuth refuses sign-in and token requests for this application until it is enabled.")
        });
        out.push(match crate::keyring::for_client(tx, client) {
            Ok(_) => check(
                "ok",
                "signing",
                "Signing keys available",
                "Tokens are signed with keys published at the JWKS endpoint.",
            ),
            Err(error) => check("error", "signing", "No signing keys", error.to_string()),
        });
    }

    let redirect_origins: BTreeSet<String> = client
        .redirect_uris
        .iter()
        .filter_map(|u| origin(u))
        .collect();
    if oidc(client) {
        if client.redirect_uris.is_empty() {
            out.push(check("error", "redirects", "No redirect URI", "riAuth sends people back only to a registered callback, so sign-in cannot complete. Add the app's exact callback URL."));
        } else {
            out.push(check(
                "ok",
                "redirects",
                format!(
                    "{} redirect {} registered",
                    client.redirect_uris.len(),
                    if client.redirect_uris.len() == 1 {
                        "URI"
                    } else {
                        "URIs"
                    }
                ),
                "Only these exact addresses receive authorization codes.",
            ));
        }
        for uri in &client.redirect_uris {
            let Ok(url) = Url::parse(uri) else { continue };
            if url.scheme() == "http" && loopback(&url) {
                out.push(if s.native {
                    check("info", "redirect-loopback", format!("{uri} is a loopback callback"), "Native apps may use any port on 127.0.0.1 or [::1] with this path.")
                } else {
                    check("warn", "redirect-loopback", format!("{uri} uses plain HTTP on this machine"), "Loopback callbacks only work for someone running the app locally. Use the HTTPS address for a deployed app.")
                });
            } else if url.scheme() == "https" && loopback(&url) {
                out.push(check(
                    "warn",
                    "redirect-loopback",
                    format!("{uri} points at this machine"),
                    "Loopback callbacks only work for someone running the app locally.",
                ));
            }
        }

        let spa = !client.confidential() && !s.native;
        let missing: Vec<&String> = redirect_origins
            .iter()
            .filter(|o| !s.origins.contains(*o))
            .collect();
        if spa && s.origins.is_empty() {
            out.push(check("warn", "origins", "No allowed origin", format!(
                "A browser app redeems its code from the page, and the token endpoint only answers origins listed here. Add {}.",
                if missing.is_empty() { "the app's origin".to_owned() } else { missing.iter().map(|o| o.as_str()).collect::<Vec<_>>().join(", ") }
            )));
        } else if spa && !missing.is_empty() {
            out.push(check(
                "info",
                "origins",
                "Some callback origins are not allowed origins",
                format!(
                    "{} can receive codes but cannot call the token endpoint from the page.",
                    missing
                        .iter()
                        .map(|o| o.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
        for o in &s.origins {
            if !redirect_origins.contains(o) {
                out.push(check("info", "origin-unmatched", format!("{o} has no redirect URI"), "Browsers on this origin may call the token and userinfo endpoints, but no callback is registered there."));
            }
        }
        if s.post_logout_redirect_uris.is_empty() {
            out.push(check(
                "info",
                "logout",
                "No post-logout redirect",
                "After signing out, people stay on riAuth's signed-out page.",
            ));
        }
        if s.app.as_ref().and_then(|a| a.launch_url.as_ref()).is_none() {
            out.push(check("warn", "launch", "No launch URL", "The app shows as Setup pending on Your applications until it has a home or login page address."));
        }
    }

    out.push(credential_check(client));

    if !client.service {
        people_checks(tx, actor, client, &mut out)?;
        let delivers_groups = client.scopes.contains("groups")
            || (s.groups_in_profile && client.scopes.contains("profile"))
            || s.claim_mappings
                .iter()
                .any(|m| m.source == ClaimSource::Groups);
        if !client.allowed_groups.is_empty() && !delivers_groups && oidc(client) {
            out.push(check("info", "groups-claim", "Group membership is not sent", "riAuth limits sign-in by group, but the app doesn't receive a groups claim. Add the groups scope if the app also decides access by group."));
        }
        if s.userinfo_only {
            out.push(check("info", "claims-placement", "Identity claims only from userinfo", "The ID token carries only sub. The app must call the userinfo endpoint for profile claims."));
        }
    }

    if existing {
        out.push(activity(tx, client)?);
    }
    Ok(out)
}

fn credential_check(client: &Client) -> Value {
    let s = &client.settings;
    match (&s.token_endpoint_auth_method, &client.secret_hash) {
        (Some(ClientAuthMethod::PrivateKeyJwt), _) => {
            let keys = s.jwks.as_ref().map_or(0, |j| j.keys.len());
            check(
                "ok",
                "credentials",
                "Private key JWT",
                format!(
                    "The app signs a client assertion with one of {keys} pinned public {}. No shared secret exists.",
                    if keys == 1 { "key" } else { "keys" }
                ),
            )
        }
        (_, Some(_)) => check(
            "ok",
            "credentials",
            "Client secret",
            "riAuth stores only a hash. If the app's copy is lost, rotate the secret and update the app.",
        ),
        _ if client.service => check(
            "error",
            "credentials",
            "No credential",
            "Service clients need a client secret or a private key.",
        ),
        _ => check(
            "ok",
            "credentials",
            "Public client with PKCE",
            "The app has no secret; each sign-in proves itself with an S256 code challenge.",
        ),
    }
}

/// Who can sign in: allowed groups with enabled members, and MFA readiness. Only people and
/// groups the actor may read are counted, so the result never discloses more than the
/// actor's own reads would.
fn people_checks(
    tx: &Tx<'_>,
    actor: &Principal,
    client: &Client,
    out: &mut Vec<Value>,
) -> Result<()> {
    let users: Vec<User> = tx
        .list::<User>("users")?
        .into_iter()
        .map(|(_, u)| u)
        .filter(|u| u.enabled && actor.allows("user.read", &format!("user/{}", u.username)))
        .collect();
    let eligible: Vec<&User> = if client.allowed_groups.is_empty() {
        out.push(check(
            "info",
            "access",
            "Anyone with an account can sign in",
            "No group is selected. The application's other policies still apply.",
        ));
        users.iter().collect()
    } else {
        let mut members = BTreeSet::new();
        let mut empty = Vec::new();
        for name in &client.allowed_groups {
            if !actor.allows("group.read", &format!("group/{name}")) {
                continue;
            }
            let Some(group) = tx.get::<Group>("groups", name)? else {
                out.push(check(
                    "error",
                    "access",
                    format!("Group {name} no longer exists"),
                    "Choose another group.",
                ));
                continue;
            };
            let enabled: Vec<String> = group
                .members
                .into_iter()
                .filter(|id| users.iter().any(|u| &u.id == id))
                .collect();
            if enabled.is_empty() {
                empty.push(name.as_str());
            }
            members.extend(enabled);
        }
        let eligible: Vec<&User> = users.iter().filter(|u| members.contains(&u.id)).collect();
        if eligible.is_empty() {
            out.push(check("warn", "access", "Nobody can sign in yet", format!(
                "No enabled person is a member of {}. Add people to a group, or choose another group.",
                client.allowed_groups.iter().map(String::as_str).collect::<Vec<_>>().join(", ")
            )));
        } else {
            out.push(check(
                "ok",
                "access",
                format!(
                    "{} {} can sign in",
                    eligible.len(),
                    if eligible.len() == 1 {
                        "person"
                    } else {
                        "people"
                    }
                ),
                format!(
                    "Members of {}.",
                    client
                        .allowed_groups
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
            if !empty.is_empty() {
                out.push(check(
                    "info",
                    "access-empty",
                    format!(
                        "{} {} no enabled members",
                        empty.join(", "),
                        if empty.len() == 1 { "has" } else { "have" }
                    ),
                    "These groups don't let anyone in yet.",
                ));
            }
        }
        eligible
    };
    if client.require_mfa {
        let without = eligible
            .iter()
            .filter(|u| u.totp_secret.is_none() && !u.has_passkeys)
            .count();
        if without > 0 {
            out.push(check("warn", "mfa", format!("{without} of {} {} no second factor", eligible.len(), if without == 1 { "person has" } else { "people have" }), "This application requires a passkey or authenticator code, so they are refused until they add one from Your applications."));
        } else if !eligible.is_empty() {
            out.push(check(
                "ok",
                "mfa",
                "Everyone who can sign in has a second factor",
                "This application requires a passkey or authenticator code.",
            ));
        }
    }
    Ok(())
}

/// Whether the app has completed a token exchange: unexpired, unrevoked tokens it holds.
fn activity(tx: &Tx<'_>, client: &Client) -> Result<Value> {
    let at = now();
    let mut active = 0usize;
    let mut last = None::<u64>;
    let mut families = BTreeMap::<String, bool>::new();
    for bucket in ["access", "refresh"] {
        for (_, grant) in tx.list::<Grant>(bucket)? {
            if grant.client_id != client.id {
                continue;
            }
            last = last.max(Some(grant.issued_at));
            if grant.expires_at <= at {
                continue;
            }
            let revoked = match families.get(&grant.family_id) {
                Some(revoked) => *revoked,
                None => {
                    let revoked = tx
                        .get::<Family>("families", &grant.family_id)?
                        .is_none_or(|f| f.revoked || f.expires_at <= at);
                    families.insert(grant.family_id.clone(), revoked);
                    revoked
                }
            };
            if !revoked && bucket == "access" {
                active += 1;
            }
        }
    }
    Ok(match last {
        None => check(
            "info",
            "activity",
            "No tokens issued yet",
            "The app hasn't completed a sign-in or token request. Once it does, this shows when.",
        ),
        Some(issued) => {
            let mut value = check(
                "ok",
                "activity",
                "Tokens issued",
                format!(
                    "{active} active access {}.",
                    if active == 1 { "token" } else { "tokens" }
                ),
            );
            value["last_issued_at"] = json!(issued);
            value
        }
    })
}
