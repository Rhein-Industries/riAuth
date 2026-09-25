use crate::{
    crypto,
    error::{Error, Result},
    model::User,
};
use serde::{Deserialize, Serialize};

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct TotpSettings {
    pub algorithm: String,
    pub digits: usize,
    pub period: u64,
}
impl Default for TotpSettings {
    fn default() -> Self {
        Self {
            algorithm: "SHA1".into(),
            digits: 6,
            period: 30,
        }
    }
}
impl TotpSettings {
    pub fn validate(&self) -> Result<()> {
        if !["SHA1", "SHA256", "SHA512"].contains(&self.algorithm.as_str())
            || ![6, 8].contains(&self.digits)
            || !(15..=120).contains(&self.period)
        {
            return Err(Error::bad(
                "TOTP requires SHA1/SHA256/SHA512, 6/8 digits and a 15–120 second period",
            ));
        }
        Ok(())
    }
}
#[derive(schemars::JsonSchema, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TotpImport {
    pub secret: String,
    /// base32 or Authentik's hexadecimal key format
    pub encoding: String,
    #[serde(default)]
    pub settings: TotpSettings,
    pub last_used_step: Option<u64>,
}
pub fn import(user: &mut User, encoded: &str) -> Result<()> {
    let input: TotpImport = serde_json::from_str(encoded)
        .map_err(|_| Error::bad("TOTP credential reference must contain TotpImport JSON"))?;
    input.settings.validate()?;
    let secret = zeroize::Zeroizing::new(input.secret);
    let bytes = match input.encoding.as_str() {
        "base32" => totp_rs::Secret::Encoded(secret.to_string())
            .to_bytes()
            .map_err(|_| Error::bad("Invalid base32 TOTP secret"))?,
        "hex" => {
            if !secret.len().is_multiple_of(2)
                || secret.len() > 128
                || !secret.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err(Error::bad("Invalid hexadecimal TOTP secret"));
            }
            (0..secret.len())
                .step_by(2)
                .map(|i| {
                    u8::from_str_radix(&secret[i..i + 2], 16)
                        .map_err(|_| Error::bad("Invalid hexadecimal TOTP secret"))
                })
                .collect::<Result<Vec<_>>>()?
        }
        _ => return Err(Error::bad("TOTP secret encoding must be base32 or hex")),
    };
    if !(16..=64).contains(&bytes.len()) {
        return Err(Error::bad("TOTP secrets must contain 16–64 bytes"));
    }
    let current = crypto::now() / input.settings.period;
    if input.last_used_step.is_some_and(|s| s > current + 1) {
        return Err(Error::bad("TOTP last-used step is in the future"));
    }
    user.totp_secret = Some(totp_rs::Secret::Raw(bytes).to_encoded().to_string());
    user.totp_settings = input.settings;
    user.totp_pending = None;
    // Never accept a code that could have been used before the cutover.
    user.totp_last_step = Some(input.last_used_step.unwrap_or(0).max(current));
    user.recovery_codes.clear();
    Ok(())
}
