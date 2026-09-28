#![cfg(feature = "platform")]

mod common;

use axum::{Router, body::Body, http::{Method, Request, StatusCode}};
use common::{Fixture, text};
use riauth::{agent::{NewAgent, Permission}, scim};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn filtered(
    app: &Router,
    token: &str,
    kind: &str,
    filter: &str,
    start: usize,
    count: usize,
    post: bool,
) -> (StatusCode, Value) {
    let (method, path, body) = if post {
        (
            Method::POST,
            format!("/scim/v2/{kind}/.search"),
            Body::from(json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:SearchRequest"],"filter":filter,"startIndex":start,"count":count}).to_string()),
        )
    } else {
        let mut query = url::form_urlencoded::Serializer::new(String::new());
        query.append_pair("filter", filter);
        query.append_pair("startIndex", &start.to_string());
        query.append_pair("count", &count.to_string());
        (Method::GET, format!("/scim/v2/{kind}?{}", query.finish()), Body::empty())
    };
    let response = app.clone().oneshot(
        Request::builder()
            .method(method)
            .uri(path)
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/scim+json")
            .body(body)
            .unwrap(),
    ).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32768).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn scim_user_email_and_active_filters_share_owned_list_and_search_semantics() {
    let f = Fixture::new();
    let permissions: Vec<Permission> = ["user.read", "user.write", "group.read", "group.write", "group.members"]
        .map(|action| Permission { action: action.into(), resource: "*".into() })
        .into();
    let owner = f.core.create_agent(&f.admin, NewAgent {
        id: "scim-filter-owner".into(), ttl: 600, parent: None,
        permissions: permissions.clone(),
    }).unwrap();
    let other = f.core.create_agent(&f.admin, NewAgent {
        id: "scim-filter-other".into(), ttl: 600, parent: None,
        permissions,
    }).unwrap();
    let owner = text(&owner["credential"], "token");
    let other = text(&other["credential"], "token");
    let users = [
        ("filter-one", "A \"Quoted\" and Name", true, json!([{"value":"first@example.test","type":"work","primary":true},{"value":"alias@example.test","type":"home","primary":false}])),
        ("filter-two", "Second", false, json!([{"value":"ALIAS@example.test","type":"work","primary":true}])),
        ("filter-three", "Third", true, json!([])),
    ];
    for (username, display, active, emails) in users {
        f.core.scim_write(&owner, "Users", None, json!({"schemas":[scim::USER],"userName":username,"displayName":display,"active":active,"emails":emails}), false).unwrap();
    }
    f.core.scim_write(&other, "Users", None, json!({"schemas":[scim::USER],"userName":"filter-outside","active":false,"emails":[{"value":"alias@example.test","primary":true}]}), false).unwrap();
    f.core.scim_write(&owner, "Groups", None, json!({"schemas":[scim::GROUP],"displayName":"Filter-Group"}), false).unwrap();
    let app = riauth::api::router(f.core.clone());

    let email = r#"EMAILS.VALUE EQ "ALIAS\u0040EXAMPLE.TEST""#;
    let (status, full) = filtered(&app, &owner, "Users", email, 1, 100, false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(full["totalResults"], 2);
    assert_eq!(full["itemsPerPage"], 2);
    let (status, search) = filtered(&app, &owner, "Users", email, 1, 100, true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(search, full);
    let (status, first) = filtered(&app, &owner, "Users", email, 1, 1, false).await;
    assert_eq!(status, StatusCode::OK);
    let (status, second) = filtered(&app, &owner, "Users", email, 2, 1, true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["totalResults"], 2);
    assert_eq!(second["totalResults"], 2);
    assert_eq!(first["itemsPerPage"], 1);
    assert_eq!(second["itemsPerPage"], 1);
    assert_ne!(first["Resources"][0]["id"], second["Resources"][0]["id"]);
    assert_eq!(filtered(&app, &owner, "Users", email, 1, 1, true).await.1, first);
    assert_eq!(filtered(&app, &owner, "Users", email, 2, 1, false).await.1, second);

    for post in [false, true] {
        let (status, inactive) = filtered(&app, &owner, "Users", "active eq false", 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(inactive["totalResults"], 1);
        assert_eq!(inactive["Resources"][0]["userName"], "filter-two");
        let (status, active) = filtered(&app, &owner, "Users", "active eq true", 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(active["totalResults"], 2);
        let (status, quoted) = filtered(&app, &owner, "Users", r#"displayName eq "A \"Quoted\" and Name""#, 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(quoted["Resources"][0]["userName"], "filter-one");
        let conjunction = r#"displayName eq "A \"Quoted\" and Name" AND emails.value EQ "ALIAS@example.test" and active eq true and emails PR"#;
        let (status, combined) = filtered(&app, &owner, "Users", conjunction, 1, 1, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(combined["totalResults"], 1);
        assert_eq!(combined["itemsPerPage"], 1);
        assert_eq!(combined["Resources"][0]["userName"], "filter-one");
        let (status, active_present) = filtered(&app, &owner, "Users", "active pr", 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(active_present["totalResults"], 3); // false is assigned and therefore present.
        let (status, present) = filtered(&app, &owner, "Users", "emails pr and emails.value pr", 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(present["totalResults"], 2); // The empty email array is absent.
        let (status, absent) = filtered(&app, &owner, "Users", "externalId pr", 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(absent["totalResults"], 0);
        let (status, groups) = filtered(&app, &owner, "Groups", r#"displayName eq "Filter-Group" and id pr"#, 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(groups["totalResults"], 1);
        let same_email = r#"EMAILS[VALUE EQ "ALIAS\u0040EXAMPLE.TEST" and TYPE eq "WORK"]"#;
        let (status, selected) = filtered(&app, &owner, "Users", same_email, 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(selected["totalResults"], 1);
        assert_eq!(selected["Resources"][0]["userName"], "filter-two");
        let grouped = r#"(active eq false or emails[value eq "first@example.test"]) and emails pr"#;
        let (status, grouped_result) = filtered(&app, &owner, "Users", grouped, 1, 1, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(grouped_result["totalResults"], 2);
        assert_eq!(grouped_result["itemsPerPage"], 1);
        let precedence = r#"active eq true or active eq false and emails[type eq "work"]"#;
        let (status, precedence_result) = filtered(&app, &owner, "Users", precedence, 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(precedence_result["totalResults"], 3); // and binds before or.
        let (status, primary_false) = filtered(&app, &owner, "Users", "emails[primary eq false and type pr]", 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(primary_false["totalResults"], 1);
        assert_eq!(primary_false["Resources"][0]["userName"], "filter-one");
        let inner_group = r#"emails[(value eq "alias@example.test" and type eq "home") or (primary eq true and type eq "work")]"#;
        let (status, inner_result) = filtered(&app, &owner, "Users", inner_group, 1, 100, post).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(inner_result["totalResults"], 2);
        for bad in [
            "active eq \"false\"",
            "active eq FALSE",
            r#"emails.value eq "bad\q""#,
            "active eq true\n",
            "active pr true",
            "active eq true and",
            "((((active eq true))))",
            "active pr or active pr or active pr or active pr or active pr or active pr or active pr or active pr or active pr",
            r#"emails[type co "work"]"#,
            r#"emails[value eq "alias@example.test"].value"#,
            r#"emails[value eq "alias@example.test""#,
            r#"emails[not (value eq "alias@example.test")]"#,
            r#"emails[emails.value eq "alias@example.test"]"#,
            "emails[primary eq \"true\"]",
            r#"displayName eq "A \"Quoted\" and Name" and emails.value eq "bad\q""#,
        ] {
            let (status, error) = filtered(&app, &owner, "Users", bad, 1, 100, post).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{bad}");
            assert_eq!(error["scimType"], "invalidFilter", "{bad}");
        }
        let (status, error) = filtered(&app, &owner, "Groups", email, 1, 100, post).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error["scimType"], "invalidFilter");
        let (status, error) = filtered(&app, &owner, "Groups", r#"userName eq "filter-one""#, 1, 100, post).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error["scimType"], "invalidFilter");
    }
    let (status, outside) = filtered(&app, &other, "Users", email, 1, 100, false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(outside["totalResults"], 1);
    assert_eq!(outside["Resources"][0]["userName"], "filter-outside");
}
