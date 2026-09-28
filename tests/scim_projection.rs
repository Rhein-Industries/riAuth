#![cfg(feature = "platform")]

mod common;

use axum::{Router, body::Body, http::{HeaderMap, Method, Request, StatusCode}};
use common::{Fixture, text};
use riauth::{agent::{NewAgent, Permission}, scim};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn request(app: &Router, token: &str, method: Method, path: String, body: Value) -> (StatusCode, HeaderMap, Value) {
    let response = app.clone().oneshot(Request::builder()
        .method(method.clone())
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/scim+json")
        .body(if method == Method::POST { Body::from(body.to_string()) } else { Body::empty() })
        .unwrap()).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), 32768).await.unwrap();
    (status, headers, serde_json::from_slice(&bytes).unwrap())
}

fn list_path(kind: &str, pairs: &[(&str, &str)]) -> String {
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    for (name, value) in pairs { query.append_pair(name, value); }
    format!("/scim/v2/{kind}?{}", query.finish())
}

#[tokio::test]
async fn scim_projection_preserves_scope_required_fields_and_etag() {
    let f = Fixture::new();
    let owner_permissions: Vec<Permission> = ["user.read", "user.write", "group.read", "group.write", "group.members"]
        .map(|action| Permission { action: action.into(), resource: "*".into() }).into();
    let owner = f.core.create_agent(&f.admin, NewAgent {
        id: "scim-projection-owner".into(), ttl: 600, parent: None, permissions: owner_permissions.clone(),
    }).unwrap();
    let other = f.core.create_agent(&f.admin, NewAgent {
        id: "scim-projection-other".into(), ttl: 600, parent: None, permissions: owner_permissions,
    }).unwrap();
    let writer = f.core.create_agent(&f.admin, NewAgent {
        id: "scim-projection-writer".into(), ttl: 600, parent: None,
        permissions: vec![Permission { action: "user.write".into(), resource: "*".into() }],
    }).unwrap();
    let owner = text(&owner["credential"], "token");
    let other = text(&other["credential"], "token");
    let writer = text(&writer["credential"], "token");
    let user = f.core.scim_write(&owner, "Users", None, json!({
        "schemas":[scim::USER], "userName":"projection-user", "displayName":"Projection User",
        "active":true, "name":{"givenName":"Ada","familyName":"Lovelace","hidden":"not-schema"},
        "emails":[{"value":"ada@example.test","type":"work","primary":true,"hidden":"not-schema"}]
    }), false).unwrap();
    let user_id = text(&user, "id");
    let group = f.core.scim_write(&owner, "Groups", None, json!({
        "schemas":[scim::GROUP], "displayName":"Projection-Group", "members":[{"value":user_id}]
    }), false).unwrap();
    let group_id = text(&group, "id");
    f.core.scim_write(&other, "Users", None, json!({
        "schemas":[scim::USER], "userName":"projection-other", "displayName":"Projection Other"
    }), false).unwrap();
    let writer_user = f.core.scim_write(&writer, "Users", None, json!({
        "schemas":[scim::USER], "userName":"projection-write-only"
    }), false).unwrap();
    let app = riauth::api::router(f.core.clone());

    let user_path = format!("/scim/v2/Users/{user_id}");
    let (status, full_headers, full) = request(&app, &owner, Method::GET, user_path.clone(), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    let etag = full_headers["etag"].clone();
    let location = full_headers["location"].clone();
    assert_eq!(full["meta"]["version"], etag.to_str().unwrap());

    let (status, selected_headers, selected) = request(&app, &owner, Method::GET,
        list_path(&format!("Users/{user_id}"), &[("attributes", "USERNAME,emails.value")]), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(selected_headers["etag"], etag);
    assert_eq!(selected_headers["location"], location);
    assert_eq!(selected["schemas"], json!([scim::USER]));
    assert_eq!(selected["id"], user_id);
    assert_eq!(selected["userName"], "projection-user");
    assert_eq!(selected["emails"], json!([{"value":"ada@example.test"}]));
    for hidden in ["meta", "displayName", "active", "password", "groups", "name"] {
        assert!(selected.get(hidden).is_none(), "{hidden}");
    }

    let (status, excluded_headers, excluded) = request(&app, &owner, Method::GET,
        list_path(&format!("Users/{user_id}"), &[("excludedAttributes", "meta.version,emails.type")]), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(excluded_headers["etag"], etag);
    assert_eq!(excluded["id"], user_id);
    assert_eq!(excluded["schemas"], json!([scim::USER]));
    assert!(excluded["meta"].get("version").is_none());
    assert_eq!(excluded["meta"]["resourceType"], "User");
    assert_eq!(excluded["emails"], json!([{"value":"ada@example.test","primary":true}]));
    assert_eq!(excluded["name"], json!({"givenName":"Ada","familyName":"Lovelace"}));
    assert!(excluded.get("password").is_none());
    let (status, _, required) = request(&app, &owner, Method::GET,
        list_path(&format!("Users/{user_id}"), &[("excludedAttributes", "id,schemas,displayName")]), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(required["id"], user_id);
    assert_eq!(required["schemas"], json!([scim::USER]));
    assert!(required.get("displayName").is_none());

    let (status, _, nested) = request(&app, &owner, Method::GET,
        list_path(&format!("Users/{user_id}"), &[("attributes", "urn:ietf:params:scim:schemas:core:2.0:User:name.givenName,meta.version")]), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(nested["name"], json!({"givenName":"Ada"}));
    assert_eq!(nested["meta"], json!({"version":etag.to_str().unwrap()}));

    let (status, group_headers, group_selected) = request(&app, &owner, Method::GET,
        list_path(&format!("Groups/{group_id}"), &[("attributes", "displayName,members.value")]), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(group_headers["etag"].to_str().unwrap(), group["meta"]["version"]);
    assert_eq!(group_selected["schemas"], json!([scim::GROUP]));
    assert_eq!(group_selected["members"], json!([{"value":user_id}]));
    assert!(group_selected.get("meta").is_none());

    let pairs = [("filter", "userName eq \"projection-user\""), ("sortBy", "userName"),
        ("count", "1"), ("attributes", "userName,emails.value")];
    let (status, _, listed) = request(&app, &owner, Method::GET, list_path("Users", &pairs), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["totalResults"], 1);
    assert_eq!(listed["Resources"][0], selected);
    let (status, _, searched) = request(&app, &owner, Method::POST, "/scim/v2/Users/.search".into(), json!({
        "schemas":["urn:ietf:params:scim:api:messages:2.0:SearchRequest"],
        "filter":"userName eq \"projection-user\"", "sortBy":"userName", "count":1,
        "attributes":["userName","emails.value"]
    })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(searched, listed);
    let (status, _, excluded_search) = request(&app, &owner, Method::POST, "/scim/v2/Users/.search".into(), json!({
        "schemas":["urn:ietf:params:scim:api:messages:2.0:SearchRequest"],
        "filter":"userName eq \"projection-user\"", "excludedAttributes":["meta.version","emails.type"]
    })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(excluded_search["Resources"][0], excluded);
    let (status, _, group_list) = request(&app, &owner, Method::GET,
        list_path("Groups", &[("attributes", "displayName,members.value")]), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(group_list["Resources"][0], group_selected);
    let (status, _, group_search) = request(&app, &owner, Method::POST, "/scim/v2/Groups/.search".into(), json!({
        "schemas":["urn:ietf:params:scim:api:messages:2.0:SearchRequest"],
        "attributes":["displayName","members.value"]
    })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(group_search, group_list);

    for pairs in [
        vec![("attributes", "password")],
        vec![("attributes", "userName"), ("excludedAttributes", "emails")],
        vec![("attributes", "urn:example:other:secret")],
        vec![("attributes", "members.value")],
    ] {
        let (status, _, error) = request(&app, &owner, Method::GET,
            list_path(&format!("Users/{user_id}"), &pairs), json!(null)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error["scimType"], "invalidValue");
    }
    let (status, _, error) = request(&app, &owner, Method::POST, "/scim/v2/Users/.search".into(), json!({
        "schemas":["urn:ietf:params:scim:api:messages:2.0:SearchRequest"], "attributes":"userName"
    })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error["scimType"], "invalidValue");

    let (status, _, _) = request(&app, &other, Method::GET,
        list_path(&format!("Users/{user_id}"), &[("attributes", "id")]), json!(null)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, outside_list) = request(&app, &other, Method::GET,
        list_path("Users", &[("filter", "userName eq \"projection-user\""), ("attributes", "id")]), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(outside_list["totalResults"], 0);
    let (status, _, _) = request(&app, &writer, Method::GET,
        list_path(&format!("Users/{}", text(&writer_user, "id")), &[("attributes", "id")]), json!(null)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, unreadable_list) = request(&app, &writer, Method::GET,
        list_path("Users", &[("attributes", "id")]), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(unreadable_list["totalResults"], 0);
}
