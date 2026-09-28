//! Sign authorization responses with keys read from the caller's transaction.

use crate::{core::Core, error::Result, keyring, model::Client, response, store::Tx};

impl Core {
    pub(crate) fn secure_authorization_response(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        mode: Option<&str>,
        callback: String,
    ) -> Result<String> {
        response::secure_authorization_response(
            &self.config.issuer,
            client,
            mode,
            callback,
            |claims| {
                self.sign_jwt(
                    &keyring::for_client(tx, client)?.active,
                    claims,
                    "oauth-authz-resp+jwt",
                )
            },
        )
    }
}
