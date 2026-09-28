//! Concrete proxy profile and outpost ingress rate limiting over storage.

use crate::{
    api::App,
    core::Core,
    error::{Error, Result},
    model::Client,
    outpost::Settings,
    proxy_server::{self, Listener},
};
use axum::http::StatusCode;
use std::{collections::BTreeMap, net::IpAddr};

pub async fn proxy_start(mut core: Core) -> anyhow::Result<proxy_server::Servers> {
    // This Core is private to the embedded proxy. No management/API router uses its local trust marker.
    let peer = proxy_server::internal_peer();
    if !core.config.trusted_proxies.contains(&peer) {
        core.config.trusted_proxies.push(peer);
    }
    proxy_server::start_with_port(core).await
}

impl proxy_server::ProxyPort for Core {
    fn listeners(&self) -> &BTreeMap<String, Listener> {
        &self.config.proxy_listeners
    }

    fn app(&self) -> App {
        App::new(self.clone())
    }

    fn bind_listener(&self, id: &str, listener: &Listener) -> crate::capability::ListenerLease {
        self.runtime.bind_proxy(id, listener)
    }
}

impl Core {
    pub(crate) fn proxy_profile(&self, id: &str, external: &str) -> Result<Settings> {
        self.store.read(|tx| {
            let client = tx
                .get::<Client>("clients", id)?
                .filter(|c| c.enabled)
                .ok_or_else(Error::forbidden)?;
            let settings = client.settings.proxy.clone().ok_or_else(Error::forbidden)?;
            settings.validate(&client)?;
            if !settings.allows_origin(external) {
                return Err(Error::forbidden());
            }
            Ok(settings)
        })
    }

    pub(crate) fn proxy_outpost_rate_limit(&self, peers: IpAddr, bucket: &str) -> Result<()> {
        if self
            .store
            .shared_rate_limit(peers, bucket, 60)
            .map_err(Error::internal)?
        {
            return Err(Error::new(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Retry shortly",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "proxy_server_tests.rs"]
mod tests;
