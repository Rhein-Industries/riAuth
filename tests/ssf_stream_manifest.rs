#[cfg(feature = "platform")]
#[path = "common/mod.rs"]
mod common;

#[cfg(feature = "platform")]
mod platform {
    use super::common::{Fixture, PASSWORD};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use riauth::{
        agent::{NewAgent, Permission},
        config::Config,
        connector_guard::ReconciliationMode,
        core::Core,
        crypto::{self, SigningKey},
        delegation::{GrantInput, HumanRole},
        jose::PublicJwks,
        model::NewUser,
        ssf::{
            ACCOUNT_DISABLED, ConfigurationInput, Delivery, DeliverySpec, PUSH, SsfAuth, Stream,
            StreamInput,
        },
        state::{ApplyRequest, DelegatedGrantSpec, Manifest, SsfStreamSpec},
        workflow::Definition,
    };
    use serde_json::{Value, json};
    use std::collections::{BTreeMap, BTreeSet};
    use tempfile::TempDir;
    use tower::ServiceExt;

    const SECRET: &str = "Bearer ssf-manifest-secret-do-not-leak";

    fn revision(f: &Fixture) -> u64 {
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    }

    fn admin_id(f: &Fixture) -> String {
        f.core
            .store
            .get::<String>("usernames", "admin")
            .unwrap()
            .unwrap()
    }

    fn user_id(f: &Fixture, username: &str) -> String {
        f.core
            .store
            .get::<String>("usernames", username)
            .unwrap()
            .unwrap()
    }

    fn stored(f: &Fixture, id: &str) -> Option<Stream> {
        f.core.store.get("ssf_streams", id).unwrap()
    }

    fn signer() -> PublicJwks {
        let key = SigningKey::generate_algorithm("ES256").unwrap();
        let jwk = serde_json::from_value(key.jwk().unwrap()).unwrap();
        PublicJwks { keys: vec![jwk] }
    }

    fn spec(
        id: &str,
        method: &str,
        endpoint: &str,
        jwks: PublicJwks,
        subjects: &[(&str, &str)],
    ) -> SsfStreamSpec {
        SsfStreamSpec {
            id: id.into(),
            issuer: "https://transmitter.example".into(),
            audience: "https://idp.example".into(),
            events_requested: BTreeSet::from([ACCOUNT_DISABLED.into()]),
            delivery_method: method.into(),
            endpoint_url: endpoint.into(),
            jwks,
            subjects: subjects
                .iter()
                .map(|(subject, username)| ((*subject).into(), (*username).into()))
                .collect(),
        }
    }

    fn manifest(streams: Vec<SsfStreamSpec>) -> Manifest {
        Manifest {
            api_version: "riauth/v1".into(),
            ssf_streams: streams,
            ..Default::default()
        }
    }

    fn direct_input(sample: &SsfStreamSpec, authorization: Option<&str>) -> StreamInput {
        StreamInput {
            id: sample.id.clone(),
            issuer: sample.issuer.clone(),
            audience: sample.audience.clone(),
            events_requested: sample.events_requested.clone(),
            events: BTreeSet::new(),
            delivery: Some(DeliverySpec {
                method: PUSH.into(),
                endpoint_url: sample.endpoint_url.clone(),
                authorization_header: authorization.map(str::to_owned),
            }),
            delivery_method: None,
            endpoint_url: None,
            jwks: sample.jwks.clone(),
            subjects: sample.subjects.clone(),
        }
    }

    fn apply(f: &Fixture, manifest: Manifest) -> Value {
        let plan = f.core.plan_state(&f.admin, manifest).unwrap();
        f.core
            .apply_state(
                &f.admin,
                ApplyRequest {
                    plan,
                    secrets: Default::default(),
                    run_id: Some("m07-ssf-streams".into()),
                },
            )
            .unwrap()
    }

    fn plan(f: &Fixture, manifest: Manifest) -> riauth::state::Plan {
        f.core.plan_state(&f.admin, manifest).unwrap()
    }

    fn actions(f: &Fixture, action: &str) -> usize {
        f.core
            .audit_events(&f.admin, 400)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action)
            .count()
    }

    fn encrypted() -> Fixture {
        let dir = TempDir::new().unwrap();
        let key_file = dir.path().join("database.key");
        riauth::config::write_private(&key_file, crypto::random_token("").as_bytes(), false)
            .unwrap();
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

    fn plant(f: &Fixture, id: &str, stream_id: &str) {
        let delivery = Delivery {
            id: id.into(),
            stream_id: stream_id.into(),
            uri: "https://receiver.example/events".into(),
            event: ACCOUNT_DISABLED.into(),
            subject: "subject".into(),
            audience: "https://idp.example".into(),
            credential_type: "password".into(),
            created_at: 1,
            next_attempt: 1,
            attempts: 0,
            delivered_at: None,
            last_status: None,
            last_failed: false,
            stopped: false,
            jti: id.into(),
        };
        f.core
            .store
            .write(|tx| tx.put("ssf_deliveries", id, &delivery))
            .unwrap();
    }

    fn stopped(f: &Fixture, id: &str) -> bool {
        f.core
            .store
            .get::<Delivery>("ssf_deliveries", id)
            .unwrap()
            .unwrap()
            .stopped
    }

    fn agent(f: &Fixture, id: &str, action: &str, resource: &str) -> String {
        f.core
            .create_agent(
                &f.admin,
                NewAgent {
                    id: id.into(),
                    permissions: vec![Permission {
                        action: action.into(),
                        resource: resource.into(),
                    }],
                    ttl: 3600,
                    parent: None,
                },
            )
            .unwrap()["credential"]["token"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn assert_public(value: &Value) {
        let text = value.to_string();
        assert!(!text.contains(SECRET));
        assert!(!text.contains("authorization_header"));
        assert!(!text.contains("\"d\""));
    }

    #[test]
    fn manifest_create_matches_admin_writer_and_replays_once() {
        let jwks = signer();
        let sample = spec(
            "tenant-a",
            "push",
            "https://receiver.example/events",
            jwks,
            &[("ext-a", "alice")],
        );
        let direct = Fixture::new();
        direct.user("alice");
        let created = direct
            .core
            .ssf_create(
                &SsfAuth::Bearer(direct.admin.clone()),
                direct_input(&sample, None),
            )
            .unwrap();
        assert!(created.get("authorization_header").is_none());
        assert!(created.get("jwks").is_none());
        assert_eq!(actions(&direct, "ssf.stream.create"), 1);
        assert_eq!(actions(&direct, "ssf.stream.reconcile"), 0);

        let manifest_fx = Fixture::new();
        manifest_fx.user("alice");
        let base = revision(&manifest_fx);
        let planned = plan(&manifest_fx, manifest(vec![sample.clone()]));
        assert!(stored(&manifest_fx, "tenant-a").is_none());
        assert_eq!(planned.changes.len(), 1);
        assert_eq!(planned.changes[0].resource, "ssf.stream/tenant-a");
        assert_eq!(planned.changes[0].action, "create");
        assert_eq!(planned.changes[0].before, Value::Null);
        assert_eq!(planned.changes[0].after["delivery_method"], PUSH);
        assert!(!planned.changes[0].credential_change);
        assert!(planned.changes[0].secret_references.is_empty());
        assert_public(&serde_json::to_value(&planned).unwrap());
        assert!(planned.group_dependencies.is_none());
        assert!(planned.client_dependencies.is_none());
        assert!(planned.user_dependencies.is_none());
        assert!(planned.client_description_dependencies.is_none());
        let first = manifest_fx
            .core
            .apply_state(
                &manifest_fx.admin,
                ApplyRequest {
                    plan: planned.clone(),
                    secrets: Default::default(),
                    run_id: Some("m07-ssf-streams".into()),
                },
            )
            .unwrap();
        let second = manifest_fx
            .core
            .apply_state(
                &manifest_fx.admin,
                ApplyRequest {
                    plan: planned,
                    secrets: Default::default(),
                    run_id: Some("m07-ssf-streams".into()),
                },
            )
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(revision(&manifest_fx), base + 1);
        assert_eq!(actions(&manifest_fx, "ssf.stream.reconcile"), 1);
        assert_eq!(actions(&manifest_fx, "ssf.stream.create"), 0);
        assert_public(&first);

        let left = stored(&direct, "tenant-a").unwrap();
        let right = stored(&manifest_fx, "tenant-a").unwrap();
        assert_eq!(left.issuer, right.issuer);
        assert_eq!(left.audience, right.audience);
        assert_eq!(left.events, right.events);
        assert_eq!(left.events_requested, right.events_requested);
        assert_eq!(left.delivery_method, PUSH);
        assert_eq!(right.delivery_method, PUSH);
        assert_eq!(left.endpoint_url, right.endpoint_url);
        assert_eq!(left.jwks, right.jwks);
        assert!(left.authorization_header.is_none());
        assert!(right.authorization_header.is_none());
        assert!(!left.standard && !right.standard);
        assert!(left.description.is_none() && right.description.is_none());
        assert_eq!(left.owner, admin_id(&direct));
        assert_eq!(right.owner, admin_id(&manifest_fx));
        assert_eq!(
            left.subjects.keys().collect::<Vec<_>>(),
            right.subjects.keys().collect::<Vec<_>>()
        );
        assert_eq!(
            left.subjects.values().next().unwrap(),
            &user_id(&direct, "alice")
        );
        assert_eq!(
            right.subjects.values().next().unwrap(),
            &user_id(&manifest_fx, "alice")
        );
    }

    #[test]
    fn export_round_trip_is_a_noop_and_hides_secrets() {
        let f = Fixture::new();
        f.user("alice");
        let jwks = signer();
        let sample = spec(
            "tenant-a",
            "push",
            "https://receiver.example/events",
            jwks.clone(),
            &[("ext-a", "alice")],
        );
        apply(&f, manifest(vec![sample]));
        let exported = f.core.export_state(&f.admin).unwrap();
        assert_public(&exported);
        let streams = exported["manifest"]["ssf_streams"].as_array().unwrap();
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0]["delivery_method"], PUSH);
        assert_eq!(streams[0]["jwks"], serde_json::to_value(&jwks).unwrap());
        let subjects = streams[0]["subjects"].as_object().unwrap();
        assert_eq!(subjects.len(), 1);
        let (key, username) = subjects.iter().next().unwrap();
        assert_eq!(username, "alice");
        assert!(key.starts_with('{'));
        assert!(key.contains("ext-a"));
        let round_trip: Manifest = serde_json::from_value(exported["manifest"].clone()).unwrap();
        let again = plan(
            &f,
            Manifest {
                api_version: "riauth/v1".into(),
                ssf_streams: round_trip.ssf_streams,
                ..Default::default()
            },
        );
        assert!(again.changes.is_empty());
        let alias = plan(
            &f,
            manifest(vec![spec(
                "tenant-a",
                "push",
                "https://receiver.example/events",
                jwks,
                &[("ext-a", "alice")],
            )]),
        );
        assert!(alias.changes.is_empty());
    }

    #[test]
    fn stale_plan_conflicts_and_leaves_the_stream_absent() {
        let f = Fixture::new();
        f.user("alice");
        let sample = spec(
            "tenant-a",
            PUSH,
            "https://receiver.example/events",
            signer(),
            &[("ext-a", "alice")],
        );
        let planned = plan(&f, manifest(vec![sample]));
        f.user("later");
        let error = f
            .core
            .apply_state(
                &f.admin,
                ApplyRequest {
                    plan: planned,
                    secrets: Default::default(),
                    run_id: None,
                },
            )
            .err()
            .unwrap();
        assert_eq!(error.code, "conflict");
        assert_eq!(
            error.message,
            "Connector plan expired or source configuration or local revision changed; create a new plan"
        );
        assert!(stored(&f, "tenant-a").is_none());
    }

    #[test]
    fn delegated_agents_and_receiver_streams_stay_outside_desired_state() {
        let f = Fixture::new();
        f.user("alice");
        f.user("pat");
        let jwks = signer();
        let receiver = f
            .core
            .ssf_config_create(
                &SsfAuth::Bearer(f.admin.clone()),
                ConfigurationInput {
                    events_requested: BTreeSet::from([ACCOUNT_DISABLED.into()]),
                    delivery: DeliverySpec {
                        method: PUSH.into(),
                        endpoint_url: "https://receiver.example/standard".into(),
                        authorization_header: None,
                    },
                    description: Some("Receiver A".into()),
                },
            )
            .unwrap();
        let receiver_id = receiver["stream_id"].as_str().unwrap().to_owned();
        let before = stored(&f, &receiver_id).unwrap();
        assert!(before.standard);
        assert_eq!(before.owner, admin_id(&f));

        let configure = agent(&f, "configure", "ssf.configure", "*");
        let other = agent(&f, "other", "ssf.manage", "ssf/other");
        let exact = agent(&f, "exact", "ssf.manage", "ssf/tenant-a");
        let wildcard = agent(&f, "wildcard", "ssf.manage", "*");
        let sample = spec(
            "tenant-a",
            PUSH,
            "https://receiver.example/events",
            jwks.clone(),
            &[("ext-a", "alice")],
        );
        for token in [&configure, &other] {
            let error = f
                .core
                .plan_state(token, manifest(vec![sample.clone()]))
                .err()
                .unwrap();
            assert_eq!(error.code, "access_denied");
        }
        assert!(stored(&f, "tenant-a").is_none());
        apply_as(&f, &exact, manifest(vec![sample.clone()]));
        let created = stored(&f, "tenant-a").unwrap();
        assert_eq!(created.owner, "agent:exact");
        assert!(created.authorization_header.is_none());
        let exported = f.core.export_state(&exact).unwrap();
        assert_public(&exported);
        let visible = exported["manifest"]["ssf_streams"].as_array().unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0]["id"], "tenant-a");
        assert!(!exported.to_string().contains("receiver.example/standard"));

        let mut subjects = BTreeMap::new();
        subjects.insert(
            visible[0]["subjects"]
                .as_object()
                .unwrap()
                .keys()
                .next()
                .unwrap()
                .clone(),
            "alice".into(),
        );
        let mut updated = sample.clone();
        updated.subjects = subjects;
        updated.delivery_method = PUSH.into();
        apply_as(&f, &exact, manifest(vec![updated]));
        let after_subjects = stored(&f, "tenant-a").unwrap();
        assert_eq!(after_subjects.owner, "agent:exact");
        assert!(after_subjects.authorization_header.is_none());

        apply_as(
            &f,
            &wildcard,
            manifest(vec![spec(
                "tenant-b",
                PUSH,
                "https://receiver.example/events",
                jwks.clone(),
                &[("ext-a", "alice")],
            )]),
        );
        assert_eq!(stored(&f, "tenant-b").unwrap().owner, "agent:wildcard");

        let collision = spec(
            &receiver_id,
            PUSH,
            "https://receiver.example/events",
            jwks,
            &[("ext-a", "alice")],
        );
        let error = f
            .core
            .plan_state(&f.admin, manifest(vec![collision]))
            .err()
            .unwrap();
        assert_eq!(error.code, "conflict");
        assert_eq!(
            error.message,
            "Receiver-managed SSF streams stay on the SSF configuration API"
        );
        let after = stored(&f, &receiver_id).unwrap();
        assert_eq!(after.endpoint_url, before.endpoint_url);
        assert_eq!(after.description, before.description);
        assert_eq!(after.owner, before.owner);
        assert_eq!(after.authorization_header, before.authorization_header);
        assert!(after.subjects.is_empty());

        f.core
            .set_human_grants(
                &f.admin,
                "pat",
                vec![GrantInput {
                    role: HumanRole::HelpDesk,
                    scope: "user/alice".into(),
                }],
            )
            .unwrap();
        let delegated = f.core.login("pat".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let exported = f.core.export_state(&delegated).unwrap();
        assert!(exported["manifest"].get("ssf_streams").is_none());
        assert!(!exported.to_string().contains("tenant-a"));
        let error = f
            .core
            .plan_state(&delegated, manifest(vec![sample]))
            .err()
            .unwrap();
        assert_eq!(error.code, "access_denied");
    }

    fn apply_as(f: &Fixture, token: &str, manifest: Manifest) -> Value {
        let plan = f.core.plan_state(token, manifest).unwrap();
        f.core
            .apply_state(
                token,
                ApplyRequest {
                    plan,
                    secrets: Default::default(),
                    run_id: None,
                },
            )
            .unwrap()
    }

    #[test]
    fn secrets_stay_stored_and_out_of_plan_export_and_audit() {
        let f = encrypted();
        f.user("alice");
        f.user("bob");
        let jwks = signer();
        let signals = spec(
            "signals",
            PUSH,
            "https://receiver.example/events",
            jwks.clone(),
            &[("ext-a", "alice")],
        );
        let kept = spec(
            "kept",
            PUSH,
            "https://receiver.example/kept",
            jwks.clone(),
            &[("ext-k", "alice")],
        );
        f.core
            .ssf_create(
                &SsfAuth::Bearer(f.admin.clone()),
                direct_input(&signals, Some(SECRET)),
            )
            .unwrap();
        f.core
            .ssf_create(
                &SsfAuth::Bearer(f.admin.clone()),
                direct_input(&kept, Some(SECRET)),
            )
            .unwrap();
        plant(&f, "pend-subject", "signals");
        let exported = f.core.export_state(&f.admin).unwrap();
        assert_public(&exported);
        assert_eq!(
            exported["manifest"]["ssf_streams"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let round_trip: Manifest = serde_json::from_value(exported["manifest"].clone()).unwrap();
        let noop = plan(
            &f,
            Manifest {
                api_version: "riauth/v1".into(),
                ssf_streams: round_trip.ssf_streams,
                ..Default::default()
            },
        );
        assert!(noop.changes.is_empty());
        assert_public(&serde_json::to_value(&noop).unwrap());
        f.core
            .apply_state(
                &f.admin,
                ApplyRequest {
                    plan: noop,
                    secrets: Default::default(),
                    run_id: None,
                },
            )
            .unwrap();
        assert_eq!(
            stored(&f, "signals")
                .unwrap()
                .authorization_header
                .as_deref(),
            Some(SECRET)
        );
        assert!(!stopped(&f, "pend-subject"));

        let mut changed = signals.clone();
        changed.subjects.insert("ext-b".into(), "bob".into());
        let planned = plan(&f, manifest(vec![changed]));
        assert_eq!(planned.changes.len(), 1);
        assert_eq!(planned.changes[0].action, "update");
        let planned_subjects = planned.changes[0].after["subjects"].as_object().unwrap();
        assert_eq!(planned_subjects.len(), 2);
        assert!(planned_subjects.values().any(|value| value == "alice"));
        assert!(planned_subjects.values().any(|value| value == "bob"));
        assert!(planned_subjects.keys().all(|key| key.starts_with('{')));
        assert_public(&serde_json::to_value(&planned).unwrap());
        assert!(!stopped(&f, "pend-subject"));
        let applied = f
            .core
            .apply_state(
                &f.admin,
                ApplyRequest {
                    plan: planned,
                    secrets: Default::default(),
                    run_id: Some("m07-ssf-streams".into()),
                },
            )
            .unwrap();
        assert_public(&applied);
        let signals_row = stored(&f, "signals").unwrap();
        assert_eq!(signals_row.authorization_header.as_deref(), Some(SECRET));
        assert_eq!(signals_row.subjects.len(), 2);
        assert_eq!(signals_row.owner, admin_id(&f));
        assert!(stopped(&f, "pend-subject"));
        let kept_row = stored(&f, "kept").unwrap();
        assert_eq!(kept_row.authorization_header.as_deref(), Some(SECRET));
        assert_eq!(kept_row.endpoint_url, "https://receiver.example/kept");
        assert_eq!(kept_row.subjects.len(), 1);
        assert_public(&f.core.export_state(&f.admin).unwrap());
        assert_public(&f.core.audit_events(&f.admin, 400).unwrap());

        plant(&f, "pend-immutable", "signals");
        let mut mismatched = signals;
        mismatched.endpoint_url = "https://receiver.example/other".into();
        mismatched.subjects.insert("ext-b".into(), "bob".into());
        let error = f
            .core
            .plan_state(&f.admin, manifest(vec![mismatched]))
            .err()
            .unwrap();
        assert_eq!(error.code, "conflict");
        assert_eq!(
            error.message,
            "SSF administrator stream configuration is immutable after create"
        );
        let signals_row = stored(&f, "signals").unwrap();
        assert_eq!(signals_row.authorization_header.as_deref(), Some(SECRET));
        assert_eq!(signals_row.endpoint_url, "https://receiver.example/events");
        assert_eq!(signals_row.subjects.len(), 2);
        assert!(!stopped(&f, "pend-immutable"));
        assert_eq!(
            stored(&f, "kept").unwrap().authorization_header.as_deref(),
            Some(SECRET)
        );
    }

    #[test]
    fn invalid_shapes_high_privilege_grants_and_automation_do_not_write() {
        let mut f = Fixture::new();
        f.user("alice");
        f.user("desk");
        let jwks = signer();
        let sample = spec(
            "tenant-a",
            PUSH,
            "https://receiver.example/events",
            jwks.clone(),
            &[("ext-a", "alice")],
        );
        let mut private_key = serde_json::to_value(&sample).unwrap();
        private_key["jwks"]["keys"][0]["d"] = json!("private-component");
        assert!(serde_json::from_value::<SsfStreamSpec>(private_key).is_err());
        let mut header = serde_json::to_value(&sample).unwrap();
        header["authorization_header"] = json!(SECRET);
        assert!(serde_json::from_value::<SsfStreamSpec>(header).is_err());
        let mut duplicate = manifest(vec![sample.clone(), sample.clone()]);
        duplicate.ssf_streams[1].subjects = sample.subjects.clone();
        let error = f.core.plan_state(&f.admin, duplicate).err().unwrap();
        assert_eq!(error.code, "invalid_request");
        assert_eq!(error.message, "Duplicate resource in manifest");
        let mut empty = sample.clone();
        empty.events_requested.clear();
        let error = f
            .core
            .plan_state(&f.admin, manifest(vec![empty]))
            .err()
            .unwrap();
        assert_eq!(
            error.message,
            "Subscribe only to account-disabled, session-revoked, and credential-change"
        );
        let error = f
            .core
            .plan_state(
                &f.admin,
                Manifest {
                    api_version: "riauth/v1".into(),
                    delegated_grants: vec![DelegatedGrantSpec {
                        username: "desk".into(),
                        grants: vec![GrantInput {
                            role: HumanRole::SecurityAdministrator,
                            scope: "key/signing".into(),
                        }],
                    }],
                    ssf_streams: vec![sample.clone()],
                    ..Default::default()
                },
            )
            .err()
            .unwrap();
        assert_eq!(
            error.message,
            "High-privilege grant changes require a reviewed grant change"
        );
        assert!(stored(&f, "tenant-a").is_none());
        assert_eq!(
            f.core.human_grants(&f.admin, "desk").unwrap()["grants"],
            json!([])
        );

        f.core.config.state_reconciliation_mode = ReconciliationMode::Automatic;
        let decision = f
            .core
            .state_reconcile(&f.admin, manifest(vec![sample.clone()]))
            .unwrap();
        assert_eq!(decision["decision"], "awaiting_review");
        assert_eq!(decision["reason"], "change_review_required");
        assert!(stored(&f, "tenant-a").is_none());

        apply(&f, manifest(vec![sample]));
        f.core
            .store
            .write(|tx| {
                let mut stream = tx.get::<Stream>("ssf_streams", "tenant-a")?.unwrap();
                let key = stream.subjects.keys().next().unwrap().clone();
                stream.subjects.insert(key, "missing-user".into());
                tx.put("ssf_streams", "tenant-a", &stream)
            })
            .unwrap();
        let error = f.core.export_state(&f.admin).err().unwrap();
        assert_eq!(error.code, "conflict");
        assert_eq!(
            error.message,
            "SSF subject binding has no local user; export refused"
        );

        let mut grouped = manifest(vec![spec(
            "tenant-b",
            PUSH,
            "https://receiver.example/events",
            jwks,
            &[("ext-a", "alice")],
        )]);
        grouped.groups.push(riauth::state::GroupSpec {
            name: "team".into(),
            members: BTreeSet::new(),
        });
        let planned = plan(&f, grouped);
        assert!(planned.group_dependencies.is_none());
        assert!(planned.client_dependencies.is_none());
        assert!(planned.user_dependencies.is_none());
        assert!(planned.client_description_dependencies.is_none());
    }

    fn workflow() -> Definition {
        serde_json::from_value(json!({
            "format":"riauth.workflow/v1", "id":"local-password", "revision":1,
            "category":"authentication", "origin":"configured", "entry":"password",
            "limits":{"max_duration_seconds":600,"max_executions":3},
            "steps":[{"id":"password","action":{"type":"verify_password"},
                "max_attempts":3,"timeout_seconds":300,"cancellable":true,
                "transitions":[{"on":"verified","to":"success"},{"on":"failed","to":"denied"}]}],
            "terminals":[{"id":"success","outcome":"authenticated","requires":[]},
                {"id":"denied","outcome":"denied","requires":[]}]
        }))
        .unwrap()
    }

    fn sso_cookie(core: &Core, session: &str) -> String {
        let request = core.portal_sign_in().unwrap();
        core.portal_decide(session, request.body["code"].as_str().unwrap(), true)
            .unwrap();
        let binding = request.cookies[0]
            .split(';')
            .next()
            .unwrap()
            .split_once('=')
            .unwrap()
            .1;
        let response = core
            .portal_poll(request.body["id"].as_str().unwrap(), Some(binding))
            .unwrap();
        response
            .cookies
            .iter()
            .find(|cookie| cookie.starts_with("riauth_sso="))
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .split_once('=')
            .unwrap()
            .1
            .to_owned()
    }

    #[tokio::test]
    async fn workflow_editor_rejects_ssf_streams() {
        let f = Fixture::new();
        let jwks = signer();
        let planned = plan(
            &f,
            Manifest {
                api_version: "riauth/v1".into(),
                workflows: vec![workflow()],
                ..Default::default()
            },
        );
        let mut tainted = planned.clone();
        tainted.manifest.ssf_streams.push(spec(
            "tenant-a",
            PUSH,
            "https://receiver.example/events",
            jwks,
            &[],
        ));
        let cookie = sso_cookie(&f.core, &f.admin);
        let origin = url::Url::parse(&f.core.config.issuer)
            .unwrap()
            .origin()
            .ascii_serialization();
        let app = riauth::api::router(f.core.clone());
        let rejected = post_apply(&app, &cookie, &origin, &tainted).await;
        assert_eq!(rejected.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            rejected.1["error_description"],
            "Workflow editor applies one workflow only"
        );
        let accepted = post_apply(&app, &cookie, &origin, &planned).await;
        assert_eq!(accepted.0, StatusCode::OK);
        assert!(stored(&f, "tenant-a").is_none());
        assert_eq!(
            f.core.list_workflow_definitions(&f.admin).unwrap()[0]["id"],
            "local-password"
        );
    }

    async fn post_apply(
        app: &axum::Router,
        cookie: &str,
        origin: &str,
        plan: &riauth::state::Plan,
    ) -> (StatusCode, Value) {
        let body = serde_json::to_string(&ApplyRequest {
            plan: plan.clone(),
            secrets: Default::default(),
            run_id: None,
        })
        .unwrap();
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/admin/workflows/apply")
                    .header("cookie", format!("riauth_sso={cookie}"))
                    .header("origin", origin)
                    .header("sec-fetch-site", "same-origin")
                    .header("x-riauth-portal", "1")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap())
    }
}

#[cfg(not(feature = "platform"))]
#[test]
fn essentials_rejects_nonempty_ssf_streams() {
    use riauth::state::{Manifest, SsfStreamSpec};
    use std::collections::BTreeMap;

    let manifest = Manifest {
        api_version: "riauth/v1".into(),
        ssf_streams: vec![SsfStreamSpec {
            id: "tenant-a".into(),
            issuer: "https://transmitter.example".into(),
            audience: "https://idp.example".into(),
            events_requested: Default::default(),
            delivery_method: "push".into(),
            endpoint_url: "https://receiver.example/events".into(),
            jwks: Default::default(),
            subjects: BTreeMap::new(),
        }],
        ..Default::default()
    };
    let error = manifest.validate().err().unwrap();
    assert_eq!(error.code, "invalid_request");
    assert_eq!(error.message, "SSF streams require Platform");
    assert!(
        Manifest {
            api_version: "riauth/v1".into(),
            ..Default::default()
        }
        .validate()
        .is_ok()
    );
}
