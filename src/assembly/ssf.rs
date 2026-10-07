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
use std::collections::BTreeSet;

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

    pub fn claim_ssf_deliveries(&self) -> Result<Vec<Delivery>> {
        Ok(self
            .claim_ssf_batch(16, &BTreeSet::new())?
            .into_iter()
            .map(|claim| claim.delivery)
            .collect())
    }

    fn claim_ssf_batch(
        &self,
        limit: usize,
        excluded: &BTreeSet<String>,
    ) -> Result<Vec<ClaimedDelivery>> {
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
        self.store.write(|tx| claim_deliveries(tx, limit, excluded))
    }

    /// Commit this worker's pin before it POSTs. False means the lease was
    /// replaced, already pinned, expired, or the stream was cancelled or moved,
    /// and the caller must not send.
    pub fn begin_ssf_dispatch(&self, id: &str, lease: &str) -> Result<bool> {
        self.store.write(|tx| begin_dispatch(tx, id, lease))
    }

    pub fn deliver_once(&self) -> Result<Vec<Value>> {
        let operator_http = delivery_client(false)?;
        let receiver_http = delivery_client(true)?;
        self.deliver_ssf_pass(|delivery, key, authorization, receiver_managed| {
            let http = if receiver_managed {
                &receiver_http
            } else {
                &operator_http
            };
            let claims = event_body(delivery, &self.config.issuer, delivery.created_at);
            let status = match self.sign_jwt(key, &claims, "secevent+jwt") {
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
            Ok(status)
        })
    }

    fn deliver_ssf_pass(
        &self,
        mut send: impl FnMut(
            &Delivery,
            &crate::crypto::SigningKey,
            Option<String>,
            bool,
        ) -> Result<Option<u16>>,
    ) -> Result<Vec<Value>> {
        let mut results = Vec::new();
        let mut claimed = BTreeSet::new();
        for _ in 0..16 {
            let Some(ClaimedDelivery {
                delivery,
                key,
                authorization,
                receiver_managed,
            }) = self.claim_ssf_batch(1, &claimed)?.into_iter().next()
            else {
                break;
            };
            claimed.insert(delivery.id.clone());
            let admitted = match delivery.lease.as_deref() {
                Some(lease) => self.begin_ssf_dispatch(&delivery.id, lease)?,
                None => false,
            };
            if !admitted {
                continue;
            }
            let status = send(&delivery, &key, authorization, receiver_managed)?;
            self.finish_ssf_delivery(&delivery.id, delivery.attempts, status)?;
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

    pub fn finish_ssf_delivery(&self, id: &str, attempt: u32, status: Option<u16>) -> Result<()> {
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

#[cfg(all(test, feature = "test-support"))]
mod delivery_pass_tests {
    use super::*;
    use crate::{
        config::Config,
        crypto::{set_test_time, with_test_time},
        jose::PublicJwks,
        model::NewUser,
    };

    const AT: u64 = 1_700_000_000;

    fn fixture() -> (tempfile::TempDir, Core) {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: "ssf-pass-test-password".into(),
                email: None,
                display_name: "Admin".into(),
                admin: true,
            },
        )
        .unwrap();
        (dir, core)
    }

    fn seed(core: &Core, count: usize) -> Vec<Delivery> {
        let stream = Stream {
            id: "pass".into(),
            issuer: "http://127.0.0.1:9".into(),
            audience: "subscriber".into(),
            events: [ACCOUNT_DISABLED.into()].into(),
            events_requested: Default::default(),
            delivery_method: PUSH.into(),
            endpoint_url: "http://127.0.0.1:9/events".into(),
            authorization_header: None,
            jwks: PublicJwks { keys: Vec::new() },
            subjects: Default::default(),
            owner: "admin".into(),
            created_at: AT,
            description: None,
            standard: false,
        };
        let rows: Vec<_> = (0..count)
            .map(|index| Delivery {
                id: format!("pass-{index}"),
                stream_id: stream.id.clone(),
                uri: stream.endpoint_url.clone(),
                event: ACCOUNT_DISABLED.into(),
                subject: "test-subject".into(),
                audience: stream.audience.clone(),
                credential_type: "password".into(),
                created_at: AT,
                next_attempt: AT - u64::try_from(count - index).unwrap(),
                attempts: 0,
                delivered_at: None,
                last_status: None,
                last_failed: false,
                stopped: false,
                jti: format!("pass-jti-{index}"),
                lease: None,
                dispatch_started: None,
            })
            .collect();
        core.store
            .write(|tx| {
                tx.put("ssf_streams", &stream.id, &stream)?;
                tx.put("ssf_jti", "pass-replay-sentinel", &42u64)?;
                for row in &rows {
                    tx.put("ssf_deliveries", &row.id, row)?;
                }
                Ok(())
            })
            .unwrap();
        rows
    }

    fn stored(core: &Core, id: &str) -> Value {
        core.store.get("ssf_deliveries", id).unwrap().unwrap()
    }

    fn queue(core: &Core) -> (u64, u64) {
        let stats = core
            .store
            .read(|tx| tx.queue_stats("ssf_deliveries", now()))
            .unwrap();
        (stats.pending, stats.failed)
    }

    fn assert_local_intent_unchanged(core: &Core, audit: &[(String, Value)]) {
        assert!(core.store.list::<Value>("audit").unwrap().as_slice() == audit);
        assert_eq!(
            core.store
                .get::<u64>("ssf_jti", "pass-replay-sentinel")
                .unwrap(),
            Some(42)
        );
    }

    #[test]
    fn just_in_time_claim_keeps_later_rows_unleased_until_send() {
        let (_dir, core) = fixture();
        with_test_time(AT, || {
            let rows = seed(&core, 17);
            let audit = core.store.list::<Value>("audit").unwrap();
            assert_eq!(queue(&core), (17, 0));
            let mut sends = 0;
            let results = core
                .deliver_ssf_pass(|delivery, _key, _authorization, receiver_managed| {
                    assert!(!receiver_managed);
                    assert_eq!(delivery.id, rows[sends].id);
                    assert_eq!(delivery.attempts, 1);
                    assert_eq!(delivery.next_attempt, now() + LEASE_SECONDS);
                    assert!(delivery.lease.is_some());
                    assert_eq!(stored(&core, &delivery.id)["dispatch_started"], true);
                    for row in &rows[sends + 1..] {
                        assert!(stored(&core, &row.id) == serde_json::to_value(row).unwrap());
                    }
                    if sends == 0 {
                        set_test_time(AT + 61);
                    }
                    sends += 1;
                    Ok(Some(204))
                })
                .unwrap();
            assert_eq!(sends, 16);
            assert_eq!(results.len(), 16);
            for (index, row) in rows[..16].iter().enumerate() {
                assert_eq!(
                    results[index],
                    json!({"id": row.id, "status": 204, "attempt": 1})
                );
                let mut expected = row.clone();
                expected.attempts = 1;
                expected.next_attempt = AT + if index == 0 { 60 } else { 121 };
                expected.delivered_at = Some(AT + 61);
                expected.last_status = Some(204);
                assert!(stored(&core, &row.id) == serde_json::to_value(expected).unwrap());
            }
            assert!(stored(&core, &rows[16].id) == serde_json::to_value(&rows[16]).unwrap());
            assert_eq!(queue(&core), (1, 0));
            assert_local_intent_unchanged(&core, &audit);
        });
    }

    #[test]
    fn one_pass_does_not_reclaim_a_failed_delivery_after_other_send_advances_clock() {
        let (_dir, core) = fixture();
        with_test_time(AT, || {
            let rows = seed(&core, 2);
            let audit = core.store.list::<Value>("audit").unwrap();
            let mut ids = Vec::new();
            let mut failed = None;
            let mut first_lease = None;
            let results = core
                .deliver_ssf_pass(|delivery, _key, _authorization, receiver_managed| {
                    assert!(!receiver_managed);
                    ids.push(delivery.id.clone());
                    assert_eq!(delivery.attempts, 1);
                    if ids.len() == 1 {
                        assert_eq!(delivery.id, rows[0].id);
                        first_lease = delivery.lease.clone();
                        Ok(Some(500))
                    } else {
                        assert_eq!(ids.len(), 2, "each row gets at most one claim per pass");
                        assert_eq!(delivery.id, rows[1].id);
                        failed = Some(stored(&core, &rows[0].id));
                        set_test_time(AT + 61);
                        Ok(Some(204))
                    }
                })
                .unwrap();
            assert_eq!(ids, vec![rows[0].id.clone(), rows[1].id.clone()]);
            assert_eq!(results.len(), 2);
            assert_eq!(results[0]["status"], 500);
            assert_eq!(results[1]["status"], 204);
            let mut expected = rows[0].clone();
            expected.attempts = 1;
            expected.next_attempt = AT + 2;
            expected.last_status = Some(500);
            expected.last_failed = true;
            assert!(stored(&core, &rows[0].id) == serde_json::to_value(expected).unwrap());
            assert!(failed.as_ref() == Some(&stored(&core, &rows[0].id)));
            assert_eq!(queue(&core), (1, 1));

            let mut retries = 0;
            let retry = core
                .deliver_ssf_pass(|delivery, _key, _authorization, receiver_managed| {
                    assert!(!receiver_managed);
                    retries += 1;
                    assert_eq!(delivery.id, rows[0].id);
                    assert_eq!(delivery.attempts, 2);
                    assert_eq!(delivery.next_attempt, AT + 121);
                    assert!(delivery.lease.is_some());
                    assert!(delivery.lease != first_lease);
                    assert_eq!(stored(&core, &delivery.id)["dispatch_started"], true);
                    Ok(Some(204))
                })
                .unwrap();
            assert_eq!(retries, 1);
            assert_eq!(retry.len(), 1);
            assert_eq!(retry[0]["attempt"], 2);
            assert_eq!(queue(&core), (0, 0));
            assert_local_intent_unchanged(&core, &audit);
        });
    }

    #[test]
    fn receiver_delivery_keeps_dispatch_lease_and_public_transport_policy() {
        let (_dir, core) = fixture();
        with_test_time(AT, || {
            let rows = seed(&core, 1);
            core.store
                .write(|tx| {
                    let mut stream = tx.get::<Stream>("ssf_streams", "pass")?.unwrap();
                    stream.standard = true;
                    stream.endpoint_url = "https://receiver.example/events".into();
                    tx.put("ssf_streams", &stream.id, &stream)?;
                    let mut delivery = rows[0].clone();
                    delivery.uri = stream.endpoint_url;
                    tx.put("ssf_deliveries", &delivery.id, &delivery)
                })
                .unwrap();
            let mut sends = 0;
            let results = core
                .deliver_ssf_pass(|delivery, _key, _authorization, receiver_managed| {
                    assert!(receiver_managed);
                    assert_eq!(delivery.attempts, 1);
                    assert!(delivery.lease.is_some());
                    assert_eq!(stored(&core, &delivery.id)["dispatch_started"], true);
                    sends += 1;
                    Ok(Some(204))
                })
                .unwrap();
            assert_eq!(sends, 1);
            assert_eq!(results.len(), 1);
            let delivered = stored(&core, &rows[0].id);
            assert_eq!(delivered["delivered_at"], AT);
            assert!(delivered["lease"].is_null());
            assert!(delivered["dispatch_started"].is_null());
        });
    }

    #[test]
    fn manual_batch_keeps_sixteen_claims_and_original_expiry_guards() {
        let (_dir, core) = fixture();
        with_test_time(AT, || {
            let rows = seed(&core, 17);
            let audit = core.store.list::<Value>("audit").unwrap();
            let claimed = core.claim_ssf_deliveries().unwrap();
            assert_eq!(claimed.len(), 16);
            for (index, row) in claimed.iter().enumerate() {
                assert_eq!(row.id, rows[index].id);
                assert_eq!(row.attempts, 1);
                assert_eq!(row.next_attempt, AT + 60);
                assert!(row.lease.is_some());
                assert!(row.dispatch_started.is_none());
            }
            assert!(stored(&core, &rows[16].id) == serde_json::to_value(&rows[16]).unwrap());
            let first = &claimed[0];
            let lease = first.lease.as_deref().unwrap();
            assert!(!core.begin_ssf_dispatch(&first.id, "wrong-owner").unwrap());
            assert!(core.begin_ssf_dispatch(&first.id, lease).unwrap());
            assert!(!core.begin_ssf_dispatch(&first.id, lease).unwrap());
            set_test_time(AT + 60);
            assert!(!core.begin_ssf_dispatch(&first.id, lease).unwrap());
            assert_eq!(stored(&core, &first.id)["attempts"], 1);
            assert_eq!(stored(&core, &first.id)["next_attempt"], AT + 60);
            assert_eq!(queue(&core), (17, 0));
            assert_local_intent_unchanged(&core, &audit);
        });
    }
}
