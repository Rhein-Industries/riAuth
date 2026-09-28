//! Server-side adapters for protocol-owned persistence ports.

use crate::{dpop::DpopTx, error::Result, store::Tx};

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
