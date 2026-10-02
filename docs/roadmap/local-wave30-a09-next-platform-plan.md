# A09 next supported-platform slice: source-only proposal

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original A09 task
`506e3979-a590-4af3-8fa8-ee90d3a517f2`. This independent supporting audit uses
existing worktree `42bb51c6-c198-4adb-bd92-0a5222853231` and reservation
`wave30_A09_next_platform_source_audit`. Primary worktree
`f2e8500e-2e56-47e3-b60e-9f81bbc8cff2` remains assigned. Root owns future claims,
runtime reservations, integration and status. This report is the only write;
the workflow proposal below has not been materialized or executed.

## Original row and fixed source

The project's local `planning/current-tasks.json` record was reread by exact UUID
on 2026-10-02; the export's modification time was
`2026-10-02T14:13:59.018971+00:00`. It records A09 `in_progress` on its primary
worktree. This is the retained current-row export, not a fresh board/API query.
The original requested outcome is:

> Produce and test server, client, container, and maintenance artifacts for explicitly supported platforms, including Linux x86-64 and ARM64.

The workstream goal is “Make riAuth modular without creating two identity
implementations.” Its completion gate is:

> Switching distributions does not create different users, different authorization rules, or silently ignored configuration.

The row also requires shared identity, authorization, revocation and credential
protection in both builds; a separate remote client with optional terminal USB;
review of implementation, tests, documentation and released artifacts as
applicable; and actual evidence rather than closure from a worker report.
Listed prerequisites are A04/A05/A08/Q08. Its old scheduling sentence does not
authorize a launch; the current user authorizes only this supporting audit.
No reassignment or status mutation was performed.

All proposed source changes refer to fixed published
`b5dcfa9dbb14e953d12edac6a6a1133ecb61033e`, read with `git show`, not the older
supporting branch's checked-out production files. Own starting HEAD was
`8dbbc74d8c325309b42c5011fb5d36a512fd6719`, clean; no merge/reset was performed.

## Reviewed bodies and evidence limits

Complete body reviews at the fixed pin covered these paths:

| Source | Concrete responsibility and result of review |
| --- | --- |
| `.github/workflows/check-local-artifacts.yml` | Manual ARM-only job; exact source checkout, three sequential locked native builds, five local archives, focused checker, bounded owned child groups and evidence upload. Every architecture-specific literal is identified below. |
| `.github/workflows/release.yml` | Tag-only native `ubuntu-24.04` x86-64 and `ubuntu-24.04-arm` ARM64 matrix, five binaries, two containers, packaging, archive/image checks, installed drill and both-architecture bundle publication. Source declaration is not execution evidence. |
| `Dockerfile` | Digest-pinned Rust 1.98.1/Debian bases; edition-selected server build, revision/edition labels, runtime dependencies, non-root UID/GID 10001 and `/data`. No maintenance or client executable is embedded. No smaller necessary container source correction was established by this review. |
| `scripts/package-release.sh` | Native host allowlist; real commit/repository/run/attempt and two images required; five binary archives, two image archives, SPDX documents, v4 provenance and checksums per architecture. Local binaries cannot satisfy those inputs by invented release identity. |
| `scripts/check-edition-artifacts.py` | Five archive inputs are supported without image arguments. Extracted maintenance/base-client version checks; actual edition route exposure; Essentials invalid agent-scope refusals; Platform agent state; direct and maintenance-preflight downgrade refusals. Revision/If-Match/idempotency headers remain in its agent writer. |
| `scripts/check-release-bundle.py` | Exact complete two-architecture asset set, checksum/provenance/features/triple/OCI agreement, maintenance member hashes and common revision/toolchain/locks. It imports SPDX validation; that imported module was not body-reviewed here. |
| `scripts/check-installed-release-gate.py` | Requires one complete native release cohort before installation; capability/version checks and redb backup/explicit edition handoff/recovery drill. Its assertions do not cover every ordinary-user authorization/configuration invariant in the original shared gate. It is outside the proposed five-local-archive execution. |
| `Cargo.toml`, `crates/riauthctl/Cargo.toml`, `rust-toolchain.toml` | Platform includes Essentials; release editions explicitly disable defaults. Standalone client defaults are empty and USB is optional. Toolchain is 1.98.1; release thin LTO/strip remain unchanged. Lockfiles were hashed, not dependency bodies reviewed. |

Root workflow/actual-ARM reports and the complete retained JSON receipt were
read as historical evidence. Only the initial artifact inventory of the much
larger primary A09 report was reviewed; this audit does not claim its full body
review. The build-free PostgreSQL shared launcher, its helper implementation
and its proposed workflow are excluded; their Sol3/Sol5 ownership is unchanged.
No other worker was contacted. CONTRIBUTING/SECURITY guidance and the supported
Linux target section of `docs/roadmap/product-contracts.md` were read; no local
AGENTS.md was found. No repository checker, artifact or helper was executed.

Selected immutable byte checks (SHA256):

| Fixed-pin body | SHA256 |
| --- | --- |
| Manual workflow | `0a98865e5ee25d2f94b745b32602949553bf4f4e3b83d8bd302fade98eafd948` |
| Release workflow | `da8607c5ea33a906185141e0caebec18c2282c22676981e0c659fd65eef4369f` |
| Dockerfile | `458ebb247170c6d4af2ff45b22e5a36e0a5380612140a43448d2ddcebc5d83cb` |
| Packager | `3060b16559b004e6e213d7ed484f575a7375df3d92d29b39fcf0726f936a2f0a` |
| Focused archive checker | `2f477665ed584bc148fb230c0658f6d0cc66aceaa5af631c4dc467872fa9ecce` |
| Bundle checker | `3272d70201eb8eb91039e90f87a69b694d6ee7fd95cd5f1f4d70b990c6d7a001` |
| Installed release gate | `cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5` |
| ARM receipt JSON | `2cbf8ee46dbdd8b2ab43ad933913dd0a16c20b3183497c2d3b9cf3d1287da227` |

The manual workflow is byte-identical to the executed definition at
`036a392656b4b5070cc86a11d5ca3258b7b868d2`. The receipt's seven input hashes
(server/client manifests and locks, LICENSE, notices, focused checker) also
match fixed source. Neither comparison proves old product binaries equal
current source; production bodies changed between product pins.

## Actual ARM receipt: credit only its measured slice

Root's retained receipt is
`docs/roadmap/evidence/wave30-a09-native-arm64-37016520583.json` at the fixed pin.
It records successful run `37016520583`, job `110868629053`, workflow definition
`036a392656b4b5070cc86a11d5ca3258b7b868d2`, product source
`9a819317efb3a13fa27cd86f884be2be00898fc0`, and product tree
`1528b61ba463d9262d6252d54174748a176f313b`.

The actual uploaded artifact is ID `11232871527`,
`riauth-local-arm64-37016520583-1`. Digest
`sha256:fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30`
is API-reported; root explicitly did not locally rehash its outer ZIP. Root's
report credits rehashes of the five archives/members, logs and input comparisons,
plus native ELF headers. This supporting audit rehashed only immutable Git
bodies/receipt JSON and compared their recorded values; it did not consume
external archives/logs/binaries or independently repeat those root checks.

All 11 recorded command steps exited zero. Essentials release build took
1004.627 s, Platform 1312.673 s, base client 204.104 s; focused five-archive
checker took 4.008 s. Observed runner was native Linux/aarch64, ARM64,
GitHub-hosted, four CPUs and 16,722,042,880 bytes RAM, with Rust 1.98.1.
Minimum host/workspace/private free space was 112,151,941,120 bytes
(104.45 GiB); 1,308 resource samples had a maximum interval of 5.40448 s.
The receipt does not measure a guaranteed x86 start margin or peak transient
allocation. Cleanup errors are empty, not independent post-VM group inventory.

The original HTTP422 dispatch refusal and invalid-workflow push records
`37014464740`/`37014464605` allocated no native build job. The corrected workflow
computes `$RUNNER_TEMP` inside the shell rather than the rejected job-level
context. That correction must remain. Historical failures remain in the
published reports; this audit performs no retry. Successful ARM evidence says
`official_release=false`, `shared_full_gate=not_run`. It establishes neither
current `b5dcfa9d` products nor x86, container or full shared-gate execution.

## ONE proposed next source claim

Reserve only `.github/workflows/check-local-artifacts.yml` for a later bounded
change: allow one manual native Linux x86-64 selection while retaining ARM64
as the default. Reuse the existing driver and unchanged focused archive checker.
No new workflow, helper, product code, primary report or shared launcher change
is necessary for this slice. Human workflow/job names may become
`Check local Linux artifacts` / `native-linux`; one job, no matrix or automatic
trigger. Preserve the existing concurrency key `riauth-local-artifacts-arm64`
as the serialization key, including for x86, and `cancel-in-progress: false`.
Its legacy name is not architecture evidence.

Exact proposed header additions/replacements, after root reserves source:

```yaml
# Add beside source_sha, keeping that required full-SHA input unchanged.
architecture:
  description: Native Linux architecture for five LOCAL artifacts
  required: true
  type: choice
  default: arm64
  options: [arm64, x86_64]

# Replace the existing job's runs-on; add to its env, not shell interpolation.
runs-on: ${{ inputs.architecture == 'x86_64' && 'ubuntu-24.04' || 'ubuntu-24.04-arm' }}
ARCHITECTURE: ${{ inputs.architecture }}
```

These are identified hunks, not a complete runnable YAML file. The inline Python
must select only the following closed map before constructing evidence, and
reject missing/unknown choices with `unsupported_native_architecture`:

| Input | Requested runner label | Required actual machine / RUNNER_ARCH | Triple | ELF e_machine | Capability/archive architecture |
| --- | --- | --- | --- | --- | --- |
| `arm64` (default) | `ubuntu-24.04-arm` | `aarch64` / `ARM64` | `aarch64-unknown-linux-gnu` | 183 | `aarch64` |
| `x86_64` | `ubuntu-24.04` | `x86_64` / `X64` | `x86_64-unknown-linux-gnu` | 62 | `x86_64` |

`X64` is the proposed fail-closed expected runner token, not an observed token
from a new allocation. Never accept caller-supplied runner labels or triples.
Map fields should be `runner`, `machine`, `runner_arch`, `target`, `elf_machine`.
Only these existing spans need substitutions (line numbers at the fixed pin):

| Span | Exact prospective replacement/constraint |
| --- | --- |
| 1, 5–9, 19–26 | Names, closed choice input, selected runner expression and `ARCHITECTURE` env above. Permissions remain `contents: read`. |
| 45–66 | Closed map selection before data initialization; derive `runner.label`; add selected architecture/triple/e_machine evidence. Keep observed system/machine/runner token/CPU/RAM and actual GitHub fields. |
| 181–193 | Require Linux, selected exact machine/runner token and `github-hosted`. Keep full product SHA, manual branch dispatch, real workflow SHA and positive real run/attempt guards. Refuse before checkout/setup/build on mismatch. |
| 218 | `target = native['target']`; existing three build argv/limits/environment remain unchanged apart from target value. |
| 240–241 | Keep ELF64 little-endian prefix guard; compare `int.from_bytes(header[18:20], 'little') == native['elf_machine']`. ARM value 183 preserves the original two-byte `b'\xb7\x00'` comparison. |
| 252, 254, 279 | Capability target uses selected `machine`; archive and checker input suffix use the same closed-map `machine`, never requested arbitrary text. |
| 322 | `riauth-local-${{ inputs.architecture }}-${{ github.run_id }}-${{ github.run_attempt }}`. Default ARM name remains identical; current-run identity stays actual. |

Keep source-check/hash reconstruction, action pins, checkout with
`persist-credentials: false`, private paths/modes, command/cleanup implementation,
archive members/modes/timestamps, five-product list, checker flags, and
before/after source/product comparisons intact. Additive provenance must not
rewrite historical receipts. No changes to threshold/config/cache policy.
The guard/controller hunks and default ARM expansion need independent source
review before root publishes; none of this proposed YAML/Python was parsed or
tested in this report-only phase.

## Exact future execution and evidence boundary

After source review/publication and a separate serialized runtime release,
root can select `architecture=x86_64` with the reviewed full product SHA
`b5dcfa9dbb14e953d12edac6a6a1133ecb61033e`. If root chooses later product source,
re-pin and review its manifests/checker rather than silently substituting it.
The future workflow-definition SHA, run/job/artifact IDs and digests do not
exist in this audit and must be recorded from that actual run, independently
of checked-out product SHA/tree. No tag or official release is needed to
dispatch this LOCAL validation.

The existing phases remain `driver.py init`, `verify`, `dependencies`, `build`.
Under private `$A09_ROOT/cargo-home`, jobs=1, incremental=0, dev/test/release
debug=0 and the unchanged server package's release thin LTO/strip profile (the
standalone client keeps its own profile), the x86 build argv expands to:

```bash
CARGO_TARGET_DIR="$A09_ROOT/target/essentials" cargo +1.98.1 build --release --locked --no-default-features --features essentials --bins --target x86_64-unknown-linux-gnu
CARGO_TARGET_DIR="$A09_ROOT/target/platform" cargo +1.98.1 build --release --locked --no-default-features --features platform --bins --target x86_64-unknown-linux-gnu
CARGO_TARGET_DIR="$A09_ROOT/target/client" cargo +1.98.1 build --manifest-path crates/riauthctl/Cargo.toml --release --locked --no-default-features --target x86_64-unknown-linux-gnu
```

These are future driver argv descriptions, not commands executed here. No
test-support/fuzzing/default server feature or terminal USB is added. Native
dependency setup remains bounded `apt-get update`, then `apt-get install -y
--no-install-recommends ca-certificates cmake libssl-dev clang pkg-config` and
`dpkg-query -W` for those packages. Record actual Rust and `/etc/os-release`,
dependency versions, all input hashes, source/tree and features. Locked Cargo
does not pin the hosted OS or apt package resolution. The proposed build
baseline is Ubuntu 24.04; the GNU triple does not establish compatibility with
every Linux distribution or libc version.

Five archives contain only the matching executable, LICENSE and notices, with
unchanged deterministic metadata. Require native ELF machine 62 for all five,
artifact-scope `linux/x86_64` capabilities and correct server edition/features.
Use exactly the existing five-path smoke, with no Docker arguments:

```bash
python3 scripts/check-edition-artifacts.py \
  --essentials-archive "$A09_ROOT/evidence/archives/local-riauth-essentials-x86_64.tar.gz" \
  --platform-archive "$A09_ROOT/evidence/archives/local-riauth-platform-x86_64.tar.gz" \
  --essentials-maintenance-archive "$A09_ROOT/evidence/archives/local-riauth-maintenance-essentials-x86_64.tar.gz" \
  --platform-maintenance-archive "$A09_ROOT/evidence/archives/local-riauth-maintenance-platform-x86_64.tar.gz" \
  --riauthctl-archive "$A09_ROOT/evidence/archives/local-riauthctl-x86_64.tar.gz"
```

Hash each binary/archive before and after the smoke; recheck exact clean source
and input hashes. Upload complete local evidence on success/refusal/failure with
the pinned action, compression=0, retention=14 days, using real run/attempt.
Root should independently verify actual upload metadata, archive/member/header
and log hashes. Do not describe an API-provided outer ZIP digest as locally
rehashed without retaining/rehashing those exact ZIP bytes.

## Resource/security prerequisites and residual outcome

The next source slice is local executable work; the execution prerequisite is
a separately approved, actually allocated native GitHub-hosted x86 runner.
This audit did not query runner availability, limits, current free disk or
network. The historical ARM allocation's 104.45 GiB minimum and 42-minute total
build time are measured there only, not a credible x86 capacity guarantee.
No x86 transient peak, duration, native dependency versions or successful
products are currently measured. Required future controls remain:

- At least 30 GiB measured free on host/workspace/private filesystems at launch,
  source verification, dependency setup and pre-build; below that, refuse. The
  runner label alone does not establish the starting margin. No automatic
  paid/self-hosted fallback, threshold reduction or cache deletion.
- Three sequential private targets and one private Cargo home; no restore of
  unrelated caches or concurrent Cargo. Existing 150-minute job bound,
  1800 s per build, 240/600 s apt bounds, 30 s metadata/capability bounds and
  180 s focused smoke. Network/toolchain/dependency availability remain unmeasured
  prerequisites to be authorized with that runtime, not fulfilled by this audit.
- Existing nominal two-second free-space/log/deadline sampling, owned-group
  stop below 10 GiB retaining the 8 GiB floor margin, TERM/KILL/wait cleanup,
  private 0700 directories and 0600 evidence/logs with sampled 8 MiB command
  caps. Sampling can lag; it is not a hard quota. Actions outside owned groups
  and runner loss can prevent cleanup/upload. Do not claim independent empty
  group inventory from an empty cleanup-error list.
- Smoke uses disposable loopback fixtures only. Preserve the unchanged checker
  writer's permission/revision/If-Match/idempotency behavior and refusal checks.
  No real tenant/cloud/registry mutation, persistent secret or external operator
  fixture; no weakening of accepted receipt-secret, route-specific headers,
  PAM fallback, Group hold, credential/audit or 60-second boundaries.

If successful, the slice supplies current-pin native Linux x86-64 Essentials
server/maintenance, Platform server/maintenance and base-client LOCAL archive
evidence. Container execution is still a concrete A09 artifact-family gap
relative to this receipt; complete official packaged release cohorts and the
original shared-distribution gate need their own actual evidence as applicable.
The existing checker does not establish ordinary-user/grant/revocation/config
equality or current format3 PostgreSQL behavior. Historical A08 evidence stays
historical; no format2 helper is promoted to a current format3 pass. The shared
gate's other workers own their work independently; this proposal neither
implements nor assesses their launcher. No whole-row completion recommendation
follows from this report or a prospective x86 smoke.

## Checks actually performed

Read-only `pwd`/Git status/pin existence, immutable `git show` body reviews and
local exact-UUID task-export read; Python standard-library SHA256/JSON and byte
comparisons over Git output. `git diff --cached --check` exited zero; staged
scope is only this new report. Report whitespace/fence/link review and existence of all 11
cited fixed-pin source/receipt paths passed. All seven receipt input hashes
matched and all 11 recorded step exits were zero. The fixed source's server
`src/bin` contains only `riauth-maintenance.rs`; the separate client has no
additional `src/bin` targets. These are tree/receipt reads, not fresh builds or
test passes. No workflow parser, repository checker, tool-version command,
product/helper execution, Cargo/test/Docker/service, query/download/dispatch,
network/browser/desktop or other-worker contact. RiWork Cua.ai Driver remains
the only permitted desktop provider if a future separately authorized task
needs desktop use. Root alone may reserve the proposed workflow slice/runtime
and reconcile A09 status.

## Approved closed architecture implementation: source/static evidence

Reservation `wave30_A09_closed_native_architecture_workflow`, same project,
original task and supporting/primary worktrees. Root approved the above plan
for source implementation only. Source commit
`3c211369856b1e4c54b2ba8d39404635ccfddaea` changes only
`.github/workflows/check-local-artifacts.yml` (37 additions, 14 deletions).
This appendix is a separate report-only commit. The original `85b06e3` report
remains an exact 20,136-byte prefix, SHA256
`cfc3522ada0fcadc15fe180a36d903e2ee6663f999195aed4afcc7ec9e140a5b`;
its proposal-phase checks/limits are historical, not overwritten by this phase.

### Immutable base and exact scope

Actual fixed source `b619fe25269ccc150e473bbcde47cdb3623ef810` resolved locally.
Its complete manual workflow is byte-identical to published `b5dcfa9d`, own
`85b06e3` HEAD and the clean working file: 18,186 bytes, SHA256
`0a98865e5ee25d2f94b745b32602949553bf4f4e3b83d8bd302fade98eafd948`.
No alignment/merge or whole-file replacement was needed. `b619` contains no
AGENTS.md. Server/client manifests and locks, Rust toolchain and focused checker
also equal `b5` byte-for-byte, with the hashes already reported above. This
comparison is current source evidence, not a build of either product pin.

The committed workflow is 19,487 bytes, SHA256
`62c378975021066d62d5b32d0eee2c87de8501d01ded1ae0fae8147e920e58d7`.
Its full diff consists only of the approved header/name, closed-map, additive
selected-platform metadata, native-host, target/ELF/capability/archive and upload
name substitutions. The source commit diff is the exact reviewable change;
no shared launcher, checker, helper, product, guide or primary A09 report changed.
All other tracked own files remained unchanged. No main/accepted edits,
history rewrite, status mutation or other-worker contact occurred.

### Implemented selection and preservation

One manual job is now named `native-linux`; the workflow is `Check local Linux
artifacts`. Required choice `architecture` defaults to `arm64` and admits only
`arm64`/`x86_64`. The job expression selects `ubuntu-24.04-arm` or
`ubuntu-24.04`; the value reaches Python through `ARCHITECTURE`, not shell
interpolation. The exact closed map in the earlier table supplies runner label,
machine, runner token, triple and ELF value. Missing or unknown input raises
`unsupported_native_architecture` before root/data initialization or any driver
guard/command. That refusal does not create driver JSON; do not infer a receipt
upload for an invalid selector. The legitimate host guard still requires Linux,
the exact selected machine/token and `github-hosted` before checkout/setup/build.
Its ARM failure label stays `not_native_hosted_arm64`.

All existing provenance fields and assertions remain; additive
`selected_platform` records requested architecture, derived triple and ELF
machine. Actual runner system/machine/token/CPU/RAM and GitHub source/workflow/run
fields remain observed, not fabricated. Both feature-selected servers and both
maintenance tools use the chosen triple; the standalone base client keeps empty
default features. All five archive names and checker paths use the same map's
machine. Default ARM target, capability metadata, product/archive names, upload
name and error labels match the original, aside from intentional human/job names
and additive selection metadata.

One source-equivalence detail differs from the proposal's integer-decoding
example: the implementation compares
`header[18:20] == native['elf_machine'].to_bytes(2, 'little')`. This preserves the
original exact two-byte check. Decoding a truncated one-byte field could instead
accept `b'\xb7'` as 183; it must remain rejected. The closed values encode exactly
ARM `b'\xb7\x00'` and x86-64 `b'\x3e\x00'`. ELF64/little-endian prefix matching
is unchanged. This is preservation within the claimed ELF hunk, not a new
validation or helper scope.

### Static checks actually executed

- `ruby -rpsych -rjson -e 'puts JSON.generate(Psych.load(STDIN.read))'` parsed
  both immutable original and changed YAML through stdin, exit zero. Parsed
  workflow records were used for the static comparisons below. No dispatch or
  hosted workflow validation was performed.
- `bash -n` through stdin passed each of the **four** `run` blocks in the seven
  YAML steps; none was executed. An initial success message mistakenly printed
  “five”; a counted recheck asserted four and passed. This was a reporting-label
  error, not a hidden fifth command or failed shell block.
- Python `ast.parse` accepted original/changed inline driver. No driver bytecode
  or module execution occurred. Literal-map/AST checks established the
  two exact mappings, early missing/unknown-input refusal, and encoded ELF
  constants including the short-field rejection.
- Complete source segments of all eight functions are byte-identical:
  `interrupted`, `save`, `require`, `sha`, `capacity`, `stop_owned`, `command`,
  `source_check`. Thus command argv/deadlines/logging, owned cleanup, source/tree
  and seven-input hashing, sampling and threshold bodies remain intact.
- Full default-ARM driver AST normalization replaced only closed-map references
  with verified ARM literals, folded the resulting string/byte constants, and
  removed only the selector prelude/additive `selected_platform` field. The
  result equals the complete original driver AST. Workflow normalization of
  names/job key, default runner, new input/env, driver and upload label equals
  the entire original parsed workflow; other steps/keys remain equal.
- Exact reverse substitution of 13 declared hunk groups reconstructs **every
  byte** of the immutable `b619`/`b5` workflow, SHA256 `0a98865e…`. The heredoc
  prefix/suffix also match after their declared header/upload substitutions.
- Context/negative checks passed: one manual job; only `workflow_dispatch`;
  `contents: read`; unchanged legacy concurrency key and no automatic cancel;
  150-minute bound; unchanged jobs=1/incremental=0/debug=0 env, all three private
  targets, four 30 GiB preflights, 10 GiB sample stop and action pins. Expressions
  use only approved input/GitHub/env fields and `always()`; the corrected
  shell-local `$RUNNER_TEMP` root setup is intact with no job-level runner.temp
  context reintroduced.
- `git diff --check` and source-stage `git diff --cached --check` exited zero.
  Staged source scope contained only the workflow; before append, the original
  report was byte-identical to `85b06e3` and all other tracked files were protected.
- `python3 scripts/check-docs.py` exited **1**, reporting only five pre-existing
  root build directories: `target-wave29-source`, `target-wave28-scim`,
  `target-wave28-portal`, `target-wave28`, `target-wave27`. No Markdown link errors
  were reported. These directories and the checker were not changed or deleted.
  Separate report whitespace/fence/no-relative-link checks and the exact original
  prefix check passed; the appendix was the sole unstaged change after the source
  commit. Report-stage `git diff --cached --check` exited zero.

The previous ARM run/receipt remains product `9a819317`, workflow `036a3926`;
it does not execute this new source. No new run/job/artifact ID, digest, tag,
release or container identity exists. X86 availability/capacity/versions,
dependency setup/build time/transient peak and checker success remain unmeasured.
The same future 30 GiB launch/10 GiB owned stop/8 GiB floor, serialized root
runtime release and receipt review are required. No build, Cargo, dependency
installation, artifact execution/download, runner query, Docker/service/network,
browser/desktop, new worker/task/worktree/managed shell or PG overlap occurred.
Root alone performs independent source review/integration and may release a
future x86 run. Primary A09 ownership/status and all original shared/container/
official-artifact evidence limits remain unchanged; Driver-only preference is
retained.

## Container cohort source design only

Reservation `wave30_A09_container_cohort_source_design`, same project/original
A09/supporting WT42/primary f2. Root reviewed/staged `3c21136`/`3df43b1`; this
phase does not publish them or execute the root-owned prospective x86 job.
The complete `3df43b15f4af90211ed0f8ec1718a6a8e4e7c2c5` report remains an exact
28,140-byte prefix, SHA256
`5492e8f7caf839fd888fe0020f1357e7cf06aa7347e17242612ebc78caaf4a89`.
The original `85b06e3` prefix and every historical outcome remain intact. Only
this appendix is materialized; all workflows/helpers/recipes below are proposals.

### Original container outcome and source reviewed

The original row requests tested **server, client, container and maintenance**
artifacts on supported Linux x86-64 and ARM64, with the stated shared
identity/authorization/configuration distribution gate. Native archives alone
do not cover containers. One native x86 container cohort would add a concrete
artifact-family sample; it would not complete both architectures, official
releases, deployed tenants or the entire shared-state gate.

Fixed source is `b619fe25269ccc150e473bbcde47cdb3623ef810`. Complete Git-body
reads covered `Dockerfile`, `.dockerignore`, the small Compose template, release
workflow, packager, focused artifact checker,
standalone client main and the historical encrypted-transition helper. Installed
gate installation/metadata/drill/main bodies were reread; its complete body was
previously reviewed and its fixed-pin bytes match that prior review. Selected
production bodies, not full-file reviews, covered Core startup/me/logout/audit,
UserView, edition assessment/token/activation, format3 agreement enforcement,
rate defaults/resolution/validation, immediate low-risk grants, delegated audit
authority and client user/group/session/grant/transport handlers. O07's corrected
diagnostic appendix (lines 626–787) was read fully; the whole 787-line report was
not reviewed. No current PostgreSQL launcher/helper was inspected or contacted.

Source-backed findings:

- `Dockerfile` builds only `riauth` with locked, explicit edition features from
  Rust 1.98.1/trixie digest
  `a8a5f0a1e5fe7dfe1d352591e4a1c7dd2c08fd70475cae872cf3458ba0df0546`.
  Runtime Debian/trixie-slim digest is
  `a99cfc517144bc59b1978475ec53b46ecabec7e43635402ee5b77cc54cd1b20a`;
  runtime dependencies/notices, revision/edition labels, `USER 10001:10001`,
  `/data` mode 0700 and entrypoint are explicit. No smaller production defect
  was established. The Dockerfile does not itself bound Cargo jobs.
- Release/package source produces both images plus five binaries and requires
  actual release inputs. `check-installed-release-gate.py::verify_assets` refuses
  an incomplete LOCAL cohort; do not forge v4 release provenance or invoke that
  entire gate with substitutes. Its `drill` supplies relevant explicit-handoff
  assertions, not a container or ordinary-user grant equivalence pass.
- Focused checker image methods exercise routes, agent refusals/state and direct
  downgrade. They do not test verified standalone-client journeys or all shared
  identity semantics. Its native tool installer uses mode 0700; a bind-mounted
  tool owned by a different measured host UID would not be executable by image
  UID10001. A new adapter must handle **public executable** permissions correctly;
  no native permission failure is claimed from source alone.
- `src/config.rs:803` requires HTTP issuers to listen on loopback. The current
  image route checker uses an HTTPS issuer with raw HTTP requests; that is not
  evidence of the client's exact discovery-issuer verification. The new native
  fixture must use host-network loopback with matching HTTP issuer/listener,
  rather than inventing a TLS waiver or changing issuer verification.
- `Core::open_store` checks edition compatibility and the complete agreement
  before migration/recovery/startup writes. Node-security format is **3**; its
  transition preflight compares issuer, authentication and all 16 effective rate
  categories. Edition activation rechecks the full config/store-bound token
  inside the locked transaction and stamps metadata; no adoption is implicit.
- `live_identity_and_grant` in the encrypted helper creates a real password user
  and low-risk auditor grant through CLI. Its whole `run`/metadata/probe path is
  ARM-specific and expects format2: it must not be called or relabeled a current
  container gate. The new fixture should reuse the public command semantics only.
- Auditor scope `audit/events` permits `/api/audit?limit=1` through
  `Core::audit_events`; it does **not** confer the broader `audit` inventory scope.
  The planned positive permission assertion therefore uses that exact HTTP
  route, not a claimed successful `riauthctl inventory audit` alias.

Fresh selected byte hashes (SHA256; hashing is distinct from body coverage):

| Fixed-pin body | SHA256 |
| --- | --- |
| Dockerfile | `458ebb247170c6d4af2ff45b22e5a36e0a5380612140a43448d2ddcebc5d83cb` |
| Encrypted helper | `09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e` |
| Edition transition | `301548a213f422e3dfd37914a83f7bf93f8b69ae7f4d0753fc2dbd61295201cb` |
| Node-security | `979f1d22a4835bce97866e302dc2a8a21df4f0d383d4bd1e6960b6a5d7a3bcb7` |
| Core | `13da3f54b28a130cd73e5380253248c650f110b4c4bfb563c88382f55cc74304` |
| Client main | `e75f26eedb1b740d6cbeacf32663f01fb5b802330be6a612eecfca35f29a7da6` |
| O07 report | `202589fec95f51e4de1724ecde6d887a0732518edfb35e073bb099a60ddd36b3` |

Release/packager/focused/installed checker hashes still match those recorded
earlier. Additional production/client body hashes were computed, not executions.

### ONE smallest prospective cohort and source claim

Propose **two new paths only** after root reservation:
`.github/workflows/check-local-container-cohort.yml` (one manual native Linux
x86-64 job) and `scripts/check-local-container-cohort.py` (bounded transport,
image build/archive and container/CLI fixture adapter). Neither exists at the
fixed pin. Leave Dockerfile, release/package/checkers, native-artifact workflow,
shared-state workflow/helpers, deployment templates and primary A09 report intact.

Use the root's eventual **actual, reviewed five-native-x86-artifact cohort** as
input, rather than rebuilding those five binaries or consuming old ARM products
as x86/current source. Two full source-built LOCAL images use exactly that
cohort's product source. Runtime inputs are full product SHA, full immutable
root-receipt commit and narrowly allowed x86 receipt path. Source/workflow SHA
and helper hash are separately recorded from the published controller revision.
The actual native receipt/run/artifact IDs, product/workflow pins and hashes
must come from that future accepted execution; they are not available here.

Controller requirements before any image build:

1. Native Linux `x86_64` / runner `X64` / GitHub-hosted `ubuntu-24.04`, manual
   branch dispatch, exact reviewed controller/product commits, clean product
   tree and input/recipe hashes. Local native Docker socket only; no Desktop,
   remote daemon, emulation, global daemon/context change or same-host O07 retry.
2. Trusted immutable root receipt with exact project/schema, native architecture,
   five products/editions/features/triple and matching product SHA/tree. Actual
   GitHub metadata must match its repository/run/attempt/artifact/name/digest;
   workflow-run head matches **build workflow SHA**, not product SHA. Expired,
   mismatched or missing inputs are refused, never replaced with newest artifacts.
3. Owned capped download of the one exact native artifact. Hash retained outer
   ZIP bytes against its declared digest; reject mismatch, do not silently use
   another transport. Strict ZIP member/path/type/size checks and exact five
   archive/member/license/notices hashes; extracted ELF64 little-endian machine
   62. Native receipt/evidence hashes and identities remain separate from the
   new container job. Proposed transport caps: 512 MiB compressed/expanded;
   root must compare actual x86 sizes before release, not assume ARM sizing.
4. Private 0700 input parent; public binaries mode 0755 when mounted read-only
   into UID10001 containers. Credentials/config/keys/sessions remain 0600 and
   unlogged. Validate native capabilities/versions/features before fixture use;
   no USB/default/test-support/fuzz feature or Store probe is added.

The one new job has `contents: read`, narrowly necessary `actions: read` for
same-repository artifact transport, no write/id-token/attestation scope, no
automatic trigger/matrix/cancel, and the legacy native-artifact serialization
key. Root's ledger also serializes this with the shared-state job; no modification
of its workflow or ownership is proposed. The job remains 150 minutes, with an
owned controller deadline of 7200 s and fixture deadline of 1200 s.

### Image recipe, product identity and exact future commands

The original Dockerfile stays unchanged. Generate one **private resource-only
validation recipe** from the checked product Dockerfile: require exactly one
existing edition-selected Cargo command, and prefix that command's environment:

```dockerfile
    && CARGO_HOME=/build/cargo-home CARGO_TARGET_DIR=/build/target \
       CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
       CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 \
       cargo build --release --locked --no-default-features --features "$RIAUTH_EDITION" --bin riauth
```

No other recipe byte changes: base digests, apt packages, source COPY set,
edition validation, release thin LTO/strip, output COPY, runtime labels/dependencies,
UID, directory permissions and entrypoint remain. Reversing the exact prefix
must reconstruct the pinned Dockerfile. Record original/derived recipe hashes
and the explicit resource-only variant in LOCAL provenance; do not claim the
canonical unmodified Docker build command was executed. Source pin denotes
checked product code; recipe/controller pins are separate. Native and image
server hashes can differ because Ubuntu native and Debian image build baselines
differ; compare each to its own recorded bytes, not falsely require equality.
The existing `.dockerignore` whitelist/private-material exclusions remain;
receipt downloads, logs, Docker configuration and fixture secrets live outside
the product context. No GitHub transport credential enters a build argument,
context or application environment.

Use one fresh private Buildx config and a dedicated builder, never the shared
default builder. Before implementation/release, root must supply a reviewed
content-addressed BuildKit image reference: **none is pinned in these source
bodies and no digest was guessed or fetched here**. Availability/driver controls
and actual native image architecture need future measurement. This tool pin is
a concrete ownership/cancellation prerequisite, not a new release/device gate.
Builder creation must provide an exclusive name and retain its actual container/
volume creation identity; refuse existing resources or unverifiable ownership.
Root reviews those controls before permitting any privileged builder lifecycle.
Application containers never inherit builder privileges/socket/credentials.

Prospective argv (all variables are controller-owned/pinned, not free shell code):

```bash
docker buildx create --name "$BUILDER" --driver docker-container --driver-opt "image=$REVIEWED_BUILDKIT_REF"
docker buildx build --builder "$BUILDER" --platform linux/amd64 --load --progress plain \
  --file "$A09_ROOT/Dockerfile.serial" --build-arg RIAUTH_EDITION=essentials \
  --build-arg "RIAUTH_COMMIT=$SOURCE_SHA" --label "org.riauth.local.owner=$OWNER" \
  --tag "$ESSENTIALS_TAG" --iidfile "$A09_ROOT/essentials.iid" "$PRODUCT_CONTEXT"
# Identical second argv with edition=platform and its own tag/iidfile.
docker image save "$ESSENTIALS_TAG"
# Stream into owned gzip mtime=0 output; repeat for Platform, then hash/validate.
docker image rm --no-prune "$ESSENTIALS_TAG" "$PLATFORM_TAG"
docker image load --input "$A09_ROOT/evidence/local-essentials-x86_64.docker.tar.gz"
docker image load --input "$A09_ROOT/evidence/local-platform-x86_64.docker.tar.gz"
```

Each build is sequential and capped at 1800 s; metadata/pull/setup/save/load
operations have explicit shorter bounds under the same sampler. Image archive
cap is 2 GiB each. Validate tar manifests/member paths and exact owned tags/IDs
before load; verify the loaded identities equal the saved images. Record actual
image config IDs, archive SHA256/size, container server SHA256/ELF, exact source/
edition/owner labels, Linux/amd64, `Config.User=10001:10001`, runtime notices and
capability target/features/version. Source base pins and actual resolved base
identities/toolchain/dependency observations are distinct fields. LOCAL image
IDs are not registry digests; no tag, push, official provenance or attestation.

### One bounded container/standalone-client/maintenance gate

All application/tool containers use explicit UID/GID10001, read-only root,
no-new-privileges, dropped capabilities and bounded `/tmp` tmpfs. Each fixture
has two owned named volumes: private configuration/key and shared data. No root
application, root ownership repair or permission relaxation after a refusal.
A fresh named volume must first report `/data` UID/GID10001 and mode0700 through
an unprivileged numeric stat probe, before keygen/init/credentials. Seed/probe
each new volume at the image's existing `/data` mount, then mount those same
recorded volumes at `/config` and `/data`. Serving mounts configuration/key
read-only and data read-write, as the small template separates them; offline
setup mounts configuration read-write only for the explicit owner tool.
Missing/unusable mapping stops the fixture. Named-volume copy-up is a source
expectation, not an actual native pass here.

Use native **host network** for the serving container with an unused random
`127.0.0.1:PORT`, and that exact HTTP issuer/listener in the generated config.
No fixed9000, public interface, bridge fallback or Docker Desktop opt-in. This
is loopback fixture scope, not network isolation or public TLS/deployment proof.
Offline tools can use network-none. Example maintenance argv retains the image's
default user and mounts only the exact public tool read-only:

```bash
docker run --rm --user 10001:10001 --network none --read-only \
  --cap-drop ALL --security-opt no-new-privileges --tmpfs /tmp:rw,nosuid,noexec,size=16m \
  --mount "type=volume,src=$CONFIG_VOLUME,dst=/config" \
  --mount "type=volume,src=$DATA_VOLUME,dst=/data" \
  --mount "type=bind,src=$ESSENTIALS_MAINTENANCE,dst=/cohort/riauth-maintenance,readonly" \
  --entrypoint /cohort/riauth-maintenance "$ESSENTIALS_IMAGE_ID" \
  --config /config/riauth.toml --json --non-interactive init \
  --issuer "$BASE" --listen "127.0.0.1:$PORT" --data-dir /data \
  --database-key-file /config/database.key --password-stdin
```

First keygen uses the same unprivileged tool to create `/config/database.key`;
application init is first-only. Check actual config/key/store ownership/private
modes. The image's entrypoint metadata is unchanged. For this separated config
layout, serving explicitly uses the same `/usr/local/bin/riauth` with
`--config /config/riauth.toml serve`, matching the small template's supported
entrypoint adaptation; no binary/wrapper/authentication bypass. Server startup
uses host network, read-only config/key mount, the same controls/data volume
and exclusive recorded name/cidfile. Native `riauthctl`
runs on the host with explicit `--server "$BASE"`, private session files,
`--json --non-interactive --request-timeout 30`; secrets enter stdin only.
Passwords, session tokens and transition-token argv values are never logged.

| Planned phase | Exact success/refusal assertions, not executed results |
| --- | --- |
| Essentials baseline | Loaded Essentials image readiness; exact discovery issuer and unchanged JWKS. Base client password login/whoami. Administrator creates one non-admin password user, one unprotected local group/membership, and low-risk `auditor`/`audit/events` grant with fresh revision/request keys. Client verifies user/group/grants; delegate password login/whoami; direct `/api/audit?limit=1` is200; delegate attempted user-create is403/client exit4, with no created child. |
| Revocation sample | Copy one delegate session privately before logout; reuse through its issuer-bound session file and require401/client exit3 before and after each handoff. Revoke a second delegate session while Platform is active and require both remain refused after return. Fresh password login remains possible. Never omit a refusal from the report or convert local missing-file failure into a server401. |
| Offline config refusals | Stop/reap the owning app first. Exact one-field private config variants for issuer, `session_ttl`, and `rate_limits.login` must parse with only that intended change. Platform maintenance plan rejects them with exit5 and the corresponding issuer/node-security blocker; source-edition image startup also rejects them. Generic invalid TOML is not a passed policy assertion. Original config stays byte-identical; no agreement record/adoption/upgrade bypass. |
| Explicit E→P→E | Baseline target plan is ready. One Platform maintenance tool calls existing plan/activate with the exact private token per direction; direct opposite-build open before activation is refused. Start the corresponding loaded image on the same volume/config/issuer; repeat fresh password login, discovery/JWKS, complete UserView/groups, exact low-risk grants and allow/deny/revoked-session assertions. All12 UserView fields remain compared; only top-level session_id/expires_at vary for fresh sessions. |
| Credential/config and edition boundaries | Both maintenance versions match the observed server version; Essentials does keygen/init, Platform does both handoffs. Format3 agreement enforcement runs at each startup/plan, preserving the same config's authentication and 16 rate settings. A second owned Platform-only fixture creates the existing agent-state sample with unchanged route-specific If-Match/request headers; Essentials direct/preflight downgrade must refuse that state. No privileged grant/group review or source-derived Group materialization is bypassed. |

The Platform-only refusal fixture is part of this single bounded gate, kept
separate so it cannot contaminate the clean return fixture. Every call goes
through existing public CLI/HTTP/Core methods. No raw store mutation, new probe,
patched callback, credential import/adoption, API alias or helper privilege
bypass. Stable snapshots remain private; public evidence reports statuses,
counts and product hashes, not passwords/tokens/keys/opaque identity values.
This samples container-shared identity/authority/configuration behavior; it does
not assert all credential kinds or complete raw-row/PG/encrypted cross-build
coverage. Existing shared-gate workers retain that separate scope.
Private candidate configs are created as UID10001 in the owned configuration
volume while the service is stopped, using an exclusive fixed-path stdin writer
with umask077; they are not host-owned 0600 files made readable by weakening
permissions. Original config/key are never overwritten. Transition tokens stay
in private controller memory; required CLI token argv is redacted in evidence.

### Ownership/resources, O07 retention and prerequisites

One private run root/config/log/evidence directory (0700/0600, exclusive paths),
empty private Docker client configuration with no operator registry credentials,
fixed local Unix daemon, fresh dedicated builder, owner-labeled images/networks/
volumes/containers and exact creation records. Docker daemon children do not
belong to the CLI process group: cleanup must explicitly cancel/remove the
owned builder and stop/remove owned application/tool containers as well as
TERM/KILL/join CLI groups. Record resources before mutable operations; on a
lost create response resolve only the exclusive owned name/labels/creation
identity. If ownership cannot be proven, stop and report cleanup blocked, never
remove a guessed or shared resource. No global prune, shared cache deletion,
daemon restart/settings or root application fallback.

At launch and before download/builder/build/fixture phases, require measured
30 GiB free on host/workspace/private and actual Docker-storage filesystems.
Sample all every nominal2s; below10 GiB stop only owned work and retain8 GiB
floor margin. Cap command logs8 MiB and evidence/resource streams; sensitive
CLI/HTTP/application output is discarded after in-memory typed assertions,
not raw-logged. Build/dependency logs contain no input credentials. Finally stop/
reap/join, recheck exact owned inventories, remove only owned disposable state,
and retain capped redacted receipt/image/archive/hash/failure/cleanup evidence.
Sampling/cancellation are not hard quotas; unknown peak/shared drain can refuse
the run. Empty cleanup claims require actual inventories, not process exit alone.

The O07 corrected run remains **EXIT1**, diagnostic child_rc0, UID/GID0:0,
mode448/0700, `uid_mapping_unsupported`, one image-provenance check and cleanup
reported okay. Evidence1878 bytes SHA256
`351349aa31e383fabd76a13dce188019437c9174af56d92a9824888abdba8af9` is retained;
the earlier1745-byte failure and its classifier limitation remain separate.
No credentials/service/host-visibility/TLS/HA checks were reached. This native
hosted proposal does not repair, retry or retrospectively pass that host mapping.

Measured here: immutable source/evidence bytes and those **recorded** historical
outcomes. Unmeasured prerequisites: root-approved new source/recipe/transport/
controller review, actual accepted native-x86 receipt and size/digest identity,
reviewed BuildKit tool-image pin, allocated native Docker/buildx/host-network/
volume behavior and actual≥30 GiB capacity/peak/durations. Historical ARM104.45
GiB minimum and O07 Desktop11.04 GiB free are not new-cohort capacity. No current
image IDs, archives, successful gate counts or new GitHub identities are invented.
Root alone can reserve implementation and eventual serialized runtime; no A09
status recommendation follows from design. No full release/registry/tag/
attestation, every-host/browser/device/tenant gate is added.

### Actual checks for this appendix

Clean existing HEAD `3df43b1`; immutable Git body/selected-span/evidence reads and
SHA256 comparisons only. Both proposed new paths are absent at `b619`; no recipe,
workflow or helper was materialized. Prefix/scope/Markdown/whitespace checks are
performed before the report-only commit; no parser or fixture test is claimed
for unimplemented source. Both original85 and complete3df byte-prefix checks,
sole-report scope, unchanged architecture-workflow bytes, Markdown fence/link/
whitespace hygiene and `git diff --check` passed. `python3 scripts/check-docs.py`
exited **1** for the same five pre-existing `target-wave29-source`,
`target-wave28-scim`, `target-wave28-portal`, `target-wave28`, `target-wave27`
directories, with no Markdown-link errors; no checker/directory edits or deletion.
One append-edit patch used an incorrect literal (`different` for `differ`) and
was refused before any mutation/check execution; correcting the exact context
applied it, with both prefixes subsequently proved intact. This was a local
documentation edit failure, not a fixture/Docker run. No Docker/native/tool-version/Cargo/build/pull/query/
download/dispatch/service/browser/network, worker contact, new task/worktree/
managed shell, merge/reset/main/push/status or current PG overlap occurred.
Driver-only desktop preference and all receipt/header/PAM/Group/60s contracts
remain unchanged.

## Native container cohort implementation — SOURCE ONLY

Reservation `wave30_A09_native_container_cohort_implementation`, project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`, supporting WT42; primary f2 unchanged.
Root approved the full `a7a314e` design. That entire report remains an exact
52,019-byte prefix, SHA256
`8584ee2e9b536dcbf0eb543dac191c362f149caa9e8440eb1af7fcbcedfc45b4`.
The original 85/3df prefixes and historical outcomes are retained verbatim.
No source alignment, merge, reset, existing production/helper/workflow edit or
board change was needed or performed.

Source commit **`be22eebb8cc957037f5e5c1f153f87df81f3dc25`** adds only:

| New body | Lines / bytes | SHA256 |
| --- | --- | --- |
| `.github/workflows/check-local-container-cohort.yml` | 88 / 3398 | `7a0b7fb12a6c2650a1a7413e856ed6a9053aed7f5e1f06c2172413c7fdf84302` |
| `scripts/check-local-container-cohort.py` | 1516 / 85922 | `0f499284f5a053026230586f936b0ba38e58898a4f8fe36885efd30161bbbcfa` |

### Reviewed inputs now supplied, distinct from a new container execution

Immutable root receipt commit
`66c814a339665e6b3f8e6c22bc59d4f6f0aa224c` supplies both input records. Full
receipt bodies were read and their bytes hashed; no official URL/registry/network
request was made here.

- Native receipt path
  `docs/roadmap/evidence/wave30-a09-native-x86-root-review.json`, 11,244 bytes,
  SHA256 `ccf7c3fb6c0bd85816097c32a24a1c1b0a67b9597346d571cd68975a4680b1af`.
  Actual SUCCESS run **37046857550**, job **110970324302**, attempt **1**;
  artifact **11246575279**, `riauth-local-x86_64-37046857550-1`, **52428080** bytes,
  API/upload digest
  `sha256:7c1bcad0f78d90eb611ab76d67943a32a0d691cd5bbd9e5588512bdde635e2a0`.
  Product source `b619fe25269ccc150e473bbcde47cdb3623ef810`, tree
  `a627df2ce21a4d255b8c1914d4f543e32f40f4de`; native build workflow source
  `0d090169f20f61f7cb59b685dccb13203526539f`, workflow hash `62c37897…`.
  Those two source identities are deliberately different. The native input has
  five binaries, ELF machine62 and eleven historical commands with exit0;
  `shared_full_gate=not_run`. The stale notices at b619 are retained explicitly.
- BuildKit receipt path
  `docs/roadmap/evidence/wave30-container-buildkit-source-pin.json`, 8628 bytes,
  SHA256 `4d263888b2bdc5a2eb922b29c85535e8321dc2cba7c246b5b6b355cef57e57b0`.
  Root's read-only v0.33.1 index pin is
  `sha256:cec9f139f45e93c5c69c60f8b07cfad9f43f4ef6b6a6cd917527fea5ff2e3dea`;
  selected native amd64 reference is
  `docker.io/moby/buildkit@sha256:98cc6a3fc46220d00f8224ae483f3274fc874e9be8d7dd1e2e2c5481209228b5`.
  Its config digest is checked by the future controller. Root's receipt states
  no layers downloaded/no native execution; that is not compatibility evidence.

The allowed local historical evidence directory was read, not replayed.
`riauth-local-x86_64-37046857550-1/evidence.json` is 60,389 bytes / SHA256
`1527aa7388a13554d3699f4869af12d7f91d3fbe1fe8ebf18e18b50c0c31b986`;
`resources.jsonl` is 195,534 bytes / SHA256
`74606c2c2709d4a02664e7c66367acf9eb8b61898abb28e8d040c37f9f4937bc`.
I independently rehashed those, the eleven recorded logs and five compressed
archives against the immutable root receipt. I did not execute/extract the
binaries or freshly repeat their ELF/three-member tests. The raw native JSON
contains `observed_server_capabilities` beyond the receipt's product summary;
the new helper compares those independently against the hashed capability logs.
Root did not retain/rehash the historical outer ZIP. The new transport must hash
actual downloaded outer ZIP bytes and refuse disagreement, with no alternate
download/retry. The historical native evidence is not a new container pass.

### Materialized control and public-interface gate

One manual native `ubuntu-24.04` job, no matrix/automatic trigger, read-only
contents/actions, pinned checkout/upload actions, legacy serialization key and
no automatic cancellation. Separate immutable controller/product/review
checkouts; exact branch controller identity, product tree, receipt hashes and
clean source checks precede product calls. Preparation steps have explicit
timeouts; the controller's nominal2s sampler covers its source/transport/Docker/
fixture operations. The launch guard measures capacity before action checkouts;
no claim of sampler coverage during those checkout actions is made.

The future controller performs one exact same-repository artifact transport,
API/run/attempt/source identity validation, strict ZIP/TAR member checks,
512 MiB compressed/expanded native limits, five archive/binary/license/notices
hashes and ELF62. Public tool files alone become0755; private input parents,
configs, keys, sessions and redacted evidence stay0700/0600. Native capabilities
and versions are checked; no native binaries are rebuilt.

The original Dockerfile and ignore/context rules remain unchanged. The private
resource-prefix recipe reverses exactly to the b619 Dockerfile; jobs1,
incremental0 and dev/test/release debug0 are scoped to its one original Cargo
command. Both LOCAL images use that explicit variant, not a claimed unmodified
canonical build. Dedicated private Buildx configuration/builder and pinned tool
image; no default builder/context, inherited registry credential, global prune,
shared cache deletion, remote daemon or registry write. Each image has its own
actual ID/server hash and both capabilities must match the recorded native
edition; Ubuntu native and Debian image binary hashes are not equated.

Image save/load is bounded, with 2 GiB per-archive compressed/expanded caps,
strict outer member types/paths and tag/config/layer/index identity checks before
load; loaded IDs, edition/source/owner labels, amd64, UID10001, entrypoint/Cmd/
working directory/volume declarations, notices, ELF and version are checked.
Base references remain pinned source inputs; public bounded build logs retain
actual resolution/package output. No independent resolved-base/toolchain audit
or official image attestation is claimed.

Application/tool containers have explicit UID10001, read-only root, dropped
capabilities, no-new-privileges, private tmpfs and finite resource bounds; no
Docker socket/credentials. Two owned volumes per fixture are first copy-up/
numeric-stat probed at `/data`, before any credentials; unsupported UID/mode
mapping stops without repair. Serving uses read-only config/key and writable
data, loopback host-network random port and the exact matching HTTP issuer.
Offline maintenance uses public mounted binaries and closed stdin/config paths.

The single fixture function implements the approved clean E→P→E journey plus
the separate Platform-state refusal fixture: administrator creates one ordinary
password user/local group/auditor grant with fresh revision/request keys;
complete 12-field UserView, me fields other than fresh session ID/expiry, group,
grant, issuer/JWKS, allow audit200, deny user-create403/client4/no new user, two
logout401/client3 samples, explicit exact-token handoffs and wrong direct-build
refusals. Private original config/key comparisons remain exact. Three offline
one-field issuer/session-policy/login-rate variants must parse exactly, show
specific plan/startup refusals and preserve the original config/key/data hashes.
The separate Platform agent sample uses the existing quoted revision/request
headers and requires direct/preflight downgrade refusal; no Source Group,
reviewed privileged grant or writer bypass. Format3/16-rate enforcement is
through pinned public entrypoints, not a new raw-store/credential probe.

Concrete static corrections from the reread b619 bodies, confined to new code:

- `src/cli/local.rs::emit_transition` uses **ok:false**, data.ready:false and
  exit5 for a blocked plan/preflight; it is not a normal success envelope and
  not a normal `error` envelope. This corrects my earlier commentary wording.
- `Config.rate_limits` skips serialization when empty; the exact one-leaf login
  override therefore supports an absent table. Full TOML equality verifies the
  intended candidate, rather than accepting a generic parse failure.
- Standalone client flags/session/JSON envelopes and its403→4/401→3 mappings,
  public maintenance keygen/init/plan/activate, Core.me/UserView/logout/audit,
  grant writer and node-security/activation ordering were reviewed directly.
  Audit scope `audit/events` uses `/api/audit`, not inventory-audit. Discovery
  can advertise edition capabilities: only the stable issuer/JWKS contract is
  compared, while UserView/group/grants remain complete comparisons.
- Docker attach transport status and the product container State.ExitCode are
  recorded separately; assertions use the actual product exit. Docker absence
  needs typed not-found output plus a live daemon check, not any exit1.

The job is150m; controller7200s, fixture1200s, builds1800s each, scoped cleanup
240s, and setup/pull/save/load/metadata operations have finite bounds. Require
30 GiB initially and before large phases, nominal2s host/private/workspace/
actual Docker-storage sampling, own-work stop at10 GiB preserving8 GiB margin,
8 MiB log/evidence caps; sampling is not a hard quota. Docker daemon children
are cleaned explicitly. Expected names are recorded before creation; actual
container/volume/image identities and labels are checked before scoped removal.
The builder's private instance/creation interval/state mount bind cleanup.
Missing ownership blocks deletion. Process start ticks/session/group are bound;
group signals precede leader reap, followed by group-empty proof. Only private
fixture/input state is removed; retained public evidence is redacted. No raw
CLI/HTTP/app output, password/token/key/opaque user IDs or transition argv is
logged. Source/input refusal before daemon setup does not trigger Docker cleanup.

### Actual static checks, failures and remaining release boundary

- Full new source bodies/patches reviewed; Python `ast.parse`, local method
  keyword signatures, closed literal public refusal codes and no existing-helper
  imports checked without importing/running the helper.
- Psych structure/manual trigger/one job/permissions/serialization/actions/
  source refs/context/step deadlines checked; **two** `bash -n` run blocks pass.
- Receipt/input hashes, AST-extracted recipe literal/reversal, historical
  evidence/log/archive hashes, unchanged tracked files and new-file whitespace
  checked. Staged source `git diff --cached --check` passes; source commit has
  only the two reserved additions. No existing Dockerfile/ignore/deploy/release/
  packager/checker/native/shared workflow/helper/product/manifest changed.
- Static audit failures retained: the first historical JSON read used the
  directory root instead of the uploaded-artifact subdirectory (FileNotFound;
  corrected read only); an attempted patch had an extraneous nonexistent
  context and was refused without mutation; a first Psych probe assumed YAML
  1.2's string `on` key (Psych's YAML1.1 decodes it as true; checker normalization
  fixed, workflow unchanged); a first AST literal probe did not resolve constant
  dictionary keys (probe corrected without importing the helper). Subsequent
  named checks pass. None was a container/fixture failure or pass.

Documentation/append-prefix/scope/whitespace checks are recorded below after
this append. There was **zero** Docker/buildx/pull/version/native binary/helper
main/import/Cargo/typecheck/test/HTTP/query/download/dispatch/service/provider/
browser execution here, no slot reservation or worker contact. No new task/
worker/worktree/managed shell, main/push/status or source alignment occurred.

Root source and independent review still precede any serialized native cohort
release. Actual native Docker/buildx driver/private-instance/archive behavior,
UID copy-up/mounts, maintenance library compatibility, capacity/peak/durations,
public-interface assertions and cleanup remain **UNRUN**. The named input pins
are now supplied; those runtime properties are not inferred from their hashes.
O07's actual UID0:0 versus10001/EXIT1 limitation and both historical receipts
remain in the preserved prefix; no root-application fallback, same-host retry,
host-mapping repair or retrospective success. No full A09/shared/ARM-container/
official-release/tenant/device conclusion or status recommendation from source
alone. Root alone integrates, releases runtime and decides original closure.
Driver-only desktop preference and all receipt/header/PAM/Group/60s limits remain.

Post-append actual checks: exact85/3df/a7 byte-prefix proofs PASS; current diff
contains only this report; total phase scope is exactly the two new source paths
and report. Both source files still match `be22eeb` byte-for-byte. Markdown
fences/whitespace and `git diff --check` PASS. `python3 scripts/check-docs.py`
EXIT1 reports only the same five pre-existing build directories:
`target-wave29-source`, `target-wave28-scim`, `target-wave28-portal`,
`target-wave28`, `target-wave27`; no Markdown-link errors. No directory deletion
or checker change. These static results do not release any runtime.

## Root full source and independent review acceptance

Root fully read all1,516 helper lines and88 workflow lines, Dockerfile/.dockerignore, the201-line implementation appendix and the complete222-line independent8f29d25b4f632e3fa1c4eba31d53340a2abe7cf1 review. Whole source hashes, AST/in-memory syntax only, manual permissions/action/provenance controls, both shell blocks and exact resource-prefix reversal were checked. No concrete source/interface blocker was established. Source be22eebb8cc957037f5e5c1f153f87df81f3dc25 is accepted for publication, without importing supporting alignment history or changing existing product/recipes. Native daemon/private builder schema, image save/load layout, UID copy-up, mounted tool compatibility, actual fixture gates and cleanup remain unmeasured.

The selected container snapshot deliberately excludes fresh session ID/expiry from cross-handoff equality; it verifies saved logged-out sessions stay refused. It does not measure same-session expiry nonrenewal. The separate PostgreSQL helper retains that assertion, currently unreached after its recorded Store-equality failure. No cross-gate success is borrowed. Root may later reserve one exact native x86 cohort after immutable publication and lane availability; no dispatch or Docker runtime is credited by this source review. Original A09/O07 remain open and both earlier Desktop failures remain failures.
