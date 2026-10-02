# A09 snapshot counts: independent immutable source review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`. Supporting worktree
`42bb51c6-c198-4adb-bd92-0a5222853231`, branch
`roadmap/local-module-boundaries-wave27`. Reservation:
`wave30_A09_snapshot_counts_independent_review`. Review date: 2026-10-02.
Own starting HEAD `d21c0bcf42a68f63d1de880f6c6b612b82ee5e69`, clean.
Primary A09 worktree `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2` remains assigned.
Only this new report is written. No author checkout or in-progress implementation
was read, and no author/other worker was contacted.

## Disposition

No concrete source blocker was found in the complete immutable diagnostic
design. Recommend root's later, separately released pure-memory validation of
the exact implementation once its immutable bodies match this design. This
review executes no proposed definition, synthetic case, bootstrap or harness.
It does not establish actual changed rows, their cause, runtime privacy,
measured memory peak, PostgreSQL success or original A09 completion.

The original complete ordered Store byte comparison remains authoritative.
Counts attach only after that comparison raises the original exact
`AssertionError`; the same exception is re-raised. There is no request retry,
new SQL capture, row exclusion, normalization of the decisive comparison or
change to the expected user-create exit 4 / HTTP 403 refusal.

## Immutable material and independent reconstruction

Reviewed all 748 appended lines, both complete patches, all six complete
function bodies and the later memory-only validation proposal in
`eae9415980bc5826ae8dc74b07af0e1ca8e62122`:
`docs/roadmap/local-wave30-a09-shared-users-refusal-plan.md`.
The complete document is 89,229 bytes, SHA256
`210bd44af62b2ceaac3d162cb10b1852e58473071487dd300625b826563835ad`.
Its unchanged 44,904-byte prior prefix hashes to
`e8077e7f985ff88d3db10e8b3faaf7385ba729c43d64bcf147b18048126882de`;
the 44,325-byte / 748-line append hashes to
`00862eafb92a5ffc96b1585acb27c6bc1c7fbf4c88ef5067b41332b2fc233ad0`.

Protected source is `f7af6cc85b938738e6152b4b0e5109e36a3abdec`.
Both source objects at the design pin are byte-identical to this baseline.
Independent zero-context patch application and reversal occurred only in
memory. Each reversal restores its entire original file, including workflow
steps outside the embedded Python controller.

| Material | Bytes | SHA256 |
| --- | ---: | --- |
| Baseline shared PostgreSQL helper | 23,518 | `d86d9a99c9e09e29eb43af52f409332e11a70caaed90b5545bf132df1f2a1982` |
| Prospective helper | 26,547 | `4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa` |
| Helper patch fence | 3,516 | `181f58bf3a54dc08b9161324f7047e709111e86f9b26d00a25310b55f1f3082a` |
| Baseline shared workflow | 77,543 | `5290d7a5d9e15eaeaa19169523668e9658b0341d4eb76ca47695c532e7aa27b2` |
| Prospective workflow | 82,149 | `226483dd9edcd5e566b6c5de9f7ab2a36ebdf0c0f204f8ec992645542a3f395f` |
| Workflow patch fence | 7,647 | `d58a77d3f4073a508de5eb2cbdf955fe468ce36ee0fac73cadd98798de52726a` |
| Prospective embedded controller | 66,841 | `bda0a0fa30db096d92404de22dd5193c91d111c3890e30fe43034405abdd2369` |
| Prospective bootstrap string | 5,111 | `a8f7df1a172e1dc89672d2be54fa147eabf6a61864935e82fd86e2cf64839d33` |

These independently reproduce root's supplied prospective file identities.
The following complete archived bodies match their reconstructed lexical
bodies byte-for-byte after removing only fence-ending newlines. ASTs also
match, with location attributes excluded. Controller line numbers below are
relative to its extracted Python text; bootstrap lines are relative to its
embedded literal, not workflow YAML.

| Complete body | Prospective location | Archived body SHA256 |
| --- | --- | --- |
| `refusal_snapshot_counts` | Helper 135–192 | `9e7b87389b97f8b3275c82b6038faaa7a581830e455fe3f022f2382656aa747e` |
| `shared_probe` | Helper 195–263 | `24924b4c926c7c5f8c6efa5e5347d9e330597e3737c792657d5c33169279467d` |
| `snapshot_counts` | Controller 278–294 | `629775bf4d54324c75ce89546167e640238be292f42dd092b7f02f640b5c7b16` |
| `failure_packet`, including `pairs` | Controller 297–342 | `1265d99c16a3dd2d2ba4792ec11a907074e61a571489a8410aadf9b28bb36561` |
| `failure_source` | Bootstrap 48–86 | `6d1e8e9f9d5fd7e1bcce0376d22ac465c39a200c1fa2bfa000b25308dcf9ff4d` |
| `Controller.helper` | Controller 1012–1074 | `63e9c51930f315deeb945545c59317b9a54d5a9381010b94ff50709d037ffb8c` |

The bootstrap and controller validators are identical complete lexical bodies
and identical ASTs. The four other imported helper bodies were hash-checked,
not independently reviewed in full. Each matches both immutable pins and its
unchanged `FIXED["imports"]` digest:

| Import | SHA256 |
| --- | --- |
| `check-exact-edition-matrix.py` | `f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887` |
| `check-installed-release-gate.py` | `cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5` |
| `check-local-encrypted-edition-transition.py` | `09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e` |
| `spdx_sbom.py` | `ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c` |

## Data and failure-path review

`refusal_snapshot_counts` accepts only exact `bytes`, at most 8 MiB per
capture. Nonempty input must end in LF. Each record needs one hex-key/hex-value
separator; lowercase even hex is mandatory. Encoded keys are 2–8,192 bytes,
encoded values 0–2 MiB, and at most 1,000,000 unique records are admitted.
Keys decode to UTF-8, must contain a nonempty bucket followed by `/`, contain
no NUL, and cannot repeat. Odd/uppercase/nonhex input, extra separators,
malformed keys, duplicates and over-limit input raise internally. They are
not printed, and parser failure cannot replace the original equality failure.

The partition has exactly six labels; no new namespace or exclusion is added:

| Fixed label | Exact key predicate |
| --- | --- |
| `http_rates` | Starts with `http_rates/` |
| `http_rate_expiry` | Starts with `index_expiry_http_rates/` |
| `http_rate_count` | Equals `index_counts/http_rates` |
| `maintenance_cursors` | Starts with `maintenance_cursors/` |
| `maintenance_bounds` | Starts with `maintenance_bounds/` |
| `protected_or_other` | Every other captured Store row |

Each has exactly `before`, `after`, `added`, `changed`, `removed`: thirty
integer counts, not row identifiers. Presence determines added/removed;
complete captured value hex determines changed. Parsed keys/value hex remain
local memory only. Identity, session, credential, configuration, agreement,
revision, receipt, audit and unknown collections stay in `protected_or_other`.
Existing transition-specific exclusions do not enter this refusal comparison.
Even an unequal raw capture with zero row differences still fails the
unchanged complete comparison; these counts do not normalize its decision.

Both validators demand exact built-in dictionaries, exact string keys, the
exact six/five field sets and exact built-in integers in 0–1,000,000. Boolean
and type-subclass values are refused. Accepted output is copied into new
fixed-key dictionaries; unknown categories/fields cannot become public keys.
No identifier, arbitrary collection, record/value hash, private path,
timestamp, error message, SQL text or private response enters the projection.

`shared_probe` captures after-state once, outside the attachment `try`, just
as the original expression did. `matrix.require` retains its complete byte
comparison and message. The matrix's inspected `require` body raises the
built-in `AssertionError`. Exact-type guarding limits attachment to that
class, and the optional parser/assignment is protected by `except
BaseException`. The outer bare `raise` has neither replacement exception nor
cause. Capture errors and other exception classes retain their original path.
This is source proof of exception identity, not an executed identity test.

Bootstrap projection starts with the original fixed exception class and
trusted frames: at most 64 traceback entries examined, eight retained, line
integers 1–4,096. Optional counts apply only to exact `AssertionError`; invalid
counts, projection failure or an oversized serialization keep the original
packet. Exclusive `O_EXCL | O_NOFOLLOW`, mode 0600 creation and original
exception propagation are unchanged. The observer still has a 2,048-byte cap.

The controller verifies a regular file's size, reads at most 2,049 bytes and
accepts exact byte packets no larger than 2,048. Its JSON pairs hook detects
duplicate keys at every object depth. It validates the original finite class
allowlist and frame schema, then constructs a new base packet using only
literal `exception_class`, `frames`, `file`, `line` keys. Missing/malformed
optional counts, unknown outer fields, duplicates anywhere, a non-assertion
class or oversized optional serialization suppress counts. A valid fixed
base may still be retained after duplicate/unknown optional data; an invalid
base, undecodable/deep malformed JSON or oversized packet is rejected. This
is the designed fallback, not acceptance of arbitrary JSON fields. The
controller's existing observer catch and bare re-raise preserve `phase_exit`
and cleanup even when no packet is published.

## Protected source proof

Complete baseline probe, SQL capture, later actions and helper `main` were
read; baseline controller observer, bootstrap, resource/process/cleanup and
phase-finalization context were read. The protected source restoration is
more precise than replacing complete changed functions:

* Helper: remove only the new counter function; replace only the after-capture
  assignment plus diagnostic comparison handler with the original comparison
  expression. The entire helper AST then equals baseline. All ten original
  functions, imports/constants, two full snapshot calls, refusal SQL, revision
  and idempotency request, HTTP/CLI assertions and later probe flow are covered.
  The multiset of all helper SQL/native/CLI/HTTP primitive-call ASTs also matches.
* Bootstrap: remove only its duplicate validator and exact optional projection
  `if`. The entire bootstrap AST then equals baseline, including traceback
  handling, file writer, helper hash check, fixture ownership and runpy wrapper.
* Controller: remove only the two new pure functions; restore only the helper
  digest, embedded bootstrap and observer read/validation statements. The
  entire controller AST then equals baseline. The observer save and original
  catch/re-raise remain exact, as does the complete success-report suffix.
* All 17 other controller methods are identical ASTs: initialization, fail/save,
  sample/monitor/guard, terminate/command, source/download, both extraction
  methods, PostgreSQL-tool checks, fixture proof, cleanup, process identification
  and run. Resource thresholds, two-second sampling, evidence/capture bounds,
  timeouts, owned groups/descendants, phase failure and finalization are unchanged.
  The whole-file patch reversal also protects actions, environment and YAML.

No extra SQL/HTTP requests or comparison exclusions were introduced. This
review does not propose a product, authority, configuration or writer change.
Accepted receipt-secret, route-header, PAM, held Group and lease contracts
remain outside the reservation and untouched.

## Later memory-only seam and remaining evidence

The archived proposal confines any later validation to one isolated process,
one pass, 30 seconds, extracted pure definitions and synthetic captures. Its
failure serializer seam ends before observer file creation; the equality seam
extracts only the comparison/attachment block. Neither requires loading the
bootstrap/helper/controller, invoking the surrounding probe or accessing a
database, environment, network or real Store records. Static validator equality
has already been established independently here. Any later transformed AST
must remain a harness-only object, with exact original source hashes recorded.

The proposed malformed-input, boundary, duplicate-schema, privacy-sentinel,
subclass/boolean, fallback and exception-identity cases are appropriate to
that seam. None ran here. The proposed <32 MiB synthetic peak / 128 MiB reserve
is an unmeasured estimate, not a parser allocation bound: 8 MiB limits input,
not dictionaries or total process memory. The archive's separate 1,246-byte
maximum fixed-packet calculation is prior static evidence, not my execution
of either serializer. Root retains release/resource decisions; no retry,
hosted dispatch or runtime slot is authorized/acquired by this report.

The root-supplied actual `37060776569` evidence establishes user-create exit 4
and HTTP 403 followed by failed complete snapshot equality. Changed rows and
their cause remain UNKNOWN. No counts or writer origin are inferred. Earlier
shared failures, including unlocated inner cause and the prior collection-GET
assertion failure, remain failures. Nothing here credits later E–P–E phases.
Root's current notice separately establishes container run `37061329815`
failed on extra archive metadata after both image builds exited 0. There was
no ARM container dispatch. Those outcomes were supplied by root, not replayed
or independently fetched in this review; published source
`74e106b819e7186d0ac41964cedbe8217fa7e881` is not their runtime result.

Original A09 still requests server/client/container/maintenance artifacts for
Linux x86-64 and ARM64, with unchanged users, authorization and configuration
when switching distributions. Its shared gate remains open. This diagnostic
review changes no closed row, task status or primary assignment, and makes no
official-release/container success or full shared-gate claim.

## Checks actually performed and limits

Read-only `git show`, `git diff-tree`, `git status`, `git rev-parse`, `rg` and
source-range reads inspected immutable documents/source and own guidance.
Independent Python stdin witnesses used source strings, hashing,
`ast.parse`/`literal_eval`, JSON decoding of the fixed source literal and
forward/reverse text patches. Final complete reconstruction/body/validator,
whole-AST restoration, five import hashes and report-prefix checks exited 0.
No candidate code object, proposed definition, synthetic input case or harness
was executed. No mutable author file was consulted.

One initial tool request failed JavaScript parsing before any command ran.
The first static AST witness exited 1 because it incorrectly searched only
the probe's top-level statements, overlooking the existing live-server block.
Only that independent witness was corrected to locate the statement container;
the full static rerun and lexical-body extension exited 0. Neither event was
a candidate function/runtime failure or prompted source changes.

No build/test/Cargo, native artifact, helper import, PostgreSQL/HTTP/provider,
network/query/download/dispatch, Docker/service/browser/desktop, harness,
deletion, alignment/merge/reset, worker/task/worktree/shell creation, contact,
main edit/push or board change occurred. Driver-only desktop preference remains.
Only the report-only commit is handed to root for independent review/integration.

`python3 scripts/check-docs.py` exited 1, reporting only the five pre-existing
private build directories `target-wave29-source`, `target-wave28-scim`,
`target-wave28-portal`, `target-wave28`, `target-wave27`. It reported no Markdown
link failures. Those directories and the checker were not changed or removed.
The unstaged `git diff --no-index --check -- /dev/null` report comparison exited
1 with no whitespace diagnostics (the new-file difference status).
`git diff --cached --check` exited 0. The explicit scope witness also exited 0:
only this new report is staged, its indexed/working bytes match, there are no
unstaged or other untracked changes, and own HEAD/history remains unchanged.

## Independent future-memory design review, 2026-10-03

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`, supporting worktree
`42bb51c6-c198-4adb-bd92-0a5222853231`. Reservation:
`wave30_A09_snapshot_memory_design_independent_review`. Own starting HEAD is
`2c9e5eadb99350b9bd6f9845154bdbf1afe75ded`, clean. This append preserves the
complete 16,004-byte original report, SHA256
`49ca6404061f27178b1f89a9a2dfe892bc5b3c1d53b34a7e93987b2f7fe7939b`.
The previous source-only recommendation remains its dated finding about
`eae9415`; this phase reviews the newly archived validation program.

### Recommendation: correct two future-payload blockers before release

The archive is syntactically valid and its source pins match the reviewed
implementation. Two concrete source defects prevent the proposed validation
from reaching its intended oracles. They concern the future child payload,
not the committed diagnostic helper/workflow or product authorization.
Recommend one bounded archive correction containing these two hunks, followed
by immutable static review. No correction or runtime is authorized here.

1. **Strict selector rejects the legitimate probe default before cases.**
   Archived payload line 168 calls `function(helper, "shared_probe")`.
   `function` line 61 requires `not node.args.defaults`. The committed source
   signature at helper line 195 is
   `shared_probe(server, config, base, scratch, revoked_token=None)`:
   its AST has one `Constant(value=None)` default. Thus the guard is false
   before `STAGE = "cases"` at payload line 175. This is a deterministic
   source-derived rejection, not an observed child execution. The smallest
   seam is the payload's default guard: permit only this named probe's one
   literal `None` default, retaining the strict rule for every other function,
   decorators, keyword defaults and annotations. The probe definition must
   remain unexecuted; only its unique equality handler is extracted.
2. **The isolated packet validator lacks its required builtin `all`.**
   The payload's explicit builtin tuple at lines 77–82 has `any` but no
   `all`. Committed `failure_packet` calls `all` at extracted controller
   line 323 when validating frames. The payload passes that unchanged
   function to `isolated` at lines 156–158, with no other binding of `all`.
   Valid-packet oracles would therefore encounter an unbound name if the first
   blocker were repaired alone. The smallest seam is adding only `"all"`
   beside `"any"` in that builtin tuple. Do not remove the namespace guard,
   stub/replace the production packet function, or weaken positive assertions.

A precise prospective selector guard is:

```python
defaults_ok = not node.args.defaults or (
    name == "shared_probe" and len(node.args.defaults) == 1
    and type(node.args.defaults[0]) is ast.Constant
    and node.args.defaults[0].value is None
)
```

Use `defaults_ok` only in place of `not node.args.defaults` in the existing
`check`; all its other predicates stay intact. The second prospective hunk
changes the builtin tuple prefix to `("type", "len", "any", "all", ... )`.
These are proposals only; no archived source or harness was materialized.
Changing the payload requires recomputing its full bytes/hash, updating the
controller's `PAYLOAD_SHA`, then recomputing the controller and assembly hash
chain and all corresponding archive tables. The six implementation-source
pins must stay unchanged. A stale hash chain must continue refusing execution.

The archived parent remains fail-closed around these defects: it requires a
completed child, numeric actual exit 0, all eight groups attempted, zero
recorded failures, matching sources, a reaped child and bounded completion.
An early internal failure with no failed case records cannot become a pass.
No actual case count, child exit or failure location was measured here.

### Complete immutable material and source identity

Read all 861 new lines in
`1aed8a40728cfba555902dc2325bb0bab0cdb313`:
`docs/roadmap/local-wave30-a09-shared-users-refusal-plan.md`, including the
complete 414-line child, 219-line controller, assembly command, oracles and
preparation limits. Its prior 96,475-byte prefix is unchanged, SHA256
`190fa98b9fc23839bd83d9451293cf252256d3074d730fdf9617b3c7ec34104f`.
The 50,525-byte append hashes to
`48e2839a6dca84a878ded795afbb8e2c701f72d2910794948e13169ae8d36786`;
the complete 147,000-byte object hashes to
`facee1a45c8035887ece58bfd42885595a035c38a85ac92a8a104bce2a6e36f5`.
The dated archive-construction delimiter failure remains in that report;
it is not a candidate case result.

| Complete future archive, including final LF | Bytes | SHA256 |
| --- | ---: | --- |
| Child payload | 24,494 | `1cd3442cb91d19de9c70134da763d072b9cc5ca317e2ed2bb633df2bb1c04181` |
| Controller | 12,634 | `278060f3de48a133b017cffc188bf9adea200337e901370e97a1f1764ded44ee` |
| Assembly command | 1,427 | `084ed5ae9d4417b4f6fbce9af8d43eb810bc2f29e7c8dea378ed256512e66653` |

All six complete input objects at actual source
`d00c9680004be856c387e4faf03c14c1e62d6b0c` match both archived `PINS` lists.
Their sizes/digests equal the six earlier reviewed source rows above, including
all four imported-script hashes. No imported helper module was loaded.
The two diagnostic implementation objects exactly match the earlier reviewed
prospective bytes and identities:

| Actual source | Bytes | SHA256 | Git blob |
| --- | ---: | --- | --- |
| PostgreSQL helper | 26,547 | `4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa` | `af764b71b95f8ad4d5e1d9b26d094dfa2eaafe49` |
| Shared workflow | 82,149 | `226483dd9edcd5e566b6c5de9f7ab2a36ebdf0c0f204f8ec992645542a3f395f` | `f63110098c13d717446a198c487c00678cfeb60a` |

The actual extracted controller is 66,841 bytes / SHA256
`bda0a0fa30db096d92404de22dd5193c91d111c3890e30fe43034405abdd2369`;
bootstrap is 5,111 bytes / SHA256
`a8f7df1a172e1dc89672d2be54fa147eabf6a61864935e82fd86e2cf64839d33`.
Both match `eae9415` and the complete-body review in `2c9e5ea`.
The decoded five-script `FIXED` import map is exact; bootstrap/controller
validator ASTs match. This byte identity carries forward the prior full
reversal/protected-body proof; it is not new runtime evidence.

### Extraction, oracles, privacy and protected comparator

Independent structural inspection finds exactly one outer `AssertionError`
handler in the committed probe containing `after_refusal`. Its decisive call
still compares complete `after_refusal == before_refusal` bytes with the
original message and bare re-raise. There are exactly two snapshot calls.
The capture query still selects every ordered Store key/value hex row, without
`WHERE`, collection exclusions or a row limit. The memory child extracts only
that handler; the surrounding probe, SQL, subprocesses and request are not
part of its executable seam. The protected assertion's strictness is not
redefined by a passing counts projection.

The counter retains the exact six-category partition, thirty integer counts,
8 MiB per-input bound, canonical hex/UTF-8/duplicate checks, full value-byte
changes and catch-all `protected_or_other`. Packet/schema projections retain
exact built-in types, boolean/subclass refusal, fixed keys, duplicate hook,
optional fallback and 2,048-byte cap described in the original review.
All identity, authorization, credential, audit, receipt, revision and unknown
records remain fully compared. No namespace or allowed mutation is added.

The counts oracle is hand-written from synthetic before/after membership and
value changes. Negative parser/schema/packet cases exercise malformed hex,
duplicates, aliases, boundaries and optional fallback; positive cases include
the exact four-row 8 MiB capture, 4,096-byte key, 1 MiB value and maximum fixed
packet. Equality cases separately require byte-identical success, preservation
of the same original error object/message on a change, and denial after row
reordering despite zero projected changes. Forced projection errors and an
assertion subclass test fallback. These are reviewed proposed oracles, not
executed passes or an independent production-authorization acceptance suite.

The source-selected producer's last statement is the complete private writer
`if` at bootstrap line 82. The proposal removes that entire block and adds
only `return payload` after the preceding nine statements. Static inspection
of that retained prefix finds no filesystem/runtime names. `InertPath` only
joins, resolves to itself and returns strings/parents. A controlled synthetic
raise/recursive descent gives a genuine traceback under the fixed inert helper
filename; no fabricated traceback object or bootstrap main is needed. Trusted
frame naming, 64 examined/eight retained bounds and original serializer remain
source-backed. The proposed frame test observes retained frames; it does not
independently measure every examined frame without execution.

Sentinel checks cover key/value/hex, URI, private path and exception message
in fixed projections. Child results contain fixed group/name/status/class,
source digests and timing, never exception `str`/`repr` or actual Store data.
The parent re-parses and validates fixed result schema, unique case names,
derived totals/groups, exact types and source digests. Matching duplicate
validators and repeated implementation tests are diagnostic-compatibility
checks, not evidence that the original shared gate passed.

### Prospective retention, deadlines and owned cleanup

The archive assembly verifies a unique section/controller and exact hash
before executing anything. The prospective controller verifies the payload
hash, Python 3.11+ prerequisite, capacity and fresh private directory. The
source-binding child reads bounded/no-follow regular files and checks all six
hashes before executing isolated definitions. Parent source/result grading
occurs later. These launch paths were read as source, never run here.

There is exactly one prospective `subprocess.Popen`, controller line 83: the
same Python executable with `-I -B -c`, DEVNULL stdin, minimal environment and
`start_new_session=True`. The payload has no subprocess call. Source functions
are extracted rather than imported; helper main, bootstrap runpy/writer,
controller construction, native binaries, SQL/PG/HTTP and product CLI remain
outside the memory child. This is an inspection of fixed trusted source, not
a general sandbox guarantee for arbitrary replacement code.

The controller retains actual outcome before grading: normal path line 195
calls `retain_outcome`, then line 197 rehashes sources and line 198 grades.
Retention exclusively writes/fsyncs `child-exit.json` with actual exit/reaping,
the complete captured `child.json`, and private `child-stderr.bin` in that order.
Stdout/stderr caps are 64 KiB/8 KiB; cap failures retain bounded partial evidence
and cannot pass. Setup without a child records null exit. No case metadata,
expected-source comparison or JSON-result grading substitutes for raw outcome
retention. Each save is no-follow, exclusive 0600; the directory is fresh 0700
and preserved, not reused or deleted.

The child has a 30-second alarm/persistent exhaustion flag. The controller's
outer clock starts before assembly, requires 30 seconds left during preparation, samples
pipes at most every 50 ms, and accepts only completion below 35 seconds. On
stop/cap/timeout it signals only its exact unreaped owned group; finally it
kills/waits if needed and closes child streams. A bounded five-second emergency
reap can make completion late, which final acceptance rejects. The last clock
is after all receipt fsyncs; no receipt write follows it. Actual timing,
capacity, allocation, reaping and retained files remain unmeasured. Future
bounded memory validation and any remote diagnostic repeat remain HELD.

### Historical boundary, actual static checks and handoff

The archived finite evidence for `37060776569` establishes actual CLI exit 4 /
HTTP 403 / `access_denied`, then failed full Store equality in the first
Essentials probe. It does not disclose changed rows/keys or their cause.
The archived product-`9a81931` trace identifies an independently committed
HTTP admission rate writer before Core authorization, separate from the
denied Core transaction. That source-derived admission explanation is not
proof of the observed writes or their exclusivity. I did not replay that run,
read private records/logs or freshly audit those product bodies in this phase.
All earlier remote/assembly failures remain dated evidence. Original A09's
shared gate stays open, the closed rows/primary assignment stay unchanged,
and no container, tenant, provider or official-release success is inferred.

Actual independent checks in this phase used Git objects and stdlib source
data only. `ast.parse` succeeded for the complete child/controller and the
assembly's Python heredoc, and for the actual helper/embedded source strings.
Archive/source hash checks, exact selector/default and missing-builtin
witnesses, raw equality/SQL preservation, before-writer cut, retention ordering,
fsync/final-clock structure and one-child-site checks exited 0. An AST witness
found 46 syntactic case-call sites; this is not an attempted case total or a
pass/fail result. No proposed selector/function, controller, assembly, candidate
case or bootstrap was compiled/evaluated/executed by this review.

Only this append is permitted. No product/source/harness/test/workflow edit,
alignment, other-worker contact, new worker/task/worktree/managed shell, native
tool/product/PG/HTTP/Cargo/CLI/browser/Driver/network query or runtime slot,
main/push/board change occurred. Driver-only preference remains for any future
separately authorized desktop work. Root owns archive correction reservation,
independent review, release, integration and status.

Final static report checks: `git diff --check` exited 0. Explicit prefix/scope
assertions exited 0 and preserve every byte of the original `2c9e5ea` report;
only this report has an append. `python3 scripts/check-docs.py` exited 1 solely
for the same five pre-existing private target directories listed in the prior
review, with no Markdown link failure. No directory or checker was altered.
These static successes do not negate the two payload blockers or establish
zero runtime failures. Staged whitespace and clean commit scope are checked
before handing the immutable report to root.
