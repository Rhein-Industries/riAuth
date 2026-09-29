use crate::{
    config::Config,
    crypto::{self, Keys, SigningKey, digest, id, now},
    error::{Error, Result},
    model::*,
    store::{Store, Tx},
};
use axum::http::StatusCode;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

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
    pub(crate) runtime: Arc<crate::capability::RuntimeStatus>,
    #[cfg(feature = "platform")]
    pub(crate) verified_access_cache: Arc<Mutex<crate::device_trust::TokenCache>>,
    #[cfg(all(feature = "platform", feature = "test-support"))]
    pub(crate) verified_access_transport:
        Arc<Mutex<Option<Arc<dyn crate::device_trust::VerifiedAccessTransport>>>>,
}

impl Core {
    /// Apply a management mutation and its retry receipt in one transaction.
    pub(crate) fn mutation(
        &self,
        token: &str,
        f: impl FnOnce(&Tx<'_>) -> Result<Value>,
    ) -> Result<Value> {
        self.mutation_checked(
            token,
            |tx, actor, context| {
                if let Some(c) = context {
                    if (actor.agent || actor.delegated) && c.revision.is_none() {
                        return Err(Error::new(
                            StatusCode::PRECONDITION_REQUIRED,
                            "precondition_required",
                            "Scoped mutations require If-Match with the current revision",
                        ));
                    }
                    let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
                    if c.revision.is_some_and(|r| r != revision) {
                        return Err(Error::conflict("Configuration revision changed"));
                    }
                }
                Ok(())
            },
            f,
        )
    }

    /// Use the same authorization, idempotency receipt, and atomic writer with
    /// a protocol-specific precondition. The check runs after receipt replay.
    pub(crate) fn mutation_checked(
        &self,
        token: &str,
        check: impl FnOnce(
            &Tx<'_>,
            &crate::agent::Principal,
            Option<&crate::context::RequestContext>,
        ) -> Result<()>,
        f: impl FnOnce(&Tx<'_>) -> Result<Value>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let actor = self.principal(tx, token)?;
            let context = crate::context::current();
            let receipt_key = context
                .as_ref()
                .and_then(|c| c.idempotency_key.as_ref())
                .map(|k| digest(&format!("{}\0{k}", actor.id)));
            // Keep the established agent/admin receipt representation stable.
            // A delegated human's current grants form a separate replay scope.
            let permissions = if actor.delegated {
                json!({"human_grants": actor.grants, "generation": tx.get::<u64>("human_grant_generations", &actor.id)?.unwrap_or(0)})
            } else {
                serde_json::to_value(&actor.permissions).map_err(Error::internal)?
            };
            if let Some(key) = &receipt_key
                && let Some(result) = crate::context::replay_receipt(
                    tx,
                    key,
                    &context.as_ref().unwrap().fingerprint,
                    &permissions,
                )?
            {
                return Ok(result);
            }
            check(tx, &actor, context.as_ref())?;
            let result = f(tx)?;
            if let Some(key) = receipt_key {
                crate::context::save_receipt(
                    tx,
                    &key,
                    context.unwrap().fingerprint,
                    permissions,
                    &result,
                )?;
            }
            Ok(result)
        })
    }
}

impl Core {
    pub fn initialize(config: Config, input: NewUser) -> Result<Self> {
        config.validate().map_err(Error::internal)?;
        crate::config::private_dir(&config.data_dir).map_err(Error::internal)?;
        let store = Store::from_config(&config)?;
        Self::initialize_store(config, store, input, |_| Ok(()))
    }

    /// All first-administrator paths use the same transaction and credential policy.
    pub(crate) fn initialize_store(
        config: Config,
        store: Store,
        input: NewUser,
        ownership: impl FnOnce(&Tx<'_>) -> Result<()>,
    ) -> Result<Self> {
        if store.get::<u32>("meta", "schema")?.is_some() {
            return Err(Error::conflict("Instance already initialized"));
        }
        let mut user = make_user(input)?;
        user.admin = true;
        Self::initialize_with_administrator(config, store, |tx| {
            ownership(tx)?;
            Ok(user)
        })
    }

    /// Shared initialization commit for password and verified passkey administrators.
    /// The closure must recheck ownership and persist any credentials in this writer.
    pub(crate) fn initialize_with_administrator(
        config: Config,
        store: Store,
        administrator: impl FnOnce(&Tx<'_>) -> Result<User>,
    ) -> Result<Self> {
        if store.get::<u32>("meta", "schema")?.is_some() {
            return Err(Error::conflict("Instance already initialized"));
        }
        let key = SigningKey::generate()?;
        let dummy = crypto::password_hash(&crypto::random_token(""))?;
        store.write(|tx| {
            if tx.get::<u32>("meta", "schema")?.is_some() {
                return Err(Error::conflict("Instance already initialized"));
            }
            let user = administrator(tx)?;
            tx.put("meta", "schema", &crate::upgrade::SCHEMA)?;
            tx.put(
                "meta",
                "index_version",
                &crate::store::maintenance::INDEX_VERSION,
            )?;
            tx.put("meta", "issuer", &config.issuer)?;
            crate::node_security::stamp(&config, tx)?;
            tx.put(
                "meta",
                "keys",
                &Keys {
                    active: key,
                    retired: vec![],
                },
            )?;
            tx.put("meta", "dummy_hash", &dummy)?;
            crate::assembly::stamp_prepared_index(tx)?;
            crate::identity::password_history::record_imported_hash(
                tx,
                config.password_history,
                &user.id,
                "",
                &user.password_hash,
            )?;
            tx.put("users", &user.id, &user)?;
            crate::delegation::record_elevation_provenance(
                tx,
                &user,
                crate::delegation::ProvenanceBasis::Bootstrap,
            )?;
            tx.put("usernames", &user.username, &user.id)?;
            tx.delete("meta", "browser_setup")?;
            tx.delete("meta", "browser_setup_passkeys")?;
            crate::recovery::stamp_lineage(tx)?;
            crate::upgrade::stamp_initial(tx)?;
            crate::edition::stamp_activation(&config, tx)?;
            audit(tx, "bootstrap", "instance.initialize", &user.username)
        })?;
        Ok(Self {
            config,
            store,
            dummy_hash: Arc::new(dummy),
            runtime: Arc::default(),
            #[cfg(feature = "platform")]
            verified_access_cache: Arc::new(Mutex::new(crate::device_trust::TokenCache::default())),
            #[cfg(all(feature = "platform", feature = "test-support"))]
            verified_access_transport: Arc::new(Mutex::new(None)),
        })
    }
    pub fn open(config: Config) -> Result<Self> {
        config.validate().map_err(Error::internal)?;
        if config.postgres.is_none() && !config.data_dir.join("riauth.redb").is_file() {
            return Err(Error::missing("Database missing; run riauth init"));
        }
        let store = Store::from_config(&config)?;
        Self::open_store(config, store)
    }

    pub(crate) fn open_store(config: Config, store: Store) -> Result<Self> {
        if store.get::<String>("meta", "issuer")?.as_deref() != Some(&config.issuer) {
            return Err(Error::bad(
                "Configured issuer does not match the initialized instance",
            ));
        }
        // An existing agreement is compared before any startup write. A missing
        // row is recorded only after the read-only edition and capability gates,
        // so a refused build does not become canonical.
        crate::node_security::enforce(&config, &store)?;
        // Edition compatibility is read-only and must run before either migration
        // or restored-lineage reconciliation mutates shared state.
        crate::edition::validate_store(&store)?;
        crate::capability::validate_store(&config, &store)?;
        crate::node_security::adopt_if_absent(&config, &store)?;
        crate::upgrade::migrate(&store)?;
        crate::recovery::verify_lineage(&store)?;
        crate::context::scrub_legacy_agent_receipts_on_open(&store)?;
        store.write(crate::assembly::backfill_prepared_index)?;
        let dummy = store
            .get::<String>("meta", "dummy_hash")?
            .ok_or_else(|| Error::internal("dummy hash missing"))?;
        store.write(|tx| crate::edition::stamp_activation(&config, tx))?;
        Ok(Self {
            config,
            store,
            dummy_hash: Arc::new(dummy),
            runtime: Arc::default(),
            #[cfg(feature = "platform")]
            verified_access_cache: Arc::new(Mutex::new(crate::device_trust::TokenCache::default())),
            #[cfg(all(feature = "platform", feature = "test-support"))]
            verified_access_transport: Arc::new(Mutex::new(None)),
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
            if let Some(u) = user.as_mut().filter(|_| valid) {
                valid = crate::authenticator::consume_password_factor(u, otp.as_deref(), at)?;
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
                crate::identity::password_history::note_rehash(
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
            if tx.postgres_other_clients()?.is_some_and(|count| count > 0) {
                return Err(Error::conflict("Stop every riAuth process connected to this database before administrator recovery"));
            }
            let mut user = user_by_name(tx, username)?;
            let credential_exposed = crate::delegation::credential_exposure(tx, &user.id)?.is_some();
            let legacy_unproven = !user.admin
                && !crate::delegation::proven_for_elevation(tx, &user)?;
            if (credential_exposed || legacy_unproven) && !reset_mfa {
                return Err(Error::conflict(
                    "Unproven or operator-exposed credentials require offline recovery with factor reset",
                ));
            }
            if user.password_hash.is_empty() && user.totp_secret.is_none() && !reset_mfa {
                return Err(Error::conflict("Passkey-only recovery requires explicit --reset-mfa; enrolled factors will be removed"));
            }
            let hashed = crypto::password_hash(password)?;
            crate::identity::password_history::accept(
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
                #[cfg(feature = "platform")]
                crate::assembly::clear_user_binding(tx, &user.id)?;
                user.has_passkeys = false;
                user.recovery_codes.clear();
                user.totp_secret = None;
                user.totp_pending = None;
                user.totp_last_step = None;
            }
            if credential_exposed || legacy_unproven {
                // A legacy address may have been set by an agent before
                // exposure tracking. Do not use it for recovery after elevation.
                user.email = None;
                user.email_verified = false;
                tx.delete(crate::delegation::CREDENTIAL_EXPOSURE, &user.id)?;
            }
            if reset_mfa {
                crate::delegation::record_elevation_provenance(
                    tx,
                    &user,
                    crate::delegation::ProvenanceBasis::OfflineRecovery,
                )?;
            }
            tx.put("users", &user.id, &user)?;
            crate::logout::queue_user(tx, &user.id)?;
            tx.delete("attempts", username)?;
            audit(tx, "local-recovery", if reset_mfa { "admin.recover.factors_reset" } else { "admin.recover" }, &user.id)
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
            crate::management::revoke_sessions(
                self,
                tx,
                crate::management::RevokeIntent::Logout { token },
            )
            .map(|outcome| outcome.body)
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
            crate::management::revoke_sessions(
                self,
                tx,
                crate::management::RevokeIntent::BearerOne {
                    token,
                    target_id: sid,
                },
            )
            .map(|outcome| outcome.body)
        })
    }
    pub fn list_users(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            // Keep the returned array in key order without also retaining the
            // complete User bucket and a second array of typed views.
            let mut users = Vec::new();
            let mut after = None;
            loop {
                let page =
                    tx.scan::<User>("users", after.as_deref(), crate::store::maintenance::PAGE)?;
                if page.is_empty() {
                    break;
                }
                let full = page.len() == crate::store::maintenance::PAGE;
                after = page.last().map(|(key, _)| key.clone());
                for (_, user) in page {
                    if actor.allows("user.read", &format!("user/{}", user.username)) {
                        users.push(json!(UserView::from(&user)));
                    }
                }
                if !full {
                    break;
                }
            }
            Ok(Value::Array(users))
        })
    }
    pub fn create_user(&self, token: &str, input: NewUser) -> Result<Value> {
        if let Some(context) = crate::context::current()
            && (context.idempotency_key.is_none() || context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "User creation requires Idempotency-Key and If-Match",
            ));
        }
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            crate::management::create_user(&self.config, tx, &actor, input)
        })
    }
    pub fn update_user(&self, token: &str, username: &str, patch: UserPatch) -> Result<Value> {
        if let Some(context) = crate::context::current()
            && (context.idempotency_key.is_none() || context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "User update requires Idempotency-Key and If-Match",
            ));
        }
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            crate::management::update_user(&self.config, tx, &actor, username, patch)
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
    fn require_group_retry_binding() -> Result<()> {
        if let Some(context) = crate::context::current()
            && (context.idempotency_key.is_none() || context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "Group writes require Idempotency-Key and If-Match",
            ));
        }
        Ok(())
    }
    pub fn create_group(&self, token: &str, name: &str) -> Result<Value> {
        Self::require_group_retry_binding()?;
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "group.write", &format!("group/{name}"))?;
            let written = crate::management::write_group(
                &self.config,
                tx,
                &actor,
                name,
                crate::management::GroupIntent::Create(&BTreeSet::new()),
                crate::management::GroupAudit::OnChange {
                    action: "group.create",
                    target: name,
                },
            )?;
            Ok(json!(written.group))
        })
    }
    pub fn group_member(
        &self,
        token: &str,
        name: &str,
        username: &str,
        add: bool,
    ) -> Result<Value> {
        Self::require_group_retry_binding()?;
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "group.members", &format!("group/{name}"))?;
            let user = user_by_name(tx, username)?;
            let written = crate::management::write_group(
                &self.config,
                tx,
                &actor,
                name,
                crate::management::GroupIntent::Member {
                    user_id: &user.id,
                    present: add,
                },
                crate::management::GroupAudit::OnChange {
                    action: if add {
                        "group.member.add"
                    } else {
                        "group.member.remove"
                    },
                    target: &format!("{name}/{}", user.id),
                },
            )?;
            Ok(json!(written.group))
        })
    }
    pub fn create_client(&self, token: &str, input: NewClient) -> Result<Value> {
        Self::require_client_retry_binding()?;
        self.mutation(token, |tx| {
            let actor = self.management(
                tx,
                token,
                "client.write",
                &format!("client/{}", input.client_id),
            )?;
            let (client, secret) = crate::management::new_client(input);
            let written = crate::management::write_client(
                tx,
                &self.config,
                &actor,
                None,
                client,
                secret,
                crate::management::Record::Direct("client.create"),
            )?;
            Ok(json!({"client": written.client.view(), "client_secret": written.secret}))
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
        Self::require_client_retry_binding()?;
        self.mutation(token, |tx| {
            let principal = self.principal(tx, token)?;
            let action = if principal.delegated {
                "client.owner_update"
            } else {
                "client.write"
            };
            let actor = self.management(tx, token, action, &format!("client/{cid}"))?;
            if actor.delegated
                && (patch.enabled.is_some()
                    || patch.allowed_groups.is_some()
                    || patch.require_mfa.is_some()
                    || patch.scopes.is_some())
            {
                return Err(Error::forbidden());
            }
            let existing = tx
                .get::<Client>("clients", cid)?
                .ok_or_else(|| Error::missing("Client not found"))?;
            let mut c = existing.clone();
            if let Some(name) = patch.name {
                c.name = name;
            }
            if let Some(enabled) = patch.enabled {
                c.enabled = enabled;
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
                c.settings = settings;
            }
            let written = crate::management::write_client(
                tx,
                &self.config,
                &actor,
                Some(&existing),
                c,
                crate::management::Secret::Keep,
                crate::management::Record::Direct("client.update"),
            )?;
            Ok(written.client.view())
        })
    }
    fn require_client_retry_binding() -> Result<()> {
        if let Some(context) = crate::context::current()
            && (context.idempotency_key.is_none() || context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "Client writes require Idempotency-Key and If-Match",
            ));
        }
        Ok(())
    }
    pub fn rotate_client_secret(&self, token: &str, cid: &str) -> Result<Value> {
        if let Some(context) = crate::context::current()
            && (context.idempotency_key.is_none() || context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "Client secret rotation requires Idempotency-Key and If-Match",
            ));
        }
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "client.rotate", &format!("client/{cid}"))?;
            let existing = tx
                .get::<Client>("clients", cid)?
                .ok_or_else(|| Error::missing("Client not found"))?;
            if existing.secret_hash.is_none() {
                return Err(Error::bad("Public clients do not have a secret"));
            }
            let written = crate::management::write_client(
                tx,
                &self.config,
                &actor,
                Some(&existing),
                existing.clone(),
                crate::management::Secret::Issue,
                crate::management::Record::Direct("client.secret.rotate"),
            )?;
            Ok(json!({"client_id": cid, "client_secret": written.secret}))
        })
    }
    /// Starts TOTP enrollment for this session (`authenticator::totp_start_in`).
    pub fn mfa_begin(&self, token: &str) -> Result<Value> {
        self.mfa_start(token, false)
    }
    /// Starts replacing the enabled authenticator app; `mfa_confirm` finishes it.
    pub fn mfa_replace(&self, token: &str) -> Result<Value> {
        self.mfa_start(token, true)
    }
    fn mfa_start(&self, token: &str, replace: bool) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            let mut started = self.totp_start_in(tx, user, &session, replace)?;
            started["instruction"] =
                json!("Store the secret in an authenticator and run riauth mfa confirm");
            Ok(started)
        })
    }
    /// Confirms the enrollment this session started; recovery codes are rotated separately.
    pub fn mfa_confirm(&self, token: &str, code: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            let mut confirmed = self.totp_confirm_in(tx, user, &session, code, false)?;
            confirmed["instruction"] =
                json!("All sessions revoked. Log in with a fresh one-time code.");
            Ok(confirmed)
        })
    }
    pub fn mfa_remove(&self, token: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            self.totp_remove_in(tx, user, &session)
        })
    }
    pub fn recovery_codes(&self, token: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            self.recovery_codes_in(tx, user, &session)
        })
    }
    pub fn change_password(
        &self,
        token: &str,
        current: String,
        password: String,
        otp: Option<String>,
    ) -> Result<Value> {
        let current = zeroize::Zeroizing::new(current);
        let password = zeroize::Zeroizing::new(password);
        let otp = otp.map(zeroize::Zeroizing::new);
        if current.len() > 1024 || otp.as_ref().is_some_and(|code| code.len() > 128) {
            return Err(Error::bad("Password or code is too long"));
        }
        // Pin this request before hashing. No verification session or factor
        // consumption may survive independently of the credential mutation.
        let (pinned, pinned_session) = self.store.read(|tx| {
            let (user, session) = self.session(tx, token)?;
            crate::password::require_local(tx, &user)?;
            // Enrolled TOTP will be proved by this request inside the writer.
            crate::password::require_fresh_mfa(&user, &session, user.totp_secret.is_some())?;
            crate::password::unlocked(tx, &user)??;
            Ok((user, session))
        })?;
        let new_hash = crypto::password_hash(&password)?;
        let password_ok = {
            let _timer = self.store.telemetry().password.timer();
            crypto::password_matches(&current, &pinned.password_hash)
        };
        self.store.write(|tx| {
            let (mut user, session) = self.session(tx, token)?;
            if user.id != pinned.id
                || user.epoch != pinned.epoch
                || user.username != pinned.username
                || user.password_hash != pinned.password_hash
                || session.id != pinned_session.id
                || session.identity.session_id != session.id
                || session.token_hash != digest(token)
            {
                return Err(Error::forbidden());
            }
            crate::password::require_local(tx, &user)?;
            let factor_required = user.totp_secret.is_some();
            // A passkey-only MFA requirement still needs this session's fresh
            // passkey assurance. A submitted recovery code keeps its existing
            // password-change permission and cannot upgrade the stored session.
            crate::password::require_fresh_mfa(&user, &session, factor_required)?;
            if let Err(locked) = crate::password::unlocked(tx, &user)? {
                return Ok(Err(locked));
            }
            let at = now();
            if !password_ok
                || !crate::authenticator::consume_password_factor(
                    &mut user,
                    otp.as_deref().map(String::as_str),
                    at,
                )?
            {
                let mut attempts = tx
                    .get::<Attempts>("attempts", &user.username)?
                    .unwrap_or_default();
                if at.saturating_sub(attempts.window_start) >= 900 {
                    attempts = Attempts {
                        window_start: at,
                        ..Default::default()
                    };
                }
                attempts.failures += 1;
                if attempts.failures >= 5 {
                    attempts.locked_until = at + 900;
                }
                tx.put("attempts", &user.username, &attempts)?;
                audit(tx, "anonymous", "login.failed", &user.username)?;
                // Failed verification commits only the shared guessing budget.
                return Ok(Err(Error::new(
                    StatusCode::UNAUTHORIZED,
                    "invalid_credentials",
                    "Invalid username, password, or one-time code",
                )));
            }
            crate::password::replace(
                tx,
                self.config.password_history,
                &mut user,
                &password,
                new_hash,
            )?;
            // History checks may take time. Expiry or freshness failure rolls
            // the factor, password, epoch and revocation writes back together.
            if session.expires_at <= now() {
                return Err(Error::unauthorized());
            }
            crate::password::require_fresh_mfa(&user, &session, factor_required)?;
            Ok(Ok(json!({"changed": true, "sessions_revoked": true})))
        })?
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
        if let Some(context) = crate::context::current()
            && (context.idempotency_key.is_none() || context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "Signing-key rotation requires Idempotency-Key and If-Match",
            ));
        }
        self.store.read(|tx| {
            self.management(tx, token, "key.rotate", "key/signing")
                .map(|_| ())
        })?;
        self.mutation(token, |tx| {
            crate::management::rotate_signing_key(self, tx, token)
        })
    }
    pub fn cleanup(&self) -> Result<()> {
        let _timer = self.store.telemetry().cleanup.timer();
        let result = crate::telemetry::in_activity(crate::telemetry::Activity::Maintenance, || {
            self.cleanup_pass()
        });
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
            .write(|tx| crate::assembly::cleanup_prepared(tx, at))?;
        self.store
            .write(|tx| crate::session_protocol::cleanup(tx, at))?;
        self.store.write(|tx| crate::source::cleanup(tx, at))?;
        self.store.write(|tx| crate::passkey::cleanup(tx, at))?;
        self.store
            .write(|tx| crate::authenticator::cleanup(tx, at))?;
        self.store
            .write(|tx| crate::identity::windows_credentials::cleanup(tx, at))?;
        self.store.write(|tx| crate::lifecycle::cleanup(tx, at))?;
        self.store
            .write(|tx| crate::provisioning::cleanup(tx, at))?;
        self.store.write(|tx| crate::directory::cleanup(tx, at))?;
        #[cfg(feature = "platform")]
        self.store
            .write(|tx| crate::cloud_directory::cleanup(tx, at))?;
        self.store.write(|tx| crate::outpost::cleanup(tx, at))?;
        self.store.write(|tx| crate::radius::cleanup(tx, at))?;
        self.store.write(|tx| crate::mtls::cleanup(tx, at))?;
        self.store.write(|tx| crate::saml::cleanup(tx, at))?;
        self.store.write(crate::context::cleanup)?;
        self.store.write(|tx| crate::browser::cleanup(tx, at))?;
        self.store.write(|tx| crate::portal::cleanup(tx, at))?;
        #[cfg(feature = "platform")]
        self.store
            .write(|tx| crate::workflow::executor::cleanup(tx, at))?;
        #[cfg(feature = "platform")]
        self.store
            .write(|tx| crate::management::cleanup_access(tx, at))?;
        self.store
            .write(|tx| crate::identity::logout_queue::cleanup(tx, at))?;
        #[cfg(feature = "platform")]
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
        #[cfg(feature = "platform")]
        crate::offboarding::cleanup(self)?;
        Ok(())
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
        crate::identity::validate_session(tx, identity, &user)?;
        Ok(user)
    }
    /// Every identity check except the session row, for identities not yet bound to a session.
    pub(crate) fn identity_user_unbound(&self, tx: &Tx<'_>, identity: &Identity) -> Result<User> {
        self.radius_eap_validate_identity(tx, identity)?;
        crate::mtls::validate_identity(tx, identity)?;
        crate::directory::validate_identity(self, tx, identity)?;
        crate::assembly::source_validate_identity(tx, identity)?;
        crate::identity::validate_user(tx, identity)
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
    audit_with_details(tx, actor, action, target, Value::Null)
}

/// `audit` with operator-supplied context, such as resolution evidence. The
/// context is stored in `details.context` and redacted like the change log.
pub(crate) fn audit_with(
    tx: &Tx<'_>,
    actor: &str,
    action: &str,
    target: &str,
    context: Option<Value>,
) -> Result<()> {
    audit_with_details(
        tx,
        actor,
        action,
        target,
        context.map_or(Value::Null, |context| json!({"context": context})),
    )
}

pub(crate) fn audit_with_details(
    tx: &Tx<'_>,
    actor: &str,
    action: &str,
    target: &str,
    extra: Value,
) -> Result<()> {
    if [
        "user.",
        "group.",
        "client.",
        "registration.",
        "source.configure",
        "source.reconcile",
        "source_link.reconcile",
        "agent.",
        "delegation.",
        "access.",
        "ssf.stream.",
        "signing_key.",
        "admin.recover",
        "directory.apply",
        "reconciliation.schedule.",
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
    if let Some(extra) = extra.as_object() {
        for (key, value) in extra {
            details[key] = value.clone();
        }
    }
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
    tx.user_group_names(uid)
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
/// Changing a factor needs an MFA session once the user has TOTP or a passkey.
#[doc(hidden)]
pub use crate::identity::require_factor_session;
pub use crate::validation::validate_name;
pub(crate) use crate::validation::{validate_display, validate_email};
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
    crate::claims::validate_conditional_references(tx, c)?;
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
