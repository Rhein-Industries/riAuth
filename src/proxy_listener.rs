use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub client_id: String,
    pub upstream: String,
    pub ca_file: Option<PathBuf>,
    #[serde(default)]
    pub allow_plain_http: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Listener {
    pub listen: SocketAddr,
    pub tls_cert_file: Option<PathBuf>,
    pub tls_key_file: Option<PathBuf>,
    pub routes: BTreeMap<String, Target>,
    #[serde(default = "body_limit")]
    pub max_body_bytes: usize,
    #[serde(default = "timeout")]
    pub upstream_timeout_seconds: u64,
}
fn body_limit() -> usize {
    8 * 1024 * 1024
}
fn timeout() -> u64 {
    30
}
pub(crate) fn origin(value: &str) -> Result<url::Url> {
    let url = url::Url::parse(value).map_err(|_| Error::bad("Invalid proxy origin"))?;
    if !["http", "https"].contains(&url.scheme()) || url.origin().ascii_serialization() != value {
        return Err(Error::bad(
            "Proxy routes require exact HTTP(S) origins without paths or credentials",
        ));
    }
    Ok(url)
}
impl Listener {
    pub fn validate(&self) -> Result<()> {
        if self.routes.is_empty()
            || self.routes.len() > 64
            || self.max_body_bytes == 0
            || self.max_body_bytes > 64 * 1024 * 1024
            || !(1..=300).contains(&self.upstream_timeout_seconds)
        {
            return Err(Error::bad(
                "Proxy requires 1..64 routes, a 1-byte..64-MiB request limit and 1..300-second upstream timeout",
            ));
        }
        if self.tls_cert_file.is_some() != self.tls_key_file.is_some()
            || !self.listen.ip().is_loopback() && self.tls_cert_file.is_none()
        {
            return Err(Error::bad(
                "Proxy listeners require certificate/key files together and native TLS on non-loopback addresses",
            ));
        }
        let mut authorities = std::collections::BTreeSet::new();
        for (external, target) in &self.routes {
            let url = crate::config::validate_server_url(external)
                .map_err(|_| Error::bad("External proxy origins require HTTPS or HTTP loopback"))?;
            if url.origin().ascii_serialization() != *external
                || !authorities.insert(authority(&url).to_owned())
                || self.tls_cert_file.is_some() && url.scheme() != "https"
            {
                return Err(Error::bad(
                    "Proxy needs unique exact authorities and HTTPS origins when TLS is enabled",
                ));
            }
            crate::core::validate_name(&target.client_id)?;
            let upstream = origin(&target.upstream)?;
            if upstream.scheme() == "http" && target.ca_file.is_some() {
                return Err(Error::bad("Plain HTTP upstream cannot use a CA file"));
            }
            if upstream.scheme() == "http"
                && !target.allow_plain_http
                && !matches!(
                    upstream.host_str(),
                    Some("localhost" | "127.0.0.1" | "[::1]")
                )
            {
                return Err(Error::bad(
                    "Non-loopback HTTP upstream requires explicit allow_plain_http",
                ));
            }
        }
        Ok(())
    }
}
pub(crate) fn authority(url: &url::Url) -> &str {
    &url[url::Position::BeforeHost..url::Position::AfterPort]
}
