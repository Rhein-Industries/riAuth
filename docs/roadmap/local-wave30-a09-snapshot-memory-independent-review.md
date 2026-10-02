# A09 corrected snapshot memory design: independent source review

Date: 2026-10-03 (Europe/Vaduz). Project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`. Reservation:
`wave30_A09_corrected_snapshot_memory_independent_review`. This report is the
sole reserved write. The audit is source/data-only; runtime remains HELD.

Recommendation: accept the three corrected design seams for root's separate
source disposition and consideration of ONE bounded memory invocation. I found
no remaining source blocker in those seams. This is not execution approval,
a case pass, an actual changed-row diagnosis or A09/shared-gate completion.
The archived child, controller and assembly have not executed in this audit.

## Immutable inputs and body-read scope

The reviewed design is `a47f7b73c3a10cf6846547db17e6553b3faa0391`, path
`docs/roadmap/local-wave30-a09-shared-users-refusal-plan.md`: 254437 bytes /
4307 lines, SHA256
`e2a9ccd6ff5baf1610a99b2a0811ab9a5c3f0cb729ae6ed974aed1057c0bb4bc`.
The runnable-design appendix begins at document line 3360. Its complete
427-line child, 227-line controller and 30-line assembly command were read,
including all case bodies, top-level launch/output paths, receipt grading and
owned cleanup. I read all three corrected diffs, the full appended design
prose and source-selection/binding boundaries.

The all-only predecessor is `4d0e77c9301535e4d6b284965e03ec608231fcbf`, same
path: 199879 bytes / 3358 lines, SHA256
`c625e93783a155c4c8d752cbb365f1da68a2cb1c33d03bd686ea3f4fc66890ce`.
Its entire bytes are the unchanged prefix of `a47f`. The base is
`1aed8a40728cfba555902dc2325bb0bab0cdb313`: 147000 bytes / 2409 lines, SHA256
`facee1a45c8035887ece58bfd42885595a035c38a85ac92a8a104bce2a6e36f5`.
Its entire bytes are the unchanged prefix of `4d0e`.

The supplied predecessor token had 43 characters and did not resolve as a Git
revision: the initial batched read printed a fatal unknown-revision message;
its individual exit code was not separately retained. Its unique `1aed8a4`
prefix resolves to the full
40-character base above, which is also the direct parent of `4d0e` in the
reviewed design history. This correction is an immutable-object lookup fact,
not a candidate preparation/case/runtime failure. No alternate stack, mutable
worktree diff, source alignment or network lookup was used.

For the predecessors I read their complete design/limit/blocker prose and all
three all-only diffs, and extracted/parsed all six older source archives for
complete byte/AST comparison. I did not reread every older report paragraph or
claim that hashes constitute body review. The current complete body read,
six whole-byte reversals and AST comparisons establish the unchanged older
case/receipt/cleanup bodies without executing them.

From immutable production pin `d00c9680004be856c387e4faf03c14c1e62d6b0c`, I
read the full `refusal_snapshot_counts` and `shared_probe` bodies (helper lines
135–263), their surrounding fixture/config-refusal context (120–305), both
controller selected definitions (278–342), and all 97 lines of the bootstrap
literal. The six complete input bytes, full 1350-line controller literal and
import map were extracted/hashed as data. This is not a full body-read claim
for the 1426-line workflow, entire helper main, large controller or other four
imported scripts. No helper/module import or native version call occurred.

## Exact archive identity and preservation

All archives below are UTF-8/LF and include their final newline. The assembly
counts include its Python heredoc wrapper; AST parsing used only its enclosed
Python body. No archive was materialized as an executable source file.

| Object / archive | Bytes / lines | SHA256 |
| --- | --- | --- |
| `1aed` child | 24494 / 414 | `1cd3442cb91d19de9c70134da763d072b9cc5ca317e2ed2bb633df2bb1c04181` |
| `1aed` controller | 12634 / 219 | `278060f3de48a133b017cffc188bf9adea200337e901370e97a1f1764ded44ee` |
| `1aed` assembly | 1427 / 25 | `084ed5ae9d4417b4f6fbce9af8d43eb810bc2f29e7c8dea378ed256512e66653` |
| `4d0e` all-only child | 24501 / 414 | `8352902481d7a2db05585dee2eb9c34789e67bf8b26b89196894d9634c10ec73` |
| `4d0e` derived controller | 12634 / 219 | `387b0725b76401a697d3852f15c30f799cbc60a2bf48fa75ea1ce53aeb6ed1ad` |
| `4d0e` derived assembly | 1427 / 25 | `a9868508c5309eaa1062f83a5adfe1bacebf98aa1b097019816d9bfd17eac7da` |
| `a47f` runnable child, document 3590–4016 | 25221 / 427 | `2a686adae4b02a7f623566b13432ca2d14f43bb5206b839552639979bdce1a7d` |
| `a47f` runnable controller, document 4022–4248 | 13019 / 227 | `e87d72ca393c448354355de881b0b5ba098985fe9666b98d657bf4e0d23dc8e2` |
| `a47f` runnable assembly, document 4254–4283 | 1668 / 30 | `21ecc40da9618eef5e9c86ff4194a389e09ce59ee86ca3f069300769233cecc7` |

The three new zero-context diffs reverse byte-for-byte to the entire `1aed`
archives. The three all-only diffs independently reverse to the same bases.
The all-only child adds exactly the `all` tuple literal; its controller and
assembly change only the resulting integrity digests. That older candidate
still omits `list` and selects old archive blocks. Its historical coverage
failure and fail-closed selector/hash incompatibility remain unexecuted source
findings, not retroactively passing results.

Current child AST comparison preserves all definitions and statements outside
the declared `function()` validator and the `isolated()` builtin tuple. Removing
only `all` and `list` from the latter restores its complete old function AST.
The child has 19 top-level function/class definitions; `main`, all case inputs,
case bodies, assertions, budgets and output paths remain equivalent to the base.
No definition was called to establish that equivalence.

The current memory controller has ten functions. Only `PAYLOAD_SHA` and
`prepare()` differ from its base at the top-level AST. All other functions,
capture/retention/grading/kill-reap logic and final post-save clock are unchanged.
There is exactly one future `subprocess.Popen` site. Full assembly reversal
also preserves its startup clock, error projection and execution handoff.
AST facts do not establish subprocess, signal, filesystem or case success.

## Three corrected seams

### Builtin coverage: both all and list are present

The actual pinned controller `failure_packet` uses `list` at line 322 before
`all` at 323. The current `isolated()` tuple includes both, child line 90 /
document 3679. Their insertion supplies the missing pure builtin bindings; it
does not add imports, an I/O primitive or another selected source body.

An independent lexical AST traversal resolved arguments, locals, nested
closures, exception bindings, nonlocal declarations and comprehension targets.
It did not run the archived checker or compile/call selected definitions.
All five selected definitions/prefixes have complete binding coverage:

| Selected body | Injected free names | Free builtin names |
| --- | --- | --- |
| helper `refusal_snapshot_counts` | `re` | `ValueError, bytes, len, type` |
| controller `snapshot_counts` | none | `any, dict, int, len, set, str, type` |
| controller `failure_packet` | `FIXED, json, snapshot_counts` | `BaseException, RecursionError, UnicodeError, ValueError, all, bytes, dict, int, len, list, set, str, type` |
| bootstrap `snapshot_counts` | none | `any, dict, int, len, set, str, type` |
| bootstrap `failure_source` prefix | `helper, json, root, snapshot_counts` | `AssertionError, BaseException, FileNotFoundError, ImportError, IndexError, KeyError, KeyboardInterrupt, ModuleNotFoundError, NameError, OSError, PermissionError, RuntimeError, SyntaxError, SystemExit, TypeError, ValueError, dict, getattr, int, len, str, type` |

No uncovered free name remains. The selected bodies contain no imports and
none of the child-denied `os/open/subprocess/runpy/tempfile/socket/urllib/print`
names. Namespace coverage is a static finding, not proof that the future
functional oracles pass. The child itself still has its declared stdlib
read/hash/memory/output operations; that is distinct from isolated source-body
capability.

### shared_probe has one exact signature exception

The actual helper definition is
`shared_probe(server, config, base, scratch, revoked_token=None)`, line 195.
The current validator accepts that exception only for this name, with exactly
those five positional argument names and one `ast.Constant(None)` default.
It rejects positional-only, keyword-only, vararg and kwarg variants there.
Decorators, keyword defaults, return annotations and annotations on every
argument kind remain rejected. Other selected functions still reject positional
defaults. The actual other selected functions meet these constraints.

Selecting `shared_probe` returns copied AST data. The child does not compile
or execute that surrounding function, SQL snapshot, CLI, serving or sleep.
It finds exactly one `Try` having the explicit `AssertionError` handler,
helper lines 238–247, and compiles only that block in the future payload.
Independent AST inspection confirms the block's only free names are
`AssertionError, BaseException, after_refusal, before_refusal, matrix,
refusal_snapshot_counts, type`, all supplied by its synthetic namespace.

The unchanged authoritative comparison remains
`matrix.require(after_refusal == before_refusal, ...)`. Optional counts attach
only to the exact original AssertionError; diagnostic failure is swallowed
and bare `raise` re-raises the original error. No bucket exclusion, zero-count
success rule, rate exemption, transaction change or relaxed full-snapshot
comparison is introduced. The row-reorder oracle deliberately still denies
different complete bytes even when projected added/changed/removed counts are
all zero.

### Archive selection binds only the uniquely bounded new section

Both new selectors use the exact runnable heading and exact
`wave30_A09_snapshot_memory_runnable_design` closing marker. Whole-report
heading and closing-marker counts must each equal one. Partitioning occurs
after the heading; an earlier-only end marker therefore produces no boundary
and refuses. The selected controller/child marker must occur once within that
section, and its closing code fence must exist. No last-match, old-section
fallback or unbounded search replaces these checks.

I independently inspected the immutable report as text/data: each heading/end
marker and selected block occurs once, the selected bytes equal the complete
new archives, and their integrity digests bind assembly to controller to child.
The report is 254437 bytes, below the unchanged 524288-byte cap. Missing,
duplicate, misplaced or truncated selectors fail in the reviewed source;
I did not execute selector functions or fabricate mutation-case passes.
Hash verification precedes controller/child execution in the prospective
command. The source bytes and hashes alone do not authorize running it.

## Pinned inputs and store-count contracts

The six child and memory-controller PINS lists are equal and unchanged. I
compared every full input to the immutable `d00c968` object, without reading
moving author files or importing any script:

| Input | Bytes | SHA256 |
| --- | ---: | --- |
| `scripts/check-local-edition-transition-postgres.py` | 26547 | `4c60a7448d3c836215068fcc4f6822655c483f172a822e085f077d27b6a0c6fa` |
| `.github/workflows/check-local-shared-handoff.yml` | 82149 | `226483dd9edcd5e566b6c5de9f7ab2a36ebdf0c0f204f8ec992645542a3f395f` |
| `scripts/check-exact-edition-matrix.py` | 18394 | `f07d934f9cfd7af20eb086f5838863c28f840ee848e62bea1d865b330643d887` |
| `scripts/check-installed-release-gate.py` | 22321 | `cbe8dcb42b4b1c36dc8df00204103b9c955feb8057275a441f60f3072d2bf8b5` |
| `scripts/check-local-encrypted-edition-transition.py` | 22045 | `09e137d88eb8e873a488448b7bdfdbe80d2327fbca716a93f445eaaae1ea9e8e` |
| `scripts/spdx_sbom.py` | 36727 | `ca063ab3d4abb6ec815151bcf762447e96682076a7f72d2dbfa9d7e6d3a1032c` |

The pinned workflow's unique controller heredoc extracts to 66841 bytes /
1350 lines, SHA256
`bda0a0fa30db096d92404de22dd5193c91d111c3890e30fe43034405abdd2369`.
Its BOOTSTRAP literal is 5111 bytes / 97 lines, SHA256
`a8f7df1a172e1dc89672d2be54fa147eabf6a61864935e82fd86e2cf64839d33`.
The FIXED import map exactly binds the helper and four other scripts above.
Both `snapshot_counts` definitions are equivalent. The full bootstrap was
parsed/read as a string; its imports, RetainedFixture, runpy/main handoff and
writer were never evaluated.

All six store-count categories and five fields remain exact:
`http_rates`, `http_rate_expiry`, `http_rate_count`, `maintenance_cursors`,
`maintenance_bounds`, `protected_or_other`; and `before`, `after`, `added`,
`changed`, `removed`. The six-by-five projection therefore has 30 integers.
Schema validation requires exact dict/key sets and true int values 0..1000000,
rejecting bool/int-subclass/float/string/null and unknown/missing fields.

The helper parser still limits each full snapshot to 8 MiB and one million
rows, validates complete canonical lowercase hex lines, unique UTF-8/non-NUL
keys with collection separators, decoded key length up to 4096 and opaque
value length up to 1 MiB. Namespace aliases fall into protected_or_other.
Values remain complete hex bytes for comparison; no JSON interpretation,
truncation, normalization or row digest replaces them. These are source
predicates, not actual Store observations from this audit.

The fixed packet remains at most 2048 bytes with allowlisted class and up to
eight basename/line frames (line true int 1..4096). Producer examination remains
64 traceback links, retaining the last eight qualifying frames within that
window. It uses genuine synthetic traceback construction only in the future
child. Optional attachment is exact-AssertionError-only. Invalid base class or
frame shape refuses the packet; malformed counts, duplicate JSON keys, extra
root fields or projection errors discard optional counts while retaining a
valid original fixed class/frames. No raw row key/value, URI, private path, exception text or locals
becomes an optional projected field. This corpus is not a claim that every
possible hostile exception/accessor or provider behavior was exercised.

## Finite cases, receipt ordering and bounds

Static enumeration of the unchanged literal loop domains and direct case
sites gives the prospective cardinality below. No work callback was invoked.
There are 46 direct case-call sites expanding to 89 cases. The controller
retains/validates derived actual totals and unique names; it does not use this
table as a padded target or predetermined passing receipt.

| Group | Prospective cases | Scope of existing assertions |
| --- | ---: | --- |
| source_binding | 9 | six full inputs, two literals, import map |
| counts | 5 | six categories, exact namespaces, two opaque values, identical bytes |
| parser | 24 | malformed/noncanonical records and exact/over size boundaries |
| schema | 14 | exact scalar types, numeric boundaries and complete key sets |
| packet | 20 | fixed shape/cap, duplicate/extra fields, frames/classes and projection fallback |
| producer | 8 | genuine synthetic trace, exact class/attachment, link/frame bound and fallback |
| equality | 8 | same bytes, same failure object/message, reordered bytes and diagnostic failures |
| privacy | 1 | synthetic key/value/hex/path/URI/message sentinels absent from projection |
| Total | 89 | executed cases in this audit: **0** |

`case()` records each attempted work item's finite group/name/status/class;
the finite corpus may collect multiple failures rather than silently dropping
them. Persistent expiry plus post-work clock checks prevent an alarm caught
inside a case from producing a successful run. This is unchanged design logic,
not a measured runtime result. Boundary packet sizes, parser behavior and all
oracle outcomes remain untested here.

The future child runs only selected pinned definitions/prefixes and the
isolated equality Try; the bootstrap writer is removed before compilation and
replaced with return of its existing payload. InertPath operations stay in
memory. Source imports/entry points, real shared_probe, SQL, CLI, network and
native provider are outside the extracted executable bodies. The only future
process is the parent's single isolated Python child; this audit launches none.

The child has a 30-second alarm/persistent flag and startup-inclusive wall
checks. The outer 35-second acceptance ceiling starts before assembly
extraction; setup must leave at least 30 seconds. Source pins and integrity
are required, capacity below 8 GiB refuses setup, and a fresh 0700 evidence
directory is mandatory. Existing evidence refuses reuse. Files are exclusive
0600/no-follow and fsynced, with no rewrite/delete fallback. This audit measured
neither fresh runtime capacity nor private fixture state.

The controller captures at most 65536 stdout / 8192 stderr bytes, samples its
selector at most every 50 ms, and kills only its unreaped Popen process group
on interruption, cap or child deadline. Its owned process has a new session;
ownership is checked before group kill. Emergency reaping has a bounded
five-second wait, and late/unreaped outcomes cannot pass the final ceiling.
The ceiling is an acceptance predicate and alarm/stop mechanism, not an
empirical guarantee about all host I/O latency or cleanup duration.

Numeric child exit/reaping and complete bounded stdout/stderr are saved before
schema/outcome/post-run source grading. Source hashes are recomputed afterward.
Controller acceptance requires exact source hashes, consistent unique case and
group counts, completed/zero failures, actual exit zero, empty stderr, no budget
exhaustion and `native_runtime:false/shared_gate:not_measured`. Receipt failure
refuses acceptance. Final elapsed time is sampled after all receipt fsyncs;
no evidence write follows that sample. Public final output is fixed JSON, not
exception text or raw private stderr. These paths are unchanged and UNEXECUTED.

## Historical evidence and independent checks

All three archived generations remain unexecuted. Their source-derived
namespace/signature/selector blockers and static preparation failures are not
actual memory results. The author's earlier heredoc collision, nonlocal-scope
checker correction and failed all-only `list` coverage remain dated; this audit
does not rewrite their report prefixes or credit a pass to the old designs.

The reported hosted run `37060776569` passed the strict public user-create
refusal (CLI exit 4, HTTP 403, access_denied) and failed complete Store snapshot
equality. Actual changed rows and origin remain UNKNOWN. No raw PG rows,
private protocol evidence, download, service or hosted rerun was inspected or
performed here. This memory projection cannot prove that rate bookkeeping
caused that failure, establish shared/native compatibility or close A09.
Other prior failed PG/helper observations retain their unknown causes. Closed
I02/I10/R05/W02/W05 and other assignments are unchanged.

Actual independent checks used only Git object reads, Python stdlib AST/data
traversal, literal extraction, hashes and byte comparisons. All nine archive
hashes, all six full-input pins, both controller/bootstrap literal hashes and
the exact five-script import map matched. All six diff reversals passed, both
protected-prefix relations held, unique new selectors bound the exact archives,
the shared_probe signature/equality selector matched, and no selected free
name remained uncovered. AST comparison preserved main/cases/assertions and
all controller functions except the declared prepare change. Static finite
case accounting matched 89 across eight groups. No compile/exec/eval of an
archived source or selected function was used to obtain these results.

This sole new report is written on own branch from clean parent
`309ee22e5e813fb3bea31a1f836b9ebb131dac0a`. The completed D01 review, all previous
reports/evidence and every production/helper/workflow/test file remain untouched.
No candidate import/main/SQL/Cargo/network/process/native/PG/case/child/harness
execution, runtime slot, source alignment, other-worker contact, new task/WT/
shell, board/status/main/push operation occurs. Root independently reads and
owns any later exact ONE memory-runtime release, integration and disposition.
No additional local source seam is requested by this review.

Final report checks are limited to docs, whitespace, source pins/archive/data
identity, new-file scope and the immutable one-report commit. The handoff gives
the actual results and clean state; no functional or runtime pass is inferred
from those static checks.

Final actual checks passed: `python3 scripts/check-docs.py` exit 0;
`git diff --check` exit 0 plus a direct new-file trailing-whitespace scan;
all full commit references resolve; and only this reserved new report differs.
A separate report/data comparison verified all 20 SHA256 references against
immutable report/archive/source/literal bytes (exit 0). The preceding complete
diff/AST/binding/selector/cardinality checks also exited 0. Aside from the
disclosed supplied-token lookup message, no independent static failure was
encountered. No candidate was executed. Commit/tree/hash and final clean state
are returned in the handoff.
