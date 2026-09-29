//! Fail-closed admission for an extension module.
//!
//! This crate forbids unsafe code and links no Wasm or scripting engine, so it
//! cannot give a native registrant a separate address space, a fuel meter, or a
//! syscall boundary. [`check`] accepts only a manifest inside the limits below.
//! It hashes the module and drops the bytes. [`execute`] then refuses. The
//! executor does not call either function. See `docs/workflows.md`.

use super::{
    Id, Label, MAX_CUSTOM_OUTPUT_BYTES, MAX_CUSTOM_OUTPUTS, MAX_CUSTOM_TIMEOUT_SECONDS,
    StagePermission,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const FORMAT: &str = "riauth.workflow-extension/v1";
/// No engine is compiled into this crate. Keep this false until a runtime that
/// enforces the limits below is linked, and do not treat a true value as
/// permission to call [`execute`] successfully: this function still refuses.
pub const RUNTIME_LINKED: bool = false;
pub const MAX_MANIFEST_BYTES: usize = 128 * 1024;
pub const MAX_FUEL: u32 = 10_000;
pub const MAX_MEMORY_BYTES: u32 = 65_536;
pub const MAX_MODULE_BYTES: u32 = MAX_MEMORY_BYTES;
pub const MAX_INPUT_BYTES: u32 = 4_096;
pub const MAX_TIMEOUT_SECONDS: u32 = MAX_CUSTOM_TIMEOUT_SECONDS;
pub const MAX_OUTPUT_BYTES: u32 = MAX_CUSTOM_OUTPUT_BYTES;

const RESERVED_SIGNALS: [&str; 5] = ["verified", "failed", "completed", "granted", "denied"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denial {
    Malformed,
    Limit,
    Permission,
    Integrity,
    ExternalRuntimeRequired,
}

impl Denial {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::Limit => "limit",
            Self::Permission => "permission",
            Self::Integrity => "integrity",
            Self::ExternalRuntimeRequired => "external_runtime_required",
        }
    }
}

/// Manifest that passed the bounds. The module bytes are not retained.
#[derive(Clone, Debug, PartialEq, Eq)]
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
}

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

/// Validate bounds and pin the module hash. The decoded module is dropped
/// before this returns.
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
        || !(1..=MAX_MEMORY_BYTES).contains(&manifest.memory_bytes)
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
    drop(module);
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
    })
}

/// Refuse to run a checked module. There is no success value in this build.
pub fn execute(checked: &Checked) -> Result<Label, Denial> {
    let _ = (checked, RUNTIME_LINKED);
    Err(Denial::ExternalRuntimeRequired)
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

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use sha2::{Digest, Sha256};

    fn document(module: &[u8], mutate: impl FnOnce(&mut serde_json::Value)) -> Vec<u8> {
        let digest: [u8; 32] = Sha256::digest(module).into();
        let mut hex = String::with_capacity(64);
        for byte in digest {
            hex.push_str(&format!("{byte:02x}"));
        }
        let mut value = serde_json::json!({
            "format": FORMAT,
            "stage": "risk-check",
            "outputs": ["allow", "block"],
            "permissions": ["read_request"],
            "fuel": 1_000,
            "memory_bytes": 4_096,
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

    #[test]
    fn limits_match_the_held_contract_and_no_runtime_is_linked() {
        assert!(!runtime_linked());
        assert_eq!(MAX_INPUT_BYTES, crate::workflow::extension::MAX_INPUT_BYTES);
        assert_eq!(MAX_TIMEOUT_SECONDS, 30);
        assert_eq!(MAX_OUTPUT_BYTES, 4_096);
        assert_eq!(MAX_FUEL, 10_000);
        assert_eq!(MAX_MEMORY_BYTES, 65_536);
        let cargo = include_str!("../../Cargo.toml");
        assert!(cargo.contains("unsafe_code = \"forbid\""));
        for name in ["wasmtime", "wasmer", "wasm3", "rhai", "mlua", "deno_core"] {
            assert!(!cargo.contains(name), "{name}");
        }
        let executor = include_str!("executor.rs");
        assert!(!executor.contains("extension_gate"));
        assert!(!executor.contains("Host::invoke"));
    }

    #[test]
    fn a_bounded_manifest_is_hashed_and_then_refused() {
        let module = b"route-v1";
        let checked = check(&document(module, |_| {})).unwrap();
        assert_eq!(checked.stage().as_str(), "risk-check");
        assert_eq!(checked.outputs().len(), 2);
        assert!(
            checked
                .permissions()
                .contains(&StagePermission::ReadRequest)
        );
        assert!(!checked.permissions().contains(&StagePermission::Network));
        assert_eq!(checked.fuel(), 1_000);
        assert_eq!(checked.memory_bytes(), 4_096);
        assert_eq!(checked.module_len(), module.len() as u32);
        assert!(!format!("{checked:?}").contains("route-v1"));
        assert_eq!(
            execute(&checked).unwrap_err(),
            Denial::ExternalRuntimeRequired
        );
        let other = check(&document(b"other-module", |_| {})).unwrap();
        assert_ne!(checked.module_sha256(), other.module_sha256());
        assert_eq!(
            execute(&other).unwrap_err().as_str(),
            "external_runtime_required"
        );
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
            check(&document(b"route-v1", |value| {
                value["format"] = serde_json::json!("other");
            }))
            .unwrap_err(),
            Denial::Malformed
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["fuel"] = serde_json::json!(0);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["fuel"] = serde_json::json!(MAX_FUEL + 1);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["memory_bytes"] = serde_json::json!(0);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["memory_bytes"] = serde_json::json!(MAX_MEMORY_BYTES + 1);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["timeout_seconds"] = serde_json::json!(31);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["max_output_bytes"] = serde_json::json!(0);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["max_input_bytes"] = serde_json::json!(MAX_INPUT_BYTES + 1);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["outputs"] = serde_json::json!(["verified"]);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["outputs"] = serde_json::json!(["allow", "allow"]);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["permissions"] = serde_json::json!(["network"]);
            }))
            .unwrap_err(),
            Denial::Permission
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["permissions"] = serde_json::json!(["read_request", "read_request"]);
            }))
            .unwrap_err(),
            Denial::Permission
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
                value["network"] = serde_json::json!("allow");
            }))
            .unwrap_err(),
            Denial::Permission
        );
        assert_eq!(
            check(&document(&vec![0u8; 64], |value| {
                value["memory_bytes"] = serde_json::json!(32);
            }))
            .unwrap_err(),
            Denial::Limit
        );
        assert_eq!(
            check(&document(b"route-v1", |value| {
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
            check(&document(b"route-v1", |value| {
                value["module_sha256"] = serde_json::json!("ab".repeat(32));
            }))
            .unwrap_err(),
            Denial::Integrity
        );
    }

    #[cfg(feature = "platform")]
    #[test]
    fn configured_custom_stage_stays_rejected() {
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
        let checked = check(&document(b"route-v1", |_| {})).unwrap();
        assert_eq!(
            execute(&checked).unwrap_err(),
            Denial::ExternalRuntimeRequired
        );
    }
}
