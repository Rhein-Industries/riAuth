# Wave30 Public CI notices source diagnosis

2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, reservation
`wave30_CI_notices_source_diagnosis`, existing WT
`a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`, clean starting HEAD
`717cc58fa154e6723f1eefe9f5c6e083cabd4992`.

**A stale client-lock digest is a proven, host-independent reason the notices
check fails.** Recommend the smallest prospective correction to
`THIRD_PARTY_NOTICES.md` line 5: replace only its client lock SHA-256 with the
digest of the accepted lock. Source inspection supports a header-only result;
it does not establish a freshly generated whole-output comparison. No
existing file or generator behavior was changed, and no generator/Cargo
execution occurred. Root must separately reserve implementation and any
validation runtime.

## Exact source and historical failure

Reviewed Git objects, without merging or importing files:

- Completed Public CI source: `b5dcfa9dbb14e953d12edac6a6a1133ecb61033e`.
- Later root source: `b619fe25269ccc150e473bbcde47cdb3623ef810`.

The nine source inputs below and all 14 tracked
`scripts/third-party-license-sources/` files are byte-identical between those
two revisions: 23 input files in total.

| Input | Git blob at both revisions | SHA-256 |
| --- | --- | --- |
| `scripts/generate-third-party-notices.py` | `fabe68a69cf18f9632b9ad619417414d8ee45bf1` | `db96e420c69c668f4ed542188f5df80bdfe124d9e5bee58caa9949e8f3637dc8` |
| `Cargo.lock` | `f1b819d47d204d73617b095513f0c6ab6eb8aa4e` | `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` |
| `crates/riauthctl/Cargo.lock` | `a471c5c447c51c8d82002d7f31060e8c32099886` | `2998555ddb2d00e8130a970fcacfed19dfee7a0b2e3305011ebd40746f67b4db` |
| `Cargo.toml` | `5660d4bb922fcdc5bfe05d7502585980f4720d06` | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |
| `crates/riauthctl/Cargo.toml` | `e97ff28b79770f55e2c1dabb2de9479705d789f0` | `af385d4d989c53987396edfdfa684cd3902a603d1cf65dd31ac72da7f08de9ea` |
| `THIRD_PARTY_NOTICES.md` | `8c05ba7e5c50c9ea91cd3c046216cbf82404daa7` | `142c0e5150de9436513d9f6f215c5422b8a3af84d4eb7d0708f155e3e18fce05` |
| `.github/workflows/ci.yml` | `7c724fd7f4dc210a2268f5a702bdc09b1a10d76b` | `fc5245ef15f0ac8d0a460b71ded533ad96ad3dcbd795b31f38b3e84221940b5a` |
| `.github/workflows/release.yml` | `897df176d6dd773f9e86f8dfdb3655003c5c32ca` | `da8607c5ea33a906185141e0caebec18c2282c22676981e0c659fd65eef4369f` |
| `rust-toolchain.toml` | `c3f67b6771b777215340531caf051bc25cef066c` | `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167` |

The release notices check at line 84 uses the same generator as CI line 106.
Toolchain source pins Rust 1.98.1; no toolchain/native version command was run
here.

Independently read/hash-verified the supplied historical log:
`/tmp/riauth-wave30-ci-37037991415/check-110940834919.log`, 818,157 bytes,
mode 0600, SHA-256
`54b582c1af19299c38a06b77d4aa1e8918f2ffc3ad16f4d7979915c2284e0a5f`.
Its all-target Rust step has 180 result-summary lines totaling 1,330 passed,
0 failed and 181 ignored. This is historical execution at `b5dcfa9`, not
fresh lane testing or proof of whole CI completion. The later docs step
reports the Markdown check passed at log line 5054, then generator failure:

```text
5118: 2026-10-02T17:53:28.4190422Z THIRD_PARTY_NOTICES.md is stale; run scripts/generate-third-party-notices.py
5119: 2026-10-02T17:53:28.4300193Z Process completed with exit code 1.
```

The check job failed; subsequent commands in that shell step and its later
release build are not claimed successful. Root reports `b619` CI still
running; no network query was made to confirm or change that state. Equal
source inputs establish the same embedded digest mismatch at `b619`, not an
observed failure or outcome of that running CI.

## Full generator trace and bounded dependency evidence

Read the complete 251-line generator (12,055 bytes), both complete manifests,
both complete locks and CI/release workflows. Static TOML inspection parsed
433 server lock packages and 289 client lock packages. Generator bodies:

1. `command` (71–72) executes subprocess output at the repository root.
2. `linux_dependencies` (75–110) runs locked metadata and normal-edge trees
   for Platform server and USB-free client. Tree target is explicitly
   `x86_64-unknown-linux-gnu`; server uses `--no-default-features --features
   platform`, client uses `--no-default-features`. It sorts the union by
   name/version, excludes both roots and rejects ambiguous IDs, unsupported
   sources and missing license declarations.
3. `override_files` (113–145) chooses reviewed, version-scoped fallback
   license files. `license_material` (148–164) reads crate license/notice
   files, the fixed extra bundled-source files and the CMS README license
   section; invalid/empty material is refused.
4. `fence` (167–169) chooses an enclosing Markdown fence. `render` (172–230)
   validates the override-source map, hashes exact notice bytes, sorts
   rows/texts/origins and includes **both complete lock-file hashes** at
   lines 196–206. `main` (233–244) compares the entire rendered output in
   `--check` mode and otherwise writes it. The final guarded entry point
   converts value/subprocess errors to failure. No body was invoked/imported
   during this audit; AST parsing was inspection only.

The notices are 752,651 bytes/14,514 lines. Complete-byte/header/inventory
and fenced-body integrity inspection found 328 unique dependency rows and
225 notice sections. All referenced IDs resolve, no section is unreferenced,
all source-label lists are sorted/unique, and every body matches its content
ID. Six bodies require allowance for the generator's added final newline
when hashing the original source bytes. The sections reconstruct the entire
license-text suffix. All dependency keys exist in the union of the two
locks; header/separator rows were excluded from that check. The 13 override
URLs parsed statically from the generator equal `SOURCES.txt`. This is
integrity inspection, not independent upstream retrieval, Cargo feature
resolution or a fresh license-compliance review.

Accepted commit `f984a84af950d495642edd3f92508c86e10b0cd7` changed the client
lock from 70,613 bytes, SHA-256
`6f546c966c70505b8ef204cea811f7300b1b359de63f01a8389caadc3164b964`,
to the current 70,912-byte lock. The old digest is exactly the digest still
printed in notices line 5. No registry package/version/checksum was removed
or upgraded: one package was added, `signal-hook-registry` 1.4.8; only the
local `riauthctl` dependency list and Tokio's dependency list otherwise
changed. The manifest added direct `aws-lc-rs`, `base64` and Tokio `signal`.

These affected versions are already recorded in the server lock with equal
registry checksums and in the notices: `aws-lc-rs` 1.18.1, `aws-lc-sys` 0.45.0,
`base64` 0.23.1, `signal-hook-registry` 1.4.8, `errno` 0.3.14 and `libc`
0.2.189. The server manifest directly enables AWS-LC/base64/Tokio `signal`;
the server Tokio 1.53.1 entry already includes `signal-hook-registry`.
These witnesses support an unchanged release union, without pretending a
lock lists the exact selected target/feature graph. No generic lock-only
replacement for the generator is proposed.

The entire notices file still equals accepted notices commit
`520686dd226534e40cc14b7a0d932eae2eaf9989`, which predates that client
update. The client lock has remained identical to `f984a84` through both
reviewed CI revisions. The server lock's printed digest is current. Since
`render` necessarily embeds the current client digest before comparison,
the old client digest alone guarantees unequal output, independent of host
or any additional graph/license difference. The historical log did not
capture a generated diff, so it cannot prove that this is the sole difference.

## Exact prospective correction and validation prerequisite

Reserve **only the generated client-lock field in `THIRD_PARTY_NOTICES.md`
line 5** for the next correction. Preserve the server digest, target,
inventory, notice texts, generator, manifests, locks and workflows. Expected
field substitution:

```diff
-Client Cargo.lock SHA-256: `6f546c966c70505b8ef204cea811f7300b1b359de63f01a8389caadc3164b964`.
+Client Cargo.lock SHA-256: `2998555ddb2d00e8130a970fcacfed19dfee7a0b2e3305011ebd40746f67b4db`.
```

This is an in-line field within the existing combined header line, not a
new standalone line. The virtual one-field replacement preserves total
length 752,651 bytes, Git blob `d747d40297dd8aa6ebd0978f82bd4c012a6ccb36`,
SHA-256 `8a92f38f2a648d08aa21f415f9ad34b966ee7df6bbf6ba89ddd926ef8833efa2`.
Reversing that exact replacement reconstructs the entire original notices
file byte-for-byte. These are **prospective source hashes**, not generated
output hashes or evidence of a passing check.

Respect the generated-file contract: a later separately authorized generator
refresh should produce/review the actual diff. If it differs beyond this
field, stop for root's review of the exact graph/text delta rather than
silently broadening the edit or relaxing the check. The necessary focused
validation is the original `python3 scripts/generate-third-party-notices.py
--check` on the corrected, pinned source; no repeat Rust test suite is needed
for this proposed notice-only delta. An already prepared Linux x86_64 host
matching CI is the direct check seam. If root requires cross-host parity,
compare generated bytes/diffs on the same pins/locks/toolchain with Linux's
result, without inventing a host-sensitive generator fix before a difference
is observed. No workflow changes or duplicate generic gates are proposed.

That check is **not execution-free**: it invokes two `cargo metadata` and two
`cargo tree` commands. Metadata is not target-filtered and may fetch missing
non-Linux crates, as the historical log's download lines demonstrate.
`--locked` preserves dependency resolution but does not guarantee offline
operation. It requires the pinned toolchain and either a complete usable
registry-source cache or separately authorized dependency acquisition and
capacity. It does not call a build/test/service/native helper, but cache
fetch/metadata allocation remains unmeasured here. No local Cargo slot is
requested or consumed. Local host pressure reported in commit `717cc58`
remains a prerequisite for root-controlled remediation and fresh headroom;
this audit grants no cleanup, network, metadata or runtime permission.

## Actual checks and remaining scope

Performed only Git-object/source reads and diffs; complete source SHA/blob
identity checks; static TOML/AST/notice structural inspection; supplied log
hash/result/boundary inspection; virtual substitution/reversal hashing; and
report-only Markdown/whitespace/scope checks. These are not execution of the
generator, metadata, tests or delivery helpers. The capacity report was
verified byte-identical to `717cc58` and remains unchanged. No alignment
merge, deletion, network query/download, dependency installation, native
version command, product/runtime/build/test, worker contact, new worker/task/
WT/shell, task-status/main/push action occurred.

Only this new report is written/committed. Root owns implementation,
validation release, integration and CI interpretation. The completed
historical check job failed; whole CI is not claimed green and the current
`b619` counterpart is not assigned an observed failure. I10/R05/W02/W05
remain DONE.

## Append-only generator preparation — runtime remains held

2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, reservation
`wave30_CI_notices_generator_preparation`, same WT/branch, clean starting
HEAD `138468ef4afcee569af560ea9af472d3470d1006`. The preceding 189 lines /
11,525 bytes, SHA-256
`754b522b57ffbc36927fe7e1716c0051f9d40623f97db8608b7bf636cba1869a`,
are preserved exactly, including the historical CI failure and prospective
hashes. **No manual notice-field edit is authorized or proposed for
execution.** Only the unmodified generator may produce the future notice
refresh, after root separately reserves/releases its serialized metadata
lane. Root's native-host run `37046857550` owns the preparation/Cargo lane;
no run-state query or contact occurred here.

### Effective homes, source pins and installed toolchain metadata

Selective in-process inspection found neither `CARGO_HOME` nor
`RUSTUP_HOME` explicitly selected; standard home fallbacks are
`/Users/dominik/.cargo` and `/Users/dominik/.rustup`, respectively. Both are
nonsymlink directories, UID 501/mode 0755/device 16777234. The selected Cargo
path from executable-path lookup is `/Users/dominik/.cargo/bin/cargo`; it
was not executed. No `RUSTUP_TOOLCHAIN` override is set; no applicable
directory override appears in Rustup settings. The default setting names
`stable-aarch64-apple-darwin`, while this repository's exact toolchain file
pins 1.98.1. No ancestor/home `.cargo/config` or `.cargo/config.toml` was
found. No registry/source environment override is present. These are
relevant selector/configuration observations, not a dump of environment,
credentials or unrelated Rustup overrides.

Static install metadata exists at
`/Users/dominik/.rustup/toolchains/1.98.1-aarch64-apple-darwin/lib/rustlib/`:

| Metadata file | Bytes | SHA-256 |
| --- | ---: | --- |
| `multirust-channel-manifest.toml` | 1,034,029 | `bad8e4235676d1dbabfa01186c008d8b89e35d4ba7eaadd41a485c4eb1fde5f7` |
| `multirust-config.toml` | 548 | `550fe97786f4afeeb12dcaee4d74dcba219418864b5c31d32cd7bdd465422a26` |
| `components` | 191 | `4e9ff0968db121f4b7f1fddbe52ef0f93498696f219ee4e9f3c5576d6407042a` |

The channel manifest is dated 2026-09-03 and declares Rust/rustc
`1.98.1 (48a229cea 2026-09-01)` and Cargo
`0.99.0 (797e8a9bc 2026-08-05)`. Installed-component metadata includes host
Cargo/rustc/rust-std, rustfmt/clippy and x86_64 Linux rust-std. Cargo/rustc
binary paths exist with executable mode; only stat was used on them. This
does not verify binary content or execute a version/compiler command.

The seven generator-facing source files (script, both locks, both manifests,
notices, toolchain file) plus all 14 override/source-list files are currently
byte-identical in this own WT to both `b5dcfa9` and `b619`: 21 inputs.
Therefore no alignment merge is needed for this proposed command's inputs.
Whole product equivalence or current main/CI state is not claimed. Fresh
identity/clean-source checks remain required immediately before any release.

### Locked full metadata and selected-release cache audit

The server lock contains 432 registry packages plus its local root; the
client lock contains 288 plus its local root. Their union is **470 distinct
registry name/version inputs**, all crates.io, with equal checksums for
overlapping entries. This conservatively covers the target-independent
metadata input set, including development/optional/non-Linux entries that
the 328-row selected release inventory excludes. No manual feature solver
or lock-only replacement for Cargo's actual graph was used.

| Cached registry family | Locked package manifests | Matching locked-version index records | Matching index/lock checksums | Cached `.crate` files |
| --- | ---: | ---: | ---: | ---: |
| `index.crates.io-1949cf8c6b5b557f` (sparse) | 470/470 | 470/470 | 470/470 | 470/470 |
| `index.crates.io-6f17d22bba15001f` (older family) | 0/470 | 267/470 | 267/267 found | 2/470 |

The sparse family's 470 complete TOML manifests have the expected package
name/version; all have `.cargo-ok` extraction markers. Local cached index
package records exist for every locked version and their checksums match
both locks. All 470 compressed archives exist by stat, totaling 85,484,831
logical bytes; their bodies were not read or hash-verified. Index metadata
read totals 29,705,005 bytes across distinct crate records; package manifests
total 1,277,289 bytes. No missing crate/version input was found in this
family. The older family is incomplete; the future environment explicitly
selects **sparse** and must not silently fall back to it.

Target-independent coverage explicitly includes all **39 Windows-related
name/version entries** (Windows target/platform crates, WinAPI and
`anstyle-wincon`), including all locked architecture/version variants.
Examples include `windows-sys` 0.48.0/0.52.0/0.61.2, `windows-targets`
0.48.5/0.52.6, `windows-link` 0.2.1, `windows` 0.41.0 and all listed
`windows_*` architecture variants. Each has its manifest, extraction marker,
matching index/checksum and archive. Android, Apple, WASI and WebAssembly
packages were included in the same full-lock audit rather than omitted
because the notice target is Linux. The downloaded Windows/non-Linux inputs
in the earlier CI log therefore are not identified as missing here.

An initial inspection assumed `.cargo-checksum.json` would accompany each
registry extraction. It is absent in all 470 sparse source directories;
that assumption cannot establish corrupt packages. The subsequent inspection
used actual package identities, `.cargo-ok` presence and cached index/lock
checksums. Four sampled extraction markers contain `{"v":1}`. No cache
repair or fabricated checksum file was attempted. This is metadata evidence,
not a proof that all source/archive bodies are intact or that offline Cargo
resolution has executed successfully.

For the 328 recorded selected-release packages, manifest license expressions
match the recorded inventory. Static inspection of the generator-declared
license inputs found no missing files and matched every recorded text
reference: 554 root license/notice files, 7 bundled extra files, 22 override
uses across 12 unique override files, and the CMS README license section.
The selected manifests total 884,163 bytes; license bytes read total
2,970,699 with repeated override uses, largest file 20,338 bytes. The first
static comparison appended CMS's README section after its override; that
reported one ordering difference. Applying the generator's declared label
sort puts the README before the HTTPS override and matches both existing
IDs. This was an inspection correction, not a generator/product defect or
runtime test. No generator function was imported or invoked.

Cached VCS metadata for Wasmi's four 0.40.0 packages names
`f384f288a149625dc7fc29bcd686f8bca3ee71c0`; wasmparser 0.221.3 names
`1b2c8585415dde926134e57b0d1e6a4c9438abe3`, matching the reviewed override
URLs. License-reference equality supports the expected digest-only refresh,
while actual target/feature selection and host parity still require the held
generator run. No upstream fetch/provider verification is inferred.

### Measured capacity and bounded future invocation

Read-only stat accounting for only these locked sparse-cache inputs:
27,526 source files, 764,895,229 logical bytes / 831,492,096 allocated bytes
(0.774387 GiB); archives use 86,425,600 allocated bytes. Source bodies were
not read by this allocation walk. This is existing occupancy, not proposed
reclamation or guaranteed APFS reclaimable space. There is no cache deletion,
extraction, movement or cleanup in this preparation.

| Actual UTC snapshot | Free bytes | Free GiB |
| --- | ---: | ---: |
| 2026-10-02T18:27:03.296128Z | 23,011,164,160 | 21.430817 |
| 2026-10-02T18:31:04.158504Z | 23,008,624,640 | 21.428452 |

These fresh snapshots supersede neither the earlier incident evidence nor
root's lane hold; the cause of recovered capacity was not investigated.
At the second snapshot, margin above a proposed 11 GiB start is 10.428452
GiB, above a 9 GiB stop is 12.428452 GiB, and above the 8 GiB floor is
13.428452 GiB. This proposes a conservative **fresh minimum 11 GiB start,
9 GiB stop, 8 GiB mandatory floor**, retaining at least 2 GiB start-to-stop
margin. No compilation/extraction/download is expected with the present
cache, but allocation/peak memory/time remain unmeasured and shared free
space can change. Root must remeasure immediately before a serialized
release; this snapshot alone is not runtime authorization.

Proposed **one sequential pair**, from this exact own WT, with no shell
variable expansion, no generator-code/flags change and no compilation/test.
The normal Cargo offline environment applies to every generator child;
explicit homes/toolchain/protocol avoid accidental selection changes. The
private target/jobs/incremental/debug environment preserves existing lane
constraints even though no build subcommand is requested.

First refresh (future only):

```sh
env CARGO_HOME=/Users/dominik/.cargo RUSTUP_HOME=/Users/dominik/.rustup RUSTUP_TOOLCHAIN=1.98.1-aarch64-apple-darwin CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/Users/dominik/orca/projects/riAuth-public-preview-local-workflow-safety-wave27/.target-wave27 CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 python3 scripts/generate-third-party-notices.py
```

Then the same environment and original check, only after successful refresh
and the exact-diff inspection below (future only):

```sh
env CARGO_HOME=/Users/dominik/.cargo RUSTUP_HOME=/Users/dominik/.rustup RUSTUP_TOOLCHAIN=1.98.1-aarch64-apple-darwin CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/Users/dominik/orca/projects/riAuth-public-preview-local-workflow-safety-wave27/.target-wave27 CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 python3 scripts/generate-third-party-notices.py --check
```

This preserves the script's internal four locked Cargo invocations per pass:
server/client `metadata --format-version 1` and `tree --target
x86_64-unknown-linux-gnu --edges normal --prefix none --format {p}`,
with their unchanged manifest/edition options. Two generator passes imply
eight metadata/tree calls. Cargo may query compiler target information as
part of metadata/tree; no independent compiler/version probe, product binary,
build/test/native helper/service is proposed. No Python import of this
generator or substitute hand-renderer is authorized.

Reservation envelope proposed for root review: **300 seconds total, at most
120 seconds for either generator invocation**, finite private exclusive-0600
raw logs (8 MiB cap per invocation), own PID/process-group tracking and
reaping on every exit, disk samples at most 2 seconds apart and stop of only
that exact owned group at 9 GiB. No global kill, cleanup, download, automatic
retry, fallback protocol/toolchain or additional command/filter is included.
Source/cache identity and capacity must be rechecked before launch; any
missing manifest/index/version/archive/license input remains a blocker,
never permission to fetch. No new supervisor/helper/file is written here;
the bounded envelope requires root's later explicit release.

The generator itself computes all content before its final direct file
write. A future timeout/interrupted write or extra diff must be preserved and
reported without commit/reset/manual notice repair or automatic rerun.
After successful refresh, the entire generated notice must equal the
prospective 752,651-byte output / SHA-256
`8a92f38f2a648d08aa21f415f9ad34b966ee7df6bbf6ba89ddd926ef8833efa2`
and differ from the baseline **only** in the existing client digest field.
Any graph/license/text/other-header/lock/source delta: **STOP before commit
and before the check**, report exact difference to root. An empty refresh
diff against the current stale baseline cannot establish correction and
also requires refusal/reporting. On the allowed exact delta, `--check` must
exit 0 and leave the refreshed notices/source/locks unchanged: this is the
required safe **empty second-pass diff**. An exit 0 alone without those
identity checks is insufficient. Historical all-target results are not rerun
or recast as fresh evidence.

### Preparation result and unchanged limits

**Offline preparation is feasible from observed static sparse-cache and
toolchain inputs; no missing crate/license input was identified.** This is
not a completed metadata resolution, generated output, successful check,
cross-host parity result or whole-CI success. Root's metadata/preparation
lane remains occupied/held; no Cargo slot was acquired or consumed.

Actual operations were selective configuration/toolchain/cache metadata
reads, cached manifests and relevant license input/source hashes, cached
package-index metadata inspection, locked-input stat/disk snapshots, Git
source identity and append-only report/Markdown/whitespace checks. No Cargo
metadata/tree/generator/compiler/native version/runtime/build/test,
network/query/install, existing notice/source/workflow/helper/lock/manifest
edit, cache cleanup/deletion/move, contact/new worker/task/WT/shell,
merge/reset/task-status/main/push action occurred. The original report
prefix and capacity report `717cc58` remain unchanged. Root owns release,
generated-diff review, integration and CI interpretation. I10/R05/W02/W05/I02
remain DONE; the independently running `b619` CI has no newly observed
outcome in this preparation.
