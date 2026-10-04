//! macOS Seatbelt launch and fail-closed native entry checks.
//!
//! No unsafe hooks run after fork. Other operating systems have no supported
//! isolation backend and refuse before starting a process.

use super::Denial;
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "macos")]
const PROFILE: &str = include_str!("guest-macos.sb");

pub(super) fn command(program: &Path) -> Result<Command, Denial> {
    #[cfg(target_os = "macos")]
    {
        let program = program.canonicalize().map_err(|_| Denial::Failed)?;
        let path = program.to_str().ok_or(Denial::Failed)?;
        let mut command = Command::new("/usr/bin/sandbox-exec");
        // Parameter values are argv, never interpolated into SBPL or a shell.
        command
            .arg("-p")
            .arg(PROFILE)
            .arg("-D")
            .arg(format!("GUEST={path}"));
        // This server's LDAP dependency can link Homebrew OpenSSL on macOS.
        // Resolve only these two conventional library files; no directory or
        // environment-selected dynamic-library read is granted.
        #[cfg(target_arch = "aarch64")]
        let openssl = Path::new("/opt/homebrew/opt/openssl@3/lib");
        #[cfg(not(target_arch = "aarch64"))]
        let openssl = Path::new("/usr/local/opt/openssl@3/lib");
        for (key, library) in [("SSL", "libssl.3.dylib"), ("CRYPTO", "libcrypto.3.dylib")] {
            let library = openssl.join(library);
            let library = library.canonicalize().unwrap_or(library);
            let library = library.to_str().ok_or(Denial::Failed)?;
            command.arg("-D").arg(format!("{key}={library}"));
        }
        command.arg(program);
        Ok(command)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = program;
        Err(Denial::ExternalRuntimeRequired)
    }
}

/// Check only supported-host program/backend metadata; never launch or probe.
pub(super) fn configured_available(program: &Path) -> Result<(), Denial> {
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [program, Path::new("/usr/bin/sandbox-exec")] {
            let path = path.canonicalize().map_err(|_| Denial::Failed)?;
            if path.to_str().is_none() {
                return Err(Denial::Failed);
            }
            let metadata = path.metadata().map_err(|_| Denial::Failed)?;
            if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
                return Err(Denial::Failed);
            }
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = program;
        Err(Denial::ExternalRuntimeRequired)
    }
}

pub(super) fn check_entry() -> Result<(), Denial> {
    #[cfg(target_os = "macos")]
    {
        check_descriptors()?;
        // Refuse the internal argv entry when invoked outside confinement.
        // These native probes do not read file contents or contact a peer.
        if !matches!(std::fs::File::open("/etc/passwd"), Err(error)
            if error.kind() == std::io::ErrorKind::PermissionDenied)
            || !matches!(std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)), Err(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied)
        {
            return Err(Denial::Failed);
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(Denial::ExternalRuntimeRequired)
    }
}

#[cfg(target_os = "macos")]
fn check_descriptors() -> Result<(), Denial> {
    // This entry runs before request parsing, Wasmi, or runtime/thread startup.
    // /dev/fd enumerates this process's open descriptors. Collect and close the
    // directory iterator BEFORE checking its entries: its own fd is then gone.
    // Refuse extras instead of closing raw fds through an unsafe ownership hook.
    let mut descriptors = Vec::new();
    for entry in std::fs::read_dir("/dev/fd").map_err(|_| Denial::Failed)? {
        if descriptors.len() >= 4096 {
            return Err(Denial::Failed);
        }
        let entry = entry.map_err(|_| Denial::Failed)?;
        let fd = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
            .ok_or(Denial::Failed)?;
        if fd > 2 {
            descriptors.push(entry.path());
        }
    }
    // Exactly one extra must have been visible: the directory iterator itself.
    // Refuse two extras without stat-ing either. In particular, a live kqueue
    // or other non-vnode descriptor must not pass by reporting EBADF on stat.
    if descriptors.len() != 1 {
        return Err(Denial::Failed);
    }
    for descriptor in descriptors {
        match std::fs::metadata(descriptor) {
            // Darwin's fdesc filesystem reports EBADF (9), rather than ENOENT,
            // when stat reaches the directory iterator's now-closed fd.
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound
                    || error.raw_os_error() == Some(9) => {}
            // An open descriptor, an unreadable descriptor, or any unexpected
            // inspection error denies. No inherited fd can reach compilation.
            _ => return Err(Denial::Failed),
        }
    }
    Ok(())
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn configured_availability_rejects_missing_nonregular_and_nonexecutable_programs() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            configured_available(&dir.path().join("missing")),
            Err(Denial::Failed)
        );
        assert_eq!(configured_available(dir.path()), Err(Denial::Failed));
        let file = dir.path().join("guest");
        std::fs::write(&file, b"metadata-only fixture, never executed").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(configured_available(&file), Err(Denial::Failed));
    }
}
