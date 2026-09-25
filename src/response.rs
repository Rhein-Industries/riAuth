use crate::{
    core::Core,
    crypto,
    error::{Error, Result},
    model::Client,
    store::Tx,
};
use axum::{
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;

pub const MODES: &[&str] = &[
    "query",
    "fragment",
    "form_post",
    "jwt",
    "query.jwt",
    "fragment.jwt",
    "form_post.jwt",
];
const FIELDS: &[&str] = &[
    "code",
    "state",
    "iss",
    "error",
    "error_description",
    "response",
    "session_state",
];

impl Core {
    pub(crate) fn secure_authorization_response(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        mode: Option<&str>,
        callback: String,
    ) -> Result<String> {
        let mode = mode.unwrap_or("query");
        if mode == "query" || mode == "form_post" {
            return Ok(callback);
        }
        let mut url = url::Url::parse(&callback).map_err(Error::internal)?;
        let (fields, original): (Vec<_>, Vec<_>) = url
            .query_pairs()
            .into_owned()
            .partition(|(k, _)| FIELDS.contains(&k.as_str()));
        url.set_query(None);
        if !original.is_empty() {
            url.query_pairs_mut().extend_pairs(original);
        }
        let response = if mode == "jwt" || mode.ends_with(".jwt") {
            let mut claims = json!({"iss": crate::issuer::for_client(&self.config.issuer,client), "aud": client.id, "iat":crypto::now(), "exp":crypto::now()+60});
            for (name, value) in fields {
                if name != "iss" {
                    claims[&name] = json!(value);
                }
            }
            let signed = self.sign_jwt(
                &crate::keyring::for_client(tx, client)?.active,
                &claims,
                "oauth-authz-resp+jwt",
            )?;
            let response = if let Some(key) = &client.settings.authorization_encryption {
                key.encrypt(&signed)?
            } else {
                signed
            };
            vec![("response".into(), response)]
        } else {
            fields
        };
        if mode.starts_with("fragment") {
            url.set_fragment(Some(
                &url::form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(response)
                    .finish(),
            ));
        } else {
            url.query_pairs_mut().extend_pairs(response);
        }
        Ok(url.into())
    }
}

pub fn is_form(mode: Option<&str>) -> bool {
    matches!(mode, Some("form_post" | "form_post.jwt"))
}
pub fn callback(location: String, form_post: bool) -> Result<Response> {
    if !form_post {
        return Ok((StatusCode::FOUND, [("location", location)]).into_response());
    }
    let mut target = url::Url::parse(&location).map_err(Error::internal)?;
    let (fields, original): (Vec<_>, Vec<_>) = target
        .query_pairs()
        .into_owned()
        .partition(|(k, _)| FIELDS.contains(&k.as_str()));
    target.set_query(None);
    if !original.is_empty() {
        target.query_pairs_mut().extend_pairs(original);
    }
    self::form_post(target.as_str(), fields)
}

pub fn form_post(target: &str, fields: Vec<(String, String)>) -> Result<Response> {
    let target = url::Url::parse(target).map_err(Error::internal)?;
    let nonce = crypto::random_token("");
    let inputs = fields
        .into_iter()
        .map(|(k, v)| {
            format!(
                "<input type=\"hidden\" name=\"{}\" value=\"{}\">",
                escape(&k),
                escape(&v)
            )
        })
        .collect::<String>();
    let html = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>Authorization response</title><form method=\"post\" action=\"{}\">{inputs}</form><script nonce=\"{nonce}\">document.forms[0].submit()</script>",
        escape(target.as_str())
    );
    let mut response = axum::response::Html(html).into_response();
    let policy = format!(
        "default-src 'none'; script-src 'nonce-{nonce}'; form-action {}; base-uri 'none'; frame-ancestors 'none'",
        target.origin().ascii_serialization()
    );
    response.headers_mut().insert(
        "content-security-policy",
        HeaderValue::from_str(&policy).map_err(Error::internal)?,
    );
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    Ok(response)
}
pub fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn logout(value: serde_json::Value) -> Result<Response> {
    let urls: Vec<&str> = value["frontchannel_urls"]
        .as_array()
        .map(|v| v.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    if urls.is_empty() {
        return if let Some(location) = value["redirect_uri"].as_str() {
            callback(location.into(), false)
        } else {
            Ok(axum::Json(value).into_response())
        };
    }
    let nonce = crypto::random_token("");
    let frames = urls
        .iter()
        .map(|u| format!("<iframe hidden src=\"{}\"></iframe>", escape(u)))
        .collect::<String>();
    let finish = value["redirect_uri"]
        .as_str()
        .map(|u| format!("<a hidden id=\"finish\" href=\"{}\"></a>", escape(u)))
        .unwrap_or_default();
    let html = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>Signing out</title>{finish}<script nonce=\"{nonce}\">function done(){{const a=document.getElementById('finish');if(a)location.replace(a.href)}}setTimeout(done,4000);addEventListener('load',done);</script>{frames}"
    );
    let origins: Vec<_> = urls
        .iter()
        .filter_map(|u| {
            url::Url::parse(u)
                .ok()
                .map(|u| u.origin().ascii_serialization())
        })
        .collect();
    let mut response = axum::response::Html(html).into_response();
    response.headers_mut().insert("content-security-policy",HeaderValue::from_str(&format!("default-src 'none'; script-src 'nonce-{nonce}'; frame-src {}; frame-ancestors 'none'; base-uri 'none'",origins.join(" "))).map_err(Error::internal)?);
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    Ok(response)
}

pub fn session_iframe(endpoint: &str) -> Result<Response> {
    let endpoint = serde_json::to_string(endpoint)
        .map_err(Error::internal)?
        .replace('<', "\\u003c");
    let nonce = crypto::random_token("");
    let html=format!(r#"<!doctype html><meta charset="utf-8"><title>Session check</title><script nonce="{nonce}">
+addEventListener('message',async e=>{{
+ if(typeof e.data!=='string'||e.data.length>2048||!e.source)return;
+ const parts=e.data.split(' ');if(parts.length!==2)return;
+ const source=e.source,origin=e.origin;
+ try{{const response=await fetch({endpoint},{{method:'POST',credentials:'include',headers:{{'Content-Type':'application/json'}},body:JSON.stringify({{client_id:parts[0],session_state:parts[1],origin}})}});
+ const result=response.ok?await response.json():{{status:'error'}};source.postMessage(result.status,origin);
+ }}catch{{source.postMessage('error',origin)}}
+}});</script>"#).replace("\n+","\n");
    let mut response = axum::response::Html(html).into_response();
    response.headers_mut().insert("content-security-policy",HeaderValue::from_str(&format!("default-src 'none'; script-src 'nonce-{nonce}'; connect-src 'self'; frame-ancestors *; base-uri 'none'")).map_err(Error::internal)?);
    Ok(response)
}
