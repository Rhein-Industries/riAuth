# wave30 A09 complete logical-reader independent source review

2026-10-03, Europe/Vaduz. Project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original A09 task
`506e3979-a590-4af3-8fa8-ee90d3a517f2`; existing Sol4 WT7
`7c85f5ef-3fac-4f72-aaed-08474d7fb454`.
Reservation `wave30_A09_complete_logical_reader_independent_review`.
The only write is this NEW report. Entry is
`b21135a5c804de6ef9308b305d47b5021af71092`; no source alignment, helper
implementation or runtime reservation accompanies this review.

## Decision and one concrete remaining gate blocker

No source correctness blocker was found in the exact reconstructed candidate.
Recommend accepting this narrowly reviewed source proposal for root's separately
reserved implementation/review process. This is not an implementation approval
issued by this worker, runtime release or original A09 completion.

The ONE concrete remaining blocker to accepting this reader/refusal gate is
missing measured reader evidence: this proposal contains no built/exported,
dependency-compatible native reader artifact and no first reader-alone
observation proving exact preservation of config, key and redb before the reader
can be used as an oracle. Build/export/copy/dependency and later complete refused-
open logical comparisons remain ordered prerequisites of a separately authorized
fixture. They are unexecuted requirements, not a discovered product defect.
A source hash or `--no-run` recipe cannot close this blocker.

Original A09 at immutable 7869b964 is the package-supported-architectures row:
produce/test server, client, container and maintenance artifacts for explicitly
supported Linux x86-64/ARM64. Its distribution-switch gate preserves users,
authorization and configuration behavior. The published actionability projection
explicitly leaves official publication to Q08/Q11; I add no release certification
or all-CI gate. That dated row projection is context, not a current task status
update. Root owns interpretation, future source/runtime, integration, publication
and status; previously DONE rows stay closed.

## Exact objects, reconstructed source and actual body coverage

The proposed helper is reconstructed TEXT, not a Git helper blob or actual binary.
The plan commit changes its report; it does not materialize this candidate.
I applied the exact 343-line zero-context diff to the actual baseline helper,
checking each hunk preimage/count. It reconstructed 119193 bytes / 2049 lines,
SHA-256 `1ac9e1342f51decd49da5d6039d4067347825271059fea58cb08cecde887be67`.
The 21274-byte LF-terminated diff hashes to
`bc82bf6446fdd99d0e0726e5f915ee0aa560b56ad4a24c402c61abcc56c35628`.

| Immutable object read | Commit | Git blob / type | Bytes | SHA-256 |
| --- | --- | --- | ---: | --- |
| baseline | `7869b96453c13fdec5af35fd5cf661849d417f36` | `45af8123aa3055e7dffa7c56d6e1eefcd628bb4d` (blob) | 99566 | `c5ede6313bf967ab1ab42429e74fb4ca9fcc03c82bd0da2e4433e2b129412837` |
| plan | `c5515afb096a6bb7499e6c37cc4db46af2372e76` | `9fc4a4e57b46509f58bf8ac779cb88fc751bb1d5` (blob) | 123022 | `96f8f9c1a2ae6da9cbf4b046e26ef355166e329f11b25215e7ce1994a5ceb8e0` |
| workflow | `7869b96453c13fdec5af35fd5cf661849d417f36` | `a396823938a014cdd2c2ad67a08cae19f58c8a8b` (blob) | 4573 | `9af6d1d40543bd9604d7105730b97cec9c665d0f197548e50ba9a87eeb6007d4` |

Plan path: `docs/roadmap/local-wave30-a09-container-offline-refusal-plan.md`.
Baseline path: `scripts/check-local-container-cohort.py`.
Workflow path: `.github/workflows/check-local-container-cohort.yml`.
Both supplied revisions were checked as commit objects; the table's paths resolve
to blobs. Candidate identity derives from the immutable baseline plus archived
diff, rather than from a claimed candidate object or executed artifact.

Body review covered the ENTIRE reconstructed 2049-line helper, including all
unchanged command, transport/archive, source, resource, Docker creation/ownership,
build, tool, fixture, authorization, handoff and cleanup context. I also read
the actual five changed baseline bodies in full, so old versus new semantics
were compared directly. Every one of the 48 unaffected existing Cohort method
spans, including decorators, matched the baseline in bytes and AST. Matching
unaffected bodies were reviewed through the complete reconstructed source;
they were not merely counted by hash.

All thirteen complete new/changed top-level or Cohort bodies below were read,
including nested parser functions. Archive B contains every one exactly once
with only inter-body whitespace left over; it orders them for review rather
than in executable source order. No selective excerpt substitutes for a
new/changed method. The four existing changed methods are `__init__`,
`direct_refusal`, `config_refusals`, `platform_refusal_fixture`; eight new
Cohort methods form the reader adapter; `main` adds the reader build after the
existing image builds.

| Complete body | Candidate lines | Bytes including decorators | SHA-256 |
| --- | --- | ---: | --- |
| __init__ | 195–269 | 4450 | `6cfe6d95c581a71c268fadd502d218c6c5fcfe85a6251510d70c2d81ff2ea26e` |
| build_reader | 1248–1334 | 5625 | `adf830cf339fcd5daba7a54ec5fab56fb201bb4bd557e9b091cced2dfc3c34f7` |
| config_refusals | 1753–1793 | 2708 | `9dd764d69132aaa987d982287653f30859bd92a9457e323456a48c20b3fbb19e` |
| direct_refusal | 1731–1741 | 781 | `43a9459a529e2046f0158d1e162070fe774a1c423cc5335aa1434d89acf58ee7` |
| logical_snapshot | 1416–1497 | 5343 | `c3ceedf1139ca49cf051f4bb4cd7b01a9c33a6784f46584b6456fae298065c81` |
| main | 1997–2045 | 2159 | `16059b85552d8baa87742a12b48b75a7e1adaf103c19f396880996f2ab3ea02c` |
| parse_reader_snapshot | 1366–1414 | 2483 | `3cb922697e6d633096ec0d7ae131c5c7b4a50139efba19050ac420356788412c` |
| platform_refusal_fixture | 1810–1839 | 1975 | `b29f1c687abea32a6965860f3cd25b1963f58cc14fd7aa17e1cede7ba9e230e5` |
| preservation_after | 1519–1556 | 2687 | `280dca0c94b5d40d6ad7771e1b61eaa6fd95595feba0bcbd656aaa18ed15ccd4` |
| preservation_before | 1499–1517 | 1313 | `e5b410652574a7b44d86835d9c2badef9c00ae3f2960732012c7105c2f42aedf` |
| reader_file_identity | 1353–1364 | 893 | `ebdf6356c6f87867bc407cab1bf64c555ba3fb4c1290cb9e6427b4bf1e74bd5b` |
| reader_io | 1346–1351 | 331 | `9c96bb2940c0de624ed61212f09e9b441274a1e6c06b00116da8f076b990151d` |
| reader_offline | 1336–1344 | 588 | `1bf0c002dc653a53f9786772144bc8f21bd0fbbd6c2d68511b973544dd2a8b0a` |

The full unchanged 118-line probe was read at product
`b619fe25269ccc150e473bbcde47cdb3623ef810`:
`tests/edition_transition_store_probe.rs`, 5722 bytes,
SHA-256 `da486cb8cd7c9d6dfcd78da2704e27688b43a8dd40c7fa58e5b4574f678aed32`.
It contains fixture/agreement mutation arms as well as snapshot; the proposed
wrapper invokes ONLY the literal snapshot action. No other arm is invoked,
removed, selectively transplanted or presumed safe for this reader.

The complete derived 35-line Dockerfile and complete 109-line workflow were
read. Literal recipe extraction from the candidate AST, exact serial-prefix
reversal to the pinned 1512-byte product Dockerfile, and stage concatenation
reproduced the entire archived 2866-byte reader Dockerfile:
SHA-256 `ba68e6ccb19b2421b250a75e01fa41844b772a3c892f99777f7d5e375ff8adc7`.
Workflow and probe archives matched actual immutable Git bytes exactly.

| Reader input equal at fixed x86 and ARM product objects | Bytes | SHA-256 | Same Git blob at both objects |
| --- | ---: | --- | --- |
| Cargo.toml | 4020 | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` | `5660d4bb922fcdc5bfe05d7502585980f4720d06` |
| Cargo.lock | 109243 | `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` | `f1b819d47d204d73617b095513f0c6ab6eb8aa4e` |
| rust-toolchain.toml | 86 | `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167` | `c3f67b6771b777215340531caf051bc25cef066c` |
| Dockerfile | 1512 | `458ebb247170c6d4af2ff45b22e5a36e0a5380612140a43448d2ddcebc5d83cb` | `f445b27204c415c4537676986a94023aaaa72f34` |
| .dockerignore | 794 | `4fbd9472316442bdd8b72e268feb1140701e87ac28dd02295c687b6d379eb093` | `85a5cfc8f9d484483c7d53178173905309215fdc` |
| tests/edition_transition_store_probe.rs | 5722 | `da486cb8cd7c9d6dfcd78da2704e27688b43a8dd40c7fa58e5b4574f678aed32` | `cba900666a5b49c1384de31b04889a75e3f3080c` |

Fixed products remain x86
`b619fe25269ccc150e473bbcde47cdb3623ef810` / tree
`a627df2ce21a4d255b8c1914d4f543e32f40f4de`, and ARM
`9a819317efb3a13fa27cd86f884be2be00898fc0` / tree
`1528b61ba463d9262d6252d54174748a176f313b`.
Their source/review/native transport selections remain distinct. Six equal
input-object identities do not assert equal built binaries or measured ARM
reader support. Cargo.toml, Dockerfile, .dockerignore, toolchain and probe bodies
were fully read; Cargo.lock was hashed and compared as an object, not reviewed
as every dependency body.

Selected product dependency bodies were reviewed precisely: crypto::digest
at crypto.rs 62–64 is SHA-256/base64url without padding, matching the parser's
43-character fingerprint grammar; Config::load path/validation span 819–873;
complete Store::inspect 883–1004 and snapshot/snapshot_all 1977–2022;
ownership local check 94–116, read/reopen/lock scratch proof 186–312,
and descriptor options/identity 375–407. Object identities:

| Selected product source | Whole object bytes | Whole object SHA-256 | Body coverage |
| --- | ---: | --- | --- |
| src/crypto.rs | 17949 | `ad220d296dd5aca6278ee9ab42d9efae42e541f1027a578774c6894862ac37e3` | Digest and adjacent AEAD/key-read context, 59–116 |
| src/config.rs | 51307 | `4155154a81c70721c3e195733bf17ca1e416e99e2c2e11a69862632c275e8353` | Selected load/path-resolution span |
| src/store.rs | 82583 | `dfb62f2e482e9e334e148a932c7900607f37297748f0c1f90a8ef65f77babc0f` | Complete inspect and full snapshot bodies |
| src/store/ownership.rs | 19389 | `5b0f3de0004ffd36e22d2ac3874a28ad928aed1b16108f42c42be8f56d54ff57` | Selected local/descriptor/scratch-lock bodies |

These whole-object hashes do not claim full body review of all four source
files. Dependency include searches were followed by selected source reads:
fuzz corpus includes in jose/LDAP are fuzzing-gated and the inspected SAML
fixture includes are unit-test-gated; the proposed feature list adds neither
fuzzing nor a unit-test target. This is a selected source check, not an exhaustive
transitive Rust dependency or linked-library closure proof.

## Native build/export, context, provenance and resources

The new target uses the existing source-pinned native builder/platform after
unchanged product image builds. The reader Dockerfile extends the exact serial
recipe; it does not replace the product image recipe or append a new product
image tag. Main .dockerignore remains its exact 794 bytes: tests are excluded.
The isolated named `reader-test` context contains exactly one hash-checked
probe file, private 0600 under 0700, so the recipe can copy that one test
without widening the main context to tests, credentials or arbitrary local data.
Private runtime root remains disjoint from product context.

The release/locked/default-features-off compile names only the existing
`edition_transition_store_probe` integration target, with
`platform,test-support` and `--no-run`. Jobs1, incremental0 and all
declared debug0 resource overrides remain explicit. The recipe refuses any
preexisting executable probe match or symlink match, then requires exactly
one regular executable match before copying to the reader-only scratch export.
It never runs the probe during the build. Cargo can still compile implicit
binary/development dependencies; I do not relabel this as a library-only build.
OpenSSL development support is a source prerequisite; runtime image retains
libssl3t64. ELF checking verifies the 20-byte architecture header, not dynamic
interpreter/library compatibility. That compatibility must be measured by a
later real selected reader run, with no fallback binary.

The local exporter is required to produce only one host-owned regular,
non-symlink, nlink1 artifact capped at128MiB, then matching selected native ELF.
The reader's actual source/tree/inputs/architecture/features/profile/recipe and
binary digest/size are recorded only on the future successful build path.
The runtime copy is0755 in private0700 root, bound read-only into the owned
UID10001 platform container. The retained0600 `.bin` contains product code
compiled before fixture credentials exist; private snapshots/key copies are
outside evidence. Root already approved the additive product-code .bin/
provenance artifact allowlist in this assignment. No new download/native
transport member/release asset or general evidence allowance is implied.

Existing command bounds and cleanup ownership remain byte-identical:
builder memory8g/3cpu, build1800s, global7200s, fixture1200s,
workflow150min / fixture-step125min / always-upload15min,
start30GiB / stop10GiB / floor8GiB, sampled2s rather than hard quota.
Reader90s and metadata/copy/cmp20s commands use the same private bounded capture,
exact creation identity, group termination/reaping and stopped container checks.
Build output stays private for this new target. Conditional provenance is
not an observed build/export/dependency result.

## Private snapshot transfer, complete equality and protected decisions

| Boundary | Source conclusion and practical limit |
| --- | --- |
| Reader eligibility | app=None; exact owning volume CreatedAt/Mountpoint is rechecked; all tracked containers using either volume must be stopped. This proves the controlled owned fixture scope, not absence of an unseen/unowned daemon adversary. |
| Only snapshot | Reset env-i declares literal snapshot, valid original config, closed private output and unused agreement path; exact ignored test name, one test thread. Store::inspect uses read-only actual redb and separately verifies encryption format; scratch lock proof beside it needs RW directory and removes scratch. This is not a whole-filesystem-write-free claim. |
| Caller output | UID/GID10001 creates unique0700 directory and exclusive noclobber0600 file before the probe. Regular type, nlink1, mode600, positive inode, empty-before/nonempty-after and8MiB bound are checked. Probe fs::write must preserve the source inode; no relaxed ownership or inferred copy inode is accepted. |
| Transfer | Docker cp has no follow-link flag and copies only from the exited owned reader. Fresh host0700 parent identity and regular0600 host-owned nlink1 file are checked. O_NOFOLLOW plus pre/open/post descriptor/path/parent identities and complete bounded read bind parsing. The local verified daemon is trusted under the same existing fixture model; no arbitrary untrusted archive or hostile-race guarantee is added. |
| Full private grammar | Complete JSON document, duplicate-key rejection, bounded integer strings, no floats/constants, max depth32/nodes100000/key4096B/string1MiB/input8MiB. Exact top-level three fields, five metadata keys and four non-bool integer counts; every row fingerprint is checked. Invalid/dirty/missing/copy/parse output fails closed; no partial map or compatibility fallback. |
| Mandatory observer proof FIRST | Every logical_snapshot hashes complete original config/key/redb before and after the reader/copy path and requires exact three-file equality before returning a map. The first successful call is mandatory reader-alone proof; verified observation count increments only after that assertion. Any reader physical difference stops with fixed component booleans, regardless of logical equality. |
| Exact refusal input bytes | preservation_before makes private exclusive copies of original config, key and selected closed candidate. preservation_after compares complete bytes using three cmp results plus original config/key SHA comparisons. No key/config contents enter stdout or evidence. |
| Whole logical preservation | The ENTIRE unfiltered row_hashes map, including its keysets, must equal; canonical full metadata and typed counts must equal. Added/removed/changed counts are diagnostics only; they never replace map equality. No shared(), sampled view, excluded meta/revision/activation/agreement/history row or reduced summary oracle is reused. |
| Physical versus logical | Only refused-open redb FILE equality is recorded separately after observer proof, exact input preservation and complete logical equality. False physical_redb_equal is observed only conditionally; it is neither presumed nor blanket permission. Complete RECORDS logical equality does not inventory every redb auxiliary/index/allocation table or certify all-page ciphertext/layout preservation. |
| Public evidence | At most32 packets publish fixed booleans and finite added/removed/changed counts. No row key, row digest, raw metadata/value/key/query/config contents or arbitrary external error is published. Snapshot/stdout errors remain bounded private, Docker logging is disabled, and main emits fixed Refusal codes or exception class only. Private roots/volumes are removed by existing owned cleanup; only approved public code/provenance remains. |

All three call paths are covered in executable source: direct_refusal retains
code2/okfalse and each required marker before the complete preservation assertion;
config_refusals retains exact one-field candidate generation and specific blocked
PLAN reasons, then complete per-direct and aggregate preservation;
platform_refusal_fixture stops its owning app and applies complete direct plus
aggregate preservation after the unchanged agent/preflight boundary.
Original failure codes remain fixed; this review does not change their meaning
retroactively.

PLAN/handoff/activation protections are retained. Newly added config PLAN and
platform-preflight immediate physical before/after assertions require all three
files unchanged, so a permitted refused-open physical difference cannot excuse
PLAN mutation. Existing plan(), handoff(), maintenance(), fixture_gate(),
snapshot(), all client/HTTP authorization/grant/session/revocation paths and
creation/cleanup wrappers are byte/AST unchanged. E-P-E still requires the exact
read-only PLAN token, explicit activation, original config/key preservation,
online identity/authorization comparisons and all refused-action assertions.
No implicit adoption, permission/header/receipt/key bypass or weaker ordinary
user contract is introduced.

## Actual static results, retained review errors and limits

```json
{
  "candidate_bytes": 119193,
  "candidate_lines": 2049,
  "candidate_sha256": "1ac9e1342f51decd49da5d6039d4067347825271059fea58cb08cecde887be67",
  "diff_bytes": 21274,
  "diff_sha256": "bc82bf6446fdd99d0e0726e5f915ee0aa560b56ad4a24c402c61abcc56c35628",
  "protected_method_count": 48,
  "whole_byte_diff_inverse": true,
  "inverse_diff_sha256": "2690edecc31448ca0e4a55bef305e024162b91207d2c8ec7cb37f79a54855bd8",
  "whole_module_ast_inverse": true,
  "complete_new_changed_body_spans_exact": true,
  "derived_reader_dockerfile_archive_exact": true,
  "derived_reader_dockerfile_bytes": 2866,
  "derived_reader_dockerfile_sha256": "ba68e6ccb19b2421b250a75e01fa41844b772a3c892f99777f7d5e375ff8adc7",
  "workflow_archive_exact": true,
  "probe_archive_exact": true,
  "six_reader_inputs_equal_at_both_product_commits": true,
  "source_parsing_only": true,
  "candidate_function_calls": 0,
  "cases": 0,
  "runtime_commands": 0
}
```

Whole-file forward reconstruction matched the required candidate hash. An
independent zero-context inverse reconstructed the complete99566-byte baseline.
For whole-module AST inversion, I replaced only the four changed existing Cohort
methods with their actual old nodes, removed exactly the eight added methods
and restored main. The resulting entire normalized AST equalled baseline.
All other48 method bodies and ASTs matched exactly. Probe/workflow/derived recipe
archives matched their complete actual source bytes.

One initial static checker exited1: I incorrectly required Archive B's
concatenated bodies to appear in executable source order. The source archive
uses review order and slightly different blank-line separation. A diagnostic
diff exceeded the tool output cap; attempting to decode that truncated diagnostic
as JSON produced a local SyntaxError. Neither event executed source under review.
The corrected check requires all thirteen complete decorated body spans exactly
once, removes each, and accepts only whitespace remainder. It passed, and all
later whole-file/AST/recipe/probe/workflow checks passed. These were reviewer
checker/transport errors, not candidate, product or reader runtime failures.
The first broad report/source-discovery displays were also truncated; they are
not counted as full body coverage. Complete candidate and focused source/body
reads described above supplied that coverage.

The separate original container37078338496 failure remains failed: its recorded
specific issuer refusal passed, then the composite physical oracle failed.
Actual changed component/row/cause is UNKNOWN; I read the plan's sanitized
historical account, not private archives/logs/store data or the root receipt body.
A source-derived possible storage-header mechanism does not explain that run.
Root-reported separate PG37082572962 success remains separate; its receipt and
fixture were not read/run here and are not borrowed for any container reader gate.

No candidate/probe/helper function, case, VM, Node child, compiler, Cargo, Docker,
PG/native/product/provider/CLI/HTTP/network/dispatch/download, service or desktop
ran. No private runtime row, key or archive contents were inspected. Source checks use immutable
Git reads, stdlib parser/data comparisons and hashes only; no candidate import
or code-object execution occurs. No runtime slot or resource was acquired or
released. No source/main/push/status/alignment or other-worker contact occurred.
Only this new report is written. Original gates and all historical failures,
D01 b211 source archive, accepted contracts and closed rows remain unchanged.

## Report-only handoff checks actually performed

The two supplied revision object types were read as commits; the baseline,
plan and workflow file object types were read as blobs. Reconstruction and body
proofs above remain parser/data checks, not candidate calls.

The new report's local Markdown target scan, UTF-8/LF/trailing-newline and
trailing-whitespace checks exited0. It contains zero Markdown links to resolve.
`python3 scripts/check-docs.py` exited1 with the sole returned diagnostic:
`docs/roadmap/local-wave30-d01-user-browser-review.md: missing SimpleNamespace(demo=d`.
That preexisting archived D01 text was not corrected or suppressed. This is a
repository docs-check failure retained separately from this report's checks;
no global docs pass is claimed.

Before staging, tracked changes were empty and the only untracked file was this
reserved report. Entry HEAD remained
`b21135a5c804de6ef9308b305d47b5021af71092`. The unchanged D01 report was
1240563 bytes, SHA-256
`94ffa7a26c0937e6c3347a3edf95d9cd35d3783c8861be1e7d73a04d597521db`.
The new report is regular mode0644. Only this report may be staged/committed;
root receives the immutable commit and final scope/clean receipt separately.
