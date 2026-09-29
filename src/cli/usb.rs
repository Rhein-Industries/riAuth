//! The server package has no terminal USB authenticator dependency. Legacy
//! commands fail before starting a ceremony; the standalone client owns USB.

use anyhow::{Result, bail};
use serde_json::Value;

const UNSUPPORTED: &str = "Terminal USB passkeys moved to riauthctl; install/build riauthctl with --features terminal-usb. For other authenticator clients use passkey start/finish";

pub(super) fn require_support() -> Result<()> {
    bail!(UNSUPPORTED)
}

pub(super) async fn perform(
    _issuer: &str,
    _challenge: Value,
    _registration: bool,
) -> Result<Value> {
    bail!(UNSUPPORTED)
}
