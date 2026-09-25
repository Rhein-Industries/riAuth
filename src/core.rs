use crate::{
    config::Config,
    crypto::{self, Keys, RetiredKey, SigningKey, digest, id, now},
    error::{Error, Result},
    model::*,
    store::{Store, Tx},
};
use axum::http::StatusCode;
use serde_json::{Value, json};
use std::{collections::BTreeSet, sync::Arc};

/// Who receives a verified password login: a bearer token, or a staged browser login.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Delivery {
    Bearer,
    Browser,
}

#[derive(Clone)]
pub struct Core {
    pub config: Config,
    pub store: Store,
    dummy_hash: Arc<String>,
}

impl Core {
    pub fn initialize(config: Config, input: NewUser) -> Result<Self> {
        config.validate().map_err(Error::internal)?;
        crate::config::private_dir(&config.data_dir).map_err(Error::internal)?;
        let store = Store::from_config(&config)?;
        if store.get::<u32>("meta", "schema")?.is_some() {
            return Err(Error::conflict("Instance already initialized"));
        }
        let key = SigningKey::generate()?;
        let dummy = crypto::password_hash(&crypto::random_token(""))?;
        let mut user = make_user(input)?;
        user.admin = true;
        store.write(|tx| {
            if tx.get::<u32>("meta", "schema")?.is_some() {
                return Err(Error::conflict("Instance already initialized"));
            }
            tx.put("meta", "schema", &crate::upgrade::SCHEMA)?;
            tx.put(
                "meta",
                "index_version",
                &crate::store::maintenance::INDEX_VERSION,
            )?;
            tx.put("meta", "issuer", &config.issuer)?;
            tx.put(
                "meta",
                "keys",
                &Keys {
                    active: key,
                    retired: vec![],
                },
            )?;
            tx.put("meta", "dummy_hash", &dummy)?;
            crate::password_history::record_imported_hash(
                tx,
                config.password_history,
                &user.id,
                "",
                &user.password_hash,
            )?;
            tx.put("users", &user.id, &user)?;
            tx.put("usernames", &user.username, &user.id)?;
            audit(tx, "bootstrap", "instance.initialize", &user.username)
        })?;
        Ok(Self {
            config,
            store,
            dummy_hash: Arc::new(dummy),
        })
    }
    pub fn open(config: Config) -> Result<Self> {
        config.validate().map_err(Error::internal)?;
        if config.postgres.is_none() && !config.data_dir.join("riauth.redb").is_file() {
            return Err(Error::missing("Database missing; run riauth init"));
        }
        let store = Store::from_config(&config)?;
        if store.get::<String>("meta", "issuer")?.as_deref() != Some(&config.issuer) {
            return Err(Error::bad(
                "Configured issuer does not match the initialized instance",
            ));
        }
        crate::upgrade::migrate(&store)?;
        let dummy = store
            .get::<String>("meta", "dummy_hash")?
            .ok_or_else(|| Error::internal("dummy hash missing"))?;
        Ok(Self {
            config,
            store,
            dummy_hash: Arc::new(dummy),
        })
    }

    pub fn login(&self, username: String, password: String, otp: Option<String>) -> Result<Value> {
        self.login_for(username, password, otp, None)
    }
    pub fn login_for(
        &self,
        username: String,
        password: String,
        otp: Option<String>,
        transaction: Option<String>,
    ) -> Result<Value> {
        self.password_login(username, password, otp, transaction, Delivery::Bearer)
    }
    /// Verifies a password and optional code. A browser delivery stages the login instead
    /// of minting a bearer session; attaching it to a browser is a separate transaction.
    #[doc(hidden)]
    pub fn password_login(
        &self,
        username: String,
        password: String,
        otp: Option<String>,
        transaction: Option<String>,
        delivery: Delivery,
    ) -> Result<Value> {
        if delivery == Delivery::Browser && transaction.is_some() {
            return Err(Error::internal("browser logins never carry a transaction"));
        }
        validate_name(&username)?;
        let password = zeroize::Zeroizing::new(password);
        if let Some(login) = self.directory_login(
            &username,
            &password,
            otp.as_deref(),
            transaction.as_deref(),
            delivery,
        )? {
            return Ok(login);
        }
        let mut verified: Option<(String, bool, Option<String>)> = None;
        // A nested result commits rate-limit and replay state even when authentication fails.
        self.store.prepared_write(|tx| {
            let at = now();
            let mut attempts = tx.get::<Attempts>("attempts", &username)?.unwrap_or_default();
            if attempts.locked_until > at {
                // A locked account costs the same hash as any other attempt.
                if verified.as_ref().is_none_or(|(previous, _, _)| previous != self.dummy_hash.as_str()) {
                    let _timer = self.store.telemetry().password.timer();
                    crypto::password_matches(&password, &self.dummy_hash);
                    verified = Some((self.dummy_hash.to_string(), false, None));
                }
                audit(tx, "anonymous", "login.locked", &username)?;
                return Ok(Err(Error::new(StatusCode::TOO_MANY_REQUESTS, "rate_limited", "Too many attempts; try again later")));
            }
            if at.saturating_sub(attempts.window_start) >= 900 { attempts = Attempts { window_start: at, ..Default::default() }; }
            let user_id = tx.get::<String>("usernames", &username)?;
            let mut user = match user_id { Some(uid) => tx.get::<User>("users", &uid)?, None => None };
            let hash = user.as_ref().map(|u| u.password_hash.as_str()).filter(|h| !h.is_empty()).unwrap_or(&self.dummy_hash);
            if verified.as_ref().is_none_or(|(previous, _, _)| previous != hash) {
                let _timer = self.store.telemetry().password.timer();
                let matches = crypto::password_matches(&password, hash);
                let upgraded = if matches { crypto::upgrade_password_hash(&password, hash)? } else { None };
                verified = Some((hash.to_owned(), matches, upgraded));
            }
            let (_, matches, upgraded) = verified.as_ref().unwrap();
            let password_ok = *matches && user.as_ref().is_some_and(|u| !u.password_hash.is_empty());
            let mut valid = password_ok && user.as_ref().is_some_and(|u| u.enabled);
            if let Some(u) = user.as_mut().filter(|_| valid)
                && let Some(secret) = &u.totp_secret {
                    if let Some(code) = otp.as_deref().filter(|c| c.starts_with("ri_recovery_")) {
                        valid = u.recovery_codes.remove(&digest(code));
                    } else {
                        let step = crypto::totp_step_with(secret, &u.username, otp.as_deref().unwrap_or(""), at, u.totp_last_step, &u.totp_settings)?;
                        valid = step.is_some();
                        if valid { u.totp_last_step = step; }
                    }
                }
            if !valid {
                attempts.failures += 1;
                if attempts.failures >= 5 { attempts.locked_until = at + 900; }
                // Bound records for unknown usernames; network-wide limits handle enumeration.
                if user.is_some() { tx.put("attempts", &username, &attempts)?; }
                audit(tx, "anonymous", "login.failed", &username)?;
                return Ok(Err(Error::new(StatusCode::UNAUTHORIZED, "invalid_credentials", "Invalid username, password, or one-time code")));
            }
            let mut u = user.unwrap();
            if let Some(hash) = upgraded.clone() {
                crate::password_history::note_rehash(
                    tx,
                    self.config.password_history,
                    &u.id,
                    &u.password_hash,
                    &hash,
                )?;
                u.password_hash = hash;
            }
            // The verified factors are spent even when the transaction binding is refused.
            tx.put("users", &u.id, &u)?;
            tx.delete("attempts", &username)?;
            let challenge = transaction.as_ref().map(|token| {
                let challenge = tx.get::<AuthenticationTransaction>("authentication", &digest(token))?
                    .filter(|c| c.expires_at > at && c.authenticated_session.is_none())
                    .ok_or_else(|| Error::bad("Authentication transaction expired or used"))?;
                if challenge.user_id.as_ref().is_some_and(|uid| uid != &u.id) { return Err(Error::forbidden()); }
                crate::oidc::reject_embedded_stage(&challenge)?;
                Ok(challenge)
            }).transpose();
            let challenge = match challenge {
                Err(error) if error.status.is_client_error() => {
                    audit(tx, &u.id, "login.transaction_rejected", &username)?;
                    return Ok(Err(error));
                }
                challenge => challenge?,
            };
            let identity = Identity { user_id: u.id.clone(), epoch: u.epoch, mfa: u.totp_secret.is_some(), auth_time: at, session_id: String::new(), amr: vec![], source: None };
            if delivery == Delivery::Browser {
                let staged = self.stage_browser_login(tx, identity, at + self.config.session_ttl, "password")?;
                audit(tx, &u.id, "login.succeeded", &staged)?;
                return Ok(Ok(json!({"staged": staged, "user": UserView::from(&u)})));
            }
            let (session, token) = self.mint_bearer_session(tx, identity, at + self.config.session_ttl)?;
            if let Some(mut challenge) = challenge {
                challenge.authenticated_session = Some(session.id.clone());
                tx.put("authentication", &digest(transaction.as_deref().unwrap()), &challenge)?;
            }
            audit(tx, &u.id, "login.succeeded", &session.id)?;
            Ok(Ok(json!({"session_token": token, "expires_at": session.expires_at, "user": UserView::from(&u)})))
        })?
    }
    /// Writes a session and its bearer token row. `identity.session_id` becomes the new id.
    pub(crate) fn mint_bearer_session(
        &self,
        tx: &Tx<'_>,
        mut identity: Identity,
        expires_at: u64,
    ) -> Result<(Session, String)> {
        let token = crypto::random_token("ri_session_");
        identity.session_id = id();
        let session = Session {
            id: identity.session_id.clone(),
            token_hash: digest(&token),
            identity,
            expires_at,
            revoked: false,
        };
        tx.put("sessions", &session.id, &session)?;
        tx.put("session_tokens", &session.token_hash, &session.id)?;
        Ok((session, token))
    }
    pub fn recover_admin(&self, username: &str, password: &str, reset_mfa: bool) -> Result<()> {
        self.store.write(|tx| {
            let mut user = user_by_name(tx, username)?;
            let hashed = crypto::password_hash(password)?;
            crate::password_history::accept(
                tx,
                self.config.password_history,
                &user.id,
                &user.password_hash,
                password,
                &hashed,
            )?;
            user.password_hash = hashed;
            user.admin = true;
            user.enabled = true;
            user.epoch += 1;
            if reset_mfa {
                crate::passkey::clear(tx, &user.id)?;
                user.has_passkeys = false;
                user.recovery_codes.clear();
                user.totp_secret = None;
                user.totp_pending = None;
                user.totp_last_step = None;
            }
            tx.put("users", &user.id, &user)?;
            crate::logout::queue_user(tx, &user.id)?;
            tx.delete("attempts", username)?;
            audit(tx, "local-recovery", "admin.recover", &user.id)
        })
    }
    pub fn me(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            if token.starts_with("ri_agent_") {
                let actor = self.principal(tx, token)?;
                return Ok(json!({"agent_id": actor.id, "permissions": actor.permissions}));
            }
            let (user, session) = self.session(tx, token)?;
            Ok(json!({"user": UserView::from(&user), "groups": groups_for(tx, &user.id)?, "session_id": session.id, "expires_at": session.expires_at, "mfa": session.identity.mfa}))
        })
    }
    pub fn logout(&self, token: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (u, mut s) = self.session(tx, token)?;
            s.revoked = true;
            tx.put("sessions", &s.id, &s)?;
            crate::logout::queue_session(tx, &s.id)?;
            crate::ssf::enqueue(tx, &u.id, crate::ssf::SESSION_REVOKED, "")?;
            audit(tx, &u.id, "session.revoke", &s.id)?;
            let propagation=crate::saml::logout::redirect(self,tx,&s.id,None)?;
            Ok(json!({"revoked": true,"saml_logout_url":propagation["redirect_uri"],"saml_logout":propagation}))
        })
    }
    pub fn sessions(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let (user, _) = self.session(tx, token)?;
            let mut list = Vec::new();
            for s in tx.list::<Session>("sessions")?.into_iter().map(|(_, v)| v).filter(|s| s.identity.user_id == user.id && !s.revoked) {
                let kind = if crate::signin::bearer_backed(tx, &s)? { "terminal" } else { "browser" };
                list.push(json!({"id": s.id, "auth_time": s.identity.auth_time, "expires_at": s.expires_at, "mfa": s.identity.mfa, "kind": kind}));
            }
            Ok(json!(list))
        })
    }
    pub fn revoke_session(&self, token: &str, sid: &str) -> Result<Value> {
        self.store.write(|tx| {
            let actor = if token.starts_with("ri_agent_") {
                self.management(tx, token, "session.revoke", &format!("session/{sid}"))?
                    .id
            } else {
                let (user, _) = self.session(tx, token)?;
                let target = tx
                    .get::<Session>("sessions", sid)?
                    .ok_or_else(|| Error::missing("Session not found"))?;
                if target.identity.user_id != user.id && !user.admin {
                    return Err(Error::forbidden());
                }
                user.id
            };
            let mut target = tx
                .get::<Session>("sessions", sid)?
                .ok_or_else(|| Error::missing("Session not found"))?;
            target.revoked = true;
            tx.put("sessions", sid, &target)?;
            crate::logout::queue_session(tx, sid)?;
            crate::ssf::enqueue(tx, &target.identity.user_id, crate::ssf::SESSION_REVOKED, "")?;
            audit(tx, &actor, "session.revoke", sid)?;
            let propagation=crate::saml::logout::redirect(self,tx,sid,None)?;
            Ok(json!({"revoked":true,"saml_logout_url":propagation["redirect_uri"],"saml_logout":propagation}))
        })
    }
    pub fn list_users(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(
                tx.list::<User>("users")?
                    .iter()
                    .filter(|(_, u)| actor.allows("user.read", &format!("user/{}", u.username)))
                    .map(|(_, u)| UserView::from(u))
                    .collect::<Vec<_>>()
            ))
        })
    }
    pub fn create_user(&self, token: &str, input: NewUser) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor =
                self.management(tx, token, "user.write", &format!("user/{}", input.username))?;
            if actor.agent && input.admin {
                return Err(Error::forbidden());
            }
            let user = make_user(input)?;
            if tx.get::<String>("usernames", &user.username)?.is_some() {
                return Err(Error::conflict("Username already exists"));
            }
            crate::password_history::record_imported_hash(
                tx,
                self.config.password_history,
                &user.id,
                "",
                &user.password_hash,
            )?;
            tx.put("users", &user.id, &user)?;
            tx.put("usernames", &user.username, &user.id)?;
            audit(tx, &actor.id, "user.create", &user.id)?;
            Ok(json!(UserView::from(&user)))
        })
    }
    pub fn update_user(&self, token: &str, username: &str, patch: UserPatch) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "user.write", &format!("user/{username}"))?;
            let mut user = user_by_name(tx, username)?;
            let previous_epoch = user.epoch;
            if actor.agent && (user.admin || patch.admin == Some(true)) {
                return Err(Error::forbidden());
            }
            if let Some(password) = patch.password {
                let hashed = crypto::password_hash(&password)?;
                crate::password_history::accept(
                    tx,
                    self.config.password_history,
                    &user.id,
                    &user.password_hash,
                    &password,
                    &hashed,
                )?;
                user.password_hash = hashed;
                user.epoch += 1;
                tx.delete("attempts", username)?;
            }
            if let Some(enabled) = patch.enabled {
                user.enabled = enabled;
                user.epoch += 1;
            }
            if let Some(admin) = patch.admin {
                user.admin = admin;
                user.epoch += 1;
            }
            if let Some(email) = patch.email {
                validate_email(&email)?;
                if user.email.as_ref() != Some(&email) {
                    user.email_verified = false;
                }
                user.email = Some(email);
            }
            if let Some(attributes) = patch.attributes {
                user.attributes = attributes;
            }
            if let Some(verified) = patch.email_verified {
                user.email_verified = verified;
            }
            if let Some(subjects) = patch.subjects {
                if user.subjects != subjects {
                    user.epoch += 1;
                }
                user.subjects = subjects;
            }
            crate::claims::validate_user(tx, &user)?;
            if let Some(name) = patch.display_name {
                validate_display(&name)?;
                user.display_name = name;
            }
            if patch.reset_mfa {
                crate::passkey::clear(tx, &user.id)?;
                user.has_passkeys = false;
                user.recovery_codes.clear();
                user.totp_secret = None;
                user.totp_pending = None;
                user.totp_last_step = None;
                user.epoch += 1;
            }
            if patch.revoke_sessions {
                user.epoch += 1;
            }
            ensure_remaining_admin(tx, &user)?;
            tx.put("users", &user.id, &user)?;
            if user.epoch != previous_epoch {
                crate::logout::queue_user(tx, &user.id)?;
            }
            if patch.revoke_sessions {
                crate::ssf::enqueue(tx, &user.id, crate::ssf::SESSION_REVOKED, "")?;
            }
            audit(tx, &actor.id, "user.update", &user.id)?;
            Ok(json!(UserView::from(&user)))
        })
    }
    pub fn list_groups(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(
                tx.list::<Group>("groups")?
                    .into_iter()
                    .filter(|(_, u)| actor.allows("group.read", &format!("group/{}", u.name)))
                    .map(|(_, g)| g)
                    .collect::<Vec<_>>()
            ))
        })
    }
    pub fn create_group(&self, token: &str, name: &str) -> Result<Value> {
        validate_name(name)?;
        self.mutation(token, |tx| {
            let user = self.management(tx, token, "group.write", &format!("group/{name}"))?;
            if tx.get::<Group>("groups", name)?.is_some() {
                return Err(Error::conflict("Group already exists"));
            }
            let group = Group {
                name: name.into(),
                members: BTreeSet::new(),
            };
            tx.put("groups", name, &group)?;
            audit(tx, &user.id, "group.create", name)?;
            Ok(json!(group))
        })
    }
    pub fn group_member(
        &self,
        token: &str,
        name: &str,
        username: &str,
        add: bool,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "group.members", &format!("group/{name}"))?;
            let user = user_by_name(tx, username)?;
            let mut group = tx
                .get::<Group>("groups", name)?
                .ok_or_else(|| Error::missing("Group not found"))?;
            if add {
                group.members.insert(user.id.clone());
            } else {
                group.members.remove(&user.id);
            }
            tx.put("groups", name, &group)?;
            audit(
                tx,
                &actor.id,
                if add {
                    "group.member.add"
                } else {
                    "group.member.remove"
                },
                &format!("{name}/{}", user.id),
            )?;
            Ok(json!(group))
        })
    }
    pub fn create_client(&self, token: &str, input: NewClient) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.management(
                tx,
                token,
                "client.write",
                &format!("client/{}", input.client_id),
            )?;
            validate_name(&input.client_id)?;
            validate_display(&input.name)?;
            if tx.get::<Client>("clients", &input.client_id)?.is_some() {
                return Err(Error::conflict("Client already exists"));
            }
            let secret = (input.settings.token_endpoint_auth_method
                != Some(crate::jose::ClientAuthMethod::PrivateKeyJwt)
                && (input.confidential || input.service))
                .then(|| crypto::random_token("ri_client_"));
            let client = Client {
                id: input.client_id,
                name: input.name,
                secret_hash: secret.as_deref().map(digest),
                redirect_uris: input.redirect_uris,
                scopes: input.scopes,
                allowed_groups: input.allowed_groups,
                require_mfa: input.require_mfa,
                enabled: true,
                service: input.service,
                settings: input.settings,
            };
            validate_client(tx, &client)?;
            tx.put("clients", &client.id, &client)?;
            audit(tx, &actor.id, "client.create", &client.id)?;
            Ok(json!({"client": client.view(), "client_secret": secret}))
        })
    }
    pub fn list_clients(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(
                tx.list::<Client>("clients")?
                    .iter()
                    .filter(|(_, u)| actor.allows("client.read", &format!("client/{}", u.id)))
                    .map(|(_, c)| c.view())
                    .collect::<Vec<_>>()
            ))
        })
    }
    pub fn update_client(&self, token: &str, cid: &str, patch: ClientPatch) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "client.write", &format!("client/{cid}"))?;
            let mut c = tx
                .get::<Client>("clients", cid)?
                .ok_or_else(|| Error::missing("Client not found"))?;
            if let Some(name) = patch.name {
                validate_display(&name)?;
                c.name = name;
            }
            if let Some(enabled) = patch.enabled {
                c.enabled = enabled;
                // Disabling then enabling a client must never resurrect existing grants.
                if !enabled {
                    revoke_client_grants(tx, cid)?;
                }
            }
            if let Some(groups) = patch.allowed_groups {
                c.allowed_groups = groups;
            }
            if let Some(required) = patch.require_mfa {
                c.require_mfa = required;
            }
            if let Some(uris) = patch.redirect_uris {
                c.redirect_uris = uris;
            }
            if let Some(scopes) = patch.scopes {
                c.scopes = scopes;
            }
            if let Some(settings) = patch.settings {
                let auth_change = c.settings.authentication_credentials_differ(&settings);
                if auth_change {
                    actor.require("client.rotate", &format!("client/{cid}"))?;
                }
                if auth_change
                    || settings.issuer != c.settings.issuer
                    || settings.pairwise_sector != c.settings.pairwise_sector
                {
                    revoke_client_grants(tx, cid)?;
                }
                if settings.token_endpoint_auth_method
                    == Some(crate::jose::ClientAuthMethod::PrivateKeyJwt)
                {
                    c.secret_hash = None;
                }
                c.settings = settings;
            }
            validate_client(tx, &c)?;
            tx.put("clients", cid, &c)?;
            audit(tx, &actor.id, "client.update", cid)?;
            Ok(c.view())
        })
    }
    pub fn rotate_client_secret(&self, token: &str, cid: &str) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "client.rotate", &format!("client/{cid}"))?;
            let mut c = tx
                .get::<Client>("clients", cid)?
                .ok_or_else(|| Error::missing("Client not found"))?;
            if c.secret_hash.is_none() {
                return Err(Error::bad("Public clients do not have a secret"));
            }
            let secret = crypto::random_token("ri_client_");
            c.secret_hash = Some(digest(&secret));
            tx.put("clients", cid, &c)?;
            revoke_client_grants(tx, cid)?;
            audit(tx, &actor.id, "client.secret.rotate", cid)?;
            Ok(json!({"client_id": cid, "client_secret": secret}))
        })
    }
    pub fn mfa_begin(&self, token: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (mut user, session) = self.session(tx, token)?;
            if session.identity.auth_time + crate::signin::FRESH_SECONDS < now() { return Err(crate::signin::reauthentication_required()); }
            require_factor_session(&user, &session)?;
            if user.totp_secret.is_some() { return Err(Error::conflict("MFA already enabled")); }
            user.totp_settings = Default::default();
            let secret = crypto::totp_secret();
            let uri = crypto::totp(&secret, &user.username)?.get_url();
            user.totp_pending = Some((secret.clone(), now() + 600));
            tx.put("users", &user.id, &user)?;
            audit(tx, &user.id, "mfa.enroll.begin", &user.id)?;
            Ok(json!({"secret": secret, "otpauth_uri": uri, "expires_in": 600, "instruction": "Store the secret in an authenticator and run riauth mfa confirm"}))
        })
    }
    pub fn mfa_confirm(&self, token: &str, code: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (mut user, _) = self.session(tx, token)?;
            let (secret, expires) = user.totp_pending.clone().ok_or_else(|| Error::bad("No pending MFA enrollment"))?;
            if expires <= now() { return Err(Error::bad("Enrollment expired")); }
            let step = crypto::totp_step(&secret, &user.username, code, now(), None)?.ok_or_else(|| Error::bad("Invalid one-time code"))?;
            user.totp_secret = Some(secret);
            user.recovery_codes.clear();
            user.totp_pending = None;
            user.totp_last_step = Some(step);
            user.epoch += 1;
            tx.put("users", &user.id, &user)?;
            crate::logout::queue_user(tx, &user.id)?;
            audit(tx, &user.id, "mfa.enabled", &user.id)?;
            Ok(json!({"mfa_enabled": true, "instruction": "All sessions revoked. Log in with a fresh one-time code."}))
        })
    }
    pub fn recovery_codes(&self, token: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (mut user, session) = self.session(tx, token)?;
            if !session.identity.mfa
                || user.totp_secret.is_none()
                || session.identity.auth_time + 300 < now()
            {
                return Err(Error::forbidden());
            }
            let codes: Vec<_> = (0..10)
                .map(|_| crypto::random_token("ri_recovery_"))
                .collect();
            user.recovery_codes = codes.iter().map(|c| digest(c)).collect();
            tx.put("users", &user.id, &user)?;
            audit(tx, &user.id, "mfa.recovery_codes.rotate", &user.id)?;
            Ok(json!({"recovery_codes": codes, "single_use": true}))
        })
    }
    pub fn change_password(
        &self,
        token: &str,
        current: String,
        password: String,
        otp: Option<String>,
    ) -> Result<Value> {
        let username = self
            .store
            .read(|tx| self.session(tx, token).map(|(u, _)| u.username))?;
        let new_hash = crypto::password_hash(&password)?;
        let fresh = self.login(username, current, otp)?;
        let reauth = fresh["session_token"]
            .as_str()
            .ok_or_else(|| Error::internal("Missing reauthentication token"))?
            .to_owned();
        let result = self.store.write(|tx| {
            let (mut user, _) = self.session(tx, token)?;
            let (verified, _) = self.session(tx, &reauth)?;
            if verified.id != user.id {
                return Err(Error::forbidden());
            }
            crate::password_history::accept(
                tx,
                self.config.password_history,
                &user.id,
                &user.password_hash,
                &password,
                &new_hash,
            )?;
            user.password_hash = new_hash;
            user.epoch += 1;
            tx.put("users", &user.id, &user)?;
            crate::logout::queue_user(tx, &user.id)?;
            audit(tx, &user.id, "user.password.change", &user.id)?;
            Ok(json!({"changed": true, "sessions_revoked": true}))
        });
        if result.is_err() {
            let _ = self.logout(&reauth);
        }
        result
    }
    pub fn audit_events(&self, token: &str, limit: usize) -> Result<Value> {
        self.store.read(|tx| {
            self.management(tx, token, "audit.read", "audit/events")?;
            Ok(json!(
                tx.scan_reverse::<Audit>("audit", None, limit.clamp(1, 1000))?
                    .into_iter()
                    .map(|(_, e)| e)
                    .collect::<Vec<_>>()
            ))
        })
    }
    pub fn rotate_key(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            self.management(tx, token, "key.rotate", "key/signing")
                .map(|_| ())
        })?;
        let replacement = SigningKey::generate()?;
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "key.rotate", "key/signing")?;
            let mut keys = keys(tx)?;
            keys.retired.retain(|key| key.expires_at > now());
            if keys.retired.len() >= 32 { return Err(Error::conflict("32 retained signing keys remain in use; wait for their retention windows before rotating")); }
            let expires_at = tx.list::<crate::logout::RpSession>("rp_sessions")?.iter().map(|(_, rp)| rp.expires_at.saturating_add(3600)).max().unwrap_or(0).max(now() + 3720);
            // Retain verification keys for recent RP logout hints as well as unexpired JWTs.
            keys.retired.push(RetiredKey {
                jwk: keys.active.jwk()?,
                expires_at,
            });
            keys.active = replacement;
            tx.put("meta", "keys", &keys)?;
            audit(tx, &actor.id, "signing_key.rotate", &keys.active.kid)?;
            Ok(json!({"kid": keys.active.kid}))
        })
    }
    pub fn cleanup(&self) -> Result<()> {
        let _timer = self.store.telemetry().cleanup.timer();
        let result = self.cleanup_pass();
        if result.is_err() {
            self.store
                .telemetry()
                .cleanup_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        result
    }
    fn cleanup_pass(&self) -> Result<()> {
        let at = now();
        self.store.write(|tx| crate::state::cleanup(tx, at))?;
        self.store
            .write(|tx| crate::authorization::cleanup(tx, at))?;
        self.store
            .write(|tx| crate::session_protocol::cleanup(tx, at))?;
        self.store.write(|tx| crate::source::cleanup(tx, at))?;
        self.store.write(|tx| crate::passkey::cleanup(tx, at))?;
        self.store
            .write(|tx| crate::windows_login::cleanup(tx, at))?;
        self.store.write(|tx| crate::lifecycle::cleanup(tx, at))?;
        self.store
            .write(|tx| crate::provisioning::cleanup(tx, at))?;
        self.store.write(|tx| crate::directory::cleanup(tx, at))?;
        self.store
            .write(|tx| crate::cloud_directory::cleanup(tx, at))?;
        self.store.write(|tx| crate::outpost::cleanup(tx, at))?;
        self.store.write(|tx| crate::radius::cleanup(tx, at))?;
        self.store.write(|tx| crate::mtls::cleanup(tx, at))?;
        self.store.write(|tx| crate::saml::cleanup(tx, at))?;
        self.store.write(crate::context::cleanup)?;
        self.store.write(|tx| crate::browser::cleanup(tx, at))?;
        self.store.write(|tx| crate::portal::cleanup(tx, at))?;
        self.store.write(|tx| crate::pam::cleanup(tx, at))?;
        self.store.write(|tx| crate::logout::cleanup(tx, at))?;
        self.store
            .write(|tx| crate::device_trust::cleanup(tx, at))?;
        self.store.write(|tx| crate::ssf::cleanup(tx, at))?;
        self.store.write(|tx| {
            for (key, (start, _)) in tx.maintenance_page::<(u64, u32)>("http_rates")? {
                if start.saturating_add(60) <= at {
                    tx.delete("http_rates", &key)?;
                }
            }
            for (key, expiry) in tx.maintenance_page::<u64>("dpop_replays")? {
                if expiry <= at {
                    tx.delete("dpop_replays", &key)?;
                }
            }
            for (key, expiry) in tx.maintenance_page::<u64>("assertion_replays")? {
                if expiry <= at {
                    tx.delete("assertion_replays", &key)?;
                }
            }
            // The monotonic retention index includes every access/refresh grant.
            // Deleting a grant may retain a session longer, never less than needed.
            for (k, v) in tx.maintenance_page::<Session>("sessions")? {
                if tx
                    .get::<u64>("session_retention", &k)?
                    .is_some_and(|expiry| expiry >= at)
                {
                    continue;
                }
                if v.expires_at
                    .saturating_add(self.config.refresh_token_ttl.max(3600))
                    < at
                {
                    tx.delete("session_tokens", &v.token_hash)?;
                    tx.delete("sessions", &k)?;
                    tx.delete("session_retention", &k)?;
                }
            }
            for (k, v) in tx.maintenance_page::<Code>("codes")? {
                if v.expires_at < at {
                    tx.delete("codes", &k)?;
                }
            }
            for (k, v) in tx.maintenance_page::<AuthenticationTransaction>("authentication")? {
                if v.expires_at < at {
                    tx.delete("authentication", &k)?;
                }
            }
            for (k, v) in tx.maintenance_page::<Device>("devices")? {
                if v.expires_at + 60 < at {
                    tx.delete("device_users", &v.user_code_hash)?;
                    tx.delete("devices", &k)?;
                }
            }
            for bucket in ["access", "refresh"] {
                for (k, v) in tx.maintenance_page::<Grant>(bucket)? {
                    if v.expires_at < at {
                        tx.delete(bucket, &k)?;
                    }
                }
            }
            for (k, v) in tx.maintenance_page::<Family>("families")? {
                if v.expires_at < at {
                    tx.delete("families", &k)?;
                }
            }
            for (k, v) in tx.maintenance_page::<Attempts>("attempts")? {
                if v.window_start + 1800 < at {
                    tx.delete("attempts", &k)?;
                }
            }
            for (k, v) in tx.maintenance_page::<Audit>("audit")? {
                // Keep an event for 90 days. Delete only once age is strictly greater.
                if v.at.saturating_add(AUDIT_RETENTION_SECONDS) < at {
                    tx.delete("audit", &k)?;
                }
            }
            Ok(())
        })?;
        crate::offboarding::cleanup(self)
    }
    pub(crate) fn session(&self, tx: &Tx<'_>, token: &str) -> Result<(User, Session)> {
        let sid = tx
            .get::<String>("session_tokens", &digest(token))?
            .ok_or_else(Error::unauthorized)?;
        let session = tx
            .get::<Session>("sessions", &sid)?
            .ok_or_else(Error::unauthorized)?;
        if session.revoked || session.expires_at <= now() {
            return Err(Error::unauthorized());
        }
        let user = self.identity_user(tx, &session.identity)?;
        Ok((user, session))
    }
    pub(crate) fn identity_user(&self, tx: &Tx<'_>, identity: &Identity) -> Result<User> {
        let user = self.identity_user_unbound(tx, identity)?;
        let session = tx
            .get::<Session>("sessions", &identity.session_id)?
            .ok_or_else(Error::unauthorized)?;
        if session.revoked || session.identity.user_id != user.id {
            return Err(Error::unauthorized());
        }
        Ok(user)
    }
    /// Every identity check except the session row, for identities not yet bound to a session.
    pub(crate) fn identity_user_unbound(&self, tx: &Tx<'_>, identity: &Identity) -> Result<User> {
        crate::radius::eap::validate_identity(self, tx, identity)?;
        crate::mtls::validate_identity(tx, identity)?;
        crate::directory::validate_identity(self, tx, identity)?;
        crate::source::validate_identity(tx, identity)?;
        let user = tx
            .get::<User>("users", &identity.user_id)?
            .ok_or_else(Error::unauthorized)?;
        if !user.enabled || user.epoch != identity.epoch {
            return Err(Error::unauthorized());
        }
        Ok(user)
    }
    pub(crate) fn dummy_password_hash(&self) -> &str {
        &self.dummy_hash
    }
    pub(crate) fn admin(&self, tx: &Tx<'_>, token: &str) -> Result<User> {
        let (user, _) = self.session(tx, token)?;
        if !user.admin {
            return Err(Error::forbidden());
        }
        Ok(user)
    }
    pub(crate) fn authorize_identity(
        &self,
        tx: &Tx<'_>,
        c: &Client,
        identity: &Identity,
    ) -> Result<User> {
        let user = self.identity_user(tx, identity)?;
        if !c.enabled || c.service || c.require_mfa && !identity.mfa {
            return Err(Error::forbidden());
        }
        if !c.allowed_groups.is_empty() && c.allowed_groups.is_disjoint(&groups_for(tx, &user.id)?)
        {
            return Err(Error::forbidden());
        }
        crate::claims::enforce(tx, c, &user, identity, &BTreeSet::new())?;
        crate::device_trust::require(self, tx, c, identity)?;
        Ok(user)
    }
}

pub(crate) fn keys(tx: &Tx<'_>) -> Result<Keys> {
    tx.get("meta", "keys")?
        .ok_or_else(|| Error::internal("signing keys missing"))
}
/// Ninety days. An audit row is removed only when `at + AUDIT_RETENTION_SECONDS < now`.
pub(crate) const AUDIT_RETENTION_SECONDS: u64 = 90 * 24 * 60 * 60;

pub(crate) fn audit(tx: &Tx<'_>, actor: &str, action: &str, target: &str) -> Result<()> {
    if [
        "user.",
        "group.",
        "client.",
        "registration.",
        "source.configure",
        "source.reconcile",
        "source_link.reconcile",
        "agent.",
        "access.",
        "signing_key.",
        "admin.recover",
        "directory.apply",
        "cloud_directory.apply",
        "certificate.",
        "offboard.",
        "device.",
        "mtls.",
    ]
    .iter()
    .any(|prefix| action.starts_with(prefix))
    {
        let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
        tx.put("meta", "revision", &revision.saturating_add(1))?;
    }
    let mut details = json!({
        "request_id": crate::context::current().map(|c| c.request_id),
        "changes": tx.changes(),
    });
    crate::store::redact_audit_value(&mut details);
    if let Some(parent) = crate::agent::audit_parent(tx, actor, action, target)? {
        details["parent_user"] = json!(parent);
    }
    let event = Audit {
        id: id(),
        at: now(),
        actor: actor.into(),
        action: action.into(),
        target: target.into(),
        run_id: crate::context::current().and_then(|c| c.run_id),
        details,
    };
    tx.put("audit", &format!("{:020}-{}", event.at, event.id), &event)
}
pub(crate) fn groups_for(tx: &Tx<'_>, uid: &str) -> Result<BTreeSet<String>> {
    let mut names = durable_groups_for(tx, uid)?;
    // Temporary entitlements affect authorization, never directory projections.
    names.extend(crate::pam::extra_groups(tx, uid, now())?);
    Ok(names)
}

pub(crate) fn durable_groups_for(tx: &Tx<'_>, uid: &str) -> Result<BTreeSet<String>> {
    Ok(tx
        .list::<Group>("groups")?
        .into_iter()
        .filter(|(_, group)| group.members.contains(uid))
        .map(|(_, group)| group.name)
        .collect())
}
/// Reject a write that would leave the instance without an enabled administrator.
/// `user` is the record as it would be stored.
pub(crate) fn ensure_remaining_admin(tx: &Tx<'_>, user: &User) -> Result<()> {
    let another_admin = tx
        .list::<User>("users")?
        .iter()
        .any(|(_, candidate)| candidate.id != user.id && candidate.admin && candidate.enabled);
    if !(another_admin || user.enabled && user.admin) {
        return Err(Error::conflict(
            "Cannot remove the last enabled administrator",
        ));
    }
    Ok(())
}
pub(crate) fn user_by_name(tx: &Tx<'_>, username: &str) -> Result<User> {
    let uid = tx
        .get::<String>("usernames", username)?
        .ok_or_else(|| Error::missing("User not found"))?;
    tx.get("users", &uid)?
        .ok_or_else(|| Error::missing("User not found"))
}
/// Atomic account revocation and credential signals shared by every user writer.
pub(crate) fn user_security_transition(
    tx: &Tx<'_>,
    user_id: &str,
    before: &Value,
    after: Option<&Value>,
) -> Result<()> {
    let disabled = after.is_none_or(|user| user["enabled"] == false);
    // Old snapshots may contain disabled parents whose children were never
    // revoked. Re-enabling must repair those credentials before enabling use.
    if disabled || before["enabled"] == false {
        crate::agent::revoke_owned(tx, user_id)?;
        crate::windows_login::revoke_user(tx, user_id)?;
        crate::logout::queue_user(tx, user_id)?;
        if disabled && before["enabled"] == true {
            crate::ssf::enqueue(tx, user_id, crate::ssf::ACCOUNT_DISABLED, "")?;
        }
    }
    let Some(after) = after else {
        return Ok(());
    };
    // A successful password login may transparently rehash the same credential.
    // Credential replacement paths bump the epoch; rehashes keep it unchanged.
    if before["password_hash"] != after["password_hash"] && before["epoch"] != after["epoch"] {
        crate::ssf::enqueue(tx, user_id, crate::ssf::CREDENTIAL_CHANGE, "password")?;
    }
    if before["totp_secret"] != after["totp_secret"]
        || before["totp_settings"] != after["totp_settings"] && !after["totp_secret"].is_null()
    {
        crate::ssf::enqueue(tx, user_id, crate::ssf::CREDENTIAL_CHANGE, "otp")?;
    }
    // Consuming a recovery code is authentication, whereas adding new codes is rotation.
    if after["recovery_codes"].as_array().is_some_and(|codes| {
        codes.iter().any(|code| {
            before["recovery_codes"]
                .as_array()
                .is_none_or(|old| !old.contains(code))
        })
    }) {
        crate::ssf::enqueue(tx, user_id, crate::ssf::CREDENTIAL_CHANGE, "recovery-code")?;
    }
    Ok(())
}
/// Changing a factor needs an MFA session once the user has TOTP or a passkey.
#[doc(hidden)]
pub fn require_factor_session(user: &User, session: &Session) -> Result<()> {
    if (user.totp_secret.is_some() || user.has_passkeys) && !session.identity.mfa {
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "mfa_required",
            "Sign in with your passkey or authenticator code first",
        ));
    }
    Ok(())
}
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.len() > 64
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.@".contains(&c))
    {
        return Err(Error::bad(
            "Names must be 1–64 ASCII letters, digits, dots, hyphens, underscores or @",
        ));
    }
    Ok(())
}
pub(crate) fn validate_display(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 200 || name.chars().any(char::is_control) {
        return Err(Error::bad(
            "Display name must be 1–200 bytes without control characters",
        ));
    }
    Ok(())
}
pub(crate) fn validate_email(email: &str) -> Result<()> {
    if email.len() > 254
        || !email.contains('@')
        || email.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        return Err(Error::bad("Invalid email address"));
    }
    Ok(())
}
pub(crate) fn make_user(input: NewUser) -> Result<User> {
    validate_name(&input.username)?;
    if let Some(email) = &input.email {
        validate_email(email)?;
    }
    let name = if input.display_name.is_empty() {
        input.username.clone()
    } else {
        input.display_name
    };
    validate_display(&name)?;
    let password = zeroize::Zeroizing::new(input.password);
    Ok(User {
        has_passkeys: false,
        totp_settings: Default::default(),
        pairwise_seed: crypto::random_token(""),
        id: id(),
        username: input.username,
        email: input.email,
        display_name: name,
        password_hash: crypto::password_hash(&password)?,
        enabled: true,
        admin: input.admin,
        epoch: 0,
        totp_secret: None,
        totp_pending: None,
        totp_last_step: None,
        created_at: now(),
        attributes: Default::default(),
        email_verified: false,
        subjects: Default::default(),
        recovery_codes: Default::default(),
    })
}
pub(crate) fn validate_client(tx: &Tx<'_>, c: &Client) -> Result<()> {
    if let Some(id) = &c.settings.signing_key {
        validate_name(id)?;
        crate::keyring::for_client(tx, c)?;
    }
    crate::issuer::validate(tx, c)?;
    crate::provider::validate_settings(c)?;
    crate::saml::validate_key(tx, c)?;
    crate::claims::validate_mappings(c)?;
    for rule in std::iter::once(&c.settings.policy.access).chain(c.settings.policy.scopes.values())
    {
        for group in rule
            .all_groups
            .iter()
            .chain(&rule.any_groups)
            .chain(&rule.denied_groups)
        {
            if tx.get::<Group>("groups", group)?.is_none() {
                return Err(Error::bad(format!("Unknown policy group: {group}")));
            }
        }
        for username in rule.users.iter().chain(&rule.denied_users) {
            if tx.get::<String>("usernames", username)?.is_none() {
                return Err(Error::bad(format!("Unknown policy user: {username}")));
            }
        }
    }
    if c.scopes.is_empty()
        || c.scopes.len() > 32
        || c.scopes.iter().any(|s| {
            s.is_empty()
                || s.len() > 64
                || !s
                    .bytes()
                    .all(|b| matches!(b, 0x21 | 0x23..=0x5b | 0x5d..=0x7e))
        })
    {
        return Err(Error::bad(
            "Provide 1–32 valid OAuth scopes, up to 64 bytes each",
        ));
    }
    if !c.service && !c.scopes.contains("openid") {
        return Err(Error::bad("OIDC clients must allow openid"));
    }
    if c.service
        && (c.scopes.iter().any(|s| {
            [
                "openid",
                "profile",
                "email",
                "groups",
                "offline_access",
                "bound_key",
            ]
            .contains(&s.as_str())
        }) || !c.allowed_groups.is_empty()
            || c.require_mfa)
    {
        return Err(Error::bad(
            "Service clients use API scopes and cannot use user groups, MFA or OIDC identity scopes",
        ));
    }
    if c.redirect_uris.len() > 32 {
        return Err(Error::bad("At most 32 redirect URIs are allowed"));
    }
    if c.settings.implicit_consent
        && (c.service
            || c.settings.native
            || !(c.confidential() || c.settings.proxy.is_some() || c.settings.saml.is_some()))
    {
        return Err(Error::bad(
            "implicit_consent requires a confidential, proxy or SAML client",
        ));
    }
    for uri in &c.redirect_uris {
        let url = url::Url::parse(uri).map_err(|_| Error::bad("Invalid redirect URI"))?;
        let local = matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
        let private_scheme = c.settings.native && crate::provider::private_scheme(&url);
        if uri.len() > 2048
            || !(url.scheme() == "https" || url.scheme() == "http" && local || private_scheme)
            || (url.host_str().is_none() && !private_scheme)
            || url.fragment().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
            || uri.contains('*')
            || url.query_pairs().any(|(k, _)| {
                [
                    "code",
                    "state",
                    "iss",
                    "error",
                    "error_description",
                    "response",
                    "session_state",
                ]
                .contains(&k.as_ref())
            })
        {
            return Err(Error::bad(
                "Redirect URIs must be exact HTTPS URLs (HTTP loopback allowed), without credentials, fragments, wildcards or reserved response parameters",
            ));
        }
    }
    for group in c
        .allowed_groups
        .iter()
        .chain(c.settings.ldap.iter().flat_map(|l| l.search_groups.iter()))
    {
        if tx.get::<Group>("groups", group)?.is_none() {
            return Err(Error::bad(format!("Unknown group: {group}")));
        }
    }
    Ok(())
}
pub(crate) fn revoke_client_grants(tx: &Tx<'_>, cid: &str) -> Result<()> {
    crate::logout::queue_client(tx, cid)?;
    for bucket in ["access", "refresh"] {
        for (_, grant) in tx.list::<Grant>(bucket)? {
            if grant.client_id == cid
                && let Some(mut family) = tx.get::<Family>("families", &grant.family_id)?
            {
                family.revoked = true;
                tx.put("families", &grant.family_id, &family)?;
            }
        }
    }
    for (key, code) in tx.list::<Code>("codes")? {
        if code.client_id == cid {
            tx.delete("codes", &key)?;
        }
    }
    for (key, device) in tx.list::<Device>("devices")? {
        if device.client_id == cid {
            tx.delete("devices", &key)?;
            tx.delete("device_users", &device.user_code_hash)?;
        }
    }
    Ok(())
}
