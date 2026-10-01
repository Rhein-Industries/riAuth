//! SCIM 2.0 user/group provisioning. Each authenticated operator owns its provisioned records.
use serde::Deserialize;

pub use crate::scim_shared::{GROUP, USER, response};

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Query {
    pub start_index: Option<usize>,
    pub count: Option<usize>,
    pub filter: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub attributes: Option<String>,
    pub excluded_attributes: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectionQuery {
    pub attributes: Option<String>,
    pub excluded_attributes: Option<String>,
}

// Compatibility paths for metadata, storage-transition and parser-fuzz callers.
pub use crate::assembly::scim_metadata as metadata;
pub(crate) use crate::assembly::scim_record_transition as record_transition;
#[cfg(feature = "fuzzing")]
pub(crate) use crate::assembly::{
    scim_fuzz_filter as fuzz_filter, scim_fuzz_resource as fuzz_resource,
};
