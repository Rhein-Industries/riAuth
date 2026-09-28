//! Shared Signals Framework receiver and transmitter.
//!
//! Push delivery only (`urn:ietf:rfc:8935`). Poll, subject add/remove, and
//! verification endpoints are not implemented. Apple Business Manager was not tested.
//!
//! Inbound SETs must be `application/secevent+jwt` with `typ` `secevent+jwt`.
//! SSF SETs use `sub_id`, not the ordinary JWT `sub`/`exp` claims. A bounded
//! issuer/JTI replay window is anchored to their required `iat` claim.
use crate::{
    agent::Principal,
    crypto::{SigningKey, now},
    error::{Error, Result},
    jose::PublicJwks,
    model::{Session, User},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

// Compatibility paths for the existing public records and event constants.
pub(crate) use crate::identity::signals::enqueue;
pub use crate::identity::signals::{
    ACCOUNT_DISABLED, CREDENTIAL_CHANGE, Delivery, PUSH, SESSION_REVOKED, SUPPORTED, Stream,
};
const PUSH_LEGACY: &str = "https://schemas.openid.net/secevent/risc/delivery-method/push";
pub const MAX_ATTEMPTS: u32 = 5;
pub(crate) const MAX_SET_AGE: u64 = 7 * 86_400;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliverySpec {
    pub method: String,
    pub endpoint_url: String,
    #[serde(default)]
    pub authorization_header: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamInput {
    pub id: String,
    pub issuer: String,
    pub audience: String,
    #[serde(default)]
    pub events_requested: BTreeSet<String>,
    #[serde(default)]
    pub events: BTreeSet<String>,
    #[serde(default)]
    pub delivery: Option<DeliverySpec>,
    #[serde(default)]
    pub delivery_method: Option<String>,
    #[serde(default)]
    pub endpoint_url: Option<String>,
    pub jwks: PublicJwks,
    /// External subject -> existing local username. This does not create users.
    #[serde(default)]
    pub subjects: BTreeMap<String, String>,
}

/// Receiver-supplied properties accepted by the advertised SSF 1.0
/// configuration endpoint. Trust keys and local subject bindings are managed
/// separately by an administrator.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationInput {
    #[serde(default)]
    pub events_requested: BTreeSet<String>,
    pub delivery: DeliverySpec,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectBindings {
    /// Canonical SSF subject JSON string (or a legacy bare iss_sub value) to
    /// an existing local username. This always replaces the full binding set.
    pub subjects: BTreeMap<String, String>,
}

pub enum SsfAuth {
    Bearer(String),
    ClientBasic {
        client_id: String,
        client_secret: String,
    },
}

pub(crate) enum Caller {
    Principal(Principal),
    Service(String),
}

/// SSF reads and writes use the caller's transaction. Inbound revocation and
/// outbound retry decisions stay in this protocol module over these records.
pub trait SsfTx {
    fn deliveries(&self) -> Result<Vec<(String, Delivery)>>;
    fn due_deliveries(&self, at: u64, limit: usize) -> Result<Vec<(String, Delivery)>>;
    fn delivery(&self, id: &str) -> Result<Option<Delivery>>;
    fn put_delivery(&self, id: &str, delivery: &Delivery) -> Result<()>;
    fn stream(&self, id: &str) -> Result<Option<Stream>>;
    fn sessions(&self) -> Result<Vec<(String, Session)>>;
    fn put_session(&self, id: &str, session: &Session) -> Result<()>;
    fn queue_user_logout(&self, user_id: &str) -> Result<()>;
    fn users(&self) -> Result<Vec<(String, User)>>;
    fn user(&self, id: &str) -> Result<Option<User>>;
    fn put_user(&self, id: &str, user: &User) -> Result<()>;
    fn audit_event(&self, actor: &str, action: &str, target: &str) -> Result<()>;
    fn active_signing_key(&self) -> Result<SigningKey>;
    fn jti_page(&self) -> Result<Vec<(String, u64)>>;
    fn delete_jti(&self, id: &str) -> Result<()>;
    fn delivery_page(&self) -> Result<Vec<(String, Delivery)>>;
    fn delete_delivery(&self, id: &str) -> Result<()>;
}

pub trait SsfDelivery: Send + 'static {
    fn deliver_once(&self) -> Result<Vec<Value>>;
}

pub fn metadata(issuer: &str) -> Value {
    let base = issuer.trim_end_matches('/');
    json!({
        "spec_version": "1_0",
        "issuer": issuer,
        "jwks_uri": format!("{base}/oauth/jwks"),
        "delivery_methods_supported": [PUSH],
        "configuration_endpoint": format!("{base}/api/ssf/streams"),
        "authorization_schemes": [{"spec_urn": "urn:ietf:rfc:6749"}],
        "events_supported": SUPPORTED,
    })
}

pub fn inbound_push_url(issuer: &str) -> String {
    format!("{}/api/ssf/events", issuer.trim_end_matches('/'))
}

pub(crate) fn normalize_method(value: &str) -> Result<String> {
    if value == PUSH || value == PUSH_LEGACY || value == "push" {
        Ok(PUSH.into())
    } else {
        Err(Error::bad(
            "Only push delivery (urn:ietf:rfc:8935) is supported",
        ))
    }
}

pub(crate) fn push_url(value: &str) -> Result<()> {
    let url = url::Url::parse(value).map_err(|_| Error::bad("Invalid push URL"))?;
    let loopback = url.scheme() == "http"
        && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !(crate::provider::secure_url(&url) || loopback)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.query().is_some()
    {
        return Err(Error::bad(
            "Push URL must be an exact HTTPS URL (HTTP loopback allowed) without credentials or a query",
        ));
    }
    Ok(())
}

pub(crate) fn authorization_header(value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        if value.is_empty() || value.len() > 2048 {
            return Err(Error::bad("Invalid delivery authorization header"));
        }
        reqwest::header::HeaderValue::from_str(value)
            .map_err(|_| Error::bad("Invalid delivery authorization header"))?;
    }
    Ok(())
}

pub(crate) fn transmitter_issuer(value: &str) -> Result<()> {
    if value.len() > 2048 {
        return Err(Error::bad("Transmitter issuer is too long"));
    }
    let url = url::Url::parse(value).map_err(|_| Error::bad("Invalid transmitter issuer"))?;
    let loopback = url.scheme() == "http"
        && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !(url.scheme() == "https" || loopback)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(Error::bad(
            "Transmitter issuer must be a canonical HTTPS URL (HTTP loopback allowed)",
        ));
    }
    Ok(())
}

pub(crate) fn audience(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > 2048 || value.chars().any(char::is_control) {
        return Err(Error::bad(
            "SSF audience must be 1–2048 characters without controls",
        ));
    }
    Ok(())
}

fn subject_key(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(Error::bad(
            "SSF subject must be 1–256 characters without controls",
        ));
    }
    Ok(())
}

/// An SSF subject is identified by its format and all of that format's fields.
/// The serialized form is the stable stream binding key. Legacy plain-string
/// bindings are interpreted only as `iss_sub` under the stream's pinned issuer.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum SubjectId {
    IssSub { iss: String, sub: String },
    Opaque { id: String },
    Email { email: String },
}

impl SubjectId {
    fn validate(&self) -> Result<()> {
        match self {
            Self::IssSub { iss, sub } => {
                if iss.is_empty()
                    || iss.len() > 2048
                    || iss.chars().any(char::is_control)
                    || (iss.contains(':') && url::Url::parse(iss).is_err())
                {
                    return Err(Error::bad("Invalid SSF subject issuer"));
                }
                subject_key(sub)
            }
            Self::Opaque { id } => subject_key(id),
            Self::Email { email } => subject_key(email),
        }
    }

    pub(crate) fn key(&self) -> String {
        serde_json::to_string(self).expect("SSF subject is serializable")
    }

    pub(crate) fn legacy_key(&self, issuer: &str) -> Option<&str> {
        match self {
            Self::IssSub { iss, sub } if iss == issuer => Some(sub),
            _ => None,
        }
    }
}

pub(crate) fn binding_subject(value: &str, issuer: &str) -> Result<SubjectId> {
    let subject = if value.starts_with('{') {
        serde_json::from_str(value).map_err(|_| Error::bad("Invalid SSF subject identifier"))?
    } else {
        SubjectId::IssSub {
            iss: issuer.to_owned(),
            sub: value.to_owned(),
        }
    };
    subject.validate()?;
    Ok(subject)
}

pub(crate) fn view(issuer: &str, stream: &Stream) -> Value {
    json!({
        "stream_id": stream.id,
        "iss": issuer,
        "transmitter_issuer": stream.issuer,
        "aud": stream.audience,
        "delivery": {"method": stream.delivery_method, "endpoint_url": stream.endpoint_url},
        "events_supported": SUPPORTED,
        "events_requested": stream.events,
        "events_delivered": stream.events,
        "subjects": stream.subjects.keys().cloned().collect::<Vec<_>>(),
        "key_ids": stream.jwks.keys.iter().map(|key| key.kid.clone()).collect::<Vec<_>>(),
        "owner": stream.owner,
        "created_at": stream.created_at,
        "inbound_push_url": inbound_push_url(issuer),
        "inbound_content_type": "application/secevent+jwt",
    })
}

pub(crate) fn configuration_view(issuer: &str, stream: &Stream) -> Value {
    let mut value = json!({
        "stream_id": stream.id,
        "iss": issuer,
        "aud": stream.audience,
        "delivery": {"method": stream.delivery_method, "endpoint_url": stream.endpoint_url},
        "events_supported": SUPPORTED,
        "events_requested": stream.events_requested,
        "events_delivered": stream.events,
    });
    if let Some(description) = &stream.description {
        value["description"] = json!(description);
    }
    value
}

pub(crate) fn requested_events(events: &BTreeSet<String>) -> Result<BTreeSet<String>> {
    if events.len() > 32
        || events
            .iter()
            .any(|event| event.is_empty() || event.len() > 2048)
    {
        return Err(Error::bad("Invalid SSF events_requested"));
    }
    Ok(events
        .iter()
        .filter(|event| SUPPORTED.contains(&event.as_str()))
        .cloned()
        .collect())
}

pub(crate) fn validate_description(description: Option<&str>) -> Result<()> {
    if description.is_some_and(|description| {
        description.len() > 1024 || description.chars().any(char::is_control)
    }) {
        return Err(Error::bad("Invalid SSF stream description"));
    }
    Ok(())
}

pub(crate) fn cancel_pending(tx: &impl SsfTx, stream_id: &str) -> Result<()> {
    for (id, mut delivery) in tx.deliveries()? {
        if delivery.stream_id == stream_id && delivery.delivered_at.is_none() && !delivery.stopped {
            delivery.stopped = true;
            tx.put_delivery(&id, &delivery)?;
        }
    }
    Ok(())
}

pub(crate) fn admin_allow(caller: &Caller, stream: &Stream) -> Result<()> {
    match caller {
        Caller::Principal(principal) if !principal.agent => Ok(()),
        Caller::Principal(principal) => {
            principal.require("ssf.manage", &format!("ssf/{}", stream.id))
        }
        Caller::Service(_) => Err(Error::forbidden()),
    }
}

pub(crate) fn config_allow(caller: &Caller, stream: &Stream) -> Result<()> {
    if !stream.standard {
        return Err(Error::forbidden());
    }
    match caller {
        Caller::Principal(principal) if !principal.agent => Ok(()),
        Caller::Principal(principal) if stream.owner == principal.id => {
            principal.require("ssf.configure", &format!("ssf/{}", stream.id))
        }
        Caller::Principal(_) => Err(Error::forbidden()),
        Caller::Service(id) if stream.owner == format!("client:{id}") => Ok(()),
        Caller::Service(_) => Err(Error::forbidden()),
    }
}

pub(crate) fn owner_of(caller: &Caller) -> String {
    match caller {
        Caller::Principal(principal) => principal.id.clone(),
        Caller::Service(id) => format!("client:{id}"),
    }
}

pub(crate) fn validation_failed() -> Error {
    Error::bad("Security event token validation failed")
}

fn delivery_error(code: &'static str) -> Error {
    Error::new(
        axum::http::StatusCode::BAD_REQUEST,
        code,
        "Security event token validation failed",
    )
}

/// The payload is consulted only to identify an RFC 8935 error. It never
/// selects a user or authorizes an event without the normal signature check.
pub(crate) fn unmatched_error(token: &str, streams: &[(String, Stream)]) -> Error {
    let mut segments = token.split('.');
    let (Some(_), Some(payload), Some(_), None) = (
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
    ) else {
        return validation_failed();
    };
    let Ok(payload) = URL_SAFE_NO_PAD.decode(payload) else {
        return validation_failed();
    };
    let Ok(claims) = serde_json::from_slice::<Value>(&payload) else {
        return validation_failed();
    };
    let Some(issuer) = claims["iss"].as_str() else {
        return validation_failed();
    };
    let issuer_streams = streams
        .iter()
        .map(|(_, stream)| stream)
        .filter(|stream| stream.issuer == issuer)
        .collect::<Vec<_>>();
    if issuer_streams.is_empty() {
        return delivery_error("invalid_issuer");
    }
    let audience_streams = issuer_streams
        .into_iter()
        .filter(|stream| match &claims["aud"] {
            Value::String(audience) => audience == &stream.audience,
            Value::Array(audiences) => audiences
                .iter()
                .any(|audience| audience == &stream.audience),
            _ => false,
        })
        .collect::<Vec<_>>();
    if claims.get("aud").is_none() {
        return validation_failed();
    }
    if audience_streams.is_empty() {
        return delivery_error("invalid_audience");
    }
    let Ok(header) = jsonwebtoken::decode_header(token) else {
        return validation_failed();
    };
    if header.typ.as_deref() != Some("secevent+jwt") {
        return validation_failed();
    }
    if audience_streams.iter().all(|stream| {
        !stream
            .jwks
            .keys
            .iter()
            .any(|key| Some(&key.kid) == header.kid.as_ref())
    }) {
        return delivery_error("invalid_key");
    }
    if audience_streams
        .iter()
        .any(|stream| stream.jwks.verify_set_signature(token))
    {
        validation_failed()
    } else {
        delivery_error("authentication_failed")
    }
}

pub(crate) fn verify_set(stream: &Stream, token: &str) -> Result<Value> {
    let header = jsonwebtoken::decode_header(token).map_err(|_| validation_failed())?;
    if header.typ.as_deref() != Some("secevent+jwt")
        || header.jku.is_some()
        || header.jwk.is_some()
        || header.x5u.is_some()
        || header.crit.as_ref().is_some_and(|crit| !crit.is_empty())
    {
        return Err(validation_failed());
    }
    let claims = stream
        .jwks
        .verify_set(token, &stream.issuer, &stream.audience)
        .map_err(|_| validation_failed())?;
    if claims.get("exp").is_some() || claims.get("sub").is_some() {
        return Err(validation_failed());
    }
    let iat = claims["iat"].as_u64().ok_or_else(validation_failed)?;
    if iat > now().saturating_add(60) || now().saturating_sub(iat) > MAX_SET_AGE {
        return Err(validation_failed());
    }
    let jti = claims["jti"].as_str().ok_or_else(validation_failed)?;
    if jti.is_empty() || jti.len() > 128 || !jti.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(validation_failed());
    }
    let events = claims["events"].as_object().ok_or_else(validation_failed)?;
    if events.is_empty() || events.len() > 4 || events.values().any(|event| !event.is_object()) {
        return Err(validation_failed());
    }
    let primary_subject = subject_of(&claims)?;
    for (event, payload) in events {
        if let Some(nested) = payload.get("subject") {
            let nested: SubjectId =
                serde_json::from_value(nested.clone()).map_err(|_| validation_failed())?;
            nested.validate().map_err(|_| validation_failed())?;
            if nested.key() != primary_subject.key() {
                return Err(validation_failed());
            }
        }
        if event == CREDENTIAL_CHANGE {
            let credential_type = payload["credential_type"].as_str();
            let change_type = payload["change_type"].as_str();
            if credential_type.is_none_or(|kind| {
                kind.is_empty() || kind.len() > 128 || kind.chars().any(char::is_control)
            }) || !matches!(change_type, Some("create" | "revoke" | "update" | "delete"))
            {
                return Err(validation_failed());
            }
        }
    }
    Ok(claims)
}

pub(crate) fn subject_of(claims: &Value) -> Result<SubjectId> {
    let subject: SubjectId =
        serde_json::from_value(claims["sub_id"].clone()).map_err(|_| validation_failed())?;
    subject.validate().map_err(|_| validation_failed())?;
    Ok(subject)
}

fn revoke_sessions(tx: &impl SsfTx, user_id: &str) -> Result<()> {
    for (id, mut session) in tx.sessions()? {
        if session.identity.user_id == user_id && !session.revoked {
            session.revoked = true;
            tx.put_session(&id, &session)?;
        }
    }
    tx.queue_user_logout(user_id)
}

fn another_admin(tx: &impl SsfTx, user: &User) -> Result<bool> {
    Ok(tx
        .users()?
        .iter()
        .any(|(_, other)| other.id != user.id && other.admin && other.enabled))
}

pub(crate) fn apply_event(tx: &impl SsfTx, stream: &Stream, user_id: &str, event: &str) -> Result<bool> {
    let mut user = tx
        .user(user_id)?
        .ok_or_else(|| Error::bad("Linked user is missing"))?;
    match event {
        ACCOUNT_DISABLED => {
            if user.admin && user.enabled && !another_admin(tx, &user)? {
                return Ok(false);
            }
            if user.enabled {
                user.enabled = false;
                user.epoch += 1;
            }
            tx.put_user(&user.id, &user)?;
            revoke_sessions(tx, &user.id)?;
            tx.audit_event(
                &format!("ssf:{}:{}", stream.id, stream.owner),
                "ssf.account-disabled",
                &user.id,
            )?;
        }
        SESSION_REVOKED => {
            user.epoch += 1;
            tx.put_user(&user.id, &user)?;
            revoke_sessions(tx, &user.id)?;
            tx.audit_event(
                &format!("ssf:{}:{}", stream.id, stream.owner),
                "ssf.session-revoked",
                &user.id,
            )?;
        }
        CREDENTIAL_CHANGE => {
            user.epoch += 1;
            tx.put_user(&user.id, &user)?;
            revoke_sessions(tx, &user.id)?;
            tx.audit_event(
                &format!("ssf:{}:{}", stream.id, stream.owner),
                "ssf.credential-change",
                &user.id,
            )?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

pub(crate) fn event_body(delivery: &Delivery, issuer: &str, at: u64) -> Value {
    let subject = binding_subject(&delivery.subject, issuer)
        .map(|subject| serde_json::to_value(subject).expect("SSF subject is serializable"))
        .unwrap_or_else(|_| json!({"format": "iss_sub", "iss": issuer, "sub": delivery.subject}));
    let mut body = json!({"subject": subject, "event_timestamp": at});
    if delivery.event == SESSION_REVOKED {
        body["initiating_entity"] = json!("admin");
        body["reason_admin"] = json!({"en": "session revoked"});
    } else if delivery.event == CREDENTIAL_CHANGE {
        body["change_type"] = json!("update");
        let kind = if delivery.credential_type.is_empty() {
            "password"
        } else {
            delivery.credential_type.as_str()
        };
        body["credential_type"] = json!(kind);
    }
    json!({
        "iss": issuer,
        "aud": delivery.audience,
        "iat": at,
        "jti": delivery.jti,
        "sub_id": subject,
        "events": { delivery.event.clone(): body },
    })
}

/// Claim each due delivery before leaving the write transaction for HTTP.
/// The same signing key and attempt number are then carried to the sender.
pub(crate) fn claim_deliveries(
    tx: &impl SsfTx,
) -> Result<Vec<(Delivery, SigningKey, Option<String>)>> {
    let mut ready = Vec::new();
    for (id, mut delivery) in tx.due_deliveries(now(), 32)? {
        if ready.len() == 16 {
            break;
        }
        if delivery.delivered_at.is_some() || delivery.stopped || delivery.next_attempt > now() {
            continue;
        }
        if delivery.attempts >= MAX_ATTEMPTS
            || delivery.created_at.saturating_add(86_400) < now()
        {
            delivery.stopped = true;
            delivery.last_failed = true;
            tx.put_delivery(&id, &delivery)?;
            continue;
        }
        if push_url(&delivery.uri).is_err() {
            delivery.stopped = true;
            delivery.last_failed = true;
            delivery.attempts += 1;
            tx.put_delivery(&id, &delivery)?;
            continue;
        }
        let Some(stream) = tx.stream(&delivery.stream_id)? else {
            delivery.stopped = true;
            tx.put_delivery(&id, &delivery)?;
            continue;
        };
        if stream.endpoint_url != delivery.uri {
            delivery.stopped = true;
            tx.put_delivery(&id, &delivery)?;
            continue;
        }
        delivery.attempts += 1;
        delivery.next_attempt = now().saturating_add(60);
        let key = tx.active_signing_key()?;
        tx.put_delivery(&id, &delivery)?;
        ready.push((delivery, key, stream.authorization_header));
    }
    Ok(ready)
}

/// Ignore stale HTTP completions when a newer attempt or stream change won.
pub(crate) fn finish_delivery(
    tx: &impl SsfTx,
    id: &str,
    attempt: u32,
    status: Option<u16>,
) -> Result<()> {
    let Some(mut delivery) = tx.delivery(id)? else {
        return Ok(());
    };
    if delivery.attempts != attempt || delivery.delivered_at.is_some() || delivery.stopped {
        return Ok(());
    }
    delivery.last_status = status;
    let success = status.is_some_and(|code| (200..300).contains(&code));
    let retry = match status {
        Some(code) if (200..300).contains(&code) => false,
        Some(429) => true,
        Some(code) if (500..600).contains(&code) => true,
        None => true,
        Some(_) => false,
    };
    if success {
        delivery.delivered_at = Some(now());
        delivery.last_failed = false;
    } else if !retry || delivery.attempts >= MAX_ATTEMPTS {
        delivery.last_failed = true;
        delivery.stopped = true;
    } else {
        delivery.last_failed = true;
        let delay = 2u64.saturating_pow(delivery.attempts.min(12)).min(3600);
        delivery.next_attempt = now().saturating_add(delay);
    }
    tx.put_delivery(id, &delivery)
}

pub async fn deliver(core: impl SsfDelivery) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        crate::telemetry::in_activity(crate::telemetry::Activity::SsfDelivery, || {
            core.deliver_once()
        })
    })
    .await
    .map_err(Error::internal)?
    .map(|_| ())
}

pub fn cleanup(tx: &impl SsfTx, at: u64) -> Result<()> {
    for (id, exp) in tx.jti_page()? {
        if exp <= at {
            tx.delete_jti(&id)?;
        }
    }
    for (id, delivery) in tx.delivery_page()? {
        if delivery.created_at.saturating_add(7 * 86_400) < at {
            tx.delete_delivery(&id)?;
        }
    }
    Ok(())
}
