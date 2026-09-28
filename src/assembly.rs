//! Server assembly of identity and protocol ports over concrete storage.

mod authorization;
mod claims;
#[cfg(feature = "platform")]
mod device_trust;
#[cfg(feature = "platform")]
mod event_map;
mod exchange;
mod issuer;
mod keyring;
mod oidc;
mod response;
#[cfg(feature = "platform")]
mod saml;
#[cfg(feature = "platform")]
mod saml_logout;
mod session_protocol;
#[cfg(feature = "platform")]
mod ssf;

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
        identity::record_transition(tx, bucket, key, before, after)?;
        #[cfg(feature = "platform")]
        crate::scim::record_transition(tx, bucket, key, before, after)?;
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
