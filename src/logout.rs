use crate::{
    crypto::now,
    error::{Error, Result},
    model::Client,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::identity::logout_queue::{
    Delivery, RpSession, cleanup, queue_client, queue_session, queue_user, queue_user_client,
    record,
};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct LogoutRequest {
    pub id_token_hint: Option<String>,
    pub client_id: Option<String>,
    pub post_logout_redirect_uri: Option<String>,
    pub state: Option<String>,
}

/// Reads needed to bind an ID token hint to its registered logout client.
pub(crate) trait LogoutHintTx: crate::keyring::KeyringTx {
    fn logout_client(&self, id: &str) -> Result<Option<Client>>;
}

pub(crate) fn verify_hint(tx: &impl LogoutHintTx, token: &str, issuer: &str) -> Result<Value> {
    let bad = || Error::bad("Invalid ID token hint");
    let header = jsonwebtoken::decode_header(token).map_err(|_| bad())?;
    if ![
        jsonwebtoken::Algorithm::RS256,
        jsonwebtoken::Algorithm::ES256,
        jsonwebtoken::Algorithm::EdDSA,
    ]
    .contains(&header.alg)
        || !matches!(header.typ.as_deref(), Some("JWT" | "dpop+id_token"))
    {
        return Err(bad());
    }
    let jwks = crate::keyring::public_keys(tx)?;
    let jwk = jwks
        .into_iter()
        .find(|j| j["kid"].as_str() == header.kid.as_deref())
        .ok_or_else(bad)?;
    if jwk["alg"].as_str()
        != Some(match header.alg {
            jsonwebtoken::Algorithm::RS256 => "RS256",
            jsonwebtoken::Algorithm::ES256 => "ES256",
            jsonwebtoken::Algorithm::EdDSA => "EdDSA",
            _ => return Err(bad()),
        })
    {
        return Err(bad());
    }
    let key = jsonwebtoken::DecodingKey::from_jwk(&serde_json::from_value(jwk).map_err(|_| bad())?)
        .map_err(|_| bad())?;
    let mut validation = jsonwebtoken::Validation::new(header.alg);
    use base64::Engine;
    let payload = token.split('.').nth(1).ok_or_else(bad)?;
    let untrusted: Value = serde_json::from_slice(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| bad())?,
    )
    .map_err(|_| bad())?;
    let cid = untrusted["aud"].as_str().ok_or_else(bad)?;
    let client = tx.logout_client(cid)?.ok_or_else(bad)?;
    validation.set_issuer(&[crate::issuer::for_client(issuer, &client)]);
    validation.validate_exp = false;
    validation.validate_aud = false;
    validation.validate_nbf = true;
    let claims = jsonwebtoken::decode::<Value>(token, &key, &validation)
        .map_err(|_| bad())?
        .claims;
    if claims["iat"].as_u64().is_none_or(|iat| iat > now() + 60) || !claims["exp"].is_u64() {
        return Err(bad());
    }
    Ok(claims)
}

/// Claims and completes logout deliveries using the server's persisted outbox.
pub trait LogoutDeliveryWorker: Clone + Send + Sync + 'static {
    fn claim_logout_deliveries(&self) -> Result<Vec<(Delivery, String)>>;
    fn finish_logout_delivery(&self, id: &str, attempt: u32, status: Option<u16>) -> Result<()>;
}

pub async fn deliver<W: LogoutDeliveryWorker>(core: W) -> Result<()> {
    let worker = core.clone();
    let pending = tokio::task::spawn_blocking(move || {
        crate::telemetry::in_activity(crate::telemetry::Activity::LogoutDelivery, || {
            worker.claim_logout_deliveries()
        })
    })
    .await
    .map_err(Error::internal)??;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(Error::internal)?;
    let mut jobs = tokio::task::JoinSet::new();
    for (delivery, token) in pending {
        let http = http.clone();
        let core = core.clone();
        jobs.spawn(async move {
            let status = http
                .post(&delivery.uri)
                .form(&[("logout_token", token)])
                .send()
                .await
                .ok()
                .map(|r| r.status().as_u16());
            tokio::task::spawn_blocking(move || {
                crate::telemetry::in_activity(crate::telemetry::Activity::LogoutDelivery, || {
                    core.finish_logout_delivery(&delivery.id, delivery.attempts, status)
                })
            })
            .await
            .map_err(Error::internal)?
        });
    }
    // A failed finish must not detach other blocking finishes and free the
    // background pass's capacity while those operations are still running.
    let mut outcome = Ok(());
    while let Some(result) = jobs.join_next().await {
        let result = result.map_err(Error::internal).and_then(|result| result);
        if outcome.is_ok() {
            outcome = result;
        }
    }
    outcome
}
