//! EAP listener configuration shape shared for downgrade-safe decoding.
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub certificate_file: PathBuf,
    pub key_file: PathBuf,
    pub client_ca_file: PathBuf,
    pub client_crl_file: PathBuf,
    pub ocsp_response_file: Option<PathBuf>,
    #[serde(default)]
    pub tls12: bool,
    #[serde(default = "fragment_size")]
    pub fragment_size: usize,
}

fn fragment_size() -> usize {
    1024
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        if !(256..=1400).contains(&self.fragment_size) {
            return Err(Error::bad("EAP-TLS fragment_size must be 256..1400 bytes"));
        }
        Ok(())
    }
}
