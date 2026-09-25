//! RFC 9449 proof validation and Authentik's bound_key ID-token profile.
use crate::{
    crypto::{digest, now},
    error::{Error, Result},
    jose::PublicJwk,
    model::{Client, Grant},
    oidc::TokenRequest,
    store::Tx,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, Validation};
use serde_json::{Value, json};

fn invalid() -> Error {
    Error::oauth(
        "invalid_dpop_proof",
        "Invalid, stale, mismatched or replayed DPoP proof",
    )
}

pub fn thumbprint(key: &PublicJwk) -> Result<String> {
    let canonical = match key.kty.as_str() {
        "RSA" => json!({"e":key.e,"kty":"RSA","n":key.n}),
        "EC" => json!({"crv":key.crv,"kty":"EC","x":key.x,"y":key.y}),
        "OKP" => json!({"crv":key.crv,"kty":"OKP","x":key.x}),
        _ => return Err(invalid()),
    };
    Ok(digest(
        &serde_json::to_string(&canonical).map_err(Error::internal)?,
    ))
}

pub fn verify(
    tx: &Tx<'_>,
    proof: &str,
    method: &str,
    endpoint: &str,
    access_token: Option<&str>,
) -> Result<String> {
    if proof.len() > 16_384 {
        return Err(invalid());
    }
    let header = jsonwebtoken::decode_header(proof).map_err(|_| invalid())?;
    if header.typ.as_deref() != Some("dpop+jwt")
        || header.jku.is_some()
        || header.x5u.is_some()
        || header.crit.as_ref().is_some_and(|c| !c.is_empty())
    {
        return Err(invalid());
    }
    let mut raw: Value = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(proof.split('.').next().ok_or_else(invalid)?)
            .map_err(|_| invalid())?,
    )
    .map_err(|_| invalid())?;
    if raw.get("b64").is_some() {
        return Err(invalid());
    }
    let key = raw
        .get_mut("jwk")
        .filter(|v| v.is_object())
        .ok_or_else(invalid)?;
    let algorithm = match header.alg {
        Algorithm::RS256 => "RS256",
        Algorithm::ES256 => "ES256",
        Algorithm::EdDSA => "EdDSA",
        _ => return Err(invalid()),
    };
    if key.get("alg").is_some_and(|v| v != algorithm) {
        return Err(invalid());
    }
    key["alg"] = json!(algorithm);
    if key.get("kid").is_none() {
        key["kid"] = json!("dpop");
    }
    // deny_unknown_fields rejects private components, symmetric keys and untrusted URL members.
    let key: PublicJwk = serde_json::from_value(key.clone()).map_err(|_| invalid())?;
    key.validate().map_err(|_| invalid())?;
    let mut validation = Validation::new(header.alg);
    validation.required_spec_claims.clear();
    validation.validate_exp = false;
    validation.validate_aud = false;
    validation.validate_nbf = true;
    validation.leeway = 0;
    let claims = jsonwebtoken::decode::<Value>(proof, &key.decoding_key()?, &validation)
        .map_err(|_| invalid())?
        .claims;
    let iat = claims["iat"].as_u64().ok_or_else(invalid)?;
    let jti = claims["jti"]
        .as_str()
        .filter(|v| !v.is_empty() && v.len() <= 256)
        .ok_or_else(invalid)?;
    let mut target = url::Url::parse(endpoint).map_err(|_| invalid())?;
    target.set_query(None);
    target.set_fragment(None);
    if iat.abs_diff(now()) > 60
        || claims["htm"].as_str() != Some(method)
        || claims["htu"].as_str() != Some(target.as_str())
    {
        return Err(invalid());
    }
    if let Some(token) = access_token {
        if claims["ath"].as_str() != Some(&digest(token)) {
            return Err(invalid());
        }
    } else if claims.get("ath").is_some() {
        return Err(invalid());
    }
    let jkt = thumbprint(&key)?;
    let replay = digest(&format!("{jkt}\0{jti}"));
    if tx
        .get::<u64>("dpop_replays", &replay)?
        .is_some_and(|exp| exp > now())
    {
        return Err(invalid());
    }
    tx.put("dpop_replays", &replay, &(now() + 120))?;
    Ok(jkt)
}

pub fn bind(
    tx: &Tx<'_>,
    client: &Client,
    request: &TokenRequest,
    grant: &mut Grant,
    authorization_jkt: Option<&str>,
) -> Result<()> {
    bind_clients(tx, &[client], request, grant, authorization_jkt)
}

/// Bind using the combined DPoP policies of every supplied client (e.g. exchange requester + target).
pub fn bind_clients(
    tx: &Tx<'_>,
    clients: &[&Client],
    request: &TokenRequest,
    grant: &mut Grant,
    authorization_jkt: Option<&str>,
) -> Result<()> {
    let bound_id = grant.scopes.contains("bound_key");
    let access_binding = clients
        .iter()
        .any(|client| client.settings.dpop_bound_access_tokens);
    let required = access_binding
        || bound_id
        || authorization_jkt.is_some()
        || grant.confirmation_jkt.is_some()
        || grant.id_token_jkt.is_some();
    let Some(proof) = request.dpop_proof.as_deref() else {
        if required {
            return Err(invalid());
        }
        return Ok(());
    };
    let issuer = tx.get::<String>("meta", "issuer")?.ok_or_else(invalid)?;
    let jkt = verify(
        tx,
        proof,
        "POST",
        &format!("{}/oauth/token", issuer.trim_end_matches('/')),
        None,
    )?;
    if [
        authorization_jkt,
        grant.confirmation_jkt.as_deref(),
        grant.id_token_jkt.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|expected| expected != jkt)
    {
        return Err(invalid());
    }
    if bound_id {
        grant.id_token_jkt = Some(jkt.clone());
    }
    // bound_key alone follows Authentik's ID-only profile; other DPoP requests bind access tokens.
    if !bound_id || access_binding || authorization_jkt.is_some() {
        grant.confirmation_jkt = Some(jkt);
    }
    if access_binding && grant.confirmation_jkt.is_none() {
        return Err(invalid());
    }
    Ok(())
}

pub fn resource(
    tx: &Tx<'_>,
    grant: &Grant,
    proof: Option<&str>,
    token: &str,
    method: &str,
    endpoint: &str,
) -> Result<()> {
    match (&grant.confirmation_jkt, proof) {
        (Some(expected), Some(proof))
            if verify(tx, proof, method, endpoint, Some(token)).map_err(|mut error| {
                if error.status.is_client_error() {
                    error.status = axum::http::StatusCode::UNAUTHORIZED;
                }
                error
            })? == *expected =>
        {
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(Error::new(
            axum::http::StatusCode::UNAUTHORIZED,
            "invalid_dpop_proof",
            "DPoP-bound token requires its matching proof",
        )),
    }
}
