#![cfg(feature = "test-support")]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::{
    crypto::{now, with_test_time},
    delegation::{GrantChangeBinding, GrantInput, HumanRole},
    error::Result,
    model::{NewUser, User, UserPatch},
};
use serde_json::{Value, json};
use std::sync::{Arc, Barrier};
use tower::ServiceExt;

fn admin(f: &Fixture, name: &str) -> String {
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: name.into(),
                password: PASSWORD.into(),
                email: None,
                display_name: name.into(),
                admin: true,
            },
        )
        .unwrap();
    f.core.login(name.into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .into()
}

fn inputs() -> Vec<GrantInput> {
    vec![GrantInput {
        role: HumanRole::SecurityAdministrator,
        scope: "key/signing".into(),
    }]
}

fn binding(change: &Value) -> GrantChangeBinding {
    GrantChangeBinding {
        digest: change["digest"].as_str().unwrap().into(),
    }
}

fn change_id(change: &Value) -> &str {
    change["proposal"]["id"].as_str().unwrap()
}

fn denied(f: &Fixture, action: impl FnOnce() -> Result<Value>) {
    let before = f.snapshot().unwrap();
    assert!(action().is_err());
    f.assert_snapshot(&before);
}

async fn call(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: Value,
) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn reviewed_grants_bind_content_live_authority_dependencies_and_single_consumption() {
    let f = Fixture::new();
    let reviewer = admin(&f, "reviewer");
    let second_reviewer = admin(&f, "second-reviewer");
    let executor = admin(&f, "executor");
    let alternate = admin(&f, "alternate");
    let holder = f.user("security");
    let holder_id = f.core.me(&holder).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let app = riauth::api::router(f.core.clone());
    let grants_path = "/api/users/security/delegated-grants";
    let revision: u64 = f.core.store.get("meta", "revision").unwrap().unwrap();

    // The old API and direct service cannot bypass the review gate.
    let before = f.snapshot().unwrap();
    let (status, _) = call(&app, "PUT", grants_path, &f.admin, json!(inputs())).await;
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&before);
    denied(&f, || {
        f.core.set_human_grants(&f.admin, "security", inputs())
    });
    denied(&f, || {
        f.core.stage_human_grants(&holder, "security", inputs())
    });

    let (status, change) = call(
        &app,
        "POST",
        &format!("{grants_path}/changes"),
        &f.admin,
        json!(inputs()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{change}");
    let id = change_id(&change);
    assert_eq!(change["proposal"]["before"], json!([]));
    assert_eq!(change["proposal"]["after"][0]["target_id"], "signing");
    assert_eq!(change["proposal"]["base_revision"], revision);
    assert!(change["proposal"]["resource_revision"].as_str().is_some());
    assert!(change["proposal"]["policy_revision"].as_str().is_some());
    assert_eq!(
        change["proposal"]["expires_at"].as_u64().unwrap()
            - change["proposal"]["created_at"].as_u64().unwrap(),
        900
    );
    assert_eq!(
        f.core.human_grants(&f.admin, "security").unwrap()["grants"],
        json!([])
    );

    denied(&f, || {
        f.core
            .execute_human_grant_change(&executor, id, binding(&change))
    });
    denied(&f, || {
        f.core
            .approve_human_grant_change(&f.admin, id, binding(&change))
    });
    denied(&f, || {
        f.core.approve_human_grant_change(
            &reviewer,
            id,
            GrantChangeBinding {
                digest: "substituted".into(),
            },
        )
    });
    // Caller-provided replacement content cannot be smuggled alongside a digest.
    let before = f.snapshot().unwrap();
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/delegated-grant-changes/{id}/approve"),
        &reviewer,
        json!({"digest": change["digest"], "grants": []}),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    f.assert_http_mutation_snapshot(&before);
    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/delegated-grant-changes/{id}/approve"),
        &reviewer,
        json!({"digest": change["digest"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    f.core
        .approve_human_grant_change(&second_reviewer, id, binding(&change))
        .unwrap();
    denied(&f, || {
        f.core
            .approve_human_grant_change(&reviewer, id, binding(&change))
    });
    denied(&f, || {
        f.core
            .execute_human_grant_change(&reviewer, id, binding(&change))
    });
    denied(&f, || {
        f.core
            .execute_human_grant_change(&f.admin, id, binding(&change))
    });

    // Even stored proposal drift cannot execute under the original approval.
    let saved: Value = f
        .core
        .store
        .get("reviewed_human_grants", id)
        .unwrap()
        .unwrap();
    let mut changed = saved.clone();
    changed["proposal"]["after"] = json!([]);
    f.core
        .store
        .write(|tx| tx.put("reviewed_human_grants", id, &changed))
        .unwrap();
    denied(&f, || {
        f.core
            .execute_human_grant_change(&executor, id, binding(&change))
    });
    f.core
        .store
        .write(|tx| tx.put("reviewed_human_grants", id, &saved))
        .unwrap();

    // Mutate rows without the audit/global revision as a stronger live-read oracle.
    // Every participant is checked, including additional reviewers and executor.
    for username in ["admin", "reviewer", "second-reviewer", "executor"] {
        let user_id: String = f.core.store.get("usernames", username).unwrap().unwrap();
        let original: User = f.core.store.get("users", &user_id).unwrap().unwrap();
        let mut reduced = original.clone();
        reduced.admin = false;
        f.core
            .store
            .write(|tx| tx.put("users", &user_id, &reduced))
            .unwrap();
        denied(&f, || {
            f.core
                .execute_human_grant_change(&executor, id, binding(&change))
        });
        f.core
            .store
            .write(|tx| tx.put("users", &user_id, &original))
            .unwrap();
        if username != "executor" {
            let mut renewed = original.clone();
            renewed.epoch += 1;
            f.core
                .store
                .write(|tx| tx.put("users", &user_id, &renewed))
                .unwrap();
            denied(&f, || {
                f.core
                    .execute_human_grant_change(&executor, id, binding(&change))
            });
            f.core
                .store
                .write(|tx| tx.put("users", &user_id, &original))
                .unwrap();
        }
    }
    let generation: Option<u64> = f
        .core
        .store
        .get("human_grant_generations", &holder_id)
        .unwrap();
    f.core
        .store
        .write(|tx| tx.put("human_grant_generations", &holder_id, &99_u64))
        .unwrap();
    denied(&f, || {
        f.core
            .execute_human_grant_change(&executor, id, binding(&change))
    });
    f.core
        .store
        .write(|tx| match generation {
            Some(value) => tx.put("human_grant_generations", &holder_id, &value),
            None => tx.delete("human_grant_generations", &holder_id),
        })
        .unwrap();
    let keys: Value = f.core.store.get("meta", "keys").unwrap().unwrap();
    let mut changed_keys = keys.clone();
    changed_keys["review_dependency_probe"] = json!(true);
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &changed_keys))
        .unwrap();
    denied(&f, || {
        f.core
            .execute_human_grant_change(&executor, id, binding(&change))
    });
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &keys))
        .unwrap();
    let mut changed_policy = f.core.clone();
    changed_policy
        .config
        .pam_approvers
        .insert("privileged".into(), ["security".into()].into());
    denied(&f, || {
        changed_policy.execute_human_grant_change(&executor, id, binding(&change))
    });
    with_test_time(change["proposal"]["expires_at"].as_u64().unwrap(), || {
        denied(&f, || {
            f.core
                .execute_human_grant_change(&executor, id, binding(&change))
        });
    });

    // Two eligible executors race the same approved proposal: one atomic effect.
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = [&executor, &alternate]
        .into_iter()
        .map(|token| {
            let core = f.core.clone();
            let token = token.clone();
            let id = id.to_string();
            let binding = binding(&change);
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                core.execute_human_grant_change(&token, &id, binding)
            })
        })
        .collect();
    let results: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        f.core.human_grants(&f.admin, "security").unwrap()["grants"],
        change["proposal"]["after"]
    );
    denied(&f, || {
        f.core
            .execute_human_grant_change(&executor, id, binding(&change))
    });
    denied(&f, || {
        f.core
            .approve_human_grant_change(&reviewer, id, binding(&change))
    });
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    for action in [
        "reviewed_grants.stage",
        "reviewed_grants.approve",
        "reviewed_grants.execute",
    ] {
        let matching: Vec<_> = events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action && event["details"]["change_id"] == id)
            .collect();
        assert_eq!(
            matching.len(),
            if action.ends_with("approve") { 2 } else { 1 }
        );
        assert!(
            matching
                .iter()
                .all(|event| event["details"]["digest"] == change["digest"])
        );
    }

    // An unchanged high grant can accompany an immediate low-risk edit.
    let mut with_auditor = inputs();
    with_auditor.push(GrantInput {
        role: HumanRole::Auditor,
        scope: "audit/events".into(),
    });
    f.core
        .set_human_grants(&f.admin, "security", with_auditor)
        .unwrap();
    denied(&f, || f.core.set_human_grants(&f.admin, "security", vec![]));
    let revoke = f
        .core
        .stage_human_grants(&f.admin, "security", vec![])
        .unwrap();
    f.core
        .approve_human_grant_change(&reviewer, change_id(&revoke), binding(&revoke))
        .unwrap();
    denied(&f, || {
        f.core
            .execute_human_grant_change(&executor, change_id(&revoke), binding(&change))
    });
    f.core
        .execute_human_grant_change(&executor, change_id(&revoke), binding(&revoke))
        .unwrap();
    assert_eq!(
        f.core.human_grants(&f.admin, "security").unwrap()["grants"],
        json!([])
    );
    assert!(
        f.core
            .store
            .read(|tx| f.core.principal(tx, &holder))
            .is_err()
    );

    // Unrelated management writes conservatively stale the saved revision.
    let stale = f
        .core
        .stage_human_grants(&f.admin, "security", inputs())
        .unwrap();
    f.core
        .approve_human_grant_change(&reviewer, change_id(&stale), binding(&stale))
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "security",
            UserPatch {
                display_name: Some("Changed".into()),
                ..Default::default()
            },
        )
        .unwrap();
    denied(&f, || {
        f.core
            .execute_human_grant_change(&executor, change_id(&stale), binding(&stale))
    });
    f.core
        .cancel_human_grant_change(&f.admin, change_id(&stale), binding(&stale))
        .unwrap();
    denied(&f, || {
        f.core
            .approve_human_grant_change(&reviewer, change_id(&stale), binding(&stale))
    });

    // Expiration is audited before reclaiming unfinished proposals.
    let pending = f
        .core
        .stage_human_grants(&f.admin, "security", inputs())
        .unwrap();
    let at = pending["proposal"]["expires_at"].as_u64().unwrap();
    assert!(at > now());
    with_test_time(at, || {
        denied(&f, || {
            f.core
                .approve_human_grant_change(&reviewer, change_id(&pending), binding(&pending))
        });
        f.core
            .stage_human_grants(&f.admin, "security", inputs())
            .unwrap();
        assert!(
            f.core
                .human_grant_change(&f.admin, change_id(&pending))
                .is_err()
        );
        assert!(
            f.core
                .audit_events(&f.admin, 100)
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["action"] == "reviewed_grants.expire"
                    && event["details"]["change_id"] == change_id(&pending))
        );
    });
    assert!(riauth::recovery::INVALIDATED.contains(&"reviewed_human_grants"));
}
