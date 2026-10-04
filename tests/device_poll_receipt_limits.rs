//! Local protocol admission fixtures; large boundaries are seeded, not polled.
#[path = "common/mod.rs"]
mod common;

use common::{Fixture, text};
use riauth::{
    context::{RequestContext, scope},
    core::Core,
    crypto::{digest, now, with_test_time},
    error::{Error, Result},
    model::{Client, Device, ProviderSettings},
    oidc::{DEVICE_GRANT, TokenRequest},
    store::with_prepared_pause,
};
use serde_json::{Value, json};
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::{Barrier, mpsc},
    time::Duration,
};

const BUCKET: &str = "device_poll_receipts";

fn fixture() -> Fixture {
    let f = Fixture::new();
    f.client("app", false);
    f
}

fn request(client: &str, code: &str, secret: Option<&str>) -> TokenRequest {
    TokenRequest {
        grant_type: DEVICE_GRANT.into(),
        client_id: Some(client.into()),
        device_code: Some(code.into()),
        client_secret: secret.map(str::to_owned),
        ..Default::default()
    }
}

fn start(core: &Core, client: &str, secret: Option<&str>) -> Result<Value> {
    core.device_start(TokenRequest {
        client_id: Some(client.into()),
        client_secret: secret.map(str::to_owned),
        scope: Some("openid".into()),
        ..Default::default()
    })
}

fn keyed(core: &Core, request: TokenRequest, key: &str, ip: u8) -> Result<Value> {
    let fingerprint = digest(&serde_json::to_string(&request).unwrap());
    scope(
        Some(RequestContext {
            idempotency_key: Some(key.into()),
            fingerprint,
            client_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 0, 2, ip))),
            ..Default::default()
        }),
        || core.token(request),
    )
}

fn refusal(result: Result<Value>) -> Error {
    match result {
        Err(error) => error,
        Ok(_) => panic!("Expected a refusal; response omitted"),
    }
}

fn capacity(result: Result<Value>) {
    let error = refusal(result);
    assert_eq!(error.status.as_u16(), 429);
    assert_eq!(error.code, "rate_limited");
    assert_eq!(error.message, "Device authorization capacity reached");
}

fn record(f: &Fixture, client: &str, key: &str) -> Value {
    f.core
        .store
        .get(
            BUCKET,
            &digest(&format!("oidc-device-poll-v1\0{client}\0{key}")),
        )
        .unwrap()
        .unwrap()
}

fn seed_receipts(
    f: &Fixture,
    template: &Value,
    count: usize,
    bindings: impl Fn(usize) -> (String, String),
) {
    f.core
        .store
        .write(|tx| {
            for index in 0..count {
                let mut value = template.clone();
                let (client, proof) = bindings(index);
                value["permissions"]["client_id"] = json!(client);
                value["permissions"]["device_code_hash"] = json!(proof);
                tx.put(
                    BUCKET,
                    &digest(&format!("seeded-poll-receipt-{index}")),
                    &value,
                )?;
            }
            Ok(())
        })
        .unwrap();
}

fn pending_template(f: &Fixture) -> (Value, Value) {
    let started = start(&f.core, "app", None).unwrap();
    let code = text(&started, "device_code");
    assert_eq!(
        refusal(keyed(&f.core, request("app", &code, None), "first", 1)).code,
        "authorization_pending"
    );
    let saved = record(f, "app", "first");
    (started, saved)
}

#[test]
fn proof_boundary_replay_conflict_and_reopen_preserve_exact_poll_timing() {
    let f = fixture();
    let at = now();
    with_test_time(at, || {
        let (started, template) = pending_template(&f);
        let code = text(&started, "device_code");
        let proof = digest(&code);
        seed_receipts(&f, &template, 510, |_| ("app".into(), proof.clone()));
        assert_eq!(f.core.store.list::<Value>(BUCKET).unwrap().len(), 511);
        assert_eq!(
            refusal(keyed(&f.core, request("app", &code, None), "last-slot", 1)).code,
            "slow_down"
        );
        assert_eq!(f.core.store.list::<Value>(BUCKET).unwrap().len(), 512);
        let before = f.snapshot().unwrap();
        capacity(keyed(&f.core, request("app", &code, None), "over-limit", 2));
        f.assert_snapshot(&before);
        assert_eq!(
            refusal(keyed(&f.core, request("app", &code, None), "last-slot", 2)).code,
            "slow_down"
        );
        f.assert_snapshot(&before);
        assert_eq!(
            refusal(keyed(&f.core, request("app", &code, None), "first", 1)).code,
            "conflict"
        );
        let mut changed = request("app", &code, None);
        changed.scope = Some("openid".into());
        assert_eq!(
            refusal(keyed(&f.core, changed, "last-slot", 1)).code,
            "conflict"
        );
        let mut changed = request("app", &code, None);
        changed.dpop_proof = Some("changed-header-proof".into());
        assert_eq!(
            refusal(keyed(&f.core, changed, "last-slot", 1)).code,
            "conflict"
        );
        f.assert_snapshot(&before);
        let saved = record(&f, "app", "last-slot");
        let stored = saved.to_string();
        assert!(
            !stored.contains(&code) && !stored.contains(started["user_code"].as_str().unwrap())
        );
        assert!(
            f.core
                .store
                .get::<Value>("receipts", &digest("oidc-device-poll-v1\0app\0last-slot"))
                .unwrap()
                .is_none()
        );
        let f = f.reopen_with(|_| {});
        let before = f.snapshot().unwrap();
        assert_eq!(
            refusal(keyed(&f.core, request("app", &code, None), "last-slot", 1)).code,
            "slow_down"
        );
        f.assert_snapshot(&before);
        capacity(keyed(
            &f.core,
            request("app", &code, None),
            "after-restart",
            1,
        ));
        f.assert_snapshot(&before);
    });
}

#[test]
fn stable_client_budget_survives_new_codes_configuration_and_ip_changes() {
    let f = fixture();
    let (started, template) = pending_template(&f);
    let second = start(&f.core, "app", None).unwrap();
    let third = start(&f.core, "app", None).unwrap();
    let second_code = text(&second, "device_code");
    let second_hash = digest(&second_code);
    let third_hash = digest(third["device_code"].as_str().unwrap());
    // Expired retained records still charge the budget while cleanup is absent.
    let mut retained = template.clone();
    let expired = now() - 120;
    retained["result"]["expires_at"] = json!(expired);
    retained["result"]["last_poll_at"] = json!(expired - 60);
    retained["retain_until"] = json!(expired + 60);
    seed_receipts(&f, &retained, 1022, |index| {
        (
            "app".into(),
            if index % 2 == 0 {
                second_hash.clone()
            } else {
                third_hash.clone()
            },
        )
    });
    assert_eq!(
        refusal(keyed(
            &f.core,
            request("app", &second_code, None),
            "client-last-slot",
            1
        ))
        .code,
        "authorization_pending"
    );
    assert_eq!(f.core.store.list::<Value>(BUCKET).unwrap().len(), 1024);
    let fresh = start(&f.core, "app", None).unwrap();
    let code = text(&fresh, "device_code");
    let before = f.snapshot().unwrap();
    capacity(keyed(&f.core, request("app", &code, None), "new-code", 2));
    f.assert_snapshot(&before);
    f.core
        .store
        .write(|tx| {
            let mut client: Client = tx.get("clients", "app")?.unwrap();
            client.settings.device_ttl = Some(1800);
            tx.put("clients", "app", &client)
        })
        .unwrap();
    let before = f.snapshot().unwrap();
    capacity(keyed(
        &f.core,
        request("app", &code, None),
        "changed-config-ip",
        3,
    ));
    assert_eq!(
        refusal(keyed(
            &f.core,
            request("app", started["device_code"].as_str().unwrap(), None),
            "first",
            1
        ))
        .code,
        "access_denied"
    );
    f.assert_snapshot(&before);
    // A client-wide key cannot be reassigned to another live device proof.
    let other = fixture();
    let (one, _) = pending_template(&other);
    let two = start(&other.core, "app", None).unwrap();
    let before = other.snapshot().unwrap();
    assert_eq!(
        refusal(keyed(
            &other.core,
            request("app", two["device_code"].as_str().unwrap(), None),
            "first",
            1
        ))
        .code,
        "conflict"
    );
    other.assert_snapshot(&before);
    assert!(one["device_code"].as_str().is_some());
}

#[test]
fn global_receipt_bound_preserves_approved_one_use_issuance_and_decision_receipts() {
    let f = fixture();
    let at = now();
    with_test_time(at, || {
        let (started, template) = pending_template(&f);
        let code = text(&started, "device_code");
        seed_receipts(&f, &template, 4094, |index| {
            (
                format!("retained-client-{}", index % 16),
                digest(&format!("retained-proof-{}", index % 32)),
            )
        });
        assert_eq!(
            refusal(keyed(
                &f.core,
                request("app", &code, None),
                "global-last-slot",
                1
            ))
            .code,
            "slow_down"
        );
        assert_eq!(f.core.store.list::<Value>(BUCKET).unwrap().len(), 4096);
        f.client("other", false);
        let other = start(&f.core, "other", None).unwrap();
        let before = f.snapshot().unwrap();
        capacity(keyed(
            &f.core,
            request("other", other["device_code"].as_str().unwrap(), None),
            "over-global",
            2,
        ));
        assert_eq!(
            refusal(keyed(
                &f.core,
                request("app", &code, None),
                "global-last-slot",
                2
            ))
            .code,
            "slow_down"
        );
        f.assert_snapshot(&before);
        let user_code = text(&started, "user_code");
        with_test_time(at + 11, || {
            let decision_context = RequestContext {
                idempotency_key: Some("reviewed-decision".into()),
                fingerprint: digest("exact-device-decision"),
                ..Default::default()
            };
            scope(Some(decision_context.clone()), || {
                f.core.device_decide(&f.admin, &user_code, true)
            })
            .unwrap();
            let before = f.snapshot().unwrap();
            assert_eq!(
                refusal(keyed(
                    &f.core,
                    request("app", &code, None),
                    "global-last-slot",
                    1
                ))
                .code,
                "conflict"
            );
            f.assert_snapshot(&before);
            let issued = keyed(&f.core, request("app", &code, None), "issue", 1).unwrap();
            assert!(issued["access_token"].is_string() && issued["id_token"].is_string());
            assert_eq!(f.core.store.list::<Value>(BUCKET).unwrap().len(), 4096);
            let before = f.snapshot().unwrap();
            assert_eq!(
                refusal(keyed(&f.core, request("app", &code, None), "issue", 1)).code,
                "invalid_grant"
            );
            f.assert_snapshot(&before);
            assert_eq!(
                scope(Some(decision_context), || f
                    .core
                    .device_decide(&f.admin, &user_code, true))
                .unwrap()["approved"],
                true
            );
            f.core
                .device_decide(&f.admin, other["user_code"].as_str().unwrap(), false)
                .unwrap();
            let before = f.snapshot().unwrap();
            assert_eq!(
                refusal(keyed(
                    &f.core,
                    request("other", other["device_code"].as_str().unwrap(), None),
                    "denied-at-capacity",
                    1,
                ))
                .code,
                "access_denied"
            );
            f.assert_snapshot(&before);
        });
        let decisions = f.core.store.list::<Value>("receipts").unwrap();
        assert_eq!(decisions.len(), 1);
        with_test_time(at + 661, || f.core.cleanup().unwrap());
        assert!(
            f.core
                .store
                .get::<Value>("receipts", &decisions[0].0)
                .unwrap()
                .unwrap()
                == decisions[0].1
        );
        assert_eq!(
            f.core.store.list::<Value>(BUCKET).unwrap().len(),
            4096 - 128
        );
    });
}

#[test]
fn retained_device_limits_are_stable_and_atomic_across_creation_writers() {
    let f = fixture();
    for _ in 0..31 {
        start(&f.core, "app", None).unwrap();
    }
    let barrier = Barrier::new(2);
    let results = std::thread::scope(|threads| {
        let a = threads.spawn(|| {
            barrier.wait();
            start(&f.core, "app", None)
        });
        let b = threads.spawn(|| {
            barrier.wait();
            start(&f.core, "app", None)
        });
        [a.join().unwrap(), b.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    for result in results {
        if result.is_err() {
            capacity(result);
        }
    }
    assert_eq!(f.core.store.list::<Device>("devices").unwrap().len(), 32);
    let f = f.reopen_with(|_| {});
    let before = f.snapshot().unwrap();
    capacity(start(&f.core, "app", None));
    f.assert_snapshot(&before);
    f.core
        .store
        .write(|tx| {
            let mut client: Client = tx.get("clients", "app")?.unwrap();
            client.settings.device_ttl = Some(1800);
            tx.put("clients", "app", &client)
        })
        .unwrap();
    let before = f.snapshot().unwrap();
    scope(
        Some(RequestContext {
            client_ip: Some("192.0.2.99".parse().unwrap()),
            ..Default::default()
        }),
        || capacity(start(&f.core, "app", None)),
    );
    f.assert_snapshot(&before);
    f.client("other", false);
    start(&f.core, "other", None).unwrap();
    let template = f.core.store.list::<Device>("devices").unwrap()[0].1.clone();
    f.core
        .store
        .write(|tx| {
            for index in 0..223 {
                let mut device = template.clone();
                device.client_id = format!("retained-client-{}", index % 8);
                device.expires_at = now() - 1;
                device.user_code_hash = digest(&format!("retained-user-code-{index}"));
                let key = digest(&format!("retained-device-code-{index}"));
                tx.put("devices", &key, &device)?;
                tx.put("device_users", &device.user_code_hash, &key)?;
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(f.core.store.list::<Device>("devices").unwrap().len(), 256);
    let before = f.snapshot().unwrap();
    capacity(start(&f.core, "other", None));
    f.assert_snapshot(&before);
}

#[test]
fn original_deadline_controls_retention_for_min_default_and_max_lifetimes() {
    for ttl in [Some(60), None, Some(1800)] {
        let f = Fixture::new();
        f.client_with_settings(
            "app",
            false,
            ProviderSettings {
                device_ttl: ttl,
                ..Default::default()
            },
        );
        let at = now();
        with_test_time(at, || {
            let (started, saved) = pending_template(&f);
            let code = text(&started, "device_code");
            let lifetime = ttl.unwrap_or(600);
            assert_eq!(started["expires_in"], lifetime);
            assert_eq!(saved["retain_until"], at + lifetime + 60);
            with_test_time(at + lifetime, || {
                let before = f.snapshot().unwrap();
                assert_eq!(
                    refusal(keyed(&f.core, request("app", &code, None), "first", 1)).code,
                    "expired_token"
                );
                f.assert_snapshot(&before);
            });
            with_test_time(at + lifetime + 59, || {
                f.core.cleanup().unwrap();
                assert!(record(&f, "app", "first") == saved);
            });
            with_test_time(at + lifetime + 60, || {
                f.core.cleanup().unwrap();
                assert!(f.core.store.list::<Value>(BUCKET).unwrap().is_empty());
                let fresh = start(&f.core, "app", None).unwrap();
                assert_eq!(
                    refusal(keyed(
                        &f.core,
                        request("app", fresh["device_code"].as_str().unwrap(), None),
                        "first",
                        2
                    ))
                    .code,
                    "authorization_pending"
                );
            });
        });
    }
}

#[test]
fn secret_rotation_client_disable_and_wrong_proof_refuse_before_replay() {
    let f = Fixture::new();
    let secret = f.client("confidential", true).unwrap();
    f.client("public", false);
    let started = start(&f.core, "confidential", Some(&secret)).unwrap();
    let code = text(&started, "device_code");
    assert_eq!(
        refusal(keyed(
            &f.core,
            request("confidential", &code, Some(&secret)),
            "first",
            1
        ))
        .code,
        "authorization_pending"
    );
    let before = f.snapshot().unwrap();
    assert_eq!(
        refusal(keyed(
            &f.core,
            request("confidential", &code, Some("wrong-fixture-secret")),
            "first",
            1
        ))
        .code,
        "invalid_client"
    );
    assert_eq!(
        refusal(keyed(&f.core, request("public", &code, None), "first", 1)).code,
        "invalid_grant"
    );
    f.assert_snapshot(&before);
    let rotated = f
        .core
        .rotate_client_secret(&f.admin, "confidential")
        .unwrap();
    let rotated = rotated["client_secret"].as_str().unwrap();
    let before = f.snapshot().unwrap();
    assert_eq!(
        refusal(keyed(
            &f.core,
            request("confidential", &code, Some(&secret)),
            "first",
            1
        ))
        .code,
        "invalid_client"
    );
    assert_eq!(
        refusal(keyed(
            &f.core,
            request("confidential", &code, Some(rotated)),
            "first",
            1
        ))
        .code,
        "invalid_grant"
    );
    f.assert_snapshot(&before);
    let stored = record(&f, "confidential", "first").to_string();
    assert!(!stored.contains(&secret) && !stored.contains(rotated) && !stored.contains(&code));
    f.core
        .store
        .write(|tx| {
            let mut client: Client = tx.get("clients", "confidential")?.unwrap();
            client.enabled = false;
            tx.put("clients", "confidential", &client)
        })
        .unwrap();
    let before = f.snapshot().unwrap();
    assert_eq!(
        refusal(keyed(
            &f.core,
            request("confidential", &code, Some(rotated)),
            "first",
            1
        ))
        .code,
        "invalid_client"
    );
    f.assert_snapshot(&before);
}

#[test]
fn legacy_live_retry_and_strict_cleanup_leave_generic_contracts_unchanged() {
    let f = fixture();
    let at = now();
    with_test_time(at, || {
        let (started, saved) = pending_template(&f);
        let code = text(&started, "device_code");
        let key = digest("oidc-device-poll-v1\0app\0first");
        let legacy = json!({"fingerprint":saved["fingerprint"], "permissions":saved["permissions"], "result":saved["result"], "expires_at":at + 86400});
        let mut unknown = legacy.clone();
        unknown["permissions"]["extra"] = json!("unrecognized");
        let unknown_key = digest("unknown-protocol-row");
        let mut malformed = legacy.clone();
        malformed["result"]["interval"] = json!("not-a-number");
        let malformed_key = digest("malformed-protocol-row");
        let mut extra_outer = legacy.clone();
        extra_outer["unknown"] = json!(true);
        let outer_key = digest("extra-outer-row");
        let mut invalid_deadline = legacy.clone();
        invalid_deadline["expires_at"] = json!(at + 300);
        let deadline_key = digest("unproven-deadline-row");
        let generic = json!({"fingerprint":"fixture-management", "permissions":[], "result":{"ok":true}, "expires_at":at + 86400});
        f.core
            .store
            .write(|tx| {
                tx.delete(BUCKET, &key)?;
                tx.put("receipts", &key, &legacy)?;
                tx.put("receipts", &unknown_key, &unknown)?;
                tx.put("receipts", &malformed_key, &malformed)?;
                tx.put("receipts", &outer_key, &extra_outer)?;
                tx.put("receipts", &deadline_key, &invalid_deadline)?;
                tx.put("receipts", "ordinary-management", &generic)
            })
            .unwrap();
        let f = f.reopen_with(|_| {});
        let before = f.snapshot().unwrap();
        assert_eq!(
            refusal(keyed(&f.core, request("app", &code, None), "first", 1)).code,
            "authorization_pending"
        );
        f.assert_snapshot(&before);
        let mut changed = request("app", &code, None);
        changed.scope = Some("openid".into());
        assert_eq!(
            refusal(keyed(&f.core, changed, "first", 1)).code,
            "conflict"
        );
        f.assert_snapshot(&before);
        f.core
            .device_decide(&f.admin, started["user_code"].as_str().unwrap(), true)
            .unwrap();
        let before = f.snapshot().unwrap();
        assert_eq!(
            refusal(keyed(&f.core, request("app", &code, None), "first", 1)).code,
            "conflict"
        );
        f.assert_snapshot(&before);
        with_test_time(at + 600, || {
            let before = f.snapshot().unwrap();
            assert_eq!(
                refusal(keyed(&f.core, request("app", &code, None), "first", 1)).code,
                "expired_token"
            );
            f.assert_snapshot(&before);
        });
        with_test_time(at + 659, || {
            f.core.cleanup().unwrap();
            assert!(
                f.core
                    .store
                    .get::<Value>("receipts", &key)
                    .unwrap()
                    .unwrap()
                    == legacy
            );
        });
        with_test_time(at + 660, || {
            f.core.cleanup().unwrap();
            assert!(
                f.core
                    .store
                    .get::<Value>("receipts", &key)
                    .unwrap()
                    .is_none()
            );
            for (key, expected) in [
                (&unknown_key, &unknown),
                (&malformed_key, &malformed),
                (&outer_key, &extra_outer),
                (&deadline_key, &invalid_deadline),
            ] {
                assert!(f.core.store.get::<Value>("receipts", key).unwrap().unwrap() == *expected);
            }
            assert!(
                f.core
                    .store
                    .get::<Value>("receipts", "ordinary-management")
                    .unwrap()
                    .unwrap()
                    == generic
            );
        });
        with_test_time(at + 7 * 86400 - 1, || {
            f.core.cleanup().unwrap();
            assert!(
                f.core
                    .store
                    .get::<Value>("receipts", "ordinary-management")
                    .unwrap()
                    .is_some()
            );
        });
        with_test_time(at + 7 * 86400, || {
            f.core.cleanup().unwrap();
            assert!(
                f.core
                    .store
                    .get::<Value>("receipts", "ordinary-management")
                    .unwrap()
                    .is_none()
            );
        });
    });
}

#[test]
fn prepared_range_revalidation_prevents_two_proofs_spending_one_client_slot() {
    let f = fixture();
    let (one, template) = pending_template(&f);
    let two = start(&f.core, "app", None).unwrap();
    seed_receipts(&f, &template, 1022, |index| {
        (
            "app".into(),
            digest(&format!("retained-proof-{}", index % 3)),
        )
    });
    let (prepared_tx, prepared_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let first_core = f.core.clone();
    let first_request = request("app", one["device_code"].as_str().unwrap(), None);
    let first = std::thread::spawn(move || {
        with_prepared_pause(
            move || {
                prepared_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(30)).unwrap();
            },
            || keyed(&first_core, first_request, "prepared-first", 1),
        )
    });
    prepared_rx.recv_timeout(Duration::from_secs(30)).unwrap();
    let winner = refusal(keyed(
        &f.core,
        request("app", two["device_code"].as_str().unwrap(), None),
        "committed-second",
        2,
    ));
    assert_eq!(winner.code, "authorization_pending");
    let before = f.snapshot().unwrap();
    release_tx.send(()).unwrap();
    capacity(first.join().unwrap());
    f.assert_snapshot(&before);
    assert_eq!(f.core.store.list::<Value>(BUCKET).unwrap().len(), 1024);
}

#[test]
fn restored_pending_device_poll_receipts_are_invalidated_with_their_proofs() {
    let f = fixture();
    let (started, _) = pending_template(&f);
    let code = text(&started, "device_code");
    assert_eq!(
        riauth::recovery::classify(BUCKET),
        Some(riauth::recovery::Class::Invalidated)
    );
    assert_eq!(
        riauth::recovery::classify("receipts"),
        Some(riauth::recovery::Class::Retained)
    );
    let management = json!({
        "fingerprint": "recovery-management-control",
        "permissions": [],
        "result": {"ok": true},
        "expires_at": now() + 86400
    });
    f.core
        .store
        .write(|tx| tx.put("receipts", "recovery-management-control", &management))
        .unwrap();
    let recovery = riauth::recovery::invalidate_restored(&f.core.store).unwrap();
    assert_eq!(recovery.invalidated[BUCKET], 1);
    assert!(f.core.store.list::<Value>(BUCKET).unwrap().is_empty());
    assert!(f.core.store.list::<Device>("devices").unwrap().is_empty());
    assert_eq!(
        f.core
            .store
            .get::<Value>("receipts", "recovery-management-control")
            .unwrap()
            .unwrap(),
        management
    );
    let before = f.snapshot().unwrap();
    assert_eq!(
        refusal(keyed(&f.core, request("app", &code, None), "first", 1)).code,
        "invalid_grant"
    );
    f.assert_snapshot(&before);
}
