//! Platform SSF Core entry points and concrete transaction operations.

use crate::{
    agent::Principal,
    core::{self, Core, audit},
    crypto::{SigningKey, digest, now},
    error::{Error, Result},
    management::ssf_streams::{admin_caller, config_caller},
    model::{Session, User},
    ssf::*,
    store::Tx,
};
use serde::Serialize;
use serde_json::{Value, json};

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

impl Core {
    pub fn ssf_metadata(&self) -> Value {
        metadata(&self.config.issuer)
    }

    pub fn ssf_create(&self, auth: &SsfAuth, input: StreamInput) -> Result<Value> {
        crate::management::ssf_streams::create_admin(self, auth, input)
    }

    /// Create the advertised receiver-managed transmitter stream. The caller
    /// cannot choose signing trust, local subjects, issuer, audience, or ID.
    pub fn ssf_config_create(&self, auth: &SsfAuth, input: ConfigurationInput) -> Result<Value> {
        crate::management::ssf_streams::create_receiver(self, auth, input)
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
        crate::management::ssf_streams::delete_receiver(self, auth, id)
    }

    pub fn ssf_config_update(&self, auth: &SsfAuth, input: Value, replace: bool) -> Result<Value> {
        crate::management::ssf_streams::update_receiver(self, auth, input, replace)
    }

    pub fn ssf_bind_subjects(
        &self,
        auth: &SsfAuth,
        id: &str,
        input: SubjectBindings,
    ) -> Result<Value> {
        crate::management::ssf_streams::bind_subjects(self, auth, id, input)
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
        crate::management::ssf_streams::delete_admin(self, auth, id)
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

    /// Counts for every stored outbound SSF delivery, plus at most 50 redacted
    /// attention rows. Rows are read one storage page at a time. Stream records
    /// are not loaded, so a deleted stream and an endpoint change are both
    /// `cancelled`. A row with `delivered_at` is `delivered`. Endpoint URLs,
    /// subjects, audiences, JTIs, credential types, and the raw event string
    /// stay on the delivery record. This read does not claim, dispatch, or
    /// change readiness.
    pub fn ssf_delivery_diagnostics(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.management(tx, token, "operations.read", "operations/ssf")?;
            let mut counts = DeliveryCounts::default();
            let mut listed = Vec::with_capacity(DIAGNOSTIC_ITEMS);
            let mut visible_attention = 0u64;
            let mut after: Option<String> = None;
            let page_size = crate::store::maintenance::PAGE;
            loop {
                let page = tx.scan::<Delivery>("ssf_deliveries", after.as_deref(), page_size)?;
                let Some(last_key) = page.last().map(|(key, _)| key.clone()) else {
                    break;
                };
                if after
                    .as_ref()
                    .is_some_and(|previous| last_key.as_str() <= previous.as_str())
                {
                    return Err(Error::internal(
                        "SSF delivery diagnostic page did not advance",
                    ));
                }
                let full = page.len() == page_size;
                after = Some(last_key);
                for (_, row) in page {
                    let class = classify_delivery(&row);
                    count_delivery(&mut counts, class);
                    let visible = delivery_visible(&actor, &row.stream_id);
                    if !visible {
                        counts.withheld = counts.withheld.saturating_add(1);
                    }
                    let Some(action) = attention_action(class) else {
                        continue;
                    };
                    counts.attention = counts.attention.saturating_add(1);
                    if !visible {
                        counts.withheld_attention = counts.withheld_attention.saturating_add(1);
                        continue;
                    }
                    visible_attention = visible_attention.saturating_add(1);
                    retain_delivery(&mut listed, delivery_item(&row, class, action));
                }
                if !full {
                    break;
                }
            }
            if listed.len() < DIAGNOSTIC_ITEMS {
                sort_deliveries(&mut listed);
            }
            let items: Vec<Value> = listed.into_iter().map(|item| item.body).collect();
            Ok(json!({
                "schema_version": "riauth.ssf-delivery-diagnostics/v1",
                "checked_at": now(),
                "affects_readiness": false,
                "limits": { "attention_items": DIAGNOSTIC_ITEMS },
                "counts": counts,
                "listed": items.len(),
                "truncated": visible_attention > u64::try_from(DIAGNOSTIC_ITEMS).unwrap_or(u64::MAX),
                "items": items,
            }))
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
        self.store
            .write(|tx| finish_delivery(tx, id, attempt, status))
    }
}

const DIAGNOSTIC_ITEMS: usize = 50;

#[derive(Clone, Copy)]
enum DeliveryClass {
    Pending,
    Retrying,
    Stopped,
    Cancelled,
    Delivered,
}

#[derive(Default, Serialize)]
struct DeliveryCounts {
    deliveries: u64,
    pending: u64,
    retrying: u64,
    stopped: u64,
    cancelled: u64,
    delivered: u64,
    attention: u64,
    withheld: u64,
    withheld_attention: u64,
}

struct ListedDelivery {
    rank: u8,
    id: String,
    body: Value,
}

fn classify_delivery(row: &Delivery) -> DeliveryClass {
    if row.delivered_at.is_some() {
        DeliveryClass::Delivered
    } else if row.stopped && row.last_failed {
        DeliveryClass::Stopped
    } else if row.stopped {
        DeliveryClass::Cancelled
    } else if row.last_failed {
        DeliveryClass::Retrying
    } else {
        DeliveryClass::Pending
    }
}

fn attention_action(class: DeliveryClass) -> Option<&'static str> {
    Some(match class {
        DeliveryClass::Stopped => "inspect_receiver",
        DeliveryClass::Retrying => "wait_for_retry",
        DeliveryClass::Cancelled => "delivery_cancelled",
        DeliveryClass::Pending | DeliveryClass::Delivered => return None,
    })
}

fn event_token(event: &str) -> &'static str {
    if event == ACCOUNT_DISABLED {
        "account_disabled"
    } else if event == SESSION_REVOKED {
        "session_revoked"
    } else if event == CREDENTIAL_CHANGE {
        "credential_change"
    } else {
        "unknown"
    }
}

fn count_delivery(counts: &mut DeliveryCounts, class: DeliveryClass) {
    counts.deliveries = counts.deliveries.saturating_add(1);
    let slot = match class {
        DeliveryClass::Pending => &mut counts.pending,
        DeliveryClass::Retrying => &mut counts.retrying,
        DeliveryClass::Stopped => &mut counts.stopped,
        DeliveryClass::Cancelled => &mut counts.cancelled,
        DeliveryClass::Delivered => &mut counts.delivered,
    };
    *slot = slot.saturating_add(1);
}

fn delivery_visible(actor: &Principal, stream_id: &str) -> bool {
    let resource = format!("ssf/{stream_id}");
    actor.allows("ssf.configure", &resource) || actor.allows("ssf.manage", &resource)
}

fn delivery_name(class: DeliveryClass) -> &'static str {
    match class {
        DeliveryClass::Pending => "pending",
        DeliveryClass::Retrying => "retrying",
        DeliveryClass::Stopped => "stopped",
        DeliveryClass::Cancelled => "cancelled",
        DeliveryClass::Delivered => "delivered",
    }
}

fn delivery_rank(class: DeliveryClass) -> u8 {
    match class {
        DeliveryClass::Stopped => 0,
        DeliveryClass::Retrying => 1,
        DeliveryClass::Cancelled => 2,
        DeliveryClass::Pending | DeliveryClass::Delivered => 3,
    }
}

fn delivery_item(row: &Delivery, class: DeliveryClass, action: &str) -> ListedDelivery {
    ListedDelivery {
        rank: delivery_rank(class),
        id: row.id.clone(),
        body: json!({
            "id": row.id,
            "stream_id": row.stream_id,
            "delivery_state": delivery_name(class),
            "attempts": row.attempts,
            "next_attempt": row.next_attempt,
            "created_at": row.created_at,
            "last_failed": row.last_failed,
            "last_status": row.last_status,
            "event": event_token(&row.event),
            "next_action": action,
        }),
    }
}

fn sort_deliveries(items: &mut [ListedDelivery]) {
    items.sort_by(|left, right| {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.id.cmp(&right.id))
    });
}

/// Keep the best `DIAGNOSTIC_ITEMS` rows. Worse rows are dropped immediately.
fn retain_delivery(items: &mut Vec<ListedDelivery>, item: ListedDelivery) {
    if items.len() < DIAGNOSTIC_ITEMS {
        items.push(item);
        if items.len() == DIAGNOSTIC_ITEMS {
            sort_deliveries(items);
        }
        return;
    }
    let keep = {
        let worst = items.last().expect("the retained list is full");
        (item.rank, item.id.as_str()) < (worst.rank, worst.id.as_str())
    };
    if !keep {
        return;
    }
    items.pop();
    let pos = items.partition_point(|existing| {
        (existing.rank, existing.id.as_str()) < (item.rank, item.id.as_str())
    });
    items.insert(pos, item);
}
