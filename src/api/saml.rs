//! SAML provider, source and logout HTTP adapters.
use super::*;

pub(super) fn saml_response(reply: crate::saml::Reply) -> Result<Response> {
    match reply {
        crate::saml::Reply::LogoutPage(value) => crate::response::logout(value),
        crate::saml::Reply::Waiting(reply) => browser_response(reply),
        crate::saml::Reply::Redirect(location) => crate::response::callback(location, false),
        crate::saml::Reply::Post {
            target,
            fields,
            cookies,
        } => {
            let mut response = crate::response::form_post(&target, fields)?;
            for cookie in cookies {
                response.headers_mut().append(
                    "set-cookie",
                    HeaderValue::from_str(&cookie).map_err(Error::internal)?,
                );
            }
            Ok(response)
        }
    }
}
pub(super) async fn saml_metadata(
    State(app): State<App>,
    Path(id): Path<String>,
) -> Result<Response> {
    app.run(move |core| {
        Ok((
            [
                ("content-type", "application/samlmetadata+xml"),
                ("cache-control", "no-store"),
            ],
            core.saml_metadata(&id)?,
        )
            .into_response())
    })
    .await
}
pub(super) async fn saml_metadata_json(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.store.read(|tx| {
            core.management(tx, &token, "client.read", &format!("client/{id}"))
                .map(|_| ())
        })?;
        Ok(Json(json!({"metadata_xml":core.saml_metadata(&id)?})))
    })
    .await
}
/// With HTML Accept, a request that needs the browser continues on its resume page.
pub(super) fn saml_handoff(app: &App, html: bool, reply: crate::saml::Reply) -> Result<Response> {
    match reply {
        crate::saml::Reply::Waiting(reply) if html => {
            let path = reply.body["resume_uri"]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| Error::internal("SAML handoff has no resume path"))?;
            see_other(app, &path, reply.cookies)
        }
        reply => saml_response(reply).map(vary_accept),
    }
}
pub(super) async fn saml_redirect(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> Result<Response> {
    let html = app.core.config.browser_ui && accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let handoff = app.clone();
    app.run(move |core| {
        let reply = core.saml_start(&id, raw.as_deref().unwrap_or(""), false, sso.as_deref())?;
        saml_handoff(&handoff, html, reply)
    })
    .await
}
pub(super) async fn saml_post(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: String,
) -> Result<Response> {
    if headers.get_all("content-type").iter().count() != 1
        || headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_none_or(|v| v.split(';').next() != Some("application/x-www-form-urlencoded"))
    {
        return Err(Error::bad("SAML POST requires form encoding"));
    }
    let html = app.core.config.browser_ui && accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let handoff = app.clone();
    app.run(move |core| {
        saml_handoff(
            &handoff,
            html,
            core.saml_start(&id, &body, true, sso.as_deref())?,
        )
    })
    .await
}
pub(super) async fn saml_initiate(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response> {
    let html = app.core.config.browser_ui && accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let handoff = app.clone();
    app.run(move |core| saml_handoff(&handoff, html, core.saml_initiate(&id, sso.as_deref())?))
        .await
}
pub(super) async fn saml_resume(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let html = app.core.config.browser_ui && accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let page = app.clone();
    let result = app
        .run(move |core| {
            let binding = binding_cookie(core, &headers, "saml", &id);
            match core.saml_resume_with(&id, binding, sso.as_deref())? {
                crate::saml::Reply::Waiting(reply) if html => {
                    let code = reply.body["user_code"].as_str().unwrap_or_default();
                    Ok(interaction::interaction_page(
                        &page,
                        code,
                        "request approve",
                        sso.is_some(),
                    ))
                }
                reply => saml_response(reply),
            }
        })
        .await;
    vary_accept(match result {
        Err(error) if html => resume_error(&app, error, "sign-in"),
        result => result.into_response(),
    })
}

pub(super) async fn saml_source_metadata(
    State(app): State<App>,
    Path(id): Path<String>,
) -> Result<Response> {
    let xml = app.run(move |core| core.saml_source_metadata(&id)).await?;
    Ok((
        [(
            "content-type",
            "application/samlmetadata+xml; charset=utf-8",
        )],
        xml,
    )
        .into_response())
}
pub(super) async fn saml_source_acs(
    State(app): State<App>,
    Path(id): Path<String>,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Result<Response> {
    let source_id = id.clone();
    let mut value = app
        .run(move |core| core.saml_source_callback(&id, pairs))
        .await?;
    // The raw return token is only a cookie. CLI and stage bodies never have one.
    let token = value
        .get("browser_return")
        .and_then(Value::as_str)
        .map(str::to_owned);
    if let Some(body) = value.as_object_mut() {
        body.remove("browser_return");
    }
    if let Some(token) = token {
        let cookie = app.core.binding_cookie(
            crate::portal::sources::RETURN_KIND,
            crate::portal::sources::ID,
            &token,
            &app.core.cookie_path(),
            600,
        );
        let path = format!("{}saml/sources/{source_id}/return", app.core.cookie_path());
        return see_other(&app, &path, vec![cookie]);
    }
    source_stage_redirect(&app.core, value)
}

pub(super) async fn saml_source_browser_return(
    State(app): State<App>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response> {
    let started = binding_cookie(
        &app.core,
        &headers,
        crate::portal::sources::KIND,
        crate::portal::sources::ID,
    )
    .map(str::to_owned);
    let returned = binding_cookie(
        &app.core,
        &headers,
        crate::portal::sources::RETURN_KIND,
        crate::portal::sources::ID,
    )
    .map(str::to_owned);
    let continue_path = format!("{}account/sources/continue", app.core.cookie_path());
    let clear = app.core.binding_cookie(
        crate::portal::sources::RETURN_KIND,
        crate::portal::sources::ID,
        "",
        &app.core.cookie_path(),
        0,
    );
    // Success and a burned return both land on the review page. A missing return
    // cookie still redirects, and does not clear a cookie this request did not spend.
    match app
        .run(move |core| {
            core.saml_source_browser_return(&id, started.as_deref(), returned.as_deref())
        })
        .await
    {
        Ok(()) => see_other(&app, &continue_path, vec![clear]),
        Err(error) if error.code == "source_browser_mismatch" => {
            see_other(&app, &continue_path, vec![clear])
        }
        Err(error) if error.status == StatusCode::BAD_REQUEST => {
            see_other(&app, &continue_path, vec![])
        }
        Err(error) => Err(error),
    }
}

pub(super) async fn saml_source_metadata_json(
    State(app): State<App>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.store.read(|tx| {
            core.management(tx, &token, "source.read", &format!("source/{id}"))
                .map(|_| ())
        })?;
        Ok(Json(
            json!({"source":id,"metadata_xml":core.saml_source_metadata(&id)?}),
        ))
    })
    .await
}

pub(super) async fn saml_logout_next(
    State(app): State<App>,
    Path(ticket): Path<String>,
) -> Result<Response> {
    app.run(move |core| saml_response(core.saml_logout_next(&ticket)?))
        .await
}
pub(super) async fn saml_logout_status(
    State(app): State<App>,
    Path(ticket): Path<String>,
) -> Result<Json<Value>> {
    app.run(move |core| core.saml_logout_status(&ticket).map(Json))
        .await
}
pub(super) async fn saml_source_logout_redirect(
    State(app): State<App>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> Result<Response> {
    app.run(move |core| {
        saml_response(core.saml_source_logout(&id, raw.as_deref().unwrap_or(""), false)?)
    })
    .await
}
pub(super) async fn saml_source_logout_post(
    State(app): State<App>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: String,
) -> Result<Response> {
    if headers.get_all("content-type").iter().count() != 1
        || headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_none_or(|v| v.split(';').next() != Some("application/x-www-form-urlencoded"))
    {
        return Err(Error::bad("SAML POST requires form encoding"));
    }
    app.run(move |core| saml_response(core.saml_source_logout(&id, &body, true)?))
        .await
}
