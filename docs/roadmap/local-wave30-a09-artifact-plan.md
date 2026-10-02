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
