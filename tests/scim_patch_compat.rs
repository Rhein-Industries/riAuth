#![cfg(feature = "platform")]

mod common;

use axum::{body::Body, http::{Method, Request, StatusCode}, Router};
use common::{text, Fixture};
use riauth::{agent::{NewAgent, Permission}, scim};
use serde_json::{json, Value};
use tower::ServiceExt;

fn agent(f: &Fixture, name: &str) -> String {
    let created = f.core.create_agent(&f.admin, NewAgent {
        id: name.into(), ttl: 600, parent: None,
        permissions: ["user.read", "user.write", "group.read", "group.write", "group.members"]
            .map(|action| Permission { action: action.into(), resource: "*".into() }).into(),
    }).unwrap();
    text(&created["credential"], "token")
}

async fn request(app: &Router, method: Method, path: &str, token: &str, version: Option<&str>, body: Option<&Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path)
        .header("authorization", format!("Bearer {token}"));
    if let Some(version) = version { builder = builder.header("if-match", version); }
    if body.is_some() { builder = builder.header("content-type", "application/scim+json"); }
    let response = app.clone().oneshot(builder.body(Body::from(body.map_or_else(String::new, Value::to_string))).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32768).await.unwrap();
    (status, if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() })
}

fn patch(operations: Value) -> Value {
    json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":operations})
}

fn version(value: &Value) -> &str { value["meta"]["version"].as_str().unwrap() }

#[tokio::test]
async fn full_user_writes_reject_invalid_email_entries_atomically() {
    let f = Fixture::new();
    let token = agent(&f, "email-write-agent");
    let app = riauth::api::router(f.core.clone());
    let path = "/scim/v2/Users";
    let valid = json!({"schemas":[scim::USER],"userName":"email-write-user",
        "emails":[{"value":"first@example.test","type":"work","primary":true,
                   "extra":"preserve"}]});
    let invalid_emails = [
        json!([{"value":"first@example.test"},{"value":"FIRST@example.test"}]),
        json!([{"value":"first@example.test","type":7}]),
        json!([{"value":"first@example.test","primary":"true"}]),
    ];
    let before_create = f.snapshot().unwrap();
    for emails in &invalid_emails {
        let mut input = valid.clone();
        input["emails"] = emails.clone();
        let (status, _) = request(&app, Method::POST, path, &token, None, Some(&input)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{emails}");
        f.assert_http_mutation_snapshot(&before_create);
    }

    let (status, created) = request(&app, Method::POST, path, &token, None, Some(&valid)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["emails"][0]["extra"], "preserve");
    let resource_path = format!("{path}/{}", text(&created, "id"));
    let before_replace = f.snapshot().unwrap();
    for emails in &invalid_emails {
        let mut input = valid.clone();
        input["emails"] = emails.clone();
        let (status, _) = request(&app, Method::PUT, &resource_path, &token,
            Some(version(&created)), Some(&input)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{emails}");
        f.assert_http_mutation_snapshot(&before_replace);
    }
    let current = f.core.scim_get(&token, "Users", created["id"].as_str().unwrap()).unwrap();
    assert_eq!(current, created);
}

#[tokio::test]
async fn stored_extra_user_subattributes_survive_roundtrip_and_typed_patch() {
    let f = Fixture::new();
    let token = agent(&f, "compat-user-agent");
    let app = riauth::api::router(f.core.clone());
    let input = json!({"schemas":[scim::USER],"userName":"compat-user",
        "name":{"givenName":"Ada","hidden":"kept-name"},
        "emails":[{"value":"ada@example.test","type":"work","primary":true,"hidden":"kept-email"}]});
    let (status, created) = request(&app, Method::POST, "/scim/v2/Users", &token, None, Some(&input)).await;
    assert_eq!(status, StatusCode::CREATED);
    let path = format!("/scim/v2/Users/{}", text(&created, "id"));
    assert_eq!(created["name"]["hidden"], "kept-name");
    assert_eq!(created["emails"][0]["hidden"], "kept-email");

    let (status, echoed) = request(&app, Method::PUT, &path, &token, Some(version(&created)), Some(&created)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(echoed["name"]["hidden"], "kept-name");
    assert_eq!(echoed["emails"][0]["hidden"], "kept-email");

    let name_change = patch(json!([{"op":"replace","path":"name.givenName","value":"Augusta"}]));
    let (status, renamed) = request(&app, Method::PATCH, &path, &token, Some(version(&echoed)), Some(&name_change)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(renamed["name"], json!({"givenName":"Augusta","hidden":"kept-name"}));
    assert_ne!(version(&renamed), version(&echoed));

    let email_change = patch(json!([{"op":"replace","path":"emails[value eq \"ada@example.test\"].type","value":"home"}]));
    let (status, changed) = request(&app, Method::PATCH, &path, &token, Some(version(&renamed)), Some(&email_change)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(changed["emails"][0]["type"], "home");
    assert_eq!(changed["emails"][0]["hidden"], "kept-email");

    let before = f.snapshot().unwrap();
    for invalid in [
        patch(json!([{"op":"replace","path":"name","value":{"hidden":"new"}}])),
        patch(json!([{"op":"replace","path":"emails[value eq \"ada@example.test\"].primary","value":"true"}])),
        patch(json!([{"op":"add","path":"emails","value":{"value":"other@example.test","hidden":"new"}}])),
    ] {
        let (status, _) = request(&app, Method::PATCH, &path, &token, Some(version(&changed)), Some(&invalid)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(version(&f.core.scim_get(&token, "Users", created["id"].as_str().unwrap()).unwrap()), version(&changed));
}

#[tokio::test]
async fn group_member_patch_accepts_echoed_display_but_rebuilds_it() {
    let f = Fixture::new();
    let token = agent(&f, "compat-group-agent");
    let app = riauth::api::router(f.core.clone());
    let first = f.core.scim_write(&token, "Users", None, json!({"schemas":[scim::USER],"userName":"compat-first","displayName":"Real First"}), false).unwrap();
    let second = f.core.scim_write(&token, "Users", None, json!({"schemas":[scim::USER],"userName":"compat-second","displayName":"Real Second"}), false).unwrap();
    let first_id = text(&first, "id");
    let second_id = text(&second, "id");
    let group = f.core.scim_write(&token, "Groups", None, json!({"schemas":[scim::GROUP],"displayName":"compat-group","members":[{"value":first_id}]}), false).unwrap();
    let path = format!("/scim/v2/Groups/{}", text(&group, "id"));
    let (status, fetched) = request(&app, Method::GET, &path, &token, None, None).await;
    assert_eq!(status, StatusCode::OK);
    let mut echoed = fetched["members"].clone();
    echoed[0]["display"] = json!("Forged First");
    let (status, same) = request(&app, Method::PATCH, &path, &token, Some(version(&fetched)), Some(&patch(json!([
        {"op":"replace","path":"members","value":echoed}
    ])))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(same["members"], fetched["members"]);
    assert_eq!(version(&same), version(&fetched));

    let (status, added) = request(&app, Method::PATCH, &path, &token, Some(version(&same)), Some(&patch(json!([
        {"op":"add","path":"members","value":[{"value":second_id,"display":"Forged Second"}]}
    ])))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(added["members"].as_array().unwrap().len(), 2);
    assert_eq!(added["members"].as_array().unwrap().iter().find(|member| member["value"] == second_id).unwrap()["display"], "compat-second");
    assert_ne!(version(&added), version(&same));

    let before = f.snapshot().unwrap();
    let (status, error) = request(&app, Method::PATCH, &path, &token, Some(version(&added)), Some(&patch(json!([
        {"op":"replace","path":format!("members[value eq \"{first_id}\"].display"),"value":"Forged"}
    ])))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error["scimType"], "mutability");
    f.assert_http_mutation_snapshot(&before);
}
