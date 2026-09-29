//! Retained temporary-access records, without entitlement issuance or approval.
pub use crate::pam_types::{AccessGrant, AccessRequest, NewAccessRequest};

pub fn extra_groups(
    tx: &crate::store::Tx<'_>,
    _user_id: &str,
    _now: u64,
) -> crate::error::Result<std::collections::BTreeSet<String>> {
    // Startup preflight rejects this bucket. Also fail closed for callers that
    // reach authorization through an offline Core path.
    if !tx.scan::<AccessGrant>("access_grants", None, 1)?.is_empty() {
        return Err(crate::error::Error::bad(
            "Temporary access requires the Platform build",
        ));
    }
    Ok(std::collections::BTreeSet::new())
}
