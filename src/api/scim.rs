//! SCIM resource request and response adapters.
use super::*;

pub(super) async fn scim_list(
    State(app): State<App>,
    headers: HeaderMap,
    Path(kind): Path<String>,
    Query(query): Query<crate::scim::Query>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_list(&token, &kind, query))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::OK)
}
pub(super) async fn scim_search(
    State(app): State<App>,
    headers: HeaderMap,
    Path(kind): Path<String>,
    Json(mut value): Json<Value>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        if value["schemas"] != json!(["urn:ietf:params:scim:api:messages:2.0:SearchRequest"]) {
            return Err(Error::bad("Invalid search request schema"));
        }
        value.as_object_mut().unwrap().remove("schemas");
        for field in ["attributes", "excludedAttributes"] {
            if let Some(selection) = value.get_mut(field) {
                let items = selection
                    .as_array()
                    .filter(|items| items.len() <= 32)
                    .ok_or_else(|| {
                        Error::bad("SCIM search projection must be an attribute array")
                    })?;
                let mut names = Vec::with_capacity(items.len());
                for item in items {
                    let name = item
                        .as_str()
                        .filter(|name| !name.contains(','))
                        .ok_or_else(|| Error::bad("Invalid SCIM search projection attribute"))?;
                    names.push(name);
                }
                *selection = json!(names.join(","));
            }
        }
        let query = serde_json::from_value(value)
            .map_err(|_| Error::bad("Unsupported search parameters"))?;
        app.run(move |core| core.scim_list(&token, &kind, query))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::OK)
}
pub(super) async fn scim_get(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
    Query(query): Query<crate::scim::ProjectionQuery>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_get_projected(&token, &kind, &id, query))
            .await
    }
    .await;
    match result {
        Ok((body, version, location)) => {
            let mut response = crate::scim::response(Ok(body), StatusCode::OK);
            if let Ok(version) = HeaderValue::from_str(&version) {
                response.headers_mut().insert("etag", version);
            }
            if let Ok(location) = HeaderValue::from_str(&location) {
                response.headers_mut().insert("location", location);
            }
            response
        }
        Err(error) => crate::scim::response(Err(error), StatusCode::OK),
    }
}
pub(super) async fn scim_create(
    State(app): State<App>,
    headers: HeaderMap,
    Path(kind): Path<String>,
    Json(value): Json<Value>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_write(&token, &kind, None, value, false))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::CREATED)
}
pub(super) async fn scim_replace(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
    Json(value): Json<Value>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_write(&token, &kind, Some(&id), value, false))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::OK)
}
pub(super) async fn scim_patch(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
    Json(value): Json<Value>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_write(&token, &kind, Some(&id), value, true))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::OK)
}
pub(super) async fn scim_delete(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_delete(&token, &kind, &id))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::NO_CONTENT)
}
