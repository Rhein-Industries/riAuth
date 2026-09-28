//! Persisted connector configuration without cloud directory execution.
pub use crate::cloud_directory_types::{Attributes, EntraDirectory, WorkspaceDirectory};

impl WorkspaceDirectory {
    pub fn validate(&self) -> crate::error::Result<()> {
        Err(crate::error::Error::bad(
            "Workspace directory requires the Platform build",
        ))
    }
}

impl EntraDirectory {
    pub fn validate(&self) -> crate::error::Result<()> {
        Err(crate::error::Error::bad(
            "Entra directory requires the Platform build",
        ))
    }
}
