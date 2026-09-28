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
