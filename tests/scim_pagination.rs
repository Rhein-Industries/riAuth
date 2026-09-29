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
                f.core
                    .scim_get(&owner, "Users", &text(resource, "id"))
                    .unwrap()
            );
        }
    }
    let descending = sorted(1, 3, "descending");
    assert_eq!(descending["totalResults"], 130);
    let mut descending_expected = sorted_expected.clone();
    descending_expected.sort_by(|left, right| {
        right
            .2
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
    let reverse_ids = f
        .core
        .scim_list(
            &owner,
            "Users",
            Query {
                sort_by: Some("id".into()),
                sort_order: Some("descending".into()),
                start_index: Some(128),
                count: Some(3),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(reverse_ids["totalResults"], 130);
    assert_eq!(
        ids(&reverse_ids),
        expected.iter().rev().skip(127).cloned().collect::<Vec<_>>()
    );
    let filtered_sorted = f
        .core
        .scim_list(
            &owner,
            "Users",
            Query {
                filter: Some(boundary_filter),
                sort_by: Some("displayName".into()),
                count: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(filtered_sorted["totalResults"], 2);
    let filtered_first = sorted_ids
        .iter()
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
    assert_eq!(
        scans.scans(ReadContext::Read, false).count(),
        before_unbounded
    );
    assert_eq!(
        scans.scans(ReadContext::Read, true).count() - before_count,
        2
    );
    assert_eq!(
        scans.scans(ReadContext::Read, true).sum() - before_rows,
        130
    );
}

#[test]
fn redb_scim_user_pages_cross_store_scan_boundary() {
    scim_user_pages_cross_store_scan_boundary(Backend::Redb);
}

#[test]
fn redb_scim_user_get_pages_related_groups() {
    let f = Backend::Redb.fixture();
    let owner = agent(&f, "get-paged-relations-owner");
    let user = f
        .core
        .scim_write(
            &owner,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"get-paged-relations-user"}),
            false,
        )
        .unwrap();
    let user_id = text(&user, "id");
    let mut groups = Vec::new();
    for index in 0..130 {
        let name = format!("get-paged-group-{index:03}");
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
            .group_member(&f.admin, &groups[index].1, "get-paged-relations-user", true)
            .unwrap();
    }
    let listed = list(&f, &owner, "Users", 1, 1)["Resources"][0].clone();
    assert_eq!(
        listed["groups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|group| text(group, "value"))
            .collect::<Vec<_>>(),
        vec![groups[0].0.clone(), groups[128].0.clone()]
    );

    let scans = &f.core.store.telemetry().reads;
    let before_unbounded = scans.scans(ReadContext::Read, false).count();
    let before_bounded = scans.scans(ReadContext::Read, true);
    let (before_count, before_rows) = (before_bounded.count(), before_bounded.sum());
    let fetched = f.core.scim_get(&owner, "Users", &user_id).unwrap();
    assert_eq!(fetched, listed);
    assert_eq!(
        scans.scans(ReadContext::Read, false).count(),
        before_unbounded
    );
    // One two-row durable-membership index page plus two SCIM Group pages.
    assert_eq!(
        scans.scans(ReadContext::Read, true).count() - before_count,
        3
    );
    assert_eq!(
        scans.scans(ReadContext::Read, true).sum() - before_rows,
        132
    );

    let before_bounded = scans.scans(ReadContext::Read, true);
    let (before_count, before_rows) = (before_bounded.count(), before_bounded.sum());
    let (projected, version, location) = f
        .core
        .scim_get_projected(&owner, "Users", &user_id, scim::ProjectionQuery::default())
        .unwrap();
    assert_eq!(projected, listed);
    assert_eq!(version, text(&listed["meta"], "version"));
    assert_eq!(location, text(&listed["meta"], "location"));
    assert_eq!(
        scans.scans(ReadContext::Read, false).count(),
        before_unbounded
    );
    assert_eq!(
        scans.scans(ReadContext::Read, true).count() - before_count,
        3
    );
    assert_eq!(
        scans.scans(ReadContext::Read, true).sum() - before_rows,
        132
    );
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

fn one_listed(f: &common::Fixture, token: &str, kind: &str, filter: &str) -> Value {
    let page = f
        .core
        .scim_list(
            token,
            kind,
            Query {
                filter: Some(filter.to_owned()),
                count: Some(10),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(page["totalResults"], 1);
    page["Resources"][0].clone()
}

fn scim_management_membership_pages_version_publication(backend: Backend) {
    let f = backend.fixture();
    let owner = agent(&f, "transition-owner");
    let user = f
        .core
        .scim_write(
            &owner,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"transition-user","displayName":"Transition User"}),
            false,
        )
        .unwrap();
    let user_id = text(&user, "id");
    for index in 0..129 {
        f.core
            .scim_write(
                &owner,
                "Users",
                None,
                json!({"schemas":[scim::USER],"userName":format!("transition-filler-{index:03}")}),
                false,
            )
            .unwrap();
    }
    let group = f
        .core
        .scim_write(
            &owner,
            "Groups",
            None,
            json!({"schemas":[scim::GROUP],"displayName":"transition-group"}),
            false,
        )
        .unwrap();
    let group_id = text(&group, "id");
    let user_before = f.core.scim_get(&owner, "Users", &user_id).unwrap();
    let group_before = f.core.scim_get(&owner, "Groups", &group_id).unwrap();
    assert_eq!(
        user_before,
        one_listed(&f, &owner, "Users", "userName eq \"transition-user\"")
    );
    assert_eq!(
        group_before,
        one_listed(&f, &owner, "Groups", "displayName eq \"transition-group\"")
    );
    assert!(user_before["groups"].as_array().unwrap().is_empty());
    assert!(group_before["members"].as_array().unwrap().is_empty());
    let user_version_before = text(&user_before["meta"], "version");
    let group_version_before = text(&group_before["meta"], "version");

    let scans = &f.core.store.telemetry().reads;
    let unbounded_before = scans.scans(ReadContext::Writer, false).sum();
    let bounded_before = scans.scans(ReadContext::Writer, true).sum();
    f.core
        .group_member(&f.admin, "transition-group", "transition-user", true)
        .unwrap();
    let unbounded_rows = scans.scans(ReadContext::Writer, false).sum() - unbounded_before;
    let bounded_rows = scans.scans(ReadContext::Writer, true).sum() - bounded_before;
    assert!(
        unbounded_rows < 130,
        "management membership write materialized {unbounded_rows} unbounded rows"
    );
    assert!(
        bounded_rows >= 130,
        "management membership write paged only {bounded_rows} SCIM rows"
    );

    let user_added = f.core.scim_get(&owner, "Users", &user_id).unwrap();
    let group_added = f.core.scim_get(&owner, "Groups", &group_id).unwrap();
    assert_eq!(
        user_added,
        one_listed(&f, &owner, "Users", "userName eq \"transition-user\"")
    );
    assert_eq!(
        group_added,
        one_listed(&f, &owner, "Groups", "displayName eq \"transition-group\"")
    );
    assert_eq!(
        user_added["groups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|group| text(group, "value"))
            .collect::<Vec<_>>(),
        vec![group_id.clone()]
    );
    assert_eq!(
        group_added["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|member| text(member, "value"))
            .collect::<Vec<_>>(),
        vec![user_id.clone()]
    );
    let user_version_added = text(&user_added["meta"], "version");
    let group_version_added = text(&group_added["meta"], "version");
    assert_ne!(user_version_added, user_version_before);
    assert_ne!(group_version_added, group_version_before);

    f.core
        .group_member(&f.admin, "transition-group", "transition-user", false)
        .unwrap();
    let user_restored = f.core.scim_get(&owner, "Users", &user_id).unwrap();
    let group_restored = f.core.scim_get(&owner, "Groups", &group_id).unwrap();
    assert_eq!(
        user_restored,
        one_listed(&f, &owner, "Users", "userName eq \"transition-user\"")
    );
    assert_eq!(
        group_restored,
        one_listed(&f, &owner, "Groups", "displayName eq \"transition-group\"")
    );
    assert!(user_restored["groups"].as_array().unwrap().is_empty());
    assert!(group_restored["members"].as_array().unwrap().is_empty());
    let user_version = text(&user_restored["meta"], "version");
    let group_version = text(&group_restored["meta"], "version");
    assert_ne!(user_version, user_version_before);
    assert_ne!(user_version, user_version_added);
    assert_ne!(group_version, group_version_before);
    assert_ne!(group_version, group_version_added);

    let rename = json!({
        "schemas": ["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
        "Operations": [{"op": "replace", "path": "displayName", "value": "Published"}]
    });
    let before = f.snapshot().unwrap();
    let missing = patch_user(&f, &owner, &user_id, None, rename.clone()).unwrap_err();
    assert_eq!(missing.status, StatusCode::PRECONDITION_REQUIRED);
    let stale = patch_user(
        &f,
        &owner,
        &user_id,
        Some(&user_version_before),
        rename.clone(),
    )
    .unwrap_err();
    assert_eq!(stale.status, StatusCode::PRECONDITION_FAILED);
    let taken = patch_user(
        &f,
        &owner,
        &user_id,
        Some(&user_version),
        json!({
            "schemas": ["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
            "Operations": [{"op": "replace", "path": "userName", "value": "taken-over"}]
        }),
    )
    .unwrap_err();
    assert_eq!(taken.status, StatusCode::BAD_REQUEST);
    assert_eq!(taken.code, "mutability");
    f.assert_http_mutation_snapshot(&before);

    let updated = patch_user(&f, &owner, &user_id, Some(&user_version), rename).unwrap();
    assert_eq!(updated["userName"], "transition-user");
    assert_eq!(updated["displayName"], "Published");
    assert_ne!(updated["meta"]["version"], user_version);
    assert_eq!(updated, f.core.scim_get(&owner, "Users", &user_id).unwrap());
    assert_eq!(
        f.core.scim_get(&owner, "Groups", &group_id).unwrap()["meta"]["version"],
        group_version
    );
}

#[test]
fn redb_scim_management_membership_pages_version_publication() {
    scim_management_membership_pages_version_publication(Backend::Redb);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_scim_management_membership_pages_version_publication() {
    scim_management_membership_pages_version_publication(Backend::Postgres);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_encrypted_scim_management_membership_pages_version_publication() {
    scim_management_membership_pages_version_publication(Backend::EncryptedPostgres);
}

fn writer_bounded_scans(f: &common::Fixture) -> (u64, u64, u64) {
    let mut output = String::new();
    f.core.store.telemetry().render(&mut output);
    let mut within_page = 0;
    let mut scans = 0;
    for line in output.lines() {
        if let Some(value) = line.strip_prefix(
            "riauth_storage_scan_rows_bucket{context=\"writer\",limit=\"bounded\",le=\"128\"} ",
        ) {
            within_page = value.parse().unwrap();
        } else if let Some(value) = line.strip_prefix(
            "riauth_storage_scan_rows_bucket{context=\"writer\",limit=\"bounded\",le=\"+Inf\"} ",
        ) {
            scans = value.parse().unwrap();
        }
    }
    let rows = f
        .core
        .store
        .telemetry()
        .reads
        .scans(ReadContext::Writer, true)
        .sum();
    (scans, within_page, rows)
}

fn scim_one_group_publication_spans_owners_and_pages(backend: Backend) {
    let f = backend.fixture();
    let owner = agent(&f, "cap-owner");
    let other = agent(&f, "cap-other");
    let user = f
        .core
        .scim_write(
            &owner,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"cap-user","displayName":"Cap User"}),
            false,
        )
        .unwrap();
    let user_id = text(&user, "id");
    let group = f
        .core
        .scim_write(
            &owner,
            "Groups",
            None,
            json!({"schemas":[scim::GROUP],"displayName":"cap-group"}),
            false,
        )
        .unwrap();
    let group_id = text(&group, "id");
    let mut other_user_id = String::new();
    let mut other_group_id = String::new();
    for index in 0..129 {
        let other_user = f
            .core
            .scim_write(
                &other,
                "Users",
                None,
                json!({"schemas":[scim::USER],"userName":format!("cap-other-user-{index:03}")}),
                false,
            )
            .unwrap();
        let other_group = f
            .core
            .scim_write(
                &other,
                "Groups",
                None,
                json!({"schemas":[scim::GROUP],"displayName":format!("cap-other-group-{index:03}")}),
                false,
            )
            .unwrap();
        if index == 0 {
            other_user_id = text(&other_user, "id");
            other_group_id = text(&other_group, "id");
        }
    }

    let version = |kind: &str, id: &str| -> String {
        text(
            &f.core.scim_get(&owner, kind, id).unwrap()["meta"],
            "version",
        )
    };
    let other_version = |kind: &str, id: &str| -> String {
        text(
            &f.core.scim_get(&other, kind, id).unwrap()["meta"],
            "version",
        )
    };
    let user_before = version("Users", &user_id);
    let group_before = version("Groups", &group_id);
    let other_user_before = other_version("Users", &other_user_id);
    let other_group_before = other_version("Groups", &other_group_id);

    let scans = &f.core.store.telemetry().reads;
    let unbounded_before = scans.scans(ReadContext::Writer, false).count();
    let (bounded_scans_before, page_scans_before, bounded_rows_before) = writer_bounded_scans(&f);
    f.core
        .group_member(&f.admin, "cap-group", "cap-other-user-000", true)
        .unwrap();
    let foreign_scans = scans.scans(ReadContext::Writer, true).count() - bounded_scans_before;
    let foreign_rows = scans.scans(ReadContext::Writer, true).sum() - bounded_rows_before;
    assert_eq!(
        scans.scans(ReadContext::Writer, false).count(),
        unbounded_before,
        "foreign membership write used an unbounded scan"
    );
    assert!(
        foreign_scans >= 4,
        "foreign membership write used only {foreign_scans} bounded scans"
    );
    assert!(
        foreign_rows >= 260,
        "foreign membership write read only {foreign_rows} rows"
    );
    assert!(
        foreign_rows <= foreign_scans * 128,
        "foreign membership write retained {foreign_rows} rows in {foreign_scans} scans"
    );
    let (bounded_scans_after, page_scans_after, _) = writer_bounded_scans(&f);
    assert_eq!(
        bounded_scans_after - bounded_scans_before,
        page_scans_after - page_scans_before,
        "a writer scan materialized more than 128 rows"
    );
    assert_eq!(version("Users", &user_id), user_before);
    assert_eq!(version("Groups", &group_id), group_before);
    assert_eq!(other_version("Users", &other_user_id), other_user_before);
    assert_eq!(other_version("Groups", &other_group_id), other_group_before);
    assert!(
        f.core.scim_get(&owner, "Groups", &group_id).unwrap()["members"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let (bounded_scans_before, page_scans_before, bounded_rows_before) = writer_bounded_scans(&f);
    let unbounded_before = scans.scans(ReadContext::Writer, false).count();
    f.core
        .group_member(&f.admin, "cap-group", "cap-user", true)
        .unwrap();
    let owned_scans = scans.scans(ReadContext::Writer, true).count() - bounded_scans_before;
    let owned_rows = scans.scans(ReadContext::Writer, true).sum() - bounded_rows_before;
    assert_eq!(
        scans.scans(ReadContext::Writer, false).count(),
        unbounded_before
    );
    assert!(
        owned_scans >= 4,
        "owned membership write used only {owned_scans} scans"
    );
    assert!(
        owned_rows >= 260,
        "owned membership write read only {owned_rows} rows"
    );
    assert!(
        owned_rows <= owned_scans * 128,
        "owned membership write retained {owned_rows} rows in {owned_scans} scans"
    );
    let (bounded_scans_after, page_scans_after, _) = writer_bounded_scans(&f);
    assert_eq!(
        bounded_scans_after - bounded_scans_before,
        page_scans_after - page_scans_before,
        "a writer scan materialized more than 128 rows"
    );
    let published_user = f.core.scim_get(&owner, "Users", &user_id).unwrap();
    let published_group = f.core.scim_get(&owner, "Groups", &group_id).unwrap();
    assert_eq!(
        published_user,
        one_listed(&f, &owner, "Users", "userName eq \"cap-user\"")
    );
    assert_eq!(
        published_group,
        one_listed(&f, &owner, "Groups", "displayName eq \"cap-group\"")
    );
    assert_ne!(text(&published_user["meta"], "version"), user_before);
    assert_ne!(text(&published_group["meta"], "version"), group_before);
    assert_eq!(
        published_group["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|member| text(member, "value"))
            .collect::<Vec<_>>(),
        vec![user_id.clone()]
    );
    assert_eq!(other_version("Users", &other_user_id), other_user_before);
    assert_eq!(other_version("Groups", &other_group_id), other_group_before);
    assert!(
        f.core.scim_get(&other, "Users", &other_user_id).unwrap()["groups"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    f.core
        .store
        .write(|tx| {
            let mut duplicate: Value = tx.get("scim_groups", &group_id)?.unwrap();
            let other_record: Value = tx.get("scim_groups", &other_group_id)?.unwrap();
            duplicate["owner"] = other_record["owner"].clone();
            tx.put("scim_groups", "cap-duplicate-group", &duplicate)
        })
        .unwrap();
    let user_published = version("Users", &user_id);
    let group_published = version("Groups", &group_id);
    let other_user_published = other_version("Users", &other_user_id);
    let other_group_published = other_version("Groups", &other_group_id);
    let before = f.snapshot().unwrap();
    let (bounded_scans_before, page_scans_before, bounded_rows_before) = writer_bounded_scans(&f);
    let unbounded_before = scans.scans(ReadContext::Writer, false).count();
    let duplicate = f
        .core
        .group_member(&f.admin, "cap-group", "cap-user", false)
        .unwrap_err();
    assert_eq!(duplicate.status, StatusCode::CONFLICT);
    assert_eq!(duplicate.message, "SCIM group identity is not unique");
    let duplicate_scans = scans.scans(ReadContext::Writer, true).count() - bounded_scans_before;
    let duplicate_rows = scans.scans(ReadContext::Writer, true).sum() - bounded_rows_before;
    assert_eq!(
        scans.scans(ReadContext::Writer, false).count(),
        unbounded_before
    );
    assert!(
        duplicate_rows < 260,
        "duplicate group publication scanned {duplicate_rows} rows across both collections"
    );
    assert!(
        duplicate_rows <= duplicate_scans * 128,
        "duplicate group publication retained {duplicate_rows} rows in {duplicate_scans} scans"
    );
    let (bounded_scans_after, page_scans_after, _) = writer_bounded_scans(&f);
    assert_eq!(
        bounded_scans_after - bounded_scans_before,
        page_scans_after - page_scans_before
    );
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(version("Users", &user_id), user_published);
    assert_eq!(version("Groups", &group_id), group_published);
    assert_eq!(other_version("Users", &other_user_id), other_user_published);
    assert_eq!(
        other_version("Groups", &other_group_id),
        other_group_published
    );
    assert_eq!(
        f.core.scim_get(&owner, "Groups", &group_id).unwrap()["members"],
        published_group["members"]
    );
}

#[test]
fn redb_scim_one_group_publication_spans_owners_and_pages() {
    scim_one_group_publication_spans_owners_and_pages(Backend::Redb);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_scim_one_group_publication_spans_owners_and_pages() {
    scim_one_group_publication_spans_owners_and_pages(Backend::Postgres);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_encrypted_scim_one_group_publication_spans_owners_and_pages() {
    scim_one_group_publication_spans_owners_and_pages(Backend::EncryptedPostgres);
}

fn scim_group_member_replace_pages_owned_users(backend: Backend) {
    let f = backend.fixture();
    let owner = agent(&f, "scope-owner");
    let other = agent(&f, "scope-other");
    f.core
        .scim_write(
            &owner,
            "Groups",
            None,
            json!({"schemas":[scim::GROUP],"displayName":"scope-group"}),
            false,
        )
        .unwrap();
    let mut first_id = String::new();
    let mut last_id = String::new();
    for index in 0..130 {
        let user = f
            .core
            .scim_write(
                &owner,
                "Users",
                None,
                json!({"schemas":[scim::USER],"userName":format!("scope-user-{index:03}")}),
                false,
            )
            .unwrap();
        if index == 0 {
            first_id = text(&user, "id");
        }
        if index == 129 {
            last_id = text(&user, "id");
        }
    }
    let foreign = f
        .core
        .scim_write(
            &other,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"scope-foreign"}),
            false,
        )
        .unwrap();
    let foreign_id = text(&foreign, "id");
    let foreign_local: String = f
        .core
        .store
        .get("usernames", "scope-foreign")
        .unwrap()
        .unwrap();
    let first_local: String = f
        .core
        .store
        .get("usernames", "scope-user-000")
        .unwrap()
        .unwrap();
    let last_local: String = f
        .core
        .store
        .get("usernames", "scope-user-129")
        .unwrap()
        .unwrap();
    let kept_local: String = f
        .core
        .store
        .get("usernames", "scope-user-001")
        .unwrap()
        .unwrap();
    f.core
        .group_member(&f.admin, "scope-group", "scope-foreign", true)
        .unwrap();
    f.core
        .group_member(&f.admin, "scope-group", "scope-user-000", true)
        .unwrap();
    let version = |token: &str, id: &str| -> String {
        text(
            &f.core.scim_get(token, "Users", id).unwrap()["meta"],
            "version",
        )
    };
    let group = f
        .core
        .scim_list(
            &owner,
            "Groups",
            Query {
                filter: Some("displayName eq \"scope-group\"".into()),
                count: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
    let group_id = text(&group["Resources"][0], "id");
    let group_before = text(
        &f.core.scim_get(&owner, "Groups", &group_id).unwrap()["meta"],
        "version",
    );
    let first_before = version(&owner, &first_id);
    let last_before = version(&owner, &last_id);
    let kept_id = text(
        &f.core
            .scim_list(
                &owner,
                "Users",
                Query {
                    filter: Some("userName eq \"scope-user-001\"".into()),
                    count: Some(1),
                    ..Default::default()
                },
            )
            .unwrap()["Resources"][0],
        "id",
    );
    let kept_before = version(&owner, &kept_id);
    let foreign_before = version(&other, &foreign_id);

    let scans = &f.core.store.telemetry().reads;
    let unbounded_before = scans.scans(ReadContext::Writer, false).count();
    let (bounded_before, page_before, rows_before) = writer_bounded_scans(&f);
    let replaced = f
        .core
        .scim_write(
            &owner,
            "Groups",
            Some(&group_id),
            json!({
                "schemas":[scim::GROUP],
                "displayName":"scope-group",
                "members":[{"value": last_id}]
            }),
            false,
        )
        .unwrap();
    let bounded_scans = scans.scans(ReadContext::Writer, true).count() - bounded_before;
    let bounded_rows = scans.scans(ReadContext::Writer, true).sum() - rows_before;
    assert_eq!(
        scans.scans(ReadContext::Writer, false).count(),
        unbounded_before,
        "group member replace listed the SCIM user collection"
    );
    assert!(
        bounded_rows >= 130,
        "group member replace read only {bounded_rows} rows"
    );
    assert!(
        bounded_rows <= bounded_scans * 128,
        "group member replace retained {bounded_rows} rows in {bounded_scans} scans"
    );
    let (bounded_after, page_after, _) = writer_bounded_scans(&f);
    assert_eq!(
        bounded_after - bounded_before,
        page_after - page_before,
        "a writer scan materialized more than 128 rows"
    );
    assert_eq!(
        replaced["members"]
            .as_array()
            .unwrap()
            .iter()
            .map(|member| text(member, "value"))
            .collect::<Vec<_>>(),
        vec![last_id.clone()]
    );
    let stored: Value = f.core.store.get("groups", "scope-group").unwrap().unwrap();
    let directory: Vec<_> = stored["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|member| member.as_str().unwrap().to_owned())
        .collect();
    assert!(directory.contains(&foreign_local));
    assert!(directory.contains(&last_local));
    assert!(!directory.contains(&first_local));
    assert!(!directory.contains(&kept_local));
    assert_eq!(directory.len(), 2);
    assert_ne!(version(&owner, &first_id), first_before);
    assert_ne!(version(&owner, &last_id), last_before);
    assert_ne!(
        text(
            &f.core.scim_get(&owner, "Groups", &group_id).unwrap()["meta"],
            "version"
        ),
        group_before
    );
    assert_eq!(version(&owner, &kept_id), kept_before);
    assert_eq!(version(&other, &foreign_id), foreign_before);
    assert_eq!(
        replaced,
        f.core.scim_get(&owner, "Groups", &group_id).unwrap()
    );
}

#[test]
fn redb_scim_group_member_replace_pages_owned_users() {
    scim_group_member_replace_pages_owned_users(Backend::Redb);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_scim_group_member_replace_pages_owned_users() {
    scim_group_member_replace_pages_owned_users(Backend::Postgres);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_encrypted_scim_group_member_replace_pages_owned_users() {
    scim_group_member_replace_pages_owned_users(Backend::EncryptedPostgres);
}
