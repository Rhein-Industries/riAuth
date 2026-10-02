# D01 continuation corrections: independent source review

Date: 2026-10-03 (Europe/Vaduz). Project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing supporting worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`. Reservation:
`wave30_D01_continuation_correction_independent_review`.
The sole write is this new report, from clean own parent
`f1e9e386a74cdde44f3adaf6ac1ef5241c9b270b`; no alignment or merge.

Recommendation: accept `9f3a4a` as the source-only correction of F2/F3/F4 in
the earlier independent boundary review. I found no additional concrete
blocker in the four changed spans and their reachable owned continuation,
observation and cleanup paths. The accepted F1 phase insertion is preserved.
No further correction hunk is proposed by this review. The complete candidate
is UNEXECUTED. Root must separately review its composition with the exact
preparation/input/partial-ownership design and any future memory or real-fixture
reservation; this recommendation supplies no runtime release or journey credit.

## Immutable inputs and full-body read scope

The principal author object is
`9f3a4a372b53fc6d73d5df45ad1a8d217ad60879`, path
`docs/roadmap/local-wave30-d01-continuation-correction-plan.md`.
I read its entire 961-line report: rationale, complete four-hunk diff,
complete 485-line candidate, preservation tables, complete 71-line static
checker and dated corrections/limits. I read the entire 467-line phase-fixed
cell at `c5d4d173857d040947a8cb3780c47c7141f98336`, document 9142–9608 of
`docs/roadmap/local-wave30-d01-user-browser-review.md`, and the complete
378-line [earlier independent boundary review](local-wave30-d01-controller-boundary-independent-review.md)
at `309ee22e5e813fb3bea31a1f836b9ebb131dac0a`.

For event and ownership context I reread the complete 225-line controller
command and 37-line preparation boundary at
`2e29c30d01ccd41a2aac3914e3ac20afa38d324f`, the same author-report path,
document 7698–7922 and 8776–8812 respectively. All five complete collector
values are visible in the cell and were parsed as Python source only.
The original 463-line cell and complete old report were also extracted and
compared as data; that is a whole-byte comparison, not a claim to have reread
every older report paragraph or encoded payload in this slice.

At `35c3fd3007c52d8142c7bee1d69aee127cc42a95` I read helper lines 532–682
of `scripts/d01-confidential-browser-demo.py`: cookie handling, callback and
protected response, renderer and main's completion predicates. The whole
helper was hash-checked and equals the entire file at
`f4ef05d8428b235511e81277ba6b4b72d5ec08ba`. This slice is a selected-body
helper read; the earlier boundary review records its separate full 757-line
read. Neither is helper execution.

| Source/data object | Bytes / lines | SHA256 |
| --- | --- | --- |
| Entire `9f3a4a` report | 61754 / 961 | `fd32648eac52c94add32138f4721a96e16f56ce4d6858964363516160410a1db` |
| Complete candidate, author document 227–711 | 32502 / 485 | `7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8` |
| Complete four-hunk diff | 2243 / 52 | `395948657075c7fb27503b507d18303085755af9e531303197b964adc7ecf144` |
| Complete archived static checker | 4514 / 71 | `91160d966e2e9078e9b9406b31a8f23a3827b174fb66a676ea5273d2e4be2007` |
| Entire phase-fixed `c5d4` cell | 31581 / 467 | `d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d` |
| Entire original `2e29` cell | 31461 / 463 | `eff0cdad3981ce0ad57d355203b082e21371553408444bf46a3b4ecfcf5fea0a` |
| Unchanged controller command | 17326 / 225 | `5eb6ab2d3d23b49fd431c41f1932082fe0c6779ad724ac77b7fded10926b26b0` |
| Unchanged preparation boundary | 2217 / 37 | `1fb412fbf551a3f76595fca551adb2828775cdd3dfa1801e798426557a6e1355` |
| Earlier `309ee22` review | 23341 / 378 | `68c5f02de8d16aec39adaf582f17f7de4595267354260fb45e467378e15b24c4` |
| Entire descriptor helper | 35749 / 757 | `75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1` |

The candidate's final newline is included. The controller command's hash
excludes its Markdown delimiter newline after the final `PY`; its Python body
is 17303 bytes / 223 lines. No archived source was materialized as an executable
file. Candidate-line references below mean this exact 485-line cell;
author-document line = candidate line + 226.

## F2: retention through dispatch and revocation on owned cleanup

The candidate's input-kind/ref/bound-handle checks remain intact at lines
386–420. `spec` is selected from the owned next decision; only the original
five kinds are admitted. Click/username/password still require a ref from the
fresh snapshot list, consume that list before awaiting input, and use the
unchanged session/target/tab. Password validity is still checked at 413–415
before dispatch; the operation reads the ephemeral input when it invokes the
original `browser_type`. No new reader, file, secret copy or schema is added.

Line 421 now clears only `spec.kind==="password"`, after the awaited input
dispatch and before its follow-up snapshot. Initial click and username steps
therefore no longer erase a preloaded password. Snapshot-only steps do not
consume it. A returned dispatch still needs the same `confirmed` or
`unverifiable` effect and public follow-up predicate; neither effect becomes
proof of actual insertion, authentication or completed application access.

The first cleanup statement, line 194, clears the same store key before the
line-195 idempotency return and before any await. Thus every call entering
owned cleanup, including a repeated call, revokes the pending input immediately.
Invalid kind/ref/password, failed pre-poll, pre-dispatch active deadline,
Driver refusal/exception, missing post-action page proof, drain failure and
the outer catch all retain their existing route to owned cleanup. If dispatch
never runs or fails, cleanup supplies revocation; if it returns successfully,
line 421 supplies consumption even before a later failure. A second cleanup
request cannot restore or resend the value. Normal protected-after cleanup
also clears it.

The ownership/context and already-stopped refusals at lines 9–30 precede owned
cleanup and remain unchanged. This hunk does not assert cleanup of unknown or
partially owned preparations. Exact private-input handoff, ref-to-field
association and revocation when preparation fails before this owned cell
remain the separate adapter's responsibilities. Clearing a store key is not
proof of cryptographic zeroization or cancellation of a pending provider call.
No private input or actual dispatch was observed in this review.

## F3: narrowly permit zero to reach the existing final snapshot

The only added allowance for a zero helper exit while page proof is absent is
the exact conjunction at lines 142–145:
`own.phase==="browser_decision"` and
`own.next_decision?.kind==="protected_after"`.
It is confined to the existing zero branch. `event.exit!==0` at line 141 still
refuses nonzero, missing, null and nonnumeric exits; the original first failure
cannot be overwritten. Other decisions/phases without the protected-page flag
retain `helper_completed_before_app_checkpoint`. Existing behavior when page
proof is already true is preserved, not a newly widened exemption.

The same observer still refuses fixture/controller completion before cleanup,
including a numeric exec exit. Readiness still checks the guard/server/helper
identities and lab against the owned context. Malformed framing, cap violations
and unavailable transport retain their fixed refusals. The added condition
does not suppress these independently reachable failures or mark the page true.

At lines 386–397 the continuation validates its declared kind, polls the same
controller, then invokes only `get_browser_state(snapshotArgs)` for
`protected_after`. The full `page` body is byte-identical: error-label absence,
status ok, complete snapshot, exact protected heading and exact authenticated
public text remain required. Only that successful predicate sets the page flag.
A refused, incomplete or wrong page still latches failure and cleans up. A
successful final snapshot goes through the original post-snapshot poll and
normal stop/cleanup; it adds no navigation, RP HTTP request or authentication
retry. Every success output still has `journey_credit:false`.

The source ordering is defensible against the unchanged helper and controller.
Helper lines 554–566 send the protected response and set the accepted-cookie
check and `demo.done`; main lines 667–673 then require all checks and finish.
Neither waits for a caller snapshot. The controller announces helper completion
once and, for zero, continues waiting for the original stop marker while
checking its active deadline and server. Therefore a queued zero event can
precede the caller's final snapshot without controller completion. Conversely,
nonzero completion or an independent controller exit still refuses. This is
source reasoning, not an observation that a historical browser took this race.

The allowance is not an inference that exit zero establishes token, cookie,
factor, consent or GUI correctness. Those helper/security predicates and all
native/CLI/provider pins remain untouched. It repairs the preemption of the
existing read-only proof step; it supplies none of that step's runtime evidence.

## F4: same-invocation partial-line drain and error paths

The inserted top-level block is exactly candidate lines 448–462. It executes
inside the existing try after entry/continuation and before catch/final output.
While a partial lexical `controllerBuffer` remains and no first failure exists,
it polls only the same `own.exec_session`. No model yield, rebind, new reader,
command, HTTP probe, marker, raw-output sink or cross-invocation buffer is added.

Before each extra poll and after its return it compares wall time with the
unchanged START+840000 active boundary. At the boundary it latches the existing
fixed deadline label. An unavailable/joined controller with an unfinished line
is refused; event/transport failures keep their own first label. If the final
fragment arrives, the unchanged observer parses the complete line; if the
buffer becomes empty without a failure, the cell returns normally. Empty
buffer means no drain poll. A completed malformed line, >16384-character
buffer, helper refusal, transport exception or late return reaches the original
cleanup. No partial raw string can survive as a stored continuation value.

`pollController` and `observeController` retain their order: capture receive
wall time, append/retain the numeric result projection, then compare exit,
cap, framing, readiness and helper/controller events. Each additional drain
poll therefore has the same receipt-before-comparison boundary. The local
buffer's 16384 limit is a character cap, not a new byte claim. Diagnostics
retain only their existing closed projection; raw event/partial strings are
not stored in `own`, the observation record, public output or durable evidence.

A drain failure invokes cleanup before final output at line 462; an unexpected
exception uses the unchanged outer catch. The first cleanup clears the password
and eventually clears the lexical buffer; repeated calls clear the password
before the idempotency return without issuing another stop sequence. Successful
final-snapshot cleanup already clears the buffer, so no additional drain or
second logical browser operation is introduced afterward. Entry's phase update
still occurs at its exact original position; a later drain failure closes that
same owned context rather than admitting a successful next step.

This is bounded by comparisons against the existing deadline on a progressing
wall clock. It is not a fixed iteration cap or a hard timeout/cancellation of
`write_stdin` or a Driver call. A stalled/backward clock or awaited call returning
late is not repaired by these four hunks. Cleanup still uses START+900000 for
the inclusive join and continues essential work when late; none of the 840/900,
30/15/5 allocations is reset or promoted to a hard cancellation guarantee.
The source still leaves true-first-event and whole-cleanup-within60 unproven.

## Independent preservation proof

An independent static check extracted all three new fences and checked their
sizes/hashes. It reversed every context/addition in all four unified hunks
against the complete candidate, reconstructing every `c5d4` cell byte.
Removing only the retained 120-byte F1 block then reconstructed every original
`2e29` cell byte. The entire old `2e29` report remains the `c5d4` prefix.

I also used a separate parser-only Acorn check on source strings received via
stdin. It did not execute the archived author checker, evaluate/compile a
candidate, create a VM or invoke any referenced tool. The structural inverse
removed exactly cleanup's first clear, restored the unique post-input clear
and helper-zero condition, and removed the exact drain/failure-cleanup statements
from the top-level try. The entire normalized AST then equaled `c5d4`; removing
the exact entry phase block separately equaled `2e29`. Position fields alone
are excluded; BigInt literals retain their decimal values. All comparisons
passed, parser process exit 0.

| Complete normalized AST | SHA256 |
| --- | --- |
| Original `2e29` | `103b85f3ea8e2b805af3afb031b56ac5378595130f9f8bac38d43df7695d472a` |
| Phase-fixed `c5d4` | `28a72ac4b2b27a5c6f6aeee63883557580e4b63e9de097c9339e9dd9fd33d750` |
| Untouched new candidate | `786f743a839085292b57dc0c695a97c338d57eef9ab314168ff081da3e100a74` |
| Four-edit structural inverse | `28a72ac4b2b27a5c6f6aeee63883557580e4b63e9de097c9339e9dd9fd33d750` |

There are thirteen complete function declarations. Exactly ten source spans
are byte-identical: `finiteObservation`, `diagnostic`, `latch`, `pollController`,
`timedDriver`, `checked`, `page`, `navigateAndSnapshot`, `entry` and
`publicPredicate`. Only `observeController`, `cleanup` and `continuation` have
the three declared function edits. F4 changes the top-level try alone.

| Changed complete function | `c5d4` SHA256 | Candidate SHA256 |
| --- | --- | --- |
| `observeController` | `702b8e20d30f9d9044293a951a52afe63bb6e86847d5e0a2c65da54c88588b5d` | `1e8968dd48f70ab7c6b88fbb250e1e4ba57879604a596a8d901a1f0bd635cb16` |
| `cleanup` | `847d37e4b772570ddb5822ee2c22ebe21cba9f0a4c34adb04f44e5826ab27537` | `0d8dc617795d72e24519b956f324cf08145838e157840418f107e8436457ffca` |
| `continuation` | `905a6e1a938488a17ad232aaa21f895c0fcc45b7c554756f05245e2e4e0573b0` | `04365b46726732d62762a351c82df05cb7354b2aefb65600674898068410c0cb` |

All five collector declarations and unescaped bodies are exact. Each body and
the unchanged controller Python body parsed with `ast.parse` only; no import,
main, command, collector or stub was invoked.

| Collector value | Bytes | SHA256 |
| --- | --- | --- |
| CLOCK | 114 | `cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d` |
| STOP | 1600 | `5de5244440b134689bedfb1235e805c65a8b726e753cbdac1d0831ef1c83d071` |
| READBACK | 4424 | `b7d3504245ae94a584b455fa65826fbaa7759ec7a50a7b81c2cf8284c9f9051f` |
| MARKER | 626 | `13c6549e0328ad695769938cbc0be7eb47d7a10ea9f00c4d897655603437a949` |
| PERSIST | 805 | `819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30` |

Whole-byte reversal also preserves the initial ownership/phase gates, single-use
refs, first-failure latch, full page/public predicates, fixed diagnostic schema,
normal/failure stop markers, exact-owned cleanup, readback and exclusive writes.
The 262144-byte canonical-evidence cap, 0700/0600 ownership, six/seven named
child-exit contracts, clock precision and 8.5 GiB sampling floor remain intact.
No helper/controller pin, original deadline, data writer, credential receipt,
header, PAM, held Group or other accepted product contract changes here.

## Separate preparation contract and historical evidence

I read the complete public blank-Driver receipt at
`8a2c4a192d04a43cd930141506a5215cca343542`, path
`docs/roadmap/evidence/wave30-d01-root-blank-driver-contract.json`:
3893 bytes, SHA256
`b8145f029796a2063dfe167f026fe299d85afcaedbec5a7abacbeb5bf1a5ea27`.
It records one isolated about:blank prepare/bind/snapshot survey and completed
exact-owned cleanup. Cooperative close was unverifiable and left windows;
exact-owned kill, session end, final window absence and PID absence followed.
Its successful raw contract does not establish all refusal/failure variants,
input handoff, hard MCP deadlines or a confidential app journey. It expressly
did not execute the preparation adapter or `c5d4` continuation.

The assignment identifies root's separate adapter and an unexecuted preparation
with four NULL inputs, with Sol4 supplying its exact observed contract. I did
not inspect mutable author files, fill those NULLs, guess receipt schemas,
contact its author or broaden coordinator/preparation source ownership.
The original boundary's `projectOwnedIdentity` and
`cleanupKnownPreparationResources` remain prerequisites to the later composed
review; the blank survey alone does not implement them. This audit cannot
attribute, kill or prove absence of an unknown preparation resource.

Both complete published D01 memory receipts were reread at
`35c3fd3007c52d8142c7bee1d69aee127cc42a95` as data:

| Historical actual receipt | Exact outcome and scope | Bytes / SHA256 |
| --- | --- | --- |
| `docs/roadmap/evidence/wave30-d01-descriptor-memory-0ec651d.json` | Root 128/128 PASS: 50 observer + 78 legacy; child 0/reaped, one spawn; child 0.101654s/controller 0.201888s. Selected helper definitions and controlled Python memory/traceback sinks only. | 5638 / `5df541995dd1f74bff9c91104400140040d986a98a9f13f8688fbcb82cc4782a` |
| `docs/roadmap/evidence/wave30-d01-cleanup-memory-2bf9ebc.json` | Root old-cell 18/18 PASS, exit 0/reaped, 0.15420270897448063s; 286 stub calls: 78 Driver/33 controller/175 collector. Old 284-line cell only. | 10004 / `f2832e4397e69e59bb86680d975f46a06ca1dbdb7c830d4f949823be11bc989a` |

The descriptor payload is
`b10cf8376edbb80b00c051305f31643b04714494260c4a49576a354361048779`;
its stdout was 1398 bytes,
`006d33cc7314d7c483f99dcb8ed59d3b6b537571b5908ee4178d14c21094024f`,
stderr empty and getter/str/repr traps zero. The old-cell stdout was 5621 bytes,
`91f8865ac7715c9a776c91b5a6645ac8bec7e58fc970067629a3f66d430a9776`,
stderr empty; its partial-JSON case concerns its own within-cell model, not
this changed multi-invocation boundary. Neither receipt executes `9f3a4a`.
The separately reported current A09 89/89 memory and 42/42 archive passes are
acknowledged only as separate root context, with no D01/shared/native gate
inference. I did not query that or any pending hosted run.

All earlier capacity/preparation/provider/request/Authorization failures,
61/76 failed memory verification, historical 78-case pass, later 128-case pass
and old 18-case pass retain their distinct pins/outcomes. Historical `c88`
unexpected failure, zero Authorization count, protected-before 403 and no
journey remain unchanged. Its sender/cause, lost provider values, true first
cleanup event and whole60 remain UNKNOWN. The roughly 119-second final
readback does not establish an earlier cleanup anchor. F2/F3/F4 corrections
are source findings, not retrospective explanations of that failure.

The author also retains its static preparation corrections: controller fence
delimiter extraction failure, a successful inverse check with an initially
aliased auxiliary candidate-AST hash, and final archive-checker stray-identifier
failure plus the draft diff-line-count correction. Those were corrected static
checks without candidate-byte changes or execution. My independent extraction,
byte/AST/collector comparisons passed on their first invocation; no runtime
failure or pass is inferred from either author's or this review's parsing.

## Actual checks and handoff boundary

Actual checks in this slice: immutable Git object resolution/read/hash/size
checks; full-body source review described above; context-checked four-hunk
byte inverse and retained phase inverse; independent parser-only complete AST
inverse, thirteen-function scope and five collector identities; Python AST
parsing of source strings; complete public JSON data reads and fixed hashes.
All these checks exited 0. The own-branch pre-write
`python3 scripts/check-docs.py` exited 0: Markdown links/build-directory layout
checked. The author's different branch five-directory checker failure remains
its dated failure and is not relabelled as this result.

There were ZERO candidate/case/cell/controller/collector/helper/adapter
executions, VM/eval/stub/memory-case calls, Driver/browser/HTTP/provider/native/
Cargo/service calls, private protocol/input reads or runtime slot changes.
No GUI, crypto, journey, human, deployed or completion evidence was created.
Future desktop work remains RiWork Cua.ai Driver MCP only with descriptions
and state inspected first; this review used no desktop provider.

Only this new report is written. Previous reports/failures/history and all
source remain untouched; no merge/alignment, other-worker contact, new
worker/task/worktree/shell, delete, main/push or board/status action occurred.
Root retains disposition, source composition, runtime reservation, integration
and publication. Final docs/pin/scope/whitespace checks and the sole immutable
report commit/clean proof are recorded in the handoff.
