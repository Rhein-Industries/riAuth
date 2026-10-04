//! Browser transport for an existing embedded source stage. Core owns every factor,
//! request and callback write; the original return cookie owns final native delivery.
use crate::{
    api::{App, binding_cookie},
    core::Core,
    crypto::{constant_eq, now},
    error::{Error, Result},
    portal::http::{browser_write_guard, portal_html},
    response::escape,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoQuery {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Resume {
    #[serde(default)]
    otp: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cancel {}

pub(crate) fn routes() -> Router<App> {
    Router::new()
        .route(
            "/oauth/resume/{authorization}/source-stage/{stage}",
            get(page),
        )
        .route(
            "/oauth/resume/{authorization}/source-stage/{stage}/resume",
            post(resume),
        )
        .route(
            "/oauth/resume/{authorization}/source-stage/{stage}/cancel",
            post(cancel),
        )
        .layer(middleware::map_response(private_headers))
        .route("/portal/assets/source-stage.js", get(script))
}

async fn private_headers(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    response
}

// Only explicit, valid media ranges opt into this browser representation. JSON wins
// when both are offered. Wildcards and malformed/duplicate Accept never opt in.
fn representations(headers: &HeaderMap) -> Option<(bool, bool)> {
    if headers.get_all("accept").iter().count() != 1 {
        return None;
    }
    let value = headers.get("accept")?.to_str().ok()?;
    let (mut html, mut json) = (false, false);
    for range in value.split(',') {
        let mut parts = range.split(';');
        let media = parts.next()?.trim();
        let (kind, subtype) = media.split_once('/')?;
        if !token(kind) || !token(subtype) {
            return None;
        }
        let (mut quality, mut has_quality) = (1000, false);
        for parameter in parts {
            let (name, value) = parameter.trim().split_once('=')?;
            let (name, value) = (name.trim(), value.trim());
            if !token(name) || !token(value) {
                return None;
            }
            if name.eq_ignore_ascii_case("q") {
                if has_quality {
                    return None;
                }
                has_quality = true;
                quality = parse_quality(value)?;
            }
        }
        if quality > 0 {
            html |= media.eq_ignore_ascii_case("text/html");
            json |= media.eq_ignore_ascii_case("application/json");
        }
    }
    Some((html, json))
}

fn token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
}

fn parse_quality(value: &str) -> Option<u16> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if !matches!(whole, "0" | "1")
        || fraction.len() > 3
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || (whole == "1" && fraction.bytes().any(|b| b != b'0'))
    {
        return None;
    }
    if whole == "1" {
        return Some(1000);
    }
    let mut quality = 0;
    let mut place = 100;
    for digit in fraction.bytes() {
        quality += u16::from(digit - b'0') * place;
        place /= 10;
    }
    Some(quality)
}

pub(crate) fn wants_html(headers: &HeaderMap) -> bool {
    representations(headers) == Some((true, false))
}

fn enabled(app: &App) -> Result<()> {
    if !app.core.config.browser_ui {
        return Err(Error::missing("Browser sign-in is unavailable"));
    }
    Ok(())
}

struct Paths {
    page: String,
    continuation: String,
    resume: String,
    cancel: String,
}

impl Paths {
    fn new(core: &Core, authorization: &str, stage: &str) -> Self {
        let authorization = url::form_urlencoded::byte_serialize(authorization.as_bytes())
            .collect::<String>()
            .replace('+', "%20");
        let stage = url::form_urlencoded::byte_serialize(stage.as_bytes())
            .collect::<String>()
            .replace('+', "%20");
        let relative = format!("oauth/resume/{authorization}/source-stage/{stage}");
        Self {
            page: format!("{}{relative}", core.cookie_path()),
            continuation: format!("{}oauth/resume/{authorization}", core.cookie_path()),
            // RiAuth.post supplies the issuer base exactly once.
            resume: format!("{relative}/resume"),
            cancel: format!("{relative}/cancel"),
        }
    }
}

/// The old GET's HTML handoff is read-only; terminal/API stages keep their result.
pub(crate) fn page_path(core: &Core, stage: &str, authorization: &str) -> Result<Option<String>> {
    core.store.read(|tx| {
        let record = crate::assembly::load_source_stage(tx, stage, authorization)?;
        match record.request.request_binding.as_deref() {
            None => Ok(None),
            Some(id) if constant_eq(id, authorization) => Ok(Some(
                Paths::new(core, &record.authorization_id, &record.id).page,
            )),
            Some(_) => Err(Error::forbidden()),
        }
    })
}

struct Context {
    paths: Paths,
    ready: bool,
}

fn preflight(
    core: &Core,
    headers: &HeaderMap,
    authorization: &str,
    stage: &str,
) -> Result<Context> {
    let state = core.authorize_state(
        authorization,
        binding_cookie(core, headers, "return", authorization),
        None,
    )?;
    let record = core
        .store
        .read(|tx| crate::assembly::load_source_stage(tx, stage, authorization))?;
    if !record
        .request
        .request_binding
        .as_deref()
        .is_some_and(|id| constant_eq(id, authorization))
    {
        return Err(Error::forbidden());
    }
    if record.expires_at <= now() || record.expires_at > now() + 600 {
        return Err(Error::bad("Source stage expired"));
    }
    let paths = Paths::new(core, &record.authorization_id, &record.id);
    if state["status"] == "complete"
        && state["continue"].as_str() == Some(paths.continuation.as_str())
    {
        return Ok(Context { paths, ready: true });
    }
    if state["status"] != "unavailable"
        || state["error"] != "source_stage"
        || !state["continue"].is_null()
        || record.used
        || record.cancelled
    {
        return Err(Error::conflict(
            "Source stage already completed or unavailable",
        ));
    }
    Ok(Context {
        paths,
        ready: false,
    })
}

async fn page(
    State(app): State<App>,
    Path((authorization, stage)): Path<(String, String)>,
    Query(_query): Query<NoQuery>,
    headers: HeaderMap,
) -> Result<Response> {
    enabled(&app)?;
    if !wants_html(&headers) {
        return Err(Error::new(
            StatusCode::NOT_ACCEPTABLE,
            "not_acceptable",
            "Request this page as text/html",
        ));
    }
    let page = app.clone();
    app.run(move |core| {
        let context = preflight(core, &headers, &authorization, &stage)?;
        if context.ready {
            return local_handoff(&page, &context.paths.continuation);
        }
        let template = page
            .core
            .runtime
            .frontend
            .page("source-stage.html")
            .replace("__AUTHORIZATION__", &escape(&authorization))
            .replace("__STAGE__", &escape(&stage))
            .replace("__RESUME__", &escape(&context.paths.resume))
            .replace("__CANCEL__", &escape(&context.paths.cancel))
            .replace("__CONTINUE__", &escape(&context.paths.continuation));
        Ok(portal_html(&template, &page, true))
    })
    .await
}

fn local_handoff(app: &App, path: &str) -> Result<Response> {
    if !path.starts_with('/') || path.starts_with("//") {
        return Err(Error::internal("Handoff target is not a local path"));
    }
    let origin = url::Url::parse(&app.core.config.issuer)
        .map_err(Error::internal)?
        .origin()
        .ascii_serialization();
    Ok((
        StatusCode::SEE_OTHER,
        [("location", format!("{origin}{path}"))],
    )
        .into_response())
}

fn write_preflight(app: &App, headers: &HeaderMap) -> Result<()> {
    enabled(app)?;
    browser_write_guard(app, headers)?;
    if !representations(headers).is_some_and(|(_, json)| json) {
        return Err(Error::new(
            StatusCode::NOT_ACCEPTABLE,
            "not_acceptable",
            "Request a JSON response",
        ));
    }
    Ok(())
}

fn outcome(value: Value, context: &Context) -> Result<Response> {
    match (value["status"].as_str(), value["code_issued"].as_bool(), value["redirect_uri"].as_str()) {
        (Some(status @ ("pending" | "local_factor_required")), Some(false), None) => {
            Ok(Json(json!({"status": status, "code_issued": false})).into_response())
        }
        (Some(status @ "complete"), Some(true), Some(uri))
        | (Some(status @ ("rejected" | "cancelled")), Some(false), Some(uri)) if !uri.is_empty() => {
            Ok(Json(json!({"status": status, "code_issued": value["code_issued"], "continue": context.paths.continuation})).into_response())
        }
        _ => Err(Error::internal("Invalid source stage outcome")),
    }
}

async fn resume(
    State(app): State<App>,
    Path((authorization, stage)): Path<(String, String)>,
    Query(_query): Query<NoQuery>,
    headers: HeaderMap,
    input: std::result::Result<Json<Resume>, JsonRejection>,
) -> Result<Response> {
    write_preflight(&app, &headers)?;
    let Json(input) = input.map_err(|_| Error::bad("Invalid source stage JSON"))?;
    app.run(move |core| {
        let context = preflight(core, &headers, &authorization, &stage)?;
        if context.ready {
            return Err(Error::bad("Source stage already used"));
        }
        // Call the existing wrapper once: its inner wrong-factor error already committed.
        outcome(
            core.source_stage_resume(&stage, &authorization, input.otp)?,
            &context,
        )
    })
    .await
}

async fn cancel(
    State(app): State<App>,
    Path((authorization, stage)): Path<(String, String)>,
    Query(_query): Query<NoQuery>,
    headers: HeaderMap,
    input: std::result::Result<Json<Cancel>, JsonRejection>,
) -> Result<Response> {
    write_preflight(&app, &headers)?;
    let Json(_input) = input.map_err(|_| Error::bad("Invalid source stage JSON"))?;
    app.run(move |core| {
        let context = preflight(core, &headers, &authorization, &stage)?;
        if context.ready {
            return Err(Error::conflict("Source stage already completed"));
        }
        outcome(core.source_stage_cancel(&stage, &authorization)?, &context)
    })
    .await
}

async fn script(State(app): State<App>) -> Result<Response> {
    enabled(&app)?;
    Ok(app.core.runtime.frontend.builtin("source-stage.js"))
}
