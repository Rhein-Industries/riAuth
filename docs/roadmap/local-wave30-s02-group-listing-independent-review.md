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
