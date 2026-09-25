use super::{Fixture, PASSWORD, text};
use riauth::{
    agent::{Agent, NewAgent, Permission},
    crypto::SigningKey,
    jose::PublicJwks,
    model::UserPatch,
    ssf::{
        ACCOUNT_DISABLED, CREDENTIAL_CHANGE, Delivery, DeliverySpec, PUSH, SsfAuth, StreamInput,
    },
    windows_login::{EnrollDevice, WindowsLogin},
};
use serde_json::Value;

pub fn subscribe(f: &Fixture, username: &str) {
    let key = SigningKey::generate_algorithm("ES256").unwrap();
    f.core
        .ssf_create(
            &SsfAuth::Bearer(f.admin.clone()),
            StreamInput {
                id: format!("watch-{username}"),
                issuer: "https://security.example".into(),
                audience: "receiver".into(),
                events_requested: [ACCOUNT_DISABLED.into(), CREDENTIAL_CHANGE.into()].into(),
                events: Default::default(),
                delivery: Some(DeliverySpec {
                    method: PUSH.into(),
                    endpoint_url: "https://receiver.example/events".into(),
                    authorization_header: None,
                }),
                delivery_method: None,
                endpoint_url: None,
                jwks: PublicJwks {
                    keys: vec![serde_json::from_value(key.jwk().unwrap()).unwrap()],
                },
                subjects: [(username.into(), username.into())].into(),
            },
        )
        .unwrap();
}

pub fn events(f: &Fixture, username: &str) -> Vec<(String, String)> {
    let mut events = f
        .core
        .store
        .list::<Delivery>("ssf_deliveries")
        .unwrap()
        .into_iter()
        .filter(|(_, delivery)| {
            delivery.subject == username
                || serde_json::from_str::<Value>(&delivery.subject)
                    .ok()
                    .and_then(|subject| subject["sub"].as_str().map(str::to_owned))
                    .as_deref()
                    == Some(username)
        })
        .map(|(_, delivery)| (delivery.event, delivery.credential_type))
        .collect::<Vec<_>>();
    events.sort();
    events
}

pub struct Dependents {
    username: String,
    agent_token: String,
    device_secret: String,
}

impl Dependents {
    pub fn create(f: &Fixture, username: &str) -> Self {
        let created = f
            .core
            .create_agent(
                &f.admin,
                NewAgent {
                    id: format!("child-{username}"),
                    parent: Some(username.into()),
                    ttl: 3600,
                    permissions: vec![Permission {
                        action: "user.read".into(),
                        resource: "*".into(),
                    }],
                },
            )
            .unwrap();
        let device = f
            .core
            .windows_device_enroll(
                &f.admin,
                EnrollDevice {
                    id: format!("device-{username}"),
                    display_name: "Test device".into(),
                    username: username.into(),
                    offline_ttl: None,
                },
            )
            .unwrap();
        Self {
            username: username.into(),
            agent_token: text(&created["credential"], "token"),
            device_secret: text(&device, "device_secret"),
        }
    }

    pub fn assert_revoked(&self, f: &Fixture) {
        assert!(f.core.me(&self.agent_token).is_err());
        assert!(
            !f.core
                .store
                .get::<Agent>("agents", &format!("child-{}", self.username))
                .unwrap()
                .unwrap()
                .enabled
        );
        assert_eq!(
            f.core
                .store
                .get::<Value>("windows_devices", &format!("device-{}", self.username))
                .unwrap()
                .unwrap()["revoked"],
            true
        );
        assert!(
            f.core
                .store
                .list::<Value>("audit")
                .unwrap()
                .iter()
                .any(|(_, event)| {
                    event["actor"] != "user-transition"
                        && event["details"]["changes"]
                            .as_array()
                            .is_some_and(|changes| {
                                changes.iter().any(|change| {
                                    change["resource"]
                                        == format!("windows_devices/device-{}", self.username)
                                        && change["after"]["revoked"] == true
                                        && change["after"]["secret_hash"] == "[redacted]"
                                })
                            })
                }),
            "device revocation must remain attributed to the original mutation actor"
        );
    }

    pub fn assert_revoked_after_reenable(&self, f: &Fixture) {
        self.assert_revoked(f);
        f.core
            .update_user(
                &f.admin,
                &self.username,
                UserPatch {
                    enabled: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        self.assert_revoked(f);
        assert!(
            f.core
                .windows_login(WindowsLogin {
                    device_id: format!("device-{}", self.username),
                    device_secret: self.device_secret.clone(),
                    username: self.username.clone(),
                    password: Some(PASSWORD.into()),
                    otp: None,
                    reauth_session: None,
                })
                .is_err()
        );
    }
}
