//! Core passkey ceremonies and concrete credential persistence.

use crate::{
    core::{Core, audit, validate_display, validate_email, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{AuthenticationTransaction, Identity, Session, User},
    passkey::{
        AdminEnrollment, AdminRegistration, Authentication, BrowserPasskeyContext, Credential, NewPasskeyAdmin,
        PasskeyMaintenance, PasskeyTx, Registration, VerifiedRegistration, credential_id, credential_key, handle,
        passkey_list_in, public_request, require_fresh_factor, unknown_passkey, user_keys, view,
        webauthn_for_issuer,
    },
    signin::{invalid_credentials, reauthentication_required},
    store::Tx,
};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use webauthn_rs::prelude::*;

impl PasskeyTx for Tx<'_> {
    fn credentials(&self) -> Result<Vec<(String, Credential)>> {
        self.list("passkeys")
    }

    #[cfg(feature = "platform")]
    fn authentication(&self, key: &str) -> Result<Option<Authentication>> {
        self.get("passkey_authentication", key)
    }

    #[cfg(feature = "platform")]
    fn delete_authentication(&self, key: &str) -> Result<()> {
        self.delete("passkey_authentication", key)
    }
}

impl PasskeyMaintenance for Tx<'_> {
    fn credential_ids_for_user(&self, user_id: &str) -> Result<Vec<String>> {
        Ok(self
            .list::<Credential>("passkeys")?
            .into_iter()
            .filter(|(_, credential)| credential.user_id == user_id)
            .map(|(_, credential)| credential.id)
            .collect())
    }

    fn delete_credential(&self, id: &str) -> Result<()> {
        self.delete("passkeys", id)
    }

    fn pending_page(&self, bucket: &str) -> Result<Vec<(String, Value)>> {
        self.maintenance_page(bucket)
    }

    fn delete_pending(&self, bucket: &str, id: &str) -> Result<()> {
        self.delete(bucket, id)
    }
}

impl VerifiedRegistration {
    fn apply(self, tx: &Tx<'_>) -> Result<Value> {
        let (credential, account_epoch) = self.into_parts();
        let mut user: User = tx
            .get("users", &credential.user_id)?
            .ok_or_else(Error::forbidden)?;
        if !user.enabled
            || user.epoch != account_epoch
            || tx.get::<Credential>("passkeys", &credential.id)?.is_some()
            || user_keys(tx, &user.id)?.len() >= 16
        {
            return Err(Error::forbidden());
        }
        user.epoch = user.epoch.checked_add(1).ok_or_else(Error::forbidden)?;
        user.has_passkeys = true;
        tx.put("passkeys", &credential.id, &credential)?;
        tx.put("users", &user.id, &user)?;
        crate::logout::queue_user(tx, &user.id)?;
        audit(tx, &user.id, "passkey.enroll", &credential.id)?;
        Ok(json!({"passkey":view(&credential),"sessions_revoked":true,"instruction":"Log in with the new passkey"}))
    }

    #[cfg(feature = "platform")]
    pub(crate) fn commit_workflow(self, tx: &Tx<'_>) -> Result<crate::passkey::workflow::Mutation> {
        let mutation = self.mutation()?;
        self.apply(tx)?;
        let user: User = tx
            .get("users", &mutation.account)?
            .ok_or_else(Error::forbidden)?;
        if user.epoch != mutation.to_epoch {
            return Err(Error::forbidden());
        }
        Ok(mutation)
    }
}

/// Idempotently discard only the ceremony owned by this workflow reservation.
pub(crate) fn discard_workflow_registration(
    tx: &Tx<'_>,
    ceremony: &str,
    binding: &str,
) -> Result<()> {
    let key = digest(ceremony);
    if let Some(pending) = tx.get::<Registration>("passkey_registration", &key)? {
        if pending.workflow_binding.as_deref() != Some(digest(binding).as_str()) {
            return Err(Error::forbidden());
        }
        tx.delete("passkey_registration", &key)?;
    }
    Ok(())
}

fn webauthn(core: &Core) -> Result<Webauthn> {
    webauthn_for_issuer(&core.config.issuer)
}

fn validate_passkey_name(name: &str) -> Result<()> {
    validate_display(name)?;
    if name.trim().is_empty() {
        return Err(Error::bad("Passkey name must not be blank"));
    }
    Ok(())
}

fn admin_registration_options(
    issuer: &str,
    user: &User,
    exclude: Vec<CredentialID>,
) -> Result<(Value, PasskeyRegistration)> {
    let (challenge, state) = webauthn_for_issuer(issuer)?
        .start_passkey_registration(
            handle(&user.id),
            &user.username,
            &user.display_name,
            Some(exclude),
        )
        .map_err(|_| Error::bad("Cannot start administrator passkey enrollment"))?;
    let mut options = json!(challenge);
    options["publicKey"]["authenticatorSelection"] =
        json!({"residentKey":"required","requireResidentKey":true,"userVerification":"required"});
    Ok((options, state))
}

impl AdminEnrollment {
    pub(crate) fn start(issuer: &str, input: NewPasskeyAdmin) -> Result<(Value, Self)> {
        validate_name(&input.username)?;
        validate_passkey_name(&input.primary_name)?;
        validate_passkey_name(&input.backup_name)?;
        if input.primary_name.trim() == input.backup_name.trim() {
            return Err(Error::bad(
                "Give the primary and backup passkeys different names",
            ));
        }
        if let Some(email) = &input.email {
            validate_email(email)?;
        }
        let display_name = if input.display_name.is_empty() {
            input.username.clone()
        } else {
            input.display_name
        };
        validate_display(&display_name)?;
        let user = User {
            has_passkeys: false,
            totp_settings: Default::default(),
            pairwise_seed: crypto::random_token(""),
            id: crypto::id(),
            username: input.username,
            email: input.email,
            display_name,
            password_hash: String::new(),
            enabled: true,
            admin: true,
            epoch: 0,
            totp_secret: None,
            totp_pending: None,
            totp_last_step: None,
            created_at: now(),
            attributes: Default::default(),
            email_verified: false,
            subjects: Default::default(),
            recovery_codes: Default::default(),
        };
        let (options, state) = admin_registration_options(issuer, &user, Vec::new())?;
        Ok((
            options,
            Self {
                user,
                state,
                primary: None,
                primary_name: input.primary_name,
                backup_name: input.backup_name,
            },
        ))
    }

    pub(crate) fn has_primary(&self) -> bool {
        self.primary.is_some()
    }

    pub(crate) fn first(
        &mut self,
        issuer: &str,
        tx: &Tx<'_>,
        response: RegisterPublicKeyCredential,
    ) -> Result<Value> {
        if self.primary.is_some() {
            return Err(Error::unauthorized());
        }
        let key = webauthn_for_issuer(issuer)?
            .finish_passkey_registration(&response, &self.state)
            .map_err(|_| Error::bad("Primary passkey verification failed"))?;
        let id = credential_id(key.cred_id());
        if tx.get::<Credential>("passkeys", &id)?.is_some() {
            return Err(Error::conflict("Credential is already enrolled"));
        }
        let primary = Credential {
            id,
            user_id: self.user.id.clone(),
            name: self.primary_name.clone(),
            created_at: now(),
            counter: 0,
            key,
        };
        let (options, state) =
            admin_registration_options(issuer, &self.user, vec![primary.key.cred_id().clone()])?;
        self.primary = Some(primary);
        self.state = state;
        Ok(options)
    }

    /// Caller supplies its authorization and lifecycle checks inside this same writer.
    pub(crate) fn finish(
        self,
        issuer: &str,
        tx: &Tx<'_>,
        response: RegisterPublicKeyCredential,
    ) -> Result<User> {
        let primary = self.primary.ok_or_else(Error::unauthorized)?;
        if tx
            .get::<String>("usernames", &self.user.username)?
            .is_some()
        {
            return Err(Error::conflict("Username already exists"));
        }
        let key = webauthn_for_issuer(issuer)?
            .finish_passkey_registration(&response, &self.state)
            .map_err(|_| Error::bad("Backup passkey verification failed"))?;
        let backup_id = credential_id(key.cred_id());
        if backup_id == primary.id
            || tx.get::<Credential>("passkeys", &backup_id)?.is_some()
            || tx.get::<Credential>("passkeys", &primary.id)?.is_some()
        {
            return Err(Error::conflict("Use a distinct, unenrolled backup passkey"));
        }
        let backup = Credential {
            id: backup_id,
            user_id: self.user.id.clone(),
            name: self.backup_name,
            created_at: now(),
            counter: 0,
            key,
        };
        let mut user = self.user;
        user.has_passkeys = true;
        tx.put("users", &user.id, &user)?;
        tx.put("usernames", &user.username, &user.id)?;
        tx.put("passkeys", &primary.id, &primary)?;
        tx.put("passkeys", &backup.id, &backup)?;
        Ok(user)
    }
}

impl Core {
    fn admin_registration_session(
        &self,
        tx: &Tx<'_>,
        token: &str,
        cookie: &str,
        pending: Option<&AdminRegistration>,
        username: &str,
    ) -> Result<(User, Session)> {
        let actor = self.management(tx, token, "user.write", &format!("user/{username}"))?;
        let (user, session) = self.browser_user(tx, cookie)?;
        if actor.agent || actor.delegated || actor.id != user.id || crate::signin::bearer_backed(tx, &session)? {
            return Err(reauthentication_required());
        }
        require_fresh_factor(&user, &session)?;
        if !session.identity.mfa {
            return Err(Error::new(
                StatusCode::FORBIDDEN,
                "mfa_required",
                "Sign in with a passkey or authenticator code first",
            ));
        }
        if let Some(pending) = pending {
            if pending.expires_at <= now()
                || pending.owner_id != user.id
                || pending.session_id != session.id
                || pending.session_epoch != user.epoch
                || !crypto::constant_eq(&pending.binding_hash, &digest(cookie))
            {
                return Err(reauthentication_required());
            }
        }
        Ok((user, session))
    }

    /// The browser administrator creates a new account only after both independent
    /// credential IDs have been verified. No provisional administrator can sign in.
    pub fn admin_passkey_start(
        &self,
        token: &str,
        cookie: &str,
        input: NewPasskeyAdmin,
    ) -> Result<Value> {
        let (public_key, enrollment) = AdminEnrollment::start(&self.config.issuer, input)?;
        self.store.write(|tx| {
            let (owner, session) = self.admin_registration_session(
                tx,
                token,
                cookie,
                None,
                &enrollment.user.username,
            )?;
            if tx
                .get::<String>("usernames", &enrollment.user.username)?
                .is_some()
            {
                return Err(Error::conflict("Username already exists"));
            }
            let pending = tx.list::<AdminRegistration>("admin_passkey_registration")?;
            let active = pending.iter().filter(|(_, p)| p.expires_at > now()).count();
            for (key, pending) in pending {
                if pending.expires_at <= now() {
                    tx.delete("admin_passkey_registration", &key)?;
                }
            }
            if active >= 128 {
                return Err(Error::conflict(
                    "Too many pending administrator enrollments",
                ));
            }
            let ceremony = crypto::random_token("ri_admin_passkey_");
            tx.put(
                "admin_passkey_registration",
                &digest(&ceremony),
                &AdminRegistration {
                    enrollment,
                    owner_id: owner.id,
                    session_id: session.id,
                    session_epoch: owner.epoch,
                    binding_hash: digest(cookie),
                    expires_at: now() + 300,
                },
            )?;
            Ok(json!({"ceremony":ceremony,"public_key":public_key,"expires_in":300}))
        })
    }

    pub fn admin_passkey_first(
        &self,
        token: &str,
        cookie: &str,
        ceremony: &str,
        response: RegisterPublicKeyCredential,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let mut pending = tx.get::<AdminRegistration>("admin_passkey_registration", &digest(ceremony))?
                .filter(|p| !p.enrollment.has_primary()).ok_or_else(Error::unauthorized)?;
            self.admin_registration_session(tx, token, cookie, Some(&pending), &pending.enrollment.user.username)?;
            let public_key = pending.enrollment.first(&self.config.issuer, tx, response)?;
            let next = crypto::random_token("ri_admin_passkey_");
            tx.delete("admin_passkey_registration", &digest(ceremony))?;
            tx.put("admin_passkey_registration", &digest(&next), &pending)?;
            Ok(json!({"ceremony":next,"public_key":public_key,"expires_in":pending.expires_at.saturating_sub(now())}))
        })
    }

    pub fn admin_passkey_finish(
        &self,
        token: &str,
        cookie: &str,
        ceremony: &str,
        response: RegisterPublicKeyCredential,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let pending = tx
                .get::<AdminRegistration>("admin_passkey_registration", &digest(ceremony))?
                .ok_or_else(Error::unauthorized)?;
            let (owner, _) = self.admin_registration_session(
                tx,
                token,
                cookie,
                Some(&pending),
                &pending.enrollment.user.username,
            )?;
            let user = pending
                .enrollment
                .finish(&self.config.issuer, tx, response)?;
            crate::delegation::record_elevation_provenance(
                tx,
                &user,
                crate::delegation::ProvenanceBasis::HumanCreate,
            )?;
            tx.delete("admin_passkey_registration", &digest(ceremony))?;
            audit(tx, &owner.id, "user.create.passkey_only", &user.id)?;
            Ok(json!({"user":crate::model::UserView::from(&user),"passkeys":2}))
        })
    }

    pub fn admin_passkey_cancel(&self, token: &str, cookie: &str, ceremony: &str) -> Result<Value> {
        self.store.write(|tx| {
            let pending = tx
                .get::<AdminRegistration>("admin_passkey_registration", &digest(ceremony))?
                .ok_or_else(Error::unauthorized)?;
            self.admin_registration_session(
                tx,
                token,
                cookie,
                Some(&pending),
                &pending.enrollment.user.username,
            )?;
            tx.delete("admin_passkey_registration", &digest(ceremony))?;
            Ok(json!({"cancelled":true}))
        })
    }
}

impl Core {
    pub fn passkeys(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let (user, _) = self.session(tx, token)?;
            Ok(json!(passkey_list_in(tx, &user.id)?))
        })
    }
    pub fn passkey_register_start(&self, token: &str, name: String) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            self.passkey_register_start_in(tx, &user, &session, name, false)
        })
    }
    /// `resident` asks browsers for a discoverable credential. It is advisory: the server
    /// state is the same, so non-resident keys still work for pinned sign-in.
    #[doc(hidden)]
    pub fn passkey_register_start_in(
        &self,
        tx: &Tx<'_>,
        user: &User,
        session: &Session,
        name: String,
        resident: bool,
    ) -> Result<Value> {
        validate_passkey_name(&name)?;
        require_fresh_factor(user, session)?;
        let keys = user_keys(tx, &user.id)?;
        if keys.len() >= 16 {
            return Err(Error::conflict("At most sixteen passkeys can be enrolled"));
        }
        let (challenge, state) = webauthn(self)?
            .start_passkey_registration(
                handle(&user.id),
                &user.username,
                &user.display_name,
                Some(keys.iter().map(|c| c.key.cred_id().clone()).collect()),
            )
            .map_err(|_| Error::bad("Cannot start passkey enrollment"))?;
        let mut public_key = json!(challenge);
        if resident {
            public_key["publicKey"]["authenticatorSelection"] = json!({"residentKey":"required","requireResidentKey":true,"userVerification":"required"});
        }
        let ceremony = crypto::random_token("ri_passkey_enroll_");
        tx.put(
            "passkey_registration",
            &digest(&ceremony),
            &Registration {
                identity: session.identity.clone(),
                name,
                expires_at: now() + 300,
                state,
                workflow_binding: None,
            },
        )?;
        Ok(json!({"ceremony":ceremony,"public_key":public_key,"expires_in":300}))
    }
    pub fn passkey_register_finish(
        &self,
        token: &str,
        ceremony: &str,
        response: RegisterPublicKeyCredential,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            self.passkey_register_finish_in(tx, user, &session, ceremony, response)
        })?
    }
    pub(crate) fn passkey_register_finish_in(
        &self,
        tx: &Tx<'_>,
        user: User,
        session: &Session,
        ceremony: &str,
        response: RegisterPublicKeyCredential,
    ) -> Result<Result<Value>> {
        match self.verify_registration_in(tx, user, session, ceremony, response, None)? {
            Ok(verified) => Ok(Ok(verified.apply(tx)?)),
            Err(error) => Ok(Err(error)),
        }
    }

    fn verify_registration_in(
        &self,
        tx: &Tx<'_>,
        user: User,
        session: &Session,
        ceremony: &str,
        response: RegisterPublicKeyCredential,
        workflow_binding: Option<&str>,
    ) -> Result<Result<VerifiedRegistration>> {
        let expected_binding = workflow_binding.map(digest);
        let pending = tx
            .get::<Registration>("passkey_registration", &digest(ceremony))?
            .filter(|p| {
                p.expires_at > now()
                    && p.identity.user_id == user.id
                    && p.identity.session_id == session.id
                    && p.identity.epoch == user.epoch
                    && p.workflow_binding == expected_binding
            })
            .ok_or_else(Error::unauthorized)?;
        tx.delete("passkey_registration", &digest(ceremony))?;
        // A ceremony does not extend the authorization that started it. Commit its
        // consumption even when freshness or the factor requirement changed meanwhile.
        if let Err(error) = require_fresh_factor(&user, session) {
            return Ok(Err(error));
        }
        if user_keys(tx, &user.id)?.len() >= 16 {
            return Ok(Err(Error::conflict("Passkey limit reached")));
        }
        let key = match webauthn(self)?.finish_passkey_registration(&response, &pending.state) {
            Ok(key) => key,
            Err(_) => return Ok(Err(Error::bad("Passkey registration verification failed"))),
        };
        let id = credential_id(key.cred_id());
        if tx.get::<Credential>("passkeys", &id)?.is_some() {
            return Ok(Err(Error::conflict("Credential is already enrolled")));
        }
        let credential = Credential {
            id,
            user_id: user.id.clone(),
            name: pending.name,
            created_at: now(),
            counter: 0,
            key,
        };
        Ok(Ok(VerifiedRegistration::new(credential, user.epoch)))
    }

    pub(crate) fn workflow_register_start_in(
        &self,
        tx: &Tx<'_>,
        user: &User,
        session: &Session,
        name: String,
        binding: &str,
        expires_at: u64,
    ) -> Result<Value> {
        let started = self.passkey_register_start_in(tx, user, session, name, false)?;
        let key = digest(started["ceremony"].as_str().ok_or_else(Error::forbidden)?);
        let mut pending: Registration = tx
            .get("passkey_registration", &key)?
            .ok_or_else(Error::forbidden)?;
        pending.workflow_binding = Some(digest(binding));
        pending.expires_at = pending.expires_at.min(expires_at);
        tx.put("passkey_registration", &key, &pending)?;
        Ok(started)
    }

    pub(crate) fn workflow_register_verify_in(
        &self,
        tx: &Tx<'_>,
        user: User,
        session: &Session,
        ceremony: &str,
        response: RegisterPublicKeyCredential,
        binding: &str,
    ) -> Result<Result<VerifiedRegistration>> {
        self.verify_registration_in(tx, user, session, ceremony, response, Some(binding))
    }

    pub fn passkey_register_cancel(&self, token: &str, ceremony: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            self.passkey_register_cancel_in(tx, &user, &session, ceremony)
        })
    }
    pub(crate) fn passkey_register_cancel_in(
        &self,
        tx: &Tx<'_>,
        user: &User,
        session: &Session,
        ceremony: &str,
    ) -> Result<Value> {
        let key = digest(ceremony);
        tx.get::<Registration>("passkey_registration", &key)?
            .filter(|pending| {
                pending.identity.user_id == user.id
                    && pending.identity.session_id == session.id
                    && pending.identity.epoch == user.epoch
                    && pending.workflow_binding.is_none()
            })
            .ok_or_else(Error::unauthorized)?;
        tx.delete("passkey_registration", &key)?;
        Ok(json!({"cancelled":true}))
    }
    pub fn passkey_rename(&self, token: &str, id: &str, name: String) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            self.passkey_rename_in(tx, &user, &session, id, name)
        })
    }
    pub(crate) fn passkey_rename_in(
        &self,
        tx: &Tx<'_>,
        user: &User,
        session: &Session,
        id: &str,
        name: String,
    ) -> Result<Value> {
        require_fresh_factor(user, session)?;
        validate_passkey_name(&name)?;
        let mut credential = tx
            .get::<Credential>("passkeys", id)?
            .filter(|credential| credential.user_id == user.id)
            .ok_or_else(|| Error::missing("Passkey not found"))?;
        credential.name = name;
        tx.put("passkeys", id, &credential)?;
        audit(tx, &user.id, "passkey.rename", id)?;
        Ok(json!({"passkey":view(&credential),"renamed":true,"sessions_revoked":false}))
    }
    pub fn passkey_remove(&self, token: &str, id: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            self.passkey_remove_in(tx, user, &session, id)
        })
    }
    pub(crate) fn passkey_remove_in(
        &self,
        tx: &Tx<'_>,
        mut user: User,
        session: &Session,
        id: &str,
    ) -> Result<Value> {
        require_fresh_factor(&user, session)?;
        let credential = tx
            .get::<Credential>("passkeys", id)?
            .filter(|c| c.user_id == user.id)
            .ok_or_else(|| Error::missing("Passkey not found"))?;
        let count = user_keys(tx, &user.id)?.len();
        if user.password_hash.is_empty() && count == 1 {
            return Err(Error::conflict(
                "Establish another local credential before removing the last passkey",
            ));
        }
        if user.password_hash.is_empty() && user.admin && count <= 2 {
            return Err(Error::conflict(
                "Passkey-only administrators must keep two passkeys",
            ));
        }
        tx.delete("passkeys", id)?;
        user.has_passkeys = count > 1;
        user.epoch += 1;
        tx.put("users", &user.id, &user)?;
        crate::logout::queue_user(tx, &user.id)?;
        audit(tx, &user.id, "passkey.remove", &credential.id)?;
        Ok(json!({"removed":true,"sessions_revoked":true}))
    }
    /// A username without keys gets a challenge with the real serialized shape and a
    /// credential id derived from the instance secret, so repeated requests are stable.
    fn passkey_decoy(&self, username: &str) -> Result<Value> {
        let (challenge, _) = webauthn(self)?
            .start_discoverable_authentication()
            .map_err(Error::internal)?;
        let mut decoy = public_request(&challenge);
        let id = Sha256::digest(format!(
            "riauth.passkey-decoy/v1\0{}\0{username}",
            self.dummy_password_hash()
        ));
        decoy["publicKey"]["allowCredentials"] =
            json!([{"type":"public-key","id":URL_SAFE_NO_PAD.encode(id)}]);
        Ok(decoy)
    }
    pub fn passkey_login_start(
        &self,
        username: &str,
        transaction: Option<String>,
    ) -> Result<Value> {
        validate_name(username)?;
        self.store.write(|tx| {
            let user = tx
                .get::<String>("usernames", username)?
                .map(|id| tx.get::<User>("users", &id))
                .transpose()?
                .flatten()
                .filter(|u| u.enabled);
            let keys = user
                .as_ref()
                .map(|u| user_keys(tx, &u.id))
                .transpose()?
                .unwrap_or_default();
            let (challenge, state) = if keys.is_empty() {
                (self.passkey_decoy(username)?, None)
            } else {
                let (challenge, state) = webauthn(self)?
                    .start_passkey_authentication(
                        &keys.iter().map(|c| c.key.clone()).collect::<Vec<_>>(),
                    )
                    .map_err(|_| Error::unauthorized())?;
                (public_request(&challenge), Some(state))
            };
            if let Some(transaction) = &transaction {
                let record = tx
                    .get::<AuthenticationTransaction>("authentication", &digest(transaction))?
                    .filter(|t| t.expires_at > now() && t.authenticated_session.is_none())
                    .ok_or_else(|| Error::bad("Authentication transaction expired or used"))?;
                crate::oidc::reject_embedded_stage(&record)?;
            }
            let ceremony = crypto::random_token("ri_passkey_auth_");
            tx.put(
                "passkey_authentication",
                &digest(&ceremony),
                &Authentication {
                    user_id: user.as_ref().map(|u| u.id.clone()),
                    epoch: user.as_ref().map(|u| u.epoch).unwrap_or(0),
                    expires_at: now() + 300,
                    state,
                    transaction,
                    interaction: None,
                    binding_hash: None,
                    session_id: None,
                    discoverable: None,
                },
            )?;
            Ok(json!({"ceremony":ceremony,"public_key":challenge,"expires_in":300}))
        })
    }
    pub fn passkey_login_finish(
        &self,
        ceremony: &str,
        response: PublicKeyCredential,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let pending = tx.get::<Authentication>("passkey_authentication", &digest(ceremony))?
                .filter(|p|p.expires_at>now()).ok_or_else(Error::unauthorized)?;
            tx.delete("passkey_authentication",&digest(ceremony))?;
            let verified = (|| {
                // Browser ceremonies stage a login for their interaction; they never mint bearer tokens.
                if pending.interaction.is_some() {return Err(Error::unauthorized());}
                let user = tx.get::<User>("users",pending.user_id.as_deref().ok_or_else(Error::unauthorized)?)?
                    .filter(|u|u.enabled && u.epoch==pending.epoch).ok_or_else(Error::unauthorized)?;
                if response.get_user_unique_id().is_some_and(|id|id!=handle(&user.id).as_bytes()) {return Err(Error::unauthorized());}
                let result = webauthn(self)?.finish_passkey_authentication(&response,pending.state.as_ref().ok_or_else(Error::unauthorized)?)
                    .map_err(|_|Error::unauthorized())?;
                if !result.user_verified() {return Err(Error::unauthorized());}
                let id = credential_id(result.cred_id());
                let mut credential = tx.get::<Credential>("passkeys",&id)?.filter(|c|c.user_id==user.id).ok_or_else(Error::unauthorized)?;
                // Compare the live counter as concurrently issued challenges hold older snapshots.
                if (result.counter()!=0 || credential.counter!=0) && result.counter()<=credential.counter {return Err(Error::unauthorized());}
                credential.counter=result.counter();
                credential.key.update_credential(&result).ok_or_else(Error::unauthorized)?;
                let transaction = pending.transaction.as_ref().map(|transaction| {
                    let challenge = tx.get::<AuthenticationTransaction>("authentication",&digest(transaction))?
                        .filter(|c|c.expires_at>now() && c.authenticated_session.is_none() && c.user_id.as_ref().is_none_or(|id|id==&user.id)).ok_or_else(Error::forbidden)?;
                    crate::oidc::reject_embedded_stage(&challenge)?;
                    Ok((digest(transaction),challenge))
                }).transpose()?;
                Ok((user,credential,transaction))
            })();
            let (user,credential,transaction) = match verified {
                Ok(verified)=>verified,
                Err(error) if error.status.is_server_error()=>return Err(error),
                Err(error)=>{audit(tx,"anonymous","passkey.login_failed","passkey")?;return Ok(Err(error));},
            };
            let identity=Identity{user_id:user.id.clone(),epoch:user.epoch,mfa:true,auth_time:now(),session_id:String::new(),amr:vec!["webauthn".into(),"mfa".into()],source:None};
            let (session,token)=self.mint_bearer_session(tx,identity,now()+self.config.session_ttl)?;
            if let Some((key,mut challenge))=transaction {
                challenge.authenticated_session=Some(session.id.clone());tx.put("authentication",&key,&challenge)?;
            }
            tx.put("passkeys",&credential.id,&credential)?;
            audit(tx,&user.id,"passkey.login",&credential.id)?;
            Ok(Ok(json!({"session_token":token,"expires_at":session.expires_at,"user":crate::model::UserView::from(&user)})))
        })?
    }
    /// Starts a browser passkey ceremony bound to one interaction and one browser binding.
    /// A pinned account lists its own keys; otherwise the browser offers discoverable keys.
    #[doc(hidden)]
    pub fn browser_passkey_start(
        &self,
        pinned_user: Option<&str>,
        interaction: &str,
        binding_hash: &str,
    ) -> Result<Value> {
        self.browser_passkey_start_for_session(pinned_user, interaction, binding_hash, None)
    }
    pub(crate) fn browser_passkey_start_for_session(
        &self,
        pinned_user: Option<&str>,
        interaction: &str,
        binding_hash: &str,
        session_id: Option<String>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            self.browser_passkey_start_in(
                tx,
                pinned_user,
                interaction,
                binding_hash,
                session_id,
                now() + 300,
            )
        })
    }
    /// The workflow executor reserves this existing verifier in its run writer.
    /// A caller may shorten the ceremony lifetime, never extend it.
    pub(crate) fn browser_passkey_start_in(
        &self,
        tx: &Tx<'_>,
        pinned_user: Option<&str>,
        interaction: &str,
        binding_hash: &str,
        session_id: Option<String>,
        expires_at: u64,
    ) -> Result<Value> {
        let webauthn = webauthn(self)?;
        let mut record = Authentication {
            user_id: None,
            epoch: 0,
            expires_at: expires_at.min(now() + 300),
            state: None,
            transaction: None,
            interaction: Some(interaction.into()),
            binding_hash: Some(binding_hash.into()),
            session_id,
            discoverable: None,
        };
        let challenge = if let Some(uid) = pinned_user {
            let user = tx
                .get::<User>("users", uid)?
                .filter(|u| u.enabled)
                .ok_or_else(Error::unauthorized)?;
            let keys = user_keys(tx, &user.id)?;
            if keys.is_empty() {
                return Err(Error::new(
                    StatusCode::CONFLICT,
                    "no_passkey",
                    "This account has no passkey. Sign in with your password.",
                ));
            }
            let (challenge, state) = webauthn
                .start_passkey_authentication(
                    &keys.iter().map(|c| c.key.clone()).collect::<Vec<_>>(),
                )
                .map_err(Error::internal)?;
            (record.user_id, record.epoch, record.state) = (Some(user.id), user.epoch, Some(state));
            challenge
        } else {
            let (challenge, state) = webauthn
                .start_discoverable_authentication()
                .map_err(Error::internal)?;
            record.discoverable = Some(state);
            challenge
        };
        let ceremony = crypto::random_token("ri_passkey_auth_");
        tx.put("passkey_authentication", &digest(&ceremony), &record)?;
        Ok(json!({"ceremony":ceremony,"public_key":public_request(&challenge),"expires_in":300}))
    }
    /// Verifies a browser passkey ceremony and returns the staged login id. The ceremony is
    /// single use: any failure after it is found still consumes it.
    #[doc(hidden)]
    pub fn browser_passkey_finish(
        &self,
        ceremony: &str,
        response: PublicKeyCredential,
        interaction: &str,
        binding: Option<&str>,
        pinned_user: Option<&str>,
    ) -> Result<String> {
        self.store.write(|tx| {
            self.browser_passkey_finish_in(
                tx,
                ceremony,
                response,
                BrowserPasskeyContext {
                    interaction,
                    binding,
                    pinned_user,
                    session_id: None,
                },
            )
        })?
    }
    pub(crate) fn browser_passkey_finish_in(
        &self,
        tx: &Tx<'_>,
        ceremony: &str,
        response: PublicKeyCredential,
        context: BrowserPasskeyContext<'_>,
    ) -> Result<Result<String>> {
        let key = digest(ceremony);
        let pending = tx
            .get::<Authentication>("passkey_authentication", &key)?
            .ok_or_else(Error::unauthorized)?;
        tx.delete("passkey_authentication", &key)?;
        let verified = (|| {
            let bound = context
                .binding
                .zip(pending.binding_hash.as_deref())
                .is_some_and(|(binding, hash)| crypto::constant_eq(&digest(binding), hash));
            if pending.expires_at <= now()
                || pending.interaction.as_deref() != Some(context.interaction)
                || !bound
                || pending
                    .session_id
                    .as_deref()
                    .is_some_and(|id| Some(id) != context.session_id)
            {
                return Err(Error::unauthorized());
            }
            let webauthn = webauthn(self)?;
            let (user, result) = if let Some(state) = pending.discoverable.clone() {
                let (user_handle, raw) = webauthn
                    .identify_discoverable_authentication(&response)
                    .map_err(|_| invalid_credentials())?;
                let credential = tx
                    .get::<Credential>("passkeys", &credential_key(raw))?
                    .ok_or_else(unknown_passkey)?;
                if handle(&credential.user_id) != user_handle {
                    return Err(invalid_credentials());
                }
                let user = tx
                    .get::<User>("users", &credential.user_id)?
                    .filter(|u| u.enabled)
                    .ok_or_else(invalid_credentials)?;
                let result = webauthn
                    .finish_discoverable_authentication(
                        &response,
                        state,
                        &[DiscoverableKey::from(&credential.key)],
                    )
                    .map_err(|_| invalid_credentials())?;
                (user, result)
            } else {
                let user = tx
                    .get::<User>(
                        "users",
                        pending.user_id.as_deref().ok_or_else(invalid_credentials)?,
                    )?
                    .filter(|u| u.enabled && u.epoch == pending.epoch)
                    .ok_or_else(invalid_credentials)?;
                if response
                    .get_user_unique_id()
                    .is_some_and(|id| id != handle(&user.id).as_bytes())
                {
                    return Err(invalid_credentials());
                }
                let result = webauthn
                    .finish_passkey_authentication(
                        &response,
                        pending.state.as_ref().ok_or_else(invalid_credentials)?,
                    )
                    .map_err(|_| invalid_credentials())?;
                (user, result)
            };
            if !result.user_verified() || context.pinned_user.is_some_and(|pin| pin != user.id) {
                return Err(invalid_credentials());
            }
            let mut credential = tx
                .get::<Credential>("passkeys", &credential_id(result.cred_id()))?
                .filter(|c| c.user_id == user.id)
                .ok_or_else(invalid_credentials)?;
            if (result.counter() != 0 || credential.counter != 0)
                && result.counter() <= credential.counter
            {
                return Err(invalid_credentials());
            }
            credential.counter = result.counter();
            credential
                .key
                .update_credential(&result)
                .ok_or_else(invalid_credentials)?;
            Ok((user, credential))
        })();
        let (user, credential) = match verified {
            Ok(verified) => verified,
            Err(error) if error.status.is_server_error() => return Err(error),
            Err(error) => {
                audit(tx, "anonymous", "passkey.login_failed", "passkey")?;
                return Ok(Err(error));
            }
        };
        tx.put("passkeys", &credential.id, &credential)?;
        let identity = Identity {
            user_id: user.id.clone(),
            epoch: user.epoch,
            mfa: true,
            auth_time: now(),
            session_id: String::new(),
            amr: vec!["webauthn".into(), "mfa".into()],
            source: None,
        };
        let staged =
            self.stage_browser_login(tx, identity, now() + self.config.session_ttl, "passkey")?;
        audit(tx, &user.id, "passkey.login", &credential.id)?;
        Ok(Ok(staged))
    }
    pub(crate) fn browser_passkey_cancel(
        &self,
        ceremony: &str,
        interaction: &str,
        binding: Option<&str>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let key = digest(ceremony);
            tx.get::<Authentication>("passkey_authentication", &key)?
                .filter(|pending| {
                    pending.interaction.as_deref() == Some(interaction)
                        && binding.zip(pending.binding_hash.as_deref()).is_some_and(
                            |(binding, hash)| crypto::constant_eq(&digest(binding), hash),
                        )
                })
                .ok_or_else(Error::unauthorized)?;
            tx.delete("passkey_authentication", &key)?;
            Ok(json!({"cancelled":true}))
        })
    }
}
