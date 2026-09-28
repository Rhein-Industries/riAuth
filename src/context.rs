use crate::{crypto::now, error::Result, store::Tx};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{cell::RefCell, net::IpAddr};

#[derive(Clone, Default)]
pub struct RequestContext {
    pub request_id: String,
    pub run_id: Option<String>,
    pub idempotency_key: Option<String>,
    pub fingerprint: String,
    pub revision: Option<u64>,
    /// Attributed client address, after trusted-proxy processing.
    pub client_ip: Option<IpAddr>,
    /// User-Agent without control characters, at most 256 bytes.
    pub user_agent: Option<String>,
}
tokio::task_local! { pub static HTTP_CONTEXT: RequestContext; }
thread_local! { static CURRENT: RefCell<Option<RequestContext>> = const { RefCell::new(None) }; }
pub fn current() -> Option<RequestContext> {
    CURRENT.with_borrow(Clone::clone)
}
/// Who asked for an interactive request, shown to the approving terminal.
pub(crate) fn requester() -> Option<Value> {
    current().map(|c| json!({"ip": c.client_ip.map(|ip| ip.to_string()), "user_agent": c.user_agent, "at": now()}))
}
pub fn scope<T>(context: Option<RequestContext>, f: impl FnOnce() -> T) -> T {
    struct Reset(Option<RequestContext>);
    impl Drop for Reset {
        fn drop(&mut self) {
            CURRENT.with_borrow_mut(|v| *v = self.0.take());
        }
    }
    let _reset = Reset(CURRENT.replace(context));
    f()
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Receipt {
    pub(crate) fingerprint: String,
    pub(crate) permissions: Value,
    pub(crate) result: Value,
    pub(crate) expires_at: u64,
}
pub fn cleanup(tx: &Tx<'_>) -> Result<()> {
    // Keep expired receipts as bounded tombstones for seven days, preventing accidental immediate reuse.
    for (id, receipt) in tx.maintenance_page::<Receipt>("receipts")? {
        if receipt.expires_at.saturating_add(6 * 86_400) <= now() {
            tx.delete("receipts", &id)?;
        }
    }
    Ok(())
}
