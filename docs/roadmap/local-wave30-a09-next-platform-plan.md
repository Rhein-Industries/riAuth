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
