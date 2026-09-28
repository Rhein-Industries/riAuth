use riauth::workflow::{
    self, Category, Code, Credential, Definition, Environment, Facts, Id, Label, Origin, Outcome,
    Proof, RunState, StagePermission, Target,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn id(value: &str) -> Id {
    Id::new(value).unwrap()
}

fn platform() -> Environment {
    let mut env = Environment::platform();
    env.sources.insert(id("corp-oidc"));
    env.stages.insert(
        id("risk-check"),
        BTreeSet::from([StagePermission::ReadRequest, StagePermission::ReadProfile]),
    );
    env
}

fn step(name: &str, action: Value, transitions: Value) -> Value {
    json!({"id": name, "action": action, "max_attempts": 1, "timeout_seconds": 60, "cancellable": true, "transitions": transitions})
}

fn go(on: &str, to: &str) -> Value {
    json!({"on": on, "to": to})
}

fn success(outcome: &str) -> Value {
    json!({"id": "success", "outcome": outcome, "requires": [], "max_proof_age_seconds": 300})
}

fn denied() -> Value {
    json!({"id": "denied", "outcome": "denied", "requires": []})
}

fn doc(category: &str, steps: Value, terminals: Value) -> Value {
    let entry = steps[0]["id"].clone();
    json!({
        "format": "riauth.workflow/v1",
        "id": "custom-flow",
        "revision": 1,
        "category": category,
        "origin": "configured",
        "entry": entry,
        "limits": {"max_duration_seconds": 900, "max_executions": 16},
        "steps": steps,
        "terminals": terminals,
    })
}

fn check(document: &Value, env: &Environment) -> Result<workflow::Validated, workflow::Invalid> {
    let definition = workflow::parse(document.to_string().as_bytes())?;
    workflow::validate(definition, env)
}

fn rejects(document: &Value, env: &Environment, code: Code) -> workflow::Invalid {
    match check(document, env) {
        Ok(_) => panic!("expected {code:?}, definition was accepted"),
        Err(error) => {
            assert_eq!(error.code, code, "{error}");
            error
        }
    }
}

fn verify(kind: &str, ok: &str) -> Value {
    step(
        kind,
        json!({"type": format!("verify_{kind}")}),
        json!([go("verified", ok), go("failed", "denied")]),
    )
}

fn custom(outputs: Value, transitions: Value) -> Value {
    let mut stage = step(
        "risk",
        json!({"type": "custom", "stage": "risk-check", "outputs": outputs, "permissions": ["read_request"], "max_output_bytes": 512}),
        transitions,
    );
    stage["timeout_seconds"] = json!(workflow::MAX_CUSTOM_TIMEOUT_SECONDS);
    stage
}

// ---------------------------------------------------------------- five categories

fn authentication() -> Value {
    doc(
        "authentication",
        json!([
            custom(
                json!(["allow", "block"]),
                json!([
                    go("allow", "source"),
                    go("block", "denied"),
                    go("failed", "denied")
                ])
            ),
            step(
                "source",
                json!({"type": "verify_source", "source": "corp-oidc"}),
                json!([
                    {"on": "verified", "when": {"type": "not", "condition": {"type": "account_has", "credential": "totp"}}, "to": "success"},
                    go("verified", "totp"),
                    go("failed", "denied"),
                ])
            ),
            verify("totp", "success"),
        ]),
        json!([{"id": "success", "outcome": "authenticated", "requires": []}, denied()]),
    )
}

fn enrollment() -> Value {
    doc(
        "enrollment",
        json!([
            step(
                "session",
                json!({"type": "resume_session"}),
                json!([go("verified", "passkey"), go("failed", "denied")])
            ),
            verify("passkey", "enroll"),
            step(
                "enroll",
                json!({"type": "enroll_credential", "credential": "totp"}),
                json!([go("completed", "success"), go("failed", "denied")])
            ),
        ]),
        json!([success("enrolled"), denied()]),
    )
}

fn recovery() -> Value {
    doc(
        "recovery",
        json!([
            step(
                "identify",
                json!({"type": "identify"}),
                json!([go("completed", "email")])
            ),
            step(
                "email",
                json!({"type": "verify_email", "purpose": "reset"}),
                json!([
                    {"on": "verified", "when": {"type": "account_has", "credential": "totp"}, "to": "totp"},
                    go("verified", "reset"),
                    go("failed", "denied"),
                ])
            ),
            verify("totp", "reset"),
            step(
                "reset",
                json!({"type": "reset_password"}),
                json!([go("completed", "success"), go("failed", "denied")])
            ),
        ]),
        json!([success("recovered"), denied()]),
    )
}

fn consent() -> Value {
    doc(
        "consent",
        json!([
            step(
                "session",
                json!({"type": "resume_session"}),
                json!([go("verified", "consent"), go("failed", "denied")])
            ),
            step(
                "consent",
                json!({"type": "request_consent"}),
                json!([go("granted", "success"), go("denied", "denied")])
            ),
        ]),
        json!([{"id": "success", "outcome": "consent_granted", "requires": []}, denied()]),
    )
}

fn sensitive_action() -> Value {
    doc(
        "sensitive_action",
        json!([
            step(
                "session",
                json!({"type": "resume_session"}),
                json!([go("verified", "passkey"), go("failed", "denied")])
            ),
            verify("passkey", "success"),
        ]),
        json!([success("action_authorized"), denied()]),
    )
}

#[test]
fn configured_flows_for_every_category_validate_on_platform_only() {
    for (category, document) in [
        (Category::Authentication, authentication()),
        (Category::Enrollment, enrollment()),
        (Category::Recovery, recovery()),
        (Category::Consent, consent()),
        (Category::SensitiveAction, sensitive_action()),
    ] {
        let validated = check(&document, &platform()).unwrap();
        assert_eq!(validated.definition().category, category);
        assert_eq!(validated.binding().revision, 1);
        assert_eq!(validated.fingerprint().len(), 64);
        rejects(
            &document,
            &Environment::essentials(),
            Code::ProfileRestricted,
        );
    }
}

#[test]
fn authentication_cannot_be_invented_by_a_custom_stage_or_second_factor() {
    let mut document = authentication();
    document["steps"][0]["transitions"][1]["to"] = json!("success");
    let error = rejects(&document, &platform(), Code::ImplicitSuccess);
    assert!(error.message.contains("no proofs"), "{error}");

    let custom_verified = doc(
        "authentication",
        json!([custom(
            json!(["verified"]),
            json!([go("verified", "success"), go("failed", "denied")])
        )]),
        json!([{"id": "success", "outcome": "authenticated", "requires": []}, denied()]),
    );
    rejects(&custom_verified, &platform(), Code::Signal);

    let totp_first = doc(
        "authentication",
        json!([verify("totp", "success")]),
        json!([{"id": "success", "outcome": "authenticated", "requires": []}, denied()]),
    );
    rejects(&totp_first, &platform(), Code::Precondition);

    let identify_only = doc(
        "authentication",
        json!([step(
            "identify",
            json!({"type": "identify"}),
            json!([go("completed", "success")])
        )]),
        json!([{"id": "success", "outcome": "authenticated", "requires": []}]),
    );
    rejects(&identify_only, &platform(), Code::ImplicitSuccess);

    let mut failure_success = authentication();
    failure_success["steps"][1]["transitions"][2]["to"] = json!("success");
    rejects(&failure_success, &platform(), Code::ImplicitSuccess);
}

#[test]
fn required_proof_alternatives_hold_on_every_branch() {
    let mut document = authentication();
    document["terminals"][0]["requires"] = json!([["source", "totp"]]);
    let error = rejects(&document, &platform(), Code::InsufficientProof);
    assert!(error.message.contains("source"), "{error}");
    document["terminals"][0]["requires"] = json!([["source", "totp"], ["source"]]);
    check(&document, &platform()).unwrap();
    document["terminals"][0]["requires"] = json!([[]]);
    rejects(&document, &platform(), Code::InsufficientProof);
    document["terminals"][0]["requires"] = json!([["totp", "totp"]]);
    rejects(&document, &platform(), Code::InsufficientProof);
}

#[test]
fn enrollment_requires_fresh_verification_not_just_a_session() {
    let mut document = enrollment();
    document["steps"][0]["transitions"][0]["to"] = json!("enroll");
    document["steps"].as_array_mut().unwrap().remove(1);
    let error = rejects(&document, &platform(), Code::Precondition);
    assert!(error.message.contains("session"), "{error}");

    let mut skip = enrollment();
    skip["steps"][2]["transitions"][1]["to"] = json!("success");
    rejects(&skip, &platform(), Code::ImplicitSuccess);

    let mut stale = enrollment();
    stale["terminals"][0]["max_proof_age_seconds"] = json!(301);
    rejects(&stale, &platform(), Code::Limit);
    stale["terminals"][0]
        .as_object_mut()
        .unwrap()
        .remove("max_proof_age_seconds");
    rejects(&stale, &platform(), Code::Limit);
}

#[test]
fn recovery_needs_a_reset_mail_proof_and_cannot_touch_other_factors() {
    let mut document = recovery();
    document["steps"][0]["transitions"][0]["to"] = json!("reset");
    document["steps"][1]["transitions"] = json!([go("verified", "denied"), go("failed", "denied")]);
    document["steps"][2]["transitions"] = json!([go("verified", "denied"), go("failed", "denied")]);
    // `email` and `totp` are now unreachable; drop them so the precondition is reported.
    let steps = document["steps"].as_array_mut().unwrap();
    steps.remove(2);
    steps.remove(1);
    rejects(&document, &platform(), Code::Precondition);

    let mut factor_reset = recovery();
    factor_reset["steps"][3]["action"] = json!({"type": "enroll_credential", "credential": "totp"});
    rejects(&factor_reset, &platform(), Code::ForbiddenAction);

    let mut invitation = recovery();
    invitation["steps"][1]["action"]["purpose"] = json!("invitation");
    rejects(&invitation, &platform(), Code::ForbiddenAction);
}

#[test]
fn consent_success_requires_the_granted_signal_and_a_session() {
    let mut document = consent();
    document["steps"][1]["transitions"][1]["to"] = json!("success");
    rejects(&document, &platform(), Code::ImplicitSuccess);

    let no_session = doc(
        "consent",
        json!([step(
            "consent",
            json!({"type": "request_consent"}),
            json!([go("granted", "success"), go("denied", "denied")])
        )]),
        json!([{"id": "success", "outcome": "consent_granted", "requires": []}, denied()]),
    );
    rejects(&no_session, &platform(), Code::Precondition);
}

#[test]
fn sensitive_actions_need_session_plus_fresh_factor() {
    let mut document = sensitive_action();
    document["steps"][0]["transitions"][0]["to"] = json!("success");
    document["steps"].as_array_mut().unwrap().remove(1);
    rejects(&document, &platform(), Code::ImplicitSuccess);

    let mut wrong_outcome = sensitive_action();
    wrong_outcome["terminals"][0]["outcome"] = json!("authenticated");
    rejects(&wrong_outcome, &platform(), Code::Outcome);

    let mut recovery_code = sensitive_action();
    recovery_code["steps"][1]["action"] = json!({"type": "verify_recovery_code"});
    rejects(&recovery_code, &platform(), Code::ForbiddenAction);
}

// ---------------------------------------------------------------- malformed and unknown nodes

#[test]
fn unknown_and_malformed_documents_are_rejected_when_parsed() {
    let cases = [
        (
            "/steps/0/action",
            json!({"type": "run_script", "source": "curl evil"}),
            "unknown variant",
        ),
        (
            "/steps/0/action",
            json!({"type": "identify", "extra": 1}),
            "unknown field",
        ),
        (
            "/steps/1/transitions/0/when",
            json!({"type": "client_ip", "cidr": "0.0.0.0/0"}),
            "unknown variant",
        ),
        ("/format", json!("riauth.workflow/v2"), "unknown variant"),
        ("/category", json!("impersonation"), "unknown variant"),
        ("/steps/0/id", json!("Bad Id"), "Id must"),
        (
            "/terminals/0/outcome",
            json!("authenticated_with_mfa"),
            "unknown variant",
        ),
    ];
    for (pointer, value, expected) in cases {
        let mut document = recovery();
        *document.pointer_mut(pointer).unwrap() = value;
        let error = rejects(&document, &platform(), Code::Syntax);
        assert!(error.message.contains(expected), "{pointer}: {error}");
    }
    let mut missing = recovery();
    missing.as_object_mut().unwrap().remove("limits");
    rejects(&missing, &platform(), Code::Syntax);

    let oversized = vec![b' '; workflow::MAX_DOCUMENT_BYTES + 1];
    assert_eq!(workflow::parse(&oversized).unwrap_err().code, Code::Limit);
}

#[test]
fn definitions_have_no_place_for_secrets_and_errors_do_not_echo_values() {
    let mut document = authentication();
    document["steps"][2]["action"]["password"] = json!("hunter2-secret");
    let error = rejects(&document, &platform(), Code::Syntax);
    assert!(error.message.starts_with("unknown field"), "{error}");
    assert!(!error.to_string().contains("password"), "{error}");

    for (pointer, value) in [
        ("/format", json!("hunter2-secret")),
        ("/category", json!("hunter2-secret")),
        ("/steps/0/action/type", json!("hunter2-secret")),
    ] {
        let mut document = authentication();
        *document.pointer_mut(pointer).unwrap() = value;
        let error = rejects(&document, &platform(), Code::Syntax);
        assert!(error.message.starts_with("unknown variant"), "{error}");
        assert!(!error.to_string().contains("hunter2"), "{error}");
    }
    let mut document = authentication();
    document["hunter2_secret_key"] = json!(1);
    let error = rejects(&document, &platform(), Code::Syntax);
    assert!(!error.to_string().contains("hunter2"), "{error}");

    let mut document = authentication();
    document["revision"] = json!("hunter2-secret");
    let error = rejects(&document, &platform(), Code::Syntax);
    assert!(!error.to_string().contains("hunter2"), "{error}");

    let mut document = authentication();
    document["steps"][0]["action"]["config"] = json!({"client_secret": "hunter2-secret"});
    let error = rejects(&document, &platform(), Code::Syntax);
    assert!(!error.to_string().contains("hunter2"), "{error}");

    let schema = riauth::schema::schema("workflow").unwrap().to_string();
    for forbidden in ["\"secret\"", "\"password\":", "\"script\"", "\"url\""] {
        assert!(!schema.contains(forbidden), "{forbidden} in schema");
    }
}

#[test]
fn references_and_identifiers_are_checked() {
    let mut unknown_target = consent();
    unknown_target["steps"][0]["transitions"][0]["to"] = json!("nowhere");
    rejects(&unknown_target, &platform(), Code::UnknownNode);

    let mut unknown_entry = consent();
    unknown_entry["entry"] = json!("nowhere");
    rejects(&unknown_entry, &platform(), Code::UnknownNode);

    let mut terminal_entry = consent();
    terminal_entry["entry"] = json!("success");
    rejects(&terminal_entry, &platform(), Code::Entry);

    let mut duplicate = consent();
    duplicate["terminals"][1]["id"] = json!("session");
    rejects(&duplicate, &platform(), Code::DuplicateId);

    let mut duplicate_step = consent();
    duplicate_step["steps"][1]["id"] = json!("session");
    rejects(&duplicate_step, &platform(), Code::DuplicateId);

    let mut env = platform();
    env.sources.clear();
    rejects(&authentication(), &env, Code::UnknownReference);

    let mut env = platform();
    env.stages.clear();
    rejects(&authentication(), &env, Code::UnknownReference);

    let mut ungranted = authentication();
    ungranted["steps"][0]["action"]["permissions"] = json!(["network"]);
    rejects(&ungranted, &platform(), Code::Permission);
    ungranted["steps"][0]["action"]["permissions"] = json!(["read_request", "read_request"]);
    rejects(&ungranted, &platform(), Code::Permission);

    let mut output = authentication();
    output["steps"][0]["action"]["max_output_bytes"] = json!(workflow::MAX_CUSTOM_OUTPUT_BYTES + 1);
    rejects(&output, &platform(), Code::Limit);
}

// ---------------------------------------------------------------- cycles and bounds

#[test]
fn loops_are_rejected_in_favor_of_bounded_attempts() {
    let mut self_loop = consent();
    self_loop["steps"][0]["transitions"][1]["to"] = json!("session");
    let error = rejects(&self_loop, &platform(), Code::Cycle);
    assert!(error.message.contains("session"), "{error}");

    let mut back_edge = consent();
    back_edge["steps"][1]["transitions"][1]["to"] = json!("session");
    rejects(&back_edge, &platform(), Code::Cycle);

    let mut long_cycle = recovery();
    long_cycle["steps"][2]["transitions"][1]["to"] = json!("identify");
    rejects(&long_cycle, &platform(), Code::Cycle);

    let mut unreachable = consent();
    unreachable["steps"].as_array_mut().unwrap().push(step(
        "orphan",
        json!({"type": "resume_session"}),
        json!([go("verified", "denied"), go("failed", "denied")]),
    ));
    rejects(&unreachable, &platform(), Code::Unreachable);
}

#[test]
fn retry_expiry_and_execution_bounds_are_enforced() {
    for (pointer, value) in [
        ("/steps/0/max_attempts", json!(0)),
        ("/steps/0/max_attempts", json!(workflow::MAX_ATTEMPTS + 1)),
        ("/steps/0/timeout_seconds", json!(0)),
        ("/steps/0/timeout_seconds", json!(901)),
        ("/limits/max_duration_seconds", json!(0)),
        (
            "/limits/max_duration_seconds",
            json!(workflow::MAX_RUN_SECONDS + 1),
        ),
        ("/limits/max_executions", json!(0)),
        (
            "/limits/max_executions",
            json!(workflow::MAX_RUN_EXECUTIONS + 1),
        ),
        ("/revision", json!(0)),
    ] {
        let mut document = consent();
        *document.pointer_mut(pointer).unwrap() = value;
        rejects(&document, &platform(), Code::Limit);
    }

    let mut custom_timeout = authentication();
    custom_timeout["steps"][0]["timeout_seconds"] = json!(workflow::MAX_CUSTOM_TIMEOUT_SECONDS + 1);
    rejects(&custom_timeout, &platform(), Code::Limit);

    let mut retries = recovery();
    for i in 0..4 {
        retries["steps"][i]["max_attempts"] = json!(5);
    }
    retries["limits"]["max_executions"] = json!(19);
    let error = rejects(&retries, &platform(), Code::Unbounded);
    assert!(error.message.contains("20"), "{error}");
    retries["limits"]["max_executions"] = json!(20);
    check(&retries, &platform()).unwrap();

    let mut too_many = consent();
    let filler: Vec<_> = (0..workflow::MAX_STEPS)
        .map(|i| {
            step(
                &format!("s{i}"),
                json!({"type": "resume_session"}),
                json!([go("verified", "denied"), go("failed", "denied")]),
            )
        })
        .collect();
    too_many["steps"].as_array_mut().unwrap().extend(filler);
    rejects(&too_many, &platform(), Code::Limit);
}

// ---------------------------------------------------------------- transitions and conditions

#[test]
fn every_signal_is_routed_explicitly_with_a_final_unconditional_transition() {
    let mut missing = consent();
    missing["steps"][1]["transitions"] = json!([go("granted", "success")]);
    let error = rejects(&missing, &platform(), Code::Signal);
    assert!(error.message.contains("denied"), "{error}");

    let mut foreign = consent();
    foreign["steps"][0]["transitions"][0]["on"] = json!("granted");
    rejects(&foreign, &platform(), Code::Signal);

    let mut conditional_last = consent();
    conditional_last["steps"][0]["transitions"][1]["when"] =
        json!({"type": "request_requires_mfa"});
    rejects(&conditional_last, &platform(), Code::Signal);

    let mut shadowed = consent();
    shadowed["steps"][0]["transitions"] = json!([
        go("verified", "consent"),
        {"on": "verified", "when": {"type": "request_requires_mfa"}, "to": "denied"},
        go("failed", "denied"),
    ]);
    rejects(&shadowed, &platform(), Code::Signal);

    let mut empty = consent();
    empty["steps"][0]["transitions"] = json!([]);
    rejects(&empty, &platform(), Code::Limit);
}

#[test]
fn only_bounded_meaningful_conditions_are_allowed() {
    let with_condition = |condition: Value| {
        let mut document = recovery();
        document["steps"][1]["transitions"][0]["when"] = condition;
        document
    };
    check(
        &with_condition(json!({"type": "all", "of": [{"type": "has_proof", "proof": "reset_email"}, {"type": "account_has", "credential": "totp"}]})),
        &platform(),
    )
    .unwrap();
    rejects(
        &with_condition(json!({"type": "any", "of": []})),
        &platform(),
        Code::Condition,
    );
    rejects(
        &with_condition(json!({"type": "has_proof", "proof": "passkey"})),
        &platform(),
        Code::Condition,
    );
    let mut deep = json!({"type": "request_requires_mfa"});
    for _ in 0..workflow::MAX_CONDITION_DEPTH {
        deep = json!({"type": "not", "condition": deep});
    }
    rejects(&with_condition(deep), &platform(), Code::Condition);
    let wide: Vec<_> = (0..workflow::MAX_CONDITION_NODES)
        .map(|_| json!({"type": "request_requires_mfa"}))
        .collect();
    rejects(
        &with_condition(json!({"type": "any", "of": wide})),
        &platform(),
        Code::Condition,
    );
}

#[test]
fn terminals_are_explicit_and_category_bound() {
    let mut denial_with_requirement = consent();
    denial_with_requirement["terminals"][1]["requires"] = json!([["session"]]);
    rejects(&denial_with_requirement, &platform(), Code::Outcome);

    let mut only_denied = consent();
    only_denied["terminals"][0]["outcome"] = json!("denied");
    only_denied["terminals"][0]["requires"] = json!([]);
    only_denied["terminals"][0]
        .as_object_mut()
        .unwrap()
        .remove("max_proof_age_seconds");
    rejects(&only_denied, &platform(), Code::Outcome);

    let mut enrollment_as_login = enrollment();
    enrollment_as_login["terminals"][0]["outcome"] = json!("authenticated");
    rejects(&enrollment_as_login, &platform(), Code::Outcome);
}

struct Inputs {
    totp: bool,
    mfa: bool,
}

impl Facts for Inputs {
    fn account_has(&self, credential: Credential) -> bool {
        credential == Credential::Totp && self.totp
    }
    fn has_proof(&self, _: Proof) -> bool {
        false
    }
    fn request_requires_mfa(&self) -> bool {
        self.mfa
    }
}

#[test]
fn resolution_follows_declared_routes_and_has_no_implicit_outcome() {
    let validated = check(&authentication(), &platform()).unwrap();
    let source = id("source");
    let verified = Label::new("verified").unwrap();
    match validated
        .resolve(
            &source,
            &verified,
            &Inputs {
                totp: true,
                mfa: false,
            },
        )
        .unwrap()
    {
        Target::Step(step) => assert_eq!(step.id.as_str(), "totp"),
        Target::Terminal(_) => panic!("TOTP must be required"),
    }
    match validated
        .resolve(
            &source,
            &verified,
            &Inputs {
                totp: false,
                mfa: false,
            },
        )
        .unwrap()
    {
        Target::Terminal(terminal) => assert_eq!(terminal.outcome, Outcome::Authenticated),
        Target::Step(_) => panic!("expected terminal"),
    }
    for signal in ["completed", "granted", "allow"] {
        let error = validated
            .resolve(
                &source,
                &Label::new(signal).unwrap(),
                &Inputs {
                    totp: false,
                    mfa: false,
                },
            )
            .err()
            .unwrap();
        assert_eq!(error.code, Code::Signal);
    }
    assert_eq!(
        validated
            .resolve(
                &id("nowhere"),
                &verified,
                &Inputs {
                    totp: false,
                    mfa: false
                }
            )
            .err()
            .unwrap()
            .code,
        Code::UnknownNode
    );
}

// ---------------------------------------------------------------- Essentials defaults and boundaries

#[test]
fn essentials_defaults_cover_every_category_without_configuration() {
    let defaults = workflow::defaults();
    let categories: BTreeSet<_> = defaults.iter().map(|d| d.category).collect();
    assert_eq!(categories.len(), 5);
    let mut ids = BTreeSet::new();
    for definition in defaults {
        assert!(ids.insert(definition.id.clone()));
        assert_eq!(definition.origin, Origin::Builtin);
        assert!(definition.id.as_str().starts_with(workflow::BUILTIN_PREFIX));
        assert_eq!(
            workflow::builtin(&definition.id).as_ref(),
            Some(&definition)
        );
        let essentials =
            workflow::validate(definition.clone(), &Environment::essentials()).unwrap();
        let platform = workflow::validate(definition, &platform()).unwrap();
        assert_eq!(essentials.fingerprint(), platform.fingerprint());
        assert!(essentials.entry().is_some());
    }
}

#[test]
fn builtin_origin_cannot_be_forged_or_extended() {
    let mut tampered = workflow::defaults()
        .into_iter()
        .find(|d| d.id.as_str() == "essentials-password-sign-in")
        .unwrap();
    tampered.steps[0].transitions.retain(|t| t.when.is_none());
    tampered.steps.truncate(1);
    let error = workflow::validate(tampered.clone(), &Environment::essentials()).unwrap_err();
    assert_eq!(error.code, Code::Builtin);

    tampered.origin = Origin::Configured;
    assert_eq!(
        workflow::validate(tampered.clone(), &platform())
            .unwrap_err()
            .code,
        Code::Builtin
    );
    tampered.id = id("password-sign-in");
    workflow::validate(tampered.clone(), &platform()).unwrap();
    assert_eq!(
        workflow::validate(tampered, &Environment::essentials())
            .unwrap_err()
            .code,
        Code::ProfileRestricted
    );

    let mut unknown_builtin = consent();
    unknown_builtin["origin"] = json!("builtin");
    rejects(&unknown_builtin, &platform(), Code::Builtin);
}

// ---------------------------------------------------------------- determinism and schema

#[test]
fn serialization_and_fingerprints_round_trip_deterministically() {
    let mut documents: Vec<Definition> = workflow::defaults();
    for document in [
        authentication(),
        enrollment(),
        recovery(),
        consent(),
        sensitive_action(),
    ] {
        documents.push(workflow::parse(document.to_string().as_bytes()).unwrap());
    }
    for definition in documents {
        let first = definition.canonical_json();
        let parsed = workflow::parse(&first).unwrap();
        assert_eq!(parsed, definition);
        assert_eq!(parsed.canonical_json(), first);
        assert_eq!(parsed.fingerprint(), definition.fingerprint());
        let pretty = serde_json::to_vec_pretty(&definition).unwrap();
        assert_eq!(
            workflow::parse(&pretty).unwrap().fingerprint(),
            definition.fingerprint()
        );
    }
    let mut revised = workflow::parse(consent().to_string().as_bytes()).unwrap();
    let before = revised.fingerprint();
    revised.revision = 2;
    assert_ne!(revised.fingerprint(), before);
}

#[test]
fn essentials_default_fingerprints_are_pinned() {
    let pinned: Vec<_> = workflow::defaults()
        .iter()
        .map(|d| format!("{} {}", d.id, d.fingerprint()))
        .collect();
    assert_eq!(
        pinned, PINNED_DEFAULTS,
        "changing a default requires a new revision"
    );
}

const PINNED_DEFAULTS: [&str; 7] = [
    "essentials-passkey-sign-in faf900177460f9ebf7135d26e861516272d93408ded84f3016012e1dc9fda739",
    "essentials-password-sign-in e88e6ab6e2997b01a78e4f3d797aec33715bc22c47409862150390966e2cf8a2",
    "essentials-passkey-enrollment fea742d5b4f6adb52f99c5a2793538c4cdb76063dc3e5c6aad504dc4ce47f817",
    "essentials-invitation ef8503cc28903179bf14898ca9a5482261e30102dfc22fcc3b250530af4d1f64",
    "essentials-password-reset ab45ea3435a5850070ae9602a42ee1ed2efaf383a9a03f08e5ca49cb59a12ce3",
    "essentials-consent d0068e280c5d7a810b96cdb772dc2e85a258a278691bce36a920a43fe2203999",
    "essentials-sensitive-action e468c0e9337715e430058547dca486ffc45f11c50d44ba41325f05ade830f173",
];

#[test]
fn published_schema_is_deterministic_and_closed() {
    assert!(riauth::schema::NAMES.contains(&"workflow"));
    let first = riauth::schema::schema("workflow").unwrap();
    assert_eq!(first, riauth::schema::schema("workflow").unwrap());
    let text = first.to_string();
    assert!(text.contains("riauth.workflow/v1"));
    assert!(!text.contains("\"additionalProperties\":true"));
    let mut open_objects = Vec::new();
    find_open_objects(&first, "$", &mut open_objects);
    assert!(
        open_objects.is_empty(),
        "objects accepting unknown fields: {open_objects:?}"
    );
}

fn find_open_objects(value: &Value, path: &str, found: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if map.contains_key("properties")
                && map.get("additionalProperties") != Some(&json!(false))
            {
                found.push(path.to_owned());
            }
            for (key, child) in map {
                find_open_objects(child, &format!("{path}.{key}"), found);
            }
        }
        Value::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                find_open_objects(child, &format!("{path}[{i}]"), found);
            }
        }
        _ => {}
    }
}

#[test]
fn proof_types_declare_binding_requirements_and_run_states_round_trip() {
    for proof in Proof::ALL {
        let binding = proof.binding();
        assert!(
            binding.account && binding.request && binding.run,
            "{proof:?}"
        );
    }
    for state in [
        RunState::Active {
            step: id("password"),
            attempt: 2,
        },
        RunState::Finished {
            terminal: id("success"),
            outcome: Outcome::Authenticated,
        },
        RunState::Cancelled {},
        RunState::Expired {},
    ] {
        let text = serde_json::to_string(&state).unwrap();
        assert_eq!(serde_json::from_str::<RunState>(&text).unwrap(), state);
        assert_eq!(state.is_final(), !matches!(state, RunState::Active { .. }));
    }
    assert!(
        serde_json::from_str::<RunState>(
            r#"{"state":"finished","terminal":"x","outcome":"authenticated","acr":"mfa"}"#
        )
        .is_err()
    );
}

#[test]
fn documented_example_is_the_shipped_consent_default() {
    let page = include_str!("../docs/workflows.md");
    let start = page.find("```json\n").unwrap() + "```json\n".len();
    let end = start + page[start..].find("```").unwrap();
    let definition = workflow::parse(&page.as_bytes()[start..end]).unwrap();
    assert_eq!(workflow::builtin(&definition.id), Some(definition.clone()));
    workflow::validate(definition, &Environment::essentials()).unwrap();
}

#[test]
fn enrollment_and_sensitive_actions_need_a_session_and_primary_factor() {
    let no_session = doc(
        "enrollment",
        json!([
            verify("password", "enroll"),
            step(
                "enroll",
                json!({"type": "enroll_credential", "credential": "passkey"}),
                json!([go("completed", "success"), go("failed", "denied")])
            ),
        ]),
        json!([success("enrolled"), denied()]),
    );
    rejects(&no_session, &platform(), Code::Precondition);

    let totp_only = doc(
        "enrollment",
        json!([
            step(
                "session",
                json!({"type": "resume_session"}),
                json!([go("verified", "totp"), go("failed", "denied")])
            ),
            verify("totp", "enroll"),
            step(
                "enroll",
                json!({"type": "enroll_credential", "credential": "password"}),
                json!([go("completed", "success"), go("failed", "denied")])
            ),
        ]),
        json!([success("enrolled"), denied()]),
    );
    rejects(&totp_only, &platform(), Code::Precondition);

    let invited_factor = doc(
        "enrollment",
        json!([
            step(
                "invitation",
                json!({"type": "verify_email", "purpose": "invitation"}),
                json!([go("verified", "enroll"), go("failed", "denied")])
            ),
            step(
                "enroll",
                json!({"type": "enroll_credential", "credential": "totp"}),
                json!([go("completed", "success"), go("failed", "denied")])
            ),
        ]),
        json!([success("enrolled"), denied()]),
    );
    rejects(&invited_factor, &platform(), Code::Precondition);

    let totp_action = doc(
        "sensitive_action",
        json!([
            step(
                "session",
                json!({"type": "resume_session"}),
                json!([go("verified", "totp"), go("failed", "denied")])
            ),
            verify("totp", "success"),
        ]),
        json!([success("action_authorized"), denied()]),
    );
    rejects(&totp_action, &platform(), Code::ImplicitSuccess);
}

#[test]
fn failure_and_denial_signals_cannot_lead_straight_to_success() {
    let mut document = authentication();
    document["steps"][2]["transitions"][1]["to"] = json!("success");
    let error = rejects(&document, &platform(), Code::ImplicitSuccess);
    assert!(error.message.contains("failed"), "{error}");

    let mut custom_failure = authentication();
    custom_failure["steps"][0]["transitions"][2]["to"] = json!("success");
    rejects(&custom_failure, &platform(), Code::ImplicitSuccess);
}

#[test]
fn account_conditions_cannot_disclose_whether_an_account_exists() {
    let probe = doc(
        "authentication",
        json!([
            step(
                "identify",
                json!({"type": "identify"}),
                json!([
                    {"on": "completed", "when": {"type": "not", "condition": {"type": "account_has", "credential": "passkey"}}, "to": "denied"},
                    go("completed", "passkey"),
                ])
            ),
            verify("passkey", "success"),
        ]),
        json!([{"id": "success", "outcome": "authenticated", "requires": []}, denied()]),
    );
    rejects(&probe, &platform(), Code::Condition);

    let mut recovery_probe = recovery();
    recovery_probe["steps"][0]["transitions"] = json!([
        {"on": "completed", "when": {"type": "account_has", "credential": "password"}, "to": "email"},
        go("completed", "denied"),
    ]);
    rejects(&recovery_probe, &platform(), Code::Condition);

    let mut after_failure = consent();
    after_failure["steps"][0]["transitions"] = json!([
        go("verified", "consent"),
        {"on": "failed", "when": {"type": "account_has", "credential": "totp"}, "to": "denied"},
        go("failed", "denied"),
    ]);
    rejects(&after_failure, &platform(), Code::Condition);
}

#[test]
fn completion_applies_account_and_request_factor_rules() {
    let builtin = |name: &str| {
        let definition = workflow::builtin(&id(name)).unwrap();
        workflow::validate(definition, &Environment::essentials()).unwrap()
    };
    let sign_in = builtin("essentials-password-sign-in");
    let success = id("success");
    let plain = Inputs {
        totp: false,
        mfa: false,
    };
    let totp = Inputs {
        totp: true,
        mfa: false,
    };
    let mfa = Inputs {
        totp: false,
        mfa: true,
    };
    assert_eq!(
        sign_in
            .complete(&success, &[Proof::Password], &plain)
            .unwrap(),
        Outcome::Authenticated
    );
    for inputs in [&totp, &mfa] {
        let error = sign_in
            .complete(&success, &[Proof::Password], inputs)
            .unwrap_err();
        assert_eq!(error.code, Code::InsufficientProof);
        sign_in
            .complete(&success, &[Proof::Password, Proof::Totp], inputs)
            .unwrap();
        sign_in
            .complete(&success, &[Proof::Password, Proof::RecoveryCode], inputs)
            .unwrap();
    }
    assert_eq!(
        sign_in.complete(&success, &[], &plain).unwrap_err().code,
        Code::InsufficientProof
    );
    assert_eq!(
        sign_in
            .complete(&success, &[Proof::Totp], &plain)
            .unwrap_err()
            .code,
        Code::InsufficientProof
    );
    assert_eq!(
        sign_in.complete(&id("denied"), &[], &plain).unwrap(),
        Outcome::Denied
    );
    assert_eq!(
        sign_in
            .complete(&id("password"), &[], &plain)
            .unwrap_err()
            .code,
        Code::UnknownNode
    );

    let action = builtin("essentials-sensitive-action");
    action
        .complete(&success, &[Proof::Session, Proof::Passkey], &totp)
        .unwrap();
    assert_eq!(
        action
            .complete(&success, &[Proof::Session, Proof::Password], &totp)
            .unwrap_err()
            .code,
        Code::InsufficientProof
    );
    assert_eq!(
        action
            .complete(&success, &[Proof::Password, Proof::Totp], &totp)
            .unwrap_err()
            .code,
        Code::InsufficientProof
    );

    let invitation = builtin("essentials-invitation");
    invitation
        .complete(&success, &[Proof::Invitation, Proof::Enrolled], &totp)
        .unwrap();
    let enrollment = builtin("essentials-passkey-enrollment");
    assert_eq!(
        enrollment
            .complete(
                &success,
                &[Proof::Session, Proof::Password, Proof::Enrolled],
                &totp
            )
            .unwrap_err()
            .code,
        Code::InsufficientProof
    );
}
