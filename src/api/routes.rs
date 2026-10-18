//! HTTP route assembly for integrated, gateway and worker processes.
use super::*;

async fn worker_not_served() -> Response {
    let mut response = (
        StatusCode::NOT_FOUND,
        [(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")],
        r#"{"error":"not_served","error_description":"This process role does not serve authentication or administration routes"}"#,
    )
        .into_response();
    let headers = response.headers_mut();
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    response
}

/// Probe-only HTTP surface for the worker role. Product routes stay unmounted.
pub(super) fn worker_router(core: Core) -> Router {
    let prefix = url::Url::parse(&core.config.issuer)
        .expect("validated issuer")
        .path()
        .trim_end_matches('/')
        .to_owned();
    let app = App::new(core);
    let probes = Router::new()
        .route("/livez", get(probes::live))
        .route("/readyz", get(probes::ready))
        .route("/healthz", get(probes::ready))
        .fallback(worker_not_served)
        .layer(middleware::from_fn_with_state(app.clone(), observe))
        .with_state(app);
    if prefix.is_empty() {
        probes
    } else {
        Router::new()
            .nest(&prefix, probes)
            .fallback(worker_not_served)
    }
}

pub fn router(core: Core) -> Router {
    let prefix = url::Url::parse(&core.config.issuer)
        .expect("validated issuer")
        .path()
        .trim_end_matches('/')
        .to_owned();
    #[cfg(feature = "platform")]
    let ssf_document = core.ssf_metadata();
    let app = App::new(core);
    let browser_ui = app.core.config.browser_ui;
    let routes = Router::new()
        .route("/", get(crate::portal::http::root))
        .merge(crate::portal::http::routes())
        .merge(crate::portal::admin::routes())
        .merge(crate::bootstrap::closed_routes(browser_ui))
        .merge(interaction::routes())
        .route("/.well-known/openid-configuration", get(discovery))
        .route("/.well-known/oauth-authorization-server", get(discovery))
        .route("/oauth/jwks", get(jwks))
        .route(
            "/oauth/authorize",
            get(authorization_details).post(authorize),
        )
        .route("/oauth/resume/{id}", get(browser_resume))
        .route("/api/authorization/{code}", get(browser_details))
        .route("/api/authorization/decision", post(browser_decide))
        .route("/oauth/token", post(token))
        .route("/oauth/register", post(dynamic_register))
        .route("/oauth/par", post(push_authorization))
        .route(
            "/api/registration",
            get(registration_templates).post(create_registration),
        )
        .route(
            "/api/registration/{id}",
            axum::routing::delete(revoke_registration),
        )
        .route("/oauth/device/code", post(device_start))
        .route("/oauth/userinfo", get(userinfo).post(userinfo))
        .route("/oauth/introspect", post(introspect))
        .route("/oauth/revoke", post(revoke))
        .route("/oauth/logout", get(end_session_get).post(end_session_post))
        .route("/oauth/logout/resume/{id}", get(logout_request_resume))
        .route("/api/logout-requests/{code}", get(logout_request_details))
        .route(
            "/api/logout-requests/{code}/decision",
            post(logout_request_decide),
        )
        .route("/oauth/session/iframe", get(session_iframe))
        .route("/oauth/session/check", post(session_check))
        .route("/api/login", post(login))
        .route("/api/account/verify-request", post(account_verify_request))
        .route("/api/account/reset-request", post(account_reset_request))
        .route("/api/account/complete", post(account_complete))
        .merge(invitation::routes())
        .merge(prepared_changes::routes())
        .route(
            "/api/account/invitations",
            get(account_invitations).post(account_invite),
        )
        .route(
            "/api/account/invitations/{username}",
            axum::routing::delete(account_invitation_revoke),
        )
        .route("/api/operations/mail", get(mail_deliveries))
        .route("/api/provisioning/targets", get(provisioning_targets))
        .route(
            "/api/provisioning/targets/{id}/test-connection",
            post(provisioning_test_connection),
        )
        .route(
            "/api/provisioning/targets/{id}/plan",
            post(provisioning_plan),
        )
        .route("/api/provisioning/plans/{id}", get(provisioning_plan_get))
        .route(
            "/api/provisioning/plans/{id}/apply",
            post(provisioning_apply),
        )
        .route("/api/provisioning/jobs", get(provisioning_jobs))
        .route("/api/provisioning/jobs/{id}/stop", post(provisioning_stop))
        .route(
            "/api/provisioning/jobs/{id}/resolve",
            post(provisioning_resolve),
        )
        .route(
            "/api/provisioning/jobs/{id}/recover-dispatch",
            post(provisioning_recover_dispatch),
        )
        .route(
            "/api/provisioning/deactivations",
            get(provisioning_deactivations),
        )
        .route(
            "/api/provisioning/deactivations/{id}/retry",
            post(provisioning_deactivation_retry),
        )
        .route(
            "/api/provisioning/deactivations/{id}/resolve",
            post(provisioning_deactivation_resolve),
        )
        .route(
            "/api/provisioning/deactivations/{id}/dismiss",
            post(provisioning_deactivation_dismiss),
        )
        .route(
            "/api/provisioning/deactivations/{id}/recover-dispatch",
            post(provisioning_deactivation_recover_dispatch),
        )
        .route(
            "/api/reconciliation/schedules",
            get(reconciliation_schedules),
        )
        .route("/api/reconciliation/jobs", get(reconciliation_jobs))
        .route(
            "/api/reconciliation/{kind}/{id}/events",
            post(reconciliation_event),
        )
        .route("/api/directories", get(directories))
        .route("/api/directories/{id}/plan", post(directory_plan))
        .route("/api/directory-plans/{id}", get(directory_plan_get))
        .route("/api/directory-plans/{id}/apply", post(directory_apply))
        .route("/api/sources", get(source_list).post(source_put))
        .route("/api/sources/{id}/start", post(source_start))
        .route("/api/source-login/finish", post(source_finish))
        .route("/api/source-links", get(source_links))
        .route(
            "/api/source-links/{id}",
            axum::routing::delete(source_unlink),
        )
        .route("/oauth/sources/{id}/callback", get(source_callback))
        .route("/api/logout", post(logout))
        .route("/api/me", get(me))
        .route("/api/me/agents", get(my_agents).post(prepare_my_agent))
        .route(
            "/api/agent-self-service",
            get(agent_self_service).put(set_agent_self_service),
        )
        .route(
            "/api/me/agents/proposals/{id}/approve",
            post(approve_my_agent),
        )
        .route(
            "/api/me/agents/{id}",
            axum::routing::delete(revoke_my_agent),
        )
        .route("/api/me/agents/{id}/rotate", post(rotate_my_agent))
        .route("/api/me/agents/{id}/activity", get(my_agent_activity))
        .route(
            "/api/me/agent-applications",
            get(available_agent_applications),
        )
        .route(
            "/api/me/agents/{id}/applications",
            get(my_agent_applications).post(approve_my_agent_application),
        )
        .route(
            "/api/me/agents/{id}/applications/{grant_id}",
            axum::routing::delete(revoke_my_agent_application),
        )
        .route("/api/sessions", get(sessions))
        .route("/api/sessions/{id}", axum::routing::delete(revoke_session))
        .route("/api/users", get(users).post(create_user))
        .route("/api/users/{username}", axum::routing::patch(update_user))
        .route("/api/users/{username}/profile", get(user_profile))
        .route("/api/users/{username}/sessions", get(user_sessions))
        .route("/api/users/{username}/consents", get(user_consents))
        .route(
            "/api/users/{username}/consents/{client_id}",
            axum::routing::delete(revoke_user_consent),
        )
        .route("/api/users/{username}/agents", get(user_agents))
        .route(
            "/api/users/{username}/delegated-grants",
            get(human_grants).put(set_human_grants),
        )
        .route(
            "/api/users/{username}/delegated-grants/changes",
            post(stage_human_grants),
        )
        .route("/api/delegated-grant-changes/{id}", get(human_grant_change))
        .route(
            "/api/delegated-grant-changes/{id}/approve",
            post(approve_human_grant_change),
        )
        .route(
            "/api/delegated-grant-changes/{id}/execute",
            post(execute_human_grant_change),
        )
        .route(
            "/api/delegated-grant-changes/{id}/cancel",
            post(cancel_human_grant_change),
        )
        .route(
            "/api/groups/{name}/membership-changes",
            post(stage_group_membership),
        )
        .route(
            "/api/group-membership-changes/{id}",
            get(group_membership_change),
        )
        .route(
            "/api/group-membership-changes/{id}/approve",
            post(approve_group_membership_change),
        )
        .route(
            "/api/group-membership-changes/{id}/execute",
            post(execute_group_membership_change),
        )
        .route(
            "/api/group-membership-changes/{id}/cancel",
            post(cancel_group_membership_change),
        )
        .route("/api/groups", get(groups).post(create_group))
        .route(
            "/api/groups/{name}/members/{username}",
            axum::routing::put(add_member).delete(remove_member),
        )
        .route("/api/clients", get(clients).post(create_client))
        .route("/api/client-creation-changes", post(stage_client_creation))
        .route(
            "/api/client-creation-changes/{id}",
            get(client_creation_change),
        )
        .route(
            "/api/client-creation-changes/{id}/approve",
            post(approve_client_creation_change),
        )
        .route(
            "/api/client-creation-changes/{id}/execute",
            post(execute_client_creation_change),
        )
        .route(
            "/api/client-creation-changes/{id}/cancel",
            post(cancel_client_creation_change),
        )
        .route(
            "/api/clients/{id}/endpoint-changes",
            post(stage_client_endpoint),
        )
        .route(
            "/api/client-endpoint-changes/{id}",
            get(client_endpoint_change),
        )
        .route(
            "/api/client-endpoint-changes/{id}/approve",
            post(approve_client_endpoint_change),
        )
        .route(
            "/api/client-endpoint-changes/{id}/execute",
            post(execute_client_endpoint_change),
        )
        .route(
            "/api/client-endpoint-changes/{id}/cancel",
            post(cancel_client_endpoint_change),
        )
        .route(
            "/api/clients/{id}/status-changes",
            post(stage_client_status),
        )
        .route("/api/client-status-changes/{id}", get(client_status_change))
        .route(
            "/api/client-status-changes/{id}/approve",
            post(approve_client_status_change),
        )
        .route(
            "/api/client-status-changes/{id}/execute",
            post(execute_client_status_change),
        )
        .route(
            "/api/client-status-changes/{id}/cancel",
            post(cancel_client_status_change),
        )
        .route(
            "/api/clients/{id}/policy-changes",
            post(stage_client_policy),
        )
        .route("/api/client-policy-changes/{id}", get(client_policy_change))
        .route(
            "/api/client-policy-changes/{id}/approve",
            post(approve_client_policy_change),
        )
        .route(
            "/api/client-policy-changes/{id}/execute",
            post(execute_client_policy_change),
        )
        .route(
            "/api/client-policy-changes/{id}/cancel",
            post(cancel_client_policy_change),
        )
        .route("/api/clients/{id}", axum::routing::patch(update_client))
        .route(
            "/api/clients/{id}/rotate-secret",
            post(rotate_client_secret),
        )
        .route("/api/mfa/enroll", post(mfa_begin))
        .route("/api/passkeys", get(passkeys))
        .route(
            "/api/passkeys/{id}",
            axum::routing::delete(passkey_remove).patch(passkey_rename),
        )
        .route(
            "/api/passkey/registration/start",
            post(passkey_register_start),
        )
        .route(
            "/api/passkey/registration/finish",
            post(passkey_register_finish),
        )
        .route(
            "/api/passkey/registration/cancel",
            post(passkey_register_cancel),
        )
        .route(
            "/api/passkey/authentication/start",
            post(passkey_login_start),
        )
        .route(
            "/api/passkey/authentication/finish",
            post(passkey_login_finish),
        )
        .route("/api/mfa/confirm", post(mfa_confirm))
        .route("/api/mfa/replace", post(mfa_replace))
        .route("/api/mfa/remove", post(mfa_remove))
        .route("/api/mfa/recovery-codes", post(recovery_codes))
        .route("/api/password", post(change_password))
        .route("/api/device/{code}", get(device_details))
        .route("/api/device/decision", post(device_decide))
        .route("/api/audit", get(audit_events))
        .route("/api/audit/review", get(audit_review))
        .route("/api/reports/users.csv", get(users_csv))
        .route("/api/reports/audit.csv", get(audit_csv))
        .route("/api/keys/rotate", post(rotate_key))
        .route("/api/keys", get(key_domains).post(configure_key))
        .route("/api/capabilities", get(capabilities))
        .route("/api/agents", get(list_agents).post(create_agent))
        .route("/api/agents/{id}", axum::routing::delete(revoke_agent))
        .route("/api/agents/{id}/rotate", post(rotate_agent))
        .route("/api/resources/{kind}/{name}", get(get_resource))
        .route("/api/consents", get(consents))
        .route("/api/consents/{id}", axum::routing::delete(revoke_consent))
        .route(
            "/api/state/plan",
            post(plan_state).layer(DefaultBodyLimit::max(2 * 1024 * 1024)),
        )
        .route(
            "/api/state/apply",
            post(apply_state).layer(DefaultBodyLimit::max(2 * 1024 * 1024)),
        )
        .route("/api/state/export", get(export_state))
        .route("/api/state/revision", get(revision))
        .route("/api/schema/{name}", get(schema))
        .route("/api/policy/explain", post(explain))
        .route("/api/policy/simulate", post(simulate_policy))
        .route("/api/state/plans/{id}", get(plan_status))
        .route("/api/operations/logout", get(logout_deliveries))
        .route("/api/operations/doctor", get(doctor))
        .route("/api/operations/storage", get(storage_allocation))
        .route(
            "/api/operations/reconciliation",
            get(reconciliation_diagnostics),
        )
        // Both editions: operations.read and provisioner.read are not platform-only.
        .route(
            "/api/operations/provisioning",
            get(provisioning_job_diagnostics),
        )
        .route(
            "/api/operations/provisioning/deactivations",
            get(provisioning_deactivation_diagnostics),
        )
        .route("/api/operations/metrics", get(metrics))
        .route("/api/operations/prometheus", get(prometheus))
        .route("/api/operations/backup", post(backup))
        .route("/api/operations/backup/stream", post(backup::stream))
        .route("/api/inventory/{kind}", get(inventory));
    #[cfg(feature = "platform")]
    let routes = routes
        .merge(platform_routes())
        .merge(crate::portal::access_review::routes());
    let routes = if browser_ui {
        routes
            .merge(crate::portal::http::browser_routes())
            .merge(crate::portal::admin::browser_routes())
            .merge(interaction::browser_routes())
    } else {
        routes
    };
    #[cfg(feature = "platform")]
    let routes = if browser_ui {
        routes.merge(crate::portal::access_review::browser_routes())
    } else {
        routes
    };
    let routes = routes
        .fallback(provider_discovery_path)
        .layer(DefaultBodyLimit::max(32 * 1024))
        .layer(middleware::from_fn_with_state(app.clone(), protect))
        .layer(middleware::from_fn_with_state(app.clone(), observe))
        .with_state(app.clone())
        .merge(
            Router::new()
                .route("/livez", get(probes::live))
                .route("/readyz", get(probes::ready))
                .route("/healthz", get(probes::ready))
                .layer(middleware::from_fn_with_state(app.clone(), observe))
                .with_state(app.clone()),
        );
    if prefix.is_empty() {
        routes
    } else {
        let discovery_app = app.clone();
        let prefixed = Router::new()
            .route(
                &format!("{prefix}/"),
                get(crate::portal::http::root)
                    .layer(middleware::from_fn_with_state(app.clone(), protect))
                    .layer(middleware::from_fn_with_state(app.clone(), observe))
                    .with_state(app.clone()),
            )
            .route(
                &format!("/.well-known/oauth-authorization-server{prefix}"),
                get(move || {
                    let app = discovery_app.clone();
                    async move { app.run(|core| core.discovery().map(Json)).await }
                }),
            );
        #[cfg(feature = "platform")]
        let prefixed = prefixed.route(
            &format!("/.well-known/ssf-configuration{prefix}"),
            get(move || {
                let document = ssf_document.clone();
                async move { Json(document) }
            }),
        );
        prefixed
            .nest(&prefix, routes)
            .fallback(move |request: Request| {
                let app = app.clone();
                async move { provider_discovery_path(State(app), request).await }
            })
    }
}

#[cfg(feature = "platform")]
fn platform_routes() -> Router<App> {
    Router::new()
        .merge(workflow::routes())
        .route(
            "/scim/v2/ServiceProviderConfig",
            get(|| async {
                crate::scim::response(
                    crate::scim::metadata("ServiceProviderConfig"),
                    StatusCode::OK,
                )
            }),
        )
        .route(
            "/scim/v2/ResourceTypes",
            get(|| async {
                crate::scim::response(crate::scim::metadata("ResourceTypes"), StatusCode::OK)
            }),
        )
        .route(
            "/scim/v2/Schemas",
            get(|| async {
                crate::scim::response(crate::scim::metadata("Schemas"), StatusCode::OK)
            }),
        )
        .route(
            "/scim/v2/Schemas/{id}",
            get(|Path(id): Path<String>| async move {
                crate::scim::response(crate::scim::metadata(&id), StatusCode::OK)
            }),
        )
        .route("/scim/v2/{kind}", get(scim_list).post(scim_create))
        .route("/scim/v2/{kind}/.search", post(scim_search))
        .route(
            "/scim/v2/{kind}/{id}",
            get(scim_get)
                .put(scim_replace)
                .patch(scim_patch)
                .delete(scim_delete),
        )
        .route("/.well-known/ssf-configuration", get(ssf_configuration))
        .route("/saml/sources/{id}/metadata", get(saml_source_metadata))
        .route(
            "/api/saml/sources/{id}/metadata",
            get(saml_source_metadata_json),
        )
        .route(
            "/saml/sources/{id}/acs",
            post(saml_source_acs).layer(DefaultBodyLimit::max(96 * 1024)),
        )
        .route("/saml/sources/{id}/return", get(saml_source_browser_return))
        .route("/saml/logout/{ticket}", get(saml_logout_next))
        .route("/saml/logout/{ticket}/status", get(saml_logout_status))
        .route(
            "/saml/sources/{id}/slo",
            get(saml_source_logout_redirect).post(saml_source_logout_post),
        )
        .route("/saml/{id}/metadata", get(saml_metadata))
        .route("/api/saml/{id}/metadata", get(saml_metadata_json))
        .route("/saml/{id}/sso", get(saml_redirect).post(saml_post))
        .route("/saml/{id}/init", get(saml_initiate))
        .route("/saml/resume/{id}", get(saml_resume))
        .route("/api/login/certificate", post(certificate_login))
        .route(
            "/api/certificates",
            get(client_certificates).post(client_certificate_bind),
        )
        .route(
            "/api/certificates/{id}",
            axum::routing::delete(client_certificate_revoke),
        )
        .route(
            "/api/radius/certificates",
            get(radius_certificates).post(radius_certificate_bind),
        )
        .route(
            "/api/radius/certificates/{id}",
            axum::routing::delete(radius_certificate_revoke),
        )
        .route(
            "/api/windows-devices",
            get(windows_devices).post(windows_device_enroll),
        )
        .route("/api/windows-devices/login", post(windows_device_login))
        .route(
            "/api/windows-devices/tickets/redeem",
            post(windows_ticket_redeem),
        )
        .route(
            "/api/windows-devices/offline/verify",
            post(windows_offline_verify),
        )
        .route(
            "/api/windows-devices/{id}",
            axum::routing::delete(windows_device_revoke),
        )
        .route("/api/workspace-directories", get(workspace_directories))
        .route("/api/workspace-directories/{id}/plan", post(workspace_plan))
        .route(
            "/api/workspace-directory-plans/{id}",
            get(workspace_plan_get),
        )
        .route(
            "/api/workspace-directory-plans/{id}/apply",
            post(workspace_apply),
        )
        .route("/api/entra-directories", get(entra_directories))
        .route("/api/entra-directories/{id}/plan", post(entra_plan))
        .route("/api/entra-directory-plans/{id}", get(entra_plan_get))
        .route("/api/entra-directory-plans/{id}/apply", post(entra_apply))
        .route(
            "/api/cloud-directories/{kind}/{id}/operations",
            get(cloud_operations),
        )
        .route(
            "/api/cloud-directories/{kind}/{id}/test-connection",
            post(cloud_test_connection),
        )
        .route(
            "/api/cloud-directories/{kind}/{id}/verify-credential",
            post(cloud_verify_credential),
        )
        .route(
            "/api/cloud-directories/{kind}/{id}/verify-controller",
            post(cloud_verify_controller),
        )
        .route(
            "/api/cloud-directories/{kind}/{id}/schedule",
            axum::routing::patch(cloud_schedule_update),
        )
        .route(
            "/oauth/source-stages/{id}/resume",
            get(source_stage_resume).post(source_stage_resume_post),
        )
        .route(
            "/oauth/source-stages/{id}/cancel",
            post(source_stage_cancel),
        )
        .merge(crate::portal::source_stage::routes())
        .route(
            "/api/offboard/jobs",
            get(offboard_jobs).post(offboard_schedule),
        )
        .route("/api/offboard/jobs/{id}", get(offboard_job))
        .route(
            "/api/offboard/jobs/{id}/reschedule",
            post(offboard_reschedule),
        )
        .route("/api/offboard/jobs/{id}/cancel", post(offboard_cancel))
        .route("/api/operations/offboarding", get(offboarding_diagnostics))
        .route(
            "/api/operations/offboarding/deactivations",
            get(offboarding_deactivation_diagnostics),
        )
        .route("/api/operations/ssf", get(ssf_delivery_diagnostics))
        .route(
            "/api/access/requests",
            get(access_requests).post(access_request_create),
        )
        .route("/api/access/requests/{id}/approve", post(access_approve))
        .route("/api/access/requests/{id}/deny", post(access_deny))
        .route("/api/access/grants", get(access_grants))
        .route("/api/access/grants/{id}/revoke", post(access_revoke))
        .route("/api/audit/map", get(audit_map))
        .route("/api/device-trust/challenge", post(device_trust_challenge))
        .route("/api/device-trust/verify", post(device_trust_verify))
        .route(
            "/api/ssf/streams",
            get(ssf_config_streams)
                .post(ssf_config_stream_create)
                .patch(ssf_config_stream_patch)
                .put(ssf_config_stream_put)
                .delete(ssf_config_stream_delete),
        )
        .route(
            "/api/ssf/admin/streams",
            get(ssf_streams).post(ssf_stream_create),
        )
        .route(
            "/api/ssf/admin/streams/{id}",
            axum::routing::delete(ssf_stream_delete),
        )
        .route(
            "/api/ssf/admin/streams/{id}/subjects",
            axum::routing::put(ssf_stream_subjects),
        )
        .route(
            "/api/ssf/events",
            post(ssf_events).layer(DefaultBodyLimit::disable()),
        )
        .route("/api/proxy/auth", get(proxy_auth))
        .route("/outpost/{id}/auth", get(outpost_auth))
        .route("/outpost/{id}/traefik", get(outpost_traefik))
        .route("/outpost/{id}/start", get(outpost_start))
        .route("/outpost/{id}/callback", get(outpost_callback))
        .route("/outpost/{id}/logout", post(outpost_logout))
}
