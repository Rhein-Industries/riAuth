//! RADIUS listener configuration retained for parsing and downgrade refusal.
use crate::{
    core::validate_name,
    error::{Error, Result},
    radius_eap_types,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Udp,
    Tls,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Nas {
    pub peer: IpAddr,
    pub client_id: String,
    pub shared_secret_file: Option<PathBuf>,
    pub certificate_sha256: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Listener {
    #[serde(default)]
    pub eap_tls: Option<radius_eap_types::Config>,
    pub listen: SocketAddr,
    pub transport: Transport,
    pub nas: BTreeMap<String, Nas>,
    pub tls_cert_file: Option<PathBuf>,
    pub tls_key_file: Option<PathBuf>,
    pub client_ca_file: Option<PathBuf>,
}

impl Listener {
    pub fn validate(&self) -> Result<()> {
        if let Some(eap) = &self.eap_tls {
            eap.validate()?;
        }
        if self.nas.is_empty() || self.nas.len() > 128 {
            return Err(Error::bad(
                "RADIUS requires one to 128 explicit NAS entries",
            ));
        }
        let mut peers = BTreeSet::new();
        for (id, nas) in &self.nas {
            validate_name(id)?;
            validate_name(&nas.client_id)?;
            if nas.peer.is_unspecified() || nas.peer.is_multicast() {
                return Err(Error::bad("RADIUS NAS needs an explicit unicast peer IP"));
            }
            match self.transport {
                Transport::Udp => {
                    if nas.shared_secret_file.is_none()
                        || nas.certificate_sha256.is_some()
                        || !peers.insert(nas.peer.to_string())
                    {
                        return Err(Error::bad(
                            "UDP RADIUS requires one shared-secret NAS per peer IP",
                        ));
                    }
                }
                Transport::Tls => {
                    let pin = nas.certificate_sha256.as_deref().unwrap_or("");
                    if nas.shared_secret_file.is_some()
                        || !URL_SAFE_NO_PAD.decode(pin).is_ok_and(|b| b.len() == 32)
                        || pin.len() != 43
                        || !peers.insert(format!("{}:{pin}", nas.peer))
                    {
                        return Err(Error::bad(
                            "RadSec requires a unique peer/certificate pin and uses the fixed radsec protocol secret",
                        ));
                    }
                }
            }
        }
        let files = [
            &self.tls_cert_file,
            &self.tls_key_file,
            &self.client_ca_file,
        ];
        if (self.transport == Transport::Tls && files.iter().any(|f| f.is_none()))
            || (self.transport == Transport::Udp && files.iter().any(|f| f.is_some()))
        {
            return Err(Error::bad(
                "RadSec requires server certificate/key and private client CA files; UDP has no TLS files",
            ));
        }
        Ok(())
    }
}
