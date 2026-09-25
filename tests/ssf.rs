mod common;

use common::security::{Dependents, events, subscribe};
use common::{Fixture, PASSWORD};
use riauth::{
    agent::{NewAgent, Permission},
    config::Config,
    core::Core,
    crypto::{self, SigningKey},
    jose::PublicJwks,
    model::*,
    oidc::TokenRequest,
    ssf::{
        ACCOUNT_DISABLED, CREDENTIAL_CHANGE, ConfigurationInput, Delivery, DeliverySpec,
        MAX_ATTEMPTS, PUSH, SESSION_REVOKED, SsfAuth, StreamInput, SubjectBindings,
    },
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
    time::Duration,
};
use tempfile::TempDir;

fn encrypted_fixture() -> Fixture {
    let dir = TempDir::new().unwrap();
    let key_file = dir.path().join("database.key");
    riauth::config::write_private(&key_file, crypto::random_token("").as_bytes(), false).unwrap();
    let config = Config {
        data_dir: dir.path().into(),
        database_key_file: Some(key_file),
        ..Default::default()
    };
    let core = Core::initialize(
        config,
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    Fixture {
        _dir: dir,
        core,
        admin,
    }
}

fn signer() -> (SigningKey, PublicJwks, String) {
    let key = SigningKey::generate_algorithm("ES256").unwrap();
    let jwk: riauth::jose::PublicJwk = serde_json::from_value(key.jwk().unwrap()).unwrap();
    let marker = jwk.x.clone().unwrap();
    (key, PublicJwks { keys: vec![jwk] }, marker)
}

fn stream(
    id: &str,
    issuer: &str,
    audience: &str,
    endpoint: &str,
    events: &[&str],
    jwks: PublicJwks,
    subjects: &[(&str, &str)],
) -> StreamInput {
    StreamInput {
        id: id.into(),
        issuer: issuer.into(),
        audience: audience.into(),
        events_requested: events.iter().map(|event| (*event).to_string()).collect(),
        events: Default::default(),
        delivery: Some(DeliverySpec {
            method: PUSH.into(),
            endpoint_url: endpoint.into(),
            authorization_header: None,
        }),
        delivery_method: None,
        endpoint_url: None,
        jwks,
        subjects: subjects
            .iter()
            .map(|(sub, user)| ((*sub).to_string(), (*user).to_string()))
            .collect(),
    }
}

fn set_token(
    key: &SigningKey,
    iss: &str,
    aud: &str,
    sub: &str,
    event: &str,
    jti: &str,
    _expires_at: u64,
) -> String {
    set_token_with_subject(
        key,
        iss,
        aud,
        json!({"format": "iss_sub", "iss": iss, "sub": sub}),
        event,
        jti,
    )
}

fn set_token_with_subject(
    key: &SigningKey,
    iss: &str,
    aud: &str,
    sub_id: Value,
    event: &str,
    jti: &str,
) -> String {
    let payload = if event == CREDENTIAL_CHANGE {
        json!({"credential_type":"password","change_type":"update"})
    } else {
        json!({})
    };
    key.sign_type(
        &json!({
            "iss": iss,
            "aud": aud,
            "iat": crypto::now(),
            "jti": jti,
            "sub_id": sub_id,
            "events": { event: payload }
        }),
        "secevent+jwt",
    )
    .unwrap()
}

fn jwt_claims(token: &str) -> Value {
    use base64::Engine;
    let payload = token.split('.').nth(1).unwrap();
    serde_json::from_slice(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .unwrap(),
    )
    .unwrap()
}

fn user_enabled(f: &Fixture, username: &str) -> bool {
    f.core
        .list_users(&f.admin)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|user| user["username"] == username)
        .unwrap()["enabled"]
        .as_bool()
        .unwrap()
}

struct Hit {
    header: String,
    body: String,
}

fn push_server(statuses: Vec<u16>) -> (String, Arc<Mutex<Vec<Hit>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let received = Arc::new(Mutex::new(Vec::new()));
    let bodies = received.clone();
    let statuses = Arc::new(Mutex::new(VecDeque::from(statuses)));
    std::thread::spawn(move || {
        for _ in 0..16 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
            let mut buf = Vec::new();
            let mut tmp = [0u8; 4096];
            let mut header_end = None;
            while header_end.is_none() && buf.len() < 65_536 {
                match stream.read(&mut tmp) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&tmp[..n]);
                        header_end = buf.windows(4).position(|window| window == b"\r\n\r\n");
                    }
                }
            }
            let Some(header_end) = header_end else {
                continue;
            };
            let header = String::from_utf8_lossy(&buf[..header_end]).to_string();
            let len = header
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            while buf.len() < header_end + 4 + len {
                match stream.read(&mut tmp) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => buf.extend_from_slice(&tmp[..n]),
                }
            }
            let start = header_end + 4;
            let end = (start + len).min(buf.len());
            bodies.lock().unwrap().push(Hit {
                header,
                body: String::from_utf8_lossy(&buf[start..end]).into_owned(),
            });
            let status = statuses.lock().unwrap().pop_front().unwrap_or(204);
            let reason = match status {
                202 => "Accepted",
                204 => "No Content",
                400 => "Bad Request",
                429 => "Too Many Requests",
                500 => "Internal Server Error",
                _ => "OK",
            };
            let response = format!(
                "HTTP/1.1 {status} {reason}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (format!("http://{addr}/events"), received)
}

fn release(f: &Fixture, id: &str) {
    f.core
        .store
        .write(|tx| {
            let mut delivery = tx.get::<Delivery>("ssf_deliveries", id)?.unwrap();
            delivery.next_attempt = 0;
            tx.put("ssf_deliveries", id, &delivery)
        })
        .unwrap();
}

#[test]
fn inbound_sets_disable_only_the_linked_account() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let (key_a, jwks_a, marker_a) = signer();
    let (key_b, jwks_b, _) = signer();
    let audience = f.core.config.issuer.clone();
    let created = f
        .core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "tenant-a",
                "https://transmitter-a.example",
                &audience,
                "https://receiver.example/events",
                &[ACCOUNT_DISABLED],
                jwks_a,
                &[("ext-a", "alice")],
            ),
        )
        .unwrap();
    assert!(created.get("jwks").is_none());
    assert!(!created.to_string().contains(&marker_a));
    assert_eq!(
        created["inbound_push_url"],
        format!("{audience}/api/ssf/events")
    );
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "tenant-b",
                "https://transmitter-b.example",
                &audience,
                "https://receiver.example/events",
                &[SESSION_REVOKED, CREDENTIAL_CHANGE],
                jwks_b,
                &[("ext-b", "bob")],
            ),
        )
        .unwrap();
    let other = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "reader".into(),
                permissions: vec![Permission {
                    action: "user.read".into(),
                    resource: "*".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let other_token = other["credential"]["token"].as_str().unwrap();
    assert_eq!(
        f.core
            .ssf_list(&SsfAuth::Bearer(other_token.into()))
            .unwrap_err()
            .status,
        axum::http::StatusCode::FORBIDDEN
    );
    let scoped = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scoped".into(),
                permissions: vec![Permission {
                    action: "ssf.manage".into(),
                    resource: "ssf/tenant-b".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let scoped_token = scoped["credential"]["token"].as_str().unwrap();
    let visible = f
        .core
        .ssf_list(&SsfAuth::Bearer(scoped_token.into()))
        .unwrap();
    assert_eq!(visible["streams"].as_array().unwrap().len(), 1);
    assert_eq!(visible["streams"][0]["stream_id"], "tenant-b");
    assert!(!visible.to_string().contains(&marker_a));
    assert!(
        f.core
            .ssf_delete(&SsfAuth::Bearer(scoped_token.into()), "tenant-a")
            .is_err()
    );

    let bad = set_token(
        &key_a,
        "https://transmitter-a.example",
        &audience,
        "ext-a",
        ACCOUNT_DISABLED,
        "bad-jti",
        crypto::now() + 120,
    );
    let mut chars = bad.into_bytes();
    let pos = chars.iter().rposition(|byte| *byte == b'.').unwrap() + 2;
    chars[pos] = if chars[pos] == b'A' { b'B' } else { b'A' };
    let bad = String::from_utf8(chars).unwrap();
    assert!(f.core.accept_set(&bad).is_err());
    assert!(user_enabled(&f, "alice"));
    assert!(f.core.me(&alice).is_ok());

    let jti = "event-1";
    let token = set_token(
        &key_a,
        "https://transmitter-a.example",
        &audience,
        "ext-a",
        ACCOUNT_DISABLED,
        jti,
        crypto::now() + 120,
    );
    assert_eq!(f.core.accept_set(&token).unwrap()["accepted"], true);
    assert!(!user_enabled(&f, "alice"));
    assert!(f.core.me(&alice).is_err());
    assert!(user_enabled(&f, "bob"));
    assert!(f.core.me(&bob).is_ok());
    assert_eq!(f.core.accept_set(&token).unwrap()["accepted"], true);

    let cross = set_token(
        &key_a,
        "https://transmitter-a.example",
        &audience,
        "ext-b",
        ACCOUNT_DISABLED,
        "cross-jti",
        crypto::now() + 120,
    );
    assert_eq!(f.core.accept_set(&cross).unwrap()["accepted"], true);
    assert!(user_enabled(&f, "bob"));
    assert!(f.core.me(&bob).is_ok());
    let unknown = set_token(
        &key_a,
        "https://transmitter-a.example",
        &audience,
        "nobody",
        ACCOUNT_DISABLED,
        "unknown-jti",
        crypto::now() + 120,
    );
    assert_eq!(f.core.accept_set(&unknown).unwrap()["accepted"], true);
    assert_eq!(
        f.core
            .list_users(&f.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let revoke = set_token(
        &key_b,
        "https://transmitter-b.example",
        &audience,
        "ext-b",
        SESSION_REVOKED,
        "revoke-bob",
        crypto::now() + 120,
    );
    assert_eq!(f.core.accept_set(&revoke).unwrap()["accepted"], true);
    assert!(user_enabled(&f, "bob"));
    assert!(f.core.me(&bob).is_err());
    let audit = f.core.audit_events(&f.admin, 100).unwrap().to_string();
    assert!(audit.contains("ssf.ignored"));
    assert!(audit.contains("ssf.account-disabled"));
    assert!(!audit.contains(&token));
    assert!(!audit.contains("eyJ"));
}

#[test]
fn ssf_subject_bindings_include_format_and_subject_issuer() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let (key, jwks, _) = signer();
    let transmitter = "https://transmitter.example";
    let audience = &f.core.config.issuer;
    let alice_id = json!({"format":"iss_sub","iss":"subject-a","sub":"shared"});
    let bob_id = json!({"format":"iss_sub","iss":"urn:example:subject-b","sub":"shared"});
    let alice_key = alice_id.to_string();
    let bob_key = bob_id.to_string();
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "format-bound",
                transmitter,
                audience,
                "https://receiver.example/events",
                &[ACCOUNT_DISABLED],
                jwks,
                &[(alice_key.as_str(), "alice"), (bob_key.as_str(), "bob")],
            ),
        )
        .unwrap();

    let opaque = set_token_with_subject(
        &key,
        transmitter,
        audience,
        json!({"format":"opaque","id":"shared"}),
        ACCOUNT_DISABLED,
        "opaque-shared",
    );
    assert_eq!(f.core.accept_set(&opaque).unwrap()["accepted"], true);
    assert!(user_enabled(&f, "alice") && user_enabled(&f, "bob"));

    let alice_set = set_token_with_subject(
        &key,
        transmitter,
        audience,
        alice_id,
        ACCOUNT_DISABLED,
        "subject-a",
    );
    f.core.accept_set(&alice_set).unwrap();
    assert!(!user_enabled(&f, "alice"));
    assert!(user_enabled(&f, "bob"));
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&bob).is_ok());

    let bob_set = set_token_with_subject(
        &key,
        transmitter,
        audience,
        bob_id,
        ACCOUNT_DISABLED,
        "subject-b",
    );
    f.core.accept_set(&bob_set).unwrap();
    assert!(!user_enabled(&f, "bob"));
}

#[test]
fn ssf_set_profile_rejects_forbidden_claims_and_uses_bounded_iat() {
    let f = Fixture::new();
    f.user("alice");
    let (key, jwks, _) = signer();
    let transmitter = "https://transmitter.example";
    let audience = &f.core.config.issuer;
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "profile",
                transmitter,
                audience,
                "https://receiver.example/events",
                &[ACCOUNT_DISABLED],
                jwks,
                &[("external", "alice")],
            ),
        )
        .unwrap();
    let base = json!({
        "iss": transmitter, "aud": audience, "iat": crypto::now(), "jti": "profile-event",
        "sub_id": {"format":"iss_sub", "iss":transmitter, "sub":"external"},
        "events": {ACCOUNT_DISABLED: {}}
    });
    for (name, mut claims) in [
        ("exp", base.clone()),
        ("sub", base.clone()),
        ("missing-iat", base.clone()),
        ("stale-iat", base.clone()),
        ("future-iat", base.clone()),
    ] {
        match name {
            "exp" => claims["exp"] = json!(crypto::now() + 300),
            "sub" => claims["sub"] = json!("external"),
            "missing-iat" => {
                claims.as_object_mut().unwrap().remove("iat");
            }
            "stale-iat" => claims["iat"] = json!(crypto::now() - 8 * 86_400),
            "future-iat" => claims["iat"] = json!(crypto::now() + 120),
            _ => unreachable!(),
        }
        let token = key.sign_type(&claims, "secevent+jwt").unwrap();
        assert!(f.core.accept_set(&token).is_err(), "{name}");
        assert!(user_enabled(&f, "alice"), "{name}");
    }
    let valid = key.sign_type(&base, "secevent+jwt").unwrap();
    f.core.accept_set(&valid).unwrap();
    assert!(!user_enabled(&f, "alice"));
}

#[test]
fn duplicate_set_acknowledges_without_reapplying_session_revocation() {
    let f = Fixture::new();
    f.user("alice");
    let (key, jwks, _) = signer();
    let transmitter = "https://transmitter.example";
    let audience = &f.core.config.issuer;
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "replay",
                transmitter,
                audience,
                "https://receiver.example/events",
                &[SESSION_REVOKED],
                jwks,
                &[("external", "alice")],
            ),
        )
        .unwrap();
    let token = set_token_with_subject(
        &key,
        transmitter,
        audience,
        json!({"format":"iss_sub", "iss":transmitter, "sub":"external"}),
        SESSION_REVOKED,
        "same-jti",
    );
    f.core.accept_set(&token).unwrap();
    let before = f
        .core
        .store
        .list::<User>("users")
        .unwrap()
        .into_iter()
        .find(|(_, user)| user.username == "alice")
        .unwrap()
        .1
        .epoch;
    assert_eq!(f.core.accept_set(&token).unwrap()["accepted"], true);
    let after = f
        .core
        .store
        .list::<User>("users")
        .unwrap()
        .into_iter()
        .find(|(_, user)| user.username == "alice")
        .unwrap()
        .1
        .epoch;
    assert_eq!(after, before);
}

#[test]
fn removing_a_subject_binding_stops_queued_disclosure() {
    let f = Fixture::new();
    f.user("alice");
    let (_key, jwks, _) = signer();
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "rebind",
                "https://transmitter.example",
                &f.core.config.issuer,
                "https://receiver.example/events",
                &[ACCOUNT_DISABLED],
                jwks,
                &[("external", "alice")],
            ),
        )
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let queued = f.core.store.list::<Delivery>("ssf_deliveries").unwrap();
    assert_eq!(queued.len(), 1);
    assert!(!queued[0].1.stopped);
    f.core
        .ssf_bind_subjects(
            &SsfAuth::Bearer(f.admin.clone()),
            "rebind",
            SubjectBindings {
                subjects: Default::default(),
            },
        )
        .unwrap();
    let stopped = f
        .core
        .store
        .get::<Delivery>("ssf_deliveries", &queued[0].0)
        .unwrap()
        .unwrap();
    assert!(stopped.stopped);
    assert!(f.core.deliver_once().unwrap().is_empty());
}

#[test]
fn local_disable_enqueues_and_delivers_without_storing_the_set() {
    let f = Fixture::new();
    f.user("alice");
    let (_key, jwks, marker) = signer();
    let (url, received) = push_server(vec![202]);
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "outbound",
                "https://transmitter.example",
                "subscriber-audience",
                &url,
                &[ACCOUNT_DISABLED],
                jwks.clone(),
                &[("ext-a", "alice")],
            ),
        )
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let queued = f.core.store.list::<Delivery>("ssf_deliveries").unwrap();
    assert_eq!(queued.len(), 1);
    assert!(queued[0].1.delivered_at.is_none());
    assert_eq!(queued[0].1.attempts, 0);
    assert!(!serde_json::to_string(&queued).unwrap().contains("eyJ"));
    assert!(!serde_json::to_string(&queued).unwrap().contains(&marker));
    let results = f.core.deliver_once().unwrap();
    assert_eq!(results[0]["status"], 202);
    let hit = received.lock().unwrap().pop().unwrap();
    assert!(
        hit.header
            .to_ascii_lowercase()
            .contains("application/secevent+jwt")
    );
    assert!(
        hit.header
            .to_ascii_lowercase()
            .contains("accept: application/json")
    );
    assert!(hit.body.starts_with("eyJ"));
    let jwks = f.core.jwks().unwrap();
    let jwk = &jwks["keys"][0];
    let key =
        jsonwebtoken::DecodingKey::from_jwk(&serde_json::from_value(jwk.clone()).unwrap()).unwrap();
    let alg = match jwk["alg"].as_str() {
        Some("ES256") => jsonwebtoken::Algorithm::ES256,
        Some("EdDSA") => jsonwebtoken::Algorithm::EdDSA,
        _ => jsonwebtoken::Algorithm::RS256,
    };
    let mut validation = jsonwebtoken::Validation::new(alg);
    validation.set_issuer(&[&f.core.config.issuer]);
    validation.set_audience(&["subscriber-audience"]);
    validation.set_required_spec_claims(&["iss", "aud", "iat", "jti"]);
    validation.validate_exp = false;
    validation.leeway = 0;
    let claims = jsonwebtoken::decode::<Value>(&hit.body, &key, &validation)
        .unwrap()
        .claims;
    assert!(claims.get("sub").is_none());
    assert!(claims.get("exp").is_none());
    assert_eq!(claims["sub_id"]["sub"], "ext-a");
    assert!(claims["events"][ACCOUNT_DISABLED].is_object());
    let audit = f.core.audit_events(&f.admin, 20).unwrap().to_string();
    assert!(!audit.contains(&hit.body));
    assert!(!audit.contains("eyJ"));
}

#[test]
fn disable_entry_points_revoke_dependents_and_enqueue_exactly_once() {
    for path in [
        "core",
        "scim-put",
        "scim-patch",
        "scim-delete",
        "manifest",
        "offboard",
        "inbound",
    ] {
        let f = Fixture::new();
        let scim_id = if path.starts_with("scim") {
            Some(f.core.scim_write(&f.admin, "Users", None,
                json!({"schemas":[riauth::scim::USER],"userName":"alice","displayName":"Alice","password":common::PASSWORD,"active":true}), false).unwrap()["id"].as_str().unwrap().to_owned())
        } else {
            f.user("alice");
            None
        };
        subscribe(&f, "alice");
        let children = Dependents::create(&f, "alice");
        match path {
            "core" => {
                f.core
                    .update_user(
                        &f.admin,
                        "alice",
                        UserPatch {
                            enabled: Some(false),
                            ..Default::default()
                        },
                    )
                    .unwrap();
            }
            "scim-put" => {
                f.core
                    .scim_write(
                        &f.admin,
                        "Users",
                        scim_id.as_deref(),
                        json!({"schemas":[riauth::scim::USER],"userName":"alice","active":false}),
                        false,
                    )
                    .unwrap();
            }
            "scim-patch" => {
                f.core.scim_write(&f.admin, "Users", scim_id.as_deref(),
                json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"active","value":false}]}), true).unwrap();
            }
            "scim-delete" => {
                f.core
                    .scim_delete(&f.admin, "Users", scim_id.as_deref().unwrap())
                    .unwrap();
            }
            "manifest" => {
                let manifest = serde_json::from_value(json!({"api_version":"riauth/v1","users":[{"username":"alice","display_name":"Alice","enabled":false}]})).unwrap();
                let plan = f.core.plan_state(&f.admin, manifest).unwrap();
                assert!(events(&f, "alice").is_empty(), "preview must not enqueue");
                assert!(
                    f.core
                        .store
                        .get::<riauth::agent::Agent>("agents", "child-alice")
                        .unwrap()
                        .unwrap()
                        .enabled
                );
                for _ in 0..2 {
                    f.core
                        .apply_state(
                            &f.admin,
                            riauth::state::ApplyRequest {
                                plan: plan.clone(),
                                secrets: Default::default(),
                                run_id: None,
                            },
                        )
                        .unwrap();
                }
            }
            "offboard" => {
                let job = f
                    .core
                    .offboard_schedule(
                        &f.admin,
                        riauth::offboarding::ScheduleRequest {
                            username: "alice".into(),
                            execute_at: riauth::offboarding::ExecuteAt::Unix(crypto::now() + 60),
                            timezone: "UTC".into(),
                        },
                    )
                    .unwrap();
                let id = job["id"].as_str().unwrap();
                f.core
                    .store
                    .write(|tx| {
                        let mut job: riauth::offboarding::Job =
                            tx.get(riauth::offboarding::BUCKET, id)?.unwrap();
                        job.execute_at = 1;
                        job.next_attempt = 1;
                        tx.put(riauth::offboarding::BUCKET, id, &job)
                    })
                    .unwrap();
                assert!(
                    f.core
                        .offboard_process("worker", |_| riauth::offboarding::BeforeCommit::Proceed)
                        .unwrap()
                );
                assert!(
                    !f.core
                        .offboard_process("worker", |_| riauth::offboarding::BeforeCommit::Proceed)
                        .unwrap()
                );
            }
            "inbound" => {
                let (key, jwks, _) = signer();
                f.core
                    .ssf_create(
                        &SsfAuth::Bearer(f.admin.clone()),
                        stream(
                            "inbound",
                            "https://upstream.example",
                            "local",
                            "https://receiver.example/events",
                            &[ACCOUNT_DISABLED],
                            jwks,
                            &[("external", "alice")],
                        ),
                    )
                    .unwrap();
                for jti in ["disable-1", "disable-2"] {
                    f.core
                        .accept_set(&set_token(
                            &key,
                            "https://upstream.example",
                            "local",
                            "external",
                            ACCOUNT_DISABLED,
                            jti,
                            crypto::now() + 120,
                        ))
                        .unwrap();
                }
            }
            _ => unreachable!(),
        }
        assert_eq!(
            events(&f, "alice"),
            vec![(ACCOUNT_DISABLED.into(), "".into())],
            "{path}"
        );
        // Repeating the disabled value must not emit a second account transition.
        f.core
            .update_user(
                &f.admin,
                "alice",
                UserPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(events(&f, "alice").len(), 1, "{path}");
        children.assert_revoked_after_reenable(&f);
    }
}

#[test]
fn failed_delivery_retries_then_records_failure() {
    let f = Fixture::new();
    f.user("alice");
    let (_key, jwks, _) = signer();
    let (url, received) = push_server(vec![500, 400]);
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "retry",
                "https://transmitter.example",
                "subscriber-audience",
                &url,
                &[ACCOUNT_DISABLED],
                jwks,
                &[("ext-a", "alice")],
            ),
        )
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let created_at = crypto::now() - 3600;
    let queued = f
        .core
        .store
        .list::<Delivery>("ssf_deliveries")
        .unwrap()
        .into_iter()
        .find(|(_, delivery)| delivery.stream_id == "retry")
        .unwrap()
        .0;
    f.core
        .store
        .write(|tx| {
            let mut delivery = tx.get::<Delivery>("ssf_deliveries", &queued)?.unwrap();
            delivery.created_at = created_at;
            tx.put("ssf_deliveries", &queued, &delivery)
        })
        .unwrap();
    let first = f.core.deliver_once().unwrap();
    assert_eq!(first[0]["status"], 500);
    assert!(f.core.deliver_once().unwrap().is_empty());
    let id = first[0]["id"].as_str().unwrap().to_owned();
    release(&f, &id);
    let second = f.core.deliver_once().unwrap();
    assert_eq!(second[0]["status"], 400);
    let delivery = f
        .core
        .store
        .get::<Delivery>("ssf_deliveries", &id)
        .unwrap()
        .unwrap();
    assert!(delivery.stopped);
    assert!(delivery.last_failed);
    assert_eq!(delivery.last_status, Some(400));
    assert!(delivery.delivered_at.is_none());
    assert!(delivery.attempts >= 2);
    release(&f, &id);
    assert!(f.core.deliver_once().unwrap().is_empty());
    let received = received.lock().unwrap();
    assert_eq!(received.len(), 2);
    let first_claims = jwt_claims(&received[0].body);
    let second_claims = jwt_claims(&received[1].body);
    assert_eq!(first_claims["iat"], created_at);
    assert_eq!(
        first_claims["events"][ACCOUNT_DISABLED]["event_timestamp"],
        created_at
    );
    assert_eq!(first_claims["jti"], second_claims["jti"]);
    assert_eq!(first_claims, second_claims);
    assert!(!serde_json::to_string(&delivery).unwrap().contains("eyJ"));
}

#[test]
fn http_500_is_bounded_and_429_retries() {
    let f = Fixture::new();
    f.user("alice");
    let (_key, jwks, _) = signer();
    let (url, received) = push_server(vec![500; MAX_ATTEMPTS as usize]);
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "cap",
                "https://transmitter.example",
                "subscriber-audience",
                &url,
                &[SESSION_REVOKED],
                jwks,
                &[("ext-a", "alice")],
            ),
        )
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                revoke_sessions: true,
                ..Default::default()
            },
        )
        .unwrap();
    let mut id = String::new();
    for attempt in 0..MAX_ATTEMPTS {
        if attempt > 0 {
            release(&f, &id);
        }
        let result = f.core.deliver_once().unwrap();
        assert_eq!(result[0]["status"], 500);
        id = result[0]["id"].as_str().unwrap().to_owned();
    }
    let delivery = f
        .core
        .store
        .get::<Delivery>("ssf_deliveries", &id)
        .unwrap()
        .unwrap();
    assert!(delivery.stopped && delivery.last_failed);
    assert_eq!(delivery.attempts, MAX_ATTEMPTS);
    release(&f, &id);
    assert!(f.core.deliver_once().unwrap().is_empty());
    assert_eq!(received.lock().unwrap().len(), MAX_ATTEMPTS as usize);

    f.user("betty");
    let (_key, jwks, _) = signer();
    let (url, received) = push_server(vec![429, 204]);
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "throttle",
                "https://transmitter.example",
                "subscriber-audience",
                &url,
                &[CREDENTIAL_CHANGE],
                jwks,
                &[("ext-b", "betty")],
            ),
        )
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "betty",
            UserPatch {
                password: Some("betty-replacement-password".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let first = f.core.deliver_once().unwrap();
    assert_eq!(first[0]["status"], 429);
    let id = first[0]["id"].as_str().unwrap();
    let mid = f
        .core
        .store
        .get::<Delivery>("ssf_deliveries", id)
        .unwrap()
        .unwrap();
    assert!(!mid.stopped);
    release(&f, id);
    let second = f.core.deliver_once().unwrap();
    assert_eq!(second[0]["status"], 204);
    let done = f
        .core
        .store
        .get::<Delivery>("ssf_deliveries", id)
        .unwrap()
        .unwrap();
    assert!(done.delivered_at.is_some());
    assert!(!done.last_failed);
    assert_eq!(received.lock().unwrap().len(), 2);
}

#[test]
fn ordinary_client_secret_cannot_create_own_signing_trust() {
    let f = Fixture::new();
    f.user("alice");
    f.user("bob");
    let ordinary = f
        .core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "ssf-client".into(),
                name: "Ordinary client".into(),
                confidential: true,
                redirect_uris: vec!["http://localhost:7777/callback".into()],
                scopes: ["openid".into()].into(),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: false,
                settings: Default::default(),
            },
        )
        .unwrap();
    let secret = ordinary["client_secret"].as_str().unwrap().to_owned();
    let other = f.client("other-client", true).unwrap();
    let (key, jwks, marker) = signer();
    let rejected = f
        .core
        .ssf_create(
            &SsfAuth::ClientBasic {
                client_id: "ssf-client".into(),
                client_secret: secret.clone(),
            },
            stream(
                "client-stream",
                "https://transmitter.example",
                &f.core.config.issuer,
                "https://receiver.example/events",
                &[ACCOUNT_DISABLED],
                jwks.clone(),
                &[("ext-a", "alice")],
            ),
        )
        .unwrap_err();
    assert_eq!(rejected.status, axum::http::StatusCode::FORBIDDEN);
    let denied_config = f
        .core
        .ssf_config_create(
            &SsfAuth::ClientBasic {
                client_id: "ssf-client".into(),
                client_secret: secret.clone(),
            },
            ConfigurationInput {
                events_requested: [ACCOUNT_DISABLED.into()].into(),
                delivery: DeliverySpec {
                    method: PUSH.into(),
                    endpoint_url: "https://receiver.example/events".into(),
                    authorization_header: None,
                },
                description: None,
            },
        )
        .unwrap_err();
    assert_eq!(denied_config.status, axum::http::StatusCode::FORBIDDEN);
    let verifier = crypto::random_token("");
    let mut authorization = f.request("ssf-client", &verifier);
    authorization.scope = "openid".into();
    authorization.redirect_uri = "http://localhost:7777/callback".into();
    let redirect = f.core.authorize(&f.admin, authorization).unwrap();
    let code = url::Url::parse(&redirect)
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == "code")
        .unwrap()
        .1
        .to_string();
    let ordinary_token = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("ssf-client".into()),
            client_secret: Some(secret.clone()),
            code: Some(code),
            redirect_uri: Some("http://localhost:7777/callback".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    let ordinary_bearer = SsfAuth::Bearer(ordinary_token["access_token"].as_str().unwrap().into());
    assert_eq!(
        f.core
            .ssf_config_create(
                &ordinary_bearer,
                ConfigurationInput {
                    events_requested: [ACCOUNT_DISABLED.into()].into(),
                    delivery: DeliverySpec {
                        method: PUSH.into(),
                        endpoint_url: "https://receiver.example/events".into(),
                        authorization_header: None,
                    },
                    description: None,
                },
            )
            .unwrap_err()
            .status,
        axum::http::StatusCode::FORBIDDEN
    );
    let (_extra_key, extra_jwks, _) = signer();
    assert_eq!(
        f.core
            .ssf_create(
                &ordinary_bearer,
                stream(
                    "ordinary-bearer-trust",
                    "https://transmitter.example",
                    &f.core.config.issuer,
                    "https://receiver.example/events",
                    &[ACCOUNT_DISABLED],
                    extra_jwks,
                    &[("ext-a", "alice")],
                ),
            )
            .unwrap_err()
            .status,
        axum::http::StatusCode::UNAUTHORIZED
    );
    let authorized = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "ssf-writer".into(),
                permissions: vec![Permission {
                    action: "ssf.manage".into(),
                    resource: "ssf/client-stream".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let token = authorized["credential"]["token"].as_str().unwrap();
    let created = f
        .core
        .ssf_create(
            &SsfAuth::Bearer(token.into()),
            stream(
                "client-stream",
                "https://transmitter.example",
                &f.core.config.issuer,
                "https://receiver.example/events",
                &[ACCOUNT_DISABLED],
                jwks,
                &[("ext-a", "alice")],
            ),
        )
        .unwrap();
    assert!(!created.to_string().contains(&marker));
    assert_eq!(
        f.core
            .ssf_list(&SsfAuth::ClientBasic {
                client_id: "ssf-client".into(),
                client_secret: secret,
            })
            .unwrap_err()
            .status,
        axum::http::StatusCode::FORBIDDEN
    );
    let err = f
        .core
        .ssf_delete(
            &SsfAuth::ClientBasic {
                client_id: "other-client".into(),
                client_secret: other,
            },
            "client-stream",
        )
        .unwrap_err();
    assert_eq!(err.status, axum::http::StatusCode::FORBIDDEN);
    let cross = set_token(
        &key,
        "https://transmitter.example",
        &f.core.config.issuer,
        "ext-b",
        ACCOUNT_DISABLED,
        "unapproved-subject",
        crypto::now() + 120,
    );
    f.core.accept_set(&cross).unwrap();
    assert!(user_enabled(&f, "alice") && user_enabled(&f, "bob"));
    let approved = set_token(
        &key,
        "https://transmitter.example",
        &f.core.config.issuer,
        "ext-a",
        ACCOUNT_DISABLED,
        "approved-subject",
        crypto::now() + 120,
    );
    f.core.accept_set(&approved).unwrap();
    assert!(!user_enabled(&f, "alice"));
    assert!(user_enabled(&f, "bob"));
    let audit = f.core.audit_events(&f.admin, 50).unwrap().to_string();
    assert!(audit.contains("ssf:client-stream:agent:ssf-writer"));
    f.core
        .ssf_delete(&SsfAuth::Bearer(f.admin.clone()), "client-stream")
        .unwrap();
}

#[test]
fn outbound_configuration_and_inbound_trust_have_separate_authority() {
    let f = Fixture::new();
    f.user("alice");
    let (_key, jwks, _) = signer();
    let configuration = || ConfigurationInput {
        events_requested: [ACCOUNT_DISABLED.into()].into(),
        delivery: DeliverySpec {
            method: PUSH.into(),
            endpoint_url: "https://receiver.example/events".into(),
            authorization_header: None,
        },
        description: None,
    };
    let configure = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "configure-only".into(),
                permissions: vec![Permission {
                    action: "ssf.configure".into(),
                    resource: "*".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let configure = SsfAuth::Bearer(configure["credential"]["token"].as_str().unwrap().into());
    let configured = f
        .core
        .ssf_config_create(&configure, configuration())
        .unwrap();
    let id = configured["stream_id"].as_str().unwrap();
    assert_eq!(configured["aud"], "agent:configure-only");
    assert_eq!(
        f.core.ssf_config_read(&configure, Some(id)).unwrap(),
        configured
    );
    assert_eq!(
        f.core
            .ssf_create(
                &configure,
                stream(
                    "unauthorized-trust",
                    "https://transmitter.example",
                    &f.core.config.issuer,
                    "https://receiver.example/events",
                    &[ACCOUNT_DISABLED],
                    jwks,
                    &[("external", "alice")],
                ),
            )
            .unwrap_err()
            .status,
        axum::http::StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.core
            .ssf_bind_subjects(
                &configure,
                id,
                SubjectBindings {
                    subjects: [("external".into(), "alice".into())].into(),
                },
            )
            .unwrap_err()
            .status,
        axum::http::StatusCode::FORBIDDEN
    );
    let manage = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "trust-only".into(),
                permissions: vec![Permission {
                    action: "ssf.manage".into(),
                    resource: "*".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let manage = SsfAuth::Bearer(manage["credential"]["token"].as_str().unwrap().into());
    assert_eq!(
        f.core
            .ssf_config_create(&manage, configuration())
            .unwrap_err()
            .status,
        axum::http::StatusCode::FORBIDDEN
    );

    let mut service_settings = ProviderSettings::default();
    service_settings.resources.insert(
        "https://other-api.example/".into(),
        ["ssf.configure".into()].into(),
    );
    let service = f
        .core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "ssf-receiver".into(),
                name: "SSF receiver".into(),
                confidential: true,
                redirect_uris: vec![],
                scopes: ["ssf.configure".into()].into(),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: true,
                settings: service_settings,
            },
        )
        .unwrap();
    let service_secret = service["client_secret"].as_str().unwrap().to_owned();
    let token = f
        .core
        .token(TokenRequest {
            grant_type: "client_credentials".into(),
            client_id: Some("ssf-receiver".into()),
            client_secret: Some(service_secret.clone()),
            scope: Some("ssf.configure".into()),
            ..Default::default()
        })
        .unwrap();
    let service = SsfAuth::Bearer(token["access_token"].as_str().unwrap().into());
    let owned = f.core.ssf_config_create(&service, configuration()).unwrap();
    let owned_id = owned["stream_id"].as_str().unwrap();
    assert_eq!(owned["aud"], "client:ssf-receiver");
    assert_eq!(
        f.core.ssf_config_read(&service, Some(owned_id)).unwrap(),
        owned
    );
    assert_eq!(
        f.core.ssf_config_read(&service, None).unwrap(),
        json!([owned])
    );
    assert_eq!(
        f.core
            .ssf_config_read(&service, Some(id))
            .unwrap_err()
            .status,
        axum::http::StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.core
            .ssf_bind_subjects(
                &service,
                owned_id,
                SubjectBindings {
                    subjects: [("external".into(), "alice".into())].into(),
                },
            )
            .unwrap_err()
            .status,
        axum::http::StatusCode::UNAUTHORIZED
    );
    let bound = f
        .core
        .token(TokenRequest {
            grant_type: "client_credentials".into(),
            client_id: Some("ssf-receiver".into()),
            client_secret: Some(service_secret),
            scope: Some("ssf.configure".into()),
            resource: Some("https://other-api.example/".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        f.core
            .ssf_config_create(
                &SsfAuth::Bearer(bound["access_token"].as_str().unwrap().into()),
                configuration(),
            )
            .unwrap_err()
            .status,
        axum::http::StatusCode::FORBIDDEN
    );
    for _ in 0..7 {
        f.core.ssf_config_create(&service, configuration()).unwrap();
    }
    assert_eq!(
        f.core
            .ssf_config_create(&service, configuration())
            .unwrap_err()
            .status,
        axum::http::StatusCode::BAD_REQUEST
    );
    f.core
        .ssf_config_create(&SsfAuth::Bearer(f.admin.clone()), configuration())
        .unwrap();
}

#[test]
fn receiver_authorization_header_is_sent_and_write_only() {
    let f = encrypted_fixture();
    f.user("alice");
    let (endpoint, received) = push_server(vec![500, 202]);
    let secret_a = "Bearer receiver-secret-one";
    let secret_b = "Bearer receiver-secret-two";
    let make_configuration = |authorization_header: Option<&str>| ConfigurationInput {
        events_requested: [CREDENTIAL_CHANGE.into()].into(),
        delivery: DeliverySpec {
            method: PUSH.into(),
            endpoint_url: endpoint.clone(),
            authorization_header: authorization_header.map(str::to_owned),
        },
        description: None,
    };
    let auth = SsfAuth::Bearer(f.admin.clone());
    let created = f
        .core
        .ssf_config_create(&auth, make_configuration(Some(secret_a)))
        .unwrap();
    let id = created["stream_id"].as_str().unwrap();
    assert!(!created.to_string().contains(secret_a));
    assert!(
        !f.core
            .ssf_config_read(&auth, Some(id))
            .unwrap()
            .to_string()
            .contains(secret_a)
    );
    let subject = json!({"format":"iss_sub","iss":f.core.config.issuer,"sub":"external"});
    f.core
        .ssf_bind_subjects(
            &auth,
            id,
            SubjectBindings {
                subjects: [(subject.to_string(), "alice".into())].into(),
            },
        )
        .unwrap();
    f.core
        .ssf_config_update(
            &auth,
            json!({"stream_id":id,"delivery":{"method":PUSH,"endpoint_url":endpoint}}),
            false,
        )
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                password: Some("replacement-password-one".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let first = f.core.deliver_once().unwrap();
    assert_eq!(first[0]["status"], 500);
    assert!(received.lock().unwrap()[0].header.contains(secret_a));
    let queued = f.core.store.list::<Delivery>("ssf_deliveries").unwrap();
    assert!(!serde_json::to_string(&queued).unwrap().contains(secret_a));

    f.core
        .ssf_config_update(
            &auth,
            json!({"stream_id":id,"delivery":{"method":PUSH,"endpoint_url":endpoint,"authorization_header":secret_b}}),
            false,
        )
        .unwrap();
    let cancelled = f
        .core
        .store
        .get::<Delivery>("ssf_deliveries", first[0]["id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert!(cancelled.stopped);
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                password: Some("replacement-password-two".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let second = f.core.deliver_once().unwrap();
    assert_eq!(second[0]["status"], 202);
    let received = received.lock().unwrap();
    assert_eq!(received.len(), 2);
    assert!(received[1].header.contains(secret_b));
    assert!(!received[1].header.contains(secret_a));
    drop(received);
    let (new_endpoint, new_received) = push_server(vec![202]);
    f.core
        .ssf_config_update(
            &auth,
            json!({"stream_id":id,"delivery":{"method":PUSH,"endpoint_url":new_endpoint}}),
            false,
        )
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                password: Some("replacement-password-three".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let third = f.core.deliver_once().unwrap();
    assert_eq!(third[0]["status"], 202);
    assert!(
        !new_received.lock().unwrap()[0]
            .header
            .lines()
            .any(|line| line.to_ascii_lowercase().starts_with("authorization:"))
    );
    assert!(
        !f.core
            .ssf_config_read(&auth, Some(id))
            .unwrap()
            .to_string()
            .contains(secret_b)
    );
    assert!(
        !f.core
            .ssf_list(&auth)
            .unwrap()
            .to_string()
            .contains(secret_b)
    );
    assert!(
        !f.core
            .audit_events(&f.admin, 100)
            .unwrap()
            .to_string()
            .contains(secret_b)
    );
    let raw = std::fs::read(f._dir.path().join("riauth.redb")).unwrap();
    assert!(
        !raw.windows(secret_a.len())
            .any(|bytes| bytes == secret_a.as_bytes())
    );
    assert!(
        !raw.windows(secret_b.len())
            .any(|bytes| bytes == secret_b.as_bytes())
    );

    let unencrypted = Fixture::new();
    assert_eq!(
        unencrypted
            .core
            .ssf_config_create(
                &SsfAuth::Bearer(unencrypted.admin.clone()),
                make_configuration(Some(secret_a)),
            )
            .unwrap_err()
            .status,
        axum::http::StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn http_metadata_stream_and_push_routes() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let f = Fixture::new();
    f.user("alice");
    let app = riauth::api::router(f.core.clone());
    let metadata = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/.well-known/ssf-configuration")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(metadata.status(), axum::http::StatusCode::OK);
    let document: Value =
        serde_json::from_slice(&metadata.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(document["spec_version"], "1_0");
    assert_eq!(document["delivery_methods_supported"][0], PUSH);
    assert!(document.get("status_endpoint").is_none());
    assert!(
        document["events_supported"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event == ACCOUNT_DISABLED)
    );
    assert!(!document.to_string().contains("urn:ietf:rfc:8936"));
    let (key, jwks, _) = signer();
    let body = serde_json::to_vec(&stream(
        "http-stream",
        "https://transmitter.example",
        &f.core.config.issuer,
        "https://receiver.example/events",
        &[ACCOUNT_DISABLED],
        jwks,
        &[("ext-a", "alice")],
    ))
    .unwrap();
    let created = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ssf/admin/streams")
                .header("authorization", format!("Bearer {}", f.admin))
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created.status(), axum::http::StatusCode::CREATED);
    let token = set_token(
        &key,
        "https://transmitter.example",
        &f.core.config.issuer,
        "ext-a",
        ACCOUNT_DISABLED,
        "http-jti",
        crypto::now() + 120,
    );
    let accepted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ssf/events")
                .header("content-type", "application/secevent+jwt")
                .body(Body::from(token.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(accepted.status(), axum::http::StatusCode::ACCEPTED);
    assert!(
        accepted
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .is_empty()
    );
    let duplicate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ssf/events")
                .header("content-type", "application/secevent+jwt")
                .body(Body::from(token))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(duplicate.status(), axum::http::StatusCode::ACCEPTED);
    assert!(
        duplicate
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .is_empty()
    );
    let invalid = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ssf/events")
                .header("content-type", "text/plain")
                .body(Body::from("bad-token"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid.status(), axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(invalid.headers()["content-language"], "en");
    let invalid_body: Value =
        serde_json::from_slice(&invalid.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(invalid_body["err"], "invalid_request");
    assert!(invalid_body.get("description").is_some());
    assert!(invalid_body.get("error").is_none());
    assert!(!user_enabled(&f, "alice"));
}

#[tokio::test]
async fn push_errors_use_rfc8935_codes_even_for_oversized_bodies() {
    use axum::{body::Body, http::Request};
    use base64::Engine;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let f = Fixture::new();
    f.user("alice");
    let (key, jwks, _) = signer();
    let (other_key, _, _) = signer();
    let issuer = "https://transmitter.example";
    let audience = f.core.config.issuer.clone();
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            stream(
                "errors",
                issuer,
                &audience,
                "https://receiver.example/events",
                &[ACCOUNT_DISABLED],
                jwks,
                &[("external", "alice")],
            ),
        )
        .unwrap();
    let valid = set_token(
        &key,
        issuer,
        &audience,
        "external",
        ACCOUNT_DISABLED,
        "error-valid",
        crypto::now() + 120,
    );
    let mut pieces = valid.split('.').map(str::to_owned).collect::<Vec<_>>();
    let mut signature = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&pieces[2])
        .unwrap();
    signature[0] ^= 1;
    pieces[2] = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(signature);
    let bad_signature = pieces.join(".");
    let invalid_profile = key
        .sign_type(
            &json!({
                "iss":issuer,"aud":audience,"iat":crypto::now(),"jti":"profile-error",
                "sub":"forbidden-top-level-sub",
                "sub_id":{"format":"iss_sub","iss":issuer,"sub":"external"},
                "events":{ACCOUNT_DISABLED:{}}
            }),
            "secevent+jwt",
        )
        .unwrap();
    let invalid_event_payload = key
        .sign_type(
            &json!({
                "iss":issuer,"aud":audience,"iat":crypto::now(),"jti":"payload-error",
                "sub_id":{"format":"iss_sub","iss":issuer,"sub":"external"},
                "events":{ACCOUNT_DISABLED:null}
            }),
            "secevent+jwt",
        )
        .unwrap();
    let mismatched_nested_subject = key
        .sign_type(
            &json!({
                "iss":issuer,"aud":audience,"iat":crypto::now(),"jti":"nested-mismatch",
                "sub_id":{"format":"iss_sub","iss":issuer,"sub":"external"},
                "events":{ACCOUNT_DISABLED:{"subject":{"format":"iss_sub","iss":issuer,"sub":"other"}}}
            }),
            "secevent+jwt",
        )
        .unwrap();
    let missing_credential_fields = key
        .sign_type(
            &json!({
                "iss":issuer,"aud":audience,"iat":crypto::now(),"jti":"credential-fields",
                "sub_id":{"format":"iss_sub","iss":issuer,"sub":"external"},
                "events":{CREDENTIAL_CHANGE:{}}
            }),
            "secevent+jwt",
        )
        .unwrap();
    let cases = [
        ("invalid_request", "bad-token".to_owned()),
        (
            "invalid_issuer",
            set_token(
                &key,
                "https://unknown.example",
                &audience,
                "external",
                ACCOUNT_DISABLED,
                "unknown-issuer",
                crypto::now() + 120,
            ),
        ),
        (
            "invalid_audience",
            set_token(
                &key,
                issuer,
                "other-audience",
                "external",
                ACCOUNT_DISABLED,
                "wrong-audience",
                crypto::now() + 120,
            ),
        ),
        (
            "invalid_key",
            set_token(
                &other_key,
                issuer,
                &audience,
                "external",
                ACCOUNT_DISABLED,
                "unknown-key",
                crypto::now() + 120,
            ),
        ),
        ("authentication_failed", bad_signature),
        ("invalid_request", invalid_profile),
        ("invalid_request", invalid_event_payload),
        ("invalid_request", mismatched_nested_subject),
        ("invalid_request", missing_credential_fields),
        ("invalid_request", "x".repeat(40_000)),
    ];
    let app = riauth::api::router(f.core.clone());
    for (expected, token) in cases {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ssf/events")
                    .header("content-type", "application/secevent+jwt")
                    .body(Body::from(token))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(response.headers()["content-language"], "en");
        let body: Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        assert_eq!(body["err"], expected);
    }
    assert!(user_enabled(&f, "alice"));
    let corrected = set_token(
        &key,
        issuer,
        &audience,
        "external",
        ACCOUNT_DISABLED,
        "payload-error",
        crypto::now() + 120,
    );
    f.core.accept_set(&corrected).unwrap();
    assert!(!user_enabled(&f, "alice"));
}

#[tokio::test]
async fn advertised_configuration_endpoint_uses_receiver_and_transmitter_fields() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let f = Fixture::new();
    f.user("alice");
    let (endpoint, received) = push_server(vec![202]);
    let app = riauth::api::router(f.core.clone());
    let post = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ssf/streams")
                .header("authorization", format!("Bearer {}", f.admin))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "events_requested": [ACCOUNT_DISABLED, "https://example.com/unknown-event"],
                        "delivery": {"method": PUSH, "endpoint_url": endpoint},
                        "description": "Receiver A"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(post.status(), axum::http::StatusCode::CREATED);
    assert_eq!(post.headers()["cache-control"], "no-store");
    let created: Value =
        serde_json::from_slice(&post.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let id = created["stream_id"].as_str().unwrap().to_owned();
    assert!(!id.is_empty());
    assert_eq!(created["iss"], f.core.config.issuer);
    assert_eq!(created["events_requested"].as_array().unwrap().len(), 2);
    assert_eq!(created["events_delivered"], json!([ACCOUNT_DISABLED]));
    assert_eq!(created["delivery"]["endpoint_url"], endpoint);
    assert!(created["aud"].as_str().is_some());
    assert!(created.get("jwks").is_none());
    assert!(created.get("subjects").is_none());

    let read = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/ssf/streams?stream_id={id}"))
                .header("authorization", format!("Bearer {}", f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(read.status(), axum::http::StatusCode::OK);
    assert_eq!(read.headers()["cache-control"], "no-store");
    let current: Value =
        serde_json::from_slice(&read.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(current, created);

    let patched = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/ssf/streams")
                .header("authorization", format!("Bearer {}", f.admin))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"stream_id":id,"description":"Updated receiver"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patched.status(), axum::http::StatusCode::OK);
    let patched: Value =
        serde_json::from_slice(&patched.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(patched["description"], "Updated receiver");
    assert_eq!(patched["events_delivered"], json!([ACCOUNT_DISABLED]));

    let replaced = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/ssf/streams")
                .header("authorization", format!("Bearer {}", f.admin))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "stream_id": id, "events_requested": [ACCOUNT_DISABLED],
                        "delivery": {"method": PUSH, "endpoint_url": endpoint}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(replaced.status(), axum::http::StatusCode::OK);
    let replaced: Value =
        serde_json::from_slice(&replaced.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(replaced.get("description").is_none());

    let subject_key =
        json!({"format":"iss_sub", "iss": f.core.config.issuer, "sub":"external"}).to_string();
    let mut bindings = json!({"subjects": {}});
    bindings["subjects"][subject_key] = json!("alice");
    let bound = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/ssf/admin/streams/{id}/subjects"))
                .header("authorization", format!("Bearer {}", f.admin))
                .header("content-type", "application/json")
                .body(Body::from(bindings.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bound.status(), axum::http::StatusCode::OK);
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let deliver_core = f.core.clone();
    let delivered = tokio::task::spawn_blocking(move || deliver_core.deliver_once())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(delivered[0]["status"], 202);
    let sent = received.lock().unwrap().pop().unwrap();
    let claims: Value = {
        use base64::Engine;
        let middle = sent.body.split('.').nth(1).unwrap();
        serde_json::from_slice(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(middle)
                .unwrap(),
        )
        .unwrap()
    };
    assert_eq!(claims["aud"], created["aud"]);
    assert_eq!(claims["sub_id"]["iss"], f.core.config.issuer);
    assert_eq!(claims["sub_id"]["sub"], "external");
    assert!(claims.get("sub").is_none() && claims.get("exp").is_none());

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/ssf/streams")
                .header("authorization", format!("Bearer {}", f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let listed: Value =
        serde_json::from_slice(&list.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(listed, json!([replaced]));

    let deleted = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/ssf/streams?stream_id={id}"))
                .header("authorization", format!("Bearer {}", f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(deleted.status(), axum::http::StatusCode::NO_CONTENT);
    assert!(
        deleted
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .is_empty()
    );
    let empty = app
        .oneshot(
            Request::builder()
                .uri("/api/ssf/streams")
                .header("authorization", format!("Bearer {}", f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let empty_list: Value =
        serde_json::from_slice(&empty.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(empty_list, json!([]));
}

#[test]
fn password_entry_points_enqueue_once_and_failed_changes_rollback() {
    use riauth::lifecycle::{MailConfig, MailSecurity, Purpose};
    for path in [
        "admin",
        "self",
        "email-reset",
        "manifest",
        "scim",
        "recovery",
    ] {
        let mut f = Fixture::new();
        let session;
        let scim_id = if path == "scim" {
            let created = f.core.scim_write(&f.admin, "Users", None,
                json!({"schemas":[riauth::scim::USER],"userName":"alice","displayName":"Alice","password":common::PASSWORD,"active":true}), false).unwrap();
            session = common::text(
                &f.core
                    .login("alice".into(), common::PASSWORD.into(), None)
                    .unwrap(),
                "session_token",
            );
            Some(created["id"].as_str().unwrap().to_owned())
        } else {
            session = f.user("alice");
            None
        };
        subscribe(&f, "alice");
        let changed = "changed-password-for-security-events";
        match path {
            "admin" => {
                f.core
                    .update_user(
                        &f.admin,
                        "alice",
                        UserPatch {
                            password: Some(changed.into()),
                            ..Default::default()
                        },
                    )
                    .unwrap();
            }
            "self" => {
                f.core
                    .change_password(&session, common::PASSWORD.into(), changed.into(), None)
                    .unwrap();
            }
            "email-reset" => {
                f.core.config.mail = Some(MailConfig {
                    host: "127.0.0.1".into(),
                    port: 2525,
                    from: "Identity <identity@example.test>".into(),
                    security: MailSecurity::Loopback,
                    username: None,
                    password_file: None,
                });
                f.core
                    .update_user(
                        &f.admin,
                        "alice",
                        UserPatch {
                            email_verified: Some(true),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                f.core.account_reset_request("alice").unwrap();
                let code = f
                    .core
                    .store
                    .list::<Value>("mail_deliveries")
                    .unwrap()
                    .into_iter()
                    .find_map(|(_, delivery)| {
                        delivery["body"].as_str().and_then(|body| {
                            body.lines()
                                .find(|line| line.starts_with("ri_mail_"))
                                .map(str::to_owned)
                        })
                    })
                    .unwrap();
                f.core
                    .account_complete(code, Purpose::Reset, Some(changed.into()))
                    .unwrap();
            }
            "manifest" => {
                let plan = f.core.plan_state(&f.admin, serde_json::from_value(json!({"api_version":"riauth/v1","users":[{"username":"alice","display_name":"Alice","password_ref":"env:PW","password_version":"v2"}]})).unwrap()).unwrap();
                assert!(events(&f, "alice").is_empty());
                for _ in 0..2 {
                    f.core
                        .apply_state(
                            &f.admin,
                            riauth::state::ApplyRequest {
                                plan: plan.clone(),
                                secrets: [("env:PW".into(), changed.into())].into(),
                                run_id: None,
                            },
                        )
                        .unwrap();
                }
            }
            "scim" => {
                f.core.scim_write(&f.admin, "Users", scim_id.as_deref(), json!({"schemas":[riauth::scim::USER],"userName":"alice","password":changed,"active":true}), false).unwrap();
            }
            "recovery" => {
                f.core.recover_admin("alice", changed, false).unwrap();
            }
            _ => unreachable!(),
        }
        assert_eq!(
            events(&f, "alice"),
            vec![(CREDENTIAL_CHANGE.into(), "password".into())],
            "{path}"
        );
        assert!(f.core.me(&session).is_err());
        assert!(
            f.core
                .update_user(
                    &f.admin,
                    "alice",
                    UserPatch {
                        password: Some(changed.into()),
                        ..Default::default()
                    }
                )
                .is_err()
        );
        assert_eq!(events(&f, "alice").len(), 1);
        let uid = f
            .core
            .store
            .get::<String>("usernames", "alice")
            .unwrap()
            .unwrap();
        let error = f.core.store.write::<()>(|tx| {
            let mut user: User = tx.get("users", &uid)?.unwrap();
            user.enabled = false;
            tx.put("users", &uid, &user)?;
            Err(riauth::error::Error::conflict("force transaction rollback"))
        });
        assert!(error.is_err());
        assert!(user_enabled(&f, "alice"));
        assert_eq!(events(&f, "alice").len(), 1);
    }
}

#[test]
fn factor_events_skip_authentication_consumption_and_deduplicate_bulk_removal() {
    let f = Fixture::new();
    let session = f.user("alice");
    subscribe(&f, "alice");
    let enrollment = f.core.mfa_begin(&session).unwrap();
    assert!(events(&f, "alice").is_empty());
    let secret = enrollment["secret"].as_str().unwrap();
    let totp = crypto::totp(secret, "alice").unwrap();
    f.core
        .mfa_confirm(&session, &totp.generate(crypto::now() - 30))
        .unwrap();
    assert_eq!(
        events(&f, "alice"),
        vec![(CREDENTIAL_CHANGE.into(), "otp".into())]
    );
    let session = common::text(
        &f.core
            .login(
                "alice".into(),
                common::PASSWORD.into(),
                Some(totp.generate(crypto::now())),
            )
            .unwrap(),
        "session_token",
    );
    assert_eq!(events(&f, "alice").len(), 1);
    let codes = f.core.recovery_codes(&session).unwrap();
    let code = codes["recovery_codes"][0].as_str().unwrap();
    f.core
        .login("alice".into(), common::PASSWORD.into(), Some(code.into()))
        .unwrap();
    assert_eq!(events(&f, "alice").len(), 2);
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                reset_mfa: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        events(&f, "alice")
            .iter()
            .filter(|(_, kind)| kind == "otp")
            .count(),
        2
    );
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                reset_mfa: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        events(&f, "alice").len(),
        3,
        "clearing already-empty factors is not a credential change"
    );
}

#[test]
fn bulk_passkey_reset_enqueues_one_signal_and_authentication_is_silent() {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
    let f = Fixture::new();
    let mut session = f.user("alice");
    subscribe(&f, "alice");
    for _ in 0..2 {
        let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
        let pending = f
            .core
            .passkey_register_start(&session, "Test authenticator".into())
            .unwrap();
        let response = authenticator
            .do_registration(
                url::Url::parse(&f.core.config.issuer).unwrap(),
                serde_json::from_value(pending["public_key"].clone()).unwrap(),
            )
            .unwrap();
        f.core
            .passkey_register_finish(&session, pending["ceremony"].as_str().unwrap(), response)
            .unwrap();
        // Adding another factor needs an MFA session, so sign in with the new passkey.
        let start = f.core.passkey_login_start("alice", None).unwrap();
        let proof = authenticator
            .do_authentication(
                url::Url::parse(&f.core.config.issuer).unwrap(),
                serde_json::from_value(start["public_key"].clone()).unwrap(),
            )
            .unwrap();
        session = common::text(
            &f.core
                .passkey_login_finish(start["ceremony"].as_str().unwrap(), proof)
                .unwrap(),
            "session_token",
        );
    }
    assert_eq!(
        events(&f, "alice"),
        vec![(CREDENTIAL_CHANGE.into(), "public-key".into()); 2]
    );
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                reset_mfa: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        events(&f, "alice"),
        vec![(CREDENTIAL_CHANGE.into(), "public-key".into()); 3]
    );
    assert!(f.core.store.list::<Value>("passkeys").unwrap().is_empty());
}

#[test]
fn invitation_completion_enqueues_the_new_credential_once() {
    use riauth::lifecycle::{Invitation, MailConfig, MailSecurity, Purpose};
    let mut f = Fixture::new();
    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
    f.core
        .account_invite(
            &f.admin,
            Invitation {
                username: "alice".into(),
                email: "alice@example.test".into(),
                display_name: "Alice".into(),
                groups: Default::default(),
            },
        )
        .unwrap();
    subscribe(&f, "alice");
    let code = f
        .core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .find_map(|(_, delivery)| {
            delivery["body"].as_str().and_then(|body| {
                body.lines()
                    .find(|line| line.starts_with("ri_mail_"))
                    .map(str::to_owned)
            })
        })
        .unwrap();
    f.core
        .account_complete(code.clone(), Purpose::Invite, Some(common::PASSWORD.into()))
        .unwrap();
    assert!(
        f.core
            .account_complete(code, Purpose::Invite, Some(common::PASSWORD.into()))
            .is_err()
    );
    assert_eq!(
        events(&f, "alice"),
        vec![(CREDENTIAL_CHANGE.into(), "password".into())]
    );
}

#[test]
fn reenabling_legacy_disabled_accounts_never_restores_child_credentials() {
    let f = Fixture::new();
    let session = f.user("alice");
    subscribe(&f, "alice");
    let children = Dependents::create(&f, "alice");
    let agent: riauth::agent::Agent = f.core.store.get("agents", "child-alice").unwrap().unwrap();
    let device: Value = f
        .core
        .store
        .get("windows_devices", "device-alice")
        .unwrap()
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    // Reproduce an old backup: the parent was disabled, but child rows and their
    // token index were retained enabled by the former cloud/SSF/offboard paths.
    f.core
        .store
        .write(|tx| {
            tx.put("agents", "child-alice", &agent)?;
            tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
            tx.put("windows_devices", "device-alice", &device)?;
            // Also model an old user write that failed to change the session epoch.
            let uid: String = tx.get("usernames", "alice")?.unwrap();
            let mut user: User = tx.get("users", &uid)?.unwrap();
            user.epoch = 0;
            // This user write repairs children; restore the stale children afterwards.
            tx.put("users", &uid, &user)?;
            tx.put("agents", "child-alice", &agent)?;
            tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
            tx.put("windows_devices", "device-alice", &device)
        })
        .unwrap();
    riauth::upgrade::migrate(&f.core.store).unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                enabled: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    children.assert_revoked(&f);
    assert!(f.core.me(&session).is_err());
    assert_eq!(
        events(&f, "alice"),
        vec![(ACCOUNT_DISABLED.into(), "".into())],
        "re-enable must not replay disable events"
    );
}
