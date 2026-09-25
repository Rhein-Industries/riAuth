use crate::{
    core::Core,
    crypto::{digest, now},
    error::{Error, Result},
    store::Tx,
};
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
struct Receipt {
    fingerprint: String,
    permissions: Value,
    result: Value,
    expires_at: u64,
}
impl Core {
    pub(crate) fn mutation(
        &self,
        token: &str,
        f: impl FnOnce(&Tx<'_>) -> Result<Value>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let actor = self.principal(tx, token)?;
            let context = current();
            let receipt_key = context.as_ref().and_then(|c| c.idempotency_key.as_ref()).map(|k| digest(&format!("{}\0{k}", actor.id)));
            let permissions = serde_json::to_value(&actor.permissions).map_err(Error::internal)?;
            if let Some(key) = &receipt_key && let Some(receipt) = tx.get::<Receipt>("receipts", key)? {
                if receipt.expires_at <= now() { return Err(Error::conflict("Idempotency receipt expired; inspect state before using a new key")); }
                if receipt.fingerprint != context.as_ref().unwrap().fingerprint { return Err(Error::conflict("Idempotency key was used for a different request")); }
                if receipt.permissions != permissions { return Err(Error::forbidden()); }
                return Ok(receipt.result);
            }
            if let Some(c) = &context {
                if actor.agent && c.revision.is_none() {
                    return Err(Error::new(axum::http::StatusCode::PRECONDITION_REQUIRED, "precondition_required", "Agent mutations require If-Match with the current revision, or use plan/apply"));
                }
                let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
                if c.revision.is_some_and(|r| r != revision) {
                    return Err(Error::conflict("Configuration revision changed"));
                }
            }
            let result = f(tx)?;
            if let Some(key) = receipt_key {
                tx.put("receipts", &key, &Receipt { fingerprint: context.unwrap().fingerprint, permissions, result: result.clone(), expires_at: now() + 86_400 })?;
            }
            Ok(result)
        })
    }
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
