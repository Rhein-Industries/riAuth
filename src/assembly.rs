//! Server assembly of identity and protocol ports over concrete storage.

mod authenticator;
mod authorization;
mod claims;
#[cfg(feature = "platform")]
mod cloud_directory_budget;
#[cfg(feature = "platform")]
pub(crate) use cloud_directory_budget::cleanup as cloud_budget_cleanup;
#[cfg(feature = "platform")]
mod cloud_directory_catalog;
#[cfg(feature = "platform")]
mod cloud_directory_plan;
#[cfg(feature = "platform")]
pub(crate) use cloud_directory_plan::cleanup_plans as cloud_plan_cleanup;
#[cfg(feature = "platform")]
mod cloud_directory_snapshot;
#[cfg(feature = "platform")]
pub(crate) use cloud_directory_snapshot::cleanup as cloud_snapshot_cleanup;
#[cfg(feature = "platform")]
mod device_trust;
mod directory;
pub use directory::cleanup as directory_cleanup;
pub(crate) use directory::{
    manages as directory_manages, validate_identity as directory_validate_identity,
};
#[cfg(feature = "platform")]
mod event_map;
mod exchange;
mod issuer;
mod keyring;
#[cfg(feature = "platform")]
mod ldap_port;
#[cfg(feature = "platform")]
mod ldap_server;
#[cfg(feature = "platform")]
pub use ldap_port::ldap_start;
mod logout;
#[cfg(feature = "platform")]
mod mtls;
#[cfg(feature = "platform")]
pub(crate) use mtls::clear_user_binding;
mod oidc;
pub(crate) use oidc::{backfill_prepared_index, cleanup_prepared, stamp_prepared_index};
#[cfg(feature = "platform")]
mod outpost;
#[cfg(feature = "platform")]
pub use outpost::cleanup as outpost_cleanup;
#[cfg(feature = "platform")]
pub(crate) use outpost::revoke_sessions as outpost_revoke_sessions;
#[cfg(feature = "platform")]
mod pam;
#[cfg(feature = "platform")]
pub use pam::{cleanup as pam_cleanup, extra_groups as pam_extra_groups};
pub(crate) mod passkey;
mod password;
mod portal_mfa;
mod portal_self_service;
mod portal_sources;
#[cfg(feature = "platform")]
mod proxy_server;
#[cfg(feature = "platform")]
pub(crate) use proxy_server::ProxyRequests;
#[cfg(feature = "platform")]
pub use proxy_server::proxy_start;
#[cfg(feature = "platform")]
mod radius;
#[cfg(feature = "platform")]
pub use radius::cleanup as radius_cleanup;
#[cfg(feature = "platform")]
pub use radius::radius_start;
mod response;
#[cfg(feature = "platform")]
mod saml;
#[cfg(feature = "platform")]
mod saml_logout;
mod session_protocol;
mod signin;
pub use signin::{bearer_backed, bind_proof, discard_staged, proof_valid};
#[cfg(feature = "platform")]
mod ssf;
#[cfg(feature = "platform")]
mod windows_login;

use crate::{
    config::Config,
    crypto,
    dpop::DpopTx,
    error::{Error, Result},
    identity::{self, agent_credentials::Agent, persistence::IdentityTx},
    postgres_store::PostgresConfig,
    store::{RecordTransitions, Store, Tx},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{path::Path, sync::Arc};
use zeroize::Zeroizing;

struct IdentityTransitions;

impl RecordTransitions for IdentityTransitions {
    fn prepare_record(&self, bucket: &str, before: Option<&Value>, after: &mut Value) {
        identity::prepare_record(bucket, before, after);
    }

    fn public_agent(&self, value: &Value) -> Result<Value> {
        Ok(serde_json::from_value::<Agent>(value.clone())
            .map_err(Error::internal)?
            .view())
    }

    fn record_transition(
        &self,
        tx: &Tx<'_>,
        bucket: &str,
        key: &str,
        before: Option<&Value>,
        after: Option<&Value>,
    ) -> Result<()> {
        identity::record_transition(tx, bucket, key, before, after, |prior| {
            crate::delegation::require_elevation_ready(tx, prior)
        })?;
        #[cfg(feature = "platform")]
        crate::scim::record_transition(tx, bucket, key, before, after)?;
        Ok(())
    }
}

#[cfg(not(feature = "platform"))]
impl crate::core::Core {
    pub(crate) fn radius_eap_validate_identity(
        &self,
        _tx: &Tx<'_>,
        identity: &crate::model::Identity,
    ) -> Result<()> {
        if identity.source.is_none() && identity.amr.iter().any(|method| method == "x509") {
            return Err(Error::unauthorized());
        }
        Ok(())
    }
}

impl Store {
    pub fn from_config(config: &Config) -> Result<Self> {
        let key = config
            .database_key_file
            .as_deref()
            .map(crypto::read_key)
            .transpose()?;
        if let Some(pg) = &config.postgres {
            Self::open_postgres(pg.clone(), key)
        } else {
            Self::open_with_key(&config.data_dir.join("riauth.redb"), key)
        }
    }

    pub fn open(path: &Path) -> Result<Self> {
        Self::open_with_key(path, None)
    }

    pub fn open_with_key(path: &Path, key: Option<Zeroizing<[u8; 32]>>) -> Result<Self> {
        Self::open_with_key_raw(path, key, Arc::new(IdentityTransitions))
    }

    pub fn open_postgres(config: PostgresConfig, key: Option<Zeroizing<[u8; 32]>>) -> Result<Self> {
        Self::open_postgres_raw(config, key, Arc::new(IdentityTransitions))
    }
}

impl IdentityTx for Tx<'_> {
    fn get<T: DeserializeOwned>(&self, bucket: &str, key: &str) -> Result<Option<T>> {
        Tx::get(self, bucket, key)
    }

    fn list<T: DeserializeOwned>(&self, bucket: &str) -> Result<Vec<(String, T)>> {
        Tx::list(self, bucket)
    }

    fn scan<T: DeserializeOwned>(
        &self,
        bucket: &str,
        after: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, T)>> {
        Tx::scan(self, bucket, after, limit)
    }

    fn put<T: Serialize>(&self, bucket: &str, key: &str, value: &T) -> Result<()> {
        Tx::put(self, bucket, key, value)
    }

    fn delete(&self, bucket: &str, key: &str) -> Result<()> {
        Tx::delete(self, bucket, key)
    }

    fn maintenance_page<T: DeserializeOwned>(&self, bucket: &str) -> Result<Vec<(String, T)>> {
        Tx::maintenance_page(self, bucket)
    }

    fn mark_security_event(&self, user: &str, event: &str, credential: &str) -> bool {
        Tx::mark_security_event(self, user, event, credential)
    }
}

impl DpopTx for Tx<'_> {
    fn primary_issuer(&self) -> Result<Option<String>> {
        self.get("meta", "issuer")
    }

    fn replay_expiry(&self, proof_id: &str) -> Result<Option<u64>> {
        self.get("dpop_replays", proof_id)
    }

    fn record_replay(&self, proof_id: &str, expires_at: u64) -> Result<()> {
        self.put("dpop_replays", proof_id, &expires_at)
    }
}

impl crate::jose::AssertionTx for Tx<'_> {
    fn assertion_replay_expiry(&self, assertion_id: &str) -> Result<Option<u64>> {
        self.get("assertion_replays", assertion_id)
    }

    fn record_assertion_replay(&self, assertion_id: &str, expires_at: u64) -> Result<()> {
        self.put("assertion_replays", assertion_id, &expires_at)
    }
}
