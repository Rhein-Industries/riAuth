use crate::error::{Error, Result};
use serde_json::{Value, json};
pub const NAMES: &[&str] = &[
    "radius-certificate",
    "windows-device",
    "windows-login",
    "client-certificate",
    "directory-plan",
    "cloud-directory-plan",
    "provisioning-plan",
    "invitation",
    "authentik-import",
    "manifest",
    "plan",
    "apply",
    "agent",
    "provider",
    "source",
    "source-input",
    "totp-import",
    "key-input",
    "registration-template",
    "registration",
    "user-create",
    "user-update",
    "client-create",
    "client-update",
    "cli-result",
];
pub fn schema(name: &str) -> Result<Value> {
    Ok(match name {
        "radius-certificate" => json!(schemars::schema_for!(crate::radius::eap::CertificateInput)),
        "windows-device" => json!(schemars::schema_for!(crate::windows_login::EnrollDevice)),
        "windows-login" => json!(schemars::schema_for!(crate::windows_login::WindowsLogin)),
        "client-certificate" => json!(schemars::schema_for!(crate::mtls::BindInput)),
        "directory-plan" => json!(schemars::schema_for!(crate::directory::Plan)),
        "cloud-directory-plan" => json!(schemars::schema_for!(crate::cloud_directory::Plan)),
        "provisioning-plan" => json!(schemars::schema_for!(crate::provisioning::Plan)),
        "invitation" => json!(schemars::schema_for!(crate::lifecycle::Invitation)),
        "authentik-import" => json!(schemars::schema_for!(crate::migration::Import)),
        "manifest" => json!(schemars::schema_for!(crate::state::Manifest)),
        "plan" => json!(schemars::schema_for!(crate::state::Plan)),
        "apply" => json!(schemars::schema_for!(crate::state::ApplyRequest)),
        "agent" => json!(schemars::schema_for!(crate::agent::NewAgent)),
        "registration-template" => json!(schemars::schema_for!(
            crate::registration::RegistrationTemplate
        )),
        "registration" => json!(schemars::schema_for!(
            crate::registration::RegistrationRequest
        )),
        "provider" => json!(schemars::schema_for!(crate::model::ProviderSettings)),
        "source" => json!(schemars::schema_for!(crate::source::SourceSpec)),
        "source-input" => json!(schemars::schema_for!(crate::source::SourceInput)),
        "totp-import" => json!(schemars::schema_for!(crate::authenticator::TotpImport)),
        "key-input" => json!(schemars::schema_for!(crate::keyring::KeyInput)),
        "user-create" => json!(schemars::schema_for!(crate::model::NewUser)),
        "user-update" => json!(schemars::schema_for!(crate::model::UserPatch)),
        "client-create" => json!(schemars::schema_for!(crate::model::NewClient)),
        "client-update" => json!(schemars::schema_for!(crate::model::ClientPatch)),
        "cli-result" => {
            json!({"$schema": "https://json-schema.org/draft/2020-12/schema", "oneOf": [
                {"type": "object", "required": ["schema_version", "ok", "data"], "additionalProperties": false, "properties": {"schema_version": {"const": "riauth.cli/v1"}, "ok": {"const": true}, "data": {}}},
                {"type": "object", "required": ["schema_version", "ok", "error", "exit_code"], "additionalProperties": false, "properties": {"schema_version": {"const": "riauth.cli/v1"}, "ok": {"const": false}, "exit_code": {"type": "integer"}, "error": {"type": "object", "required": ["code", "message", "http_status", "retryable"], "properties": {"code": {"type": "string"}, "message": {"type": "string"}, "http_status": {"type": ["integer", "null"]}, "retryable": {"type": "boolean"}}}}}
            ]})
        }
        _ => return Err(Error::missing("Unknown schema; see capabilities.schemas")),
    })
}
