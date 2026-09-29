//! Shared durable security-event intent without the SSF transport or management adapter.
pub(crate) use crate::identity::signals::enqueue;
pub use crate::identity::signals::{
    ACCOUNT_DISABLED, CREDENTIAL_CHANGE, Delivery, PUSH, SESSION_REVOKED, SUPPORTED, Stream,
};

pub fn cleanup(tx: &crate::store::Tx<'_>, _at: u64) -> crate::error::Result<()> {
    crate::identity::signals::ensure_absent(tx)
}
