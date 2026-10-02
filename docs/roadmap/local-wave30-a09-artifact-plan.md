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

## Append: source-first hosted ARM64 alternative for root seam review

This append reserves **this same report only**. The original 316 lines,
23,842 bytes, remain byte-for-byte the `d4b69673b9cbdd322801ecbec61c53ab8528c08d`
report (SHA-256
`90602a657e52fc81a231173c084b8857a3d25724c6e6d45e5ec0b0497df89ac0`).
No current artifact credit, resource receipt or shared-gate completion is added.
The existing Darwin 9.8982 GiB observation/blocker remains historical and intact.

The exact proposed source seam is one **new** file,
`.github/workflows/check-local-artifacts.yml`, with the full contents below;
no hunk to an existing workflow/helper/guide is proposed. It is manually
dispatchable only, with `contents: read`, one `ubuntu-24.04-arm` job and a
required full `source_sha` input chosen and reviewed by root. The workflow
definition commit and checked-out product commit are separately recorded.
Root owns any later integration and dispatch. The input has no default: root
can select `88790deb62d32c84fa17dceb12cd93a727224e94` or a separately reviewed
later full source SHA. The running workflow definition must first be integrated
on the repository default branch for manual dispatch, per
[GitHub's manual-run documentation](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow).
No dispatch was performed or requested by this append.

### Actual provider/storage constraint, not a runner success

Read-only official documentation inspected on 2026-10-02 describes a fresh
GitHub-hosted VM per non-single-CPU job. Its public standard runner table lists
`ubuntu-24.04-arm` as ARM64, 4 vCPUs, 16 GB RAM and **14 GB SSD**.
[GitHub hosted-runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
That separates a potential remote native VM from this worker's Darwin backing
storage, but supplies **no 30 GiB available-space guarantee**. The documented
storage is smaller than the required starting threshold; no actual runner was
allocated or sampled. Therefore this alternative is currently **capacity
unestablished and infeasible against the published minimum storage contract**,
not a build-ready replacement. Its first live receipt would have to measure
both runner root/workspace and private storage and refuse if any has less than
30 GiB available. No installed-tool/cache deletion, floor reduction, different
runner label or paid/larger-runner substitution is hidden in the proposal.

The real GitHub environment exposes workflow SHA/ref, run ID/attempt and native
runner identity, while `RUNNER_TEMP` is job-scoped temporary storage.
[GitHub variables reference](https://docs.github.com/en/actions/reference/workflows-and-actions/variables).
The proposal uses those allowlisted values, not copied historical/fake CI IDs;
it never emits official-packager `riauth.build/v4`, signatures or attestations.
An upload remains a workflow evidence artifact, not a released product asset.

### Exact new-file proposal (not applied or executed)

```yaml
name: Check local ARM64 artifacts

on:
  workflow_dispatch:
    inputs:
      source_sha:
        description: Root-reviewed full 40-character product source SHA
        required: true
        type: string

permissions:
  contents: read

concurrency:
  group: riauth-local-artifacts-arm64
  cancel-in-progress: false

jobs:
  native-arm64:
    runs-on: ubuntu-24.04-arm
    timeout-minutes: 150
    defaults:
      run:
        shell: bash
    env:
      SOURCE_SHA: ${{ inputs.source_sha }}
      A09_ROOT: ${{ runner.temp }}/riauth-a09-${{ github.run_id }}-${{ github.run_attempt }}
      CARGO_BUILD_JOBS: '1'
      CARGO_INCREMENTAL: '0'
      CARGO_PROFILE_DEV_DEBUG: '0'
      CARGO_PROFILE_TEST_DEBUG: '0'
      CARGO_PROFILE_RELEASE_DEBUG: '0'
      PYTHONOPTIMIZE: '0'
    steps:
      - name: Establish owned evidence and fail-closed native capacity
        run: |
          set -euo pipefail
          umask 077
          mkdir -m 700 "$A09_ROOT"
          cat > "$A09_ROOT/driver.py" <<'PY'
          import datetime, gzip, hashlib, json, os, pathlib, platform, re
          import resource, shutil, signal, subprocess, sys, tarfile, time

          root = pathlib.Path(os.environ['A09_ROOT'])
          evidence = root / 'evidence'
          evidence.mkdir(mode=0o700, exist_ok=True)
          state = evidence / 'evidence.json'
          data = json.loads(state.read_text()) if state.exists() else {
              'schema': 'riauth.local-artifact-evidence/v1',
              'official_release': False, 'source_verified': False,
              'shared_full_gate': 'not_run', 'steps': [], 'cleanup_errors': [],
              'action_pins': {
                  'checkout': '3d3c42e5aac5ba805825da76410c181273ba90b1',
                  'rust_toolchain': '02cb101ec7c40f2c49e1d9714d64511d8e1b74de',
                  'upload': 'ea165f8d65b6e75b540449e92b4886f43607fa02'},
              'github': {key: os.environ.get(key) for key in (
                  'GITHUB_REPOSITORY', 'GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT',
                  'GITHUB_WORKFLOW_REF', 'GITHUB_WORKFLOW_SHA', 'GITHUB_SHA',
                  'GITHUB_REF', 'GITHUB_EVENT_NAME')},
              'runner': {'label': 'ubuntu-24.04-arm',
                         'system': platform.system(), 'machine': platform.machine(),
                         'environment': os.environ.get('RUNNER_ENVIRONMENT'),
                         'arch': os.environ.get('RUNNER_ARCH'),
                         'cpus': os.cpu_count(),
                         'memory_bytes': os.sysconf('SC_PAGE_SIZE') * os.sysconf('SC_PHYS_PAGES')}}
          GiB = 1024 ** 3

          def interrupted(signum, frame):
              raise InterruptedError('received_signal_' + str(signum))

          for sig in (signal.SIGINT, signal.SIGTERM):
              signal.signal(sig, interrupted)

          def save():
              temporary = state.with_suffix('.tmp')
              temporary.write_text(json.dumps(data, indent=2, sort_keys=True) + '\n')
              temporary.chmod(0o600)
              temporary.replace(state)

          def require(ok, reason):
              if not ok:
                  raise RuntimeError(reason)

          def sha(path):
              h = hashlib.sha256()
              with path.open('rb') as stream:
                  for block in iter(lambda: stream.read(1024 * 1024), b''):
                      h.update(block)
              return h.hexdigest()

          def capacity(threshold=10 * GiB):
              paths = {'host_root': pathlib.Path('/'),
                       'workspace': pathlib.Path(os.environ['GITHUB_WORKSPACE']),
                       'private': root}
              snapshot = {'at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                          'free_bytes': {label: shutil.disk_usage(path).free
                                         for label, path in paths.items()},
                          'device_ids': {label: os.stat(path).st_dev for label, path in paths.items()}}
              with (evidence / 'resources.jsonl').open('a') as output:
                  output.write(json.dumps(snapshot, sort_keys=True) + '\n')
              minima = data.setdefault('minimum_free_bytes', {})
              for label, available in snapshot['free_bytes'].items():
                  minima[label] = min(minima.get(label, available), available)
              save()
              require(min(snapshot['free_bytes'].values()) >= threshold,
                      'storage_guard: insufficient measured available bytes')

          def stop_owned(process):
              # Only a process group created here with start_new_session=True.
              for sig, grace in ((signal.SIGTERM, 2), (signal.SIGKILL, 5)):
                  try:
                      os.killpg(process.pid, sig)
                  except ProcessLookupError:
                      pass
                  except PermissionError:
                      data['cleanup_errors'].append('owned_group_signal_refused')
                  try:
                      process.wait(timeout=grace)
                      if sig == signal.SIGKILL:
                          break
                  except subprocess.TimeoutExpired:
                      if sig == signal.SIGKILL:
                          data['cleanup_errors'].append('owned_group_did_not_reap')

          def command(label, argv, seconds, env=None):
              capacity()
              log = evidence / (label + '.log')
              require(not log.exists(), 'refuse_existing_command_log')
              started = time.monotonic()
              record = {'name': label, 'argv': argv, 'status': 'running'}
              data['steps'].append(record)
              save()
              process = None
              try:
                  with log.open('xb') as output:
                      process = subprocess.Popen(argv, stdout=output, stderr=subprocess.STDOUT,
                                                 env=env, start_new_session=True)
                      while process.poll() is None:
                          capacity()
                          require(time.monotonic() - started < seconds, 'command_deadline')
                          require(log.stat().st_size <= 8 * 1024 * 1024, 'command_log_limit')
                          time.sleep(2)
                      record['exit_code'] = process.returncode
                      require(process.returncode == 0, 'command_failed: ' + label)
                      capacity()
                      require(log.stat().st_size <= 8 * 1024 * 1024, 'command_log_limit')
                  record['status'] = 'passed'
              except BaseException:
                  record['status'] = 'failed_or_interrupted'
                  raise
              finally:
                  if process is not None:
                      stop_owned(process)
                      record['exit_code'] = process.returncode
                  record['elapsed_seconds'] = round(time.monotonic() - started, 3)
                  if log.exists():
                      record['log_bytes'] = log.stat().st_size
                      record['log_sha256'] = sha(log)
                  save()
              require(not data['cleanup_errors'], 'owned_cleanup_failed')

          def source_check():
              actual = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True, timeout=10).strip()
              require(actual == os.environ['SOURCE_SHA'], 'checked_source_sha_mismatch')
              require(not subprocess.check_output(
                  ['git', 'status', '--porcelain', '--untracked-files=no'], text=True, timeout=10).strip(),
                  'tracked_source_changed')
              data['source_sha'] = actual
              data['source_tree'] = subprocess.check_output(
                  ['git', 'rev-parse', 'HEAD^{tree}'], text=True, timeout=10).strip()
              data['source_verified'] = True
              data['inputs'] = {name: sha(pathlib.Path(name)) for name in (
                  'Cargo.toml', 'Cargo.lock', 'crates/riauthctl/Cargo.toml',
                  'crates/riauthctl/Cargo.lock', 'LICENSE', 'THIRD_PARTY_NOTICES.md',
                  'scripts/check-edition-artifacts.py')}
              save()

          phase = sys.argv[1]
          try:
              if phase == 'init':
                  require(re.fullmatch('[0-9a-f]{40}', os.environ['SOURCE_SHA']), 'invalid_full_source_sha')
                  require(platform.system() == 'Linux' and platform.machine() == 'aarch64'
                          and os.environ.get('RUNNER_ARCH') == 'ARM64'
                          and os.environ.get('RUNNER_ENVIRONMENT') == 'github-hosted', 'not_native_hosted_arm64')
                  require(os.environ.get('GITHUB_EVENT_NAME') == 'workflow_dispatch'
                          and os.environ.get('GITHUB_REF_TYPE') == 'branch', 'manual_branch_dispatch_required')
                  require(re.fullmatch('[0-9a-f]{40}', os.environ.get('GITHUB_WORKFLOW_SHA', '')),
                          'missing_real_workflow_sha')
                  for key in ('GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT'):
                      require(re.fullmatch('[1-9][0-9]*', os.environ.get(key, '')), 'missing_real_run_identity')
                  data['requested_source_sha'] = os.environ['SOURCE_SHA']
                  capacity(30 * GiB)
              elif phase == 'verify':
                  source_check()
                  capacity(30 * GiB)
              elif phase == 'dependencies':
                  capacity(30 * GiB)
                  command('apt-update', ['sudo', 'apt-get', 'update'], 240)
                  command('apt-install', ['sudo', 'env', 'DEBIAN_FRONTEND=noninteractive',
                      'apt-get', 'install', '-y', '--no-install-recommends',
                      'ca-certificates', 'cmake', 'libssl-dev', 'clang', 'pkg-config'], 600)
                  command('native-dependency-versions', ['dpkg-query', '-W',
                      'ca-certificates', 'cmake', 'libssl-dev', 'clang', 'pkg-config'], 30)
              elif phase == 'build':
                  source_check()
                  original_inputs = dict(data['inputs'])
                  capacity(30 * GiB)
                  command('rustc-version', ['rustc', '+1.98.1', '--version'], 30)
                  data['rustc'] = (evidence / 'rustc-version.log').read_text().strip()
                  require(data['rustc'].startswith('rustc 1.98.1 '), 'wrong_toolchain')
                  command('linux-baseline', ['cat', '/etc/os-release'], 30)
                  (root / 'cargo-home').mkdir(mode=0o700)
                  environment = dict(os.environ, CARGO_HOME=str(root / 'cargo-home'))
                  data['build_environment'] = {key: environment[key] for key in (
                      'CARGO_BUILD_JOBS', 'CARGO_INCREMENTAL', 'CARGO_PROFILE_DEV_DEBUG',
                      'CARGO_PROFILE_TEST_DEBUG', 'CARGO_PROFILE_RELEASE_DEBUG')}
                  target = 'aarch64-unknown-linux-gnu'
                  for edition in ('essentials', 'platform'):
                      environment['CARGO_TARGET_DIR'] = str(root / 'target' / edition)
                      command('build-' + edition, ['cargo', '+1.98.1', 'build', '--release', '--locked',
                          '--no-default-features', '--features', edition, '--bins', '--target', target],
                          1800, environment)
                  environment['CARGO_TARGET_DIR'] = str(root / 'target' / 'client')
                  command('build-client', ['cargo', '+1.98.1', 'build', '--manifest-path',
                      'crates/riauthctl/Cargo.toml', '--release', '--locked', '--no-default-features',
                      '--target', target], 1800, environment)
                  archives = evidence / 'archives'
                  archives.mkdir(mode=0o700)
                  products = [('essentials', 'riauth'), ('platform', 'riauth'),
                              ('essentials', 'riauth-maintenance'), ('platform', 'riauth-maintenance'),
                              ('client', 'riauthctl')]
                  data['products'] = []
                  for edition, name in products:
                      capacity()
                      binary = root / 'target' / edition / target / 'release' / name
                      require(binary.is_file() and not binary.is_symlink(), 'missing_regular_binary')
                      with binary.open('rb') as stream:
                          header = stream.read(20)
                      require(header[:6] == b'\x7fELF\x02\x01' and header[18:20] == b'\xb7\x00',
                              'not_elf64_little_endian_aarch64')
                      stem = name if edition == 'client' else name + '-' + edition
                      observed = None
                      if name == 'riauth':
                          command('capabilities-' + edition, [str(binary), '--json', 'capabilities'], 30)
                          envelope = json.loads((evidence / ('capabilities-' + edition + '.log')).read_text())
                          observed = envelope['data']
                          expected = ['essentials'] if edition == 'essentials' else ['essentials', 'platform']
                          require(envelope.get('ok') is True and observed.get('scope') == 'artifact'
                                  and observed.get('edition') == edition
                                  and observed.get('build_features') == expected
                                  and observed.get('target') == {'os': 'linux', 'arch': 'aarch64'},
                                  'observed_build_metadata_mismatch')
                      archive = archives / ('local-' + stem + '-aarch64.tar.gz')
                      with archive.open('xb') as raw, gzip.GzipFile(fileobj=raw, mode='wb', mtime=0) as zipped:
                          with tarfile.open(fileobj=zipped, mode='w') as tar:
                              for member_name, member_path in ((name, binary), ('LICENSE', pathlib.Path('LICENSE')),
                                  ('THIRD_PARTY_NOTICES.md', pathlib.Path('THIRD_PARTY_NOTICES.md'))):
                                  info = tar.gettarinfo(str(member_path), arcname=member_name)
                                  info.uid = info.gid = info.mtime = 0
                                  info.uname = info.gname = ''
                                  info.mode = 0o755 if member_name == name else 0o644
                                  with member_path.open('rb') as stream:
                                      tar.addfile(info, stream)
                      data['products'].append({'edition': edition, 'binary': name,
                          'declared_build_features': [] if edition == 'client' else (
                              ['essentials'] if edition == 'essentials' else ['essentials', 'platform']),
                          'observed_server_capabilities': observed,
                          'target': target, 'binary_bytes': binary.stat().st_size,
                          'binary_sha256': sha(binary), 'archive': archive.name,
                          'archive_bytes': archive.stat().st_size, 'archive_sha256': sha(archive)})
                      save()
                  smoke = ['python3', 'scripts/check-edition-artifacts.py']
                  for flag, stem in (
                      ('essentials-archive', 'riauth-essentials'), ('platform-archive', 'riauth-platform'),
                      ('essentials-maintenance-archive', 'riauth-maintenance-essentials'),
                      ('platform-maintenance-archive', 'riauth-maintenance-platform'),
                      ('riauthctl-archive', 'riauthctl')):
                      smoke += ['--' + flag, str(archives / ('local-' + stem + '-aarch64.tar.gz'))]
                  command('focused-native-archive-smoke', smoke, 180)
                  source_check()
                  require(data['inputs'] == original_inputs, 'build_inputs_changed')
                  for item in data['products']:
                      binary = root / 'target' / item['edition'] / target / 'release' / item['binary']
                      require(sha(binary) == item['binary_sha256']
                              and sha(archives / item['archive']) == item['archive_sha256'],
                              'tested_product_bytes_changed')
                  require(not data['cleanup_errors'], 'owned_cleanup_failed')
                  data['largest_reaped_child_max_rss_bytes'] = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss * 1024
                  data['native_archive_slice'] = 'passed'
              else:
                  raise RuntimeError('unknown_phase')
              data['last_phase'] = phase
              data['last_phase_status'] = 'passed'
          except BaseException as error:
              data['last_phase'] = phase
              data['last_phase_status'] = 'failed_or_refused'
              data['failure'] = str(error)
              save()
              raise
          save()
          PY
          python3 "$A09_ROOT/driver.py" init
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
        with:
          ref: ${{ inputs.source_sha }}
          persist-credentials: false
          fetch-depth: 1
      - name: Verify exact reviewed source and remaining capacity
        run: python3 "$A09_ROOT/driver.py" verify
      - uses: dtolnay/rust-toolchain@02cb101ec7c40f2c49e1d9714d64511d8e1b74de
        with:
          toolchain: 1.98.1
      - name: Install the release path's native dependencies under owned bounds
        run: python3 "$A09_ROOT/driver.py" dependencies
      - name: Build five local binaries, archive and run focused native smoke
        run: python3 "$A09_ROOT/driver.py" build
      - name: Upload exact local outputs and failure/refusal evidence
        if: ${{ always() }}
        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02
        with:
          name: riauth-local-arm64-${{ github.run_id }}-${{ github.run_attempt }}
          path: ${{ env.A09_ROOT }}/evidence/
          if-no-files-found: error
          compression-level: 0
          retention-days: 14
```

### Bounds, provenance and remaining gate

The three exact build argument lists above are sequential release/locked
Essentials `--bins`, Platform `--bins`, then independent base client, each with
its own private target and common private Cargo home. There is no cache action,
Docker, image build/push, service dependency, PG probe, automatic matrix,
tag/release operation, official packager or invented run identity. Unlike the
earlier local/offline plan, this **future remote** proposal permits locked Cargo
dependency fetching and the pinned toolchain/action setup only as part of a
later root-owned authorized dispatch. Existing release setup supplies exactly
ca-certificates/cmake/libssl-dev/clang/pkg-config. Apt package versions and OS
baseline are recorded; mutable hosted images/apt resolution are not called a
reproducible minimum-libc proof. Action SHA pins match release.yml at
`88790deb62d32c84fa17dceb12cd93a727224e94`.

Initial and post-checkout preflights, plus dependency/build phase starts,
require **30 GiB actually free** on `/`, workspace and the private job path.
Shared device IDs are recorded without adding duplicate free-space samples.
Every owned command polls those scopes every two seconds, stopping at **10 GiB**
with 2 GiB margin over the 8 GiB floor. It stops/reaps only the session/process
group that it created: TERM with two-second grace, then KILL with five-second
bound, with cleanup refusals retained. No process-name/global kill, cache prune
or unrelated path removal occurs. Two-second sampling is not a hard filesystem
quota or an absolute guarantee against an unsampled growth burst. Setup actions
are GitHub-managed processes, outside these owned command groups; the guards
check capacity again before subsequent commands, and the 150-minute job bound
applies throughout. No claim that runner provisioning/action setup passed is
made here.

Owned apt/update/install and metadata commands have finite individual bounds;
each build is at most 1,800 seconds, focused smoke 180 seconds. Command logs are
guarded at 8 MiB (sampling can retain a small overshoot, marked by a failing log
limit); a failure stops the slice and retains the recorded exit/deadline/resource
reason and actual captured-log hash rather than retrying or changing an expected
result. `always()` uploads only the private allowlisted evidence directory:
five local archives, input/product/checker hashes, real workflow/run identity,
measured native resources and bounded build/smoke/dependency logs. Fixture keys,
password/session files, full environment, source checkout and Cargo cache/target
trees are not uploaded. Logs are from the source-pinned focused fixture and
build commands; no user/deployed credential input is supplied. GitHub provides
the upload/run artifact metadata; it is not a cryptographic artifact attestation.
Cancellation, platform job timeout, runner loss or upload failure can prevent a
final artifact receipt; primary Actions failure logs and any surviving partial
evidence must remain failures, not a successful or complete upload claim. After
upload, GitHub discards the owned ephemeral VM/temp storage; the proposal deletes
no caches to create room and claims no independently observed teardown receipt.

No repository local driver is strictly needed for this first archive/refusal
slice: all inline orchestration belongs to the one proposed new workflow file,
and `check-edition-artifacts.py` remains unchanged. The prior proposal for
`scripts/check-local-linux-artifacts.py` is a **separate later root-reserved
source/runtime gate** for real base-client flows, ordinary-user IDs/group
authorization preservation, format-3 rate/authentication/capability mismatch and
offline E→P→E handoff. This workflow's native smoke does not establish that full
gate, dependency inventories, Linux x86-64/current containers, HTTPS/passkeys or
the complete per-artifact acceptance. The format-2 encrypted helper remains
untouched. Existing receipt/header/PAM/Group/removal/audit safeguards, the
non-renewed 60s lease boundary and RiWork Cua.ai Driver preference persist.

Read-only work for this append: complete fixed release workflow reread; pinned
CI/action/checker source inspection; original report byte/hash comparison;
official GitHub runner/manual-dispatch/variables documentation reads; explicit
project orchestrator finding sent exit 0. No actual hosted capacity/run was
queried or provisioned. PyYAML is not installed locally; no dependency was
installed. Proposed commands/inline code have not been executed. Report append,
static syntax/link/pin/whitespace checks and its commit are the only authorized
local writes. Root's next decision is review of this **new-file source text**
and the unresolved remote storage prerequisite, before any workflow edit or
dispatch.

Append validation, without executing the proposed workflow/inline driver:
installed Ruby Psych parsed the complete YAML; `bash -n` parsed all four `run`
blocks; Python AST parsed the inline driver. Static assertions checked the sole
manual trigger, one native job, read-only contents permission, 150-minute job
bound, sequential/private build settings, floor/start guard constants, owned
process-group cleanup and failure upload. All three action SHAs matched the
fixed release source; 13 local links/anchors and 15 source/blob rows remained
valid, and the three official-document URLs were the pages read above. The new
workflow file is absent. The proposed full workflow text including its terminal
newline is **18,130 bytes**, SHA-256
`883e428ce876154850889949d21760576229f82800f89d6acde28e2671ef34aa`.
The original prefix comparison, final newline/trailing-whitespace and
report-only staged-scope checks passed. No product command, workflow dispatch,
remote runner probe, runtime, Cargo, image/service, desktop, main/push/status or
new worker/task/worktree action was performed. No actual successful artifact,
resource or cleanup receipt was fabricated by these static checks.

## Append: reviewed manual ARM workflow materialized, runtime held

Root's `wave30_A09_manual_arm_workflow_implementation` reservation authorized
only the new workflow and this report append in existing WT f2e8500e. The actual
[manual ARM64 workflow](../../.github/workflows/check-local-artifacts.yml) is now
materialized byte-for-byte from the fully reviewed YAML fence above:

| Actual source pin | Value |
| --- | --- |
| Source-only commit | `5962a63e363028f1994fb4f94a2608ea2dad55f8` |
| Parent | `f886ae9e074b94883bc05619748025ce3e84b221` |
| Sole source path | `.github/workflows/check-local-artifacts.yml` (new file) |
| Git blob | `f3003eb4558f9f43ccc21be95283efb91fc70e91` |
| File bytes/lines | 18,130 bytes / 325 lines, including terminal newline |
| File SHA-256 | `883e428ce876154850889949d21760576229f82800f89d6acde28e2671ef34aa` |

The source commit has exactly one added file. No existing workflow, helper,
product, configuration, guide, notice, dependency lock or edition/security
algorithm changed. No alignment/merge/reset was performed. This worker source
commit identifies the new-file delta; it is not root's current published-main
pin or a product artifact built from that pin. Root alone integrates and chooses
the later reviewed full product `source_sha` for any dispatch.

The original **entire f886 report**, 778 lines / 50,992 bytes, remains the exact
prefix of this append, SHA-256
`30bd8dc9a8b3f99dcd88edeed0d03b31b02da5f733c46419a015f2b641396eb5`.
That also preserves the earlier d4b6967 316-line / 23,842-byte acceptance,
historical evidence and original resource blocker without rewriting any prior
observation that the then-proposed workflow was absent.

Actual local validation performed on the materialized source:

| Check | Actual outcome |
| --- | --- |
| Extraction and exact byte/SHA comparison against immutable f886 reviewed fence | Passed: 18,130 bytes, same approved SHA above; no reformatting |
| Installed Ruby Psych `YAML.safe_load` | Exit 0; parsed the actual workflow text |
| `bash -n` on each of the four workflow `run` blocks | Four syntax passes; none of the blocks executed |
| Python AST parse of the actual inline driver | Passed; no inline driver phase executed |
| Static trigger/job/source/permissions/concurrency checks | Sole required full `source_sha` input without default, `workflow_dispatch` only, one native ARM job, `contents: read`, fixed concurrency without automatic cancellation, 150-minute job bound retained |
| Action/guard/private-build/provenance/upload assertions | Pinned actions match fixed release source; 30 GiB phase starts, 10 GiB stop margin over 8 GiB floor, jobs=1/incremental=0/debug=0, private targets/Cargo home, owned process groups, real workflow/run identities and `always()` upload retained exactly |
| `python3 scripts/check-docs.py` before and after source materialization | Exit 0 twice; Markdown links and build-directory layout checked, with no layout error or checker change |
| `git diff --no-index --check /dev/null .github/workflows/check-local-artifacts.yml` and staged whitespace/scope | Exit 0; only the new workflow staged in the source commit |
| `python3 scripts/check-repo-hygiene.py` on the staged source | Exit 0; 963 tracked files checked, no private material printed |
| Source-commit path/blob/content and report-prefix verification | Passed; source only, report unchanged at the source commit, clean worktree immediately after it |

The contributor guidance was reread. Root's explicit no-Cargo/no-runtime
reservation governs this bounded source materialization; broader contributor
build/test commands were not invoked. The first claim and immutable source
handoff were delivered through the explicit project orchestrator, both exit 0.
Report-final docs/whitespace/scope/prefix checks and the separate evidence commit
follow this append; their immutable hash is provided in the final orchestrator
receipt rather than embedded as a self-referential commit hash here.

**Dispatch, download, tool setup, build, remote runner provisioning/probing and
all runtime remain held** until root reviews the actual immutable source,
integrates it and explicitly owns dispatch. No GitHub workflow/run/artifact ID,
capacity sample, binary/archive hash, successful smoke or cleanup receipt was
produced by this local work. Existing action pins and Git SHA hashes are source
evidence only. The advertised runner storage constraint remains unresolved:
the job must measure and refuse below its unchanged threshold; no label-only
success, threshold reduction, larger-runner substitution or cache deletion is
authorized. Current Linux x86-64/containers, TLS/passkeys and the original shared
identity/authorization/configuration full gate remain open; the format-2
encrypted helper and separate later local-driver plan remain untouched. No new
worker/task/worktree, Cargo, Docker/service, desktop, status, main or push action
occurred. RiWork Cua.ai Driver preference persists.

## Append: actual pre-runner rejection and environment-context correction

Root reserved `wave30_A09_workflow_environment_correction` in the same WT/task:
only the workflow's `A09_ROOT` environment seam and this append. All prior
**845 report lines / 55,969 bytes** remain an exact prefix, SHA-256
`a3744240c48dede62c64b81e07c8ae9264a5b6638d60e4900e67695ca08d1322`.
The old proposal/materialization phases, earlier failures and resource blocker
are preserved as dated evidence rather than rewritten to imply they succeeded.

### Root's actual failure receipt, before any runner or build

Root reports publication of the reviewed exact workflow at
`3a57affd9023a48c32085d6bcfb1d17ca4feb901`, followed by **one manual dispatch
rejected with HTTP 422 before runner allocation**. The diagnostic identified
line 27, column 17: unrecognized named-value `runner` in `runner.temp` at
`jobs.env`. The invalid-workflow push metadata entries `37014464740` (main)
and `37014464605` (accepted) report failure. These IDs are root-reported failed
metadata records, not a successful manual job, runner sample or build receipt.
Root released the slot; no manual job, tool setup or Cargo/build ran.

This worker independently read the published Git object: its workflow blob is
the original `f3003eb4558f9f43ccc21be95283efb91fc70e91`, matching the pre-fix
worker file exactly. This worker did not repeat a dispatch, query a runner, or
independently fetch those failed run records; the HTTP/result observation above
is explicitly attributed to root's supplied receipt. No partial artifact or
capacity credit is inferred from either failed metadata entry.

I missed key-specific GitHub context availability when proposing/materializing
the workflow. The earlier Psych/bash/AST/exact-byte checks really passed their
local syntax and preservation checks, but **did not validate GitHub expression
contexts at each workflow key**. That omission allowed the invalid job-level
`runner.temp` expression through. It is a source/check-coverage error, not a
repository documentation-checker bug or evidence that GitHub had accepted the
workflow. Those historical checks must not be treated as full semantic or
dispatch validation.

### Exact source correction and preserved controls

[GitHub's context-availability table](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#context-availability)
does not allow `runner` in `jobs.<job_id>.env`; it allows `env` in step `with`.
The correction removes the one job-level `A09_ROOT` expression and, immediately
after the initial `umask 077`, adds these two shell lines before `mkdir`:

```sh
export A09_ROOT="$RUNNER_TEMP/riauth-a09-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
printf 'A09_ROOT=%s\n' "$A09_ROOT" >> "$GITHUB_ENV"
```

`export` makes the actual runner-derived value available to the current shell
and Python driver. The fixed name/value environment-file write makes it
available to subsequent driver and upload steps; writing `GITHUB_ENV` alone
does not update the writing step's environment. This follows
[GitHub's environment-file documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#setting-an-environment-variable).
There is no default path or fabricated runner/run value, no default-variable
override, and no change to upload's `${{ env.A09_ROOT }}/evidence/` expression.

| Corrected actual source pin | Value |
| --- | --- |
| Minimal source-only commit | `84c498a76f034d721e95c9d9e1979bdc178e15c3` |
| Parent | `ea03829a51793b9e0cbcbfc680137dd76b8e9fc3` |
| Sole source path | `.github/workflows/check-local-artifacts.yml` |
| Git blob | `0e5426c9212433fffbbd578b71ec9fb682ad9c07` |
| Bytes/lines | 18,186 bytes / 326 lines, terminal newline retained |
| Corrected SHA-256 | `0a98865e5ee25d2f94b745b32602949553bf4f4e3b83d8bd302fade98eafd948` |
| Source diff | Exactly 2 insertions / 1 deletion, only the authorized environment seam |

Whole-file transformation comparison against published `3a57aff` passed:
removing the old environment line and inserting only those two lines produces
the complete corrected file exactly. All Python heredoc bytes and every later
step are unchanged. The extracted Python body, excluding the final heredoc
newline, retains SHA-256
`1bc3624052342975132f72887f65146a8470331d04ebaf06b0035801b5ff258f`.
Action pins, manual-only trigger, required full source SHA, native runner,
read-only permission, real provenance fields, concurrency/time bounds, private
jobs=1/sequential five binaries, local archives, focused smoke, failure upload,
30 GiB start / 10 GiB stop / 8 GiB floor and complete driver remain unchanged.
No alignment, other workflow/helper/product/guide/configuration/algorithm edit
occurred. Root owns publication/integration of this delta; its worker commit
is not a new published-main or runtime-source receipt.

### Actual correction checks and their limits

Installed Psych parsed the corrected YAML, all four shell `run` blocks passed
`bash -n`, and the unchanged inline Python passed AST parsing, without executing
any workflow block or driver phase. A static check using the official table
detected exactly the old job-env unavailable context `runner` and none in the
corrected job environment. The existing step `with` expressions use allowed
`inputs`, `github` and `env` contexts. Ordering checks confirmed `umask`, export,
fixed `GITHUB_ENV` write, then `mkdir`. Parsed structure normalization and byte
comparison confirmed that no other workflow field or later step changed.

`python3 scripts/check-docs.py`, staged `git diff --check`, exact source scope
and `python3 scripts/check-repo-hygiene.py` passed with exit 0; hygiene checked
963 files and printed no private material. The report was unchanged at the
source commit, and the worktree was clean immediately afterward. All prior
report prefixes are preserved for the separate append-only evidence commit.

One initial **local validation snippet** exited 1 on invalid `assert` syntax
before parsing the workflow; the snippet was corrected and the same static
checks then passed, with no intervening workflow change. An optional official
runner-source page read returned `Internal Error`; no source-code inspection is
claimed from that request. Context/persistence conclusions use the independently
read official documentation above. No dependency/checker was installed or
changed, and no runtime failure was hidden by either local correction.

These are source/syntax/context-availability checks, **not GitHub server
acceptance, a dispatch success, native capacity, toolchain setup, artifact
production or a passing smoke**. Root alone may re-release one corrected
dispatch after immutable source review/publication. No retry/dispatch/build/
download/tool setup/runner/runtime was performed here; the slot remains unused
by this worker. No threshold/provider substitution, larger runner, cache
deletion, new worker/task/worktree, Cargo/Docker/service, desktop, status, main
or push action occurred. The original A09 artifact/shared gate, Linux x86-64/
container/TLS/passkey scope and physical/tenant/escrow limits remain open.
RiWork Cua.ai Driver preference persists.

## Append: actual native ARM64 cohort and one held build-free handoff seam

This phase reserves only an append to this report. All prior **961 lines / 63,148 bytes** remain an exact prefix, SHA-256 `9b125ef47d8b449e7ae09f1ef19efbe1b44dd793b5bddb40603d844db1a5caea`. The live original A09 row and shared distribution gate were reread; no task status was changed. Historical artifact failures, the first actual HTTP 422 rejection, invalid-push failures `37014464740`/`37014464605`, and the admitted context-validation miss remain intact. The successful corrected run is a separate receipt.

### Actual run/source/artifact identity

Root reports one corrected manual run **37016520583 / job 110868629053 / attempt 1**, conclusion **SUCCESS**, slot released to SCIM. This worker read only the already downloaded immutable `/tmp/riauth-wave30-a09-37016520583`: full JSON documents, all logs, every resource row and pinned checker/helper bodies. No GitHub query/download, archive extraction, binary/helper/native execution or service occurred.

Workflow source **`036a392656b4b5070cc86a11d5ca3258b7b868d2`**, blob `0e5426c9212433fffbbd578b71ec9fb682ad9c07`, is the exact corrected 18,186-byte workflow, SHA-256 `0a98865e5ee25d2f94b745b32602949553bf4f4e3b83d8bd302fade98eafd948`. Product source **`9a819317efb3a13fa27cd86f884be2be00898fc0`**, tree `1528b61ba463d9262d6252d54174748a176f313b`, was requested and verified in the run; this worker matched all seven input hashes against that Git object. JSON records `workflow_dispatch`, real `Rhein-Industries/riAuth` repository/main workflow ref/run/attempt IDs. Workflow, product, worker-report and future validator pins remain distinct.

Root identifies uploaded artifact **11232871527**, digest prefix **`fc2a83dd`**. Exact workflow/run derive upload name `riauth-local-arm64-37016520583-1`. The downloaded directory has no outer artifact ZIP/full-digest metadata; the full 64-hex digest/name confirmation was requested from root through the project orchestrator. No full digest is invented or claimed to have been independently rehashed here. A later transport gate must verify exact ID/run/repository/full root-confirmed digest before native execution.

| Actual evidence | SHA-256 |
| --- | --- |
| `evidence.json`, 60,299 bytes | `fd8280125b38e6afbe13970be4617df8fdbe72e7280a4e6dedb2f2a84e4c9ab1` |
| `resources.jsonl`, 278,604 bytes | `f81d488c07a2616e4541926c7d98bf5a87bfb38d97923a5c200cc03bf2a321ad` |

The run explicitly records **`official_release: false`**, **`shared_full_gate: not_run`**, **`native_archive_slice: passed`**, empty cleanup errors. These are five **LOCAL native ARM archives**, with no official release/signature/registry/container/image/current-x86 credit. Root independently checked all archive/binary/input/log hashes and ELF ARM64. This worker additionally matched five compressed file hashes/lengths, seven fixed Git inputs and eleven full logs; binary/ELF observations remain root/run evidence. Archives were not opened or extracted here.

| Product / declared features | Archive bytes / SHA-256 | Binary bytes / SHA-256 |
| --- | --- | --- |
| essentials `riauth` / `essentials` | `local-riauth-essentials-aarch64.tar.gz` / 13,996,440 / `848758095ccab44919ddc643dd91ced3f77d4acc083be0edf63925fdb2b51981` | 37,242,432 / `0e07481de56505904864e42d63be743b235001e95c44af772654d1185a4f40e8` |
| platform `riauth` / `essentials,platform` | `local-riauth-platform-aarch64.tar.gz` / 18,375,977 / `9f615ca8fbe3cb1dfd400dea6c6d03a1f2677d7510b0ad6f1541c974a72723e8` | 49,565,208 / `c8c93c0605c83271049dd15794a8960cb3021245f0b439b7de3bbb9d37aec0ef` |
| essentials `riauth-maintenance` / `essentials` | `local-riauth-maintenance-essentials-aarch64.tar.gz` / 5,946,216 / `e43ed01029baeb7cae4821e125d168d687a7e60ab701afc8f46c9fea442adc3a` | 13,768,992 / `8c6b1d9a8df61aa5c7d16105b609a0e5dc67e9609d35de85fc822c78033a30ac` |
| platform `riauth-maintenance` / `essentials,platform` | `local-riauth-maintenance-platform-aarch64.tar.gz` / 6,788,229 / `510c55acf014273a53acb5b87d55b40381b9d00f51ef8b654493a307250f92df` | 16,062,840 / `2b2461adae8101f4e38c2842963bd273403c4c1ad06c6528a45ab1fd12f7bff1` |
| client `riauthctl` / `no-default-features; no optional client feature` | `local-riauthctl-aarch64.tar.gz` / 3,641,341 / `d826f26176ababda4fe3930e00a051c0416e2b10c9ec22a1f31ca4f4ef8118da` | 9,185,272 / `7c8831ed42d89a31f3ab2042cd03ab460ef19b991a41b7667b1bfe9c91c04291` |

All five target `aarch64-unknown-linux-gnu`; each archive contains exactly its binary, LICENSE and THIRD_PARTY_NOTICES.md per producing source/root validation. Historical c01/Q08 products are not relabelled as current.

| Pinned source input | Actual SHA-256 |
| --- | --- |
| `Cargo.lock` | `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` |
| `Cargo.toml` | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |
| `LICENSE` | `ef79ab7079893da02af81ebb8a57bec27dc7a601d505b228b1d5356a3aa5b1a9` |
| `THIRD_PARTY_NOTICES.md` | `142c0e5150de9436513d9f6f215c5422b8a3af84d4eb7d0708f155e3e18fce05` |
| `crates/riauthctl/Cargo.lock` | `2998555ddb2d00e8130a970fcacfed19dfee7a0b2e3305011ebd40746f67b4db` |
| `crates/riauthctl/Cargo.toml` | `af385d4d989c53987396edfdfa684cd3902a603d1cf65dd31ac72da7f08de9ea` |
| `scripts/check-edition-artifacts.py` | `2f477665ed584bc148fb230c0658f6d0cc66aceaa5af631c4dc467872fa9ecce` |

### Actual builds and focused smoke

Observed host: Linux `aarch64`, GitHub-hosted `ubuntu-24.04-arm`, four CPUs, 16,722,042,880 bytes RAM, Ubuntu 24.04.5 LTS. Rust `1.98.1 (48a229cea 2026-09-01)`. Observed packages: ca-certificates `20260601~24.04.1`, clang `1:18.0-59~exp2`, CMake `3.28.3-1build7`, OpenSSL dev `3.0.13-0ubuntu3.16`, pkg-config `1.8.1-2build1`. This is an actual run baseline, not a supported minimum libc guarantee or future availability.

Three sequential locked release commands used separate private targets, jobs=1, incremental=0, dev/test/release debug=0 and native target. Actual suffixes:

```sh
cargo +1.98.1 build --release --locked --no-default-features --features essentials --bins --target aarch64-unknown-linux-gnu
cargo +1.98.1 build --release --locked --no-default-features --features platform --bins --target aarch64-unknown-linux-gnu
cargo +1.98.1 build --manifest-path crates/riauthctl/Cargo.toml --release --locked --no-default-features --target aarch64-unknown-linux-gnu
```

| Actual step | Exit / elapsed seconds | Full log SHA-256 |
| --- | --- | --- |
| `apt-update` | 0 / 8.004 | `19f808a68f6d524d9f9935feddf906fb1ae08319070432aaafa24242c9f52af6` |
| `apt-install` | 0 / 12.482 | `f6a65260955ad1d97b665b697b960013880647bcd84e7d15bdcf004875df8074` |
| `native-dependency-versions` | 0 / 2.002 | `0eeacaea693f7b9759023b1e40d5e8a6628e8542bb927792da179ba1d125db65` |
| `rustc-version` | 0 / 2.002 | `2073de5abf149185c2f4d7473dc833bc921b99135560d24ae7b2293f33a9c376` |
| `linux-baseline` | 0 / 2.002 | `87458b3b1f1ca0e5c693ade771de7007c3783bdc692fa6b39eb6414bf2af2763` |
| `build-essentials` | 0 / 1004.627 | `281169d70624e7b7ebc4605de2650b2c4a9db100bea9a60d3b54bc07d9bbeec3` |
| `build-platform` | 0 / 1312.673 | `46294d77993c0ea5d00fe8650c92522599fb47f380742866da68a4096abf34a4` |
| `build-client` | 0 / 204.104 | `4c2345f5d1f2c58cb6606bc672a960ae9dc233e80569158d05c210966f6fce49` |
| `capabilities-essentials` | 0 / 2.002 | `4bdc14319c2d0d6c70997f688837533919379906efef266d9b4648d3ea710c86` |
| `capabilities-platform` | 0 / 2.004 | `beb2ae858a9d081910939d488ba04249f728fcbbbc85daa778698478643c3e02` |
| `focused-native-archive-smoke` | 0 / 4.008 | `d0b3f6ce4435ad1d80cdbdaff55477e24ce2942ac26d512761a27b598d49ef48` |

Essentials succeeded with **three dead-code warnings**: `discard_workflow_registration`, `workflow_register_start_in`/`workflow_register_verify_in`, `PostLogoutReturn::allowed_by`. They are retained, not described as warning-free. Platform/client logs finish successful optimized release builds. Apt-trigger service restarts/deferred restarts are recorded remote setup output, not this worker's actions.

The unchanged focused checker SHA-256 is `2f477665ed584bc148fb230c0658f6d0cc66aceaa5af631c4dc467872fa9ecce`. Actual native invocation took all five archives and **no image arguments**; exit 0 / 4.008 seconds, 76-byte log `Native archives: edition, route, agent issuance and downgrade checks passed`. The full source body checks artifact capabilities/features, fresh maintenance init/server start/readiness, shared/edition route boundaries, Essentials agent refusals, Platform agent state and direct Essentials downgrade/preflight refusal, plus maintenance/client version. It does not test full base-client remote flows, shared E→P→E preservation, HTTPS/passkey devices or images. Full capability logs match JSON product objects: Essentials 60 compiled features/87 state entries, Platform 87/87; configured/runtime-ready/usable remain null at artifact scope.

### Actual resource samples

All **1,308** rows were parsed: first `2026-10-02T13:57:46.739159+00:00`, last `2026-10-02T14:40:37.351946+00:00`, sample span **2,570.612787 seconds**, largest actual interval **5.40448 seconds**, all timestamps increasing. Host/private/workspace minima are each **112,151,941,120 bytes = 104.449634552 GiB**; each reports device 2049 throughout, so these are named observations on one backing device, not three independent reserves. Initial free bytes 115,879,784,448 met 30 GiB preflight; no recorded 10 GiB stop/8 GiB floor event.

`largest_reaped_child_max_rss_bytes: 4454891520` is the driver's child-resource metric, not simultaneous whole-runner peak memory or future capacity. Samples do not establish continuous storage between observations or write/external-IO safety after paused checks. The prior Darwin resource blocker remains historical; no Linux artifact was pretended to execute on Darwin. The remote executor has ended and its slot is released to SCIM, with no next-gate reservation or capacity credit inherited by this worker.

### ONE held source-first next seam

Propose **only `scripts/check-local-edition-transition-postgres.py`** at exact product `9a819317efb3a13fa27cd86f884be2be00898fc0`. The proposed diff below is contained solely in this report, not applied or owned. This existing native Linux/aarch64 helper is build-free; it requires local `initdb`, `pg_ctl`, `createdb`, `psql`, already compares full non-transition PostgreSQL row hashes (including identity/credential/grant/revocation rows), and checks wrong-build open/connected-client refusal/explicit E→P→E. Extending it is smaller than building a new Store probe. This is a plain local PostgreSQL sample, not encrypted/redb equivalence or physical recovery.

| Pinned helper/dependency | SHA-256 / boundary |
| --- | --- |
| `check-local-edition-transition-postgres.py` | `d6b0d023164756b2fcabe1a89794400d5ee70b43b6b4874abed3fd2f6a2cce7d`, 13,127 bytes / 236 lines; metadata currently requires format 2 |
| `check-local-encrypted-edition-transition.py` | `09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e`, 22,045 bytes / 398 lines; metadata line 61 requires format 2, full run needs separately compiled Store probe |
| `check-exact-edition-matrix.py` | `f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887`; main runs debug jobs=2/Cargo analysis and must not be called |
| `check-installed-release-gate.py` | `cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5`; reuse private CLI/serving primitives, never official verify-assets/main for this local cohort |
| `spdx_sbom.py` | `ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c`; imported dependency unchanged, no generator/packager main invoked |
| `src/node_security.rs` | `979f1d22a4835bce97866e302dc2a8a21df4f0d383d4bd1e6960b6a5d7a3bcb7`; strict format 3/all 16 effective rates/issuer/authentication/capability refusal and atomic supported handoff |

Exact hunk intent: require format 3/all 16 valid rates and compare the complete map; reuse unchanged encrypted helper's **live_identity_and_grant function only** (never its metadata/run/probe); add one ordinary group and repeat password/user ID/group/grant equality, audit allow/user-admin deny, revoked-session refusal and fixed session expiry through E→P→E; copied authentication/shared-rate config drift must refuse plan/start without changing stored rows/agreement or leaving readiness; refuse added as well as changed/removed non-transition rows; bound the owned connected-client reaping. No stored agreement deletion/adoption, privileged group/client creation, receipt secret handling, product/edition writer algorithm or encrypted helper edit.

Complete proposed helper SHA-256 `575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1` (21,716 bytes / 370 lines). AST parsed only, no import/helper execution. Diff uses zero context to avoid Markdown trailing-space lines; a later reviewer applies it with explicit zero-context support against the exact pinned body. Root may reject/refine before granting source ownership.

```diff
--- a/scripts/check-local-edition-transition-postgres.py
+++ b/scripts/check-local-edition-transition-postgres.py
@@ -20,0 +21,3 @@
+import tomllib
+import urllib.error
+import urllib.request
@@ -32,0 +36,5 @@
+encrypted_fixture = load("encrypted_fixture", ROOT / "check-local-encrypted-edition-transition.py")
+RATE_NAMES = frozenset(("portal_start", "portal_approve", "login", "passkey", "account",
+                        "source_start", "source_callback", "saml", "mfa", "device_start",
+                        "device_verify", "browser_decision", "browser_state",
+                        "forward_auth", "outpost_start", "general"))
@@ -71,2 +79,6 @@
-    matrix.require(result["meta/node_security"]["format"] == 2,
-                   "transition requires a current security agreement")
+    matrix.require(result["meta/node_security"]["format"] == 3,
+                   "transition requires a current format-3 security agreement")
+    rates = result["meta/node_security"]["effective_rate_limits"]
+    matrix.require(set(rates) == RATE_NAMES and
+                   all(type(value) is int and 1 <= value <= 100_000 for value in rates.values()),
+                   "format-3 effective rate map is incomplete or invalid")
@@ -79 +91 @@
-    for field in ("issuer", "authentication"):
+    for field in ("issuer", "authentication", "effective_rate_limits"):
@@ -91 +103,2 @@
-    changed = sorted(key for key, value in before.items() if after.get(key) != value)
+    changed = sorted(set(before).symmetric_difference(after) |
+                     {key for key, value in before.items() if after.get(key) != value})
@@ -92,0 +106,103 @@
+
+
+def authenticated_status(base, path, token):
+    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
+    request = urllib.request.Request(base + path, headers={"Authorization": "Bearer " + token})
+    try:
+        with opener.open(request, timeout=5) as response:
+            response.read()
+            return response.status
+    except urllib.error.HTTPError as error:
+        error.read()
+        return error.code
+
+
+def ordinary_fixture(server, config, base, scratch):
+    # Reuse only the live fixture; never the historical format-2 metadata/run/probe.
+    encrypted_fixture.live_identity_and_grant(server, config, base, scratch)
+    session = scratch / "group-admin-session.json"
+    with gate.serving(server, config, base, scratch / "group-fixture.log"):
+        gate.remote(server, base, session, "login", "admin", "--password-stdin",
+                    input="q08-disposable-password\n")
+        for command in (("group", "create", "shared-fixture"),
+                        ("group", "add-member", "shared-fixture", "delegate")):
+            revision = gate.remote(server, base, session, "revision")["revision"]
+            gate.cli(server, "--server", base, "--session-file", session, "--non-interactive",
+                     "--if-revision", revision, "--idempotency-key", os.urandom(16).hex(), *command)
+        gate.remote(server, base, session, "logout")
+
+
+def shared_probe(server, config, base, scratch, revoked_token=None):
+    session = scratch / f"delegate-{time.monotonic_ns()}.json"
+    admin = scratch / f"admin-{time.monotonic_ns()}.json"
+    with gate.serving(server, config, base, scratch / f"shared-{time.monotonic_ns()}.log"):
+        if revoked_token is not None:
+            matrix.require(authenticated_status(base, "/api/me", revoked_token) == 401,
+                           "previously logged-out session became valid")
+        gate.remote(server, base, session, "login", "delegate", "--password-stdin",
+                    input="q08-delegate-disposable-password\n")
+        me = gate.remote(server, base, session, "whoami")
+        matrix.require(me["user"]["username"] == "delegate" and not me["user"]["admin"],
+                       "ordinary credential did not identify the same non-admin user")
+        token = json.loads(session.read_text())["token"]
+        matrix.require(authenticated_status(base, "/api/audit?limit=1", token) == 200,
+                       "active auditor grant stopped authorizing audit read")
+        matrix.require(authenticated_status(base, "/api/users", token) == 403,
+                       "ordinary auditor gained user administration")
+        time.sleep(0.2)
+        again = gate.remote(server, base, session, "whoami")
+        matrix.require(again["expires_at"] == me["expires_at"], "session expiry was renewed")
+        gate.remote(server, base, session, "logout")
+        matrix.require(authenticated_status(base, "/api/me", token) == 401,
+                       "logout failed to revoke the session")
+        gate.remote(server, base, admin, "login", "admin", "--password-stdin",
+                    input="q08-disposable-password\n")
+        grants = gate.remote(server, base, admin, "grants", "get", "delegate")
+        matrix.require(grants["grants"] == [{"role": "auditor", "scope": "audit/events",
+                                            "target_id": "events"}], "auditor grant changed")
+        group = gate.remote(server, base, admin, "get", "group", "shared-fixture")
+        matrix.require(me["groups"], "ordinary membership missing")
+        gate.remote(server, base, admin, "logout")
+        return {"user": me["user"], "groups": me["groups"], "group": group,
+                "grants": grants, "audit_status": 200, "users_status": 403}, token
+
+
+def shared_config_refusals(programs, port, database, server, maintenance, config, base, scratch):
+    original = config.read_text()
+    parsed = tomllib.loads(original)
+    baseline = rows(programs, port, database)
+    metadata = transition_metadata(programs, port, database)
+    candidates = {}
+    authentication, count = re.subn(r"(?m)^(session_ttl\s*=\s*)\d+$",
+                                   lambda m: m[1] + str(parsed["session_ttl"] + 1), original)
+    matrix.require(count == 1, "session_ttl configuration field missing")
+    candidates["authentication"] = (authentication, "Configured token lifetimes or password policy")
+    value = metadata["meta/node_security"]["effective_rate_limits"]["general"] + 1
+    if "general" in parsed.get("rate_limits", {}):
+        rate, count = re.subn(r"(?m)^(general\s*=\s*)\d+$", lambda m: m[1] + str(value), original)
+        matrix.require(count == 1, "general rate configuration field missing")
+    elif re.search(r"(?m)^\[rate_limits\]\s*$", original):
+        rate, count = re.subn(r"(?m)^\[rate_limits\]\s*$",
+                             lambda m: m[0] + f"\ngeneral = {value}", original)
+        matrix.require(count == 1, "duplicate rate_limits table")
+    else:
+        rate = original + f"\n[rate_limits]\ngeneral = {value}\n"
+    matrix.require(tomllib.loads(rate)["rate_limits"]["general"] == value, "rate fixture invalid")
+    candidates["rate"] = (rate, "Configured HTTP rate limit for general")
+    for label, (text, message) in candidates.items():
+        candidate = scratch / f"refuse-{label}.toml"
+        candidate.write_text(text)
+        blocked = gate.cli(maintenance, "--config", candidate,
+                           "transition-plan", "--target", "platform", expected=5)
+        matrix.require(any(item["resource"] == "meta/node_security"
+                           for item in blocked["data"]["blockers"]), "shared configuration was ignored")
+        refused = gate.cli(server, "--config", candidate, "serve", expected=None)
+        matrix.require(message in refused["error"]["message"], "shared configuration startup refusal changed")
+        matrix.require(rows(programs, port, database) == baseline and
+                       transition_metadata(programs, port, database) == metadata,
+                       "configuration refusal changed stored rows or security agreement")
+        try:
+            status = gate.get_status(base + "/readyz")
+        except OSError:
+            status = None
+        matrix.require(status != 200, "refused process left a serving listener")
@@ -138,0 +255,4 @@
+            ordinary_fixture(essentials / "riauth", config, base, scratch)
+            shared_before, revoked_token = shared_probe(essentials / "riauth", config, base, scratch)
+            shared_config_refusals(programs, port, database, essentials / "riauth",
+                                   platform_bins / "riauth-maintenance", config, base, scratch)
@@ -171 +291,5 @@
-                connected.wait(timeout=5)
+                try:
+                    connected.wait(timeout=5)
+                except subprocess.TimeoutExpired:
+                    connected.kill()
+                    connected.wait(timeout=5)
@@ -190,0 +315,2 @@
+            shared_platform, _ = shared_probe(platform_bins / "riauth", config, base, scratch, revoked_token)
+            matrix.require(shared_platform == shared_before, "upgrade changed shared identity/authorization")
@@ -209,0 +336,2 @@
+            shared_return, _ = shared_probe(essentials / "riauth", config, base, scratch, revoked_token)
+            matrix.require(shared_return == shared_before, "downgrade changed shared identity/authorization")
@@ -221,0 +350,6 @@
+                      "agreement_format": 3, "all_effective_rates_preserved": True,
+                      "shared_configuration_refusals": ["authentication", "general_rate"],
+                      "shared_identity_authorization_sample": "passed",
+                      "shared_sample_sha256": hashlib.sha256(json.dumps(
+                          shared_before, sort_keys=True).encode()).hexdigest(),
+                      "full_shared_gate": "not_certified",
```

### Held native transport, invocation and finite cleanup

Root must review this exact single-helper seam before any source edit. Runtime then needs a **fresh native ARM executor**; the build runner ended. A manual-only read-only transport/launcher is a separate later root-owned source/runtime gate, with real new run/job IDs, pinned checkout/download/upload actions, 20-minute bound and no Cargo/toolchain/build/tag/release/registry. A download action SHA must be reviewed rather than guessed. This append does not reserve/edit that workflow or implement a launcher.

Proposed root-owned metadata/download commands, **not executed here**, in a fresh private root:

```sh
umask 077
timeout --signal=TERM --kill-after=5s 120s gh api repos/Rhein-Industries/riAuth/actions/artifacts/11232871527 > "$A09_VALIDATE_ROOT/artifact-metadata.json"
timeout --signal=TERM --kill-after=5s 120s bash --noprofile --norc -c 'ulimit -f 262144; exec gh api "$1"' riauth-a09 repos/Rhein-Industries/riAuth/actions/artifacts/11232871527/zip > "$A09_VALIDATE_ROOT/artifact.zip"
```

The ZIP has a Bash file-size bound of at most 256 MiB and finite timeout; retain any nonzero exit as failure and never consume a partial file. Require exact ID `11232871527`, upload name `riauth-local-arm64-37016520583-1`, producing run `37016520583`, repository, unexpired artifact and full root-confirmed SHA-256 beginning `fc2a83dd`. Compare outer ZIP SHA-256 to that exact full digest before extracting; a prefix is insufficient. Missing/expired/deleted artifact refuses, with no rebuild/old-cohort fallback. No transport/native capacity availability has been claimed.

In an owned 0700 root, safely extract only regular allowlisted ZIP members. Verify both JSON hashes, producing product/workflow pins, eleven log hashes, all five archive and binary hashes/lengths plus LICENSE/notices. Gzip extraction may reuse installed-gate `install_member` only after archive verification and exact three-member/256 MiB bounds; reject symlinks/traversal/extra members. Install into `installed/essentials`, `installed/platform`, `installed/client`; verify native ELF AArch64 and actual Linux/aarch64 before execution. Do not call official packaging/SBOM/provenance entry points. Pin the future validator Git SHA/script/import hashes separately from the unchanged product source. The existing helper alone does not verify transport identity, so the source-reviewed launcher must implement these guards before runtime approval.

After separate native PostgreSQL-tool and launcher reservation, the ONE focused target proposed is:

```sh
env -u PYTHONOPTIMIZE timeout --signal=INT --kill-after=10s 600s python3 scripts/check-local-edition-transition-postgres.py --artifacts "$A09_VALIDATE_ROOT/installed" --evidence "$A09_VALIDATE_ROOT/public/shared-handoff.json" --source-revision 9a819317efb3a13fa27cd86f884be2be00898fc0
```

PostgreSQL tools/version/native shared-library availability are unobserved for a new executor; previous build logs do not establish them. Missing tools refuse and need separately reserved setup, not implicit installation. No local Darwin execution. The narrow helper uses the native server's existing remote CLI; the fifth client remains hash-bound/retained and its full remote-flow gate stays open.

Preserve **30 GiB start / 10 GiB stop / 8 GiB floor** for the launcher. Read actual host/private/workspace free bytes/device IDs at intended at-most-two-second intervals, report actual gaps/minima/unknown/error states, terminate only owned launch processes at stop-margin/unavailable sampling and retain failure/cleanup evidence. No label-only capacity success, threshold lowering/provider substitution/larger runner/cache deletion. Jobs=1/inc=0/dev+testdebug=0 remain if any later separately reserved build occurs, but this next seam has no builds.

Finite cleanup must be implemented/reviewed in the later launcher: 20-minute job, transport 120 seconds/256 MiB, helper 600 seconds; imported readiness/HTTP/CLI deadlines; proposed owned connected-client TERM→5 seconds→KILL→5 seconds. Capture raw stdout/stderr privately because imported error messages may include private envelopes; upload only allowlisted phase/exit/error-class/hashes, sanitized results/resources/cleanup status. Never upload passwords, tokens, sessions, connection/config/database files or raw private failure traces. Always stop the exact spawned private cluster with immediate `pg_ctl` mode, bounded 10-second TERM/KILL/reap, before removing only its fresh owned fixture directory; cleanup failure fails the gate. SIGINT can enter existing `finally`, but hard timeout alone does not prove cleanup. Root must review the launcher's failure/transport/resource/cleanup source before dispatch. It is not implemented or owned by this report.

A future pass credits a current-format-3 local PostgreSQL shared identity/authorization/configuration **sample**, not universal `shared_full_gate`, release or A09 closure. Current Linux x86-64, containers, HTTPS/TLS, devices/passkeys, full client flow, encrypted/redb, physical/HA/tenant/escrow and external cloud gates remain unproved. Existing receipt-secret, route-specific required/optional headers, PAM fallback, permission/review/removal/audit/credential protections and Group/input limits remain intact. Fixed-session-expiry fixture observations do not change the non-renewed 60-second admission lease or create an atomic paused-IO fence.

### Actual worker checks and preserved failures

Worker checks read full downloaded JSON/log/resource contents, matched all eleven full log hashes/seven fixed inputs/five compressed archive files, recomputed all 1,308 samples and compared both capability logs to full JSON product objects. Full checker/helper bodies and format-3 contracts were read at immutable Git objects. One workflow read mistakenly used product `9a81931`, where that file is absent; corrected to workflow `036a392`. One report-generation command failed Python parsing on an unterminated literal **before any write**; the corrected authoring command then wrote only this append. No native/runtime/build attempt occurred or failed here. The proposed helper diff was AST parsed without import or materialization.

No workflow/helper/product/other-guide/main edit, dispatch/download/build/Cargo/native/helper/service runtime, cache deletion, status/push, other lane, new worker/task/worktree or desktop occurred. Evidence directory was read only. RiWork Cua.ai Driver preference persists. Root alone reviews/integrates, reserves any future native/source gate and decides release/closure.

The first report whitespace check flagged six blank unified-diff context lines containing a single space. The embedded diff was regenerated with zero context, preserving the exact proposed helper bytes/hash; no repository checker was changed. Final docs, whitespace, report-only scope, reconstructed-diff AST and repository hygiene checks are recorded in the separate immutable handoff.

One local reconstruction/refinement snippet initially asserted on an insertion hunk because it treated zero-count unified-diff line positions as ordinary context positions. It failed before writing; the static parser was corrected to handle zero-count insertions, reconstructed the exact proposal, and then added strict refusal of extra non-transition rows. This was a local authoring/check correction, not native helper execution.

Final appendix checks passed with exit 0: repository docs checker, whitespace, hygiene (963 files), exact report-only scope and prior 961-line/63,148-byte prefix, reconstruction of the embedded zero-context diff against pinned product source, unchanged helper entry point, Python AST and the three proposed/recorded shell blocks with `bash -n`. The helper proposal was never imported, materialized or executed. These static report checks add no native/runtime acceptance.

## Append: approved format-3 PostgreSQL helper materialized, runtime held

2026-10-02, original A09 task / existing WTf2. Root approved source-only reservation `wave30_A09_postgres_shared_gate_helper`: exactly the helper delta reviewed in `f0796af` and an append to this report. The preceding **1,255 lines / 93,763 bytes** remain an exact prefix, SHA-256 `1b0f9a68168c3a53f3438dc316ab75b2b23635373e7b010a3611aa39be162744`. Earlier missing-metadata observations, actual HTTP 422/invalid-push failures, local authoring/static corrections and historical artifact limitations remain dated phases, not rewritten.

### Newly supplied artifact metadata, with unchanged integrity limits

Root supplied exact GitHub API-reported artifact identity: ID **11232871527**, name **`riauth-local-arm64-37016520583-1`**, digest **`sha256:fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30`**. Root expressly did **not** retain/re-hash the outer ZIP. This new metadata resolves the earlier lack of a full API-reported digest; it does not retroactively turn the prior prefix-only observation into full-digest knowledge or establish independent outer-ZIP verification. This worker did not download or re-hash an outer ZIP.

The complete accepted root receipt `docs/roadmap/evidence/wave30-a09-native-arm64-37016520583.json` was read as an immutable Git object at `a74d3225dd8c845d0c7dff43ab922749c3a957b5`, SHA-256 `2cbf8ee46dbdd8b2ab43ad933913dd0a16c20b3183497c2d3b9cf3d1287da227`, blob `ceee1550df02541ee64fa16eb126b819d59fa2d6`. It agrees on the real run/job/workflow/product pins, all five local product hashes, resource/log/input hashes, `outer_zip_rehashed_by_root: false`, `shared_full_gate: not_run` and `official_release: false`. Root also names `planning/evidence/wave30-a09-remote-result-review.json`; it was not found as a tracked Git object or at the shared Git-root planning path accessible here, so no independent read of that planning file is claimed. The complete accepted receipt and root's explicit message supply the metadata above without requiring contact or external API access. No accepted root docs or planning files were edited/copied/merged.

### Actual source pins and approved byte proof

Before editing, the protected helper matched product `9a819317efb3a13fa27cd86f884be2be00898fc0` byte for byte: 13,127 bytes / 236 lines, SHA-256 `d6b0d023164756b2fcabe1a89794400d5ee70b43b6b4874abed3fd2f6a2cce7d`. The zero-context diff embedded in committed `f0796af897425b43a258b0d30214741beb238a32` was reconstructed against that exact protected body. Only the approved helper was written, its prior mode 0644 retained, and the result matched the proposed complete text hash exactly. The helper was never imported or executed.

| Materialized source | Exact value |
| --- | --- |
| Source-only commit | `d36e13ad17541d21c88ed90d842e0a3e6db2280d` |
| Parent proposal/report commit | `f0796af897425b43a258b0d30214741beb238a32` |
| Sole source path | `scripts/check-local-edition-transition-postgres.py` |
| Git blob | `d55d51aad16f508b3cb993a15f44e23aeb912729` |
| Complete source | 21,716 bytes / 370 lines, SHA-256 `575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1` |
| Exact delta | 139 insertions / 5 deletions; approved proposal bytes, no extra correction |

The implemented source requires strict current format 3 and all 16 valid effective rates, compares the complete effective-rate map through supported E→P→E, rejects added as well as changed/removed non-transition rows, creates one disposable ordinary password/auditor/group fixture and compares its ID/user/groups/grants and authorization outcomes across editions. It checks audit allow/user administration deny, previously logged-out session refusal, one-session fixed expiry and authentication/general-rate plan/start refusals with unchanged rows/agreement and no ready listener. The owned connected-client cleanup has TERM/5 seconds/KILL/5 seconds bounds. These are **authored checks**, not observed native test results.

| Source hunk mapping | Actual helper start line |
| --- | ---: |
| `transition_metadata` | 66 |
| `require_target_metadata` | 88 |
| `require_preserved` | 102 |
| `authenticated_status` | 108 |
| `ordinary_fixture` | 120 |
| `shared_probe` | 135 |
| `shared_config_refusals` | 170 |
| `main` | 211 |

### Stable supporting API equality and protected inputs

Supporting API bodies were reread at product `9a81931`: `Core::me` returns a stable `UserView` and durable group membership beside transient session ID/expiry/MFA fields; `UserView` contains stored user identity/account fields, not session ID/expiry. `Core::human_grants` returns username plus stored grants; `Core::get_resource("group", ...)` returns the stored group object. The AST-checked `shared_probe` equality snapshot contains exactly **user, groups, group, grants, audit_status, users_status**. It excludes session IDs, expiry and tokens. The returned token is kept separately in memory for refusal checks, and the expiry comparison is only within the same session. No equality adjustment or broader source edit was necessary.

The encrypted helper remains byte-exact at product source, SHA-256 `09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e`; its unchanged `live_identity_and_grant` function source segment has SHA-256 `893ebad3662b67c835a35826ae202022b478f4184cd69a42e2c6b9a38220cf44`. Static call inspection finds only this function through `encrypted_fixture`. Its historical format-2 metadata/run/compiled-probe path is not called. Installed-gate, exact-matrix and SPDX helper bytes remain equal to the pinned product hashes already recorded above; the workflow remains exact `036a392` bytes/SHA-256 `0a98865e5ee25d2f94b745b32602949553bf4f4e3b83d8bd302fade98eafd948`. No product/edition/node-security/CLI writer, encrypted helper or workflow was edited.

### Actual static checks and still-held runtime

Source validation passed with exit 0: approved-diff reconstruction/exact complete SHA/length/line/mode proof, Python AST, stable snapshot-field/token separation and encrypted-function-call inspection, protected byte comparisons, docs checker, staged whitespace and repository hygiene (963 files). The report was unchanged at the source-only commit. No helper import, native process, PostgreSQL tool/service/fixture, Cargo/build, transport/download/launcher/workflow execution or contact occurred. No current artifact or shared-gate result is credited by this source change.

A future fresh native build-free launcher still requires separate immutable source review/reservation: exact artifact transport/full-digest comparison, safe hash-bound extraction, real new run provenance, actual PostgreSQL/native/resource preflight, 30 GiB start/10 GiB stop/8 GiB floor, finite timeout and owned cleanup, private capture/allowlisted failure evidence. Root's API-reported digest does not substitute for verifying transport bytes. Neither this helper nor this report implements/owns that launcher. The previous native executor ended and its slot was released; no runtime slot is consumed here.

Original A09 remains open. Linux x86-64/container/TLS/passkey/device/full-client/encrypted-redb/full shared/physical/tenant/escrow/paused-IO limits and accepted receipt-secret, route-header, PAM, permission/review/removal/audit/credential/Group/input/non-renewed-60-second protections persist. Root alone reviews/integrates and decides later native acceptance/release/status. No other file, main, push, status, worker/task/worktree/shell or desktop was changed/created. RiWork Cua.ai Driver preference persists.

## Phase I — build-free native launcher design only (2026-10-02)

Reservation `wave30_A09_build_free_native_launcher_design` owns **only this report append**, on existing task `506e3979-a590-4af3-8fa8-ee90d3a517f2` / WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. Parent `0a2bd3951a3c29726a9be4b285cf7b72ef56a661` contains the already reviewed helper/source report. Its complete prior report is preserved byte for byte: 101,377 bytes / 1,305 lines, SHA-256 `c6abe294c6c9ba4f06633ebef90d334ea14459d7ea3d349edef901aa2d0ec495`, Git blob `ff4cf44508862fa4b53fd1267da96040bb62e35f`. No prior missing-metadata phase, HTTP 422 / invalid-push failure, proposal, observed build result or qualification is rewritten.

The sole future source seam proposed for immutable root review is a **new** mode-100644 `.github/workflows/check-local-shared-handoff.yml` (one add hunk, lines 1–1279). It is not materialized here. No existing workflow/helper/product/guide is changed. Complete proposed file bytes appear below, including the inline controller and its retention adapter; no omitted external driver or setup script is required.

| Complete proposed text | Bytes | SHA-256 |
| --- | ---: | --- |
| New workflow, including terminal newline | 73,279 | `0bad3f22ac0c1534e64f12b5454efd686bac8bb260919f7d16753f968bb518b4` |
| Shell-created controller, including terminal newline | 59,371 | `8018cd2119e9763aa1832664d03ca28732739f2c4749d2b2a2cfcdb266e5a6b4` |
| Private bootstrap string parsed from controller AST | 1,506 | `1adc47c2b81f78c75f1f2122aabff7f08f48786fafd840880f3106418326e4de` |

### Fixed receipt and separate source roles

The proposal consumes only the accepted LOCAL ARM64 cohort: product `9a819317efb3a13fa27cd86f884be2be00898fc0`, tree `1528b61ba463d9262d6252d54174748a176f313b`, build workflow `036a392656b4b5070cc86a11d5ca3258b7b868d2`, run `37016520583`, job `110868629053`, attempt 1. Artifact API identity remains ID `11232871527`, name `riauth-local-arm64-37016520583-1`, reported outer digest `sha256:fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30`. Root did not retain or re-hash that outer ZIP; this proposal requires the **future actual download** to match it before consuming any ZIP member. No current transport verification is claimed.

Receipt `docs/roadmap/evidence/wave30-a09-native-arm64-37016520583.json` was reread at immutable `a74d3225dd8c845d0c7dff43ab922749c3a957b5`, complete SHA-256 `2cbf8ee46dbdd8b2ab43ad933913dd0a16c20b3183497c2d3b9cf3d1287da227`. All 18 already-downloaded files were read/re-hashed without opening an archive or executing a binary/script: 11 logs, evidence JSON, resources JSONL and five archives. The fixed controller constants contain each exact member size/SHA, all five archive/binary sizes/SHAs, both license/notice identities, build inputs, source/run/workflow identities and the prior resource aggregates. No outer ZIP was present or manufactured. The earlier dated metadata limitation remains part of this report.

The root-selected `validator_source_sha` must be a full lowercase 40-character published commit containing helper SHA-256 `575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1` and all four unchanged import hashes. It must differ from the fixed product source and the actual new workflow source; no invented validator publication SHA is supplied. Checkout uses that exact input, then checks real Git HEAD and every protected helper before transport. The controller records actual workflow SHA, GitHub SHA, new run/attempt and its own measured text SHA separately from product/build/validator pins. Archive provenance is the old real build receipt; validator provenance is the newly selected reviewed source; validation provenance would be the fresh actual run.

### Operator decision, resource and transport boundaries

One manually dispatched, 20-minute, native `ubuntu-24.04-arm` job has only `contents: read` and `actions: read`, a fixed concurrency group and `cancel-in-progress: false`. It has five steps: private preflight/supervisor; pinned checkout; one controlled sample; always cleanup/finalization; always fixed sanitized upload. Native Linux/aarch64, actual GitHub-hosted runner context, unprivileged UID and **at least 30 GiB free in every sampled scope** must pass before checkout. Existing `RUNNER_TEMP` is read only inside the shell step; `A09_ROOT` is exported there and persisted by literal `printf` to `GITHUB_ENV`. Job env uses only the valid inputs context. Checkout/upload use the same reviewed action pins as the accepted build workflow. The previous job-env `runner.temp` defect is not repeated.

A dedicated monitor starts before checkout and continues through owned cleanup: intended interval 2 seconds, actual UTC/monotonic gap, free-byte minima and device ID for host root/workspace/private root. Scopes sharing a device are not added as separate reserves. Sample failure, unavailable sampling or a gap/heartbeat over 6 seconds refuses/terminates owned work; the first sample requires 30 GiB, subsequent samples stop at 10 GiB, and an observed value below 8 GiB is a distinct floor failure. This is an actual-sampling policy, not a guarantee against unobserved instantaneous consumption. The resource log is capped at 2 MiB. The supervisor has an 18-minute absolute lifetime with a 60-second cleanup reserve inside the 20-minute job; a helper cannot start without its full 600-second allowance plus that reserve. No build, Cargo, cache eviction, threshold reduction, larger-runner/provider substitution or implicit setup is present. The old run's minimum 104.4496 GiB/maximum 5.40448-second gap does not establish capacity/availability of a future runner.

Transport is exactly one metadata GET for the fixed same-repository artifact and one logical ZIP download, each supervised for at most 120 seconds, without retries. Metadata is bounded to 2 MiB; ZIP to 256 MiB, with finite private output and 15-second socket operations under the external wall bound. The ZIP endpoint may make one signed HTTPS storage redirect; authorization is stripped at the origin change, and a second redirect/unlisted storage host fails. The signed URL/response body/token never appears in public evidence. Fixed identity, run/head SHA, expiration, size and digest must match. The entire ZIP must reach EOF, satisfy advertised/API size and SHA before atomic consumption; partial/overlarge/hash-wrong bytes are never extracted. Sanitized HTTP status/fixed failure code is retained without raw errors or URLs.

ZIP extraction manually permits exactly the 18 known regular paths (five under `archives/`), exact names/sizes/hashes, no duplicate, extra, encrypted, directory, link, NUL-truncated, traversal or unsupported-compression entry. No automatic extraction API is used. The two JSON/log identities and prior resource sample/minimum/gap aggregates are verified. Each fixed tar must then expose exactly three regular non-sparse members—its binary, LICENSE and THIRD_PARTY_NOTICES.md—with exact known size/hash and no PAX/link/extra member. All five installed binaries must be ELF64/little-endian/AArch64 before any native execution. The compressed archives total 48,748,203 bytes; binaries total 125,824,744 bytes; five checked document pairs total 3,768,635 bytes. These are measured accepted-file sizes, not a new host free-space measurement or a PostgreSQL maximum-write promise.

### Installed PostgreSQL and owned retention/cleanup prerequisite

Only already-installed regular native tools at `/usr/lib/postgresql/16/bin` are accepted: initdb, pg_ctl, createdb, psql and postgres. The controller verifies root ownership/non-writable permission, ELF/hash, then five finite version commands (10 seconds each), matching sanitized PostgreSQL 16 versions. It prepends exactly that directory to the helper PATH. Missing tools/version/native evidence fails; there is no apt/setup/download/container or alternate PostgreSQL fallback. The fresh runner's tool availability remains unobserved.

Concrete source issue found in the pinned helper: its `TemporaryDirectory` removes the fixture when leaving the context, including if the helper's finally-block PostgreSQL stop raises. Simply timing out that process would not make fixture deletion safe. This was reported before designing a cleanup adaptation. The proposed **inline private bootstrap**, which root must review as part of this new workflow seam, replaces only that exact `TemporaryDirectory(prefix="local-a08-postgres-", dir=evidence.parent)` context with one fresh marked retained directory. It refuses any other call shape, leaves the helper/assertions/SQL/API/import files byte-exact, and never deletes from the adapter. This intentionally changes temporary-fixture retention; it is not represented as identical cleanup behavior. No existing helper modification is proposed.

The persistent owned supervisor enables and reads back Linux `PR_SET_CHILD_SUBREAPER` via Python standard-library ctypes/libc `prctl`, before any helper child exists. **That native OS capability is an explicit unexercised runtime prerequisite**, needed to own and reap daemonized PostgreSQL descendants across shell steps; it is not a new package/dependency or a substitute provider. Missing/failed capability, process identity or safe directory-removal support fails. No ctypes/library/native execution occurred in this design turn.

Only one `python3 -I -B` bootstrap executes the pinned current-format-3 helper, at most 600 seconds. Its imports are checked before and after; it uses the original ordinary password/user-ID/group/auditor audit-allow/users-deny/logged-out-session refusal sample, complete format-3/16-rate E→P→E preservation, authentication/general-rate plan/start refusals and existing connected-client refusal. The encrypted module is imported unchanged solely for `live_identity_and_grant`; its historical format-2 runner/probe is not called. Shared equality excludes transient session IDs/expiry, exactly as the accepted helper. No client/TLS/device/encrypted-storage/full-shared campaign is added.

Owned stdout/stderr are drained separately to mode-600 private finite captures (8 MiB per channel; smaller version/Git captures), never printed/uploaded; pipe loss/overflow fails. PostgreSQL database/WAL files do **not** inherit a capture file-size rlimit. Native child env excludes retrieval tokens, proxy/Python-path/optimization and PostgreSQL credential variables. Fixture/config/database/password/session/raw JSON/raw helper error text remain private.

After any success/failure/timeout, cleanup first verifies fresh root/fixture nonce, UID, marker, exact cluster path, and—if present—the bounded postmaster PID file/cluster/fresh timestamp plus live owned PID/start fingerprint/native postgres hash/cmdline. Daemon ancestry is owned by the verified subreaper; PID identity is rechecked before signaling. Only owned descendants receive TERM (5 seconds), then KILL (5 seconds), followed by actual reaping and a fresh remaining-process scan. A missing live PostgreSQL PID-file proof, unavailable ownership identity, signal/reap/capture-reader problem or surviving process fails cleanup and retains the fixture privately. Proven-owned process-free scratch is deleted only after shutdown/reap; only this job's fresh private captures/download/installed/fixture paths are removed, never source or unrelated cluster/cache/process. First phase failure stays first; cleanup failures are added separately. The remaining capture readers have one shared 3-second bound. A forced VM loss/job kill can prevent finalization/upload; the design does not claim cleanup evidence in that case.

### Fixed sanitized evidence and held future action

Only four explicit files may be uploaded using pinned upload-artifact: `public/controller.json`, `public/resources.jsonl`, `public/cleanup.json`, `public/helper-redacted.json`. They contain fixed phases/codes/actual exits, expected/observed hashes and source/run identities, actual resource samples, cleanup counts/states/failures, and either a fixed not-run placeholder or a strictly key/type/value/hash-validated redacted helper report. The helper report exposes only the allowlisted non-secret assertions/counts/hash/edition fields and sanitized matching PostgreSQL version; raw report hash is retained. Raw stdout/stderr, signed URLs, token/session/user/config/database material, archives/ZIP/temp files and private fixture paths are excluded. Failure retains fixed phase/exit/capture hashes, transport status/codes and cleanup evidence; there is no blind retry.

The future source reservation requested for root decision is **only the new workflow path**, exactly the complete text below, plus an append-only report evidence phase after materialization/static validation. Existing helpers/workflows remain protected. After immutable source review/integration/publication, root alone may reserve one fresh hosted download/PG/helper slot and select a full published validator SHA. The prospective dispatch command is `gh workflow run check-local-shared-handoff.yml --repo Rhein-Industries/riAuth --ref main -f validator_source_sha=ROOT_REVIEWED_FULL_SHA`; it is documentation only and was not issued. Any metadata expiration/digest/path/tool/resource/proof mismatch is a terminal failure of this one attempt, requiring a new explicit root decision.

### Complete proposed new workflow (not created or executed)

```yaml
name: Check local native shared handoff

on:
  workflow_dispatch:
    inputs:
      validator_source_sha:
        description: Root-reviewed full source SHA containing the exact pinned validator and imports
        required: true
        type: string

permissions:
  contents: read
  actions: read

concurrency:
  group: riauth-local-shared-handoff-arm64
  cancel-in-progress: false

jobs:
  native-arm64:
    runs-on: ubuntu-24.04-arm
    timeout-minutes: 20
    defaults:
      run:
        shell: bash
    env:
      A09_VALIDATOR_SOURCE_SHA: ${{ inputs.validator_source_sha }}
    steps:
      - name: Start owned native preflight and continuous resource supervisor
        timeout-minutes: 1
        env:
          A09_RETRIEVAL_TOKEN: ${{ github.token }}
        run: |
          set -euo pipefail
          umask 077
          export A09_ROOT="$RUNNER_TEMP/riauth-a09-shared-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
          printf 'A09_ROOT=%s\n' "$A09_ROOT" >> "$GITHUB_ENV"
          mkdir -m 700 "$A09_ROOT"
          cat > "$A09_ROOT/controller.py" <<'PY'
          # Proposal only: run on the explicitly authorized fresh native GitHub job.
          import ctypes
          import datetime
          import hashlib
          import json
          import os
          import pathlib
          import platform
          import re
          import resource
          import runpy
          import shutil
          import signal
          import stat
          import struct
          import subprocess
          import sys
          import tarfile
          import tempfile
          import threading
          import time
          import urllib.error
          import urllib.parse
          import urllib.request
          import zipfile

          FIXED = json.loads(r'''{
            "repository": "Rhein-Industries/riAuth",
            "product_sha": "9a819317efb3a13fa27cd86f884be2be00898fc0",
            "product_tree": "1528b61ba463d9262d6252d54174748a176f313b",
            "build_workflow_sha": "036a392656b4b5070cc86a11d5ca3258b7b868d2",
            "build_run": 37016520583,
            "build_job": 110868629053,
            "artifact_id": 11232871527,
            "artifact_name": "riauth-local-arm64-37016520583-1",
            "zip_sha256": "fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30",
            "files": {
              "apt-install.log": {
                "bytes": 5262,
                "sha256": "f6a65260955ad1d97b665b697b960013880647bcd84e7d15bdcf004875df8074"
              },
              "apt-update.log": {
                "bytes": 2350,
                "sha256": "19f808a68f6d524d9f9935feddf906fb1ae08319070432aaafa24242c9f52af6"
              },
              "archives/local-riauth-essentials-aarch64.tar.gz": {
                "bytes": 13996440,
                "sha256": "848758095ccab44919ddc643dd91ced3f77d4acc083be0edf63925fdb2b51981"
              },
              "archives/local-riauth-maintenance-essentials-aarch64.tar.gz": {
                "bytes": 5946216,
                "sha256": "e43ed01029baeb7cae4821e125d168d687a7e60ab701afc8f46c9fea442adc3a"
              },
              "archives/local-riauth-maintenance-platform-aarch64.tar.gz": {
                "bytes": 6788229,
                "sha256": "510c55acf014273a53acb5b87d55b40381b9d00f51ef8b654493a307250f92df"
              },
              "archives/local-riauth-platform-aarch64.tar.gz": {
                "bytes": 18375977,
                "sha256": "9f615ca8fbe3cb1dfd400dea6c6d03a1f2677d7510b0ad6f1541c974a72723e8"
              },
              "archives/local-riauthctl-aarch64.tar.gz": {
                "bytes": 3641341,
                "sha256": "d826f26176ababda4fe3930e00a051c0416e2b10c9ec22a1f31ca4f4ef8118da"
              },
              "build-client.log": {
                "bytes": 5282,
                "sha256": "4c2345f5d1f2c58cb6606bc672a960ae9dc233e80569158d05c210966f6fce49"
              },
              "build-essentials.log": {
                "bytes": 28384,
                "sha256": "281169d70624e7b7ebc4605de2650b2c4a9db100bea9a60d3b54bc07d9bbeec3"
              },
              "build-platform.log": {
                "bytes": 17533,
                "sha256": "46294d77993c0ea5d00fe8650c92522599fb47f380742866da68a4096abf34a4"
              },
              "capabilities-essentials.log": {
                "bytes": 13183,
                "sha256": "4bdc14319c2d0d6c70997f688837533919379906efef266d9b4648d3ea710c86"
              },
              "capabilities-platform.log": {
                "bytes": 14616,
                "sha256": "beb2ae858a9d081910939d488ba04249f728fcbbbc85daa778698478643c3e02"
              },
              "evidence.json": {
                "bytes": 60299,
                "sha256": "fd8280125b38e6afbe13970be4617df8fdbe72e7280a4e6dedb2f2a84e4c9ab1"
              },
              "focused-native-archive-smoke.log": {
                "bytes": 76,
                "sha256": "d0b3f6ce4435ad1d80cdbdaff55477e24ce2942ac26d512761a27b598d49ef48"
              },
              "linux-baseline.log": {
                "bytes": 400,
                "sha256": "87458b3b1f1ca0e5c693ade771de7007c3783bdc692fa6b39eb6414bf2af2763"
              },
              "native-dependency-versions.log": {
                "bytes": 142,
                "sha256": "0eeacaea693f7b9759023b1e40d5e8a6628e8542bb927792da179ba1d125db65"
              },
              "resources.jsonl": {
                "bytes": 278604,
                "sha256": "f81d488c07a2616e4541926c7d98bf5a87bfb38d97923a5c200cc03bf2a321ad"
              },
              "rustc-version.log": {
                "bytes": 36,
                "sha256": "2073de5abf149185c2f4d7473dc833bc921b99135560d24ae7b2293f33a9c376"
              }
            },
            "products": [
              {
                "archive": "local-riauth-essentials-aarch64.tar.gz",
                "archive_bytes": 13996440,
                "archive_sha256": "848758095ccab44919ddc643dd91ced3f77d4acc083be0edf63925fdb2b51981",
                "binary": "riauth",
                "binary_bytes": 37242432,
                "binary_sha256": "0e07481de56505904864e42d63be743b235001e95c44af772654d1185a4f40e8",
                "declared_build_features": [
                  "essentials"
                ],
                "edition": "essentials",
                "target": "aarch64-unknown-linux-gnu"
              },
              {
                "archive": "local-riauth-platform-aarch64.tar.gz",
                "archive_bytes": 18375977,
                "archive_sha256": "9f615ca8fbe3cb1dfd400dea6c6d03a1f2677d7510b0ad6f1541c974a72723e8",
                "binary": "riauth",
                "binary_bytes": 49565208,
                "binary_sha256": "c8c93c0605c83271049dd15794a8960cb3021245f0b439b7de3bbb9d37aec0ef",
                "declared_build_features": [
                  "essentials",
                  "platform"
                ],
                "edition": "platform",
                "target": "aarch64-unknown-linux-gnu"
              },
              {
                "archive": "local-riauth-maintenance-essentials-aarch64.tar.gz",
                "archive_bytes": 5946216,
                "archive_sha256": "e43ed01029baeb7cae4821e125d168d687a7e60ab701afc8f46c9fea442adc3a",
                "binary": "riauth-maintenance",
                "binary_bytes": 13768992,
                "binary_sha256": "8c6b1d9a8df61aa5c7d16105b609a0e5dc67e9609d35de85fc822c78033a30ac",
                "declared_build_features": [
                  "essentials"
                ],
                "edition": "essentials",
                "target": "aarch64-unknown-linux-gnu"
              },
              {
                "archive": "local-riauth-maintenance-platform-aarch64.tar.gz",
                "archive_bytes": 6788229,
                "archive_sha256": "510c55acf014273a53acb5b87d55b40381b9d00f51ef8b654493a307250f92df",
                "binary": "riauth-maintenance",
                "binary_bytes": 16062840,
                "binary_sha256": "2b2461adae8101f4e38c2842963bd273403c4c1ad06c6528a45ab1fd12f7bff1",
                "declared_build_features": [
                  "essentials",
                  "platform"
                ],
                "edition": "platform",
                "target": "aarch64-unknown-linux-gnu"
              },
              {
                "archive": "local-riauthctl-aarch64.tar.gz",
                "archive_bytes": 3641341,
                "archive_sha256": "d826f26176ababda4fe3930e00a051c0416e2b10c9ec22a1f31ca4f4ef8118da",
                "binary": "riauthctl",
                "binary_bytes": 9185272,
                "binary_sha256": "7c8831ed42d89a31f3ab2042cd03ab460ef19b991a41b7667b1bfe9c91c04291",
                "declared_build_features": [],
                "edition": "client",
                "target": "aarch64-unknown-linux-gnu"
              }
            ],
            "inputs": {
              "Cargo.lock": "b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426",
              "Cargo.toml": "58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8",
              "LICENSE": "ef79ab7079893da02af81ebb8a57bec27dc7a601d505b228b1d5356a3aa5b1a9",
              "THIRD_PARTY_NOTICES.md": "142c0e5150de9436513d9f6f215c5422b8a3af84d4eb7d0708f155e3e18fce05",
              "crates/riauthctl/Cargo.lock": "2998555ddb2d00e8130a970fcacfed19dfee7a0b2e3305011ebd40746f67b4db",
              "crates/riauthctl/Cargo.toml": "af385d4d989c53987396edfdfa684cd3902a603d1cf65dd31ac72da7f08de9ea",
              "scripts/check-edition-artifacts.py": "2f477665ed584bc148fb230c0658f6d0cc66aceaa5af631c4dc467872fa9ecce"
            },
            "documents": {
              "LICENSE": {
                "bytes": 1076,
                "sha256": "ef79ab7079893da02af81ebb8a57bec27dc7a601d505b228b1d5356a3aa5b1a9"
              },
              "THIRD_PARTY_NOTICES.md": {
                "bytes": 752651,
                "sha256": "142c0e5150de9436513d9f6f215c5422b8a3af84d4eb7d0708f155e3e18fce05"
              }
            },
            "imports": {
              "check-exact-edition-matrix.py": "f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887",
              "check-installed-release-gate.py": "cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5",
              "check-local-edition-transition-postgres.py": "575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1",
              "check-local-encrypted-edition-transition.py": "09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e",
              "spdx_sbom.py": "ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c"
            },
            "original_resource_samples": 1308,
            "original_resource_minimum": {
              "host_root": 112151941120,
              "private": 112151941120,
              "workspace": 112151941120
            },
            "original_maximum_gap": 5.40448
          }''')
          GIB = 1024 ** 3
          START = 30 * GIB
          STOP = 10 * GIB
          FLOOR = 8 * GIB
          INTERVAL = 2.0
          MAX_GAP = 6.0
          ZIP_LIMIT = 256 * 1024 ** 2
          CAPTURE_LIMIT = 8 * 1024 ** 2
          ROOT = pathlib.Path(os.environ["A09_ROOT"])
          PRIVATE = ROOT / "private"
          PUBLIC = ROOT / "public"
          OWNER = ROOT / "owner.json"
          HELPER = "check-local-edition-transition-postgres.py"
          PG_NAMES = ("initdb", "pg_ctl", "createdb", "psql", "postgres")
          PUBLIC_NAMES = ("controller.json", "resources.jsonl", "cleanup.json", "helper-redacted.json")


          class Refusal(Exception):
              def __init__(self, code):
                  self.code = code
                  super().__init__(code)


          def require(condition, code):
              if not condition:
                  raise Refusal(code)


          def digest(path):
              h = hashlib.sha256()
              with path.open("rb") as source:
                  for block in iter(lambda: source.read(1024 ** 2), b""):
                      h.update(block)
              return h.hexdigest()


          def regular(path):
              info = path.lstat()
              require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1, "regular_file_required")
              return info


          def write_new(path, value):
              require(not path.exists(), "owned_file_already_exists")
              temporary = path.with_name(path.name + ".new")
              with os.fdopen(os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), "w") as out:
                  json.dump(value, out, sort_keys=True)
                  out.write("\n")
              # Each name has one designated writer inside the fresh mode-0700 directory.
              temporary.replace(path)


          def atomic(path, value):
              temporary = path.with_name(path.name + ".next")
              with temporary.open("w") as out:
                  json.dump(value, out, indent=2, sort_keys=True)
                  out.write("\n")
              temporary.chmod(0o600)
              temporary.replace(path)


          def read_json(path, maximum=2 * 1024 ** 2):
              require(regular(path).st_size <= maximum, "json_size_limit")
              return json.loads(path.read_text())


          def native():
              require(platform.system() == "Linux" and platform.machine() == "aarch64", "native_arm64_required")
              require(os.environ.get("RUNNER_OS") == "Linux" and os.environ.get("RUNNER_ARCH") == "ARM64",
                      "runner_native_context_required")
              require(os.environ.get("RUNNER_ENVIRONMENT") == "github-hosted", "hosted_runner_required")
              require(os.getuid() != 0, "unprivileged_user_required")


          def environment():
              # No token, proxy, Python search/optimization or PostgreSQL credentials reach native children.
              allowed = ("HOME", "USER", "LOGNAME", "LANG", "LC_ALL", "TZ", "RUNNER_TRACKING_ID")
              result = {name: os.environ[name] for name in allowed if name in os.environ}
              result.update(PATH=os.environ.get("PATH", ""), TMPDIR=str(PRIVATE), PYTHONDONTWRITEBYTECODE="1")
              return result


          def elf(path):
              regular(path)
              with path.open("rb") as source:
                  header = source.read(64)
              require(len(header) == 64 and header[:6] == b"\x7fELF\x02\x01" and
                      header[6] == 1 and struct.unpack("<H", header[16:18])[0] in (2, 3) and
                      struct.unpack("<H", header[18:20])[0] == 183 and
                      struct.unpack("<H", header[52:54])[0] == 64, "native_elf_required")


          def identity(pid):
              # PID fingerprint is UID + kernel start ticks, not an untrusted command string.
              try:
                  directory = pathlib.Path("/proc") / str(pid)
                  text = (directory / "stat").read_text()
                  tail = text[text.rfind(")") + 2:].split()
                  return {"pid": pid, "ppid": int(tail[1]), "start": int(tail[19]),
                          "uid": directory.stat().st_uid, "state": tail[0]}
              except (FileNotFoundError, ProcessLookupError):
                  return None
              except (OSError, ValueError, IndexError):
                  raise Refusal("process_identity_unavailable")


          def owned_descendants():
              entries = {}
              for entry in pathlib.Path("/proc").iterdir():
                  if entry.name.isdigit():
                      item = identity(int(entry.name))
                      if item is not None:
                          entries[item["pid"]] = item
              owned = {os.getpid()}
              while True:
                  more = {pid for pid, item in entries.items() if item["ppid"] in owned}
                  if more <= owned:
                      break
                  owned |= more
              return [entries[pid] for pid in owned - {os.getpid()} if pid in entries]


          def same_process(item):
              current = identity(item["pid"])
              return current is not None and current["uid"] == os.getuid() and current["start"] == item["start"]


          def signal_owned(item, sig):
              if same_process(item):
                  os.kill(item["pid"], sig)


          def reap_owned():
              count = 0
              while True:
                  try:
                      pid, unused = os.waitpid(-1, os.WNOHANG)
                      if not pid:
                          return count
                      count += 1
                  except ChildProcessError:
                      return count


          def owner():
              value = read_json(OWNER, 4096)
              require(value["uid"] == os.getuid() and value["root"] == str(ROOT) and
                      value["run"] == int(os.environ["GITHUB_RUN_ID"]) and
                      value["attempt"] == int(os.environ["GITHUB_RUN_ATTEMPT"]), "owner_marker_mismatch")
              require(ROOT.resolve() == ROOT and ROOT.parent == pathlib.Path(os.environ["RUNNER_TEMP"]).resolve(),
                      "owned_root_path_mismatch")
              require(stat.S_ISDIR(ROOT.lstat().st_mode) and ROOT.stat().st_uid == os.getuid() and
                      stat.S_IMODE(ROOT.stat().st_mode) == 0o700, "owned_root_permissions")
              return value


          def request(name):
              value = owner()
              write_new(PRIVATE / (name + ".request"), {"nonce": value["nonce"]})


          def wait_result(name, maximum):
              started = time.monotonic()
              value = owner()
              while time.monotonic() - started < maximum:
                  path = PRIVATE / (name + ".result")
                  if path.exists():
                      result = read_json(path, 4096)
                      require(result.get("nonce") == value["nonce"], "result_marker_mismatch")
                      return result
                  supervisor = identity(value["supervisor_pid"])
                  require(supervisor is not None and supervisor["start"] == value["supervisor_start"],
                          "supervisor_lost")
                  time.sleep(0.2)
              raise Refusal("supervisor_reply_timeout")


          class NoRedirect(urllib.request.HTTPRedirectHandler):
              def redirect_request(self, req, fp, code, msg, headers, newurl):
                  return None


          def transfer(kind):
              # Only supervisor invokes this subprocess, once per kind; wall clock is bounded externally.
              value = owner()
              require(read_json(PRIVATE / "transport-owner.json", 4096) == {"nonce": value["nonce"]},
                      "transport_not_authorized")
              require(kind in ("metadata", "zip"), "transport_kind_invalid")
              write_new(PRIVATE / (kind + "-attempt.json"), {"nonce": value["nonce"]})
              token = os.environ["A09_RETRIEVAL_TOKEN"]
              require(bool(token), "retrieval_token_missing")
              api = "https://api.github.com/repos/" + FIXED["repository"] + "/actions/artifacts/" + str(FIXED["artifact_id"])
              url = api if kind == "metadata" else api + "/zip"
              maximum = 2 * 1024 ** 2 if kind == "metadata" else ZIP_LIMIT
              name = "metadata.json" if kind == "metadata" else "artifact.zip"
              part = PRIVATE / (name + ".part")
              target = PRIVATE / name
              require(not target.exists() and not part.exists(), "transport_already_attempted")
              resource.setrlimit(resource.RLIMIT_FSIZE, (maximum, maximum))
              opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
              headers = {"Authorization": "Bearer " + token, "Accept": "application/vnd.github+json",
                         "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "riauth-local-a09-build-free"}
              started = time.monotonic()
              try:
                  response = opener.open(urllib.request.Request(url, headers=headers), timeout=15)
              except urllib.error.HTTPError as error:
                  atomic(PRIVATE / (kind + "-http.json"), {"status": error.code})
                  require(kind == "zip" and error.code == 302, "transport_http_refusal")
                  location = error.headers.get("Location", "")
                  error.close()
                  parsed = urllib.parse.urlsplit(location)
                  host = parsed.hostname or ""
                  require(parsed.scheme == "https" and parsed.username is None and parsed.password is None and
                          parsed.port in (None, 443) and not parsed.fragment and
                          (host.endswith(".blob.core.windows.net") or host.endswith(".actions.githubusercontent.com")),
                          "transport_redirect_refused")
                  # The signed artifact URL is private. Never forward GitHub Authorization across origins.
                  response = opener.open(urllib.request.Request(location, headers={"User-Agent": "riauth-local-a09-build-free"}),
                                         timeout=15)
              with response:
                  atomic(PRIVATE / (kind + "-http.json"), {"status": response.status})
                  require(response.status == 200, "transport_status")
                  advertised = response.headers.get("Content-Length")
                  if advertised is not None:
                      require(advertised.isdigit() and int(advertised) <= maximum, "transport_advertised_bound")
                  size = 0
                  h = hashlib.sha256()
                  with os.fdopen(os.open(part, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), "wb") as out:
                      while True:
                          require(time.monotonic() - started < 120, "transport_wall_bound")
                          block = response.read(1024 ** 2)
                          if not block:
                              break
                          size += len(block)
                          require(size <= maximum, "transport_size_bound")
                          h.update(block)
                          out.write(block)
                  require(size > 0 and (advertised is None or size == int(advertised)), "transport_incomplete")
              if kind == "zip":
                  require(h.hexdigest() == FIXED["zip_sha256"], "zip_digest_mismatch")
              else:
                  metadata = read_json(part)
                  run = metadata.get("workflow_run", {})
                  require(metadata.get("id") == FIXED["artifact_id"] and
                          metadata.get("name") == FIXED["artifact_name"] and
                          metadata.get("digest") == "sha256:" + FIXED["zip_sha256"] and
                          metadata.get("expired") is False and
                          type(metadata.get("size_in_bytes")) is int and
                          0 < metadata["size_in_bytes"] <= ZIP_LIMIT and
                          run.get("id") == FIXED["build_run"] and
                          run.get("head_sha") == FIXED["product_sha"], "artifact_metadata_mismatch")
              # Incomplete, overlarge or hash-wrong streams stay .part and are never consumed.
              require(time.monotonic() - started < 120, "transport_wall_bound")
              part.replace(target)
              write_new(PRIVATE / (kind + "-transport.json"), {"bytes": size, "sha256": h.hexdigest(),
                                                              "elapsed_seconds": time.monotonic() - started,
                                                              "owner_nonce": value["nonce"]})


          BOOTSTRAP = r'''
          import hashlib, json, os, pathlib, runpy, sys, tempfile
          sys.dont_write_bytecode = True
          root = pathlib.Path(os.environ["A09_ROOT"])
          private = root / "private"
          marker = json.loads((root / "owner.json").read_text())
          helper = pathlib.Path(sys.argv[1])
          artifacts = pathlib.Path(sys.argv[2])
          expected = sys.argv[3]
          if hashlib.sha256(helper.read_bytes()).hexdigest() != expected:
              raise RuntimeError("helper_changed")
          class RetainedFixture:
              def __init__(self, suffix=None, prefix=None, dir=None, **kwargs):
                  if suffix is not None or prefix != "local-a08-postgres-" or pathlib.Path(dir).resolve() != private or kwargs:
                      raise RuntimeError("unexpected_temporary_directory")
                  self.path = private / "local-a08-postgres-owned"
                  self.path.mkdir(mode=0o700)
                  data = {"nonce": marker["nonce"], "root": str(root), "uid": os.getuid()}
                  fd = os.open(self.path / ".a09-owner.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
                  with os.fdopen(fd, "w") as out:
                      json.dump(data, out, sort_keys=True)
              def __enter__(self):
                  return str(self.path)
              def __exit__(self, *exc):
                  return False
          # Retain exactly this fixture; do not alter helper assertions, subprocesses, SQL or output.
          tempfile.TemporaryDirectory = RetainedFixture
          sys.argv = [str(helper), "--artifacts", str(artifacts), "--evidence", str(private / "helper.json"),
                      "--source-revision", sys.argv[4]]
          runpy.run_path(str(helper), run_name="__main__")
          '''


          class Controller:
              def __init__(self):
                  self.owner = owner()
                  self.deadline = self.owner["started_monotonic"] + 1080
                  self.lock = threading.RLock()
                  self.last_sample = None
                  self.resource_failure = None
                  self.end_monitor = threading.Event()
                  self.minima = {}
                  self.samples = 0
                  self.max_gap = 0.0
                  self.tools = {}
                  self.capture_readers = []
                  self.capture_records = []
                  self.interrupted = None
                  self.first_failure = None
                  self.phases = []
                  self.cleanup_failures = []
                  self.reaped = 0
                  self.data = {"schema": "riauth.local-build-free-shared-controller/v1",
                               "official_release": False, "full_shared_gate": "not_certified",
                               "sample_result": "not_run", "controller_exit_code": None,
                               "product_sha": FIXED["product_sha"],
                               "build_workflow_sha": FIXED["build_workflow_sha"],
                               "build_run": FIXED["build_run"], "build_job": FIXED["build_job"],
                               "artifact_id": FIXED["artifact_id"], "artifact_name": FIXED["artifact_name"],
                               "expected_zip_sha256": FIXED["zip_sha256"],
                               "validator_source_sha": os.environ["A09_VALIDATOR_SOURCE_SHA"],
                               "expected_validator_sha256": FIXED["imports"][HELPER],
                               "controller_sha256": digest(ROOT / "controller.py"),
                               "workflow_sha": os.environ["GITHUB_WORKFLOW_SHA"],
                               "github_sha": os.environ["GITHUB_SHA"],
                               "repository": os.environ["GITHUB_REPOSITORY"],
                               "run_id": int(os.environ["GITHUB_RUN_ID"]),
                               "run_attempt": int(os.environ["GITHUB_RUN_ATTEMPT"]),
                               "event": os.environ["GITHUB_EVENT_NAME"],
                               "action_pins": {
                                   "checkout": "3d3c42e5aac5ba805825da76410c181273ba90b1",
                                   "upload": "ea165f8d65b6e75b540449e92b4886f43607fa02"},
                               "resource_policy": {"start_bytes": START, "stop_bytes": STOP, "floor_bytes": FLOOR,
                                                   "intended_interval_seconds": INTERVAL, "maximum_gap_seconds": MAX_GAP},
                               "helper_timeout_seconds": 600, "global_timeout_seconds": 1080}

              def fail(self, phase, code, exit_code=None):
                  if self.first_failure is None:
                      self.first_failure = {"phase": phase, "code": code, "exit_code": exit_code}

              def save(self):
                  with self.lock:
                      self.data.update(first_failure=self.first_failure, phases=self.phases,
                                       cleanup_failures=self.cleanup_failures,
                                       resource_samples=self.samples, minimum_free_bytes=self.minima,
                                       maximum_actual_gap_seconds=self.max_gap)
                      atomic(PUBLIC / "controller.json", self.data)

              def sample(self, initial=False):
                  paths = {"host_root": pathlib.Path("/"),
                           "workspace": pathlib.Path(os.environ["GITHUB_WORKSPACE"]),
                           "private": ROOT}
                  now = time.monotonic()
                  gap = 0.0 if self.last_sample is None else now - self.last_sample
                  free = {name: shutil.disk_usage(path).free for name, path in paths.items()}
                  devices = {name: os.stat(path).st_dev for name, path in paths.items()}
                  item = {"at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                          "monotonic_seconds": now - self.owner["started_monotonic"],
                          "actual_gap_seconds": gap, "free_bytes": free, "device_ids": devices}
                  with self.lock:
                      output = PUBLIC / "resources.jsonl"
                      require(output.stat().st_size < 2 * 1024 ** 2, "resource_log_bound")
                      with output.open("a") as out:
                          out.write(json.dumps(item, sort_keys=True) + "\n")
                      self.last_sample = now
                      self.samples += 1
                      self.max_gap = max(self.max_gap, gap)
                      for name, amount in free.items():
                          self.minima[name] = min(self.minima.get(name, amount), amount)
                      require(gap <= MAX_GAP, "resource_sample_gap")
                      require(all(amount >= FLOOR for amount in free.values()), "resource_floor_observed")
                      require(all(amount >= (START if initial else STOP) for amount in free.values()),
                              "resource_start_capacity" if initial else "resource_stop_capacity")

              def monitor(self):
                  due = time.monotonic() + INTERVAL
                  while not self.end_monitor.wait(max(0, due - time.monotonic())):
                      try:
                          self.sample()
                      except Exception as error:
                          with self.lock:
                              self.resource_failure = error.code if isinstance(error, Refusal) else "resource_sample_unavailable"
                          return
                      due = time.monotonic() + INTERVAL

              def guard(self):
                  with self.lock:
                      require(self.resource_failure is None, self.resource_failure or "resource_sample_unavailable")
                      require(self.last_sample is not None and time.monotonic() - self.last_sample <= MAX_GAP,
                              "resource_sample_lost")
                  require(self.interrupted is None, "controller_signal")
                  require(time.monotonic() < self.deadline - 60, "global_cleanup_reserve")
                  if (PRIVATE / "finalize.request").exists():
                      require(read_json(PRIVATE / "finalize.request", 4096).get("nonce") == self.owner["nonce"],
                              "finalize_marker_mismatch")
                      raise Refusal("finalize_requested")

              def terminate(self, child):
                  if child.poll() is None:
                      fingerprint = identity(child.pid)
                      if fingerprint is not None:
                          signal_owned(fingerprint, signal.SIGTERM)
                      try:
                          child.wait(timeout=5)
                      except subprocess.TimeoutExpired:
                          if fingerprint is not None:
                              signal_owned(fingerprint, signal.SIGKILL)
                          child.wait(timeout=5)

              def command(self, name, argv, timeout, env=None, capture=CAPTURE_LIMIT):
                  self.guard()
                  started = time.monotonic()
                  if name == "focused-shared-helper":
                      require(self.deadline - started >= timeout + 60, "full_helper_budget_unavailable")
                  paths = [PRIVATE / (name + ".stdout"), PRIVATE / (name + ".stderr")]
                  row = {"phase": name, "exit_code": None, "elapsed_seconds": None, "status": "started"}
                  self.phases.append(row)
                  self.capture_records.append((row, paths))
                  self.save()
                  child = None
                  readers = []
                  capture_failure = threading.Event()

                  def drain(pipe, path):
                      try:
                          count = 0
                          with path.open("xb") as output:
                              while True:
                                  block = pipe.read(65536)
                                  if not block:
                                      break
                                  count += len(block)
                                  if count >= capture:
                                      capture_failure.set()
                                      break
                                  output.write(block)
                      except Exception:
                          capture_failure.set()
                      finally:
                          pipe.close()

                  try:
                      child = subprocess.Popen(argv, env=environment() if env is None else env,
                                               cwd=os.environ["GITHUB_WORKSPACE"], stdin=subprocess.DEVNULL,
                                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
                      for pipe, path in zip((child.stdout, child.stderr), paths):
                          reader = threading.Thread(target=drain, args=(pipe, path), daemon=True)
                          reader.start()
                          readers.append(reader)
                          self.capture_readers.append(reader)
                      while child.poll() is None:
                          self.guard()
                          require(time.monotonic() - started < timeout, "phase_timeout")
                          require(not capture_failure.is_set(), "private_capture_bound")
                          time.sleep(0.2)
                      row["exit_code"] = child.wait()
                      require(time.monotonic() - started < timeout, "phase_timeout")
                      for reader in readers:
                          reader.join(timeout=2)
                      require(not capture_failure.is_set() and not any(reader.is_alive() for reader in readers),
                              "private_capture_incomplete_or_bound")
                      require(row["exit_code"] == 0, "phase_exit")
                      row["status"] = "passed"
                      return paths[0]
                  except Exception as error:
                      code = error.code if isinstance(error, Refusal) else "phase_unavailable"
                      if child is not None:
                          try:
                              self.terminate(child)
                          except Exception:
                              self.cleanup_failures.append("phase_child_reap_failed")
                          row["exit_code"] = child.returncode
                      row["status"] = "failed"
                      self.fail(name, code, row["exit_code"])
                      raise Refusal(code)
                  finally:
                      # Descendants retaining a pipe are terminated in the final owned-process sweep.
                      row["elapsed_seconds"] = round(time.monotonic() - started, 6)
                      self.save()

              def source(self):
                  sha = os.environ["A09_VALIDATOR_SOURCE_SHA"]
                  require(re.fullmatch("[0-9a-f]{40}", sha) is not None, "validator_full_sha_required")
                  require(sha != FIXED["product_sha"] and sha != os.environ["GITHUB_WORKFLOW_SHA"], "source_roles_must_be_distinct")
                  root = pathlib.Path(os.environ["GITHUB_WORKSPACE"])
                  git = shutil.which("git")
                  require(git is not None, "git_missing")
                  output = self.command("validator-head", [git, "rev-parse", "HEAD"], 10, capture=4096)
                  require(output.read_text().strip() == sha, "validator_checkout_mismatch")
                  actual = {}
                  for name, expected in FIXED["imports"].items():
                      path = root / "scripts" / name
                      regular(path)
                      actual[name] = digest(path)
                      self.data["validator_import_sha256"] = dict(actual)
                      self.save()
                      require(actual[name] == expected, "validator_import_mismatch")
                  self.data["validator_import_sha256"] = actual
                  self.save()

              def download(self):
                  write_new(PRIVATE / "transport-owner.json", {"nonce": self.owner["nonce"]})
                  env = environment()
                  env.update({name: os.environ[name] for name in
                              ("A09_ROOT", "RUNNER_TEMP", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "A09_RETRIEVAL_TOKEN")})
                  for kind in ("metadata", "zip"):
                      try:
                          self.command("artifact-" + kind, [sys.executable, "-I", "-B", str(ROOT / "controller.py"), "transfer", kind],
                                       120, env)
                      except Refusal:
                          failure_path = PRIVATE / (kind + "-failure.json")
                          if failure_path.exists():
                              failure = read_json(failure_path, 4096)
                              require(set(failure) == {"code", "http_status"} and
                                      re.fullmatch("[a-z_]{1,80}", failure["code"]) is not None and
                                      (failure["http_status"] is None or
                                       type(failure["http_status"]) is int and 100 <= failure["http_status"] <= 599),
                                      "transfer_failure_shape")
                              self.data[kind + "_transport_failure"] = failure
                              if self.first_failure is not None and self.first_failure["phase"] == "artifact-" + kind:
                                  self.first_failure["code"] = failure["code"]
                              self.save()
                          raise
                      summary = read_json(PRIVATE / (kind + "-transport.json"), 4096)
                      require(summary.pop("owner_nonce") == self.owner["nonce"], "transport_owner_mismatch")
                      self.data[kind + "_transport"] = summary
                      self.save()
                  require(digest(PRIVATE / "artifact.zip") == FIXED["zip_sha256"], "zip_post_transfer_mismatch")
                  metadata = read_json(PRIVATE / "metadata.json")
                  require(regular(PRIVATE / "artifact.zip").st_size == metadata["size_in_bytes"], "zip_metadata_size_mismatch")

              def extract_zip(self):
                  extracted = PRIVATE / "download"
                  extracted.mkdir(mode=0o700)
                  (extracted / "archives").mkdir(mode=0o700)
                  expected = FIXED["files"]
                  with zipfile.ZipFile(PRIVATE / "artifact.zip") as archive:
                      entries = archive.infolist()
                      require(len(entries) == 18 and len({entry.filename for entry in entries}) == 18 and
                              {entry.filename for entry in entries} == set(expected), "zip_exact_allowlist")
                      for entry in entries:
                          self.guard()
                          parts = pathlib.PurePosixPath(entry.filename).parts
                          mode = entry.external_attr >> 16
                          require(not entry.is_dir() and not (entry.flag_bits & 1) and "\\" not in entry.filename and
                                  not entry.filename.startswith("/") and all(part not in ("", ".", "..") for part in parts) and
                                  stat.S_IFMT(mode) in (0, stat.S_IFREG) and
                                  entry.compress_type in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED) and
                                  entry.orig_filename == entry.filename and not (entry.external_attr & 0x10),
                                  "zip_regular_member_required")
                          spec = expected[entry.filename]
                          require(entry.file_size == spec["bytes"] and entry.compress_size <= ZIP_LIMIT, "zip_member_bound")
                          target = extracted / entry.filename
                          count = 0
                          h = hashlib.sha256()
                          with archive.open(entry) as source, os.fdopen(
                                  os.open(target, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), "wb") as out:
                              while True:
                                  self.guard()
                                  block = source.read(1024 ** 2)
                                  if not block:
                                      break
                                  count += len(block)
                                  require(count <= spec["bytes"], "zip_member_expansion_bound")
                                  h.update(block)
                                  out.write(block)
                          require(count == spec["bytes"] and h.hexdigest() == spec["sha256"], "zip_member_hash_mismatch")
                  self.data["validated_input_files"] = expected
                  self.save()
                  evidence = read_json(extracted / "evidence.json")
                  require(evidence["source_sha"] == FIXED["product_sha"] and evidence["source_tree"] == FIXED["product_tree"] and
                          evidence["github"]["GITHUB_WORKFLOW_SHA"] == FIXED["build_workflow_sha"] and
                          int(evidence["github"]["GITHUB_RUN_ID"]) == FIXED["build_run"] and
                          int(evidence["github"]["GITHUB_RUN_ATTEMPT"]) == 1 and
                          evidence["github"]["GITHUB_REPOSITORY"] == FIXED["repository"] and
                          evidence["official_release"] is False and evidence["shared_full_gate"] == "not_run" and
                          evidence["inputs"] == FIXED["inputs"] and evidence["source_verified"] is True and
                          evidence["native_archive_slice"] == "passed" and evidence["cleanup_errors"] == [], "build_evidence_identity")
                  products = [{k: v for k, v in product.items() if k != "observed_server_capabilities"}
                              for product in evidence["products"]]
                  require(products == FIXED["products"], "build_product_identity")
                  records = [json.loads(line) for line in (extracted / "resources.jsonl").read_text().splitlines()]
                  require(len(records) == FIXED["original_resource_samples"], "build_resource_sample_count")
                  minimum = {scope: min(item["free_bytes"][scope] for item in records)
                             for scope in ("host_root", "workspace", "private")}
                  times = [datetime.datetime.fromisoformat(item["at_utc"]).timestamp() for item in records]
                  gaps = [after - before for before, after in zip(times, times[1:])]
                  require(minimum == FIXED["original_resource_minimum"] and all(gap > 0 for gap in gaps) and
                          abs(max(gaps) - FIXED["original_maximum_gap"]) < 0.00001,
                          "build_resource_identity")

              def extract_binaries(self):
                  installed = PRIVATE / "installed"
                  installed.mkdir(mode=0o700)
                  for product in FIXED["products"]:
                      self.guard()
                      destination = installed / product["edition"]
                      destination.mkdir(mode=0o700, exist_ok=True)
                      expected = {product["binary"]: {"bytes": product["binary_bytes"], "sha256": product["binary_sha256"]},
                                  **FIXED["documents"]}
                      seen = set()
                      archive = PRIVATE / "download" / "archives" / product["archive"]
                      require(digest(archive) == product["archive_sha256"], "tar_archive_hash_mismatch")
                      with tarfile.open(archive, mode="r|gz") as source:
                          for member in source:
                              self.guard()
                              require(member.name in expected and member.name not in seen and member.isfile() and
                                      member.type in (tarfile.REGTYPE, tarfile.AREGTYPE) and
                                      not member.issparse() and not member.linkname and not member.pax_headers,
                                      "tar_exact_regular_members")
                              seen.add(member.name)
                              spec = expected[member.name]
                              require(member.size == spec["bytes"], "tar_member_bound")
                              # Documents are checked in each archive and never overwritten.
                              target = destination / (product["binary"] + "-" + member.name if member.name != product["binary"]
                                                       else product["binary"])
                              h = hashlib.sha256()
                              size = 0
                              with source.extractfile(member) as stream, os.fdopen(
                                      os.open(target, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), "wb") as out:
                                  while True:
                                      self.guard()
                                      block = stream.read(1024 ** 2)
                                      if not block:
                                          break
                                      size += len(block)
                                      require(size <= spec["bytes"], "tar_member_expansion_bound")
                                      h.update(block)
                                      out.write(block)
                              require(size == spec["bytes"] and h.hexdigest() == spec["sha256"], "tar_member_hash_mismatch")
                      require(seen == set(expected), "tar_three_member_required")
                      executable = destination / product["binary"]
                      elf(executable)
                      executable.chmod(0o700)
                  self.data["validated_products"] = FIXED["products"]
                  self.data["validated_license_hashes"] = FIXED["documents"]
                  self.save()
                  return installed

              def postgres_tools(self):
                  directory = pathlib.Path("/usr/lib/postgresql/16/bin")
                  require(directory.is_dir() and not directory.is_symlink(), "installed_postgres16_missing")
                  selected = {}
                  hashes = {}
                  versions = {}
                  for name in PG_NAMES:
                      path = directory / name
                      elf(path)
                      require(path.stat().st_uid == 0 and not (path.stat().st_mode & 0o022) and os.access(path, os.X_OK),
                              "postgres_tool_permissions")
                      selected[name] = str(path)
                      hashes[name] = digest(path)
                  env = environment()
                  env["PATH"] = str(directory) + ":" + env["PATH"]
                  for name in PG_NAMES:
                      output = self.command("postgres-version-" + name, [selected[name], "--version"], 10, env, capture=4096)
                      raw = output.read_text().strip()
                      match = re.fullmatch(r"(?:initdb|pg_ctl|createdb|psql|postgres) \(PostgreSQL\) (16(?:\.[0-9]+)+)(?: \(Ubuntu [A-Za-z0-9.+~:-]+\))?", raw)
                      require(match is not None, "postgres_version_refused")
                      versions[name] = {"version": match.group(1), "output_sha256": digest(output)}
                  require(len({item["version"] for item in versions.values()}) == 1, "postgres_versions_disagree")
                  self.tools = selected
                  self.data["postgres_tool_sha256"] = hashes
                  self.data["postgres_versions"] = versions
                  self.save()
                  return env

              def helper(self, installed, env):
                  self.guard()
                  require(self.deadline - time.monotonic() >= 660, "full_helper_budget_unavailable")
                  bootstrap = ROOT / "bootstrap.py"
                  with bootstrap.open("x") as out:
                      out.write(BOOTSTRAP)
                  bootstrap.chmod(0o600)
                  env["A09_ROOT"] = str(ROOT)
                  helper = pathlib.Path(os.environ["GITHUB_WORKSPACE"]) / "scripts" / HELPER
                  self.command("focused-shared-helper",
                               [sys.executable, "-I", "-B", str(bootstrap), str(helper), str(installed),
                                FIXED["imports"][HELPER], FIXED["product_sha"]], 600, env)
                  report = read_json(PRIVATE / "helper.json", 64 * 1024)
                  booleans = ("other_client_refused", "issuer_and_authentication_preserved", "all_effective_rates_preserved",
                              "active_capabilities_switched", "edition_and_version_metadata_coordinated")
                  literals = {"schema": "riauth.local-native-postgres-transition/v1", "release_gate_result": False,
                              "architecture": "linux/aarch64", "backend": "postgresql", "agreement_format": 3,
                              "source_revision": FIXED["product_sha"], "validator_sha256": FIXED["imports"][HELPER],
                              "shared_configuration_refusals": ["authentication", "general_rate"],
                              "shared_identity_authorization_sample": "passed", "full_shared_gate": "not_certified",
                              "editions": ["essentials", "platform", "essentials"]}
                  counts = ("baseline_rows", "upgrade_preserved_rows", "downgrade_preserved_rows")
                  expected_keys = set(literals) | set(booleans) | set(counts) | {"postgres_version", "binary_sha256", "shared_sample_sha256"}
                  require(set(report) == expected_keys and all(report.get(key) == value and type(report[key]) is type(value)
                                                             for key, value in literals.items()), "helper_report_literals")
                  require(all(report[key] is True for key in booleans) and
                          all(type(report[key]) is int and report[key] > 0 for key in counts) and
                          report["baseline_rows"] == report["upgrade_preserved_rows"] and
                          report["downgrade_preserved_rows"] >= report["baseline_rows"] and
                          isinstance(report["shared_sample_sha256"], str) and
                          re.fullmatch("[0-9a-f]{64}", report["shared_sample_sha256"]) is not None, "helper_report_assertions")
                  binary_hashes = {edition: {product["binary"]: product["binary_sha256"] for product in FIXED["products"]
                                            if product["edition"] == edition}
                                   for edition in ("essentials", "platform")}
                  require(report["binary_sha256"] == binary_hashes, "helper_binary_hashes")
                  require(type(report["postgres_version"]) is str and
                          report["postgres_version"] == (PRIVATE / "postgres-version-psql.stdout").read_text().strip(),
                          "helper_postgres_version")
                  sanitized = {key: report[key] for key in sorted(expected_keys - {"postgres_version"})}
                  sanitized["postgres_version"] = self.data["postgres_versions"]["psql"]["version"]
                  sanitized["raw_postgres_version_output_sha256"] = self.data["postgres_versions"]["psql"]["output_sha256"]
                  sanitized["helper_evidence_sha256"] = digest(PRIVATE / "helper.json")
                  atomic(PUBLIC / "helper-redacted.json", sanitized)
                  for name, expected in FIXED["imports"].items():
                      self.guard()
                      path = pathlib.Path(os.environ["GITHUB_WORKSPACE"]) / "scripts" / name
                      regular(path)
                      require(digest(path) == expected, "validator_import_changed_during_helper")
                  self.data["sample_result"] = "passed"
                  self.save()

              def fixture_proof(self):
                  fixture = PRIVATE / "local-a08-postgres-owned"
                  if not fixture.exists():
                      return None
                  require(not fixture.is_symlink() and fixture.resolve().parent == PRIVATE and
                          fixture.stat().st_uid == os.getuid() and stat.S_IMODE(fixture.stat().st_mode) == 0o700,
                          "fixture_ownership_path")
                  marker = read_json(fixture / ".a09-owner.json", 4096)
                  require(marker == {"nonce": self.owner["nonce"], "root": str(ROOT), "uid": os.getuid()},
                          "fixture_marker_mismatch")
                  cluster = fixture / "postgres-cluster"
                  if cluster.exists():
                      require(not cluster.is_symlink() and cluster.resolve().parent == fixture and
                              cluster.stat().st_uid == os.getuid(), "cluster_ownership_path")
                  return fixture

              def cleanup(self):
                  cleanup = {"schema": "riauth.local-build-free-cleanup/v1", "owned_fixture_removed": False,
                             "owned_processes_reaped": 0, "remaining_owned_processes": 0,
                             "postgres_pidfile_verified": False, "postgres_state": "not_observed",
                             "failures": self.cleanup_failures}
                  fixture = None
                  try:
                      fixture = self.fixture_proof()
                      descendants = owned_descendants()
                      cluster = None if fixture is None else fixture / "postgres-cluster"
                      pidfile = None if cluster is None else cluster / "postmaster.pid"
                      if pidfile is not None and pidfile.exists():
                          require(regular(pidfile).st_size <= 4096, "postgres_pidfile_bound")
                          lines = pidfile.read_text().splitlines()
                          require(len(lines) >= 3 and lines[0].isdigit() and lines[1] == str(cluster) and lines[2].isdigit(),
                                  "postgres_pidfile_invalid")
                          require(int(lines[2]) >= self.owner["started_utc_seconds"] - 2, "postgres_pidfile_not_fresh")
                          master = identity(int(lines[0]))
                          if master is not None:
                              matching = next((item for item in descendants if item["pid"] == master["pid"] and
                                               item["start"] == master["start"]), None)
                              require(matching is not None and same_process(matching), "postgres_not_owned_descendant")
                              executable = pathlib.Path("/proc") / str(master["pid"]) / "exe"
                              require("postgres" in self.tools and str(executable.resolve()) == self.tools["postgres"] and
                                      digest(executable) == self.data["postgres_tool_sha256"]["postgres"],
                                      "postgres_owned_executable_mismatch")
                              arguments = (pathlib.Path("/proc") / str(master["pid"]) / "cmdline").read_bytes().split(b"\0")
                              require(b"-D" in arguments and
                                      arguments[arguments.index(b"-D") + 1] == str(cluster).encode(),
                                      "postgres_owned_cluster_mismatch")
                              cleanup["postgres_state"] = "live_owned_master_verified"
                          else:
                              cleanup["postgres_state"] = "owned_pidfile_no_live_master"
                          cleanup["postgres_pidfile_verified"] = True
                      elif any(item for item in descendants if self.is_postgres(item)):
                          cleanup["postgres_state"] = "live_owned_process_without_pidfile"
                          self.cleanup_failures.append("live_owned_postgres_pidfile_unavailable")
                      else:
                          cleanup["postgres_state"] = "no_live_owned_postgres"
                  except Exception as error:
                      self.cleanup_failures.append(error.code if isinstance(error, Refusal) else "fixture_cleanup_proof_unavailable")
                  # Subreaper ancestry proves these are this controller's descendants, even after daemonization.
                  # PID fingerprints are rechecked per signal; never use a generic pg_ctl/killall or unrelated PID.
                  for sig, seconds in ((signal.SIGTERM, 5), (signal.SIGKILL, 5)):
                      end = time.monotonic() + seconds
                      while time.monotonic() < end:
                          descendants = owned_descendants()
                          if not descendants:
                              break
                          for item in descendants:
                              try:
                                  signal_owned(item, sig)
                              except ProcessLookupError:
                                  pass
                              except Exception:
                                  self.cleanup_failures.append("owned_process_signal_failed")
                          self.reaped += reap_owned()
                          time.sleep(0.1)
                  self.reaped += reap_owned()
                  remaining = owned_descendants()
                  reader_deadline = time.monotonic() + 3
                  for reader in self.capture_readers:
                      reader.join(timeout=max(0, reader_deadline - time.monotonic()))
                  if any(reader.is_alive() for reader in self.capture_readers):
                      self.cleanup_failures.append("private_capture_reader_remains")
                  else:
                      for row, paths in self.capture_records:
                          row["private_capture"] = {
                              channel: {"bytes": regular(path).st_size, "sha256": digest(path)}
                              for channel, path in zip(("stdout", "stderr"), paths) if path.exists()}
                  cleanup["owned_processes_reaped"] = self.reaped
                  cleanup["remaining_owned_processes"] = len(remaining)
                  if remaining:
                      self.cleanup_failures.append("owned_processes_remain")
                  # Failure retains the fixture privately. Only proven-owned, process-free scratch may be deleted.
                  if not self.cleanup_failures and fixture is not None:
                      shutil.rmtree(fixture)
                      cleanup["owned_fixture_removed"] = True
                  if not self.cleanup_failures and not remaining:
                      # Download, captures and installed binaries are owned scratch, never caches or source.
                      for path in PRIVATE.iterdir():
                          if path.name.endswith(".request") or path.name.endswith(".result"):
                              continue
                          if path.is_dir() and not path.is_symlink():
                              shutil.rmtree(path)
                          else:
                              path.unlink()
                      cleanup["owned_private_scratch_removed"] = True
                  else:
                      cleanup["owned_private_scratch_removed"] = False
                  atomic(PUBLIC / "cleanup.json", cleanup)
                  if self.cleanup_failures:
                      self.fail("cleanup", "owned_cleanup_failed")
                  self.save()

              def is_postgres(self, item):
                  try:
                      return same_process(item) and pathlib.Path("/proc", str(item["pid"]), "exe").resolve().name == "postgres"
                  except OSError:
                      return False

              def run(self):
                  phase = "preflight"
                  monitor = None
                  def interrupted(signum, frame):
                      self.interrupted = signum
                  for sig in (signal.SIGTERM, signal.SIGINT):
                      signal.signal(sig, interrupted)
                  try:
                      native()
                      require(re.fullmatch("[0-9a-f]{40}", os.environ["GITHUB_WORKFLOW_SHA"]) is not None and
                              re.fullmatch("[0-9a-f]{40}", os.environ["GITHUB_SHA"]) is not None, "github_source_context_invalid")
                      require(os.environ["GITHUB_REPOSITORY"] == FIXED["repository"] and
                              os.environ["GITHUB_EVENT_NAME"] == "workflow_dispatch", "manual_same_repository_required")
                      require(re.fullmatch("[0-9a-f]{40}", os.environ["A09_VALIDATOR_SOURCE_SHA"]) is not None,
                              "validator_full_sha_required")
                      # Linux libc is a runtime prerequisite. Adopt only our descendants so daemonized PG can be reaped.
                      libc = ctypes.CDLL(None, use_errno=True)
                      require(libc.prctl(36, 1, 0, 0, 0) == 0, "subreaper_unavailable")
                      actual = ctypes.c_int()
                      require(libc.prctl(37, ctypes.byref(actual), 0, 0, 0) == 0 and actual.value == 1,
                              "subreaper_not_verified")
                      require(shutil.rmtree.avoids_symlink_attacks, "owned_cleanup_filesystem_unsupported")
                      self.sample(initial=True)
                      monitor = threading.Thread(target=self.monitor, name="owned-resource-monitor", daemon=True)
                      monitor.start()
                      self.save()
                      write_new(PRIVATE / "init.result", {"nonce": self.owner["nonce"], "exit_code": 0})
                      phase = "wait_for_checkout"
                      while not (PRIVATE / "run.request").exists():
                          self.guard()
                          time.sleep(0.2)
                      require(read_json(PRIVATE / "run.request", 4096).get("nonce") == self.owner["nonce"], "run_marker_mismatch")
                      phase = "validator_source"
                      self.source()
                      phase = "artifact_transport"
                      self.download()
                      phase = "zip_validation"
                      self.extract_zip()
                      phase = "tar_validation"
                      installed = self.extract_binaries()
                      phase = "installed_postgres"
                      env = self.postgres_tools()
                      phase = "focused_shared_sample"
                      self.helper(installed, env)
                  except Exception as error:
                      code = error.code if isinstance(error, Refusal) else "controller_phase_unavailable"
                      self.fail(phase, code)
                  finally:
                      try:
                          self.cleanup()
                      except Exception as error:
                          self.cleanup_failures.append(error.code if isinstance(error, Refusal) else "cleanup_unavailable")
                          self.fail("cleanup", "cleanup_unavailable")
                          atomic(PUBLIC / "cleanup.json", {"schema": "riauth.local-build-free-cleanup/v1",
                                                         "failures": self.cleanup_failures, "completed": False})
                      try:
                          if self.last_sample is not None:
                              self.sample()
                              require(self.resource_failure is None, self.resource_failure or "resource_sample_unavailable")
                          require(self.interrupted is None, "controller_signal")
                      except Exception as error:
                          self.fail("resources", error.code if isinstance(error, Refusal) else "resource_final_sample_unavailable")
                      self.end_monitor.set()
                      if monitor is not None:
                          monitor.join(timeout=3)
                          if monitor.is_alive():
                              self.cleanup_failures.append("resource_monitor_not_stopped")
                              self.fail("cleanup", "resource_monitor_not_stopped")
                      self.data["controller_exit_code"] = 0 if self.first_failure is None else 1
                      self.save()
                      result = {"nonce": self.owner["nonce"], "exit_code": self.data["controller_exit_code"]}
                      for name in ("init", "run", "finalize"):
                          path = PRIVATE / (name + ".result")
                          if not path.exists():
                              write_new(path, result)
                      return result["exit_code"]


          def initialize():
              require(ROOT.name == "riauth-a09-shared-" + os.environ["GITHUB_RUN_ID"] + "-" +
                      os.environ["GITHUB_RUN_ATTEMPT"] and
                      ROOT.parent == pathlib.Path(os.environ["RUNNER_TEMP"]).resolve(), "initial_root_name")
              require(set(path.name for path in ROOT.iterdir()) == {"controller.py"}, "initial_root_not_fresh")
              PRIVATE.mkdir(mode=0o700)
              PUBLIC.mkdir(mode=0o700)
              for name in PUBLIC_NAMES:
                  if name.endswith(".jsonl"):
                      (PUBLIC / name).touch(mode=0o600)
                  else:
                      write_new(PUBLIC / name, {"schema": "riauth.local-build-free-pending/v1", "result": "not_run"})
              value = {"root": str(ROOT), "nonce": os.urandom(32).hex(), "uid": os.getuid(),
                       "run": int(os.environ["GITHUB_RUN_ID"]), "attempt": int(os.environ["GITHUB_RUN_ATTEMPT"]),
                       "started_monotonic": time.monotonic(), "started_utc_seconds": int(time.time())}
              # Private daemon descriptors are redirected; no token or failure text enters GHA console.
              with (PRIVATE / "supervisor.stdout").open("xb") as out, (PRIVATE / "supervisor.stderr").open("xb") as err:
                  child = subprocess.Popen([sys.executable, "-I", "-B", str(ROOT / "controller.py"), "supervise"],
                                           stdin=subprocess.DEVNULL, stdout=out, stderr=err, start_new_session=True)
              fingerprint = identity(child.pid)
              require(fingerprint is not None, "supervisor_start_lost")
              value.update(supervisor_pid=child.pid, supervisor_start=fingerprint["start"])
              write_new(OWNER, value)
              result = wait_result("init", 20)
              return result["exit_code"]


          def main():
              os.umask(0o077)
              mode = sys.argv[1]
              if mode == "init":
                  return initialize()
              if mode == "supervise":
                  # Parent creates marker immediately after Popen; bounded wait resolves that startup race.
                  end = time.monotonic() + 5
                  while not OWNER.exists() and time.monotonic() < end:
                      time.sleep(0.05)
                  return Controller().run()
              if mode == "transfer":
                  kind = sys.argv[2]
                  require(kind in ("metadata", "zip"), "transport_kind_invalid")
                  try:
                      transfer(kind)
                      return 0
                  except Exception as error:
                      code = error.code if isinstance(error, Refusal) else "transport_unavailable"
                      status = error.code if isinstance(error, urllib.error.HTTPError) else None
                      if status is None and (PRIVATE / (kind + "-http.json")).exists():
                          status = read_json(PRIVATE / (kind + "-http.json"), 4096)["status"]
                      write_new(PRIVATE / (kind + "-failure.json"), {"code": code, "http_status": status})
                      return 1
              if mode == "run":
                  request("run")
                  return wait_result("run", 1020)["exit_code"]
              if mode == "finalize":
                  if not (PRIVATE / "finalize.result").exists():
                      request("finalize")
                  result = wait_result("finalize", 40)
                  value = owner()
                  for name in PUBLIC_NAMES:
                      path = PUBLIC / name
                      require(regular(path).st_size <= 2 * 1024 ** 2 and path.stat().st_uid == os.getuid(),
                              "public_upload_file_invalid")
                  supervisor = identity(value["supervisor_pid"])
                  if supervisor is not None and supervisor["state"] != "Z":
                      end = time.monotonic() + 5
                      while identity(value["supervisor_pid"]) is not None and time.monotonic() < end:
                          time.sleep(0.1)
                      require(identity(value["supervisor_pid"]) is None or
                              identity(value["supervisor_pid"])["state"] == "Z", "supervisor_not_exited")
                  return result["exit_code"]
              raise Refusal("unknown_controller_mode")


          if __name__ == "__main__":
              try:
                  sys.exit(main())
              except Exception:
                  # Raw exceptions, arguments, paths and credentials never reach public upload or console.
                  sys.exit(1)
          PY
          python3 -I -B "$A09_ROOT/controller.py" init
      - name: Check out only the root-reviewed validator source
        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1
        timeout-minutes: 2
        with:
          ref: ${{ inputs.validator_source_sha }}
          persist-credentials: false
          fetch-depth: 1
      - name: Verify immutable transport and run one focused native shared sample
        timeout-minutes: 18
        run: |
          set -euo pipefail
          umask 077
          python3 -I -B "$A09_ROOT/controller.py" run
      - name: Finish owned cleanup and retain sanitized failure evidence
        if: ${{ always() }}
        timeout-minutes: 1
        run: |
          set -euo pipefail
          umask 077
          python3 -I -B "$A09_ROOT/controller.py" finalize
      - name: Upload only the four fixed sanitized evidence documents
        if: ${{ always() }}
        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02
        timeout-minutes: 1
        with:
          name: riauth-local-shared-arm64-${{ github.run_id }}-${{ github.run_attempt }}
          path: |
            ${{ env.A09_ROOT }}/public/controller.json
            ${{ env.A09_ROOT }}/public/resources.jsonl
            ${{ env.A09_ROOT }}/public/cleanup.json
            ${{ env.A09_ROOT }}/public/helper-redacted.json
          if-no-files-found: error
          compression-level: 0
          retention-days: 14
          include-hidden-files: false
```

### Actual source-only checks and limitations

Installed Psych parsed the complete YAML; Bash `-n` passed all three run blocks; Python AST parsed the controller and bootstrap without importing/executing either. Static policy/source checks passed: manual-only/read permissions, one native job/fixed concurrency/20-minute cap, action pins, actual context locations, exact four upload paths, transport/helper bounds, native/resource guards, retention-only adapter, verified subreaper source, no Cargo/apt/Docker executable literal, exact receipt and all 18 compressed/log/JSON bytes, all five protected helper/import hashes, unchanged old build workflow and byte-exact parent report. The new workflow path remains absent.

Authoring failures are preserved: one orchestration draft call failed JavaScript parsing (`SyntaxError: Unexpected token '}'`) before any nested tool/filesystem call; one later policy assertion exited 1 because a whole-text search for “cargo” also matched fixed **Cargo.lock/Cargo.toml input identities**. That assertion was narrowed to executable-literal AST inspection and passed; no repository checker was changed, and neither event was a workflow/helper/runtime result. Final source-only validation exit 0 replaces no historical result.

No actual GitHub query/dispatch/download, controller/adapter/helper import or execution, proposed ctypes/native-artifact/PostgreSQL execution or service, Cargo/build, archive opening/extraction, desktop, new worker/task/worktree/shell, other lane/main/push/status or contact occurred. Only this report is appended. The root-accepted build result remains five LOCAL ARM64 archives with native smoke exit 0; this design adds **no** new run/artifact/host/tool/cleanup/E→P→E credit.

A09 original supported-platform/artifact/shared-distribution gate remains open. `official_release: false`, prior `shared_full_gate: not_run`, proposed helper/controller `full_shared_gate: not_certified`, Linux x86-64/container/TLS/passkey/device/full-client/encrypted/full-shared/physical/tenant/escrow/paused-IO limits remain explicit. The accepted receipt-secret, route-specific header, PAM, permission/review/removal/audit/credential/Group/input/non-renewed-60-second protections remain unchanged. Root alone decides later source ownership, actual runtime/resources, review/integration/publication/status. RiWork Cua.ai Driver preference persists; no desktop operation is needed.

Final repository static evidence (2026-10-02): docs checker exit 0; staged whitespace/scope checks exit 0 with only this report and no deletions; tracked-file hygiene exit 0 (963 files). The complete embedded workflow/controller/adapter hashes above and all protected source bytes remain unchanged. These checks grant no runtime or artifact acceptance.

## Phase J — artifact run-head identity correction, report only (2026-10-02)

Reservation `wave30_A09_launcher_artifact_run_identity` owns only an append to this report for project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, existing A09 `506e3979-a590-4af3-8fa8-ee90d3a517f2` / WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. The entire immutable `47c28516df7508d4839a0705d55cd284d5c6a30a` report/proposal remains the dated prefix: **190,883 bytes / 2,651 lines**, SHA-256 `2c5eeaa8edf5b16156de6df3e70bf3e7dbae862dbfd50dced7fd87d90355bfc3`. Its original 1,279-line workflow fence is preserved, including the F1 defect and earlier static-check results. It is not replaced or silently corrected.

### F1 finding and factual source

I incorrectly compared artifact metadata `workflow_run.head_sha` to the separately checked-out product source. Root's exact read-only API observation supplied with this reservation reports artifact ID `11232871527`, name `riauth-local-arm64-37016520583-1`, run ID `37016520583`, **head SHA `036a392656b4b5070cc86a11d5ca3258b7b868d2`**, digest `sha256:fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30`, size **49,177,062 bytes**, and `expired: false`. These are root-supplied facts at the observation time, not a new worker API query, ZIP hash or current availability claim. Root retains `planning/evidence/wave30-a09-artifact-metadata-review.json`; no independent receipt-file hash is credited here.

The already downloaded exact evidence JSON was reread locally and its complete SHA-256 reconfirmed as `fd8280125b38e6afbe13970be4617df8fdbe72e7280a4e6dedb2f2a84e4c9ab1`. It independently contains `GITHUB_SHA == GITHUB_WORKFLOW_SHA == 036a392656b4b5070cc86a11d5ca3258b7b868d2`, while `source_sha == 9a819317efb3a13fa27cd86f884be2be00898fc0` and `source_tree == 1528b61ba463d9262d6252d54174748a176f313b`. Its input hashes and five product records also match the unchanged fixed constants. No archive was opened or executed.

Therefore the original line-501 metadata predicate would reject this correct artifact before ZIP download. Prior YAML/Bash/AST checks did not catch this semantic identity error; those checks were not native/transport acceptance. The sole proposed correction binds artifact run head to the **build workflow source**, preserving product identity verification through the exact digest-pinned build evidence/source tree/input/product/binary checks. It does not relax artifact identity, digest, expiration, size, run ID or any other condition.

### Exact zero-context virtual-file diff against the immutable 47c fence

This is a proposed patch to the fenced new-file text, not an applied workflow edit. All indentation, terminal newlines and other workflow/controller/bootstrap bytes are unchanged.

```diff
--- a/.github/workflows/check-local-shared-handoff.yml
+++ b/.github/workflows/check-local-shared-handoff.yml
@@ -501 +501 @@
-                          run.get("head_sha") == FIXED["product_sha"], "artifact_metadata_mismatch")
+                          run.get("head_sha") == FIXED["build_workflow_sha"], "artifact_metadata_mismatch")
```

| Complete proposed object | Original bytes / SHA-256 at 47c | Corrected bytes / SHA-256 |
| --- | --- | --- |
| Workflow, all 1,279 lines and terminal newline | 73,279 / `0bad3f22ac0c1534e64f12b5454efd686bac8bb260919f7d16753f968bb518b4` | **73,286** / `cf2365e64376a7a227a2d057995e81242f286d25a877a2bb0455b55c8c516f21` |
| Shell-created controller, all 1,203 lines and terminal newline | 59,371 / `8018cd2119e9763aa1832664d03ca28732739f2c4749d2b2a2cfcdb266e5a6b4` | **59,378** / `4acc7d31ea08ca1b5466b9bdc8c5f7b2d35699aea8488cb2703e3e2401c8c2b0` |
| Private bootstrap string | 1,506 / `1adc47c2b81f78c75f1f2122aabff7f08f48786fafd840880f3106418326e4de` | **Unchanged**, same complete hash |

The seven additional bytes are solely `product_sha` → `build_workflow_sha` in the unique `run.get("head_sha")` equality comparator. The full corrected workflow is defined by applying exactly this one replacement to the exact 47c fence; no second full-file variant or source patch is introduced.

### Actual source-only proofs

Actual exit-0 checks parsed both original/corrected YAML with installed Psych, ran Bash `-n` on all three run blocks in each, and parsed both complete controller/inline bootstrap ASTs. No controller/helper/bootstrap import or execution occurred. Whole corrected-workflow reversal equals all 73,279 original bytes; whole corrected-controller reversal equals all 59,371 original bytes. The bootstrap string remains byte-identical.

AST inspection finds exactly one `Compare` with left `run.get("head_sha")`, unchanged equality operator, and right `FIXED` subscript. The original key is `product_sha`; the corrected key is `build_workflow_sha`. Replacing only that corrected key back to `product_sha` in a copied AST yields complete normalized `ast.dump(..., include_attributes=False)` equality with the original controller. Reversing the one run-block predicate also yields equality of the entire parsed YAML document. This proves that permissions/manual trigger/action/context/resource/transport/extraction/capture/cleanup controls and all fixed data are untouched.

Source inspection reconfirmed the unchanged product-source/tree/input/product/archive/binary predicates and fixed hashes. All five protected helper/import files and the old build workflow remain byte-exact; the new `.github/workflows/check-local-shared-handoff.yml` still does not exist. No production/helper/workflow/source file is edited.

### Review boundaries and held work

This corrected proposal is ready for **root review first**. WTa130 is independently reviewing **original 47c**; this append claims no contact, receipt, agreement or endorsement from that reviewer. Root alone can decide whether to materialize the corrected new workflow and later reserve/dispatch transport/PostgreSQL/helper runtime after publication.

No remote query/dispatch/download, proposed library/native-artifact/helper/PostgreSQL/Cargo/build/service, archive extraction, desktop, new worker/task/worktree/shell, source/main/push/status or contact occurred. This append grants no new artifact, run, capacity, tool, cleanup, E→P→E or release credit. Previous HTTP 422/invalid-push/authoring failures and all original gates/limits remain intact. A09 stays open; `official_release: false`, prior `shared_full_gate: not_run` and proposed `full_shared_gate: not_certified` remain unchanged. Accepted security/permission/review/receipt/header/PAM/removal/audit/credential/Group/input/non-renewed-60-second protections and the RiWork Cua.ai Driver preference persist.

Final repository static checks for this correction: docs checker exit 0; staged whitespace/scope checks exit 0; tracked-file hygiene exit 0 (963 files). The sole delta is this report append, with zero deletions; no other file is edited. These checks do not grant materialization, transport or runtime acceptance.

## Phase K — invalid validator-input redaction correction, report only (2026-10-02)

Reservation `wave30_A09_launcher_invalid_validator_redaction` owns only this report append for project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, existing A09 `506e3979-a590-4af3-8fa8-ee90d3a517f2` / WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. The entire immutable `82e40339571d2ea6406d64590ccebd0616f48993` report is preserved as the historical prefix: **197,845 bytes / 2,699 lines**, SHA-256 `718bf8410065416ac8530162a4b14d75747a9c7632192caf333add5a01d5e4f5`. This includes the complete `47c28516df7508d4839a0705d55cd284d5c6a30a` dated proposal and Phase J/F1 patch/proofs. Neither historical prefix nor the F1 correction is rewritten.

### F2 finding and exact proposed boundary

I put the raw validator input into `Controller.__init__.self.data["validator_source_sha"]` before the existing `run()` syntax check. The constructor → refusal → finally cleanup/save → always upload chain could therefore serialize invalid input into the public controller JSON. Root identified this source defect; the pending independent review is not claimed as an endorsement. No workflow has been materialized or executed, and no actual invalid-input disclosure is reported.

The sole new proposed change replaces that public initializer value with exactly:

```python
os.environ["A09_VALIDATOR_SOURCE_SHA"] if re.fullmatch("[0-9a-f]{40}", os.environ["A09_VALIDATOR_SOURCE_SHA"]) is not None else None
```

A valid full lowercase SHA remains the same string. An invalid string becomes Python `None` / public JSON `null`. The initializer does not emit an invalid value, prefix, digest or derived error text. The existing raw full-input validation/refusal is retained unchanged before `init.result` and thus checkout, source-role checks and transport. This is redaction of one public field, not acceptance/coercion of invalid input. No additional import, fallback, validation mode or source-role relaxation is introduced.

### Exact zero-context diff against the 82e F1 variant

The starting virtual file is constructed **in memory only** from the immutable 47c complete workflow fence plus the exact Phase J line-501 F1 patch retained in 82e. Its complete workflow/controller hashes were checked before this new replacement. The following one-line patch changes only the public-data initializer at proposed workflow line **570**; the line-501 artifact head predicate stays `FIXED["build_workflow_sha"]`.

```diff
--- a/.github/workflows/check-local-shared-handoff.yml
+++ b/.github/workflows/check-local-shared-handoff.yml
@@ -570 +570 @@
-                               "validator_source_sha": os.environ["A09_VALIDATOR_SOURCE_SHA"],
+                               "validator_source_sha": os.environ["A09_VALIDATOR_SOURCE_SHA"] if re.fullmatch("[0-9a-f]{40}", os.environ["A09_VALIDATOR_SOURCE_SHA"]) is not None else None,
```

| Complete proposed object | F1 variant at 82e: bytes / SHA-256 | F1 + F2 corrected: bytes / SHA-256 |
| --- | --- | --- |
| Workflow, all 1,279 lines and terminal newline | 73,286 / `cf2365e64376a7a227a2d057995e81242f286d25a877a2bb0455b55c8c516f21` | **73,380** / `0de9c7be9b380bd32c748a19699739e8ed7dacb2313d1a366bb000f701e15037` |
| Shell-created controller, all 1,203 lines and terminal newline | 59,378 / `4acc7d31ea08ca1b5466b9bdc8c5f7b2d35699aea8488cb2703e3e2401c8c2b0` | **59,472** / `b84868ddce285e3c2979318b778bbb4d522ca0d212a1310c59d25c12bbd8f9e3` |
| Private bootstrap string | 1,506 / `1adc47c2b81f78c75f1f2122aabff7f08f48786fafd840880f3106418326e4de` | **Unchanged**, same complete hash |

The 94 additional bytes are solely the approved conditional expression on that existing line. All remaining workflow/controller/bootstrap bytes are identical to the F1 variant. The complete latest proposal is defined by the immutable 47c fence plus the retained exact F1 patch and this exact F2 patch; no real workflow/helper/source file is created or edited.

### Typed isolated-expression truth table

Only the approved initializer AST expression was compiled/evaluated in memory, with a fresh synthetic `os.environ` mapping and standard-library regex. No actual validator environment value was read, and no controller class/function/import/main/bootstrap/helper/native code was executed. Cases use the required string-input domain; invalid case values and their prefixes/hashes/error text are omitted from this report and check output.

| Synthetic case | Input type | Actual output type | JSON public field |
| --- | --- | --- | --- |
| valid lowercase full sha | `str` | `str` | Unchanged full lowercase SHA string |
| uppercase full sha | `str` | `NoneType` | `null` |
| short sha | `str` | `NoneType` | `null` |
| leading whitespace | `str` | `NoneType` | `null` |
| trailing whitespace | `str` | `NoneType` | `null` |
| whitespace only | `str` | `NoneType` | `null` |
| empty | `str` | `NoneType` | `null` |
| static invalid marker | `str` | `NoneType` | `null` |

All eight isolated cases passed exit-0 assertions. The valid case is byte-identical to its supplied synthetic 40-character lowercase input. Each invalid case produces exactly the one-field serialized document `{"validator_source_sha": null}`; no invalid-value-derived field is generated. These are expression-only checks, not an observed GHA invalid-input run, checkout/source-role approval or native/runtime acceptance.

### Actual whole-object proofs

Actual source-only checks passed: Psych parsed both F1 and F1+F2 YAML documents; Bash `-n` passed all three run blocks in each; both complete controller and unchanged bootstrap ASTs parsed. Reversing only the new initializer line restores every original F1 workflow byte (73,286) and controller byte (59,378). The retained F1 head-SHA comparison remains unchanged.

AST inspection located exactly one `Controller.__init__` assignment to `self.data`, containing exactly one `validator_source_sha` key. Its original value is the raw environment subscript. Its new value exactly matches the approved `IfExp`: original subscript for the true branch; regex `fullmatch` tested with `is not None`; literal `None` for the false branch. Replacing just that copied dictionary value with the original subscript yields complete normalized `ast.dump(..., include_attributes=False)` equality against the F1 controller. Reversing the initializer in the sole modified run block also restores equality of the entire parsed YAML document.

The unchanged `run()` full-input check was inspected in the complete AST source segment and remains before `init.result`, `self.source()` and `self.download()`. Protected helper/import hashes and the existing build workflow remain byte-exact. Permission/manual/action/context/resource/transport/extraction/capture/cleanup controls, valid-source data, product pins and format-3/authorization/configuration assertions are unchanged. The new workflow path remains absent.

### Root review and held execution

The proposal containing both corrections is ready for **root review first**. The independent review draft is not a reviewed immutable receipt; no contact, agreement or endorsement is inferred. Root will review the F1/F2 corrected proposal and independent report before any later workflow ownership/materialization reservation.

No actual workflow/helper/source edit, metadata query/download/dispatch, proposed library/native/PostgreSQL/helper/Cargo/service/runtime, archive execution/extraction, desktop, new worker/task/worktree/shell, main/push/status or worker contact occurred. This report adds no artifact/run/capacity/tool/cleanup/E→P→E/release credit. A09 remains open, with `official_release: false`, prior `shared_full_gate: not_run`, proposed `full_shared_gate: not_certified`, all accepted protection contracts and existing platform/shared-gate limits preserved. Materialization/transport/PostgreSQL/helper/runtime remain HELD. RiWork Cua.ai Driver preference persists.

Final repository static evidence for F2: docs checker exit 0; staged whitespace/scope checks exit 0; tracked-file hygiene exit 0 (963 files). Only this report append is staged, with zero deletions; both historical prefixes, the F1 variant, protected source files and absent-workflow boundary remain intact. These checks grant no workflow materialization or runtime acceptance.

## Phase L — exact corrected launcher materialization, source only (2026-10-02)

Reservation `wave30_A09_corrected_shared_launcher_materialization` owns only the **new** `.github/workflows/check-local-shared-handoff.yml` and an append to this existing report for project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, existing A09 `506e3979-a590-4af3-8fa8-ee90d3a517f2` / WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. The complete `ab993268b7d4ffefb42957228107d30f0ef9070d` report remains byte-exact: **206,172 bytes / 2,768 lines**, SHA-256 `748ce732c062527dc4f2cc5631211a689bd65238ce5c8651ac352f42c6668a54`. All original 47c/82e/ab993 proposal, failed authoring/HTTP-422/invalid-push and actual historical result phases remain intact.

### Immutable source-only result

| Materialized object | Exact pin |
| --- | --- |
| Source-only commit | `502d500931a8c300e785e3fe78c073d3655f21cb` |
| Source commit parent | `ab993268b7d4ffefb42957228107d30f0ef9070d` |
| Source tree | `e1473e2866ea9feb45682512fd916c4f5239ee90` |
| Sole new source path | [check-local-shared-handoff.yml](../../.github/workflows/check-local-shared-handoff.yml) |
| Git mode / complete source | **100644**, **73,380 bytes / 1,279 lines**, terminal newline preserved |
| Git blob | `a47187df6d2374310471138b07a930c481877655` |
| Whole workflow SHA-256 | `0de9c7be9b380bd32c748a19699739e8ed7dacb2313d1a366bb000f701e15037` |
| Extracted inline controller | **59,472 bytes**, SHA-256 `b84868ddce285e3c2979318b778bbb4d522ca0d212a1310c59d25c12bbd8f9e3` |
| Extracted bootstrap string | **1,506 bytes**, unchanged SHA-256 `1adc47c2b81f78c75f1f2122aabff7f08f48786fafd840880f3106418326e4de` |
| Source commit delta | One new workflow, **1,279 insertions / zero deletions**; report unchanged at source commit |

Materialization reconstructed the immutable 47c complete workflow fence, applied exactly the retained 82e **F1 line-501** hunk and ab993 **F2 line-570** hunk, and checked each intermediate/full SHA. Reversing each hunk restored the complete previous virtual file. The resulting file was created exclusively at the approved new path and compared byte for byte with the reviewed two-fix variant, both in the filesystem and the source commit Git blob. No third change, action/pin/trigger/context/threshold/runner/transport/cleanup/authority alteration, helper edit or alignment occurred.

F1 still binds artifact metadata run head to `FIXED["build_workflow_sha"]` (`036a392656b4b5070cc86a11d5ca3258b7b868d2`), preserving separate product `9a819317efb3a13fa27cd86f884be2be00898fc0`/tree/input/archive/binary verification. F2 still publishes the full validator SHA only when the exact lowercase-40 regex matches and otherwise publishes `None`; original raw-input refusal remains before checkout/source-role/transport. These are the two reviewed changes already recorded above, not new behavior added during materialization.

### Actual static checks

Source-only checks passed with exit 0 before the source commit:

- Exact fence/two-hunk reconstruction, every intermediate SHA, whole reverse equality, materialized length/line/mode/SHA and committed-blob equality.
- Installed **Psych** safe YAML parsing; **Bash -n** on exactly three run blocks; complete inline controller/bootstrap **Python AST** parsing. No workflow block, controller/constructor/import/main, bootstrap or helper was executed.
- Explicit manual-only input/trigger, `contents: read` + `actions: read`, one `ubuntu-24.04-arm` job/20-minute cap, fixed concurrency/no auto-cancel, exact checkout/upload action pins and five-step scope.
- Context placement: job env references the inputs context; token references the github context only in the initial step env; actual `RUNNER_TEMP` is read by that shell step; literal `printf` exports `A09_ROOT` through `GITHUB_ENV`; later upload paths use the env context. No job-level runner expression is present.
- Exact static private guards: 30-GiB initial/10-GiB stop/8-GiB floor, intended 2-second/maximum 6-second-gap monitor, bounded immutable artifact/ZIP/18-member/five-archive identities, private finite capture, original 600-second helper and 60-second cleanup reserve, retained owned fixture, verified subreaper source, PID ownership and fixed four sanitized upload paths. These are inspected source controls, **not** measured host capacity or exercised cleanup.
- F1 comparator/F2 public initializer AST equality, original full-input refusal order, all five protected helper/import byte hashes and the old build workflow hash unchanged.
- Docs checker exit 0; staged whitespace/scope checks exit 0; tracked-file hygiene exit 0 (**964 files**). Precommit scope was only the new workflow, no other tracked/untracked change; the complete report was still original ab993 bytes. The source commit then left a clean tree.

A validator-orchestration draft failed JavaScript parsing once with **`SyntaxError: Unexpected token '.'`**, before invoking any nested tool/check, because its static assertion strings contained unescaped GitHub expression interpolation. Only checker orchestration quoting was corrected; materialized workflow bytes remained exact and no proposed block was run. The subsequent static checks above passed. This was an authoring failure, not a GitHub/workflow/runtime test result; no repository checker was changed.

### Root review and held runtime

The new workflow now exists as immutable source; earlier dated “absent” phases remain historical and unchanged. This materialization grants **no** shared-gate, capacity, availability, PostgreSQL-tool, transport/ZIP, cleanup, native test or release credit. Prior root-observed build/native-smoke results stay attached to their original product/build/run/artifact pins, not this source-only commit.

The corrected-source independent review is active in WTa130. No contact, completed review, endorsement or receipt is inferred here. Root joins the immutable source and independent report before any separate integration/publication and runtime release. A future actual runner still must satisfy every declared native/source/tool/resource/transport/ownership condition; the root-selected published validator source remains distinct from product/workflow/run identities.

Actual dispatch/query/download/runner/PostgreSQL/native/helper/Cargo/build/services and all proposed workflow blocks remain **HELD** and were not executed. No other existing source/helper/workflow/guide, main/push/status/task/worker/worktree/terminal session, cache deletion or contact was changed/performed. A09 stays open; `official_release: false`, prior `shared_full_gate: not_run` and proposed `full_shared_gate: not_certified` persist with all platform/shared-sample and accepted permission/review/receipt/header/PAM/removal/audit/credential/Group/input/non-renewed-60-second protections. RiWork Cua.ai Driver preference persists; no desktop operation occurred.

Final report static evidence: docs checker exit 0; staged whitespace/scope checks exit 0; tracked-file hygiene exit 0 (964 files). This separate evidence delta contains only the report append with zero deletions; exact workflow/controller/bootstrap pins and the complete ab993 prefix remain unchanged. Materialization is source-only; independent source review/integration/publication and every runtime release remain root-owned and held.

## Phase M — first shared-run actual refusal and cleanup evidence (2026-10-02)

Reservation `wave30_A09_first_shared_run_actual_evidence` owns only this report append for project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09 `506e3979-a590-4af3-8fa8-ee90d3a517f2`, existing WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. The complete materialization-evidence parent `7ae335588f15070a6ddc4cfe41cae0569dfbf203` report is preserved byte for byte: **213,497 bytes / 2,815 lines**, SHA-256 `33648d9cbbf69cd29e4b09b21074300448cc1ed49053618c19af08782ba40314`. Every prior proposal/correction/source-only materialization, failure and historical artifact-result phase remains unchanged.

### Evidence read and availability boundary

The worker read all four supplied public documents in `/tmp/riauth-wave30-a09-shared-37042000805/riauth-local-shared-arm64-37042000805-1` and the complete `/tmp/riauth-wave30-a09-shared-37042000805/job.log` as local text only. No document script, controller, helper, archive or native binary was executed; no remote query/download occurred. Exact bytes were hashed locally:

| Supplied local document | Bytes | SHA-256 |
| --- | --- | --- |
| `controller.json` | 1,821 | `a8c3508f2654bf40a5d0aef71e778b1f2a89553cb6a7d115f3b948a1e0e0cdf5` |
| `cleanup.json` | 295 | `012ab6f866ccf6f7163430a0a8d8945383c1e044f025f186a51de67639a369db` |
| `helper-redacted.json` | 70 | `ea7793e91dfdf9fc5f1d08bacad8429368cd69cd2131927da2ccf74b330c8f08` |
| `resources.jsonl` | 877 | `23874319699a86a2c9d355654c985d6d65c63f972289730e6fb87b59d4a3a773` |
| `job.log`, all 1,452 lines | 239,523 | `94b97af5f0a7748e91882d33c209db146418f32acc6f2f09ced6064baa8fe5ef` |

The additional root-named `planning/evidence/wave30-a09-shared-source-role-failure-review.json` was **not available to this worker**: absent at this WT, the primary repository and exact same-repository registered-worktree paths, and absent as a tracked file at published `b619fe25269ccc150e473bbcde47cdb3623ef810`. An exact-basename local lookup produced no match; its `/tmp` search also reported permission denied for the unrelated `RustDesk-service` directory (exit 2). No private directory was opened or permission changed. No receipt content/hash/read is claimed. Root's supplied run/job/conclusion and dispatch-input attribution are identified as root-provided facts; the five actual local texts independently support the results below. The unavailability of that receipt is retained as a handoff limit, not substituted with a fabricated receipt or remote query.

### First actual run and immutable source roles

| Actual or expected role | Pin and evidence boundary |
| --- | --- |
| Actual first shared run / job / attempt | **37042000805 / 110954128114 / 1**; root supplies job identity and FAILURE conclusion, public controller supplies run/attempt; local job log records the two exit-1 steps |
| Actual repository / event | `Rhein-Industries/riAuth` / `workflow_dispatch` in public controller |
| Actual workflow, GitHub source context and selected validator input | All three equal **`b619fe25269ccc150e473bbcde47cdb3623ef810`**; controller records `workflow_sha`, `github_sha`, `validator_source_sha`; checkout log line 1351 and run environment line 1389 confirm b619 |
| Reviewed authored workflow source | `502d500931a8c300e785e3fe78c073d3655f21cb`; separate original source-only report `7ae335588f15070a6ddc4cfe41cae0569dfbf203` |
| Whole workflow SHA-256 | `0de9c7be9b380bd32c748a19699739e8ed7dacb2313d1a366bb000f701e15037` (73,380 bytes); complete local/published-b619/authored-502 source-byte equality checked without execution |
| Actual controller SHA-256 in public result | `b84868ddce285e3c2979318b778bbb4d522ca0d212a1310c59d25c12bbd8f9e3`, matching the reviewed inline controller |
| Expected validator helper SHA-256 | `575dfb049324ede3cb0ddf4bd71e0b42ef46704a015f6a576a38c4052a1ff2c1`; this first run refused before validator-head/import-hash phases and did not validate those runtime inputs |
| Expected product / prior build-workflow / prior build run and job | `9a819317efb3a13fa27cd86f884be2be00898fc0` / `036a392656b4b5070cc86a11d5ca3258b7b868d2` / 37016520583 and 110868629053; retained fixed controller expectations and historical provenance, not products fetched or tested in this first shared run |
| Expected prior build artifact / ZIP digest | 11232871527 / `riauth-local-arm64-37016520583-1` / `fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30`; **no transport or ZIP validation occurred in this run** |
| Action pins | Checkout `3d3c42e5aac5ba805825da76410c181273ba90b1`; upload `ea165f8d65b6e75b540449e92b4886f43607fa02`; public result and local log agree |

`controller.json` records first failure exactly as `{phase: validator_source, code: source_roles_must_be_distinct, exit_code: null}`, overall `controller_exit_code: 1`, `phases: []`, and `sample_result: not_run`. The failure is the existing source-role guard rejecting identical workflow/validator SHAs before even the `validator-head` command. Root identifies its selection of b619 for both roles as the dispatch-input mistake. This observation identifies **no workflow/controller/helper/product defect** and authorizes no source correction.

The complete source `run()` calls `source()` before `download()`, extraction, installed PostgreSQL selection and focused helper. With this exact first failure and empty command phases, artifact metadata/ZIP transport, archive/binary/license/input verification, PostgreSQL discovery/init/start and shared E→P→E/configuration/identity/authorization sample were **not run**. `helper-redacted.json` remains the exact pending document with `result: not_run`. Expected product/helper/archive pins appearing in a public result are fixed expectations, not newly validated artifacts or runtime coverage.

### Actual short resource sample

The job log identifies Ubuntu **24.04.5**, runner image `ubuntu-24.04-arm`, image version **20260927.135.1**, and runner **2.337.0**. The supervisor recorded **three** actual samples from `2026-10-02T17:37:53.472895+00:00` through `2026-10-02T17:37:55.496351+00:00`, covering approximately **2.023457 seconds** of sampled monotonic time. The minimum available bytes were **115,846,656,000** (approximately **107.890606 GiB**) for each of host root, private scratch and workspace. All three paths recorded the same device ID **2049**; these are three sampled locations on one observed device, not independent storage capacities.

The largest actual recorded interval was **2.0004846349999994 seconds** (rounded **2.000485**). JSONL-derived count/minima/max-gap agree exactly with `controller.json`; adjacent monotonic differences agree with the recorded gaps. Samples stayed above the unchanged 30-GiB start, 10-GiB stop and 8-GiB floor, within the unchanged 6-second maximum-gap policy. This is measured capacity for the brief refused run only. It does not establish storage sufficiency for the unrun artifact/PG/helper phases, a later host or any sustained gate; no threshold/provider substitution or capacity claim from a runner label is made.

### Cleanup and failure-preserving finalize

`cleanup.json` records `failures: []`, `owned_private_scratch_removed: true`, `remaining_owned_processes: 0`, `owned_processes_reaped: 0`, `owned_fixture_removed: false`, `postgres_pidfile_verified: false`, and `postgres_state: no_live_owned_postgres`. The controller's separate `cleanup_failures` is also the empty list. No fixture or PostgreSQL/helper was created, so the false fixture-removal/PIDfile-verification fields do **not** indicate a cleanup failure and do **not** prove exercised fixture ownership, live-PG termination, timeout/descendant handling, or PID/marker protection. Only the actual reported private-scratch removal and zero remaining processes are credited, with no invented ownership proof.

Local `job.log` line **1392** records the controller run step exit **1**; line **1402** records finalize exit **1**. The unchanged controller's finally block sets its result to 1 whenever `first_failure` exists and writes that result for run/finalize; `finalize` returns that same original result. Here finalize's exit 1 preserves `validator_source/source_roles_must_be_distinct`; it is **not a second cleanup failure**. Actual cleanup-failure evidence remains `[]` in both public documents.

The always-upload step nevertheless retained exactly four public documents. Local log lines **1424**, **1431**, **1434** report four files, upload-reported ZIP SHA-256 `19122b29ce7fc46aa65b95885dd5bd38e03cb8c0dfbfe56f6fc94452f916899d`, and evidence artifact **11242688193**, name **`riauth-local-shared-arm64-37042000805-1`**, size **3,577 bytes**. This digest/size is upload-log provenance; the worker did not retain/re-hash an outer ZIP, query artifact metadata or download anything. This sanitized failure-evidence upload is separate from prior binary artifact 11232871527 and grants no release/shared-sample success.

### Remaining scope and later root-owned run

Root reports a separate dispatch of the **unchanged** workflow using published accepted validator `0a243c9` (same tree, distinct SHA), run **37043196924**. That fact is supplied context only: this worker did not query, download, execute or inspect that run and claims no outcome, capacity, cleanup, native test or shared-gate result for it. Its complete validator pin, runtime evidence and later decision remain root-owned.

A09 stays **open**. This first run's actual fields remain `official_release: false`, `full_shared_gate: not_certified`, `sample_result: not_run`; earlier `shared_full_gate: not_run` and Linux x86/container/TLS/device/full shared/platform limits remain attached to their dated evidence. No source/workflow/helper/product correction or implementation occurred in this reservation; no artifacts were produced, fetched or executed by this worker. All prior accepted permission/review/receipt/header/PAM/removal/audit/credential/Group/input/non-renewed-60-second protections remain intact. RiWork Cua.ai Driver preference persists; no desktop operation occurred.

Actual report verification: docs checker exit 0; staged tracked-file hygiene exit 0 (964 files); whitespace and append-only scope/prefix proofs exit 0. Published-b619/authored-502/local workflow byte equality, inline controller hash and all six protected helper/import/build-workflow hashes passed; none was edited or executed. The reserved delta contains only this report append, zero deletions and no untracked files. The unavailable root planning receipt and uninspected later run remain explicit limits; this evidence is ready for root review/integration and grants no shared-gate closure.
