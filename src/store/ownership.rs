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
//!
//! Opening can change what a path reaches: it can trigger an automount or cross
//! a mount replaced since the check. The owner therefore opens the file itself,
//! checks it again against a fresh mount table (on Linux through the mount of
//! the open descriptor) and only then hands that descriptor to redb.

use crate::error::{Error, Result};
use axum::http::StatusCode;
use redb::{Database, DatabaseError, ReadOnlyDatabase};
use std::{
    fs::{File, Metadata, OpenOptions},
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

/// What the check before an open resolved.
pub(super) struct Checked {
    /// The canonical store file, or the directory a new one is created in.
    location: PathBuf,
    existed: bool,
    /// The mount holding `location`, where the platform has a mount table.
    mount: Option<Mount>,
}

#[derive(PartialEq)]
struct Mount {
    id: String,
    kind: String,
}

/// Reads the mount table, or `None` where the platform has none.
type Tables<'a> = &'a mut dyn FnMut(&Path) -> Result<Option<String>>;
/// The mount ID an open descriptor reports, where the platform reports one.
type Opened<'a> = &'a dyn Fn(&File) -> Option<String>;

/// Refuses a redb path on a filesystem other hosts can open, before redb
/// creates, repairs or locks the file.
pub(super) fn require_local(path: &Path) -> Result<Checked> {
    check(path, &mut mount_table)
}

/// [`require_local`] against a supplied `/proc/self/mountinfo` table.
#[doc(hidden)]
pub fn require_local_in(mountinfo: &str, path: &Path) -> Result<()> {
    check(path, &mut |_| Ok(Some(mountinfo.to_owned()))).map(drop)
}

fn check(path: &Path, tables: Tables<'_>) -> Result<Checked> {
    let (location, existed) = location(path)?;
    let mut mount = None;
    if let Some(mountinfo) = tables(path)? {
        mount = mount_of(&mountinfo, &location);
        refuse_shared(path, &location, mount.as_ref().map(|m| m.kind.as_str()))?;
    }
    Ok(Checked {
        location,
        existed,
        mount,
    })
}

/// Opens `path` for its only owner, creating the file when absent. redb gets
/// the descriptor that was checked, and a second open must then meet its lock.
pub(super) fn open_owner(path: &Path) -> Result<Database> {
    open(path, &mut mount_table, &opened_mount)
}

/// [`open_owner`] with supplied tables: `before` for the check, `after` once
/// the file is open, whose descriptor reports mount `opened`.
#[doc(hidden)]
pub fn open_owner_in(path: &Path, before: &str, after: &str, opened: Option<&str>) -> Result<()> {
    let mut reads = 0;
    let mut tables = |_: &Path| -> Result<Option<String>> {
        reads += 1;
        Ok(Some(if reads == 1 { before } else { after }.to_owned()))
    };
    open(path, &mut tables, &|_: &File| opened.map(str::to_owned)).map(drop)
}

fn open(path: &Path, tables: Tables<'_>, opened: Opened<'_>) -> Result<Database> {
    let checked = check(path, &mut *tables)?;
    let file = options(!checked.existed)
        .open(path)
        .map_err(|error| match error.kind() {
            // Created or removed since the check; exclusive creation never
            // follows a link planted there.
            ErrorKind::AlreadyExists | ErrorKind::NotFound => changed(path),
            _ => Error::internal(error),
        })?;
    let identity = file.metadata().ok().as_ref().and_then(identity);
    let owned = own(path, &checked, file, identity, tables, opened);
    // A file this call created and a check then refused holds no records.
    if let Err(error) = &owned
        && error.code == "storage_not_exclusive"
        && !checked.existed
        && identity.is_some()
        && current(path) == identity
    {
        let _ = std::fs::remove_file(path);
    }
    owned
}

fn own(
    path: &Path,
    checked: &Checked,
    file: File,
    identity: Option<(u64, u64)>,
    tables: Tables<'_>,
    opened: Opened<'_>,
) -> Result<Database> {
    recheck(path, checked, Some(&file), tables, opened)?;
    let db = Database::builder()
        .create_file(file)
        .map_err(|error| open_error(path, error))?;
    // The probe opens by path, which must still reach the file redb holds.
    same_file(path, identity)?;
    lock_enforced(path, path)?;
    same_file(path, identity)?;
    Ok(db)
}

/// Checks an opened store again against a fresh mount table: the path must
/// still reach the mount checked before and, when given, the open file. On
/// Linux that file's own mount, which no later mount change can move, must not
/// be shared.
pub(super) fn reopened(path: &Path, checked: &Checked, file: Option<&File>) -> Result<()> {
    recheck(path, checked, file, &mut mount_table, &opened_mount)
}

fn recheck(
    path: &Path,
    checked: &Checked,
    file: Option<&File>,
    tables: Tables<'_>,
    opened: Opened<'_>,
) -> Result<()> {
    if let Some(file) = file {
        same_file(path, file.metadata().ok().as_ref().and_then(identity))?;
    }
    let Some(mountinfo) = tables(path)? else {
        return Ok(());
    };
    let (location, _) = location(path)?;
    let opened = file.and_then(opened);
    let before = checked.mount.as_ref();
    let now = mount_of(&mountinfo, &location);
    if now.as_ref() != before {
        return Err(changed(path));
    }
    let kind = match &opened {
        Some(id) => Some(kind_of(&mountinfo, id).ok_or_else(|| changed(path))?),
        None => now.as_ref().map(|mount| mount.kind.as_str()),
    };
    refuse_shared(path, &location, kind)
}

/// Read-only inspection must not take the store's writer lock, because a
/// writer open can repair the file. It asks startup's lock question of a
/// scratch database beside the store instead, removes it, and fails closed
/// without an answer. Returns the store, opened read-only and unlocked, for
/// the recheck after redb opens it.
pub(super) fn require_lock_support(path: &Path, checked: &Checked) -> Result<File> {
    probe_in_place(path, checked, &opened_mount)
}

/// [`require_lock_support`] where open descriptors report mount `opened`.
#[doc(hidden)]
pub fn lock_support_in(path: &Path, opened: &dyn Fn(&File) -> Option<String>) -> Result<()> {
    probe_in_place(path, &require_local(path)?, opened).map(drop)
}

fn probe_in_place(path: &Path, checked: &Checked, opened: Opened<'_>) -> Result<File> {
    // Only identifies the store; it takes no lock and reads nothing.
    let store = File::open(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => changed(path),
        _ => Error::internal(error),
    })?;
    let directory = checked.location.parent().unwrap_or(&checked.location);
    let probe = directory.join(format!(
        ".riauth-lock-probe-{}",
        crate::crypto::random_token("")
    ));
    let unverified = |reason: String| {
        not_exclusive(format!(
            "Cannot confirm that the filesystem holding the embedded redb store {} enforces file locks: {reason}. Run the command as the store's owner, with the store in a directory on its own filesystem rather than mounted as a single file.",
            path.display()
        ))
    };
    let failed = |error: &dyn std::fmt::Display| {
        unverified(format!(
            "a scratch database in {} failed ({error})",
            directory.display()
        ))
    };
    let scratch = options(true).open(&probe).map_err(|error| failed(&error))?;
    // Beside a store mounted as a file, or on another filesystem than its
    // directory, the scratch database would answer for different storage.
    let result = if same_place(&scratch, &store, opened) {
        match Database::builder().create_file(scratch) {
            Ok(scratch) => {
                let enforced = lock_enforced(&probe, path);
                drop(scratch);
                enforced
            }
            Err(error) => Err(failed(&error)),
        }
    } else {
        drop(scratch);
        Err(unverified(format!(
            "the store is mounted apart from {} or lies on another filesystem, so a scratch database there would test different storage",
            directory.display()
        )))
    };
    let _ = std::fs::remove_file(&probe);
    result.map(|()| store)
}

/// Whether two open files share a filesystem and, where descriptors report
/// one, a mount. A file mounted on its own is a different place.
fn same_place(a: &File, b: &File, opened: Opened<'_>) -> bool {
    let device = |file: &File| {
        file.metadata()
            .ok()
            .as_ref()
            .and_then(identity)
            .map(|(device, _)| device)
    };
    device(a) == device(b) && opened(a) == opened(b)
}

/// A second, read-only open must meet the lock `probe` is held under; redb
/// opens without one where the filesystem refuses byte-range locks.
fn lock_enforced(probe: &Path, store: &Path) -> Result<()> {
    match ReadOnlyDatabase::open(probe) {
        Err(DatabaseError::DatabaseAlreadyOpen) => Ok(()),
        // redb locks before reading, so reaching the header means no lock stopped it.
        Ok(_) | Err(DatabaseError::RepairAborted) => Err(not_exclusive(format!(
            "The filesystem holding the embedded redb store {} does not enforce file locks, so nothing would stop a second process from opening it. Move the data directory to local storage with file-lock support on the one host that runs riAuth, or configure PostgreSQL.",
            store.display()
        ))),
        Err(error) => Err(Error::internal(error)),
    }
}

/// Maps redb's lock conflict to an operator diagnostic.
pub(super) fn open_error(path: &Path, error: DatabaseError) -> Error {
    match error {
        DatabaseError::DatabaseAlreadyOpen => Error::new(
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

/// Where opening `path` reads or creates the file, with every link resolved,
/// and whether anything exists there yet. Creating through a dangling link
/// writes wherever it points, so a link or ancestor that does not resolve
/// fails closed instead of falling back to the lexical parent.
fn location(path: &Path) -> Result<(PathBuf, bool)> {
    let resolved = match path.symlink_metadata() {
        // An existing entry, or a link that must reach one.
        Ok(_) => path.canonicalize().map(|location| (location, true)),
        // A new store is created in its directory.
        Err(error) if error.kind() == ErrorKind::NotFound => match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.canonicalize(),
            _ => Path::new(".").canonicalize(),
        }
        .map(|location| (location, false)),
        Err(error) => Err(error),
    };
    resolved.map_err(|error| {
        not_exclusive(format!(
            "Cannot confirm where the embedded redb store {} would be opened ({error}). Point data_dir at an existing local directory, and make every symbolic link on the path resolve to existing local storage.",
            path.display()
        ))
    })
}

fn refuse_shared(path: &Path, location: &Path, kind: Option<&str>) -> Result<()> {
    match kind.filter(|kind| SHARED.contains(kind)) {
        Some(kind) => Err(not_exclusive(format!(
            "The embedded redb store {} resolves to {}, on a {kind} network or cluster filesystem, where file locks cannot keep one owning process across hosts. Stop every process using it and move the data directory to local storage on the one host that runs riAuth, or configure PostgreSQL for several service processes. Remote gateways and workers must use the authorized API, never this file.",
            path.display(),
            location.display()
        ))),
        None => Ok(()),
    }
}

fn changed(path: &Path) -> Error {
    not_exclusive(format!(
        "The storage reached by the embedded redb store {} changed while riAuth was opening it, for example through an automount or a replaced mount or link. Mount the data directory on local storage before starting riAuth, then retry.",
        path.display()
    ))
}

fn not_exclusive(message: String) -> Error {
    Error::new(StatusCode::BAD_REQUEST, "storage_not_exclusive", message)
}

fn options(create: bool) -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    if create {
        options.create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    }
    options
}

fn same_file(path: &Path, identity: Option<(u64, u64)>) -> Result<()> {
    if identity.is_some() && current(path) != identity {
        return Err(changed(path));
    }
    Ok(())
}

/// The identity of the file `path` reaches now.
fn current(path: &Path) -> Option<(u64, u64)> {
    std::fs::metadata(path).ok().as_ref().and_then(identity)
}

#[cfg(unix)]
fn identity(metadata: &Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((metadata.dev(), metadata.ino()))
}

/// Stable std has no file identity on other platforms; the probe then relies
/// on the path alone.
#[cfg(not(unix))]
fn identity(_: &Metadata) -> Option<(u64, u64)> {
    None
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

/// The mount ID Linux reports for an open descriptor.
#[cfg(target_os = "linux")]
fn opened_mount(file: &File) -> Option<String> {
    use std::os::fd::AsRawFd;
    let info = std::fs::read_to_string(format!("/proc/self/fdinfo/{}", file.as_raw_fd())).ok()?;
    info.lines()
        .find_map(|line| Some(line.strip_prefix("mnt_id:")?.trim().to_owned()))
}

#[cfg(not(target_os = "linux"))]
fn opened_mount(_: &File) -> Option<String> {
    None
}

/// The type of the mount containing `path` when it is a shared filesystem.
#[doc(hidden)]
pub fn shared_filesystem<'a>(mountinfo: &'a str, path: &Path) -> Option<&'a str> {
    deepest(mountinfo, path)
        .map(|(_, kind)| kind)
        .filter(|kind| SHARED.contains(kind))
}

fn mount_of(mountinfo: &str, path: &Path) -> Option<Mount> {
    deepest(mountinfo, path).map(|(id, kind)| Mount {
        id: id.to_owned(),
        kind: kind.to_owned(),
    })
}

/// The ID and type of the mount holding `path` in a `/proc/self/mountinfo`
/// table. The deepest mount point wins, and a later mount over the same point
/// hides an earlier one.
fn deepest<'a>(mountinfo: &'a str, path: &Path) -> Option<(&'a str, &'a str)> {
    let path = path.to_string_lossy();
    let path = Path::new(&*path);
    let mut deepest: Option<(usize, &str, &str)> = None;
    for (id, point, kind) in rows(mountinfo) {
        let point = Path::new(&point);
        if path.starts_with(point) {
            let depth = point.components().count();
            if deepest.is_none_or(|(deepest, ..)| depth >= deepest) {
                deepest = Some((depth, id, kind));
            }
        }
    }
    deepest.map(|(_, id, kind)| (id, kind))
}

fn kind_of<'a>(mountinfo: &'a str, id: &str) -> Option<&'a str> {
    rows(mountinfo).find_map(|(row, _, kind)| (row == id).then_some(kind))
}

/// Mount ID, unescaped mount point and type of each table row.
fn rows(mountinfo: &str) -> impl Iterator<Item = (&str, String, &str)> {
    mountinfo.lines().filter_map(|line| {
        let mut fields = line.split(' ');
        let id = fields.next()?;
        let point = unescape(fields.nth(3)?);
        // Optional fields end at a lone "-", which the filesystem type follows.
        let kind = fields.skip_while(|field| *field != "-").nth(1)?;
        Some((id, point, kind))
    })
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
