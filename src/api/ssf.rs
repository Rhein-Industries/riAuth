//! Shared Signals Framework stream configuration and event delivery adapters.
use super::*;

pub(super) async fn ssf_configuration(State(app): State<App>) -> Json<Value> {
    Json(app.core.ssf_metadata())
}

pub(super) fn ssf_caller(headers: &HeaderMap) -> Result<crate::ssf::SsfAuth> {
    let header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(Error::unauthenticated)?;
    let (scheme, _) = header.split_once(' ').ok_or_else(Error::unauthorized)?;
    if scheme.eq_ignore_ascii_case("basic") {
        let request =
            client_credentials_from_headers(headers, crate::oidc::TokenRequest::default())?;
        Ok(crate::ssf::SsfAuth::ClientBasic {
            client_id: request.client_id.ok_or_else(Error::unauthorized)?,
            client_secret: request.client_secret.ok_or_else(Error::unauthorized)?,
        })
    } else {
        Ok(crate::ssf::SsfAuth::Bearer(bearer(headers)?))
    }
}
#[derive(Deserialize)]
pub(super) struct SsfStreamQuery {
    stream_id: Option<String>,
}
pub(super) async fn ssf_config_streams(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<SsfStreamQuery>,
) -> Result<Response> {
    let auth = ssf_caller(&headers)?;
    let value = app
        .run(move |core| core.ssf_config_read(&auth, query.stream_id.as_deref()))
        .await?;
    let mut response = Json(value).into_response();
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    Ok(response)
}
pub(super) async fn ssf_config_stream_create(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::ssf::ConfigurationInput>,
) -> Result<Response> {
    let auth = ssf_caller(&headers)?;
    let value = app
        .run(move |core| core.ssf_config_create(&auth, input))
        .await?;
    let mut response = (StatusCode::CREATED, Json(value)).into_response();
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    Ok(response)
}
pub(super) async fn ssf_config_stream_patch(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Value>,
) -> Result<Response> {
    ssf_config_stream_update(app, headers, input, false).await
}
pub(super) async fn ssf_config_stream_put(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Value>,
) -> Result<Response> {
    ssf_config_stream_update(app, headers, input, true).await
}
pub(super) async fn ssf_config_stream_update(
    app: App,
    headers: HeaderMap,
    input: Value,
    replace: bool,
) -> Result<Response> {
    let auth = ssf_caller(&headers)?;
    let value = app
        .run(move |core| core.ssf_config_update(&auth, input, replace))
        .await?;
    let mut response = Json(value).into_response();
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    Ok(response)
}
pub(super) async fn ssf_config_stream_delete(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<SsfStreamQuery>,
) -> Result<Response> {
    let id = query
        .stream_id
        .ok_or_else(|| Error::bad("stream_id is required"))?;
    let auth = ssf_caller(&headers)?;
    app.run(move |core| core.ssf_config_delete(&auth, &id))
        .await?;
    let mut response = StatusCode::NO_CONTENT.into_response();
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    Ok(response)
}
pub(super) async fn ssf_streams(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let auth = ssf_caller(&headers)?;
    app.run(move |core| core.ssf_list(&auth).map(Json)).await
}
pub(super) async fn ssf_stream_create(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::ssf::StreamInput>,
) -> Result<Response> {
    let auth = ssf_caller(&headers)?;
    let value = app.run(move |core| core.ssf_create(&auth, input)).await?;
    Ok((StatusCode::CREATED, Json(value)).into_response())
}
pub(super) async fn ssf_stream_delete(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let auth = ssf_caller(&headers)?;
    app.run(move |core| core.ssf_delete(&auth, &id).map(Json))
        .await
}
pub(super) async fn ssf_stream_subjects(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<crate::ssf::SubjectBindings>,
) -> Result<Json<Value>> {
    let auth = ssf_caller(&headers)?;
    app.run(move |core| core.ssf_bind_subjects(&auth, &id, input).map(Json))
        .await
}
pub(super) async fn ssf_events(State(app): State<App>, request: Request) -> Result<Response> {
    let content_type = request
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim();
    if !content_type.eq_ignore_ascii_case("application/secevent+jwt") {
        return Ok(ssf_delivery_error(
            "invalid_request",
            "Content-Type must be application/secevent+jwt",
        ));
    }
    let body = match axum::body::to_bytes(request.into_body(), 16_385).await {
        Ok(body) if body.len() <= 16_384 => body,
        _ => {
            return Ok(ssf_delivery_error(
                "invalid_request",
                "Security event token validation failed",
            ));
        }
    };
    let body = match String::from_utf8(body.to_vec()) {
        Ok(body) => body,
        Err(_) => {
            return Ok(ssf_delivery_error(
                "invalid_request",
                "Security event token validation failed",
            ));
        }
    };
    match app.run(move |core| core.accept_set(&body)).await {
        Ok(_) => Ok(StatusCode::ACCEPTED.into_response()),
        Err(error) if error.status == StatusCode::BAD_REQUEST => {
            Ok(ssf_delivery_error(error.code, &error.message))
        }
        Err(error) => Err(error),
    }
}

pub(super) fn ssf_delivery_error(code: &str, description: &str) -> Response {
    let mut response = (
        StatusCode::BAD_REQUEST,
        Json(json!({"err": code, "description": description})),
    )
        .into_response();
    response
        .headers_mut()
        .insert("content-language", HeaderValue::from_static("en"));
    response
}
