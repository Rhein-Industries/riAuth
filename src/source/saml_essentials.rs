//! Persisted SAML shapes remain readable; upstream SAML behavior is absent.
pub use super::saml_types::Settings;
pub(crate) use super::saml_types::UpstreamSession;

// Compatibility path for the Essentials cleanup stub.
#[allow(unused_imports)]
pub(crate) use crate::assembly::source_saml_cleanup_stub as cleanup;
