//! Transaction operations required by shared identity policy and durable effects.
//!
//! Storage implements this port with its existing transaction methods. Keeping
//! those methods behind the port preserves indexes, audit changes, prepared
//! conflict tracking, and per-transaction security-event deduplication.
use crate::error::Result;
use serde::{Serialize, de::DeserializeOwned};

pub trait IdentityTx {
    fn get<T: DeserializeOwned>(&self, bucket: &str, key: &str) -> Result<Option<T>>;
    fn list<T: DeserializeOwned>(&self, bucket: &str) -> Result<Vec<(String, T)>>;
    fn put<T: Serialize>(&self, bucket: &str, key: &str, value: &T) -> Result<()>;
    fn delete(&self, bucket: &str, key: &str) -> Result<()>;
    fn maintenance_page<T: DeserializeOwned>(&self, bucket: &str) -> Result<Vec<(String, T)>>;
    fn mark_security_event(&self, user: &str, event: &str, credential: &str) -> bool;
}
