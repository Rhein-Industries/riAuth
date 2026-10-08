//! Trusted deployment files, read once before storage is opened. Requests only
//! select keys from this immutable snapshot; they never join filesystem paths.
use crate::{
    api::App,
    error::{Error, Result},
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, Uri},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Read,
    path::{Path as FsPath, PathBuf},
};

const MAX_ENTRIES: usize = 256;
const MAX_FILES: usize = 128;
const MAX_DEPTH: usize = 4;
const MAX_FILE_BYTES: usize = 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Frontend {
    /// Trusted operator files, relative to the configuration file when loaded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme_dir: Option<PathBuf>,
}

impl Frontend {
    pub fn is_default(&self) -> bool {
        self.theme_dir.is_none()
    }

    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        if self
            .theme_dir
            .as_ref()
            .is_some_and(|path| path.as_os_str().is_empty())
        {
            anyhow::bail!("frontend.theme_dir must not be empty");
        }
        Ok(())
    }
}

struct Asset {
    bytes: Vec<u8>,
    mime: &'static str,
}

#[derive(Default)]
pub(crate) struct Theme {
    selected: Frontend,
    pages: BTreeMap<String, String>,
    assets: BTreeMap<String, Asset>,
}

impl Theme {
    pub(crate) fn load(frontend: &Frontend) -> Result<Self> {
        frontend.validate().map_err(Error::internal)?;
        let mut theme = Self {
            selected: frontend.clone(),
            ..Self::default()
        };
        if let Some(root) = &frontend.theme_dir {
            if !supported_file_open() {
                return Err(Error::bad(
                    "Frontend themes require supported no-follow file opens",
                ));
            }
            let metadata = fs::symlink_metadata(root).map_err(|_| invalid())?;
            if is_link(&metadata) || !metadata.is_dir() {
                return Err(invalid());
            }
            let root = fs::canonicalize(root).map_err(|_| invalid())?;
            let mut budget = Budget::default();
            theme.read_directory(&root, "", 0, &mut budget)?;
        }
        Ok(theme)
    }

    /// Internal setup/Core handoffs must carry the snapshot for this exact config.
    pub(crate) fn require_config(&self, frontend: &Frontend) -> Result<()> {
        if &self.selected != frontend {
            return Err(Error::bad(
                "Frontend configuration requires a fresh startup snapshot",
            ));
        }
        Ok(())
    }

    fn read_directory(
        &mut self,
        root: &FsPath,
        prefix: &str,
        depth: usize,
        budget: &mut Budget,
    ) -> Result<()> {
        for entry in fs::read_dir(root).map_err(|_| invalid())? {
            budget.entries = budget.entries.checked_add(1).ok_or_else(invalid)?;
            if budget.entries > MAX_ENTRIES {
                return Err(invalid());
            }
            let entry = entry.map_err(|_| invalid())?;
            let name = entry.file_name().into_string().map_err(|_| invalid())?;
            let key = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            if !valid_key(&key) || !budget.keys.insert(key.to_ascii_lowercase()) {
                return Err(invalid());
            }
            let path = entry.path();
            let before = fs::symlink_metadata(&path).map_err(|_| invalid())?;
            if is_link(&before) {
                return Err(invalid());
            }
            if before.is_dir() {
                if depth >= MAX_DEPTH || !allowed_directory(&key) {
                    return Err(invalid());
                }
                self.read_directory(&path, &key, depth + 1, budget)?;
                continue;
            }
            if !before.is_file() {
                return Err(invalid());
            }
            budget.files = budget.files.checked_add(1).ok_or_else(invalid)?;
            if budget.files > MAX_FILES {
                return Err(invalid());
            }
            let (page, mime) = classify(&key).ok_or_else(invalid)?;
            let remaining = MAX_TOTAL_BYTES
                .checked_sub(budget.bytes)
                .ok_or_else(invalid)?;
            let limit = MAX_FILE_BYTES.min(remaining);
            if before.len() > limit as u64 {
                return Err(invalid());
            }
            let mut file = open_regular(&path, &before)?;
            let opened = file.metadata().map_err(|_| invalid())?;
            let mut bytes = Vec::new();
            (&mut file)
                .take(limit as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| invalid())?;
            let finished = file.metadata().map_err(|_| invalid())?;
            let after = fs::symlink_metadata(&path).map_err(|_| invalid())?;
            if bytes.len() > limit
                || bytes.len() as u64 != opened.len()
                || is_link(&after)
                || !same_file(&opened, &finished)
                || !same_file(&opened, &after)
            {
                return Err(invalid());
            }
            budget.bytes = budget.bytes.checked_add(bytes.len()).ok_or_else(invalid)?;
            if page {
                self.pages.insert(
                    key[6..].to_owned(),
                    String::from_utf8(bytes).map_err(|_| invalid())?,
                );
            } else {
                if matches!(
                    mime,
                    "text/css; charset=utf-8"
                        | "text/javascript; charset=utf-8"
                        | "image/svg+xml; charset=utf-8"
                ) {
                    std::str::from_utf8(&bytes).map_err(|_| invalid())?;
                }
                self.assets.insert(key, Asset { bytes, mime });
            }
        }
        Ok(())
    }

    pub(crate) fn page(&self, name: &str) -> &str {
        self.pages
            .get(name)
            .map(String::as_str)
            .unwrap_or_else(|| embedded_page(name))
    }

    pub(crate) fn builtin(&self, name: &str) -> Response {
        if let Some(asset) = self.assets.get(&format!("assets/{name}")) {
            return response(asset.mime, Body::from(asset.bytes.clone()));
        }
        match embedded_asset(name) {
            Some((mime, bytes)) => response(mime, Body::from(bytes)),
            None => Error::missing("Frontend asset not found").into_response(),
        }
    }

    pub(crate) fn additional(&self, key: &str) -> Response {
        if valid_key(key)
            && let Some(asset) = self.assets.get(&format!("theme-assets/{key}"))
        {
            return response(asset.mime, Body::from(asset.bytes.clone()));
        }
        Error::missing("Frontend asset not found").into_response()
    }
}

#[derive(Default)]
struct Budget {
    entries: usize,
    files: usize,
    bytes: usize,
    keys: BTreeSet<String>,
}

fn invalid() -> Error {
    Error::bad(
        "Invalid frontend theme: require bounded regular files, known pages/assets and canonical relative names",
    )
}

fn supported_file_open() -> bool {
    cfg!(any(
        target_os = "macos",
        windows,
        all(
            target_os = "linux",
            any(target_env = "gnu", target_env = "musl"),
            any(target_arch = "x86_64", target_arch = "aarch64")
        )
    ))
}

fn open_regular(path: &FsPath, before: &fs::Metadata) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    // libc 0.2.189: unix/bsd/mod.rs (macOS), linux/{gnu,musl}/b64/{x86_64,aarch64}.
    // AArch64's Linux O_NOFOLLOW differs from x86_64. No new dependency or unsafe FFI.
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(0x20000 | 2048); // O_NOFOLLOW | O_NONBLOCK
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(0x8000 | 2048); // O_NOFOLLOW | O_NONBLOCK
    }
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(0x100 | 0x4); // O_NOFOLLOW | O_NONBLOCK
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    if !supported_file_open() {
        return Err(invalid());
    }
    let file = options.open(path).map_err(|_| invalid())?;
    let opened = file.metadata().map_err(|_| invalid())?;
    // Never read a FIFO, device, socket or reparse point, including a replacement
    // between the directory metadata check and this nonblocking/no-follow open.
    if !opened.is_file() || is_link(&opened) || !same_file(before, &opened) {
        return Err(invalid());
    }
    Ok(file)
}

fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true;
        } // FILE_ATTRIBUTE_REPARSE_POINT
    }
    metadata.file_type().is_symlink()
}

fn same_file(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != after.dev() || before.ino() != after.ino() {
            return false;
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if before.creation_time() != after.creation_time()
            || before.file_attributes() != after.file_attributes()
        {
            return false;
        }
    }
    before.len() == after.len() && before.modified().ok() == after.modified().ok()
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 240
        && key.split('/').all(|part| {
            let stem = part
                .split('.')
                .next()
                .unwrap_or_default()
                .to_ascii_uppercase();
            let device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || (stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'));
            !part.is_empty()
                && !device
                && part.len() <= 64
                && !part.starts_with('.')
                && !part.ends_with('.')
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        })
}

fn allowed_directory(key: &str) -> bool {
    matches!(key, "pages" | "assets" | "theme-assets") || key.starts_with("theme-assets/")
}

fn classify(key: &str) -> Option<(bool, &'static str)> {
    if let Some(page) = key.strip_prefix("pages/") {
        return known_page(page).then_some((true, "text/html; charset=utf-8"));
    }
    if let Some(name) = key.strip_prefix("assets/") {
        return embedded_asset(name).map(|(mime, _)| (false, mime));
    }
    let name = key.strip_prefix("theme-assets/")?;
    let (_, extension) = name.rsplit_once('.')?;
    Some((
        false,
        match extension {
            "css" => "text/css; charset=utf-8",
            "js" => "text/javascript; charset=utf-8",
            "svg" => "image/svg+xml; charset=utf-8",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "ico" => "image/x-icon",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "otf" => "font/otf",
            _ => return None,
        },
    ))
}

fn known_page(name: &str) -> bool {
    matches!(
        name,
        "apps.html"
            | "signin.html"
            | "admin.html"
            | "account.html"
            | "security.html"
            | "agents.html"
            | "device.html"
            | "source-stage.html"
            | "setup.html"
            | "sources.html"
            | "events.html"
            | "access-review.html"
    )
}

fn embedded_page(name: &str) -> &'static str {
    match name {
        "apps.html" => include_str!("index.html"),
        "signin.html" => include_str!("signin.html"),
        "admin.html" => include_str!("admin.html"),
        "account.html" => include_str!("account.html"),
        "security.html" => include_str!("self_service/security.html"),
        "agents.html" => include_str!("self_service/agents.html"),
        "device.html" => include_str!("device.html"),
        "source-stage.html" => include_str!("source-stage.html"),
        "setup.html" => include_str!("setup.html"),
        "sources.html" => include_str!("sources.html"),
        "events.html" => include_str!("events.html"),
        "access-review.html" => include_str!("access-review.html"),
        _ => "",
    }
}

fn embedded_asset(name: &str) -> Option<(&'static str, &'static [u8])> {
    let (mime, bytes): (_, &[u8]) = match name {
        "app.css" => ("text/css; charset=utf-8", include_bytes!("app.css")),
        "admin.css" => ("text/css; charset=utf-8", include_bytes!("admin.css")),
        "security.css" => (
            "text/css; charset=utf-8",
            include_bytes!("self_service/security.css"),
        ),
        "agents.css" => (
            "text/css; charset=utf-8",
            include_bytes!("self_service/agents.css"),
        ),
        "map.css" => ("text/css; charset=utf-8", include_bytes!("map.css")),
        "access-review.css" => (
            "text/css; charset=utf-8",
            include_bytes!("access-review.css"),
        ),
        "riauth-mark.svg" => (
            "image/svg+xml; charset=utf-8",
            include_bytes!("../../assets/riauth-mark.svg"),
        ),
        "appearance.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("appearance.js"),
        ),
        "app.js" => ("text/javascript; charset=utf-8", include_bytes!("app.js")),
        "admin.js" => ("text/javascript; charset=utf-8", include_bytes!("admin.js")),
        "auth.js" => ("text/javascript; charset=utf-8", include_bytes!("auth.js")),
        "account.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("account.js"),
        ),
        "capabilities.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("capabilities.js"),
        ),
        "signin.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("signin.js"),
        ),
        "setup.js" => ("text/javascript; charset=utf-8", include_bytes!("setup.js")),
        "device.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("device.js"),
        ),
        "sources.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("sources.js"),
        ),
        "source-login.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("source-login.js"),
        ),
        "source-stage.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("source-stage.js"),
        ),
        "security.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("self_service/security.js"),
        ),
        "agents.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("self_service/agents.js"),
        ),
        "access-review.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("access-review.js"),
        ),
        "map.js" => ("text/javascript; charset=utf-8", include_bytes!("map.js")),
        "grant-review.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("grant-review.js"),
        ),
        "membership-review.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("membership-review.js"),
        ),
        "client-creation-review.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("client-creation-review.js"),
        ),
        "client-policy-review.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("client-policy-review.js"),
        ),
        "client-status-review.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("client-status-review.js"),
        ),
        "client-endpoint-review.js" => (
            "text/javascript; charset=utf-8",
            include_bytes!("client-endpoint-review.js"),
        ),
        _ => return None,
    };
    Some((mime, bytes))
}

fn response(mime: &'static str, body: Body) -> Response {
    let mut response = body.into_response();
    response
        .headers_mut()
        .insert("content-type", HeaderValue::from_static(mime));
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response
}

pub(crate) async fn builtin(State(app): State<App>, uri: Uri) -> Response {
    app.core
        .runtime
        .frontend
        .builtin(uri.path().rsplit('/').next().unwrap_or_default())
}

/// The configured issuer prefix may be percent-encoded; theme keys may not.
/// Axum already selected the issuer route, so compare only its raw key suffix.
pub(crate) fn canonical_asset_path(uri: &Uri, key: &str) -> bool {
    valid_key(key)
        && uri
            .path()
            .strip_suffix(key)
            .is_some_and(|prefix| prefix.ends_with("/portal/theme-assets/"))
}

pub(crate) async fn additional(
    State(app): State<App>,
    Path(key): Path<String>,
    uri: Uri,
) -> Response {
    if !canonical_asset_path(&uri, &key) {
        return Error::missing("Frontend asset not found").into_response();
    }
    app.core.runtime.frontend.additional(&key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &FsPath, key: &str, bytes: &[u8]) {
        let path = root.join(key);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn load(root: &FsPath) -> Result<Theme> {
        Theme::load(&Frontend {
            theme_dir: Some(root.into()),
        })
    }

    #[test]
    fn default_and_empty_theme_use_all_embedded_templates() {
        let root = tempfile::tempdir().unwrap();
        let empty = load(root.path()).unwrap();
        let default = Theme::load(&Frontend::default()).unwrap();
        for name in [
            "apps.html",
            "signin.html",
            "admin.html",
            "account.html",
            "security.html",
            "agents.html",
            "device.html",
            "source-stage.html",
            "setup.html",
            "sources.html",
            "events.html",
            "access-review.html",
        ] {
            assert!(!embedded_page(name).is_empty());
            assert_eq!(empty.page(name), embedded_page(name));
            assert_eq!(default.page(name), embedded_page(name));
        }
        assert!(empty.assets.is_empty());
        assert!(default.assets.is_empty());
    }

    #[test]
    fn each_known_page_override_retains_explicit_placeholders() {
        let root = tempfile::tempdir().unwrap();
        for name in [
            "apps.html",
            "signin.html",
            "admin.html",
            "account.html",
            "security.html",
            "agents.html",
            "device.html",
            "source-stage.html",
            "setup.html",
            "sources.html",
            "events.html",
            "access-review.html",
        ] {
            let template = format!(
                "<main>{name} __BASE__ __CODE__ __COMMAND__ __AUTHORIZATION__ __STAGE__ __RESUME__ __CANCEL__ __CONTINUE__</main>"
            );
            write(root.path(), &format!("pages/{name}"), template.as_bytes());
        }
        let theme = load(root.path()).unwrap();
        for (name, body) in &theme.pages {
            assert_eq!(theme.page(name), body);
            assert!(body.contains("__BASE__"));
            assert!(body.contains("__CODE__"));
            assert!(body.contains("__STAGE__"));
        }
        assert_eq!(theme.pages.len(), 12);
    }

    #[test]
    fn frontend_default_serde_and_config_relative_path() {
        let mut config = crate::config::Config::default();
        let original = serde_json::to_value(&config).unwrap();
        assert!(original.get("frontend").is_none());
        assert!(!toml::to_string(&config).unwrap().contains("[frontend]"));
        let decoded: crate::config::Config = serde_json::from_value(original.clone()).unwrap();
        assert!(decoded.frontend.is_default());
        let mut explicit = original;
        explicit["frontend"] = serde_json::json!({});
        let decoded: crate::config::Config = serde_json::from_value(explicit).unwrap();
        assert!(
            serde_json::to_value(decoded)
                .unwrap()
                .get("frontend")
                .is_none()
        );
        assert!(
            serde_json::from_str::<Frontend>(r#"{"theme_dir":"theme","unknown":true}"#).is_err()
        );
        config.frontend.theme_dir = Some(PathBuf::new());
        assert!(config.validate().is_err());
        let root = tempfile::tempdir().unwrap();
        config.frontend.theme_dir = Some("theme".into());
        let path = root.path().join("riauth.toml");
        fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
        let decoded = crate::config::Config::load(&path).unwrap();
        assert_eq!(decoded.frontend.theme_dir, Some(root.path().join("theme")));
    }

    #[test]
    fn invalid_names_types_and_utf8_refuse_the_whole_theme() {
        for (key, bytes) in [
            ("pages/unknown.html", b"unknown".as_slice()),
            ("assets/unknown.js", b"unknown".as_slice()),
            ("theme-assets/file.html", b"unknown".as_slice()),
            ("theme-assets/file.py", b"unknown".as_slice()),
            ("theme-assets/file%2ejs", b"unknown".as_slice()),
            ("theme-assets/.hidden.js", b"unknown".as_slice()),
            ("pages/apps.html", b"\xff".as_slice()),
            ("assets/app.css", b"\xff".as_slice()),
            ("theme-assets/test.js", b"\xff".as_slice()),
            ("theme-assets/test.svg", b"\xff".as_slice()),
        ] {
            let root = tempfile::tempdir().unwrap();
            write(root.path(), "assets/auth.js", b"// valid first asset");
            write(root.path(), key, bytes);
            assert!(load(root.path()).is_err(), "{key}");
        }
        for key in [
            "../logo.svg",
            "a/../logo.svg",
            "a//logo.svg",
            "/logo.svg",
            "//host/logo.svg",
            "https://host/logo.svg",
            "a\\logo.svg",
            "a?x.svg",
            "a#x.svg",
            "a%2fb.svg",
            "a\n.svg",
            "NUL.png",
            "a/COM1.js",
        ] {
            assert!(!valid_key(key), "{key}");
        }
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "theme-assets/Logo.svg", b"<svg/>");
        write(root.path(), "theme-assets/logo.svg", b"<svg/>");
        // Case-insensitive filesystems may keep one file; the normalized key set
        // still rejects a second equivalent spelling on case-sensitive hosts.
        let mut budget = Budget::default();
        assert!(
            budget
                .keys
                .insert("theme-assets/Logo.svg".to_ascii_lowercase())
        );
        assert!(
            !budget
                .keys
                .insert("theme-assets/logo.svg".to_ascii_lowercase())
        );
        if fs::read_dir(root.path().join("theme-assets"))
            .unwrap()
            .count()
            == 2
        {
            assert!(load(root.path()).is_err());
        }
    }

    #[test]
    fn file_aggregate_depth_and_count_bounds_are_enforced() {
        let root = tempfile::tempdir().unwrap();
        write(
            root.path(),
            "theme-assets/large.png",
            &vec![0; MAX_FILE_BYTES + 1],
        );
        assert!(load(root.path()).is_err());
        let root = tempfile::tempdir().unwrap();
        for n in 0..8 {
            write(
                root.path(),
                &format!("theme-assets/{n}.png"),
                &vec![0; MAX_FILE_BYTES],
            );
        }
        assert!(load(root.path()).is_ok());
        write(root.path(), "theme-assets/extra.png", &[0]);
        assert!(load(root.path()).is_err());
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "theme-assets/a/b/c/image.png", &[0]);
        assert!(load(root.path()).is_ok());
        write(root.path(), "theme-assets/a/b/c/d/image.png", &[0]);
        assert!(load(root.path()).is_err());
        let root = tempfile::tempdir().unwrap();
        for n in 0..MAX_FILES {
            write(root.path(), &format!("theme-assets/{n}.png"), &[0]);
        }
        assert!(load(root.path()).is_ok());
        write(root.path(), "theme-assets/extra.png", &[0]);
        assert!(load(root.path()).is_err());
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("theme-assets")).unwrap();
        for n in 0..MAX_ENTRIES {
            fs::create_dir(root.path().join(format!("theme-assets/dir{n}"))).unwrap();
        }
        assert!(load(root.path()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_roots_directories_and_files_are_rejected() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let linked = root.path().join("linked");
        let real = root.path().join("real");
        fs::create_dir(&real).unwrap();
        symlink(&real, &linked).unwrap();
        assert!(load(&linked).is_err());
        symlink(&real, real.join("theme-assets")).unwrap();
        assert!(load(&real).is_err());
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "theme-assets/real.png", &[0]);
        symlink(
            root.path().join("theme-assets/real.png"),
            root.path().join("theme-assets/link.png"),
        )
        .unwrap();
        assert!(load(root.path()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn replaced_file_symlink_and_local_socket_refuse_before_content_read() {
        use std::os::unix::{fs::symlink, net::UnixListener};
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("checked.svg");
        fs::write(&path, b"<svg/>").unwrap();
        let before = fs::symlink_metadata(&path).unwrap();
        fs::rename(&path, root.path().join("original.svg")).unwrap();
        symlink(root.path().join("original.svg"), &path).unwrap();
        assert!(open_regular(&path, &before).is_err());
        let socket = root.path().join("theme-assets/s.svg");
        fs::create_dir(socket.parent().unwrap()).unwrap();
        let _listener = UnixListener::bind(&socket).unwrap();
        let metadata = fs::symlink_metadata(&socket).unwrap();
        assert!(!metadata.is_file());
        assert!(open_regular(&socket, &before).is_err());
        // Remove unrelated root files by using the theme-assets parent as a
        // complete tree with one special file in its allowed child directory.
        let theme_root = root.path().join("socket-theme");
        fs::create_dir_all(theme_root.join("theme-assets")).unwrap();
        fs::rename(&socket, theme_root.join("theme-assets/s.svg")).unwrap();
        assert!(load(&theme_root).is_err());
    }

    #[test]
    fn snapshot_survives_file_changes_and_requires_matching_handoff() {
        let root = tempfile::tempdir().unwrap();
        write(root.path(), "pages/apps.html", b"first __BASE__");
        write(root.path(), "theme-assets/logo.svg", b"<svg/>");
        let theme = load(root.path()).unwrap();
        write(root.path(), "pages/apps.html", b"second");
        assert_eq!(theme.page("apps.html"), "first __BASE__");
        assert!(
            theme
                .require_config(&Frontend {
                    theme_dir: Some(root.path().into())
                })
                .is_ok()
        );
        assert!(theme.require_config(&Frontend::default()).is_err());
        assert!(
            Theme::default()
                .require_config(&Frontend {
                    theme_dir: Some(root.path().into())
                })
                .is_err()
        );
    }
}
