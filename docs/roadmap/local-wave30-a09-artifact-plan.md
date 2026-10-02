# Wave30 A09 supported-artifact audit and proposed first slice

Read-only product audit, 2026-10-02. Project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`, existing WT
`f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. The only reserved write is this
report. Product/workflow/scripts/other guides remain unchanged. D04 guide/report
are in root staging; independent D04 execution belongs to WTe1.

All current product statements below refer to immutable published source
`88790deb62d32c84fa17dceb12cd93a727224e94`, read with `git show`/`git ls-tree`/
`git grep`. Worker HEAD before this report was
`971b3a229fffd9bdd0193b6450d82900ac64da5b`; it was not merged or reset to the
published pin. Root must provide the approved source snapshot before any later
production. A report commit is not an artifact source pin or A09 completion.

## Original row and support declaration

The live original row was reread before writing. Its acceptance is: “Produce and
test server, client, container, and maintenance artifacts for explicitly
supported platforms, including Linux x86-64 and ARM64.” The workstream is
Architecture and separate builds, with prerequisites A04/A05/A08/Q08. Its shared
distribution gate is: “Switching distributions does not create different users,
different authorization rules, or silently ignored configuration.” A source
workflow, this audit, and an old local binary do not establish that gate.

[Product contracts](product-contracts.md#supported-platform-targets-and-release-gate)
declare these required targets, with a named minimum distro/libc baseline and
native execution still required before claiming support:

| Platform | Server | Remote client | Local maintenance | Server container | Evidence at the fixed pin |
| --- | --- | --- | --- | --- | --- |
| Linux x86-64 / `x86_64-unknown-linux-gnu` / `linux/amd64` | Essentials and Platform | One base `riauthctl`, shared by both editions | Separate Essentials and Platform assemblies | Essentials and Platform | Current source has the release matrix and validators. Historical generic x86-64 draft assets are for `5ef0261`, not this source; no current exact tested cohort was found in the inspected records. |
| Linux ARM64 / `aarch64-unknown-linux-gnu` / `linux/arm64` | Essentials and Platform | One base `riauthctl`, shared by both editions | Separate Essentials and Platform assemblies | Essentials and Platform | Native local five-binary/five-archive/two-image cohorts exist at `6ca4779` and `f3aba63`; their exact historical results and limits follow. No current-source cohort was produced by this audit. |
| macOS / Windows / other hosts | No declared server-release support | No declared client-release support | No declared maintenance-release support | No declared additional container target | Historical macOS source builds, P12 Windows login integration and opt-in USB hardware work do not expand this declaration. |

Platform includes Essentials; both alternative server packages execute `riauth`
and share identity/security writers. The maintenance executable is
`riauth-maintenance`, built for its edition. The base client's independent Cargo
workspace has no server/store dependency and defaults to no features; USB is an
explicit optional variant with separate hardware evidence. Containers contain
the server, LICENSE and notices, not the client or maintenance tool. The image
smoke explicitly mounts the matching maintenance binary read-only when needed.

Every required artifact still needs observed installation/startup, HTTPS and
discovery, management, passkey server verification, revocation, configuration
rejection, distribution transitions and backup/restore, plus dependency closure
without terminal USB. Passing a bounded loopback/redb slice would credit only
that slice. Published checksums/provenance are not completed signatures.

## Current production and validation wiring

The [release workflow](../../.github/workflows/release.yml) has only a `push`
`v*` tag trigger, no `workflow_dispatch`/`workflow_call` and no dispatch inputs.
It validates tag/version agreement and requires the tagged revision to equal
current main. Its native runners are `ubuntu-24.04` x86-64 and
`ubuntu-24.04-arm` ARM64, toolchain 1.98.1. It builds both server/maintenance
editions and the base client, builds each edition's architecture-specific
container, packages them, reloads saved images, executes artifact/installed
gates, requests provenance/package attestations and uploads the bundles. The
publish job combines both architectures and creates a verified-tag **draft**
release. No such action was executed here.

The [public CI workflow](../../.github/workflows/ci.yml) permits dispatch without
inputs, but has only x86-64 Ubuntu jobs and no artifact package/upload job or
native ARM64 release cohort. Its broad integration/audit campaigns cannot serve
as the requested small artifact-production dispatch. These are the only two
workflow files at the pin. Therefore **no existing tag-free dispatch produces
both Linux architectures**; inventing parameters or reusing a tag would not fix
that. A new workflow seam requires separate root review/reservation.

[Dockerfile](../../Dockerfile) pins Rust/Trixie and Debian/Trixie-slim base
digests, validates the edition argument, labels edition/revision and runs UID/GID
10001 with `/data` mode 0700. Its apt dependency resolution is not frozen by
those image digests. Trixie image execution and the Ubuntu runner declaration
are not a measured minimum libc/distro compatibility guarantee.

[The packager](../../scripts/package-release.sh) refuses a non-Linux or wrong
native architecture and requires real commit/repository/GitHub run identities.
Per architecture it emits five native archives, two saved image archives, three
SPDX package documents, build provenance and checksums: **12 files**, 24 for the
two-architecture exact bundle. Earlier Q08's nine-file inventory predates the
three package documents and must remain historical. The official bundle and
installed entry points must not be called with fabricated GitHub identities.

[The archive checker](../../scripts/check-edition-artifacts.py) accepts five
local archives without a GitHub run identity. It checks edition capabilities,
maintenance/client version, fresh local bootstrap, shared and edition-specific
routes, agent rejection and direct downgrade refusal/preflight. Its optional
image arguments require both images. It does not itself prove the whole
identity/configuration gate, HTTPS/passkeys or the base client's remote flows.

[The installed gate](../../scripts/check-installed-release-gate.py) verifies
exact assets/provenance before installation. Its reusable metadata and `drill`
helpers exercise native target/features, E→P→E offline handoff, unchanged JWKS,
login, encrypted backup, isolated restore, explicit recovery completion and old
session refusal. `drill` uses the legacy server CLI for remote calls and does not
assert unchanged ordinary-user IDs/grants or every format-3 rate category. Those
checks require a small local-only driver before claiming the shared gate.

Two existing broad/historical entry points are unsuitable for this slice:
`check-exact-edition-matrix.py` main builds debug binaries with **jobs=2** and
may start PostgreSQL; `check-local-encrypted-edition-transition.py` explicitly
requires agreement **format 2** at line 61. Current `src/node_security.rs` uses
format 3 and preserves effective rates across a handoff. Do not blindly rerun or
weaken either historical fixture and call it current-format evidence.

## Preserved historical artifacts and actual outcomes

[Earlier Q08](q08-exact-edition-bundles.md) measured macOS ARM64 source builds at
`4ca7558ee8563a0c5eedc1f13d4a9a6faa6a8d4a`, not Linux release execution. Its
dated GitHub observation found draft `v0.1.1` with four generic x86-64 assets,
provenance source `5ef0261e8b6d54e3532b69041511c84d9887648a` and Rust 1.93.0.
That observation was not repeated; no assets were downloaded here and no present
GitHub release state is asserted.

[The `6ca4779` native record](q08-native-arm64-local-6ca4779.md), binary source
`6ca4779b68cf41e29af490d2aca6ea3d59e1f2bd`, built five ELF AArch64 binaries and
five private archives/two local images in a Linux ARM64 VM. Fresh edition/backend
and same-edition recovery checks passed. Archive smoke **exit 1**, image smoke
**exit 1**, and installed cross-edition smoke **exit 1** remain failures: the
active-capability agreement refused before the expected later blocker/handoff.
These images were assembled with a private local Dockerfile, not the release
Dockerfile. The private manifest SHA-256 is
`c7063655e4ba24802913a5c945060966a40922664a1d5fd93140bc2e083bcb5a`.

[Accepted A08 native record](evidence/a08-native-linux-arm64-2026-09-29/RUN.md)
and its [final manifest](evidence/a08-native-linux-arm64-2026-09-29/native-arm64-evidence-v2.json)
identify binary source `f3aba63ac3b824843a40b99623f1619ef8edc19f`, plain validator
`730d373078c4b128373d7afa5f3e278f63b065fd`, Linux AArch64 execution and Rust
1.98.1. The manifest SHA-256 is
`638822d3c437d06678d27d4fb68987ebd2f6a884dd36c5843e9addf7727b5c7e`.
It records these product binary hashes; they are historical manifest values,
not hashes of current builds or freshly rehashed binaries:

| Binary | Bytes | Historical SHA-256 |
| --- | ---: | --- |
| Essentials `riauth` | 36,390,184 | `86a799d3060e5e602c7e458a9c409ffc8c06f881e2d426a1e6d9be226337445d` |
| Essentials `riauth-maintenance` | 13,310,224 | `c8164dfaa1a21c2bc8ac485eee02789d1aa6c8b2739904021485474cbd4350c2` |
| Platform `riauth` | 49,302,784 | `78209c1766eb53be741de0946a0e4ce4bc3f2217deda84a6e4f4bda85fc42b62` |
| Platform `riauth-maintenance` | 16,783,720 | `91f850d63a36290afe6f7b49c1943bea496b0754f50dcaa6b716af4790e8543e` |
| Base `riauthctl` | 8,101,312 | `573de0718e3301f2553145eb3d8fd01ce82bc69525a2176f075abc612c02a1e1` |

The corresponding local `linux/arm64` image IDs are Essentials
`sha256:ae84172a41d2dbe607a581d48fcde79cd1d293e00244994e257a757fd6def842`
and Platform
`sha256:a78109a5fb3bcab3e37c4a25580e5be53af6e485807abddff737cd248d7e7d6d`.
Saved-image archive hashes are respectively
`5f4347c9748a6b9b92bdf3f426282a082ab9aacf694514de43abdb2759b8ca15`
and `8ef3774017201248cd2cc1aa0d69b992d6e6990e8dc6f0981caa4de8d03a7604`.
These are local images assembled from those binaries over the pinned runtime
base, not current official containers.

The later installed E→P→E/redb, PostgreSQL 17.11, edition/dependency matrices,
same-edition recovery, archive integrity/smoke and mounted-maintenance image
checks passed at their recorded pins. Earlier archive provenance-order, image
maintenance-boundary and PostgreSQL client-drain attempts remain failed records;
the later fixture corrections are identified in RUN.md, without product rebuild.
The [encrypted supplement](evidence/a08-native-linux-arm64-2026-09-29/encrypted-storage/ENCRYPTED-RUN.md)
uses unchanged product binaries, validator
`9f8efaf0372e02b0bc0042251d591f1facc0f783` and recovery supplement
`082563b362acb218f276e4dbcaf476e303b561b7`: plain/encrypted handoffs and rejection
checks, 30/33 redb and 36/39 PostgreSQL logical rows, unchanged ciphertext and
each edition's 16 PostgreSQL logical-restore checks. Earlier PATH/revision fixture
failures remain recorded. These runs assert format 2, not today's format 3.

[The accepted integration review](evidence/a08-native-linux-arm64-2026-09-29/integration-review.json)
records 61 plain and 43 encrypted references verified, without new execution.
`d05-acceptance-evidence.json` at `c01c39ab4e092423d5522bedc50fff87656d8c0a`
credits these same historical products; c01 is an evidence/publication pin, not
their binary source. The tracked O07 plan also selects the old `f3aba63` image
and preserves host-mapping refusals. No c01/current-source official container is
inferred. `release_gate_result: false` and official identities/signatures,
previous-version rollback, deployed HA, physical recovery and external escrow
limits remain intact.

This audit only statted the seven retained archive paths named by the accepted
manifest. All were present with the recorded lengths: native archives total
48,167,504 bytes; image archives 95,497,739 bytes. No archive was extracted, no
binary/image was executed or rehashed, and no daemon was queried.

## ONE proposed production/validation slice, held for root reservation

Produce a **native Linux ARM64 five-binary/five-local-archive cohort** from exact
source `88790deb62d32c84fa17dceb12cd93a727224e94`: both servers, both matching
maintenance tools, base client. Use an immutable export approved by root, in
this WT's private artifact area on a root-reserved existing Linux ARM64 executor;
no new worker/task/WT. Verify `uname` Linux/aarch64 and ELF AArch64 execution;
cross-compilation or x86 emulation alone does not pass. No version/tag, registry,
GitHub run identity or official-release label is needed for this local slice.

After root reserves the executor/Cargo/runtime slot, the exact three sequential
build commands proposed are below. `RIAUTH_A09_ROOT` must be a newly reserved
owner-only private path; source cwd must contain the approved immutable export.
Toolchain/native cmake/clang/pkg-config/OpenSSL and the locked dependency cache
must already be available. `--offline` refuses missing prerequisites, which must
be reported before any separately authorized fetching; it is not permission to
install a toolchain/dependency implicitly.

```sh
env CARGO_TARGET_DIR="$RIAUTH_A09_ROOT/target/essentials" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 cargo +1.98.1 build --release --locked --offline --no-default-features --features essentials --bins --target aarch64-unknown-linux-gnu
env CARGO_TARGET_DIR="$RIAUTH_A09_ROOT/target/platform" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 cargo +1.98.1 build --release --locked --offline --no-default-features --features platform --bins --target aarch64-unknown-linux-gnu
env CARGO_TARGET_DIR="$RIAUTH_A09_ROOT/target/client" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 cargo +1.98.1 build --manifest-path crates/riauthctl/Cargo.toml --release --locked --offline --no-default-features --target aarch64-unknown-linux-gnu
```

Copy only the five named binaries, recording hashes/lengths/native target;
assemble five `local-*` archives containing exactly the matching binary, LICENSE
and THIRD_PARTY_NOTICES.md. Record source Git pin/export hash, both manifests and
locks, toolchain, Linux baseline, exact command/environment, each exit/log hash,
binary/archive hashes and inventory. The server lock SHA-256 is
`b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426`;
the **current** client lock SHA-256 is
`2998555ddb2d00e8130a970fcacfed19dfee7a0b2e3305011ebd40746f67b4db`, differing
from historical A08's `6f546c966c70505b8ef204cea811f7300b1b359de63f01a8389caadc3164b964`.
Do not reuse the older client as an exact-current product.

The exact existing archive target proposed, with Python optimization disabled,
is:

```sh
env -u PYTHONOPTIMIZE python3 scripts/check-edition-artifacts.py --essentials-archive "$RIAUTH_A09_ROOT/archives/local-riauth-essentials-aarch64.tar.gz" --platform-archive "$RIAUTH_A09_ROOT/archives/local-riauth-platform-aarch64.tar.gz" --essentials-maintenance-archive "$RIAUTH_A09_ROOT/archives/local-riauth-maintenance-essentials-aarch64.tar.gz" --platform-maintenance-archive "$RIAUTH_A09_ROOT/archives/local-riauth-maintenance-platform-aarch64.tar.gz" --riauthctl-archive "$RIAUTH_A09_ROOT/archives/local-riauthctl-aarch64.tar.gz"
```

For the missing shared gate, the smallest proposed follow-up source reservation
is **new `scripts/check-local-linux-artifacts.py` only**, a build-free local
driver with `--source-revision`, `--arch aarch64`, `--archives` and `--out`:

| Proposed driver hunk | Required observation and refusal boundary |
| --- | --- |
| Native/archive/provenance preflight | Refuse wrong/missing architecture, wrong source/lock/archive/binary hash, nonregular members, extra members, pre-existing output or insufficient free storage. Safely extract exact members into a fresh 0700 directory; immutable inputs stay untouched. Never generate fake `riauth.build/v4` CI identities. |
| Setup/metadata and shared flow | Reuse installed-gate metadata/drill primitives, add real base-client discovery/login/status calls with private session files. Use fresh redb and loopback only, stdin passwords, deadlines and fresh owned paths. During E→P→E use supported offline plan/activate; assert ordinary user's same ID, groups/authorization decisions, revoked session refusal and unchanged signing keys. Platform-only routes remain refused in Essentials. |
| Current configuration gate | With all fixture writers stopped, require format-3 matching record to report `recorded: false`; preserve issuer/authentication/effective shared rates through the supported handoff. Omitted/default rates must agree; changed shared rate/authentication/capability must refuse without implicit adoption or a serving listener. Check Essentials rejects valid Platform-only config, and stale transition tokens refuse. Do not edit/delete stored agreements to bypass the fence. |
| Evidence/cleanup | Fixed allowlisted results and nonsecret hashes only, 0600 report, bounded private logs, every failure retained. Stop/reap only spawned processes, clean only fresh owned fixtures. No Cargo/build/PG/container/network-fetch action in the driver and no assertion downgrade to obtain a pass. |

This driver is a proposal, not an implemented or executed checker; its final
CLI and source hash require root's exact seam review before edits. Existing
scripts, workflow, node-security/edition algorithms and operations docs need no
change for the proposed first slice. Container/current x86-64 production, HTTPS,
passkey and full packaged per-artifact gates remain separately unproved. Any
later image must record its own immutable image ID, OCI architecture, revision,
edition and embedded binary/archive digest; historical image IDs cannot fill
those fields for a current cohort.

Keep permission/removal/audit/review safeguards, route-specific optional versus
required headers, reviewed client-creation receipt-secret recovery and PAM
fallback. The fixture must not print issued secrets or change the accepted
Group/input limits. The non-renewed 60-second shared admission lease is unchanged
and is not an atomic fence against a paused process before external IO. This
local redb plan supplies no deployed multi-node/physical/tenant/escrow result and
does not duplicate R05/D01/D04 services.

## Measured resources and actual checks

Read-only metadata identified Darwin ARM64; `sysctl` at
`2026-10-02T12:52:08Z` reported 16 logical CPUs and 68,719,476,736 bytes
(64 GiB) RAM. `df -Pk .` at `12:49:42Z` reported **10,378,984 KiB
available = 9.8982 GiB**, only **1.8982 GiB above the 8 GiB floor**. This is an
instantaneous host observation, not available Linux executor memory, private
build capacity or a peak-resource bound. No build is ready on this storage.

The historical fresh `6ca4779` ARM cohort observed about 80→58.4 GiB host free
space (about 21.6 GiB incremental), jobs=4; it is not a measured peak for today's
source or the smaller slice. A provisional planning minimum is **30 GiB free**
(8 GiB floor + that observed increment, rounded up). Root must reserve storage
and an existing native Linux ARM64 executor, measure its memory/filesystems and
tool/cache availability, and reserve the exact sequential build/runtime slots.
Monitor both host backing storage and private build filesystem continuously;
stop with margin **before** either reaches 8 GiB. Record actual elapsed time,
minimum free space and peak memory if execution is later authorized. The jobs=1,
incremental=0 and dev/test debug=0 restrictions persist. No cache removal is
authorized by this prerequisite.

Performed here: live A09 row/initial exact claim, immutable Git/source/workflow
reads and hash calculation, tracked historical JSON/readme/review inspection,
seven allowlisted retained-file size stats, host `df`/`sysctl` metadata and
report-only scope/whitespace/link/pin checks. No product build/test, Cargo,
Docker query/run/build, service, workflow dispatch, download, tag/release,
registry/network mutation, desktop or status/main/push mutation. One attempted
read of nonexistent `src/cli/maintenance.rs` reported a missing Git path; the
actual maintenance entry is `src/bin/riauth-maintenance.rs`. No product failure
was hidden or rerun. Root received the initial claim and narrowed proposal via
the explicit project orchestrator, both sends exit 0.

Report validation: 13 relative links/anchors checked against the fixed Git
source, 15 file/blob mappings checked, eight historical commit objects verified,
final newline/trailing whitespace and `git diff --no-index --check` passed.
Only the reserved report was untracked/changed. The global repository docs
checker was not invoked in this read-only product audit. One timestamp wording
patch initially refused an extraneous unmatched context line and made no change;
the corrected report-only patch succeeded. This is no product runtime result.

## Immutable source hunk map

Every blob below is from `88790deb62d32c84fa17dceb12cd93a727224e94`:

| File | Git blob | Relevant seam |
| --- | --- | --- |
| `docs/roadmap/product-contracts.md` | `a9c132f4d4ed82020501cc39e2209863c6b091a9` | Package conventions and Supported-platform targets/release gate |
| `.github/workflows/release.yml` | `897df176d6dd773f9e86f8dfdb3655003c5c32ca` | Tag trigger, native matrix, package/installed/publish jobs |
| `.github/workflows/ci.yml` | `7c724fd7f4dc210a2268f5a702bdc09b1a10d76b` | Dispatch trigger and x86-only check/integration/audit jobs |
| `Dockerfile` | `f445b27204c415c4537676986a94023aaaa72f34` | Pinned bases, edition build, revision labels, nonroot server image |
| `Cargo.toml` | `5660d4bb922fcdc5bfe05d7502585980f4720d06` | Additive edition features/release profile |
| `Cargo.lock` | `f1b819d47d204d73617b095513f0c6ab6eb8aa4e` | Locked server graph |
| `crates/riauthctl/Cargo.toml` | `e97ff28b79770f55e2c1dabb2de9479705d789f0` | Independent base/opt-in USB client |
| `crates/riauthctl/Cargo.lock` | `a471c5c447c51c8d82002d7f31060e8c32099886` | Current locked client graph |
| `scripts/package-release.sh` | `2663c67109f017170412d66eddaf3126f90c089f` | Native/identity preconditions and exact package outputs |
| `scripts/check-release-bundle.py` | `37eeafd8ccd62d49ee2cc1cceb11c7003b739c43` | Both-architecture exact set/hash/provenance verification |
| `scripts/check-edition-artifacts.py` | `f88b8db533cd2e9aafd40132a06f88b4ba41b9c6` | Five-archive local smoke and optional two-image gate |
| `scripts/check-installed-release-gate.py` | `0425336649312787d6835741619943fd4dbcb4fe` | Real release entry; reusable metadata/drill helpers |
| `scripts/check-exact-edition-matrix.py` | `57a69e396b1e1efdccda4cd3b7b7c1bbc1b01cf5` | Main's jobs=2 debug builds/optional PG; config fixtures |
| `scripts/check-local-encrypted-edition-transition.py` | `aa59f2c9af46dcd055127ff2a069b468478ff15d` | Historical format-2 metadata assertion |
| `src/node_security.rs` | `da029997ad9c1a805cc3c6a1f6a50954f79a16c3` | Current format-3 rates/authentication/capability handoff checks |

A09 remains open: root's next decision is the local driver/source and executor
reservation above, followed by actual artifact production/validation and
independent acceptance. This audit cannot provide those runtime receipts.
