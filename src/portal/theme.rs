//! Operator-trusted presentation assets. HTTP serves a bounded startup snapshot;
//! a request never selects a filesystem path. Themes cannot replace auth scripts.
use anyhow::{Context, Result, bail};
use std::{collections::BTreeMap, fs, io::Read, path::Path};

#[derive(Default)]
pub(crate) struct Theme {
    assets: BTreeMap<String, (&'static str, Vec<u8>)>,
}

impl Theme {
    pub(crate) fn load(config: &crate::config::Config) -> Result<Self> {
        let Some(dir) = config
            .portal_theme_dir
            .as_deref()
            .filter(|_| config.browser_ui)
        else {
            return Ok(Self::default());
        };
        if !fs::symlink_metadata(dir)?.file_type().is_dir() {
            bail!("portal_theme_dir must be a real directory, not a symlink");
        }
        let mut theme = Self::default();
        let mut total = 0;
        for entry in fs::read_dir(dir).with_context(|| format!("Read theme {}", dir.display()))? {
            let entry = entry?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("Theme filenames must be ASCII"))?;
            if theme.assets.len() >= 32
                || name.len() > 80
                || name.starts_with('.')
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            {
                bail!("Theme requires at most 32 simple ASCII filenames");
            }
            let (mime, limit) = match Path::new(&name).extension().and_then(|e| e.to_str()) {
                Some("css") if name == "theme.css" => ("text/css; charset=utf-8", 256 * 1024),
                Some("svg") => ("image/svg+xml", 256 * 1024),
                Some("png") => ("image/png", 1024 * 1024),
                Some("jpg" | "jpeg") => ("image/jpeg", 1024 * 1024),
                Some("webp") => ("image/webp", 1024 * 1024),
                Some("woff2") => ("font/woff2", 512 * 1024),
                _ => bail!(
                    "Unsupported theme asset {name}; use theme.css, SVG, PNG, JPEG, WebP or WOFF2"
                ),
            };
            if !entry.file_type()?.is_file() {
                bail!("Theme assets must be regular files, not symlinks or directories");
            }
            let mut options = fs::OpenOptions::new();
            options.read(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
            }
            let file = options.open(entry.path())?;
            let metadata = file.metadata()?;
            if !metadata.is_file() || metadata.len() > limit {
                bail!("Theme asset {name} is not a bounded regular file");
            }
            let mut bytes = Vec::new();
            file.take(limit + 1).read_to_end(&mut bytes)?;
            total += bytes.len();
            if bytes.len() as u64 > limit || total > 4 * 1024 * 1024 {
                bail!("Theme assets exceed the per-file or 4 MiB total limit");
            }
            if matches!(
                Path::new(&name).extension().and_then(|e| e.to_str()),
                Some("css" | "svg")
            ) {
                std::str::from_utf8(&bytes).context("Theme CSS and SVG must be UTF-8")?;
            }
            theme.assets.insert(name, (mime, bytes));
        }
        Ok(theme)
    }

    pub(crate) fn asset(&self, name: &str) -> Option<(&'static str, &[u8])> {
        self.assets
            .get(name)
            .map(|(mime, bytes)| (*mime, bytes.as_slice()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_secret_files_directories_and_oversized_assets() {
        let dir = tempfile::tempdir().unwrap();
        let configured = crate::config::Config {
            portal_theme_dir: Some(dir.path().into()),
            ..Default::default()
        };
        let secret = dir.path().join("credential.key");
        fs::write(&secret, "private").unwrap();
        assert!(Theme::load(&configured).is_err());
        fs::remove_file(secret).unwrap();
        fs::create_dir(dir.path().join("theme.css")).unwrap();
        assert!(Theme::load(&configured).is_err());
        fs::remove_dir(dir.path().join("theme.css")).unwrap();
        let file = fs::File::create(dir.path().join("theme.css")).unwrap();
        file.set_len(256 * 1024 + 1).unwrap();
        assert!(Theme::load(&configured).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinked_assets_and_directories() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        fs::create_dir(&real).unwrap();
        fs::write(dir.path().join("private"), "secret").unwrap();
        symlink(dir.path().join("private"), real.join("theme.css")).unwrap();
        let mut configured = crate::config::Config {
            portal_theme_dir: Some(real.clone()),
            ..Default::default()
        };
        assert!(Theme::load(&configured).is_err());
        fs::remove_file(real.join("theme.css")).unwrap();
        let link = dir.path().join("linked");
        symlink(&real, &link).unwrap();
        configured.portal_theme_dir = Some(link);
        assert!(Theme::load(&configured).is_err());
    }
}
