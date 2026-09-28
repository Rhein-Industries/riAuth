//! Concrete proxy profile and outpost ingress rate limiting over storage.

use crate::{
    core::Core,
    error::{Error, Result},
    model::Client,
    outpost::Settings,
};
use axum::http::StatusCode;
use std::net::IpAddr;

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
