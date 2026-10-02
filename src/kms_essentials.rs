//! Local JWT signing with persisted remote references rejected without Vault Transit.
pub use crate::kms_types::{RemoteKey, VaultSigner};

impl VaultSigner {
    pub fn validate(&self) -> crate::error::Result<()> {
        Err(crate::error::Error::bad(
            "External signing requires the Platform build",
        ))
    }
}

impl crate::core::Core {
    pub(crate) fn sign_jwt(
        &self,
        key: &crate::crypto::SigningKey,
        claims: &serde_json::Value,
        typ: &str,
    ) -> crate::error::Result<String> {
        let _timer = self.store.telemetry().signing.timer();
        let result = if key.remote.is_some() {
            Err(crate::error::Error::new(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "signer_unavailable",
                "Remote signing key requires the Platform build; no token was issued",
            ))
        } else {
            key.sign_type(claims, typ)
        };
        if result.is_err() {
            self.store
                .telemetry()
                .signing_errors
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if key.remote.is_some() {
                self.store.telemetry().remote_signing_failed(
                    crate::telemetry::RemoteSigningFailure::EditionUnsupported,
                );
            }
        }
        result
    }

    pub(crate) fn external_signing_key(
        &self,
        _name: &str,
    ) -> crate::error::Result<crate::crypto::SigningKey> {
        Err(crate::error::Error::bad(
            "External signing requires the Platform build",
        ))
    }
}
