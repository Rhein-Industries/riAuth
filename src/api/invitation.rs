//! Shared invitation-to-passkey transport. No caller-selected account or session.
use super::*;
use webauthn_rs::prelude::RegisterPublicKeyCredential;

pub(super) fn routes() -> Router<App> {
    Router::new()
        .route("/api/account/accept/passkey/start", post(start))
        .route("/api/account/accept/passkey/finish", post(finish))
        .route("/api/account/accept/passkey/cancel", post(cancel))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Start {
    token: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Finish {
    token: String,
    ceremony: String,
    response: RegisterPublicKeyCredential,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cancel {
    token: String,
    ceremony: String,
}

async fn start(State(app): State<App>, Json(input): Json<Start>) -> Result<Json<Value>> {
    app.run(move |core| {
        core.account_invitation_passkey_start(input.token, input.name)
            .map(Json)
    })
    .await
}

async fn finish(State(app): State<App>, Json(input): Json<Finish>) -> Result<Json<Value>> {
    app.run(move |core| {
        core.account_invitation_passkey_finish(input.token, &input.ceremony, input.response)
            .map(Json)
    })
    .await
}

async fn cancel(State(app): State<App>, Json(input): Json<Cancel>) -> Result<Json<Value>> {
    app.run(move |core| {
        core.account_invitation_passkey_cancel(input.token, &input.ceremony)
            .map(Json)
    })
    .await
}
