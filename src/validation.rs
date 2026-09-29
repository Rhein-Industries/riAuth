use crate::error::{Error, Result};

pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.len() > 64
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.@".contains(&c))
    {
        return Err(Error::bad(
            "Names must be 1–64 ASCII letters, digits, dots, hyphens, underscores or @",
        ));
    }
    Ok(())
}

pub(crate) fn validate_display(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 200 || name.chars().any(char::is_control) {
        return Err(Error::bad(
            "Display name must be 1–200 bytes without control characters",
        ));
    }
    Ok(())
}

pub(crate) fn validate_email(email: &str) -> Result<()> {
    if email.len() > 254
        || !email.contains('@')
        || email.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        return Err(Error::bad("Invalid email address"));
    }
    Ok(())
}
