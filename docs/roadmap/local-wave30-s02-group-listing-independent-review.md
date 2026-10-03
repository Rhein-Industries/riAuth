# Wave30 S02 Group-listing independent source review

## Disposition and reservation

**Hold the exact bundled source before Cargo/runtime and final acceptance.**
The production `Core::list_groups` hunk is source-consistent with its old result,
authorization and snapshot contract. I found one concrete build-policy blocker
in the new fixture: the package forbids unsafe code, but the fixture defines an
unsafe allocator and unsafe probe operations. This is a source-derived blocker,
not an observed compiler error. No allocator, test or measurement ran here.

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; reservation
`wave30_S02_group_listing_independent_source_review`; existing Sol2 worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, own branch
`roadmap/local-revisions-coordination-wave27`, starting clean parent
`74ac03f5695155ac3bcbfa0d3e429b21dd0c6639`. Only this new report is reserved.
Original S02 task `fdda2152-73a0-4dce-9e5e-aff4b232a6fd` and primary
`1e336a3d-bb03-4057-a6a3-df2b057b2af3` remain root-owned and unchanged.

I read the complete 309-line accepted plan at the immutable parent:
`docs/roadmap/local-wave30-s02-index-pagination-plan.md`. Its original outcome
is “Reduce unnecessary scans and whole-dataset materialization while preserving
consistent results”; its gate is performance improvement under equivalent
security settings, with concurrency still proving the identity invariants.
The plan's dated live-board observation is historical, not a fresh board read or
authorization to change status. The chosen seam preserves the full Group response
and held canonical Group format. A source review/report alone cannot finish S02.

`runtime_run_by_this_review: false`; fixture and allocator **UNCOMPILED/UNRUN**.
Remote A09 run37087561409 was identified by root as owning validation; I did not
query it or take/release a local runtime slot. No D01 composition runtime credit
is added, and no closed row is reopened.

## Exact source and full-read scope

Reviewed source: `74c1fb7660791864a68b9caebb7977f30a53e41c`.
Its exact first parent is `cdebc177eb8a0985b772ad810e436ee182c2718b`.
The complete diff has only these two paths:

| Path/body | Exact identity |
| --- | --- |
| New `src/core.rs` | 57,913 bytes / 1,414 LF characters; blob `f206276b9c9e73167f2ac5010e3d2e628b8c0824`; SHA-256 `686e7e732f256e7b435ab69991b71fb26d7467214877d2e0f0c840741446caea`. |
| Parent `src/core.rs` | 57,266 bytes / 1,399 LF; blob `f02efcde5e9604b7251a8171e0aa437dbd8d8ae2`; SHA-256 `13da3f54b28a130cd73e5380253248c650f110b4c4bfb563c88382f55cc74304`. |
| New `tests/s02_group_listing_paging.rs` | 55,835 bytes / 1,641 LF; blob `39228eaa52ad711e6a9033229878dcba8b69d10b`; SHA-256 `b340155ed571b3b0195c40a141239f5337585356e4e7d8582dd5801162500f42`. All 1,641 lines read. |
| Accepted plan at parent | 29,390 bytes / 309 LF; blob `58227d9dc604723a33a8434ea32cf69eb53bee27`; SHA-256 `2d57c0192e48df80b30f63efac6768a2db394af919e15a00d0f1f37879b24570`. All lines read. |
| Frozen method at test lines356–367 | Exact 448 bytes including indentation/body braces; SHA-256 `8c3e2a22db02f3a1d60ef164a88679c9fcfa782a6de29209ec67697ea8bf1c03`. Byte-identical to the parent production method. |
| New method at Core lines554–580 | Exact 1,095 bytes; SHA-256 `2c8da6879cfc9b6c20cbd8287ba2fc50f9c88405a886dac0b342a2904e26e495`. Replacing it with the frozen method restores the entire parent Core blob. |

Numstat is 22 insertions/7 deletions in Core and 1,641 insertions in the new
test: 1,663 added/7 removed overall. Complete test body reads used consecutive
ranges1–280,281–550,551–840,841–1120,1121–1380,1381–1641. This includes the
whole allocator, self-check, frozen control, seed/copies, expectations,
measurements, snapshot oracles, threads, and sole ignored test. Full bodies
were read rather than equated with their hashes.

Relevant complete bodies read at the same source pin included:

- The whole old/new listing method and public Core membership caller; mutation/
  receipt transaction wrappers; session, identity-user and audit/revision bodies.
- The complete `src/agent.rs`, including principal, per-row permissions,
  expiry/enabled/parent checks, create/revoke caller and browser identity path;
  Agent/permission persisted types and view; selected human-grant allows/active
  bodies and shared user/session liveness bodies.
- `Store::read/write`, Tx types, list/forward scan/range fetch, point/scan
  telemetry, encode/decode and full snapshot; the prepared scan adapter's full
  branch establishing ordinary reads do not use cross-transaction preparation.
- The Group model; Group index update/digest/binding/member mapping bodies;
  shared Group member writer, persistence, review guard, audit, revocation and
  relevant record-transition branches. The fixture has no inbound SCIM records,
  so its Group transition returns without inventing downstream changes.
- Store/config encryption initialization, private-directory/key creation, startup
  and cold allocation-cache construction; telemetry histogram/row/byte methods;
  AEAD/key bodies; the complete PostgreSQL policy/pool module and range query.
  API/portal Group route wiring was inspected as source, without HTTP execution.

I did not read every unrelated Core/Store/management/protocol body, the entire
standard-library/dependency implementations, or private runtime artifacts. The
context files hashed below are identity witnesses, not all full-file audits:

| Context | Immutable blob; SHA-256 |
| --- | --- |
| `src/store.rs` | `8e78d889220c9e956fe00a4f7a69b603063d3ebf`; `dfb62f2e482e9e334e148a932c7900607f37297748f0c1f90a8ef65f77babc0f`. |
| `src/store/prepared.rs` | `d7387cd0a5c9dd45e3e32de0b3b1109c9c5b89de`; `882e1b6be7602d735fdf91abeea83a06a97a699ce1b88e353dca6da124bdd845`. |
| `src/store/maintenance.rs` | `6d6999eaba0eacecfc5587dcc1e94dd8a69e5f3e`; `95c8f28cda073ffec4ea9f7ebe217be113222ec730ba2981b86aa39d1f5d7a70`. |
| `src/agent.rs` | `0ab2592fb624eabc7b503f5263636108281c9c5d`; `4e15ea4f292e294800c5f2aa7c05fef56bac5ded1cb4f865ad8fc35b7e289d9f`. |
| `src/management.rs` | `8ff588258fd859949015604518003c5950bcd2ad`; `c3e0fde68ae8bec0706ac306133997272d8602c0319ad54498bba462d25c3b70`. |
| `src/model.rs` | `8daa94fa28ae30efce78afe587a0bd1524d4f297`; `36ffd978fde0935f018895fe212725266c429c8addf6de69f102d658f7766f8a`. |
| `src/telemetry.rs` | `3a6f02efcef7fff5e50065285570901e8616715b`; `d23e9a03f2129601a1a37ace5ad80b0473bbc63a6ddb47be75728882fb0e12fd`. |
| `Cargo.toml` | `5660d4bb922fcdc5bfe05d7502585980f4720d06`; `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8`. |

Sixteen context paths, including these files, identity/assembly, PostgreSQL,
config/crypto and API/portal wiring, are byte-identical between parent and source.
No source alignment/import was needed. `CONTRIBUTING.md` was read; the user's
source-only reservation limits its ordinary broad build instructions.

## F1 — package unsafe prohibition prevents this fixture's normal build

**Blocker.** `Cargo.toml:90–91`, unchanged from the parent, defines
`[lints.rust] unsafe_code = "forbid"`. The new integration-test target is part
of that package. Ignoring a test prevents invocation, not compilation of its
crate or global allocator. `test-support` does not exempt these definitions.

The fixture has `#[global_allocator]` at128, `unsafe impl GlobalAlloc` at133,
unsafe trait-method declarations/bodies134–158, raw allocator/reallocator/
deallocator probe operations251–282, and an unsafe self-check block295–324.
There is no target-specific source exemption or lint-policy change in the exact
two-path diff. An in-crate `allow(unsafe_code)` cannot override `forbid`.
Thus the proposed normal Cargo target cannot compile under the recorded package
policy. This is a direct source/lint conflict; no compiler diagnostic or exit
code is claimed because Cargo/rustc were prohibited in this review.

**Smallest prospective reservation:** isolate only the allocator and raw probe
implementation behind safe interfaces in a dedicated **test-only companion
crate**, then keep the integration target's declaration/windows/checks safe.
A concrete possible boundary is `tests/support/s02_alloc_meter/Cargo.toml` and
`src/lib.rs` below it, a local dev-dependency/lock entry, and the necessary meter/
probe adaptations in this one test. `GlobalAlloc`'s unsafe implementation and
raw-pointer self-check must both leave the package-forbidden target; moving
only the impl leaves the probe blocked. The helper must preserve exact System
forwarding, counters, failure accounting and every existing oracle/window.

That is a proposal, not acquired ownership or an edit. It requires root's
explicit new file/dependency reservation and independent source review. Do not
weaken the production/package prohibition, add a blanket lint cap, replace the
heap evidence with theoretical row arithmetic, or silently run a special
nonstandard build. Root may choose another bounded safe measurement design;
there is no adequate one-line `allow` fix within the current two-path scope.

No additional concrete production/type/security defect was established by the
selected source audit. This does not infer that all remaining type/lint/runtime
checks will pass after F1 is corrected.

## Production traversal and refusal equivalence

The entire method remains one `Store::read` closure. Principal is resolved
before the first Group scan and captured once. redb uses the same read
transaction; PostgreSQL is read-only `REPEATABLE READ` (`store.rs:1008–1040`).
No page opens another transaction or refreshes the actor mid-array.

`Tx::list` already delegates to forward scan with `usize::MAX` (1586–1587).
The new reader uses existing `maintenance::PAGE=128`. Forward seek range starts
at `groups/` or `groups/{actual_previous_key}\0`, and ends at `groups0`.
The storage adapter returns the suffix of the actual stored key; redb uses
ordered ranges with `take(limit)`, and PG orders its bytea primary key with
range bounds/`LIMIT`. The candidate advances from the last **unfiltered storage
key**, not `Group.name`, an allowed name or a member index. Therefore hidden
rows cannot stall the cursor and key/name mismatches do not skip/reorder pages.

Per-row authorization is unchanged: `group.read` on `group/{stored Group.name}`,
with the same `Principal::allows` edition/action/scope semantics. The return
remains a complete `Value::Array` in storage-key order, with all visible Group
names and every BTreeSet member serialized in order. It adds no external cursor,
response envelope, truncation, deduplication, migration or membership-format
change. The API and portal still call that same public Core method.

Empty/short pages stop; a full final page requires the terminal empty scan.
Expected bounded scan count is `count / 128 + 1`, including0 and exact128.
For513 Groups this defines five scans and513 rows; it is not an observed native
result. Scan/decryption/deserialize errors propagate with `?`; no error becomes
EOF or a tolerant skip. A malformed hidden row also errors before permission
filtering. Earlier accumulated JSON is local and discarded on an error, rather
than exposed as a partial response. AEAD binds the full actual storage key.
Unknown Group JSON fields retain the old typed-deserialization behavior.

Agent token hash, enabled/expiry/parent checks (`agent.rs:153–184,255–269`) and
session expiry/revocation/user-epoch checks (`core.rs:1060–1084` and shared
identity) are untouched. Revocation and Group writers retain their normal
transaction, permission, review, audit, revision and receipt rules. The fixture
uses context-free public Core calls, expressly not HTTP/header or browser
coverage; no bypass of a route's required headers is introduced.

The change reduces simultaneous intermediate bucket retention. It still decodes
all Groups, retains the full visible JSON and has no bound on one legacy Group's
member bytes. It does not promise fewer total rows/bytes read, constant heap,
faster PG latency, a released-artifact gain or the held canonical Group cutover.

## Independent correctness and security definitions

| Definition in the new test | Exact source outcome and limits |
| --- | --- |
| Full independent result | `expected`536–550 constructs names/members from fixture constants, not the control or candidate output. Every successful ordinary/warmup/measured read compares the complete Value, never counts alone. |
| Key/name compatibility | 1524–1570 seeds129 storage keys with reversed visible names, expects the exact scoped row and full reversed-name array. Trusted Tx construction is explicit; it is not a claim that a normal Group writer accepts a key/name mismatch or that LDAP binding becomes tolerant. |
| Boundary/permissions | 1492–1523 covers0/1/128/129, administrator, three exact read scopes, a valid empty scope, and invalid token. The513-row pair also verifies empty scope without changing it into blanket403. |
| No read effects | `snapshot`/`same_snapshot`512–518 and checked/refused reads compare the complete durable BTreeMap before/after, including all audit, credential, receipt, revision and metadata rows. No collection/field exclusion or state-hash-only substitution. |
| Writer denial | 930–948 calls the real public member writer with the read-only agent, expects exact403/access_denied and unchanged complete snapshot. |
| Malformed construction | 950–973 attempts a malformed Group through ordinary Tx::put, expects exact500/server_error and rollback of the complete snapshot. Index decoding refuses it before persistence. It does **not** test listing an already corrupt encrypted row or native decryption failure; those are propagation source witnesses only. |
| Ordered member change | 1322–1409 captures Principal and first128-row page in one Tx, then waits for the normal public later-page member removal to commit on a joined worker. Remaining pages retain the full old result; fresh control/paged calls see the exact new result. |
| Ordered authority retirement | Same schedule with normal public Agent revocation: existing read retains captured authority; a fresh request refuses401/invalid_token before any Group scan. Repeat revoke is409/conflict with no durable change. |
| Exact writer snapshots | 1146–1289 independently builds Group/member indexes, digest, revision, token-map retirement and complete audit changes. Only unpredictable new audit id/time come from AFTER, with canonical UUID/key, real start/end bounds and exact event schema/content checked. Entire remaining AFTER snapshot must equal the reconstructed map. |
| Public Core overlap | 1416–1478 performs eight actual public listings against a normal membership writer, accepting only complete independently specified pre/post Values. It reports whether call intervals overlap; `between_page_injection=false`. It does not assert overlap must occur or pretend the ordered helper injected into an actual Core method. |

Expired agent/session, disabled user/parent and disabled-but-still-mapped agent
cases are not explicitly exercised by this new fixture. Current principal source
preserves those refusals; the original plan's broader authority cases must not
be credited as new executions or claimed covered by this fixture's invalid/
revoked cases. No newly weakened authority path was found, and this observation
does not invent a mandatory all-policy or all-provider campaign.

Expected normal membership effects match the source's binding/source-digest,
user/group-member indexes, revision and audit. Seed users have no sessions and
no inbound SCIM bindings; reviewed-membership holders are absent with default
unprotected Group configuration. The full state oracle will fail if startup or
a real writer adds another effect. It does not silently learn missing effects
from AFTER or delete them from comparison.

## Allocator and measurement source audit

The proposed meter delegates original Layout/pointer arguments and returned
pointers unchanged to `System`. Successful alloc/zeroed adds requested bytes;
dealloc subtracts its current layout; realloc adds/subtracts only the size delta,
and a null replacement leaves the original block/accounting live. ProbeBlock
keeps the old pointer/layout until successful resize, so an assertion on a null
result leaves Drop owning the original allocation. Zeroed content and retained
first byte across growth/shrink are explicit definitions.

The nonallocating atomic spin lock serializes account/window updates; System
calls are outside it. Checked LIVE arithmetic latches INVALID without unwind;
INVALID is not reset to erase an earlier accounting error. The self-check plans
64/96-byte blocks, growth256/shrink32, peak352/end0 and exact operation counters
`(2,2,3,3,2)`. Three failure branches call `account(..., false)` directly, with
no dangerous native OOM request. They are synthetic accounting definitions,
not native allocator-failure execution or proof of all System behavior.

Window ownership is const destructor-free TLS; unavailable TLS counts as
foreign. Active-window foreign operations fail `valid_heap`, and measurement
failures must be zero. Begin rejects nested/concurrent windows; finish disables
accounting before rendering/validation/drop and retains the complete returned
JSON through peak/end capture. The returned value is consumed after finish, so
its requested allocation remains live in the intended measured interval.

**Contamination and cache limits:** zero FOREIGN counts would mean no foreign
operation was *accounted while ACTIVE*. It is not proof that no other thread
had a System call spanning a window boundary: System runs outside the lock.
Nor does `end_extra >= 0` prove no pre-window cache allocation was freed; it
only rejects a net drop below baseline. Peak-extra is requested live Rust heap
relative to baseline, not total allocation traffic, physical allocator peak,
native crypto heap or RSS. A faithful future claim must retain these limits and
reject observed contamination/invalid/failing samples, not describe the meter
as an attribution oracle for every byte.

The source places all fixture writer workers outside cost windows and parks the
owned watchdog after startup rendezvous. It starts no router/background jobs
or allocation-metrics scrape; Store's allocation cache is created cold and its
refresh path is explicitly requested, not called by this listing. This is
source reasoning, not a measured cache/thread/quiescence result. redb/page/OS
cache state is not snapshotted, attested or equalized by durable snapshot
comparison. Closed copies and warmups bound the workload but do not establish
cold-cache, cache-capacity or independent-process equality.

Three AB/BA/AB pairs each open two fresh copies of a **closed** encrypted seed.
The same key, complete state, normalized config, authority and revision are
checked without snapshot exclusions; no live Store handle is copied. Five
warmups per lane/scope and ten samples per lane/scope define60 warmups and120
samples, not120 successful results in this audit. Scoped output is exactly
three Groups across first/middle/last pages; admin output is all513 Groups,
each with128 synthetic members.

The frozen lane uses the live Core Store/principal through Deref and the exact
parent method, not a copied authentication implementation. Cost windows include
the complete actual public/frozen call under the same instrumentation. Expected
JSON, durable snapshots, rendering, telemetry extraction and result drop occur
outside the window. Numeric raw observations are emitted and retained in the
preallocated observations Vec before their evidence assertions. No AFTER output
determines the result oracle.

Native point/scan/row/byte counters come from actual Store telemetry definitions.
They separate bounded/unbounded scans and require no writer holds/commits.
`bytes` sums materialized stored **value bytes** from scan/point reads, not key
bytes, decoded heap or physical IO. For each pair/scope, all samples must retain
equivalent point counts and byte totals; row/call assertions check complete
traversal. These are instrumentation/assertion definitions, not measured values.

Every raw sample and twelve lane/scope/pair summaries precede the allocation
cost decision. Nearest-rank p50/p95 of ten instrumented durations is defined;
scoped median peak-extra must improve in all three pairs. Admin allocation and
latency remain reported even if regressed; there is no timing ratio gate and no
statistical, uninstrumented-latency or throughput claim. The result does not
publish a raw seed/credential/directory/member record; fixed labels and numeric
metrics only. stdout persistence/privacy/caps belong to the future outer owner;
this fixture does not itself fsync a public receipt before every failure.

## Threads, clocks and cleanup

The single ignored test creates a real-clock300-second Budget. The parked owned
watchdog requests process exit124 at expiry; cooperative checks surround fixture
work. Channel receives have bounded waits of at most10 seconds; rendezvous
sends pair with those receivers. Public writer workers are owned, joined on
success and joined by Drop on unwind rather than silently detached. The ordered
reader waits for the writer's commit receipt before continuing its same Tx.
No sleep, test clock, retry increase, raw admission deletion or seal bypass
appears in the fixture.

Each success path drops Core/credentials before explicitly closing its owned
TempDir and checking absence. The shared key's directory lives until every
copied Store is closed. All declared writer workers and the watchdog are joined
before the final finite receipt. That receipt precedes the final budget check;
a late stdout return cannot turn the test into a pass.

These are guards, not executed cleanup evidence. Filesystem IO/join can stall;
process exit bypasses TempDir Drop, and unwind cleanup is best effort where an
explicit close was not reached. The watchdog is joined before final stdout,
so the separate outer deadline remains necessary for a stalled final write.
The accepted plan already requires owned process-group supervision, bounded
private captures, first-failure retention and cleanup of only the owned fixture
root. No such supervisor was invoked or newly implemented by this source commit
or review. A normal passing fixture must not be credited on a watchdog/timeout
failure with unknown residual files/threads.

## Actual static checks, errors and residual gate

Actual checks: immutable Git parent/tree/diff/body reads; complete test/plan
reads; full-blob SHA/identity checks; exact frozen-body and whole-Core byte
inverse; sixteen context-file equalities; standard-library `tomllib` parse of
Cargo policy; exact sole ignored-test and120/60 declaration arithmetic; source
searches; changed-object whitespace check; own Markdown/scope/clean checks.
The source extractor matches braces/quoted strings only for the two listing
methods. It is not a full Rust AST/type/lint parser. No Rust syntax/compiler,
allocator or candidate function was executed.

A large combined context output was truncated; required bounded Store scan,
maintenance and other relevant bodies were reread. Two exploratory line ranges
exceeded a short file's end and printed no lines; the PostgreSQL module and
config helper locations were subsequently read at their actual bounds. These
are static read/output limitations, not product failures. No compiler error,
Cargo exit, elapsed native sample, p50/p95 result, allocation reduction or CI
pass is attributed to this review. Parent `python3 scripts/check-docs.py`
passed, and the immutable source diff passed `git diff --check`.

The one reachable blocking correction is F1's allocator/probe build boundary,
requiring root reservation before code changes or Cargo. After that source
review, root may separately release only the planned named fixture under the
accepted private target/jobs1/incremental0/debug0/capacity and outer supervision
contract. Actual valid native cost reduction and successful exact security/
concurrency outcomes remain required by the original S02 gate. No broad suite,
new host/provider requirement or report-only completion is proposed.

Historical S01/Q02/PG/Q09/S04/reconciliation evidence in the accepted plan retains
its precise old source/run/fixture attribution. It is not new Group-listing
allocation evidence or a build of74c1fb. All actual earlier failures and limits
remain unchanged. No Cargo/rustc/test/benchmark/allocator/candidate execution,
artifact/private-file read, native/provider/HTTP/service/browser/Driver/network
operation, deletion, source edit, merge/alignment, worker contact, new worker/
task/worktree/shell, board/status mutation, main edit or push occurred here.
Only this report is committed; root owns integration/publication/disposition.
Receipt-secret, route headers, PAM, review/removal/audit, held Group and shared
nonrenewed60-second admission/paused-IO contracts remain protected.

## wave30_S02_safe_native_materialization_independent_review

### Dated disposition and exact reservation

2026-10-03, project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, existing Sol2
worktree `ed9ac424-59f4-4520-905b-919aea3521eb`, clean starting parent
`4dcf8773e3989809e026b1fae7d235ff3eedc36e`. This reservation appends only this
report. Its entire previous 25,015-byte / 346-LF prefix is preserved, SHA-256
`6cc6952f2a670386b4f41a713049366494b083326201ed56c582da640c0f96cb`.

**Recommend source acceptance of the exact safe design, with runtime still
HELD.** I found no concrete new compile-policy, type-use, authorization,
snapshot or runtime-oracle blocker by source inspection. The design removes
the allocator/probe conflict without changing `unsafe_code = "forbid"`, any
manifest, dependency, production method or security predicate. The previous
F1 remains accurate for the committed unsafe fixture at74c1fb; this design has
not replaced that source file or compiled anything. Source acceptance here
means accepting the proposed replacement for a separately reserved next step.

The measurable quantity is **the number of raw records in each fetched row
batch**, observed by existing native storage telemetry. It can establish a
bounded materialization improvement with complete results and equivalent
authority if the named fixture actually passes. It does not establish live
heap, peak decoded/output memory, RSS, fewer total rows/bytes/scans, throughput
or a latency improvement. The design explicitly retains latency results and a
false-capable comparison, and does not grade latency as a pass requirement.
No S02 completion or board change follows from this review.

I reread the original S02 outcome, goal and completion gate in the accepted
parent plan: reduce unnecessary scans and whole-dataset materialization while
preserving consistent results; remove measured bottlenecks without removing
correctness guarantees; improve performance under equivalent security
settings while concurrency still proves identity invariants. The dated task
observation there is historical, not a fresh board query. Root has accepted
assessment of the narrower fetched-row-batch measure without live-heap
inference. The actual measurement and concurrency/security outcomes remain
unverified; the original goal is not replaced with a report-only gate or a new
mandatory broad campaign.

`runtime_run_by_this_review: false`; proposed fixture **UNMATERIALIZED,
UNCOMPILED, UNRUN**. No candidate function, telemetry self-check, allocator,
test or benchmark executed. No runtime slot was acquired, released or queried.

### Complete design reads and immutable reconstruction

Reviewed DESIGN commit `02b1e38bc2919ccb7c99711bcb304f3f050e9ba8`, whose sole
changed path is `docs/roadmap/local-wave30-s02-index-pagination-plan.md`.
Its parent is `d6b5545ba1d7827fadb098846da7f2bfdfa02b10`; the whole prior
45,580-byte report is its exact prefix, SHA-256
`eadea67eb08175ba194b0b0621093fe4947bba45f89786bb3c48442a5d61e503`.
The design appends 2,880 lines and does not edit its actual Core, fixture,
build inputs or earlier report bytes.

| Object reconstructed in memory | Exact identity |
| --- | --- |
| Complete design report | 159,464 bytes / 3,445 LF; blob `28c9ce5b506be80873c402e65dbfc594f8bf7e97`; SHA-256 `1fc39d4b93a92b82bc4269e8506c7d2eda8fc1a0a4c8129350bb63f77fc69e7a`. |
| Complete safe Rust fence, report783–2497 | 60,049 bytes / 1,715 LF; SHA-256 `a93636a21a4a5cae53b2a5b844e8e5534e10fc0b1941cc43927f75b720cf63d6`. |
| Complete zero-context diff fence, report2503–3392 | 36,823 bytes / 890 LF / 42 hunks; SHA-256 `b124564d9e35a0bec61cc66852baccac9daac954f42c0bbeb99714101a756135`. |
| Original committed fixture at74c1fb | 55,835 bytes / 1,641 LF; SHA-256 `b340155ed571b3b0195c40a141239f5337585356e4e7d8582dd5801162500f42`. |
| Unchanged production Core at74c1fb and the design commit | 57,913 bytes / 1,414 LF; SHA-256 `686e7e732f256e7b435ab69991b71fb26d7467214877d2e0f0c840741446caea`. |

Full body reading covered every safe-fixture line in consecutive ranges
1–290,291–580,581–870,871–1160,1161–1450,1451–1715. I read all 890 diff lines,
the new prose567–781 and static/held-command tail3395–3445, as well as the
earlier implementation appendix311–565 and original acceptance. The original
1,641-line fixture and 309-line accepted plan were fully read for the preserved
earlier review. A blob/hash lookup is not counted as a body read.

A reviewer-owned, data-only unified-diff parser checked each hunk's old/new
line counts and literal removed lines. Applying all42 hunks in memory restores
the exact safe fence; reversing all42 restores the entire original55,835-byte
fixture. No candidate file was written. Whole-file equality is stronger than
accepting a few intended hunks or trusting the prose.

The whole frozen method is still the exact parent production body, 448 bytes
without its trailing LF / SHA-256
`8c3e2a22db02f3a1d60ef164a88679c9fcfa782a6de29209ec67697ea8bf1c03`.
Including that LF gives449 bytes /
`3e3c0d28943e72391d0087a4d0b8b46fe12fad5d1f918d2e751bd05e3eae012c`.
The original fixture, safe fence and parent Core match in both representations.
Replacing the complete paging method with that frozen body restores the entire
57,266-byte parent Core, preserving the earlier whole-Core inverse proof.

Byte comparisons independently confirmed27 unchanged top-level functions:
`require`, `core_ok`, `snapshot`, `same_snapshot`, `token`, the three name/id
constructors, `expected`, `close_dir`, `private_dir`, `fixture_root`, `populate`,
`checked_read`, `refused_read`, both denied/malformed writer checks,
`percentile`, `source_digest`, `expected_audit`, `admin_id`, `advance_revision`,
the member/revoke full-state oracles, `finish_snapshot`, `ordered_interleaving`
and `core_overlap`. Complete larger preserved segments are:

| Segment | Bytes / SHA-256 |
| --- | --- |
| Frozen struct/implementation | 605 / `eadb41e53019975d5e13d63fbb4d172f06a5422ab8455f7efcb63394951496d0`. |
| Lane declaration/implementation | 455 / `99bcb7e310d636dc924000bb154469d1d372820eb32998543c94a0559951b72d`. |
| Budget/watchdog/owned worker/helpers through before User id constructor | 4,218 / `f5c4feb81f8d159fb00d2125bc9962fa0f846bf8b6e1a12e6399e9c90ff676d0`. |
| Credentials/Fixture/Seed through before populate | 7,547 / `b12511e4d500fb3f5cf211f3872d14619068ea5654340c73dda9b2f3a06e6a2c`. |

These are literal source comparisons, not a Rust AST or compiler result. The
design's actual `src/core.rs`, `src/store.rs`, `src/telemetry.rs`, `Cargo.toml`,
`Cargo.lock`, `rust-toolchain.toml` and original test all match74c1fb. The
complete safe fence contains no `unsafe`, global allocator, raw probe,
HeapWindow/HeapSample, allow override or RUSTFLAGS token. `tomllib` confirms
the unchanged package forbid policy; exactly one test and one ignore attribute
remain. No allocator workaround is added through a helper, dependency or new
crate; existing dependency internals were not newly audited.

### Native telemetry contract and complete data comparisons

I reread `Sizes`, `render_buckets` and `Reads` in immutable
`src/telemetry.rs`, plus the complete forward scan, `raw_scan_base` and
`raw_scan_fetch` bodies in `src/store.rs`. The existing ROWS bounds are exactly
`[0,1,16,128,1024,8192,65536]`; render with the fixed name and empty labels
produces seven finite cumulative buckets, +Inf, count and sum:10 lines.
Observe increments count, total rows and all applicable cumulative buckets.
`Reads::scan` selects bounded versus unbounded by the actual requested limit.
`raw_scan_base` records the **actual fetched vector length** and stored value
bytes after the backend fetch and before typed decryption/decoding. Redb's
forward range iterator actually takes the requested limit; `Tx::list` supplies
`usize::MAX`, while the production paging method supplies128. No observer
estimates this population from the returned authorized JSON or a timer.

The new parser, safe-fixture45–144, is source-consistent with that public API:
4096-byte/10-line bounds; exact fixed names and eight unique bucket slots;
canonical u64 text; duplicate/missing/unknown fields refused; monotone
cumulative buckets; +Inf=count; render count/sum checked against the native
accessors. Delta uses checked subtraction for every bucket, count, row total
and other native counter. It does not accept a reset as apparent improvement.
Public `Sizes::render/observe/count/sum` signatures and the new Rust type uses
match by source; no forbidden private field access or new Debug requirement
was introduced. This is not a compiled typing claim.

The proposed self-check145–187 uses a separate safe `Sizes` object. Its literal
five observations `[128,128,128,128,1]` require count5/sum513 and cumulative
`[0,1,1,5,5,5,5,5]`. Its five parser negatives are duplicate bucket,
non-number, noncanonical number, inconsistent count and missing count. None
executed here. Reviewer arithmetic on declared data, independently of the
candidate functions, gave the following complete expected populations:

| Groups | Frozen fetched sizes / cumulative buckets | Paged fetched sizes / cumulative buckets |
| --- | --- | --- |
| 0 | `[0]` / `[1,1,1,1,1,1,1,1]` | `[0]` / `[1,1,1,1,1,1,1,1]` |
| 1 | `[1]` / `[0,1,1,1,1,1,1,1]` | `[1]` / `[0,1,1,1,1,1,1,1]` |
| 128 | `[128]` / `[0,0,0,1,1,1,1,1]` | `[128,0]` / `[1,1,1,2,2,2,2,2]` |
| 129 | `[129]` / `[0,0,0,0,1,1,1,1]` | `[128,1]` / `[0,1,1,2,2,2,2,2]` |
| 513 | `[513]` / `[0,0,0,0,1,1,1,1]` | `[128,128,128,128,1]` / `[0,1,1,5,5,5,5,5]` |

The +Inf slot is the last entry. Empty final fetches at exact page multiples
are real scan observations, not dropped samples. In `Counters::assert_read`
712–754, count/sum and all eight frequency deltas must equal these expected
data, and the opposite scan family must be empty. Every reader must have zero
writer holds/commits. For513 rows, native count5/sum513, all five at most128,
exactly one at most1 and none zero also force four128-row batches and one1-row
batch; the sum prevents a different smaller population being credited.

The measured decision941–1004 checks every sample's actual frequencies, not
only the design's arithmetic: frozen one fetch above128 with513 total rows;
paged zero fetches above128, five fetches and the same513 total rows. Native
point counts and point-plus-scan stored value bytes must match across both
lanes and every sample in each scope/pair. Stored value-byte equality excludes
backend page IO, keys, allocation overhead, network bytes and total resident
memory; none is asserted to improve. Paged scan count deliberately rises1→5.

This can support **bounded raw-fetch row materialization** on the declared
local encrypted-redb dataset, with the maximum fetched record population
513→128 if actually observed. Source shows that this vector feeds one decoded
page, but the histogram is not a live-heap instrument. A large single Group is
still large; the full admin output retains all513 Groups. No row-count bound
is presented as a byte cap, peak decoded/output allocation or RSS proof.

### Comparison, results and latency preservation

The original encrypted closed seed, same agreed configuration/credentials,
full initial snapshot and startup settlement remain byte-identical. Each lane
gets its own copied closed database; Core/Store handles are never copied.
The shared encryption key stays live until all copies close. Only data_dir is
normalized when checking complete configuration equivalence. Both controls
use the live principal and Store; no authority/scan substitute was copied.

The sole fixture still declares three AB/BA/AB pairs, two scopes, two lanes,
five warmups and ten retained measurements per lane/scope:60 warmups and120
samples. This arithmetic is declaration evidence, not120 actual observations.
`measure`888–933 times the complete list call with Instant; returned JSON
stays alive through the native-after observation. Render/parser/counter
snapshots, equality checks, output, summaries and result drop are outside that
timer. Actual record and equality oracles remain outside the timer without
being omitted. Native telemetry inside the production fetch remains present
in both lanes. No concurrent writer or shared-Store background task is started
in the measured copies; relaxed histogram snapshots require that quiet
condition, and inconsistent/backwards observations fail instead of being
normalized. The watchdog does not access a Store.

Each numeric sample precedes grading, and observations are retained in the
preallocated120-entry vector. Twelve pair/scope/lane summaries retain ten
samples each, nearest-rank p50/p95 elapsed nanoseconds and median native
fetches above128. The separate scoped-p50 comparison can be false; all raw
timings and both scopes remain visible. `latency_grade:false`, `heap_claim:false`
and `rss_claim:false` are truthful. Materialization acceptance does not turn a
latency regression into speed improvement or statistical/general throughput
evidence. If this narrower cost improves but latency worsens, root must retain
both outcomes when interpreting the original gate.

### Expiry and normal parent disable: exact durable effects

I read the full added authority code1160–1361 and checked its callers and
effects against immutable principal/Agent models; normal create/update_user
and management writer; shared identity prepare/transition; User generations;
credential/audit redaction; agent retirement; logout/windows/SSF/downstream
branches; SCIM transition and workflow account-run sealing. The following
context identities supplement the full-body versus selected-body distinction:

| Immutable context at74c1fb, equal at the design commit | Git blob / SHA-256 |
| --- | --- |
| `src/identity/agent_credentials.rs` | `7dff935aa86389fbaee000246e7da24e7185e75b` / `00b5a94006008c9133663b2fe19869fa9b14904f13363507bce7fffe0103df95`. |
| `src/identity/logout_queue.rs` | `83678e6b32a131612d2e9ef1cad479fd22afc7a1` / `b20fd1c606bc52666383feb8d294702e3ab67dfca2db8fe6b8e98dbce619beb1`. |
| `src/identity/signals.rs` | `d151eef2409f29579a202f2269c6aa2cd62e3b0e` / `c658aa600257eb816122654e30d70167830cc394446debc4309ab721848fa8f9`. |
| `src/identity/downstream.rs` | `79640cb1cec0a0ff6b9b7a922dcb996d735da026` / `76ea7d3ed5151f6184dabc2a49e5028153cbb9a2e19418a11fa71c2a7b5d129b`. |
| `src/identity/windows_credentials.rs` | `63607f30471c2ac64f68fb3ad71187a9448ed4a2` / `999518427de03cf6223c02911473d466e799baaf384521f944c27733a447fc88`. |
| `src/assembly.rs` | `14f11d06e424cbb022a996d7b52205ad555b2742` / `eedcee9a4c6197cebefd6badfd150d89ab292bfbfbe8bb1b4034cae2a2eb4f3a`. |
| `src/assembly/scim_runtime.rs` | `cd5cf20402e04719d3e6dd6e0d61a87b5f990aee` / `9cd66862c726c11db78a44a714a1b8ae0c9a1ef2012e82f1a802718170a43c55`. |
| `src/workflow/executor/version.rs` | `4c203dd5d6ab59a8dbf0567eb34ee55669a1e219` / `23cdda785d1957165b1e9f7394ffceb8752ace5ea43403a79c9335817ddd3697`. |

Whole Agent-credential, logout-queue, signals, windows-credential and shared
identity modules were read. For the larger management/Store/assembly/SCIM/
downstream/workflow files, the named relevant complete bodies/branches were
read; their full-file hashes do not mean every unrelated body was audited.

Expiry: public creation and both positive reader results precede a declared
trusted canonical `Tx::put`, setting only expires_at to captured real now.
No agent index/transition mutates other durable values for that write. The
expected whole map retains enabled/hash/token mapping and every other record;
there is no audit/revision omission because this trusted Store write does not
call the Core audit writer. Both readers then require exact401 invalid_token,
zero Group scans/writes and full-map equality. The live predicate uses
`expires_at > now`; the test checks now>=boundary and reports whether it
observed the same second. This is not natural TTL elapse, a clock override or
a promise that wall time cannot jump backwards between reads.

Parent: public create resolves the exact enabled non-admin parent by username;
the stored parent binding is the stable User id. Positive scoped reads precede
normal public update_user with enabled=false. Management increments epoch;
shared prepare_record preserves at least old+1. Under this exact patch, no
second epoch bump is requested. User index maintenance increments provisioning
and listing generations once each. The identity transition deletes any grants,
retires each parent-owned token, disables the child, and considers windows,
logout, SSF and downstream effects in the same writer. The seed creates none
of those target User's grants/devices/RP sessions/links/jobs/streams; the SCIM
record set and account-run index are also absent. Their inspected empty paths
introduce no additional durable record. The administrator's own session and
all Groups remain unchanged. Group membership is not removed on User disable.

The expected entire map1205–1273 therefore includes exactly User enabled=false
and epoch+1, child enabled=false, exact child token-map deletion, both generation
increments, revision+1, and one audit. Audit changes sort Agent before User.
The expected UserView uses the fixed public projection, redacts
password_available, and stamps unchanged password/pairwise_seed credentials;
the factor-free seed has no TOTP/recovery material. Agent credentials are
redacted with unchanged token hash, and user.update has no agent-parent audit
detail. Audit UUID/time alone come from the observed added event, with the
existing canonical key/schema/real-interval checks. All other expected values
are constructed from the prior state and intended effects; no collection or
record is excluded and unexpected effects fail the whole map.

Both fresh child requests must refuse401 before scans with no durable effects;
the unrelated unparented scoped credential must still return the entire exact
three-Group result in both lanes. This tests **normal atomic retirement**. Its
refusal occurs at the deleted mapping and does not isolate a legacy enabled,
unexpired child whose disabled/missing parent survives beside it. That fallback
remains source-reviewed via parent_active, not claimed as executed coverage.
No reinsertion of a retired token or weakening of writers is proposed.

### Preserved security, interleaving, guards and remaining gate

The exact frozen method still uses live `Core::principal` in the same Store
read transaction as the scan. The paged body still traverses every ordered
storage key, authorizes each stored Group.name and emits the full ordered
Value array. No early authorized-count stop, rank/name cursor, global Group
format redesign, decode-error suppression or new writer is introduced. The
key/name mismatch case and complete expected arrays remain unchanged. Each
page uses the original Tx; malformed/decryption errors propagate. The existing
malformed writer check proves transactional decode refusal/rollback, not
execution against an already corrupted persisted database.

The unchanged ordered interleavings commit normal membership removal or agent
revocation after the first page of a held read Tx. Same-Tx later pages and its
captured principal must return the entire old result; the exact writer/audit/
revision/index snapshot and fresh request then prove the new state. Retried
membership removal has no effects, repeated revocation conflicts, and all
fresh authority refusals are exact. The separate public Core overlap case
retains eight full old-or-new comparisons and an honest interval-overlap
boolean. It does not guarantee between-page injection in the unhooked public
method; the ordered lower-level proof is separately identified. No PostgreSQL
runtime or remote peer inference is made.

All original budgets, bounded channel waits, joined owned writers, watchdog,
private target identity, closed seed copies and explicit cleanup bodies are
byte-preserved. One extra isolated authority copy makes17 proposed success
directories, none created here. All copies close before the shared key's
directory; all owned workers are joined before the final finite receipt/check.
The300s watchdog/cooperative checks still need the separately reserved outer
supervisor for stalled IO/stdout and exit/unwind cleanup. These are source
guards, not executed cleanup, capacity or hard kernel IO guarantees. There is
no test clock, sleep, retry increase, raw admission deletion or meter bypass.

Actual checks in this review were immutable body/line/identity reads, full
42-hunk forward/reverse data reconstruction, protected-body and whole-Core
inverse comparisons, unchanged context/build-input comparisons, declared
histogram/case/sample arithmetic, tomllib policy inspection, source searches,
documentation/layout and Git whitespace/scope/prefix checks. The original
package source was never passed to a compiler/parser/typecheck. Function
comparisons use textual declaration/closing-line boundaries, not a full Rust
AST. No candidate or archived checker was invoked.

Static-check limitations/corrections are retained: exploratory Git reads of
nonexistent `src/redaction.rs` and `src/assembly/scim.rs` returned path errors;
actual redaction in Store and `src/assembly/scim_runtime.rs` were then read.
A combined output was truncated; the affected management writer ranges were
reread separately in full. One reviewer hash assertion mistakenly applied the
old448-byte frozen identity to a449-byte extraction including its trailing LF;
it failed before data arithmetic. Removing only that delimiter from the hash
boundary restored the exact prior448-byte identity, and the complete equality/
inverse/data checks passed. No candidate bytes changed, and none of these
static preparation mistakes is a compiler, telemetry, oracle or runtime failure.

No new concrete correction hunk is requested. Root may accept the exact safe
design, reserve its source materialization, and only then separately release
the already proposed single named ignored fixture with the unchanged forbid,
private target/jobs1/incremental0/debug0/free-space/outer-supervision contracts.
This review releases no command. Actual compilation,120 valid native samples,
bounded-cost reduction, full result/state/authority/concurrency checks,
latency values and owned cleanup remain required evidence, not inferred passes.
The narrower observed fetched-row population can supply this bounded
materialization facet of the original performance gate; root owns final
interpretation using the actual quantitative and security results. It cannot
supply the superseded live-allocation metric or erase a reported regression.

All earlier actual failures, historical S01/Q02/PG/Q09/S04 results, this report's
original F1 and precise limits remain unchanged. No artifact/private-evidence
read, Cargo/rustc/rustfmt/test/benchmark, native/allocator/candidate function,
HTTP/provider/service/Driver/browser/network/query/cleanup operation, source
or manifest edit, merge/alignment, new worker/task/worktree/shell, worker
contact, board/status/main edit or push occurred. Only this report append is
committed. Receipt-secret, optional/required route headers, PAM, live authority,
review/receipt/removal/audit, held Group, shared nonrenewed60-second admissions
and paused-IO limitations remain unchanged; root owns integration and status.
