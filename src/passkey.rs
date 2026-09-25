//! WebAuthn ceremonies persist private challenge state on the server.
use crate::{
    core::{Core, audit, require_factor_session, validate_display, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{AuthenticationTransaction, Identity, Session, User},
    signin::{FRESH_SECONDS, invalid_credentials, reauthentication_required},
    store::Tx,
};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use webauthn_rs::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
struct Credential {
    id: String,
    user_id: String,
    name: String,
    created_at: u64,
    counter: u32,
    key: Passkey,
}
#[derive(Serialize, Deserialize)]
struct Registration {
    identity: Identity,
    name: String,
    expires_at: u64,
    state: PasskeyRegistration,
}
#[derive(Serialize, Deserialize)]
struct Authentication {
    user_id: Option<String>,
    epoch: u64,
    expires_at: u64,
    state: Option<PasskeyAuthentication>,
    transaction: Option<String>,
    /// Browser ceremonies only: `portal`, `oidc:{id}` or `saml:{id}`. The API finish refuses them.
    #[serde(default)]
    interaction: Option<String>,
    /// Digest of the browser binding that started the ceremony.
    #[serde(default)]
    binding_hash: Option<String>,
    /// Usernameless state; the credential names its user.
    #[serde(default)]
    discoverable: Option<DiscoverableAuthentication>,
}

fn webauthn(core: &Core) -> Result<Webauthn> {
    let uri = url::Url::parse(&core.config.issuer).map_err(Error::internal)?;
    let origin = url::Url::parse(&uri.origin().ascii_serialization()).map_err(Error::internal)?;
    WebauthnBuilder::new(
        uri.host_str()
            .ok_or_else(|| Error::bad("WebAuthn RP host missing"))?,
        &origin,
    )
    .map_err(|_| Error::bad("Invalid WebAuthn relying-party origin"))?
    .rp_name("riAuth")
    .build()
    .map_err(Error::internal)
}
fn handle(user_id: &str) -> Uuid {
    let hash = Sha256::digest(format!("riauth.webauthn-user/v1\0{user_id}"));
    Uuid::from_bytes(hash[..16].try_into().unwrap())
}
fn credential_key(raw: &[u8]) -> String {
    digest(&URL_SAFE_NO_PAD.encode(raw))
}
fn credential_id(id: &CredentialID) -> String {
    credential_key(id.as_slice())
}
fn user_keys(tx: &Tx<'_>, user_id: &str) -> Result<Vec<Credential>> {
    Ok(tx
        .list::<Credential>("passkeys")?
        .into_iter()
        .filter(|(_, c)| c.user_id == user_id)
        .map(|(_, c)| c)
        .collect())
}
fn view(c: &Credential) -> Value {
    json!({"id":c.id,"name":c.name,"created_at":c.created_at,"algorithm":c.key.cred_algorithm()})
}
/// Request options as sent to clients: no transports, mediation or extensions, so a challenge
/// never reveals how a key was enrolled and the decoy keeps the real shape.
fn public_request(challenge: &RequestChallengeResponse) -> Value {
    let mut value = json!(challenge);
    if let Some(options) = value.as_object_mut() {
        options.remove("mediation");
    }
    if let Some(options) = value["publicKey"].as_object_mut() {
        options.remove("extensions");
    }
    if let Some(entries) = value["publicKey"]["allowCredentials"].as_array_mut() {
        for entry in entries.iter_mut().filter_map(Value::as_object_mut) {
            entry.remove("transports");
        }
    }
    value
}
fn unknown_passkey() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "unknown_passkey",
        "This passkey isn't registered with riAuth. Use another passkey or sign in with your password.",
    )
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
        validate_display(&name)?;
        if now().saturating_sub(session.identity.auth_time) > FRESH_SECONDS {
            return Err(reauthentication_required());
        }
        require_factor_session(user, session)?;
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
        mut user: User,
        session: &Session,
        ceremony: &str,
        response: RegisterPublicKeyCredential,
    ) -> Result<Result<Value>> {
        let pending = tx
            .get::<Registration>("passkey_registration", &digest(ceremony))?
            .filter(|p| {
                p.expires_at > now()
                    && p.identity.user_id == user.id
                    && p.identity.session_id == session.id
                    && p.identity.epoch == user.epoch
            })
            .ok_or_else(Error::unauthorized)?;
        if user_keys(tx, &user.id)?.len() >= 16 {
            return Err(Error::conflict("Passkey limit reached"));
        }
        tx.delete("passkey_registration", &digest(ceremony))?;
        let key = match webauthn(self)?.finish_passkey_registration(&response, &pending.state) {
            Ok(key) => key,
            Err(_) => return Ok(Err(Error::bad("Passkey registration verification failed"))),
        };
        let id = credential_id(key.cred_id());
        if tx.get::<Credential>("passkeys", &id)?.is_some() {
            return Ok(Err(Error::conflict("Credential is already enrolled")));
        }
        let credential = Credential {
            id: id.clone(),
            user_id: user.id.clone(),
            name: pending.name,
            created_at: now(),
            counter: 0,
            key,
        };
        tx.put("passkeys", &id, &credential)?;
        user.has_passkeys = true;
        user.epoch += 1;
        tx.put("users", &user.id, &user)?;
        crate::logout::queue_user(tx, &user.id)?;
        audit(tx, &user.id, "passkey.enroll", &id)?;
        Ok(Ok(
            json!({"passkey":view(&credential),"sessions_revoked":true,"instruction":"Log in with the new passkey"}),
        ))
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
        if now().saturating_sub(session.identity.auth_time) > FRESH_SECONDS {
            return Err(reauthentication_required());
        }
        require_factor_session(&user, session)?;
        let credential = tx
            .get::<Credential>("passkeys", id)?
            .filter(|c| c.user_id == user.id)
            .ok_or_else(|| Error::missing("Passkey not found"))?;
        let count = user_keys(tx, &user.id)?.len();
        if count == 1 && user.password_hash.is_empty() {
            return Err(Error::conflict(
                "Establish another local credential before removing the last passkey",
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
        self.store.write(|tx| {
            let webauthn = webauthn(self)?;
            let mut record = Authentication {
                user_id: None,
                epoch: 0,
                expires_at: now() + 300,
                state: None,
                transaction: None,
                interaction: Some(interaction.into()),
                binding_hash: Some(binding_hash.into()),
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
        })
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
            let key = digest(ceremony);
            let pending = tx
                .get::<Authentication>("passkey_authentication", &key)?
                .ok_or_else(Error::unauthorized)?;
            tx.delete("passkey_authentication", &key)?;
            let verified = (|| {
                let bound = binding
                    .zip(pending.binding_hash.as_deref())
                    .is_some_and(|(binding, hash)| crypto::constant_eq(&digest(binding), hash));
                if pending.expires_at <= now()
                    || pending.interaction.as_deref() != Some(interaction)
                    || !bound
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
                if !result.user_verified() || pinned_user.is_some_and(|pin| pin != user.id) {
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
        })?
    }
}
pub(crate) fn passkey_list_in(tx: &Tx<'_>, user_id: &str) -> Result<Vec<Value>> {
    Ok(user_keys(tx, user_id)?.iter().map(view).collect())
}
pub fn clear(tx: &Tx<'_>, user_id: &str) -> Result<()> {
    for credential in user_keys(tx, user_id)? {
        tx.delete("passkeys", &credential.id)?;
    }
    Ok(())
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for bucket in ["passkey_registration", "passkey_authentication"] {
        for (id, value) in tx.maintenance_page::<Value>(bucket)? {
            if value["expires_at"].as_u64().is_none_or(|exp| exp <= at) {
                tx.delete(bucket, &id)?;
            }
        }
    }
    Ok(())
}

/// USB client operations use the same verified issuer origin as the HTTPS API client.
pub async fn usb(issuer: &str, challenge: Value, registration: bool) -> anyhow::Result<Value> {
    use futures_util::StreamExt;
    use webauthn_authenticator_rs::{
        WebauthnAuthenticator,
        ctap2::CtapAuthenticator,
        transport::{TokenEvent, Transport},
        usb::USBTransport,
    };
    let uri = crate::config::validate_server_url(issuer)?;
    let origin = url::Url::parse(&uri.origin().ascii_serialization())?;
    let ui = webauthn_authenticator_rs::ui::Cli {};
    let transport = USBTransport::new().await?;
    let mut tokens = transport.watch().await?;
    eprintln!("Connect your FIDO2 authenticator. Touch it and enter its PIN when requested.");
    let token = tokio::time::timeout(std::time::Duration::from_secs(60), async {
        while let Some(event) = tokens.next().await {
            if let TokenEvent::Added(token) = event
                && let Some(authenticator) = CtapAuthenticator::new(token, &ui).await
            {
                return Ok::<_, anyhow::Error>(authenticator);
            }
        }
        anyhow::bail!("No FIDO2 authenticator is available")
    })
    .await??;
    let mut authenticator = WebauthnAuthenticator::new(token);
    // The CTAP implementation uses block_in_place to drive its async transport.
    if registration {
        Ok(json!(authenticator.do_registration(
            origin,
            serde_json::from_value(challenge)?
        )?))
    } else {
        Ok(json!(authenticator.do_authentication(
            origin,
            serde_json::from_value(challenge)?
        )?))
    }
}
