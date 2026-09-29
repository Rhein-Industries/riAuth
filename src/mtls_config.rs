//! Client-certificate configuration shape shared for safe edition transitions.
use crate::{
    config::Config,
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Both modes let users without a client certificate reach ordinary sign-in.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientCertMode {
    #[default]
    Optional,
    Required,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClientCertAuth {
    pub trust_anchors_file: PathBuf,
    #[serde(default)]
    pub mode: ClientCertMode,
    #[serde(default)]
    pub crl_file: Option<PathBuf>,
    #[serde(default)]
    pub forwarded_header: Option<String>,
}

impl ClientCertAuth {
    pub fn validate(&self, config: &Config) -> Result<()> {
        if self.trust_anchors_file.as_os_str().is_empty()
            || self
                .crl_file
                .as_ref()
                .is_some_and(|path| path.as_os_str().is_empty())
        {
            return Err(Error::bad("Client certificate trust files must be set"));
        }
        if let Some(header) = &self.forwarded_header {
            validate_header_name(header)?;
            if config.trusted_proxies.is_empty() {
                return Err(Error::bad(
                    "forwarded_header requires at least one trusted proxy address",
                ));
            }
        }
        if config.tls_cert_file.is_none() && self.forwarded_header.is_none() {
            return Err(Error::bad(
                "Client certificate authentication without native TLS requires forwarded_header",
            ));
        }
        Ok(())
    }
}

fn validate_header_name(name: &str) -> Result<()> {
    if !(1..=64).contains(&name.len())
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        || name.starts_with('-')
        || name.ends_with('-')
    {
        return Err(Error::bad("forwarded_header must be one HTTP header token"));
    }
    if matches!(
        name.to_ascii_lowercase().as_str(),
        "authorization"
            | "cookie"
            | "host"
            | "content-type"
            | "content-length"
            | "x-forwarded-for"
            | "forwarded"
            | "connection"
            | "transfer-encoding"
    ) {
        return Err(Error::bad(
            "forwarded_header cannot replace an existing request header",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct TlsClientCerts {
    pub ders: Vec<Vec<u8>>,
}
