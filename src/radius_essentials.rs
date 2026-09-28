//! RADIUS configuration remains decodable; the protocol and EAP server are absent.
pub use crate::model::client_settings::radius::{Attribute, ReplyValue, Settings};
pub use crate::radius_listener::{Listener, Nas, Transport};

impl Settings {
    pub fn validate(&self, _client: &crate::model::Client) -> crate::error::Result<()> {
        Err(crate::error::Error::bad(
            "RADIUS clients require the Platform build",
        ))
    }
}

pub mod eap {
    pub use crate::radius_eap_types::Config;
    pub const CERTIFICATE_ACR: &str = "urn:riauth:acr:certificate";

    pub(crate) fn validate_identity(
        _core: &crate::core::Core,
        _tx: &crate::store::Tx<'_>,
        identity: &crate::model::Identity,
    ) -> crate::error::Result<()> {
        if identity.source.is_none() && identity.amr.iter().any(|method| method == "x509") {
            return Err(crate::error::Error::unauthorized());
        }
        Ok(())
    }
}

pub fn cleanup(_tx: &crate::store::Tx<'_>, _at: u64) -> crate::error::Result<()> {
    Ok(())
}

#[cfg(feature = "fuzzing")]
pub(crate) fn fuzz_packet(_data: &[u8]) {}
