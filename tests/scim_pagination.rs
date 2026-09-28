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
    for (group, user) in [
        ("paged-group-00", "paged-user-00"),
        ("paged-group-01", "paged-user-00"),
        ("paged-group-00", "paged-user-01"),
        ("paged-group-00", "other-paged-user"),
        ("other-paged-group", "paged-user-00"),
    ] {
        f.core.group_member(&f.admin, group, user, true).unwrap();
    }

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

        let before_page = scanned();
        let middle = list(&f, &owner, kind, 1, 8);
        let page_reads = scanned() - before_page;
        assert!(
            page_reads <= 60,
            "{kind} eight-item page scanned {page_reads} records"
        );
        let before_sort = scanned();
        let sorted_page = f
            .core
            .scim_list(
                &owner,
                kind,
                Query {
                    start_index: Some(1),
                    count: Some(8),
                    sort_by: Some("id".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        let sorted_reads = scanned() - before_sort;
        assert!(
            sorted_reads <= 60,
            "{kind} sorted eight-item page scanned {sorted_reads} records"
        );
        assert_eq!(sorted_page, middle);

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
            assert_eq!(&resource, expected);
        }
        let linked = full["Resources"]
            .as_array()
            .unwrap()
            .iter()
            .find(|resource| {
                resource[if kind == "Users" {
                    "userName"
                } else {
                    "displayName"
                }] == if kind == "Users" {
                    "paged-user-00"
                } else {
                    "paged-group-00"
                }
            })
            .unwrap();
        assert_eq!(
            linked[if kind == "Users" { "groups" } else { "members" }]
                .as_array()
                .unwrap()
                .len(),
            2
        );
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

fn scim_user_pages_cross_store_scan_boundary(backend: Backend) {
    let f = backend.fixture();
    let owner = agent(&f, "boundary-scim-owner");
    let mut created = Vec::with_capacity(130);
    for index in 0..130 {
        let user_name = format!("boundary-user-{index:03}");
        let resource = f
            .core
            .scim_write(
                &owner,
                "Users",
                None,
                json!({"schemas":[scim::USER],"userName":user_name}),
                false,
            )
            .unwrap();
        created.push((text(&resource, "id"), user_name));
    }
    created.sort_by(|left, right| left.0.cmp(&right.0));
    let expected: Vec<_> = created.iter().map(|(id, _)| id.clone()).collect();
    // One group makes a full User view scan a measurable opposite-bucket read.
    let group = f
        .core
        .scim_write(
            &owner,
            "Groups",
            None,
            json!({"schemas":[scim::GROUP],"displayName":"boundary-group"}),
            false,
        )
        .unwrap();
    let group_id = text(&group, "id");
    f.core
        .group_member(&f.admin, "boundary-group", &created[128].1, true)
        .unwrap();

    // The store scan reads 128 rows at a time. These pages cover both sides
    // of the cursor transition and must account for every owned resource.
    let first = list(&f, &owner, "Users", 1, 128);
    let tail = list(&f, &owner, "Users", 129, 2);
    let crossing = list(&f, &owner, "Users", 128, 2);
    for (page, start, size) in [(&first, 1, 128), (&tail, 129, 2), (&crossing, 128, 2)] {
        assert_eq!(page["totalResults"], 130);
        assert_eq!(page["startIndex"], start);
        assert_eq!(page["itemsPerPage"], size);
    }
    let ids = |page: &Value| {
        page["Resources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|resource| text(resource, "id"))
            .collect::<Vec<_>>()
    };
    let mut combined = ids(&first);
    combined.extend(ids(&tail));
    assert_eq!(combined, expected);
    assert_eq!(ids(&crossing), expected[127..129]);
    let beyond = list(&f, &owner, "Users", 131, 2);
    assert_eq!(beyond["totalResults"], 130);
    assert_eq!(beyond["itemsPerPage"], 0);

    // A presence filter matches every record and must retain the same bounded
    // key-order pages. A sparse compound filter matches rows on either side
    // of the 128-record store cursor transition.
    let filtered = |filter: &str, start, count| {
        f.core
            .scim_list(
                &owner,
                "Users",
                Query {
                    filter: Some(filter.into()),
                    start_index: Some(start),
                    count: Some(count),
                    ..Default::default()
                },
            )
            .unwrap()
    };
    let filtered_first = filtered("userName pr", 1, 128);
    let filtered_tail = filtered("userName pr", 129, 2);
    assert_eq!(filtered_first["totalResults"], 130);
    assert_eq!(filtered_tail["totalResults"], 130);
    let mut filtered_ids = ids(&filtered_first);
    filtered_ids.extend(ids(&filtered_tail));
    assert_eq!(filtered_ids, expected);

    let boundary_filter = format!(
        "active eq true and (userName eq {} or userName eq {})",
        serde_json::to_string(&created[127].1).unwrap(),
        serde_json::to_string(&created[128].1).unwrap()
    );
    let scanned = || {
        f.core
            .store
            .telemetry()
            .scanned_records
            .load(Ordering::Relaxed)
    };
    let before = scanned();
    let match_before = filtered(&boundary_filter, 1, 1);
    let read_records = scanned() - before;
    assert!(
        read_records <= 140,
        "filtered one-item page scanned {read_records} records"
    );
    let match_after = filtered(&boundary_filter, 2, 1);
    let no_more = filtered(&boundary_filter, 3, 1);
    for (page, start, id) in [
        (&match_before, 1, &expected[127]),
        (&match_after, 2, &expected[128]),
    ] {
        assert_eq!(page["totalResults"], 2);
        assert_eq!(page["startIndex"], start);
        assert_eq!(page["itemsPerPage"], 1);
        assert_eq!(text(&page["Resources"][0], "id"), id.as_str());
        assert_eq!(
            page["Resources"][0],
            f.core.scim_get(&owner, "Users", id).unwrap()
        );
    }
    assert_eq!(no_more["totalResults"], 2);
    assert_eq!(no_more["itemsPerPage"], 0);

    // Group.members resolves a user in the second opposite-bucket scan batch.
    let group_page = list(&f, &owner, "Groups", 1, 1);
    assert_eq!(group_page["totalResults"], 1);
    assert_eq!(
        group_page["Resources"][0]["members"][0]["value"],
        expected[128]
    );
    assert_eq!(
        group_page["Resources"][0],
        f.core.scim_get(&owner, "Groups", &group_id).unwrap()
    );
}

#[test]
fn redb_scim_user_pages_cross_store_scan_boundary() {
    scim_user_pages_cross_store_scan_boundary(Backend::Redb);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_scim_user_pages_cross_store_scan_boundary() {
    scim_user_pages_cross_store_scan_boundary(Backend::Postgres);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_paged_scim_lists() {
    paged_scim_lists_count_without_building_every_resource(Backend::Postgres);
}
