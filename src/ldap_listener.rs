use crate::{
    core::validate_name,
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Listener {
    pub listen: SocketAddr,
    pub client_id: String,
    pub allowed_peers: BTreeSet<IpAddr>,
    pub tls_cert_file: Option<PathBuf>,
    pub tls_key_file: Option<PathBuf>,
    #[serde(default)]
    pub ldaps: bool,
    #[serde(default)]
    pub local_unencrypted: bool,
}
impl Listener {
    pub fn validate(&self) -> Result<()> {
        validate_name(&self.client_id)?;
        if self.allowed_peers.is_empty()
            || self.allowed_peers.len() > 128
            || self
                .allowed_peers
                .iter()
                .any(|ip| ip.is_unspecified() || ip.is_multicast())
        {
            return Err(Error::bad(
                "LDAP listeners require one to 128 explicit peer IPs",
            ));
        }
        if self.local_unencrypted {
            if !self.listen.ip().is_loopback()
                || self.ldaps
                || self.tls_cert_file.is_some()
                || self.tls_key_file.is_some()
            {
                return Err(Error::bad(
                    "Unencrypted LDAP is restricted to an explicit loopback listener",
                ));
            }
        } else if self.tls_cert_file.is_none() || self.tls_key_file.is_none() {
            return Err(Error::bad(
                "LDAP requires certificate and key files for LDAPS or mandatory STARTTLS",
            ));
        }
        Ok(())
    }
}
