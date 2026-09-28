//! Configuration shape retained for safe downgrade checks; the LDAP provider is absent.
pub use crate::ldap_listener::Listener;
pub use crate::model::client_settings::ldap::Settings;

impl Settings {
    pub fn validate(&self, _client: &crate::model::Client) -> crate::error::Result<()> {
        Err(crate::error::Error::bad(
            "LDAP provider requires the Platform build",
        ))
    }
}
