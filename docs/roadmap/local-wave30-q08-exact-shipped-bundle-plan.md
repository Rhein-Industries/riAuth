# Q08 exact shipped bundle: source-first plan

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. Original Q08 task:
`4f8fc732-549d-4202-b3d7-60288a9f690a`. Reservation:
`wave30_Q08_exact_shipped_bundle_source_plan`, 2026-10-03. Supporting worktree:
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`, starting at
`e87be4122c624d58c5eb3f6a2120c0688ea8257e`.

Recommendation: obtain and bind one exact shipped release cohort before reserving
the existing build-free bundle verifier. No concrete current product or release
path defect was demonstrated by this source review. The supplied accepted local
artifacts do not establish an exact shipped release, and this review did not query
whether a newer external release exists. No product implementation is proposed.
Q08 disposition and its original assignment remain root-owned; this report does
not recommend completion from source definitions or local artifact passes alone.

## Original row and provenance

The exact row in the project `planning/current-tasks.json` requests:

> Verify Essentials, Platform, supported backends/architectures, excluded capabilities, and incompatible-configuration rejection.

Its prerequisites are A05, A06, A08, S05 and Q02. Its workstream gate says that
security, compatibility, speed and recovery claims each need the appropriate
evidence; passing one category does not establish the others. The row also
requires relevant implementation, tests, documentation and released artifacts as
applicable, with actual verification and external prerequisites reported.

The export is at
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/current-tasks.json`.
Two reads during this audit had different file identities:

| Export observation | Bytes | SHA256 | File mtime, UTC |
| --- | ---: | --- | --- |
| Initial read | 243722 | `b17fca3cbecf381938c7cf7371c1fd46c76dfab7c8816fde9da6accc09eb1dec` | 2026-10-03 05:30:12.150602 |
| Later read | 244354 | `0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835` | 2026-10-03 05:51:05.061711 |

Both selected Q08 rows retain `status=todo`, original worktree
`b64ea3dc-db1c-4bf8-b82b-990a52a440fe`, creation epoch `1790534568`
(2026-09-27 18:42:48 UTC) and update epoch `1790894718`
(2026-10-01 22:45:18 UTC). A fresh export-file mtime is not a freshly updated row
or a live board API observation. No callable RiWork task reader was exposed in
tool discovery, so these are filesystem-export observations. The older
`delegation-current-tasks.json` was inspected for metadata only, not used as a
fresh row or release baseline.

The later export marks A05, A06, A08, S05 and Q02 done. That records their board
states; it does not substitute for artifact evidence. Their original outcomes
are, respectively: two builds from one revision with additive optional features;
usable capability advertisement and unavailable-module refusal; preserved
identifiers/configuration and safe downgrade refusal; the same identity and
management contracts on redb/PostgreSQL; and shared credential/session/authority,
plan/apply, retry, permission, secret and audit contracts. Their dated scheduling
text is superseded by the latest finish-everything authorization. No status or
assignment was changed here.

## Fixed source and advertised release scope

All published-source inspection used immutable Git objects at
`6a4066c72ac58225cd444a806032eef8515e646b`. No source alignment or historical
worker-file import occurred. Applicable repository guidance was checked; no
`AGENTS.md` was found in this worktree or its applicable ancestor directories.

The full release notes, limitations, edition guide and Q08/Q10/Q11 reports were
read. The release notes advertise native Linux `x86_64` and `aarch64` release
archives. The workflow uses `x86_64-unknown-linux-gnu` / `linux/amd64` and
`aarch64-unknown-linux-gnu` / `linux/arm64`, on native Linux runner jobs. Other
operating systems are source-build territory; the macOS local Q08 matrix is not
a shipped Linux artifact. No Windows package, tenant, browser or external host
profile is invented for this gate.

Each architecture has exactly twelve release files:

| Files per architecture | Count |
| --- | ---: |
| Essentials and Platform server archives | 2 |
| Essentials and Platform maintenance archives | 2 |
| Standalone base `riauthctl` archive | 1 |
| Essentials and Platform saved-image archives | 2 |
| Separate Essentials, Platform and client package SPDX documents | 3 |
| `build-provenance-linux-ARCH.json` and `SHA256SUMS-linux-ARCH` | 2 |

Thus the combined bundle has 24 files. Server and maintenance builds explicitly
use no default features: Essentials records `["essentials"]`, Platform records
`["essentials", "platform"]`. The standalone client records
`no-default-features`; terminal USB support is a separate optional feature.
Artifact capability metadata describes compiled functionality, with `usable`
unset in artifact scope; live configured usability requires instance evidence.
A capability count or a version string alone does not identify the bytes.

The root and client manifests both say `0.1.1`; the fixed docs distinguish old
v0.1.1 backup readers from later unreleased v3-capable builds sharing that string.
No new tag, version, key, release or signing identity is inferred from it.

## Complete source bodies reviewed

The following complete bodies were read at the fixed pin. SHA256 values identify
source bytes, not executed artifacts or successful checks.

| Source | Bytes | SHA256 |
| --- | ---: | --- |
| `.github/workflows/release.yml` | 9864 | `da8607c5ea33a906185141e0caebec18c2282c22676981e0c659fd65eef4369f` |
| `.github/workflows/ci.yml` | 11879 | `fc5245ef15f0ac8d0a460b71ded533ad96ad3dcbd795b31f38b3e84221940b5a` |
| `.github/workflows/check-local-artifacts.yml` | 19487 | `62c378975021066d62d5b32d0eee2c87de8501d01ded1ae0fae8147e920e58d7` |
| `.github/workflows/check-local-container-cohort.yml` | 4574 | `3cf3d01fa751e418673b79dabed4e3edc7f44dd0a395d3fb2b58e98585743851` |
| `scripts/package-release.sh` | 4762 | `3060b16559b004e6e213d7ed484f575a7375df3d92d29b39fcf0726f936a2f0a` |
| `scripts/check-release-bundle.py` | 6473 | `3272d70201eb8eb91039e90f87a69b694d6ee7fd95cd5f1f4d70b990c6d7a001` |
| `scripts/check-installed-release-gate.py` | 22321 | `cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5` |
| `scripts/check-edition-artifacts.py` | 13118 | `2f477665ed584bc148fb230c0658f6d0cc66aceaa5af631c4dc467872fa9ecce` |
| `scripts/check-exact-edition-matrix.py` | 18394 | `f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887` |
| `scripts/check-release-attestation.py` | 20918 | `c239cdabd3f3915ce902ccad8c96b55900bb2f56781c9340ffb8ac75fd4952f3` |
| `scripts/check-release-evidence.py` | 19722 | `bf0c6c2a7c748dd9a7164c71f50cb191feb2a93ea8acf0a751deec5377771532` |
| `scripts/release-version.py` | 459 | `2163723d91b5f122d9287f35c4ecac7b6b6f21b9d187a2d0c374ccf2aa370e61` |
| `tests/test_installed_release_gate.py` | 5911 | `9bfc4130a0b4859825691b24ae3e9b155dab6a273ca066ebf35a479429828a12` |
| `Dockerfile` | 1512 | `458ebb247170c6d4af2ff45b22e5a36e0a5380612140a43448d2ddcebc5d83cb` |

Root/client manifests and the Rust toolchain file were also read completely;
lockfiles were inspected for byte identity, not claimed as a reviewed dependency
graph. The fixed lock hashes are server
`b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426`
and client `2998555ddb2d00e8130a970fcacfed19dfee7a0b2e3305011ebd40746f67b4db`.
`rust-toolchain.toml` selects Rust 1.98.1, SHA256
`887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167`.

Additional scoped body reads, not whole-file review claims:

- `scripts/spdx_sbom.py`: package graph/binding/producer functions from
  `LINUX_TARGETS` through the command dispatcher; its complete file is 36727 bytes,
  SHA256 `ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c`.
- `src/node_security.rs`: production agreement/validation/adoption and transition
  logic; `src/edition.rs`: complete body; `src/edition/transition.rs`: function
  inventory, snapshot-token and activation transaction bodies. Relevant rate
  configuration and live principal checks were read in `src/config.rs` and
  `src/agent.rs`, without claiming whole-file review.
- `scripts/check-local-container-cohort.py`: whole-file identity and method
  inventory, fixture and logical-preservation methods; not a repeat of the full
  prior A09 controller audit. The fixed file is 138392 bytes, SHA256
  `6726dfd945445873214e934aa1f20815dc99ab24660b2205a45fd43c1486f221`.
- `scripts/check-local-edition-transition-postgres.py`: format agreement,
  complete refusal-snapshot construction, ordinary shared probe, stable expiry,
  configuration-refusal and relevant setup/transition bodies. The older encrypted
  helper was read only for reused fixture/format metadata, not executed.
- `docs/disaster-recovery.md`: the binary/edition/release/backup-format selection
  section. README, getting-started and documentation index references supplied
  target context; no broader recovery or all-doc audit is claimed.

The five bundle/SPDX/installed/edition/attestation files in this worktree equal
their fixed published objects. Also, the installed checker, edition checker,
exact-matrix checker, node-security, edition, edition-transition and Dockerfile
objects are each byte-equal at 6a4066, historical b619 and historical 9a819.
This seven-file equality does not make the three complete products, compiled
binaries, manifests or release identities equal.

## What the existing gates actually establish

`release.yml` validates the tag against the manifest and current main, invokes CI,
builds both native architectures with explicit editions, checks the archives and
images, runs the installed gate, uploads artifacts and requests four groups of
attestations. The combined publish job verifies the complete bundle and creates
a **draft** release. These are definitions; no tag job or publication was run here.

The packager requires commit/repository/run/attempt and image identities, emits
five native archives and two image archives per architecture, package SPDX
documents bound to their byte hashes, v4 build provenance and checksums. An SPDX
producer's graph/hash statement is not a cryptographic release attestation.

The bundle verifier checks exact filename/checksum sets, all asset bytes, package
SPDX graph markers and archive hashes, commit/repository/run/attempt/native target
provenance, explicit edition features, maintenance binary hashes, base-client
features and cross-architecture revision/toolchain/lock agreement. It hashes
images as opaque archives; it does not load them. It does not independently
authenticate a signer, prove native behavior, or compare recorded lock hashes
against a selected checkout. The installed gate adds that checkout/lock binding.

The installed gate requires the requested commit to equal its checkout, the
exact twelve per-architecture assets to be non-symlink regular files, all
checksums/SPDX/provenance and lock hashes to match, and native Linux of the selected
architecture before binary execution. Its archive installer requires exactly
three regular members and caps the selected binary at 256 MiB. It compares
server/maintenance/client metadata, artifact feature states and edition/version
agreement; it then exercises explicit E-P-E handoff, direct downgrade refusal,
same-build isolated backup/restore, closed serving before recovery completion,
old-session rejection and fresh login/JWKS. This is not an earlier-version binary
rollback or a PostgreSQL restore test.

The edition checker exercises shared-route agreement, absent Platform routes in
Essentials, forbidden Essentials agent scopes/parent forms, retained Platform
state refusal and the loaded-image equivalent. It trusts its producer-supplied
input paths; an operator should bind shipped bytes before executing them.
The local exact-matrix script instead rebuilds editions, checks the declared
Platform-only capability/dependency difference, and exercises fresh redb plus
optional local PostgreSQL/configuration smoke. Its nine-name release inventory
is explicitly `present_unverified` and predates the three SPDX files. Rebuilding
it is not the proposed shipped-bundle gate.

The attestation verifier's offline mode binds a local artifact hash, supplied
bundle/trusted root, repository, tag/workflow, source digest and release run/
attempt. It requires cryptographically verified certificate source identity and
hosted-runner/run invocation before trusting workflow-controlled predicate data.
Its source-contract mode and the release-evidence source audit cannot prove that
signing or publication occurred. Neither mode was invoked here.

## Accepted evidence and its exact limits

Complete bodies of the following fixed published receipts/reports were read.
The execution statements below are attributed to those accepted bodies. This
audit did not open/download their artifact ZIPs, rehash inner binaries, replay
commands or inspect private rows, keys, tokens or logs.

| Evidence | Actual attribution | What remains distinct |
| --- | --- | --- |
| Native x86 receipt, run `37046857550`, job `110970324302` | Source `b619fe25269ccc150e473bbcde47cdb3623ef810`; native Linux x86 ELF identity, five LOCAL archives, capability/license agreement and edition archive smoke; eleven commands exit 0 | Artifact `11246575279`, 52428080 bytes, API digest `7c1bcad0f78d90eb611ab76d67943a32a0d691cd5bbd9e5588512bdde635e2a0`; this receipt did not rehash the outer ZIP; no official release or full shared gate |
| Native ARM receipt, run `37016520583`, job `110868629053` | Source `9a819317efb3a13fa27cd86f884be2be00898fc0`; native Linux aarch64 ELF identity, five LOCAL archives and edition smoke; eleven commands exit 0 | Artifact `11232871527`, API digest `fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30`; outer rehash not claimed there; source differs from x86; no official release |
| Native PG receipt, run `37082572962`, job `111086077482` | Reuses exact 9a819 ARM binaries; native PostgreSQL 16.15, plaintext connection fixture; E-P-E, format 3/all sixteen rates, ordinary user/group/grant/audit agreement, denied administration, sampled nonrenewal/logout and configuration/other-client refusals; validator exit 0 | Output artifact `11259328231`, retained outer SHA256 `2fb5052a1ee04d9aafbda4cfbad334ea34d7d3065f6468d44d9bc1a26e28a46b`; no TLS/HA/official-release claim; full shared gate not certified |
| Linux bind-replay receipt, run `37099059596`, job `111134825816` | Current controller `e17f723c211b9cba4c8f07257f3e40560325c223` reuses b619 x86 images; eleven fixed checks passed, including UID 10001/private encrypted copy-up, format 3/rates, ordinary identity/refusal/logout, logical preservation, exact-plan E-P-E and incompatible open refusal | Artifact `11265212727`, outer SHA256 `4afb3c28549f918e032d201a1b58f86fd2660adaac4e0e7dc8764025d8735435`; no image build this run, official release false and full shared gate not run; not ARM execution |
| Q08 2026-09-29 macOS source matrix | Local source `4ca7558ee8563a0c5eedc1f13d4a9a6faa6a8d4a`, then-current compiled/dependency catalog, fresh redb/PG smoke and unavailable-module refusals | Source-built macOS binaries, historical catalog counts, no shipped Linux cohort; dated draft-release inspection is not a present release query |
| Q08 native ARM local report | Source `6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd`, native local identities/backend smoke and same-edition recovery | Historical cross-edition archive/image/installed attempts failed before Platform serve; those failures remain failed, not rewritten by later passes |
| A08 same-edition recovery and encrypted inventory | Later source `f3aba63ac3b824843a40b99623f1619ef8edc19f`: redb v3 isolated restore, signing-key preservation, serving gate, old-session rejection/new admin; encrypted redb/PG transition and PG restore records retained | Local dated binaries and validators, `release_gate_result=false`; no current/previous shipped pair, deployed recovery, external escrow, PG TLS/PITR or production topology inferred |
| Q10 installed-release report | Existing strict installed gate and metadata fixture definitions; recorded missing exact release assets stopped a placeholder invocation before native verification | Source readiness and historical local recovery do not establish installed checks on a named current shipped cohort or an earlier-version rollback |

The native PG shared probe compares the ordinary session's exact `expires_at`
before and after a later `whoami`, then requires logout to make the old token
return 401. This is a sampled nonrenewal/logout result, not natural-TTL expiry or
all-session-duration evidence. The current container snapshot intentionally omits
new session ID/expiry values when comparing user representations; its logged-out
token refusal should not be relabeled as that PG expiry sample.

Current node-security source enforces format 3, issuer, active capabilities,
authentication settings and all sixteen effective HTTP rate categories. Missing,
legacy, future or malformed agreement is refused until a distinct explicit
adoption operation; ordinary startup and an edition handoff do not silently adopt
policy. Transition activation binds the candidate configuration and full durable
snapshot token under the writer transaction, preserves shared policy/rates and
credential fields, and refuses incompatible retained resources. PostgreSQL also
checks other clients. The dated encrypted helper's format-2 assertion is not a
current format-3 oracle; the accepted PG helper deliberately reuses only its
fixture function. Do not run the old helper unchanged as a shipped gate.

The successful container receipt preserves prior failures, including the two
Docker Desktop UID/GID 0:0 postcondition failures and the later physical-file
comparison failure. Physical redb equality is not logical-row equality:
successful replay records all ten physical comparisons false with exact logical
preservation. This review does not retrospectively assign unknown earlier native
causes. Build-free archive validation (including the accepted 42-case pass) proves
archive structure/binding only, not container UID, identity or application behavior.

## Smallest next reservation, if exact inputs become available

The single first gate should be the existing **build-free** verifier, using
supplied immutable assets, with no rebuild, image load or product execution:

```sh
python3 scripts/check-release-bundle.py "$EXACT_DIST" "$RELEASE_COMMIT" "$RELEASE_REPOSITORY" "$RELEASE_RUN_ID" "$RELEASE_RUN_ATTEMPT"
```

These are unbound required inputs, not invented current values. Before root can
release that one command, it needs:

1. One actual release/tag and full source commit/repository, producer run ID and
   attempt, with a retained release asset manifest identifying both architectures.
   The two different LOCAL native source commits above cannot fill this slot.
2. The exact 24-file directory, transport and file sizes/SHA256 values, producer
   checksum/provenance/SPDX documents, immutable regular-file ownership and no
   symlink or concurrent writer. No bytes may be repaired or repackaged to pass.
3. The exact checker and imported SPDX source. The reviewed pair is fixed 6a4066,
   hashes `3272d702...c6d7a001` and `ca063ab3...a1032c` above; both currently equal
   this worktree. Root must bind that pair to the selected release format rather
   than assume every future release keeps v4 or Rust 1.98.1.
4. Signer/workflow/ref/source/run identity, attestation bundle(s), artifact-subject
   digests and the approved trusted root needed to authenticate the claimed
   release. Plain checksums and producer provenance cannot supply that trust.
   An attestation invocation is a separately reserved step, not part of this
   proposed one command.

Prospective supervision: one pinned installed Python child, at most 120 seconds
with a 125-second outer acceptance deadline checked after joined output and saved
receipt; 2-second disk observations, start at least 10 GiB, own stop at 9 GiB and
floor 8 GiB. A conservative additional **1 GiB disk planning allowance** is conditional
on the actual supplied asset/metadata inventory and producer-sized maintenance
archive member counts, not a guaranteed peak or an input-memory quota. The
verifier streams byte hashes in 1 MiB chunks, but loads JSON and tar metadata;
their real sizes must be examined before that estimate is credible. If that
inventory or capacity is unavailable, refuse before invocation. No new Cargo
target or copied archive directory is needed for this read-only command.
No runtime, memory peak or disk peak was measured in this source phase.

The future wrapper must use exclusive private 0600 bounded captures (proposed
16 MiB), record complete bounded stdout/stderr, numeric exit, elapsed time and
disk observations before grading, and emit only fixed public failure classes.
Malformed/missing/extra/checksum/SPDX/native-provenance/feature/source/run/
cross-architecture disagreement should fail visibly. On first unexpected result,
timeout, capacity/capture/cleanup failure, stop; do not retry, rebuild, alter
oracles, rewrite archives or reduce thresholds. Cleanup is only its captured
owned process group with TERM/KILL as required, consuming wait/reap and absence
proof. Preserve its log/receipt and every supplied asset unchanged. No daemon,
container, listener, global prune or cache deletion belongs to this gate.

A success would establish complete byte/provenance consistency for the named
cohort. It would not establish signer trust, backend/native execution, recovery or
Q08 completion. After root authenticates and binds those bytes, the existing
edition and installed gates can be separately reserved on the advertised native
Linux targets. Existing backend/format/refusal coverage can then be matched to
the exact shipped binaries rather than replaced by a universal optional-profile
campaign. A previous-version binary/backup baseline is needed only for an actual
rollback claim; none is supplied here. No tenant, Windows, all-browser, every-host,
performance, TLS/HA or deployment campaign is added to Q08's original row.

## Static checks and handoff boundary

Actual work in this reservation: immutable Git body reads, filesystem export and
file metadata/hash inspection, selected historical-source equivalence, and static
report/source-scope checks. No validator/module import, unit fixture, native
binary, version probe, Cargo, Docker, provider, network/release query, artifact
download, service, browser or desktop action ran. No runtime lane was acquired or
released. Current shipped artifact availability remains unobserved.

Static checks completed with exit 0: `git diff --check`; strict UTF-8, final
newline, paired Markdown code fences, no trailing whitespace or merge markers;
the sole-new-file allowlist and empty existing tracked/staged diff; the fixed
source/hash/equivalence comparisons described above; and the unchanged complete
S02 report byte/hash check. No test execution is included in these results.

Only this new report is authorized to change. All production/manifests/tests/
workflows/helpers and existing reports remain unchanged; parent history is
preserved. In particular the S02 report remains 283833 bytes, SHA256
`3f8929fc6f5bbdd7f45c4af80bf4319ec8715fb07f7ae0e874e1080d753f9921`,
with its actual PASS and every local/CI failed receipt intact. Credential issuance,
route-specific receipt headers, PAM fallback, Group, authorization, revocation,
audit, removal and the existing 60-second protocol limits were not modified.

Root's next decision is input provenance and a concrete finite reservation, not
a new task or a source rewrite. Q08 stays at its original exported todo/assignment
for this handoff; no whole-task completion, release or publication claim is made.
