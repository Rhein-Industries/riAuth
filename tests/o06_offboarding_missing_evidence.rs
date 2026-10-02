//! Missing delivery evidence remains unknown. It must not crowd retained work
//! out of the bounded diagnostic list. All downstream outcomes below are
//! synthetic; no remote delivery or operator attestation is performed.
#![cfg(all(feature = "platform", feature = "test-support"))]

mod common;

use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    crypto::{now, with_test_time},
    identity::downstream::{
        self, Deactivation, Dismissal, DismissalReason, DispatchRecovery, DispatchRecoveryReason,
        Observed, Resolution, Status as DeliveryStatus, UnlinkedCreate,
    },
    offboarding::{BUCKET, BeforeCommit, ExecuteAt, Job, ScheduleRequest, Status},
};
use serde_json::{Value, json};

const RETAIN: u64 = 90 * 86_400;
const PRIVATE_ERROR: &str = "https://secret.example/delivery?access_token=SECRET-ERROR";

fn completed_job(f: &Fixture, username: &str) -> Job {
    f.user(username);
    let due = now() + 60;
    let scheduled = f
        .core
        .offboard_schedule(
            &f.admin,
            ScheduleRequest {
                username: username.into(),
                execute_at: ExecuteAt::Unix(due),
                timezone: "UTC".into(),
            },
        )
        .unwrap();
    assert!(
        with_test_time(due, || {
            f.core
                .offboard_process("evidence-fixture", |_| BeforeCommit::Proceed)
        })
        .unwrap()
    );
    let job: Job = f
        .core
        .store
        .get(BUCKET, scheduled["id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(job.status, Status::Done);
    job
}

fn delivery(job: &Job, id: &str, target: &str, status: DeliveryStatus) -> Deactivation {
    Deactivation {
        id: id.into(),
        link: "SECRET-LINK".into(),
        target: target.into(),
        target_url: "https://secret.example/scim?access_token=SECRET-URL".into(),
        user_id: job.user_id.clone(),
        username: job.username.clone(),
        epoch: job.result.as_ref().unwrap()["local"]["epoch"]
            .as_u64()
            .unwrap(),
        remote_id: "SECRET-REMOTE".into(),
        external_id: "SECRET-EXTERNAL".into(),
        link_digest: "SECRET-DIGEST".into(),
        status,
        hold: None,
        attempts: 1,
        next_attempt: now(),
        lease_owner: None,
        lease_until: 0,
        dispatch_started: None,
        actor: Some("SECRET-ACTOR".into()),
        last_error: Some(PRIVATE_ERROR.into()),
        outcome: None,
        created_at: now(),
        delivered_at: None,
        uncertain: false,
        resolution: None,
        dismissal: None,
        dispatch_recoveries: Vec::new(),
        unlinked_create: None,
    }
}

fn resolved(row: &mut Deactivation) {
    row.resolution = Some(Resolution {
        observed: Observed::Applied,
        evidence: "SECRET-RESOLUTION".into(),
        by: "SECRET-OPERATOR".into(),
        at: now(),
        create_settlement: None,
    });
}

fn waived(row: &mut Deactivation) {
    row.dismissal = Some(Dismissal {
        reason: DismissalReason::PermanentlyUnverifiable,
        evidence: "SECRET-WAIVER".into(),
        by: "SECRET-OPERATOR".into(),
        at: now(),
        previous_status: DeliveryStatus::Failed,
        revision: "SECRET-REVIEW".into(),
    });
}

fn references(items: &[(&str, &str)]) -> Value {
    json!({"targets": items.iter().map(|(target, id)| {
        json!({"target": target, "delivery": id})
    }).collect::<Vec<_>>()})
}

fn put_job(
    f: &Fixture,
    template: &Job,
    id: &str,
    items: &[(&str, &str)],
    edit: impl FnOnce(&mut Job),
) {
    let mut job = template.clone();
    job.id = id.into();
    job.result.as_mut().unwrap()["downstream"] = references(items);
    edit(&mut job);
    f.core.store.write(|tx| tx.put(BUCKET, id, &job)).unwrap();
}

fn put_rows(f: &Fixture, rows: &[Deactivation]) {
    f.core
        .store
        .write(|tx| {
            for row in rows {
                tx.put(downstream::BUCKET, &row.id, row)?;
            }
            Ok(())
        })
        .unwrap();
}

fn diagnostics(f: &Fixture, token: &str) -> Value {
    let before = f.snapshot().unwrap();
    let report = f.core.offboarding_diagnostics(token).unwrap();
    f.assert_snapshot(&before);
    assert_redacted(&report);
    report
}

fn raw(f: &Fixture, id: &str) -> Value {
    let before = f.snapshot().unwrap();
    let view = f.core.offboard_get(&f.admin, id).unwrap();
    f.assert_snapshot(&before);
    assert!(view.get("downstream_evidence_status").is_none());
    assert_eq!(view["downstream"].as_object().unwrap().len(), 3);
    view
}

fn item<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap_or_else(|| panic!("missing diagnostic job {id}"))
}

fn count(report: &Value, key: &str) -> u64 {
    report["counts"][key].as_u64().unwrap()
}

fn assert_redacted(value: &Value) {
    match value {
        Value::Object(fields) => {
            for key in [
                "result",
                "last_error",
                "target_url",
                "remote_id",
                "external_id",
                "link",
                "link_digest",
                "actor",
                "lease_owner",
                "resolution",
                "dismissal",
                "dispatch_recoveries",
                "unlinked_create",
                "evidence",
                "request_key",
            ] {
                assert!(!fields.contains_key(key), "private diagnostic field {key}");
            }
            for child in fields.values() {
                assert_redacted(child);
            }
        }
        Value::Array(items) => items.iter().for_each(assert_redacted),
        Value::String(text) => {
            for secret in ["SECRET-", "secret.example", "access_token", "https://"] {
                assert!(!text.contains(secret), "private diagnostic text");
            }
        }
        _ => {}
    }
}

#[test]
fn retention_removes_success_failure_and_ambiguity_without_proving_completion() {
    let f = Fixture::new();
    let job = completed_job(&f, "retention");
    let at = now();
    let old = at - RETAIN;
    let mut rows = Vec::new();
    for (id, status) in [
        ("success", DeliveryStatus::Delivered),
        ("failure", DeliveryStatus::Failed),
        ("ambiguous", DeliveryStatus::Failed),
        ("stale", DeliveryStatus::Stale),
        ("cancelled", DeliveryStatus::Superseded),
        ("resolved", DeliveryStatus::Failed),
    ] {
        let mut row = delivery(&job, id, "payroll", status);
        row.created_at = old;
        row.next_attempt = old;
        if id == "success" {
            row.outcome = Some("deactivated".into());
            row.delivered_at = Some(old);
        } else if id == "ambiguous" {
            row.uncertain = true;
        } else if id == "resolved" {
            resolved(&mut row);
        }
        rows.push(row);
    }
    assert_eq!(rows[0].delivery_state(), "succeeded");
    assert_eq!(rows[1].delivery_state(), "failed");
    assert_eq!(rows[2].delivery_state(), "ambiguous");
    assert_eq!(rows[5].delivery_state(), "resolved");
    let ordinary = rows.clone();
    let ids: Vec<_> = ordinary
        .iter()
        .map(|row| ("payroll", row.id.as_str()))
        .collect();
    put_job(&f, &job, &job.id, &ids, |_| {});

    for id in [
        "waiver", "leased", "recovery", "unlinked", "pending", "overflow",
    ] {
        let mut row = delivery(&job, id, "payroll", DeliveryStatus::Failed);
        row.created_at = old;
        row.next_attempt = old;
        match id {
            "waiver" => {
                row.status = DeliveryStatus::Dismissed;
                waived(&mut row);
            }
            "leased" => row.lease_owner = Some("SECRET-LEASE".into()),
            "recovery" => row.dispatch_recoveries.push(DispatchRecovery {
                reason: DispatchRecoveryReason::WorkerLost,
                evidence: "SECRET-RECOVERY".into(),
                workers_quiesced: true,
                remote_requests_settled: true,
                by: "SECRET-OPERATOR".into(),
                at,
                revision: "SECRET-REVISION".into(),
                previous: json!({"token": "SECRET-PREVIOUS"}),
            }),
            "unlinked" => {
                row.unlinked_create = Some(UnlinkedCreate {
                    source_job: "SECRET-SOURCE".into(),
                    target: row.target.clone(),
                    target_url: row.target_url.clone(),
                    user_id: row.user_id.clone(),
                    external_id: row.external_id.clone(),
                    request_key: "SECRET-REQUEST".into(),
                })
            }
            "pending" => row.status = DeliveryStatus::Pending,
            "overflow" => row.next_attempt = u64::MAX,
            _ => unreachable!(),
        }
        rows.push(row);
    }
    put_rows(&f, &rows);
    let retained = diagnostics(&f, &f.admin);
    assert_eq!(
        item(&retained, &job.id)["downstream_evidence_status"],
        "retained"
    );
    assert_eq!(count(&retained, "downstream_evidence_unavailable_only"), 0);
    let previous_raw = raw(&f, &job.id);
    assert_eq!(previous_raw["downstream"]["state"], "incomplete");
    assert_eq!(
        previous_raw["downstream"]["targets"][0]["delivery_state"],
        "succeeded"
    );
    assert_eq!(
        previous_raw["downstream"]["targets"][2]["delivery_state"],
        "ambiguous"
    );

    // At equality all ordinary terminal evidence is still retained.
    f.core
        .store
        .write(|tx| riauth::provisioning::cleanup(tx, at))
        .unwrap();
    for row in &rows {
        let saved: Deactivation = f
            .core
            .store
            .get(downstream::BUCKET, &row.id)
            .unwrap()
            .unwrap();
        assert!(saved.revision().unwrap() == row.revision().unwrap());
    }
    f.core
        .store
        .write(|tx| riauth::provisioning::cleanup(tx, at + 1))
        .unwrap();
    for row in &ordinary {
        assert!(
            f.core
                .store
                .get::<Deactivation>(downstream::BUCKET, &row.id)
                .unwrap()
                .is_none()
        );
    }
    for row in &rows[ordinary.len()..] {
        let saved: Deactivation = f
            .core
            .store
            .get(downstream::BUCKET, &row.id)
            .unwrap()
            .unwrap();
        assert!(saved.revision().unwrap() == row.revision().unwrap());
    }

    let after_raw = raw(&f, &job.id);
    assert_eq!(after_raw["status"], "done");
    assert_eq!(after_raw["downstream"]["state"], "incomplete");
    assert_eq!(after_raw["result"], previous_raw["result"]);
    assert!(
        after_raw["downstream"]["targets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|target| {
                target.as_object().unwrap().len() == 3 && target["status"] == "expired"
            })
    );
    let unavailable = diagnostics(&f, &f.admin);
    assert_eq!(count(&unavailable, "attention"), 1);
    assert_eq!(count(&unavailable, "downstream_incomplete"), 1);
    assert_eq!(count(&unavailable, "downstream_delivered"), 0);
    assert_eq!(count(&unavailable, "downstream_resolved"), 0);
    assert_eq!(
        count(&unavailable, "downstream_evidence_unavailable_only"),
        1
    );
    let attention = item(&unavailable, &job.id);
    assert_eq!(attention["downstream_evidence_status"], "unavailable_only");
    assert_eq!(
        attention["next_action"],
        "inspect_missing_delivery_evidence"
    );
    assert_eq!(attention["remote_completion_verified"], false);
    assert_eq!(attention["recorded_targets"], ordinary.len());
    assert!(
        attention["targets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|target| {
                target["delivery_state"] == "expired" && target["delivered_at"].is_null()
            })
    );
}

#[test]
fn missing_only_history_cannot_crowd_retained_actionable_jobs_under_cap_fifty() {
    let f = Fixture::new();
    let job = completed_job(&f, "priority");
    let at = now();
    let mut expired = delivery(&job, "old-delivery", "payroll", DeliveryStatus::Failed);
    expired.next_attempt = at - RETAIN - 1;
    put_rows(&f, &[expired]);
    put_job(&f, &job, &job.id, &[("payroll", "old-delivery")], |_| {});
    f.core
        .store
        .write(|tx| riauth::provisioning::cleanup(tx, at))
        .unwrap();
    assert!(
        f.core
            .store
            .get::<Deactivation>(downstream::BUCKET, "old-delivery")
            .unwrap()
            .is_none()
    );

    let mut ambiguous = delivery(&job, "ambiguous-live", "payroll", DeliveryStatus::Failed);
    ambiguous.uncertain = true;
    let mut waiver = delivery(&job, "waiver-live", "payroll", DeliveryStatus::Dismissed);
    waived(&mut waiver);
    let mut success = delivery(&job, "success-live", "payroll", DeliveryStatus::Delivered);
    success.delivered_at = Some(at);
    success.outcome = Some("deactivated".into());
    let mut attested = delivery(&job, "resolved-live", "payroll", DeliveryStatus::Failed);
    resolved(&mut attested);
    put_rows(
        &f,
        &[
            delivery(&job, "failed-live", "payroll", DeliveryStatus::Failed),
            ambiguous,
            waiver,
            delivery(
                &job,
                "cancelled-live",
                "payroll",
                DeliveryStatus::Superseded,
            ),
            delivery(&job, "pending-live", "payroll", DeliveryStatus::Pending),
            success,
            attested,
        ],
    );
    for (id, reference) in [
        ("z-failed", "failed-live"),
        ("z-ambiguous", "ambiguous-live"),
        ("z-waived", "waiver-live"),
        ("z-cancelled", "cancelled-live"),
        ("z-pending", "pending-live"),
        ("z-success", "success-live"),
        ("z-resolved", "resolved-live"),
    ] {
        put_job(&f, &job, id, &[("payroll", reference)], |_| {});
    }
    for (id, reference) in [
        ("z-mixed-failed", "failed-live"),
        ("z-mixed-pending", "pending-live"),
        ("001-missing-success", "success-live"),
        ("001-missing-resolved", "resolved-live"),
    ] {
        put_job(
            &f,
            &job,
            id,
            &[("payroll", "old-delivery"), ("payroll", reference)],
            |_| {},
        );
    }
    for (id, status) in [
        ("z-local-retry", Status::Scheduled),
        ("z-overdue", Status::Scheduled),
        ("z-local-failed", Status::Failed),
    ] {
        put_job(&f, &job, id, &[], |job| {
            job.status = status;
            job.result = None;
            job.last_error = (id != "z-overdue").then(|| PRIVATE_ERROR.into());
            job.execute_at = if id == "z-overdue" {
                at - 400
            } else {
                at + 600
            };
            job.next_attempt = job.execute_at;
        });
    }
    let initial = diagnostics(&f, &f.admin);
    for id in [&job.id[..], "001-missing-success", "001-missing-resolved"] {
        let unknown = item(&initial, id);
        assert_eq!(unknown["downstream_evidence_status"], "unavailable_only");
        assert_eq!(unknown["downstream_state"], "incomplete");
        assert_eq!(unknown["remote_completion_verified"], false);
    }
    for id in ["z-mixed-failed", "z-mixed-pending"] {
        assert_eq!(
            item(&initial, id)["downstream_evidence_status"],
            "mixed_unavailable"
        );
    }
    assert_eq!(
        item(&initial, "z-ambiguous")["next_action"],
        "attest_remote_state"
    );
    assert_eq!(
        item(&initial, "z-waived")["next_action"],
        "confirm_waiver_not_delivery"
    );
    assert_eq!(
        item(&initial, "z-cancelled")["next_action"],
        "inspect_deactivation"
    );
    assert_eq!(
        item(&initial, "z-local-retry")["next_action"],
        "wait_for_local_retry"
    );
    assert_eq!(
        item(&initial, "z-local-retry")["downstream_evidence_status"],
        Value::Null
    );
    assert_eq!(
        item(&initial, "z-overdue")["next_action"],
        "check_worker_duty"
    );
    assert_eq!(
        item(&initial, "z-local-failed")["next_action"],
        "inspect_local_failure"
    );
    assert_eq!(raw(&f, "z-success")["downstream"]["state"], "delivered");
    assert_eq!(raw(&f, "z-resolved")["downstream"]["state"], "resolved");

    // These IDs would sort before every retained actionable clone at rank 1.
    for n in 0..50 {
        put_job(
            &f,
            &job,
            &format!("000-history-{n:02}"),
            &[("payroll", "old-delivery")],
            |_| {},
        );
    }
    let flooded = diagnostics(&f, &f.admin);
    assert_eq!(count(&flooded, "jobs"), 65);
    assert_eq!(count(&flooded, "attention"), 63);
    assert_eq!(count(&flooded, "downstream_incomplete"), 58);
    assert_eq!(count(&flooded, "downstream_pending"), 2);
    assert_eq!(count(&flooded, "downstream_delivered"), 1);
    assert_eq!(count(&flooded, "downstream_resolved"), 1);
    assert_eq!(count(&flooded, "downstream_evidence_unavailable_only"), 53);
    assert_eq!(flooded["limits"]["attention_items"], 50);
    assert_eq!(flooded["listed"], 50);
    assert_eq!(flooded["truncated"], true);
    let listed: Vec<_> = flooded["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        &listed[..10],
        &[
            "z-overdue",
            "z-local-failed",
            "z-ambiguous",
            "z-cancelled",
            "z-failed",
            "z-mixed-failed",
            "z-waived",
            "z-mixed-pending",
            "z-pending",
            "z-local-retry",
        ]
    );
    assert!(
        flooded["items"].as_array().unwrap()[10..]
            .iter()
            .all(|item| {
                item["downstream_evidence_status"] == "unavailable_only"
                    && item["downstream_state"] == "incomplete"
                    && item["remote_completion_verified"] == false
                    && item["next_action"] == "inspect_missing_delivery_evidence"
            })
    );
    assert!(!listed.contains(&"z-success") && !listed.contains(&"z-resolved"));
}

fn agent(f: &Fixture, id: &str, permissions: &[(&str, &str)]) -> String {
    f.core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.into(),
                ttl: 3600,
                permissions: permissions
                    .iter()
                    .map(|(action, resource)| Permission {
                        action: (*action).into(),
                        resource: (*resource).into(),
                    })
                    .collect(),
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn unknown_evidence_preserves_hidden_targets_withheld_counts_and_read_only_permissions() {
    let f = Fixture::new();
    let visible = completed_job(&f, "visible");
    let withheld = completed_job(&f, "withheld");
    put_rows(
        &f,
        &[
            delivery(
                &visible,
                "failed-visible",
                "payroll",
                DeliveryStatus::Failed,
            ),
            delivery(
                &visible,
                "pending-hidden",
                "private-payroll",
                DeliveryStatus::Pending,
            ),
        ],
    );
    put_job(
        &f,
        &visible,
        &visible.id,
        &[
            ("payroll", "failed-visible"),
            ("private-payroll", "missing-hidden"),
        ],
        |_| {},
    );
    put_job(
        &f,
        &visible,
        "hidden-missing",
        &[("private-payroll", "missing-hidden")],
        |_| {},
    );
    put_job(
        &f,
        &visible,
        "hidden-pending",
        &[
            ("private-payroll", "pending-hidden"),
            ("private-payroll", "missing-hidden"),
        ],
        |_| {},
    );
    put_job(
        &f,
        &withheld,
        &withheld.id,
        &[("private-payroll", "withheld-missing")],
        |_| {},
    );
    let operations = agent(
        &f,
        "counts-only",
        &[("operations.read", "operations/offboarding")],
    );
    let scoped = agent(
        &f,
        "scoped",
        &[
            ("operations.read", "operations/offboarding"),
            ("user.offboard", "user/visible"),
            ("provisioner.read", "provisioner/payroll"),
        ],
    );
    let denied = agent(&f, "no-operations", &[("user.offboard", "*")]);
    let before_denial = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .offboarding_diagnostics(&denied)
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    f.assert_snapshot(&before_denial);

    let counts = diagnostics(&f, &operations);
    assert_eq!(count(&counts, "jobs"), 4);
    assert_eq!(count(&counts, "attention"), 4);
    assert_eq!(count(&counts, "withheld"), 4);
    assert_eq!(count(&counts, "withheld_attention"), 4);
    assert_eq!(count(&counts, "downstream_evidence_unavailable_only"), 2);
    assert_eq!(counts["listed"], 0);
    for name in [
        "visible",
        "payroll",
        "private-payroll",
        visible.id.as_str(),
        withheld.id.as_str(),
    ] {
        assert!(!counts.to_string().contains(name));
    }
    let report = diagnostics(&f, &scoped);
    assert_eq!(count(&report, "withheld"), 1);
    assert_eq!(count(&report, "withheld_attention"), 1);
    assert_eq!(report["listed"], 3);
    assert_eq!(report["truncated"], false);
    assert!(!report.to_string().contains("private-payroll"));
    assert!(!report.to_string().contains("withheld-missing"));
    assert!(
        report["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["username"] == "visible")
    );
    let mixed = item(&report, &visible.id);
    assert_eq!(mixed["recorded_targets"], 2);
    assert_eq!(mixed["hidden_targets"], 1);
    assert_eq!(mixed["targets"].as_array().unwrap().len(), 1);
    assert_eq!(mixed["downstream_evidence_status"], "mixed_unavailable");
    let unknown = item(&report, "hidden-missing");
    assert_eq!(unknown["downstream_evidence_status"], "unavailable_only");
    assert_eq!(unknown["downstream_state"], "incomplete");
    assert_eq!(unknown["recorded_targets"], 1);
    assert_eq!(unknown["hidden_targets"], 1);
    assert_eq!(unknown["targets"], json!([]));
    let pending = item(&report, "hidden-pending");
    assert_eq!(pending["downstream_state"], "pending");
    assert_eq!(pending["downstream_evidence_status"], "mixed_unavailable");
    assert_eq!(pending["hidden_targets"], 2);
    for entry in report["items"].as_array().unwrap() {
        assert_eq!(entry["next_action"], "inspect_hidden_targets");
        assert_eq!(entry["remote_completion_verified"], false);
    }
}
