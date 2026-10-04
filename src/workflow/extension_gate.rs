//! Isolated guest for one configured custom stage.
//!
//! Platform links Wasmi 0.40. Configuration [`check`] hashes the module,
//! applies the structural caps, and runs `Module::new` in a child process
//! under the manifest timeout. The parent does not keep that image. A request
//! binds the manifest without compiling it. [`execute`] re-executes this
//! server binary. The child parses, translates, instantiates, and calls
//! `route` inside the step's wall-clock deadline. When the parent's monotonic
//! clock is at or past that deadline, the parent reaps the child, discards
//! its stdout, and reports `elapsed`, including when the child has already
//! exited with a label. Fuel exhaustion stays `fuel` or `timeout`. Essentials
//! does not link Wasmi; [`execute`] there returns `external_runtime_required`
//! and does not spawn. The in-process host in [`super::extension`] stays unwired.
//!
//! Wasmi 0.40.0 has no epoch or interrupt. `Store::call_hook` runs only when
//! the host calls Wasm or Wasm calls a host function, and `call_resumable`
//! pauses only when a host function returns an error. This guest has no
//! imports, so the engine cannot preempt `route`. `route.call` holds
//! `&mut Store` for that call. The parent does not detach the child or an IO
//! thread. Stopping a thread would require `unsafe`, which this crate forbids.
//! The fuel budget is `min(manifest fuel, timeout_seconds * FUEL_PER_SECOND)`.
//! The same `timeout_seconds` is the parent's kill deadline, measured from
//! just before spawn. That interval includes process creation and the time
//! until the parent observes and reaps the child. It is not an exact kernel
//! schedule. Equal fuel budgets report `fuel`. A strictly smaller timeout
//! budget reports `timeout`.
//!
//! Admission applies a separate resource cap before `Module::new`: one
//! `() -> i32` function, two exports, at most [`MAX_GUEST_LOCALS`] i32 locals,
//! and no data segment. A function body whose translation charge exceeds the
//! fuel budget is refused before a process starts. Wasmi 0.40.0 charges
//! [`GUEST_TRANSLATION_FUEL_PER_BYTE`] fuel per body byte on the child's first
//! call. The refusal is `timeout` when the timeout budget is strictly smaller
//! than the manifest fuel, and `fuel` otherwise. `check` runs `Module::new` for
//! a fitting body in the helper under the manifest timeout and drops the
//! result. A validation observed at or after that deadline is `elapsed` and is
//! not admitted. Each [`execute`] is a new process, so each call pays the
//! translation charge again inside the step deadline. Validation of a fitting
//! body is not fuel-metered. The structural walk stays on the caller. The
//! measured Wasmi deadline is the child process, not an engine epoch.
//!
//! The helper is this process's own executable, from `current_exe`, with only
//! [`GUEST_ARGV`]. An installed or renamed server re-executes that path. The
//! child environment is cleared. The helper refuses to run when any variable
//! remains, except macOS `__CF_USER_TEXT_ENCODING` after exec when its value
//! is three short `0x` hexadecimal fields. The parent does not pass that
//! variable. Its working directory is a private empty
//! directory removed after reap, and stderr is discarded. The request carries
//! the module, the projected identifier frame, the declared labels, and the
//! caps. It does not carry a bearer token, password, configuration path, or
//! database URL. On macOS the parent uses `/usr/bin/sandbox-exec` and a
//! default-deny Seatbelt profile before exec. The helper refuses any inherited
//! descriptor above stderr before reading IPC or compiling; inability to audit
//! `/dev/fd` also refuses. It does not close raw descriptors through unsafe
//! ownership hooks. Fork/spawn, network access, Mach IPC and filesystem writes
//! are denied; loader reads, file metadata and selected CPU/page-size sysctl
//! reads are allowed. Direct
//! unsandboxed argv entry is refused by native confinement probes. Other OSes
//! have no supported backend and return `external_runtime_required` before
//! spawn. macOS sandbox setup, a missing executable or a spawn failure is
//! `failed`, with no unsandboxed fallback. Stdout is a
//! fixed buffer; a larger write is `output` and is not a label.
//!
//! The interpreter value stack is allocated when the child calls `route`.
//! `ResourceLimiter` does not account for it. The child sets the Wasmi
//! `StackLimits` initial and maximum height to [`GUEST_VALUE_STACK_SLOTS`]
//! `UntypedVal` slots (8 bytes each). `ValueStack::new` reserves that buffer,
//! and `extend_by` returns `StackOverflow` before `Vec::reserve` when a frame
//! would make the live length reach the height. The child reports that trap as
//! `limit`. A frame that fits runs in the child until it returns, spends its
//! fuel, or the parent kills the process. The call depth stays
//! [`GUEST_CALL_DEPTH`] frames.
//!
//! `route` returns a label length. [`super::Label`] is at most
//! [`MAX_ROUTE_LABEL_BYTES`] bytes, so the child reads at most that many bytes
//! from guest memory even when the manifest output cap is 4,096. It reads only
//! a length that equals a declared label. A longer return is `output` and is
//! not read. A shorter length that equals no declared label is
//! `undeclared_output` and is not read.

use super::{
    Action, Definition, Id, Label, MAX_CUSTOM_OUTPUT_BYTES, MAX_CUSTOM_OUTPUTS,
    MAX_CUSTOM_TIMEOUT_SECONDS, StagePermission,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[cfg(feature = "platform")]
mod isolation;

pub const FORMAT: &str = "riauth.workflow-extension/v1";
/// Internal argv token for the platform server's guest re-exec.
///
/// This is not a clap command. The server recognizes it before argument
/// parsing and does not list it in `--help`.
pub const GUEST_ARGV: &str = "riauth.extension-guest/v1";
/// True only on the Platform build, which links Wasmi. Essentials keeps this
/// false and refuses execution.
pub const RUNTIME_LINKED: bool = cfg!(feature = "platform");
pub const MAX_MANIFEST_BYTES: usize = 128 * 1024;
pub const MAX_FUEL: u32 = 10_000;
/// One WebAssembly page. A guest memory cannot be smaller and must not grow.
pub const MAX_MEMORY_BYTES: u32 = 65_536;
pub const MAX_MODULE_BYTES: u32 = MAX_MEMORY_BYTES;
/// Compiler work is limited to the one exported `route` function.
pub const MAX_GUEST_FUNCTIONS: u32 = 1;
/// `route` may declare this many i32 locals. The count is rejected before Wasmi compiles.
pub const MAX_GUEST_LOCALS: u32 = 32;
/// Wasmi value-stack height in `UntypedVal` slots.
///
/// The initial height equals the maximum, so `ValueStack::new` reserves this
/// many slots and an accepted frame does not ask the `Vec` to grow.
/// `extend_by` rejects a frame when `additional >= height - len`, before
/// `Vec::reserve`. `UntypedVal` is 8 bytes, so the reserved buffer is at
/// least 512 bytes and is separate from the 64 KiB linear memory.
pub const GUEST_VALUE_STACK_SLOTS: usize = 64;
/// Wasmi `maximum_recursion_depth`. Another frame is refused when this many
/// frames are already on the call stack. That trap is also `limit`.
pub const GUEST_CALL_DEPTH: usize = 16;
pub const MAX_INPUT_BYTES: u32 = 4_096;
pub const MAX_TIMEOUT_SECONDS: u32 = MAX_CUSTOM_TIMEOUT_SECONDS;
pub const MAX_OUTPUT_BYTES: u32 = MAX_CUSTOM_OUTPUT_BYTES;
/// `workflow::Label` is at most this many bytes.
///
/// The host copies at most this many bytes from guest memory. A higher
/// manifest `max_output_bytes` stays valid and does not enlarge the copy.
pub const MAX_ROUTE_LABEL_BYTES: u32 = 32;
/// Instruction budget charged for one manifest second.
///
/// That same second is the parent's monotonic deadline. A result observed at
/// or after it is elapsed, including a child that has already exited. Fuel
/// exhaustion is not reported as elapsed time. The interval includes spawn
/// and reap overhead and is not an exact kernel schedule.
pub const FUEL_PER_SECOND: u64 = 1_000;
/// Fuel Wasmi 0.40.0 charges per function-body byte before lazy translation.
///
/// `CompilationMode::LazyTranslation` validates in `Module::new` and stores the
/// body without a deferred validator, so the first call charges this translation
/// cost and not the separate validation cost. At the 10,000 fuel cap that is
/// 1,428 body bytes.
pub const GUEST_TRANSLATION_FUEL_PER_BYTE: u64 = 7;
/// Guest output occupies memory at offset 0. The host writes the input frame
/// at this offset and nowhere else.
pub const INPUT_OFFSET: usize = 4_096;
pub const INPUT_WINDOW: usize = 4_096;

const RESERVED_SIGNALS: [&str; 5] = ["verified", "failed", "completed", "granted", "denied"];
#[cfg(feature = "platform")]
const KIND_ACCOUNT: u8 = 1;
#[cfg(feature = "platform")]
const KIND_REQUEST: u8 = 2;
#[cfg(feature = "platform")]
const KIND_CLIENT: u8 = 3;
#[cfg(feature = "platform")]
const KIND_GROUP: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denial {
    Malformed,
    Limit,
    Permission,
    Integrity,
    ExternalRuntimeRequired,
    Fuel,
    Timeout,
    Output,
    UndeclaredOutput,
    InputLimit,
    Failed,
    /// The parent's monotonic deadline passed before a result was accepted.
    ///
    /// A child that is still running is killed and reaped. A child that has
    /// already exited is reaped without another signal, and its stdout is
    /// discarded. This is not fuel exhaustion. The interval includes spawn
    /// and reap overhead and is not an exact kernel schedule.
    Elapsed,
}

impl Denial {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::Limit => "limit",
            Self::Permission => "permission",
            Self::Integrity => "integrity",
            Self::ExternalRuntimeRequired => "external_runtime_required",
            Self::Fuel => "fuel",
            Self::Timeout => "timeout",
            Self::Output => "output",
            Self::UndeclaredOutput => "undeclared_output",
            Self::InputLimit => "input_limit",
            Self::Failed => "failed",
            Self::Elapsed => "elapsed",
        }
    }

    #[cfg(feature = "platform")]
    const fn code(self) -> u8 {
        match self {
            Self::Malformed => 1,
            Self::Limit => 2,
            Self::Permission => 3,
            Self::Integrity => 4,
            Self::ExternalRuntimeRequired => 5,
            Self::Fuel => 6,
            Self::Timeout => 7,
            Self::Output => 8,
            Self::UndeclaredOutput => 9,
            Self::InputLimit => 10,
            Self::Failed => 11,
            Self::Elapsed => 12,
        }
    }
}

#[cfg(feature = "platform")]
fn denial_from_code(code: u8) -> Option<Denial> {
    Some(match code {
        1 => Denial::Malformed,
        2 => Denial::Limit,
        3 => Denial::Permission,
        4 => Denial::Integrity,
        5 => Denial::ExternalRuntimeRequired,
        6 => Denial::Fuel,
        7 => Denial::Timeout,
        8 => Denial::Output,
        9 => Denial::UndeclaredOutput,
        10 => Denial::InputLimit,
        11 => Denial::Failed,
        12 => Denial::Elapsed,
        _ => return None,
    })
}

/// Manifest that passed the bounds. Module bytes stay here so execution uses
/// the same image that was hashed; [`Debug`] does not print them.
#[derive(Clone)]
pub struct Checked {
    stage: Id,
    outputs: Vec<Label>,
    permissions: BTreeSet<StagePermission>,
    fuel: u32,
    memory_bytes: u32,
    max_input_bytes: u32,
    max_output_bytes: u32,
    timeout_seconds: u32,
    module_len: u32,
    module_sha256: [u8; 32],
    module: Vec<u8>,
}

impl fmt::Debug for Checked {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Checked")
            .field("stage", &self.stage)
            .field("outputs", &self.outputs)
            .field("permissions", &self.permissions)
            .field("fuel", &self.fuel)
            .field("memory_bytes", &self.memory_bytes)
            .field("max_input_bytes", &self.max_input_bytes)
            .field("max_output_bytes", &self.max_output_bytes)
            .field("timeout_seconds", &self.timeout_seconds)
            .field("module_len", &self.module_len)
            .field("module_sha256", &self.module_sha256)
            .finish_non_exhaustive()
    }
}

impl PartialEq for Checked {
    fn eq(&self, other: &Self) -> bool {
        self.stage == other.stage
            && self.outputs == other.outputs
            && self.permissions == other.permissions
            && self.fuel == other.fuel
            && self.memory_bytes == other.memory_bytes
            && self.max_input_bytes == other.max_input_bytes
            && self.max_output_bytes == other.max_output_bytes
            && self.timeout_seconds == other.timeout_seconds
            && self.module_len == other.module_len
            && self.module_sha256 == other.module_sha256
            && self.module == other.module
    }
}

impl Eq for Checked {}

impl Checked {
    pub fn stage(&self) -> &Id {
        &self.stage
    }
    pub fn outputs(&self) -> &[Label] {
        &self.outputs
    }
    pub fn permissions(&self) -> &BTreeSet<StagePermission> {
        &self.permissions
    }
    pub fn fuel(&self) -> u32 {
        self.fuel
    }
    pub fn memory_bytes(&self) -> u32 {
        self.memory_bytes
    }
    pub fn max_input_bytes(&self) -> u32 {
        self.max_input_bytes
    }
    pub fn max_output_bytes(&self) -> u32 {
        self.max_output_bytes
    }
    pub fn timeout_seconds(&self) -> u32 {
        self.timeout_seconds
    }
    pub fn module_len(&self) -> u32 {
        self.module_len
    }
    pub fn module_sha256(&self) -> &[u8; 32] {
        &self.module_sha256
    }
    pub fn module_sha256_hex(&self) -> String {
        self.module_sha256
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

/// Identifiers the executor already holds. [`execute`] copies a field only when
/// the step requested that permission and the manifest granted it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GuestFacts {
    pub account: Option<String>,
    pub request: Option<String>,
    pub client: Option<String>,
    pub groups: Vec<String>,
}

/// Step caps. Both must be within the manifest or execution is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepBounds {
    pub timeout_seconds: u32,
    pub max_output_bytes: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    stage: String,
    outputs: Vec<String>,
    permissions: Vec<StagePermission>,
    fuel: u32,
    memory_bytes: u32,
    max_input_bytes: u32,
    max_output_bytes: u32,
    timeout_seconds: u32,
    network: String,
    module_sha256: String,
    module_base64: String,
}

pub fn runtime_linked() -> bool {
    RUNTIME_LINKED
}

/// Check every configured manifest, including Wasmi validation on Platform.
/// The map key is the stage id. Configuration and capability use this path.
pub(crate) fn stage_registration(
    documents: &BTreeMap<String, String>,
) -> Result<BTreeMap<Id, Checked>, Denial> {
    register_stages(documents, check)
}

/// Bind manifests for a request without compiling them.
///
/// The child validates the bytes again inside the wall-clock deadline.
#[cfg(feature = "platform")]
pub(crate) fn stage_binding(
    documents: &BTreeMap<String, String>,
) -> Result<BTreeMap<Id, Checked>, Denial> {
    register_stages(documents, bind)
}

fn register_stages(
    documents: &BTreeMap<String, String>,
    admit_one: fn(&[u8]) -> Result<Checked, Denial>,
) -> Result<BTreeMap<Id, Checked>, Denial> {
    if documents.len() > 16 {
        return Err(Denial::Limit);
    }
    let mut registered = BTreeMap::new();
    for (key, document) in documents {
        let checked = admit_one(document.as_bytes())?;
        if checked.stage().as_str() != key {
            return Err(Denial::Malformed);
        }
        registered.insert(checked.stage().clone(), checked);
    }
    Ok(registered)
}

/// The manifest's outputs, permissions, and caps cover this definition's
/// custom entry. The graph shape is checked separately.
pub(crate) fn covers(checked: &Checked, definition: &Definition) -> bool {
    let Some(step) = definition.steps.first() else {
        return false;
    };
    let Action::Custom {
        stage,
        outputs,
        permissions,
        max_output_bytes,
    } = &step.action
    else {
        return false;
    };
    stage == checked.stage()
        && outputs.len() == checked.outputs().len()
        && outputs
            .iter()
            .all(|label| checked.outputs().contains(label))
        && permissions
            .iter()
            .all(|permission| checked.permissions().contains(permission))
        && *max_output_bytes <= checked.max_output_bytes()
        && step.timeout_seconds <= checked.timeout_seconds()
}

/// Validate bounds, pin the module hash, and on Platform reject a module that
/// is not the fixed guest shape. Structural caps and the translation budget
/// run on the caller. A fitting body is passed to `Module::new` in the helper
/// under the manifest timeout, and that image is dropped. A result observed
/// at or after the deadline is [`Denial::Elapsed`] and is not admitted.
/// [`execute`] compiles again inside the step deadline.
pub fn check(document: &[u8]) -> Result<Checked, Denial> {
    let checked = decode_manifest(document)?;
    #[cfg(feature = "platform")]
    {
        let body_len = admit_sections(&checked.module)?;
        admit_translation_budget(checked.fuel, checked.timeout_seconds, body_len)?;
        let program = resolve_guest_program()?;
        validate_with(
            &program,
            &checked,
            std::time::Duration::from_secs(u64::from(checked.timeout_seconds)),
            PollTiming::IMMEDIATE,
        )?;
    }
    Ok(checked)
}

/// Hash, bounds, and structural admission for a request.
///
/// This does not call `Module::new`. The child does that inside the wall-clock
/// deadline. An over-budget body is still `fuel` or `timeout`.
#[cfg(feature = "platform")]
pub(crate) fn bind(document: &[u8]) -> Result<Checked, Denial> {
    let checked = decode_manifest(document)?;
    let body_len = admit_sections(&checked.module)?;
    admit_translation_budget(checked.fuel, checked.timeout_seconds, body_len)?;
    Ok(checked)
}

fn decode_manifest(document: &[u8]) -> Result<Checked, Denial> {
    if document.is_empty() {
        return Err(Denial::Malformed);
    }
    if document.len() > MAX_MANIFEST_BYTES {
        return Err(Denial::Limit);
    }
    let manifest: Manifest = serde_json::from_slice(document).map_err(|_| Denial::Malformed)?;
    if manifest.format != FORMAT {
        return Err(Denial::Malformed);
    }
    let stage = Id::new(manifest.stage).map_err(|_| Denial::Malformed)?;
    if manifest.outputs.is_empty() || manifest.outputs.len() > MAX_CUSTOM_OUTPUTS {
        return Err(Denial::Limit);
    }
    let mut outputs = Vec::with_capacity(manifest.outputs.len());
    for output in manifest.outputs {
        let label = Label::new(output).map_err(|_| Denial::Malformed)?;
        if RESERVED_SIGNALS.contains(&label.as_str()) {
            return Err(Denial::Limit);
        }
        outputs.push(label);
    }
    if outputs.iter().collect::<BTreeSet<_>>().len() != outputs.len() {
        return Err(Denial::Limit);
    }
    if manifest.network != "deny" || manifest.permissions.contains(&StagePermission::Network) {
        return Err(Denial::Permission);
    }
    if manifest.permissions.iter().collect::<BTreeSet<_>>().len() != manifest.permissions.len() {
        return Err(Denial::Permission);
    }
    let permissions: BTreeSet<_> = manifest.permissions.into_iter().collect();
    if !(1..=MAX_FUEL).contains(&manifest.fuel)
        || manifest.memory_bytes != MAX_MEMORY_BYTES
        || !(1..=MAX_INPUT_BYTES).contains(&manifest.max_input_bytes)
        || !(1..=MAX_OUTPUT_BYTES).contains(&manifest.max_output_bytes)
        || !(1..=MAX_TIMEOUT_SECONDS).contains(&manifest.timeout_seconds)
    {
        return Err(Denial::Limit);
    }
    let module = STANDARD
        .decode(manifest.module_base64.as_bytes())
        .map_err(|_| Denial::Malformed)?;
    let module_len = u32::try_from(module.len()).map_err(|_| Denial::Limit)?;
    if module_len == 0 || module_len > MAX_MODULE_BYTES || module_len > manifest.memory_bytes {
        return Err(Denial::Limit);
    }
    let module_sha256 = parse_sha256(&manifest.module_sha256)?;
    let digest: [u8; 32] = Sha256::digest(&module).into();
    if digest != module_sha256 {
        return Err(Denial::Integrity);
    }
    Ok(Checked {
        stage,
        outputs,
        permissions,
        fuel: manifest.fuel,
        memory_bytes: manifest.memory_bytes,
        max_input_bytes: manifest.max_input_bytes,
        max_output_bytes: manifest.max_output_bytes,
        timeout_seconds: manifest.timeout_seconds,
        module_len,
        module_sha256,
        module,
    })
}

/// Run `route` under the fuel, memory, timeout, and output caps.
///
/// On Platform this spawns the server binary and waits until the child exits
/// or the step timeout kills it. On Essentials this returns
/// [`Denial::ExternalRuntimeRequired`] and does not interpret the bytes or
/// spawn a process.
pub fn execute(
    checked: &Checked,
    facts: &GuestFacts,
    requested: &BTreeSet<StagePermission>,
    bounds: StepBounds,
) -> Result<Label, Denial> {
    #[cfg(not(feature = "platform"))]
    {
        let _ = (checked, facts, requested, bounds);
        Err(Denial::ExternalRuntimeRequired)
    }
    #[cfg(feature = "platform")]
    {
        execute_guest(checked, facts, requested, bounds)
    }
}

fn parse_sha256(value: &str) -> Result<[u8; 32], Denial> {
    if value.len() != 64 {
        return Err(Denial::Malformed);
    }
    let mut out = [0u8; 32];
    let bytes = value.as_bytes();
    for (index, byte) in out.iter_mut().enumerate() {
        let high = nibble(bytes[index * 2])?;
        let low = nibble(bytes[index * 2 + 1])?;
        *byte = (high << 4) | low;
    }
    Ok(out)
}

fn nibble(byte: u8) -> Result<u8, Denial> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(Denial::Malformed),
    }
}

#[cfg(feature = "platform")]
fn valid_fact(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..")
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
}

#[cfg(feature = "platform")]
fn project(
    facts: &GuestFacts,
    requested: &BTreeSet<StagePermission>,
    granted: &BTreeSet<StagePermission>,
    max_input_bytes: u32,
) -> Result<Vec<u8>, Denial> {
    if requested.contains(&StagePermission::Network) || !requested.is_subset(granted) {
        return Err(Denial::Permission);
    }
    let mut used = 0u32;
    let mut records = Vec::new();
    let mut take = |kind: u8, value: &str| -> Result<(), Denial> {
        if !valid_fact(value) {
            return Err(Denial::InputLimit);
        }
        let len = u32::try_from(value.len()).map_err(|_| Denial::InputLimit)?;
        used = used.checked_add(len).ok_or(Denial::InputLimit)?;
        if used > max_input_bytes {
            return Err(Denial::InputLimit);
        }
        records.push((kind, value.as_bytes().to_vec()));
        Ok(())
    };
    if requested.contains(&StagePermission::ReadProfile) {
        take(
            KIND_ACCOUNT,
            facts.account.as_deref().ok_or(Denial::InputLimit)?,
        )?;
    }
    if requested.contains(&StagePermission::ReadRequest) {
        take(
            KIND_REQUEST,
            facts.request.as_deref().ok_or(Denial::InputLimit)?,
        )?;
        take(
            KIND_CLIENT,
            facts.client.as_deref().ok_or(Denial::InputLimit)?,
        )?;
    }
    if requested.contains(&StagePermission::ReadGroups) {
        if facts.groups.len() > super::extension::MAX_GROUPS {
            return Err(Denial::InputLimit);
        }
        let mut groups = facts.groups.clone();
        groups.sort();
        for group in &groups {
            take(KIND_GROUP, group)?;
        }
    }
    let count = u32::try_from(records.len()).map_err(|_| Denial::InputLimit)?;
    let mut frame = count.to_le_bytes().to_vec();
    for (kind, bytes) in records {
        let len = u8::try_from(bytes.len()).map_err(|_| Denial::InputLimit)?;
        frame.push(kind);
        frame.push(len);
        frame.extend(bytes);
    }
    if frame.len() > INPUT_WINDOW {
        return Err(Denial::InputLimit);
    }
    Ok(frame)
}

#[cfg(feature = "platform")]
fn execute_guest(
    checked: &Checked,
    facts: &GuestFacts,
    requested: &BTreeSet<StagePermission>,
    bounds: StepBounds,
) -> Result<Label, Denial> {
    if bounds.timeout_seconds == 0
        || bounds.timeout_seconds > checked.timeout_seconds
        || bounds.max_output_bytes == 0
        || bounds.max_output_bytes > checked.max_output_bytes
    {
        return Err(Denial::Limit);
    }
    let mut frame = project(
        facts,
        requested,
        checked.permissions(),
        checked.max_input_bytes,
    )?;
    let timeout_fuel = u64::from(bounds.timeout_seconds).saturating_mul(FUEL_PER_SECOND);
    let manifest_fuel = u64::from(checked.fuel);
    let timeout_tighter = timeout_fuel < manifest_fuel;
    let fuel = timeout_fuel.min(manifest_fuel);
    // Structural charge only. Wasmi `Module::new` runs in the child.
    let body_len = admit_sections(&checked.module)?;
    admit_translation_budget(checked.fuel, bounds.timeout_seconds, body_len)?;
    let output_cap = bounds.max_output_bytes.min(MAX_ROUTE_LABEL_BYTES);
    let request = encode_request(MODE_RUN, timeout_tighter, fuel, checked, output_cap, &frame)?;
    zeroize::Zeroize::zeroize(frame.as_mut_slice());
    let program = resolve_guest_program()?;
    let supervised = supervise(
        &program,
        request,
        std::time::Duration::from_secs(u64::from(bounds.timeout_seconds)),
    )?;
    let label = supervised.outcome?;
    if u32::try_from(label.as_str().len()).unwrap_or(u32::MAX) > output_cap {
        return Err(Denial::Output);
    }
    if checked.outputs.iter().any(|candidate| candidate == &label) {
        Ok(label)
    } else {
        Err(Denial::UndeclaredOutput)
    }
}

#[cfg(feature = "platform")]
fn classify(error: &wasmi::Error, timeout_tighter: bool) -> Denial {
    match error.as_trap_code() {
        Some(wasmi::core::TrapCode::OutOfFuel) => {
            if timeout_tighter {
                Denial::Timeout
            } else {
                Denial::Fuel
            }
        }
        Some(
            wasmi::core::TrapCode::MemoryOutOfBounds | wasmi::core::TrapCode::TableOutOfBounds,
        ) => Denial::Limit,
        Some(wasmi::core::TrapCode::StackOverflow) => Denial::Limit,
        _ => Denial::Failed,
    }
}

#[cfg(feature = "platform")]
fn guest_config() -> wasmi::Config {
    let mut config = wasmi::Config::default();
    config.consume_fuel(true);
    config.ignore_custom_sections(true);
    // Validate at `Module::new`. Wasmi translates on the first call and charges
    // 7 fuel per function-body byte before that translation starts.
    config.compilation_mode(wasmi::CompilationMode::LazyTranslation);
    config.floats(false);
    config.wasm_multi_memory(false);
    config.wasm_bulk_memory(false);
    config.wasm_reference_types(false);
    config.wasm_tail_call(false);
    config.wasm_saturating_float_to_int(false);
    config.enforced_limits(wasmi::EnforcedLimits::strict());
    config.set_stack_limits(
        wasmi::StackLimits::new(
            GUEST_VALUE_STACK_SLOTS,
            GUEST_VALUE_STACK_SLOTS,
            GUEST_CALL_DEPTH,
        )
        .expect("guest value stack fits"),
    );
    config
}

#[cfg(feature = "platform")]
fn admit_translation_budget(fuel: u32, timeout_seconds: u32, body_len: u32) -> Result<(), Denial> {
    let manifest_fuel = u64::from(fuel);
    let timeout_fuel = u64::from(timeout_seconds).saturating_mul(FUEL_PER_SECOND);
    let budget = timeout_fuel.min(manifest_fuel);
    let charge = u64::from(body_len).saturating_mul(GUEST_TRANSLATION_FUEL_PER_BYTE);
    if charge > budget {
        if timeout_fuel < manifest_fuel {
            Err(Denial::Timeout)
        } else {
            Err(Denial::Fuel)
        }
    } else {
        Ok(())
    }
}

#[cfg(feature = "platform")]
fn fresh_module(wasm: &[u8]) -> Result<wasmi::Module, Denial> {
    let engine = wasmi::Engine::new(&guest_config());
    let module = wasmi::Module::new(&engine, wasm).map_err(|_| Denial::Malformed)?;
    if module.imports().next().is_some() {
        return Err(Denial::Permission);
    }
    let mut names = module
        .exports()
        .map(|export| export.name().to_owned())
        .collect::<Vec<_>>();
    names.sort();
    if names != ["memory", "route"] {
        return Err(Denial::Limit);
    }
    match module.get_export("route") {
        Some(wasmi::ExternType::Func(ty))
            if ty.params().is_empty() && ty.results() == [wasmi::core::ValType::I32] => {}
        _ => return Err(Denial::Limit),
    }
    match module.get_export("memory") {
        Some(wasmi::ExternType::Memory(ty))
            if u32::from(ty.initial_pages()) == 1
                && ty
                    .maximum_pages()
                    .is_some_and(|pages| u32::from(pages) == 1) => {}
        _ => return Err(Denial::Limit),
    }
    Ok(module)
}

#[cfg(feature = "platform")]
fn admit_sections(wasm: &[u8]) -> Result<u32, Denial> {
    if wasm.len() < 8 || wasm[0..4] != [0x00, b'a', b's', b'm'] || wasm[4..8] != [1, 0, 0, 0] {
        return Err(Denial::Malformed);
    }
    let mut cursor = 8usize;
    let mut last = 0u8;
    let mut saw_type = false;
    let mut saw_function = false;
    let mut saw_memory = false;
    let mut saw_code = false;
    let mut body_len = None;
    while cursor < wasm.len() {
        let id = wasm[cursor];
        cursor += 1;
        let (size, read) = read_leb(&wasm[cursor..])?;
        cursor += read;
        let size = usize::try_from(size).map_err(|_| Denial::Malformed)?;
        let end = cursor.checked_add(size).ok_or(Denial::Malformed)?;
        if end > wasm.len() {
            return Err(Denial::Malformed);
        }
        let payload = &wasm[cursor..end];
        cursor = end;
        if id == 0 {
            continue;
        }
        if id <= last || id > 12 {
            return Err(Denial::Malformed);
        }
        last = id;
        match id {
            1 => {
                saw_type = true;
                admit_type(payload)?;
            }
            2 => {
                let (count, _) = read_leb(payload)?;
                if count != 0 {
                    return Err(Denial::Permission);
                }
            }
            3 => {
                saw_function = true;
                admit_functions(payload)?;
            }
            4 | 6 | 9 => {
                let (count, _) = read_leb(payload)?;
                if count != 0 {
                    return Err(Denial::Limit);
                }
            }
            5 => {
                saw_memory = true;
                admit_memory(payload)?;
            }
            7 => admit_exports(payload)?,
            8 => return Err(Denial::Limit),
            10 => {
                saw_code = true;
                body_len = Some(admit_code(payload)?);
            }
            11 => admit_data(payload)?,
            _ => {}
        }
    }
    if !saw_type || !saw_function || !saw_memory || !saw_code {
        return Err(Denial::Limit);
    }
    body_len.ok_or(Denial::Limit)
}

#[cfg(feature = "platform")]
fn admit_type(payload: &[u8]) -> Result<(), Denial> {
    let (count, mut cursor) = take_leb(payload, 0)?;
    if count != 1 {
        return Err(Denial::Limit);
    }
    if payload.get(cursor) != Some(&0x60) {
        return Err(Denial::Malformed);
    }
    cursor += 1;
    let (params, cursor) = take_leb(payload, cursor)?;
    if params != 0 {
        return Err(Denial::Limit);
    }
    let (results, cursor) = take_leb(payload, cursor)?;
    if results != 1 || payload.get(cursor) != Some(&0x7f) || cursor + 1 != payload.len() {
        return Err(Denial::Limit);
    }
    Ok(())
}

#[cfg(feature = "platform")]
fn admit_functions(payload: &[u8]) -> Result<(), Denial> {
    let (count, cursor) = take_leb(payload, 0)?;
    if count != MAX_GUEST_FUNCTIONS {
        return Err(Denial::Limit);
    }
    let (index, cursor) = take_leb(payload, cursor)?;
    if index != 0 {
        return Err(Denial::Limit);
    }
    if cursor != payload.len() {
        return Err(Denial::Malformed);
    }
    Ok(())
}

#[cfg(feature = "platform")]
fn admit_exports(payload: &[u8]) -> Result<(), Denial> {
    let (count, _) = take_leb(payload, 0)?;
    if count != 2 {
        return Err(Denial::Limit);
    }
    Ok(())
}

#[cfg(feature = "platform")]
fn admit_code(payload: &[u8]) -> Result<u32, Denial> {
    let (count, cursor) = take_leb(payload, 0)?;
    if count != MAX_GUEST_FUNCTIONS {
        return Err(Denial::Limit);
    }
    let (body_len, cursor) = take_leb(payload, cursor)?;
    let size = usize::try_from(body_len).map_err(|_| Denial::Malformed)?;
    let end = cursor.checked_add(size).ok_or(Denial::Malformed)?;
    if end != payload.len() {
        return Err(Denial::Malformed);
    }
    let body = &payload[cursor..end];
    let (groups, mut local_cursor) = take_leb(body, 0)?;
    if groups > MAX_GUEST_LOCALS {
        return Err(Denial::Limit);
    }
    let mut locals = 0u32;
    for _ in 0..groups {
        let (count, next) = take_leb(body, local_cursor)?;
        let type_at = next;
        if type_at >= body.len() {
            return Err(Denial::Malformed);
        }
        if body[type_at] != 0x7f {
            return Err(Denial::Limit);
        }
        local_cursor = type_at + 1;
        locals = locals.checked_add(count).ok_or(Denial::Limit)?;
        if locals > MAX_GUEST_LOCALS {
            return Err(Denial::Limit);
        }
    }
    Ok(body_len)
}

#[cfg(feature = "platform")]
fn admit_data(payload: &[u8]) -> Result<(), Denial> {
    let (count, cursor) = take_leb(payload, 0)?;
    if count != 0 || cursor != payload.len() {
        return Err(Denial::Limit);
    }
    Ok(())
}

#[cfg(feature = "platform")]
fn take_leb(bytes: &[u8], cursor: usize) -> Result<(u32, usize), Denial> {
    let (value, read) = read_leb(bytes.get(cursor..).ok_or(Denial::Malformed)?)?;
    Ok((value, cursor + read))
}

#[cfg(feature = "platform")]
fn admit_memory(payload: &[u8]) -> Result<(), Denial> {
    let (count, mut cursor) = read_leb(payload)?;
    if count != 1 || cursor >= payload.len() {
        return Err(Denial::Limit);
    }
    let flags = payload[cursor];
    cursor += 1;
    if flags != 0x01 {
        return Err(Denial::Limit);
    }
    let (min, read) = read_leb(&payload[cursor..])?;
    cursor += read;
    let (max, read) = read_leb(&payload[cursor..])?;
    cursor += read;
    if cursor != payload.len() || min != 1 || max != 1 {
        return Err(Denial::Limit);
    }
    Ok(())
}

#[cfg(feature = "platform")]
fn read_leb(bytes: &[u8]) -> Result<(u32, usize), Denial> {
    let mut result = 0u32;
    let mut shift = 0;
    for (index, byte) in bytes.iter().enumerate().take(5) {
        let value = u32::from(byte & 0x7f);
        if shift == 28 && value > 0x0f {
            return Err(Denial::Malformed);
        }
        result |= value << shift;
        if byte & 0x80 == 0 {
            return Ok((result, index + 1));
        }
        shift += 7;
    }
    Err(Denial::Malformed)
}

#[cfg(feature = "platform")]
const REQUEST_MAGIC: &[u8; 8] = b"RIAUTHXG";
#[cfg(feature = "platform")]
const REQUEST_VERSION: u8 = 1;
#[cfg(feature = "platform")]
const MODE_RUN: u8 = 0;
#[cfg(feature = "platform")]
const MODE_HOLD: u8 = 1;
#[cfg(feature = "platform")]
const MODE_FLOOD: u8 = 2;
#[cfg(feature = "platform")]
const MODE_VALIDATE: u8 = 3;
#[cfg(feature = "platform")]
const STATUS_LABEL: u8 = 0;
#[cfg(feature = "platform")]
const STATUS_DENIAL: u8 = 1;
#[cfg(feature = "platform")]
const STATUS_VALID: u8 = 2;
#[cfg(feature = "platform")]
const REQUEST_HEADER_LEN: usize = 36;
#[cfg(feature = "platform")]
const MAX_GUEST_REQUEST_BYTES: usize = REQUEST_HEADER_LEN
    + MAX_CUSTOM_OUTPUTS * (1 + MAX_ROUTE_LABEL_BYTES as usize)
    + MAX_MODULE_BYTES as usize
    + INPUT_WINDOW;
#[cfg(feature = "platform")]
const MAX_RESPONSE_BYTES: usize = 64;

#[cfg(feature = "platform")]
fn guest_program(current: &std::path::Path) -> &std::path::Path {
    current
}

#[cfg(feature = "platform")]
fn resolve_guest_program() -> Result<std::path::PathBuf, Denial> {
    // Unsupported hosts must refuse without relying on executable discovery
    // (or on a companion server having been built for a unit-test harness).
    if !cfg!(target_os = "macos") {
        return Err(Denial::ExternalRuntimeRequired);
    }
    let current = std::env::current_exe().map_err(|_| Denial::Failed)?;
    #[cfg(test)]
    if let Some(server) = harness_server(&current) {
        if server.is_file() {
            return Ok(server);
        }
        return Err(Denial::Failed);
    }
    Ok(guest_program(&current).to_path_buf())
}

#[cfg(all(feature = "platform", test))]
fn harness_server(current: &std::path::Path) -> Option<std::path::PathBuf> {
    let name = current.file_name()?.to_str()?;
    let parent = current.parent()?;
    if parent.file_name()?.to_str()? != "deps" || !name.starts_with("riauth-") {
        return None;
    }
    Some(parent.parent()?.join("riauth"))
}

#[cfg(feature = "platform")]
struct Supervised {
    outcome: Result<Label, Denial>,
    #[cfg(all(test, target_os = "macos"))]
    pid: u32,
}

#[cfg(feature = "platform")]
fn encode_request(
    mode: u8,
    timeout_tighter: bool,
    fuel: u64,
    checked: &Checked,
    output_cap: u32,
    frame: &[u8],
) -> Result<Vec<u8>, Denial> {
    if checked.outputs.is_empty()
        || checked.outputs.len() > MAX_CUSTOM_OUTPUTS
        || frame.len() > INPUT_WINDOW
        || checked.module.is_empty()
        || checked.module.len() > usize::try_from(MAX_MODULE_BYTES).unwrap_or(usize::MAX)
        || output_cap == 0
        || output_cap > MAX_ROUTE_LABEL_BYTES
        || fuel == 0
        || fuel > u64::from(MAX_FUEL)
    {
        return Err(Denial::Limit);
    }
    let mut request = Vec::new();
    request.extend_from_slice(REQUEST_MAGIC);
    request.push(REQUEST_VERSION);
    request.push(mode);
    request.push(u8::from(timeout_tighter));
    request.extend_from_slice(&fuel.to_le_bytes());
    request.extend_from_slice(&checked.memory_bytes.to_le_bytes());
    request.extend_from_slice(&output_cap.to_le_bytes());
    let count = u8::try_from(checked.outputs.len()).map_err(|_| Denial::Limit)?;
    request.push(count);
    let module_len = u32::try_from(checked.module.len()).map_err(|_| Denial::Limit)?;
    let frame_len = u32::try_from(frame.len()).map_err(|_| Denial::Limit)?;
    request.extend_from_slice(&module_len.to_le_bytes());
    request.extend_from_slice(&frame_len.to_le_bytes());
    for label in &checked.outputs {
        let bytes = label.as_str().as_bytes();
        let len = u8::try_from(bytes.len()).map_err(|_| Denial::Limit)?;
        if len == 0 {
            return Err(Denial::Limit);
        }
        request.push(len);
        request.extend_from_slice(bytes);
    }
    request.extend_from_slice(&checked.module);
    request.extend_from_slice(frame);
    if request.len() > MAX_GUEST_REQUEST_BYTES {
        return Err(Denial::Limit);
    }
    Ok(request)
}

#[cfg(feature = "platform")]
struct DecodedRequest {
    mode: u8,
    timeout_tighter: bool,
    fuel: u64,
    memory_bytes: u32,
    output_cap: u32,
    labels: Vec<Vec<u8>>,
    module: Vec<u8>,
    frame: Vec<u8>,
}

#[cfg(feature = "platform")]
fn read_array<const N: usize>(bytes: &[u8], at: usize) -> Result<[u8; N], Denial> {
    let end = at.checked_add(N).ok_or(Denial::Failed)?;
    bytes
        .get(at..end)
        .ok_or(Denial::Failed)?
        .try_into()
        .map_err(|_| Denial::Failed)
}

#[cfg(feature = "platform")]
fn decode_request(bytes: &[u8]) -> Result<DecodedRequest, Denial> {
    if bytes.len() < REQUEST_HEADER_LEN || bytes.len() > MAX_GUEST_REQUEST_BYTES {
        return Err(Denial::Failed);
    }
    if bytes.get(..8) != Some(REQUEST_MAGIC.as_slice()) || bytes[8] != REQUEST_VERSION {
        return Err(Denial::Failed);
    }
    let mode = bytes[9];
    if mode > MODE_VALIDATE {
        return Err(Denial::Failed);
    }
    let timeout_tighter = match bytes[10] {
        0 => false,
        1 => true,
        _ => return Err(Denial::Failed),
    };
    let fuel = u64::from_le_bytes(read_array(bytes, 11)?);
    let memory_bytes = u32::from_le_bytes(read_array(bytes, 19)?);
    let output_cap = u32::from_le_bytes(read_array(bytes, 23)?);
    let label_count = usize::from(bytes[27]);
    let module_len =
        usize::try_from(u32::from_le_bytes(read_array(bytes, 28)?)).unwrap_or(usize::MAX);
    let frame_len =
        usize::try_from(u32::from_le_bytes(read_array(bytes, 32)?)).unwrap_or(usize::MAX);
    if label_count == 0
        || label_count > MAX_CUSTOM_OUTPUTS
        || module_len == 0
        || module_len > usize::try_from(MAX_MODULE_BYTES).unwrap_or(usize::MAX)
        || frame_len > INPUT_WINDOW
        || output_cap == 0
        || output_cap > MAX_ROUTE_LABEL_BYTES
        || memory_bytes != MAX_MEMORY_BYTES
        || fuel == 0
        || fuel > u64::from(MAX_FUEL)
    {
        return Err(Denial::Failed);
    }
    let mut cursor = REQUEST_HEADER_LEN;
    let mut labels = Vec::with_capacity(label_count);
    for _ in 0..label_count {
        let len = usize::from(*bytes.get(cursor).ok_or(Denial::Failed)?);
        cursor += 1;
        if len == 0 || len > usize::try_from(MAX_ROUTE_LABEL_BYTES).unwrap_or(usize::MAX) {
            return Err(Denial::Failed);
        }
        let end = cursor.checked_add(len).ok_or(Denial::Failed)?;
        labels.push(bytes.get(cursor..end).ok_or(Denial::Failed)?.to_vec());
        cursor = end;
    }
    let module_end = cursor.checked_add(module_len).ok_or(Denial::Failed)?;
    let frame_end = module_end.checked_add(frame_len).ok_or(Denial::Failed)?;
    if frame_end != bytes.len() {
        return Err(Denial::Failed);
    }
    Ok(DecodedRequest {
        mode,
        timeout_tighter,
        fuel,
        memory_bytes,
        output_cap,
        labels,
        module: bytes[cursor..module_end].to_vec(),
        frame: bytes[module_end..frame_end].to_vec(),
    })
}

#[cfg(feature = "platform")]
fn decode_response(bytes: &[u8]) -> Result<Label, Denial> {
    match bytes {
        [STATUS_LABEL, len, rest @ ..] => {
            let len = usize::from(*len);
            if len == 0
                || len > usize::try_from(MAX_ROUTE_LABEL_BYTES).unwrap_or(32)
                || rest.len() != len
            {
                return Err(Denial::Failed);
            }
            let text = std::str::from_utf8(rest).map_err(|_| Denial::Failed)?;
            Label::new(text).map_err(|_| Denial::Failed)
        }
        [STATUS_DENIAL, code] => Err(denial_from_code(*code).unwrap_or(Denial::Failed)),
        _ => Err(Denial::Failed),
    }
}

#[cfg(feature = "platform")]
struct Captured {
    bytes: [u8; MAX_RESPONSE_BYTES + 1],
    len: usize,
    overflow: bool,
    io_error: bool,
}

#[cfg(feature = "platform")]
fn read_capped(
    mut stdout: std::process::ChildStdout,
    overflowed: &std::sync::atomic::AtomicBool,
) -> Captured {
    use std::io::Read;
    use std::sync::atomic::Ordering;
    let mut bytes = [0u8; MAX_RESPONSE_BYTES + 1];
    let mut len = 0usize;
    loop {
        if len >= bytes.len() {
            overflowed.store(true, Ordering::Release);
            return Captured {
                bytes,
                len: MAX_RESPONSE_BYTES,
                overflow: true,
                io_error: false,
            };
        }
        match stdout.read(&mut bytes[len..]) {
            Ok(0) => break,
            Ok(count) => len += count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => {
                return Captured {
                    bytes,
                    len,
                    overflow: false,
                    io_error: true,
                };
            }
        }
    }
    let overflow = len > MAX_RESPONSE_BYTES;
    if overflow {
        overflowed.store(true, Ordering::Release);
    }
    Captured {
        bytes,
        len: len.min(MAX_RESPONSE_BYTES),
        overflow,
        io_error: false,
    }
}

#[cfg(feature = "platform")]
fn write_request(mut stdin: std::process::ChildStdin, mut request: Vec<u8>) {
    let _ = std::io::Write::write_all(&mut stdin, &request);
    let _ = std::io::Write::flush(&mut stdin);
    zeroize::Zeroize::zeroize(request.as_mut_slice());
    drop(stdin);
}

#[cfg(feature = "platform")]
struct GuestProcess {
    child: std::process::Child,
    reader: Option<std::thread::JoinHandle<Captured>>,
    writer: Option<std::thread::JoinHandle<()>>,
    workdir: std::path::PathBuf,
    finished: bool,
}

#[cfg(feature = "platform")]
impl GuestProcess {
    fn kill_wait(&mut self) {
        if self.finished {
            return;
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.finished = true;
    }

    fn join_io(&mut self) -> Captured {
        if let Some(writer) = self.writer.take() {
            let _ = writer.join();
        }
        if let Some(reader) = self.reader.take() {
            return reader.join().unwrap_or(Captured {
                bytes: [0; MAX_RESPONSE_BYTES + 1],
                len: 0,
                overflow: false,
                io_error: true,
            });
        }
        Captured {
            bytes: [0; MAX_RESPONSE_BYTES + 1],
            len: 0,
            overflow: false,
            io_error: true,
        }
    }
}

#[cfg(feature = "platform")]
impl Drop for GuestProcess {
    fn drop(&mut self) {
        self.kill_wait();
        let _ = self.join_io();
        let _ = std::fs::remove_dir_all(&self.workdir);
    }
}

#[cfg(feature = "platform")]
fn guest_workdir() -> Result<std::path::PathBuf, Denial> {
    guest_workdir_in(&std::env::temp_dir(), || {
        use rand::TryRng;
        let mut nonce = [0u8; 16];
        rand::rngs::SysRng
            .try_fill_bytes(&mut nonce)
            .map_err(|_| Denial::Failed)?;
        Ok(nonce)
    })
}

#[cfg(feature = "platform")]
fn guest_workdir_in(
    root: &std::path::Path,
    mut next_nonce: impl FnMut() -> Result<[u8; 16], Denial>,
) -> Result<std::path::PathBuf, Denial> {
    const ATTEMPTS: usize = 8;
    for _ in 0..ATTEMPTS {
        let nonce = next_nonce()?;
        let path = root.join(format!("riauth-guest-{:032x}", u128::from_be_bytes(nonce)));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(Denial::Failed),
        }
    }
    Err(Denial::Failed)
}

#[cfg(feature = "platform")]
fn failed_child(mut child: std::process::Child, workdir: &std::path::Path) {
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(workdir);
}

/// Parent-side delays around observation. Production uses [`PollTiming::IMMEDIATE`].
///
/// Tests place the parent after a child that has already finished. The delays
/// are not a model of the kernel scheduler.
#[cfg(feature = "platform")]
#[derive(Clone, Copy)]
struct PollTiming {
    before_poll: std::time::Duration,
    after_join: std::time::Duration,
}

#[cfg(feature = "platform")]
impl PollTiming {
    const IMMEDIATE: Self = Self {
        before_poll: std::time::Duration::ZERO,
        after_join: std::time::Duration::ZERO,
    };
}

#[cfg(feature = "platform")]
struct Observed {
    status: std::process::ExitStatus,
    captured: Captured,
}

#[cfg(feature = "platform")]
struct Launched {
    result: Result<Observed, Denial>,
    #[cfg(all(test, target_os = "macos"))]
    pid: u32,
}

#[cfg(feature = "platform")]
fn supervise(
    program: &std::path::Path,
    request: Vec<u8>,
    timeout: std::time::Duration,
) -> Result<Supervised, Denial> {
    supervise_timed(program, request, timeout, PollTiming::IMMEDIATE)
}

#[cfg(feature = "platform")]
fn supervise_timed(
    program: &std::path::Path,
    request: Vec<u8>,
    timeout: std::time::Duration,
    timing: PollTiming,
) -> Result<Supervised, Denial> {
    let launched = launch(program, request, timeout, timing)?;
    let outcome = match launched.result {
        Ok(observed) => interpret(observed.status, observed.captured),
        Err(denial) => Err(denial),
    };
    Ok(Supervised {
        outcome,
        #[cfg(all(test, target_os = "macos"))]
        pid: launched.pid,
    })
}

/// `Module::new` for configuration admission. Success is not a workflow label.
#[cfg(feature = "platform")]
fn validate_with(
    program: &std::path::Path,
    checked: &Checked,
    timeout: std::time::Duration,
    timing: PollTiming,
) -> Result<(), Denial> {
    let output_cap = checked.max_output_bytes.min(MAX_ROUTE_LABEL_BYTES);
    let request = encode_request(
        MODE_VALIDATE,
        false,
        u64::from(checked.fuel),
        checked,
        output_cap,
        &[],
    )?;
    let launched = launch(program, request, timeout, timing)?;
    match launched.result {
        Ok(observed) => interpret_validation(observed.status, observed.captured),
        Err(denial) => Err(denial),
    }
}

#[cfg(feature = "platform")]
fn launch(
    program: &std::path::Path,
    request: Vec<u8>,
    timeout: std::time::Duration,
    timing: PollTiming,
) -> Result<Launched, Denial> {
    if request.is_empty() || request.len() > MAX_GUEST_REQUEST_BYTES || timeout.is_zero() {
        return Err(Denial::Failed);
    }
    // Resolve the confinement backend before creating resources. Unsupported
    // hosts and missing sandbox setup never fall back to an ordinary child.
    let mut command = isolation::command(program)?;
    let workdir = guest_workdir()?;
    command
        .arg(GUEST_ARGV)
        .env_clear()
        .current_dir(&workdir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let started = std::time::Instant::now();
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&workdir);
            return Err(Denial::Failed);
        }
    };
    #[cfg(all(test, target_os = "macos"))]
    let pid = child.id();
    let stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            failed_child(child, &workdir);
            return Err(Denial::Failed);
        }
    };
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            drop(stdin);
            failed_child(child, &workdir);
            return Err(Denial::Failed);
        }
    };
    let overflowed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let writer = std::thread::spawn(move || write_request(stdin, request));
    let reader_flag = std::sync::Arc::clone(&overflowed);
    let reader = std::thread::spawn(move || read_capped(stdout, &reader_flag));
    let mut guest = GuestProcess {
        child,
        reader: Some(reader),
        writer: Some(writer),
        workdir,
        finished: false,
    };
    let result = wait_for_guest(&mut guest, started + timeout, &overflowed, timing);
    Ok(Launched {
        result,
        #[cfg(all(test, target_os = "macos"))]
        pid,
    })
}

#[cfg(feature = "platform")]
fn wait_for_guest(
    guest: &mut GuestProcess,
    deadline: std::time::Instant,
    overflowed: &std::sync::atomic::AtomicBool,
    timing: PollTiming,
) -> Result<Observed, Denial> {
    if !timing.before_poll.is_zero() {
        std::thread::sleep(timing.before_poll);
    }
    loop {
        if std::time::Instant::now() >= deadline {
            return Err(expire(guest));
        }
        if overflowed.load(std::sync::atomic::Ordering::Acquire) {
            guest.kill_wait();
            discard_output(guest.join_io());
            return Err(Denial::Output);
        }
        match guest.child.try_wait() {
            Ok(Some(status)) => {
                // `try_wait` has reaped the pid. Do not signal it again.
                guest.finished = true;
                if std::time::Instant::now() >= deadline {
                    discard_output(guest.join_io());
                    return Err(Denial::Elapsed);
                }
                let captured = guest.join_io();
                if !timing.after_join.is_zero() {
                    std::thread::sleep(timing.after_join);
                }
                if std::time::Instant::now() >= deadline {
                    discard_output(captured);
                    return Err(Denial::Elapsed);
                }
                return Ok(Observed { status, captured });
            }
            Ok(None) => {}
            Err(_) => {
                guest.kill_wait();
                discard_output(guest.join_io());
                return Err(Denial::Failed);
            }
        }
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            continue;
        }
        std::thread::sleep(remaining.min(std::time::Duration::from_millis(10)));
    }
}

/// Reap a child that has reached the deadline and drop whatever it wrote.
#[cfg(feature = "platform")]
fn expire(guest: &mut GuestProcess) -> Denial {
    match guest.child.try_wait() {
        Ok(Some(_)) => guest.finished = true,
        Ok(None) => guest.kill_wait(),
        Err(_) => {
            guest.kill_wait();
            discard_output(guest.join_io());
            return Denial::Failed;
        }
    }
    discard_output(guest.join_io());
    Denial::Elapsed
}

#[cfg(feature = "platform")]
fn discard_output(mut captured: Captured) {
    zeroize::Zeroize::zeroize(&mut captured.bytes);
}

#[cfg(feature = "platform")]
fn interpret(status: std::process::ExitStatus, captured: Captured) -> Result<Label, Denial> {
    if captured.overflow {
        return Err(Denial::Output);
    }
    if captured.io_error || !status.success() {
        return Err(Denial::Failed);
    }
    decode_response(&captured.bytes[..captured.len])
}

#[cfg(feature = "platform")]
fn interpret_validation(
    status: std::process::ExitStatus,
    captured: Captured,
) -> Result<(), Denial> {
    if captured.overflow {
        return Err(Denial::Output);
    }
    if captured.io_error || !status.success() {
        return Err(Denial::Failed);
    }
    match captured.bytes[..captured.len] {
        [STATUS_VALID] => Ok(()),
        [STATUS_DENIAL, code] => Err(denial_from_code(code).unwrap_or(Denial::Failed)),
        _ => Err(Denial::Failed),
    }
}

#[cfg(feature = "platform")]
fn invoke_guest(request: &mut DecodedRequest) -> Result<Label, Denial> {
    let outcome = invoke_guest_inner(request);
    zeroize::Zeroize::zeroize(request.frame.as_mut_slice());
    outcome
}

#[cfg(feature = "platform")]
fn invoke_guest_inner(request: &DecodedRequest) -> Result<Label, Denial> {
    let body_len = admit_sections(&request.module)?;
    let charge = u64::from(body_len).saturating_mul(GUEST_TRANSLATION_FUEL_PER_BYTE);
    if charge > request.fuel {
        return Err(if request.timeout_tighter {
            Denial::Timeout
        } else {
            Denial::Fuel
        });
    }
    let module = fresh_module(&request.module)?;
    let mut labels = Vec::with_capacity(request.labels.len());
    for bytes in &request.labels {
        let text = std::str::from_utf8(bytes).map_err(|_| Denial::Failed)?;
        labels.push(Label::new(text).map_err(|_| Denial::Failed)?);
    }
    let engine = module.engine().clone();
    let mut store = wasmi::Store::new(
        &engine,
        wasmi::StoreLimitsBuilder::new()
            .memory_size(usize::try_from(request.memory_bytes).map_err(|_| Denial::Limit)?)
            .table_elements(0)
            .instances(1)
            .tables(0)
            .memories(1)
            .trap_on_grow_failure(true)
            .build(),
    );
    store.limiter(|limits| limits);
    store.set_fuel(request.fuel).map_err(|_| Denial::Failed)?;
    let linker = wasmi::Linker::<wasmi::StoreLimits>::new(&engine);
    let instance = linker
        .instantiate(&mut store, &module)
        .and_then(|ready| ready.start(&mut store))
        .map_err(|error| classify(&error, request.timeout_tighter))?;
    let memory = instance.get_memory(&store, "memory").ok_or(Denial::Limit)?;
    let mut window = vec![0u8; INPUT_WINDOW];
    window[..request.frame.len()].copy_from_slice(&request.frame);
    memory
        .write(&mut store, INPUT_OFFSET, &window)
        .map_err(|_| Denial::Limit)?;
    zeroize::Zeroize::zeroize(window.as_mut_slice());
    let route = instance
        .get_typed_func::<(), i32>(&store, "route")
        .map_err(|_| Denial::Limit)?;
    let length = match route.call(&mut store, ()) {
        Ok(length) => length,
        Err(error) => return Err(classify(&error, request.timeout_tighter)),
    };
    // Manifest caps reach 4,096 bytes. A label is at most 32, so the copy stays there.
    let Ok(reported) = u32::try_from(length) else {
        return Err(Denial::Output);
    };
    if reported > request.output_cap {
        return Err(Denial::Output);
    }
    let reported = usize::try_from(reported).map_err(|_| Denial::Output)?;
    if !labels.iter().any(|label| label.as_str().len() == reported) {
        return Err(Denial::UndeclaredOutput);
    }
    let mut output = vec![0u8; reported];
    if memory.read(&store, 0, &mut output).is_err() {
        zeroize::Zeroize::zeroize(output.as_mut_slice());
        return Err(Denial::Output);
    }
    let matched = labels
        .into_iter()
        .find(|label| label.as_str().as_bytes() == output.as_slice());
    zeroize::Zeroize::zeroize(output.as_mut_slice());
    matched.ok_or(Denial::UndeclaredOutput)
}

/// Empty environment, or only macOS `__CF_USER_TEXT_ENCODING` in its short hex form.
#[cfg(feature = "platform")]
fn guest_environment_is_closed() -> bool {
    let mut vars = std::env::vars_os();
    match vars.next() {
        None => true,
        Some((key, value)) => {
            vars.next().is_none()
                && cfg!(target_os = "macos")
                && key == "__CF_USER_TEXT_ENCODING"
                && corefoundation_text_encoding(&value)
        }
    }
}

#[cfg(feature = "platform")]
fn corefoundation_text_encoding(value: &std::ffi::OsStr) -> bool {
    let Some(text) = value.to_str() else {
        return false;
    };
    if text.len() > 32 {
        return false;
    }
    let mut fields = text.split(':');
    fields.next().is_some_and(hex_field)
        && fields.next().is_some_and(hex_field)
        && fields.next().is_some_and(hex_field)
        && fields.next().is_none()
}

#[cfg(feature = "platform")]
fn hex_field(text: &str) -> bool {
    let Some(digits) = text.strip_prefix("0x") else {
        return false;
    };
    (1..=8).contains(&digits.len()) && digits.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Platform server entry. Reads one bounded request and writes one status.
///
/// The maintenance binary does not call this, and clap does not advertise it.
#[cfg(feature = "platform")]
pub fn run_isolated_guest() -> i32 {
    if isolated_guest_entry().is_ok() { 0 } else { 1 }
}

#[cfg(feature = "platform")]
fn isolated_guest_entry() -> Result<(), ()> {
    use std::io::Read;
    if !guest_environment_is_closed() {
        return write_status(Err(Denial::Failed)).map_err(|_| ());
    }
    if let Err(denial) = isolation::check_entry() {
        return write_status(Err(denial)).map_err(|_| ());
    }
    let mut input = Vec::new();
    let limit = u64::try_from(MAX_GUEST_REQUEST_BYTES)
        .map_err(|_| ())?
        .saturating_add(1);
    let mut stdin = std::io::stdin().take(limit);
    stdin.read_to_end(&mut input).map_err(|_| ())?;
    if input.len() > MAX_GUEST_REQUEST_BYTES {
        return write_status(Err(Denial::Failed)).map_err(|_| ());
    }
    let mut request = match decode_request(&input) {
        Ok(request) => request,
        Err(denial) => return write_status(Err(denial)).map_err(|_| ()),
    };
    zeroize::Zeroize::zeroize(input.as_mut_slice());
    match request.mode {
        MODE_HOLD => hold_or_refuse(),
        MODE_FLOOD => flood_or_refuse(),
        MODE_RUN => {
            let outcome = invoke_guest(&mut request);
            write_status(outcome).map_err(|_| ())
        }
        MODE_VALIDATE => validate_module(&request),
        _ => write_status(Err(Denial::Failed)).map_err(|_| ()),
    }
}

#[cfg(feature = "platform")]
fn validate_module(request: &DecodedRequest) -> Result<(), ()> {
    let outcome = fresh_module(&request.module).map(|_| ());
    write_validation(outcome).map_err(|_| ())
}

#[cfg(feature = "platform")]
fn write_validation(outcome: Result<(), Denial>) -> std::io::Result<()> {
    match outcome {
        Ok(()) => {
            let mut out = std::io::stdout().lock();
            std::io::Write::write_all(&mut out, &[STATUS_VALID])?;
            std::io::Write::flush(&mut out)
        }
        Err(denial) => write_status(Err(denial)),
    }
}

#[cfg(feature = "platform")]
fn hold_or_refuse() -> Result<(), ()> {
    #[cfg(debug_assertions)]
    {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }
    #[cfg(not(debug_assertions))]
    {
        write_status(Err(Denial::Failed)).map_err(|_| ())
    }
}

#[cfg(feature = "platform")]
fn flood_or_refuse() -> Result<(), ()> {
    #[cfg(debug_assertions)]
    {
        let mut out = std::io::stdout().lock();
        let junk = [b'A'; 200];
        std::io::Write::write_all(&mut out, &junk).map_err(|_| ())?;
        std::io::Write::flush(&mut out).map_err(|_| ())?;
        Ok(())
    }
    #[cfg(not(debug_assertions))]
    {
        write_status(Err(Denial::Failed)).map_err(|_| ())
    }
}

#[cfg(feature = "platform")]
fn write_status(outcome: Result<Label, Denial>) -> std::io::Result<()> {
    let mut buf = [0u8; 2 + 32];
    let written = match &outcome {
        Ok(label) => {
            let bytes = label.as_str().as_bytes();
            if bytes.is_empty()
                || bytes.len() > usize::try_from(MAX_ROUTE_LABEL_BYTES).unwrap_or(32)
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "label",
                ));
            }
            buf[0] = STATUS_LABEL;
            buf[1] = u8::try_from(bytes.len())
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "label"))?;
            buf[2..2 + bytes.len()].copy_from_slice(bytes);
            2 + bytes.len()
        }
        Err(denial) => {
            buf[0] = STATUS_DENIAL;
            buf[1] = denial.code();
            2
        }
    };
    let mut out = std::io::stdout().lock();
    std::io::Write::write_all(&mut out, &buf[..written])?;
    std::io::Write::flush(&mut out)
}

#[cfg(all(feature = "platform", test, target_os = "macos"))]
fn process_alive(pid: u32) -> bool {
    std::process::Command::new("/bin/kill")
        .arg("-0")
        .arg(pid.to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(test)]
#[cfg_attr(
    any(not(feature = "platform"), not(target_os = "macos")),
    allow(dead_code)
)]
pub(crate) mod fixture {
    pub fn allow() -> Vec<u8> {
        module(&store_label(*b"allo", Some(b'w')), &i32_const(5), &[])
    }

    /// `allow`, then `return`, then `nops` bytes that execution never reaches.
    ///
    /// Wasmi counts those bytes in the translation charge.
    pub fn allow_with_unreachable_tail(nops: usize) -> Vec<u8> {
        let mut body = store_label(*b"allo", Some(b'w'));
        body.extend(i32_const(5));
        body.push(0x0f);
        body.extend(vec![0x01; nops]);
        module(&body, &[], &[])
    }

    /// `allow`, then `return`, then `bytes` of opcode `0xff`.
    ///
    /// The tail is inside the function body and is not a valid instruction.
    pub fn allow_with_invalid_tail(bytes: usize) -> Vec<u8> {
        let mut body = store_label(*b"allo", Some(b'w'));
        body.extend(i32_const(5));
        body.push(0x0f);
        body.extend(vec![0xff; bytes]);
        module(&body, &[], &[])
    }

    pub fn block() -> Vec<u8> {
        module(&store_label(*b"bloc", Some(b'k')), &i32_const(5), &[])
    }

    pub fn allow_with_trailer() -> Vec<u8> {
        let mut body = store_label(*b"allo", Some(b'w'));
        body.extend(store8(5, b'S'));
        body.extend(store8(6, b'E'));
        body.extend(store8(7, b'C'));
        body.extend(store8(8, b'R'));
        body.extend(store8(9, b'E'));
        body.extend(store8(10, b'T'));
        module(&body, &i32_const(5), &[])
    }

    pub fn spin() -> Vec<u8> {
        module(&[0x03, 0x40, 0x0c, 0x00, 0x0b], &i32_const(0), &[])
    }

    pub fn oversized() -> Vec<u8> {
        module(&[], &i32_const(5_000), &[])
    }

    pub fn return_length(length: i32) -> Vec<u8> {
        module(&[], &i32_const(length), &[])
    }

    /// Write `len` copies of `byte` at offset 0 and return that length.
    pub fn repeated_label(byte: u8, len: usize) -> Vec<u8> {
        let mut body = Vec::new();
        let word = i32::from_le_bytes([byte, byte, byte, byte]);
        let mut at = 0usize;
        while at + 4 <= len {
            body.extend(i32_const(i32::try_from(at).expect("offset")));
            body.extend(i32_const(word));
            body.extend([0x36, 0x02, 0x00]);
            at += 4;
        }
        while at < len {
            body.extend(store8(i32::try_from(at).expect("offset"), byte));
            at += 1;
        }
        module(&body, &i32_const(i32::try_from(len).expect("length")), &[])
    }

    pub fn undeclared() -> Vec<u8> {
        module(&store_label(*b"nope", None), &i32_const(4), &[])
    }

    pub fn memory_grow() -> Vec<u8> {
        let mut body = i32_const(1);
        body.extend([0x40, 0x00, 0x1a]);
        module(&body, &i32_const(5), &[])
    }

    /// `allow` only when the input frame is exactly one account record.
    pub fn account_kind() -> Vec<u8> {
        let mut body = i32_const(4_096);
        body.extend([0x28, 0x02, 0x00]);
        body.extend(i32_const(1));
        body.push(0x46);
        body.extend(i32_const(4_100));
        body.extend([0x2d, 0x00, 0x00]);
        body.extend(i32_const(1));
        body.push(0x46);
        body.push(0x71);
        body.extend([0x04, 0x7f]);
        body.extend(store_label(*b"allo", Some(b'w')));
        body.extend(i32_const(5));
        body.push(0x05);
        body.extend(store_label(*b"bloc", Some(b'k')));
        body.extend(i32_const(5));
        body.push(0x0b);
        module(&body, &[], &[])
    }

    /// `block` when `marker` occurs in the first 96 input bytes, otherwise `allow`.
    pub fn hides_byte(marker: u8) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend(i32_const(0));
        body.extend([0x21, 0x00]);
        body.extend(i32_const(0));
        body.extend([0x21, 0x01]);
        body.extend([0x02, 0x40, 0x03, 0x40]);
        body.extend([0x20, 0x00]);
        body.extend(i32_const(96));
        body.push(0x4f);
        body.extend([0x0d, 0x01]);
        body.extend([0x20, 0x00]);
        body.extend(i32_const(4_096));
        body.push(0x6a);
        body.extend([0x2d, 0x00, 0x00]);
        body.extend(i32_const(i32::from(marker)));
        body.push(0x46);
        body.extend([0x04, 0x40]);
        body.extend(i32_const(1));
        body.extend([0x21, 0x01, 0x0c, 0x02, 0x0b]);
        body.extend([0x20, 0x00]);
        body.extend(i32_const(1));
        body.push(0x6a);
        body.extend([0x21, 0x00, 0x0c, 0x00, 0x0b, 0x0b]);
        body.extend([0x20, 0x01, 0x45, 0x04, 0x7f]);
        body.extend(store_label(*b"allo", Some(b'w')));
        body.extend(i32_const(5));
        body.push(0x05);
        body.extend(store_label(*b"bloc", Some(b'k')));
        body.extend(i32_const(5));
        body.push(0x0b);
        let locals = [0x01, 0x02, 0x7f];
        module_with_locals(&locals, &body)
    }

    pub fn with_import() -> Vec<u8> {
        let mut wasm = header();
        wasm.extend(section(1, &[0x01, 0x60, 0x00, 0x01, 0x7f]));
        let mut import = vec![0x01, 0x03];
        import.extend(b"env");
        import.push(0x04);
        import.extend(b"evil");
        import.extend([0x00, 0x00]);
        wasm.extend(section(2, &import));
        wasm.extend(section(3, &[0x01, 0x00]));
        wasm.extend(memory_limits(1, Some(1)));
        let mut exports = vec![0x02, 6];
        exports.extend(b"memory");
        exports.extend([0x02, 0x00, 5]);
        exports.extend(b"route");
        exports.extend([0x00, 0x01]);
        wasm.extend(section(7, &exports));
        wasm.extend(code_section(&function_body(&[0x00], &i32_const(0))));
        wasm
    }

    pub fn two_pages() -> Vec<u8> {
        module_memory(2, Some(2), &i32_const(0))
    }

    pub fn unbounded_memory() -> Vec<u8> {
        module_memory(1, None, &i32_const(0))
    }

    pub fn with_start() -> Vec<u8> {
        let mut wasm = header();
        wasm.extend(section(1, &[0x01, 0x60, 0x00, 0x01, 0x7f]));
        wasm.extend(section(3, &[0x01, 0x00]));
        wasm.extend(memory_limits(1, Some(1)));
        wasm.extend(exports_route_memory());
        wasm.extend(section(8, &[0x00]));
        wasm.extend(code_section(&function_body(&[0x00], &i32_const(0))));
        wasm
    }

    pub fn with_locals(count: u32) -> Vec<u8> {
        let mut locals = vec![0x01];
        locals.extend(leb(count));
        locals.push(0x7f);
        module_with_locals(&locals, &i32_const(0))
    }

    /// `count` live `i32.load` results, then drops, then `i32.const 0`.
    ///
    /// Wasmi 0.40 keeps each live load in its own register. `N` loads need a
    /// value-stack height of `N + 2` before `route` can be entered.
    pub fn live_loads(count: u32) -> Vec<u8> {
        let mut body = Vec::new();
        for _ in 0..count {
            body.extend(i32_const(0));
            body.extend([0x28, 0x02, 0x00]);
        }
        body.extend(std::iter::repeat_n(0x1a, count as usize));
        module(&body, &i32_const(0), &[])
    }

    pub fn two_functions() -> Vec<u8> {
        let mut wasm = header();
        wasm.extend(section(1, &[0x01, 0x60, 0x00, 0x01, 0x7f]));
        wasm.extend(section(3, &[0x02, 0x00, 0x00]));
        wasm.extend(memory_limits(1, Some(1)));
        wasm.extend(exports_route_memory());
        let body = function_body(&[0x00], &i32_const(0));
        let mut payload = vec![0x02];
        payload.extend(leb(u32::try_from(body.len()).unwrap()));
        payload.extend(&body);
        payload.extend(leb(u32::try_from(body.len()).unwrap()));
        payload.extend(&body);
        wasm.extend(section(10, &payload));
        wasm
    }

    pub fn with_data_segment() -> Vec<u8> {
        let mut wasm = allow();
        wasm.extend(section(11, &[0x01]));
        wasm
    }

    pub fn with_sentinel(mut module: Vec<u8>) -> Vec<u8> {
        let mut payload = vec![0x08];
        payload.extend(b"sentinel");
        payload.extend(b"MODULE-BYTES-SENTINEL");
        module.extend(section(0, &payload));
        module
    }

    pub fn document(module: &[u8], mutate: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
        use base64::Engine;
        use base64::engine::general_purpose::STANDARD;
        use sha2::{Digest, Sha256};
        let digest: [u8; 32] = Sha256::digest(module).into();
        let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        let mut value = serde_json::json!({
            "format": super::FORMAT,
            "stage": "risk-check",
            "outputs": ["allow", "block"],
            "permissions": ["read_profile"],
            "fuel": 10_000,
            "memory_bytes": super::MAX_MEMORY_BYTES,
            "max_input_bytes": 256,
            "max_output_bytes": 128,
            "timeout_seconds": 30,
            "network": "deny",
            "module_sha256": hex,
            "module_base64": STANDARD.encode(module),
        });
        mutate(&mut value);
        serde_json::to_vec(&value).unwrap()
    }

    fn module(prepare: &[u8], result: &[u8], extra: &[u8]) -> Vec<u8> {
        let mut body = prepare.to_vec();
        body.extend(result);
        module_with_locals(&[0x00], &body)
            .into_iter()
            .chain(extra.iter().copied())
            .collect()
    }

    fn module_with_locals(locals: &[u8], instructions: &[u8]) -> Vec<u8> {
        module_memory(1, Some(1), &function_body(locals, instructions))
    }

    fn module_memory(min: u32, max: Option<u32>, body: &[u8]) -> Vec<u8> {
        let mut wasm = header();
        wasm.extend(section(1, &[0x01, 0x60, 0x00, 0x01, 0x7f]));
        wasm.extend(section(3, &[0x01, 0x00]));
        wasm.extend(memory_limits(min, max));
        wasm.extend(exports_route_memory());
        wasm.extend(code_section(body));
        wasm
    }

    fn header() -> Vec<u8> {
        vec![0x00, b'a', b's', b'm', 0x01, 0x00, 0x00, 0x00]
    }

    fn exports_route_memory() -> Vec<u8> {
        let mut payload = vec![0x02, 6];
        payload.extend(b"memory");
        payload.extend([0x02, 0x00, 5]);
        payload.extend(b"route");
        payload.extend([0x00, 0x00]);
        section(7, &payload)
    }

    fn memory_limits(min: u32, max: Option<u32>) -> Vec<u8> {
        let mut payload = vec![0x01];
        match max {
            Some(max) => {
                payload.push(0x01);
                payload.extend(leb(min));
                payload.extend(leb(max));
            }
            None => {
                payload.push(0x00);
                payload.extend(leb(min));
            }
        }
        section(5, &payload)
    }

    fn code_section(body: &[u8]) -> Vec<u8> {
        let mut payload = vec![0x01];
        payload.extend(leb(body.len() as u32));
        payload.extend(body);
        section(10, &payload)
    }

    fn function_body(locals: &[u8], instructions: &[u8]) -> Vec<u8> {
        let mut body = locals.to_vec();
        body.extend(instructions);
        body.push(0x0b);
        body
    }

    fn section(id: u8, payload: &[u8]) -> Vec<u8> {
        let mut out = vec![id];
        out.extend(leb(payload.len() as u32));
        out.extend(payload);
        out
    }

    fn leb(mut value: u32) -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                break;
            }
        }
        out
    }

    fn sleb(mut value: i32) -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let mut byte = (value as u8) & 0x7f;
            value >>= 7;
            let done = (value == 0 && byte & 0x40 == 0) || (value == -1 && byte & 0x40 != 0);
            if !done {
                byte |= 0x80;
            }
            out.push(byte);
            if done {
                break;
            }
        }
        out
    }

    fn i32_const(value: i32) -> Vec<u8> {
        let mut out = vec![0x41];
        out.extend(sleb(value));
        out
    }

    fn store_label(word: [u8; 4], fifth: Option<u8>) -> Vec<u8> {
        let mut out = i32_const(0);
        out.extend(i32_const(i32::from_le_bytes(word)));
        out.extend([0x36, 0x02, 0x00]);
        if let Some(byte) = fifth {
            out.extend(store8(4, byte));
        }
        out
    }

    fn store8(offset: i32, byte: u8) -> Vec<u8> {
        let mut out = i32_const(offset);
        out.extend(i32_const(i32::from(byte)));
        out.extend([0x3a, 0x00, 0x00]);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[cfg(all(feature = "platform", target_os = "macos"))]
    fn native_probe(root: &std::path::Path) -> std::path::PathBuf {
        let source = root.join("probe.c");
        let binary = root.join("probe");
        std::fs::write(&source, include_bytes!("extension_gate/native-probe.c")).unwrap();
        let compiled = std::process::Command::new("cc")
            .arg("-Wall")
            .arg("-Wextra")
            .arg("-Werror")
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        binary
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn kernel_isolation_denies_escaped_io_and_child_creation() {
        let installed = disposable_server();
        let probe = native_probe(&installed.root);
        let result = isolation::command(&probe)
            .unwrap()
            .env_clear()
            .current_dir(&installed.root)
            .output()
            .unwrap();
        assert!(result.status.success(), "{:?}", result);
        assert_eq!(result.stdout, b"read/write/network/fork/spawn denied\n");
        assert!(!installed.root.join("write-sentinel").exists());
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn inherited_descriptors_and_unsandboxed_entry_refuse_before_ipc() {
        use std::io::Write;
        let installed = disposable_server();
        let probe = native_probe(&installed.root);
        let confined = isolation::command(&installed.server).unwrap();
        let checked = check(&fixture::document(&fixture::allow(), |_| {})).unwrap();
        let request = encode_request(MODE_RUN, false, 10_000, &checked, 32, &[]).unwrap();
        for mode in ["inherit", "inherit-socket", "inherit-kqueue"] {
            if mode == "inherit-kqueue" {
                // Darwin closes kqueues across exec even after dup2 clears
                // FD_CLOEXEC. Prove that with fcntl, rather than attributing
                // the closure to the guest's /dev/fd audit.
                let confined_probe = isolation::command(&probe).unwrap();
                let closed = std::process::Command::new(&probe)
                    .arg(mode)
                    .arg(confined_probe.get_program())
                    .args(confined_probe.get_args())
                    .arg("fd-closed")
                    .env_clear()
                    .current_dir(&installed.root)
                    .output()
                    .unwrap();
                assert!(closed.status.success(), "{closed:?}");
                assert_eq!(closed.stdout, b"descriptor 100 closed\n");
            }
            let mut child = std::process::Command::new(&probe)
                .arg(mode)
                .arg(confined.get_program())
                .args(confined.get_args())
                .arg(GUEST_ARGV)
                .env_clear()
                .current_dir(&installed.root)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            let mut stdin = child.stdin.take().unwrap();
            // The same request returns allow without an inherited descriptor.
            // BrokenPipe is expected if entry refuses before the write arrives.
            let _ = stdin.write_all(&request);
            drop(stdin);
            let result = child.wait_with_output().unwrap();
            assert!(result.status.success(), "{mode}: {result:?}");
            if mode == "inherit-kqueue" {
                assert_eq!(result.stdout, b"\x00\x05allow", "{mode}");
            } else {
                assert_eq!(
                    result.stdout,
                    [STATUS_DENIAL, Denial::Failed.code()],
                    "{mode}"
                );
            }
        }
        let direct = std::process::Command::new(&installed.server)
            .arg(GUEST_ARGV)
            .env_clear()
            .current_dir(&installed.root)
            .output()
            .unwrap();
        assert!(direct.status.success());
        assert_eq!(direct.stdout, [STATUS_DENIAL, Denial::Failed.code()]);
    }

    #[cfg(all(feature = "platform", not(target_os = "macos")))]
    #[test]
    fn unsupported_hosts_refuse_validation_and_execution_without_spawning() {
        let document = fixture::document(&fixture::allow(), |_| {});
        assert_eq!(
            check(&document).unwrap_err(),
            Denial::ExternalRuntimeRequired
        );
        let bound = bind(&document).unwrap();
        assert_eq!(
            execute(&bound, &GuestFacts::default(), &BTreeSet::new(), bounds()),
            Err(Denial::ExternalRuntimeRequired)
        );
        assert!(matches!(
            isolation::command(std::path::Path::new("/missing")),
            Err(Denial::ExternalRuntimeRequired)
        ));
        // Binding/structural admission remain portable and do not compile.
        assert!(
            bind(&fixture::document(
                &fixture::allow_with_invalid_tail(1),
                |_| {}
            ))
            .is_ok()
        );
        assert_eq!(
            bind(&fixture::document(
                &fixture::allow_with_unreachable_tail(12_000),
                |_| {}
            ))
            .unwrap_err(),
            Denial::Fuel
        );
    }

    fn bounds() -> StepBounds {
        StepBounds {
            timeout_seconds: 30,
            max_output_bytes: 128,
        }
    }

    #[cfg(any(not(feature = "platform"), target_os = "macos"))]
    fn profile() -> BTreeSet<StagePermission> {
        BTreeSet::from([StagePermission::ReadProfile])
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    fn run(
        module: &[u8],
        mutate: impl FnOnce(&mut serde_json::Value),
        facts: GuestFacts,
        requested: &BTreeSet<StagePermission>,
        step: StepBounds,
    ) -> Result<Label, Denial> {
        let checked = check(&fixture::document(module, mutate)).unwrap();
        execute(&checked, &facts, requested, step)
    }

    #[test]
    fn limits_match_the_held_contract() {
        assert_eq!(RUNTIME_LINKED, cfg!(feature = "platform"));
        assert_eq!(MAX_INPUT_BYTES, crate::workflow::extension::MAX_INPUT_BYTES);
        assert_eq!(MAX_TIMEOUT_SECONDS, 30);
        assert_eq!(MAX_OUTPUT_BYTES, 4_096);
        assert_eq!(MAX_FUEL, 10_000);
        assert_eq!(MAX_MEMORY_BYTES, 65_536);
        assert_eq!(FUEL_PER_SECOND, 1_000);
        assert_eq!(MAX_GUEST_FUNCTIONS, 1);
        assert_eq!(MAX_GUEST_LOCALS, 32);
        assert_eq!(GUEST_VALUE_STACK_SLOTS, 64);
        assert_eq!(GUEST_CALL_DEPTH, 16);
        assert_eq!(GUEST_TRANSLATION_FUEL_PER_BYTE, 7);
        assert_eq!(MAX_ROUTE_LABEL_BYTES, 32);
        assert!(Label::new("a".repeat(32)).is_ok());
        assert!(Label::new("a".repeat(33)).is_err());
        let cargo = include_str!("../../Cargo.toml");
        assert!(cargo.contains("unsafe_code = \"forbid\""));
        assert!(cargo.contains("dep:wasmi"));
        assert!(cargo.contains("essentials = []"));
        for name in ["wasmtime", "wasmer", "wasm3", "rhai", "mlua", "deno_core"] {
            assert!(!cargo.contains(name), "{name}");
        }
        let executor = include_str!("executor.rs");
        assert!(executor.contains("extension_gate"));
        assert!(!executor.contains("Host::invoke"));
    }

    #[test]
    fn bounds_permissions_and_integrity_deny_before_execution() {
        assert_eq!(check(b"").unwrap_err(), Denial::Malformed);
        assert_eq!(
            check(&vec![b' '; MAX_MANIFEST_BYTES + 1]).unwrap_err(),
            Denial::Limit
        );
        assert_eq!(check(b"{}").unwrap_err(), Denial::Malformed);
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["format"] = serde_json::json!("other");
            }))
            .unwrap_err(),
            Denial::Malformed
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["fuel"] = serde_json::json!(0);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["fuel"] = serde_json::json!(MAX_FUEL + 1);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["memory_bytes"] = serde_json::json!(0);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["memory_bytes"] = serde_json::json!(4_096);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["memory_bytes"] = serde_json::json!(MAX_MEMORY_BYTES + 1);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["timeout_seconds"] = serde_json::json!(31);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["max_output_bytes"] = serde_json::json!(0);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["max_input_bytes"] = serde_json::json!(MAX_INPUT_BYTES + 1);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["outputs"] = serde_json::json!(["verified"]);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["outputs"] = serde_json::json!(["allow", "allow"]);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["permissions"] = serde_json::json!(["network"]);
            }))
            .unwrap_err(),
            Denial::Permission
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["permissions"] = serde_json::json!(["read_profile", "read_profile"]);
            }))
            .unwrap_err(),
            Denial::Permission
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["network"] = serde_json::json!("allow");
            }))
            .unwrap_err(),
            Denial::Permission
        );
        assert_eq!(
            check(&fixture::document(&[0u8; 64], |value| {
                value["memory_bytes"] = serde_json::json!(32);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                let hash = value["module_sha256"]
                    .as_str()
                    .unwrap()
                    .to_ascii_uppercase();
                value["module_sha256"] = serde_json::json!(hash);
            }))
            .unwrap_err(),
            Denial::Malformed
        );
        assert_eq!(
            check(&fixture::document(&fixture::allow(), |value| {
                value["module_sha256"] = serde_json::json!("ab".repeat(32));
            }))
            .unwrap_err(),
            Denial::Integrity
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn a_bounded_guest_returns_one_declared_label_and_hides_module_bytes() {
        let module = fixture::with_sentinel(fixture::allow_with_trailer());
        let checked = check(&fixture::document(&module, |_| {})).unwrap();
        assert_eq!(checked.stage().as_str(), "risk-check");
        assert!(!checked.permissions().contains(&StagePermission::Network));
        assert!(!format!("{checked:?}").contains("MODULE-BYTES-SENTINEL"));
        assert!(!format!("{checked:?}").contains("allow_with"));
        let label = execute(&checked, &GuestFacts::default(), &BTreeSet::new(), bounds()).unwrap();
        assert_eq!(label.as_str(), "allow");
        assert!(!format!("{label:?}").contains("SECRET"));
        let other = check(&fixture::document(&fixture::block(), |_| {})).unwrap();
        assert_ne!(checked.module_sha256(), other.module_sha256());
        assert_eq!(
            execute(&other, &GuestFacts::default(), &BTreeSet::new(), bounds())
                .unwrap()
                .as_str(),
            "block"
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn attacks_and_budgets_deny_without_a_label() {
        assert_eq!(
            check(&fixture::document(&fixture::with_import(), |_| {})).unwrap_err(),
            Denial::Permission
        );
        assert_eq!(
            check(&fixture::document(&fixture::two_pages(), |_| {})).unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::unbounded_memory(), |_| {})).unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::with_start(), |_| {})).unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::two_functions(), |_| {})).unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(&fixture::with_locals(1_000_000), |_| {})).unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&fixture::document(
                &fixture::with_locals(MAX_GUEST_LOCALS + 1),
                |_| {}
            ))
            .unwrap_err(),
            Denial::Limit
        );
        assert!(
            check(&fixture::document(
                &fixture::with_locals(MAX_GUEST_LOCALS),
                |_| {}
            ))
            .is_ok()
        );
        assert_eq!(
            check(&fixture::document(&fixture::with_data_segment(), |_| {})).unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            run(
                &fixture::memory_grow(),
                |_| {},
                GuestFacts::default(),
                &BTreeSet::new(),
                bounds()
            )
            .unwrap_err(),
            Denial::Failed
        );
        assert_eq!(
            run(
                &fixture::spin(),
                |value| value["fuel"] = serde_json::json!(1_000),
                GuestFacts::default(),
                &BTreeSet::new(),
                bounds()
            )
            .unwrap_err(),
            Denial::Fuel
        );
        assert_eq!(
            run(
                &fixture::spin(),
                |value| value["fuel"] = serde_json::json!(5_000),
                GuestFacts::default(),
                &BTreeSet::new(),
                StepBounds {
                    timeout_seconds: 1,
                    max_output_bytes: 128,
                }
            )
            .unwrap_err(),
            Denial::Timeout
        );
        assert_eq!(
            run(
                &fixture::spin(),
                |value| value["fuel"] = serde_json::json!(1_000),
                GuestFacts::default(),
                &BTreeSet::new(),
                StepBounds {
                    timeout_seconds: 1,
                    max_output_bytes: 128,
                }
            )
            .unwrap_err(),
            Denial::Fuel
        );
        assert_eq!(
            run(
                &fixture::oversized(),
                |_| {},
                GuestFacts::default(),
                &BTreeSet::new(),
                bounds()
            )
            .unwrap_err(),
            Denial::Output
        );
        assert_eq!(
            run(
                &fixture::allow(),
                |_| {},
                GuestFacts::default(),
                &BTreeSet::new(),
                StepBounds {
                    timeout_seconds: 30,
                    max_output_bytes: 4,
                }
            )
            .unwrap_err(),
            Denial::Output
        );
        assert_eq!(
            run(
                &fixture::undeclared(),
                |_| {},
                GuestFacts::default(),
                &BTreeSet::new(),
                bounds()
            )
            .unwrap_err(),
            Denial::UndeclaredOutput
        );
        let loose = StepBounds {
            timeout_seconds: 31,
            max_output_bytes: 128,
        };
        let checked = check(&fixture::document(&fixture::allow(), |_| {})).unwrap();
        assert_eq!(
            execute(&checked, &GuestFacts::default(), &BTreeSet::new(), loose).unwrap_err(),
            Denial::Limit
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn value_stack_refuses_a_frame_that_would_reach_the_slot_cap() {
        // Pinned Wasmi 0.40.0: N live loads need height N + 2. Height 64 runs
        // 62 loads and refuses 63 before `Vec::reserve`.
        let over = fixture::live_loads(63);
        assert!(check(&fixture::document(&over, |_| {})).is_ok());
        assert_eq!(
            run(
                &over,
                |_| {},
                GuestFacts::default(),
                &BTreeSet::new(),
                bounds()
            )
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            run(
                &fixture::live_loads(62),
                |_| {},
                GuestFacts::default(),
                &BTreeSet::new(),
                bounds()
            )
            .unwrap_err(),
            Denial::UndeclaredOutput
        );
        assert_eq!(
            run(
                &fixture::with_locals(MAX_GUEST_LOCALS),
                |_| {},
                GuestFacts::default(),
                &BTreeSet::new(),
                bounds()
            )
            .unwrap_err(),
            Denial::UndeclaredOutput
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    fn translation_charge(module: &[u8]) -> u64 {
        let body = admit_sections(module).expect("fixture has one function body");
        u64::from(body).saturating_mul(GUEST_TRANSLATION_FUEL_PER_BYTE)
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn admission_refuses_a_large_unreachable_body_before_validation() {
        // Twelve thousand unreachable bytes exceed the 10,000 fuel cap even at
        // one fuel per byte, so admission must refuse them before `Module::new`.
        // An illegal opcode in that tail would be `malformed` if the validator
        // walked it. A short unreachable tail still returns `allow`.
        let bare = fixture::allow_with_unreachable_tail(0);
        assert_eq!(
            run(
                &bare,
                |_| {},
                GuestFacts::default(),
                &BTreeSet::new(),
                bounds()
            )
            .unwrap()
            .as_str(),
            "allow"
        );
        let fitting = fixture::allow_with_unreachable_tail(32);
        assert!(translation_charge(&fitting) <= u64::from(MAX_FUEL));
        assert_eq!(
            run(
                &fitting,
                |_| {},
                GuestFacts::default(),
                &BTreeSet::new(),
                bounds()
            )
            .unwrap()
            .as_str(),
            "allow"
        );
        let unreachable = fixture::allow_with_unreachable_tail(12_000);
        assert!(unreachable.len() <= usize::try_from(MAX_MODULE_BYTES).unwrap());
        assert!(translation_charge(&unreachable) > u64::from(MAX_FUEL));
        assert_eq!(
            check(&fixture::document(&unreachable, |_| {})).unwrap_err(),
            Denial::Fuel
        );
        let invalid = fixture::allow_with_invalid_tail(12_000);
        assert!(translation_charge(&invalid) > u64::from(MAX_FUEL));
        assert_eq!(
            check(&fixture::document(&invalid, |_| {})).unwrap_err(),
            Denial::Fuel
        );
        let invalid_small = fixture::allow_with_invalid_tail(1);
        assert!(translation_charge(&invalid_small) <= u64::from(MAX_FUEL));
        assert_eq!(
            check(&fixture::document(&invalid_small, |_| {})).unwrap_err(),
            Denial::Malformed
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn low_budgets_refuse_translation_and_a_repeat_call_pays_again() {
        let bare = fixture::allow_with_unreachable_tail(0);
        let charge = translation_charge(&bare);
        let charge = u32::try_from(charge).unwrap();
        assert!(charge > 1 && charge < MAX_FUEL);
        assert_eq!(
            check(&fixture::document(&bare, |value| {
                value["fuel"] = serde_json::json!(charge - 1);
            }))
            .unwrap_err(),
            Denial::Fuel
        );
        let exact = check(&fixture::document(&bare, |value| {
            value["fuel"] = serde_json::json!(charge);
        }))
        .unwrap();
        assert_eq!(
            execute(&exact, &GuestFacts::default(), &BTreeSet::new(), bounds()).unwrap_err(),
            Denial::Fuel
        );
        // Each execution is a new process. A second call parses again and still
        // cannot pay both translation and `route`.
        assert_eq!(
            execute(&exact, &GuestFacts::default(), &BTreeSet::new(), bounds()).unwrap_err(),
            Denial::Fuel
        );
        let wide = fixture::allow_with_unreachable_tail(200);
        let wide_charge = translation_charge(&wide);
        assert!(wide_charge > 1_000 && wide_charge <= u64::from(MAX_FUEL));
        assert_eq!(
            check(&fixture::document(&wide, |value| {
                value["timeout_seconds"] = serde_json::json!(1);
            }))
            .unwrap_err(),
            Denial::Timeout
        );
        let checked = check(&fixture::document(&wide, |_| {})).unwrap();
        assert_eq!(
            execute(
                &checked,
                &GuestFacts::default(),
                &BTreeSet::new(),
                StepBounds {
                    timeout_seconds: 1,
                    max_output_bytes: 128,
                }
            )
            .unwrap_err(),
            Denial::Timeout
        );
        assert_eq!(
            execute(&checked, &GuestFacts::default(), &BTreeSet::new(), bounds())
                .unwrap()
                .as_str(),
            "allow"
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn the_host_reads_at_most_one_label_from_the_guest() {
        let wide = StepBounds {
            timeout_seconds: 30,
            max_output_bytes: MAX_OUTPUT_BYTES,
        };
        let widen = |value: &mut serde_json::Value| {
            value["max_output_bytes"] = serde_json::json!(MAX_OUTPUT_BYTES);
        };
        assert_eq!(
            run(
                &fixture::return_length(i32::try_from(MAX_OUTPUT_BYTES).unwrap()),
                widen,
                GuestFacts::default(),
                &BTreeSet::new(),
                wide
            )
            .unwrap_err(),
            Denial::Output
        );
        assert_eq!(
            run(
                &fixture::return_length(i32::try_from(MAX_ROUTE_LABEL_BYTES).unwrap() + 1),
                widen,
                GuestFacts::default(),
                &BTreeSet::new(),
                wide
            )
            .unwrap_err(),
            Denial::Output
        );
        assert_eq!(
            run(
                &fixture::return_length(i32::try_from(MAX_ROUTE_LABEL_BYTES).unwrap()),
                widen,
                GuestFacts::default(),
                &BTreeSet::new(),
                wide
            )
            .unwrap_err(),
            Denial::UndeclaredOutput
        );
        assert_eq!(
            run(
                &fixture::return_length(4),
                widen,
                GuestFacts::default(),
                &BTreeSet::new(),
                wide
            )
            .unwrap_err(),
            Denial::UndeclaredOutput
        );
        let label = "a".repeat(usize::try_from(MAX_ROUTE_LABEL_BYTES).unwrap());
        assert_eq!(
            run(
                &fixture::repeated_label(b'a', usize::try_from(MAX_ROUTE_LABEL_BYTES).unwrap()),
                |value| {
                    widen(value);
                    value["outputs"] = serde_json::json!([label.clone()]);
                },
                GuestFacts::default(),
                &BTreeSet::new(),
                wide
            )
            .unwrap()
            .as_str(),
            label
        );
        assert_eq!(
            run(
                &fixture::repeated_label(b'a', usize::try_from(MAX_ROUTE_LABEL_BYTES).unwrap()),
                |value| {
                    value["max_output_bytes"] = serde_json::json!(MAX_ROUTE_LABEL_BYTES - 1);
                    value["outputs"] = serde_json::json!([label]);
                },
                GuestFacts::default(),
                &BTreeSet::new(),
                StepBounds {
                    timeout_seconds: 30,
                    max_output_bytes: MAX_ROUTE_LABEL_BYTES - 1,
                }
            )
            .unwrap_err(),
            Denial::Output
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn the_guest_sees_only_the_granted_identifiers() {
        let facts = GuestFacts {
            account: Some("zzz".into()),
            request: Some("request-1".into()),
            client: Some("session-1".into()),
            groups: vec!["staff".into()],
        };
        let hidden = hides(&fixture::hides_byte(b'z'), &facts, &profile());
        assert_eq!(hidden.as_str(), "block");
        let request_only = BTreeSet::from([StagePermission::ReadRequest]);
        let visible = hides(&fixture::hides_byte(b'z'), &facts, &request_only);
        assert_eq!(
            visible.as_str(),
            "allow",
            "withheld account reached the guest"
        );
        let kind = run(
            &fixture::account_kind(),
            |_| {},
            GuestFacts {
                account: Some("user-1".into()),
                ..GuestFacts::default()
            },
            &profile(),
            bounds(),
        )
        .unwrap();
        assert_eq!(kind.as_str(), "allow");
        let withheld = run(
            &fixture::account_kind(),
            |_| {},
            GuestFacts {
                account: Some("user-1".into()),
                ..GuestFacts::default()
            },
            &BTreeSet::new(),
            bounds(),
        )
        .unwrap();
        assert_eq!(withheld.as_str(), "block");
        assert_eq!(
            run(
                &fixture::account_kind(),
                |_| {},
                GuestFacts::default(),
                &profile(),
                bounds()
            )
            .unwrap_err(),
            Denial::InputLimit
        );
        assert_eq!(
            run(
                &fixture::allow(),
                |_| {},
                GuestFacts {
                    account: Some("PasswordValue".into()),
                    ..GuestFacts::default()
                },
                &profile(),
                bounds()
            )
            .unwrap_err(),
            Denial::InputLimit
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    fn hides(module: &[u8], facts: &GuestFacts, requested: &BTreeSet<StagePermission>) -> Label {
        run(
            module,
            |value| {
                value["permissions"] =
                    serde_json::json!(["read_profile", "read_request", "read_groups"]);
                value["max_input_bytes"] = serde_json::json!(512);
            },
            facts.clone(),
            requested,
            bounds(),
        )
        .unwrap_or_else(|error| panic!("guest failed: {}", error.as_str()))
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn a_nonreturning_helper_is_killed_reaped_and_is_not_fuel() {
        let checked = check(&fixture::document(&fixture::allow(), |_| {})).unwrap();
        let request = encode_request(MODE_HOLD, false, 10_000, &checked, 32, &[]).unwrap();
        let program = resolve_guest_program().unwrap();
        let started = std::time::Instant::now();
        let supervised = supervise(&program, request, std::time::Duration::from_secs(1)).unwrap();
        let elapsed = started.elapsed();
        assert_eq!(supervised.outcome, Err(Denial::Elapsed));
        assert!(
            elapsed >= std::time::Duration::from_millis(900),
            "{elapsed:?}"
        );
        assert!(elapsed < std::time::Duration::from_secs(8), "{elapsed:?}");
        assert!(
            !process_alive(supervised.pid),
            "guest pid {} is still alive",
            supervised.pid
        );
        assert_eq!(Denial::Elapsed.as_str(), "elapsed");
        assert_ne!(Denial::Fuel.as_str(), "elapsed");
        assert_ne!(Denial::Timeout.as_str(), "elapsed");
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn a_flooding_helper_is_capped_and_reaped_without_a_label() {
        let checked = check(&fixture::document(&fixture::allow(), |_| {})).unwrap();
        let request = encode_request(MODE_FLOOD, false, 10_000, &checked, 32, &[]).unwrap();
        let program = resolve_guest_program().unwrap();
        let supervised = supervise(&program, request, std::time::Duration::from_secs(30)).unwrap();
        assert_eq!(supervised.outcome, Err(Denial::Output));
        assert!(
            !process_alive(supervised.pid),
            "guest pid {} is still alive",
            supervised.pid
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn a_missing_helper_denies_without_a_label() {
        let checked = check(&fixture::document(&fixture::allow(), |_| {})).unwrap();
        let request = encode_request(MODE_RUN, false, 10_000, &checked, 32, &[]).unwrap();
        let error = supervise(
            std::path::Path::new("/nonexistent/riauth-extension-guest"),
            request,
            std::time::Duration::from_secs(1),
        );
        assert_eq!(error.err(), Some(Denial::Failed));
    }

    #[cfg(feature = "platform")]
    #[test]
    fn an_installed_or_renamed_server_reexecs_that_exact_path() {
        let installed = std::path::Path::new("/usr/local/bin/riauth");
        assert_eq!(guest_program(installed), installed);
        let renamed = std::path::Path::new("/opt/riauth/images/identity");
        assert_eq!(guest_program(renamed), renamed);
        if cfg!(target_os = "macos") {
            let program = resolve_guest_program().unwrap();
            assert_eq!(
                program.file_name().and_then(|name| name.to_str()),
                Some("riauth")
            );
            assert!(program.is_file(), "{program:?}");
        } else {
            assert_eq!(
                resolve_guest_program(),
                Err(Denial::ExternalRuntimeRequired)
            );
        }
        let cli = include_str!("../../src/cli.rs");
        assert!(!cli.contains("extension-guest"));
        let maintenance = include_str!("../../src/bin/riauth-maintenance.rs");
        assert!(!maintenance.contains("extension_gate"));
        const {
            assert!(MAX_GUEST_REQUEST_BYTES <= 80 * 1024);
            assert!(MAX_RESPONSE_BYTES <= 64);
        }
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    struct DisposableCopy {
        root: std::path::PathBuf,
        server: std::path::PathBuf,
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    impl Drop for DisposableCopy {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    /// Copy of the built server under a temp name, outside the Cargo layout.
    #[cfg(all(feature = "platform", target_os = "macos"))]
    fn disposable_server() -> DisposableCopy {
        let source = resolve_guest_program().unwrap();
        assert!(source.is_file(), "{source:?}");
        assert_eq!(
            source.file_name().and_then(|name| name.to_str()),
            Some("riauth")
        );
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("riauth-installed-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let server = root.join("identity-server");
        std::fs::copy(&source, &server).unwrap();
        let mode = std::fs::metadata(&source).unwrap().permissions();
        std::fs::set_permissions(&server, mode).unwrap();
        assert_ne!(server, source);
        assert_eq!(guest_program(&server), server.as_path());
        assert_eq!(
            server.file_name().and_then(|name| name.to_str()),
            Some("identity-server")
        );
        assert!(server.components().all(|component| {
            let text = component.as_os_str().to_string_lossy();
            text != "deps" && text != "target" && text != "debug"
        }));
        let rendered = server.to_string_lossy();
        assert!(!rendered.contains("/deps/"), "{rendered}");
        assert!(!rendered.contains("/target"), "{rendered}");
        DisposableCopy { root, server }
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    fn assert_deadline_discarded(
        supervised: &Supervised,
        elapsed: std::time::Duration,
        lower: std::time::Duration,
    ) {
        assert_eq!(supervised.outcome.as_ref(), Err(&Denial::Elapsed));
        assert!(elapsed >= lower, "{elapsed:?} lower {lower:?}");
        // Hang detector. Startup and reap are inside the interval; this is not
        // a kernel scheduling guarantee.
        assert!(elapsed < std::time::Duration::from_secs(30), "{elapsed:?}");
        assert!(
            !process_alive(supervised.pid),
            "guest pid {} is still alive",
            supervised.pid
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn a_completed_guest_observed_after_the_deadline_is_not_a_label() {
        let installed = disposable_server();
        let document = fixture::document(&fixture::allow(), |_| {});
        let checked = check(&document).unwrap();
        let request = encode_request(MODE_RUN, false, 10_000, &checked, 32, &[]).unwrap();
        let started = std::time::Instant::now();
        let timely = supervise(
            &installed.server,
            request.clone(),
            std::time::Duration::from_secs(30),
        )
        .unwrap();
        let timely_elapsed = started.elapsed();
        let label = timely.outcome.expect("in-time allow");
        assert_eq!(label.as_str(), "allow");
        assert!(label.as_str().len() <= 32);
        assert!(
            timely_elapsed < std::time::Duration::from_secs(15),
            "in-time allow took {timely_elapsed:?}"
        );
        // The deadline is past this completion. The parent then waits longer,
        // so it observes the finished child only after the deadline. The slack
        // is process startup, not a kernel schedule.
        let budget = timely_elapsed + std::time::Duration::from_secs(3);
        let delay = budget + std::time::Duration::from_millis(500);

        let late_poll = PollTiming {
            before_poll: delay,
            after_join: std::time::Duration::ZERO,
        };
        let started = std::time::Instant::now();
        let delayed =
            supervise_timed(&installed.server, request.clone(), budget, late_poll).unwrap();
        assert_deadline_discarded(&delayed, started.elapsed(), delay);

        let late_join = PollTiming {
            before_poll: std::time::Duration::ZERO,
            after_join: delay,
        };
        let started = std::time::Instant::now();
        let joined = supervise_timed(&installed.server, request, budget, late_join).unwrap();
        assert_deadline_discarded(&joined, started.elapsed(), delay);

        assert!(
            validate_with(
                &installed.server,
                &checked,
                std::time::Duration::from_secs(30),
                PollTiming::IMMEDIATE,
            )
            .is_ok()
        );
        let started = std::time::Instant::now();
        let late_validation = validate_with(&installed.server, &checked, budget, late_poll);
        assert_eq!(late_validation, Err(Denial::Elapsed));
        let elapsed = started.elapsed();
        assert!(elapsed >= delay, "{elapsed:?}");
        assert!(elapsed < std::time::Duration::from_secs(30), "{elapsed:?}");
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    fn assert_environment_refusal(
        program: &std::path::Path,
        root: &std::path::Path,
        key: &str,
        value: &str,
    ) {
        let slug: String = key
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect();
        let workdir = root.join(format!("env-{slug}"));
        let mut builder = std::fs::DirBuilder::new();
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&workdir).unwrap();
        let mut child = std::process::Command::new(program)
            .arg(GUEST_ARGV)
            .env_clear()
            .env(key, value)
            .current_dir(&workdir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let reader = std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0u8; MAX_RESPONSE_BYTES + 1];
            let mut len = 0usize;
            while len < buf.len() {
                match stdout.read(&mut buf[len..]) {
                    Ok(0) => break,
                    Ok(count) => len += count,
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
            (buf, len)
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while std::time::Instant::now() < deadline {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
                Err(_) => break,
            }
        }
        if child.try_wait().ok().flatten().is_none() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let (buf, len) = reader.join().expect("sentinel reader");
        assert!(
            (2..=MAX_RESPONSE_BYTES).contains(&len),
            "response len {len}"
        );
        assert_ne!(buf[0], STATUS_LABEL);
        assert_eq!(buf[0], STATUS_DENIAL);
        assert_eq!(buf[1], Denial::Failed.code());
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn a_copied_renamed_server_serves_the_guest_with_an_empty_environment() {
        let installed = disposable_server();
        let checked = check(&fixture::document(&fixture::allow(), |_| {})).unwrap();
        let request = encode_request(MODE_RUN, false, 10_000, &checked, 32, &[]).unwrap();
        let allowed = supervise(
            &installed.server,
            request.clone(),
            std::time::Duration::from_secs(30),
        )
        .unwrap();
        let label = allowed.outcome.expect("empty environment still runs");
        assert_eq!(label.as_str(), "allow");
        assert!(label.as_str().len() <= 32);

        let spin = check(&fixture::document(&fixture::spin(), |value| {
            value["fuel"] = serde_json::json!(1_000);
        }))
        .unwrap();
        let spinning = encode_request(MODE_RUN, false, 1_000, &spin, 32, &[]).unwrap();
        let fueled = supervise(
            &installed.server,
            spinning,
            std::time::Duration::from_secs(30),
        )
        .unwrap();
        assert_eq!(fueled.outcome, Err(Denial::Fuel));

        let holding = encode_request(MODE_HOLD, false, 10_000, &checked, 32, &[]).unwrap();
        let started = std::time::Instant::now();
        let held = supervise(
            &installed.server,
            holding,
            std::time::Duration::from_secs(1),
        )
        .unwrap();
        assert_deadline_discarded(
            &held,
            started.elapsed(),
            std::time::Duration::from_millis(900),
        );

        let flooding = encode_request(MODE_FLOOD, false, 10_000, &checked, 32, &[]).unwrap();
        let flooded = supervise(
            &installed.server,
            flooding,
            std::time::Duration::from_secs(30),
        )
        .unwrap();
        assert_eq!(flooded.outcome, Err(Denial::Output));
        assert!(
            !process_alive(flooded.pid),
            "guest pid {} is still alive",
            flooded.pid
        );

        let missing = installed.root.join("missing-server");
        assert!(!missing.exists());
        assert_eq!(
            supervise(&missing, request.clone(), std::time::Duration::from_secs(1)).err(),
            Some(Denial::Failed)
        );

        let corrupt = installed.root.join("corrupt-server");
        std::fs::write(&corrupt, b"\x7fELF not a real server").unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&corrupt, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        // macOS can create the process and have it exit before main. Either
        // shape is `failed` and neither is a label.
        match supervise(&corrupt, request, std::time::Duration::from_secs(1)) {
            Err(denial) => assert_eq!(denial, Denial::Failed),
            Ok(supervised) => {
                assert_eq!(supervised.outcome, Err(Denial::Failed));
                assert!(
                    !process_alive(supervised.pid),
                    "guest pid {} is still alive",
                    supervised.pid
                );
            }
        }

        let invalid = bind(&fixture::document(
            &fixture::allow_with_invalid_tail(1),
            |_| {},
        ))
        .unwrap();
        assert_eq!(
            validate_with(
                &installed.server,
                &invalid,
                std::time::Duration::from_secs(30),
                PollTiming::IMMEDIATE,
            ),
            Err(Denial::Malformed)
        );
        assert_environment_refusal(
            &installed.server,
            &installed.root,
            "RIAUTH_TEST_SENTINEL",
            "secret",
        );
        assert_environment_refusal(
            &installed.server,
            &installed.root,
            "__CF_USER_TEXT_ENCODING",
            "secret",
        );
    }

    #[cfg(all(feature = "platform", target_os = "macos"))]
    #[test]
    fn request_binding_leaves_wasm_validation_to_the_child() {
        let invalid = fixture::allow_with_invalid_tail(1);
        let document = fixture::document(&invalid, |_| {});
        assert_eq!(check(&document).unwrap_err(), Denial::Malformed);
        let bound = bind(&document).unwrap();
        assert_eq!(
            execute(&bound, &GuestFacts::default(), &BTreeSet::new(), bounds()).unwrap_err(),
            Denial::Malformed
        );
    }

    #[cfg(feature = "platform")]
    #[test]
    fn guest_workdir_retries_collisions_without_claiming_existing_paths() {
        use std::fs;
        use std::path::Path;

        fn names(root: &Path) -> BTreeSet<std::ffi::OsString> {
            fs::read_dir(root)
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect()
        }

        fn assert_private_directory(path: &Path) {
            let metadata = fs::symlink_metadata(path).unwrap();
            assert!(metadata.is_dir());
            assert!(!metadata.file_type().is_symlink());
            #[cfg(unix)]
            {
                use std::os::unix::fs::{MetadataExt, PermissionsExt};
                assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
                assert_eq!(
                    metadata.uid(),
                    fs::metadata(path.parent().unwrap()).unwrap().uid()
                );
            }
        }

        #[cfg(unix)]
        fn identity(path: &Path) -> (u64, u64, u32, u32) {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::symlink_metadata(path).unwrap();
            (
                metadata.dev(),
                metadata.ino(),
                metadata.mode(),
                metadata.uid(),
            )
        }

        let fixture = tempfile::Builder::new()
            .prefix("riauth-guest-workdir-test-")
            .tempdir()
            .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(fixture.path()).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }

        let root = fixture.path().join("directory");
        fs::create_dir(&root).unwrap();
        let collision = root.join("riauth-guest-01010101010101010101010101010101");
        fs::create_dir(&collision).unwrap();
        fs::write(
            collision.join("marker"),
            b"directory remains owned by fixture",
        )
        .unwrap();
        let before = names(&root);
        #[cfg(unix)]
        let collision_identity = identity(&collision);
        let mut draws = 0;
        let owned = guest_workdir_in(&root, || {
            draws += 1;
            assert!(draws <= 2);
            Ok([draws; 16])
        })
        .unwrap();
        assert_eq!(draws, 2);
        assert_eq!(
            owned,
            root.join("riauth-guest-02020202020202020202020202020202")
        );
        assert_private_directory(&owned);
        assert!(names(&owned).is_empty());
        fs::remove_dir(&owned).unwrap();
        assert!(!owned.exists());
        assert_eq!(names(&root), before);
        assert_eq!(
            fs::read(collision.join("marker")).unwrap(),
            b"directory remains owned by fixture"
        );
        #[cfg(unix)]
        assert_eq!(identity(&collision), collision_identity);

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let root = fixture.path().join("symlink");
            fs::create_dir(&root).unwrap();
            let target = fixture.path().join("symlink-target");
            fs::create_dir(&target).unwrap();
            fs::write(target.join("marker"), b"target remains owned by fixture").unwrap();
            let collision = root.join("riauth-guest-03030303030303030303030303030303");
            symlink(&target, &collision).unwrap();
            let before = names(&root);
            let collision_identity = identity(&collision);
            let target_identity = identity(&target);
            let mut draws = 0;
            let owned = guest_workdir_in(&root, || {
                draws += 1;
                assert!(draws <= 2);
                Ok([draws + 2; 16])
            })
            .unwrap();
            assert_eq!(draws, 2);
            assert_eq!(
                owned,
                root.join("riauth-guest-04040404040404040404040404040404")
            );
            assert_private_directory(&owned);
            assert!(names(&owned).is_empty());
            fs::remove_dir(&owned).unwrap();
            assert!(!owned.exists());
            assert_eq!(names(&root), before);
            assert_eq!(fs::read_link(&collision).unwrap(), target);
            assert_eq!(identity(&collision), collision_identity);
            assert_eq!(identity(&target), target_identity);
            assert_eq!(
                fs::read(target.join("marker")).unwrap(),
                b"target remains owned by fixture"
            );
        }

        let root = fixture.path().join("exhaustion");
        fs::create_dir(&root).unwrap();
        let collisions: Vec<_> = (1u8..=8)
            .map(|n| {
                let path = root.join(format!("riauth-guest-{}", format!("{n:02x}").repeat(16)));
                fs::create_dir(&path).unwrap();
                fs::write(path.join("marker"), [n]).unwrap();
                path
            })
            .collect();
        let before = names(&root);
        #[cfg(unix)]
        let collision_identities: Vec<_> = collisions.iter().map(|p| identity(p)).collect();
        let mut draws = 0;
        assert_eq!(
            guest_workdir_in(&root, || {
                draws += 1;
                assert!(draws <= 8, "no ninth allocation attempt");
                Ok([draws; 16])
            }),
            Err(Denial::Failed)
        );
        assert_eq!(draws, 8);
        assert_eq!(names(&root), before);
        for (index, collision) in collisions.iter().enumerate() {
            assert_eq!(
                fs::read(collision.join("marker")).unwrap(),
                [index as u8 + 1]
            );
            #[cfg(unix)]
            assert_eq!(identity(collision), collision_identities[index]);
        }

        let root = fixture.path().join("entropy");
        fs::create_dir(&root).unwrap();
        let mut draws = 0;
        assert_eq!(
            guest_workdir_in(&root, || {
                draws += 1;
                Err(Denial::Failed)
            }),
            Err(Denial::Failed)
        );
        assert_eq!(draws, 1);
        assert!(names(&root).is_empty());

        let before = names(fixture.path());
        let missing = fixture.path().join("missing-parent");
        let mut draws = 0;
        assert_eq!(
            guest_workdir_in(&missing, || {
                draws += 1;
                Ok([9; 16])
            }),
            Err(Denial::Failed)
        );
        assert_eq!(draws, 1);
        assert!(!missing.exists());
        assert_eq!(names(fixture.path()), before);

        let fixture_path = fixture.path().to_owned();
        drop(fixture);
        assert!(!fixture_path.exists());
    }

    #[cfg(not(feature = "platform"))]
    #[test]
    fn essentials_checks_the_manifest_and_does_not_execute() {
        assert!(!runtime_linked());
        let checked = check(&fixture::document(&fixture::allow(), |_| {})).unwrap();
        assert_eq!(checked.module_len(), fixture::allow().len() as u32);
        assert_eq!(
            execute(&checked, &GuestFacts::default(), &profile(), bounds()).unwrap_err(),
            Denial::ExternalRuntimeRequired
        );
        let cargo = include_str!("../../Cargo.toml");
        assert!(cargo.contains("essentials = []"));
        assert!(!cargo.contains("wasmtime"));
    }

    #[cfg(feature = "platform")]
    #[test]
    fn configured_custom_stage_without_a_manifest_stays_rejected() {
        let definition = crate::workflow::parse(
            br#"{
            "format": "riauth.workflow/v1",
            "id": "risk-route",
            "revision": 1,
            "category": "authentication",
            "origin": "configured",
            "entry": "risk",
            "limits": {"max_duration_seconds": 600, "max_executions": 4},
            "steps": [{
                "id": "risk",
                "action": {
                    "type": "custom",
                    "stage": "risk-check",
                    "outputs": ["allow", "block"],
                    "permissions": ["read_request"],
                    "max_output_bytes": 128
                },
                "max_attempts": 1,
                "timeout_seconds": 30,
                "cancellable": true,
                "transitions": [
                    {"on": "allow", "to": "denied"},
                    {"on": "block", "to": "denied"},
                    {"on": "failed", "to": "denied"}
                ]
            }],
            "terminals": [{"id": "denied", "outcome": "denied", "requires": []}]
        }"#,
        )
        .unwrap();
        let mut configured: crate::config::Config = toml::from_str(
            "issuer='http://127.0.0.1:9000'\nlisten='127.0.0.1:9000'\ndata_dir='data'\naccess_token_ttl=300\nrefresh_token_ttl=2592000\nsession_ttl=28800\n",
        )
        .unwrap();
        configured.workflows.insert(
            definition.id.as_str().to_owned(),
            crate::workflow::ConfiguredWorkflow {
                active: true,
                definition,
            },
        );
        let error = configured.validate().unwrap_err().to_string();
        assert!(error.contains("Unknown stage"), "{error}");
    }
}
