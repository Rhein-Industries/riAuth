use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use zeroize::{Zeroize, Zeroizing};

const MAX_SESSION_BYTES: u64 = 64 * 1024;

#[derive(Serialize, Deserialize)]
pub(crate) struct SavedSession {
    pub(crate) issuer: String,
    #[serde(default)]
    pub(crate) api_base: Option<String>,
    pub(crate) token: String,
    pub(crate) expires_at: u64,
}

#[derive(Deserialize)]
pub(crate) struct AgentCredential {
    issuer: String,
    pub(crate) token: String,
    expires_at: u64,
}

impl Drop for SavedSession {
    fn drop(&mut self) {
        self.token.zeroize();
    }
}

impl Drop for AgentCredential {
    fn drop(&mut self) {
        self.token.zeroize();
    }
}

pub(crate) fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub(crate) fn default_path() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .context("Use --session-file when HOME and XDG_CONFIG_HOME are unset")?;
    if !base.is_absolute() {
        bail!("XDG_CONFIG_HOME or HOME must be absolute; use --session-file");
    }
    Ok(base.join("riauthctl/session.json"))
}

pub(crate) fn read(path: &Path, issuer: &str) -> Result<SavedSession> {
    let text = read_private_text(path, MAX_SESSION_BYTES)
        .context("No valid private saved session; run `riauthctl login`")?;
    let saved: SavedSession =
        serde_json::from_str(&text).context("Saved session is invalid JSON")?;
    if saved.issuer != issuer {
        bail!(
            "Saved session belongs to another issuer; use a separate --session-file or log in here"
        );
    }
    if saved.expires_at <= now()? {
        bail!("Session expired; run `riauthctl login`");
    }
    if saved.token.is_empty() {
        bail!("Saved session is missing its token");
    }
    Ok(saved)
}

pub(crate) fn read_agent(path: &Path, api_base: &str) -> Result<AgentCredential> {
    let text =
        read_private_text(path, MAX_SESSION_BYTES).context("No valid private agent credential")?;
    let credential: AgentCredential =
        serde_json::from_str(&text).context("Agent credential is invalid JSON")?;
    if credential.issuer != api_base
        || credential.expires_at <= now()?
        || !credential.token.starts_with("ri_agent_")
    {
        bail!("Agent credential is invalid, expired, or belongs to another management issuer");
    }
    Ok(credential)
}

pub(crate) fn read_private_text(path: &Path, limit: u64) -> Result<Zeroizing<String>> {
    let link_metadata = fs::symlink_metadata(path).context("Cannot open private file")?;
    if link_metadata.file_type().is_symlink() {
        bail!("Private file must not be a symlink");
    }
    let file = fs::File::open(path).context("Cannot open private file")?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        bail!("Private file must be a bounded regular file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            bail!("Private file must have owner-only permissions (0600 or 0400)");
        }
    }
    let mut text = Zeroizing::new(String::new());
    file.take(limit + 1).read_to_string(&mut text)?;
    if text.len() as u64 > limit {
        bail!("Private file exceeds its size limit");
    }
    Ok(text)
}

pub(crate) fn write(path: &Path, saved: &SavedSession, private_default_dir: bool) -> Result<()> {
    write_private(path, &serde_json::to_vec(saved)?, true, private_default_dir)
}

pub(crate) fn write_private(
    path: &Path,
    data: &[u8],
    replace: bool,
    private_default_dir: bool,
) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let created = !parent.exists();
    if created {
        fs::create_dir_all(parent)?;
    }
    if created || private_default_dir {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
    }
    let temp = parent.join(format!(".riauthctl-{}.tmp", uuid::Uuid::new_v4()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> Result<()> {
        let mut file = options.open(&temp)?;
        file.write_all(data)?;
        file.sync_all()?;
        if replace {
            fs::rename(&temp, path)?;
        } else {
            fs::hard_link(&temp, path).context("Plan destination already exists")?;
        }
        Ok(())
    })();
    let _ = fs::remove_file(&temp);
    result
}
