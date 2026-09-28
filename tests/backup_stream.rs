mod common;

use common::Fixture;
use riauth::operations::stream::{
    MAGIC, Phase, Progress, StreamLimits, StreamOptions, StreamSummary, restore_stream,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

const PREAMBLE: usize = 32;

struct Seed {
    fixture: Fixture,
    alice: String,
    access: String,
}

impl Seed {
    /// A user, a client and a grant, plus enough records for several frames.
    fn new(extra: usize) -> Self {
        let fixture = Fixture::new();
        fixture.client("app", false);
        let alice = fixture.user("alice");
        let tokens = fixture.tokens("app", &alice, None);
        let access = common::text(&tokens, "access_token");
        let payload = "p".repeat(2048);
        fixture
            .core
            .store
            .write(|tx| {
                for index in 0..extra {
                    tx.put(
                        "audit",
                        &format!("stream-{index:06}"),
                        &serde_json::json!({"index": index, "payload": payload}),
                    )?;
                }
                Ok(())
            })
            .unwrap();
        Self {
            fixture,
            alice,
            access,
        }
    }

    fn snapshot(&self) -> BTreeMap<String, Value> {
        self.fixture.snapshot().unwrap()
    }

    fn backup(&self, key: &str, options: StreamOptions<'_>) -> (Vec<u8>, StreamSummary) {
        let mut out = Vec::new();
        let summary = self
            .fixture
            .core
            .backup_stream(&self.fixture.admin, key, &mut out, options)
            .unwrap();
        (out, summary)
    }

    fn assert_restored(&self, output: &Path, before: &BTreeMap<String, Value>) {
        let restored = riauth::core::Core::open(
            riauth::config::Config::load(&output.join("riauth.toml")).unwrap(),
        )
        .unwrap();
        let after = restored.store.read(|tx| tx.snapshot()).unwrap();
        for (key, value) in before {
            if key.starts_with("clients/")
                || key.starts_with("groups/")
                || key.starts_with("usernames/")
                || key == "meta/keys"
                || key == "meta/issuer"
            {
                assert_eq!(after.get(key), Some(value), "restored {key} changed");
            }
            if key.starts_with("users/") {
                let restored_user = after.get(key).expect("restored user missing");
                for field in [
                    "id",
                    "username",
                    "subjects",
                    "pairwise_seed",
                    "password_hash",
                ] {
                    assert_eq!(
                        restored_user.get(field),
                        value.get(field),
                        "restored {key}/{field} changed"
                    );
                }
                assert_eq!(
                    restored_user["epoch"].as_u64().unwrap(),
                    value["epoch"].as_u64().unwrap() + riauth::recovery::STRIDE,
                );
            }
        }
        assert!(restored.userinfo(&self.access).is_err());
        assert!(restored.me(&self.alice).is_err());
        assert!(restored.me(&self.fixture.admin).is_err());
        assert!(restored.store.ready().is_err());
    }
}

fn key_file(directory: &Path, name: &str, key: &str) -> PathBuf {
    let path = directory.join(format!("{name}.key"));
    riauth::config::write_private(&path, key.as_bytes(), false).unwrap();
    path
}

fn archive(directory: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = directory.join(format!("{name}.riauth-backup"));
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Byte ranges `(start, end)` of each frame after the preamble.
fn frames(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut frames = Vec::new();
    let mut at = PREAMBLE;
    while at < bytes.len() {
        let len = u32::from_be_bytes(bytes[at + 1..at + 5].try_into().unwrap()) as usize;
        frames.push((at, at + 5 + len));
        at += 5 + len;
    }
    assert_eq!(at, bytes.len());
    frames
}

fn splice(bytes: &[u8], order: &[(usize, usize)]) -> Vec<u8> {
    let mut out = bytes[..PREAMBLE].to_vec();
    for (start, end) in order {
        out.extend_from_slice(&bytes[*start..*end]);
    }
    out
}

fn rejects(directory: &Path, name: &str, bytes: &[u8], key: &Path) -> riauth::error::Error {
    let input = archive(directory, name, bytes);
    let output = directory.join(format!("{name}-out"));
    let error = riauth::operations::restore(&input, key, &output, None)
        .expect_err(&format!("{name} restored"));
    assert!(
        !output.exists(),
        "{name} created output before verification"
    );
    error
}

#[test]
fn stream_backup_restores_identity_with_the_same_recovery_policy_as_v2() {
    let seed = Seed::new(600);
    let before = seed.snapshot();
    let key = riauth::crypto::random_token("");
    let mut seen = Vec::new();
    let mut record = |progress: &Progress| seen.push(*progress);
    let (bytes, summary) = seed.backup(
        &key,
        StreamOptions {
            progress: Some(&mut record),
            ..Default::default()
        },
    );
    assert!(bytes.starts_with(MAGIC));
    assert_eq!(summary.api_version, "riauth.backup/v3");
    assert_eq!(summary.records, before.len() as u64);
    assert_eq!(summary.bytes, bytes.len() as u64);
    assert_eq!(summary.frames as usize, frames(&bytes).len());
    assert!(summary.frames > 4, "{summary:?}");
    let rendered = String::from_utf8_lossy(&bytes);
    assert!(!rendered.contains(common::PASSWORD));
    assert!(!rendered.contains("PRIVATE KEY"));
    assert!(!rendered.contains("stream-000001"));

    assert!(seen.iter().all(|p| p.phase == Phase::Export));
    assert!(seen.windows(2).all(|w| w[0].bytes < w[1].bytes
        && w[0].frames + 1 == w[1].frames
        && w[0].records <= w[1].records));
    let last = seen.last().unwrap();
    assert_eq!(
        (last.frames, last.records, last.bytes),
        (summary.frames, summary.records, summary.bytes)
    );

    let directory = tempfile::tempdir().unwrap();
    let key_path = key_file(directory.path(), "stream", &key);
    let input = archive(directory.path(), "stream", &bytes);
    let output = directory.path().join("restored");
    let mut phases = Vec::new();
    let mut observe = |progress: &Progress| phases.push(*progress);
    let result = restore_stream(
        &input,
        &key_path,
        &output,
        None,
        StreamOptions {
            progress: Some(&mut observe),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(result["verified"], true);
    assert_eq!(result["serving_allowed"], false);
    assert!(result["recovery"]["snapshot_created_at"].is_number());
    for phase in [Phase::Verify, Phase::Import] {
        let last = phases.iter().rfind(|p| p.phase == phase).unwrap();
        assert_eq!((last.records, last.bytes), (summary.records, summary.bytes));
    }
    seed.assert_restored(&output, &before);

    // `operations::restore` (the CLI path) detects the stream by its magic.
    let dispatched = directory.path().join("dispatched");
    riauth::operations::restore(&input, &key_path, &dispatched, None).unwrap();
    seed.assert_restored(&dispatched, &before);

    // The existing v2 archive from the same store still restores unchanged.
    let v2 = seed.fixture.core.backup(&seed.fixture.admin, &key).unwrap();
    assert_eq!(v2["api_version"], "riauth.backup/v2");
    let v2_input = directory.path().join("v2.json");
    std::fs::write(&v2_input, serde_json::to_vec(&v2).unwrap()).unwrap();
    let v2_output = directory.path().join("v2-restored");
    riauth::operations::restore(&v2_input, &key_path, &v2_output, None).unwrap();
    seed.assert_restored(&v2_output, &before);
}

#[test]
fn stream_restore_rejects_truncation_reordering_tampering_and_wrong_keys() {
    let seed = Seed::new(300);
    let key = riauth::crypto::random_token("");
    let (bytes, _) = seed.backup(&key, StreamOptions::default());
    let (other, _) = seed.backup(&key, StreamOptions::default());
    let layout = frames(&bytes);
    assert!(
        layout.len() >= 5,
        "need header, several record frames and a trailer"
    );
    let (header, trailer) = (layout[0], *layout.last().unwrap());
    let records = &layout[1..layout.len() - 1];
    let directory = tempfile::tempdir().unwrap();
    let key_path = key_file(directory.path(), "right", &key);
    let source_before = seed.snapshot();
    let mut cases: Vec<(String, Vec<u8>)> = Vec::new();

    // Truncation at every frame boundary, inside the preamble and inside frames.
    for (index, (start, end)) in layout.iter().enumerate() {
        cases.push((
            format!("cut-before-frame-{index}"),
            bytes[..*start].to_vec(),
        ));
        cases.push((
            format!("cut-inside-head-{index}"),
            bytes[..start + 3].to_vec(),
        ));
        cases.push((
            format!("cut-inside-body-{index}"),
            bytes[..end - 1].to_vec(),
        ));
    }
    cases.push(("cut-magic".into(), bytes[..8].to_vec()));
    cases.push(("cut-stream-id".into(), bytes[..20].to_vec()));

    // Reordering, dropping, repeating and relocating frames.
    let mut swapped = layout.clone();
    swapped.swap(1, 2);
    cases.push(("swap-records".into(), splice(&bytes, &swapped)));
    let mut dropped = layout.clone();
    dropped.remove(2);
    cases.push(("drop-record-frame".into(), splice(&bytes, &dropped)));
    let mut repeated = layout.clone();
    repeated.insert(2, layout[1]);
    cases.push(("repeat-record-frame".into(), splice(&bytes, &repeated)));
    cases.push((
        "trailer-early".into(),
        splice(&bytes, &[header, records[0], trailer]),
    ));
    cases.push(("no-header".into(), splice(&bytes, &layout[1..])));
    cases.push(("header-only".into(), splice(&bytes, &[header])));
    let mut header_twice = layout.clone();
    header_twice.insert(1, header);
    cases.push(("header-twice".into(), splice(&bytes, &header_twice)));
    let mut appended = bytes.clone();
    appended.extend_from_slice(&bytes[trailer.0..trailer.1]);
    cases.push(("second-trailer".into(), appended));
    let mut garbage = bytes.clone();
    garbage.push(0);
    cases.push(("trailing-byte".into(), garbage));

    // Splicing a frame from another archive under the same key.
    let other_layout = frames(&other);
    let mut spliced = bytes[..layout[1].0].to_vec();
    spliced.extend_from_slice(&other[other_layout[1].0..other_layout[1].1]);
    spliced.extend_from_slice(&bytes[layout[1].1..]);
    cases.push(("foreign-frame".into(), spliced));
    let mut foreign_id = bytes.clone();
    foreign_id[16..32].copy_from_slice(&other[16..32]);
    cases.push(("foreign-stream-id".into(), foreign_id));

    // Single-byte tampering in every region of the archive.
    let (r_start, r_end) = records[0];
    let tamper = [
        ("magic", 0),
        ("stream-id", 20),
        ("header-kind", header.0),
        ("record-kind", r_start),
        ("record-aead-prefix", r_start + 5),
        ("record-nonce", r_start + 5 + 14),
        ("record-ciphertext", r_start + 5 + 40),
        ("record-tag", r_end - 1),
        ("trailer-ciphertext", trailer.0 + 5 + 30),
        ("trailer-tag", trailer.1 - 1),
    ];
    for (name, at) in tamper {
        let mut changed = bytes.clone();
        changed[at] ^= 0x01;
        cases.push((format!("flip-{name}"), changed));
    }
    for (name, kind) in [("records-as-trailer", 3u8), ("records-as-header", 1)] {
        let mut changed = bytes.clone();
        changed[r_start] = kind;
        cases.push((name.into(), changed));
    }
    for (name, len) in [("length-short", 39u32), ("length-huge", u32::MAX)] {
        let mut changed = bytes.clone();
        changed[r_start + 1..r_start + 5].copy_from_slice(&len.to_be_bytes());
        cases.push((name.into(), changed));
    }

    for (name, case) in &cases {
        rejects(directory.path(), name, case, &key_path);
    }
    let huge = cases
        .iter()
        .find(|(name, _)| name == "length-huge")
        .unwrap();
    assert!(
        rejects(directory.path(), "length-huge-message", &huge.1, &key_path)
            .message
            .contains("frame exceeds")
    );
    let truncated = rejects(
        directory.path(),
        "cut-message",
        &bytes[..trailer.0],
        &key_path,
    );
    assert!(
        truncated.message.contains("truncated"),
        "{}",
        truncated.message
    );

    let wrong = key_file(directory.path(), "wrong", &riauth::crypto::random_token(""));
    let input = archive(directory.path(), "wrong-key", &bytes);
    let output = directory.path().join("wrong-key-out");
    let error = riauth::operations::restore(&input, &wrong, &output, None).unwrap_err();
    assert!(
        error.message.contains("authentication failed"),
        "{}",
        error.message
    );
    assert!(!output.exists());

    assert_eq!(seed.snapshot(), source_before);
    // The pristine archive still restores after all of the above.
    let output = directory.path().join("pristine");
    riauth::operations::restore(
        &archive(directory.path(), "pristine", &bytes),
        &key_path,
        &output,
        None,
    )
    .unwrap();
}

/// Re-seal every frame with the real key after `edit`, recomputing the
/// transcript. This models a buggy or malicious writer that holds the key.
fn reseal(bytes: &[u8], key: &str, edit: impl Fn(u8, &mut Value)) -> Vec<u8> {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use sha2::{Digest, Sha256};
    let key: [u8; 32] = URL_SAFE_NO_PAD.decode(key).unwrap().try_into().unwrap();
    let aad = |index: usize, kind: u8| {
        let mut aad = b"riauth.backup/v3".to_vec();
        aad.extend_from_slice(&bytes[16..32]);
        aad.extend_from_slice(&(index as u64).to_be_bytes());
        aad.push(kind);
        aad
    };
    let mut out = bytes[..PREAMBLE].to_vec();
    let mut transcript = Sha256::new();
    transcript.update(&out);
    for (index, (start, end)) in frames(bytes).into_iter().enumerate() {
        let kind = bytes[start];
        let plain =
            riauth::crypto::unseal(&key, &aad(index, kind), &bytes[start + 5..end]).unwrap();
        let mut value: Value = serde_json::from_slice(&plain).unwrap();
        if kind == 3 {
            value["transcript"] = URL_SAFE_NO_PAD.encode(transcript.clone().finalize()).into();
        }
        edit(kind, &mut value);
        let sealed = riauth::crypto::seal(
            &key,
            &aad(index, kind),
            &serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
        let mut frame = vec![kind];
        frame.extend_from_slice(&(sealed.len() as u32).to_be_bytes());
        frame.extend_from_slice(&sealed);
        transcript.update(&frame);
        out.extend_from_slice(&frame);
    }
    out
}

#[test]
fn stream_restore_validates_structure_behind_authentication() {
    let seed = Seed::new(300);
    let key = riauth::crypto::random_token("");
    let (bytes, _) = seed.backup(&key, StreamOptions::default());
    let directory = tempfile::tempdir().unwrap();
    let key_path = key_file(directory.path(), "structure", &key);

    // The helper itself produces a restorable archive when nothing changes.
    let unchanged = reseal(&bytes, &key, |_, _| {});
    let output = directory.path().join("unchanged");
    riauth::operations::restore(
        &archive(directory.path(), "unchanged", &unchanged),
        &key_path,
        &output,
        None,
    )
    .unwrap();
    seed.assert_restored(&output, &seed.snapshot());

    type Edit = Box<dyn Fn(u8, &mut Value)>;
    // The last record name of the first records frame, to repeat it across a frame boundary.
    let boundary = {
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
        let raw: [u8; 32] = URL_SAFE_NO_PAD.decode(&key).unwrap().try_into().unwrap();
        let (start, end) = frames(&bytes)[1];
        let mut aad = b"riauth.backup/v3".to_vec();
        aad.extend_from_slice(&bytes[16..32]);
        aad.extend_from_slice(&1u64.to_be_bytes());
        aad.push(2);
        let plain = riauth::crypto::unseal(&raw, &aad, &bytes[start + 5..end]).unwrap();
        let frame: Value = serde_json::from_slice(&plain).unwrap();
        frame["records"].as_array().unwrap().last().unwrap()[0].clone()
    };
    let first_records = |f: fn(&mut Value)| -> Edit {
        Box::new(move |kind, value: &mut Value| {
            if kind == 2 && value["index"] == 1 {
                f(value)
            }
        })
    };
    let cases: Vec<(&str, Edit, &str)> = vec![
        (
            "duplicate-record",
            first_records(|v| {
                let first = v["records"][0].clone();
                v["records"].as_array_mut().unwrap().insert(1, first);
            }),
            "payload is invalid",
        ),
        (
            "unordered-records",
            first_records(|v| v["records"].as_array_mut().unwrap().swap(0, 1)),
            "payload is invalid",
        ),
        (
            "wrong-index",
            first_records(|v| v["index"] = 7.into()),
            "payload is invalid",
        ),
        (
            "duplicate-across-frames",
            Box::new(move |kind, v: &mut Value| {
                if kind == 2 && v["index"] == 2 {
                    v["records"][0][0] = boundary.clone();
                }
            }),
            "payload is invalid",
        ),
        (
            "records-kind-string",
            first_records(|v| v["kind"] = "trailer".into()),
            "payload is invalid",
        ),
        (
            "header-kind-string",
            Box::new(|kind, v: &mut Value| {
                if kind == 1 {
                    v["kind"] = "records".into();
                }
            }),
            "Unsupported backup format",
        ),
        (
            "empty-records",
            first_records(|v| v["records"] = serde_json::json!([])),
            "payload is invalid",
        ),
        (
            "invalid-record-key",
            first_records(|v| v["records"][0][0] = "noslash".into()),
            "record key",
        ),
        (
            "unknown-field",
            first_records(|v| v["extra"] = true.into()),
            "payload is invalid",
        ),
        (
            "record-count",
            Box::new(|kind, v: &mut Value| {
                if kind == 3 {
                    v["record_count"] = (v["record_count"].as_u64().unwrap() - 1).into();
                }
            }),
            "payload is invalid",
        ),
        (
            "frame-count",
            Box::new(|kind, v: &mut Value| {
                if kind == 3 {
                    v["frames"] = (v["frames"].as_u64().unwrap() + 1).into();
                }
            }),
            "payload is invalid",
        ),
        (
            "header-version",
            Box::new(|kind, v: &mut Value| {
                if kind == 1 {
                    v["api_version"] = "riauth.backup/v2".into();
                }
            }),
            "Unsupported backup format",
        ),
        (
            "issuer-mismatch",
            Box::new(|kind, v: &mut Value| {
                if kind == 1 {
                    v["config"]["issuer"] = "https://elsewhere.example".into();
                }
            }),
            "issuer mismatch",
        ),
        (
            "unsupported-schema",
            Box::new(|kind, v: &mut Value| {
                if kind == 2 {
                    for record in v["records"].as_array_mut().unwrap() {
                        if record[0] == "meta/schema" {
                            record[1] = 999.into();
                        }
                    }
                }
            }),
            "Unsupported backup schema",
        ),
    ];
    for (name, edit, message) in cases {
        let error = rejects(
            directory.path(),
            name,
            &reseal(&bytes, &key, edit),
            &key_path,
        );
        assert!(error.message.contains(message), "{name}: {}", error.message);
    }

    // Post-decryption restore validation is shared with v2: an archive without
    // an enabled administrator fails and never writes a usable configuration.
    let no_admin = reseal(&bytes, &key, |kind, v| {
        if kind == 2 {
            for record in v["records"].as_array_mut().unwrap() {
                if record[0].as_str().unwrap().starts_with("users/") {
                    record[1]["admin"] = false.into();
                }
            }
        }
    });
    let output = directory.path().join("no-admin-out");
    let error = riauth::operations::restore(
        &archive(directory.path(), "no-admin", &no_admin),
        &key_path,
        &output,
        None,
    )
    .unwrap_err();
    assert!(
        error.message.contains("no enabled administrator"),
        "{}",
        error.message
    );
    assert!(!output.join("riauth.toml").exists());
}

#[test]
fn stream_quotas_bound_writing_and_reading() {
    let seed = Seed::new(200);
    let key = riauth::crypto::random_token("");
    let (bytes, summary) = seed.backup(&key, StreamOptions::default());
    let exact = StreamLimits {
        max_archive_bytes: summary.bytes,
        ..Default::default()
    };
    seed.backup(
        &key,
        StreamOptions {
            limits: exact,
            ..Default::default()
        },
    );

    let short = StreamLimits {
        max_archive_bytes: summary.bytes - 1,
        ..Default::default()
    };
    let mut out = Vec::new();
    let error = seed
        .fixture
        .core
        .backup_stream(
            &seed.fixture.admin,
            &key,
            &mut out,
            StreamOptions {
                limits: short,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(error.message.contains("quota"), "{}", error.message);
    // The quota is enforced before a frame is written, never after.
    assert!(out.len() as u64 <= short.max_archive_bytes);
    assert!(out.len() < bytes.len());

    for limits in [
        StreamLimits {
            max_frame_bytes: 1024,
            ..Default::default()
        },
        StreamLimits {
            max_frame_bytes: 9 * 1024 * 1024,
            ..Default::default()
        },
        StreamLimits {
            max_archive_bytes: 8,
            ..Default::default()
        },
        StreamLimits {
            max_archive_bytes: riauth::operations::stream::MAX_ARCHIVE_BYTES + 1,
            ..Default::default()
        },
    ] {
        let mut out = Vec::new();
        let error = seed
            .fixture
            .core
            .backup_stream(
                &seed.fixture.admin,
                &key,
                &mut out,
                StreamOptions {
                    limits,
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert!(error.message.contains("Invalid backup stream limits"));
        assert!(out.is_empty());
    }

    // A smaller frame limit produces more, smaller frames and still restores.
    let small = StreamLimits {
        max_frame_bytes: 8192,
        ..Default::default()
    };
    let (small_bytes, small_summary) = seed.backup(
        &key,
        StreamOptions {
            limits: small,
            ..Default::default()
        },
    );
    assert!(small_summary.frames > summary.frames);
    assert!(frames(&small_bytes).iter().all(|(s, e)| e - s <= 8192 + 45));

    let directory = tempfile::tempdir().unwrap();
    let key_path = key_file(directory.path(), "quota", &key);
    let input = archive(directory.path(), "quota", &bytes);
    for (name, limits, message) in [
        ("read-quota", short, "quota"),
        (
            "read-frame",
            StreamLimits {
                max_frame_bytes: 4096,
                ..Default::default()
            },
            "frame exceeds",
        ),
    ] {
        let output = directory.path().join(name);
        let error = restore_stream(
            &input,
            &key_path,
            &output,
            None,
            StreamOptions {
                limits,
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(error.message.contains(message), "{name}: {}", error.message);
        assert!(!output.exists());
    }
    let small_input = archive(directory.path(), "small", &small_bytes);
    let output = directory.path().join("small-out");
    restore_stream(
        &small_input,
        &key_path,
        &output,
        None,
        StreamOptions {
            limits: small,
            ..Default::default()
        },
    )
    .unwrap();
    seed.assert_restored(&output, &seed.snapshot());

    // A record larger than the configured frame fails at the source-page
    // boundary, before it is decoded or serialized into a frame.
    seed.fixture
        .core
        .store
        .write(|tx| tx.put("backup_payload", "large", &"x".repeat(16 * 1024)))
        .unwrap();
    let mut out = Vec::new();
    let error = seed
        .fixture
        .core
        .backup_stream(
            &seed.fixture.admin,
            &key,
            &mut out,
            StreamOptions {
                limits: small,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(
        error.message.contains("snapshot page limit"),
        "{}",
        error.message
    );
}

#[test]
fn stream_cancellation_and_sink_failures_leave_unrestorable_partial_output() {
    let seed = Seed::new(300);
    let key = riauth::crypto::random_token("");
    let directory = tempfile::tempdir().unwrap();
    let key_path = key_file(directory.path(), "cancel", &key);

    let cancel = AtomicBool::new(true);
    let mut out = Vec::new();
    let error = seed
        .fixture
        .core
        .backup_stream(
            &seed.fixture.admin,
            &key,
            &mut out,
            StreamOptions {
                cancel: Some(&cancel),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(error.code, "cancelled");
    assert!(out.is_empty());

    // Cancel from another observer once a few frames are out.
    let cancel = AtomicBool::new(false);
    let mut trip = |progress: &Progress| {
        if progress.frames == 3 {
            cancel.store(true, Ordering::Release);
        }
    };
    let mut out = Vec::new();
    let error = seed
        .fixture
        .core
        .backup_stream(
            &seed.fixture.admin,
            &key,
            &mut out,
            StreamOptions {
                cancel: Some(&cancel),
                progress: Some(&mut trip),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(error.code, "cancelled");
    assert_eq!(frames(&out).len(), 3);
    let partial = rejects(directory.path(), "cancelled", &out, &key_path);
    assert!(partial.message.contains("truncated"));

    // A sink that fails mid-stream (for example a closed connection).
    struct Failing(usize);
    impl Write for Failing {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.0 < bytes.len() {
                return Err(std::io::ErrorKind::BrokenPipe.into());
            }
            self.0 -= bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    assert!(
        seed.fixture
            .core
            .backup_stream(
                &seed.fixture.admin,
                &key,
                &mut Failing(64 * 1024),
                StreamOptions::default()
            )
            .is_err()
    );

    // Unauthorized callers produce no bytes.
    let mut out = Vec::new();
    let alice = seed.alice.clone();
    assert!(
        seed.fixture
            .core
            .backup_stream(&alice, &key, &mut out, StreamOptions::default())
            .is_err()
    );
    assert!(out.is_empty());

    // Restore cancellation before output exists, and during import.
    let (bytes, summary) = seed.backup(&key, StreamOptions::default());
    let input = archive(directory.path(), "restore-cancel", &bytes);
    let cancel = AtomicBool::new(true);
    let output = directory.path().join("cancel-verify");
    let error = restore_stream(
        &input,
        &key_path,
        &output,
        None,
        StreamOptions {
            cancel: Some(&cancel),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(error.code, "cancelled");
    assert!(!output.exists());

    let cancel = AtomicBool::new(false);
    let mut saw_trailer = false;
    let mut trip = |progress: &Progress| {
        if progress.phase == Phase::Verify && progress.frames == summary.frames {
            saw_trailer = true;
            cancel.store(true, Ordering::Release);
        }
    };
    let output = directory.path().join("cancel-verify-trailer");
    let error = restore_stream(
        &input,
        &key_path,
        &output,
        None,
        StreamOptions {
            cancel: Some(&cancel),
            progress: Some(&mut trip),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(saw_trailer);
    assert_eq!(error.code, "cancelled");
    assert!(!output.exists());

    let cancel = AtomicBool::new(false);
    let mut trip = |progress: &Progress| {
        if progress.phase == Phase::Import && progress.frames == 2 {
            cancel.store(true, Ordering::Release);
        }
    };
    let output = directory.path().join("cancel-import");
    let error = restore_stream(
        &input,
        &key_path,
        &output,
        None,
        StreamOptions {
            cancel: Some(&cancel),
            progress: Some(&mut trip),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(error.code, "cancelled");
    // The import transaction rolled back and no configuration was written.
    assert!(!output.join("riauth.toml").exists());
}

#[test]
fn stream_restore_import_trailer_cancellation_prevents_commit() {
    let seed = Seed::new(300);
    let key = riauth::crypto::random_token("");
    let directory = tempfile::tempdir().unwrap();
    let key_path = key_file(directory.path(), "cancel-trailer", &key);
    let (bytes, summary) = seed.backup(&key, StreamOptions::default());
    let input = archive(directory.path(), "restore-cancel-trailer", &bytes);
    let output = directory.path().join("cancel-import-trailer");
    let cancel = AtomicBool::new(false);
    let mut saw_trailer = false;
    let mut trip = |progress: &Progress| {
        if progress.phase == Phase::Import && progress.frames == summary.frames {
            saw_trailer = true;
            cancel.store(true, Ordering::Release);
        }
    };
    let error = restore_stream(
        &input,
        &key_path,
        &output,
        None,
        StreamOptions {
            cancel: Some(&cancel),
            progress: Some(&mut trip),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(saw_trailer);
    assert_eq!(error.code, "cancelled");
    assert!(!output.join("riauth.toml").exists());
}

/// Records the largest single write and interleaving with snapshot paging.
#[derive(Default)]
struct Meter {
    bytes: u64,
    writes: u64,
    largest: usize,
}

impl Write for Meter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes += bytes.len() as u64;
        self.writes += 1;
        self.largest = self.largest.max(bytes.len());
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn stream_codec_output_granularity_is_independent_of_store_size() {
    let mut largest = Vec::new();
    for extra in [200, 1600] {
        let seed = Seed::new(extra);
        let key = riauth::crypto::random_token("");
        let mut meter = Meter::default();
        let mut first_output_at_records = None;
        let mut observe = |progress: &Progress| {
            if progress.records > 0 && first_output_at_records.is_none() {
                first_output_at_records = Some(progress.records);
            }
        };
        let summary = seed
            .fixture
            .core
            .backup_stream(
                &seed.fixture.admin,
                &key,
                &mut meter,
                StreamOptions {
                    progress: Some(&mut observe),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(meter.bytes, summary.bytes);
        let first = first_output_at_records.unwrap();
        assert!(first < summary.records);
        // One sealed frame is at most the flush target plus one record.
        assert!(meter.largest < 256 * 1024 + 8 * 1024, "{}", meter.largest);
        largest.push((summary.records, summary.frames, meter.largest, first));
    }
    let (small, large) = (largest[0], largest[1]);
    assert!(
        large.0 > small.0 * 4 && large.1 > small.1 * 3,
        "{largest:?}"
    );
    assert!(large.2 <= small.2 + 8 * 1024, "{largest:?}");
    // The first sealed frame leaves after the same number of records at
    // either size: output is not deferred until the scan completes.
    assert_eq!(small.3, large.3, "{largest:?}");
}

/// Peak-RSS harness for `scripts/measure-backup-memory.sh`. Each mode runs in
/// its own process so that the seeding cost is not attributed to backup.
#[test]
#[ignore = "run through scripts/measure-backup-memory.sh"]
fn measure_backup_memory() {
    let dir = PathBuf::from(std::env::var("RIAUTH_MEASURE_DIR").unwrap());
    let mode = std::env::var("RIAUTH_MEASURE_MODE").unwrap();
    let config = riauth::config::Config {
        data_dir: dir.join("data"),
        ..Default::default()
    };
    let token_file = dir.join("admin.token");
    let key_file = dir.join("backup.key");
    match mode.as_str() {
        "seed" => {
            let records: usize = std::env::var("RIAUTH_MEASURE_RECORDS")
                .unwrap()
                .parse()
                .unwrap();
            std::fs::create_dir_all(&config.data_dir).unwrap();
            let core = riauth::core::Core::initialize(
                config,
                riauth::model::NewUser {
                    username: "admin".into(),
                    password: common::PASSWORD.into(),
                    email: None,
                    display_name: "Administrator".into(),
                    admin: true,
                },
            )
            .unwrap();
            let token = common::text(
                &core
                    .login("admin".into(), common::PASSWORD.into(), None)
                    .unwrap(),
                "session_token",
            );
            std::fs::write(&token_file, token).unwrap();
            let key = riauth::crypto::random_token("");
            riauth::config::write_private(&key_file, key.as_bytes(), false).unwrap();
            let payload = "m".repeat(1024);
            for start in (0..records).step_by(10_000) {
                core.store
                    .write(|tx| {
                        for index in start..(start + 10_000).min(records) {
                            tx.put(
                                "audit",
                                &format!("measure-{index:08}"),
                                &serde_json::json!({"payload": payload}),
                            )?;
                        }
                        Ok(())
                    })
                    .unwrap();
            }
        }
        "v3" => {
            let core = riauth::core::Core::open(config).unwrap();
            let token = std::fs::read_to_string(&token_file).unwrap();
            let file = std::fs::File::create(dir.join("archive.v3")).unwrap();
            let mut out = std::io::BufWriter::new(file);
            let key = std::fs::read_to_string(&key_file).unwrap();
            let summary = core
                .backup_stream(&token, &key, &mut out, StreamOptions::default())
                .unwrap();
            println!("R01_MEASURE {}", serde_json::to_string(&summary).unwrap());
        }
        // Baseline: page through the same records without the codec, so the
        // store's own read cache is not attributed to backup.
        "scan" => {
            let core = riauth::core::Core::open(config).unwrap();
            let mut after: Option<String> = None;
            let mut records = 0usize;
            core.store
                .read(|tx| {
                    loop {
                        let page = tx.scan::<Value>("audit", after.as_deref(), 128)?;
                        let Some((last, _)) = page.last() else {
                            break;
                        };
                        after = Some(last.clone());
                        records += page.len();
                    }
                    Ok(())
                })
                .unwrap();
            println!("R01_MEASURE {{\"scanned\":{records}}}");
        }
        "v2" => {
            let core = riauth::core::Core::open(config).unwrap();
            let token = std::fs::read_to_string(&token_file).unwrap();
            let key = std::fs::read_to_string(&key_file).unwrap();
            match core.backup(&token, &key) {
                Ok(value) => {
                    let bytes = serde_json::to_vec(&value).unwrap();
                    std::fs::write(dir.join("archive.v2"), &bytes).unwrap();
                    println!("R01_MEASURE {{\"bytes\":{}}}", bytes.len());
                }
                Err(error) => println!("R01_MEASURE {{\"error\":{:?}}}", error.message),
            }
        }
        "restore" => {
            let output = dir.join(format!("restored-{}", std::process::id()));
            restore_stream(
                &dir.join("archive.v3"),
                &key_file,
                &output,
                None,
                StreamOptions::default(),
            )
            .unwrap();
            println!("R01_MEASURE {{\"restored\":true}}");
        }
        other => panic!("unknown mode {other}"),
    }
}

/// The HTTP export streams under backpressure, holds the single export slot,
/// stops when its client hangs up, and a complete body verifies and restores.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_stream_backpressures_cancels_on_disconnect_and_restores() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use std::time::{Duration, Instant};
    use tower::ServiceExt;

    // Several MiB of records, far more than the export's 1 MiB queue.
    let seed = Seed::new(3000);
    let before = seed.snapshot();
    let key = riauth::crypto::random_token("");
    let app = riauth::api::router(seed.fixture.core.clone());
    let request = |token: Option<&str>| {
        let mut request = Request::post("/api/operations/backup/stream")
            .header("content-type", "application/json");
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        request
            .body(Body::from(
                serde_json::json!({"encryption_key": key}).to_string(),
            ))
            .unwrap()
    };

    // Callers without the backup permission get an error, never archive bytes.
    let denied = app.clone().oneshot(request(None)).await.unwrap();
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let forbidden = app
        .clone()
        .oneshot(request(Some(&seed.alice)))
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);

    // Read only the first chunk: the export then waits on its full queue.
    let admin = seed.fixture.admin.clone();
    let mut stalled = app.clone().oneshot(request(Some(&admin))).await.unwrap();
    assert_eq!(stalled.status(), StatusCode::OK);
    assert_eq!(
        stalled.headers()["x-riauth-backup-format"],
        "riauth.backup/v3"
    );
    let first = stalled
        .body_mut()
        .frame()
        .await
        .unwrap()
        .unwrap()
        .into_data()
        .unwrap();
    assert!(first.starts_with(MAGIC));
    let busy = app.clone().oneshot(request(Some(&admin))).await.unwrap();
    assert_eq!(busy.status(), StatusCode::SERVICE_UNAVAILABLE);

    // Hanging up cancels the export well before the 60 second stall timeout.
    drop(stalled);
    let started = Instant::now();
    let complete = loop {
        let response = app.clone().oneshot(request(Some(&admin))).await.unwrap();
        if response.status() == StatusCode::OK {
            break response;
        }
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "a disconnected export kept its slot"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    let bytes = complete.into_body().collect().await.unwrap().to_bytes();

    // The complete body authenticates as a whole and restores as before.
    let directory = tempfile::tempdir().unwrap();
    let key_path = key_file(directory.path(), "http", &key);
    let archive_path = archive(directory.path(), "http", &bytes);
    let verified = riauth::operations::stream::verify_file(
        &archive_path,
        &riauth::crypto::read_key(&key_path).unwrap(),
        StreamOptions::default(),
    )
    .unwrap();
    assert_eq!(verified.summary.bytes, bytes.len() as u64);
    assert!(verified.summary.records >= 3000);
    let output = directory.path().join("restored");
    riauth::operations::restore(&archive_path, &key_path, &output, None).unwrap();
    seed.assert_restored(&output, &before);
}

/// Shutting a server down cancels its running export and refuses new ones.
/// The signal belongs to that server: one started later in the same process
/// exports normally.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_stream_shutdown_cancels_export_and_restart_serves_again() {
    use axum::{
        Extension,
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use riauth::api::Shutdown;
    use tower::ServiceExt;

    let seed = Seed::new(3000);
    let key = riauth::crypto::random_token("");
    let admin = seed.fixture.admin.clone();
    let request = || {
        Request::post("/api/operations/backup/stream")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {admin}"))
            .body(Body::from(
                serde_json::json!({"encryption_key": key}).to_string(),
            ))
            .unwrap()
    };
    let server = |shutdown: &Shutdown| {
        riauth::api::router(seed.fixture.core.clone()).layer(Extension(shutdown.clone()))
    };

    let stopping = Shutdown::default();
    let first = server(&stopping);
    let mut running = first.clone().oneshot(request()).await.unwrap();
    assert_eq!(running.status(), StatusCode::OK);
    // Streaming has begun; the export now waits on its full queue.
    running.body_mut().frame().await.unwrap().unwrap();
    stopping.begin();
    assert!(
        running.into_body().collect().await.is_err(),
        "a cancelled export ended its body as if complete"
    );
    let refused = first.oneshot(request()).await.unwrap();
    assert_eq!(refused.status(), StatusCode::SERVICE_UNAVAILABLE);

    let restarted = server(&Shutdown::default())
        .oneshot(request())
        .await
        .unwrap();
    assert_eq!(restarted.status(), StatusCode::OK);
    let bytes = restarted.into_body().collect().await.unwrap().to_bytes();
    let directory = tempfile::tempdir().unwrap();
    let key_path = key_file(directory.path(), "restart", &key);
    let verified = riauth::operations::stream::verify_file(
        &archive(directory.path(), "restart", &bytes),
        &riauth::crypto::read_key(&key_path).unwrap(),
        StreamOptions::default(),
    )
    .unwrap();
    assert_eq!(verified.summary.bytes, bytes.len() as u64);
}

/// Every started export leaves a durable `started` event and exactly one
/// terminal event for its actor and stream ID. A response cut short, here a
/// small archive already queued when its server shuts down, is recorded as
/// cancelled and never as completed; a whole archive is recorded as completed
/// with the size and transcript the client verifies. No event holds the key.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_stream_audit_records_completion_only_for_a_whole_archive() {
    use axum::{
        Extension,
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use riauth::{api::Shutdown, model::Audit, operations::stream::preamble_stream_id};
    use std::time::Duration;
    use tower::ServiceExt;

    let seed = Seed::new(0);
    let core = seed.fixture.core.clone();
    let key = riauth::crypto::random_token("");
    let admin = seed.fixture.admin.clone();
    let request = || {
        Request::post("/api/operations/backup/stream")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {admin}"))
            .body(Body::from(
                serde_json::json!({"encryption_key": key}).to_string(),
            ))
            .unwrap()
    };
    let server =
        |shutdown: &Shutdown| riauth::api::router(core.clone()).layer(Extension(shutdown.clone()));
    let events = |stream_id: &str| {
        let target = format!("backup/{stream_id}");
        let mut events: Vec<Audit> = core
            .store
            .list::<Audit>("audit")
            .unwrap()
            .into_iter()
            .map(|(_, event)| event)
            .filter(|event| event.target == target)
            .collect();
        events.sort_by(|a, b| a.action.cmp(&b.action));
        events
    };
    let actions = |events: &[Audit]| {
        events
            .iter()
            .map(|event| event.action.clone())
            .collect::<Vec<_>>()
    };

    let stopping = Shutdown::default();
    let mut cut = server(&stopping).oneshot(request()).await.unwrap();
    assert_eq!(cut.status(), StatusCode::OK);
    let mut preamble = Vec::new();
    while preamble.len() < 32 {
        let frame = cut.body_mut().frame().await.unwrap().unwrap();
        preamble.extend_from_slice(&frame.into_data().unwrap());
    }
    let cut_id = preamble_stream_id(&preamble).unwrap();
    // The small export finishes queueing its whole archive meanwhile.
    tokio::time::sleep(Duration::from_millis(200)).await;
    stopping.begin();
    assert!(cut.into_body().collect().await.is_err());
    let cut_events = events(&cut_id);
    assert_eq!(
        actions(&cut_events),
        ["operations.backup.cancelled", "operations.backup.started"]
    );
    assert_eq!(cut_events[0].details["reason"], "server is shutting down");

    let whole = server(&Shutdown::default())
        .oneshot(request())
        .await
        .unwrap();
    assert_eq!(whole.status(), StatusCode::OK);
    let bytes = whole.into_body().collect().await.unwrap().to_bytes();
    let directory = tempfile::tempdir().unwrap();
    let key_path = key_file(directory.path(), "audited", &key);
    let verified = riauth::operations::stream::verify_file(
        &archive(directory.path(), "audited", &bytes),
        &riauth::crypto::read_key(&key_path).unwrap(),
        StreamOptions::default(),
    )
    .unwrap();
    let whole_events = events(&verified.summary.stream_id);
    assert_eq!(
        actions(&whole_events),
        ["operations.backup.completed", "operations.backup.started"]
    );
    let admin_id = core
        .store
        .get::<String>("usernames", "admin")
        .unwrap()
        .unwrap();
    assert!(
        whole_events
            .iter()
            .chain(&cut_events)
            .all(|event| event.actor == admin_id)
    );
    assert_eq!(whole_events[0].details["bytes"], bytes.len() as u64);
    assert_eq!(
        whole_events[0].details["transcript"],
        verified.summary.transcript
    );
    let audit = serde_json::to_string(&core.store.list::<Value>("audit").unwrap()).unwrap();
    assert!(
        !audit.contains(key.as_str()),
        "an audit event holds the backup key"
    );
}
