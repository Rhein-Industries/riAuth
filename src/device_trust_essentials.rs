//! Shared config shape with no device verifier or challenge engine.
pub use crate::device_trust_types::{Challenge, DeviceVerification, MAX_FRESHNESS, TrustConfig};

pub fn validate_config(_config: &TrustConfig) -> anyhow::Result<()> {
    anyhow::bail!("Device trust requires the Platform build")
}

pub fn policy_reason(
    _core: &crate::core::Core,
    _tx: &crate::store::Tx<'_>,
    client: &crate::model::Client,
    _identity: Option<&crate::model::Identity>,
) -> crate::error::Result<Option<&'static str>> {
    Ok(client
        .settings
        .require_device_trust
        .then_some("device_trust_verifier_unconfigured"))
}

pub fn require(
    core: &crate::core::Core,
    tx: &crate::store::Tx<'_>,
    client: &crate::model::Client,
    identity: &crate::model::Identity,
) -> crate::error::Result<()> {
    if policy_reason(core, tx, client, Some(identity))?.is_some() {
        return Err(crate::error::Error::oauth(
            "unmet_authentication_requirements",
            "Device trust requires the Platform build",
        ));
    }
    Ok(())
}
