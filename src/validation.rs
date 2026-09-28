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
