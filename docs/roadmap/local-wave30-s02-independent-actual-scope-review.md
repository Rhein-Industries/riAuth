# Wave30 S02 independent actual-result and original-scope review

Review date: 2026-10-03 UTC. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`;
original S02 `fdda2152-73a0-4dce-9e5e-aff4b232a6fd`.
Reservation: `wave30_S02_independent_actual_and_original_scope_review`.
Supporting WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`, existing shell217,
entry `3c8a30f4f16c00fc007919dcec466c3b3220f6db`, initially clean.
Original primary WT `1e336a3d-bb03-4057-a6a3-df2b057b2af3` is unchanged.
This reservation owns only this new report. No source, test, helper, historical
report, task status, main, push or runtime change is made.

## Independent disposition

**Bounded original-scope DONE candidate for root disposition.** No remaining
concrete local defect or unmet requirement was found in the reviewed seam.
The accepted production implementation removes the unnecessary whole-bucket
Group intermediate fetch, and the actual corrected fixture measures that
fetch-row cost improvement under equivalent encryption and authority while
passing complete result, noninterference, writer, snapshot and concurrency
oracles. This is implementation plus reached verification, not report-only
implementation credit. Root alone decides integration and original-row status.

The improvement is precisely **raw rows materialized by an individual native
fetch**: one513-row control fetch becomes five bounded fetches, each at most128
rows and totaling513. The number of fetches increases; total rows and stored
value bytes do not decrease. Paged complete-call p50 is slower in all six
pair/scope comparisons. Scoped p50 is not lower in any pair. No heap, RSS,
allocation-byte, IO, throughput, latency-speedup or universal deployment claim
follows. There is no newly imposed all-platform or all-operation campaign.

## Original acceptance and state provenance

Read the exact UUID row in the project's local task export:
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/current-tasks.json`.
Complete export:243,722B, SHA-256
`b17fca3cbecf381938c7cf7371c1fd46c76dfab7c8816fde9da6accc09eb1dec`.
The later metadata/body reread observed mtime
`2026-10-03T05:30:12.150602+00:00`, with the same bytes as the earlier read.
This is an explicit export observation, not a fresh live project-query result.
The row says `in_progress`, title `[P2] S02 — Index and paginate expensive
operations`, primary WT1e336, prerequisites S01/Q02. No board mutation occurs.

The exact original acceptance remains:

> Reduce unnecessary scans and whole-dataset materialization while preserving consistent results.

The workstream goal is “Remove measured bottlenecks without removing
correctness guarantees.” The completion gate is “Performance improves under
equivalent security settings, and concurrency tests still prove the identity
invariants.” Both editions retain the same identity, authorization, revocation
and credential-protection semantics for shared capabilities. Completion asks
for applicable implementation, tests, documentation and artifact evidence,
actual verification, gaps and external prerequisites; documentation alone is
insufficient. Historical leave-todo/no-launch scheduling is superseded by the
current user reservation, without extending this read-only permission.

Read `CONTRIBUTING.md` and `SECURITY.md` at the run source. No `AGENTS.md` exists
in the worktree, its relevant directory chain or ancestor guidance locations.
The explicit reservation limits broader contributing checks to static report
checks. Desktop preference remains RiWork Cua.ai Driver; no desktop is used.

## Immutable source identities and actual body coverage

| Role | Full Git object / complete file identity |
| --- | --- |
| Original production change | `74c1fb7660791864a68b9caebb7977f30a53e41c` |
| Complete accepted Core | `src/core.rs`:57,913B/1,414 lines; SHA-256 `686e7e732f256e7b435ab69991b71fb26d7467214877d2e0f0c840741446caea` |
| Corrected authored fixture | `ec4316259b3cb04496e5467336ecebee15b92f2c` |
| Complete corrected fixture | `tests/s02_group_listing_paging.rs`:60,276B/1,730 lines; SHA-256 `7360487b4f866f1d524eb8c25a154386d33203f7c4879734bfbd129f9473d404` |
| Accepted fixture port | `8d66b513e9e241aedc5e1d8d56f8084dc95de188` |
| Accepted publication | `e17f723c211b9cba4c8f07257f3e40560325c223` |
| Actual clean run HEAD | `536ab811c3cf396df5ceae6509e109164cec9b2c` |

Read the complete Core1–1414 and corrected fixture1–1730 as source bodies,
not just hashes or function-name counts. Read the complete executed
25,993B/535-line supervisor as data, including its schemas, capture, cleanup,
preflight, grading, persistence and final-clock paths. No reviewed code was
imported or executed in this review.

Independently compared all358 mode/blob/path entries under the supervisor's
selected `src`, `crates`, Cargo files, toolchain, build/config paths,
`tests/common` and the selected fixture. Both actual536 and acceptede17 equal
accepted8d66 exactly. Canonical NUL-delimited `git ls-tree -rz` manifest
SHA-256: `870542b9c27355022ab757e4321b090eae290c9e7092978308f1623e561308aa`.
The fixture bytes match ec431,8d66,536 ande17. This equality does not assert
that this supporting worktree is aligned to those trees or that unrelated
tests, artifacts and current main all passed.

Replacing only the new Core method with its parent's old method reproduces
the entire57,266B old Core, SHA-256
`13da3f54b28a130cd73e5380253248c650f110b4c4bfb563c88382f55cc74304`.
The fixture's449B frozen method is byte-identical to that old method,
SHA-256 `3e3c0d28943e72391d0087a4d0b8b46fe12fad5d1f918d2e751bd05e3eae012c`.
The accepted1,096B method has SHA-256
`1b634db3ff1707a18d874720e3158537fe050ad34f6704db0c8ccdc8f342869e`.
The frozen control dereferences the actual Core and uses its actual principal
and Store; it is neither a copied security implementation nor a result oracle.

Supporting body reads at536, all byte-equal to74c1:

| File | SHA-256 | Read witnesses |
| --- | --- | --- |
| `src/store.rs` | `dfb62f2e482e9e334e148a932c7900607f37297748f0c1f90a8ef65f77babc0f` | `Store::read`1008; point accounting1447; list/scan/range/fetch1586–1698; decode1150; writer/import/index hooks; full snapshot1984 |
| `src/agent.rs` | `4e15ea4f292e294800c5f2aa7c05fef56bac5ded1cb4f865ad8fc35b7e289d9f` | allows91, principal153, public revoke235, active authority252 and parent256 |
| `src/telemetry.rs` | `d23e9a03f2129601a1a37ace5ad80b0473bbc63a6ddb47be75728882fb0e12fd` | complete Sizes109–145 and read point/scan accounting306–337 |
| `src/store/maintenance.rs` | `95c8f28cda073ffec4ea9f7ebe217be113222ec730ba2981b86aa39d1f5d7a70` | PAGE128; complete Group index update288–350, bindings/membership checks and rebuild bounds |
| `src/management.rs` | `c3e0fde68ae8bec0706ac306133997272d8602c0319ad54498bba462d25c3b70` | public revocation395; Group writer644–777; User update1681–1820 |

Also read the ordinary `scan_records` path in `src/store/prepared.rs`220–254:
ordinary reads return fetched records directly; prepared write/range tracking
is a separate contract. These supporting modules were selected-body reads,
with whole-file identities compared; they were not all read in full. The
source-audit report's accepted inventory was read for scope and historical
attribution, without upgrading its other source definitions or old runs into
fresh executed evidence.

The production method authenticates inside one `Store::read`, pages by the
last persisted key, authorizes each stored Group.name with the unchanged
`group.read` permission, and returns the full ordered JSON array. Every page
uses the same transaction and captured principal. Decode/storage errors still
propagate. redb uses one read transaction; PG's source uses read-only
REPEATABLE READ, but PG was not exercised by this invocation. No external
cursor, schema/index migration, route, mutation, receipt or permission change
is introduced. The visible response and a single large Group may still be
large; the raw-row bound is not a per-value byte or whole-operation memory cap.

## Exact permitted actual captures and receipt attribution

Read only the five authorized fixed files under WTe1's own target:
`/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target/`.
Each is regular, nonsymlink, nlink1, UID501, mode0600, with matching size/hash:

| Fixed basename | Bytes | SHA-256 |
| --- | ---: | --- |
| `s02-group-list-owned-string-v2-supervisor.py` | 25,993 | `835e110e354294fbab39704eeb735fdd3381f88a45aaef82526a3bdfb680c8b4` |
| `s02-group-list-owned-string-v2.log` | 75,770 | `740c72eafa5d5e99b16d81a1e051b626dc316f2a8040570d5236474375774ddb` |
| `s02-group-list-owned-string-v2-evidence.json` | 22,908 | `e2425797bb6c92a7c209019d7229d13c036bf99d2a36aade0f5a2f1ec81bedf1` |
| `s02-group-list-owned-string-v2-observations.json` | 72,117 | `2a7d4d5893b4f9d9600f3b49b1350dcc26df1336bd45203f94667d1f4877d7b9` |
| `s02-group-list-owned-string-v2-decision.json` | 681 | `25dc5259ff77cad2a43b3c0e40c816923d55fba105efd535265583e412f94553` |

Read the complete root cleanup receipt at project
`planning/evidence/wave30-s02-lint-corrected-runtime-cleanup-root-review.json`:
1,425B/SHA-256 `1c0368200524cb6e12d559739a0c56d247b8bc8b33908fd92f495e74e09d5887`,
mode0600, timestamp `2026-10-03T05:27:18.591369+00:00`.
It records clean536, sourceec431, actual Cargo0, real consuming status0,
reaped leader44999/group44999 absent, all77 metadata children joined and
root's fresh empty group-members observation, with the lane released.
Its `result_grade_pending_root_review=true` is retained; this new independent
review is not a historical root acceptance mutation.

Also read root's production source-read receipt:
`planning/evidence/wave30-s02-74c1-root-source-read.json`,1,101B,
SHA-256 `cb4937088abfc8d2639a05c9eaed4644652e82b64fe6756efffff7a3ff1dafeb`.
That dated source-only record is distinct from the later successful runtime.

Actual argv, read from the executed supervisor and evidence, was the pinned
Darwin ARM64 Cargo1.98.1 `test --locked --features test-support --test
s02_group_listing_paging
group_listing_paging_preserves_snapshot_authority_and_measures_materialization
-- --exact --ignored --test-threads=1 --nocapture`, private WTe1 target,
offline, jobs1, incremental0, dev/test debug0. This is a default-Platform
encrypted-redb fixture, not an Essentials, PG, HTTP, Linux or full-CI invocation.
No command is run again by this review.

Actual retained outcome: one test passed,0 failed/ignored/measured/filtered,
126.20s libtest time; Cargo exit0, consuming raw status0, WNOWAIT code1/status0,
leader reaped, group absent, pipe EOF/full capture, no termination signals,
no compiler-error code/panic location, no supervisor first failure. Decision
`candidate_pass=true`; captured evidence/observation hashes match its links.
Root supplied that the corrected invocation succeeded. The five files do not
contain an independent consuming wrapper-exit receipt; the final supervisor
clock is later than its persisted/printed pre-final observations.

The complete log retains nonfatal build warnings: three debug-strip attempts
reported rust-objcopy SIGABRT/missing libLLVM, and the riauth link reported
an oversized unwind section. Cargo and the test nevertheless returned0.
These warnings are not erased, turned into a current all-green claim, or
diagnosed/repaired here. Only riauth recompilation was observed; recorded
source/tool/cache pins before and after match, and dependency metadata guards
did not fire.

Recomputed resources:65 disk samples, minimum16,189,808,640B
(15.077934265GiB), maximum sampled gap2.100096334s. Start16,636,358,656B;
post-cleanup16,230,907,904B. Sampled start-to-min change446,550,016B is shared
host available-space movement, not own allocation peak or IO. All observations
stay above9GiB stop/8GiB floor. Evidence elapsed132.437746375s,
decision pre-final132.502263959s, cleanup0.132802667s. Reviewed bounds remain
13GiB start/4GiB planning drain/1800s outer/1790s monitor/10s cleanup reserve,
2s nominal samples/32MiB log cap,300s fixture/10s channel waits. No hard
kernel-IO or future-capacity guarantee follows.

## All139 records independently checked against source

Parsed every body in the closed-record JSON and complete log. The log has176
lines; all139 JSON records equal the observation sequence exactly. Duplicate
JSON keys/nonfinite constants were rejected by the independent data reader.
The supervisor's closed schemas were extracted as AST literal data, without
calling its parser, validator or grade function. All field sets, enums, exact
booleans, nonbool u64 numbers and fixed-length histogram arrays conform.

| Sequence indices, zero-based | Schema suffix | Actual count |
| --- | --- | ---: |
| 0 | telemetry-self-check/v1 | 1 |
| 1–120 | group-list-sample/v2 | 120 |
| 121–122 | ordered-snapshot/v1 | 2 |
| 123 | core-overlap/v1 | 1 |
| 124 | authority-boundaries/v1 | 1 |
| 125–136 | group-list-summary/v2 | 12 |
| 137 | measurement-decision/v2 | 1 |
| 138 | group-list-result/v1 | 1 |

Every sample cell is uniquely identified by pair0–2, control/paged,
scoped/admin, sample0–9. There are exactly12 cells/10 samples each; AB/BA/AB
lane order and scoped-then-admin within each lane match the complete main
body. All summaries have unique corresponding cells. Source lines890–935
time the complete actual list call, retain its result through capture/grading,
and exclude observer rendering, snapshots and value-drop from that interval.
Five warmups per lane/scope precede ten measured calls. Both lanes open
fresh identical closed-seed copies with the same encryption key/configuration
and credentials; only the copied data-directory path is normalized. Real
writer/concurrency work occurs outside these measurement windows.

All60 control records have one unbounded scan/513 rows, zero bounded scans,
histogram `[0,0,0,0,1,1,1,1]`. All60 paged records have five bounded scans/
513 rows, zero unbounded scans, histogram `[0,1,1,5,5,5,5,5]`. In both cases
the opposite histogram is all zeros. Bounds are0/1/16/128/1024/8192/65536/+Inf.
Each paged distribution has all five fetches at most128 rows; together with
sum513 and one fetch at most1, this proves four128-row fetches and one1-row
fetch. Fetches over128 are1 control versus0 paged. The largest fetched-row
batch falls513→128 (75.048733%); this is a row-count calculation, not measured
heap/RSS savings. Total fetched rows remain513, and fetch count rises1→5.

For every scoped sample in every lane/pair:2 point reads and1,024,387 native
point-plus-scan stored-value bytes. For every admin sample:5 point reads and
1,025,375 bytes. All120 samples have zero writer holds/commits and fixed true
encrypted-at-rest attribution. Source Store raw point/scan accounting counts
stored values, excluding key bytes; it is neither disk IO nor decoded/output
allocation size. `Sizes` counts actual fetched raw-vector rows before decode.
The self-check observed five scans/513 rows/the same paged histogram and
passed five strict parser-refusal assertions; no custom allocator is used.

Independently recomputed nearest-rank p50 (fifth sorted value of10), p95
(tenth), and median oversized-fetch count for all12 cells. All stored summaries
match exactly. Complete-call times below are milliseconds; the percentages
are descriptive arithmetic from these samples, without inferential claims.

| Pair/scope | Control p50 | Paged p50 | Control p95 | Paged p95 | Paged p50 increase |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0/scoped | 40.989792 | 43.016042 | 41.434041 | 46.485916 | 4.943304% |
| 0/admin | 45.120750 | 47.825917 | 45.981667 | 48.170459 | 5.995395% |
| 1/scoped | 41.275125 | 42.980917 | 43.292584 | 45.902375 | 4.132736% |
| 1/admin | 45.819458 | 47.776625 | 47.261541 | 48.549625 | 4.271476% |
| 2/scoped | 41.291042 | 43.030750 | 42.394500 | 43.973333 | 4.213282% |
| 2/admin | 45.984250 | 47.733167 | 46.404084 | 48.679667 | 3.803296% |

Measurement decision truthfully records point/value-byte equivalence and
fetch materialization improvement true, scoped p50-lower-in-every-pair false,
latency_grade/heap_claim/rss_claim false. The fixture deliberately asserts
native equivalence and fetched-row cost, not a latency win. The supervisor
grade checks numeric exit/cleanup/libtest completeness and exact record/cell
counts; it delegates semantic/cost assertions to the actual Rust fixture.
The independent checks above do not simply trust its `candidate_pass` label.

## Reached security, correctness and concurrency oracles

The following are actual reached assertions because the complete straight-line
selected test finished successfully and emitted its final result after them.
This differs from merely finding a function definition. Private durable maps,
credentials and fixture directories were not reopened in this review; reached
map equality is established by the executed assertions, pinned source and
successful libtest result rather than reproducing private contents publicly.

| Fixture witnesses at ec431 | Actual supported conclusion |
| --- | --- |
| expected380; checked_read762; main1568–1660 | Independently constructed complete Group JSON for0/1/128/129 and513 rows, exact persisted-key ordering, complete128-member sets, scoped first/middle/last Groups, valid no-match empty array, invalid-token401/invalid_token before Group traversal. Trusted129-row key/name inversion preserves storage order and permissions based on stored name. |
| snapshots357–361; counters677–755; measured calls890–935 | Full durable-map equality on ordinary reads/refusals, no exclusions, zero writer-hold/commit deltas. Same principle applies to warmups/edge reads; measured raw records separately expose all120 read windows. |
| denied_member_write807; malformed_write_refusal827 | Read-only scoped credential receives403/access_denied for public membership mutation with unchanged complete state. Canonical writer rejects malformed members with500/server_error and unchanged state. No persisted corruption, raw malformed import or decoder-error runtime is claimed. |
| expected_audit1021; assert_member_effect1058 | Exact normal membership change, independently computed Group digest and both membership indexes, revision+1, exactly one audit with complete fixed schema/content. Only new UUID/time are obtained from the event and independently bound to canonical key/real writer interval; no broad nondeterministic map exclusion. |
| ordered_interleaving1408, membership record121 | After first page/principal in one Tx, channel-ordered real public membership writer commits a later-page change, is joined, and remaining old snapshot stays complete old JSON. Fresh control and Core calls see the exact committed new JSON. Repeating unchanged membership yields the same whole durable map. |
| assert_revoke_effect1120; ordered_interleaving1408, revoke record122 | Ordered public revocation retires exact token mapping, disables Agent, advances revision and records one exact audit. The existing snapshot keeps its captured result; fresh reads refuse401 before Group scan. Repeat revoke409/conflict preserves full state. Writer is joined. |
| core_overlap1502; record123 | Eight real unhooked Core calls return only the complete pre- or post-membership oracle, writer joined, exact writer durable effect and fresh post-state verified. Actual call-interval overlap=true. This is not proof of injection between two pages of an actual Core call; between_page_injection=false is preserved. |
| authority_boundaries1280; parent oracle1203; record124 | Trusted canonical stored expiry at real now refuses exactly, with enabled Agent and retained token mapping. Same-second observation=true. Public parent disable verifies exact User enabled/epoch change, Agent disable/token retirement, two listing/provisioning generations, revision/audit and every unrelated durable value. Unrelated unparented scoped authority and complete Group bodies remain usable. |
| OwnedWorker319–341; close421–439; final1686–1730 | Three real writer workers and watchdog joined; all owned fixture directories closed/deleted with checked absence before final numeric result. These are fixture assertions; root additionally verified Cargo-group cleanup. No private-directory read or fresh process probe was performed here. |

Expiry is a trusted stored-timestamp compatibility case, not elapsed natural
TTL evidence. Public parent disable also retires the child token; it does not
run the legacy state with an enabled child/token beneath a disabled parent.
The source still checks parent liveness on every principal use, but that
particular compatibility runtime remains unclaimed. The actual authority
record explicitly says natural_ttl_elapsed_claim=false and
legacy_enabled_child_with_disabled_parent_runtime_claim=false. Those limits
are not invented new blockers for this small reader change.

The context-free Core fixture retains normal public writers but does not
exercise HTTP optional/required headers or idempotency receipt replay; those
contracts and the reviewed client-creation receipt-secret/PAM paths are
unchanged source. Existing indexed membership/User pagination/maintenance
mechanisms remain accepted. Held authoritative-Group cutover/mirror designs,
their older benchmarks and unrelated closed rows receive no new credit here.

## Historical failures and review checks

The first nativev1 attempt remains failed: Cargo101, E0277/E0308, zero closed
fixture records, no reached security/cost oracles. Read its immutable public
report appendix at `d2c88749d3149ff39b5bb6f057589b35a9f26c51`; old private
captures were not read. The explicit owned String correction and later
lint-only changes do not retroactively pass that invocation.

The accepted root CI receipt at8d66,
`docs/roadmap/evidence/wave30-s02-ci-lint-source-root-review.json`, preserves
run37096740915/job111128393479 at
`efd01727f2f4d92fe10d721740146e9e40f639cf`: two fixture Clippy lints, tests
skipped. That CI stays failed. Its corrected_runtime=false is a dated
source-review observation, not a denial of the separate later native run.
No raw historical CI log was queried/downloaded and no current Linux/full-CI
pass is inferred.

Actual review checks: full selected source/supervisor/evidence/log bodies;
whole Core inverse and frozen-body equality; accepted/run protected manifest
and fixture identity; strict AST/data schema/type/key/cell/sequence checks;
all120 sample counters/histograms and all12 summaries independently
recomputed; decision/evidence/log hash links;65 resource samples and77 child
metadata records checked; safe-file modes/ownership/identity; static Python
AST parsing only, without running the supervisor; documentation links,
tracked-file hygiene, whitespace and sole-new-report scope checks.

Read-command mistakes were corrected without broadening scope: `686e` was
initially treated as a Git revision although it is a file SHA-256; a guessed
supervisor basename did not exist; an unquoted shell glob matched no paths;
short `8d66` became ambiguous with a tree object. Reads were repeated with
the full immutable commit and exact authorized basename. One combined tool
display truncated; relevant omitted bodies were reread in bounded chunks.
These are static lookup/display errors, not product or runtime outcomes.

The supporting branch remains deliberately unaligned. This review acquired
or released no validation slot, performed no Cargo/compiler/test/native/helper
execution, process/signal probe, network/query/download, browser/Driver,
secret/lab/protocol/archive read, cache deletion, worker contact or new managed
resource. Root's actual prior lane release is attributed to its receipt.

Unmeasured profiles remain explicit limits: Essentials execution, plain redb,
PG/encrypted PG, Linux, HTTP/transport, sparse arbitrary tenants, large legacy
values, natural TTL passage, legacy enabled-child parent compatibility,
between-page scheduling inside actual Core, heap/RSS/IO/throughput and released
artifact performance. None is silently credited; no source-backed requirement
in the original bounded local slice demands a new universal campaign. No next
source/test/runtime correction is proposed. Root retains final independent
disposition, publication and task status authority.
