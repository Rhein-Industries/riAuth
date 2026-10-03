# Wave30 S02 index and pagination source audit

## Original row, reservation and recommendation

Project **891e7443-8dac-4c1b-897f-9e53cb59c7ee**, original S02 task
**fdda2152-73a0-4dce-9e5e-aff4b232a6fd**. Supporting worktree
**e1b4399a-8c0d-46b8-880c-a71a4ebf53e7**, branch
`roadmap/local-extension-isolation-wave27`; starting clean HEAD
`55932d0e68df5caf14469cfeb9280f4f1d673367`. Reservation:
`wave30_S02_initial_source_audit`. The original primary worktree remains
`1e336a3d-bb03-4057-a6a3-df2b057b2af3`.

Read the live project task list on 2026-10-03, without updating it:

| Row | Exact task UUID | Observed status | Original outcome |
| --- | --- | --- | --- |
| S02 | `fdda2152-73a0-4dce-9e5e-aff4b232a6fd` | `todo` | “Reduce unnecessary scans and whole-dataset materialization while preserving consistent results.” |
| S01 prerequisite | `7d464105-7720-491a-b382-357500344e51` | `done` | “Instrument locks, connection pools, collection scans, authentication work, and background-job interference.” |
| Q02 prerequisite | `0d69659e-c4e6-4c6a-90ae-07dec81df6eb` | `done` | “Cover credentials, sessions, authorization, plan/apply, retries, permissions, secrets, and audit across shared implementations.” |

S02's workstream goal is **“Remove measured bottlenecks without removing
correctness guarantees.”** Its completion gate is **“Performance improves under
equivalent security settings, and concurrency tests still prove the identity
invariants.”** Both editions retain the same shared identity, authorization,
revocation and credential semantics. Applicable implementation, tests,
documentation and artifact evidence must be reviewed; a report alone cannot
complete implementation. Q02 expressly separates security, compatibility,
speed and recovery evidence.

The live rows contain older leave-todo/no-launch scheduling text. The current
user authorization supersedes that scheduling restriction, while this particular
reservation remains source/report-only. Root owns disposition and any next
implementation/runtime reservation. No board update is recommended from this
audit alone.

**Recommend one bounded next slice:** change only the internal
`Core::list_groups` traversal from whole-bucket `Tx::list` to 128-row seeks
within its existing single read transaction, preserving its complete array
response. Measure actual allocation cost against the frozen current body,
with identical security and complete results. This is a reachable source-backed
materialization bottleneck, not a demonstrated latency or RSS regression.
Existing indexes and cursors are substantial accepted implementation; their
presence and historical measurements do not measure this proposed change.

## Immutable source and read scope

All production reads used published Git object
**`d644e2c617501aca8efd43f6350c72b788c19d4d`**. Its parents are
`6e949b97573d7a4cc42cf5ac1ec43c5fc3963599` and
`232817b8617d1905946c2d90e09cc8fc980f1a3c`.
The first-parent comparison has **no changes** under `src`, `crates`,
`Cargo.toml`, `Cargo.lock`, `.cargo`, `build.rs` or `rust-toolchain.toml`.
The separately changed edition-transition PostgreSQL fixture script does not
establish a new S02 optimization or measurement.

Read `CONTRIBUTING.md`; no applicable `AGENTS.md` was found in the worktree,
ancestor guidance locations or the report's directory chain. Its broad Cargo
guidance is limited by the user's explicit static-only reservation. No branch
alignment, source import or mutable primary-worker read occurred.

Body review covered the complete small `src/user_listing.rs` and
`src/store/prepared.rs` modules, both complete admin-listing pagination tests,
the complete S02 cutover proposal, the complete S04 root disposition, and the
contention shell recipe. Selected source bodies reviewed were:

- `Core::list_users`, `list_groups`, `list_clients`, Group create/member writers
  and `groups_for`; `Core::principal`, `Principal::allows/require`, API and portal
  Group-reader wiring; the complete `Group` model.
- `Store::read`, `Tx::list/scan/scan_reverse`, raw range fetch and scan accounting,
  key-only snapshot traversal/digest, bounded snapshot pages, full snapshots,
  and the record writer/import/delete hooks.
- Maintenance index updates, Group binding/member/DN/user membership indexes,
  connector due cursor, maintenance sweep cursor and index rebuild bodies.
- The full `scim_list` body and sort/cap declarations; LDAP selected-user/member
  traversal and its transport paging body; outbound provisioning snapshot
  plan/generation validation and its User/Link page advancement.
- The complete `indexed_user_group_membership` shared contract and backend
  expansion; contention scan-context/prepared-conflict tests and selected
  characterization workload sections; telemetry scan-counter bodies.

The large SCIM, LDAP, provisioning, shared-contract, contention and Q09 files
were **not** all reviewed end to end. Hashing a complete blob is an identity
check, not a full body review. Q09 evidence sections and S04/reconciliation/PG
report witnesses were inspected as attributed historical records; private raw
benchmark outputs, historical binaries and remote CI logs were not reopened.

These exact published blobs also equal the corresponding objects at historical
CI source `b619fe25269ccc150e473bbcde47cdb3623ef810`:

| Path | Git blob at d644 |
| --- | --- |
| `src/core.rs` | `f02efcde5e9604b7251a8171e0aa437dbd8d8ae2` |
| `src/store.rs` | `8e78d889220c9e956fe00a4f7a69b603063d3ebf` |
| `src/store/maintenance.rs` | `6d6999eaba0eacecfc5587dcc1e94dd8a69e5f3e` |
| `src/store/prepared.rs` | `d7387cd0a5c9dd45e3e32de0b3b1109c9c5b89de` |
| `src/agent.rs` | `0ab2592fb624eabc7b503f5263636108281c9c5d` |
| `src/user_listing.rs` | `3f1fb65a4d31b6b118bb03b12d9b5e892575d250` |
| `src/telemetry.rs` | `3a6f02efcef7fff5e50065285570901e8616715b` |
| `src/model.rs` | `8daa94fa28ae30efce78afe587a0bd1524d4f297` |
| `src/assembly/scim_runtime.rs` | `cd5cf20402e04719d3e6dd6e0d61a87b5f990aee` |
| `src/assembly/ldap_server.rs` | `ee6902e0a82d91e708d2cce64f0257c659213179` |
| `src/ldap_server.rs` | `bbf085bf641e9c047379a7ab83cd4bce6973f8e2` |
| `src/provisioning.rs` | `e572dfb0462f36302e672951b735a634b45fddbf` |
| `tests/admin_listing_pagination.rs` | `ac1ad337ecf7f8aaf28f65aa1c33f1d9d250d9b3` |
| `tests/contracts/shared.rs` | `6d881fbd547fdf29e60a0c4414853903a9134520` |
| `tests/contracts.rs` | `36b579dcbae1a01af654d59136b23e5b6df3b2b1` |
| `tests/contention.rs` | `afb7e7668130d063a24986ffd786ec83514c531b` |
| `scripts/test-contracts-postgres.sh` | `ddad88960767ac53915023a6c251eb8b9a2d345b` |

For the proposed hunk, complete `src/core.rs` is 57,266 bytes, SHA-256
`13da3f54b28a130cd73e5380253248c650f110b4c4bfb563c88382f55cc74304`.
`src/store.rs` is 82,583 bytes, SHA-256
`dfb62f2e482e9e334e148a932c7900607f37297748f0c1f90a8ef65f77babc0f`.
These identities do not assert whole-tree/binary equivalence to the earlier CI.

## Accepted implementation inventory and honest limits

| Surface | Actual source mechanism | Bound and consistency; remaining materialization |
| --- | --- | --- |
| Storage range access | `Tx::scan_direction` constructs a collection-prefix range; forward cursor excludes the prior key with a NUL successor, reverse scan excludes its before bound. redb uses tree ranges and `take(limit)`; PG queries binary primary keys in order with range predicates and `LIMIT`. | Seek pagination, not offset discard. Zero-limit returns empty. `Tx::list` explicitly uses `usize::MAX`. Ordinary scans retain the raw and then decoded page; a row cap does not bound one legacy value's bytes. PG's `records_v1` declares `key bytea PRIMARY KEY`. |
| Snapshot consistency | One `Store::read`: redb read transaction; PG read-only `REPEATABLE READ`. | Repeated page fetches inside this closure share one snapshot. Starting a new read per page would change this contract. |
| Maintenance indexes | Current `PAGE=128`, `INDEX_VERSION=9`; same-writer queue due/age/count, retention/rate, user/group membership, Group binding/source digest, DN-fold and generation updates. | Due selection uses derived keys rather than every source row. Connector due claims inspect 16 entries and at most one wrap with a frozen cutoff. Maintenance uses a frozen upper key and durable 128-row cursor. Rebuild visits one source row per page but remains a total linear offline writer; a single Group row can be large. |
| Identity Group membership | `Core::groups_for` uses `tx.user_group_names` and live PAM extras. Member/DN index pages are 128 rows; long member IDs use bounded index sentinels plus exact overflow point reads. Group binding checks compare independent source/binding records. | Avoids every Group body for each session read. The returned user's group-name set still materializes. Durable Group source and full `Group.members` remain authoritative. Temporary entitlements are not directory Group projection. |
| Legacy management Users | `Core::list_users`, lines497–524, pages 128, authenticates first, applies per-row `user.read`, accumulates only visible JSON views. | Complete legacy array still materializes, with password hashes excluded. Row traversal remains linear for a complete inventory. This is the already accepted pattern for the proposed Group reader. |
| External User cursor | `list_users_page`, accepted source history `5397db3d3be91c435865b75b7e9d7b357aa05788` and `09d60256aa8f043e48e83c58ca5fcdc7a8efec3c`. | Limit1–100; at most10,000 examined rows per call, scan batches≤128, sealed cursor≤4096 encoded bytes, TTL3600s, after key≤256. Live actor is resolved before accepting cursor. Exact actor/credential/limit, permissions/grants, revision, User generation and restore epoch bind continuation; mismatches refuse rather than serve a mixed inventory. Sparse permissions may produce an empty page with a continuation. |
| Inbound SCIM | `assembly/scim_runtime.rs::scim_list`, lines1287–1414, scans 128 rows in one read snapshot; filters owner/deleted/binding/read authority before results. Native id sort uses forward/reverse key order. Other sorts retain a heap window. | Count defaults100, caps1000; non-id window `startIndex-1+count`≤4096. Exact `totalResults` requires a full qualifying scan even with count0. Selected-page relation projection avoids full member views for discarded records. This is bounded intermediate work, not O(page size) total work. |
| LDAP provider/listener | Index-directed selected-user reads, fallback User pages128 after at most128 member point reads, full exact-ID checks. Provider caps selected users2000 and result bytes4MiB. Listener supports page size≤500. | Listener paging slices an already projected result Vec, not a new storage cursor. Connection cookie binds query fingerprint, management revision and expiry300s; old cookie is consumed. The complete permitted result remains materialized before transport paging. No universal per-value bound is implied. |
| Outbound SCIM plan | `provisioning_plan` advances User and Link snapshot pages128, storing bounded progress. | Selected users≤2000; snapshot scans≤100,000, bytes≤2MiB, TTL3600s, plan resources≤2064. Actor/target/revision/User and Link generations/target and Group fingerprints gate continuation. Configured complete Groups and selected-user set still materialize. The final job is not repeatedly rebuilt without its existing completion/lease rules. |
| Backup/migration snapshots | `snapshot_page_bounded` caps rows≤128 and raw key/value bytes; PG sizes keys before fetching values. An oversized first record errors instead of pretending EOF. Key-only next-record discovery supports incremental digest. | Caller supplies byte cap and must retain the same read transaction. Full `snapshot`/`snapshot_all` still explicitly produce complete maps where used; source inspection does not establish every caller as streaming. |
| Optimistic preparation | `prepared_write` tracks exact points/ranges and expiry, revalidates under writer, retries at most4 times. | Prepared reads are outside the writer and not a single long database snapshot. Cached facts and ranges are revalidated before commit; conflicts/expiry cannot be dropped to speed a scan. Staged deletions must not hide further range results. |
| Ordinary Group list — chosen seam | `src/core.rs:554–565`: actor first, then `tx.list::<Group>("groups")`, per-row `group.read`, typed visible Vec, JSON serialization. | Whole raw bucket and whole decoded Group bucket precede filtering; retained visible typed Groups also precede final JSON. API `session_handler!(groups, list_groups)` and portal `read!(groups, list_groups)` share it. This is the sole proposed production change. Other whole-list call sites are inventory observations, not additional reservations. |

Current index version9 is the accepted O06 resolved-deactivation counter rebuild
version. It is **not** the held S02 version9 Group-chunk mirror. The
[authoritative Group proposal](s02-authoritative-group-cutover.md) remains
design-only: source rows still contain complete member sets. That report
attributes 4MiB mirror lookup medians45–126ms versus legacy8–55ms across four
backend/encryption modes to held `63f0cdc..7c695e0`. Raw measurement receipts
were not read here. Neither that held stack nor its proposed canonical-format
migration is accepted or needed for the small reader change below.

## Performance and concurrency evidence classification

| Existing record | What is actually credited | What it does not establish |
| --- | --- | --- |
| S01 telemetry / `tests/contention.rs` | Native scan counts/row sums/bytes and read/writer/prepared contexts exist. Selected definitions verify point reads, scans, prepared retries/exhaustion and expiration accounting. Ignored characterization records phase p50/p95/max and `observations_only=true`. | Definitions were not executed in this audit. Sum of bytes read is not peak live heap/RSS. The characterization source's older “lists all groups” comment is not current indexed-session behavior. |
| `tests/admin_listing_pagination.rs` | Complete definitions cover131 Users over two internal pages, exact order/per-row scope/no password hash, revoked authority, unchanged legacy array, limit17 external paging and stale actor/permission/generation/revision cursor refusals. | No Group-list allocation measurement; no fresh run here. |
| Q02 `indexed_user_group_membership` | Complete body seeds300 Groups/130 memberships, confirms no unbounded Group scan in `me`, orders a real Group writer between reads of one snapshot, checks revision advancement, replace/delete/rollback and rebuild/reopen. Backend expansion selects plain/encrypted redb and separately ignored plain/encrypted PG. | Backend definitions are not executed outcomes by themselves; indexed session lookup is a different operation from `list_groups`. |
| Accepted PG CI attribution | [O07 source/evidence review](local-wave30-o07-original-scope-independent-review.md) records b619 run37041786903 integration110960005237, full log448,957B SHA-256 `f6b092d43208a33e0ebbdf6804e20e18ca3cd0e351cbf8face8943676e3e86d4`, shared contracts91/91 in172.41s:45 plain PG,45 encrypted PG, one PG-archive→redb case. Relevant current fixture/store/Core blobs above equal b619. | This audit read the accepted report witnesses, not that raw CI log; no new CI query/runtime. A named full-suite result is historical security/concurrency evidence, not a Group-list speed comparison or fresh d644 binary pass. |
| [Q09 measurements](q09-benchmark-slice.md), selected complete evidence sections | Source `47aa248ce1c68284773746bbb5fd59c6098afdea`, harness SHA-256 `515d0ed39887a9180f3f24d36d56b11d28da4dde776f8dcaa0d75c6bb1a01b4e`: four macOS dev product runs,9 accounts/8 Groups/72 memberships,40 quiet+40 interference reads each, zero read errors. Tables report p50/p95/p99, throughput and2–4 RSS samples; no maintenance completion in these short passes. Later secure Linux ARM64 Docker-VM report sections attribute12+12 reads and71.57–71.66s passes with maintenance completion to harness `4ab934a44a80062dda0dba5d8640d396631a3ef0`. | Historical bodies specify `observations_only=true`, `performance_claim=false`. Early5892563/db533e7 records explicitly place JSON outside this worktree. TLS/encryption/backend shape and compiler/harness identity differ by record. No raw output or old binary was reopened, no current/released/native-Linux inference, and no before/after Group-list comparison. |
| [Accepted S04 root disposition](local-wave30-s04-root-disposition.md) | Full body records source `5120a0ddb51d015fbf89fbca19da127444e26f8b`, one ignored run1pass/17filtered/20.47s, actual log SHA-256 `6287eb35f69017af138304ca7ca96624765d7a7b60ba49efab936ae2781e0e6d`. Same-build conservative control versus scoped reuse:96 calls/lane;192→0 writer holds,96→0 commits,749,695→0µs occupancy, paired identity/authority/concurrency/replay checks. | A reviewed executed bounded writer-cost improvement under equivalent security, not a new execution here or Group-list allocation/RSS/throughput result. S04 remains closed. |
| [Reconciliation pagination correction](local-wave30-reconciliation-pagination-ci-report.md) | Source `9810c33edba8027433c1a4e24442d878710b9a8a`; unchanged513-Link fixture: baseline exit101 versus corrected exit0, real reopen and5 snapshot pages, no downstream delivery. Accepted report provides both captured log hashes and exact source/fixture witnesses. | Test times1.44s/1.71s are explicitly not performance evidence. Its filtered revoked-authority sibling is not credited as run; this is a controller progress/lease correction, not this reader optimization. |

No original-row DONE recommendation follows solely from this inventory. Existing
executions support the established storage/identity mechanisms and prerequisites;
the proposed materialization change still needs actual equivalent-security cost
and concurrency evidence. This does not add mandatory all-host, released-artifact,
all-verb or broad benchmark gates to S02.

## One proposed implementation seam

Prospective ownership, subject to root's separate reservation:

| File | Exact prospective scope |
| --- | --- |
| `src/core.rs` | Only `Core::list_groups` body: same `Store::read`, principal first; loop `tx.scan::<Group>("groups", after, maintenance::PAGE)`, use last **storage key** for exclusive continuation, filter with stored **Group.name**, push each visible Group directly as JSON, return `Value::Array`. Use the existing `list_users` termination pattern. |
| `tests/s02_group_listing_paging.rs` — new | One ignored `group_listing_paging_preserves_snapshot_authority_and_measures_materialization` fixture, its private frozen-baseline/cost helpers and local result/security/concurrency assertions. No shared helper edit. |
| This new report | Append exact implementation/runtime evidence only after reservation; preserve this source-only plan. |

No external Group cursor, route change, new receipt, schema/index-version bump,
Group membership cutover, generation mutation, config change or dependency is
required. Existing API/portal dispatch receives the same complete array.
Do not paginate by permission-filtered name, cache the principal between
requests, open a transaction per page, truncate the output, make malformed
records disappear or substitute member indexes for the source Group's response.

The change still scans and decodes every Group for this complete inventory, and
an individual `Group.members` set may be large. Its defensible claim is removing
unnecessary **simultaneous whole-bucket intermediate materialization**, especially
for scoped readers. All-visible output still holds all visible JSON. It cannot
promise fewer total rows/bytes read, constant whole-operation memory, per-row
byte bounds, lower RSS, faster PG latency or a faster canonical membership probe.
More PG round trips could regress latency; measurement must expose that limit.

## One bounded prospective fixture and measurement recipe

This is a design, not implemented or run. Use a fixed default-Platform local
**encrypted redb** fixture, with the same key, configuration, synthetic durable
data, credentials, permissions and revision in each compared lane. No HTTP/PG
service, disabled encryption, forced clock, lease/rate bypass or remote input.
Small boundary cases0/1/128/129 plus one513-Group dataset are enough. For the
measured dataset, use128 stable synthetic User IDs in complete canonical member
sets and a scoped reader seeing exactly three Groups spanning first/middle/last
pages. Seed through the existing trusted `Tx::put` test path; its synthetic
storage construction is not public-writer or tenant evidence. Use normal public
Core methods for the authority and membership mutations under test.

Use a **same-build frozen control** containing the exact d644 `list_groups`
body, including `principal` inside the read closure, `Tx::list`, exact permission
check and typed-Vec→JSON conversion. Verify its literal body against the pinned
source before implementation. It is a cost comparator, not the result oracle
and not an older binary. The candidate lane calls the actual public
`Core::list_groups`. Both must match independently constructed **complete**
expected JSON from the fixture's known keys/names/member IDs on every call.

The only additional measurement mechanism proposed is a fixture-local
`GlobalAlloc` wrapper delegating unchanged allocation/deallocation/reallocation
to `std::alloc::System`, using allocation-size atomics without allocating in
the counter path. It belongs solely to the new test executable, never product
code. Record current allocation baseline and peak extra live Rust-allocated
bytes during each complete call, retaining the returned JSON until the window
closes. All other fixture work/expected JSON/counter rendering stays outside
the window; no concurrent fixture threads during measured samples. The meter's
implementation and accounting, including realloc/failure/deallocation, need
independent source review and a small self-consistency assertion before use.
It measures this process's Rust heap allocation cost, not RSS, redb cache
capacity, native OpenSSL allocation or distributed capacity. An unrelated
allocation cannot be silently attributed to the reader.

Plan three identical closed-seed pairs in AB/BA/AB order, five untimed warmups
and ten measured calls per lane/pair. Compare native read point/scan/row/byte
counters and complete outputs, report every pair's measured peak-extra-heap
and elapsed samples plus p50/p95. Do not extrapolate throughput or accept a
timing ratio without the raw observations. For the513-row scoped request,
expect baseline one unbounded Group scan/513 rows; candidate five bounded
Group scans/513 rows and no unbounded Group scan. Principal point reads stay
equivalent. Native row totals alone are not peak-heap measurements.

Root's gate for this slice should require a real measured reduction in the
unnecessary intermediate allocation cost under the same encryption/authority,
with complete correctness/concurrency assertions passing. If allocation does
not improve, or instrumentation cannot faithfully isolate the cost, return the
actual result and re-evaluate; do not replace evidence with theoretical128-row
arithmetic. Report latency even if it worsens. No result is predicted here.

Complete before/control and after/candidate expectations:

| Case | Required unchanged outcome |
| --- | --- |
| Full administrator inventory | Exact ordered array, exact name and complete member IDs for all513 Groups; BTreeSet serialization order retained. No secret-bearing output added. |
| Scoped valid reader | Exact three visible Groups and every member; hidden Groups absent; ordering by persisted key, permission resource from stored name. Valid authenticated reader with no matching scope receives the existing empty array, not a new blanket403. |
| Invalid/revoked credentials, disabled identity/parent | Same existing status/code before any Group traversal; compare native Group-scan window separately from principal's own reads. A subsequent call after revocation cannot use prior authority. |
| Ordinary read noninterference | Complete durable snapshot, revision, audit and receipt/domain rows equal before/after every read/refusal. Telemetry counters may advance; persisted data may not. |
| Snapshot interleaving | Follow the existing indexed-membership contract's channels-ordered pattern: open one read transaction and read first Group page/principal, commit a real `Core::group_member` change affecting a later page on another thread, then read remaining pages in the original transaction. Old snapshot retains exact old membership; subsequent real Core listing sees the exact committed new membership. Bound channel waits and join the writer. |
| Authority interleaving | Analogous ordered snapshot retains its captured authority and rows; a fresh request after normal credential retirement refuses. No page opens a new snapshot or fetches a new actor mid-array. |
| Real Core read/write overlap | Bounded concurrent calls must return an entire valid pre- or post-write snapshot, never a mixed/partial array. Report whether overlap actually occurred; underlying ordered transaction case alone does not prove a writer was injected between two pages of an actual Core call. No production timing hook is proposed. |
| Error/edge preservation | Empty/exact-page/final-short-page behavior correct; storage/decode errors remain errors rather than tolerant skips. No new malformed-record acceptance or key/name normalization. |

Snapshot comparisons for ordered writers distinguish the one expected Group,
derived indexes, management revision and audit effect from read effects. Use
the fixture's full expected before/after state and normal writer outcomes,
not physical-file equality, aggregate row counts or a partial projection alone.
Do not claim every Group policy/race schedule from this one bounded case.

One prospective exact command, **not executed or released**:

```sh
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support --test s02_group_listing_paging \
  group_listing_paging_preserves_snapshot_authority_and_measures_materialization \
  -- --exact --ignored --test-threads=1 --nocapture
```

The one ignored fixture would run the compared lanes; no broad characterization
script or second backend campaign is proposed. Root must first reserve the two
prospective code paths, review immutable source/control/meter definitions and
approve an exact bounded supervisor. A possible future envelope is1800s total
including Cargo and300s for the fixture, own process-group termination/join,
private0600 capped captures and fixed public metrics only. No names/member IDs,
keys/tokens/passwords or low-entropy credential hashes in public output.
Use only the existing own target, jobs1/incremental0/dev+test debug0; fresh
capacity must justify the compile allowance above a9GiB stop/8GiB floor.
A13GiB starting threshold is a prospective conservative4GiB allowance, not
measured peak or a current capacity claim. No slot, preflight/cache inventory,
binary execution or runtime has been requested or acquired by this audit.

If the owner wants a PG-specific latency claim, that requires its own later
equivalent-security evidence; it is not a prerequisite invented for this local
allocation slice. Likewise no Group-format migration, broad benchmark campaign,
external hardware or release-artifact claim is part of this proposal.

## Checks actually performed and handoff boundaries

Actual work: read-only live RiWork rows; clean own branch/path checks;
`git show`/tree/history/diff reads at fixed d644; full-blob identities/hashes and
the selected b619 equalities above; source/report body review at the scope
stated; documentation-link and Git whitespace/scope checks for this new report.
No production/test file was edited. A lookup of nonexistent `tests/common.rs`
returned Git exit128; fixture helpers are under `tests/common/`. A few exploratory
`rg` searches returned exit1 for no matches; large combined output was truncated
and relevant bounded bodies were reread. These are static lookup/output limits,
not product, compilation or test failures.

No Cargo, runtime, benchmark, service, artifact execution, native/library/CLI
probe, network/download/CI query, browser/desktop/Driver action, worker contact,
merge/alignment/reset/cleanup, main edit/push or task/board change occurred.
Only this new report is authorized for commit. Original primary assignment,
completed O07 recommendation and all other closed rows remain untouched.
Credential issuance/replay, route-specific optional/required headers, PAM
fallback, canonical Group authority, review/removal, audit and shared60s
admission contracts are preserved.

Root receives this immutable plan for a separate implementation decision.
The one residual is **unmeasured removal of `list_groups` whole-bucket
intermediate materialization** with the exact result/authority/snapshot
contract above. No external input prevents this proposed local slice.
Actual cost improvement, the new fixture's runtime, independent review and
integration remain unperformed; this audit does not close S02.

## Reserved source implementation — 2026-10-03

Reservation **`wave30_S02_group_listing_source`**, same project/task/supporting
worktree and original primary assignment. Root read the entire original plan and
approved exactly three paths: `src/core.rs::list_groups`, new
`tests/s02_group_listing_paging.rs`, and this report's appendix. This section
preserves the entire cdebc report prefix: **29,390 bytes**, SHA-256
`2d57c0192e48df80b30f63efac6768a2db394af919e15a00d0f1f37879b24570`.
Earlier source-only and historical evidence statements remain dated observations.

**Source commit `74c1fb7660791864a68b9caebb7977f30a53e41c`** contains only the
reader hunk and new fixture. **UNCOMPILED / UNRUN**: no typecheck, allocator
self-check, dataset preparation, benchmark, concurrency execution or cleanup
postcondition has occurred. No runtime slot was acquired or released. No
measured performance conclusion or original S02 completion follows from source.

### Source identity, production scope and exact reversal

Fixed published comparison pin:
**`fab6721a5ac96a39a04263a27e67c252246835ee`**. Before the reader change, these
complete own cdebc blobs equal fab: Core, agent, Store, maintenance, model,
telemetry, management, assembly, SCIM runtime, identity transitions, Config,
edition and edition transition; `Cargo.toml`, `Cargo.lock`, and toolchain.
Those sixteen comparisons were actually performed with immutable Git bytes.
No alignment, branch-history import or wholesale production replacement was
needed. They are selected-path identities, not whole-production-tree or binary
claims. Root must independently validate any broader compile/source alignment
and actual cache/capacity before a later release.

| Committed file | Bytes / lines | SHA-256 |
| --- | --- | --- |
| `src/core.rs` | 57,913 / 1,414 | `686e7e732f256e7b435ab69991b71fb26d7467214877d2e0f0c840741446caea` |
| `tests/s02_group_listing_paging.rs` | 55,835 / 1,641 | `b340155ed571b3b0195c40a141239f5337585356e4e7d8582dd5801162500f42` |

Actual static reconstruction replaced **only** the new method with the old
method and recovered the **entire** cdebc/d644/fab Core byte sequence. The
production diff is 22 additions/7 deletions in one hunk. `list_users`, all other
Core methods and signatures remain byte-exact. The new method captures the live
principal first inside the original `Store::read`; uses 128-row `Tx::scan` with
the actual last **storage key**; filters on stored **Group.name**; immediately
converts only visible rows to JSON; returns the complete `Value::Array`. Empty,
exact-full and final-short pages use the accepted User-reader termination
pattern. Every scan/decode error propagates; there is no partial response or
tolerant skip. Total full-inventory traversal and large individual member sets
remain explicitly unbounded by bytes.

The fixture's `Frozen` wrapper dereferences the real Core. Its **whole old
`pub fn list_groups` method is literally byte-identical**, including principal
resolution inside the same read closure and old typed-Vec-to-JSON conversion.
The static check found exactly one copy of that old method in the new file.
No copied principal, permission evaluator, storage implementation or production
control switch was introduced. The frozen method is a same-build comparator;
independent expected JSON is constructed from seed constants, not either lane.

No API/portal/client, model/member format, schema/index version, writer,
receipt/permission service, header policy, manifest/dependency, source-TOTP hook,
activation or W07 change occurred. All other tracked production files stayed
unchanged. Source edits are confined to the reserved method and new test target.

### Allocator definition and refusal boundaries

The new executable alone selects `MeteredSystem`; every allocation delegates
unchanged pointer/layout behavior to `std::alloc::System`. All threads remain
globally accounted; there is no alternate allocator or unmetered lane. Each
successful alloc/alloc-zeroed increases live **requested** bytes, dealloc
decreases them, successful realloc adjusts the old/new size delta, and a null
alloc/realloc result changes no live bytes. Failed realloc retains the original
allocation. The self-check's `ProbeBlock` retains ownership/current layout through
failure and assertion paths and frees it once.

A non-allocating atomic accounting lock serializes counter updates and
begin/finish transitions with `SeqCst`; System calls run outside that lock.
Accounting callbacks contain no formatting, allocation or panicking arithmetic.
Overflow/underflow permanently invalidates evidence rather than unwinding from
`GlobalAlloc`. Const destructor-free TLS marks the window owner; unavailable TLS
or any other thread's allocator operation marks the window foreign. Foreign
operations are included in accounting and **refuse** evidence, never silently
subtracted. The parked watchdog's startup handshake is outside all windows.

Begin captures global live baseline and initializes peak. Finish retains the
entire returned JSON, closes the window and captures peak-extra/current-extra,
allocation-operation counts, null-result count, contamination and invalidity.
The sample must have no contamination/accounting failure/allocation refusal and
must observe allocations plus a positive live returned-output delta. Result
checks, complete snapshots, telemetry reading/rendering and returned-value drop
are outside the cost/timer window. Both lanes use the same meter and policy.

The meaningful self-check definition allocates64 bytes plus96 zeroed bytes,
verifies zero contents, grows64→256, verifies content preservation, shrinks→32,
exercises the exact null-result accounting branches for alloc/zeroed/realloc,
then releases both blocks. Its expected peak is352 and final extra live bytes0,
with exact operation counts. The three null cases are **accounting-path
simulations**, not a native allocator OOM fault injection. No huge allocation is
requested. The self-check has **not run**.

The metric excludes native direct-malloc allocation, System/malloc arena and
realloc-internal temporary overhead, redb disk/cache capacity and RSS. Latency
is explicitly the instrumented same-build call interval; it is not production
unmetered throughput. No speed/RSS/physical-capacity claim is made from these
definitions or from page arithmetic.

### Exact fixture, cost windows and assertions defined

One ignored function:
`group_listing_paging_preserves_snapshot_authority_and_measures_materialization`.
It requires the command's canonical own `$PWD/target`, creates fresh private
temporary labs only beneath it, generates a private encryption key, and uses
normal `Core::initialize/login/create_agent/open` setup. It clones128 stable
synthetic Users through trusted `Tx::put`; each of513 canonical Group rows has
all128 members. The data is a synthetic storage fixture, not tenant/public
Group-writer acceptance evidence. Configuration has a real database-key file
and no PostgreSQL/services/remote profiles.

The closed empty seed is opened once through normal public startup before its
full snapshot is captured. This accommodates normal accepted startup work,
including conditional edition-provenance observation, without raw pin/ledger
repair or exclusions. Same-source copied lanes must then match the **complete**
logical snapshot and normalized configuration. No physical file-hash equality
or automatic issuer/key/edition correction is assumed. Initial seed key remains
alive until every copied Store is closed.

Three fresh closed-seed pairs use **AB/BA/AB**. Each lane measures two scopes:
scoped exactly first/middle/last three Groups, and full administrator inventory.
Per lane/scope: five warmups and ten measured calls. Thus the definition emits
**120 measured rows** (three pairs×two lanes×two scopes×ten) and twelve
pair/lane/scope summaries; warmups are separate60 calls. Complete expected JSON
and full logical durable snapshots are compared after each call. No fixture
setup, mutation/concurrency, snapshot or rendering work enters the measurement
window. Actual fit within300 seconds is unknown; it cannot be inferred from
the source count.

Native point/byte counters must agree across every measured control/paged
sample for a given scope/pair. The control expects one unbounded Group scan
with513 rows; the candidate expects five bounded scans with513 rows and no
unbounded scan. Writer-hold and commit deltas must remain zero in every read.
Raw numeric sample outcomes precede their grading. All twelve summaries, with
nearest-rank p50/p95 latency and median heap cost, precede the cost decision,
even when allocation/latency regresses. The actual cost gate requires **lower
median scoped peak-extra live requested Rust bytes in every pair**. No invented
latency ratio is a pass condition; both scopes' raw latency/regressions stay
visible. No value of any quantity is yet known.

Additional definitions, all inside the same ignored fixture:

- 0/1/128/129 Group boundaries: exact administrator/scoped/empty-scope arrays,
  native page counts including the empty terminating fetch, invalid credential
  401/`invalid_token` before any Group scan, and full read/refusal noninterference.
- 129 trusted compatibility rows with storage keys ordered opposite their
  names, crossing a page boundary: exact full key-ordered array and stored-name
  permission scope. These are reader-compatibility rows, not acceptance by the
  canonical Group writer or LDAP binding service.
- Read-only principal's actual public membership-write refusal403/
  `access_denied` with full rollback; malformed Group through `Tx::put`
  500/`server_error` with full rollback. No malformed persisted row/raw import is
  installed; reader decode-error propagation is source-reviewed, not runtime
  corruption injection.
- Channels-ordered snapshot: capture principal and first page in one real
  `Store::read`; another owned thread commits normal public membership removal
  on the later Group or public scoped-Agent revocation; continue all pages in
  the original transaction and require the exact old result. A buffered result
  receipt must report writer success before the old-snapshot assertion. Join
  the writer, then require the exact new membership or fresh401 refusal.
- Full normal-writer snapshot oracles: independent Group/source-binding digest,
  source-digest row, reciprocal member/user indexes, one revision increment and
  exact single audit event/content; Agent revocation's exact enabled/token-map,
  redacted audit and revision effects. Only the actual audit UUID/time are
  admitted after canonical UUID, real-time interval and exact key checks; all
  event fields and every other durable row must match. No-op membership retry
  changes no logical state; repeat revocation conflicts409 with no new audit.
- Eight actual unhooked Core reads against a normal public writer: every
  response must be the whole pre- or post-write JSON. Captured call intervals
  report whether overlap was actually observed. This does **not** inject the
  writer between two pages of the actual Core call; the separate ordered Tx
  test is clearly labeled. No production hook, sleep, forced clock or rate/
  review/authority bypass is used.

Credentials held by the fixture use `Zeroizing<String>`. Snapshots contain
private values in memory; no claim that serde JSON snapshots are zeroized is
made. Equality assertions never Debug-print snapshots/results. Public output
contains fixed labels and numeric/boolean metrics only; error messages, paths,
tokens, password/key material, member identities and credential hashes are not
printed. Generic fixture errors use fixed labels; Core errors expose only their
public static code and numeric status.

### Thread, deadline and cleanup policy definitions

The real monotonic budget starts at fixture entry:300 seconds, with bounded
channel waits capped10 seconds. One parked owned watchdog requests process
exit124 at the deadline or watchdog synchronization failure. It uses no core
dumping abort. Worker guards join on ordinary success/unwind; completion channels
are buffered so an abandoned receive cannot strand a writer in its send. The
watchdog is canceled and joined on normal completion/unwind. No worker is
detached or reused as an external agent.

The source does **not** promise a kernel IO quota or that `process::exit`, Store
drop/IO, joins or stdout cannot stall. A separately reviewed outer supervisor
must enforce its own process-group deadlines, retain exit124/timeout as failure
without guessing its cause, and clean only its private fixture resources after
a forced exit. There is no runtime/cleanup proof yet.

Normal success explicitly closes each Store before `TempDir::close`, checks
directory absence, and keeps the shared key until all copies close. Sixteen
temporary lab directories are expected by the source recipe; none was created
in this source phase. Automatic Drop on an earlier panic remains best effort,
so no failed-run cleanup success is fabricated. The pre-final numeric receipt
follows explicit file cleanup and owned thread joins. A final post-output clock
check rejects late completion, with no subsequent fixture output/file writes.
Libtest/outer-supervisor completion remains a separate runtime observation.

### Static checks actually performed and held runtime request

Actually passed: explicit-file `rustfmt --edition 2024` parse/format and
`--check`; `python3 scripts/check-docs.py`; Git whitespace and exact code/report
path checks; whole-Core reconstruction; literal frozen-method equality; exactly
one ignored test definition; report-prefix identity; sixteen immutable selected
source/build-input comparisons to fab. No dependencies/toolchain were changed.
Rustfmt parsing is **not** Rust typechecking or unsafe-allocator verification.

During source work, one lookup used an unmatched zsh glob and printed
`no matches found`; it was replaced by literal paths. Two `apply_patch` attempts
did not match current lines and applied nothing; the exact current lines
were read and the narrowly scoped patches then applied. Static result-type review
also selected the generic fixture error helper for `config::private_dir`'s
public anyhow result. These are source preparation events, not compiler/test/
runtime failures. No runtime receipt, failure or performance number is invented.

The sole future exact command remains **HELD**:

```sh
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support --test s02_group_listing_paging \
  group_listing_paging_preserves_snapshot_authority_and_measures_materialization \
  -- --exact --ignored --test-threads=1 --nocapture
```

Root must fully review the meter/control/fixture and independent review first,
then separately approve source alignment, fresh capacity and an exact supervisor.
The changed production method requires compiling the library; this is not
asserted to be a test-only warm-cache run. Private own target, jobs1,
incremental0, dev/test debug0; conservative prospective13GiB start/9GiB stop/
8GiB floor and1800s compilation-plus-test envelope remain proposals pending
measured capacity, not current evidence or guarantees. The supervisor must
respect the300-second fixture policy, private capped captures, joined owned
process group, final clocks and exact failed outcomes. No deletion, broad suite,
PG service, alternative target/feature or automatic rerun is proposed.

Code source is ready for independent static review; allocator fidelity/typecheck,
actual snapshot/concurrency outcomes, capacity/cleanup and measured allocation
improvement remain **unknown**. There is no known static scope blocker, and no
claim those checks passed. Root alone owns later reservations, integration/push
and the original row's status. Primary S02 assignment and all completed rows
remain untouched; no board action, external/worker contact, merge/reset, source
alignment, Cargo/runtime/native/artifact execution, service/browser/desktop or
network action occurred. This appendix is the only report change.

## Safe measurement fixture design after the manifest blocker — 2026-10-03

Project **891e7443-8dac-4c1b-897f-9e53cb59c7ee**, original S02
**fdda2152-73a0-4dce-9e5e-aff4b232a6fd**, same supporting WTe1/branch and
unchanged original primary. Reservation **`wave30_S02_safe_measurement_fixture_design`**.
This is **DESIGN ONLY**. The entire d6b report prefix remains byte-exact:
**45,580 bytes**, SHA-256
`eadea67eb08175ba194b0b0621093fe4947bba45f89786bb3c48442a5d61e503`.
No proposed Rust source has been written to the fixture, compiled, typechecked
or executed. The production reader at 74c1 remains unchanged.

### Concrete blocker and recommendation

I missed the manifest's applicable unsafe-code policy in the earlier static
review. The complete current manifest has
`[lints.rust]\nunsafe_code = "forbid"`; the committed integration fixture's
`unsafe impl GlobalAlloc`, unsafe methods and unsafe blocks conflict with it.
Root identified that contradiction by source review. It is a **source compile
blocker**, not an executed Cargo/compiler failure. The prior appendix's
“no known static scope blocker” and allocator-readiness statements are historical
and superseded by this discovery. Its UNCOMPILED/UNRUN statement remains true.

**Propose replacing only the new fixture with the complete safe-Rust design
below.** Delete the custom global allocator, raw allocation probes, accounting
lock/TLS/counters, heap windows and every live-allocation assertion/output.
Keep the repository policy and manifests unchanged; add no crate, dependency,
allow attribute, Rust-flag override or allocator indirection. Keep
`src/core.rs::list_groups` exactly at 74c1. Retain the literal frozen old method,
independently constructed complete JSON, encrypted-redb settings, full durable
read/refusal/writer oracles, ABBAAB pairs, five warmups/ten samples and both
scopes, ordered real writer snapshot and actual Core interval observations.

Replace the invalid allocator cost gate with **observed raw-fetch row
materialization**, using the existing safe public native histogram. This is
a precise narrower metric, **not** renamed live heap, RSS, decoded-Group peak
or total output memory. The root must approve that measurement scope before
any source edit/runtime; this report alone cannot declare S02's original
equivalent-security/performance gate met. All actual latency rows/p50/p95 and
the boolean comparison remain reported, including regression. There is no
assumed speedup or latency ratio threshold.

### Exact source backing and metric meaning

Complete body reads this phase: `Cargo.toml` and
`src/identity/agent_credentials.rs`. The complete current fixture bytes were
read for the design transformation and scope proofs; its original full-body
review belongs to the preceding source implementation. This phase reread the
changed measurement/control/counter sections and relevant fixture setup,
writer/concurrency/cleanup sections. Selected complete function/section reads:
`Sizes::{observe,count,sum,render}`, `render_buckets`, `Reads::{point,scan,scans}`,
the scan histogram render site; `Tx::{list,scan,scan_direction,raw_scan_base,
raw_scan_fetch,put,record_change,changes}`, public-record/credential/audit
redaction; `Principal`, `Core::principal`, `authority_active/parent_active`,
normal Agent creation; normal User update and shared User-record writer;
identity prepare/transition/child retirement; empty-job downstream/logout/SSF/
Windows effects; audit revision/change serialization; User/UserView/UserPatch.
Relevant parent fixture bodies were selected reads, **not** an assertion of a
new execution or a full read of all existing Agent-parent tests. Larger modules
were not all reread end to end. Original sixteen selected fab equalities remain
source identity evidence; no new whole-tree/binary claim.

`src/telemetry.rs` at fab has safe `Sizes` cumulative row buckets
**0,1,16,128,1024,8192,65536,+Inf**, count and row sum. The existing public
`Sizes::render` exposes those buckets. `Reads::scan` records the actual
`records.len()` of the `Vec<(String,Vec<u8>)>` returned by
`raw_scan_fetch`, before typed decode; the bound label records the requested
limit class. Both frozen and candidate reads share this implementation.
Therefore the measured population is **raw storage records in one fetch
result**, not simultaneously live Rust bytes or decoded/output objects.

The proposed local `ScanHistogram` reads only those already existing counters
via the existing safe render function into a local string, outside every list
call timer. It accepts exactly ten fixed metric lines, canonical unsigned
integers, all eight unique fixed buckets, count/sum, monotonic cumulative
buckets and exact +Inf=count. Unknown/missing/duplicate/changed-schema lines
refuse; the parser has no IO/process/dependency or production hook.
Count/sum must also match the existing direct accessors. Checked subtraction
refuses reset/backward counters. The declared same-executable isolated Store
has no concurrent storage worker during cost samples; contamination producing
unexpected points/bytes/scans/buckets fails the exact existing oracles rather
than being subtracted.

The observer self-consistency definition uses actual safe `Sizes::observe`
on local objects for [128,128,128,128,1], checks the independent literal bucket
counts **[0,1,1,5,5,5,5,5]**, count5/sum513, and fixed duplicate/malformed/
noncanonical/inconsistent/missing parser negatives. It does not run either
reader or fabricate a measured sample. It has **not executed**.

For each of120 proposed measured calls, actual native deltas must have exact
full result/snapshot/point/value-byte invariants, zero writer/commit, and the
entire histogram:

- Frozen: one unbounded fetch with row sum513 and exactly one observed fetch
  **over128 rows**. Count1 plus sum513 establishes that fetch's exact row size.
- Candidate: five bounded fetches totaling513 rows; actual cumulative le128
  equals every fetch count, so **zero observed fetches over128 rows**, with
  the exact expected bucket frequencies. The histogram's finite bucket alone
  bounds each fetch by128; the source limit is additional source evidence.
- Independent page/bucket frequency expectations for0/1/128/129 remain.
  Those constants are **oracles**, not performance evidence without real native
  observations. The source gate must compare actual delta data to them.

The source now grades this observed fetch-batch reduction in **all** samples,
pairs and scopes, while native point reads and total encoded value bytes remain
equal between lanes. It emits every raw row and twelve summary rows before its
cost decision. The complete returned JSON remains live until after result and
snapshot checks, but its memory is not measured.

Traversal remains513 rows in total; fetch count increases1→5, so **no fewer
scans, less total IO or smaller encoded byte sum is claimed**. Fetch batches
are bounded by rows, not bytes; one record and the complete administrator JSON
can still be large. There is no decoded-object high-water counter, live-output
memory witness, heap/RSS/cache-capacity measurement or allocator evidence.

Actual monotonic complete-call intervals are recorded with five warmups/ten
samples per lane/scope in the same ABBAAB recipe. Native rendering, full snapshot,
grading, stdout and Value drop stay outside that call timer. All raw ns values,
nearest-rank p50/p95 and “scoped p50 lower in every pair” boolean are emitted.
A false boolean/regression is preserved and is not called a speedup.
`latency_grade=false` states the gate is fetched-record materialization cost.
Root retains the original-row disposition decision; a future observed batch
pass does not silently substitute for any still-required live-heap/speed claim.

**No external allocation observer is proposed or needed for this narrower
native-cost design.** If root requires actual live heap instead, that is an
unresolved separate measurement decision: this proposal supplies no supported,
pinned observer, no sampling accuracy claim, and no invocation. Selecting an
external observer would need an exact source/tool/provenance and independent
runtime reservation; sampled RSS/whole-process peaks could not be substituted
for Rust live requested allocation. No new package/dependency/tool deployment
or generic campaign is authorized here.

### Precise expiry and parent authority consideration

The pinned `Core::principal` resolves token mapping and exact credential hash,
then `authority_active` requires `enabled && expires_at > crypto::now() &&
parent_active`. Thus **expires_at <= real now** refuses401/`invalid_token`
before Group scanning, even if the credential mapping and enabled flag remain.
A parent, when present, must resolve to an existing enabled User in the same
read transaction. Both reader bodies invoke this shared principal inside their
own same-snapshot closure; no principal/time logic is copied.

The revised source proposes one additional isolated owned lab outside every
cost window and two explicitly separated cases:

1. Create a valid unparented scoped Agent through normal public
   `Core::create_agent`, prove both lanes' exact positive result, then set its
   canonical stored Agent expiry to the captured real `crypto::now()` through
   trusted `Tx::put`. This is **declared compatibility setup**, not a public
   expiry mutation API, elapsed natural TTL proof or clock override. The entire
   snapshot may change only that Agent's expiry field; enabled/hash/token-map
   remain. Each lane must then refuse401 exactly without Group traversal or
   durable effects. The receipt reports whether the real clock was still in the
   captured second; no same-second equality is assumed. A backwards real clock
   or unexpected writer consequence fails. No sleep/time hook/forced clock/
   config TTL relaxation is introduced. The strict equality boundary is also
   explicit in the inspected production predicate; runtime credit remains only
   the timestamp/real-time relation actually observed.
2. Create a normal child scoped Agent bound to existing enabled non-admin
   synthetic User0. Prove both reader lanes positively, then disable the parent
   through the normal public `Core::update_user`. The complete expected durable
   map independently includes only User enabled/epoch, two generation increments,
   child enabled=false and exact token-map removal, one revision and one audit
   with exact sorted redacted Agent/User changes. Seed has no owner RP session,
   device, grants, downstream job/link or SSF stream; unexpected extra durable
   effects fail complete equality. Each lane refuses child401 before scans with
   no writes; unrelated unparented scoped authority and complete Group contents
   survive. No writer service is changed or privately repaired.

The public disabled-parent case proves **normal atomic retirement and fresh
request refusal**. It does **not** isolate the legacy fallback where an enabled,
unexpired Agent/token mapping survives beside a disabled/missing parent; normal
creation refuses such a parent, and normal disable retires it. That separate
`parent_active` branch is source-reviewed, not mislabeled as runtime covered.
A synthetic legacy compatibility insertion would need a separately explicit
test-setup decision if root wants its isolated execution; this proposal does
not restore a retired credential or treat that broader campaign as mandatory.
No production parent/expiry change, activation hook, receipt/header/PAM/Group
contract or authority exception is proposed.

### Full revised fixture and precise diff — not materialized

| Design object | Bytes | Lines | SHA-256 |
| --- | --- | --- | --- |
| Committed old fixture74c1 | 55,835 | 1,641 | `b340155ed571b3b0195c40a141239f5337585356e4e7d8582dd5801162500f42` |
| Proposed whole safe fixture | 60,049 | 1,715 | `a93636a21a4a5cae53b2a5b844e8e5534e10fc0b1941cc43927f75b720cf63d6` |
| Exact zero-context unified diff below | 36,823 | See literal text | `b124564d9e35a0bec61cc66852baccac9daac954f42c0bbeb99714101a756135` |

The two fences contain exact newline-terminated UTF-8 design objects. The
proposed source's bytes are the content of the `rust` fence; the diff's bytes
are the content of the `diff` fence. Do not copy a report fence into the actual
fixture before root's exact source reservation.

Static text proofs performed: original fixture hash/bytes; zero `unsafe`
tokens/global-allocator/raw-allocation or HeapWindow/HeapSample symbols in the
proposed source; exactly one ignored test; literal whole frozen method;
byte-exact original independent `expected`, `checked_read/refused_read`,
writer rollback functions, existing audit/member/revoke oracles, ordered
snapshot and actual-Core overlap bodies. Ordinary fixture setup, closed copies,
credentials, cleanup,300/10s policy and owned threads are retained. The normal
authority lab adds one copied directory: proposed normal success now has17
owned labs, **none created in this phase**. Both directions of the complete
text diff must reconstruct the original55,835-byte fixture and proposed
60,049-byte design; this establishes edit scope, not Rust typing or runtime.

`src/core.rs` remains57,913B SHA
`686e7e732f256e7b435ab69991b71fb26d7467214877d2e0f0c840741446caea`.
The production74c1 method and committed test55835B/b340155 are unchanged.
No Cargo/lock/toolchain or lint setting is edited. No new crate/function has
executed. The ordinary watch/join/drop and hard-IO limits remain those in the
previous report:300s cooperative/watchdog policy and a separately reviewed
outer supervisor, not a kernel IO/memory quota or an executed cleanup promise.

Whole proposed source:

```rust
//! One local encrypted-redb comparison. No service, clock override or product hook.
//! Existing storage histograms measure raw rows retained by each fetch, not heap
//! bytes, peak decoded/output memory or RSS. Timers measure actual list calls.
//! All output is fixed labels and numeric data; no allocation policy is changed.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use riauth::{
    agent::{Agent, NewAgent, Permission},
    config::{self, Config},
    core::Core,
    crypto,
    error::{Error, Result},
    model::{Group, NewUser, User, UserPatch, UserView},
    store::{Tx, maintenance::PAGE},
    telemetry::{ReadContext, Sizes},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    hint::black_box,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex,
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tempfile::TempDir;
use zeroize::Zeroizing;

const PASSWORD: &str = "s02-synthetic-fixture-password-only";
const USERS: usize = 128;
const GROUPS: usize = 513;
const SELECTED: [usize; 3] = [0, 256, 512];
const WARMUPS: usize = 5;
const SAMPLES: usize = 10;
const FIXTURE_LIMIT: Duration = Duration::from_secs(300);
const THREAD_LIMIT: Duration = Duration::from_secs(10);
// Pinned public Sizes::render schema at fab6721; a changed schema refuses.
const SCAN_BOUNDS: [u64; 7] = [0, 1, 16, 128, 1_024, 8_192, 65_536];

#[derive(Clone, Copy, PartialEq, Eq)]
struct ScanHistogram {
    buckets: [u64; 8], // cumulative finite bounds above, followed by +Inf
    count: u64,
    rows: u64,
}
impl ScanHistogram {
    fn parse(text: &str) -> Option<Self> {
        if text.len() > 4096 || text.lines().count() != 10 {
            return None;
        }
        let mut buckets = [None; 8];
        let mut count = None;
        let mut rows = None;
        for line in text.lines() {
            let (name, value) = line.rsplit_once(' ')?;
            let number: u64 = value.parse().ok()?;
            if number.to_string() != value {
                return None;
            }
            let slot = if name == "s02_scan_rows_count" {
                &mut count
            } else if name == "s02_scan_rows_sum" {
                &mut rows
            } else {
                let index = (0..8).find(|index| {
                    let bound = if *index == 7 {
                        "+Inf".to_owned()
                    } else {
                        SCAN_BOUNDS[*index].to_string()
                    };
                    name == format!("s02_scan_rows_bucket{{le=\"{bound}\"}}")
                })?;
                &mut buckets[index]
            };
            if slot.replace(number).is_some() {
                return None;
            }
        }
        let buckets: [u64; 8] = buckets
            .into_iter()
            .collect::<Option<Vec<_>>>()?
            .try_into()
            .ok()?;
        let count = count?;
        let rows = rows?;
        if buckets[7] != count || buckets.windows(2).any(|pair| pair[0] > pair[1]) {
            return None;
        }
        Some(Self {
            buckets,
            count,
            rows,
        })
    }
    fn read(sizes: &Sizes) -> Self {
        let mut text = String::new();
        sizes.render(&mut text, "s02_scan_rows", "");
        let value = Self::parse(&text).expect("native scan histogram schema");
        assert!(
            value.count == sizes.count() && value.rows == sizes.sum(),
            "native scan histogram counter consistency"
        );
        value
    }
    fn delta(self, before: Self) -> Self {
        Self {
            buckets: std::array::from_fn(|index| {
                self.buckets[index]
                    .checked_sub(before.buckets[index])
                    .expect("native scan bucket went backwards")
            }),
            count: self
                .count
                .checked_sub(before.count)
                .expect("native scan count went backwards"),
            rows: self
                .rows
                .checked_sub(before.rows)
                .expect("native scan rows went backwards"),
        }
    }
    fn expected(sizes: &[u64]) -> Self {
        Self {
            buckets: std::array::from_fn(|index| {
                sizes
                    .iter()
                    .filter(|size| index == 7 || **size <= SCAN_BOUNDS[index])
                    .count() as u64
            }),
            count: sizes.len() as u64,
            rows: sizes.iter().sum(),
        }
    }
    fn over_128(self) -> u64 {
        self.count
            .checked_sub(self.buckets[3])
            .expect("native 128 bucket exceeds count")
    }
}
fn telemetry_self_check() {
    // Exercise the actual safe public observer, independent literal expected
    // frequencies, and strict parser refusals; no candidate list code is run.
    let sizes = Sizes::default();
    assert!(
        ScanHistogram::read(&sizes) == ScanHistogram::expected(&[]),
        "empty native histogram"
    );
    for size in [128, 128, 128, 128, 1] {
        sizes.observe(size);
    }
    let actual = ScanHistogram::read(&sizes);
    assert!(
        actual
            == ScanHistogram {
                buckets: [0, 1, 1, 5, 5, 5, 5, 5],
                count: 5,
                rows: 513,
            },
        "literal native histogram self-check"
    );
    let mut text = String::new();
    sizes.render(&mut text, "s02_scan_rows", "");
    let duplicate = text.replacen(
        "s02_scan_rows_bucket{le=\"0\"} 0",
        "s02_scan_rows_bucket{le=\"1\"} 0",
        1,
    );
    let malformed = text.replacen("s02_scan_rows_sum 513", "s02_scan_rows_sum false", 1);
    let noncanonical = text.replacen("s02_scan_rows_sum 513", "s02_scan_rows_sum 0513", 1);
    let inconsistent = text.replacen("s02_scan_rows_count 5", "s02_scan_rows_count 6", 1);
    let missing = text.replacen("s02_scan_rows_count 5\n", "", 1);
    assert!(
        [duplicate, malformed, noncanonical, inconsistent, missing]
            .iter()
            .all(|case| ScanHistogram::parse(case).is_none()),
        "native histogram parser refusal"
    );
    println!(
        "{}",
        json!({"schema":"riauth.s02-telemetry-self-check/v1","scan_count":5,"rows":513,"buckets":[0,1,1,5,5,5,5,5],"parser_refusal_cases":5,"heap_claim":false,"rss_claim":false})
    );
}

// Whole frozen method is byte-identical to cdebc/d644/fab6721 list_groups.
// Deref supplies Core's actual Store and public principal; no frozen authority
// helper or copied scan implementation can replace the live shared service.
struct Frozen<'a>(&'a Core);
impl Deref for Frozen<'_> {
    type Target = Core;
    fn deref(&self) -> &Core {
        self.0
    }
}
impl Frozen<'_> {
    pub fn list_groups(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(
                tx.list::<Group>("groups")?
                    .into_iter()
                    .filter(|(_, u)| actor.allows("group.read", &format!("group/{}", u.name)))
                    .map(|(_, g)| g)
                    .collect::<Vec<_>>()
            ))
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lane {
    Control,
    Paged,
}
impl Lane {
    fn label(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Paged => "paged",
        }
    }
    fn list(self, core: &Core, token: &str) -> Result<Value> {
        match self {
            Self::Control => Frozen(core).list_groups(token),
            Self::Paged => core.list_groups(token),
        }
    }
}

#[derive(Clone, Copy)]
struct Budget(Instant);
impl Budget {
    fn check(self) {
        assert!(Instant::now() < self.0, "s02 fixture deadline");
    }
    fn thread_wait(self) -> Duration {
        self.check();
        self.0
            .saturating_duration_since(Instant::now())
            .min(THREAD_LIMIT)
    }
}

// Cooperative checks alone cannot interrupt stalled filesystem IO or join.
// The parked owned watchdog requests non-coredumping process exit124 at 300s.
// A future outer supervisor must enforce its deadline even if exit/IO stalls,
// retain that failure and clean only its private fixture root.
// Normal success/unwind cancels and joins this thread. No product hook/clock.
struct Watchdog {
    stop: Arc<(Mutex<bool>, Condvar)>,
    handle: Option<JoinHandle<()>>,
}
impl Watchdog {
    fn new(budget: Budget) -> Self {
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let state = stop.clone();
        let (ready, receiver) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            let (lock, wake) = &*state;
            let mut stopped = lock.lock().unwrap_or_else(|_| std::process::exit(124));
            if ready.send(()).is_err() {
                return;
            }
            while !*stopped {
                let remaining = budget.0.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    std::process::exit(124);
                }
                let (next, _) = wake
                    .wait_timeout(stopped, remaining)
                    .unwrap_or_else(|_| std::process::exit(124));
                stopped = next;
            }
        });
        let owned = Self {
            stop,
            handle: Some(handle),
        };
        require(
            receiver.recv_timeout(budget.thread_wait()),
            "watchdog startup",
        );
        // Acquiring after the rendezvous proves watchdog released the mutex
        // into its wait; thread/TLS startup is outside every measured window.
        drop(
            owned
                .stop
                .0
                .lock()
                .unwrap_or_else(|_| std::process::exit(124)),
        );
        owned
    }
}
impl Drop for Watchdog {
    fn drop(&mut self) {
        *self
            .stop
            .0
            .lock()
            .unwrap_or_else(|_| std::process::exit(124)) = true;
        self.stop.1.notify_one();
        if let Some(handle) = self.handle.take() {
            if handle.join().is_err() {
                std::process::exit(124);
            }
        }
    }
}

struct OwnedWorker<T> {
    handle: Option<JoinHandle<T>>,
}
impl<T> OwnedWorker<T> {
    fn new(work: impl FnOnce() -> T + Send + 'static) -> Self
    where
        T: Send + 'static,
    {
        Self {
            handle: Some(thread::spawn(work)),
        }
    }
    fn join(mut self) -> T {
        require(
            self.handle.take().expect("owned worker").join(),
            "owned worker panic",
        )
    }
}
impl<T> Drop for OwnedWorker<T> {
    fn drop(&mut self) {
        // Workers have bounded channel waits. If IO never completes, the
        // process deadline above bounds this join; never silently detach.
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn require<T, E>(result: std::result::Result<T, E>, label: &'static str) -> T {
    result.unwrap_or_else(|_| panic!("{label}"))
}
fn core_ok<T>(result: Result<T>, label: &'static str) -> T {
    result.unwrap_or_else(|error| {
        panic!(
            "{label}: status={} code={}",
            error.status.as_u16(),
            error.code
        )
    })
}
type Snapshot = BTreeMap<String, Value>;
fn snapshot(core: &Core) -> Snapshot {
    core_ok(core.store.read(|tx| tx.snapshot()), "full snapshot")
}
fn same_snapshot(core: &Core, expected: &Snapshot) {
    assert!(snapshot(core) == *expected, "full durable snapshot changed");
}
fn token(value: &Value) -> Zeroizing<String> {
    Zeroizing::new(
        value["credential"]["token"]
            .as_str()
            .expect("private credential response")
            .to_owned(),
    )
}
fn user_id(index: usize) -> String {
    format!("s02-user-{index:03}")
}
fn user_name(index: usize) -> String {
    format!("s02-person-{index:03}")
}
fn group_name(index: usize) -> String {
    format!("s02-group-{index:03}")
}
fn expected(count: usize, scoped: bool, removed: Option<usize>) -> Value {
    // Construct from fixture constants, not observed output or the frozen code.
    Value::Array(
        (0..count)
            .filter(|index| !scoped || SELECTED.contains(index))
            .map(|index| {
                let members: Vec<_> = (0..USERS)
                    .filter(|member| !(removed == Some(index) && *member == 0))
                    .map(user_id)
                    .collect();
                json!({"name":group_name(index),"members":members})
            })
            .collect(),
    )
}

struct Credentials {
    admin: Zeroizing<String>,
    scoped: Zeroizing<String>,
    empty: Zeroizing<String>,
}
impl Credentials {
    fn copy(&self) -> Self {
        Self {
            admin: self.admin.clone(),
            scoped: self.scoped.clone(),
            empty: self.empty.clone(),
        }
    }
}
struct Fixture {
    core: Core, // close Store before deleting owned directory
    _dir: TempDir,
    credentials: Credentials,
}
struct Seed {
    _dir: TempDir,
    config: Config,
    credentials: Credentials,
    state: Snapshot,
}
fn close_dir(directory: TempDir) {
    let path = directory.path().to_path_buf();
    require(directory.close(), "owned fixture directory cleanup");
    assert!(
        !require(path.try_exists(), "owned fixture cleanup postcondition"),
        "owned fixture directory remains"
    );
}
impl Fixture {
    fn close(self) {
        let Self {
            core,
            _dir,
            credentials,
        } = self;
        drop(core);
        drop(credentials);
        close_dir(_dir);
    }
}
fn private_dir(root: &Path) -> TempDir {
    require(
        tempfile::Builder::new()
            .prefix("s02-group-list-")
            .tempdir_in(root),
        "owned private fixture directory",
    )
}
fn fixture_root() -> PathBuf {
    let selected = require(
        std::env::var_os("CARGO_TARGET_DIR").ok_or(()),
        "private target required",
    );
    let root = require(
        PathBuf::from(selected).canonicalize(),
        "private target identity",
    );
    let own = require(
        require(std::env::current_dir(), "worktree identity")
            .join("target")
            .canonicalize(),
        "own target identity",
    );
    assert!(root == own, "refuse target outside own worktree");
    root
}
impl Seed {
    fn new(root: &Path, budget: Budget) -> Self {
        budget.check();
        let dir = private_dir(root);
        let key = dir.path().join("storage.key");
        require(
            config::write_private(&key, crypto::random_token("").as_bytes(), false),
            "private encryption key",
        );
        let config = Config {
            data_dir: dir.path().join("data"),
            database_key_file: Some(key),
            ..Default::default()
        };
        let core = core_ok(
            Core::initialize(
                config.clone(),
                NewUser {
                    username: "admin".into(),
                    password: PASSWORD.into(),
                    email: None,
                    display_name: "S02 synthetic administrator".into(),
                    admin: true,
                },
            ),
            "encrypted seed initialize",
        );
        let login = core_ok(
            core.login("admin".into(), PASSWORD.into(), None),
            "seed login",
        );
        let admin = Zeroizing::new(
            login["session_token"]
                .as_str()
                .expect("private login response")
                .into(),
        );
        let template: User = core_ok(
            core.store.read(|tx| {
                let id = tx
                    .get::<String>("usernames", "admin")?
                    .ok_or_else(Error::unauthorized)?;
                tx.get("users", &id)?.ok_or_else(Error::unauthorized)
            }),
            "seed User template",
        );
        core_ok(
            core.store.write(|tx| {
                for index in 0..USERS {
                    budget.check();
                    let mut user = template.clone();
                    user.id = user_id(index);
                    user.username = user_name(index);
                    user.admin = false;
                    user.display_name = user.username.clone();
                    tx.put("users", &user.id, &user)?;
                    tx.put("usernames", &user.username, &user.id)?;
                }
                Ok(())
            }),
            "synthetic stable Users",
        );
        let scoped = token(&core_ok(
            core.create_agent(
                &admin,
                NewAgent {
                    id: "s02-scoped".into(),
                    ttl: 3600,
                    parent: None,
                    permissions: SELECTED
                        .iter()
                        .map(|index| Permission {
                            action: "group.read".into(),
                            resource: format!("group/{}", group_name(*index)),
                        })
                        .collect(),
                },
            ),
            "scoped public credential creation",
        ));
        let empty = token(&core_ok(
            core.create_agent(
                &admin,
                NewAgent {
                    id: "s02-empty".into(),
                    ttl: 3600,
                    parent: None,
                    permissions: vec![Permission {
                        action: "group.read".into(),
                        resource: "group/s02-not-present".into(),
                    }],
                },
            ),
            "empty-scope public credential creation",
        ));
        // Scoped credentials may add accepted edition-provenance observations
        // on the next normal open. Settle that PUBLIC startup once before the
        // closed seed snapshot; copied lanes must match it with no exclusions.
        drop(core);
        let core = core_ok(Core::open(config.clone()), "seed public startup");
        let state = snapshot(&core);
        drop(core); // closed file is the only seed copy source
        Self {
            _dir: dir,
            config,
            credentials: Credentials {
                admin,
                scoped,
                empty,
            },
            state,
        }
    }
    fn open(&self, root: &Path, budget: Budget) -> Fixture {
        budget.check();
        let dir = private_dir(root);
        let mut config = self.config.clone();
        config.data_dir = dir.path().join("data");
        require(
            config::private_dir(&config.data_dir),
            "private copied data directory",
        );
        require(
            std::fs::copy(
                self.config.data_dir.join("riauth.redb"),
                config.data_dir.join("riauth.redb"),
            ),
            "closed seed copy",
        );
        let core = core_ok(Core::open(config), "encrypted copied fixture open");
        same_snapshot(&core, &self.state);
        let mut actual = core.config.clone();
        actual.data_dir = self.config.data_dir.clone();
        assert!(
            require(serde_json::to_value(actual), "copied configuration")
                == require(serde_json::to_value(&self.config), "seed configuration"),
            "normalized configuration changed"
        );
        Fixture {
            core,
            _dir: dir,
            credentials: self.credentials.copy(),
        }
    }
    fn close(fixture: Fixture) -> Self {
        let Fixture {
            core,
            _dir,
            credentials,
        } = fixture;
        let config = core.config.clone();
        let state = snapshot(&core);
        drop(core);
        Self {
            _dir,
            config,
            credentials,
            state,
        }
    }
    fn cleanup(self) {
        let Self {
            _dir,
            config,
            credentials,
            state,
        } = self;
        drop(state);
        drop(credentials);
        drop(config);
        close_dir(_dir);
    }
}

fn populate(core: &Core, count: usize, budget: Budget) {
    core_ok(
        core.store.write(|tx| {
            let members: BTreeSet<_> = (0..USERS).map(user_id).collect();
            for index in 0..count {
                budget.check();
                let name = group_name(index);
                tx.put(
                    "groups",
                    &name,
                    &Group {
                        name: name.clone(),
                        members: members.clone(),
                    },
                )?;
            }
            Ok(())
        }),
        "synthetic canonical Groups",
    );
}

#[derive(Clone, Copy)]
struct Counters {
    points: u64,
    bytes: u64,
    bounded: u64,
    bounded_rows: u64,
    unbounded: u64,
    unbounded_rows: u64,
    bounded_histogram: ScanHistogram,
    unbounded_histogram: ScanHistogram,
    writer_holds: u64,
    commits: u64,
}
impl Counters {
    fn read(core: &Core) -> Self {
        let telemetry = core.store.telemetry();
        let reads = &telemetry.reads;
        let bounded_histogram = ScanHistogram::read(reads.scans(ReadContext::Read, true));
        let unbounded_histogram = ScanHistogram::read(reads.scans(ReadContext::Read, false));
        Self {
            points: reads.points(ReadContext::Read),
            bytes: reads.bytes(ReadContext::Read),
            bounded: bounded_histogram.count,
            bounded_rows: bounded_histogram.rows,
            unbounded: unbounded_histogram.count,
            unbounded_rows: unbounded_histogram.rows,
            bounded_histogram,
            unbounded_histogram,
            writer_holds: telemetry.write_hold.count(),
            commits: telemetry.commit.count(),
        }
    }
    fn delta(self, before: Self) -> Self {
        let subtract = |after: u64, before: u64| {
            after.checked_sub(before).expect("native counter went backwards")
        };
        Self {
            points: subtract(self.points, before.points),
            bytes: subtract(self.bytes, before.bytes),
            bounded: subtract(self.bounded, before.bounded),
            bounded_rows: subtract(self.bounded_rows, before.bounded_rows),
            unbounded: subtract(self.unbounded, before.unbounded),
            unbounded_rows: subtract(self.unbounded_rows, before.unbounded_rows),
            bounded_histogram: self.bounded_histogram.delta(before.bounded_histogram),
            unbounded_histogram: self.unbounded_histogram.delta(before.unbounded_histogram),
            writer_holds: subtract(self.writer_holds, before.writer_holds),
            commits: subtract(self.commits, before.commits),
        }
    }
    fn assert_read(self, lane: Lane, count: usize) {
        assert!(
            self.writer_holds == 0 && self.commits == 0,
            "reader acquired writer or committed"
        );
        let empty = ScanHistogram::expected(&[]);
        match lane {
            Lane::Control => {
                assert!(
                    (
                        self.bounded,
                        self.bounded_rows,
                        self.unbounded,
                        self.unbounded_rows
                    ) == (0, 0, 1, count as u64),
                    "control complete scan accounting"
                );
                assert!(
                    self.bounded_histogram == empty
                        && self.unbounded_histogram == ScanHistogram::expected(&[count as u64]),
                    "control exact native row histogram"
                );
            }
            Lane::Paged => {
                assert!(
                    (
                        self.bounded,
                        self.bounded_rows,
                        self.unbounded,
                        self.unbounded_rows
                    ) == ((count / PAGE + 1) as u64, count as u64, 0, 0),
                    "paged exact native scan accounting"
                );
                let mut sizes = vec![PAGE as u64; count / PAGE];
                sizes.push((count % PAGE) as u64);
                assert!(
                    self.unbounded_histogram == empty
                        && self.bounded_histogram == ScanHistogram::expected(&sizes),
                    "paged exact native row histogram"
                );
            }
        }
    }
    fn fetches_over_128(self) -> u64 {
        self.bounded_histogram.over_128() + self.unbounded_histogram.over_128()
    }
}

fn checked_read(
    core: &Core,
    lane: Lane,
    credential: &str,
    oracle: &Value,
    count: usize,
    budget: Budget,
) -> Counters {
    budget.check();
    let state = snapshot(core);
    let before = Counters::read(core);
    let value = core_ok(lane.list(core, credential), "list result");
    let delta = Counters::read(core).delta(before);
    assert!(value == *oracle, "complete independent Group JSON oracle");
    delta.assert_read(lane, count);
    same_snapshot(core, &state);
    budget.check();
    delta
}

fn refused_read(core: &Core, lane: Lane, credential: &str, budget: Budget) {
    budget.check();
    let state = snapshot(core);
    let before = Counters::read(core);
    let error = match lane.list(core, credential) {
        Err(error) => error,
        Ok(_) => panic!("retired or invalid credential accepted"),
    };
    assert!(
        error.status.as_u16() == 401 && error.code == "invalid_token",
        "exact authority refusal"
    );
    let delta = Counters::read(core).delta(before);
    assert!(
        delta.bounded == 0 && delta.unbounded == 0,
        "authority refusal traversed Groups"
    );
    assert!(
        delta.writer_holds == 0 && delta.commits == 0,
        "authority refusal mutated"
    );
    same_snapshot(core, &state);
    budget.check();
}

fn denied_member_write(fixture: &Fixture, budget: Budget) {
    budget.check();
    let before = snapshot(&fixture.core);
    let error = match fixture.core.group_member(
        &fixture.credentials.scoped,
        &group_name(512),
        &user_name(0),
        false,
    ) {
        Err(error) => error,
        Ok(_) => panic!("read-only principal mutated membership"),
    };
    assert!(
        error.status.as_u16() == 403 && error.code == "access_denied",
        "exact Group writer scope refusal"
    );
    same_snapshot(&fixture.core, &before);
    budget.check();
}

fn malformed_write_refusal(fixture: &Fixture, budget: Budget) {
    budget.check();
    let before = snapshot(&fixture.core);
    let result = fixture.core.store.write(|tx| {
        tx.put(
            "groups",
            "s02-malformed",
            &json!({"name":"s02-malformed","members":17}),
        )
    });
    let error = match result {
        Err(error) => error,
        Ok(()) => panic!("malformed Group was accepted"),
    };
    assert!(
        error.status.as_u16() == 500 && error.code == "server_error",
        "canonical Group decode refusal"
    );
    same_snapshot(&fixture.core, &before);
    // No raw import or malformed persisted row is used to create this case.
    // The shared writer rejects it; scan decode errors still propagate in
    // both source bodies, rather than an invented corruption-runtime claim.
    budget.check();
}

#[derive(Clone, Copy)]
struct Observation {
    pair: usize,
    lane: Lane,
    scoped: bool,
    sample: usize,
    elapsed_ns: u64,
    native: Counters,
}
fn emit(observation: Observation) {
    let Observation {
        pair,
        lane,
        scoped,
        sample,
        elapsed_ns,
        native,
    } = observation;
    println!(
        "{}",
        json!({
            "schema":"riauth.s02-group-list-sample/v2", "pair":pair,
            "lane":lane.label(), "scope":if scoped {"scoped"} else {"admin"},
            "sample":sample,"elapsed_ns":elapsed_ns,
            "point_reads":native.points,"native_point_plus_scan_value_bytes":native.bytes,
            "bounded_scans":native.bounded,"bounded_rows":native.bounded_rows,
            "unbounded_scans":native.unbounded,"unbounded_rows":native.unbounded_rows,
            "row_histogram_bounds":[0,1,16,128,1024,8192,65536],
            "bounded_cumulative_buckets":native.bounded_histogram.buckets,
            "unbounded_cumulative_buckets":native.unbounded_histogram.buckets,
            "fetches_over_128_rows":native.fetches_over_128(),
            "writer_holds":native.writer_holds,"commits":native.commits,
            "encrypted_at_rest":true,"latency_interval":"complete_list_call",
            "heap_claim":false,"rss_claim":false
        })
    );
}

fn measure(
    fixture: &Fixture,
    lane: Lane,
    pair: usize,
    scoped: bool,
    observations: &mut Vec<Observation>,
    budget: Budget,
) {
    let credential = if scoped {
        &fixture.credentials.scoped
    } else {
        &fixture.credentials.admin
    };
    let oracle = expected(GROUPS, scoped, None);
    let state = snapshot(&fixture.core);
    for _ in 0..WARMUPS {
        checked_read(&fixture.core, lane, credential, &oracle, GROUPS, budget);
    }
    for sample in 0..SAMPLES {
        budget.check();
        let before = Counters::read(&fixture.core);
        let start = Instant::now();
        let result = black_box(lane.list(black_box(&fixture.core), black_box(credential)));
        let elapsed = start.elapsed();
        // Returned complete JSON stays live. Observer rendering, grading,
        // snapshots and returned-value drop are outside the call timer.
        let native = Counters::read(&fixture.core).delta(before);
        let observation = Observation {
            pair,
            lane,
            scoped,
            sample,
            elapsed_ns: require(u64::try_from(elapsed.as_nanos()), "elapsed numeric bound"),
            native,
        };
        emit(observation); // finite raw numeric outcome before evidence grading
        observations.push(observation);
        let value = core_ok(result, "measured list result");
        assert!(value == oracle, "measured complete independent JSON oracle");
        native.assert_read(lane, GROUPS);
        same_snapshot(&fixture.core, &state);
        drop(value);
        budget.check();
    }
    same_snapshot(&fixture.core, &state);
}

fn percentile(mut values: Vec<u64>, numerator: usize, denominator: usize) -> u64 {
    assert!(!values.is_empty(), "empty measurement summary");
    values.sort_unstable();
    let rank = (values.len() * numerator).div_ceil(denominator);
    values[rank.saturating_sub(1)]
}
fn summaries(observations: &[Observation]) {
    let mut fetch_materialization_improved = true;
    let mut native_equivalent = true;
    let mut scoped_latency_p50_lower_in_every_pair = true;
    for pair in 0..3 {
        for scoped in [true, false] {
            let select = |lane| {
                observations
                    .iter()
                    .filter(move |row| row.pair == pair && row.scoped == scoped && row.lane == lane)
            };
            for lane in [Lane::Control, Lane::Paged] {
                let selected: Vec<_> = select(lane).collect();
                assert!(selected.len() == SAMPLES, "complete sample count");
                println!(
                    "{}",
                    json!({
                        "schema":"riauth.s02-group-list-summary/v2","pair":pair,"lane":lane.label(),
                        "scope":if scoped {"scoped"} else {"admin"},"samples":selected.len(),
                        "median_fetches_over_128_rows":percentile(selected.iter().map(|row| row.native.fetches_over_128()).collect(),1,2),
                        "p50_complete_call_elapsed_ns":percentile(selected.iter().map(|row| row.elapsed_ns).collect(),1,2),
                        "p95_complete_call_elapsed_ns":percentile(selected.iter().map(|row| row.elapsed_ns).collect(),95,100),
                        "heap_claim":false,"rss_claim":false
                    })
                );
            }
            let control_native = select(Lane::Control).next().expect("control sample").native;
            for row in select(Lane::Paged).chain(select(Lane::Control)) {
                native_equivalent &= row.native.points == control_native.points
                    && row.native.bytes == control_native.bytes;
                // These are actual native frequency observations, not a
                // PAGE-based prediction or a memory/throughput inference.
                fetch_materialization_improved &= match row.lane {
                    Lane::Control => row.native.fetches_over_128() == 1
                        && row.native.unbounded == 1
                        && row.native.unbounded_rows == 513,
                    Lane::Paged => row.native.fetches_over_128() == 0
                        && row.native.bounded == 5
                        && row.native.bounded_rows == 513,
                };
            }
            if scoped {
                let p50 = |lane| {
                    percentile(select(lane).map(|row| row.elapsed_ns).collect(), 1, 2)
                };
                scoped_latency_p50_lower_in_every_pair &= p50(Lane::Paged) < p50(Lane::Control);
            }
        }
    }
    // Every raw row and every pair/scope summary precedes the decision. A
    // latency regression remains visible and is never called a speedup.
    println!(
        "{}",
        json!({"schema":"riauth.s02-measurement-decision/v2","native_point_and_value_byte_equivalence":native_equivalent,"observed_fetch_materialization_improved":fetch_materialization_improved,"scoped_latency_p50_lower_in_every_pair":scoped_latency_p50_lower_in_every_pair,"latency_grade":false,"heap_claim":false,"rss_claim":false})
    );
    assert!(
        native_equivalent,
        "security point reads or total stored bytes differ"
    );
    assert!(
        fetch_materialization_improved,
        "measured raw-fetch materialization did not improve"
    );
}

fn source_digest(group: &Value) -> String {
    let typed: Group = require(serde_json::from_value(group.clone()), "expected Group type");
    URL_SAFE_NO_PAD.encode(Sha256::digest(require(
        serde_json::to_vec(&typed),
        "expected Group serialization",
    )))
}

// Exact full-snapshot oracle: only nondeterministic audit id/time are read from
// the new event, then independently type/bound/key checked. Entire event schema,
// changes, revision and every other durable value are constructed or retained.
fn expected_audit(
    before: &Snapshot,
    after: &Snapshot,
    actor: &str,
    action: &str,
    target: &str,
    changes: Value,
    start: u64,
    end: u64,
) -> (String, Value) {
    let added: Vec<_> = after
        .iter()
        .filter(|(key, _)| key.starts_with("audit/") && !before.contains_key(*key))
        .collect();
    assert!(added.len() == 1, "writer must emit one new audit");
    let (key, row) = added[0];
    let id = row["id"].as_str().expect("audit id type");
    let uuid = require(uuid::Uuid::parse_str(id), "audit id format");
    assert!(uuid.to_string() == id, "canonical audit id");
    let at = row["at"].as_u64().expect("audit time type");
    assert!(
        start <= at && at <= end,
        "audit time outside real writer interval"
    );
    assert!(*key == format!("audit/{at:020}-{id}"), "audit key binding");
    let event = json!({"id":id,"at":at,"actor":actor,"action":action,"target":target,"run_id":null,"details":{"request_id":null,"changes":changes}});
    assert!(*row == event, "complete writer audit schema and content");
    (key.clone(), event)
}
fn admin_id(before: &Snapshot) -> &str {
    before["usernames/admin"].as_str().expect("admin stable id")
}
fn advance_revision(expected: &mut Snapshot) {
    let revision = expected["meta/revision"]
        .as_u64()
        .expect("management revision type");
    expected.insert("meta/revision".into(), json!(revision + 1));
}
fn assert_member_effect(
    before: &Snapshot,
    after: &Snapshot,
    index: usize,
    present: bool,
    start: u64,
    end: u64,
) {
    let name = group_name(index);
    let uid = user_id(0);
    let key = format!("groups/{name}");
    let old = before[&key].clone();
    let mut group: Group = require(serde_json::from_value(old.clone()), "expected old Group");
    if present {
        assert!(
            group.members.insert(uid.clone()),
            "expected membership addition"
        );
    } else {
        assert!(group.members.remove(&uid), "expected membership removal");
    }
    let new = require(serde_json::to_value(group), "expected new Group");
    let digest = source_digest(&new);
    let group_hash = crypto::digest(&name);
    let user_hash = crypto::digest(&uid);
    let mut expected = before.clone();
    expected.insert(key.clone(), new.clone());
    expected.insert(
        format!("index_group_bindings/{name}"),
        json!({"name":name,"source_digest":digest}),
    );
    expected.insert(format!("index_group_source_digests/{name}"), json!(digest));
    let user_group = format!("index_user_groups/{user_hash}/{group_hash}");
    let member = format!("index_group_members/{group_hash}/{user_hash}");
    if present {
        expected.insert(user_group, json!(name));
        expected.insert(member, json!(uid)); // Some(String) serializes as String
    } else {
        expected.remove(&user_group);
        expected.remove(&member);
    }
    advance_revision(&mut expected);
    let action = if present {
        "group.member.add"
    } else {
        "group.member.remove"
    };
    let (audit_key, event) = expected_audit(
        before,
        after,
        admin_id(before),
        action,
        &format!("{name}/{uid}"),
        json!([{"resource":key,"before":old,"after":new}]),
        start,
        end,
    );
    expected.insert(audit_key, event);
    assert!(
        expected == *after,
        "complete normal membership writer snapshot"
    );
}
fn assert_revoke_effect(
    before: &Snapshot,
    after: &Snapshot,
    credential: &str,
    start: u64,
    end: u64,
) {
    let key = "agents/s02-scoped";
    let agent: Agent = require(
        serde_json::from_value(before[key].clone()),
        "expected old Agent",
    );
    let mut old_view = agent.view();
    old_view["agent_credential"] = json!("[redacted]");
    let mut new_view = old_view.clone();
    new_view["enabled"] = json!(false);
    let mut expected = before.clone();
    expected.get_mut(key).expect("known Agent")["enabled"] = json!(false);
    assert!(
        expected
            .remove(&format!("agent_tokens/{}", crypto::digest(credential)))
            .is_some(),
        "retired exact agent token mapping"
    );
    advance_revision(&mut expected);
    let (audit_key, event) = expected_audit(
        before,
        after,
        admin_id(before),
        "agent.revoke",
        "s02-scoped",
        json!([{"resource":key,"before":old_view,"after":new_view}]),
        start,
        end,
    );
    expected.insert(audit_key, event);
    assert!(
        expected == *after,
        "complete normal revocation writer snapshot"
    );
}

// Expiry and parent liveness stay in the shared principal. The expiry row is
// a declared trusted compatibility fixture, not elapsed-TTL or remote evidence.
// Parent disable uses the real public writer; it also retires the child token.
fn authority_agent(
    fixture: &Fixture,
    id: &str,
    parent: Option<String>,
) -> Zeroizing<String> {
    token(&core_ok(
        fixture.core.create_agent(
            &fixture.credentials.admin,
            NewAgent {
                id: id.into(),
                ttl: 3600,
                parent,
                permissions: SELECTED
                    .iter()
                    .map(|index| Permission {
                        action: "group.read".into(),
                        resource: format!("group/{}", group_name(*index)),
                    })
                    .collect(),
            },
        ),
        "authority fixture public credential creation",
    ))
}
fn expected_user_audit(user: &User) -> Value {
    assert!(
        user.totp_secret.is_none()
            && user.totp_pending.is_none()
            && user.recovery_codes.is_empty(),
        "factor-free parent fixture required"
    );
    let mut view = require(serde_json::to_value(UserView::from(user)), "expected User view");
    // public_record + fixed sensitive-name redaction at the pinned Store.
    view["password_available"] = json!("[redacted]");
    if !user.password_hash.is_empty() {
        view["password"] = json!("[redacted]");
    }
    if !user.pairwise_seed.is_empty() {
        view["pairwise_seed"] = json!("[redacted]");
    }
    view
}
fn assert_parent_disable_effect(
    before: &Snapshot,
    after: &Snapshot,
    credential: &str,
    start: u64,
    end: u64,
) {
    let user_key = format!("users/{}", user_id(0));
    let old_user: User = require(
        serde_json::from_value(before[&user_key].clone()),
        "expected enabled parent",
    );
    assert!(old_user.enabled && !old_user.admin, "enabled non-admin parent");
    let mut new_user = old_user.clone();
    new_user.enabled = false;
    new_user.epoch = old_user.epoch.checked_add(1).expect("parent epoch bound");
    let agent_key = "agents/s02-parent-owned";
    let old_agent: Agent = require(
        serde_json::from_value(before[agent_key].clone()),
        "expected parent-owned Agent",
    );
    assert!(
        old_agent.enabled && old_agent.parent_user.as_deref() == Some(old_user.id.as_str()),
        "exact active parent binding"
    );
    let mut old_view = old_agent.view();
    old_view["agent_credential"] = json!("[redacted]");
    let mut new_view = old_view.clone();
    new_view["enabled"] = json!(false);
    let mut expected = before.clone();
    expected.insert(
        user_key.clone(),
        require(serde_json::to_value(&new_user), "expected disabled parent"),
    );
    expected.get_mut(agent_key).expect("known child Agent")["enabled"] = json!(false);
    assert!(
        expected
            .remove(&format!("agent_tokens/{}", crypto::digest(credential)))
            .is_some(),
        "parent disable retired exact child token"
    );
    for key in ["provisioning_user_generation/all", "user_listing_generation/all"] {
        let previous = expected.get(key).and_then(Value::as_u64).unwrap_or(0);
        expected.insert(
            key.into(),
            json!(previous.checked_add(1).expect("parent generation bound")),
        );
    }
    advance_revision(&mut expected);
    // BTreeMap-backed change records sort agents before users. This fixture has
    // no owner RP/session, Windows device, grants, downstream link/job or SSF
    // stream; any unexpected durable consequence fails the COMPLETE map.
    let changes = json!([
        {"resource":agent_key,"before":old_view,"after":new_view},
        {"resource":user_key,"before":expected_user_audit(&old_user),"after":expected_user_audit(&new_user)}
    ]);
    let (audit_key, event) = expected_audit(
        before,
        after,
        admin_id(before),
        "user.update",
        &old_user.id,
        changes,
        start,
        end,
    );
    expected.insert(audit_key, event);
    assert!(expected == *after, "complete public parent-disable snapshot");
}
fn authority_boundaries(fixture: &Fixture, budget: Budget) {
    budget.check();
    let expired = authority_agent(fixture, "s02-expiry-boundary", None);
    for lane in [Lane::Control, Lane::Paged] {
        checked_read(
            &fixture.core,
            lane,
            &expired,
            &expected(GROUPS, true, None),
            GROUPS,
            budget,
        );
    }
    let before = snapshot(&fixture.core);
    let key = "agents/s02-expiry-boundary";
    let mut record: Agent = require(
        serde_json::from_value(before[key].clone()),
        "expiry fixture canonical Agent",
    );
    assert!(record.enabled && record.parent_user.is_none(), "isolated expiry fixture");
    let boundary = crypto::now();
    record.expires_at = boundary;
    core_ok(
        fixture.core.store.write(|tx| tx.put("agents", &record.id, &record)),
        "trusted canonical expiry fixture",
    );
    let mut expected_state = before.clone();
    expected_state.get_mut(key).expect("known expiry Agent")["expires_at"] = json!(boundary);
    same_snapshot(&fixture.core, &expected_state);
    assert!(crypto::now() >= boundary, "real clock moved before expiry boundary");
    let same_second_observed = crypto::now() == boundary;
    for lane in [Lane::Control, Lane::Paged] {
        refused_read(&fixture.core, lane, &expired, budget);
    }
    // Token mapping remains valid and enabled; refusal therefore exercises the
    // stored expiration, rather than a missing token or an Agent disable.
    assert!(
        expected_state
            .get(&format!("agent_tokens/{}", crypto::digest(&expired)))
            .and_then(Value::as_str)
            == Some("s02-expiry-boundary"),
        "expired credential mapping retained"
    );
    let owned = authority_agent(fixture, "s02-parent-owned", Some(user_name(0)));
    for lane in [Lane::Control, Lane::Paged] {
        checked_read(
            &fixture.core,
            lane,
            &owned,
            &expected(GROUPS, true, None),
            GROUPS,
            budget,
        );
    }
    let before = snapshot(&fixture.core);
    let start = crypto::now();
    core_ok(
        fixture.core.update_user(
            &fixture.credentials.admin,
            &user_name(0),
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        ),
        "public parent disable",
    );
    let end = crypto::now();
    let after = snapshot(&fixture.core);
    assert_parent_disable_effect(&before, &after, &owned, start, end);
    for lane in [Lane::Control, Lane::Paged] {
        refused_read(&fixture.core, lane, &owned, budget);
        // Unrelated unparented authority and canonical Group bodies survive.
        checked_read(
            &fixture.core,
            lane,
            &fixture.credentials.scoped,
            &expected(GROUPS, true, None),
            GROUPS,
            budget,
        );
    }
    println!(
        "{}",
        json!({"schema":"riauth.s02-authority-boundaries/v1","expiry_source":"trusted_canonical_stored_timestamp_at_real_now","expiry_refusal_exact":true,"expiry_same_second_observed":same_second_observed,"natural_ttl_elapsed_claim":false,"public_parent_disable_exact_snapshot":true,"child_token_retired":true,"legacy_enabled_child_with_disabled_parent_runtime_claim":false,"unrelated_authority_preserved":true})
    );
    budget.check();
}

// Ordered underlying snapshot proof, deliberately distinct from the unhooked
// actual-Core overlap case. Every page uses the same Tx and captured Principal.
fn finish_snapshot(
    tx: &Tx<'_>,
    actor: &riauth::agent::Principal,
    first: Vec<(String, Group)>,
    budget: Budget,
) -> Result<Value> {
    let mut page = first;
    let mut values = Vec::new();
    loop {
        budget.check();
        let full = page.len() == PAGE;
        let after = page.last().map(|(key, _)| key.clone());
        for (_, group) in page {
            if actor.allows("group.read", &format!("group/{}", group.name)) {
                values.push(json!(group));
            }
        }
        if !full {
            break;
        }
        page = tx.scan("groups", after.as_deref(), PAGE)?;
        if page.is_empty() {
            break;
        }
    }
    Ok(Value::Array(values))
}

fn ordered_interleaving(fixture: &Fixture, revoke: bool, budget: Budget) {
    budget.check();
    let before = snapshot(&fixture.core);
    let writer_core = fixture.core.clone();
    let admin = fixture.credentials.admin.clone();
    let (start, go) = mpsc::sync_channel(0);
    let (finished, done) = mpsc::sync_channel(1);
    let worker = OwnedWorker::new(move || {
        require(
            go.recv_timeout(budget.thread_wait()),
            "ordered writer start",
        );
        let at = crypto::now();
        let result = if revoke {
            writer_core.revoke_agent(&admin, "s02-scoped")
        } else {
            writer_core.group_member(&admin, &group_name(512), &user_name(0), false)
        };
        let end = crypto::now();
        drop(writer_core);
        let _ = finished.send(result.is_ok());
        (result, at, end)
    });
    core_ok(
        fixture.core.store.read(|tx| {
            let actor = fixture.core.principal(tx, &fixture.credentials.scoped)?;
            let first = tx.scan::<Group>("groups", None, PAGE)?;
            assert!(first.len() == PAGE, "ordered first page");
            require(start.send(()), "ordered writer release");
            assert!(
                require(
                    done.recv_timeout(budget.thread_wait()),
                    "ordered writer result receipt"
                ),
                "ordered public writer did not commit"
            );
            let observed = finish_snapshot(tx, &actor, first, budget)?;
            assert!(
                observed == expected(GROUPS, true, None),
                "ordered snapshot retained whole old result"
            );
            Ok(())
        }),
        "ordered snapshot read",
    );
    let (result, at, end) = worker.join();
    core_ok(result, "ordered normal writer");
    let after = snapshot(&fixture.core);
    if revoke {
        assert_revoke_effect(&before, &after, &fixture.credentials.scoped, at, end);
        let duplicate = match fixture
            .core
            .revoke_agent(&fixture.credentials.admin, "s02-scoped")
        {
            Err(error) => error,
            Ok(_) => panic!("retired Agent accepted another revocation"),
        };
        assert!(
            duplicate.status.as_u16() == 409 && duplicate.code == "conflict",
            "repeat revocation conflict"
        );
        same_snapshot(&fixture.core, &after);
        for lane in [Lane::Control, Lane::Paged] {
            refused_read(&fixture.core, lane, &fixture.credentials.scoped, budget);
        }
    } else {
        assert_member_effect(&before, &after, 512, false, at, end);
        core_ok(
            fixture.core.group_member(
                &fixture.credentials.admin,
                &group_name(512),
                &user_name(0),
                false,
            ),
            "unchanged normal membership retry",
        );
        same_snapshot(&fixture.core, &after);
        for lane in [Lane::Control, Lane::Paged] {
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.scoped,
                &expected(GROUPS, true, Some(512)),
                GROUPS,
                budget,
            );
        }
    }
    println!(
        "{}",
        json!({"schema":"riauth.s02-ordered-snapshot/v1","operation":if revoke {"revoke"} else {"membership"},"writer_joined":true,"old_snapshot_exact":true,"fresh_request_checked":true})
    );
}

fn core_overlap(fixture: &Fixture, budget: Budget) {
    budget.check();
    let before = snapshot(&fixture.core);
    let old = expected(GROUPS, true, None);
    let new = expected(GROUPS, true, Some(512));
    let (start, go) = mpsc::sync_channel(0);
    let (finished, done): (_, Receiver<()>) = mpsc::sync_channel(1);
    let writer_core = fixture.core.clone();
    let admin = fixture.credentials.admin.clone();
    let worker = OwnedWorker::new(move || {
        require(
            go.recv_timeout(budget.thread_wait()),
            "overlap writer start",
        );
        let at = crypto::now();
        let call_start = Instant::now();
        let result = writer_core.group_member(&admin, &group_name(512), &user_name(0), false);
        let call_end = Instant::now();
        let end = crypto::now();
        drop(writer_core);
        let _ = finished.send(());
        (result, at, end, call_start, call_end)
    });
    require(start.send(()), "overlap writer release");
    let mut intervals = Vec::with_capacity(8);
    for _ in 0..8 {
        budget.check();
        let at_start = Instant::now();
        let value = core_ok(
            fixture.core.list_groups(&fixture.credentials.scoped),
            "concurrent actual Core reader",
        );
        let at_end = Instant::now();
        intervals.push((at_start, at_end));
        assert!(
            value == old || value == new,
            "concurrent Core returned mixed result"
        );
    }
    require(
        done.recv_timeout(budget.thread_wait()),
        "overlap writer completion",
    );
    let (result, at, end, call_start, call_end) = worker.join();
    core_ok(result, "concurrent normal writer");
    let after = snapshot(&fixture.core);
    assert_member_effect(&before, &after, 512, false, at, end);
    checked_read(
        &fixture.core,
        Lane::Paged,
        &fixture.credentials.scoped,
        &new,
        GROUPS,
        budget,
    );
    let observed_overlap = intervals
        .iter()
        .any(|(start, end)| *start < call_end && call_start < *end);
    println!(
        "{}",
        json!({"schema":"riauth.s02-core-overlap/v1","attempts":8,"observed_call_interval_overlap":observed_overlap,"writer_joined":true,"complete_pre_or_post_oracles":true,"between_page_injection":false})
    );
}

#[test]
#[ignore = "bounded local native materialization/security comparison; requires an explicit runtime reservation"]
fn group_listing_paging_preserves_snapshot_authority_and_measures_materialization() {
    assert!(
        riauth::context::current().is_none(),
        "context-free Core fixture required"
    );
    let budget = Budget(Instant::now() + FIXTURE_LIMIT);
    let watchdog = Watchdog::new(budget);
    telemetry_self_check();
    let root = fixture_root();
    let empty_seed = Seed::new(&root, budget);
    for count in [0, 1, 128, 129] {
        let fixture = empty_seed.open(&root, budget);
        populate(&fixture.core, count, budget);
        for lane in [Lane::Control, Lane::Paged] {
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.admin,
                &expected(count, false, None),
                count,
                budget,
            );
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.scoped,
                &expected(count, true, None),
                count,
                budget,
            );
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.empty,
                &json!([]),
                count,
                budget,
            );
            refused_read(&fixture.core, lane, "ri_agent_invalid_s02_fixture", budget);
        }
        fixture.close();
    }
    // Persisted key deliberately differs from visible name and sort order.
    // This is a trusted synthetic compatibility row, not a normal Group writer
    // acceptance claim. Existing list_groups authorizes the stored name.
    {
        let fixture = empty_seed.open(&root, budget);
        core_ok(
            fixture.core.store.write(|tx| {
                for index in 0..129 {
                    tx.put(
                        "groups",
                        &format!("s02-storage-{index:03}"),
                        &Group {
                            name: group_name(128 - index),
                            members: BTreeSet::from([user_id(0)]),
                        },
                    )?;
                }
                Ok(())
            }),
            "trusted key-name compatibility rows",
        );
        for lane in [Lane::Control, Lane::Paged] {
            let oracle = json!([{"name":group_name(0),"members":[user_id(0)]}]);
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.scoped,
                &oracle,
                129,
                budget,
            );
            let all = Value::Array(
                (0..129)
                    .rev()
                    .map(|index| json!({"name":group_name(index),"members":[user_id(0)]}))
                    .collect(),
            );
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.admin,
                &all,
                129,
                budget,
            );
        }
        fixture.close();
    }
    let dataset = empty_seed.open(&root, budget);
    populate(&dataset.core, GROUPS, budget);
    let seed = Seed::close(dataset);
    let mut observations = Vec::with_capacity(3 * 2 * 2 * SAMPLES);
    for (pair, order) in [
        [Lane::Control, Lane::Paged],
        [Lane::Paged, Lane::Control],
        [Lane::Control, Lane::Paged],
    ]
    .into_iter()
    .enumerate()
    {
        // Fresh identical CLOSED seed copies for each pair; live Core/Store
        // handles are never copied and no concurrent job runs during a sample.
        let control = seed.open(&root, budget);
        let paged = seed.open(&root, budget);
        for lane in order {
            let fixture = if lane == Lane::Control {
                &control
            } else {
                &paged
            };
            for scoped in [true, false] {
                measure(fixture, lane, pair, scoped, &mut observations, budget);
            }
        }
        // A valid credential with no matching scope retains the empty-array
        // contract even when all 513 source rows exist.
        for (fixture, lane) in [(&control, Lane::Control), (&paged, Lane::Paged)] {
            checked_read(
                &fixture.core,
                lane,
                &fixture.credentials.empty,
                &json!([]),
                GROUPS,
                budget,
            );
        }
        control.close();
        paged.close();
    }
    // Write/concurrency work is strictly outside all measured call windows.
    let membership = seed.open(&root, budget);
    denied_member_write(&membership, budget);
    malformed_write_refusal(&membership, budget);
    ordered_interleaving(&membership, false, budget);
    let revoked = seed.open(&root, budget);
    ordered_interleaving(&revoked, true, budget);
    let overlap = seed.open(&root, budget);
    core_overlap(&overlap, budget);
    let authority = seed.open(&root, budget);
    authority_boundaries(&authority, budget);
    summaries(&observations); // raw numeric samples precede the cost gate
    budget.check();
    authority.close();
    overlap.close();
    revoked.close();
    membership.close();
    seed.cleanup();
    empty_seed.cleanup(); // keep shared private key alive until all copied stores close
    drop(observations);
    budget.check();
    drop(watchdog); // joined before the pre-final numeric receipt
    budget.check();
    println!(
        "{}",
        json!({"schema":"riauth.s02-group-list-result/v1","pairs":3,"order":"ABBAAB","warmups_per_lane_scope":5,"samples_per_lane_scope":10,"total_samples":120,"groups":513,"members_per_group":128,"scoped_groups":3,"all_owned_threads_joined":true,"private_fixture_directories_removed_and_checked":true,"cost_gate":"observed_raw_fetch_row_materialization","latency_grade":false,"heap_claim":false,"rss_claim":false,"pg_claim":false})
    );
    // This numeric receipt precedes final completion grading. A late stdout
    // return cannot make libtest pass; outer supervision must bound IO stalls.
    // No fixture output or file mutation follows this final clock check.
    budget.check();
}
```

Exact old-to-proposed zero-context unified diff (no blank context-space lines):

```diff
--- a/tests/s02_group_listing_paging.rs
+++ b/tests/s02_group_listing_paging.rs
@@ -2,3 +2,3 @@
-//! The allocator is private to this test executable; it never replaces System's
-//! behavior. Its metric is live requested Rust heap bytes, not RSS or malloc's
-//! internal realloc/cache overhead. All output is fixed labels and numeric data.
+//! Existing storage histograms measure raw rows retained by each fetch, not heap
+//! bytes, peak decoded/output memory or RSS. Timers measure actual list calls.
+//! All output is fixed labels and numeric data; no allocation policy is changed.
@@ -13 +13 @@
-    model::{Group, NewUser, User},
+    model::{Group, NewUser, User, UserPatch, UserView},
@@ -15 +15 @@
-    telemetry::ReadContext,
+    telemetry::{ReadContext, Sizes},
@@ -20,2 +19,0 @@
-    alloc::{GlobalAlloc, Layout, System},
-    cell::Cell,
@@ -23 +21 @@
-    hint::{black_box, spin_loop},
+    hint::black_box,
@@ -28 +25,0 @@
-        atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst},
@@ -45,135 +42,62 @@
-
-// A non-allocating lock serializes accounting and window transitions. System
-// calls happen OUTSIDE this lock; no panicking/allocating callback runs inside.
-// SeqCst makes baseline/active/peak/finish an ordered accounting boundary.
-static ACCOUNT_LOCK: AtomicBool = AtomicBool::new(false);
-static LIVE: AtomicUsize = AtomicUsize::new(0);
-static PEAK: AtomicUsize = AtomicUsize::new(0);
-static ACTIVE: AtomicBool = AtomicBool::new(false);
-static INVALID: AtomicBool = AtomicBool::new(false);
-static FOREIGN: AtomicUsize = AtomicUsize::new(0);
-static ALLOCS: AtomicUsize = AtomicUsize::new(0);
-static ZEROED: AtomicUsize = AtomicUsize::new(0);
-static REALLOCS: AtomicUsize = AtomicUsize::new(0);
-static FAILURES: AtomicUsize = AtomicUsize::new(0);
-static DEALLOCS: AtomicUsize = AtomicUsize::new(0);
-thread_local! {
-    // Const, destructor-free TLS; lookup does not allocate. Unavailable TLS is
-    // conservatively foreign. Every thread still contributes to global LIVE.
-    static WINDOW_OWNER: Cell<bool> = const { Cell::new(false) };
-}
-
-struct AccountingLock;
-impl AccountingLock {
-    fn acquire() -> Self {
-        while ACCOUNT_LOCK
-            .compare_exchange(false, true, SeqCst, SeqCst)
-            .is_err()
-        {
-            spin_loop();
-        }
-        Self
-    }
-}
-impl Drop for AccountingLock {
-    fn drop(&mut self) {
-        ACCOUNT_LOCK.store(false, SeqCst);
-    }
-}
-
-#[derive(Clone, Copy)]
-enum AllocationOp {
-    Alloc,
-    Zeroed,
-    Realloc,
-    Dealloc,
-}
-
-fn account(op: AllocationOp, old_size: usize, new_size: usize, success: bool) {
-    let _lock = AccountingLock::acquire();
-    if success {
-        let current = LIVE.load(SeqCst);
-        let next = if new_size >= old_size {
-            current.checked_add(new_size - old_size)
-        } else {
-            current.checked_sub(old_size - new_size)
-        };
-        if let Some(next) = next {
-            LIVE.store(next, SeqCst);
-        } else {
-            // GlobalAlloc must not unwind; never turn broken accounting into
-            // apparently valid evidence, even outside a measurement window.
-            INVALID.store(true, SeqCst);
-        }
-    }
-    if ACTIVE.load(SeqCst) {
-        if !WINDOW_OWNER.try_with(Cell::get).unwrap_or(false) {
-            FOREIGN.fetch_add(1, SeqCst);
-        }
-        let counter = match op {
-            AllocationOp::Alloc => &ALLOCS,
-            AllocationOp::Zeroed => &ZEROED,
-            AllocationOp::Realloc => &REALLOCS,
-            AllocationOp::Dealloc => &DEALLOCS,
-        };
-        counter.fetch_add(1, SeqCst);
-        if !success {
-            FAILURES.fetch_add(1, SeqCst);
-        }
-        PEAK.fetch_max(LIVE.load(SeqCst), SeqCst);
-    }
-}
-
-struct MeteredSystem;
-#[global_allocator]
-static ALLOCATOR: MeteredSystem = MeteredSystem;
-
-// SAFETY: forward the original valid GlobalAlloc arguments and returned pointer
-// unchanged. Accounting never touches allocated memory, allocates, or unwinds.
-unsafe impl GlobalAlloc for MeteredSystem {
-    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
-        let pointer = unsafe { System.alloc(layout) };
-        account(AllocationOp::Alloc, 0, layout.size(), !pointer.is_null());
-        pointer
-    }
-    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
-        let pointer = unsafe { System.alloc_zeroed(layout) };
-        account(AllocationOp::Zeroed, 0, layout.size(), !pointer.is_null());
-        pointer
-    }
-    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
-        unsafe { System.dealloc(pointer, layout) };
-        account(AllocationOp::Dealloc, layout.size(), 0, true);
-    }
-    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
-        let replacement = unsafe { System.realloc(pointer, layout, new_size) };
-        // On failure the original allocation is still live, unchanged.
-        account(
-            AllocationOp::Realloc,
-            layout.size(),
-            new_size,
-            !replacement.is_null(),
-        );
-        replacement
-    }
-}
-
-#[derive(Clone, Copy)]
-struct HeapSample {
-    baseline: usize,
-    peak_extra: usize,
-    end_extra: Option<usize>,
-    foreign: usize,
-    invalid: bool,
-    allocs: usize,
-    zeroed: usize,
-    reallocs: usize,
-    failures: usize,
-    deallocs: usize,
-}
-struct HeapWindow {
-    baseline: usize,
-    finished: bool,
-}
-impl HeapWindow {
-    fn begin() -> Self {
+// Pinned public Sizes::render schema at fab6721; a changed schema refuses.
+const SCAN_BOUNDS: [u64; 7] = [0, 1, 16, 128, 1_024, 8_192, 65_536];
+
+#[derive(Clone, Copy, PartialEq, Eq)]
+struct ScanHistogram {
+    buckets: [u64; 8], // cumulative finite bounds above, followed by +Inf
+    count: u64,
+    rows: u64,
+}
+impl ScanHistogram {
+    fn parse(text: &str) -> Option<Self> {
+        if text.len() > 4096 || text.lines().count() != 10 {
+            return None;
+        }
+        let mut buckets = [None; 8];
+        let mut count = None;
+        let mut rows = None;
+        for line in text.lines() {
+            let (name, value) = line.rsplit_once(' ')?;
+            let number: u64 = value.parse().ok()?;
+            if number.to_string() != value {
+                return None;
+            }
+            let slot = if name == "s02_scan_rows_count" {
+                &mut count
+            } else if name == "s02_scan_rows_sum" {
+                &mut rows
+            } else {
+                let index = (0..8).find(|index| {
+                    let bound = if *index == 7 {
+                        "+Inf".to_owned()
+                    } else {
+                        SCAN_BOUNDS[*index].to_string()
+                    };
+                    name == format!("s02_scan_rows_bucket{{le=\"{bound}\"}}")
+                })?;
+                &mut buckets[index]
+            };
+            if slot.replace(number).is_some() {
+                return None;
+            }
+        }
+        let buckets: [u64; 8] = buckets
+            .into_iter()
+            .collect::<Option<Vec<_>>>()?
+            .try_into()
+            .ok()?;
+        let count = count?;
+        let rows = rows?;
+        if buckets[7] != count || buckets.windows(2).any(|pair| pair[0] > pair[1]) {
+            return None;
+        }
+        Some(Self {
+            buckets,
+            count,
+            rows,
+        })
+    }
+    fn read(sizes: &Sizes) -> Self {
+        let mut text = String::new();
+        sizes.render(&mut text, "s02_scan_rows", "");
+        let value = Self::parse(&text).expect("native scan histogram schema");
@@ -181,16 +105,6 @@
-            !WINDOW_OWNER.with(|owner| owner.replace(true)),
-            "nested heap window"
-        );
-        let _lock = AccountingLock::acquire();
-        // Do not panic while holding the accounting lock (panic may allocate).
-        if ACTIVE.load(SeqCst) {
-            drop(_lock);
-            WINDOW_OWNER.with(|owner| owner.set(false));
-            panic!("concurrent heap window");
-        }
-        let baseline = LIVE.load(SeqCst);
-        PEAK.store(baseline, SeqCst);
-        for counter in [&FOREIGN, &ALLOCS, &ZEROED, &REALLOCS, &FAILURES, &DEALLOCS] {
-            counter.store(0, SeqCst);
-        }
-        ACTIVE.store(true, SeqCst);
+            value.count == sizes.count() && value.rows == sizes.sum(),
+            "native scan histogram counter consistency"
+        );
+        value
+    }
+    fn delta(self, before: Self) -> Self {
@@ -198,140 +112,70 @@
-            baseline,
-            finished: false,
-        }
-    }
-    fn finish(mut self) -> HeapSample {
-        let sample = {
-            let _lock = AccountingLock::acquire();
-            ACTIVE.store(false, SeqCst);
-            HeapSample {
-                baseline: self.baseline,
-                peak_extra: PEAK.load(SeqCst).saturating_sub(self.baseline),
-                end_extra: LIVE.load(SeqCst).checked_sub(self.baseline),
-                foreign: FOREIGN.load(SeqCst),
-                invalid: INVALID.load(SeqCst),
-                allocs: ALLOCS.load(SeqCst),
-                zeroed: ZEROED.load(SeqCst),
-                reallocs: REALLOCS.load(SeqCst),
-                failures: FAILURES.load(SeqCst),
-                deallocs: DEALLOCS.load(SeqCst),
-            }
-        };
-        WINDOW_OWNER.with(|owner| owner.set(false));
-        self.finished = true;
-        sample
-    }
-}
-impl Drop for HeapWindow {
-    fn drop(&mut self) {
-        if !self.finished {
-            let _lock = AccountingLock::acquire();
-            ACTIVE.store(false, SeqCst);
-            WINDOW_OWNER.with(|owner| owner.set(false));
-        }
-    }
-}
-
-fn valid_heap(sample: HeapSample) {
-    assert!(!sample.invalid, "heap accounting overflow or underflow");
-    assert!(
-        sample.foreign == 0,
-        "foreign allocation contaminated sample"
-    );
-    assert!(
-        sample.end_extra.is_some(),
-        "sample released pre-window heap"
-    );
-}
-
-struct ProbeBlock {
-    pointer: *mut u8,
-    layout: Layout,
-}
-impl ProbeBlock {
-    fn allocate(layout: Layout, zeroed: bool) -> Self {
-        // SAFETY: self-check callers provide valid positive-size layouts.
-        let pointer = unsafe {
-            if zeroed {
-                std::alloc::alloc_zeroed(layout)
-            } else {
-                std::alloc::alloc(layout)
-            }
-        };
-        let pointer = black_box(pointer);
-        assert!(!pointer.is_null(), "meter allocation refused");
-        Self { pointer, layout }
-    }
-    fn resize(&mut self, layout: Layout) {
-        assert!(
-            layout.align() == self.layout.align(),
-            "meter resize alignment"
-        );
-        // SAFETY: this block owns a valid allocation, alignments match, and a
-        // null result retains ownership of the original pointer/layout.
-        let pointer =
-            black_box(unsafe { std::alloc::realloc(self.pointer, self.layout, layout.size()) });
-        assert!(!pointer.is_null(), "meter resize refused");
-        self.pointer = pointer;
-        self.layout = layout;
-    }
-}
-impl Drop for ProbeBlock {
-    fn drop(&mut self) {
-        // SAFETY: ownership and current layout are retained through failed
-        // realloc/assertion paths; exactly one Drop releases the allocation.
-        unsafe { std::alloc::dealloc(self.pointer, self.layout) };
-    }
-}
-
-fn meter_self_check() {
-    let small = require(Layout::from_size_align(64, 8), "small layout");
-    let zeroed = require(Layout::from_size_align(96, 8), "zeroed layout");
-    let grown = require(Layout::from_size_align(256, 8), "grown layout");
-    let shrunk = require(Layout::from_size_align(32, 8), "shrunk layout");
-    let window = HeapWindow::begin();
-    let baseline = window.baseline;
-    // SAFETY: layouts have positive sizes; non-null pointers are checked before
-    // access/realloc/free, and each is freed once using its then-current layout.
-    unsafe {
-        let mut block = ProbeBlock::allocate(small, false);
-        block.pointer.write(7);
-        assert!(LIVE.load(SeqCst) == baseline + 64, "alloc accounting");
-        let zeros = ProbeBlock::allocate(zeroed, true);
-        assert!(
-            std::slice::from_raw_parts(zeros.pointer, 96)
-                .iter()
-                .all(|byte| *byte == 0),
-            "zeroed allocator semantics"
-        );
-        assert!(LIVE.load(SeqCst) == baseline + 160, "zeroed accounting");
-        block.resize(grown);
-        assert!(*block.pointer == 7, "growth preserves bytes");
-        assert!(LIVE.load(SeqCst) == baseline + 352, "growth accounting");
-        block.resize(shrunk);
-        assert!(*block.pointer == 7, "shrink preserves bytes");
-        assert!(LIVE.load(SeqCst) == baseline + 128, "shrink accounting");
-        // Exercise the exact null-result accounting branches without requesting
-        // dangerous huge allocations. This is not a native OOM fault injection.
-        account(AllocationOp::Alloc, 0, 4096, false);
-        account(AllocationOp::Zeroed, 0, 4096, false);
-        account(AllocationOp::Realloc, 32, 4096, false);
-        assert!(
-            LIVE.load(SeqCst) == baseline + 128,
-            "failure leaves heap live"
-        );
-        drop(zeros);
-        drop(block);
-    }
-    let sample = window.finish();
-    valid_heap(sample);
-    assert!(sample.peak_extra == 352, "meter peak self-check");
-    assert!(sample.end_extra == Some(0), "meter balanced self-check");
-    assert!(
-        (
-            sample.allocs,
-            sample.zeroed,
-            sample.reallocs,
-            sample.failures,
-            sample.deallocs
-        ) == (2, 2, 3, 3, 2),
-        "meter operation self-check"
+            buckets: std::array::from_fn(|index| {
+                self.buckets[index]
+                    .checked_sub(before.buckets[index])
+                    .expect("native scan bucket went backwards")
+            }),
+            count: self
+                .count
+                .checked_sub(before.count)
+                .expect("native scan count went backwards"),
+            rows: self
+                .rows
+                .checked_sub(before.rows)
+                .expect("native scan rows went backwards"),
+        }
+    }
+    fn expected(sizes: &[u64]) -> Self {
+        Self {
+            buckets: std::array::from_fn(|index| {
+                sizes
+                    .iter()
+                    .filter(|size| index == 7 || **size <= SCAN_BOUNDS[index])
+                    .count() as u64
+            }),
+            count: sizes.len() as u64,
+            rows: sizes.iter().sum(),
+        }
+    }
+    fn over_128(self) -> u64 {
+        self.count
+            .checked_sub(self.buckets[3])
+            .expect("native 128 bucket exceeds count")
+    }
+}
+fn telemetry_self_check() {
+    // Exercise the actual safe public observer, independent literal expected
+    // frequencies, and strict parser refusals; no candidate list code is run.
+    let sizes = Sizes::default();
+    assert!(
+        ScanHistogram::read(&sizes) == ScanHistogram::expected(&[]),
+        "empty native histogram"
+    );
+    for size in [128, 128, 128, 128, 1] {
+        sizes.observe(size);
+    }
+    let actual = ScanHistogram::read(&sizes);
+    assert!(
+        actual
+            == ScanHistogram {
+                buckets: [0, 1, 1, 5, 5, 5, 5, 5],
+                count: 5,
+                rows: 513,
+            },
+        "literal native histogram self-check"
+    );
+    let mut text = String::new();
+    sizes.render(&mut text, "s02_scan_rows", "");
+    let duplicate = text.replacen(
+        "s02_scan_rows_bucket{le=\"0\"} 0",
+        "s02_scan_rows_bucket{le=\"1\"} 0",
+        1,
+    );
+    let malformed = text.replacen("s02_scan_rows_sum 513", "s02_scan_rows_sum false", 1);
+    let noncanonical = text.replacen("s02_scan_rows_sum 513", "s02_scan_rows_sum 0513", 1);
+    let inconsistent = text.replacen("s02_scan_rows_count 5", "s02_scan_rows_count 6", 1);
+    let missing = text.replacen("s02_scan_rows_count 5\n", "", 1);
+    assert!(
+        [duplicate, malformed, noncanonical, inconsistent, missing]
+            .iter()
+            .all(|case| ScanHistogram::parse(case).is_none()),
+        "native histogram parser refusal"
@@ -341 +185 @@
-        json!({"schema":"riauth.s02-meter-self-check/v1","peak_extra_bytes":352,"balanced":true,"null_accounting_cases":3})
+        json!({"schema":"riauth.s02-telemetry-self-check/v1","scan_count":5,"rows":513,"buckets":[0,1,1,5,5,5,5,5],"parser_refusal_cases":5,"heap_claim":false,"rss_claim":false})
@@ -826,0 +671,2 @@
+    bounded_histogram: ScanHistogram,
+    unbounded_histogram: ScanHistogram,
@@ -833,0 +680,2 @@
+        let bounded_histogram = ScanHistogram::read(reads.scans(ReadContext::Read, true));
+        let unbounded_histogram = ScanHistogram::read(reads.scans(ReadContext::Read, false));
@@ -837,4 +685,6 @@
-            bounded: reads.scans(ReadContext::Read, true).count(),
-            bounded_rows: reads.scans(ReadContext::Read, true).sum(),
-            unbounded: reads.scans(ReadContext::Read, false).count(),
-            unbounded_rows: reads.scans(ReadContext::Read, false).sum(),
+            bounded: bounded_histogram.count,
+            bounded_rows: bounded_histogram.rows,
+            unbounded: unbounded_histogram.count,
+            unbounded_rows: unbounded_histogram.rows,
+            bounded_histogram,
+            unbounded_histogram,
@@ -845,0 +696,3 @@
+        let subtract = |after: u64, before: u64| {
+            after.checked_sub(before).expect("native counter went backwards")
+        };
@@ -847,8 +700,10 @@
-            points: self.points - before.points,
-            bytes: self.bytes - before.bytes,
-            bounded: self.bounded - before.bounded,
-            bounded_rows: self.bounded_rows - before.bounded_rows,
-            unbounded: self.unbounded - before.unbounded,
-            unbounded_rows: self.unbounded_rows - before.unbounded_rows,
-            writer_holds: self.writer_holds - before.writer_holds,
-            commits: self.commits - before.commits,
+            points: subtract(self.points, before.points),
+            bytes: subtract(self.bytes, before.bytes),
+            bounded: subtract(self.bounded, before.bounded),
+            bounded_rows: subtract(self.bounded_rows, before.bounded_rows),
+            unbounded: subtract(self.unbounded, before.unbounded),
+            unbounded_rows: subtract(self.unbounded_rows, before.unbounded_rows),
+            bounded_histogram: self.bounded_histogram.delta(before.bounded_histogram),
+            unbounded_histogram: self.unbounded_histogram.delta(before.unbounded_histogram),
+            writer_holds: subtract(self.writer_holds, before.writer_holds),
+            commits: subtract(self.commits, before.commits),
@@ -861,0 +717 @@
+        let empty = ScanHistogram::expected(&[]);
@@ -863,19 +719,38 @@
-            Lane::Control => assert!(
-                (
-                    self.bounded,
-                    self.bounded_rows,
-                    self.unbounded,
-                    self.unbounded_rows
-                ) == (0, 0, 1, count as u64),
-                "control complete scan accounting"
-            ),
-            Lane::Paged => assert!(
-                (
-                    self.bounded,
-                    self.bounded_rows,
-                    self.unbounded,
-                    self.unbounded_rows
-                ) == ((count / PAGE + 1) as u64, count as u64, 0, 0),
-                "paged exact native scan accounting"
-            ),
-        }
+            Lane::Control => {
+                assert!(
+                    (
+                        self.bounded,
+                        self.bounded_rows,
+                        self.unbounded,
+                        self.unbounded_rows
+                    ) == (0, 0, 1, count as u64),
+                    "control complete scan accounting"
+                );
+                assert!(
+                    self.bounded_histogram == empty
+                        && self.unbounded_histogram == ScanHistogram::expected(&[count as u64]),
+                    "control exact native row histogram"
+                );
+            }
+            Lane::Paged => {
+                assert!(
+                    (
+                        self.bounded,
+                        self.bounded_rows,
+                        self.unbounded,
+                        self.unbounded_rows
+                    ) == ((count / PAGE + 1) as u64, count as u64, 0, 0),
+                    "paged exact native scan accounting"
+                );
+                let mut sizes = vec![PAGE as u64; count / PAGE];
+                sizes.push((count % PAGE) as u64);
+                assert!(
+                    self.unbounded_histogram == empty
+                        && self.bounded_histogram == ScanHistogram::expected(&sizes),
+                    "paged exact native row histogram"
+                );
+            }
+        }
+    }
+    fn fetches_over_128(self) -> u64 {
+        self.bounded_histogram.over_128() + self.unbounded_histogram.over_128()
@@ -982 +856,0 @@
-    heap: HeapSample,
@@ -992 +865,0 @@
-        heap,
@@ -998 +871 @@
-            "schema":"riauth.s02-group-list-sample/v1", "pair":pair,
+            "schema":"riauth.s02-group-list-sample/v2", "pair":pair,
@@ -1001,5 +874 @@
-            "heap_baseline_bytes":heap.baseline,"peak_extra_live_rust_bytes":heap.peak_extra,
-            "end_extra_live_rust_bytes":heap.end_extra,"foreign_allocator_operations":heap.foreign,
-            "accounting_invalid":heap.invalid,"allocations":heap.allocs,"zeroed_allocations":heap.zeroed,
-            "reallocations":heap.reallocs,"allocation_failures":heap.failures,"deallocations":heap.deallocs,
-            "point_reads":native.points,"materialized_stored_bytes":native.bytes,
+            "point_reads":native.points,"native_point_plus_scan_value_bytes":native.bytes,
@@ -1007,0 +877,4 @@
+            "row_histogram_bounds":[0,1,16,128,1024,8192,65536],
+            "bounded_cumulative_buckets":native.bounded_histogram.buckets,
+            "unbounded_cumulative_buckets":native.unbounded_histogram.buckets,
+            "fetches_over_128_rows":native.fetches_over_128(),
@@ -1009 +882,2 @@
-            "encrypted_at_rest":true,"instrumented_latency":true
+            "encrypted_at_rest":true,"latency_interval":"complete_list_call",
+            "heap_claim":false,"rss_claim":false
@@ -1035 +908,0 @@
-        let window = HeapWindow::begin();
@@ -1039,3 +912,2 @@
-        // Returned complete JSON stays LIVE through finish. Checks, snapshot,
-        // formatting, counter render and drop are outside the cost window.
-        let heap = window.finish();
+        // Returned complete JSON stays live. Observer rendering, grading,
+        // snapshots and returned-value drop are outside the call timer.
@@ -1049 +920,0 @@
-            heap,
@@ -1054,6 +924,0 @@
-        valid_heap(heap);
-        assert!(heap.failures == 0, "allocation refusal in measurement");
-        assert!(
-            heap.allocs > 0 && heap.end_extra.is_some_and(|bytes| bytes > 0),
-            "meter missed live returned JSON"
-        );
@@ -1077 +942 @@
-    let mut allocation_improved = true;
+    let mut fetch_materialization_improved = true;
@@ -1078,0 +944 @@
+    let mut scoped_latency_p50_lower_in_every_pair = true;
@@ -1092 +958 @@
-                        "schema":"riauth.s02-group-list-summary/v1","pair":pair,"lane":lane.label(),
+                        "schema":"riauth.s02-group-list-summary/v2","pair":pair,"lane":lane.label(),
@@ -1094,3 +960,4 @@
-                        "median_peak_extra_live_rust_bytes":percentile(selected.iter().map(|row| row.heap.peak_extra as u64).collect(),1,2),
-                        "p50_instrumented_elapsed_ns":percentile(selected.iter().map(|row| row.elapsed_ns).collect(),1,2),
-                        "p95_instrumented_elapsed_ns":percentile(selected.iter().map(|row| row.elapsed_ns).collect(),95,100)
+                        "median_fetches_over_128_rows":percentile(selected.iter().map(|row| row.native.fetches_over_128()).collect(),1,2),
+                        "p50_complete_call_elapsed_ns":percentile(selected.iter().map(|row| row.elapsed_ns).collect(),1,2),
+                        "p95_complete_call_elapsed_ns":percentile(selected.iter().map(|row| row.elapsed_ns).collect(),95,100),
+                        "heap_claim":false,"rss_claim":false
@@ -1099,19 +965,0 @@
-            }
-            let control = percentile(
-                select(Lane::Control)
-                    .map(|row| row.heap.peak_extra as u64)
-                    .collect(),
-                1,
-                2,
-            );
-            let paged = percentile(
-                select(Lane::Paged)
-                    .map(|row| row.heap.peak_extra as u64)
-                    .collect(),
-                1,
-                2,
-            );
-            // Scoped allocation improvement is the gate. Report admin cost and
-            // latency honestly; never grade latency by an invented fixed ratio.
-            if scoped {
-                allocation_improved &= paged < control;
@@ -1122,0 +971,10 @@
+                // These are actual native frequency observations, not a
+                // PAGE-based prediction or a memory/throughput inference.
+                fetch_materialization_improved &= match row.lane {
+                    Lane::Control => row.native.fetches_over_128() == 1
+                        && row.native.unbounded == 1
+                        && row.native.unbounded_rows == 513,
+                    Lane::Paged => row.native.fetches_over_128() == 0
+                        && row.native.bounded == 5
+                        && row.native.bounded_rows == 513,
+                };
@@ -1124,4 +982,14 @@
-        }
-    }
-    // Every raw row and all scope/pair latency summaries precede the cost
-    // decision, including an allocation or latency regression.
+            if scoped {
+                let p50 = |lane| {
+                    percentile(select(lane).map(|row| row.elapsed_ns).collect(), 1, 2)
+                };
+                scoped_latency_p50_lower_in_every_pair &= p50(Lane::Paged) < p50(Lane::Control);
+            }
+        }
+    }
+    // Every raw row and every pair/scope summary precedes the decision. A
+    // latency regression remains visible and is never called a speedup.
+    println!(
+        "{}",
+        json!({"schema":"riauth.s02-measurement-decision/v2","native_point_and_value_byte_equivalence":native_equivalent,"observed_fetch_materialization_improved":fetch_materialization_improved,"scoped_latency_p50_lower_in_every_pair":scoped_latency_p50_lower_in_every_pair,"latency_grade":false,"heap_claim":false,"rss_claim":false})
+    );
@@ -1133,2 +1001,2 @@
-        allocation_improved,
-        "no measured scoped allocation improvement"
+        fetch_materialization_improved,
+        "measured raw-fetch materialization did not improve"
@@ -1289,0 +1158,203 @@
+}
+
+// Expiry and parent liveness stay in the shared principal. The expiry row is
+// a declared trusted compatibility fixture, not elapsed-TTL or remote evidence.
+// Parent disable uses the real public writer; it also retires the child token.
+fn authority_agent(
+    fixture: &Fixture,
+    id: &str,
+    parent: Option<String>,
+) -> Zeroizing<String> {
+    token(&core_ok(
+        fixture.core.create_agent(
+            &fixture.credentials.admin,
+            NewAgent {
+                id: id.into(),
+                ttl: 3600,
+                parent,
+                permissions: SELECTED
+                    .iter()
+                    .map(|index| Permission {
+                        action: "group.read".into(),
+                        resource: format!("group/{}", group_name(*index)),
+                    })
+                    .collect(),
+            },
+        ),
+        "authority fixture public credential creation",
+    ))
+}
+fn expected_user_audit(user: &User) -> Value {
+    assert!(
+        user.totp_secret.is_none()
+            && user.totp_pending.is_none()
+            && user.recovery_codes.is_empty(),
+        "factor-free parent fixture required"
+    );
+    let mut view = require(serde_json::to_value(UserView::from(user)), "expected User view");
+    // public_record + fixed sensitive-name redaction at the pinned Store.
+    view["password_available"] = json!("[redacted]");
+    if !user.password_hash.is_empty() {
+        view["password"] = json!("[redacted]");
+    }
+    if !user.pairwise_seed.is_empty() {
+        view["pairwise_seed"] = json!("[redacted]");
+    }
+    view
+}
+fn assert_parent_disable_effect(
+    before: &Snapshot,
+    after: &Snapshot,
+    credential: &str,
+    start: u64,
+    end: u64,
+) {
+    let user_key = format!("users/{}", user_id(0));
+    let old_user: User = require(
+        serde_json::from_value(before[&user_key].clone()),
+        "expected enabled parent",
+    );
+    assert!(old_user.enabled && !old_user.admin, "enabled non-admin parent");
+    let mut new_user = old_user.clone();
+    new_user.enabled = false;
+    new_user.epoch = old_user.epoch.checked_add(1).expect("parent epoch bound");
+    let agent_key = "agents/s02-parent-owned";
+    let old_agent: Agent = require(
+        serde_json::from_value(before[agent_key].clone()),
+        "expected parent-owned Agent",
+    );
+    assert!(
+        old_agent.enabled && old_agent.parent_user.as_deref() == Some(old_user.id.as_str()),
+        "exact active parent binding"
+    );
+    let mut old_view = old_agent.view();
+    old_view["agent_credential"] = json!("[redacted]");
+    let mut new_view = old_view.clone();
+    new_view["enabled"] = json!(false);
+    let mut expected = before.clone();
+    expected.insert(
+        user_key.clone(),
+        require(serde_json::to_value(&new_user), "expected disabled parent"),
+    );
+    expected.get_mut(agent_key).expect("known child Agent")["enabled"] = json!(false);
+    assert!(
+        expected
+            .remove(&format!("agent_tokens/{}", crypto::digest(credential)))
+            .is_some(),
+        "parent disable retired exact child token"
+    );
+    for key in ["provisioning_user_generation/all", "user_listing_generation/all"] {
+        let previous = expected.get(key).and_then(Value::as_u64).unwrap_or(0);
+        expected.insert(
+            key.into(),
+            json!(previous.checked_add(1).expect("parent generation bound")),
+        );
+    }
+    advance_revision(&mut expected);
+    // BTreeMap-backed change records sort agents before users. This fixture has
+    // no owner RP/session, Windows device, grants, downstream link/job or SSF
+    // stream; any unexpected durable consequence fails the COMPLETE map.
+    let changes = json!([
+        {"resource":agent_key,"before":old_view,"after":new_view},
+        {"resource":user_key,"before":expected_user_audit(&old_user),"after":expected_user_audit(&new_user)}
+    ]);
+    let (audit_key, event) = expected_audit(
+        before,
+        after,
+        admin_id(before),
+        "user.update",
+        &old_user.id,
+        changes,
+        start,
+        end,
+    );
+    expected.insert(audit_key, event);
+    assert!(expected == *after, "complete public parent-disable snapshot");
+}
+fn authority_boundaries(fixture: &Fixture, budget: Budget) {
+    budget.check();
+    let expired = authority_agent(fixture, "s02-expiry-boundary", None);
+    for lane in [Lane::Control, Lane::Paged] {
+        checked_read(
+            &fixture.core,
+            lane,
+            &expired,
+            &expected(GROUPS, true, None),
+            GROUPS,
+            budget,
+        );
+    }
+    let before = snapshot(&fixture.core);
+    let key = "agents/s02-expiry-boundary";
+    let mut record: Agent = require(
+        serde_json::from_value(before[key].clone()),
+        "expiry fixture canonical Agent",
+    );
+    assert!(record.enabled && record.parent_user.is_none(), "isolated expiry fixture");
+    let boundary = crypto::now();
+    record.expires_at = boundary;
+    core_ok(
+        fixture.core.store.write(|tx| tx.put("agents", &record.id, &record)),
+        "trusted canonical expiry fixture",
+    );
+    let mut expected_state = before.clone();
+    expected_state.get_mut(key).expect("known expiry Agent")["expires_at"] = json!(boundary);
+    same_snapshot(&fixture.core, &expected_state);
+    assert!(crypto::now() >= boundary, "real clock moved before expiry boundary");
+    let same_second_observed = crypto::now() == boundary;
+    for lane in [Lane::Control, Lane::Paged] {
+        refused_read(&fixture.core, lane, &expired, budget);
+    }
+    // Token mapping remains valid and enabled; refusal therefore exercises the
+    // stored expiration, rather than a missing token or an Agent disable.
+    assert!(
+        expected_state
+            .get(&format!("agent_tokens/{}", crypto::digest(&expired)))
+            .and_then(Value::as_str)
+            == Some("s02-expiry-boundary"),
+        "expired credential mapping retained"
+    );
+    let owned = authority_agent(fixture, "s02-parent-owned", Some(user_name(0)));
+    for lane in [Lane::Control, Lane::Paged] {
+        checked_read(
+            &fixture.core,
+            lane,
+            &owned,
+            &expected(GROUPS, true, None),
+            GROUPS,
+            budget,
+        );
+    }
+    let before = snapshot(&fixture.core);
+    let start = crypto::now();
+    core_ok(
+        fixture.core.update_user(
+            &fixture.credentials.admin,
+            &user_name(0),
+            UserPatch {
+                enabled: Some(false),
+                ..Default::default()
+            },
+        ),
+        "public parent disable",
+    );
+    let end = crypto::now();
+    let after = snapshot(&fixture.core);
+    assert_parent_disable_effect(&before, &after, &owned, start, end);
+    for lane in [Lane::Control, Lane::Paged] {
+        refused_read(&fixture.core, lane, &owned, budget);
+        // Unrelated unparented authority and canonical Group bodies survive.
+        checked_read(
+            &fixture.core,
+            lane,
+            &fixture.credentials.scoped,
+            &expected(GROUPS, true, None),
+            GROUPS,
+            budget,
+        );
+    }
+    println!(
+        "{}",
+        json!({"schema":"riauth.s02-authority-boundaries/v1","expiry_source":"trusted_canonical_stored_timestamp_at_real_now","expiry_refusal_exact":true,"expiry_same_second_observed":same_second_observed,"natural_ttl_elapsed_claim":false,"public_parent_disable_exact_snapshot":true,"child_token_retired":true,"legacy_enabled_child_with_disabled_parent_runtime_claim":false,"unrelated_authority_preserved":true})
+    );
+    budget.check();
@@ -1481 +1552 @@
-#[ignore = "bounded local allocation/security comparison; requires an explicit runtime reservation"]
+#[ignore = "bounded local native materialization/security comparison; requires an explicit runtime reservation"]
@@ -1489 +1560 @@
-    meter_self_check();
+    telemetry_self_check();
@@ -1613 +1684 @@
-    // Write/concurrency work is strictly outside all allocation/latency windows.
+    // Write/concurrency work is strictly outside all measured call windows.
@@ -1621,0 +1693,2 @@
+    let authority = seed.open(&root, budget);
+    authority_boundaries(&authority, budget);
@@ -1623,0 +1697 @@
+    authority.close();
@@ -1635 +1709 @@
-        json!({"schema":"riauth.s02-group-list-result/v1","pairs":3,"order":"ABBAAB","warmups_per_lane_scope":5,"samples_per_lane_scope":10,"total_samples":120,"groups":513,"members_per_group":128,"scoped_groups":3,"all_owned_threads_joined":true,"private_fixture_directories_removed_and_checked":true,"cost_gate":"measured_scoped_peak_extra_live_rust_bytes","rss_claim":false,"pg_claim":false})
+        json!({"schema":"riauth.s02-group-list-result/v1","pairs":3,"order":"ABBAAB","warmups_per_lane_scope":5,"samples_per_lane_scope":10,"total_samples":120,"groups":513,"members_per_group":128,"scoped_groups":3,"all_owned_threads_joined":true,"private_fixture_directories_removed_and_checked":true,"cost_gate":"observed_raw_fetch_row_materialization","latency_grade":false,"heap_claim":false,"rss_claim":false,"pg_claim":false})
```

### Static checks, held command and handoff

Only the report is written. Actual final static checks passed: exact whole
d6b prefix, both literal design hashes, whole-file zero-context patch and
inverse across42 hunks, retained frozen/result/refusal/writer/concurrency
bodies, no unsafe/allocator/allow tokens, unchanged actual Core/test/build
inputs, report-only scope, documentation links and corrected Git whitespace.
The proposed Rust was **not** materialized into a source file or
passed to rustfmt/rustc/Cargo; no parser/typecheck/fixture/telemetry self-check
execution is claimed. Native measurement, cost/speed/security/concurrency
outcomes, cleanup, fit within300s and actual capacity remain **unknown**.
An exploratory lookup for nonexistent `src/identity/user_record.rs` and
`src/audit.rs` returned a static path error; the actual shared User writer and
audit bodies were read at the literal management/core paths instead. Combined
large read output was truncated; relevant sections were reread in bounded
reads. A first Git whitespace check refused six blank context-space lines in
the report's initial unified-diff fence. The design diff was re-encoded with
zero context to preserve a standard exact patch and clean documentation;
the proposed Rust bytes did not change. No compiler/runtime failure was
inferred from those lookups or this report-only whitespace refusal.

If the exact safe source and narrowed native-cost gate are approved, the
single prospective command remains **HELD** and applies to that future source,
not to the currently blocked unsafe fixture:

```sh
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support --test s02_group_listing_paging \
  group_listing_paging_preserves_snapshot_authority_and_measures_materialization \
  -- --exact --ignored --test-threads=1 --nocapture
```

No lint override/RUSTFLAGS/new package/extra observer/second executable is part
of that proposal. The300-second fixture, actual-source review and private jobs1/
incremental0/dev+testdebug0 library rebuild still require root's separate
capacity/source/outer-supervisor reservation. Prior13GiB start/4GiB planning/
9GiB stop/8GiB floor/1800s envelope remain **prospective**, not remeasured here.
The A09 remote lane37087561409 is root-owned; no lane was requested, acquired,
released or queried by this design.

Recommendation: approve the safe fixture replacement only if the root accepts
**measured bounded raw-fetch materialization with equivalent-security oracles**
as this slice's metric, while retaining honest latency. Otherwise the precise
unresolved decision is which supported observation can measure the required
live-allocation quantity under the unchanged forbid policy. Neither choice is
a claimed pass or original-row completion. S02 stays root-owned in_progress;
all earlier failures/pins/evidence, primary assignment, completed rows and
protected contracts remain unchanged. No code, test, manifest, helper, existing
guide/other report, runtime, service, worker, desktop/provider, network, branch
alignment, main, push or board change occurred.
