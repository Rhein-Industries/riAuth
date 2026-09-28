//! S05 storage contracts. Each body runs unchanged against all four backends.
//! Q02 covers product-level grant, revocation, receipt, and connector behavior;
//! these cases exercise transaction and range semantics beneath those flows.

use crate::common::backend::{Backend, BackendFixture};
use riauth::{error::Error, store::Store};
use std::{
    sync::{Arc, Barrier, mpsc},
    thread,
    time::Duration,
};

macro_rules! parity_contract {
    ($name:ident) => {
        mod $name {
            use super::Backend;

            #[test]
            fn redb() {
                super::$name(Backend::Redb);
            }

            #[test]
            fn redb_encrypted() {
                super::$name(Backend::EncryptedRedb);
            }

            #[test]
            #[ignore = "requires an isolated cluster; use scripts/test-contracts-postgres.sh"]
            fn postgres() {
                super::$name(Backend::Postgres);
            }

            #[test]
            #[ignore = "requires an isolated cluster; use scripts/test-contracts-postgres.sh"]
            fn postgres_encrypted() {
                super::$name(Backend::EncryptedPostgres);
            }
        }
    };
}

// redb owns one in-process Database handle for its file. Its Store clone is a
// separate caller of that handle; PostgreSQL opens a separate connection pool.
fn peer_store(f: &BackendFixture, backend: Backend) -> Store {
    match backend {
        Backend::Redb | Backend::EncryptedRedb => f.core.store.clone(),
        Backend::Postgres | Backend::EncryptedPostgres => {
            Store::from_config(&f.core.config).unwrap()
        }
    }
}

const ATOMIC: &str = "s05_contract_atomic";

fn atomic_state(store: &Store) -> (Option<u64>, Option<u64>) {
    store
        .read(|tx| Ok((tx.get(ATOMIC, "first")?, tx.get(ATOMIC, "second")?)))
        .unwrap()
}

// A failed write and a preview must leave no partial result. A second caller
// sees the old pair until commit, and an active read keeps one stable snapshot.
fn fault_rollback_and_commit_visibility(backend: Backend) {
    let f = backend.fixture();
    let writer = f.core.store.clone();
    let observer = peer_store(&f, backend);
    writer
        .write(|tx| {
            tx.put(ATOMIC, "first", &1u64)?;
            tx.put(ATOMIC, "second", &2u64)
        })
        .unwrap();

    let fault: riauth::error::Result<()> = writer.write(|tx| {
        tx.put(ATOMIC, "first", &10u64)?;
        tx.delete(ATOMIC, "second")?;
        assert_eq!(tx.get::<u64>(ATOMIC, "first")?, Some(10));
        assert_eq!(tx.get::<u64>(ATOMIC, "second")?, None);
        Err(Error::bad("intentional contract fault"))
    });
    assert!(fault.is_err());
    assert_eq!(atomic_state(&observer), (Some(1), Some(2)));

    writer
        .preview(|tx| {
            tx.put(ATOMIC, "first", &20u64)?;
            tx.delete(ATOMIC, "second")?;
            assert_eq!(tx.get::<u64>(ATOMIC, "first")?, Some(20));
            Ok(())
        })
        .unwrap();
    assert_eq!(atomic_state(&observer), (Some(1), Some(2)));

    let (staged, waiting) = mpsc::channel();
    let (release, resumed) = mpsc::channel();
    let concurrent_writer = writer.clone();
    let pending = thread::spawn(move || {
        concurrent_writer.write(|tx| {
            tx.put(ATOMIC, "first", &3u64)?;
            tx.delete(ATOMIC, "second")?;
            staged.send(()).unwrap();
            resumed.recv_timeout(Duration::from_secs(10)).unwrap();
            Ok(())
        })
    });
    waiting.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(atomic_state(&observer), (Some(1), Some(2)));
    release.send(()).unwrap();
    pending.join().unwrap().unwrap();
    assert_eq!(atomic_state(&observer), (Some(3), None));

    observer
        .read(|tx| {
            assert_eq!(tx.get::<u64>(ATOMIC, "first")?, Some(3));
            writer.write(|other| other.put(ATOMIC, "first", &4u64))?;
            assert_eq!(tx.get::<u64>(ATOMIC, "first")?, Some(3));
            Ok(())
        })
        .unwrap();
    assert_eq!(atomic_state(&observer), (Some(4), None));
}

// Both callers race for one durable claim. The losing transaction must neither
// report success nor create a second consumption record.
fn concurrent_claim_is_single_use(backend: Backend) {
    let f = backend.fixture();
    let left = f.core.store.clone();
    let right = peer_store(&f, backend);
    left.write(|tx| tx.put("s05_contract_claims", "one", &true))
        .unwrap();

    let start = Arc::new(Barrier::new(2));
    let mut workers = Vec::new();
    for (store, caller) in [(left.clone(), "left"), (right.clone(), "right")] {
        let start = start.clone();
        workers.push(thread::spawn(move || {
            start.wait();
            store.write(|tx| {
                if tx.get::<bool>("s05_contract_claims", "one")? != Some(true) {
                    return Ok(false);
                }
                tx.delete("s05_contract_claims", "one")?;
                tx.put("s05_contract_consumptions", caller, &1u8)?;
                Ok(true)
            })
        }));
    }
    let results: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|won| **won).count(), 1);
    assert_eq!(
        right.get::<bool>("s05_contract_claims", "one").unwrap(),
        None
    );
    assert_eq!(
        right.list::<u8>("s05_contract_consumptions").unwrap().len(),
        1
    );
    drop(left);
    drop(right);

    let reopened = f.reopen_with(|_| {});
    assert_eq!(
        reopened
            .core
            .store
            .get::<bool>("s05_contract_claims", "one")
            .unwrap(),
        None
    );
    assert_eq!(
        reopened
            .core
            .store
            .list::<u8>("s05_contract_consumptions")
            .unwrap()
            .len(),
        1
    );
}

const PAGE: &str = "s05_contract_page";

fn pages(store: &Store, reverse: bool) -> Vec<(String, u32)> {
    store
        .read(|tx| {
            let mut result = Vec::new();
            let mut cursor: Option<String> = None;
            for _ in 0..8 {
                let page = if reverse {
                    tx.scan_reverse::<u32>(PAGE, cursor.as_deref(), 2)?
                } else {
                    tx.scan::<u32>(PAGE, cursor.as_deref(), 2)?
                };
                if page.is_empty() {
                    return Ok(result);
                }
                cursor = Some(page.last().unwrap().0.clone());
                result.extend(page);
            }
            Err(Error::bad("contract pagination did not terminate"))
        })
        .unwrap()
}

// Forward and reverse pages use exclusive cursors, preserve bytewise key order,
// and never spill into the adjacent buckets on either side of this prefix.
fn range_pages_are_ordered_and_bucket_bounded(backend: Backend) {
    let f = backend.fixture();
    let store = &f.core.store;
    let peer = peer_store(&f, backend);
    let entries = [
        ("a", 1),
        ("aa", 2),
        ("ab", 3),
        ("b", 4),
        ("ba", 5),
        ("é", 6),
    ];
    store
        .write(|tx| {
            for (key, value) in entries {
                tx.put(PAGE, key, &value)?;
            }
            tx.put("s05_contract_page-", "z", &99u32)?;
            tx.put("s05_contract_page0", "a", &100u32)
        })
        .unwrap();

    let expected: Vec<_> = entries
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
    assert_eq!(pages(store, false), expected);
    assert_eq!(
        pages(store, true),
        expected.iter().rev().cloned().collect::<Vec<_>>()
    );
    store
        .read(|tx| {
            assert!(tx.scan::<u32>(PAGE, None, 0)?.is_empty());
            assert!(tx.scan_reverse::<u32>(PAGE, None, 0)?.is_empty());
            assert_eq!(
                tx.scan::<u32>(PAGE, Some("aa"), 2)?,
                expected[2..4].to_vec()
            );
            assert_eq!(
                tx.scan_reverse::<u32>(PAGE, Some("ba"), 2)?,
                vec![expected[3].clone(), expected[2].clone()]
            );
            Ok(())
        })
        .unwrap();

    store
        .read(|tx| {
            let first = tx.scan::<u32>(PAGE, None, 2)?;
            assert_eq!(first, expected[..2].to_vec());
            peer.write(|other| {
                other.delete(PAGE, "ab")?;
                other.put(PAGE, "ac", &7u32)
            })?;
            // Page two remains part of the same read snapshot.
            assert_eq!(
                tx.scan::<u32>(PAGE, Some("aa"), 2)?,
                expected[2..4].to_vec()
            );
            Ok(())
        })
        .unwrap();
    let changed = vec![
        ("a".into(), 1),
        ("aa".into(), 2),
        ("ac".into(), 7),
        ("b".into(), 4),
        ("ba".into(), 5),
        ("é".into(), 6),
    ];
    assert_eq!(pages(store, false), changed);
    assert_eq!(
        pages(store, true),
        changed.into_iter().rev().collect::<Vec<_>>()
    );
}

parity_contract!(fault_rollback_and_commit_visibility);
parity_contract!(concurrent_claim_is_single_use);
parity_contract!(range_pages_are_ordered_and_bucket_bounded);
