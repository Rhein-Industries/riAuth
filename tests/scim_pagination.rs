#![cfg(feature = "platform")]

mod common;

use axum::http::StatusCode;
use common::{backend::Backend, text};
use riauth::{
    agent::{NewAgent, Permission},
    context::{self, RequestContext},
    error::Result,
    scim::{self, Query},
    telemetry::ReadContext,
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
        // Reverse the sort key relative to insertion order, with pairs that
        // compare equal after case folding and must fall back to SCIM id.
        let display_name = format!(
            "{}-{:03}",
            if index % 2 == 0 { "Rank" } else { "rank" },
            (129 - index) / 2
        );
        let resource = f
            .core
            .scim_write(
                &owner,
                "Users",
                None,
                json!({"schemas":[scim::USER],"userName":user_name,"displayName":display_name}),
                false,
            )
            .unwrap();
        created.push((text(&resource, "id"), user_name, display_name));
    }
    created.sort_by(|left, right| left.0.cmp(&right.0));
    let expected: Vec<_> = created.iter().map(|(id, _, _)| id.clone()).collect();
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

    let mut sorted_expected = created.clone();
    sorted_expected.sort_by(|left, right| {
        left.2
            .to_lowercase()
            .cmp(&right.2.to_lowercase())
            .then_with(|| left.0.cmp(&right.0))
    });
    let sorted_ids: Vec<_> = sorted_expected.iter().map(|row| row.0.clone()).collect();
    let sorted = |start, count, order: &str| {
        f.core
            .scim_list(
                &owner,
                "Users",
                Query {
                    sort_by: Some("displayName".into()),
                    sort_order: Some(order.into()),
                    start_index: Some(start),
                    count: Some(count),
                    ..Default::default()
                },
            )
            .unwrap()
    };
    for (start, count) in [(1, 2), (64, 3), (128, 3), (131, 1)] {
        let page = sorted(start, count, "ascending");
        assert_eq!(page["totalResults"], 130);
        assert_eq!(page["startIndex"], start);
        let expected = sorted_ids
            .iter()
            .skip(start - 1)
            .take(count)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(ids(&page), expected);
        assert_eq!(page["itemsPerPage"], expected.len());
        for resource in page["Resources"].as_array().unwrap() {
            assert_eq!(
                *resource,
                f.core.scim_get(&owner, "Users", &text(resource, "id")).unwrap()
            );
        }
    }
    let descending = sorted(1, 3, "descending");
    assert_eq!(descending["totalResults"], 130);
    let mut descending_expected = sorted_expected.clone();
    descending_expected.sort_by(|left, right| {
        right.2
            .to_lowercase()
            .cmp(&left.2.to_lowercase())
            .then_with(|| left.0.cmp(&right.0))
    });
    assert_eq!(
        ids(&descending),
        descending_expected
            .iter()
            .take(3)
            .map(|row| row.0.clone())
            .collect::<Vec<_>>()
    );
    let reverse_ids = f.core.scim_list(&owner, "Users", Query {
        sort_by: Some("id".into()),
        sort_order: Some("descending".into()),
        start_index: Some(128),
        count: Some(3),
        ..Default::default()
    }).unwrap();
    assert_eq!(reverse_ids["totalResults"], 130);
    assert_eq!(
        ids(&reverse_ids),
        expected.iter().rev().skip(127).cloned().collect::<Vec<_>>()
    );
    let filtered_sorted = f.core.scim_list(&owner, "Users", Query {
        filter: Some(boundary_filter),
        sort_by: Some("displayName".into()),
        count: Some(1),
        ..Default::default()
    }).unwrap();
    assert_eq!(filtered_sorted["totalResults"], 2);
    let filtered_first = sorted_ids.iter()
        .find(|id| **id == expected[127] || **id == expected[128])
        .unwrap();
    assert_eq!(ids(&filtered_sorted), vec![filtered_first.clone()]);

    // Group.members resolves a user in the second opposite-bucket scan batch.
    let group_page = list(&f, &owner, "Groups", 1, 1);
    assert_eq!(group_page["totalResults"], 1);
    assert_eq!(
        group_page["Resources"][0]["members"][0]["value"],
        expected[128]
    );
    let scans = &f.core.store.telemetry().reads;
    let before_unbounded = scans.scans(ReadContext::Read, false).count();
    let before_bounded = scans.scans(ReadContext::Read, true);
    let (before_count, before_rows) = (before_bounded.count(), before_bounded.sum());
    assert_eq!(
        group_page["Resources"][0],
        f.core.scim_get(&owner, "Groups", &group_id).unwrap()
    );
    assert_eq!(scans.scans(ReadContext::Read, false).count(), before_unbounded);
    assert_eq!(scans.scans(ReadContext::Read, true).count() - before_count, 2);
    assert_eq!(scans.scans(ReadContext::Read, true).sum() - before_rows, 130);
}

#[test]
fn redb_scim_user_pages_cross_store_scan_boundary() {
    scim_user_pages_cross_store_scan_boundary(Backend::Redb);
}

#[test]
fn redb_scim_user_get_pages_related_groups() {
    let f = Backend::Redb.fixture();
    let owner = agent(&f, "get-paged-relations-owner");
    let user = f.core.scim_write(
        &owner,
        "Users",
        None,
        json!({"schemas":[scim::USER],"userName":"get-paged-relations-user"}),
        false,
    ).unwrap();
    let user_id = text(&user, "id");
    let mut groups = Vec::new();
    for index in 0..130 {
        let name = format!("get-paged-group-{index:03}");
        let group = f.core.scim_write(
            &owner,
            "Groups",
            None,
            json!({"schemas":[scim::GROUP],"displayName":name}),
            false,
        ).unwrap();
        groups.push((text(&group, "id"), name));
    }
    groups.sort_by(|left, right| left.0.cmp(&right.0));
    for index in [0, 128] {
        f.core.group_member(&f.admin, &groups[index].1, "get-paged-relations-user", true).unwrap();
    }
    let listed = list(&f, &owner, "Users", 1, 1)["Resources"][0].clone();
    assert_eq!(
        listed["groups"].as_array().unwrap().iter().map(|group| text(group, "value")).collect::<Vec<_>>(),
        vec![groups[0].0.clone(), groups[128].0.clone()]
    );

    let scans = &f.core.store.telemetry().reads;
    let before_unbounded = scans.scans(ReadContext::Read, false).count();
    let before_bounded = scans.scans(ReadContext::Read, true);
    let (before_count, before_rows) = (before_bounded.count(), before_bounded.sum());
    let fetched = f.core.scim_get(&owner, "Users", &user_id).unwrap();
    assert_eq!(fetched, listed);
    assert_eq!(scans.scans(ReadContext::Read, false).count(), before_unbounded);
    // One two-row durable-membership index page plus two SCIM Group pages.
    assert_eq!(scans.scans(ReadContext::Read, true).count() - before_count, 3);
    assert_eq!(scans.scans(ReadContext::Read, true).sum() - before_rows, 132);

    let before_bounded = scans.scans(ReadContext::Read, true);
    let (before_count, before_rows) = (before_bounded.count(), before_bounded.sum());
    let (projected, version, location) = f.core.scim_get_projected(
        &owner,
        "Users",
        &user_id,
        scim::ProjectionQuery::default(),
    ).unwrap();
    assert_eq!(projected, listed);
    assert_eq!(version, text(&listed["meta"], "version"));
    assert_eq!(location, text(&listed["meta"], "location"));
    assert_eq!(scans.scans(ReadContext::Read, false).count(), before_unbounded);
    assert_eq!(scans.scans(ReadContext::Read, true).count() - before_count, 3);
    assert_eq!(scans.scans(ReadContext::Read, true).sum() - before_rows, 132);
}

fn patch_user(
    f: &common::Fixture,
    token: &str,
    id: &str,
    version: Option<&str>,
    body: Value,
) -> Result<Value> {
    let core = f.core.clone();
    let token = token.to_owned();
    let id = id.to_owned();
    // HTTP installs a request context even when the header is absent.
    context::scope(
        Some(RequestContext {
            if_match: version.map(str::to_owned),
            ..Default::default()
        }),
        move || core.scim_write(&token, "Users", Some(&id), body, true),
    )
}

fn scim_if_match_pages_relation_versions(backend: Backend) {
    let f = backend.fixture();
    let owner = agent(&f, "if-match-relations-owner");
    let user = f
        .core
        .scim_write(
            &owner,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"if-match-relations-user"}),
            false,
        )
        .unwrap();
    let user_id = text(&user, "id");
    let mut groups = Vec::new();
    for index in 0..130 {
        let name = format!("if-match-group-{index:03}");
        let group = f
            .core
            .scim_write(
                &owner,
                "Groups",
                None,
                json!({"schemas":[scim::GROUP],"displayName":name}),
                false,
            )
            .unwrap();
        groups.push((text(&group, "id"), name));
    }
    groups.sort_by(|left, right| left.0.cmp(&right.0));
    for index in [0, 128] {
        f.core
            .group_member(&f.admin, &groups[index].1, "if-match-relations-user", true)
            .unwrap();
    }
    let current = f.core.scim_get(&owner, "Users", &user_id).unwrap();
    let version = text(&current["meta"], "version");
    assert_eq!(
        current["groups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|group| text(group, "value"))
            .collect::<Vec<_>>(),
        vec![groups[0].0.clone(), groups[128].0.clone()]
    );
    let external_id = json!({
        "schemas": ["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
        "Operations": [{"op": "replace", "path": "externalId", "value": "paged-ext"}]
    });
    let before = f.snapshot().unwrap();
    let missing = patch_user(&f, &owner, &user_id, None, external_id.clone()).unwrap_err();
    assert_eq!(missing.status, StatusCode::PRECONDITION_REQUIRED);
    let stale = patch_user(
        &f,
        &owner,
        &user_id,
        Some("\"stale-relation-version\""),
        external_id.clone(),
    )
    .unwrap_err();
    assert_eq!(stale.status, StatusCode::PRECONDITION_FAILED);
    let renamed = patch_user(
        &f,
        &owner,
        &user_id,
        Some(&version),
        json!({
            "schemas": ["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
            "Operations": [{"op": "replace", "path": "userName", "value": "taken-over"}]
        }),
    )
    .unwrap_err();
    assert_eq!(renamed.status, StatusCode::BAD_REQUEST);
    assert_eq!(renamed.code, "mutability");
    f.assert_http_mutation_snapshot(&before);

    let scans = &f.core.store.telemetry().reads;
    let unbounded_before = scans.scans(ReadContext::Writer, false).sum();
    let bounded_before = scans.scans(ReadContext::Writer, true).sum();
    let updated = patch_user(&f, &owner, &user_id, Some(&version), external_id).unwrap();
    assert_eq!(updated["userName"], "if-match-relations-user");
    assert_eq!(updated["externalId"], "paged-ext");
    assert_ne!(updated["meta"]["version"], version);
    assert_eq!(updated, f.core.scim_get(&owner, "Users", &user_id).unwrap());
    let unbounded_rows = scans.scans(ReadContext::Writer, false).sum() - unbounded_before;
    let bounded_rows = scans.scans(ReadContext::Writer, true).sum() - bounded_before;
    assert!(
        unbounded_rows < 130,
        "conditional write materialized {unbounded_rows} unbounded rows"
    );
    assert!(
        bounded_rows >= 130,
        "conditional write paged only {bounded_rows} related rows"
    );
}

#[test]
fn redb_scim_if_match_pages_relation_versions() {
    scim_if_match_pages_relation_versions(Backend::Redb);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_scim_if_match_pages_relation_versions() {
    scim_if_match_pages_relation_versions(Backend::Postgres);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_encrypted_scim_if_match_pages_relation_versions() {
    scim_if_match_pages_relation_versions(Backend::EncryptedPostgres);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_scim_user_pages_cross_store_scan_boundary() {
    scim_user_pages_cross_store_scan_boundary(Backend::Postgres);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_encrypted_scim_user_pages_cross_store_scan_boundary() {
    scim_user_pages_cross_store_scan_boundary(Backend::EncryptedPostgres);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_paged_scim_lists() {
    paged_scim_lists_count_without_building_every_resource(Backend::Postgres);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_encrypted_paged_scim_lists() {
    paged_scim_lists_count_without_building_every_resource(Backend::EncryptedPostgres);
}
