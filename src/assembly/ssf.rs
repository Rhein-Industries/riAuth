//! Platform SSF Core entry points and concrete transaction operations.

use crate::{
    core::{self, Core, audit, user_by_name},
    crypto::{self, digest, now, SigningKey},
    error::{Error, Result},
    jose::PublicJwks,
    model::{Grant, Session, User},
    ssf::*,
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

impl SsfTx for Tx<'_> {
    fn deliveries(&self) -> Result<Vec<(String, Delivery)>> {
        self.list("ssf_deliveries")
    }

    fn due_deliveries(&self, at: u64, limit: usize) -> Result<Vec<(String, Delivery)>> {
        self.due("ssf_deliveries", at, limit)
    }

    fn delivery(&self, id: &str) -> Result<Option<Delivery>> {
        self.get("ssf_deliveries", id)
    }

    fn put_delivery(&self, id: &str, delivery: &Delivery) -> Result<()> {
        self.put("ssf_deliveries", id, delivery)
    }

    fn stream(&self, id: &str) -> Result<Option<Stream>> {
        self.get("ssf_streams", id)
    }

    fn sessions(&self) -> Result<Vec<(String, Session)>> {
        self.list("sessions")
    }

    fn put_session(&self, id: &str, session: &Session) -> Result<()> {
        self.put("sessions", id, session)
    }

    fn queue_user_logout(&self, user_id: &str) -> Result<()> {
        crate::logout::queue_user(self, user_id)
    }

    fn users(&self) -> Result<Vec<(String, User)>> {
        self.list("users")
    }

    fn user(&self, id: &str) -> Result<Option<User>> {
        self.get("users", id)
    }

    fn put_user(&self, id: &str, user: &User) -> Result<()> {
        self.put("users", id, user)
    }

    fn audit_event(&self, actor: &str, action: &str, target: &str) -> Result<()> {
        audit(self, actor, action, target)
    }

    fn active_signing_key(&self) -> Result<SigningKey> {
        Ok(core::keys(self)?.active)
    }

    fn jti_page(&self) -> Result<Vec<(String, u64)>> {
        self.maintenance_page("ssf_jti")
    }

    fn delete_jti(&self, id: &str) -> Result<()> {
        self.delete("ssf_jti", id)
    }

    fn delivery_page(&self) -> Result<Vec<(String, Delivery)>> {
        self.maintenance_page("ssf_deliveries")
    }

    fn delete_delivery(&self, id: &str) -> Result<()> {
        self.delete("ssf_deliveries", id)
    }
}

impl SsfDelivery for Core {
    fn deliver_once(&self) -> Result<Vec<Value>> {
        Core::deliver_once(self)
    }
}

fn require_protected_authorization(core: &Core, value: Option<&str>) -> Result<()> {
    if value.is_some() && !core.store.encrypted_at_rest() {
        return Err(Error::bad(
            "Delivery authorization requires configured database encryption",
        ));
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
        // The due index commits with each delivery. An empty snapshot needs no
        // writer; a concurrent enqueue will be picked up by a later pass. Treat
        // this only as a hint: reread claims, stream state and retries below.
        if self
            .store
            .read(|tx| tx.due_deliveries(now(), 1))?
            .is_empty()
        {
            return Ok(Vec::new());
        }
        let pending = self.store.write(|tx| claim_deliveries(tx))?;
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
        self.store.write(|tx| finish_delivery(tx, id, attempt, status))
    }
}
