//! Terminal USB authenticator operations stay private to the CLI.

use anyhow::{Result, bail};
use serde_json::Value;

const UNSUPPORTED: &str = "USB passkeys are unavailable in this build; rebuild with --features terminal-usb or use passkey start/finish with an authenticator client";

pub(super) fn require_support() -> Result<()> {
    if cfg!(feature = "terminal-usb") {
        Ok(())
    } else {
        bail!(UNSUPPORTED)
    }
}

#[cfg(not(feature = "terminal-usb"))]
pub(super) async fn perform(
    _issuer: &str,
    _challenge: Value,
    _registration: bool,
) -> Result<Value> {
    bail!(UNSUPPORTED)
}

/// USB operations use the same verified issuer origin as the HTTPS API client.
#[cfg(feature = "terminal-usb")]
pub(super) async fn perform(issuer: &str, challenge: Value, registration: bool) -> Result<Value> {
    use futures_util::StreamExt;
    use serde_json::json;
    use webauthn_authenticator_rs::{
        WebauthnAuthenticator,
        ctap2::CtapAuthenticator,
        transport::{TokenEvent, Transport},
        usb::USBTransport,
    };
    let uri = crate::config::validate_server_url(issuer)?;
    let origin = url::Url::parse(&uri.origin().ascii_serialization())?;
    let ui = webauthn_authenticator_rs::ui::Cli {};
    let transport = USBTransport::new().await?;
    let mut tokens = transport.watch().await?;
    eprintln!("Connect your FIDO2 authenticator. Touch it and enter its PIN when requested.");
    let token = tokio::time::timeout(std::time::Duration::from_secs(60), async {
        while let Some(event) = tokens.next().await {
            if let TokenEvent::Added(token) = event
                && let Some(authenticator) = CtapAuthenticator::new(token, &ui).await
            {
                return Ok::<_, anyhow::Error>(authenticator);
            }
        }
        anyhow::bail!("No FIDO2 authenticator is available")
    })
    .await??;
    let mut authenticator = WebauthnAuthenticator::new(token);
    // The CTAP implementation uses block_in_place to drive its async transport.
    if registration {
        Ok(json!(authenticator.do_registration(
            origin,
            serde_json::from_value(challenge)?
        )?))
    } else {
        Ok(json!(authenticator.do_authentication(
            origin,
            serde_json::from_value(challenge)?
        )?))
    }
}
