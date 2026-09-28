#![cfg(feature = "platform")]

mod common;

use common::{backend::Backend, text};
use riauth::{
    agent::{NewAgent, Permission},
    scim::{self, Query},
};
use serde_json::{Value, json};
use std::sync::atomic::Ordering;

fn agent(f: &common::Fixture, id: &str) -> String {
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.into(),
                ttl: 600,
                parent: None,
                permissions: [
                    "user.read",
                    "user.write",
                    "group.read",
                    "group.write",
                    "group.members",
                ]
                .map(|action| Permission {
                    action: action.into(),
                    resource: "*".into(),
                })
                .into(),
            },
        )
        .unwrap();
    text(&created["credential"], "token")
}

fn list(f: &common::Fixture, token: &str, kind: &str, start: usize, count: usize) -> Value {
    f.core
        .scim_list(
            token,
            kind,
            Query {
                start_index: Some(start),
                count: Some(count),
                ..Default::default()
            },
        )
        .unwrap()
}

fn paged_scim_lists_count_without_building_every_resource(backend: Backend) {
    let f = backend.fixture();
    let owner = agent(&f, "paged-scim-owner");
    let other = agent(&f, "paged-scim-other");
    for index in 0..16 {
        f.core
            .scim_write(
                &owner,
                "Users",
                None,
                json!({"schemas":[scim::USER],"userName":format!("paged-user-{index:02}")}),
                false,
            )
            .unwrap();
        f.core
            .scim_write(
                &owner,
                "Groups",
                None,
                json!({"schemas":[scim::GROUP],"displayName":format!("paged-group-{index:02}")}),
                false,
            )
            .unwrap();
    }
    f.core
        .scim_write(
            &other,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"other-paged-user"}),
            false,
        )
        .unwrap();
    f.core
        .scim_write(
            &other,
            "Groups",
            None,
            json!({"schemas":[scim::GROUP],"displayName":"other-paged-group"}),
            false,
        )
        .unwrap();

    for kind in ["Users", "Groups"] {
        let scanned = || {
            f.core
                .store
                .telemetry()
                .scanned_records
                .load(Ordering::Relaxed)
        };
        let before = scanned();
        let first = list(&f, &owner, kind, 1, 1);
        let read_records = scanned() - before;
        assert!(
            read_records <= 40,
            "{kind} first page scanned {read_records} records"
        );
        assert_eq!(first["totalResults"], 16);
        assert_eq!(first["itemsPerPage"], 1);

        let full = list(&f, &owner, kind, 1, 100);
        let ids: Vec<_> = full["Resources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|resource| text(resource, "id"))
            .collect();
        assert_eq!(ids.len(), 16);
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
        for (index, expected) in full["Resources"].as_array().unwrap().iter().enumerate() {
            let page = list(&f, &owner, kind, index + 1, 1);
            assert_eq!(page["totalResults"], 16);
            assert_eq!(&page["Resources"][0], expected);
            let resource = f.core.scim_get(&owner, kind, &ids[index]).unwrap();
            assert_eq!(resource["meta"]["version"], expected["meta"]["version"]);
        }
        assert_eq!(list(&f, &owner, kind, 17, 1)["itemsPerPage"], 0);
        assert_eq!(list(&f, &owner, kind, 1, 0)["totalResults"], 16);

        let filtered = f
            .core
            .scim_list(
                &owner,
                kind,
                Query {
                    filter: Some("id pr".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(filtered, full);
        let sorted = f
            .core
            .scim_list(
                &owner,
                kind,
                Query {
                    sort_by: Some("id".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(sorted, full);
    }
}

#[test]
fn redb_paged_scim_lists() {
    paged_scim_lists_count_without_building_every_resource(Backend::Redb);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_paged_scim_lists() {
    paged_scim_lists_count_without_building_every_resource(Backend::Postgres);
}
