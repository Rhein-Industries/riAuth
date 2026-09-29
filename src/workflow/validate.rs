use super::*;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const VERIFIED: &str = "verified";
const FAILED: &str = "failed";
const COMPLETED: &str = "completed";
const GRANTED: &str = "granted";
const DENIED: &str = "denied";
const RESERVED_SIGNALS: [&str; 5] = [VERIFIED, FAILED, COMPLETED, GRANTED, DENIED];

/// Distribution profile a definition is validated for. Platform is additive:
/// it accepts every Essentials definition plus configured ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Essentials,
    Platform,
}

/// External references a definition may use, supplied by the caller from the
/// current configuration. The model never looks them up itself.
#[derive(Clone, Debug)]
pub struct Environment {
    pub profile: Profile,
    pub sources: BTreeSet<Id>,
    /// Registered custom stages and the permissions each registration grants.
    pub stages: BTreeMap<Id, BTreeSet<StagePermission>>,
}

impl Environment {
    pub fn essentials() -> Self {
        Self::new(Profile::Essentials)
    }
    pub fn platform() -> Self {
        Self::new(Profile::Platform)
    }
    fn new(profile: Profile) -> Self {
        Self {
            profile,
            sources: BTreeSet::new(),
            stages: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Syntax,
    Limit,
    DuplicateId,
    UnknownNode,
    UnknownReference,
    Entry,
    ForbiddenAction,
    ProfileRestricted,
    Builtin,
    Permission,
    Signal,
    Condition,
    Outcome,
    Cycle,
    Unreachable,
    Unbounded,
    Precondition,
    ImplicitSuccess,
    InsufficientProof,
    Evidence,
    Binding,
    Stale,
    Replay,
    Provenance,
    Path,
    RunExpired,
    MutationPending,
}

/// Validation failure. Messages name fields and identifiers, never field values
/// from unparsed input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invalid {
    pub code: Code,
    pub path: String,
    pub message: String,
}

impl fmt::Display for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for Invalid {}

pub(super) fn fail(code: Code, path: impl Into<String>, message: impl Into<String>) -> Invalid {
    Invalid {
        code,
        path: path.into(),
        message: message.into(),
    }
}

/// Parse a definition document without validating it.
pub fn parse(bytes: &[u8]) -> Result<Definition, Invalid> {
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(fail(Code::Limit, "$", "Workflow document exceeds 64 KiB"));
    }
    serde_json::from_slice(bytes).map_err(|error| {
        let text = error.to_string();
        let text = text.split(" at line ").next().unwrap_or_default();
        let safe = [
            "unknown variant",
            "unknown field",
            "missing field",
            "duplicate field",
            "Id must",
            "Label must",
        ]
        .iter()
        .any(|prefix| text.starts_with(prefix));
        let message = if safe {
            scrub(text).chars().take(200).collect()
        } else {
            format!("Invalid workflow document ({:?})", error.classify())
        };
        fail(
            Code::Syntax,
            format!("line {} column {}", error.line(), error.column()),
            message,
        )
    })
}

/// Drop the first backtick-quoted token of an unknown variant or field error,
/// which is input text; the remaining expected names come from the schema.
fn scrub(text: &str) -> String {
    let echoes = text.starts_with("unknown variant") || text.starts_with("unknown field");
    match (echoes, text.find('`')) {
        (true, Some(start)) => match text[start + 1..].find('`') {
            Some(end) => format!("{}{}", text[..start].trim_end(), &text[start + end + 2..]),
            None => text[..start].trim_end().to_owned(),
        },
        _ => text.to_owned(),
    }
}

/// A definition that passed [`validate`]. Its fields are private so an executor
/// can only be handed checked content.
#[derive(Clone, Debug)]
pub struct Validated {
    definition: Definition,
    fingerprint: String,
    source_registration: Option<SourceRegistrationBinding>,
    extension_sha256: Option<String>,
}

pub enum Target<'a> {
    Step(&'a Step),
    Terminal(&'a Terminal),
}

impl Definition {
    /// Deterministic canonical encoding used for the fingerprint.
    pub fn canonical_json(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
    pub fn fingerprint(&self) -> String {
        Sha256::digest(self.canonical_json())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

impl Validated {
    pub fn definition(&self) -> &Definition {
        &self.definition
    }
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
    pub fn binding(&self) -> RunBinding {
        RunBinding {
            workflow: self.definition.id.clone(),
            revision: self.definition.revision,
            fingerprint: self.fingerprint.clone(),
            source_registration: self.source_registration.clone(),
            extension_sha256: self.extension_sha256.clone(),
        }
    }
    /// Bind the one source action to the live registration selected by the
    /// server. A definition document cannot provide this fingerprint.
    #[cfg(feature = "platform")]
    pub(crate) fn with_source_registration(
        mut self,
        registration: SourceRegistrationBinding,
    ) -> Result<Self, Invalid> {
        let mut sources = self.definition.steps.iter().filter_map(|step| {
            if let Action::VerifySource { source } = &step.action {
                Some(source)
            } else {
                None
            }
        });
        if self.definition.origin != Origin::Configured
            || sources.next() != Some(&registration.source)
            || sources.next().is_some()
            || registration.fingerprint.len() != 43
            || !registration
                .fingerprint
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(fail(
                Code::Binding,
                "source_registration",
                "Source registration does not match the workflow",
            ));
        }
        self.source_registration = Some(registration);
        Ok(self)
    }
    /// Pin the module hash that started this run. [`binding`] then covers it.
    #[cfg(feature = "platform")]
    pub(crate) fn pin_extension(mut self, hash: String) -> Self {
        self.extension_sha256 = Some(hash);
        self
    }
    pub fn step(&self, id: &Id) -> Option<&Step> {
        self.definition.steps.iter().find(|s| &s.id == id)
    }
    pub fn entry(&self) -> Option<&Step> {
        self.step(&self.definition.entry)
    }
    /// Select the transition for a step's signal. There is no default target:
    /// a signal the step cannot emit is an error, never an implicit outcome.
    pub fn resolve(
        &self,
        step: &Id,
        signal: &Label,
        facts: &dyn Facts,
    ) -> Result<Target<'_>, Invalid> {
        let current = self
            .step(step)
            .ok_or_else(|| fail(Code::UnknownNode, step.as_str(), "Unknown step"))?;
        let transition = current
            .transitions
            .iter()
            .filter(|t| &t.on == signal)
            .find(|t| t.when.as_ref().is_none_or(|c| c.evaluate(facts)))
            .ok_or_else(|| {
                fail(
                    Code::Signal,
                    step.as_str(),
                    format!("No transition for signal `{signal}`"),
                )
            })?;
        if let Some(next) = self.step(&transition.to) {
            return Ok(Target::Step(next));
        }
        self.definition
            .terminals
            .iter()
            .find(|t| t.id == transition.to)
            .map(Target::Terminal)
            .ok_or_else(|| fail(Code::UnknownNode, transition.to.as_str(), "Unknown node"))
    }
}

/// Factor rules are applied only after the evidence module has loaded and
/// checked every held proof against the trusted run and its recorded path.
#[allow(dead_code)] // Called by the W03 seam when W02 wires a production store.
pub(super) fn completion_rules(
    target: &Terminal,
    held: &[Proof],
    facts: &dyn Facts,
) -> Result<(), Invalid> {
    if !target.outcome.is_success() {
        return Ok(());
    }
    let proofs = mask(held);
    let required =
        target.requires.is_empty() || target.requires.iter().any(|a| proofs & mask(a) == mask(a));
    if floor(target.outcome, proofs) && required && factors(target.outcome, proofs, facts) {
        Ok(())
    } else {
        Err(fail(
            Code::InsufficientProof,
            target.id.as_str(),
            format!(
                "{:?} needs more than {} for this account and request",
                target.outcome,
                describe(proofs)
            ),
        ))
    }
}

impl Action {
    /// Signals this action can emit; each needs at least one transition.
    pub fn signals(&self) -> Vec<Label> {
        let fixed = |names: &[&'static str]| names.iter().map(|n| Label::fixed(n)).collect();
        match self {
            Action::Identify {} => fixed(&[COMPLETED]),
            Action::RequestConsent {} => fixed(&[GRANTED, DENIED]),
            Action::EnrollCredential { .. }
            | Action::ReplaceTotp {}
            | Action::RemovePasskey {}
            | Action::ResetPassword {} => fixed(&[COMPLETED, FAILED]),
            Action::Custom { outputs, .. } => {
                let mut signals = outputs.clone();
                signals.push(Label::fixed(FAILED));
                signals
            }
            _ => fixed(&[VERIFIED, FAILED]),
        }
    }
    /// Proof gained when this action emits `signal`. Custom stages gain none.
    pub fn proof(&self, signal: &Label) -> Option<Proof> {
        let signal = signal.as_str();
        match self {
            Action::ResumeSession {} if signal == VERIFIED => Some(Proof::Session),
            Action::VerifyPassword {} if signal == VERIFIED => Some(Proof::Password),
            Action::VerifyPasskey {} if signal == VERIFIED => Some(Proof::Passkey),
            Action::VerifyTotp {} if signal == VERIFIED => Some(Proof::Totp),
            Action::VerifyRecoveryCode {} if signal == VERIFIED => Some(Proof::RecoveryCode),
            Action::VerifyEmail { purpose } if signal == VERIFIED => Some(match purpose {
                EmailPurpose::Reset => Proof::ResetEmail,
                EmailPurpose::Invitation => Proof::Invitation,
            }),
            Action::VerifySource { .. } if signal == VERIFIED => Some(Proof::Source),
            Action::RequestConsent {} if signal == GRANTED => Some(Proof::Consent),
            Action::EnrollCredential { .. } if signal == COMPLETED => Some(Proof::Enrolled),
            Action::ReplaceTotp {} if signal == COMPLETED => Some(Proof::Enrolled),
            Action::RemovePasskey {} if signal == COMPLETED => Some(Proof::PasskeyRemoved),
            Action::ResetPassword {} if signal == COMPLETED => Some(Proof::PasswordReset),
            _ => None,
        }
    }
    fn name(&self) -> &'static str {
        match self {
            Action::Identify {} => "identify",
            Action::ResumeSession {} => "resume_session",
            Action::VerifyPassword {} => "verify_password",
            Action::VerifyPasskey {} => "verify_passkey",
            Action::VerifyTotp {} => "verify_totp",
            Action::VerifyRecoveryCode {} => "verify_recovery_code",
            Action::VerifyEmail { .. } => "verify_email",
            Action::VerifySource { .. } => "verify_source",
            Action::RequestConsent {} => "request_consent",
            Action::EnrollCredential { .. } => "enroll_credential",
            Action::ReplaceTotp {} => "replace_totp",
            Action::RemovePasskey {} => "remove_passkey",
            Action::ResetPassword {} => "reset_password",
            Action::Custom { .. } => "custom",
        }
    }
}

impl Category {
    pub fn success(self) -> Outcome {
        match self {
            Category::Authentication => Outcome::Authenticated,
            Category::Enrollment => Outcome::Enrolled,
            Category::Recovery => Outcome::Recovered,
            Category::Consent => Outcome::ConsentGranted,
            Category::SensitiveAction => Outcome::ActionAuthorized,
        }
    }
    fn allows(self, action: &Action) -> bool {
        use Action::*;
        use Category::*;
        matches!(
            (self, action),
            (_, Custom { .. })
                | (
                    Authentication,
                    Identify {}
                        | VerifyPassword {}
                        | VerifyPasskey {}
                        | VerifyTotp {}
                        | VerifyRecoveryCode {}
                        | VerifySource { .. }
                )
                | (
                    Enrollment,
                    ResumeSession {}
                        | VerifyPassword {}
                        | VerifyPasskey {}
                        | VerifyTotp {}
                        | VerifySource { .. }
                        | EnrollCredential { .. }
                        | ReplaceTotp {}
                        | VerifyEmail {
                            purpose: EmailPurpose::Invitation
                        }
                )
                | (
                    Recovery,
                    Identify {}
                        | VerifyTotp {}
                        | VerifyRecoveryCode {}
                        | ResetPassword {}
                        | VerifyEmail {
                            purpose: EmailPurpose::Reset
                        }
                )
                | (
                    Consent,
                    ResumeSession {} | VerifyPasskey {} | RequestConsent {}
                )
                | (
                    SensitiveAction,
                    ResumeSession {}
                        | VerifyPassword {}
                        | VerifyPasskey {}
                        | VerifyTotp {}
                        | RemovePasskey {}
                )
        )
    }
    fn needs_fresh_proof(self) -> bool {
        matches!(
            self,
            Category::Enrollment | Category::Recovery | Category::SensitiveAction
        )
    }
}

fn mask(proofs: &[Proof]) -> u16 {
    proofs.iter().fold(0, |m, p| m | p.bit())
}

/// Proofs that establish which account a run is acting for.
fn account_bound() -> u16 {
    mask(&[
        Proof::Session,
        Proof::Password,
        Proof::Passkey,
        Proof::Source,
        Proof::ResetEmail,
        Proof::Invitation,
    ])
}

/// The requirement `action` places on every path into it, if `held` misses it.
fn unmet(action: &Action, held: u16) -> Option<&'static str> {
    let any = |proofs: &[Proof]| held & mask(proofs) != 0;
    match action {
        Action::VerifyTotp {} | Action::VerifyRecoveryCode {} | Action::RequestConsent {}
            if held & account_bound() == 0 =>
        {
            Some("an account-binding proof")
        }
        Action::EnrollCredential { credential } => {
            // TOTP is a second factor: it may authorize adding a first passkey
            // to an existing session, but cannot alone bootstrap a password,
            // another TOTP secret or recovery codes. Runtime adapters still
            // require the exact account, session and request-bound receipt.
            let reverified = any(&[Proof::Session])
                && (any(&[Proof::Password, Proof::Passkey, Proof::Source])
                    || (*credential == Credential::Passkey && any(&[Proof::Totp])));
            let invited = any(&[Proof::Invitation])
                && matches!(credential, Credential::Passkey | Credential::Password);
            (!reverified && !invited).then_some(
                "a session with fresh password, passkey or source verification (TOTP only for passkey enrollment), or an invitation for a first passkey or password",
            )
        }
        Action::ReplaceTotp {}
            if held & mask(&[Proof::Session, Proof::Passkey])
                != mask(&[Proof::Session, Proof::Passkey])
                && held & mask(&[Proof::Session, Proof::Password, Proof::Totp])
                    != mask(&[Proof::Session, Proof::Password, Proof::Totp]) =>
        {
            Some("a live session with a fresh passkey, or fresh password and current TOTP proofs")
        }
        Action::RemovePasskey {}
            if held & mask(&[Proof::Session, Proof::Passkey])
                != mask(&[Proof::Session, Proof::Passkey])
                && held & mask(&[Proof::Session, Proof::Password, Proof::Totp])
                    != mask(&[Proof::Session, Proof::Password, Proof::Totp]) =>
        {
            Some("a live session with a fresh passkey, or fresh password and current TOTP proofs")
        }
        Action::ResetPassword {} if !any(&[Proof::ResetEmail]) => Some("a reset mail proof"),
        _ => None,
    }
}

/// Account- and request-relative factor rules applied at completion. A
/// passkey, TOTP or recovery code satisfies a second-factor requirement.
#[allow(dead_code)] // Called through completion_rules once W02 uses the seam.
fn factors(outcome: Outcome, held: u16, facts: &dyn Facts) -> bool {
    let any = |proofs: &[Proof]| held & mask(proofs) != 0;
    let second = any(&[Proof::Passkey, Proof::Totp, Proof::RecoveryCode]);
    let totp = facts.account_has(Credential::Totp);
    match outcome {
        Outcome::Authenticated => second || !(totp || facts.request_requires_mfa()),
        Outcome::ActionAuthorized => second || !totp,
        Outcome::Enrolled if !any(&[Proof::Invitation]) => second || !totp,
        _ => true,
    }
}

fn mentions_account(condition: &Condition) -> bool {
    match condition {
        Condition::AccountHas { .. } => true,
        Condition::All { of } | Condition::Any { of } => of.iter().any(mentions_account),
        Condition::Not { condition } => mentions_account(condition),
        _ => false,
    }
}

/// Minimum proofs for each success outcome, independent of the definition.
fn floor(outcome: Outcome, held: u16) -> bool {
    let any = |proofs: &[Proof]| held & mask(proofs) != 0;
    match outcome {
        Outcome::Authenticated => any(&[Proof::Password, Proof::Passkey, Proof::Source]),
        Outcome::Enrolled => any(&[Proof::Enrolled]),
        Outcome::Recovered => any(&[Proof::PasswordReset]),
        Outcome::ConsentGranted => any(&[Proof::Consent]) && any(&[Proof::Session]),
        Outcome::ActionAuthorized => {
            any(&[Proof::Session]) && any(&[Proof::Password, Proof::Passkey])
        }
        Outcome::Denied => true,
    }
}

fn describe(held: u16) -> String {
    let names: Vec<_> = Proof::ALL
        .iter()
        .filter(|p| held & p.bit() != 0)
        .map(|p| format!("{p:?}").to_lowercase())
        .collect();
    if names.is_empty() {
        "no proofs".into()
    } else {
        names.join(", ")
    }
}

/// Validate a definition for a profile and set of external references.
pub fn validate(definition: Definition, environment: &Environment) -> Result<Validated, Invalid> {
    check(&definition, environment)?;
    let fingerprint = definition.fingerprint();
    Ok(Validated {
        definition,
        fingerprint,
        source_registration: None,
        extension_sha256: None,
    })
}

fn check(d: &Definition, env: &Environment) -> Result<(), Invalid> {
    match d.origin {
        Origin::Builtin => {
            if builtin(&d.id).as_ref() != Some(d) {
                return Err(fail(
                    Code::Builtin,
                    "origin",
                    "Built-in definitions must match a shipped Essentials default exactly",
                ));
            }
        }
        Origin::Configured => {
            if env.profile == Profile::Essentials {
                return Err(fail(
                    Code::ProfileRestricted,
                    "origin",
                    "Essentials uses only built-in default workflows; configured workflows require Platform",
                ));
            }
            if d.id.as_str().starts_with(BUILTIN_PREFIX) {
                return Err(fail(
                    Code::Builtin,
                    "id",
                    format!("The `{BUILTIN_PREFIX}` prefix is reserved for built-in workflows"),
                ));
            }
        }
    }
    if d.revision == 0 {
        return Err(fail(Code::Limit, "revision", "Revision starts at 1"));
    }
    let limits = d.limits;
    if !(1..=MAX_RUN_SECONDS).contains(&limits.max_duration_seconds) {
        return Err(fail(
            Code::Limit,
            "limits.max_duration_seconds",
            format!("Run duration must be 1..={MAX_RUN_SECONDS} seconds"),
        ));
    }
    if !(1..=MAX_RUN_EXECUTIONS).contains(&limits.max_executions) {
        return Err(fail(
            Code::Limit,
            "limits.max_executions",
            format!("Step executions must be 1..={MAX_RUN_EXECUTIONS}"),
        ));
    }
    if d.steps.is_empty() || d.steps.len() > MAX_STEPS {
        return Err(fail(
            Code::Limit,
            "steps",
            format!("A workflow needs 1..={MAX_STEPS} steps"),
        ));
    }
    if d.terminals.is_empty() || d.terminals.len() > MAX_TERMINALS {
        return Err(fail(
            Code::Limit,
            "terminals",
            format!("A workflow needs 1..={MAX_TERMINALS} terminals"),
        ));
    }

    // Steps occupy node indices 0..n, terminals n..n+m.
    let n = d.steps.len();
    let mut index = BTreeMap::new();
    let ids = d
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| (format!("steps[{i}].id"), &s.id))
        .chain(
            d.terminals
                .iter()
                .enumerate()
                .map(|(i, t)| (format!("terminals[{i}].id"), &t.id)),
        );
    for (node, (path, id)) in ids.enumerate() {
        if index.insert(id, node).is_some() {
            return Err(fail(
                Code::DuplicateId,
                path,
                format!("Duplicate node id `{id}`"),
            ));
        }
    }
    let entry = *index.get(&d.entry).ok_or_else(|| {
        fail(
            Code::UnknownNode,
            "entry",
            format!("Unknown node `{}`", d.entry),
        )
    })?;
    if entry >= n {
        return Err(fail(
            Code::Entry,
            "entry",
            "Entry must be a step, not a terminal",
        ));
    }

    let producible = d
        .steps
        .iter()
        .flat_map(|s| {
            s.action
                .signals()
                .into_iter()
                .filter_map(|l| s.action.proof(&l))
        })
        .fold(0, |m, p| m | p.bit());
    let mut edges = vec![Vec::new(); n + d.terminals.len()];
    for (i, step) in d.steps.iter().enumerate() {
        let path = format!("steps[{i}]");
        check_step(d, env, step, &path)?;
        let signals = step.action.signals();
        for (j, t) in step.transitions.iter().enumerate() {
            let path = format!("{path}.transitions[{j}]");
            if !signals.contains(&t.on) {
                return Err(fail(
                    Code::Signal,
                    format!("{path}.on"),
                    format!("`{}` does not emit `{}`", step.action.name(), t.on),
                ));
            }
            let target = *index.get(&t.to).ok_or_else(|| {
                fail(
                    Code::UnknownNode,
                    format!("{path}.to"),
                    format!("Unknown node `{}`", t.to),
                )
            })?;
            if let Some(condition) = &t.when {
                let mut nodes = 0;
                check_condition(
                    condition,
                    1,
                    &mut nodes,
                    producible,
                    &format!("{path}.when"),
                )?;
            }
            if matches!(t.on.as_str(), FAILED | DENIED)
                && target >= n
                && d.terminals[target - n].outcome.is_success()
            {
                return Err(fail(
                    Code::ImplicitSuccess,
                    format!("{path}.to"),
                    format!("Signal `{}` cannot lead directly to success", t.on),
                ));
            }
            edges[i].push((t.on.clone(), target));
        }
        for signal in &signals {
            let routes: Vec<_> = step
                .transitions
                .iter()
                .filter(|t| &t.on == signal)
                .collect();
            let Some((last, earlier)) = routes.split_last() else {
                return Err(fail(
                    Code::Signal,
                    format!("{path}.transitions"),
                    format!("Signal `{signal}` has no transition"),
                ));
            };
            if last.when.is_some() || earlier.iter().any(|t| t.when.is_none()) {
                return Err(fail(
                    Code::Signal,
                    format!("{path}.transitions"),
                    format!("Signal `{signal}` must end with exactly one unconditional transition"),
                ));
            }
        }
    }

    let success = d.category.success();
    let mut has_success = false;
    for (i, t) in d.terminals.iter().enumerate() {
        let path = format!("terminals[{i}]");
        if t.outcome != Outcome::Denied && t.outcome != success {
            return Err(fail(
                Code::Outcome,
                format!("{path}.outcome"),
                format!(
                    "A {:?} workflow can end only as {success:?} or Denied",
                    d.category
                ),
            ));
        }
        if t.outcome == Outcome::Denied {
            if !t.requires.is_empty() || t.max_proof_age_seconds.is_some() {
                return Err(fail(
                    Code::Outcome,
                    path,
                    "Denial terminals take no proof requirements",
                ));
            }
            continue;
        }
        has_success = true;
        if t.requires.len() > MAX_ALTERNATIVES {
            return Err(fail(
                Code::Limit,
                format!("{path}.requires"),
                format!("At most {MAX_ALTERNATIVES} alternatives"),
            ));
        }
        for (k, alternative) in t.requires.iter().enumerate() {
            let unique: BTreeSet<_> = alternative.iter().collect();
            if alternative.is_empty() || unique.len() != alternative.len() {
                return Err(fail(
                    Code::InsufficientProof,
                    format!("{path}.requires[{k}]"),
                    "Alternatives must be nonempty lists of distinct proofs",
                ));
            }
        }
        match t.max_proof_age_seconds {
            Some(age) if age == 0 || age > limits.max_duration_seconds => {
                return Err(fail(
                    Code::Limit,
                    format!("{path}.max_proof_age_seconds"),
                    "Proof age must be positive and within the run duration",
                ));
            }
            Some(age)
                if d.category.needs_fresh_proof() && age > MAX_SENSITIVE_PROOF_AGE_SECONDS =>
            {
                return Err(fail(
                    Code::Limit,
                    format!("{path}.max_proof_age_seconds"),
                    format!(
                        "Proof age is at most {MAX_SENSITIVE_PROOF_AGE_SECONDS} seconds for this category"
                    ),
                ));
            }
            None if d.category.needs_fresh_proof() => {
                return Err(fail(
                    Code::Limit,
                    format!("{path}.max_proof_age_seconds"),
                    "This category requires a proof age bound",
                ));
            }
            _ => {}
        }
    }
    if !has_success {
        return Err(fail(
            Code::Outcome,
            "terminals",
            "No success terminal is defined",
        ));
    }

    let order = topological(&edges, entry, d, n)?;
    if let Some(node) = (0..edges.len()).find(|node| !order.contains(node)) {
        return Err(fail(
            Code::Unreachable,
            node_path(node, n),
            "Node is unreachable from entry",
        ));
    }

    // Worst-case executions along any path, retries included.
    let mut executions = vec![0u32; edges.len()];
    for &node in &order {
        if node < n {
            let total = executions[node] + u32::from(d.steps[node].max_attempts);
            for (_, target) in &edges[node] {
                executions[*target] = executions[*target].max(total);
            }
        }
    }
    let worst = executions.iter().copied().max().unwrap_or_default();
    if worst > u32::from(limits.max_executions) {
        return Err(fail(
            Code::Unbounded,
            "limits.max_executions",
            format!("A path can execute {worst} steps including retries"),
        ));
    }

    // Every set of proofs that can be held on arrival at each node. Conditions
    // are ignored, so each branch counts as possible.
    let mut held: Vec<BTreeSet<u16>> = vec![BTreeSet::new(); edges.len()];
    held[entry].insert(0);
    for &node in &order {
        let arriving = std::mem::take(&mut held[node]);
        if node >= n {
            let terminal = &d.terminals[node - n];
            if !terminal.outcome.is_success() {
                continue;
            }
            for &proofs in &arriving {
                if !floor(terminal.outcome, proofs) {
                    return Err(fail(
                        Code::ImplicitSuccess,
                        node_path(node, n),
                        format!(
                            "{:?} is reachable holding only {}",
                            terminal.outcome,
                            describe(proofs)
                        ),
                    ));
                }
                if !terminal.requires.is_empty()
                    && !terminal
                        .requires
                        .iter()
                        .any(|a| proofs & mask(a) == mask(a))
                {
                    return Err(fail(
                        Code::InsufficientProof,
                        node_path(node, n),
                        format!(
                            "A path holding {} satisfies no required alternative",
                            describe(proofs)
                        ),
                    ));
                }
            }
            continue;
        }
        let step = &d.steps[node];
        for &proofs in &arriving {
            if let Some(what) = unmet(&step.action, proofs) {
                return Err(fail(
                    Code::Precondition,
                    node_path(node, n),
                    format!(
                        "`{}` requires {what}; a path arrives holding {}",
                        step.action.name(),
                        describe(proofs)
                    ),
                ));
            }
        }
        for (j, (transition, (signal, target))) in
            step.transitions.iter().zip(&edges[node]).enumerate()
        {
            let gained = step.action.proof(signal).map_or(0, Proof::bit);
            let discloses = transition.when.as_ref().is_some_and(mentions_account)
                && arriving
                    .iter()
                    .any(|proofs| (proofs | gained) & account_bound() == 0);
            if discloses {
                return Err(fail(
                    Code::Condition,
                    format!("{}.transitions[{j}].when", node_path(node, n)),
                    "Account conditions need an account-binding proof on every path; otherwise routing reveals whether the account exists",
                ));
            }
            for proofs in &arriving {
                held[*target].insert(proofs | gained);
            }
        }
    }
    Ok(())
}

fn node_path(node: usize, steps: usize) -> String {
    if node < steps {
        format!("steps[{node}]")
    } else {
        format!("terminals[{}]", node - steps)
    }
}

/// Depth-first order from the entry, rejecting any cycle including self-loops.
/// Returns reachable nodes in topological order.
fn topological(
    edges: &[Vec<(Label, usize)>],
    entry: usize,
    d: &Definition,
    n: usize,
) -> Result<Vec<usize>, Invalid> {
    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        New,
        Open,
        Done,
    }
    let mut marks = vec![Mark::New; edges.len()];
    let mut post = Vec::new();
    let mut stack = vec![(entry, 0usize)];
    marks[entry] = Mark::Open;
    while let Some((node, next)) = stack.last_mut() {
        let node = *node;
        if let Some((_, target)) = edges[node].get(*next) {
            *next += 1;
            let target = *target;
            match marks[target] {
                Mark::Open => {
                    let id = if target < n {
                        &d.steps[target].id
                    } else {
                        &d.terminals[target - n].id
                    };
                    return Err(fail(
                        Code::Cycle,
                        node_path(node, n),
                        format!(
                            "Transition back to `{id}` forms a loop; use bounded max_attempts instead"
                        ),
                    ));
                }
                Mark::New => {
                    marks[target] = Mark::Open;
                    stack.push((target, 0));
                }
                Mark::Done => {}
            }
        } else {
            marks[node] = Mark::Done;
            post.push(node);
            stack.pop();
        }
    }
    post.reverse();
    Ok(post)
}

fn check_step(d: &Definition, env: &Environment, step: &Step, path: &str) -> Result<(), Invalid> {
    let action_path = format!("{path}.action");
    if !d.category.allows(&step.action) {
        return Err(fail(
            Code::ForbiddenAction,
            action_path,
            format!(
                "`{}` is not allowed in a {:?} workflow",
                step.action.name(),
                d.category
            ),
        ));
    }
    if !(1..=MAX_ATTEMPTS).contains(&step.max_attempts) {
        return Err(fail(
            Code::Limit,
            format!("{path}.max_attempts"),
            format!("Attempts must be 1..={MAX_ATTEMPTS}"),
        ));
    }
    let mut timeout = MAX_STEP_TIMEOUT_SECONDS.min(d.limits.max_duration_seconds);
    match &step.action {
        Action::VerifySource { source } if !env.sources.contains(source) => {
            return Err(fail(
                Code::UnknownReference,
                format!("{action_path}.source"),
                format!("Unknown or inactive source `{source}`"),
            ));
        }
        Action::Custom {
            stage,
            outputs,
            permissions,
            max_output_bytes,
        } => {
            if env.profile != Profile::Platform || d.origin != Origin::Configured {
                return Err(fail(
                    Code::ProfileRestricted,
                    action_path,
                    "Custom stages require a configured Platform workflow",
                ));
            }
            let granted = env.stages.get(stage).ok_or_else(|| {
                fail(
                    Code::UnknownReference,
                    format!("{action_path}.stage"),
                    format!("Unknown stage `{stage}`"),
                )
            })?;
            let unique: BTreeSet<_> = permissions.iter().collect();
            if unique.len() != permissions.len() || !permissions.iter().all(|p| granted.contains(p))
            {
                return Err(fail(
                    Code::Permission,
                    format!("{action_path}.permissions"),
                    "Permissions must be distinct and granted by the stage registration",
                ));
            }
            let unique: BTreeSet<_> = outputs.iter().collect();
            if outputs.is_empty()
                || outputs.len() > MAX_CUSTOM_OUTPUTS
                || unique.len() != outputs.len()
                || outputs
                    .iter()
                    .any(|o| RESERVED_SIGNALS.contains(&o.as_str()))
            {
                return Err(fail(
                    Code::Signal,
                    format!("{action_path}.outputs"),
                    format!(
                        "Declare 1..={MAX_CUSTOM_OUTPUTS} distinct outputs not named like built-in signals"
                    ),
                ));
            }
            if !(1..=MAX_CUSTOM_OUTPUT_BYTES).contains(max_output_bytes) {
                return Err(fail(
                    Code::Limit,
                    format!("{action_path}.max_output_bytes"),
                    format!("Output must be 1..={MAX_CUSTOM_OUTPUT_BYTES} bytes"),
                ));
            }
            timeout = timeout.min(MAX_CUSTOM_TIMEOUT_SECONDS);
        }
        _ => {}
    }
    if !(1..=timeout).contains(&step.timeout_seconds) {
        return Err(fail(
            Code::Limit,
            format!("{path}.timeout_seconds"),
            format!("Timeout must be 1..={timeout} seconds"),
        ));
    }
    if step.transitions.is_empty() || step.transitions.len() > MAX_TRANSITIONS {
        return Err(fail(
            Code::Limit,
            format!("{path}.transitions"),
            format!("A step needs 1..={MAX_TRANSITIONS} transitions"),
        ));
    }
    Ok(())
}

fn check_condition(
    condition: &Condition,
    depth: usize,
    nodes: &mut usize,
    producible: u16,
    path: &str,
) -> Result<(), Invalid> {
    *nodes += 1;
    if depth > MAX_CONDITION_DEPTH || *nodes > MAX_CONDITION_NODES {
        return Err(fail(
            Code::Condition,
            path,
            format!(
                "Conditions are limited to depth {MAX_CONDITION_DEPTH} and {MAX_CONDITION_NODES} nodes"
            ),
        ));
    }
    match condition {
        Condition::All { of } | Condition::Any { of } => {
            if of.is_empty() {
                return Err(fail(
                    Code::Condition,
                    path,
                    "Composite conditions need at least one member",
                ));
            }
            for (i, member) in of.iter().enumerate() {
                check_condition(
                    member,
                    depth + 1,
                    nodes,
                    producible,
                    &format!("{path}.of[{i}]"),
                )?;
            }
        }
        Condition::Not { condition } => {
            check_condition(
                condition,
                depth + 1,
                nodes,
                producible,
                &format!("{path}.condition"),
            )?;
        }
        Condition::HasProof { proof } if producible & proof.bit() == 0 => {
            return Err(fail(
                Code::Condition,
                path,
                format!("No step in this workflow can produce {proof:?}"),
            ));
        }
        _ => {}
    }
    Ok(())
}
