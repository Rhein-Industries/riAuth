//! SAML source signing-key reads kept in the caller's storage transaction.

use crate::{
    core::{self, Core},
    crypto::{Keys, SigningKey},
    error::{Error, Result},
    source::{Source, saml::Settings},
    store::Tx,
};

/// A narrow key-read capability for SAML XML built inside an existing transaction.
pub(crate) struct SamlSigningKeyRead<'tx, 'db> {
    tx: &'tx Tx<'db>,
}

impl<'tx, 'db> SamlSigningKeyRead<'tx, 'db> {
    pub(crate) fn new(tx: &'tx Tx<'db>) -> Self {
        Self { tx }
    }

    pub(crate) fn key(&self, settings: &Settings) -> Result<SigningKey> {
        let keys = if settings.signing_key == "signing" {
            core::keys(self.tx)?
        } else {
            self.tx
                .get::<Keys>("key_domains", &settings.signing_key)?
                .ok_or_else(|| Error::bad("SAML source signing domain is missing"))?
        };
        let key = keys.active;
        if key.remote.is_some() || key.algorithm != "RS256" {
            return Err(Error::bad(
                "SAML source requires a local RS256 signing/decryption key",
            ));
        }
        let private =
            risaml::crypto::keys::load_private_key(&key.pem, None).map_err(Error::internal)?;
        let public = risaml::crypto::keys::load_certificate(&settings.sp_certificate_pem)
            .map_err(Error::internal)?;
        if private.to_spki_der().is_none() || private.to_spki_der() != public.to_spki_der() {
            return Err(Error::bad(
                "SAML source SP certificate does not match its signing domain",
            ));
        }
        Ok(key)
    }
}

impl Settings {
    pub(crate) fn key(&self, tx: &Tx<'_>) -> Result<SigningKey> {
        SamlSigningKeyRead::new(tx).key(self)
    }
}

impl Core {
    pub(crate) fn saml_source_callback_key(&self, settings: &Settings) -> Result<SigningKey> {
        self.store.read(|tx| settings.key(tx))
    }

    pub(crate) fn with_saml_source_metadata(
        &self,
        id: &str,
        render: impl FnOnce(&Source, &SamlSigningKeyRead<'_, '_>) -> Result<String>,
    ) -> Result<String> {
        self.store.read(|tx| {
            let source = super::source_enabled(tx, id)?;
            render(&source, &SamlSigningKeyRead::new(tx))
        })
    }
}
