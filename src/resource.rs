//! RFC 8707: one explicitly registered resource audience per token.
use crate::{
    error::{Error, Result},
    model::{Client, Grant},
};
use std::collections::BTreeSet;

pub fn validate_uri(value: &str) -> Result<()> {
    let uri = url::Url::parse(value).map_err(|_| invalid())?;
    if value.len() > 2048
        || uri.fragment().is_some()
        || !uri.username().is_empty()
        || uri.password().is_some()
        || uri.as_str() != value
    {
        return Err(invalid());
    }
    Ok(())
}
pub fn validate(client: &Client, resource: Option<&str>, scopes: &BTreeSet<String>) -> Result<()> {
    if let Some(resource) = resource {
        validate_uri(resource)?;
        let allowed = client
            .settings
            .resources
            .get(resource)
            .ok_or_else(invalid)?;
        if !scopes.is_subset(allowed) {
            return Err(Error::oauth(
                "invalid_scope",
                "Scopes exceed the registered resource policy",
            ));
        }
    }
    Ok(())
}
pub fn bind(
    client: &Client,
    grant: &mut Grant,
    requested: Option<&str>,
    authorized: Option<&str>,
) -> Result<()> {
    if requested.is_some() && requested != authorized {
        return Err(invalid());
    }
    validate(client, authorized, &grant.scopes)?;
    grant.resource = authorized.map(String::from);
    Ok(())
}
pub fn audience(grant: &Grant) -> &str {
    grant.resource.as_deref().unwrap_or(&grant.client_id)
}
pub fn invalid() -> Error {
    Error::oauth(
        "invalid_target",
        "Use one registered resource authorized for this grant",
    )
}
