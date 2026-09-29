//! Shipped Essentials default journeys. They need no administrator-authored
//! definition, reference no configured source or stage, and are the only
//! definitions an Essentials profile accepts.

use super::*;

fn id(value: &str) -> Id {
    Id(value.to_owned())
}

fn go(on: &'static str, to: &str) -> Transition {
    Transition {
        on: Label::fixed(on),
        when: None,
        to: id(to),
    }
}

fn when(on: &'static str, condition: Condition, to: &str) -> Transition {
    Transition {
        when: Some(condition),
        ..go(on, to)
    }
}

fn has(credential: Credential) -> Condition {
    Condition::AccountHas { credential }
}

fn step(
    name: &str,
    action: Action,
    max_attempts: u8,
    timeout_seconds: u32,
    transitions: Vec<Transition>,
) -> Step {
    Step {
        id: id(name),
        action,
        max_attempts,
        timeout_seconds,
        cancellable: true,
        transitions,
    }
}

/// A step that commits a credential change and cannot be cancelled mid-way.
fn commit(name: &str, action: Action, transitions: Vec<Transition>) -> Step {
    Step {
        cancellable: false,
        ..step(name, action, 1, 300, transitions)
    }
}

fn success(outcome: Outcome, max_proof_age_seconds: Option<u32>) -> Terminal {
    Terminal {
        id: id("success"),
        outcome,
        requires: Vec::new(),
        max_proof_age_seconds,
    }
}

fn denied() -> Terminal {
    Terminal {
        id: id("denied"),
        outcome: Outcome::Denied,
        requires: Vec::new(),
        max_proof_age_seconds: None,
    }
}

fn definition(
    name: &str,
    category: Category,
    max_duration_seconds: u32,
    max_executions: u16,
    steps: Vec<Step>,
    terminals: Vec<Terminal>,
) -> Definition {
    Definition {
        format: Format::V1,
        id: id(name),
        revision: 1,
        category,
        origin: Origin::Builtin,
        entry: steps[0].id.clone(),
        limits: Limits {
            max_duration_seconds,
            max_executions,
        },
        steps,
        terminals,
    }
}

/// Fresh verification with an existing factor, then `next` on success.
fn reverify(next: &str) -> Vec<Step> {
    vec![
        step(
            "session",
            Action::ResumeSession {},
            1,
            60,
            vec![
                when("verified", has(Credential::Passkey), "passkey"),
                when("verified", has(Credential::Password), "password"),
                go("verified", "denied"),
                go("failed", "denied"),
            ],
        ),
        step(
            "passkey",
            Action::VerifyPasskey {},
            3,
            300,
            vec![go("verified", next), go("failed", "denied")],
        ),
        step(
            "password",
            Action::VerifyPassword {},
            3,
            300,
            vec![
                when("verified", has(Credential::Totp), "totp"),
                go("verified", next),
                go("failed", "denied"),
            ],
        ),
        step(
            "totp",
            Action::VerifyTotp {},
            3,
            300,
            vec![go("verified", next), go("failed", "denied")],
        ),
    ]
}

/// All shipped defaults, in a stable order.
pub fn defaults() -> Vec<Definition> {
    let fresh = Some(MAX_SENSITIVE_PROOF_AGE_SECONDS);
    let mut enrollment = reverify("enroll");
    enrollment.push(commit(
        "enroll",
        Action::EnrollCredential {
            credential: Credential::Passkey,
        },
        vec![go("completed", "success"), go("failed", "denied")],
    ));
    vec![
        definition(
            "essentials-passkey-sign-in",
            Category::Authentication,
            600,
            8,
            vec![step(
                "passkey",
                Action::VerifyPasskey {},
                3,
                300,
                vec![go("verified", "success"), go("failed", "denied")],
            )],
            vec![success(Outcome::Authenticated, None), denied()],
        ),
        definition(
            "essentials-password-sign-in",
            Category::Authentication,
            900,
            16,
            vec![
                step(
                    "password",
                    Action::VerifyPassword {},
                    3,
                    300,
                    vec![
                        when("verified", has(Credential::Totp), "totp"),
                        when("verified", Condition::RequestRequiresMfa {}, "denied"),
                        go("verified", "success"),
                        go("failed", "denied"),
                    ],
                ),
                step(
                    "totp",
                    Action::VerifyTotp {},
                    3,
                    300,
                    vec![go("verified", "success"), go("failed", "recovery-code")],
                ),
                step(
                    "recovery-code",
                    Action::VerifyRecoveryCode {},
                    3,
                    300,
                    vec![go("verified", "success"), go("failed", "denied")],
                ),
            ],
            vec![success(Outcome::Authenticated, None), denied()],
        ),
        definition(
            "essentials-passkey-enrollment",
            Category::Enrollment,
            1_200,
            16,
            enrollment,
            vec![success(Outcome::Enrolled, fresh), denied()],
        ),
        definition(
            "essentials-invitation",
            Category::Enrollment,
            900,
            4,
            vec![
                step(
                    "invitation",
                    Action::VerifyEmail {
                        purpose: EmailPurpose::Invitation,
                    },
                    1,
                    300,
                    vec![go("verified", "enroll"), go("failed", "denied")],
                ),
                commit(
                    "enroll",
                    Action::EnrollCredential {
                        credential: Credential::Passkey,
                    },
                    vec![go("completed", "success"), go("failed", "denied")],
                ),
            ],
            vec![success(Outcome::Enrolled, fresh), denied()],
        ),
        definition(
            "essentials-password-reset",
            Category::Recovery,
            2_400,
            4,
            vec![
                step(
                    "identify",
                    Action::Identify {},
                    1,
                    300,
                    vec![go("completed", "email")],
                ),
                step(
                    "email",
                    Action::VerifyEmail {
                        purpose: EmailPurpose::Reset,
                    },
                    1,
                    1_800,
                    vec![go("verified", "reset"), go("failed", "denied")],
                ),
                commit(
                    "reset",
                    Action::ResetPassword {},
                    vec![go("completed", "success"), go("failed", "denied")],
                ),
            ],
            vec![success(Outcome::Recovered, fresh), denied()],
        ),
        definition(
            "essentials-consent",
            Category::Consent,
            600,
            4,
            vec![
                step(
                    "session",
                    Action::ResumeSession {},
                    1,
                    60,
                    vec![go("verified", "consent"), go("failed", "denied")],
                ),
                step(
                    "consent",
                    Action::RequestConsent {},
                    1,
                    300,
                    vec![go("granted", "success"), go("denied", "denied")],
                ),
            ],
            vec![success(Outcome::ConsentGranted, None), denied()],
        ),
        definition(
            "essentials-sensitive-action",
            Category::SensitiveAction,
            900,
            12,
            reverify("success"),
            vec![success(Outcome::ActionAuthorized, fresh), denied()],
        ),
    ]
}

/// The shipped default with this identifier, if any.
pub fn builtin(id: &Id) -> Option<Definition> {
    defaults().into_iter().find(|d| &d.id == id)
}
