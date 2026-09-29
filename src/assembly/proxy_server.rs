//! Concrete proxy profile and outpost ingress rate limiting over storage.

use crate::{
    api::App,
    core::Core,
    error::{Error, Result},
    model::Client,
    outpost::Settings,
    proxy_server::{self, Listener},
};
use axum::{
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde_json::Value;
use std::{collections::BTreeMap, net::IpAddr};

#[derive(Clone)]
pub(crate) struct ProxyRequests {
    app: App,
}

impl ProxyRequests {
    pub(crate) async fn profile(&self, id: String, external: String) -> Result<Settings> {
        self.app
            .run(move |core| core.proxy_profile(&id, &external))
            .await
    }

    pub(crate) async fn outpost_rate_limit(&self, peer: IpAddr, bucket: String) -> Result<()> {
        self.app
            .run(move |core| core.proxy_outpost_rate_limit(peer, &bucket))
            .await
    }

    pub(crate) async fn outpost_start(&self, id: String, return_to: String) -> Result<Response> {
        self.app
            .run(move |core| {
                crate::api::browser_response(core.outpost_start(
                    &id,
                    proxy_server::internal_peer(),
                    &return_to,
                )?)
            })
            .await
    }

    pub(crate) async fn outpost_callback(
        &self,
        id: String,
        headers: HeaderMap,
        pairs: Vec<(String, String)>,
    ) -> Result<Response> {
        self.app
            .run(move |core| {
                crate::api::browser_response(core.outpost_callback(
                    &id,
                    proxy_server::internal_peer(),
                    &headers,
                    pairs,
                )?)
            })
            .await
    }

    pub(crate) async fn outpost_logout(&self, id: String, headers: HeaderMap) -> Result<Response> {
        self.app
            .run(move |core| {
                crate::api::browser_response(core.outpost_logout(
                    &id,
                    proxy_server::internal_peer(),
                    &headers,
                )?)
            })
            .await
    }

    pub(crate) async fn authenticate(
        &self,
        id: String,
        headers: HeaderMap,
    ) -> Result<(Value, HeaderMap)> {
        self.app
            .run(move |core| core.outpost_auth(&id, proxy_server::internal_peer(), &headers))
            .await
    }

    pub(crate) async fn login_url(&self, id: String, headers: HeaderMap) -> Result<String> {
        self.app
            .run(move |core| core.outpost_login_url(&id, proxy_server::internal_peer(), &headers))
            .await
    }

    pub(crate) async fn recheck(&self, id: String, headers: HeaderMap) -> Result<()> {
        self.app
            .run(move |core| {
                core.outpost_auth(&id, proxy_server::internal_peer(), &headers)
                    .map(|_| ())
            })
            .await
    }
}

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

    fn requests(&self) -> ProxyRequests {
        ProxyRequests {
            app: App::new(self.clone()),
        }
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
