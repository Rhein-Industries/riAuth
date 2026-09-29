# S02: authoritative Group cutover proposal

**Status: design for review; no cutover implemented.** The held v9 chunk mirror
is not a performance or process-memory solution. `groups/<name>` still holds
the entire `Group.members` set. The mirror lookup reads and hashes every chunk,
and redb's authoritative key probe may cache the large source leaf. On the
same 4 MiB fixture, measured mirror lookup medians were 45–126 ms across the
four plain/encrypted redb/PostgreSQL modes, versus 8–55 ms for the legacy
read. A narrower metadata mirror cannot establish authority cheaply while
that large redb value remains the source.

## Proposed source and invariant

Use a new **canonical format**, not another derived mirror. After an offline
schema/activation cutover, `groups/<name>` is a small, versioned authoritative
root containing the exact name, member count, and authenticated membership
tree root. Membership is stored in bounded encrypted source rows (at most
48 KiB plaintext payload each) under a separate canonical collection. A
hash-keyed, bounded-fanout Merkle trie gives membership and nonmembership
proofs in O(log N) bounded rows; leaves bind the complete member ID and group
name, with separately authenticated bounded overflow chunks for legacy long
IDs. Reject a hash-key/full-ID mismatch. The root row is the source of Group
existence, so a missing `groups/<name>` cannot be masked by leftover member or
index rows. Compare its embedded name with the key before using it. Group
name folding and authorization continue to use the exact source identity.

Each read uses one storage snapshot. A point membership or LDAP/claims index
candidate must be checked against a root-to-leaf source proof; a missing or
invalid proof fails closed, including for a stale derived index. An existence
check reads only the small authoritative root, not the member tree. A full
Group stream verifies the tree and count before publishing output. On open and
after restore, validate every source tree, reject orphans/malformed rows and
rebuild all derived membership, DN-fold and binding indexes from verified
source. This detects offline corruption before serving. No design can detect
an adversary who rewrites both source and its root while the server is live
without an external trust anchor; PostgreSQL source-table writes must remain
restricted to the application, and native restores go through the existing
recovery gate. Any raw source import must invalidate serving until validation;
it cannot bypass the tree writer and leave a trusted root behind.

## Cutover work that must land together

1. **Migration and storage.** Bump the canonical schema and index/activation
   versions together. Stop writers, take a pre-upgrade backup, and stage one
   old Group at a time into the new source collections. Verify exact name,
   member set, count, root and indexes before an atomic activation switch.
   Keep the legacy row authoritative until that switch; never serve a mixed
   format. A failure leaves the old state usable. The initial legacy redb row
   cannot be read with a strict per-record RSS bound: its existing large leaf
   must be loaded during this *offline conversion*, and old Group values may
   need extra memory. Preflight sizes and fail before activation if conversion
   cannot finish. New normal operations must never read the legacy row.
2. **Writers and audit.** Replace the full-`Group` `Tx::put` path with atomic
   source-tree path updates, root update, derived index/reviewed-holder update,
   revision and audit. Incremental member add/remove changes O(log N) source
   rows; replace-members and SCIM ownership merge compare sorted, bounded
   input/output streams. Keep the existing permission, review, idempotency and
   change semantics. The generic audit change currently copies complete
   before/after Group JSON; preserve that logical audit record with chunked
   storage/streamed serialization, including old audit readers. Do not claim
   a whole-operation bound while request parsing, `GroupWrite`, or audit still
   materializes the member set.
3. **Full responses.** Admin Group list, create/member responses, SCIM Group
   GET/list/write responses and resource ETags must serialize from the same
   verified snapshot in exact current member/order form. Hash-key trie order
   differs from `BTreeSet` order, so use bounded external merge sorting for
   full output. Stream to a bounded private spool before HTTP emission (or
   return an explicit size error before output), so a slow client cannot hold
   a database snapshot indefinitely.
   Preserve exact ETags and filtered visibility. The existing JSON `Value`
   return signatures require an API/transport refactor; they are a release
   gate, not a detail deferred after storage cutover.
4. **Backup, restore, recovery, rollback.** Classify root/member/overflow rows
   as canonical retained source; classify v9 chunks and all indexes as derived
   once legacy migration completes. v3 streaming backup carries each bounded
   source row. Restore validates a complete versioned source before rebuilding
   indexes; v1/v2/old v3 archives run the legacy converter under the same
   offline gate. Never reconstruct source from a restored index or mirror.
   Keep redb and PostgreSQL plain/encrypted semantics identical. Older
   binaries must refuse the newer activation; rollback is restore of the
   pre-cutover backup followed by the normal recovery/credential procedure,
   not a silent format downgrade.

## Proof required before acceptance

Use identical 200 KiB and 4 MiB Groups on plain/encrypted redb and PostgreSQL,
with fresh processes for peak RSS and alternating warm/cold 25+ sample latency
runs. Record median/p95, OS peak RSS, heap high-water, physical/logical bytes,
and source rows/pages touched for existence, present/absent membership,
claims/LDAP, one-member write, replace-members, full response and backup.
At 4 MiB, common point-read p95 must beat the current mirror and legacy read;
at 200 KiB it must not materially regress against legacy;
peak RSS growth from 200 KiB must be bounded by a stated fixed working budget
(proposed 2 MiB) rather than Group size. No zero-only telemetry assertion is
evidence. Compare serialized output/ETags, audit, order, DN collisions and
authorization with v9. Inject missing roots, altered names/leaves/overflow,
stale indexes, interrupted conversion and restore failures; require fail-closed
behavior in redb and PostgreSQL, including encrypted stores. Exercise both
editions and old-backup restore. If these gates cannot be met, retain the old
source and do not promote the held mirror stack as a bounded solution.

## Current source seams

- [`src/store.rs`](../../src/store.rs): `Tx::put`, `import_record`,
  `record_change`, `record_key_probe`, and the redb/PostgreSQL raw reads.
- [`src/store/maintenance.rs`](../../src/store/maintenance.rs): v9 chunk mirror,
  whole-chunk `bound_group_source_exists`, group indexes and rebuild.
- [`src/management.rs`](../../src/management.rs): full-Group writer,
  reviewed holders, `GroupWrite` and audit calls.
- [`src/core.rs`](../../src/core.rs) and [`src/scim.rs`](../../src/scim.rs):
  full Group/list values, member projection and response/ETag construction.
- [`src/operations.rs`](../../src/operations.rs),
  [`src/operations/stream.rs`](../../src/operations/stream.rs),
  [`src/recovery.rs`](../../src/recovery.rs) and
  [`src/upgrade.rs`](../../src/upgrade.rs): backup, restore classification,
  index rebuild and version activation.
