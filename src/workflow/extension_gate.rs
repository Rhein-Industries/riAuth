//! Isolated guest for one configured custom stage.
//!
//! Platform links Wasmi 0.40 and runs a module only after [`check`] has hashed
//! it and rejected host imports. The guest exports `memory` and `route`, sees
//! one fixed 64 KiB page, and returns one declared label. Essentials does not
//! link Wasmi; [`execute`] there still returns `external_runtime_required`.
//! The in-process host in [`super::extension`] stays unwired.
//!
//! Wasmi 0.40.0 cannot stop a running guest at a wall-clock deadline. Its
//! `Config` has no epoch or interrupt control. `Store::call_hook` runs only
//! when the host calls Wasm or Wasm calls a host function, and `call_resumable`
//! pauses only when a host function returns an error. This guest has no
//! imports, so validation, translation, and `route` run on the caller until
//! they return or the call spends its fuel. `route.call` holds `&mut Store`
//! for that whole call, so another thread cannot drain its fuel. A detached
//! guest thread would still be running after the caller continued, and
//! stopping it requires `unsafe`, which this crate forbids. Fuel is the
//! instruction budget
//! `timeout_seconds * FUEL_PER_SECOND`. The engine installs
//! `min(manifest fuel, that budget)` before the instance starts. When those
//! two budgets are equal, fuel exhaustion is reported as `fuel`.
//!
//! Admission applies a separate resource cap before `Module::new`: one
//! `() -> i32` function, two exports, at most [`MAX_GUEST_LOCALS`] i32 locals,
//! and no data segment. A function body whose translation charge exceeds
//! `min(manifest fuel, timeout_seconds * FUEL_PER_SECOND)` is refused before
//! `Module::new`. Wasmi 0.40.0 charges [`GUEST_TRANSLATION_FUEL_PER_BYTE`]
//! fuel per body byte on the first call. The refusal uses the same rule as
//! execution: `timeout` when the timeout budget is strictly smaller than the
//! manifest fuel, and `fuel` otherwise. `check` validates a body that fits
//! once. The first `execute` reuses that module. A later `execute` parses the
//! bytes again, so every call pays the translation charge. Validation of a
//! fitting body is not fuel-metered, and it is not a wall-clock interrupt.
//!
//! The interpreter value stack is allocated when `route` is called.
//! `ResourceLimiter` does not account for it. This guest sets the Wasmi
//! `StackLimits` initial and maximum height to [`GUEST_VALUE_STACK_SLOTS`]
//! `UntypedVal` slots (8 bytes each). `ValueStack::new` reserves that buffer,
//! and `extend_by` returns `StackOverflow` before `Vec::reserve` when a frame
//! would make the live length reach the height. The host reports that trap as
//! `limit`. A frame that fits still runs on the caller until it returns or
//! spends its fuel. The call depth stays [`GUEST_CALL_DEPTH`] frames.
//!
//! `route` returns a label length. [`super::Label`] is at most
//! [`MAX_ROUTE_LABEL_BYTES`] bytes, so the host reads at most that many bytes
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

pub const FORMAT: &str = "riauth.workflow-extension/v1";
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
/// Instruction budget charged for one manifest second. This is Wasmi fuel,
/// not a wall-clock second.
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
        }
    }
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
    /// Wasmi image of `module`. The first execution reuses it. A later
    /// execution parses `module` again so translation fuel is charged again.
    #[cfg(feature = "platform")]
    prepared: PreparedGuest,
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

/// Check every configured manifest. The map key is the stage id.
pub(crate) fn stage_registration(
    documents: &BTreeMap<String, String>,
) -> Result<BTreeMap<Id, Checked>, Denial> {
    if documents.len() > 16 {
        return Err(Denial::Limit);
    }
    let mut registered = BTreeMap::new();
    for (key, document) in documents {
        let checked = check(document.as_bytes())?;
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
/// is not the fixed guest shape. A body that fits the translation budget is
/// validated once. The bytes and that prepared module are retained for [`execute`].
pub fn check(document: &[u8]) -> Result<Checked, Denial> {
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
    #[cfg(feature = "platform")]
    let prepared = admit(&module, manifest.fuel, manifest.timeout_seconds)?;
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
        #[cfg(feature = "platform")]
        prepared,
    })
}

/// Run `route` under the fuel, memory, timeout, and output caps.
///
/// The guest has no host imports. On Essentials this returns
/// [`Denial::ExternalRuntimeRequired`] and does not interpret the bytes.
pub fn execute(
    checked: &Checked,
    facts: &GuestFacts,
    requested: &BTreeSet<StagePermission>,
    bounds: StepBounds,
) -> Result<Label, Denial> {
    #[cfg(not(feature = "platform"))]
    {
        let _ = (checked, facts, requested, bounds);
        return Err(Denial::ExternalRuntimeRequired);
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
    let frame = project(
        facts,
        requested,
        checked.permissions(),
        checked.max_input_bytes,
    )?;
    let timeout_fuel = u64::from(bounds.timeout_seconds).saturating_mul(FUEL_PER_SECOND);
    let manifest_fuel = u64::from(checked.fuel);
    let timeout_tighter = timeout_fuel < manifest_fuel;
    let fuel = timeout_fuel.min(manifest_fuel);
    let module = checkout_module(checked)?;
    let engine = module.engine().clone();
    let mut store = wasmi::Store::new(
        &engine,
        wasmi::StoreLimitsBuilder::new()
            .memory_size(usize::try_from(checked.memory_bytes).map_err(|_| Denial::Limit)?)
            .table_elements(0)
            .instances(1)
            .tables(0)
            .memories(1)
            .trap_on_grow_failure(true)
            .build(),
    );
    store.limiter(|limits| limits);
    store.set_fuel(fuel).map_err(|_| Denial::Failed)?;
    let linker = wasmi::Linker::<wasmi::StoreLimits>::new(&engine);
    let instance = linker
        .instantiate(&mut store, &module)
        .and_then(|ready| ready.start(&mut store))
        .map_err(|error| classify(&error, timeout_tighter))?;
    let memory = instance.get_memory(&store, "memory").ok_or(Denial::Limit)?;
    let mut window = vec![0u8; INPUT_WINDOW];
    window[..frame.len()].copy_from_slice(&frame);
    memory
        .write(&mut store, INPUT_OFFSET, &window)
        .map_err(|_| Denial::Limit)?;
    zeroize::Zeroize::zeroize(window.as_mut_slice());
    let route = instance
        .get_typed_func::<(), i32>(&store, "route")
        .map_err(|_| Denial::Limit)?;
    let length = match route.call(&mut store, ()) {
        Ok(length) => length,
        Err(error) => return Err(classify(&error, timeout_tighter)),
    };
    // Manifest caps reach 4,096 bytes. A label is at most 32, so the copy stays there.
    let Ok(reported) = u32::try_from(length) else {
        return Err(Denial::Output);
    };
    let output_cap = bounds.max_output_bytes.min(MAX_ROUTE_LABEL_BYTES);
    if reported > output_cap {
        return Err(Denial::Output);
    }
    let reported = usize::try_from(reported).map_err(|_| Denial::Output)?;
    if !checked
        .outputs
        .iter()
        .any(|label| label.as_str().len() == reported)
    {
        return Err(Denial::UndeclaredOutput);
    }
    let mut output = vec![0u8; reported];
    if memory.read(&store, 0, &mut output).is_err() {
        zeroize::Zeroize::zeroize(output.as_mut_slice());
        return Err(Denial::Output);
    }
    let matched = checked
        .outputs
        .iter()
        .find(|label| label.as_str().as_bytes() == output.as_slice())
        .cloned();
    zeroize::Zeroize::zeroize(output.as_mut_slice());
    matched.ok_or(Denial::UndeclaredOutput)
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
struct PreparedGuest {
    inner: std::sync::Arc<PreparedGuestInner>,
}

#[cfg(feature = "platform")]
struct PreparedGuestInner {
    module: wasmi::Module,
    /// Wasmi caches a successful translation and permanently fails a translation
    /// that ran out of fuel. Either result would change the next call's budget.
    spent: std::sync::atomic::AtomicBool,
}

#[cfg(feature = "platform")]
impl Clone for PreparedGuest {
    fn clone(&self) -> Self {
        Self {
            inner: std::sync::Arc::clone(&self.inner),
        }
    }
}

#[cfg(feature = "platform")]
impl PreparedGuest {
    fn new(module: wasmi::Module) -> Self {
        Self {
            inner: std::sync::Arc::new(PreparedGuestInner {
                module,
                spent: std::sync::atomic::AtomicBool::new(false),
            }),
        }
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
fn admit(wasm: &[u8], fuel: u32, timeout_seconds: u32) -> Result<PreparedGuest, Denial> {
    let body_len = admit_sections(wasm)?;
    admit_translation_budget(fuel, timeout_seconds, body_len)?;
    Ok(PreparedGuest::new(fresh_module(wasm)?))
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
fn checkout_module(checked: &Checked) -> Result<wasmi::Module, Denial> {
    let first = checked.prepared.inner.spent.compare_exchange(
        false,
        true,
        std::sync::atomic::Ordering::AcqRel,
        std::sync::atomic::Ordering::Acquire,
    );
    if first.is_ok() {
        Ok(checked.prepared.inner.module.clone())
    } else {
        fresh_module(&checked.module)
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

#[cfg(test)]
#[cfg_attr(not(feature = "platform"), allow(dead_code))]
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
        for _ in 0..count {
            body.push(0x1a);
        }
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

    fn bounds() -> StepBounds {
        StepBounds {
            timeout_seconds: 30,
            max_output_bytes: 128,
        }
    }

    fn profile() -> BTreeSet<StagePermission> {
        BTreeSet::from([StagePermission::ReadProfile])
    }

    #[cfg(feature = "platform")]
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
            check(&fixture::document(&vec![0u8; 64], |value| {
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

    #[cfg(feature = "platform")]
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

    #[cfg(feature = "platform")]
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
            check(&fixture::document(&fixture::with_locals(MAX_GUEST_LOCALS + 1), |_| {}))
                .unwrap_err(),
            Denial::Limit
        );
        assert!(check(&fixture::document(&fixture::with_locals(MAX_GUEST_LOCALS), |_| {})).is_ok());
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

    #[cfg(feature = "platform")]
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

    #[cfg(feature = "platform")]
    fn translation_charge(module: &[u8]) -> u64 {
        let body = admit_sections(module).expect("fixture has one function body");
        u64::from(body).saturating_mul(GUEST_TRANSLATION_FUEL_PER_BYTE)
    }

    #[cfg(feature = "platform")]
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

    #[cfg(feature = "platform")]
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
        // The prepared module has already translated or failed. A second call
        // parses again and still cannot pay both translation and `route`.
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

    #[cfg(feature = "platform")]
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

    #[cfg(feature = "platform")]
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

    #[cfg(feature = "platform")]
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
