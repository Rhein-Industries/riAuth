//! O02: the embedded redb store has exactly one owning process.
//!
//! redb refuses a second opener through advisory OS file locks. A lock only
//! excludes processes that share the kernel holding it, and redb opens without
//! one where the filesystem refuses byte-range locks. On a network or cluster
//! filesystem another host, such as a gateway or worker mounting the same
//! volume, could open the file without either side seeing a conflict. Remote
//! components use the authorized API and multiple service processes use
//! PostgreSQL, so storage refuses both a second owner and a location where it
//! cannot stay the only one.

use crate::error::{Error, Result};
use axum::http::StatusCode;
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

/// Linux filesystem types that exist to share one tree between hosts. The 9p
/// and virtiofs transports are absent because gVisor and Kata runtimes also use
/// them for single-owner local volumes.
const SHARED: &[&str] = &[
    "nfs",
    "nfs4",
    "cifs",
    "smb3",
    "smbfs",
    "ncpfs",
    "afs",
    "coda",
    "ceph",
    "glusterfs",
    "lustre",
    "gpfs",
    "beegfs",
    "pvfs2",
    "ocfs2",
    "gfs2",
    "davfs",
    "fuse.sshfs",
    "fuse.ceph-fuse",
    "fuse.glusterfs",
    "fuse.s3fs",
    "fuse.gcsfuse",
    "fuse.rclone",
    "fuse.juicefs",
    "fuse.goofys",
    "fuse.blobfuse",
    "fuse.blobfuse2",
    "fuse.mountpoint-s3",
];

/// Refuses a redb path on a filesystem other hosts can open, before redb
/// creates, repairs or locks the file.
pub(super) fn require_local(path: &Path) -> Result<()> {
    let location = location(path)?;
    match mount_table(path)? {
        Some(mountinfo) => refuse_shared(&mountinfo, path, &location),
        None => Ok(()),
    }
}

/// [`require_local`] against a supplied `/proc/self/mountinfo` table.
#[doc(hidden)]
pub fn require_local_in(mountinfo: &str, path: &Path) -> Result<()> {
    refuse_shared(mountinfo, path, &location(path)?)
}

/// Where opening `path` reads or creates the file, with every link resolved.
/// Creating through a dangling link writes wherever it points, so a link or
/// ancestor that does not resolve fails closed instead of falling back to the
/// lexical parent.
fn location(path: &Path) -> Result<PathBuf> {
    let resolved = match path.symlink_metadata() {
        // An existing entry, or a link that must reach one.
        Ok(_) => path.canonicalize(),
        // A new store is created in its directory.
        Err(error) if error.kind() == ErrorKind::NotFound => match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.canonicalize(),
            _ => Path::new(".").canonicalize(),
        },
        Err(error) => Err(error),
    };
    resolved.map_err(|error| {
        not_exclusive(format!(
            "Cannot confirm where the embedded redb store {} would be opened ({error}). Point data_dir at an existing local directory, and make every symbolic link on the path resolve to existing local storage.",
            path.display()
        ))
    })
}

fn refuse_shared(mountinfo: &str, path: &Path, location: &Path) -> Result<()> {
    match shared_filesystem(mountinfo, Path::new(&*location.to_string_lossy())) {
        Some(kind) => Err(not_exclusive(format!(
            "The embedded redb store {} resolves to {}, on a {kind} network or cluster filesystem, where file locks cannot keep one owning process across hosts. Stop every process using it and move the data directory to local storage on the one host that runs riAuth, or configure PostgreSQL for several service processes. Remote gateways and workers must use the authorized API, never this file.",
            path.display(),
            location.display()
        ))),
        None => Ok(()),
    }
}

/// Called while `path` is open for writing: a second open must meet the
/// owner's lock, or the filesystem enforces none and nothing is excluded.
pub(super) fn require_exclusive(path: &Path) -> Result<()> {
    match redb::ReadOnlyDatabase::open(path) {
        Err(redb::DatabaseError::DatabaseAlreadyOpen) => Ok(()),
        // redb locks before reading, so reaching the header means no lock stopped it.
        Ok(_) | Err(redb::DatabaseError::RepairAborted) => Err(not_exclusive(format!(
            "The filesystem holding the embedded redb store {} does not enforce file locks, so nothing would stop a second process from opening it. Move the data directory to local storage with file-lock support on the one host that runs riAuth, or configure PostgreSQL.",
            path.display()
        ))),
        Err(error) => Err(Error::internal(error)),
    }
}

/// Maps redb's lock conflict to an operator diagnostic.
pub(super) fn open_error(path: &Path, error: redb::DatabaseError) -> Error {
    match error {
        redb::DatabaseError::DatabaseAlreadyOpen => Error::new(
            StatusCode::CONFLICT,
            "storage_owned",
            format!(
                "The embedded redb store {} is already owned by another riAuth process. Stop the server or maintenance command holding it before a local storage command, manage a running server through its authorized API (riauth --server or riauthctl), or configure PostgreSQL for several service processes.",
                path.display()
            ),
        ),
        error => Error::internal(error),
    }
}

fn not_exclusive(message: String) -> Error {
    Error::new(StatusCode::BAD_REQUEST, "storage_not_exclusive", message)
}

#[cfg(target_os = "linux")]
fn mount_table(path: &Path) -> Result<Option<String>> {
    match std::fs::read("/proc/self/mountinfo") {
        Ok(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).into_owned())),
        Err(error) => Err(not_exclusive(format!(
            "Cannot confirm that the embedded redb store {} is on local storage: /proc/self/mountinfo is unreadable ({error}). Mount /proc for the riAuth process or configure PostgreSQL.",
            path.display()
        ))),
    }
}

/// Other platforms expose no mount table without native calls; the lock probe
/// still applies there. Released server artifacts are Linux.
#[cfg(not(target_os = "linux"))]
fn mount_table(_: &Path) -> Result<Option<String>> {
    Ok(None)
}

/// The type of the mount containing `path` when it is a shared filesystem.
/// `mountinfo` uses the `/proc/self/mountinfo` format. The deepest mount point
/// wins, and a later mount over the same point hides an earlier one.
#[doc(hidden)]
pub fn shared_filesystem<'a>(mountinfo: &'a str, path: &Path) -> Option<&'a str> {
    let mut deepest: Option<(usize, &str)> = None;
    for line in mountinfo.lines() {
        let mut fields = line.split(' ');
        let Some(point) = fields.nth(4) else {
            continue;
        };
        // Optional fields end at a lone "-", which the filesystem type follows.
        let Some(kind) = fields.skip_while(|field| *field != "-").nth(1) else {
            continue;
        };
        let point = unescape(point);
        let point = Path::new(&point);
        if path.starts_with(point) {
            let depth = point.components().count();
            if deepest.is_none_or(|(deepest, _)| depth >= deepest) {
                deepest = Some((depth, kind));
            }
        }
    }
    deepest
        .map(|(_, kind)| kind)
        .filter(|kind| SHARED.contains(kind))
}

/// Mount points escape space, tab, newline and backslash as three octal digits.
fn unescape(field: &str) -> String {
    let mut bytes = Vec::with_capacity(field.len());
    let mut rest = field.as_bytes();
    while let Some((&first, tail)) = rest.split_first() {
        match tail {
            [a @ b'0'..=b'3', b @ b'0'..=b'7', c @ b'0'..=b'7', ..] if first == b'\\' => {
                bytes.push(((a - b'0') << 6) | ((b - b'0') << 3) | (c - b'0'));
                rest = &tail[3..];
            }
            _ => {
                bytes.push(first);
                rest = tail;
            }
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}
