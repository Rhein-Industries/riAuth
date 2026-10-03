# A09 HTTP admission expectation: independent source review

Date: 2026-10-03 (Europe/Vaduz). Project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`; supporting worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`. Reservation:
`wave30_A09_http_admission_expectation_independent_review`.
The sole write is this new report, from clean own parent
`1f90af6274da5f621a7d27f5abab4c17c3738ba3`; no alignment or merge.

Recommendation: accept the bounded source design at `f607693` for its separately
owned implementation review. I found no concrete defect in the BEFORE-derived
generator, its caller guards or the exact two-file diffs against the pinned
product contract. No corrective hunk is proposed. The candidate is UNEXECUTED;
implementation, validator selection, memory/native validation and release remain
HELD/root-owned. This is neither retrospective identification of the changed
hosted tuple nor A09/shared-gate completion.

## Immutable source and complete-read boundary

Principal author: `f6076932855384997b23bbe5859cbac7208206dd`, path
`docs/roadmap/local-wave30-a09-shared-users-refusal-plan.md`.
I read the complete 598-line appendix at document 4454–5051: its actual-failure
limits, product trace, prerequisites, full new generator and `shared_probe`,
both exact diffs, preservation proof and prospective validation boundary.
The entire 262952-byte parent report at
`406efd21c18e33ac2b2ded79a6382269d80c7130` is its unchanged prefix. That was a
whole-byte comparison; I did not reread every older report paragraph or archive.

Baseline helper/workflow: `d00c9680004be856c387e4faf03c14c1e62d6b0c`.
I read all 466 baseline helper lines and both full proposed function bodies,
then reconstructed the complete new helper as text. I reconstructed the entire
workflow and parsed its complete inline controller and bootstrap as source data.
Semantic workflow body reads covered its fixed inputs/imports/resource limits,
closed count/failure projections, complete bootstrap, native helper invocation/
report check and final checkout/run/cleanup/four-document upload boundaries.
Full workflow/controller byte inverses and AST comparisons cover every remaining
protected body; that does not claim a new semantic reread of all 1350 controller
lines. No reconstructed source was written as an executable file or evaluated.

Product: `9a819317efb3a13fa27cd86f884be2be00898fc0` only. Selected product
bodies below were actually read, distinct from whole-file hash checks:

| Immutable product path | Read contract/spans |
| --- | --- |
| `src/api.rs` | Complete `protect`, 872–1207, worker run 84–95, user-create handler 2467–2475, rate key/proxy processing 2958–2990; route registration located at 401. |
| `src/store.rs` | Shared rate writer 739–772 and plain-format open context; complete PostgreSQL writer 1094–1136; audit-bearing bucket filter, get/put/import/delete/raw bytea writer, selected span 1310–1575. |
| `src/store/maintenance.rs` | 1–300: COUNTED/QUEUES, expiry key/counter rule and complete rate-index branch in `update_indexes`. |
| `src/crypto.rs` | 1–150, including complete production/test clock distinction, digest and plaintext/encryption primitives. |
| `src/config.rs` | Effective defaults/resolver, default key/proxy configuration, URL validation and finite rate validation: selected 120–235, 450–575, 776–796. |
| `src/assembly.rs`, `src/identity.rs`, `src/assembly/scim_runtime.rs` | Selected assembly transitions/key loading 170–300, identity 1–165 and complete SCIM record-transition dispatch 156–250; unrelated rate/index buckets have no identity/SCIM transition. |
| `src/core.rs` | Mutation/receipt/precondition writer 38–111, creation 524–538 and session/identity reader 1045–1084. |
| `src/agent.rs`, `src/management.rs`, `src/context.rs` | Principal/permission bodies 1–220; user-create 1643–1678; request context, management permission projection and complete receipt replay/save, selected context 1–135, 330–390. |
| `src/cli.rs`, `src/cli/transport.rs`, `src/cli/local.rs` | User create 2593–2622; transport 1–265 including saved session, redirect refusal and one-send call; init/key default 375–420. |

I also read the selected fixed helper-import bodies at `d00c968`: matrix
`init_instance` and PostgreSQL setup 166–192, and installed gate's complete
CLI/remote/serving wrappers (selected 1–160, 232–288). These are source reads,
not CLI, database, process or network invocations. The full baseline helper
equals the validator's file at `8a2c4a192d04a43cd930141506a5215cca343542`;
the full baseline workflow equals
`7869b96453c13fdec5af35fd5cf661849d417f36`.

| Report/source identity | Bytes / lines | SHA256 |
| --- | --- | --- |
| Entire `f607693` report | 299665 / 5051 | `9b9a2372c5d5974ddd8901bbfb23c42e86e2ef246496089fcf75926f3f2b362f` |
| Preserved prior prefix | 262952 / 4453 | `c0983d37c00318e01b894b60fbf832e84d904e89db68a13b7039c1145ec4228f` |
| Complete new appendix | 36713 / 598 | `37cb525917ff133d4b70175f0dd9b699a96cd4355ee6b7d24f3e9f3ade14d456` |
| Generator fence, document 4690–4759 | 3664 / 70 | `ca8b693a06c4c6213e7004b1e24a66c488090ba8501c7003460ad368ccc77b75` |
| New probe fence, document 4769–4854 | 5699 / 86 | `eaa5f164fcd9514d3a3959e163b474bcd6b3a46ac0a1c7e9b8f6c4d27e2e0f07` |
| Complete six-hunk helper diff | 5434 / 102 | `c0a2e55a47d935cb35e965de1ba95ce095f8d9a33f38092c913bf10cf9e36d68` |
| Complete one-hunk workflow diff | 384 / 5 | `956d56571e0c9f4b7f7f255176a3b1acc19890138c598100d8de1f831a7d0b25` |
| Baseline helper | 26547 / 466 | `4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa` |
| Complete helper candidate | 31339 / 556 | `32a2952e3b4c3ff2ab73a747238862d3446c31bf4896c869bc9efacb1ba8fe49` |
| Baseline workflow | 82149 / 1426 | `226483dd9edcd5e566b6c5de9f7ab2a36ebdf0c0f204f8ec992645542a3f395f` |
| Complete workflow candidate | 82149 / 1426 | `cddca77a2f75bf02c8ceff0a1a16e792636bc5a09bd68b5ead8494e6ca1bb9fc` |
| Baseline complete inline controller | 66841 / 1350 | `bda0a0fa30db096d92404de22dd5193c91d111c3890e30fe43034405abdd2369` |
| Candidate complete inline controller | 66841 / 1350 | `f094ba6bd03bfaabad7a2e9516ab16c67fd08af0e6f0fa6413e754534cc3e588` |
| Exact unchanged bootstrap value | 5111 / 97 | `a8f7df1a172e1dc89672d2be54fa147eabf6a61864935e82fd86e2cf64839d33` |

## Independent product trace: committed admission versus refused mutation

The socket peer is read in `protect`; forwarding is interpreted only for a
trusted socket peer. This fixture requires no trusted proxies. `/api/users`
selects `general`; IPv4 loopback remains itself under `rate_key`. PostgreSQL
calls the shared rate writer before `next.run` and therefore before management
authorization. Its transaction finishes independently of the later Core writer.
Limited admission returns 429. A successful non-limited admission can still
reach the unchanged 403 management refusal.

The exact rate ID is derived in source from UTF-8 IP, NUL and category using
SHA256/URL-safe-no-pad encoding. The writer obtains production Unix seconds
from `SystemTime` through `crypto::now`, reads `(u64,u32)`, retains the old start/
count only while saturating start+60 is strictly greater than request time,
otherwise resets to `(request_time,0)`, and always puts saturating count+1.
Its limited result uses the prior count. Reclamation/eviction runs only when
the row is absent; the proposed prerequisite requires the exact existing row.
I did not compute or inspect an actual private rate ID/tuple.

`Tx::put` converts that tuple to a JSON array. With no database key it persists
compact JSON bytes. `update_indexes` deletes the prior expiry key and inserts
the new expiry key: saturating start+60 padded to twenty digits, slash, digest
of the rate ID. The value is the compact JSON string containing the ASCII rate
ID. Existing-row replacement leaves the collection count unchanged. The rate
bucket is outside QUEUES and audit-bearing/identity/SCIM transition cases, so
this writer adds no cursor/bound, identity, revision, receipt or audit mutation.
This is a selected-source conclusion, not an observed hosted write trace.

For the attempted user creation, the embedded CLI receives the revision and
idempotency key explicitly. Its user-create arm constructs one direct POST;
the transport reads the saved session locally and has one explicit `send`, no
application-level discovery/retry and redirects disabled. This is a count of
source operations, not a newly measured wire-request count or proof about
unobserved proxies/transport failures. `gate.remote` calls the embedded server
executable, not standalone `riauthctl` discovery behavior.

Core requires both headers, checks the current principal/permissions, receipt
and revision inside its mutation writer, then management requires exact
`user.write` before password history, user persistence, provenance or audit.
Failure occurs before receipt save; PostgreSQL evaluates the closure result
before commit, so the failing Core writer rolls back. It cannot roll back the
earlier committed rate writer. The unchanged session reader checks revocation/
expiry and current identity rather than renewing the session. Full equality
with zero admission effect is consequently not the right expected product
behavior for this scoped, successful HTTP admission plus refused mutation.
This does not identify the hosted record that actually changed.

## BEFORE-only oracle and finite prerequisites

The complete new generator has exactly five parameters: BEFORE bytes, lower
and upper wall-clock nanoseconds, monotonic elapsed nanoseconds and effective
general limit. There is no AFTER parameter or reference. Its rate identity,
candidate times, tuple arithmetic and rewrite keys are selected independently
of AFTER. The caller uses AFTER only in whole-byte equality with yielded
complete expectations; the unchanged failure diagnostic separately classifies
BEFORE/AFTER without granting admission success.

The unchanged strict parser first validates BEFORE: exact bytes type, final
newline, <=8 MiB capture, finite key/value/row bounds, lowercase even hex, UTF-8
non-NUL bucket/key names and no duplicate decoded key. The new pack/repack check
then requires exact bytea-key ordering and canonical complete framing. Other
values remain their original hex bytes; they are not decoded/reformatted to
conceal a protected change. Sorted Python byte keys match the fixture's bytea
`ORDER BY key` ordering. Empty/missing/noncanonical evidence refuses.

The generator independently derives the single fixed loopback/general row and
requires its existing canonical compact array. The count must be 1..u32::MAX;
start must be <= the lower bracket second and therefore within the validated
u64 time range. The existing collection-count bytes must equal the finite actual
number of rate rows. Exactly one expiry entry may carry this rate ID, and its
key/value must match the old tuple. Missing/new rate rows, stale duplicate expiry
indexes, inconsistent counters and a prospective new-key collision refuse.
These are deliberately narrower prerequisites than all valid possible stores.

The bracket surrounds only the unchanged create CLI call. Clock/limit inputs
must be true Python integers, not bool; elapsed must be nonnegative and <=5s,
wall interval nonnegative and <=6s, and endpoint wall-versus-monotonic drift
<=250ms. Wall endpoints lie in the declared nonnegative u64-second envelope.
The derived inclusive floor-second range has at most seven possibilities.
No AFTER row supplies or narrows those possibilities, and no clock is changed.

For a still-current window, feasible seconds collapse to one unchanged start
with count+1 and identical index/counter bytes. At equality with start+60 the
production rule resets: a feasible bracketed request second becomes start,
count becomes 1, and only the old/new independently derived expiry entries
change. Saturation and strict greater-than comparison match the Rust writer.
A bracket crossing that boundary admits only those finite source-defined
possibilities. Prior count >= effective limit is excluded because that candidate
would return 429, incompatible with the separately required 403. Saturation
does not authorize an unbounded count; the count and limit bounds remain explicit.

Every yielded expectation copies the complete BEFORE map, replaces only this
one tuple and its exact expiry entry, then packs the entire sorted snapshot
under the original 8 MiB cap. There is no bucket exclusion, ignored field,
after-selected row or diagnostic-count substitute. A missing increment, an
increment of two, a wrong current start, an expired count other than 1, another
bucket, or protected/counter/index drift cannot match the specified transaction
footprint. Feasible reset seconds remain an explicit timing ambiguity; matching
one is not evidence of which actual request second occurred.

Endpoint drift checks cannot certify an unobserved clock excursion. The proposal
assumes the isolated helper and product use the same native host clock, not a
remote/distributed clock. The bracket limits are post-call evidence guards,
not hard cancellation or a reduction of the existing CLI timeout. Missing/
encrypted rows, alternate routing, concurrent traffic, unsupported shapes,
slow calls or clock behavior outside this envelope fail rather than broadening
the accepted rewrite. Input/output byte caps establish no measured CPU/RSS peak.

## Caller and preservation assessment

The caller reads its actual TOML, requires absent database key and trusted
proxies, exact plain IPv4-loopback base, matching issuer/listen and an explicit
or default-600 general limit in 1..100000. These agree with pinned Config's
resolver/validation and fixture init, which supplies no encryption-key flag.
The exact local PostgreSQL connection/database/user/port guard remains intact.
Store opening additionally requires the matching plain storage format; random
encrypted serialization is not treated as predictable plaintext.

Revision is obtained before the BEFORE capture. The create call and private
stdin, revision, idempotency-key arguments are unchanged. Public exit 4, HTTP
403 and `access_denied` must all pass before capture comparison. There remain
exactly two complete SQL captures and one explicit create call inside this
seam. The later 0.2-second sleep, expiry equality, logout/revocation, admin
login, exact auditor grant/group checks and E-P-E operations are unchanged.

The new comparison catches only ValueError/OverflowError prerequisite failures
as `admitted=False`. Memory/interruption/other errors do not become successes.
The exact assertion message, exact-AssertionError diagnostic attachment,
BaseException-safe diagnostic fallback and re-raise remain intact. Complete
snapshots and generated bytes are local variables, not newly printed/public
evidence. No actual private rows, tokens, session files or protocol data were
read by this audit. The comparison covers every record in the captured
`records_v1` table; it does not certify unrelated PostgreSQL tables/files.

Independent text forward/reversal checked every hunk position/count/context:
six helper hunks and one workflow hunk reconstruct the declared complete
candidates, and both full inverses restore all d00 bytes. Inverse helper AST
equals the entire original AST. The generator/probe fences equal their complete
reconstructed function bytes/ASTs; only stdlib `base64` is added as an import.
Python source was parsed to AST only, never imported or evaluated.

Of eleven old top-level helper functions, ten remain byte/AST-identical:
`load`, `rows`, `transition_metadata`, `require_target_metadata`,
`require_preserved`, `authenticated_status`, `ordinary_fixture`,
`refusal_snapshot_counts`, `shared_config_refusals` and `main`.
The sole old changed function is `shared_probe`; the one new function is the
generator. Source-ordered AST/text call lists preserve all ten `gate.remote`
calls, the one nested `subprocess.run` query and the one `time.sleep` in that
probe. The query body still executes through its two existing call sites.

The entire workflow changes only the fixed helper import digest. Restoring that
digest restores its whole bytes and complete controller AST, including all
class/method bodies; the twenty-two top-level controller functions also compare
exact. Bootstrap bytes/AST and all four other helper imports remain exact.
All other FIXED data is identical: product/tree/build/artifact/action identities,
resource/file bounds and transport/cleanup/upload contracts. Selecting the old
helper or the report-only proposal commit as validator source would not satisfy
the new helper digest. Root must separately select an immutable implemented
source containing the complete helper plus all four protected imports; updating
the workflow hash alone does not implement the rule.

## Actual hosted evidence and remaining limits

I read all 618 lines of the immutable root receipt at
`6c7d32424c3a8dcc8515ce2fe7eee6e184e4a011`, path
`docs/roadmap/evidence/wave30-a09-hosted-counts-37077820186.json`:
21046 bytes, SHA256
`a8631bb8aeaa438955cbc9a6dd825386bdb3166192831e073bd699b2fadffe14`.
This is a public sanitized data read, not a remote query, archive extraction or
fresh execution. The separately named four raw sanitized members were not
opened/rehashed directly in this slice; their sizes/hashes are in that receipt.

Run 37077820186/job 111071534441 used workflow `7869b964`, validator `8a2c4a1`
and product `9a819317`. It remains FAILED: helper phase exit 1 after 3.204996s,
AssertionError frames bootstrap 88, helper 466/352/239 and matrix 79. The public
CLI/403/access-denied checks preceding helper239 passed. Its finite count
projection establishes only:

| Category | Before / after | Added / changed / removed |
| --- | --- | --- |
| http_rates | 2 / 2 | 0 / 1 / 0 |
| http_rate_expiry | 2 / 2 | 0 / 0 / 0 |
| http_rate_count | 1 / 1 | 0 / 0 / 0 |
| maintenance_cursors | 0 / 0 | 0 / 0 / 0 |
| maintenance_bounds | 0 / 0 | 0 / 0 / 0 |
| protected_or_other | 51 / 51 | 0 / 0 / 0 |

The opaque changed key/value/field are UNKNOWN. No observed tuple, increment,
window, expiry or causal attribution is inferred from those counts. The pending
`helper-redacted.json`/`sample_result=not_run` is not a completed sample and does
not imply the bootstrap/helper was uninvoked; the actual phase ran and failed.
Later session-expiry and E-P-E assertions were not reached.

The receipt records cleanup failures empty, remaining owned processes zero,
one owned fixture process reaped, fixture/private scratch removed and no live
owned PostgreSQL; pidfile verification remains false. Six resource samples
had minimum free 115572277248 bytes and maximum gap 2.0006767590000436s.
The output ZIP artifact 11257332482 was 17555 bytes with SHA256
`596b7754c542722f2b976147a0b0f35ba6111f296d1abc0a17890c38e81648b8`.
The distinct input transport was 49177062 bytes,
`fc2a83dd286b70e455802af7713b60724d97b9e598240f6d9037c279601e4d30`.
No archive was opened, downloaded or used by this audit.

The root receipt preserves its initial wrong helper-member allowlist refusal
before extraction and subsequent corrected member selection from the retained
ZIP; it is not another hosted invocation or a product cause. All older native/
hosted failures and unknown causes remain failed/unknown. Earlier 89/89 memory
and 42/42 archive passes are separate bounded results; neither evaluates this
new oracle or establishes a shared/native/HA/release/tenant gate. No new case
count or runtime result is claimed here.

## Actual static checks and handoff

Actual checks: Git object reads/resolutions and hashes; full-body source review
as scoped above; both complete candidate forward/inverses; helper/controller/
bootstrap Python AST parsing and preservation; function/call/import/fixed-input
comparisons; complete public JSON receipt read. All finished static comparisons
exited 0. One exploratory `src/login.rs` lookup failed (path absent, exit 1);
immutable source search located `Core::session` in `src/core.rs`, whose body
was then read. This was a source lookup correction, not runtime attribution.

The own-tree pre-write `python3 scripts/check-docs.py` exited 0. Final report
docs, pin, source-scope, direct whitespace and staged/committed whitespace checks
are recorded in the handoff. Original report prefixes and all product/helper/
workflow files remain untouched; no source alignment or cleanup/delete occurred.

This slice performed ZERO candidate/function/harness/case/VM evaluations,
helper/bootstrap/controller imports or executions, native/PG/SQL/HTTP/product
CLI/Cargo/browser/runtime calls, external queries/downloads or slot changes.
Future Driver-only desktop preference remains unchanged; no desktop interaction
was needed. Root owns any separately reserved synthetic oracle validation,
source materialization/validator composition and bounded hosted repeat. Even
a later exact admission match would establish only this captured refusal
footprint, not subsequent E-P-E or the whole original A09 outcome.

Only this new report is committed. No other-worker contact, new worker/task/
worktree/shell, board/status, main/push or other file change occurred. Root alone
interprets disposition, integrates/publishes and reserves implementation/runtime.
