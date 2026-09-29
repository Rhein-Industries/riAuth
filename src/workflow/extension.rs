//! Bounded host for an exceptional custom workflow stage.
//!
//! This is not a scripting runtime and not a sandbox around native code. A
//! registrant already running in the process can call anything else it can
//! name. The host bounds only the contract it offers: a filtered identifier
//! view, a network permit with no socket and no response body, a measured
//! output that is then dropped, a deadline, and a fixed number of in-flight
//! calls. The result is a declared routing label. It is not evidence and
//! [`Action::proof`] stays empty for it.
//!
//! A call that misses the deadline is discarded. The worker is not preempted;
//! it keeps one in-flight slot until it returns. Configuration and the
//! executor do not call this host.

use super::*;
use std::fmt;
use std::{
    collections::{BTreeMap, BTreeSet},
    panic,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

/// Capability name in [`crate::agent::FEATURES`]. Platform compiles it. The
/// instance report leaves it not configured, and this build cannot disable it,
/// because the server path does not call [`Host`].
pub const CAPABILITY: &str = "workflow.controlled_extensions";
pub const MAX_REGISTRATIONS: usize = 16;
pub const MAX_IN_FLIGHT: usize = 4;
pub const MAX_INPUT_BYTES: u32 = 4_096;
pub const MAX_GROUPS: usize = 32;
pub const MAX_NETWORK_HOSTS: usize = 8;
pub const MAX_NETWORK_REQUESTS: u8 = 4;
pub const MAX_NETWORK_RESPONSE_BYTES: u32 = 4_096;

const RESERVED_SIGNALS: [&str; 5] = ["verified", "failed", "completed", "granted", "denied"];

/// Why the host refused to produce a routing label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denial {
    NotCompiled,
    CapabilityDisabled,
    Profile,
    Unregistered,
    Permission,
    Limit,
    InputLimit,
    OutputLimit,
    Timeout,
    NetworkDenied,
    NetworkLimited,
    UndeclaredOutput,
    Capacity,
    Failed,
}

impl Denial {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotCompiled => "not_compiled",
            Self::CapabilityDisabled => "capability_disabled",
            Self::Profile => "profile",
            Self::Unregistered => "unregistered",
            Self::Permission => "permission",
            Self::Limit => "limit",
            Self::InputLimit => "input_limit",
            Self::OutputLimit => "output_limit",
            Self::Timeout => "timeout",
            Self::NetworkDenied => "network_denied",
            Self::NetworkLimited => "network_limited",
            Self::UndeclaredOutput => "undeclared_output",
            Self::Capacity => "capacity",
            Self::Failed => "failed",
        }
    }
}

/// Routing label accepted from a stage. The output bytes used to measure the
/// limit are not retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    Route { signal: Label },
}

impl Decision {
    pub fn signal(&self) -> &Label {
        match self {
            Self::Route { signal } => signal,
        }
    }
}

/// Opaque identifiers the caller already holds. Only granted permissions are
/// copied into the [`View`], and only when each value matches the identifier
/// rules below.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExtensionFacts {
    pub account: Option<String>,
    pub request: Option<String>,
    pub client: Option<String>,
    pub groups: Vec<String>,
}

/// One custom step to run. `timeout_seconds` is the step's own bound.
pub struct ExtensionCall<'a> {
    pub profile: Profile,
    pub origin: Origin,
    pub action: &'a Action,
    pub timeout_seconds: u32,
    pub facts: ExtensionFacts,
}

/// Identifiers visible to the stage. Absent permissions are `None`, including
/// when the caller supplied a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    account: Option<String>,
    request: Option<String>,
    client: Option<String>,
    groups: Option<Vec<String>>,
}

impl View {
    pub fn account(&self) -> Option<&str> {
        self.account.as_deref()
    }
    pub fn request(&self) -> Option<&str> {
        self.request.as_deref()
    }
    pub fn client(&self) -> Option<&str> {
        self.client.as_deref()
    }
    pub fn groups(&self) -> Option<&[String]> {
        self.groups.as_deref()
    }
}

/// Permit to treat `response_bytes` as received from `host`. No body is included
/// and no connection is opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkPermit {
    pub host: String,
    pub response_bytes: u32,
}

/// What the stage returns to the host. `output` is measured and dropped.
pub enum Attempt {
    Route { signal: Label, output: Vec<u8> },
    Failed,
}

pub struct Invocation {
    view: View,
    network: NetworkGate,
    output_limit: u32,
}

impl Invocation {
    pub fn view(&self) -> &View {
        &self.view
    }
    pub fn output_limit(&self) -> u32 {
        self.output_limit
    }
    pub fn fetch(&mut self, host: &str, response_bytes: u32) -> Result<NetworkPermit, Denial> {
        self.network.charge(host, response_bytes)
    }
}

/// Wall-clock and byte caps for one registration. The effective call uses the
/// tighter of these values and the step's own timeout and output cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceLimits {
    pub timeout: Duration,
    pub max_output_bytes: u32,
    pub max_input_bytes: u32,
}

/// Network permission is either absent or an exact hostname budget.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkPolicy {
    Deny,
    Allow {
        hosts: BTreeSet<String>,
        max_requests: u8,
        max_response_bytes: u32,
    },
}

#[derive(Clone)]
pub struct Registration {
    stage: Id,
    permissions: BTreeSet<StagePermission>,
    limits: ResourceLimits,
    network: NetworkPolicy,
    call: Arc<dyn Fn(&mut Invocation) -> Attempt + Send + Sync>,
}

impl Registration {
    pub fn new(
        stage: Id,
        permissions: impl IntoIterator<Item = StagePermission>,
        limits: ResourceLimits,
        network: NetworkPolicy,
        call: impl Fn(&mut Invocation) -> Attempt + Send + Sync + 'static,
    ) -> Result<Self, Denial> {
        let list: Vec<_> = permissions.into_iter().collect();
        let granted: BTreeSet<_> = list.iter().copied().collect();
        if granted.len() != list.len() {
            return Err(Denial::Permission);
        }
        let registration = Self {
            stage,
            permissions: granted,
            limits,
            network,
            call: Arc::new(call),
        };
        registration.check()?;
        Ok(registration)
    }

    fn check(&self) -> Result<(), Denial> {
        check_limits(self.limits)?;
        check_network(&self.permissions, &self.network)
    }
}

impl fmt::Debug for Registration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Registration")
            .field("stage", &self.stage)
            .field("permissions", &self.permissions)
            .field("limits", &self.limits)
            .field("network", &self.network)
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct Host {
    inner: Arc<Inner>,
}

struct Inner {
    stages: Mutex<BTreeMap<Id, Registration>>,
    in_flight: AtomicUsize,
    compiled: bool,
    enabled: bool,
}

struct Flight(Arc<Inner>);

impl Drop for Flight {
    fn drop(&mut self) {
        self.0.in_flight.fetch_sub(1, Ordering::AcqRel);
    }
}

struct NetworkGate {
    policy: NetworkPolicy,
    requests: u8,
    bytes: u32,
}

impl NetworkGate {
    fn new(policy: NetworkPolicy) -> Self {
        Self {
            policy,
            requests: 0,
            bytes: 0,
        }
    }

    fn charge(&mut self, host: &str, response_bytes: u32) -> Result<NetworkPermit, Denial> {
        let NetworkPolicy::Allow {
            hosts,
            max_requests,
            max_response_bytes,
        } = &self.policy
        else {
            return Err(Denial::NetworkDenied);
        };
        if self.requests >= *max_requests {
            return Err(Denial::NetworkLimited);
        }
        self.requests = self.requests.saturating_add(1);
        if !valid_network_host(host) || !hosts.iter().any(|allowed| allowed == host) {
            return Err(Denial::NetworkLimited);
        }
        let next = self.bytes.saturating_add(response_bytes);
        if response_bytes > *max_response_bytes || next > *max_response_bytes {
            return Err(Denial::NetworkLimited);
        }
        self.bytes = next;
        Ok(NetworkPermit {
            host: host.to_owned(),
            response_bytes,
        })
    }
}

impl Host {
    /// `enabled` is the caller's capability bit. Compilation follows this binary.
    /// The bit is fixed for this value; do not keep a host across an activation change.
    pub fn open(enabled: bool) -> Self {
        Self::from_gate(cfg!(feature = "platform"), enabled)
    }

    fn from_gate(compiled: bool, enabled: bool) -> Self {
        Self {
            inner: Arc::new(Inner {
                stages: Mutex::new(BTreeMap::new()),
                in_flight: AtomicUsize::new(0),
                compiled,
                enabled: enabled && compiled,
            }),
        }
    }

    pub fn register(&self, registration: Registration) -> Result<(), Denial> {
        self.gate()?;
        registration.check()?;
        let mut stages = self.inner.stages.lock().map_err(|_| Denial::Failed)?;
        if stages.contains_key(&registration.stage) || stages.len() >= MAX_REGISTRATIONS {
            return Err(Denial::Limit);
        }
        stages.insert(registration.stage.clone(), registration);
        Ok(())
    }

    pub fn invoke(&self, call: ExtensionCall<'_>) -> Result<Decision, Denial> {
        self.gate()?;
        if call.profile != Profile::Platform || call.origin != Origin::Configured {
            return Err(Denial::Profile);
        }
        let Action::Custom {
            stage,
            outputs,
            permissions,
            max_output_bytes,
        } = call.action
        else {
            return Err(Denial::Limit);
        };
        if outputs.is_empty() || outputs.iter().collect::<BTreeSet<_>>().len() != outputs.len() {
            return Err(Denial::Limit);
        }
        let registration = {
            let stages = self.inner.stages.lock().map_err(|_| Denial::Failed)?;
            stages.get(stage).cloned().ok_or(Denial::Unregistered)?
        };
        let requested = effective_permissions(permissions, &registration.permissions)?;
        let output_limit =
            effective_output(*max_output_bytes, registration.limits.max_output_bytes)?;
        let timeout = effective_timeout(call.timeout_seconds, registration.limits.timeout)?;
        let view = project(&call.facts, &requested, registration.limits.max_input_bytes)?;
        let network = NetworkGate::new(effective_network(&requested, &registration.network));
        let attempt = self.run(timeout, registration.call, view, network, output_limit)?;
        match attempt {
            Attempt::Failed => Err(Denial::Failed),
            Attempt::Route { signal, output } => accept(signal, output, outputs, output_limit),
        }
    }

    fn gate(&self) -> Result<(), Denial> {
        if !self.inner.compiled {
            return Err(Denial::NotCompiled);
        }
        if !self.inner.enabled {
            return Err(Denial::CapabilityDisabled);
        }
        Ok(())
    }

    fn run(
        &self,
        timeout: Duration,
        call: Arc<dyn Fn(&mut Invocation) -> Attempt + Send + Sync>,
        view: View,
        network: NetworkGate,
        output_limit: u32,
    ) -> Result<Attempt, Denial> {
        let inner = Arc::clone(&self.inner);
        let occupied = inner.in_flight.fetch_add(1, Ordering::AcqRel);
        if occupied >= MAX_IN_FLIGHT {
            inner.in_flight.fetch_sub(1, Ordering::AcqRel);
            return Err(Denial::Capacity);
        }
        let guard = Flight(inner);
        let (sender, receiver) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("riauth-workflow-extension".to_owned())
            .spawn(move || {
                let _guard = guard;
                let mut invocation = Invocation {
                    view,
                    network,
                    output_limit,
                };
                let attempt =
                    panic::catch_unwind(panic::AssertUnwindSafe(|| call(&mut invocation)));
                let attempt = match attempt {
                    Ok(attempt) => attempt,
                    Err(_) => Attempt::Failed,
                };
                let _ = sender.send(attempt);
            });
        if spawned.is_err() {
            return Err(Denial::Failed);
        }
        match receiver.recv_timeout(timeout) {
            Ok(attempt) => Ok(attempt),
            Err(mpsc::RecvTimeoutError::Timeout) => Err(Denial::Timeout),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(Denial::Failed),
        }
    }
}

fn accept(
    signal: Label,
    output: Vec<u8>,
    outputs: &[Label],
    limit: u32,
) -> Result<Decision, Denial> {
    let over = output.len() > usize::try_from(limit).unwrap_or(usize::MAX);
    drop(output);
    if over {
        return Err(Denial::OutputLimit);
    }
    if RESERVED_SIGNALS.contains(&signal.as_str()) || !outputs.iter().any(|item| item == &signal) {
        return Err(Denial::UndeclaredOutput);
    }
    Ok(Decision::Route { signal })
}

fn effective_permissions(
    requested: &[StagePermission],
    granted: &BTreeSet<StagePermission>,
) -> Result<BTreeSet<StagePermission>, Denial> {
    if requested.iter().collect::<BTreeSet<_>>().len() != requested.len() {
        return Err(Denial::Permission);
    }
    if requested
        .iter()
        .any(|permission| !granted.contains(permission))
    {
        return Err(Denial::Permission);
    }
    Ok(requested.iter().copied().collect())
}

fn effective_output(step: u32, registered: u32) -> Result<u32, Denial> {
    if !(1..=MAX_CUSTOM_OUTPUT_BYTES).contains(&step)
        || !(1..=MAX_CUSTOM_OUTPUT_BYTES).contains(&registered)
    {
        return Err(Denial::Limit);
    }
    Ok(step.min(registered))
}

fn effective_timeout(step_seconds: u32, registered: Duration) -> Result<Duration, Denial> {
    if step_seconds == 0 || step_seconds > MAX_CUSTOM_TIMEOUT_SECONDS {
        return Err(Denial::Limit);
    }
    let timeout = Duration::from_secs(u64::from(step_seconds)).min(registered);
    if timeout < Duration::from_millis(1) {
        return Err(Denial::Limit);
    }
    Ok(timeout)
}

fn effective_network(
    requested: &BTreeSet<StagePermission>,
    policy: &NetworkPolicy,
) -> NetworkPolicy {
    if requested.contains(&StagePermission::Network) {
        policy.clone()
    } else {
        NetworkPolicy::Deny
    }
}

fn project(
    facts: &ExtensionFacts,
    permissions: &BTreeSet<StagePermission>,
    max_input_bytes: u32,
) -> Result<View, Denial> {
    let mut used = 0u32;
    let mut take = |value: &str| -> Result<String, Denial> {
        if !valid_fact(value) {
            return Err(Denial::InputLimit);
        }
        let len = u32::try_from(value.len()).map_err(|_| Denial::InputLimit)?;
        used = used.checked_add(len).ok_or(Denial::InputLimit)?;
        if used > max_input_bytes {
            return Err(Denial::InputLimit);
        }
        Ok(value.to_owned())
    };
    let account = if permissions.contains(&StagePermission::ReadProfile) {
        Some(take(facts.account.as_deref().ok_or(Denial::InputLimit)?)?)
    } else {
        None
    };
    let (request, client) = if permissions.contains(&StagePermission::ReadRequest) {
        (
            Some(take(facts.request.as_deref().ok_or(Denial::InputLimit)?)?),
            Some(take(facts.client.as_deref().ok_or(Denial::InputLimit)?)?),
        )
    } else {
        (None, None)
    };
    let groups = if permissions.contains(&StagePermission::ReadGroups) {
        if facts.groups.len() > MAX_GROUPS {
            return Err(Denial::InputLimit);
        }
        let mut copied = Vec::with_capacity(facts.groups.len());
        for group in &facts.groups {
            copied.push(take(group)?);
        }
        Some(copied)
    } else {
        None
    };
    Ok(View {
        account,
        request,
        client,
        groups,
    })
}

fn check_limits(limits: ResourceLimits) -> Result<(), Denial> {
    let max_timeout = Duration::from_secs(u64::from(MAX_CUSTOM_TIMEOUT_SECONDS));
    if limits.timeout < Duration::from_millis(1) || limits.timeout > max_timeout {
        return Err(Denial::Limit);
    }
    if !(1..=MAX_CUSTOM_OUTPUT_BYTES).contains(&limits.max_output_bytes) {
        return Err(Denial::Limit);
    }
    if limits.max_input_bytes == 0 || limits.max_input_bytes > MAX_INPUT_BYTES {
        return Err(Denial::Limit);
    }
    Ok(())
}

fn check_network(
    permissions: &BTreeSet<StagePermission>,
    network: &NetworkPolicy,
) -> Result<(), Denial> {
    let granted = permissions.contains(&StagePermission::Network);
    match network {
        NetworkPolicy::Deny if granted => Err(Denial::Permission),
        NetworkPolicy::Deny => Ok(()),
        NetworkPolicy::Allow { .. } if !granted => Err(Denial::Permission),
        NetworkPolicy::Allow {
            hosts,
            max_requests,
            max_response_bytes,
        } => {
            if hosts.is_empty()
                || hosts.len() > MAX_NETWORK_HOSTS
                || !(1..=MAX_NETWORK_REQUESTS).contains(max_requests)
                || !(1..=MAX_NETWORK_RESPONSE_BYTES).contains(max_response_bytes)
                || hosts.iter().any(|host| !valid_network_host(host))
            {
                Err(Denial::Limit)
            } else {
                Ok(())
            }
        }
    }
}

/// Lowercase letters, digits, `.`, `_`, and `-`, at most 64 bytes, with no
/// empty, leading, trailing, or repeated dot. This accepts account, request,
/// client, and group ids used by the server and rejects free text.
fn valid_fact(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..")
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
}

fn valid_network_host(host: &str) -> bool {
    if host.len() > 253
        || host.len() < 3
        || !host.contains('.')
        || host.starts_with('.')
        || host.ends_with('.')
        || host.contains("..")
        || host.ends_with(".local")
        || host.ends_with(".localhost")
    {
        return false;
    }
    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() < 2
        || labels
            .iter()
            .all(|label| !label.is_empty() && label.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return false;
    }
    labels.iter().all(|label| {
        let bytes = label.as_bytes();
        (1..=63).contains(&bytes.len())
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
            && bytes[0] != b'-'
            && bytes[bytes.len() - 1] != b'-'
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    };

    fn id(value: &str) -> Id {
        Id::new(value).unwrap()
    }

    fn allow() -> Label {
        Label::new("allow").unwrap()
    }

    fn limits(timeout: Duration, max_output_bytes: u32, max_input_bytes: u32) -> ResourceLimits {
        ResourceLimits {
            timeout,
            max_output_bytes,
            max_input_bytes,
        }
    }

    fn custom(permissions: Vec<StagePermission>, max_output_bytes: u32) -> Action {
        Action::Custom {
            stage: id("risk-check"),
            outputs: vec![allow(), Label::new("block").unwrap()],
            permissions,
            max_output_bytes,
        }
    }

    fn facts() -> ExtensionFacts {
        ExtensionFacts {
            account: Some("hidden-account".into()),
            request: Some("request-1".into()),
            client: Some("app".into()),
            groups: vec!["staff".into()],
        }
    }

    fn call<'a>(
        action: &'a Action,
        timeout_seconds: u32,
        facts: ExtensionFacts,
    ) -> ExtensionCall<'a> {
        ExtensionCall {
            profile: Profile::Platform,
            origin: Origin::Configured,
            action,
            timeout_seconds,
            facts,
        }
    }

    fn route_allow(_: &mut Invocation) -> Attempt {
        Attempt::Route {
            signal: allow(),
            output: b"secret-output".to_vec(),
        }
    }

    fn register(
        host: &Host,
        permissions: impl IntoIterator<Item = StagePermission>,
        network: NetworkPolicy,
        resource: ResourceLimits,
        extension: impl Fn(&mut Invocation) -> Attempt + Send + Sync + 'static,
    ) {
        host.register(
            Registration::new(id("risk-check"), permissions, resource, network, extension).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn capability_name_is_platform_only() {
        assert_eq!(CAPABILITY, "workflow.controlled_extensions");
        assert!(crate::agent::FEATURES.contains(&CAPABILITY));
        assert!(crate::agent::PLATFORM_FEATURES.contains(&CAPABILITY));
        assert!(
            Host::open(true)
                .register(
                    Registration::new(
                        id("risk-check"),
                        [],
                        limits(Duration::from_secs(1), 32, 32),
                        NetworkPolicy::Deny,
                        route_allow,
                    )
                    .unwrap()
                )
                .is_ok()
        );
    }

    #[test]
    fn disabled_or_uncompiled_gate_denies_before_registration() {
        let disabled = Host::open(false);
        let error = disabled
            .register(
                Registration::new(
                    id("risk-check"),
                    [],
                    limits(Duration::from_secs(1), 32, 32),
                    NetworkPolicy::Deny,
                    route_allow,
                )
                .unwrap(),
            )
            .unwrap_err();
        assert_eq!(error, Denial::CapabilityDisabled);
        let action = custom(vec![], 32);
        assert_eq!(
            disabled
                .invoke(call(&action, 30, ExtensionFacts::default()))
                .unwrap_err(),
            Denial::CapabilityDisabled
        );

        let essentials = Host::from_gate(false, true);
        assert_eq!(
            essentials
                .invoke(call(&action, 30, ExtensionFacts::default()))
                .unwrap_err(),
            Denial::NotCompiled
        );
    }

    #[cfg(not(feature = "platform"))]
    #[test]
    fn essentials_binary_open_is_not_compiled() {
        let host = Host::open(true);
        let action = custom(vec![], 32);
        assert_eq!(
            host.invoke(call(&action, 30, ExtensionFacts::default()))
                .unwrap_err(),
            Denial::NotCompiled
        );
    }

    #[test]
    fn granted_request_view_routes_without_keeping_output_or_a_proof() {
        let host = Host::open(true);
        let called = Arc::new(AtomicBool::new(false));
        let seen = Arc::new(Mutex::new(None));
        let flag = Arc::clone(&called);
        let seen_view = Arc::clone(&seen);
        register(
            &host,
            [StagePermission::ReadRequest, StagePermission::ReadProfile],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 64, 128),
            move |invocation| {
                flag.store(true, Ordering::Release);
                *seen_view.lock().unwrap() = Some(invocation.view().clone());
                route_allow(invocation)
            },
        );
        let action = custom(vec![StagePermission::ReadRequest], 64);
        let decision = host.invoke(call(&action, 30, facts())).unwrap();
        assert_eq!(decision.signal().as_str(), "allow");
        assert!(action.proof(decision.signal()).is_none());
        assert!(!format!("{decision:?}").contains("secret-output"));
        assert!(!format!("{decision:?}").contains("hidden-account"));
        let view = seen.lock().unwrap().clone().unwrap();
        assert_eq!(view.request(), Some("request-1"));
        assert_eq!(view.client(), Some("app"));
        assert_eq!(view.account(), None);
        assert_eq!(view.groups(), None);
        assert!(called.load(Ordering::Acquire));
    }

    #[test]
    fn profile_permission_and_identity_shape_deny_without_calling() {
        let host = Host::open(true);
        let called = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&called);
        register(
            &host,
            [StagePermission::ReadRequest],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 64, 64),
            move |_| {
                flag.store(true, Ordering::Release);
                route_allow(&mut Invocation {
                    view: View {
                        account: None,
                        request: None,
                        client: None,
                        groups: None,
                    },
                    network: NetworkGate::new(NetworkPolicy::Deny),
                    output_limit: 64,
                })
            },
        );
        let action = custom(vec![StagePermission::ReadProfile], 64);
        assert_eq!(
            host.invoke(call(&action, 30, facts())).unwrap_err(),
            Denial::Permission
        );
        let read_request = custom(vec![StagePermission::ReadRequest], 64);
        let mut builtin = call(&read_request, 30, facts());
        builtin.origin = Origin::Builtin;
        assert_eq!(host.invoke(builtin).unwrap_err(), Denial::Profile);
        let mut essentials = call(&read_request, 30, facts());
        essentials.profile = Profile::Essentials;
        assert_eq!(host.invoke(essentials).unwrap_err(), Denial::Profile);
        let mut shaped = facts();
        shaped.account = Some("PasswordValue".into());
        let profiled = custom(vec![StagePermission::ReadProfile], 64);
        let host = Host::open(true);
        register(
            &host,
            [StagePermission::ReadProfile],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 64, 64),
            {
                let flag = Arc::clone(&called);
                move |_| {
                    flag.store(true, Ordering::Release);
                    Attempt::Failed
                }
            },
        );
        assert_eq!(
            host.invoke(call(&profiled, 30, shaped)).unwrap_err(),
            Denial::InputLimit
        );
        assert!(!called.load(Ordering::Acquire));
        assert_eq!(
            host.invoke(call(&Action::VerifyPassword {}, 30, facts()))
                .unwrap_err(),
            Denial::Limit
        );
    }

    #[test]
    fn unknown_stage_duplicate_registration_and_count_are_denied() {
        let host = Host::open(true);
        let action = custom(vec![], 32);
        assert_eq!(
            host.invoke(call(&action, 30, ExtensionFacts::default()))
                .unwrap_err(),
            Denial::Unregistered
        );
        let resource = limits(Duration::from_millis(50), 32, 32);
        for index in 0..MAX_REGISTRATIONS {
            host.register(
                Registration::new(
                    id(&format!("stage-{index}")),
                    [],
                    resource,
                    NetworkPolicy::Deny,
                    route_allow,
                )
                .unwrap(),
            )
            .unwrap();
        }
        assert_eq!(
            host.register(
                Registration::new(
                    id("stage-0"),
                    [],
                    resource,
                    NetworkPolicy::Deny,
                    route_allow,
                )
                .unwrap()
            )
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            host.register(
                Registration::new(
                    id("stage-extra"),
                    [],
                    resource,
                    NetworkPolicy::Deny,
                    route_allow,
                )
                .unwrap()
            )
            .unwrap_err(),
            Denial::Limit
        );
    }

    #[test]
    fn output_input_and_registration_bounds_deny() {
        let host = Host::open(true);
        register(
            &host,
            [StagePermission::ReadGroups],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 8, 8),
            |invocation| Attempt::Route {
                signal: allow(),
                output: vec![b'x'; invocation.output_limit() as usize],
            },
        );
        let action = custom(vec![StagePermission::ReadGroups], 32);
        let decision = host
            .invoke(call(
                &action,
                30,
                ExtensionFacts {
                    groups: vec!["staff".into()],
                    ..ExtensionFacts::default()
                },
            ))
            .unwrap();
        assert_eq!(decision.signal().as_str(), "allow");

        let oversized = Host::open(true);
        register(
            &oversized,
            [StagePermission::ReadRequest],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 8, 64),
            |_| Attempt::Route {
                signal: allow(),
                output: vec![b'x'; 9],
            },
        );
        assert_eq!(
            oversized
                .invoke(call(
                    &custom(vec![StagePermission::ReadRequest], 32),
                    30,
                    facts()
                ))
                .unwrap_err(),
            Denial::OutputLimit
        );

        let tight = Host::open(true);
        register(
            &tight,
            [StagePermission::ReadRequest],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 32, 8),
            route_allow,
        );
        assert_eq!(
            tight
                .invoke(call(
                    &custom(vec![StagePermission::ReadRequest], 32),
                    30,
                    facts()
                ))
                .unwrap_err(),
            Denial::InputLimit
        );
        assert_eq!(
            Registration::new(
                id("too-wide"),
                [],
                limits(Duration::from_secs(1), MAX_CUSTOM_OUTPUT_BYTES + 1, 32),
                NetworkPolicy::Deny,
                route_allow,
            )
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            Registration::new(
                id("too-long"),
                [],
                limits(
                    Duration::from_secs(u64::from(MAX_CUSTOM_TIMEOUT_SECONDS) + 1),
                    32,
                    32
                ),
                NetworkPolicy::Deny,
                route_allow,
            )
            .unwrap_err(),
            Denial::Limit
        );
        let groups = Host::open(true);
        register(
            &groups,
            [StagePermission::ReadGroups],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 32, MAX_INPUT_BYTES),
            route_allow,
        );
        let too_many = ExtensionFacts {
            groups: (0..=MAX_GROUPS).map(|index| format!("g{index}")).collect(),
            ..ExtensionFacts::default()
        };
        assert_eq!(
            groups
                .invoke(call(
                    &custom(vec![StagePermission::ReadGroups], 32),
                    30,
                    too_many
                ))
                .unwrap_err(),
            Denial::InputLimit
        );
    }

    #[test]
    fn timeout_discards_a_late_label_and_releases_the_slot() {
        let host = Host::open(true);
        let (done_tx, done_rx) = mpsc::channel();
        register(
            &host,
            [],
            NetworkPolicy::Deny,
            limits(Duration::from_millis(40), 32, 32),
            move |_| {
                thread::sleep(Duration::from_millis(250));
                let _ = done_tx.send(());
                Attempt::Route {
                    signal: allow(),
                    output: b"late".to_vec(),
                }
            },
        );
        let action = custom(vec![], 32);
        assert_eq!(
            host.invoke(call(&action, 30, ExtensionFacts::default()))
                .unwrap_err(),
            Denial::Timeout
        );
        done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let started = std::time::Instant::now();
        let released = loop {
            match host.invoke(call(&action, 30, ExtensionFacts::default())) {
                Err(Denial::Capacity) if started.elapsed() < Duration::from_secs(2) => {
                    thread::sleep(Duration::from_millis(5));
                }
                result => break result,
            }
        };
        assert_eq!(released.unwrap_err(), Denial::Timeout);
    }

    #[test]
    fn step_timeout_is_tighter_than_a_long_registration() {
        let host = Host::open(true);
        register(
            &host,
            [],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(30), 32, 32),
            |_| {
                thread::sleep(Duration::from_millis(1_200));
                route_allow(&mut Invocation {
                    view: View {
                        account: None,
                        request: None,
                        client: None,
                        groups: None,
                    },
                    network: NetworkGate::new(NetworkPolicy::Deny),
                    output_limit: 32,
                })
            },
        );
        let action = custom(vec![], 32);
        assert_eq!(
            host.invoke(call(&action, 1, ExtensionFacts::default()))
                .unwrap_err(),
            Denial::Timeout
        );
    }

    #[test]
    fn in_flight_cap_denies_the_extra_call_until_a_slot_returns() {
        let host = Host::open(true);
        let release = Arc::new(AtomicBool::new(false));
        let started = Arc::new(AtomicUsize::new(0));
        let (started_tx, started_rx) = mpsc::channel();
        let gate = Arc::clone(&release);
        let counter = Arc::clone(&started);
        register(
            &host,
            [],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 32, 32),
            move |_| {
                counter.fetch_add(1, Ordering::AcqRel);
                started_tx.send(()).ok();
                while !gate.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(5));
                }
                Attempt::Route {
                    signal: allow(),
                    output: Vec::new(),
                }
            },
        );
        let mut joins = Vec::new();
        for _ in 0..MAX_IN_FLIGHT {
            let host = host.clone();
            joins.push(thread::spawn(move || {
                let action = custom(vec![], 32);
                host.invoke(call(&action, 30, ExtensionFacts::default()))
            }));
        }
        for _ in 0..MAX_IN_FLIGHT {
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        }
        let action = custom(vec![], 32);
        assert_eq!(
            host.invoke(call(&action, 30, ExtensionFacts::default()))
                .unwrap_err(),
            Denial::Capacity
        );
        assert!(started_rx.try_recv().is_err());
        release.store(true, Ordering::Release);
        for join in joins {
            assert!(matches!(join.join().unwrap(), Ok(Decision::Route { .. })));
        }
        assert_eq!(started.load(Ordering::Acquire), MAX_IN_FLIGHT);
    }

    #[test]
    fn network_budget_denies_missing_permission_unlisted_hosts_and_excess() {
        assert_eq!(
            Registration::new(
                id("open"),
                [StagePermission::Network],
                limits(Duration::from_secs(1), 32, 32),
                NetworkPolicy::Deny,
                route_allow,
            )
            .unwrap_err(),
            Denial::Permission
        );
        assert_eq!(
            Registration::new(
                id("open"),
                [],
                limits(Duration::from_secs(1), 32, 32),
                NetworkPolicy::Allow {
                    hosts: BTreeSet::from(["risk.example".into()]),
                    max_requests: 1,
                    max_response_bytes: 32,
                },
                route_allow,
            )
            .unwrap_err(),
            Denial::Permission
        );
        for host_name in [
            "localhost",
            "127.0.0.1",
            "risk.local",
            "Evil.example",
            "a",
            "",
        ] {
            let mut hosts = BTreeSet::new();
            if !host_name.is_empty() {
                hosts.insert(host_name.into());
            }
            assert_eq!(
                Registration::new(
                    id("open"),
                    [StagePermission::Network],
                    limits(Duration::from_secs(1), 32, 32),
                    NetworkPolicy::Allow {
                        hosts,
                        max_requests: 1,
                        max_response_bytes: 32,
                    },
                    route_allow,
                )
                .unwrap_err(),
                Denial::Limit,
                "{host_name}"
            );
        }

        let withheld = Host::open(true);
        let (withheld_tx, withheld_rx) = mpsc::channel();
        register(
            &withheld,
            [StagePermission::Network, StagePermission::ReadRequest],
            NetworkPolicy::Allow {
                hosts: BTreeSet::from(["risk.example".into()]),
                max_requests: 1,
                max_response_bytes: 10,
            },
            limits(Duration::from_secs(2), 32, 64),
            move |invocation| {
                let result = invocation.fetch("risk.example", 1);
                withheld_tx.send(result).ok();
                Attempt::Failed
            },
        );
        let action = custom(vec![StagePermission::ReadRequest], 32);
        assert_eq!(
            withheld.invoke(call(&action, 30, facts())).unwrap_err(),
            Denial::Failed
        );
        assert_eq!(
            withheld_rx.recv().unwrap().unwrap_err(),
            Denial::NetworkDenied
        );

        let host = Host::open(true);
        let (report_tx, report_rx) = mpsc::channel();
        register(
            &host,
            [StagePermission::Network, StagePermission::ReadRequest],
            NetworkPolicy::Allow {
                hosts: BTreeSet::from(["risk.example".into()]),
                max_requests: 3,
                max_response_bytes: 10,
            },
            limits(Duration::from_secs(2), 32, 64),
            move |invocation| {
                let unlisted = invocation.fetch("other.example", 1);
                let first = invocation.fetch("risk.example", 6);
                let second = invocation.fetch("risk.example", 5);
                let third = invocation.fetch("risk.example", 1);
                report_tx.send((unlisted, first, second, third)).ok();
                Attempt::Failed
            },
        );
        let action = custom(
            vec![StagePermission::ReadRequest, StagePermission::Network],
            32,
        );
        assert_eq!(
            host.invoke(call(&action, 30, facts())).unwrap_err(),
            Denial::Failed
        );
        let (unlisted, first, second, third) = report_rx.recv().unwrap();
        assert_eq!(unlisted.unwrap_err(), Denial::NetworkLimited);
        assert!(first.is_ok());
        assert_eq!(second.unwrap_err(), Denial::NetworkLimited);
        assert_eq!(third.unwrap_err(), Denial::NetworkLimited);

        let permitted = Host::open(true);
        let (permit_tx, permit_rx) = mpsc::channel();
        register(
            &permitted,
            [StagePermission::Network],
            NetworkPolicy::Allow {
                hosts: BTreeSet::from(["risk.example".into()]),
                max_requests: 1,
                max_response_bytes: 4,
            },
            limits(Duration::from_secs(2), 32, 32),
            move |invocation| {
                let permit = invocation.fetch("risk.example", 4).unwrap();
                permit_tx.send(permit).ok();
                Attempt::Route {
                    signal: allow(),
                    output: Vec::new(),
                }
            },
        );
        let action = custom(vec![StagePermission::Network], 32);
        let decision = permitted
            .invoke(call(&action, 30, ExtensionFacts::default()))
            .unwrap();
        let NetworkPermit {
            host,
            response_bytes,
        } = permit_rx.recv().unwrap();
        assert_eq!(host, "risk.example");
        assert_eq!(response_bytes, 4);
        assert_eq!(decision.signal().as_str(), "allow");
        assert!(action.proof(decision.signal()).is_none());
    }

    #[test]
    fn undeclared_reserved_and_panicking_results_fail_closed() {
        let host = Host::open(true);
        register(
            &host,
            [],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 32, 32),
            |_| Attempt::Route {
                signal: Label::new("verified").unwrap(),
                output: Vec::new(),
            },
        );
        let action = custom(vec![], 32);
        assert_eq!(
            host.invoke(call(&action, 30, ExtensionFacts::default()))
                .unwrap_err(),
            Denial::UndeclaredOutput
        );

        let host = Host::open(true);
        register(
            &host,
            [],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 32, 32),
            |_| panic!("extension fault"),
        );
        assert_eq!(
            host.invoke(call(&action, 30, ExtensionFacts::default()))
                .unwrap_err(),
            Denial::Failed
        );
        let host = Host::open(true);
        register(
            &host,
            [],
            NetworkPolicy::Deny,
            limits(Duration::from_secs(2), 32, 32),
            route_allow,
        );
        assert!(
            host.invoke(call(&action, 30, ExtensionFacts::default()))
                .is_ok()
        );
    }
}
