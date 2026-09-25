use super::*;

#[test]
fn last_administrator_is_preserved() {
    let f = Fixture::new();
    assert!(
        f.core
            .update_user(
                &f.admin,
                "admin",
                UserPatch {
                    enabled: Some(false),
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert!(
        f.core
            .update_user(
                &f.admin,
                "admin",
                UserPatch {
                    admin: Some(false),
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert!(f.core.me(&f.admin).is_ok());
}

#[test]
fn disable_then_enable_never_resurrects_user_or_client_tokens() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let tokens = f.tokens("app", &alice, None);
    for enabled in [false, true] {
        f.core
            .update_user(
                &f.admin,
                "alice",
                UserPatch {
                    enabled: Some(enabled),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    let alice = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let tokens = f.tokens("app", &alice, None);
    for enabled in [false, true] {
        f.core
            .update_client(
                &f.admin,
                "app",
                ClientPatch {
                    enabled: Some(enabled),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
}

#[test]
fn terminal_browser_handoff_silent_consent_and_rp_logout_are_bound_to_sessions() {
    let f = Fixture::new();
    f.client("app", false);
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(riauth::model::ProviderSettings {
                    post_logout_redirect_uris: vec!["https://app.example.test/signed-out".into()],
                    backchannel_logout_uri: Some("https://app.example.test/backchannel".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let verifier = crypto::random_token("");
    let start = f
        .core
        .browser_start(f.request("app", &verifier), None)
        .unwrap();
    let code = text(&start.body, "user_code");
    let path = start
        .refresh
        .unwrap()
        .split_once("url=")
        .unwrap()
        .1
        .to_owned();
    let id = path.rsplit('/').next().unwrap();
    let binding = start.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    assert!(f.core.browser_resume(id, Some("wrong-browser")).is_err());
    let details = f.core.browser_details(&f.admin, &code).unwrap();
    f.core
        .browser_decide(
            &f.admin,
            riauth::browser::BrowserDecision {
                code: code.clone(),
                approve: true,
                transaction_id: Some(text(&details, "transaction_id")),
                remember: true,
            },
        )
        .unwrap();
    let returned = f.core.browser_resume(id, Some(binding)).unwrap();
    let callback = url::Url::parse(&returned.location.unwrap()).unwrap();
    let query: std::collections::BTreeMap<_, _> = callback.query_pairs().collect();
    assert_eq!(query["state"], "state with & delimiters");
    let tokens = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(query["code"].to_string()),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    let sso_cookie = returned.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    let mut silent = f.request("app", &crypto::random_token(""));
    silent.prompt = Some("none".into());
    assert!(
        f.core
            .browser_start(silent.clone(), Some(sso_cookie))
            .unwrap()
            .location
            .is_some()
    );
    assert!(f.core.browser_start(silent.clone(), None).is_err());
    let logout = || riauth::logout::LogoutRequest {
        id_token_hint: Some(text(&tokens, "id_token")),
        client_id: Some("app".into()),
        post_logout_redirect_uri: Some("https://app.example.test/signed-out".into()),
        state: Some("logout-state".into()),
    };
    let mut bad = logout();
    bad.post_logout_redirect_uri = Some("https://attacker.invalid/".into());
    assert!(f.core.end_session(bad, Some(sso_cookie), None).is_err());
    assert_eq!(
        f.core.end_session(logout(), None, None).unwrap()["interaction_required"],
        true
    );
    let result = f
        .core
        .end_session(logout(), Some(sso_cookie), None)
        .unwrap();
    assert_eq!(
        result["redirect_uri"],
        "https://app.example.test/signed-out?state=logout-state"
    );
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(f.core.browser_start(silent, Some(sso_cookie)).is_err());
    let ready = f.core.claim_logout_deliveries().unwrap();
    assert_eq!(ready.len(), 1);
    let (delivery, jwt) = &ready[0];
    let claims: Value = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(jwt.split('.').nth(1).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(claims["aud"], "app");
    assert!(claims["events"]["http://schemas.openid.net/event/backchannel-logout"].is_object());
    assert_eq!(claims["sid"], delivery.sid);
    assert!(claims.get("nonce").is_none());
    assert!(f.core.claim_logout_deliveries().unwrap().is_empty());
    f.core
        .finish_logout_delivery(&delivery.id, delivery.attempts, Some(204))
        .unwrap();
    assert!(f.core.claim_logout_deliveries().unwrap().is_empty());
}

#[test]
fn consent_revocation_is_confined_to_the_users_application() {
    let f = Fixture::new();
    f.client("app", false);
    f.client("other", false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    let app = f.tokens("app", &alice, None);
    let other = f.tokens("other", &alice, None);
    let bob_app = f.tokens("app", &bob, None);
    let unredeemed = f.exchange_request("app", &alice, None);
    f.core.revoke_consent(&alice, "app").unwrap();
    assert!(f.core.userinfo(&text(&app, "access_token")).is_err());
    assert!(f.core.token(unredeemed).is_err());
    assert!(f.core.userinfo(&text(&other, "access_token")).is_ok());
    assert!(f.core.userinfo(&text(&bob_app, "access_token")).is_ok());
    assert!(f.core.me(&alice).is_ok());
}

#[test]
fn unknown_policy_dependencies_and_user_policy_on_service_clients_are_rejected() {
    let f = Fixture::new();
    f.client("app", false);
    let mut settings = ProviderSettings::default();
    settings
        .policy
        .access
        .denied_groups
        .insert("missing".into());
    assert!(
        f.core
            .update_client(
                &f.admin,
                "app",
                ClientPatch {
                    settings: Some(settings.clone()),
                    ..Default::default()
                }
            )
            .is_err()
    );
    f.core.create_group(&f.admin, "missing").unwrap();
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(settings.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    settings.policy.access.denied_users.insert("unknown".into());
    assert!(
        f.core
            .update_client(
                &f.admin,
                "app",
                ClientPatch {
                    settings: Some(settings.clone()),
                    ..Default::default()
                }
            )
            .is_err()
    );
    settings.policy.access.denied_users.clear();
    assert!(
        f.core
            .create_client(
                &f.admin,
                NewClient {
                    client_id: "job".into(),
                    name: "Job".into(),
                    confidential: true,
                    service: true,
                    redirect_uris: vec![],
                    scopes: strings(&["api"]),
                    allowed_groups: Default::default(),
                    require_mfa: false,
                    settings
                }
            )
            .is_err()
    );
}

#[test]
fn requested_claims_require_consent_scopes_and_enforce_essential_values() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.scope = "openid".into();
    request.claims = Some(json!({"userinfo":{"email":{"essential":true,"value":"alice@example.test"}},"id_token":{"acr":{"essential":true,"values":[riauth::assurance::PASSWORD]}}}).to_string());
    let details = f
        .core
        .authorization_prepare(Some(&alice), request.clone())
        .unwrap();
    assert!(
        details["scopes"]
            .as_array()
            .unwrap()
            .contains(&json!("email"))
    );
    let mut wrong = request.clone();
    wrong.claims = Some(
        json!({"userinfo":{"email":{"essential":true,"value":"bob@example.test"}}}).to_string(),
    );
    assert_eq!(
        f.core.authorize(&alice, wrong).unwrap_err().code,
        "unmet_authentication_requirements"
    );
    let callback = f.core.authorize(&alice, request).unwrap();
    let code = url::Url::parse(&callback)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    let out = f
        .core
        .token(TokenRequest {
            client_id: Some("app".into()),
            grant_type: "authorization_code".into(),
            code: Some(code),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        f.core.userinfo(&text(&out, "access_token")).unwrap()["email"],
        "alice@example.test"
    );
    let id = fixture_jwks(&f)
        .verify(&text(&out, "id_token"), &f.core.config.issuer, "app")
        .unwrap();
    assert_eq!(id["acr"], riauth::assurance::PASSWORD);
    let mut step_up = f.request("app", &crypto::random_token(""));
    step_up.acr_values = Some(riauth::assurance::MFA.into());
    assert!(
        f.core
            .authorization_prepare(Some(&alice), step_up.clone())
            .unwrap()["reauthentication_required"]
            == true
    );
    assert_eq!(
        f.core.authorize(&alice, step_up).unwrap_err().code,
        "login_required"
    );
}

#[test]
fn default_assurance_and_userinfo_essential_acr_trigger_step_up_and_signed_requests_bind_semantics()
{
    let f = Fixture::new();
    f.client("step-up", false);
    let user = f.user("assurance-user");
    let settings = ProviderSettings {
        default_acr_values: vec![riauth::assurance::MFA.into()],
        ..Default::default()
    };
    f.core
        .update_client(
            &f.admin,
            "step-up",
            ClientPatch {
                settings: Some(settings),
                ..Default::default()
            },
        )
        .unwrap();
    let request = f.request("step-up", &crypto::random_token(""));
    assert_eq!(
        f.core
            .authorization_prepare(Some(&user), request.clone())
            .unwrap()["reauthentication_required"],
        true
    );
    f.core
        .update_client(
            &f.admin,
            "step-up",
            ClientPatch {
                settings: Some(Default::default()),
                ..Default::default()
            },
        )
        .unwrap();
    let mut essential = request;
    essential.claims = Some(
        json!({"userinfo":{"acr":{"essential":true,"value":riauth::assurance::MFA}}}).to_string(),
    );
    assert_eq!(
        f.core
            .authorization_prepare(Some(&user), essential)
            .unwrap()["reauthentication_required"],
        true
    );
    f.core
        .update_client(
            &f.admin,
            "step-up",
            ClientPatch {
                settings: Some(ProviderSettings {
                    jwks: Some(fixture_jwks(&f)),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let mut claims = serde_json::to_value(f.request("step-up", &crypto::random_token(""))).unwrap();
    claims.as_object_mut().unwrap().retain(|_, v| !v.is_null());
    claims.as_object_mut().unwrap().remove("decision");
    claims["iss"] = json!("step-up");
    claims["aud"] = json!(f.core.config.issuer);
    claims["iat"] = json!(now());
    claims["exp"] = json!(now() + 120);
    claims["jti"] = json!(crypto::id());
    let signed = fixture_signing_key(&f)
        .sign_type(&claims, "oauth-authz-req+jwt")
        .unwrap();
    let mut parsed = f
        .core
        .resolve_authorization(vec![
            ("client_id".into(), "step-up".into()),
            ("request".into(), signed),
        ])
        .unwrap();
    parsed.scope = "openid".into();
    assert_eq!(
        f.core
            .authorization_prepare(Some(&user), parsed)
            .unwrap_err()
            .code,
        "invalid_request_object"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs()
 {
    use riauth::agent::{NewAgent, Permission};
    let mut source = Fixture::new();
    let destination = Fixture::new();
    source.core.create_group(&source.admin, "staff").unwrap();
    let _user = source.user("provisioned");
    source
        .core
        .group_member(&source.admin, "staff", "provisioned", true)
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/scim/v2", listener.local_addr().unwrap());
    let core = destination.core.clone();
    let server = tokio::spawn(async move {
        axum::serve(listener, riauth::api::router(core))
            .await
            .unwrap()
    });
    let dir = tempfile::TempDir::new().unwrap();
    let token_file = dir.path().join("scim-token");
    riauth::config::write_private(&token_file, destination.admin.as_bytes(), false).unwrap();
    source.core.config.scim_targets.insert(
        "directory".into(),
        riauth::provisioning::Target {
            url,
            token_file: Some(token_file),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: true,
        },
    );
    let credential = source
        .core
        .create_agent(
            &source.admin,
            NewAgent {
                id: "provisioner".into(),
                permissions: vec![
                    Permission {
                        action: "provisioner.read".into(),
                        resource: "provisioner/directory".into(),
                    },
                    Permission {
                        action: "provisioner.sync".into(),
                        resource: "provisioner/directory".into(),
                    },
                ],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let agent = text(&credential["credential"], "token");
    assert_eq!(
        source
            .core
            .provisioning_targets(&agent)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let plan = source.core.provisioning_plan(&agent, "directory").unwrap();
    assert_eq!(plan["resources"].as_array().unwrap().len(), 2);
    assert!(!plan.to_string().contains(&destination.admin));
    assert!(
        source
            .core
            .provisioning_apply(&source.admin, &text(&plan, "id"))
            .is_err()
    );
    source
        .core
        .provisioning_apply(&agent, &text(&plan, "id"))
        .unwrap();
    for _ in 0..2 {
        let core = source.core.clone();
        tokio::task::spawn_blocking(move || core.provisioning_step())
            .await
            .unwrap()
            .unwrap();
    }
    assert_eq!(
        source.core.provisioning_jobs(&agent).unwrap()[0]["completed"],
        true,
        "{}",
        source.core.provisioning_jobs(&agent).unwrap()
    );
    let users = destination
        .core
        .scim_list(&destination.admin, "Users", Default::default())
        .unwrap();
    assert_eq!(users["totalResults"], 1);
    let id = text(&users["Resources"][0], "id");
    assert_eq!(users["Resources"][0]["active"], true);
    let groups = destination
        .core
        .scim_list(&destination.admin, "Groups", Default::default())
        .unwrap();
    assert_eq!(groups["Resources"][0]["members"][0]["value"], id);
    // A remote field outside the managed mapping survives a conditional patch.
    destination.core.scim_write(&destination.admin,"Users",Some(&id),json!({"schemas":[riauth::scim::USER],"userName":"provisioned","externalId":users["Resources"][0]["externalId"],"active":true,"name":{"givenName":"Remote-owned name"},"emails":[{"value":"provisioned@example.test","primary":true}]}),false).unwrap();
    source
        .core
        .update_user(
            &source.admin,
            "provisioned",
            UserPatch {
                display_name: Some("Updated Name".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let plan = source.core.provisioning_plan(&agent, "directory").unwrap();
    source
        .core
        .provisioning_apply(&agent, &text(&plan, "id"))
        .unwrap();
    for _ in 0..2 {
        let core = source.core.clone();
        tokio::task::spawn_blocking(move || core.provisioning_step())
            .await
            .unwrap()
            .unwrap();
    }
    let user = destination
        .core
        .scim_get(&destination.admin, "Users", &id)
        .unwrap();
    assert_eq!(user["displayName"], "Updated Name");
    assert_eq!(user["name"]["givenName"], "Remote-owned name");
    source
        .core
        .group_member(&source.admin, "staff", "provisioned", false)
        .unwrap();
    let plan = source.core.provisioning_plan(&agent, "directory").unwrap();
    source
        .core
        .provisioning_apply(&agent, &text(&plan, "id"))
        .unwrap();
    for _ in 0..2 {
        let core = source.core.clone();
        tokio::task::spawn_blocking(move || core.provisioning_step())
            .await
            .unwrap()
            .unwrap();
    }
    assert_eq!(
        destination
            .core
            .scim_get(&destination.admin, "Users", &id)
            .unwrap()["active"],
        false
    );
    let group = destination
        .core
        .scim_list(&destination.admin, "Groups", Default::default())
        .unwrap();
    assert_eq!(group["Resources"][0]["members"], json!([]));
    let plan = source.core.provisioning_plan(&agent, "directory").unwrap();
    source
        .core
        .provisioning_apply(&agent, &text(&plan, "id"))
        .unwrap();
    source
        .core
        .revoke_agent(&source.admin, "provisioner")
        .unwrap();
    let core = source.core.clone();
    tokio::task::spawn_blocking(move || core.provisioning_step())
        .await
        .unwrap()
        .unwrap();
    let jobs = source.core.provisioning_jobs(&source.admin).unwrap();
    assert_eq!(
        jobs.as_array()
            .unwrap()
            .iter()
            .find(|j| j["id"] == plan["id"])
            .unwrap()["stale"],
        true
    );
    server.abort();
}

#[test]
fn forward_auth_outpost_binds_browser_origin_policy_and_parent_session() {
    use axum::http::{HeaderMap, HeaderValue, StatusCode};
    let mut f = Fixture::new();
    let peer = "127.0.0.1".parse().unwrap();
    f.core.config.trusted_proxies = vec![peer];
    let alice = f.user("proxy-user");
    f.core.create_group(&f.admin, "staff").unwrap();
    f.core
        .group_member(&f.admin, "staff", "proxy-user", true)
        .unwrap();
    let origin = "http://localhost:7778";
    let settings = riauth::outpost::Settings {
        domain: None,
        external_origin: origin.into(),
        session_ttl: 3600,
    };
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "proxy".into(),
                name: "Protected app".into(),
                confidential: false,
                redirect_uris: vec![settings.callback("proxy")],
                scopes: strings(&["openid", "profile", "email", "groups"]),
                allowed_groups: strings(&["staff"]),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    proxy: Some(settings),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    assert!(
        f.core
            .outpost_start("proxy", "192.0.2.1".parse().unwrap(), origin)
            .is_err()
    );
    for target in [
        "https://attacker.test",
        "http://localhost:7778@attacker.test",
        "http://localhost:7778/outpost/proxy/logout",
        "http://localhost:7778\\@attacker.test",
    ] {
        assert!(f.core.outpost_start("proxy", peer, target).is_err());
    }
    let start = f
        .core
        .outpost_start("proxy", peer, &format!("{origin}/reports?q=test"))
        .unwrap();
    let location = url::Url::parse(start.location.as_ref().unwrap()).unwrap();
    let mut request: Authorization = serde_urlencoded::from_str(location.query().unwrap()).unwrap();
    request.decision = Some("approve".into());
    let callback = f.core.authorize(&alice, request).unwrap();
    let pairs = url::Url::parse(&callback)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect::<Vec<_>>();
    let mut headers = HeaderMap::new();
    assert!(
        f.core
            .outpost_callback("proxy", peer, &headers, pairs.clone())
            .is_err()
    );
    headers.insert(
        "cookie",
        HeaderValue::from_str(start.cookies[0].split(';').next().unwrap()).unwrap(),
    );
    let mut wrong = pairs.clone();
    wrong.iter_mut().find(|(k, _)| k == "iss").unwrap().1 = "https://attacker.test".into();
    assert!(
        f.core
            .outpost_callback("proxy", peer, &headers, wrong)
            .is_err()
    );
    let logged = f
        .core
        .outpost_callback("proxy", peer, &headers, pairs.clone())
        .unwrap();
    assert_eq!(
        logged.location.as_deref(),
        Some("http://localhost:7778/reports?q=test")
    );
    assert!(
        f.core
            .outpost_callback("proxy", peer, &headers, pairs)
            .is_err()
    );
    assert!(logged.cookies[0].contains("HttpOnly; SameSite=Lax"));
    assert!(!logged.cookies[0].contains("ri_access_"));
    headers.insert(
        "cookie",
        HeaderValue::from_str(logged.cookies[0].split(';').next().unwrap()).unwrap(),
    );
    headers.insert(
        "x-original-url",
        HeaderValue::from_static("http://localhost:7778/reports"),
    );
    headers.insert("x-authentik-username", HeaderValue::from_static("admin"));
    let (body, output) = f.core.outpost_auth("proxy", peer, &headers).unwrap();
    assert_eq!(body["username"], "proxy-user");
    assert_eq!(output["x-authentik-username"], "proxy-user");
    assert_eq!(output["x-authentik-groups"], "staff");
    headers.insert(
        "x-original-url",
        HeaderValue::from_static("http://attacker.test/reports"),
    );
    assert!(f.core.outpost_auth("proxy", peer, &headers).is_err());
    headers.insert(
        "x-original-url",
        HeaderValue::from_static("http://localhost:7778/reports"),
    );
    f.core
        .group_member(&f.admin, "staff", "proxy-user", false)
        .unwrap();
    assert!(f.core.outpost_auth("proxy", peer, &headers).is_err());
    f.core
        .group_member(&f.admin, "staff", "proxy-user", true)
        .unwrap();
    assert!(f.core.outpost_logout("proxy", peer, &headers).is_err());
    headers.insert("origin", HeaderValue::from_static("http://attacker.test"));
    assert!(f.core.outpost_logout("proxy", peer, &headers).is_err());
    assert!(f.core.outpost_auth("proxy", peer, &headers).is_ok());
    headers.insert("origin", HeaderValue::from_static("http://localhost:7778"));
    assert!(
        f.core
            .outpost_logout("proxy", peer, &headers)
            .is_err_and(|error| error.status == StatusCode::FORBIDDEN)
    );
    assert!(f.core.outpost_auth("proxy", peer, &headers).is_ok());
    f.core.logout(&alice).unwrap();
    assert!(f.core.outpost_auth("proxy", peer, &headers).is_err());
    headers.insert(
        "x-original-url",
        HeaderValue::from_static("http://localhost:7778/outpost/proxy/logout"),
    );
    assert!(
        f.core
            .outpost_logout("proxy", peer, &headers)
            .unwrap()
            .cookies[0]
            .contains("Max-Age=0")
    );
}

#[test]
fn group_policy_is_checked_again_at_refresh_userinfo_and_proxy() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    f.core.create_group(&f.admin, "developers").unwrap();
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                allowed_groups: Some(strings(&["developers"])),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        f.core
            .authorize(&alice, f.request("app", &crypto::random_token("")))
            .is_err()
    );
    f.core
        .group_member(&f.admin, "developers", "alice", true)
        .unwrap();
    let tokens = f.tokens("app", &alice, None);
    assert_eq!(
        f.core.userinfo(&text(&tokens, "access_token")).unwrap()["groups"],
        json!(["developers"])
    );
    assert!(
        f.core
            .proxy_auth(&text(&tokens, "access_token"), "wrong-audience")
            .is_err()
    );
    assert!(
        f.core
            .proxy_auth(&text(&tokens, "access_token"), "app")
            .is_ok()
    );
    f.core
        .group_member(&f.admin, "developers", "alice", false)
        .unwrap();
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(
        f.core
            .proxy_auth(&text(&tokens, "access_token"), "app")
            .is_err()
    );
    assert!(
        f.core
            .token(TokenRequest {
                grant_type: "refresh_token".into(),
                client_id: Some("app".into()),
                refresh_token: Some(text(&tokens, "refresh_token")),
                ..Default::default()
            })
            .is_err()
    );
}

#[test]
fn updated_default_acr_rejects_userinfo_introspection_and_proxy_consistently() {
    let f = Fixture::new();
    let secret = f.client("app", true);
    let alice = f.user("alice");
    // tokens() authorizes without explicit acr_values from a password-only session.
    assert!(
        f.request("app", &crypto::random_token(""))
            .acr_values
            .is_none()
    );
    let tokens = f.tokens("app", &alice, secret.clone());
    let access = text(&tokens, "access_token");
    let inspect = TokenRequest {
        client_id: Some("app".into()),
        client_secret: secret,
        token: Some(access.clone()),
        ..Default::default()
    };
    assert!(f.core.userinfo(&access).is_ok());
    assert_eq!(f.core.introspect(inspect.clone()).unwrap()["active"], true);
    assert!(f.core.proxy_auth(&access, "app").is_ok());
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    default_acr_values: vec![riauth::assurance::MFA.into()],
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        f.core.userinfo(&access).unwrap_err().code,
        "unmet_authentication_requirements"
    );
    assert_eq!(f.core.introspect(inspect).unwrap()["active"], false);
    assert!(f.core.proxy_auth(&access, "app").is_err());
}

#[test]
fn non_admins_cannot_manage_identities_and_views_never_return_secrets() {
    let f = Fixture::new();
    let alice = f.user("alice");
    assert_eq!(f.core.list_users(&alice).unwrap_err().code, "access_denied");
    assert_eq!(
        f.core
            .update_user(
                &alice,
                "alice",
                UserPatch {
                    admin: Some(true),
                    ..Default::default()
                }
            )
            .unwrap_err()
            .code,
        "access_denied"
    );
    let view = f.core.list_users(&f.admin).unwrap().to_string();
    assert!(!view.contains("password_hash"));
    assert!(!view.contains(PASSWORD));
    assert!(!view.contains("totp_secret"));
    let secret = f.client("app", true).unwrap();
    let clients = f.core.list_clients(&f.admin).unwrap().to_string();
    assert!(!clients.contains(&secret));
    assert!(!clients.contains("secret_hash"));
    let audit = f.core.audit_events(&f.admin, 100).unwrap().to_string();
    assert!(!audit.contains(&secret));
    assert!(!audit.contains(&f.admin));
    assert!(!audit.contains(PASSWORD));
}
