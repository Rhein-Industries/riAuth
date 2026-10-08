//! An owner approves one of their agents for one application. The agent then
//! exchanges its credential for an access token that stands for the owner,
//! and every use of that token rechecks the approval, the agent, the owner and
//! the application. Approvals are not management permissions.
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD, strings, text};
use riauth::{
    agent::{AgentProposalInput, ApplicationAccessInput, NewAgent, Permission},
    crypto,
    exchange::{ACCESS_TOKEN, AGENT_TOKEN, ExchangePolicy, TOKEN_EXCHANGE},
    model::{Client, ClientPatch, NewClient, ProviderSettings, UserPatch},
    oidc::TokenRequest,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use tower::ServiceExt;

const JMAP: &str = "jmap";
const RESOURCE: &str = "https://mail.example.test/jmap";

fn permissions(pairs: &[(&str, &str)]) -> Vec<Permission> {
    pairs
        .iter()
        .map(|(action, resource)| Permission {
            action: (*action).into(),
            resource: (*resource).into(),
        })
        .collect()
}

/// An agent prepared and approved by the person behind `session`.
fn agent(f: &Fixture, session: &str, id: &str, ttl: u64) -> String {
    let prepared = f
        .core
        .prepare_my_agent(
            session,
            AgentProposalInput {
                id: id.into(),
                permissions: permissions(&[("profile.read", "self")]),
                ttl,
            },
        )
        .unwrap();
    let approved = f
        .core
        .approve_my_agent(
            session,
            &text(&prepared, "proposal_id"),
            &text(&prepared, "digest"),
        )
        .unwrap();
    text(&approved["credential"], "token")
}

/// A confidential mail application that accepts agents; returns its secret.
fn application(f: &Fixture, cid: &str, adjust: impl FnOnce(&mut NewClient)) -> String {
    let mut input = NewClient {
        client_id: cid.into(),
        name: cid.into(),
        confidential: true,
        redirect_uris: vec!["http://localhost:7777/callback?existing=1".into()],
        scopes: strings(&[
            "openid",
            "profile",
            "email",
            "groups",
            "offline_access",
            "mail",
            "contacts",
        ]),
        allowed_groups: Default::default(),
        require_mfa: false,
        service: false,
        settings: ProviderSettings {
            agent_access: true,
            access_token_ttl: Some(3600),
            resources: BTreeMap::from([(RESOURCE.to_owned(), strings(&["mail"]))]),
            ..Default::default()
        },
    };
    adjust(&mut input);
    text(
        &f.core.create_client(&f.admin, input).unwrap(),
        "client_secret",
    )
}

fn access(client_id: &str, scopes: &[&str]) -> ApplicationAccessInput {
    ApplicationAccessInput {
        client_id: client_id.into(),
        scopes: strings(scopes),
        resource: None,
        ttl: None,
    }
}

fn exchange(agent_token: &str, audience: &str) -> TokenRequest {
    TokenRequest {
        grant_type: TOKEN_EXCHANGE.into(),
        subject_token: Some(agent_token.into()),
        subject_token_type: Some(AGENT_TOKEN.into()),
        audience: Some(audience.into()),
        ..Default::default()
    }
}

fn introspect(f: &Fixture, cid: &str, secret: &str, token: &str) -> Value {
    f.core
        .introspect(TokenRequest {
            client_id: Some(cid.into()),
            client_secret: Some(secret.into()),
            token: Some(token.into()),
            ..Default::default()
        })
        .unwrap()
}

fn verified(f: &Fixture, token: &str, audience: &str) -> Value {
    let jwks: riauth::jose::PublicJwks = serde_json::from_value(f.core.jwks().unwrap()).unwrap();
    jwks.verify(token, &f.core.config.issuer, audience).unwrap()
}

fn user_id(f: &Fixture, session: &str) -> String {
    text(&f.core.me(session).unwrap()["user"], "id")
}

/// A live application token passes validation; userinfo then asks for openid.
fn assert_live(f: &Fixture, secret: &str, token: &str) {
    assert_eq!(introspect(f, JMAP, secret, token)["active"], true);
    assert_eq!(
        f.core.userinfo(token).unwrap_err().code,
        "insufficient_scope"
    );
}

fn assert_ended(f: &Fixture, secret: &str, token: &str) {
    assert_eq!(introspect(f, JMAP, secret, token)["active"], false);
    assert_eq!(f.core.userinfo(token).unwrap_err().code, "invalid_token");
}

#[test]
fn approval_needs_ownership_a_live_agent_an_opted_in_application_and_fresh_sign_in() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let other = f.user("other");
    let owner_id = user_id(&f, &owner);
    let token = agent(&f, &owner, "mail-helper", 3600);
    f.core.create_group(&f.admin, "staff").unwrap();
    application(&f, JMAP, |_| {});
    application(&f, "plain", |client| client.settings.agent_access = false);
    application(&f, "staff-mail", |client| {
        client.allowed_groups = strings(&["staff"]);
    });

    // Only the owner, with a session, reaches the agent.
    assert_eq!(
        f.core
            .approve_my_agent_application(&other, "mail-helper", access(JMAP, &["mail"]))
            .unwrap_err()
            .code,
        "not_found"
    );
    assert_eq!(
        f.core
            .approve_my_agent_application(&token, "mail-helper", access(JMAP, &["mail"]))
            .unwrap_err()
            .code,
        "invalid_token"
    );
    // The application must exist and accept agents.
    for (cid, code) in [("missing", "not_found"), ("plain", "invalid_request")] {
        assert_eq!(
            f.core
                .approve_my_agent_application(&owner, "mail-helper", access(cid, &["mail"]))
                .unwrap_err()
                .code,
            code,
            "{cid}"
        );
    }
    // Scopes are a non-empty subset of the application's, never login or refresh scopes.
    for scopes in [
        vec![],
        vec!["openid", "mail"],
        vec!["offline_access"],
        vec!["bound_key"],
        vec!["calendar"],
    ] {
        assert_eq!(
            f.core
                .approve_my_agent_application(&owner, "mail-helper", access(JMAP, &scopes))
                .unwrap_err()
                .code,
            "invalid_scope",
            "{scopes:?}"
        );
    }
    // A resource must be registered for the application and cover the scopes.
    let mut unregistered = access(JMAP, &["mail"]);
    unregistered.resource = Some("https://other.example.test/jmap".into());
    let mut wider = access(JMAP, &["mail", "contacts"]);
    wider.resource = Some(RESOURCE.into());
    for (input, code) in [(unregistered, "invalid_target"), (wider, "invalid_scope")] {
        assert_eq!(
            f.core
                .approve_my_agent_application(&owner, "mail-helper", input)
                .unwrap_err()
                .code,
            code
        );
    }
    let mut ttl = access(JMAP, &["mail"]);
    ttl.ttl = Some(30);
    assert_eq!(
        f.core
            .approve_my_agent_application(&owner, "mail-helper", ttl)
            .unwrap_err()
            .code,
        "invalid_request"
    );
    // The owner must pass the application's own policy.
    assert_eq!(
        f.core
            .approve_my_agent_application(&owner, "mail-helper", access("staff-mail", &["mail"]))
            .unwrap_err()
            .code,
        "access_denied"
    );
    f.core
        .group_member(&f.admin, "staff", "owner", true)
        .unwrap();
    f.core
        .approve_my_agent_application(&owner, "mail-helper", access("staff-mail", &["mail"]))
        .unwrap();

    // Approval needs a fresh sign-in; revocation does not.
    crypto::with_test_time(crypto::now() + 600, || {
        assert_eq!(
            f.core
                .approve_my_agent_application(&owner, "mail-helper", access(JMAP, &["mail"]))
                .unwrap_err()
                .code,
            "reauthentication_required"
        );
    });
    let mut capped = access(JMAP, &["mail", "contacts"]);
    capped.ttl = Some(2_592_000);
    let approved = f
        .core
        .approve_my_agent_application(&owner, "mail-helper", capped)
        .unwrap();
    let agent_expiry = f.core.my_agents(&owner).unwrap()["agents"][0]["expires_at"].clone();
    assert_eq!(approved["expires_at"], agent_expiry);
    assert_eq!(approved["active"], true);
    assert_eq!(approved["scopes"], json!(["contacts", "mail"]));
    // One usable approval per application and resource.
    assert_eq!(
        f.core
            .approve_my_agent_application(&owner, "mail-helper", access(JMAP, &["mail"]))
            .unwrap_err()
            .code,
        "conflict"
    );
    let listed = f.core.my_agent_applications(&owner, "mail-helper").unwrap();
    assert_eq!(listed["applications"].as_array().unwrap().len(), 2);
    assert_eq!(
        f.core
            .my_agent_applications(&other, "mail-helper")
            .unwrap_err()
            .code,
        "not_found"
    );

    // An approval is not a management permission.
    let me = f.core.me(&token).unwrap();
    assert_eq!(
        me["permissions"],
        json!([{"action": "profile.read", "resource": "user/owner"}])
    );
    let audit = f.core.audit_events(&f.admin, 200).unwrap();
    let granted = audit
        .as_array()
        .unwrap()
        .iter()
        .find(|event| {
            event["action"] == "agent.application.grant" && event["details"]["client"] == JMAP
        })
        .unwrap();
    assert_eq!(granted["actor"], owner_id);
    assert_eq!(granted["target"], "mail-helper");
    assert_eq!(granted["details"]["agent"], "mail-helper");
    assert_eq!(granted["details"]["scopes"], json!(["contacts", "mail"]));
    assert!(!audit.to_string().contains(&token));

    // A revoked agent cannot be approved.
    f.core.revoke_my_agent(&owner, "mail-helper").unwrap();
    let fresh = text(
        &f.core.login("owner".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    assert_eq!(
        f.core
            .approve_my_agent_application(
                &fresh,
                "mail-helper",
                access("staff-mail", &["contacts"])
            )
            .unwrap_err()
            .code,
        "conflict"
    );
}

#[test]
fn exchange_issues_only_an_owner_access_token_naming_the_agent() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let token = agent(&f, &owner, "mail-helper", 7200);
    let secret = application(&f, JMAP, |client| {
        client.settings.pairwise_sector = Some("mail.example.test".into());
    });
    f.core
        .approve_my_agent_application(&owner, "mail-helper", access(JMAP, &["mail", "profile"]))
        .unwrap();
    let mut scoped = access(JMAP, &["mail"]);
    scoped.resource = Some(RESOURCE.into());
    scoped.ttl = Some(120);
    let approval = f
        .core
        .approve_my_agent_application(&owner, "mail-helper", scoped)
        .unwrap();

    let before = crypto::now();
    let issued = f.core.token(exchange(&token, JMAP)).unwrap();
    assert_eq!(issued["issued_token_type"], ACCESS_TOKEN);
    assert_eq!(issued["token_type"], "Bearer");
    assert_eq!(issued["scope"], "mail profile");
    assert!(issued.get("refresh_token").is_none() && issued.get("id_token").is_none());
    let at = text(&issued, "access_token");
    let claims = verified(&f, &at, JMAP);
    // The subject is the owner's subject for this application, as in its own tokens.
    let own = f.tokens(JMAP, &owner, Some(secret.clone()));
    let own_claims = verified(&f, &text(&own, "access_token"), JMAP);
    assert_eq!(claims["sub"], own_claims["sub"]);
    assert_ne!(claims["sub"], user_id(&f, &owner));
    assert_eq!(claims["aud"], JMAP);
    assert_eq!(claims["client_id"], JMAP);
    assert_eq!(claims["scope"], "mail profile");
    assert_eq!(
        claims["act"],
        json!({"sub": "agent:mail-helper", "iss": f.core.config.issuer})
    );
    let exp = claims["exp"].as_u64().unwrap();
    assert!((before + 3600..=crypto::now() + 3600).contains(&exp));
    let inspected = introspect(&f, JMAP, &secret, &at);
    assert_eq!(inspected["active"], true);
    assert_eq!(inspected["sub"], claims["sub"]);
    assert_eq!(inspected["client_id"], JMAP);
    assert_eq!(inspected["act"], claims["act"]);
    // There is no openid scope, so userinfo answers but refuses.
    assert_eq!(f.core.userinfo(&at).unwrap_err().code, "insufficient_scope");

    // A resource is named exactly as approved; expiry follows that approval.
    let mut request = exchange(&token, JMAP);
    request.resource = Some(RESOURCE.into());
    let narrow = f.core.token(request).unwrap();
    let narrow_claims = verified(&f, &text(&narrow, "access_token"), RESOURCE);
    assert_eq!(narrow_claims["aud"], RESOURCE);
    assert_eq!(narrow_claims["scope"], "mail");
    assert_eq!(narrow_claims["exp"], approval["expires_at"]);

    // Requested scopes stay within the approval.
    for (scope, code) in [
        ("mail contacts", "invalid_scope"),
        ("openid", "invalid_scope"),
        ("calendar", "invalid_scope"),
    ] {
        let mut request = exchange(&token, JMAP);
        request.scope = Some(scope.into());
        assert_eq!(f.core.token(request).unwrap_err().code, code, "{scope}");
    }
    let mut request = exchange(&token, JMAP);
    request.scope = Some("mail".into());
    assert_eq!(f.core.token(request).unwrap()["scope"], "mail");

    // The agent credential is the only authentication.
    let mut request = exchange(&token, JMAP);
    request.client_id = Some(JMAP.into());
    request.client_secret = Some(secret.clone());
    assert_eq!(f.core.token(request).unwrap_err().code, "invalid_request");
    let mut request = exchange(&token, JMAP);
    request.client_id = Some("plain".into());
    assert_eq!(f.core.token(request).unwrap_err().code, "invalid_request");
    let mut request = exchange(&token, JMAP);
    request.actor_token = Some(token.clone());
    request.actor_token_type = Some(ACCESS_TOKEN.into());
    assert_eq!(f.core.token(request).unwrap_err().code, "invalid_request");
    let mut request = exchange(&token, JMAP);
    request.audience = None;
    assert_eq!(f.core.token(request).unwrap_err().code, "invalid_target");

    // The application's own token revocation applies.
    f.core
        .revoke(TokenRequest {
            client_id: Some(JMAP.into()),
            client_secret: Some(secret.clone()),
            token: Some(at.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(introspect(&f, JMAP, &secret, &at)["active"], false);
    let audit = f.core.audit_events(&f.admin, 200).unwrap();
    assert!(audit.as_array().unwrap().iter().any(|event| {
        event["action"] == "token.exchanged"
            && event["actor"] == "agent:mail-helper"
            && event["target"] == JMAP
    }));
    assert!(!audit.to_string().contains(&token));
}

#[test]
fn dpop_bound_applications_require_a_proof_for_agent_tokens() {
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    let f = Fixture::new();
    let owner = f.user("owner");
    let token = agent(&f, &owner, "mail-helper", 3600);
    let secret = application(&f, JMAP, |client| {
        client.settings.dpop_bound_access_tokens = true;
    });
    f.core
        .approve_my_agent_application(&owner, "mail-helper", access(JMAP, &["mail"]))
        .unwrap();
    assert_eq!(
        f.core.token(exchange(&token, JMAP)).unwrap_err().code,
        "invalid_dpop_proof"
    );
    let key = crypto::SigningKey::generate_algorithm("ES256").unwrap();
    let mut header = Header::new(Algorithm::ES256);
    header.typ = Some("dpop+jwt".into());
    header.jwk = Some(serde_json::from_value(key.jwk().unwrap()).unwrap());
    let proof = encode(
        &header,
        &json!({"jti": crypto::id(), "iat": crypto::now(), "htm": "POST", "htu": format!("{}/oauth/token", f.core.config.issuer.trim_end_matches('/'))}),
        &EncodingKey::from_ec_pem(key.pem.as_bytes()).unwrap(),
    )
    .unwrap();
    let mut request = exchange(&token, JMAP);
    request.dpop_proof = Some(proof);
    let issued = f.core.token(request).unwrap();
    assert_eq!(issued["token_type"], "DPoP");
    let at = text(&issued, "access_token");
    assert!(verified(&f, &at, JMAP)["cnf"]["jkt"].is_string());
    assert!(introspect(&f, JMAP, &secret, &at)["cnf"]["jkt"].is_string());
}

#[test]
fn other_agents_and_other_owners_agents_get_nothing() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let other = f.user("other");
    let approved = agent(&f, &owner, "approved", 3600);
    let sibling = agent(&f, &owner, "sibling", 3600);
    let foreign = agent(&f, &other, "foreign", 3600);
    application(&f, JMAP, |_| {});
    application(&f, "calendar", |_| {});
    f.core
        .approve_my_agent_application(&owner, "approved", access(JMAP, &["mail"]))
        .unwrap();
    f.core
        .approve_my_agent_application(&other, "foreign", access("calendar", &["mail"]))
        .unwrap();

    assert!(f.core.token(exchange(&approved, JMAP)).is_ok());
    for (credential, audience) in [
        (sibling.as_str(), JMAP),
        (foreign.as_str(), JMAP),
        (approved.as_str(), "calendar"),
        ("ri_agent_not-a-credential", JMAP),
    ] {
        assert_eq!(
            f.core
                .token(exchange(credential, audience))
                .unwrap_err()
                .code,
            "invalid_grant",
            "{audience}"
        );
    }
    // The owner's session is not an agent credential.
    assert_eq!(
        f.core.token(exchange(&owner, JMAP)).unwrap_err().code,
        "invalid_grant"
    );
    // Another person cannot revoke or list this approval.
    let listed = f.core.my_agent_applications(&owner, "approved").unwrap();
    let approval_id = text(&listed["applications"][0], "id");
    assert_eq!(
        f.core
            .revoke_my_agent_application(&other, "approved", &approval_id)
            .unwrap_err()
            .code,
        "not_found"
    );
    assert_eq!(
        f.core
            .revoke_my_agent_application(&owner, "sibling", &approval_id)
            .unwrap_err()
            .code,
        "not_found"
    );
}

#[test]
fn management_permissions_confer_no_application_access() {
    let f = Fixture::new();
    let admin_id = user_id(&f, &f.admin);
    let secret = application(&f, JMAP, |_| {});
    application(&f, "later", |client| client.settings.agent_access = false);
    // An unowned agent and an administrator's own agent, both able to manage clients.
    let mut credentials = BTreeMap::new();
    for (id, parent) in [("manager", None), ("admin-helper", Some("admin"))] {
        let token = text(
            &f.core
                .create_agent(
                    &f.admin,
                    NewAgent {
                        id: id.into(),
                        permissions: permissions(&[("client.write", "*"), ("client.read", "*")]),
                        ttl: 3600,
                        parent: parent.map(str::to_owned),
                    },
                )
                .unwrap()["credential"],
            "token",
        );
        assert_eq!(
            f.core.token(exchange(&token, JMAP)).unwrap_err().code,
            "invalid_grant",
            "{id}"
        );
        // client.write may opt an application in, but approves nothing.
        f.core
            .update_client(
                &token,
                "later",
                ClientPatch {
                    settings: Some(ProviderSettings {
                        agent_access: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            f.core.token(exchange(&token, "later")).unwrap_err().code,
            "invalid_grant"
        );
        f.core
            .update_client(
                &f.admin,
                "later",
                ClientPatch {
                    settings: Some(ProviderSettings::default()),
                    ..Default::default()
                },
            )
            .unwrap();
        credentials.insert(id, token);
    }
    // The owner's approval is what grants access, and only to its owner's agent.
    f.core
        .approve_my_agent_application(&f.admin, "admin-helper", access(JMAP, &["mail"]))
        .unwrap();
    assert_eq!(
        f.core
            .token(exchange(&credentials["manager"], JMAP))
            .unwrap_err()
            .code,
        "invalid_grant"
    );
    let at = text(
        &f.core
            .token(exchange(&credentials["admin-helper"], JMAP))
            .unwrap(),
        "access_token",
    );
    assert_eq!(introspect(&f, JMAP, &secret, &at)["sub"], admin_id);
    // The approval changes nothing the agent may manage.
    assert_eq!(
        f.core.me(&credentials["admin-helper"]).unwrap()["permissions"],
        json!([
            {"action": "client.write", "resource": "*"},
            {"action": "client.read", "resource": "*"},
        ])
    );
    // The application token is no management credential.
    assert_eq!(f.core.list_clients(&at).unwrap_err().code, "invalid_token");
}

#[test]
fn revoking_the_approval_or_the_agent_ends_outstanding_tokens_at_once() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let secret = application(&f, JMAP, |_| {});
    let first = agent(&f, &owner, "first", 3600);
    let second = agent(&f, &owner, "second", 3600);
    let third = agent(&f, &owner, "third", 3600);
    for id in ["first", "second", "third"] {
        f.core
            .approve_my_agent_application(&owner, id, access(JMAP, &["mail"]))
            .unwrap();
    }
    let tokens: Vec<String> = [&first, &second, &third]
        .into_iter()
        .map(|credential| {
            text(
                &f.core.token(exchange(credential, JMAP)).unwrap(),
                "access_token",
            )
        })
        .collect();
    for token in &tokens {
        assert_live(&f, &secret, token);
    }

    // Revoking the approval needs no fresh sign-in and repeats safely.
    let approval_id = text(
        &f.core.my_agent_applications(&owner, "first").unwrap()["applications"][0],
        "id",
    );
    crypto::with_test_time(crypto::now() + 600, || {
        let revoked = f
            .core
            .revoke_my_agent_application(&owner, "first", &approval_id)
            .unwrap();
        assert_eq!(revoked["active"], false);
    });
    assert_ended(&f, &secret, &tokens[0]);
    assert_eq!(
        f.core.token(exchange(&first, JMAP)).unwrap_err().code,
        "invalid_grant"
    );
    f.core
        .revoke_my_agent_application(&owner, "first", &approval_id)
        .unwrap();
    assert!(
        f.core
            .audit_events(&f.admin, 200)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "agent.application.revoke")
            .count()
            == 1
    );

    // Revoking the agent ends its tokens.
    assert_live(&f, &secret, &tokens[1]);
    f.core.revoke_my_agent(&owner, "second").unwrap();
    assert_ended(&f, &secret, &tokens[1]);

    // Rotating the credential ends tokens obtained with the old one; the
    // approval stays and the new credential obtains a new token.
    let rotated = f.core.rotate_my_agent(&owner, "third", 3600).unwrap();
    assert_ended(&f, &secret, &tokens[2]);
    let renewed = text(
        &f.core
            .token(exchange(&text(&rotated["credential"], "token"), JMAP))
            .unwrap(),
        "access_token",
    );
    assert_live(&f, &secret, &renewed);
    assert_eq!(
        f.core.token(exchange(&third, JMAP)).unwrap_err().code,
        "invalid_grant"
    );
}

#[test]
fn expiry_ends_tokens_and_the_approval() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let secret = application(&f, JMAP, |_| {});
    let short = agent(&f, &owner, "short", 600);
    f.core
        .approve_my_agent_application(&owner, "short", access(JMAP, &["mail"]))
        .unwrap();
    let token = text(
        &f.core.token(exchange(&short, JMAP)).unwrap(),
        "access_token",
    );
    // The token never outlives the agent, even with a one-hour access TTL.
    let expires = verified(&f, &token, JMAP)["exp"].as_u64().unwrap();
    let agent_expiry = f.core.my_agents(&owner).unwrap()["agents"][0]["expires_at"]
        .as_u64()
        .unwrap();
    assert_eq!(expires, agent_expiry);
    crypto::with_test_time(agent_expiry + 1, || {
        assert_ended(&f, &secret, &token);
        assert_eq!(
            f.core.token(exchange(&short, JMAP)).unwrap_err().code,
            "invalid_grant"
        );
    });

    // An approval that expires before its agent ends tokens at its own expiry.
    let long = agent(&f, &owner, "long", 7200);
    let mut brief = access(JMAP, &["mail"]);
    brief.ttl = Some(120);
    let approval = f
        .core
        .approve_my_agent_application(&owner, "long", brief)
        .unwrap();
    let token = text(
        &f.core.token(exchange(&long, JMAP)).unwrap(),
        "access_token",
    );
    let ends = approval["expires_at"].as_u64().unwrap();
    crypto::with_test_time(ends, || {
        assert_ended(&f, &secret, &token);
        assert_eq!(
            f.core.token(exchange(&long, JMAP)).unwrap_err().code,
            "invalid_grant"
        );
        assert_eq!(
            f.core.my_agent_applications(&owner, "long").unwrap()["applications"][0]["active"],
            false
        );
    });
}

#[test]
fn disabling_or_promoting_the_owner_ends_tokens_and_revives_nothing() {
    for change in ["disable", "promote"] {
        let f = Fixture::new();
        let owner = f.user("owner");
        let secret = application(&f, JMAP, |_| {});
        let credential = agent(&f, &owner, "mail-helper", 3600);
        f.core
            .approve_my_agent_application(&owner, "mail-helper", access(JMAP, &["mail"]))
            .unwrap();
        let token = text(
            &f.core.token(exchange(&credential, JMAP)).unwrap(),
            "access_token",
        );
        assert_live(&f, &secret, &token);
        let (apply, restore) = match change {
            "disable" => (
                UserPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
                UserPatch {
                    enabled: Some(true),
                    ..Default::default()
                },
            ),
            _ => (
                UserPatch {
                    admin: Some(true),
                    ..Default::default()
                },
                UserPatch {
                    admin: Some(false),
                    ..Default::default()
                },
            ),
        };
        f.core.update_user(&f.admin, "owner", apply).unwrap();
        assert_ended(&f, &secret, &token);
        assert_eq!(
            f.core.token(exchange(&credential, JMAP)).unwrap_err().code,
            "invalid_grant",
            "{change}"
        );
        f.core.update_user(&f.admin, "owner", restore).unwrap();
        assert_ended(&f, &secret, &token);
        assert_eq!(
            f.core.token(exchange(&credential, JMAP)).unwrap_err().code,
            "invalid_grant",
            "{change}"
        );
    }
}

#[test]
fn the_application_policy_and_opt_in_are_checked_on_every_use() {
    let f = Fixture::new();
    let owner = f.user("owner");
    f.core.create_group(&f.admin, "staff").unwrap();
    f.core
        .group_member(&f.admin, "staff", "owner", true)
        .unwrap();
    let secret = application(&f, JMAP, |client| {
        client.allowed_groups = strings(&["staff"]);
    });
    let credential = agent(&f, &owner, "mail-helper", 3600);
    f.core
        .approve_my_agent_application(&owner, "mail-helper", access(JMAP, &["mail"]))
        .unwrap();
    let token = text(
        &f.core.token(exchange(&credential, JMAP)).unwrap(),
        "access_token",
    );
    assert_live(&f, &secret, &token);
    // Losing the application's group ends the token; regaining it restores access.
    f.core
        .group_member(&f.admin, "staff", "owner", false)
        .unwrap();
    assert_ended(&f, &secret, &token);
    assert_eq!(
        f.core.token(exchange(&credential, JMAP)).unwrap_err().code,
        "invalid_grant"
    );
    f.core
        .group_member(&f.admin, "staff", "owner", true)
        .unwrap();
    assert_live(&f, &secret, &token);

    // A scope the application no longer has is never issued, even if approved.
    let mut client: Client = f.core.store.get("clients", JMAP).unwrap().unwrap();
    let wide = access(JMAP, &["mail", "contacts"]);
    let second = agent(&f, &owner, "contacts-helper", 3600);
    f.core
        .approve_my_agent_application(&owner, "contacts-helper", wide)
        .unwrap();
    let both = text(
        &f.core.token(exchange(&second, JMAP)).unwrap(),
        "access_token",
    );
    assert_live(&f, &secret, &both);
    client.scopes.remove("contacts");
    f.core
        .update_client(
            &f.admin,
            JMAP,
            ClientPatch {
                scopes: Some(client.scopes.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(introspect(&f, JMAP, &secret, &both)["active"], false);
    assert_eq!(
        f.core.token(exchange(&second, JMAP)).unwrap_err().code,
        "invalid_scope"
    );
    let mut narrower = exchange(&second, JMAP);
    narrower.scope = Some("mail".into());
    assert_eq!(f.core.token(narrower).unwrap()["scope"], "mail");

    // Opting the application out ends tokens and approvals; opting in again revives neither.
    client.settings.agent_access = false;
    f.core
        .update_client(
            &f.admin,
            JMAP,
            ClientPatch {
                settings: Some(client.settings.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(introspect(&f, JMAP, &secret, &token)["active"], false);
    assert_eq!(
        f.core.token(exchange(&credential, JMAP)).unwrap_err().code,
        "invalid_grant"
    );
    client.settings.agent_access = true;
    f.core
        .update_client(
            &f.admin,
            JMAP,
            ClientPatch {
                settings: Some(client.settings.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(introspect(&f, JMAP, &secret, &token)["active"], false);
    assert_eq!(
        f.core.token(exchange(&credential, JMAP)).unwrap_err().code,
        "invalid_grant"
    );
    assert_eq!(
        f.core.my_agent_applications(&owner, "mail-helper").unwrap()["applications"][0]["active"],
        false
    );

    // Only user-facing clients without session-bound policy accept agents.
    let refused = f
        .core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "worker".into(),
                name: "worker".into(),
                confidential: true,
                redirect_uris: vec![],
                scopes: strings(&["mail"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: true,
                settings: ProviderSettings {
                    agent_access: true,
                    ..Default::default()
                },
            },
        )
        .unwrap_err();
    assert!(
        refused.message.contains("agent_access"),
        "{}",
        refused.message
    );
    // Proxy-client tokens never leave riAuth's outpost.
    #[cfg(feature = "platform")]
    {
        let proxy = riauth::outpost::Settings {
            domain: None,
            external_origin: "https://reports.example.test".into(),
            session_ttl: 3600,
        };
        let refused = f
            .core
            .create_client(
                &f.admin,
                NewClient {
                    client_id: "reports".into(),
                    name: "reports".into(),
                    confidential: false,
                    redirect_uris: vec![proxy.callback("reports")],
                    scopes: strings(&["openid", "profile"]),
                    allowed_groups: Default::default(),
                    require_mfa: false,
                    service: false,
                    settings: ProviderSettings {
                        proxy: Some(proxy),
                        agent_access: true,
                        ..Default::default()
                    },
                },
            )
            .unwrap_err();
        assert!(
            refused.message.contains("agent_access"),
            "{}",
            refused.message
        );
    }
    // The setting is shown only when set, so existing clients serialize as before.
    let listed = f.core.list_clients(&f.admin).unwrap();
    let view = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|client| client["client_id"] == JMAP)
        .unwrap();
    assert_eq!(view["settings"]["agent_access"], true);
    assert!(
        serde_json::to_value(ProviderSettings::default())
            .unwrap()
            .get("agent_access")
            .is_none()
    );
}

#[test]
fn withdrawing_consent_for_the_application_ends_agent_access() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let secret = application(&f, JMAP, |_| {});
    application(&f, "calendar", |_| {});
    let credential = agent(&f, &owner, "mail-helper", 3600);
    for cid in [JMAP, "calendar"] {
        f.core
            .approve_my_agent_application(&owner, "mail-helper", access(cid, &["mail"]))
            .unwrap();
    }
    let token = text(
        &f.core.token(exchange(&credential, JMAP)).unwrap(),
        "access_token",
    );
    assert_live(&f, &secret, &token);
    f.core.revoke_consent(&owner, JMAP).unwrap();
    assert_ended(&f, &secret, &token);
    assert_eq!(
        f.core.token(exchange(&credential, JMAP)).unwrap_err().code,
        "invalid_grant"
    );
    // Another application's approval is untouched.
    assert!(f.core.token(exchange(&credential, "calendar")).is_ok());
}

#[test]
fn agents_still_cannot_authorize_consent_or_launder_application_tokens() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let credential = agent(&f, &owner, "mail-helper", 3600);
    let jmap_secret = application(&f, JMAP, |_| {});
    f.core
        .approve_my_agent_application(&owner, "mail-helper", access(JMAP, &["mail"]))
        .unwrap();
    let token = text(
        &f.core.token(exchange(&credential, JMAP)).unwrap(),
        "access_token",
    );
    // RI-SES-001: neither the agent credential nor its application token is a session.
    for bearer in [&credential, &token] {
        let verifier = crypto::random_token("");
        assert!(
            f.core
                .authorize(bearer, f.request(JMAP, &verifier))
                .is_err()
        );
        assert!(
            f.core
                .approve_my_agent_application(bearer, "mail-helper", access(JMAP, &["mail"]))
                .is_err()
        );
    }
    assert!(f.core.me(&token).is_err());

    // The application token is never an exchange subject, even under a trust policy
    // that accepts the same application's ordinary user tokens.
    let worker = text(
        &f.core
            .create_client(
                &f.admin,
                NewClient {
                    client_id: "worker".into(),
                    name: "worker".into(),
                    confidential: true,
                    redirect_uris: vec![],
                    scopes: strings(&["mail"]),
                    allowed_groups: Default::default(),
                    require_mfa: false,
                    service: true,
                    settings: ProviderSettings {
                        allowed_grants: strings(&["client_credentials", TOKEN_EXCHANGE]),
                        exchange: Some(ExchangePolicy {
                            subject_clients: strings(&[JMAP]),
                            target_clients: strings(&["api"]),
                            scopes: strings(&["mail"]),
                            allow_delegation: false,
                            allow_impersonation: true,
                        }),
                        ..Default::default()
                    },
                },
            )
            .unwrap(),
        "client_secret",
    );
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "api".into(),
                name: "api".into(),
                confidential: true,
                redirect_uris: vec![],
                scopes: strings(&["mail"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: true,
                settings: ProviderSettings {
                    exchange_from: strings(&["worker"]),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let onward = |subject: &str| TokenRequest {
        grant_type: TOKEN_EXCHANGE.into(),
        client_id: Some("worker".into()),
        client_secret: Some(worker.clone()),
        subject_token: Some(subject.into()),
        subject_token_type: Some(ACCESS_TOKEN.into()),
        audience: Some("api".into()),
        scope: Some("mail".into()),
        ..Default::default()
    };
    let verifier = crypto::random_token("");
    let mut request = f.request(JMAP, &verifier);
    request.scope = "openid mail".into();
    let redirect = f.core.authorize(&owner, request).unwrap();
    let code = url::Url::parse(&redirect)
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == "code")
        .unwrap()
        .1
        .into_owned();
    let ordinary = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some(JMAP.into()),
            client_secret: Some(jmap_secret),
            code: Some(code),
            code_verifier: Some(verifier),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            ..Default::default()
        })
        .unwrap();
    assert!(
        f.core
            .token(onward(&text(&ordinary, "access_token")))
            .is_ok()
    );
    assert_eq!(
        f.core.token(onward(&token)).unwrap_err().code,
        "invalid_grant"
    );
}

#[tokio::test]
async fn browser_owner_routes_approve_list_and_revoke_application_access() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let credential = agent(&f, &owner, "browser-agent", 3600);
    let secret = application(&f, JMAP, |_| {});
    let reply = f
        .core
        .portal_password(None, "owner".into(), PASSWORD.into(), None, false)
        .unwrap();
    let cookie = reply
        .cookies
        .iter()
        .find_map(|value| value.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap()
        .to_owned();
    let page = f.core.portal_security(Some(&cookie)).unwrap();
    let user_id = page["user"]["id"].clone();
    let session_id = page["current_session_id"].clone();
    let origin = url::Url::parse(&f.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let app = riauth::api::router(f.core.clone());
    let send = |method: &str, path: &str, body: Option<Value>| {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("cookie", format!("riauth_sso={cookie}"))
            .header("origin", &origin)
            .header("x-riauth-portal", "1")
            .header("sec-fetch-site", "same-origin");
        if body.is_some() {
            request = request.header("content-type", "application/json");
        }
        let request = request
            .body(Body::from(body.map(|b| b.to_string()).unwrap_or_default()))
            .unwrap();
        let app = app.clone();
        async move {
            let response = app.oneshot(request).await.unwrap();
            let status = response.status();
            let bytes = axum::body::to_bytes(response.into_body(), 65_536)
                .await
                .unwrap();
            (
                status,
                serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null),
            )
        }
    };
    let path = "/api/portal/agents/browser-agent/applications";
    let stale = json!({"expected_user_id": user_id, "expected_session_id": "another", "application": {"client_id": JMAP, "scopes": ["mail"]}});
    assert_eq!(
        send("POST", path, Some(stale)).await.0,
        StatusCode::CONFLICT
    );
    let body = json!({"expected_user_id": user_id, "expected_session_id": session_id, "application": {"client_id": JMAP, "scopes": ["mail"]}});
    let (status, approved) = send("POST", path, Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{approved}");
    let (status, listed) = send("GET", path, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["applications"][0]["id"], approved["id"]);
    let token = text(
        &f.core.token(exchange(&credential, JMAP)).unwrap(),
        "access_token",
    );
    assert_live(&f, &secret, &token);
    let (status, revoked) = send(
        "POST",
        &format!("{path}/{}/revoke", text(&approved, "id")),
        Some(json!({"expected_user_id": user_id, "expected_session_id": session_id})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{revoked}");
    assert_eq!(revoked["active"], false);
    assert_ended(&f, &secret, &token);

    // The bearer routes serve the CLI, including DELETE for revocation.
    let bearer = |method: &str, path: String, body: Option<Value>| {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("authorization", format!("Bearer {owner}"));
        if body.is_some() {
            request = request.header("content-type", "application/json");
        }
        let request = request
            .body(Body::from(body.map(|b| b.to_string()).unwrap_or_default()))
            .unwrap();
        let app = app.clone();
        async move {
            let response = app.oneshot(request).await.unwrap();
            let status = response.status();
            let bytes = axum::body::to_bytes(response.into_body(), 65_536)
                .await
                .unwrap();
            (
                status,
                serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null),
            )
        }
    };
    let (status, approved) = bearer(
        "POST",
        "/api/me/agents/browser-agent/applications".into(),
        Some(json!({"client_id": JMAP, "scopes": ["mail"], "ttl": 600})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{approved}");
    let (status, listed) = bearer(
        "GET",
        "/api/me/agents/browser-agent/applications".into(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["applications"].as_array().unwrap().len(), 2);
    let (status, revoked) = bearer(
        "DELETE",
        format!(
            "/api/me/agents/browser-agent/applications/{}",
            text(&approved, "id")
        ),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{revoked}");
    assert_eq!(revoked["active"], false);
}
