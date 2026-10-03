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
