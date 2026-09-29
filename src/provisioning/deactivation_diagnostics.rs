//! Redacted deactivation counts for both editions.
//!
//! Item visibility is `readable_user`: `provisioner.read` on the stored target
//! and `user.read` on the live account. A missing account is withheld for
//! every caller, so the stored username and target stay off this response.
//! `user.offboard` is not consulted. The Platform offboarding aggregate keeps
//! its own route and its own authorization.

use super::deactivation::readable_user;
use crate::{
    core::Core,
    crypto::now,
    error::{Error, Result},
    identity::downstream::{BUCKET, Deactivation, Status},
};
use serde::Serialize;
use serde_json::{Value, json};

const DIAGNOSTIC_ITEMS: usize = 50;
const KNOWN_HOLDS: &[&str] = &[
    "unlinked_create_requires_settlement",
    "recovered_dispatch",
    "awaiting_dispatch_ack",
    "target_unconfigured",
    "awaiting_controller",
    "awaiting_controller_authority",
    "awaiting_prior_delivery",
    "manual_mode",
    "removal_review_required",
    "guarded_removal",
    "retry",
];

#[derive(Default, Serialize)]
struct Counts {
    deactivations: u64,
    pending: u64,
    running: u64,
    delivered: u64,
    superseded: u64,
    stale: u64,
    failed: u64,
    dismissed: u64,
    delivery_pending: u64,
    delivery_failed: u64,
    delivery_ambiguous: u64,
    delivery_dismissed: u64,
    delivery_succeeded: u64,
    delivery_resolved: u64,
    delivery_cancelled: u64,
    attention: u64,
    withheld: u64,
    withheld_attention: u64,
}

struct Listed {
    rank: u8,
    id: String,
    body: Value,
}

fn needs_attention(state: &str) -> bool {
    matches!(state, "pending" | "failed" | "ambiguous" | "dismissed")
}

fn count_row(counts: &mut Counts, row: &Deactivation, state: &str) {
    counts.deactivations = counts.deactivations.saturating_add(1);
    let status = match row.status {
        Status::Pending => &mut counts.pending,
        Status::Running => &mut counts.running,
        Status::Delivered => &mut counts.delivered,
        Status::Superseded => &mut counts.superseded,
        Status::Stale => &mut counts.stale,
        Status::Failed => &mut counts.failed,
        Status::Dismissed => &mut counts.dismissed,
    };
    *status = status.saturating_add(1);
    let delivery = match state {
        "pending" => &mut counts.delivery_pending,
        "failed" => &mut counts.delivery_failed,
        "ambiguous" => &mut counts.delivery_ambiguous,
        "dismissed" => &mut counts.delivery_dismissed,
        "succeeded" => &mut counts.delivery_succeeded,
        "resolved" => &mut counts.delivery_resolved,
        "cancelled" => &mut counts.delivery_cancelled,
        _ => &mut counts.delivery_failed,
    };
    *delivery = delivery.saturating_add(1);
}

fn status_name(status: Status) -> &'static str {
    match status {
        Status::Pending => "pending",
        Status::Running => "running",
        Status::Delivered => "delivered",
        Status::Superseded => "superseded",
        Status::Stale => "stale",
        Status::Failed => "failed",
        Status::Dismissed => "dismissed",
    }
}

fn public_outcome(outcome: Option<&str>) -> Option<&str> {
    match outcome {
        Some(
            "deactivated" | "already_inactive" | "reviewed_delivery" | "remote_active"
            | "remote_inactive",
        ) => outcome,
        _ => None,
    }
}

fn next_action(state: &str, status: &str, hold: Option<&str>, has_error: bool) -> &'static str {
    match state {
        "ambiguous" => "attest_remote_state",
        "dismissed" => "waiver_is_not_remote_delivery",
        "resolved" => "attestation_is_not_remote_delivery",
        "expired" => "delivery_record_expired",
        "cancelled" => "account_changed_before_delivery",
        "failed" if status == "stale" => "inspect_and_replan",
        "failed" => "retry_or_replan_deactivation",
        "pending" => match hold {
            Some(
                "manual_mode"
                | "removal_review_required"
                | "guarded_removal"
                | "awaiting_controller"
                | "target_unconfigured",
            ) => "review_provisioning_plan",
            Some("awaiting_controller_authority") => "restore_controller_authority",
            Some("awaiting_prior_delivery") => "wait_for_provisioning_job",
            Some(
                "awaiting_dispatch_ack"
                | "unlinked_create_requires_settlement"
                | "recovered_dispatch",
            ) => "wait_for_dispatch_settlement",
            Some("retry") => "wait_for_retry",
            _ if has_error => "inspect_deactivation",
            _ => "wait_for_deactivation",
        },
        _ => "inspect_deactivation",
    }
}

fn rank(state: &str) -> u8 {
    match state {
        "failed" => 0,
        "ambiguous" => 1,
        "dismissed" => 2,
        "pending" => 3,
        _ => 4,
    }
}

fn listed_item(row: &Deactivation, username: &str, state: &str) -> Listed {
    let status = status_name(row.status);
    let recognized = row
        .hold
        .as_deref()
        .is_none_or(|hold| KNOWN_HOLDS.contains(&hold));
    let hold = row
        .hold
        .as_deref()
        .filter(|hold| KNOWN_HOLDS.contains(hold));
    let has_error = row.last_error.is_some();
    Listed {
        rank: rank(state),
        id: row.id.clone(),
        body: json!({
            "id": row.id,
            "username": username,
            "recorded_username_matches": username == row.username,
            "status": status,
            "delivery_state": state,
            "hold": hold,
            "hold_recognized": recognized,
            "outcome": public_outcome(row.outcome.as_deref()),
            "attempts": row.attempts,
            "next_attempt": row.next_attempt,
            "has_error": has_error,
            "uncertain": row.uncertain,
            "delivered_at": row.delivered_at,
            "remote_completion_verified": false,
            "has_unlinked_create": row.unlinked_create.is_some(),
            "dispatch_recovery_count": row.dispatch_recoveries.len(),
            "next_action": next_action(state, status, hold, has_error),
        }),
    }
}

fn sort_attention(items: &mut [Listed]) {
    items.sort_by(|left, right| {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn retain_attention(items: &mut Vec<Listed>, item: Listed) {
    if items.len() < DIAGNOSTIC_ITEMS {
        items.push(item);
        if items.len() == DIAGNOSTIC_ITEMS {
            sort_attention(items);
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
    let position = items.partition_point(|existing| {
        (existing.rank, existing.id.as_str()) < (item.rank, item.id.as_str())
    });
    items.insert(position, item);
}

impl Core {
    /// Counts every stored deactivation and lists at most 50 redacted attention
    /// rows the caller may already read. The target name, stored username,
    /// error text, and remote identifiers stay on `GET /api/provisioning/deactivations`.
    pub fn provisioning_deactivation_diagnostics(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.management(tx, token, "operations.read", "operations/provisioning")?;
            let mut counts = Counts::default();
            let mut listed = Vec::with_capacity(DIAGNOSTIC_ITEMS);
            let mut visible_attention = 0u64;
            let mut after: Option<String> = None;
            loop {
                let page = tx.scan::<Deactivation>(
                    BUCKET,
                    after.as_deref(),
                    crate::store::maintenance::PAGE,
                )?;
                let Some(last_key) = page.last().map(|(key, _)| key.clone()) else {
                    break;
                };
                if after
                    .as_ref()
                    .is_some_and(|previous| last_key.as_str() <= previous.as_str())
                {
                    return Err(Error::internal(
                        "Deactivation diagnostic page did not advance",
                    ));
                }
                let full = page.len() == crate::store::maintenance::PAGE;
                after = Some(last_key);
                for (_, row) in page {
                    let state = row.delivery_state();
                    count_row(&mut counts, &row, state);
                    let account = readable_user(tx, &row, &actor)?;
                    if account.is_none() {
                        counts.withheld = counts.withheld.saturating_add(1);
                    }
                    if !needs_attention(state) {
                        continue;
                    }
                    counts.attention = counts.attention.saturating_add(1);
                    let Some(user) = account else {
                        counts.withheld_attention = counts.withheld_attention.saturating_add(1);
                        continue;
                    };
                    visible_attention = visible_attention.saturating_add(1);
                    retain_attention(&mut listed, listed_item(&row, &user.username, state));
                }
                if !full {
                    break;
                }
            }
            if listed.len() < DIAGNOSTIC_ITEMS {
                sort_attention(&mut listed);
            }
            let items: Vec<Value> = listed.into_iter().map(|item| item.body).collect();
            Ok(json!({
                "schema_version": "riauth.provisioning-deactivation-diagnostics/v1",
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
}
