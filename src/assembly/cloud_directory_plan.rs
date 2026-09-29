//! Reviewed cloud-directory plan reads over concrete storage.

use crate::{
    cloud_directory::Plan,
    core::Core,
    error::{Error, Result},
};
use serde_json::{Value, json};

impl Core {
    pub(crate) fn cloud_plan_get_authorized(
        &self,
        token: &str,
        kind: &str,
        id: &str,
    ) -> Result<Value> {
        self.store.read(|tx| {
            let plan = tx
                .get::<Plan>("cloud_directory_plans", id)?
                .ok_or_else(|| Error::missing("Cloud directory plan not found"))?;
            if plan.kind != kind {
                return Err(Error::missing("Cloud directory plan not found"));
            }
            let actor = self.management(
                tx,
                token,
                "directory.read",
                &format!("{}/{}", plan.kind, plan.directory),
            )?;
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            Ok(json!(plan))
        })
    }
}
