//! SSF stream configuration writers. Both the receiver-managed endpoint and
//! the administrator/CLI endpoint enter this transaction and receipt boundary.
//! Desired-state administrator streams use the same insert and subject
//! replacement on the caller's plan transaction, without a delivery secret.
//! The browser admin page has no SSF stream editor yet. A future browser
//! adapter must send a stable Idempotency-Key and its reviewed revision.

use crate::{
    agent::Principal,
    core::{Core, audit, user_by_name, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    jose::PublicJwks,
    model::Grant,
    ssf::*,
    state::SsfStreamSpec,
    store::Tx,
};
use axum::http::StatusCode;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn require_protected_authorization(core: &Core, value: Option<&str>) -> Result<()> {
    if value.is_some() && !core.store.encrypted_at_rest() {
        return Err(Error::bad(
            "Delivery authorization requires configured database encryption",
        ));
    }
    Ok(())
}
pub(crate) fn admin_caller(core: &Core, tx: &Tx<'_>, auth: &SsfAuth) -> Result<Caller> {
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

pub(crate) fn config_caller(core: &Core, tx: &Tx<'_>, auth: &SsfAuth) -> Result<Caller> {
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

struct StreamReceipt {
    key: String,
    fingerprint: String,
    scope: Value,
}

impl StreamReceipt {
    fn current(
        tx: &Tx<'_>,
        actor: &Caller,
        operation: &str,
        target: Option<&str>,
    ) -> Result<Option<Self>> {
        let Some(context) = crate::context::current() else {
            return Ok(None);
        };
        let Some(key) = context.idempotency_key else {
            return Ok(None);
        };
        let owner = owner_of(actor);
        let authority = match actor {
            Caller::Principal(principal) if principal.delegated => json!({
                "grants": principal.grants,
                "generation": tx.get::<u64>("human_grant_generations", &principal.id)?.unwrap_or(0),
            }),
            Caller::Principal(principal) => json!(principal.permissions),
            Caller::Service(_) => json!("live-ssf-configure-grant"),
        };
        Ok(Some(Self {
            key: digest(&format!("ssf.stream\0{owner}\0{key}")),
            fingerprint: context.fingerprint,
            scope: json!({
                "ssf_stream": operation,
                "owner": owner,
                "target": target,
                "authority": authority,
            }),
        }))
    }

    fn replay(&self, tx: &Tx<'_>) -> Result<Option<Value>> {
        crate::context::replay_receipt(tx, &self.key, &self.fingerprint, &self.scope)
    }

    fn save(self, tx: &Tx<'_>, result: &Value) -> Result<()> {
        crate::context::save_receipt(tx, &self.key, self.fingerprint, self.scope, result)
    }
}

fn require_revision(tx: &Tx<'_>, actor: &Caller) -> Result<()> {
    if let Some(context) = crate::context::current() {
        if matches!(actor, Caller::Principal(principal) if (principal.agent || principal.delegated) && context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "Scoped mutations require If-Match with the current revision",
            ));
        }
        if let Some(expected) = context.revision {
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            if revision != expected {
                return Err(Error::conflict("Configuration revision changed"));
            }
        }
    }
    Ok(())
}

fn write(
    core: &Core,
    auth: &SsfAuth,
    administrative: bool,
    operation: &str,
    target: Option<&str>,
    authorize: impl FnOnce(&Tx<'_>, &Caller) -> Result<()>,
    apply: impl FnOnce(&Tx<'_>, &Caller) -> Result<Value>,
) -> Result<Value> {
    core.store.write(|tx| {
        let actor = if administrative {
            admin_caller(core, tx, auth)?
        } else {
            config_caller(core, tx, auth)?
        };
        // This check precedes receipt replay. A revoked token or permission
        // cannot recover the old result, including a deleted stream's receipt.
        authorize(tx, &actor)?;
        let receipt = StreamReceipt::current(tx, &actor, operation, target)?;
        if let Some(result) = receipt
            .as_ref()
            .map(|receipt| receipt.replay(tx))
            .transpose()?
            .flatten()
        {
            return Ok(result);
        }
        require_revision(tx, &actor)?;
        let result = apply(tx, &actor)?;
        if let Some(receipt) = receipt {
            receipt.save(tx, &result)?;
        }
        Ok(result)
    })
}

fn require_config_target(tx: &Tx<'_>, actor: &Caller, id: &str) -> Result<()> {
    if let Some(stream) = tx.get::<Stream>("ssf_streams", id)? {
        if !stream.standard {
            return Err(Error::missing("SSF stream not found"));
        }
        config_allow(actor, &stream)
    } else {
        // A deleted stream has no current row to authorize against. Exact-key
        // delete replay still needs the actor's live, exact resource scope.
        match actor {
            Caller::Principal(principal) => {
                principal.require("ssf.configure", &format!("ssf/{id}"))
            }
            Caller::Service(_) => Ok(()),
        }
    }
}

struct PreparedAdmin {
    events: BTreeSet<String>,
    events_requested: BTreeSet<String>,
    method: String,
    endpoint: String,
    authorization: Option<String>,
}

fn prepare_admin_delivery(core: &Core, input: &StreamInput) -> Result<PreparedAdmin> {
    validate_name(&input.id)?;
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
    require_protected_authorization(core, authorization.as_deref())?;
    if input.subjects.len() > 64 {
        return Err(Error::bad("At most 64 subjects may be linked to a stream"));
    }
    let events_requested = if input.events_requested.is_empty() {
        input.events.clone()
    } else {
        input.events_requested.clone()
    };
    Ok(PreparedAdmin {
        events,
        events_requested,
        method,
        endpoint,
        authorization,
    })
}

fn resolve_subject_ids(
    tx: &Tx<'_>,
    issuer: &str,
    subjects: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    if subjects.len() > 64 {
        return Err(Error::bad("At most 64 subjects may be linked to a stream"));
    }
    let mut resolved = BTreeMap::new();
    for (external, username) in subjects {
        let subject = binding_subject(external, issuer)?;
        validate_name(username)?;
        let user = user_by_name(tx, username)?;
        if resolved.insert(subject.key(), user.id).is_some() {
            return Err(Error::bad("Duplicate SSF subject binding"));
        }
    }
    Ok(resolved)
}

fn canonical_subjects(
    issuer: &str,
    subjects: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    if subjects.len() > 64 {
        return Err(Error::bad("At most 64 subjects may be linked to a stream"));
    }
    let mut canonical = BTreeMap::new();
    for (external, username) in subjects {
        let subject = binding_subject(external, issuer)?;
        validate_name(username)?;
        if canonical.insert(subject.key(), username.clone()).is_some() {
            return Err(Error::bad("Duplicate SSF subject binding"));
        }
    }
    Ok(canonical)
}

fn insert_admin_stream(
    tx: &Tx<'_>,
    actor: &Caller,
    input: &StreamInput,
    prepared: &PreparedAdmin,
) -> Result<Stream> {
    if tx.get::<Stream>("ssf_streams", &input.id)?.is_some() {
        return Err(Error::conflict("SSF stream already exists"));
    }
    if tx.list::<Stream>("ssf_streams")?.len() >= 32 {
        return Err(Error::bad("At most 32 SSF streams are allowed"));
    }
    let stream = Stream {
        id: input.id.clone(),
        issuer: input.issuer.clone(),
        audience: input.audience.clone(),
        events: prepared.events.clone(),
        events_requested: prepared.events_requested.clone(),
        delivery_method: prepared.method.clone(),
        endpoint_url: prepared.endpoint.clone(),
        authorization_header: prepared.authorization.clone(),
        jwks: input.jwks.clone(),
        subjects: resolve_subject_ids(tx, &input.issuer, &input.subjects)?,
        owner: owner_of(actor),
        created_at: now(),
        description: None,
        standard: false,
    };
    tx.put("ssf_streams", &stream.id, &stream)?;
    Ok(stream)
}

/// Replace the stored binding map. Pending deliveries are cancelled only when
/// the map changes, matching `bind_subjects`. The caller owns the audit.
fn persist_subject_replacement(
    tx: &Tx<'_>,
    stream: &mut Stream,
    subjects: BTreeMap<String, String>,
) -> Result<bool> {
    if stream.subjects == subjects {
        return Ok(false);
    }
    cancel_pending(tx, &stream.id)?;
    stream.subjects = subjects;
    tx.put("ssf_streams", &stream.id, stream)?;
    Ok(true)
}

pub(crate) fn create_admin(core: &Core, auth: &SsfAuth, input: StreamInput) -> Result<Value> {
    let prepared = prepare_admin_delivery(core, &input)?;
    let id = input.id.clone();
    write(
        core,
        auth,
        true,
        "admin.create",
        Some(&id),
        |_, actor| match actor {
            Caller::Principal(principal) => principal.require("ssf.manage", &format!("ssf/{id}")),
            Caller::Service(_) => Err(Error::forbidden()),
        },
        |tx, actor| {
            let stream = insert_admin_stream(tx, actor, &input, &prepared)?;
            audit(tx, &stream.owner, "ssf.stream.create", &stream.id)?;
            Ok(view(&core.config.issuer, &stream))
        },
    )
}

pub(crate) fn create_receiver(
    core: &Core,
    auth: &SsfAuth,
    input: ConfigurationInput,
) -> Result<Value> {
    let delivered = requested_events(&input.events_requested)?;
    let method = normalize_method(&input.delivery.method)?;
    receiver_push_url(&input.delivery.endpoint_url)?;
    authorization_header(input.delivery.authorization_header.as_deref())?;
    require_protected_authorization(core, input.delivery.authorization_header.as_deref())?;
    validate_description(input.description.as_deref())?;
    write(
        core,
        auth,
        false,
        "receiver.create",
        None,
        |_, actor| {
            if let Caller::Principal(principal) = actor {
                principal.require("ssf.configure", "*")?;
            }
            Ok(())
        },
        |tx, actor| {
            let streams = tx.list::<Stream>("ssf_streams")?;
            if streams.len() >= 32 {
                return Err(Error::bad("At most 32 SSF streams are allowed"));
            }
            let owner = owner_of(actor);
            if !matches!(actor, Caller::Principal(principal) if !principal.agent && !principal.delegated)
            {
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
            let stream = Stream {
                id: id.clone(),
                issuer: core.config.issuer.clone(),
                audience: owner.clone(),
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
            Ok(configuration_view(&core.config.issuer, &stream))
        },
    )
}

pub(crate) fn delete_receiver(core: &Core, auth: &SsfAuth, id: &str) -> Result<()> {
    validate_name(id)?;
    write(
        core,
        auth,
        false,
        "receiver.delete",
        Some(id),
        |tx, actor| require_config_target(tx, actor, id),
        |tx, actor| {
            let stream = tx
                .get::<Stream>("ssf_streams", id)?
                .filter(|stream| stream.standard)
                .ok_or_else(|| Error::missing("SSF stream not found"))?;
            config_allow(actor, &stream)?;
            cancel_pending(tx, id)?;
            tx.delete("ssf_streams", id)?;
            audit(tx, &owner_of(actor), "ssf.stream.delete", id)?;
            Ok(Value::Null)
        },
    )?;
    Ok(())
}

pub(crate) fn delete_admin(core: &Core, auth: &SsfAuth, id: &str) -> Result<Value> {
    validate_name(id)?;
    write(
        core,
        auth,
        true,
        "admin.delete",
        Some(id),
        |_, actor| match actor {
            Caller::Principal(principal) => principal.require("ssf.manage", &format!("ssf/{id}")),
            Caller::Service(_) => Err(Error::forbidden()),
        },
        |tx, actor| {
            let stream = tx
                .get::<Stream>("ssf_streams", id)?
                .ok_or_else(|| Error::missing("SSF stream not found"))?;
            admin_allow(actor, &stream)?;
            cancel_pending(tx, id)?;
            tx.delete("ssf_streams", id)?;
            audit(tx, &owner_of(actor), "ssf.stream.delete", id)?;
            Ok(json!({"deleted": true, "stream_id": id}))
        },
    )
}

pub(crate) fn bind_subjects(
    core: &Core,
    auth: &SsfAuth,
    id: &str,
    input: SubjectBindings,
) -> Result<Value> {
    validate_name(id)?;
    if input.subjects.len() > 64 {
        return Err(Error::bad("At most 64 subjects may be linked to a stream"));
    }
    write(
        core,
        auth,
        true,
        "admin.subjects",
        Some(id),
        |tx, actor| {
            let stream = tx
                .get::<Stream>("ssf_streams", id)?
                .ok_or_else(|| Error::missing("SSF stream not found"))?;
            admin_allow(actor, &stream)
        },
        |tx, actor| {
            let mut stream = tx
                .get::<Stream>("ssf_streams", id)?
                .ok_or_else(|| Error::missing("SSF stream not found"))?;
            admin_allow(actor, &stream)?;
            let subjects = resolve_subject_ids(tx, &stream.issuer, &input.subjects)?;
            if persist_subject_replacement(tx, &mut stream, subjects)? {
                audit(tx, &owner_of(actor), "ssf.stream.subjects", id)?;
            }
            Ok(view(&core.config.issuer, &stream))
        },
    )
}

pub(crate) fn update_receiver(
    core: &Core,
    auth: &SsfAuth,
    input: Value,
    replace: bool,
) -> Result<Value> {
    let fields = input
        .as_object()
        .ok_or_else(|| Error::bad("SSF stream configuration must be an object"))?;
    let id = fields
        .get("stream_id")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::bad("stream_id is required"))?;
    validate_name(id)?;
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
        core,
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
    let supported: BTreeSet<String> = SUPPORTED.iter().map(|value| (*value).to_owned()).collect();
    write(
        core,
        auth,
        false,
        if replace {
            "receiver.put"
        } else {
            "receiver.patch"
        },
        Some(id),
        |tx, actor| require_config_target(tx, actor, id),
        |tx, actor| {
            let mut stream = tx
                .get::<Stream>("ssf_streams", id)?
                .filter(|stream| stream.standard)
                .ok_or_else(|| Error::missing("SSF stream not found"))?;
            config_allow(actor, &stream)?;
            if fields
                .get("iss")
                .is_some_and(|value| value != &json!(core.config.issuer))
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
                receiver_push_url(&delivery.endpoint_url)?;
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
            receiver_push_url(&next_endpoint)?;
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
            audit(tx, &owner_of(actor), "ssf.stream.update", id)?;
            Ok(configuration_view(&core.config.issuer, &stream))
        },
    )
}

pub(crate) struct ManifestStreamEffect {
    pub action: &'static str,
    pub before: Value,
    pub after: Value,
}

fn stream_input_from_spec(spec: &SsfStreamSpec) -> StreamInput {
    StreamInput {
        id: spec.id.clone(),
        issuer: spec.issuer.clone(),
        audience: spec.audience.clone(),
        events_requested: spec.events_requested.clone(),
        events: BTreeSet::new(),
        delivery: None,
        delivery_method: Some(spec.delivery_method.clone()),
        endpoint_url: Some(spec.endpoint_url.clone()),
        jwks: spec.jwks.clone(),
        subjects: spec.subjects.clone(),
    }
}

fn manifest_spec(tx: &Tx<'_>, stream: &Stream) -> Result<SsfStreamSpec> {
    let mut subjects = BTreeMap::new();
    for (subject, user_id) in &stream.subjects {
        let user = tx
            .get::<crate::model::User>("users", user_id)?
            .ok_or_else(|| {
                Error::conflict("SSF subject binding has no local user; export refused")
            })?;
        subjects.insert(subject.clone(), user.username);
    }
    Ok(SsfStreamSpec {
        id: stream.id.clone(),
        issuer: stream.issuer.clone(),
        audience: stream.audience.clone(),
        events_requested: stream.events.clone(),
        delivery_method: stream.delivery_method.clone(),
        endpoint_url: stream.endpoint_url.clone(),
        jwks: stream.jwks.clone(),
        subjects,
    })
}

fn normalized_spec(spec: &SsfStreamSpec, prepared: &PreparedAdmin) -> Result<SsfStreamSpec> {
    Ok(SsfStreamSpec {
        id: spec.id.clone(),
        issuer: spec.issuer.clone(),
        audience: spec.audience.clone(),
        events_requested: prepared.events.clone(),
        delivery_method: prepared.method.clone(),
        endpoint_url: prepared.endpoint.clone(),
        jwks: spec.jwks.clone(),
        subjects: canonical_subjects(&spec.issuer, &spec.subjects)?,
    })
}

fn configuration_matches(stream: &Stream, spec: &SsfStreamSpec, prepared: &PreparedAdmin) -> bool {
    stream.issuer == spec.issuer
        && stream.audience == spec.audience
        && stream.events == prepared.events
        && stream.delivery_method == prepared.method
        && stream.endpoint_url == prepared.endpoint
        && stream.jwks == spec.jwks
}

/// Administrator streams only. The direct API keeps its receipt, revision, and
/// `ssf.stream.create` / `ssf.stream.subjects` audits. This path persists with
/// the same insert and subject replacement and lets desired-state emit one
/// `ssf.stream.reconcile`. Preview writes nothing. A stored authorization
/// header is never read into the manifest and is never cleared.
pub(crate) fn reconcile_manifest_stream(
    core: &Core,
    tx: &Tx<'_>,
    actor: &Principal,
    spec: &SsfStreamSpec,
    preview: bool,
) -> Result<Option<ManifestStreamEffect>> {
    if actor.delegated {
        return Err(Error::forbidden());
    }
    let input = stream_input_from_spec(spec);
    let prepared = prepare_admin_delivery(core, &input)?;
    actor.require("ssf.manage", &format!("ssf/{}", spec.id))?;
    let normalized = normalized_spec(spec, &prepared)?;
    let after = serde_json::to_value(&normalized).map_err(Error::internal)?;
    match tx.get::<Stream>("ssf_streams", &spec.id)? {
        Some(stream) if stream.standard => Err(Error::conflict(
            "Receiver-managed SSF streams stay on the SSF configuration API",
        )),
        Some(stream) => {
            if !configuration_matches(&stream, spec, &prepared) {
                return Err(Error::conflict(
                    "SSF administrator stream configuration is immutable after create",
                ));
            }
            let resolved = resolve_subject_ids(tx, &stream.issuer, &normalized.subjects)?;
            if stream.subjects == resolved {
                return Ok(None);
            }
            let before =
                serde_json::to_value(manifest_spec(tx, &stream)?).map_err(Error::internal)?;
            if !preview {
                let mut stream = stream;
                persist_subject_replacement(tx, &mut stream, resolved)?;
            }
            Ok(Some(ManifestStreamEffect {
                action: "update",
                before,
                after,
            }))
        }
        None => {
            resolve_subject_ids(tx, &spec.issuer, &spec.subjects)?;
            if tx.list::<Stream>("ssf_streams")?.len() >= 32 {
                return Err(Error::bad("At most 32 SSF streams are allowed"));
            }
            if !preview {
                insert_admin_stream(tx, &Caller::Principal(actor.clone()), &input, &prepared)?;
            }
            Ok(Some(ManifestStreamEffect {
                action: "create",
                before: Value::Null,
                after,
            }))
        }
    }
}

/// Non-secret administrator streams this actor may manage. Delegated humans
/// receive none. Standard receiver streams and delivery authorization stay out.
pub(crate) fn export_admin_streams(tx: &Tx<'_>, actor: &Principal) -> Result<Vec<SsfStreamSpec>> {
    if actor.delegated {
        return Ok(Vec::new());
    }
    let mut rows = tx.list::<Stream>("ssf_streams")?;
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    let mut specs = Vec::new();
    for (_, stream) in rows {
        if stream.standard || !actor.allows("ssf.manage", &format!("ssf/{}", stream.id)) {
            continue;
        }
        specs.push(manifest_spec(tx, &stream)?);
    }
    Ok(specs)
}
