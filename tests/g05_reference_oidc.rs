//! Disposable reference RP for the G05 Authentik cutover rehearsal.
//! The RP runs in process; the real application and Authentik remain external gates.
mod common;

use common::{Fixture, PASSWORD, text};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use riauth::{
    crypto::{self, now},
    oidc::TokenRequest,
    state::ApplyRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use url::Url;

const CLIENT: &str = "reference-rp";
const CALLBACK: &str = "http://localhost:7777/callback?existing=1";
const AUTHENTIK_BACKEND: &str = "authentik-backend";
const RIAUTH_BACKEND: &str = "riauth-local";

/// The small amount of RP behavior that must survive a route change: exact
/// callback/state/issuer checks and independent validation against public JWKS.
#[derive(Serialize, Deserialize)]
struct ReferenceRp {
    issuer: String,
    backend: String,
}

impl ReferenceRp {
    fn sign_in(&self, f: &Fixture, session: &str) -> (Value, Value) {
        assert_eq!(self.backend, RIAUTH_BACKEND);
        let verifier = crypto::random_token("");
        let mut request = f.request(CLIENT, &verifier);
        request.scope = "openid profile groups offline_access".into();
        let callback = f.core.authorize(session, request).unwrap();
        let url = Url::parse(&callback).unwrap();
        assert_eq!(url.path(), "/callback");
        assert_eq!(url.host_str(), Some("localhost"));
        let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["existing"], "1");
        assert_eq!(query["state"], "state with & delimiters");
        assert_eq!(query["iss"], self.issuer);
        let tokens = f
            .core
            .token(TokenRequest {
                grant_type: "authorization_code".into(),
                client_id: Some(CLIENT.into()),
                code: Some(query["code"].clone()),
                redirect_uri: Some(CALLBACK.into()),
                code_verifier: Some(verifier),
                ..Default::default()
            })
            .unwrap();
        let claims = self.validate_id_token(f, &tokens);
        assert_eq!(claims["nonce"], "expected-nonce");
        (tokens, claims)
    }

    fn validate_id_token(&self, f: &Fixture, tokens: &Value) -> Value {
        self.try_validate_id_token(f, tokens).unwrap()
    }

    fn try_validate_id_token(
        &self,
        f: &Fixture,
        tokens: &Value,
    ) -> jsonwebtoken::errors::Result<Value> {
        let jwks = f.core.jwks().unwrap();
        let key = &jwks["keys"][0];
        let decoding = DecodingKey::from_rsa_components(
            key["n"].as_str().unwrap(),
            key["e"].as_str().unwrap(),
        )
        .unwrap();
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[CLIENT]);
        validation.set_issuer(&[self.issuer.as_str()]);
        jsonwebtoken::decode::<Value>(tokens["id_token"].as_str().unwrap(), &decoding, &validation)
            .map(|token| token.claims)
    }
}

fn migrated_fixture(f: &Fixture) -> Value {
    let issuer = &f.core.config.issuer;
    let input = json!({
        "api_version":"riauth.authentik-import/v1", "issuer":issuer,
        "users":[
            {"pk":42,"uid":"authentik-alice-sub","uuid":"uuid-alice","username":"alice",
             "name":"Alice","email":"alice@example.test","groups":["g-eng"],
             "attributes":{},"type":"internal","is_active":true,"roles":[]},
            {"pk":43,"uid":"authentik-bob-sub","uuid":"uuid-bob","username":"bob",
             "name":"Bob","email":"bob@example.test","groups":[],
             "attributes":{},"type":"internal","is_active":true,"roles":[]}
        ],
        "groups":[{"pk":"g-eng","name":"engineering","parents":[]}],
        "providers":[{"pk":1,"name":"Reference RP","client_id":CLIENT,"client_type":"public",
            "grant_types":["authorization_code","refresh_token"],
            "redirect_uris":[{"matching_mode":"strict","url":CALLBACK}],
            "property_mappings":[],"sub_mode":"hashed_user_id","issuer_mode":"per_provider",
            "include_claims_in_id_token":true}],
        "applications":[{"pk":"app-reference","slug":CLIENT,"provider":1,
            "name":"Reference RP","policy_engine_mode":"any"}],
        "policy_bindings":[{"pk":"eng-only","target":"app-reference","policy":null,
            "group":"g-eng","user":null,"negate":false,"enabled":true,"order":0}],
        "sources":[],
        "passwords":{"alice":{"reference":"env:ALICE","version":"v1"},
                     "bob":{"reference":"env:BOB","version":"v1"}},
        "clients":{"reference-rp":{"issuer":format!("{issuer}/application/o/{CLIENT}/"),
            "scopes":["openid","profile","groups","offline_access"],
            "settings":{"groups_in_profile":true},
            "translated_mapping_ids":[],"translated_binding_ids":[],
            "authentication_flow_reviewed":true,"require_mfa":false}}
    });
    riauth::migration::convert(serde_json::from_value(input).unwrap()).unwrap()
}

#[test]
fn local_reference_rp_cutover_rehearsal() {
    let f = Fixture::new();
    let report = migrated_fixture(&f);
    assert_eq!(report["ready_for_plan"], true, "{}", report["blockers"]);
    assert_eq!(report["summary"]["blocking"], 0);
    assert_eq!(report["old_tokens_and_sessions_imported"], false);
    let manifest = serde_json::from_value(report["manifest"].clone()).unwrap();
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    f.core
        .apply_state(
            &f.admin,
            ApplyRequest {
                plan,
                secrets: [
                    ("env:ALICE".into(), PASSWORD.into()),
                    ("env:BOB".into(), PASSWORD.into()),
                ]
                .into(),
                run_id: Some("g05-local-reference-rp".into()),
            },
        )
        .unwrap();
    let issuer = format!("{}/application/o/{CLIENT}/", f.core.config.issuer);
    assert_eq!(f.core.provider_discovery(CLIENT).unwrap()["issuer"], issuer);
    eprintln!(
        "G05-01 preflight ready=true blockers=0; plan/apply accepted; discovery issuer matched"
    );
    let saved_rp = ReferenceRp {
        issuer: issuer.clone(),
        backend: AUTHENTIK_BACKEND.into(),
    };
    let route_file = f._dir.path().join("reference-rp-route.json");
    let saved_config = serde_json::to_vec(&saved_rp).unwrap();
    fs::write(&route_file, &saved_config).unwrap();
    fs::write(
        &route_file,
        serde_json::to_vec(&ReferenceRp {
            issuer,
            backend: RIAUTH_BACKEND.into(),
        })
        .unwrap(),
    )
    .unwrap();
    let rp: ReferenceRp = serde_json::from_slice(&fs::read(&route_file).unwrap()).unwrap();
    let alice = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let bob = text(
        &f.core.login("bob".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );

    // Imported group binding denies Bob before issuing a code or RP session.
    let mut denied = f.request(CLIENT, &crypto::random_token(""));
    denied.scope = "openid profile groups offline_access".into();
    let before = f
        .core
        .store
        .list::<riauth::model::Code>("codes")
        .unwrap()
        .len();
    assert!(f.core.authorize(&bob, denied).is_err());
    assert_eq!(
        f.core
            .store
            .list::<riauth::model::Code>("codes")
            .unwrap()
            .len(),
        before
    );
    eprintln!("G05-02 denied bob; authorization-code count unchanged");

    let (tokens, claims) = rp.sign_in(&f, &alice);
    assert_eq!(claims["sub"], "authentik-alice-sub");
    assert_eq!(claims["groups"], json!(["engineering"]));
    assert_eq!(claims["amr"], json!(["pwd"]));
    assert_eq!(
        f.core.userinfo(&text(&tokens, "access_token")).unwrap()["sub"],
        claims["sub"]
    );
    eprintln!(
        "G05-03 callback state/issuer/PKCE and JWKS JWT accepted; sub=authentik-alice-sub groups=engineering amr=pwd"
    );
    let refresh = TokenRequest {
        grant_type: "refresh_token".into(),
        client_id: Some(CLIENT.into()),
        refresh_token: Some(text(&tokens, "refresh_token")),
        ..Default::default()
    };
    let rotated = f.core.token(refresh.clone()).unwrap();
    assert_ne!(tokens["refresh_token"], rotated["refresh_token"]);
    assert_eq!(f.core.token(refresh).unwrap_err().code, "invalid_grant");
    assert!(f.core.userinfo(&text(&rotated, "access_token")).is_err());
    eprintln!("G05-04 refresh rotated; replay invalid_grant and family access rejected");

    // Rehearse the reviewed MFA policy after enrolling a new riAuth factor.
    common::client_policy::replace(&f.core, &f.admin, CLIENT, None, Some(true));
    assert!(
        f.core
            .authorize(&alice, f.request(CLIENT, &crypto::random_token("")))
            .is_err()
    );
    let enrollment = f.core.mfa_begin(&alice).unwrap();
    let totp = crypto::totp(&text(&enrollment, "secret"), "alice").unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate((now() / 30 - 1) * 30).to_string())
        .unwrap();
    assert!(f.core.me(&alice).is_err());
    let mfa = text(
        &f.core
            .login(
                "alice".into(),
                PASSWORD.into(),
                Some(totp.generate(now()).to_string()),
            )
            .unwrap(),
        "session_token",
    );
    let (mfa_tokens, mfa_claims) = rp.sign_in(&f, &mfa);
    assert_eq!(mfa_claims["sub"], claims["sub"]);
    assert_eq!(mfa_claims["amr"], json!(["pwd", "otp"]));
    eprintln!(
        "G05-05 password-only denied after require_mfa; TOTP sign-in accepted with stable sub and pwd+otp"
    );
    let recovery = f.core.recovery_codes(&mfa).unwrap();
    let code = recovery["recovery_codes"][0].as_str().unwrap().to_owned();
    let recovered = text(
        &f.core
            .login("alice".into(), PASSWORD.into(), Some(code.clone()))
            .unwrap(),
        "session_token",
    );
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(code))
            .is_err()
    );
    let (_, recovered_claims) = rp.sign_in(&f, &recovered);
    assert_eq!(recovered_claims["sub"], claims["sub"]);
    eprintln!("G05-06 riAuth recovery code accepted once; reused code denied; RP sub stable");

    f.core.logout(&mfa).unwrap();
    assert!(f.core.userinfo(&text(&mfa_tokens, "access_token")).is_err());
    assert!(f.core.me(&mfa).is_err());
    assert!(f.core.me(&recovered).is_ok());
    eprintln!("G05-07 logout revoked session and access; separate recovered session survived");

    // The RP's issuer stays byte-identical; only its backend route changes.
    // No Authentik service is present to prove a rollback sign-in or session.
    fs::write(&route_file, &saved_config).unwrap();
    assert_eq!(fs::read(&route_file).unwrap(), saved_config);
    let rolled_back: ReferenceRp = serde_json::from_slice(&fs::read(&route_file).unwrap()).unwrap();
    assert_eq!(rolled_back.backend, AUTHENTIK_BACKEND);
    assert_eq!(rolled_back.issuer, rp.issuer);
    eprintln!(
        "G05-08 saved RP route file restored to Authentik backend; issuer unchanged; peer sign-in untested"
    );
}
