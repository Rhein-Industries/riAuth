#![cfg(feature = "platform")]

mod common;

use axum::http::StatusCode;
use common::Fixture;
use riauth::{
    delegation::{GrantInput, HumanRole},
    model::{User, UserPatch},
};
use serde_json::Value;

#[test]
fn promotion_guard_rolls_back_untrusted_writes_and_removes_prior_authority() {
    let f = Fixture::new();
    f.user("untrusted");
    let untrusted_id: String = f.core.store.get("usernames", "untrusted").unwrap().unwrap();
    f.core
        .set_human_grants(
            &f.admin,
            "untrusted",
            vec![GrantInput {
                role: HumanRole::Auditor,
                scope: "audit/events".into(),
            }],
        )
        .unwrap();
    f.core
        .store
        .write(|tx| tx.delete("elevation_provenance", &untrusted_id))
        .unwrap();
    let before = f.snapshot().unwrap();
    let error = f
        .core
        .store
        .write(|tx| {
            let mut user = tx.get::<User>("users", &untrusted_id)?.unwrap();
            user.admin = true;
            user.epoch += 1;
            tx.put("users", &untrusted_id, &user)
        })
        .unwrap_err();
    assert_eq!(error.status, StatusCode::CONFLICT);
    f.assert_snapshot(&before);

    let prior_session = f.user("ready");
    let ready_id: String = f.core.store.get("usernames", "ready").unwrap().unwrap();
    f.core
        .set_human_grants(
            &f.admin,
            "ready",
            vec![GrantInput {
                role: HumanRole::Auditor,
                scope: "audit/events".into(),
            }],
        )
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "ready",
            UserPatch {
                admin: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(f.core.me(&prior_session).is_err());
    assert!(
        f.core
            .store
            .get::<Value>("human_grants", &ready_id)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .get::<User>("users", &ready_id)
            .unwrap()
            .unwrap()
            .admin
    );
}
