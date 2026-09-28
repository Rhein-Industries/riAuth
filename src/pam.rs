//! Temporary group entitlements. Durable `Group.members` is never changed.
use crate::{
    error::{Error, Result},
    validation::validate_name,
};

pub use crate::assembly::{pam_cleanup as cleanup, pam_extra_groups as extra_groups};
pub use crate::pam_types::{AccessGrant, AccessRequest, NewAccessRequest};

fn validate_reason(reason: &str) -> Result<()> {
    let length = reason.chars().count();
    if !(1..=280).contains(&length) || reason.chars().any(char::is_control) {
        return Err(Error::bad(
            "Reason must be 1–280 characters without control characters",
        ));
    }
    let lower = reason.to_ascii_lowercase();
    let blocked = [
        "password",
        "secret",
        "bearer ",
        "private key",
        "private_key",
        "begin ",
    ];
    if blocked.iter().any(|needle| lower.contains(needle))
        || [
            "ri_session_",
            "ri_agent_",
            "ri_client_",
            "ri_mail_",
            "ri_recovery_",
            "ri_portal_",
        ]
        .iter()
        .any(|needle| reason.contains(needle))
    {
        return Err(Error::bad("Reason must not contain secrets"));
    }
    Ok(())
}

pub(crate) fn validate_request(input: &NewAccessRequest) -> Result<()> {
    validate_name(&input.group)?;
    validate_reason(&input.reason)?;
    if !(60..=86_400).contains(&input.ttl) {
        return Err(Error::bad("Duration must be 60–86400 seconds"));
    }
    Ok(())
}
