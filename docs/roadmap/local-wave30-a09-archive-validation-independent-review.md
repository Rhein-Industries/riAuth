# A09 archive-validation design: independent source review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original A09
`506e3979-a590-4af3-8fa8-ee90d3a517f2`. Existing supporting worktree
`42bb51c6-c198-4adb-bd92-0a5222853231`, branch
`roadmap/local-module-boundaries-wave27`. Reservation:
`wave30_A09_archive_validation_design_independent_review`.
Review date: 2026-10-03. Own starting HEAD
`cb375ac51efee0ea6da48a704aaa02da82577a20`, clean.
Only this new report is written; no source alignment or worker contact.

## Disposition and smallest required hunk

**Hold the current exact design for root's already reserved final-clock
correction. No additional concrete source blocker was found in the complete
payload/supervisor and selected helper bodies.** This recommendation is static
source review, not permission to stage or run the bundle. Retained positive
acceptance, derived members, case outcomes, resource use and owned cleanup
remain unexecuted by this review.

The root's known blocker is present: supervisor lines 242–243 compute `okay`,
then its decision journal is written/fsynced and closed at lines 246–248.
Lines 252–257 print/return that stored decision without checking elapsed time
again. The last `time.monotonic()` call in `main` is line 184, in the joined
observation before envelope reads and decision writes. Thus a late grading,
fsync or close can still produce `passed` despite the declared 125-second
acceptance limit. This is a source defect, not an observed runtime cause.

Smallest prospective seam: after the last journal fsync/close and before the
public result/return, sample monotonic time and refuse final success when
`now >= OUTER_END` or cancellation is set. That guard must control both the
public result and numeric parent exit; a pre-final journal decision must not
be mistaken for final elapsed acceptance. Keep joined raw outcome retention
before grading and do not add a new write after the final acceptance clock.
Root owns the separately reserved exact hunk and new listing hash. No duplicate
correction, harness, fixture or source file was produced here.

There is no source basis to replace a refusal oracle, lower a cap, skip the
retained positive, broaden admitted metadata or change the accepted helper.
The future one-child execution and any container rerun remain HELD.

## Immutable scope and identities

Read the complete 931-line append at
`cbf4560d3ea097e122a092c0686ec3a424724311`:
`docs/roadmap/local-wave30-a09-container-independent-review.md`, including both
full listings, every case builder/oracle, assembly boundary and supervision
path. Its previous 54,888-byte prefix is exact, SHA256
`6566731b11924d35ea06096cb7014358c29cf4aade1602f0f7f4fb69624f17f1`.
The 57,232-byte append hashes to
`b71a02cf83306349dd22ec3b797f49e3e0a41a5c8e762797e681abf2a8985d9f`.
Whole document: 112,120 bytes, SHA256
`fe32d99b55042f272bc7e4077ec254e8151b73b51ca6849407c738b2b2f13095`,
Git blob `1fe92fde5ef45b8c32e090c8f685758c6de21b62`.

| Complete prospective listing, final LF included | Bytes | SHA256 |
| --- | ---: | --- |
| Payload | 25,305 | `f6a57790e77e85153c0a2fa84f03c257fa80931621004a53d67946aa1c0db9e3` |
| Supervisor | 16,622 | `af65db878cda7d529acfe2e4af1c46c3018e7bb9e3e2f42d0601460996b04385` |

Both listings were parsed only as AST text. The supervisor's literal payload
digest matches. Their ordered 42-case registries are identical and unique;
42 is a source registry size, not an attempted/passed count. The shell listing
is a prospective invocation, not an assembly/run performed here.

Accepted helper:
`59f5c6b465a380d270acf68ba7a1719a41998673:scripts/check-local-container-cohort.py`,
99,566 bytes / SHA256
`c5ede6313bf967ab1ab42429e74fb4ca9fcc03c82bd0da2e4433e2b129412837`,
blob `45af8123aa3055e7dffa7c56d6e1eefcd628bb4d`.
Read all selected original support definitions and all three selected methods
in full, including nested exact comparator/encoder and all validator branches.
The six complete source segments have these identities (no ending newline):

| Selected source body | Helper lines | SHA256 |
| --- | --- | --- |
| `Refusal` | 99–100 | `4596647a722d40b790668fd444cc5a72cdd59da4ba8c1a7d51d43af6644ffa19` |
| `require` | 103–105 | `156639c40a6f5fdd446d4fb76cf43156451930f376e6907ef6a87f51d806aaca` |
| `digest` | 108–109 | `b46c442db8153d4caa4e3d7ebb3560f6dc261f2c614ac20722d41b32fb1785fd` |
| `Cohort.check_budget` | 328–332 | `e6d6c489c420481805482f244b83cec5980f5eacad9a588633da6728064064b1` |
| `Cohort.validate_image_tar` | 1030–1102 | `45a928aa378568f3d619c255ffb86ea868ec6e87441b8150ed8e619776f1e3b9` |
| `Cohort.expected_v28_legacy_members` | 1104–1214 | `cad74a557a0f4728b20965bcd0fb791009157badd150dfae65de3febc3b1dd29` |

Read the full 257-line independent source review
`370c141412aa99eab42545d076f92022b537ab53`:
`docs/roadmap/local-wave30-a09-legacy-metadata-independent-review.md`.
Its 18,092 bytes hash to
`db836dd977247695f7d764f0ab61dc26ec33b76b9374f696192bd20e9b4c3322`,
blob `36b869ba30d16eacf0605b7e7a2fc4c241c23b1b`.
Its upstream-source observations/access failures remain that review's
attribution, not newly fetched or independently rehashed upstream evidence.
I did not reread every unrelated original helper method manually.

Independent in-memory removal of exactly the 117 added helper lines restores
every parent byte, SHA256
`45353945c6d867039d29ada0f110cf2c2b85a907d179bb9049268046875ffc73`.
Removing only the new method and its five-line hook restores the complete
parent AST; all 50 other existing Cohort methods are identical. The workflow
is unchanged from the parent, 4,573 bytes / SHA256
`9af6d1d40543bd9604d7105730b97cec9c665d0f197548e50ba9a87eeb6007d4`.
These are source-preservation checks, not runtime assertions.

## Transport pins and decisive positives

The supervisor freezes bounded regular, same-UID, single-link source/receipt/
metadata inputs into exclusive 0600 files only after recording/fsyncing their
observed source hashes and checking exact pins. The child rechecks those frozen
pins before candidate assembly. Original receipt and small metadata pins are:

| Input | Bytes | SHA256 |
| --- | ---: | --- |
| Raw retained public receipt | 16,872 | `ab7c5ce7b87e672db255edc4393ccb27348cb777b14f5f7880f87a68a986ab7d` |
| Public small-metadata document | 17,651 | `430471f20c63eb533003415c19c3d19c8f83b542af7acb75621cfe150040e5b4` |
| Outer ZIP | 49,206,215 | `d150a5950cbe6ff2dc4399614eca2bf4dfa274ac0db755739ef328a29f9c76b6` |
| Unique selected inner image archive | 48,934,031 | `e9e1291895a1cf6c894147acaa0b4efca4a885d4f07dc88f5aee2152c9604905` |

The first three pins were compared against the public member-pin table in
`246e505dddd4defa90ce549441611be893e35d64`:
`docs/roadmap/evidence/wave30-a09-container-37061329815.json`.
The inner pin is design/source-attributed here. Neither ZIP nor inner archive
was opened, rehashed, extracted, derived or validated by me. The root-review
Git wrapper is a different 45,674-byte object, SHA256
`c84ccc9911b17e0bbfbeaceb94a81a4325ace0d5ca353fb30e3366b2c333e039`;
it must not replace the raw receipt in the future input.

Future transport hashes the opened outer descriptor before `ZipFile`, admits
one exact literal inner member with bounded regular/unspecified ZIP mode,
streams only that member into a fresh regular file and checks exact inner
length/hash. Final outer descriptor identity/size/times must agree. It never
uses extract/extractall or opens other ZIP members. Outer/source/receipt/inner
pins precede candidate calls; a pin mismatch stops instead of substituting
another artifact, source, architecture or daemon profile.

The first candidate operation is `retained_archive`: the complete original
validator on the exact retained Essentials export. Only after it passes does
the separate bounded tar pass collect opaque streamed hashes and twelve pinned
public metadata payloads. `retained_derived_six` invokes the original derivation
on those actual inputs and fsyncs count/set digest before comparing with the
six literal external metadata-member pins. Candidate output is not reused as
its own expected set. Member names or raw payloads are not journaled.
All this is a reviewed prospective order, not an observed positive result.

## Isolation and case-oracle review

AST dictionaries have unique source definition/method names. Selection admits
only complete `Refusal`, `require`, `digest` and three named Cohort methods;
the method signatures have respectively 1/4/8 arguments, no decorators,
defaults or keyword-only arguments. `ArchiveSlice` contains no constructor or
other Cohort method and is allocated with `object.__new__`. Exact source
bodies are used, including budget/cap/hash guards. Selected ASTs have no
imports or native/HTTP/SQL/process names. The child's module imports are
stdlib only. No Cohort constructor, module imports/main, Docker/native/CLI,
fixture gate or storage/authority writer enters this candidate seam.
The namespace intentionally retains ordinary Python builtins; isolation here
means fixed trusted AST selection, not a sandbox for arbitrary replacement code.

Tiny fixtures begin only after both retained positives. The two opaque layer
strings are synthetic framing/hash inputs, not extracted filesystem archives.
Base/top Go CreateID and saved-V1 bytes use explicit literal field-order
templates and externally pinned observed Config fragments. No candidate
serializer, exact comparator or derivation function builds their oracle.
Template order matches the reviewed source mechanism: sorted outer ID fields,
preserved nested Config fragments, prefixed chain/ID input, unprefixed saved
parent, intermediate OS added after ID and top OS included before ID.
That source agreement is not newly computed real IDs or a Go execution.

The tiny baseline is a complete validator positive, followed by distinct
negative seams:

| Family | Meaningful source/oracle boundary |
| --- | --- |
| Extra/altered metadata | A plausible correctly hashed but unrelated V1 member must remain unreferenced. Wrong paths, altered bytes, rehashed id/parent/time/OS/config/unknown field and missing metadata must fail exact expected-byte binding. |
| Canonical bytes | Duplicate equal-valued id preserves decoded shape but changes bytes. Malformed/scalar records also refuse; no permissive JSON-member classifier is used. |
| Framing/path | Duplicate or nonregular required member and traversal fail original member guards. Short underlying tar data uses closed `tar_read_error`, not a manufactured copied-size outcome. |
| Layer/config/profile | Manifest/OCI order is checked before legacy derivation; streamed mismatch, rootfs reorder/duplicate/schema, exact boolean typing, unknown top field, invalid/noncanonical time and 129 layers target their distinct early source guards. |
| Caps | Advertised compressed stat overflow is explicitly a scalar probe. Expanded header overflow refuses before payload allocation. An 8 MiB+1 manifest streams without retention, reaching the original missing-small-manifest refusal. No source cap is lowered and no real giant-file test is claimed. |
| Original identity | Wrong tag, selected image, OCI config/index/digest/reference/payload retain the original distinct refusal ordering. |
| Compatibility/budget | Unsupported daemon with legacy extras refuses the extras; a no-legacy altered digest still reaches the original blob check. The Event probe exercises original budget refusal, not an actual disk shortage. |

`observe` enforces the exact next fixed case, records/fsyncs its closed actual
outcome before comparing to the literal expected code, and stops on the first
unexpected result. Expected refusals are negative outcomes, not successful
operations. The reused tiny file is preserved on that first failure; previous
expected inputs have deterministic design text plus per-case hashes.
Canonical date/Go-byte agreement and the tar truncation outcome remain to be
executed under the pinned future Python, not inferred from AST success.

## Supervisor, retention and output limits

Future staging requires fresh mode-0700 owned directory, exact project/source
marker, mode-0600 listings, no symlinks/foreign or multiply linked inputs and
absent output names. Root must independently bind the supervisor hash before
invocation. Both parent and the sole child use `-I -S -B`; the parent rejects
Python other than 3.14.6 or missing waitid/WNOWAIT/session/group primitives.
One Popen at supervisor line 113 uses the same Python, minimal environment,
DEVNULL stdin and a new session. There is no native or Docker subprocess.
No staging path was created and no artifact/library/tool version was executed.

The 120-second child budget starts at parent startup, consuming setup time.
The parent declares 125 seconds total. Before launch, minimum free space must
be at least 10 GiB across input/output filesystems; subsequent two-second
samples and child block/step checks refuse below 8 GiB. This is bounded sampling,
not proof a concurrent drain cannot cross the floor. Peak allocation, disk and
elapsed behavior were not measured here; the final-clock gap remains above.

Waitid/WNOWAIT observes exit while leaving the leader unreaped for any required
group termination. TERM/KILL paths verify PID=PGID=SID before signaling; finally
waits/reaps, closes pipe/selector/log descriptors and requires an empty group.
The post-reap `killpg(pid, 0)` is an emptiness probe, not a termination signal;
an observed surviving group refuses success. No group/cleanup outcome occurred
in this review. Signals/caps/timeouts cannot be relabeled a passing result.
Parent SIGKILL or uninterruptible kernel I/O does not have a proved cleanup or
hard wall-time guarantee merely from these source checks.

Child stdout/stderr are individually capped at 64 KiB and retained privately;
child/parent JSONL caps are 128 KiB/32 KiB. Case/pin witnesses are fsynced before
their expected comparisons. The parent's joined journal similarly retains
actual numeric exit/signal, minimum sampled capacity, elapsed time, capture
sizes and reaping/group flags before child-envelope grading. Partial journals
and nonzero children are not discarded or replaced with expected passes.
The envelope and rows admit only closed schemas, fixed case/reason/outcome
enums, bounded exact integers/digests and a prefix of the ordered registry.
Success requires every case, no first failure, requested/actual zero exit and
proved reaping/group emptiness. The final elapsed acceptance must still be fixed.
Output contains no captured stream, raw exception, dynamic member/path,
protocol/private input or opaque layer bytes. No caches, archives or evidence
are deleted by the design.

## Actual checks, failures and residual acceptance

Actual checks used immutable Git objects, source text, stdlib AST/literal
parsing, hashing and in-memory byte/AST reversal. Archive lengths/digests,
both full ASTs, mirrored allowlists, selected signatures, no-import/native
selection, entire 117-line reversal/full AST restoration, 50 other methods,
pin tables, one-child flags, observation-before-oracle order and missing final
clock witnesses exited 0. None evaluated a source definition, selector,
serializer, template, case builder, stub, constructor, payload or supervisor.
An independent symbolic AST walk also matched the full source order and
literal oracles to the registry: three initial positives followed by 39
negatives. It executed no lambda, operation or builder. These are proposed
source positions, not attempted/passed runtime counts.

One initial receipt-pin witness exited 1: I incorrectly equated the published
root-review wrapper with the raw retained receipt. Inspection of its public
member-pin table corrected that witness, and exact receipt/small/outer pin
checks then exited 0. Neither was an archive/case failure or a source correction.
No retained filesystem evidence was consulted to repair the assumption.
The first symbolic-order witness also exited 1 because it attempted to expand
an unrelated non-observation loop. Restricting only that independent walker to
loops containing an `observe` call produced the complete static match, exit 0.
No candidate code or case was run or corrected after either witness error.

Historical run `37061329815` remains FAILED at
`image_tar_unreferenced_member` after both image builds exited 0. Later load,
UID/mount checks, application execution and E→P→E are unreached. The inner
`e9e12918…` and outer `d150a595…` transport are unvalidated by the new source
until root's separately released future child actually runs. No ARM dispatch,
native image portability, full A09 or O07 closure follows from this review.
Earlier UID/metadata failures, stale notices and native artifact receipts
retain their exact historical scope; accepted receipt-secret/header/PAM/Group
protections and closed rows are untouched and untested here.

Only this new report may differ. No actual archive/path/layer was opened,
extracted or derived; no private evidence, protocol or secret was read. No
candidate/harness/helper/archive-library execution, native/Cargo/CC/Docker/HTTP/CLI/
browser/Driver/network/query/download/dispatch, managed-state action, new
worker/task/worktree/shell, alignment/merge, source-worker contact, main/push/
status action or runtime slot occurred. Driver-only desktop preference remains.
Root owns source acceptance/staging, the final-clock correction, independent
review, serialized future runtime and integration; all runtime remains HELD.

Report validation: `python3 scripts/check-docs.py` exited 1 only for the five
pre-existing private directories `target-wave29-source`, `target-wave28-scim`,
`target-wave28-portal`, `target-wave28`, `target-wave27`; no Markdown link
failure was reported. The directories and checker remain unchanged.
The new-file `git diff --no-index --check -- /dev/null` comparison exited 1
with no whitespace diagnostics (new-file difference status).
`git diff --cached --check` exited 0. Single-file scope and clean post-commit
state are checked for the immutable handoff.
