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
    core::{self, Core, audit, user_by_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    jose::PublicJwks,
    model::{Grant, Session, User},
    store::Tx,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const ACCOUNT_DISABLED: &str =
    "https://schemas.openid.net/secevent/risc/event-type/account-disabled";
pub const SESSION_REVOKED: &str =
    "https://schemas.openid.net/secevent/caep/event-type/session-revoked";
pub const CREDENTIAL_CHANGE: &str =
    "https://schemas.openid.net/secevent/caep/event-type/credential-change";
pub const PUSH: &str = "urn:ietf:rfc:8935";
const PUSH_LEGACY: &str = "https://schemas.openid.net/secevent/risc/delivery-method/push";
pub const SUPPORTED: &[&str] = &[ACCOUNT_DISABLED, SESSION_REVOKED, CREDENTIAL_CHANGE];
pub const MAX_ATTEMPTS: u32 = 5;
const MAX_SET_AGE: u64 = 7 * 86_400;

#[derive(Clone, Serialize, Deserialize)]
pub struct Stream {
    pub id: String,
    /// Peer transmitter `iss` required on inbound SETs.
    pub issuer: String,
    /// Audience required on inbound SETs and sent on outbound SETs.
    pub audience: String,
    pub events: BTreeSet<String>,
    #[serde(default)]
    pub events_requested: BTreeSet<String>,
    pub delivery_method: String,
    pub endpoint_url: String,
    #[serde(default)]
    pub authorization_header: Option<String>,
    pub jwks: PublicJwks,
    /// External subject -> local user id. Never returned by the management API.
    pub subjects: BTreeMap<String, String>,
    pub owner: String,
    pub created_at: u64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub standard: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Delivery {
    pub id: String,
    pub stream_id: String,
    pub uri: String,
    pub event: String,
    pub subject: String,
    pub audience: String,
    pub credential_type: String,
    pub created_at: u64,
    pub next_attempt: u64,
    pub attempts: u32,
    pub delivered_at: Option<u64>,
    pub last_status: Option<u16>,
    #[serde(default)]
    pub last_failed: bool,
    #[serde(default)]
    pub stopped: bool,
    pub jti: String,
}

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

enum Caller {
    Principal(Principal),
    Service(String),
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

fn normalize_method(value: &str) -> Result<String> {
    if value == PUSH || value == PUSH_LEGACY || value == "push" {
        Ok(PUSH.into())
    } else {
        Err(Error::bad(
            "Only push delivery (urn:ietf:rfc:8935) is supported",
        ))
    }
}

fn push_url(value: &str) -> Result<()> {
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

fn authorization_header(value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        if value.is_empty() || value.len() > 2048 {
            return Err(Error::bad("Invalid delivery authorization header"));
        }
        reqwest::header::HeaderValue::from_str(value)
            .map_err(|_| Error::bad("Invalid delivery authorization header"))?;
    }
    Ok(())
}

fn require_protected_authorization(core: &Core, value: Option<&str>) -> Result<()> {
    if value.is_some() && !core.store.encrypted_at_rest() {
        return Err(Error::bad(
            "Delivery authorization requires configured database encryption",
        ));
    }
    Ok(())
}

fn transmitter_issuer(value: &str) -> Result<()> {
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

fn audience(value: &str) -> Result<()> {
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
enum SubjectId {
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

    fn key(&self) -> String {
        serde_json::to_string(self).expect("SSF subject is serializable")
    }

    fn legacy_key(&self, issuer: &str) -> Option<&str> {
        match self {
            Self::IssSub { iss, sub } if iss == issuer => Some(sub),
            _ => None,
        }
    }
}

fn binding_subject(value: &str, issuer: &str) -> Result<SubjectId> {
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

fn view(issuer: &str, stream: &Stream) -> Value {
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

fn configuration_view(issuer: &str, stream: &Stream) -> Value {
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

fn requested_events(events: &BTreeSet<String>) -> Result<BTreeSet<String>> {
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

fn validate_description(description: Option<&str>) -> Result<()> {
    if description.is_some_and(|description| {
        description.len() > 1024 || description.chars().any(char::is_control)
    }) {
        return Err(Error::bad("Invalid SSF stream description"));
    }
    Ok(())
}

fn cancel_pending(tx: &Tx<'_>, stream_id: &str) -> Result<()> {
    for (id, mut delivery) in tx.list::<Delivery>("ssf_deliveries")? {
        if delivery.stream_id == stream_id && delivery.delivered_at.is_none() && !delivery.stopped {
            delivery.stopped = true;
            tx.put("ssf_deliveries", &id, &delivery)?;
        }
    }
    Ok(())
}

fn admin_caller(core: &Core, tx: &Tx<'_>, auth: &SsfAuth) -> Result<Caller> {
    match auth {
        SsfAuth::Bearer(token) => {
            let principal = core.principal(tx, token)?;
            if principal.agent
                && !principal
                    .permissions
                    .iter()
                    .any(|permission| permission.action == "ssf.manage")
            {
                return Err(Error::forbidden());
            }
            Ok(Caller::Principal(principal))
        }
        SsfAuth::ClientBasic { .. } => Err(Error::forbidden()),
    }
}

fn config_caller(core: &Core, tx: &Tx<'_>, auth: &SsfAuth) -> Result<Caller> {
    match auth {
        SsfAuth::ClientBasic { .. } => Err(Error::forbidden()),
        SsfAuth::Bearer(token) => match core.principal(tx, token) {
            Ok(principal) => {
                if principal.agent
                    && !principal
                        .permissions
                        .iter()
                        .any(|permission| permission.action == "ssf.configure")
                {
                    return Err(Error::forbidden());
                }
                Ok(Caller::Principal(principal))
            }
            Err(error) if error.status == axum::http::StatusCode::UNAUTHORIZED => {
                let grant = tx
                    .get::<Grant>("access", &digest(token))?
                    .ok_or_else(Error::unauthorized)?;
                if grant.identity.is_some()
                    || grant.exchange.is_some()
                    || grant.confirmation_jkt.is_some()
                    || grant.resource.is_some()
                    || !grant.scopes.contains("ssf.configure")
                {
                    return Err(Error::forbidden());
                }
                let client = core.validate_grant(tx, &grant)?;
                if !client.service {
                    return Err(Error::forbidden());
                }
                Ok(Caller::Service(client.id))
            }
            Err(error) => Err(error),
        },
    }
}

fn admin_allow(caller: &Caller, stream: &Stream) -> Result<()> {
    match caller {
        Caller::Principal(principal) if !principal.agent => Ok(()),
        Caller::Principal(principal) => {
            principal.require("ssf.manage", &format!("ssf/{}", stream.id))
        }
        Caller::Service(_) => Err(Error::forbidden()),
    }
}

fn config_allow(caller: &Caller, stream: &Stream) -> Result<()> {
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

fn owner_of(caller: &Caller) -> String {
    match caller {
        Caller::Principal(principal) => principal.id.clone(),
        Caller::Service(id) => format!("client:{id}"),
    }
}

pub(crate) fn enqueue(
    tx: &Tx<'_>,
    user_id: &str,
    event: &str,
    credential_type: &str,
) -> Result<()> {
    if !SUPPORTED.contains(&event) {
        return Ok(());
    }
    if !tx.mark_security_event(user_id, event, credential_type) {
        return Ok(());
    }
    for (_, stream) in tx.list::<Stream>("ssf_streams")? {
        if stream.delivery_method != PUSH || !stream.events.contains(event) {
            continue;
        }
        for (subject, linked) in &stream.subjects {
            if linked != user_id {
                continue;
            }
            let id = crypto::id();
            let delivery = Delivery {
                id: id.clone(),
                stream_id: stream.id.clone(),
                uri: stream.endpoint_url.clone(),
                event: event.into(),
                subject: subject.clone(),
                audience: stream.audience.clone(),
                credential_type: credential_type.into(),
                created_at: now(),
                next_attempt: now(),
                attempts: 0,
                delivered_at: None,
                last_status: None,
                last_failed: false,
                stopped: false,
                jti: id.clone(),
            };
            tx.put("ssf_deliveries", &delivery.id, &delivery)?;
        }
    }
    Ok(())
}

fn validation_failed() -> Error {
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
fn unmatched_error(token: &str, streams: &[(String, Stream)]) -> Error {
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

fn verify_set(stream: &Stream, token: &str) -> Result<Value> {
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

fn subject_of(claims: &Value) -> Result<SubjectId> {
    let subject: SubjectId =
        serde_json::from_value(claims["sub_id"].clone()).map_err(|_| validation_failed())?;
    subject.validate().map_err(|_| validation_failed())?;
    Ok(subject)
}

fn revoke_sessions(tx: &Tx<'_>, user_id: &str) -> Result<()> {
    for (id, mut session) in tx.list::<Session>("sessions")? {
        if session.identity.user_id == user_id && !session.revoked {
            session.revoked = true;
            tx.put("sessions", &id, &session)?;
        }
    }
    crate::logout::queue_user(tx, user_id)
}

fn another_admin(tx: &Tx<'_>, user: &User) -> Result<bool> {
    Ok(tx
        .list::<User>("users")?
        .iter()
        .any(|(_, other)| other.id != user.id && other.admin && other.enabled))
}

fn apply_event(tx: &Tx<'_>, stream: &Stream, user_id: &str, event: &str) -> Result<bool> {
    let mut user = tx
        .get::<User>("users", user_id)?
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
            tx.put("users", &user.id, &user)?;
            revoke_sessions(tx, &user.id)?;
            audit(
                tx,
                &format!("ssf:{}:{}", stream.id, stream.owner),
                "ssf.account-disabled",
                &user.id,
            )?;
        }
        SESSION_REVOKED => {
            user.epoch += 1;
            tx.put("users", &user.id, &user)?;
            revoke_sessions(tx, &user.id)?;
            audit(
                tx,
                &format!("ssf:{}:{}", stream.id, stream.owner),
                "ssf.session-revoked",
                &user.id,
            )?;
        }
        CREDENTIAL_CHANGE => {
            user.epoch += 1;
            tx.put("users", &user.id, &user)?;
            revoke_sessions(tx, &user.id)?;
            audit(
                tx,
                &format!("ssf:{}:{}", stream.id, stream.owner),
                "ssf.credential-change",
                &user.id,
            )?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn event_body(delivery: &Delivery, issuer: &str, at: u64) -> Value {
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

impl Core {
    pub fn ssf_metadata(&self) -> Value {
        metadata(&self.config.issuer)
    }

    pub fn ssf_create(&self, auth: &SsfAuth, input: StreamInput) -> Result<Value> {
        core::validate_name(&input.id)?;
        transmitter_issuer(&input.issuer)?;
        audience(&input.audience)?;
        input.jwks.validate()?;
        let mut events = if input.events_requested.is_empty() {
            input.events.clone()
        } else {
            input.events_requested.clone()
        };
        if !input.events.is_empty() && !input.events_requested.is_empty() {
            events = &input.events_requested | &input.events;
        }
        if events.is_empty()
            || events.len() > SUPPORTED.len()
            || events
                .iter()
                .any(|event| !SUPPORTED.contains(&event.as_str()))
        {
            return Err(Error::bad(
                "Subscribe only to account-disabled, session-revoked, and credential-change",
            ));
        }
        let (method, endpoint) = if let Some(delivery) = &input.delivery {
            (delivery.method.clone(), delivery.endpoint_url.clone())
        } else {
            (
                input.delivery_method.clone().unwrap_or_default(),
                input.endpoint_url.clone().unwrap_or_default(),
            )
        };
        let method = normalize_method(&method)?;
        push_url(&endpoint)?;
        let authorization = input
            .delivery
            .as_ref()
            .and_then(|delivery| delivery.authorization_header.clone());
        authorization_header(authorization.as_deref())?;
        require_protected_authorization(self, authorization.as_deref())?;
        if input.subjects.len() > 64 {
            return Err(Error::bad("At most 64 subjects may be linked to a stream"));
        }
        self.store.write(|tx| {
            let actor = admin_caller(self, tx, auth)?;
            let Caller::Principal(principal) = &actor else {
                return Err(Error::forbidden());
            };
            principal.require("ssf.manage", &format!("ssf/{}", input.id))?;
            if tx.get::<Stream>("ssf_streams", &input.id)?.is_some() {
                return Err(Error::conflict("SSF stream already exists"));
            }
            if tx.list::<Stream>("ssf_streams")?.len() >= 32 {
                return Err(Error::bad("At most 32 SSF streams are allowed"));
            }
            let mut subjects = BTreeMap::new();
            for (external, username) in &input.subjects {
                let subject = binding_subject(external, &input.issuer)?;
                core::validate_name(username)?;
                let user = user_by_name(tx, username)?;
                if subjects.insert(subject.key(), user.id).is_some() {
                    return Err(Error::bad("Duplicate SSF subject binding"));
                }
            }
            let stream = Stream {
                id: input.id.clone(),
                issuer: input.issuer.clone(),
                audience: input.audience.clone(),
                events,
                events_requested: if input.events_requested.is_empty() {
                    input.events.clone()
                } else {
                    input.events_requested.clone()
                },
                delivery_method: method,
                endpoint_url: endpoint,
                authorization_header: authorization,
                jwks: input.jwks.clone(),
                subjects,
                owner: owner_of(&actor),
                created_at: now(),
                description: None,
                standard: false,
            };
            tx.put("ssf_streams", &stream.id, &stream)?;
            audit(tx, &stream.owner, "ssf.stream.create", &stream.id)?;
            Ok(view(&self.config.issuer, &stream))
        })
    }

    /// Create the advertised receiver-managed transmitter stream. The caller
    /// cannot choose signing trust, local subjects, issuer, audience, or ID.
    pub fn ssf_config_create(&self, auth: &SsfAuth, input: ConfigurationInput) -> Result<Value> {
        let delivered = requested_events(&input.events_requested)?;
        let method = normalize_method(&input.delivery.method)?;
        push_url(&input.delivery.endpoint_url)?;
        authorization_header(input.delivery.authorization_header.as_deref())?;
        require_protected_authorization(self, input.delivery.authorization_header.as_deref())?;
        validate_description(input.description.as_deref())?;
        self.store.write(|tx| {
            let actor = config_caller(self, tx, auth)?;
            if let Caller::Principal(principal) = &actor {
                principal.require("ssf.configure", "*")?;
            }
            let streams = tx.list::<Stream>("ssf_streams")?;
            if streams.len() >= 32 {
                return Err(Error::bad("At most 32 SSF streams are allowed"));
            }
            let owner = owner_of(&actor);
            if !matches!(&actor, Caller::Principal(principal) if !principal.agent) {
                if streams
                    .iter()
                    .filter(|(_, stream)| stream.standard && stream.owner == owner)
                    .count()
                    >= 8
                {
                    return Err(Error::bad(
                        "At most 8 receiver streams are allowed per owner",
                    ));
                }
                if streams
                    .iter()
                    .filter(|(_, stream)| {
                        stream.standard
                            && (stream.owner.starts_with("agent:")
                                || stream.owner.starts_with("client:"))
                    })
                    .count()
                    >= 24
                {
                    return Err(Error::bad("Receiver stream capacity is exhausted"));
                }
            }
            let id = crypto::id();
            let audience = owner.clone();
            let stream = Stream {
                id: id.clone(),
                issuer: self.config.issuer.clone(),
                audience,
                events: delivered,
                events_requested: input.events_requested,
                delivery_method: method,
                endpoint_url: input.delivery.endpoint_url,
                authorization_header: input.delivery.authorization_header,
                jwks: PublicJwks::default(),
                subjects: BTreeMap::new(),
                owner,
                created_at: now(),
                description: input.description,
                standard: true,
            };
            tx.put("ssf_streams", &id, &stream)?;
            audit(tx, &stream.owner, "ssf.stream.create", &id)?;
            Ok(configuration_view(&self.config.issuer, &stream))
        })
    }

    pub fn ssf_config_read(&self, auth: &SsfAuth, id: Option<&str>) -> Result<Value> {
        if let Some(id) = id {
            core::validate_name(id)?;
        }
        self.store.read(|tx| {
            let actor = config_caller(self, tx, auth)?;
            if let Some(id) = id {
                let stream = tx
                    .get::<Stream>("ssf_streams", id)?
                    .filter(|stream| stream.standard)
                    .ok_or_else(|| Error::missing("SSF stream not found"))?;
                config_allow(&actor, &stream)?;
                return Ok(configuration_view(&self.config.issuer, &stream));
            }
            Ok(Value::Array(
                tx.list::<Stream>("ssf_streams")?
                    .into_iter()
                    .filter_map(|(_, stream)| {
                        config_allow(&actor, &stream)
                            .is_ok()
                            .then(|| configuration_view(&self.config.issuer, &stream))
                    })
                    .collect(),
            ))
        })
    }

    pub fn ssf_config_delete(&self, auth: &SsfAuth, id: &str) -> Result<()> {
        core::validate_name(id)?;
        self.store.write(|tx| {
            let actor = config_caller(self, tx, auth)?;
            let stream = tx
                .get::<Stream>("ssf_streams", id)?
                .filter(|stream| stream.standard)
                .ok_or_else(|| Error::missing("SSF stream not found"))?;
            config_allow(&actor, &stream)?;
            cancel_pending(tx, id)?;
            tx.delete("ssf_streams", id)?;
            audit(tx, &owner_of(&actor), "ssf.stream.delete", id)
        })
    }

    pub fn ssf_config_update(&self, auth: &SsfAuth, input: Value, replace: bool) -> Result<Value> {
        let fields = input
            .as_object()
            .ok_or_else(|| Error::bad("SSF stream configuration must be an object"))?;
        let id = fields
            .get("stream_id")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::bad("stream_id is required"))?;
        core::validate_name(id)?;
        if fields.keys().any(|field| {
            !matches!(
                field.as_str(),
                "stream_id"
                    | "iss"
                    | "aud"
                    | "events_supported"
                    | "events_requested"
                    | "events_delivered"
                    | "delivery"
                    | "description"
            )
        }) {
            return Err(Error::bad("Unknown SSF stream configuration property"));
        }
        let requested = fields
            .get("events_requested")
            .map(|value| {
                serde_json::from_value::<BTreeSet<String>>(value.clone())
                    .map_err(|_| Error::bad("Invalid events_requested"))
            })
            .transpose()?;
        let delivery = fields
            .get("delivery")
            .map(|value| {
                serde_json::from_value::<DeliverySpec>(value.clone())
                    .map_err(|_| Error::bad("Invalid delivery configuration"))
            })
            .transpose()?;
        let authorization_supplied = fields
            .get("delivery")
            .and_then(Value::as_object)
            .is_some_and(|delivery| delivery.contains_key("authorization_header"));
        authorization_header(
            delivery
                .as_ref()
                .and_then(|delivery| delivery.authorization_header.as_deref()),
        )?;
        require_protected_authorization(
            self,
            delivery
                .as_ref()
                .and_then(|delivery| delivery.authorization_header.as_deref()),
        )?;
        let description = fields
            .get("description")
            .map(|value| {
                if value.is_null() {
                    Ok(None)
                } else {
                    value
                        .as_str()
                        .map(|value| Some(value.to_owned()))
                        .ok_or_else(|| Error::bad("Invalid SSF stream description"))
                }
            })
            .transpose()?;
        validate_description(description.as_ref().and_then(|value| value.as_deref()))?;
        if replace && delivery.is_none() {
            return Err(Error::bad("PUT requires delivery configuration"));
        }
        let supported: BTreeSet<String> =
            SUPPORTED.iter().map(|value| (*value).to_owned()).collect();
        self.store.write(|tx| {
            let actor = config_caller(self, tx, auth)?;
            let mut stream = tx
                .get::<Stream>("ssf_streams", id)?
                .filter(|stream| stream.standard)
                .ok_or_else(|| Error::missing("SSF stream not found"))?;
            config_allow(&actor, &stream)?;
            if fields
                .get("iss")
                .is_some_and(|value| value != &json!(self.config.issuer))
                || fields
                    .get("aud")
                    .is_some_and(|value| value != &json!(stream.audience))
                || fields.get("events_supported").is_some_and(|value| {
                    serde_json::from_value::<BTreeSet<String>>(value.clone()).ok()
                        != Some(supported.clone())
                })
                || fields
                    .get("events_delivered")
                    .is_some_and(|value| value != &json!(stream.events))
            {
                return Err(Error::bad("Transmitter-supplied SSF property mismatch"));
            }
            let next_requested = requested.clone().unwrap_or_else(|| {
                if replace {
                    BTreeSet::new()
                } else {
                    stream.events_requested.clone()
                }
            });
            let next_events = requested_events(&next_requested)?;
            let (next_method, next_endpoint, next_authorization) = if let Some(delivery) = &delivery
            {
                let method = normalize_method(&delivery.method)?;
                push_url(&delivery.endpoint_url)?;
                let authorization = if replace
                    || authorization_supplied
                    || delivery.endpoint_url != stream.endpoint_url
                {
                    delivery.authorization_header.clone()
                } else {
                    stream.authorization_header.clone()
                };
                (method, delivery.endpoint_url.clone(), authorization)
            } else {
                (
                    stream.delivery_method.clone(),
                    stream.endpoint_url.clone(),
                    stream.authorization_header.clone(),
                )
            };
            if next_events != stream.events
                || next_endpoint != stream.endpoint_url
                || next_authorization != stream.authorization_header
            {
                cancel_pending(tx, id)?;
            }
            stream.events_requested = next_requested;
            stream.events = next_events;
            stream.delivery_method = next_method;
            stream.endpoint_url = next_endpoint;
            stream.authorization_header = next_authorization;
            if let Some(description) = &description {
                stream.description = description.clone();
            } else if replace {
                stream.description = None;
            }
            tx.put("ssf_streams", id, &stream)?;
            audit(tx, &owner_of(&actor), "ssf.stream.update", id)?;
            Ok(configuration_view(&self.config.issuer, &stream))
        })
    }

    pub fn ssf_bind_subjects(
        &self,
        auth: &SsfAuth,
        id: &str,
        input: SubjectBindings,
    ) -> Result<Value> {
        core::validate_name(id)?;
        if input.subjects.len() > 64 {
            return Err(Error::bad("At most 64 subjects may be linked to a stream"));
        }
        self.store.write(|tx| {
            let actor = admin_caller(self, tx, auth)?;
            let mut stream = tx
                .get::<Stream>("ssf_streams", id)?
                .ok_or_else(|| Error::missing("SSF stream not found"))?;
            admin_allow(&actor, &stream)?;
            let mut subjects = BTreeMap::new();
            for (external, username) in &input.subjects {
                let subject = binding_subject(external, &stream.issuer)?;
                core::validate_name(username)?;
                let user = user_by_name(tx, username)?;
                if subjects.insert(subject.key(), user.id).is_some() {
                    return Err(Error::bad("Duplicate SSF subject binding"));
                }
            }
            if stream.subjects != subjects {
                cancel_pending(tx, id)?;
                stream.subjects = subjects;
                tx.put("ssf_streams", id, &stream)?;
                audit(tx, &owner_of(&actor), "ssf.stream.subjects", id)?;
            }
            Ok(view(&self.config.issuer, &stream))
        })
    }

    pub fn ssf_list(&self, auth: &SsfAuth) -> Result<Value> {
        self.store.read(|tx| {
            let actor = admin_caller(self, tx, auth)?;
            let streams = tx
                .list::<Stream>("ssf_streams")?
                .into_iter()
                .filter_map(|(_, stream)| {
                    admin_allow(&actor, &stream)
                        .is_ok()
                        .then(|| view(&self.config.issuer, &stream))
                })
                .collect::<Vec<_>>();
            Ok(json!({
                "streams": streams,
                "inbound_push_url": inbound_push_url(&self.config.issuer),
                "inbound_content_type": "application/secevent+jwt",
            }))
        })
    }

    pub fn ssf_delete(&self, auth: &SsfAuth, id: &str) -> Result<Value> {
        core::validate_name(id)?;
        self.store.write(|tx| {
            let actor = admin_caller(self, tx, auth)?;
            let stream = tx
                .get::<Stream>("ssf_streams", id)?
                .ok_or_else(|| Error::missing("SSF stream not found"))?;
            admin_allow(&actor, &stream)?;
            cancel_pending(tx, id)?;
            tx.delete("ssf_streams", id)?;
            audit(tx, &owner_of(&actor), "ssf.stream.delete", id)?;
            Ok(json!({"deleted": true, "stream_id": id}))
        })
    }

    /// Accept a push SET. Unknown linked subjects are ignored with HTTP 202 at the route.
    /// The raw token is not stored or audited.
    pub fn accept_set(&self, token: &str) -> Result<Value> {
        if token.len() > 16_384 || token.chars().any(char::is_control) {
            return Err(validation_failed());
        }
        self.store.write(|tx| {
            let mut matched = Vec::new();
            let streams = tx.list::<Stream>("ssf_streams")?;
            for (_, stream) in &streams {
                if let Ok(claims) = verify_set(stream, token) {
                    matched.push((stream.clone(), claims));
                }
            }
            let Some((_, claims)) = matched.first() else {
                return Err(unmatched_error(token, &streams));
            };
            let iss = claims["iss"].as_str().unwrap_or_default();
            let jti = claims["jti"].as_str().unwrap_or_default();
            let iat = claims["iat"].as_u64().unwrap_or(0);
            let replay = digest(&format!("{iss}\0{jti}"));
            if tx
                .get::<u64>("ssf_jti", &replay)?
                .is_some_and(|until| until > now())
            {
                return Ok(json!({"accepted": true}));
            }
            tx.put(
                "ssf_jti",
                &replay,
                &iat.saturating_add(MAX_SET_AGE).saturating_add(60),
            )?;
            let subject = subject_of(claims)?;
            let subject_key = subject.key();
            let events = claims["events"]
                .as_object()
                .map(|events| events.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            let mut applied = false;
            for (stream, _) in &matched {
                let Some(user_id) = stream.subjects.get(&subject_key).or_else(|| {
                    subject
                        .legacy_key(&stream.issuer)
                        .and_then(|legacy| stream.subjects.get(legacy))
                }) else {
                    audit(
                        tx,
                        &format!("ssf:{}:{}", stream.id, stream.owner),
                        "ssf.ignored",
                        &stream.id,
                    )?;
                    continue;
                };
                let mut acted = false;
                for event in &events {
                    if !stream.events.contains(event) || !SUPPORTED.contains(&event.as_str()) {
                        continue;
                    }
                    if apply_event(tx, stream, user_id, event)? {
                        acted = true;
                        applied = true;
                    }
                }
                if !acted {
                    audit(
                        tx,
                        &format!("ssf:{}:{}", stream.id, stream.owner),
                        "ssf.ignored",
                        &stream.id,
                    )?;
                }
            }
            let _ = applied;
            Ok(json!({"accepted": true}))
        })
    }

    pub fn deliver_once(&self) -> Result<Vec<Value>> {
        let pending = self.store.write(|tx| {
            let mut ready = Vec::new();
            for (id, mut delivery) in tx.due::<Delivery>("ssf_deliveries", now(), 32)? {
                if ready.len() == 16 {
                    break;
                }
                if delivery.delivered_at.is_some()
                    || delivery.stopped
                    || delivery.next_attempt > now()
                {
                    continue;
                }
                if delivery.attempts >= MAX_ATTEMPTS
                    || delivery.created_at.saturating_add(86_400) < now()
                {
                    delivery.stopped = true;
                    delivery.last_failed = true;
                    tx.put("ssf_deliveries", &id, &delivery)?;
                    continue;
                }
                if push_url(&delivery.uri).is_err() {
                    delivery.stopped = true;
                    delivery.last_failed = true;
                    delivery.attempts += 1;
                    tx.put("ssf_deliveries", &id, &delivery)?;
                    continue;
                }
                let Some(stream) = tx.get::<Stream>("ssf_streams", &delivery.stream_id)? else {
                    delivery.stopped = true;
                    tx.put("ssf_deliveries", &id, &delivery)?;
                    continue;
                };
                if stream.endpoint_url != delivery.uri {
                    delivery.stopped = true;
                    tx.put("ssf_deliveries", &id, &delivery)?;
                    continue;
                }
                delivery.attempts += 1;
                delivery.next_attempt = now().saturating_add(60);
                let key = core::keys(tx)?.active;
                tx.put("ssf_deliveries", &id, &delivery)?;
                ready.push((delivery, key, stream.authorization_header));
            }
            Ok(ready)
        })?;
        let http = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(Error::internal)?;
        let mut results = Vec::new();
        for (delivery, key, authorization) in pending {
            let claims = event_body(&delivery, &self.config.issuer, delivery.created_at);
            let status = match self.sign_jwt(&key, &claims, "secevent+jwt") {
                Ok(token) => {
                    let mut request = http
                        .post(&delivery.uri)
                        .header("content-type", "application/secevent+jwt")
                        .header("accept", "application/json")
                        .body(token);
                    if let Some(authorization) = authorization {
                        let mut value = reqwest::header::HeaderValue::from_str(&authorization)
                            .map_err(Error::internal)?;
                        value.set_sensitive(true);
                        request = request.header(reqwest::header::AUTHORIZATION, value);
                    }
                    request
                        .send()
                        .ok()
                        .map(|response| response.status().as_u16())
                }
                Err(_) => None,
            };
            self.finish_delivery(&delivery.id, delivery.attempts, status)?;
            if status.is_none_or(|code| !(200..300).contains(&code)) {
                tracing::warn!(
                    stream_id = %delivery.stream_id,
                    attempt = delivery.attempts,
                    status = status.unwrap_or(0),
                    "SSF delivery not accepted"
                );
            }
            results
                .push(json!({"id": delivery.id, "status": status, "attempt": delivery.attempts}));
        }
        Ok(results)
    }

    fn finish_delivery(&self, id: &str, attempt: u32, status: Option<u16>) -> Result<()> {
        self.store.write(|tx| {
            let Some(mut delivery) = tx.get::<Delivery>("ssf_deliveries", id)? else {
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
            tx.put("ssf_deliveries", id, &delivery)
        })
    }
}

pub async fn deliver(core: Core) -> Result<()> {
    tokio::task::spawn_blocking(move || core.deliver_once())
        .await
        .map_err(Error::internal)?
        .map(|_| ())
}

pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, exp) in tx.maintenance_page::<u64>("ssf_jti")? {
        if exp <= at {
            tx.delete("ssf_jti", &id)?;
        }
    }
    for (id, delivery) in tx.maintenance_page::<Delivery>("ssf_deliveries")? {
        if delivery.created_at.saturating_add(7 * 86_400) < at {
            tx.delete("ssf_deliveries", &id)?;
        }
    }
    Ok(())
}
