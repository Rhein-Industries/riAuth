# A09 prebuilt x86 replay: independent source review

Date: 2026-10-03. Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Reservation: `wave30_A09_prebuilt_replay_independent_source_review`.
Existing worktree: `ed9ac424-59f4-4520-905b-919aea3521eb`; report parent
`5f1d674f1ed0c958177250a9a17342a64bdddb29`.

**Recommendation: accept the exact proposed source for root's separately reserved
implementation review. No concrete additive source blocker found.** This is not
a replay, a successful container gate, or current-build evidence. Both candidates
remain unexecuted. Runtime, integration, publication, and task disposition belong
to root. No runtime slot was acquired or released by this review.

## Immutable source and reconstruction

The reviewed proposal is commit
`33bff929ec2087cfe1eb0021b563d8e09b010165`,
`docs/roadmap/local-wave30-a09-container-offline-refusal-plan.md`, against parent
`d1352c5653aa49b9f83e39b8903535ededea4d70`. Its complete 882-line addition occupies
lines 2720–3601. The preceding 163,146 bytes remain an exact prefix; those older
2,719 lines were not all reread in this review.

| Source/data object | Bytes / lines | SHA-256 |
| --- | --- | --- |
| Complete proposal report | 216289 / 3601 | `6792be92084fca37ce17aa50dd02602f1768c3ad1b95661eb73baec81d6099b0` |
| Current helper | 120263 / 2071 | `a9b91ea849840daee8fe25791e6751e77ec355fdd9e628ced2e317f00c9e74dd` |
| Complete helper diff, proposal lines 3033–3326 | 19701 / 294 | `c56e540762ddb426a5b9354d4c9b4f1d0c0259ac3ac2fb9a5a47c02b2a6981f4` |
| Reconstructed helper | 138268 / 2335 | `9f95975a70caf47c06b224226134649f552169714f9ebf3b54b4b55da98f5908` |
| Current workflow | 4573 / 109 | `9af6d1d40543bd9604d7105730b97cec9c665d0f197548e50ba9a87eeb6007d4` |
| Complete workflow diff, proposal lines 3332–3342 | 525 / 11 | `d1e2721981aa4d4b35e8515dcb68c4805870f3a77b10f6f6ec6b57d47c8207ea` |
| Reconstructed workflow / complete supplied fence | 4574 / 109 | `3cf3d01fa751e418673b79dabed4e3edc7f44dd0a395d3fb2b58e98585743851` |

The current helper is byte-identical at author
`f00756f318568b125b13b4391488eb4fc184fa9d`, staged
`66064a6b4df4f925bde371e606be65a405e9337a`, and root
`238dc6c0f24e372a677f971c76944527d6d52629`. Both full diffs were applied only to
in-memory data, then reversed to the entire original files: 12 helper hunks and
three workflow hunks. The workflow also equals its complete supplied fence.
Python AST parsing accepted both helper sources without importing either.

The new `import_prebuilt_x86` method is candidate lines 1346–1591. Its
decorator-aware method span is 17,126 bytes, SHA
`026c59132d39d6625d0e705f9fe5cc52bb9e006db2e05317c86bb9b2f132bc7c`.
The author's 17,127-byte span includes the leading blank LF at line 1345 and
matches SHA `0ff882c18027849f7e09dc524e2f6fc7b6d06c678893333b137ed54e9bf72448`.
The supplied 75-line `main` fence is 3,383 bytes, SHA
`419c840e5fcb6f1ce9c89342e76404c58349befb291d3f57278053af86aa2cb3`.

## What was read and what was compared

Full body reads covered all 2,071 current helper lines; the entire 882-line
addition, including its prose, diffs, new import, main, and workflow; all 109
original workflow lines; the original Dockerfile and `.dockerignore`; all 118
lines of `tests/edition_transition_store_probe.rs`; and the complete committed
root reader receipt, native receipt, and logical review identified below.
Consequently the fixture, transport, tar, ELF, legacy metadata, refusal, target
ownership, diagnostic, and cleanup paths were read, not inferred from hashes.

The public cohort JSON was fully parsed and hashed. Its selected provenance,
images, reader, resources, steps, creation records, cleanup, and preservation
packets were inspected; this is not a claim to have read every serialized field
as prose. All seven native input hashes and six reader input hashes were compared
with immutable product blobs. Apart from the Dockerfile, ignore file, and probe,
this was a data identity comparison, not a full-body audit of those inputs.

The original `Cohort` has 60 methods; the candidate has 61. The six modified
methods are `__init__`, `daemon_setup`, `image_identity`, `validate_image_tar`,
`expected_v28_legacy_members`, and `cleanup`. All 54 remaining method source
spans, AST bodies, and order are unchanged:

```
save capacity sample_loop check_budget process_identity kill_group command docker
inspect absent prove_absent git source_check http github_json native_transport
elf capabilities recover_builder create_volume confirm_volume create_container
confirm_container remove_container tool build_images inspect_product build_reader
reader_offline reader_io reader_file_identity parse_reader_snapshot logical_snapshot
preservation_before preservation_after maintenance shell new_fixture start_app
stop_app client login mutate token revoked snapshot revoke_copy offline_hashes
direct_refusal plan config_refusals handoff platform_refusal_fixture fixture_gate
```

All other top-level imports/constants/helpers remain AST-equivalent. The complete
main exception/finally handling and bottom main guard are unchanged. In particular,
the existing 22-line bounded `OSError` diagnostic is unchanged: this proposal
does not retrospectively recover an errno, failing site, sender, or cause.

## Historical evidence, with failed outcomes retained

Only the approved JSON evidence under `/tmp/riauth-root-a09-37087561409` was read.
No ZIP, image archive, layer, executable, raw protocol file, or private input was
opened. No remote metadata was refreshed.

| Public witness | Bytes | SHA-256 |
| --- | --- | --- |
| `artifact-api.json` | 928 | `56c37e2fe93ce4bc58ca3fdc6fb627238d006c60f22388ad1232c442b413c5bf` |
| `run-api.json` | 3611 | `d031a2c86012dcb94a9f4611edaab28bb9ddfcfc909fcc8f2bc16cc74869098e` |
| `evidence/container-cohort.json` | 460868 | `e775e6916221f32919b92eb5cef3e4eda597f6cadf80294fd271cea79c4c535f` |
| `evidence/launch.json` | 163 | `2b28472e1855b43d570fb6c9635362c8b200ff0421d1bb4fc45341e75bb1c3f5` |

The dated artifact wrapper contains one item, not a fresh API observation.
`run-api.json` is a retained CLI run summary, not a raw run endpoint response.
Run `37087561409`, job `111100895046`, completed **failure** on workflow
`3dab`; the cohort step failed and the always-upload step succeeded. Artifact
`11262263011`, `riauth-local-container-x86_64-37087561409-1`, is bound to that
failed run: 115,531,946 bytes, SHA
`78bb19d402870f934250e9dc53e99dc9f2008f18f96b4091a62cc14c1f8493b0`.

The committed witnesses were read at `66064a6b4df4f925bde371e606be65a405e9337a`:

| Witness under `docs/roadmap/` | Bytes / lines | SHA-256 |
| --- | --- | --- |
| `evidence/wave30-a09-reader-37087561409-root-review.json` | 5333 / 153 | `0cd61c168960c9220a0092219e6b18023ea7c06c62b30c193bf5ec6b047a86f2` |
| `evidence/wave30-a09-native-x86-root-review.json` | 11244 / 341 | `ccf7c3fb6c0bd85816097c32a24a1c1b0a67b9597346d571cd68975a4680b1af` |
| `local-wave30-a09-root-container-logical-review.md` | 11721 / 195 | `95c1ce6622f55a063976364ff47b302873796d2a89cebffd68bd67d76f3239ec` |

The failed cohort used product `b619`, tree `a627`, controller `3dab`, and the
older 119,193-byte helper SHA
`1ac9e1342f51decd49da5d6039d4067347825271059fea58cb08cecde887be67`.
The full source identities are product
`b619fe25269ccc150e473bbcde47cdb3623ef810`, tree
`a627df2ce21a4d255b8c1914d4f543e32f40f4de`, controller/workflow
`3dab109446b2dd773f524601bf4737bf3861cbb6`, and source review
`66c814a339665e6b3f8e6c22bc59d4f6f0aa224c`.
Its checks array is empty; fixture result is `failed_or_refused`, failure
`unexpected_OSError`, shared full gate `not_run`, and official release false.
Actual retained evidence includes 12 reader observations and seven offline
preservation packets with zero added/removed/changed rows and exact full logical
metadata/counts. Config/key/candidate preservation is true. Physical redb equality
in those refusal packets is false; this is not erased or described as byte equality.
Each separate reader operation reports physical preservation.

The two historical image builds exited zero (846.153 and 1108.151 seconds), as did
the 1090.526-second reader build. There were 1,802 owned child steps with recorded
reaped/empty groups, 149 created containers and two volumes; this does not mean all
steps exited zero. Cleanup errors and remaining CLI/container/volume counts are
empty/zero, and builder children were removed. The parsed resource summary records
1,583 samples, a 2.001190951999888-second maximum gap, and minimum 84,826,279,936
free bytes. Raw JSONL was not reread; the prior root gap-check correction remains
historical. Full E→P→E completion and a separate Platform fixture were not proved.

The separate historical native slice, run `37046857550`, job `110970324302`,
passed its 11 steps and five-product archive checks on product `b619`, workflow
`0d090169f20f61f7cb59b685dccb13203526539f`. Artifact `11246575279` was 52,428,080
bytes. Its ZIP digest was initially API-reported in the root receipt; that receipt
does not establish a root-retained outer-ZIP hash. The later cohort records its own
outer-ZIP rehash. Neither historical slice is a current official release or S02
build, and none was rerun for this report.

## Transport, source, and reader admission

The default path and two original build methods remain intact. The new flag
defaults false, requires x86 before private resource setup, and skips BuildKit
setup/builds only on the explicit replay path. The workflow's three hunks alter
description, advertised architecture, and this flag only. Action pins, permissions,
serialization, timeouts, root paths, and always-upload contract are unchanged.

Before ZIP interpretation the proposed importer binds the exact 5,333-byte root
receipt hash, fixed failed run/artifact/repository/source, metadata digest/size,
and permitted signed-storage origin. Redirects are disabled. GitHub authorization
is not forwarded to the signed storage request. Transport is streamed with an
exact compressed size cap and whole SHA check, within the existing controller
budget and an explicit 180-second bound with 15-second socket timeouts.

The exact 11-member whitelist is the cohort JSON, launch JSON, resource JSONL,
three builder logs, two build logs, the two image archives, and reader binary.
Every size/hash literal equals the sealed root manifest; total expanded bytes are
115,530,414, below the 512-MiB cap. Duplicate names, directories, encryption,
nonregular types, and unlisted names fail closed. Extraction would be into a
fresh 0700 directory using exclusive 0600 files, exact declared/copied byte counts,
and per-file hashes. This review only compared declarations; it extracted nothing.

The reader's six inputs match immutable `b619` blobs; the original native input
manifest's seven hashes also match. The complete unchanged Dockerfile hash is
`458ebb247170c6d4af2ff45b22e5a36e0a5380612140a43448d2ddcebc5d83cb`.
Recipe serialization hashes to
`a55e90aa4da02870996c9dc755bbbbc80e0a1658674b0ab3f5fb141450a07440`;
the exact 1,172-byte reader stage plus serialized recipe hashes to
`ba68e6ccb19b2421b250a75e01fa41844b772a3c892f99777f7d5e375ff8adc7`.
This binds the existing release, locked, no-default, `platform,test-support`
snapshot-reader build rather than a newly invented reader.

The exact reader is 12,199,608 bytes, SHA
`b6be98dddc96df4220e9423425ebb668a0dda0a7ac439dd7d7e73dc187e7b707`.
Its proposed admission checks regular-file type, owner, one link, initial 0600
mode, size/hash, and ELF machine 62 before enabling execution inside the private
directory. Existing immutable reader identity, read-only snapshot invocation,
owner shutdown, full collection/metadata/count comparison, physical-reader
preservation, and private config/key checks remain byte-for-byte unchanged. The
probe has other mutation arms, but this invocation selects only `Snapshot`.

## Images, ownership, and failed-load behavior

| Imported archive | Bytes | Archive SHA-256 |
| --- | --- | --- |
| `local-essentials-x86_64.docker.tar.gz` | 48936063 | `d1c47ee3a3e17864b5f18aa99dd9bb5d2a6d8c0f5c9de9d4e052f3377a6b66a5` |
| `local-platform-x86_64.docker.tar.gz` | 53552130 | `5a81a46ee436d1eb56614cdce7f572325a71dfc4bb8e5b5d43f2c0cb3c1d4347` |

The exact config/image IDs are respectively
`sha256:fc6ce9897850b8561f08b8dee9128f1bc1096f7053264086755ee6b0b0328b82`
and `sha256:ae00ecffdb5245b37927d292da80776cf9e750098072f79d0b8716c5fb669f31`.

The historical owner applies only to imported image content. Container, volume,
process, and fixture ownership remains the fresh invocation's UUID. Both exact
historical tags and image IDs, plus the entire historical-owner image inventory,
must be absent before import ownership is assigned. The proposed receipts preserve
these absence proofs, fresh resource owner, expected image IDs, per-image attempted
loads, and confirmation status.

Both archives are validated before either load. The original tar path/type/cap,
config digest, streamed ordered layers, OCI references, exact tag, source labels,
linux/amd64 profile, and narrowly reconstructed Docker 28.0.4 legacy metadata
checks remain; only their image-owner input changes. Both whole archive hashes
are checked again before loading. No unknown JSON, extra image, path, digest, or
legacy-member allowance is introduced by the replay patch.

Immediately before each load, its attempted flag is saved. Successful admission
requires the exact image ID, sole exact tag, label/source/edition/architecture,
user/entrypoint/command, copied server hash, and license/notices source binding.
Native and historical capability/version comparisons remain. The exact copied
server hashes are `62a933b0f94401e15699ced6b3ae1e18f9a06e42993d50f0f438902fcfdce78c`
and `a28827dffe27a2a42ec3fd44411ae5c343402c1a461582b17d236cb5de6dd668`.
These are historical image outputs, not the separate five-product native hashes.
Final historical-owner inventory must equal exactly the two admitted images.

Cleanup before registration cannot claim old images. After registration, deletion
requires the proved-absent exact ID/tag, a recorded attempted load, exact historical
labels, and no additional tags; it uses no-prune removal. A lost response can still
be settled only if those identity checks succeed. Unexpected replacement, extra
tags, or a registered-but-unattempted object is refused rather than adopted.
Fresh container/volume ownership and all mount/ID/creation checks remain unchanged.

**Cleanup limit:** an untagged partial-load image cannot be safely removed through
the tag-first proof. The expected-ID absence/inventory checks then fail and report
cleanup blockage; there is no successful gate or indiscriminate deletion. This is
a fail-closed residual, not proof that every failed load is removable. Absence
checks assume the existing fresh serialized host; they are not an atomic fence
against an unrelated concurrent loader or paused external IO. No broader cleanup
change or additional runtime campaign is recommended by this source review.

## Fixture/security and evidence boundaries

The complete original fixture retains its exact API and embedded CLI operations,
header/admission rules, public refusal codes, scoped user visibility, grant/group
checks, revocation, full logical snapshots, explicit handoffs, real owner shutdown,
and wrong-direct-build/refusal tests. Effective-rate config, sharedUsers protection,
and isolated Platform downgrade checks remain unchanged. No production writer,
schema, capability policy, source authority, admission contract, assertion, or
logical-snapshot exclusion changes. No current S02, official build, release,
tenant, ARM, deployed HA, or full-lifetime IO exclusion is credited.

Imported observations reset to zero: historical 12 reader observations and seven
preservation packets are build provenance only, not new checks. The receipt marks
historical failed origin, builds false and imports true. It does not claim the old
empty checks array passed. Existing first-failure status, result/refusal handling,
resource sampling, deadlines, owned child reaping, diagnostic, and receipt/upload
formats remain. Always-upload retains whatever was produced; it cannot promise
all product files after an early import failure. Cooperative checks and socket
timeouts do not imply hard cancellation of every blocked kernel IO operation.

The actual old `OSError` site/errno/cause remains **UNKNOWN**. No TIME_WAIT, port,
current-reader, or container attribution is made. The new diagnostic's presence
does not retroactively instrument that failed run.

## Actual static checks and corrections

- Git object identities, whole historical-prefix preservation, both full diff
  forward/inverse reconstructions, exact candidate/fence hashes, AST parsing,
  and all 54 protected source/AST/order comparisons passed.
- All declared 11-member sizes/hashes matched the sealed root manifest. The seven
  native/six reader input blob checks, full Docker/reader-stage recipe bindings,
  approved public JSON identities, and historical counts above passed as data
  checks. No archive content or executable was read to perform these comparisons.
- Two initial reviewer method-span assertions failed while locating the author's
  extra LF; prepending its actual leading LF established the exact recorded span.
  A later independent diff-parser check initially mishandled zero-count insertion
  coordinates. Correcting that reviewer parser, without changing candidate data,
  produced the exact complete hashes and 12/3-hunk inverses. These are static
  preparation failures, not candidate/runtime failures.
- The short tree abbreviation `a627` was ambiguous in Git. Its exact identity was
  obtained through the fixed product's tree; no source pin or candidate changed.
- Precommit digest-length checks found two report transcription errors; both were
  corrected against the immutable proposal and approved public launch JSON.
- `python3 scripts/check-docs.py` and Git whitespace/scope checks passed, including
  the completed report. No link or layout suppression was introduced.

No helper, workflow, fixture, probe, CLI, library, compiler, candidate function,
harness, Docker/PG service, network request, or GUI ran. No artifact was extracted.
No cleanup/delete operation, other-worker contact, alignment, main edit, board
mutation, or push occurred. Prior S02 report bytes remain unchanged. Root may
reserve implementation and a bounded replay separately; this report grants no
execution or closure credit.
