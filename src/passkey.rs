//! WebAuthn ceremonies persist private challenge state on the server.
#[cfg(feature = "platform")]
use crate::crypto;
use crate::{
    crypto::{digest, now},
    error::{Error, Result},
    identity::require_factor_session,
    model::{Identity, Session, User},
    signin::{FRESH_SECONDS, reauthentication_required},
};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use webauthn_rs::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Credential {
    pub(crate) id: String,
    pub(crate) user_id: String,
    pub(crate) name: String,
    pub(crate) created_at: u64,
    pub(crate) counter: u32,
    pub(crate) key: Passkey,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Registration {
    pub(crate) identity: Identity,
    pub(crate) name: String,
    pub(crate) expires_at: u64,
    pub(crate) state: PasskeyRegistration,
}

/// A new administrator is invisible until two distinct credentials have been verified.
/// The initiating administrator must keep the same fresh, browser-owned MFA session
/// throughout the ceremony. The pending record contains no password or live authority.
#[derive(Serialize, Deserialize)]
pub(crate) struct AdminRegistration {
    #[serde(flatten)]
    pub(crate) enrollment: AdminEnrollment,
    pub(crate) owner_id: String,
    pub(crate) session_id: String,
    pub(crate) session_epoch: u64,
    pub(crate) binding_hash: String,
    pub(crate) expires_at: u64,
}

/// Shared two-credential administrator policy, used by management and first setup.
/// These are pending credentials, never a provisional live identity.
#[derive(Serialize, Deserialize)]
pub(crate) struct AdminEnrollment {
    pub(crate) user: User,
    pub(crate) state: PasskeyRegistration,
    pub(crate) primary: Option<Credential>,
    pub(crate) primary_name: String,
    pub(crate) backup_name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewPasskeyAdmin {
    pub username: String,
    #[serde(default)]
    pub display_name: String,
    pub email: Option<String>,
    pub primary_name: String,
    pub backup_name: String,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Authentication {
    pub(crate) user_id: Option<String>,
    pub(crate) epoch: u64,
    pub(crate) expires_at: u64,
    pub(crate) state: Option<PasskeyAuthentication>,
    pub(crate) transaction: Option<String>,
    /// Bound ceremonies: `portal`, `oidc:{id}`, `saml:{id}` or `workflow:{id}`.
    /// The standalone API finish refuses them.
    #[serde(default)]
    pub(crate) interaction: Option<String>,
    /// Digest of the browser binding or server-owned workflow reservation.
    #[serde(default)]
    pub(crate) binding_hash: Option<String>,
    /// Reauthentication must finish in the session that requested it.
    #[serde(default)]
    pub(crate) session_id: Option<String>,
    /// Usernameless state; the credential names its user.
    #[serde(default)]
    pub(crate) discoverable: Option<DiscoverableAuthentication>,
}

pub(crate) struct BrowserPasskeyContext<'a> {
    pub interaction: &'a str,
    pub binding: Option<&'a str>,
    pub pinned_user: Option<&'a str>,
    pub session_id: Option<&'a str>,
}

pub(crate) fn webauthn_for_issuer(issuer: &str) -> Result<Webauthn> {
    let uri = url::Url::parse(issuer).map_err(Error::internal)?;
    let origin = url::Url::parse(&uri.origin().ascii_serialization()).map_err(Error::internal)?;
    WebauthnBuilder::new(
        uri.domain()
            .ok_or_else(|| Error::bad("Passkeys require an issuer hostname, not an IP address"))?,
        &origin,
    )
    .map_err(|_| Error::bad("Invalid WebAuthn relying-party origin"))?
    .rp_name("riAuth")
    .build()
    .map_err(Error::internal)
}
pub(crate) fn handle(user_id: &str) -> Uuid {
    let hash = Sha256::digest(format!("riauth.webauthn-user/v1\0{user_id}"));
    Uuid::from_bytes(hash[..16].try_into().unwrap())
}
pub(crate) fn credential_key(raw: &[u8]) -> String {
    digest(&URL_SAFE_NO_PAD.encode(raw))
}
pub(crate) fn credential_id(id: &CredentialID) -> String {
    credential_key(id.as_slice())
}

/// Shared credential reads and ceremony cleanup use the caller's transaction.
/// Assembly maps the port to the existing passkey collections.
pub(crate) trait PasskeyTx {
    fn credentials(&self) -> Result<Vec<(String, Credential)>>;
    #[cfg(feature = "platform")]
    fn authentication(&self, key: &str) -> Result<Option<Authentication>>;
    #[cfg(feature = "platform")]
    fn delete_authentication(&self, key: &str) -> Result<()>;
}

/// Credential removal and bounded ceremony cleanup without exposing record shapes.
pub trait PasskeyMaintenance {
    fn credential_ids_for_user(&self, user_id: &str) -> Result<Vec<String>>;
    fn delete_credential(&self, id: &str) -> Result<()>;
    fn pending_page(&self, bucket: &str) -> Result<Vec<(String, Value)>>;
    fn delete_pending(&self, bucket: &str, id: &str) -> Result<()>;
}

pub(crate) fn user_keys(tx: &impl PasskeyTx, user_id: &str) -> Result<Vec<Credential>> {
    Ok(tx
        .credentials()?
        .into_iter()
        .filter(|(_, c)| c.user_id == user_id)
        .map(|(_, c)| c)
        .collect())
}

pub(crate) fn view(c: &Credential) -> Value {
    json!({"id":c.id,"name":c.name,"created_at":c.created_at,"algorithm":c.key.cred_algorithm()})
}
/// Request options as sent to clients: no transports, mediation or extensions, so a challenge
/// never reveals how a key was enrolled and the decoy keeps the real shape.
pub(crate) fn public_request(challenge: &RequestChallengeResponse) -> Value {
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
pub(crate) fn unknown_passkey() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "unknown_passkey",
        "This passkey isn't registered with riAuth. Use another passkey or sign in with your password.",
    )
}

pub(crate) fn require_fresh_factor(user: &User, session: &Session) -> Result<()> {
    if now().saturating_sub(session.identity.auth_time) > FRESH_SECONDS {
        return Err(reauthentication_required());
    }
    require_factor_session(user, session)
}

/// Idempotent cleanup for a server-owned workflow reservation. The ordinary
/// browser cancellation API keeps its existing strict missing-ceremony behavior.
#[cfg(feature = "platform")]
pub(crate) fn discard_workflow_ceremony(
    tx: &impl PasskeyTx,
    ceremony: &str,
    context: BrowserPasskeyContext<'_>,
) -> Result<()> {
    let key = digest(ceremony);
    if let Some(pending) = tx.authentication(&key)? {
        if pending.interaction.as_deref() != Some(context.interaction)
            || pending.user_id.as_deref() != context.pinned_user
            || pending.session_id.as_deref() != context.session_id
            || !context
                .binding
                .zip(pending.binding_hash.as_deref())
                .is_some_and(|(binding, hash)| crypto::constant_eq(&digest(binding), hash))
        {
            return Err(Error::forbidden());
        }
        tx.delete_authentication(&key)?;
    }
    Ok(())
}
pub(crate) fn passkey_list_in(tx: &impl PasskeyTx, user_id: &str) -> Result<Vec<Value>> {
    Ok(user_keys(tx, user_id)?.iter().map(view).collect())
}
pub(crate) fn passkey_count(tx: &impl PasskeyTx, user_id: &str) -> Result<usize> {
    Ok(user_keys(tx, user_id)?.len())
}
pub fn clear(tx: &impl PasskeyMaintenance, user_id: &str) -> Result<()> {
    for id in tx.credential_ids_for_user(user_id)? {
        tx.delete_credential(&id)?;
    }
    Ok(())
}
pub fn cleanup(tx: &impl PasskeyMaintenance, at: u64) -> Result<()> {
    for bucket in ["passkey_registration", "passkey_authentication"] {
        for (id, value) in tx.pending_page(bucket)? {
            if value["expires_at"].as_u64().is_none_or(|exp| exp <= at) {
                tx.delete_pending(bucket, &id)?;
            }
        }
    }
    Ok(())
}
