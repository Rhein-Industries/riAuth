//! Compact browser administration: `/admin` and its same-origin JSON routes.
//!
//! Every route authenticates the administrator's browser session (SSO cookie) as a management
//! credential and calls the same `Core` methods as the bearer API, sharing authorization
//! and validation. Mutations also share If-Match revisions, idempotency receipts and audit.
//! Reads require the portal header; POSTs also pass `browser_write_guard`.
use super::http::{browser_write_guard, portal_html};
use crate::{
    agent::browser_credential,
    api::{App, sso_cookie},
    claims::{Explain, Simulation},
    delegation::GrantInput,
    error::{Error, Result},
    model::{ClientPatch, NewClient, NewUser, UserPatch, UserView},
    passkey::NewPasskeyAdmin,
    provisioning::DismissDeactivation,
};
use axum::{
    Json, Router,
    extract::rejection::JsonRejection,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
    routing::{get, patch, post, put},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use webauthn_rs::prelude::RegisterPublicKeyCredential;

pub fn routes() -> Router<App> {
    let routes = Router::new()
        .route("/api/admin/session", get(session))
        .route(
            "/api/admin/agent-self-service",
            get(agent_self_service).put(set_agent_self_service),
        )
        .route("/api/admin/users", get(users).post(create_user))
        .route("/api/admin/users/passkey/start", post(passkey_admin_start))
        .route("/api/admin/users/passkey/first", post(passkey_admin_first))
        .route(
            "/api/admin/users/passkey/finish",
            post(passkey_admin_finish),
        )
        .route(
            "/api/admin/users/passkey/cancel",
            post(passkey_admin_cancel),
        )
        .route("/api/admin/users/{username}", patch(update_user))
        .route(
            "/api/admin/users/{username}/delegated-grants",
            get(human_grants).put(set_human_grants),
        )
        .route(
            "/api/admin/users/{username}/delegated-grants/changes",
            post(stage_human_grants),
        )
        .route(
            "/api/admin/delegated-grant-changes/{id}",
            get(human_grant_change),
        )
        .route(
            "/api/admin/delegated-grant-changes/{id}/approve",
            post(approve_human_grant_change),
        )
        .route(
            "/api/admin/delegated-grant-changes/{id}/execute",
            post(execute_human_grant_change),
        )
        .route(
            "/api/admin/delegated-grant-changes/{id}/cancel",
            post(cancel_human_grant_change),
        )
        .route("/api/admin/invitations", get(invitations).post(invite))
        .route(
            "/api/admin/invitations/{username}",
            axum::routing::delete(revoke_invitation),
        )
        .route(
            "/api/admin/groups/{name}/membership-changes",
            post(stage_group_membership),
        )
        .route(
            "/api/admin/group-membership-changes/{id}",
            get(group_membership_change),
        )
        .route(
            "/api/admin/group-membership-changes/{id}/approve",
            post(approve_group_membership_change),
        )
        .route(
            "/api/admin/group-membership-changes/{id}/execute",
            post(execute_group_membership_change),
        )
        .route(
            "/api/admin/group-membership-changes/{id}/cancel",
            post(cancel_group_membership_change),
        )
        .route("/api/admin/groups", get(groups).post(create_group))
        .route(
            "/api/admin/groups/{name}/members/{username}",
            put(add_member).delete(remove_member),
        )
        .route("/api/admin/clients", get(clients).post(create_client))
        .route(
            "/api/admin/client-creation-changes",
            post(stage_client_creation),
        )
        .route(
            "/api/admin/client-creation-changes/{id}",
            get(client_creation_change),
        )
        .route(
            "/api/admin/client-creation-changes/{id}/approve",
            post(approve_client_creation_change),
        )
        .route(
            "/api/admin/client-creation-changes/{id}/execute",
            post(execute_client_creation_change),
        )
        .route(
            "/api/admin/client-creation-changes/{id}/cancel",
            post(cancel_client_creation_change),
        )
        .route(
            "/api/admin/clients/{id}/endpoint-changes",
            post(stage_client_endpoint),
        )
        .route(
            "/api/admin/client-endpoint-changes/{id}",
            get(client_endpoint_change),
        )
        .route(
            "/api/admin/client-endpoint-changes/{id}/approve",
            post(approve_client_endpoint_change),
        )
        .route(
            "/api/admin/client-endpoint-changes/{id}/execute",
            post(execute_client_endpoint_change),
        )
        .route(
            "/api/admin/client-endpoint-changes/{id}/cancel",
            post(cancel_client_endpoint_change),
        )
        .route(
            "/api/admin/clients/{id}/status-changes",
            post(stage_client_status),
        )
        .route(
            "/api/admin/client-status-changes/{id}",
            get(client_status_change),
        )
        .route(
            "/api/admin/client-status-changes/{id}/approve",
            post(approve_client_status_change),
        )
        .route(
            "/api/admin/client-status-changes/{id}/execute",
            post(execute_client_status_change),
        )
        .route(
            "/api/admin/client-status-changes/{id}/cancel",
            post(cancel_client_status_change),
        )
        .route(
            "/api/admin/clients/{id}/policy-changes",
            post(stage_client_policy),
        )
        .route(
            "/api/admin/client-policy-changes/{id}",
            get(client_policy_change),
        )
        .route(
            "/api/admin/client-policy-changes/{id}/approve",
            post(approve_client_policy_change),
        )
        .route(
            "/api/admin/client-policy-changes/{id}/execute",
            post(execute_client_policy_change),
        )
        .route(
            "/api/admin/client-policy-changes/{id}/cancel",
            post(cancel_client_policy_change),
        )
        .route("/api/admin/clients/{id}", patch(update_client))
        .route("/api/admin/clients/{id}/rotate-secret", post(rotate_secret))
        .route("/api/admin/clients/{id}/diagnostics", get(diagnostics))
        .route("/api/admin/clients/{id}/explain", post(explain))
        .route("/api/admin/policy/simulate", post(simulate_policy))
        .route("/api/admin/client-checks", post(check_client))
        .route("/api/admin/provisioning/deactivations", get(deactivations))
        .route(
            "/api/admin/provisioning/deactivations/{id}/dismiss",
            post(dismiss_deactivation),
        )
        .route("/api/admin/audit", get(audit));
    #[cfg(feature = "platform")]
    let routes = routes
        .merge(access_routes())
        .merge(cloud_routes())
        .merge(workflow_routes());
    routes
}

#[cfg(feature = "platform")]
fn workflow_routes() -> Router<App> {
    Router::new()
        .route("/api/admin/workflows", get(workflows))
        .route("/api/admin/workflows/plan", post(workflow_plan))
        .route("/api/admin/workflows/apply", post(workflow_apply))
        .route("/api/admin/workflows/review", post(workflow_review))
        .route("/api/admin/workflows/activate", post(workflow_activate))
        .route("/api/admin/workflows/revoke", post(workflow_revoke))
}

#[cfg(feature = "platform")]
async fn workflows(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.list_workflow_definitions(&token).map(Json))
        .await
}

#[cfg(feature = "platform")]
async fn workflow_plan(
    State(app): State<App>,
    headers: HeaderMap,
    Json(definition): Json<crate::workflow::Definition>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    let manifest = crate::state::Manifest {
        api_version: "riauth/v1".into(),
        workflows: vec![definition],
        ..Default::default()
    };
    app.run(move |core| {
        core.plan_state(&token, manifest)
            .map(|plan| Json(json!(plan)))
    })
    .await
}

#[cfg(feature = "platform")]
async fn workflow_apply(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::state::ApplyRequest>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    // This browser surface only applies a single workflow. The shared plan
    // remains bound to the actor, revision, manifest, and exact stored hash.
    if input.plan.manifest.workflows.len() != 1
        || !input.plan.manifest.users.is_empty()
        || !input.plan.manifest.groups.is_empty()
        || !input.plan.manifest.clients.is_empty()
        || !input.plan.manifest.sources.is_empty()
        || !input.plan.manifest.source_links.is_empty()
        || !input.plan.manifest.delegated_grants.is_empty()
        || !input.plan.manifest.ssf_streams.is_empty()
        || input.plan.manifest.has_connectors()
        || !input.secrets.is_empty()
    {
        return Err(Error::bad("Workflow editor applies one workflow only"));
    }
    app.run(move |core| core.apply_state(&token, input).map(Json))
        .await
}

#[cfg(feature = "platform")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowReviewBody {
    plan_id: String,
    decision: String,
}

#[cfg(feature = "platform")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowPlanBody {
    plan_id: String,
}

#[cfg(feature = "platform")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowRevokeBody {
    workflow_id: String,
    approval_id: Option<String>,
}

#[cfg(feature = "platform")]
async fn workflow_review(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<WorkflowReviewBody>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| {
        core.review_workflow(&token, &input.plan_id, &input.decision)
            .map(Json)
    })
    .await
}

#[cfg(feature = "platform")]
async fn workflow_activate(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<WorkflowPlanBody>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.activate_workflow(&token, &input.plan_id).map(Json))
        .await
}

#[cfg(feature = "platform")]
async fn workflow_revoke(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<WorkflowRevokeBody>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| {
        let approval_id = input
            .approval_id
            .as_deref()
            .ok_or_else(|| Error::bad("Workflow revocation requires approval_id"))?;
        core.revoke_workflow_approval_targeted(&token, &input.workflow_id, approval_id)
            .map(Json)
    })
    .await
}

pub fn browser_routes() -> Router<App> {
    Router::new()
        .route("/admin", get(page))
        .route("/admin/", get(page))
        .route(
            "/portal/assets/admin.css",
            get(crate::portal::theme::builtin),
        )
        .route(
            "/portal/assets/admin.js",
            get(crate::portal::theme::builtin),
        )
}

#[cfg(feature = "platform")]
fn cloud_routes() -> Router<App> {
    Router::new()
        .route("/api/admin/cloud-directories", get(cloud_directories))
        .route(
            "/api/admin/cloud-directories/{kind}/{id}/operations",
            get(cloud_operations),
        )
        .route(
            "/api/admin/cloud-directories/{kind}/{id}/test-connection",
            post(cloud_test_connection),
        )
        .route(
            "/api/admin/cloud-directories/{kind}/{id}/verify-credential",
            post(cloud_verify_credential),
        )
        .route(
            "/api/admin/cloud-directories/{kind}/{id}/verify-controller",
            post(cloud_verify_controller),
        )
        .route(
            "/api/admin/cloud-directories/{kind}/{id}/schedule",
            patch(cloud_schedule_update),
        )
}

#[cfg(feature = "platform")]
async fn cloud_directories(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| {
        let mut rows = core
            .cloud_directories(&token, "workspace")?
            .as_array()
            .cloned()
            .unwrap_or_default();
        rows.extend(
            core.cloud_directories(&token, "entra")?
                .as_array()
                .cloned()
                .unwrap_or_default(),
        );
        Ok(Json(json!(rows)))
    })
    .await
}

#[cfg(feature = "platform")]
async fn cloud_operations(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.cloud_operations(&token, &kind, &id).map(Json))
        .await
}

#[cfg(feature = "platform")]
async fn cloud_test_connection(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run_connector(
        crate::background::ConnectorWork::target(&kind, &id),
        move |core| core.cloud_test_connection(&token, &kind, &id).map(Json),
    )
    .await
}

#[cfg(feature = "platform")]
async fn cloud_verify_credential(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run_connector(
        crate::background::ConnectorWork::target(&kind, &id),
        move |core| core.cloud_verify_credential(&token, &kind, &id).map(Json),
    )
    .await
}

#[cfg(feature = "platform")]
async fn cloud_verify_controller(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.cloud_verify_controller(&token, &kind, &id).map(Json))
        .await
}

#[cfg(feature = "platform")]
async fn cloud_schedule_update(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
    Json(input): Json<crate::reconciliation::CloudScheduleUpdate>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| {
        core.cloud_schedule_update(&token, &kind, &id, input)
            .map(Json)
    })
    .await
}

#[cfg(feature = "platform")]
fn access_routes() -> Router<App> {
    Router::new()
        .route("/api/admin/access/requests", get(access_requests))
        .route("/api/admin/access/requests/{id}/approve", post(approve))
        .route("/api/admin/access/requests/{id}/deny", post(deny))
        .route("/api/admin/access/grants", get(access_grants))
        .route("/api/admin/access/grants/{id}/revoke", post(revoke_grant))
}

async fn page(State(app): State<App>) -> Response {
    portal_html(app.core.runtime.frontend.page("admin.html"), &app, true)
}

/// A read carries the portal header, which a cross-site page cannot add without a CORS
/// preflight that riAuth never grants, and any Fetch Metadata must say same-origin.
fn reader(app: &App, headers: &HeaderMap) -> Result<String> {
    let fetch_site = headers.get_all("sec-fetch-site");
    if headers.get_all("x-riauth-portal").iter().count() != 1
        || headers.get("x-riauth-portal").and_then(|h| h.to_str().ok()) != Some("1")
        || fetch_site.iter().count() > 1
        || fetch_site
            .iter()
            .next()
            .is_some_and(|h| h.to_str().ok() != Some("same-origin"))
    {
        return Err(Error::forbidden());
    }
    credential(app, headers)
}

fn writer(app: &App, headers: &HeaderMap) -> Result<String> {
    browser_write_guard(app, headers)?;
    credential(app, headers)
}

fn credential(app: &App, headers: &HeaderMap) -> Result<String> {
    sso_cookie(app, headers)
        .map(browser_credential)
        .ok_or_else(Error::unauthorized)
}

/// The signed-in administrator and the configuration revision edits send as If-Match.
async fn session(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    let cookie = sso_cookie(&app, &headers).unwrap_or_default().to_owned();
    app.run(move |core| {
        core.store.read(|tx| {
            let (user, session) = core.browser_user(tx, &cookie)?;
            core.management(tx, &token, "state.read", "state/revision")?;
            // Public equality marker only: domain separation keeps this from being
            // a credential or a session lookup key. Bind both the SSO cookie and
            // its session so a new sign-in as the same account clears UI intent.
            let session_marker = crate::crypto::digest(&format!(
                "riauth/admin-session-marker/v1\n{cookie}\n{}",
                session.id
            ));
            Ok(Json(json!({
                "user": UserView::from(&user),
                "session_marker": session_marker,
                "mfa": session.identity.mfa,
                "edition": crate::edition::NAME,
                "reviewed_client_creation": core.config.reviewed_client_creation,
                "expires_at": session.expires_at,
                "revision": tx.get::<u64>("meta", "revision")?.unwrap_or(0),
            })))
        })
    })
    .await
}

macro_rules! read {
    ($name:ident, $method:ident) => {
        async fn $name(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
            let token = reader(&app, &headers)?;
            app.run(move |core| core.$method(&token).map(Json)).await
        }
    };
}
read!(users, list_users);
read!(groups, list_groups);
read!(clients, list_clients);
read!(invitations, account_invitations);
read!(deactivations, provisioning_deactivations);

async fn dismiss_deactivation(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<DismissDeactivation>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| {
        core.provisioning_deactivation_dismiss(&token, &id, input)
            .map(Json)
    })
    .await
}

async fn human_grants(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.human_grants(&token, &username).map(Json))
        .await
}
async fn set_human_grants(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Json(grants): Json<Vec<GrantInput>>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.set_human_grants(&token, &username, grants).map(Json))
        .await
}
async fn stage_human_grants(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Json(grants): Json<Vec<GrantInput>>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.stage_human_grants(&token, &username, grants).map(Json))
        .await
}
async fn human_grant_change(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.human_grant_change(&token, &id).map(Json))
        .await
}
macro_rules! grant_change_handler {
    ($name:ident) => {
        async fn $name(
            State(app): State<App>,
            headers: HeaderMap,
            Path(id): Path<String>,
            Json(binding): Json<crate::delegation::GrantChangeBinding>,
        ) -> Result<Json<Value>> {
            let token = writer(&app, &headers)?;
            app.run(move |core| core.$name(&token, &id, binding).map(Json))
                .await
        }
    };
}
grant_change_handler!(approve_human_grant_change);
grant_change_handler!(execute_human_grant_change);
grant_change_handler!(cancel_human_grant_change);

async fn stage_group_membership(
    State(app): State<App>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(input): Json<crate::model::GroupMembershipInput>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.stage_group_membership(&token, &name, input).map(Json))
        .await
}
async fn group_membership_change(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.group_membership_change(&token, &id).map(Json))
        .await
}
// Every review class accepts only the immutable canonical digest.
grant_change_handler!(approve_group_membership_change);
grant_change_handler!(execute_group_membership_change);
grant_change_handler!(cancel_group_membership_change);

async fn stage_client_policy(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<crate::model::ClientPolicyInput>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.stage_client_policy(&token, &id, input).map(Json))
        .await
}
async fn client_policy_change(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.client_policy_change(&token, &id).map(Json))
        .await
}
grant_change_handler!(approve_client_policy_change);
grant_change_handler!(execute_client_policy_change);
grant_change_handler!(cancel_client_policy_change);

async fn stage_client_endpoint(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<crate::model::ClientEndpointInput>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.stage_client_endpoint(&token, &id, input).map(Json))
        .await
}
async fn client_endpoint_change(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.client_endpoint_change(&token, &id).map(Json))
        .await
}
grant_change_handler!(approve_client_endpoint_change);
grant_change_handler!(execute_client_endpoint_change);
grant_change_handler!(cancel_client_endpoint_change);

async fn stage_client_status(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<crate::model::ClientStatusInput>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.stage_client_status(&token, &id, input).map(Json))
        .await
}
async fn client_status_change(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.client_status_change(&token, &id).map(Json))
        .await
}
grant_change_handler!(approve_client_status_change);
grant_change_handler!(execute_client_status_change);
grant_change_handler!(cancel_client_status_change);

async fn stage_client_creation(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewClient>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.stage_client_creation(&token, input).map(Json))
        .await
}
async fn client_creation_change(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.client_creation_change(&token, &id).map(Json))
        .await
}
grant_change_handler!(approve_client_creation_change);
grant_change_handler!(execute_client_creation_change);
grant_change_handler!(cancel_client_creation_change);

/// Access review is limited to administrators here; `pam` still decides who may approve.
#[cfg(feature = "platform")]
macro_rules! access_read {
    ($name:ident, $method:ident) => {
        async fn $name(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
            let token = reader(&app, &headers)?;
            app.run(move |core| {
                core.store.read(|tx| core.principal(tx, &token).map(drop))?;
                core.$method(&token).map(Json)
            })
            .await
        }
    };
}
#[cfg(feature = "platform")]
access_read!(access_requests, list_access_requests);
#[cfg(feature = "platform")]
access_read!(access_grants, list_access_grants);

#[cfg(feature = "platform")]
fn access_writer(app: &App, headers: &HeaderMap) -> Result<String> {
    let token = writer(app, headers)?;
    super::access_review::require_write_preconditions(headers)?;
    Ok(token)
}

macro_rules! decide {
    ($name:ident, $writer:ident, |$core:ident, $token:ident, $id:ident| $call:expr) => {
        async fn $name(
            State(app): State<App>,
            headers: HeaderMap,
            Path($id): Path<String>,
        ) -> Result<Json<Value>> {
            let $token = $writer(&app, &headers)?;
            app.run(move |$core| {
                $core
                    .store
                    .read(|tx| $core.principal(tx, &$token).map(drop))?;
                $call.map(Json)
            })
            .await
        }
    };
}
#[cfg(feature = "platform")]
decide!(approve, access_writer, |core, token, id| core
    .decide_access(&token, &id, true));
#[cfg(feature = "platform")]
decide!(deny, access_writer, |core, token, id| core
    .decide_access(&token, &id, false));
#[cfg(feature = "platform")]
decide!(revoke_grant, access_writer, |core, token, id| core
    .revoke_access(&token, &id));
decide!(rotate_secret, writer, |core, token, id| core
    .rotate_client_secret(&token, &id));

#[derive(Deserialize)]
struct AuditQuery {
    limit: Option<usize>,
}
async fn audit(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| {
        core.audit_events(&token, query.limit.unwrap_or(50).min(200))
            .map(Json)
    })
    .await
}

async fn create_user(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewUser>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.create_user(&token, input).map(Json))
        .await
}

async fn passkey_admin_start(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewPasskeyAdmin>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    let cookie = sso_cookie(&app, &headers)
        .ok_or_else(Error::unauthorized)?
        .to_owned();
    app.run(move |core| core.admin_passkey_start(&token, &cookie, input).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyAnswer {
    ceremony: String,
    credential: RegisterPublicKeyCredential,
}

async fn passkey_admin_first(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyAnswer>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    let cookie = sso_cookie(&app, &headers)
        .ok_or_else(Error::unauthorized)?
        .to_owned();
    app.run(move |core| {
        core.admin_passkey_first(&token, &cookie, &input.ceremony, input.credential)
            .map(Json)
    })
    .await
}

async fn passkey_admin_finish(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyAnswer>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    let cookie = sso_cookie(&app, &headers)
        .ok_or_else(Error::unauthorized)?
        .to_owned();
    app.run(move |core| {
        core.admin_passkey_finish(&token, &cookie, &input.ceremony, input.credential)
            .map(Json)
    })
    .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyCancel {
    ceremony: String,
}

async fn passkey_admin_cancel(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyCancel>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    let cookie = sso_cookie(&app, &headers)
        .ok_or_else(Error::unauthorized)?
        .to_owned();
    app.run(move |core| {
        core.admin_passkey_cancel(&token, &cookie, &input.ceremony)
            .map(Json)
    })
    .await
}
async fn agent_self_service(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.agent_self_service(&token).map(Json))
        .await
}

async fn set_agent_self_service(
    State(app): State<App>,
    headers: HeaderMap,
    Json(setting): Json<crate::agent::AgentSelfService>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.set_agent_self_service(&token, setting).map(Json))
        .await
}

async fn update_user(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Json(input): Json<UserPatch>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    // The shared user writer refuses a changed address marked verified in the same change.
    app.run(move |core| core.update_user(&token, &username, input).map(Json))
        .await
}

/// The account invitation API's own methods: the same permissions, mail requirement,
/// receipts and audit as `/api/account/invitations`.
async fn invite(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::lifecycle::Invitation>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.account_invite(&token, input).map(Json))
        .await
}
async fn revoke_invitation(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.account_invitation_revoke(&token, &username).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewGroup {
    name: String,
}
async fn create_group(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewGroup>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.create_group(&token, &input.name).map(Json))
        .await
}
async fn add_member(
    State(app): State<App>,
    headers: HeaderMap,
    Path((group, username)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.group_member(&token, &group, &username, true).map(Json))
        .await
}
async fn remove_member(
    State(app): State<App>,
    headers: HeaderMap,
    Path((group, username)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| {
        core.group_member(&token, &group, &username, false)
            .map(Json)
    })
    .await
}

async fn create_client(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewClient>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.create_client(&token, input).map(Json))
        .await
}
async fn update_client(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<ClientPatch>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.update_client(&token, &id, input).map(Json))
        .await
}

/// The setup wizard's preflight: the create path's authorization and validation, no write.
async fn check_client(
    State(app): State<App>,
    headers: HeaderMap,
    input: std::result::Result<Json<NewClient>, JsonRejection>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    // The wizard shows this text, so a draft riAuth cannot read is explained, not a bare 422.
    let Json(input) = input.map_err(|rejection| {
        Error::bad(format!(
            "riAuth couldn't read this configuration: {}",
            rejection.body_text()
        ))
    })?;
    app.run(move |core| core.check_new_client(&token, input).map(Json))
        .await
}
async fn diagnostics(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.client_diagnostics(&token, &id).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SignInTest {
    username: String,
    #[serde(default)]
    scope: BTreeSet<String>,
    #[serde(default)]
    mfa: bool,
}
/// Simulates a person's sign-in with the policy engine; no token is issued.
async fn explain(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<SignInTest>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| {
        core.explain(
            &token,
            Explain {
                client_id: id,
                username: input.username,
                scope: input.scope,
                mfa: input.mfa,
            },
        )
        .map(Json)
    })
    .await
}

/// Same read-only management decision as the bearer API and CLI, with the
/// browser's same-origin POST guard and SSO credential.
async fn simulate_policy(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Simulation>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.simulate_policy(&token, input).map(Json))
        .await
}
