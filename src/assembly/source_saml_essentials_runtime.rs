//! Persisted SAML shapes remain readable; upstream SAML behavior is absent.
pub use super::saml_types::Settings;
pub(crate) use super::saml_types::UpstreamSession;

impl Settings {
    pub fn validate(&self, _source: &super::Source) -> crate::error::Result<()> {
        Err(crate::error::Error::bad(
            "SAML sources require the Platform build",
        ))
    }

    pub(crate) fn key(
        &self,
        _tx: &crate::store::Tx<'_>,
    ) -> crate::error::Result<crate::crypto::SigningKey> {
        Err(crate::error::Error::bad(
            "SAML sources require the Platform build",
        ))
    }

    pub(super) fn authorization(
        &self,
        _tx: &crate::store::Tx<'_>,
        _core: &crate::core::Core,
        _source: &super::Source,
        _pending: &super::Login,
        _state: &str,
    ) -> crate::error::Result<String> {
        Err(crate::error::Error::bad(
            "SAML sources require the Platform build",
        ))
    }
}

pub(crate) fn cleanup(_tx: &crate::store::Tx<'_>, _at: u64) -> crate::error::Result<()> {
    Ok(())
}
