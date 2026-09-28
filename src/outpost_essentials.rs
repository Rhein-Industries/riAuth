//! Proxy configuration shape retained for downgrade checks; no proxy protocol adapter.
pub use crate::model::client_settings::proxy::{Domain, Settings};

impl Settings {
    pub fn validate(&self, _client: &crate::model::Client) -> crate::error::Result<()> {
        Err(crate::error::Error::bad(
            "Proxy clients require the Platform build",
        ))
    }
}

pub(crate) fn revoke_sessions(
    _tx: &crate::store::Tx<'_>,
    _user_id: &str,
    _client_id: &str,
) -> crate::error::Result<()> {
    Ok(())
}

pub fn cleanup(_tx: &crate::store::Tx<'_>, _at: u64) -> crate::error::Result<()> {
    Ok(())
}
