//! Embedded source stage transaction entrypoints over concrete Core storage.

use crate::{core::Core, error::Result};
use serde_json::Value;

impl Core {
    pub fn source_stage_cancel(&self, stage_id: &str, authorization_id: &str) -> Result<Value> {
        self.store
            .write(|tx| self.cancel_stage(tx, stage_id, authorization_id))
    }
}
