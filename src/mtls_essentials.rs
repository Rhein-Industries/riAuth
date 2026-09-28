//! Shared client-certificate configuration and TLS request shape; no login adapter.
pub use crate::mtls_config::{ClientCertAuth, ClientCertMode, TlsClientCerts};

pub(crate) struct Material;

impl ClientCertAuth {
    pub(crate) fn material(&self) -> crate::error::Result<Material> {
        Err(crate::error::Error::bad(
            "Client-certificate login requires the Platform build",
        ))
    }

    pub(crate) fn verifier(
        &self,
        _material: &Material,
    ) -> crate::error::Result<std::sync::Arc<dyn rustls::server::danger::ClientCertVerifier>> {
        Err(crate::error::Error::bad(
            "Client-certificate login requires the Platform build",
        ))
    }
}

pub(crate) fn validate_identity(
    _tx: &crate::store::Tx<'_>,
    identity: &crate::model::Identity,
) -> crate::error::Result<()> {
    if identity.amr.iter().any(|method| method == "cert") {
        return Err(crate::error::Error::unauthorized());
    }
    Ok(())
}

pub(crate) fn cleanup(_tx: &crate::store::Tx<'_>, _at: u64) -> crate::error::Result<()> {
    Ok(())
}
