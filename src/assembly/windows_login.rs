//! Windows device reads over concrete storage and live authorization state.

use crate::{
    core::Core,
    error::Result,
    identity::windows_credentials::{DEVICES, Device},
    model::User,
    windows_login::{invalid, offline_claims, secret_matches, view},
};
use serde_json::{Value, json};

impl Core {
    pub fn windows_devices(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let mut rows = Vec::new();
            for (_, device) in tx.list::<Device>(DEVICES)? {
                if actor.allows("device.enroll", &format!("device/{}", device.id)) {
                    rows.push(view(&device));
                }
            }
            rows.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
            Ok(json!(rows))
        })
    }

    pub fn windows_offline_verify(&self, device_secret: &str, ticket: &str) -> Result<Value> {
        let claims = offline_claims(device_secret, ticket)?;
        self.store.read(|tx| {
            let Some(device) = tx.get::<Device>(DEVICES, &claims.device_id)? else {
                return Err(invalid());
            };
            if !secret_matches(Some(&device), device_secret) || device.revoked {
                return Err(invalid());
            }
            if device.user_id != claims.user_id || device.username != claims.username {
                return Err(invalid());
            }
            let Some(user) = tx.get::<User>("users", &device.user_id)? else {
                return Err(invalid());
            };
            if !user.enabled || user.epoch != claims.epoch || user.username != claims.username {
                return Err(invalid());
            }
            Ok(json!({
                "active": true,
                "device_id": device.id,
                "username": user.username,
                "user_id": user.id,
                "epoch": user.epoch,
                "expires_at": claims.exp,
            }))
        })
    }
}
