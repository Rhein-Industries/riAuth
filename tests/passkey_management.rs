use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use riauth::{
    config::Config,
    core::Core,
    crypto::{digest, now},
    model::{NewUser, Session, User},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
use webauthn_rs::prelude::{PublicKeyCredential, RegisterPublicKeyCredential};

const PASSWORD: &str = "passkey-management-test-password";
const ORIGIN: &str = "http://localhost:9000";
type Authenticator = WebauthnAuthenticator<SoftPasskey>;

struct Fixture {
    _dir: tempfile::TempDir,
    core: Core,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::TempDir::new().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                issuer: ORIGIN.into(),
                ..Default::default()
            },
            NewUser {
                username: "alice".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Alice".into(),
                admin: true,
            },
        )
        .unwrap();
        Self { _dir: dir, core }
    }

    fn password_browser(&self, sso: Option<&str>) -> String {
        let reply = self
            .core
            .portal_password(sso, "alice".into(), PASSWORD.into(), None, false)
            .unwrap();
        cookie(&reply.cookies, "riauth_sso")
    }

    fn register(
        &self,
        authenticator: &mut Authenticator,
        start: &Value,
    ) -> RegisterPublicKeyCredential {
        let mut options = start["public_key"].clone();
        // The software fixture creates a non-resident credential.
        options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
        authenticator
            .do_registration(
                ORIGIN.parse().unwrap(),
                serde_json::from_value(options).unwrap(),
            )
            .unwrap()
    }

    fn enroll(&self) -> (Authenticator, String) {
        let sso = self.password_browser(None);
        let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
        let start = self
            .core
            .portal_passkey_register_start(Some(&sso), "Laptop".into())
            .unwrap();
        let proof = self.register(&mut authenticator, &start);
        let raw_id = serde_json::to_value(&proof).unwrap()["rawId"]
            .as_str()
            .unwrap()
            .to_owned();
        self.core
            .portal_passkey_register_finish(Some(&sso), text(&start, "ceremony"), proof)
            .unwrap();
        (authenticator, raw_id)
    }

    fn passkey_browser(&self, authenticator: &mut Authenticator, raw_id: &str) -> String {
        let start = self.core.portal_passkey_start(None, false).unwrap();
        let mut options = start.body["public_key"].clone();
        options["publicKey"]["allowCredentials"] = json!([{"type":"public-key","id":raw_id}]);
        let proof = authenticator
            .do_authentication(
                ORIGIN.parse().unwrap(),
                serde_json::from_value(options).unwrap(),
            )
            .unwrap();
        let mut proof = serde_json::to_value(proof).unwrap();
        let handle = Sha256::digest(format!("riauth.webauthn-user/v1\0{}", self.user().id));
        proof["response"]["userHandle"] = json!(URL_SAFE_NO_PAD.encode(&handle[..16]));
        let reply = self
            .core
            .portal_passkey_finish(
                None,
                Some(&cookie(&start.cookies, "riauth_passkey")),
                text(&start.body, "ceremony"),
                serde_json::from_value(proof).unwrap(),
            )
            .unwrap();
        cookie(&reply.cookies, "riauth_sso")
    }

    fn user(&self) -> User {
        let id: String = self.core.store.get("usernames", "alice").unwrap().unwrap();
        self.core.store.get("users", &id).unwrap().unwrap()
    }

    fn session_id(&self, sso: &str) -> String {
        self.core
            .store
            .get::<Value>("browser_sessions", &digest(sso))
            .unwrap()
            .unwrap()["session_id"]
            .as_str()
            .unwrap()
            .into()
    }

    fn age(&self, sso: &str, seconds: u64) {
        let id = self.session_id(sso);
        self.core
            .store
            .write(|tx| {
                let mut session: Session = tx.get("sessions", &id)?.unwrap();
                session.identity.auth_time = now() - seconds;
                tx.put("sessions", &id, &session)
            })
            .unwrap();
    }
}

fn text<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field].as_str().unwrap()
}
fn cookie(cookies: &[String], name: &str) -> String {
    cookies
        .iter()
        .find_map(|cookie| cookie.strip_prefix(&format!("{name}=")))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .into()
}

/// Real WebAuthn signatures must remain scoped to one live workflow attempt,
/// with credential use and W03 completion committed by only one writer.
#[cfg(feature = "platform")]
#[test]
fn workflow_passkey_is_bound_and_consumed_once() {
    use riauth::workflow::{Outcome, RunState};

    let f = Fixture::new();
    let (mut authenticator, _) = f.enroll();
    let login = |username: &str| {
        text(
            &f.core
                .login(username.into(), PASSWORD.into(), None)
                .unwrap(),
            "session_token",
        )
        .to_owned()
    };
    let alice = login("alice");
    let second = login("alice");
    f.core
        .create_user(
            &alice,
            NewUser {
                username: "bob".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Bob".into(),
                admin: false,
            },
        )
        .unwrap();
    let bob = login("bob");
    let sessions = f.core.store.list::<Value>("sessions").unwrap();
    let mut sign = |options: Value| {
        authenticator
            .do_authentication(
                ORIGIN.parse().unwrap(),
                serde_json::from_value(options).unwrap(),
            )
            .unwrap()
    };

    let cancelled = f.core.workflow_passkey_start(&alice).unwrap();
    let challenge = f
        .core
        .workflow_passkey_challenge(&alice, &cancelled.id)
        .unwrap();
    assert!(
        serde_json::to_value(&challenge)
            .unwrap()
            .get("ceremony")
            .is_none()
    );
    let old_response = sign(challenge.public_key);
    assert!(
        f.core
            .workflow_passkey_challenge(&alice, &cancelled.id)
            .is_err()
    );
    for token in [&bob, &second] {
        assert!(
            f.core
                .workflow_passkey(token, &cancelled.id, old_response.clone())
                .is_err()
        );
    }
    f.core.workflow_cancel(&alice, &cancelled.id).unwrap();
    assert!(
        f.core
            .store
            .list::<Value>("passkey_authentication")
            .unwrap()
            .is_empty()
    );

    let run_id = f.core.workflow_passkey_start(&alice).unwrap().id;
    f.core.workflow_passkey_challenge(&alice, &run_id).unwrap();
    let retry = f
        .core
        .workflow_passkey(&alice, &run_id, old_response)
        .unwrap();
    assert!(matches!(retry.state, RunState::Active { attempt: 2, .. }));
    assert!(
        f.core
            .store
            .list::<Value>("passkey_authentication")
            .unwrap()
            .is_empty()
    );
    assert!(
        f.core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .is_empty()
    );
    let challenge = f.core.workflow_passkey_challenge(&alice, &run_id).unwrap();
    let response = sign(challenge.public_key);
    let run: Value = f.core.store.get("workflow_runs", &run_id).unwrap().unwrap();
    let request_id = text(&run["record"], "request");
    let ceremony_key = digest(text(&run["in_flight"], "passkey"));
    let credentials = f.core.store.list::<Value>("passkeys").unwrap();

    // Changing authority must not spend the challenge, advance the credential
    // counter or manufacture evidence. Restoring these fixture rows then lets
    // the original, still-unconsumed response exercise the success transaction.
    for (bucket, key, pointer, value) in [
        (
            "workflow_requests",
            request_id,
            "/id",
            json!("another-request"),
        ),
        (
            "workflow_requests",
            request_id,
            "/expires_at",
            json!(now() - 1),
        ),
        (
            "workflow_runs",
            run_id.as_str(),
            "/record/state/attempt",
            json!(3),
        ),
    ] {
        let original: Value = f.core.store.get(bucket, key).unwrap().unwrap();
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &changed))
            .unwrap();
        assert!(
            f.core
                .workflow_passkey(&alice, &run_id, response.clone())
                .is_err(),
            "{bucket}{pointer}"
        );
        assert!(
            f.core
                .store
                .get::<Value>("passkey_authentication", &ceremony_key)
                .unwrap()
                .is_some()
        );
        assert_eq!(f.core.store.list::<Value>("passkeys").unwrap(), credentials);
        assert!(
            f.core
                .store
                .list::<Value>("workflow_evidence")
                .unwrap()
                .is_empty()
        );
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &original))
            .unwrap();
    }
    // A passkey proof must satisfy the existing request MFA floor itself.
    f.core
        .store
        .write(|tx| {
            let mut request: Value = tx.get("workflow_requests", request_id)?.unwrap();
            request["requires_mfa"] = json!(true);
            tx.put("workflow_requests", request_id, &request)
        })
        .unwrap();
    let outcomes = std::thread::scope(|scope| {
        let first = scope.spawn(|| f.core.workflow_passkey(&alice, &run_id, response.clone()));
        let second = scope.spawn(|| f.core.workflow_passkey(&alice, &run_id, response.clone()));
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    let finished = outcomes
        .into_iter()
        .find_map(std::result::Result::ok)
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    let evidence = f.core.store.list::<Value>("workflow_evidence").unwrap();
    assert_eq!(evidence.len(), 1);
    let receipt = &evidence[0].1;
    for field in ["account", "account_epoch", "session", "request", "binding"] {
        assert_eq!(receipt[field], run["record"][field], "{field}");
    }
    assert_eq!(receipt["run"], run_id);
    assert_eq!(receipt["step"], "passkey");
    assert_eq!(receipt["attempt"], 2);
    assert_eq!(receipt["proof"], "passkey");
    assert_eq!(receipt["consumed"], true);
    let used_credentials = f.core.store.list::<Value>("passkeys").unwrap();
    assert!(
        used_credentials[0].1["counter"].as_u64().unwrap()
            > credentials[0].1["counter"].as_u64().unwrap()
    );
    assert!(f.core.workflow_passkey(&alice, &run_id, response).is_err());
    assert_eq!(f.core.store.list::<Value>("sessions").unwrap(), sessions);
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );
    assert!(
        f.core
            .store
            .list::<Value>("passkey_authentication")
            .unwrap()
            .is_empty()
    );

    // A signature obtained before account-epoch revocation cannot finish a run.
    let revoked = f.core.workflow_passkey_start(&alice).unwrap();
    let challenge = f
        .core
        .workflow_passkey_challenge(&alice, &revoked.id)
        .unwrap();
    let response = sign(challenge.public_key);
    let user_id = f.user().id;
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &user_id)?.unwrap();
            user.epoch += 1;
            tx.put("users", &user.id, &user)
        })
        .unwrap();
    assert!(
        f.core
            .workflow_passkey(&alice, &revoked.id, response)
            .is_err()
    );
    assert_eq!(
        f.core.store.list::<Value>("workflow_evidence").unwrap(),
        evidence
    );
}

#[test]
fn enrollment_finish_rechecks_freshness_and_factor_then_spends_ceremony() {
    let f = Fixture::new();
    let sso = f.password_browser(None);
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f
        .core
        .portal_passkey_register_start(Some(&sso), "Laptop".into())
        .unwrap();
    let proof = f.register(&mut authenticator, &start);
    f.age(&sso, 301);
    let finish = || {
        f.core
            .portal_passkey_register_finish(Some(&sso), text(&start, "ceremony"), proof.clone())
    };
    assert_eq!(finish().err().unwrap().code, "reauthentication_required");
    assert_eq!(finish().err().unwrap().status, StatusCode::UNAUTHORIZED);
    assert!(f.core.store.list::<Value>("passkeys").unwrap().is_empty());

    let (mut enrolled, raw_id) = f.enroll();
    let sso = f.passkey_browser(&mut enrolled, &raw_id);
    let start = f
        .core
        .portal_passkey_register_start(Some(&sso), "Backup".into())
        .unwrap();
    let proof = f.register(&mut authenticator, &start);
    // Another tab can re-authenticate the same session with only a password while
    // the authenticator prompt is open. Its old MFA authorization must not survive.
    let downgraded = f.password_browser(Some(&sso));
    assert_eq!(f.session_id(&sso), f.session_id(&downgraded));
    let finish = || {
        f.core.portal_passkey_register_finish(
            Some(&downgraded),
            text(&start, "ceremony"),
            proof.clone(),
        )
    };
    assert_eq!(finish().err().unwrap().code, "mfa_required");
    assert_eq!(finish().err().unwrap().status, StatusCode::UNAUTHORIZED);
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
}

#[test]
fn rename_requires_fresh_mfa_preserves_key_and_last_passkey_is_protected() {
    let f = Fixture::new();
    let (mut authenticator, raw_id) = f.enroll();
    let id = f.core.store.list::<Value>("passkeys").unwrap().remove(0).0;
    let password = f.password_browser(None);
    assert_eq!(
        f.core
            .portal_passkey_rename(Some(&password), &id, "Phone".into())
            .unwrap_err()
            .code,
        "mfa_required"
    );
    let sso = f.passkey_browser(&mut authenticator, &raw_id);
    f.age(&sso, 301);
    assert_eq!(
        f.core
            .portal_passkey_rename(Some(&sso), &id, "Phone".into())
            .unwrap_err()
            .code,
        "reauthentication_required"
    );
    f.age(&sso, 0);
    for invalid in ["", "   ", "Laptop\nkey"] {
        assert!(
            f.core
                .portal_passkey_rename(Some(&sso), &id, invalid.into())
                .is_err()
        );
    }
    let epoch = f.user().epoch;
    let mut before = f.core.store.get::<Value>("passkeys", &id).unwrap().unwrap();
    let renamed = f
        .core
        .portal_passkey_rename(Some(&sso), &id, "Phone".into())
        .unwrap();
    assert_eq!(renamed["sessions_revoked"], false);
    assert_eq!(renamed["passkey"]["id"], id);
    // The only persisted credential change is its label: key, counters, owner and
    // creation time remain byte-for-byte equal in their serialized representation.
    before["name"] = json!("Phone");
    assert_eq!(
        f.core.store.get::<Value>("passkeys", &id).unwrap().unwrap(),
        before
    );
    assert_eq!(f.user().epoch, epoch);
    assert!(f.core.portal_apps(Some(&sso)).is_ok());
    let user_id = f.user().id;
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &user_id)?.unwrap();
            user.password_hash.clear();
            tx.put("users", &user.id, &user)
        })
        .unwrap();
    let list = f.core.portal_passkeys(Some(&sso)).unwrap();
    assert_eq!(list["password_available"], false);
    assert_eq!(list["passkey_only"], true);
    assert_eq!(list["passkeys"][0]["removable"], false);
    assert_eq!(
        f.core
            .portal_passkey_remove(Some(&sso), &id)
            .err()
            .unwrap()
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(f.user().epoch, epoch);
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
}

#[test]
fn pinned_passkey_reauthentication_cannot_restore_a_changed_or_revoked_session() {
    let f = Fixture::new();
    let (mut authenticator, raw_id) = f.enroll();
    for revoke in [false, true] {
        let sso = f.passkey_browser(&mut authenticator, &raw_id);
        let start = f.core.portal_passkey_start(Some(&sso), true).unwrap();
        let proof: PublicKeyCredential = authenticator
            .do_authentication(
                ORIGIN.parse().unwrap(),
                serde_json::from_value(start.body["public_key"].clone()).unwrap(),
            )
            .unwrap();
        let session_id = f.session_id(&sso);
        if revoke {
            f.core.portal_sign_out(Some(&sso)).unwrap();
        } else {
            // A different live browser session for the same account is not the
            // session that authorized this pinned ceremony.
            let other = f.password_browser(None);
            let other_id = f.session_id(&other);
            assert_ne!(session_id, other_id);
            f.core
                .store
                .write(|tx| {
                    let mut mapping: Value = tx.get("browser_sessions", &digest(&sso))?.unwrap();
                    mapping["session_id"] = json!(other_id);
                    tx.put("browser_sessions", &digest(&sso), &mapping)
                })
                .unwrap();
        }
        let sessions_before = f.core.store.list::<Session>("sessions").unwrap().len();
        let binding = cookie(&start.cookies, "riauth_passkey");
        let finish = || {
            f.core.portal_passkey_finish(
                Some(&sso),
                Some(&binding),
                text(&start.body, "ceremony"),
                proof.clone(),
            )
        };
        assert_eq!(finish().err().unwrap().status, StatusCode::UNAUTHORIZED);
        assert_eq!(finish().err().unwrap().status, StatusCode::UNAUTHORIZED);
        assert!(
            f.core
                .store
                .list::<Value>("browser_logins")
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            f.core.store.list::<Session>("sessions").unwrap().len(),
            sessions_before
        );
    }
}
