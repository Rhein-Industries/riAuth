//! O02: the embedded redb store keeps exactly one owning process.

use axum::http::StatusCode;
use riauth::{
    config::Config,
    store::{Store, shared_filesystem},
};
use std::path::Path;

#[test]
fn redb_refuses_a_second_or_shared_owner_with_actionable_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("riauth.redb");
    let config = Config {
        data_dir: dir.path().into(),
        ..Default::default()
    };
    let owner = Store::from_config(&config).unwrap();
    owner
        .write(|tx| tx.put("authority", "revoked", &false))
        .unwrap();

    // A second server or maintenance command, and offline inspection, are refused
    // while the owner runs. The error names the store and the supported routes.
    let second = Store::from_config(&config)
        .err()
        .expect("a second redb owner opened");
    let inspected = Store::inspect(&config, |_, _| Ok(())).unwrap_err();
    let location = file.display().to_string();
    for error in [&second, &inspected] {
        assert_eq!(error.status, StatusCode::CONFLICT);
        assert_eq!(error.code, "storage_owned");
        for hint in [&*location, "riauth --server", "riauthctl", "PostgreSQL"] {
            assert!(error.message.contains(hint), "{}", error.message);
        }
    }
    // The refusal leaves the owner's authority intact: its revocation commits.
    owner
        .write(|tx| tx.put("authority", "revoked", &true))
        .unwrap();
    drop(owner);
    // Closing releases ownership, and the next owner reads that revocation.
    let next = Store::from_config(&config).unwrap();
    assert_eq!(
        next.get::<bool>("authority", "revoked").unwrap(),
        Some(true)
    );
    drop(next);
    Store::inspect(&config, |backend, tx| {
        assert_eq!((backend, tx.is_some()), ("redb", true));
        Ok(())
    })
    .unwrap();

    // On Linux a store is refused on a filesystem that other hosts can mount.
    let mountinfo = "\
22 1 8:1 / / rw,relatime shared:1 - ext4 /dev/sda1 rw
30 22 0:40 / /srv/nfs rw,relatime shared:9 master:2 - nfs4 filer:/export rw,vers=4.2
31 30 8:2 / /srv/nfs/local rw - xfs /dev/sdb1 rw
32 22 0:41 / /mnt/team\\040share rw - cifs //nas/share rw
33 22 8:3 / /var/lib/riauth rw - ext4 /dev/sdc1 rw
34 33 0:42 / /var/lib/riauth rw - nfs filer:/riauth rw
35 22 0:43 / /home rw - fuse.fuse-overlayfs fuse-overlayfs rw
36 22 0:44 / /data rw - overlay overlay rw,lowerdir=/l
";
    for (path, expected) in [
        ("/srv/nfs/riauth.redb", Some("nfs4")),
        ("/srv/nfs/local/riauth.redb", None),
        ("/srv/nfsx/riauth.redb", None),
        ("/mnt/team share/riauth.redb", Some("cifs")),
        ("/var/lib/riauth/riauth.redb", Some("nfs")),
        ("/home/riauth/riauth.redb", None),
        ("/data/riauth.redb", None),
        ("/tmp/riauth.redb", None),
    ] {
        assert_eq!(
            shared_filesystem(mountinfo, Path::new(path)),
            expected,
            "{path}"
        );
    }
}

// Creating through a dangling link writes wherever it points, so the check runs
// on the resolved location and an unresolvable link fails closed before redb
// opens anything. Valid local links keep working.
#[cfg(unix)]
#[test]
fn redb_checks_the_resolved_location_and_refuses_dangling_links() {
    use riauth::store::require_local_in;
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let (local, remote) = (root.join("local"), root.join("remote"));
    std::fs::create_dir(&local).unwrap();
    std::fs::create_dir(&remote).unwrap();
    // `remote` stands for an NFS mount inside an otherwise local tree.
    let mountinfo = format!(
        "22 1 8:1 / / rw - ext4 /dev/sda1 rw\n40 22 0:40 / {} rw - nfs4 filer:/x rw\n",
        remote
            .display()
            .to_string()
            .replace('\\', "\\134")
            .replace(' ', "\\040")
    );
    let refused = |path: &Path, reason: &str| {
        let error = require_local_in(&mountinfo, path).unwrap_err();
        assert_eq!(
            (error.status, error.code),
            (StatusCode::BAD_REQUEST, "storage_not_exclusive")
        );
        assert!(error.message.contains(reason), "{}", error.message);
    };

    // The reviewed bypass: a local store link to a share file not yet created.
    let target = remote.join("riauth.redb");
    let dangling = local.join("riauth.redb");
    symlink(&target, &dangling).unwrap();
    refused(&dangling, "Cannot confirm");
    let error = Store::open(&dangling).err().expect("dangling link opened");
    assert_eq!(error.code, "storage_not_exclusive");
    assert!(!target.exists() && dangling.symlink_metadata().is_ok());
    // Resolved links are checked at their target, for an existing file and for
    // a new store under a directory link.
    std::fs::write(remote.join("existing.redb"), b"").unwrap();
    symlink(remote.join("existing.redb"), local.join("linked.redb")).unwrap();
    refused(&local.join("linked.redb"), "nfs4");
    symlink(&remote, local.join("share")).unwrap();
    refused(&local.join("share/riauth.redb"), "nfs4");
    // A dangling ancestor and a link loop fail closed.
    symlink(remote.join("missing"), local.join("gone")).unwrap();
    refused(&local.join("gone/riauth.redb"), "Cannot confirm");
    symlink(local.join("loop.redb"), local.join("loop.redb")).unwrap();
    refused(&local.join("loop.redb"), "Cannot confirm");

    // Valid local paths are unchanged: a plain new store, a directory link and a
    // link to an existing local store all open.
    let real = local.join("real");
    std::fs::create_dir(&real).unwrap();
    symlink(&real, local.join("alias")).unwrap();
    require_local_in(&mountinfo, &local.join("plain.redb")).unwrap();
    require_local_in(&mountinfo, &local.join("alias/riauth.redb")).unwrap();
    drop(Store::open(&local.join("alias/riauth.redb")).unwrap());
    assert!(real.join("riauth.redb").is_file());
    symlink(real.join("riauth.redb"), local.join("store.redb")).unwrap();
    require_local_in(&mountinfo, &local.join("store.redb")).unwrap();
    drop(Store::open(&local.join("store.redb")).unwrap());
}

// Opening can trigger an automount or cross a mount replaced since the check, so
// the opened file is checked again against a fresh table, on Linux through the
// mount of its descriptor, before redb uses it. Any change fails closed, and a
// file created for the refused open is removed.
#[test]
fn redb_rechecks_the_opened_mount_and_fails_closed_on_changes() {
    use riauth::store::open_owner_in;
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().canonicalize().unwrap().join("data");
    std::fs::create_dir(&data).unwrap();
    let file = data.join("riauth.redb");
    let point = data
        .display()
        .to_string()
        .replace('\\', "\\134")
        .replace(' ', "\\040");
    let local = "22 1 8:1 / / rw - ext4 /dev/sda1 rw\n".to_owned();
    let autofs = format!("{local}30 22 0:40 / {point} rw - autofs systemd-1 rw\n");
    let automounted = format!("{autofs}31 30 0:41 / {point} rw - nfs4 filer:/riauth rw\n");
    let replaced = format!("{local}32 22 8:1 /srv {point} rw - ext4 /dev/sda1 rw\n");
    let elsewhere = format!("{local}33 22 0:42 / /exports rw - nfs4 filer:/x rw\n");
    let refused = |result: riauth::error::Result<()>, reason: &str| {
        let error = result.unwrap_err();
        assert_eq!(
            (error.status, error.code),
            (StatusCode::BAD_REQUEST, "storage_not_exclusive")
        );
        assert!(error.message.contains(reason), "{}", error.message);
        // Only a file this open created is removed.
        assert!(!cfg!(unix) || !file.exists());
    };

    // The reviewed race: autofs was checked, and the open itself mounted NFS.
    refused(open_owner_in(&file, &autofs, &automounted, None), "changed");
    refused(
        open_owner_in(&file, &autofs, &automounted, Some("31")),
        "changed",
    );
    // A mount replaced after the check fails closed even when it is local.
    refused(
        open_owner_in(&file, &local, &replaced, Some("32")),
        "changed",
    );
    // The descriptor's own mount decides when the path still looks local.
    refused(open_owner_in(&file, &local, &elsewhere, Some("33")), "nfs4");
    // A descriptor on a mount the fresh table does not list fails closed.
    refused(open_owner_in(&file, &local, &local, Some("99")), "changed");

    // Unchanged local storage opens, by path and by the descriptor's mount.
    open_owner_in(&file, &local, &elsewhere, Some("22")).unwrap();
    open_owner_in(&file, &local, &local, None).unwrap();
    // An existing store refused after a change stays in place and opens later.
    let error = open_owner_in(&file, &local, &replaced, None).unwrap_err();
    assert_eq!(error.code, "storage_not_exclusive");
    assert!(file.is_file());
    drop(Store::open(&file).unwrap());
}

// Inspection cannot take the store's writer lock without risking a repair, so it
// asks startup's lock question of a scratch database beside the store, leaves the
// store untouched and fails closed when it cannot ask.
#[test]
fn redb_inspection_checks_lock_support_without_touching_the_store() {
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: dir.path().into(),
        ..Default::default()
    };
    Store::from_config(&config)
        .unwrap()
        .write(|tx| tx.put("authority", "revoked", &true))
        .unwrap();
    let names = || {
        let mut names: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        names.sort();
        names
    };
    let (listed, bytes) = (
        names(),
        std::fs::read(dir.path().join("riauth.redb")).unwrap(),
    );
    let revoked = Store::inspect(&config, |_, tx| {
        tx.unwrap().get::<bool>("authority", "revoked")
    })
    .unwrap();
    assert_eq!(revoked, Some(true));
    assert_eq!(names(), listed);
    assert_eq!(
        std::fs::read(dir.path().join("riauth.redb")).unwrap(),
        bytes
    );

    // Without a scratch database the question has no answer: fail closed.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = |mode| {
            std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(mode)).unwrap()
        };
        mode(0o500);
        // A superuser can still write there; the refusal needs a real denial.
        if std::fs::File::create(dir.path().join("probe")).is_err() {
            let error = Store::inspect(&config, |_, _| Ok(())).unwrap_err();
            assert_eq!(error.code, "storage_not_exclusive");
            assert!(
                error.message.contains("scratch database"),
                "{}",
                error.message
            );
        }
        mode(0o700);
        let _ = std::fs::remove_file(dir.path().join("probe"));
        assert_eq!(
            std::fs::read(dir.path().join("riauth.redb")).unwrap(),
            bytes
        );
    }
}

// A scratch database answers only for storage it shares a filesystem and mount
// with. A store mounted as a single file, such as a 9p or virtiofs file bound into
// a local directory, cannot be probed beside it, so inspection fails closed there.
// A dangling store link is refused as startup refuses it, not reported missing.
#[cfg(unix)]
#[test]
fn redb_inspection_refuses_a_store_it_cannot_probe_in_place() {
    use riauth::store::lock_support_in;
    use std::os::unix::fs::{MetadataExt, symlink};
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    std::fs::create_dir(&data).unwrap();
    let config = Config {
        data_dir: data.clone(),
        ..Default::default()
    };
    let file = data.join("riauth.redb");
    drop(Store::open(&file).unwrap());
    let store = std::fs::metadata(&file).unwrap();
    let store = (store.dev(), store.ino());
    // Descriptors report the store file on `mount` and everything else on 22.
    let mounts = |mount: &'static str| {
        move |opened: &std::fs::File| {
            let metadata = opened.metadata().unwrap();
            let id = (metadata.dev(), metadata.ino());
            Some(if id == store { mount } else { "22" }.to_owned())
        }
    };
    let listed = || {
        let mut names: Vec<_> = std::fs::read_dir(&data)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        names.sort();
        names
    };
    let (names, bytes) = (listed(), std::fs::read(&file).unwrap());
    lock_support_in(&file, &mounts("22")).unwrap();
    let error = lock_support_in(&file, &mounts("41")).unwrap_err();
    assert_eq!(
        (error.status, error.code),
        (StatusCode::BAD_REQUEST, "storage_not_exclusive")
    );
    assert!(error.message.contains("mounted apart"), "{}", error.message);
    assert_eq!(listed(), names);
    assert_eq!(std::fs::read(&file).unwrap(), bytes);

    // Only an absent entry, or an absent directory, is a missing store.
    std::fs::remove_file(&file).unwrap();
    let missing = |config: &Config| Store::inspect(config, |_, tx| Ok(tx.is_none())).unwrap();
    assert!(missing(&config));
    assert!(missing(&Config {
        data_dir: dir.path().join("absent"),
        ..Default::default()
    }));
    let target = dir.path().join("elsewhere.redb");
    symlink(&target, &file).unwrap();
    let error = Store::inspect(&config, |_, _| Ok(())).unwrap_err();
    assert_eq!(error.code, "storage_not_exclusive");
    assert!(!target.exists());
}
