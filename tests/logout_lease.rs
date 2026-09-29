//! Logout claim, pin, expiry, and stale completion on one embedded store.
//!
//! The test clock moves the 60-second lease without sleeping. Claim, pin, and
//! finish do not append audit rows.
#![cfg(feature = "test-support")]

#[path = "common/mod.rs"]
mod common;

use common::Fixture;
use riauth::{
    crypto::{set_test_time, with_test_time},
    logout::Delivery,
    model::Audit,
};

fn actions(fixture: &Fixture) -> Vec<String> {
    let mut actions = fixture
        .core
        .store
        .list::<Audit>("audit")
        .unwrap()
        .into_iter()
        .map(|(_, row)| row.action)
        .collect::<Vec<_>>();
    actions.sort();
    actions
}

fn delivery(id: &str, at: u64, created_at: u64) -> Delivery {
    Delivery {
        id: id.into(),
        client_id: "rp".into(),
        sid: "sid".into(),
        subject: "subject".into(),
        uri: "http://127.0.0.1:1/logout".into(),
        created_at,
        next_attempt: at,
        attempts: 0,
        delivered_at: None,
        last_status: None,
        last_failed: false,
        lease: None,
        dispatch_started: None,
    }
}

fn load(fixture: &Fixture, id: &str) -> Delivery {
    fixture
        .core
        .store
        .get::<Delivery>("logout_deliveries", id)
        .unwrap()
        .unwrap()
}

#[test]
fn logout_lease_pins_one_attempt_until_expiry() {
    let fixture = Fixture::new();
    fixture.client("rp", false);
    let before = actions(&fixture);
    let at = 1_700_000_000u64;
    with_test_time(at, || {
        fixture
            .core
            .store
            .write(|tx| {
                tx.put(
                    "logout_deliveries",
                    "expired-delivery",
                    &delivery("expired-delivery", at, at - 86_401),
                )
            })
            .unwrap();
        assert!(fixture.core.claim_logout_deliveries().unwrap().is_empty());
        let expired = load(&fixture, "expired-delivery");
        assert_eq!(expired.next_attempt, u64::MAX);
        assert!(expired.lease.is_none());
        assert!(expired.dispatch_started.is_none());
        assert_eq!(expired.attempts, 0);

        fixture
            .core
            .store
            .write(|tx| {
                tx.put(
                    "logout_deliveries",
                    "lease-delivery",
                    &delivery("lease-delivery", at, at),
                )
            })
            .unwrap();
        let first = fixture.core.claim_logout_deliveries().unwrap();
        assert_eq!(first.len(), 1);
        let lease = first[0].0.lease.clone().unwrap();
        assert_eq!(first[0].0.attempts, 1);
        assert!(first[0].0.dispatch_started.is_none());
        assert!(fixture.core.claim_logout_deliveries().unwrap().is_empty());
        assert!(
            fixture
                .core
                .begin_logout_dispatch("lease-delivery", &lease)
                .unwrap()
        );
        assert!(
            !fixture
                .core
                .begin_logout_dispatch("lease-delivery", &lease)
                .unwrap()
        );
        assert!(
            !fixture
                .core
                .begin_logout_dispatch("lease-delivery", "other-lease")
                .unwrap()
        );
        let pinned = load(&fixture, "lease-delivery");
        assert_eq!(pinned.dispatch_started, Some(true));
        assert_eq!(pinned.lease.as_deref(), Some(lease.as_str()));

        set_test_time(at + 60);
        assert!(
            !fixture
                .core
                .begin_logout_dispatch("lease-delivery", &lease)
                .unwrap()
        );
        let second = fixture.core.claim_logout_deliveries().unwrap();
        assert_eq!(second.len(), 1);
        let next = second[0].0.lease.clone().unwrap();
        assert_ne!(next, lease);
        assert_eq!(second[0].0.attempts, 2);
        assert!(second[0].0.dispatch_started.is_none());
        fixture
            .core
            .finish_logout_delivery("lease-delivery", 1, Some(204))
            .unwrap();
        let held = load(&fixture, "lease-delivery");
        assert!(held.delivered_at.is_none());
        assert_eq!(held.attempts, 2);
        assert_eq!(held.lease.as_deref(), Some(next.as_str()));
        assert!(
            fixture
                .core
                .begin_logout_dispatch("lease-delivery", &next)
                .unwrap()
        );
        fixture
            .core
            .finish_logout_delivery("lease-delivery", 2, Some(204))
            .unwrap();
        let done = load(&fixture, "lease-delivery");
        assert_eq!(done.delivered_at, Some(at + 60));
        assert!(done.lease.is_none());
        assert!(done.dispatch_started.is_none());
        assert_eq!(done.last_status, Some(204));
        assert!(!done.last_failed);
        assert!(fixture.core.claim_logout_deliveries().unwrap().is_empty());

        fixture
            .core
            .store
            .write(|tx| {
                tx.put(
                    "logout_deliveries",
                    "retry-delivery",
                    &delivery("retry-delivery", at + 60, at + 60),
                )
            })
            .unwrap();
        let retry = fixture.core.claim_logout_deliveries().unwrap();
        assert_eq!(retry.len(), 1);
        let retry_lease = retry[0].0.lease.clone().unwrap();
        assert!(
            fixture
                .core
                .begin_logout_dispatch("retry-delivery", &retry_lease)
                .unwrap()
        );
        fixture
            .core
            .finish_logout_delivery("retry-delivery", 1, None)
            .unwrap();
        let failed = load(&fixture, "retry-delivery");
        assert!(failed.delivered_at.is_none());
        assert!(failed.lease.is_none());
        assert!(failed.dispatch_started.is_none());
        assert!(failed.last_failed);
        assert_eq!(failed.attempts, 1);
        assert_eq!(failed.next_attempt, at + 62);
        assert!(fixture.core.claim_logout_deliveries().unwrap().is_empty());
        set_test_time(at + 62);
        let again = fixture.core.claim_logout_deliveries().unwrap();
        assert_eq!(again.len(), 1);
        assert_eq!(again[0].0.attempts, 2);
        assert_ne!(again[0].0.lease.as_deref(), Some(retry_lease.as_str()));
    });
    assert_eq!(actions(&fixture), before);
}
