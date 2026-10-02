# D01 observer-aware controller boundary: independent source review

Date: 2026-10-03 (Europe/Vaduz). Project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`. Reservation:
`wave30_D01_controller_boundary_independent_review`. This is an independent
implementation audit of archived design source, followed by the explicitly
approved creation of this one report. The confidential fixture remains HELD.

Recommendation: hold all-phase fixture release. Immutable `2e29` has four
source-backed boundary findings below. The separately dated, root-reviewed
`c5d4` design supplies only the successful-entry phase correction. It does not
fix the other three findings or supply the missing preparation adapter,
private-input handoff and partial-ownership cleanup. No finding is an observed
runtime failure or an attribution for the historical `c88` failure.

## Immutable inputs and actual review scope

The principal author object is
`2e29c30d01ccd41a2aac3914e3ac20afa38d324f`, path
`docs/roadmap/local-wave30-d01-user-browser-review.md`. Its complete report is
635391 bytes / 9093 lines, SHA256
`c3d7d338676cc1156a9958253c2b51eab823eb25b7478c9ec1b0d4fba37c91a8`.
The selected appended design begins at document line 7428. All document-line
references in the four findings refer to this immutable object, rather than a
moving worktree or the later correction.

The prior cleanup design is
`2bf9ebc2b1dfd1a00ce5c77b9c7489b60d407609`, the same path: 518404 bytes / 7426
lines, SHA256
`2ae110bff13b1943ee8edf0e8b2fb72f38ce417052032cb653fc945d62c4a982`.
Those entire bytes are the unchanged prefix of `2e29`. The fixed public object
is `35c3fd3007c52d8142c7bee1d69aee127cc42a95`. Its report has 520805 bytes / 7441
lines, SHA256
`849f0d158b8ff757c577e574bdafd95b3c2a94f377ac05751a36a7d77272c82c`.
That public report is a distinct archive/publication, not a byte-identical
`2bf9` prefix. I have not imported or aligned any of these files.

Full body reads were completed for the new controller, browser cell, 37-line
preparation boundary, all five readable and embedded collectors, and the
controller/cell/STOP/READBACK diffs. I also read the full retained baseline
controller and 284-line baseline cell, the relevant prior cleanup-design prose
and five-line expectation correction, the two complete public memory receipts,
and all 757 lines of the published descriptor helper. Whole-report hashes do
not imply that every older report paragraph or encoded harness was reread.
The prior memory harness was not executed or given a new independent runtime
result. The later phase correction was inspected as a hunk and its complete
cell was compared byte-for-byte; its separate static checker was not run here.

| Archived source | Bytes / lines | SHA256 |
| --- | --- | --- |
| `2e29` controller command, document 7698–7922 | 17326 / 225 | `5eb6ab2d3d23b49fd431c41f1932082fe0c6779ad724ac77b7fded10926b26b0` |
| Its Python body | 17303 / 223 | `ce8d3f7b5cc08ce114e7f60a69a498305dadcc51956c8824d362213b58fc0efa` |
| Baseline controller command | 14905 / 195 | `6662bdbb168bf0ba6ec5ff24b6f3b88dc15b54b4dfc3b800fdaca1cbee403bbb` |
| Baseline Python body | 14882 / 193 | `271b4a00711a084e21792b0310839e69d4fb8779fa5e6d003bde04caf0c82c72` |
| `2e29` browser cell, document 7987–8449 | 31461 / 463 | `eff0cdad3981ce0ad57d355203b082e21371553408444bf46a3b4ecfcf5fea0a` |
| Baseline browser cell | 18091 / 284 | `0a7cea070bdab32b419a82eb26a7a991b71a2b14a55553f890b53788cda04a5e` |
| Preparation boundary, document 8776–8812 | 2217 / 37 | `1fb412fbf551a3f76595fca551adb2828775cdd3dfa1801e798426557a6e1355` |
| CLOCK | 114 / 2 | `cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d` |
| STOP | 1600 / 29 | `5de5244440b134689bedfb1235e805c65a8b726e753cbdac1d0831ef1c83d071` |
| READBACK | 4424 / 52 | `b7d3504245ae94a584b455fa65826fbaa7759ec7a50a7b81c2cf8284c9f9051f` |
| PERSIST | 805 / 18 | `819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30` |
| MARKER | 626 / 10 | `13c6549e0328ad695769938cbc0be7eb47d7a10ea9f00c4d897655603437a949` |

The shell command has no final newline after its `PY` terminator. Its Markdown
fence needs a delimiter newline that is excluded from the command hash. The
baseline archive uses an adjoining `PY`/closing fence; extracting just the
command, including the terminator, recovers its stated hash. The often cited
193 baseline lines describe the Python body; the complete shell command has
195 lines. All other selected source fences retain their literal final newline.

## Four findings and smallest source recommendations

### F1 — successful entry does not transition its stored phase

The initial cell gate, lines 11–14 / document 7997–8000, requires
`helper_pid===null` and no ready/listener proof while `phase` is
`prepared_entry`. `observeController` later stores the helper PID and both
proof flags, cell lines 133–134 / document 8119–8120. After the final successful
entry poll, cell lines 375–378 / document 8361–8364, there is no assignment to
`own.phase` anywhere in the old cell. The next invocation therefore correctly
refuses the newly populated context under its still-selected prepared-entry
gate. Its refusal occurs before the normal cleanup body. This is a deterministic
source path; no real caller or historical failure is attributed to it.

The minimal correction is the guarded assignment/store immediately after the
final entry poll and before the existing failure cleanup:

```javascript
  if(record.first_failure===null) {
    own.phase="browser_decision";
    store("d01_immediate_owned_handles",own);
  }
```

This is exactly the separately dated design at
`c5d4d173857d040947a8cb3780c47c7141f98336`, already root-reviewed. Its 120-byte
insertion produces a 31581-byte / 467-line cell, SHA256
`d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d`.
Removing that block reconstructs every byte of the `2e29` cell. The whole
635391-byte author report remains its prefix. No await, tool, failure-latch,
deadline, marker or ownership change is in that correction. I independently
verified the byte reversal, not a new lifecycle execution or JavaScript AST
run. The other three source paths below remain byte-unchanged in that cell.

### F2 — non-password actions erase a preloaded private input

The input branch includes click, username and password. Password typing reads
`d01_fresh_password_input`, cell lines 405–407 / document 8391–8393; line 414 /
document 8400 clears that store unconditionally after every input operation.
Consequently, if the single private password handoff is completed before the
listener/entry sequence, a successful initial Sign in click or username input
erases it before the password action. The later password guard then refuses
and cleans up. This conditional source path does not prove how an unavailable
private-input adapter actually behaves. No password value or secret reader
was inspected or executed.

The smallest caller-side correction is to clear the input at that location
only when the consumed kind is password:

```diff
-    store("d01_fresh_password_input",null);
+    if(spec.kind==="password")store("d01_fresh_password_input",null);
```

Pair that change with revocation at the beginning of owned cleanup, before its
first await, so early refusal/exception paths also discard the pending input.
The exact reader/transfer must still be supplied and reviewed. It must bind the
input to the freshly owned fixture and keep its bytes out of public output,
observation records and durable metadata. This recommendation does not invent
its schema, promise repeated secret reads, or authorize later model/terminal
credential transfer. It is not fixed by the `c5d4` phase insertion.

### F3 — the final snapshot can be preempted by successful helper completion

`observeController`, cell lines 137–143 / document 8123–8129, immediately
latches `helper_completed_before_app_checkpoint` for a zero helper exit while
`record.protected_after_page` is not yet true. `continuation`, cell lines
385–390 / document 8371–8376, polls before the explicitly allowed
`protected_after` snapshot. If the protected page has loaded and the helper's
zero exit is queued, but this cell has not yet snapshotted that page, the poll
latches failure and runs cleanup before the final snapshot can execute.

This is consistent with the fixed helper: `Handler.get` lines 554–566 renders
the authenticated protected page, records its status/check and sets
`demo.done=True`; `main` lines 667–673 then checks all security predicates and
finishes. No prior browser snapshot is required by that helper. A previous
generic snapshot can set the cell flag and avoid this path, so I do not claim
that every valid journey fails or that an actual caller took this ordering.

The smallest scoped condition recommendation is to permit the queued zero
exit to reach the existing read-only snapshot only in the already declared
final continuation. Preserve the nonzero-exit and other-stage refusal:

```diff
-      else if(!cleanupStarted&&record.protected_after_page!==true)
+      else if(!cleanupStarted&&record.protected_after_page!==true&&
+              !(own.phase==="browser_decision"&&
+                own.next_decision?.kind==="protected_after"))
         latch("helper_completed_before_app_checkpoint",receivedWall);
```

The next operation remains `get_browser_state` with the same bound handles and
the existing `page(...,"protected_after")` predicate. No new protected HTTP
request, navigation, credential submission or retry is needed. Missing page
proof still latches failure; successful proof proceeds directly to the existing
stop/cleanup. A zero exit alone earns no journey credit. Root must review this
proposed boundary against the supplied step adapter and any separately
reserved memory envelope before use. It is not implemented or executed here,
and it does not fix other possible asynchronous step-predicate limitations.

### F4 — incomplete controller events are discarded between cell invocations

`controllerBuffer` starts empty on every invocation, cell line 93 / document
8079. Lines 112–118 / document 8098–8104 concatenate tool output and parse only
newline-terminated records, correctly preserving a partial suffix within that
invocation. Neither successful entry nor continuation saves/drains that suffix
before final output. Only `own` and the finite observation record are stored.
The next invocation begins with a new empty buffer.

A controller observation split across two calls can therefore lose its first
fragment. The remainder can become invalid JSON, or a fragment containing a
failure/completion observation can remain unprocessed while an otherwise
successful cell returns. This is a conditional chunk-boundary defect, not an
observation of actual MCP chunking or a retrospective origin claim. The old
18-case memory pass covers its own within-cell partial JSON case; it does not
establish the changed multi-invocation transport boundary.

The smallest privacy-preserving recommendation is a same-cell drain before
the final output boundary (cell line 445 / document 8431): while a partial
record remains and no failure is latched, poll only the same owned controller
within the existing active deadline. Retain each numeric receipt before
comparison, keep the existing 16384-character cap, and latch a fixed failure
and complete cleanup if framing cannot finish. Do not persist unvalidated raw
output or a partial diagnostic string in `own`, durable evidence or public
output. This recommendation deliberately supplies no new timeout/cancellation
guarantee or larger budget. The exact drain placement and its finite call
behavior need root's source reservation/review; no loop was implemented or run.

## Boundary and privacy assessment

The explicit START placeholder refuses as archived. The distinct output and
provider basenames are also release placeholders. Actual substituted source,
fresh artifact/helper/provider hashes, absent output paths and capacity must
be reviewed for any later invocation; this audit gives none of that runtime
evidence. The unchanged native/CLI/HTTP commands and artifact pins are source
facts, not freshly executed checks.

The controller creates the owned private lab, 0700 subdirectories, an exclusive
0600 password file and CLI client-secret file before the marker permits the
RP listener to start. However, the cell's password store has no supplied
reader/transfer body. The preparation contract also calls an unavailable
`projectOwnedIdentity` adapter and `cleanupKnownPreparationResources`. Its
prepare/list-windows/bind sequence is commented source, not a completed caller.
These are concrete prerequisites. No generic CallToolResult is treated as an
attested PID/window/session/target/tab schema. No unknown browser PID may be
killed, and no missing ownership/absence proof earns release credit.

The main cell uses the cached session, target and tab for browser operations.
Only refs from the last accepted snapshot matching its provider-token syntax
survive; an input consumes that ref list before awaiting dispatch. This is a
source guard, not proof that a returned ref is a specific password textbox.
The future exact step adapter must establish that association before transfer.
Post-action public role/label matching and an `unverifiable` dispatch receipt
are insufficient proof of credential insertion or journey completion. The
source correctly leaves `journey_credit:false` in every result branch.

The synchronous first-failure latch precedes cleanup awaits and cannot be
replaced by later observations. Controller/helper completion and browser
predicate failures use fixed labels. The observer transport admits only
`observed` plus null or the four-key diagnostic; site, exception class and own
function are finite allowlists, and function/line must be jointly null or a
whitelisted function with a true integer in 1..1024. Python bool is not an
accepted integer. An observed latch with null diagnostic remains valid.
The first projected observation is retained, including a null diagnostic;
malformed projection gets a fixed diagnostic error without a raw fallback or
mutation of first failure, outcome or independently established absence.

`retain_observation` sanitizes the nested helper field at both load sites as
well as the top-level field. The JavaScript and READBACK projections repeat
the same closed shape. No exception message, arbitrary class name, filename,
trace values/locals, request header/query, token/cookie, subject or sender is
added to the finite public diagnostic. The fixed helper's builtin traceback
descriptor avoids hostile overridden traceback getters. Those source facts
are distinct from the root's actual memory result below.

Cleanup requests the original stop, then the exact owned browser kill/session
end/window check, owned controller join, fresh PID/port/lab readback and an
exclusive metadata write. Normal final cleanup writes stop without ui-failure;
failure cleanup retains both markers. READBACK opens bounded canonical owned
0600 evidence without following a symlink, checks six named child exits if the
helper never started or seven if it did, and only learns a missing helper PID
from the canonical record plus matching named exits. It projects fixed fields
and checks the known PIDs and ports; it does not read arguments/environment.
The absent partial-ownership preparation body cannot inherit these proofs.

Metadata/Driver errors remain recorded even when independent join, session,
window, PID, port and lab proofs establish resource release. Missing proof or
an unjoined collector prevents release. A collector returning a session ID is
recorded as unjoined, not assumed complete. The canonical evidence cap is
262144 bytes; the controller text buffer cap is 16384 characters. CLOCK and
PERSIST retain decimal clock precision rather than rounding through JS Number.

The source retains START+840 active / START+900 inclusive timing, 180-second
preparation, helper 600 / pending 180, native and HTTP 5, CLI 60, listener 30 and
the 8.5 GiB sampling floor. Driver cleanup allocations 30/15/5 seconds are
observational: the advertised invocation shape in this design supplies no
hard per-call timeout/cancel. The collector/tool yield settings are also not
hard cancellation. Essential cleanup continues when late. Both true-first-event
proof and whole-cleanup-within-60 remain false. This audit did not inspect
current Driver state or invoke any GUI tool; future desktop interaction remains
RiWork Cua.ai Driver MCP only, with descriptions/state checked first.

## Actual published memory evidence and historical limits

At fixed public `35c3fd3007c52d8142c7bee1d69aee127cc42a95`, I read the complete
`docs/roadmap/evidence/wave30-d01-descriptor-memory-0ec651d.json`: 5638 bytes /
155 lines, SHA256
`5df541995dd1f74bff9c91104400140040d986a98a9f13f8688fbcb82cc4782a`.
Root executed one exact payload SHA256
`b10cf8376edbb80b00c051305f31643b04714494260c4a49576a354361048779`:
128/128 completed, 50 observer and 78 legacy, child exit 0/reaped, one spawn,
no first failure and no cleanup failures. Child elapsed 0.101654 seconds;
controller elapsed 0.201888 seconds, within35 true. Stdout was 1398 bytes,
SHA256 `006d33cc7314d7c483f99dcb8ed59d3b6b537571b5908ee4178d14c21094024f`;
stderr was empty. Getter/str/repr traps were zero. Legacy handler-call counts
were 44 baseline / 80 candidate. The receipt records the helper SHA256
`75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1`.

The published helper and source
`f4ef05d8428b235511e81277ba6b4b72d5ec08ba` match at 35749 bytes / 757 lines.
That actual pass covers selected helper definitions, memory sinks and genuine
controlled Python frame/traceback captures, with synthetic line/depth
annotations. It does not execute this controller/cell/preparation/cleanup,
module main/server constructor, native provider, CLI, HTTP/listener, GUI or
confidential browser journey. I did not execute the payload. The pending or
UNEXECUTED language in the older `2e29` appendix remains a dated observation;
this later published receipt updates context without rewriting that prefix.

I also read the complete public
`docs/roadmap/evidence/wave30-d01-cleanup-memory-2bf9ebc.json`: 10004 bytes /
368 lines, SHA256
`f2832e4397e69e59bb86680d975f46a06ca1dbdb7c830d4f949823be11bc989a`.
The root's one old-cell memory invocation exited 0, completed 18/18 cases,
and reaped its child. Its 286 stub tool calls comprise 78 Driver / 33 controller
/ 175 collector calls (127 clock, 16 stop, 16 readback, 16 persist).
Elapsed was 0.15420270897448063 seconds, stdout 5621 bytes, SHA256
`91f8865ac7715c9a776c91b5a6645ac8bec7e58fc970067629a3f66d430a9776`, stderr
empty. That pass belongs to the old 284-line cell, not the changed design.
The corrected three absence expectations retain Driver errors/lateness and
whole60=false; the prior strict expectation was a root requirement error,
not an executed failure or a product repair.

All older capacity/preparation/provider/Authorization/request failures, lost
provider values and report corrections remain dated. The 61/76 failed memory
verification, historical 78-case pass and later actual 128-case pass are
distinct. The historical `c88` unexpected failure, zero Authorization count,
protected-before 403 and lack of journey remain unchanged. Its sender, cause,
true first cleanup event and whole60 are UNKNOWN; the roughly 119-second final
absence readback after controller completion does not establish an earlier
cleanup anchor. None of the four candidate findings is retrospective runtime
attribution. No original D01/D05 completion or other closed-row change follows.

## Checks actually performed and scope preservation

The source audit used Git object reads and static stdlib parsing/comparison.
The full archived zero-context controller and cell diffs reverse byte-for-byte
to the 14905-byte `6662` command and 18091-byte `0a7cea` cell. Python AST
comparison, excluding position attributes, preserves all nine original
controller functions; only `finite_observation` and `retain_observation` are
new. Each of the five embedded collector literals equals its entire readable
archive and parses as Python. CLOCK and PERSIST also equal their old literals.
No JavaScript AST execution/parsing result is claimed from this audit; exact
JS byte reversal reconstructs the baseline parser input. The author's and
later phase designer's separate syntax/AST results are their static evidence.

Two audit preparation errors were corrected without candidate execution: an
initial baseline fence extraction could not find a standalone PY terminator
(ValueError, exit 1); a first reversal attempted command-coordinate hunks
against the Python body (assertion, exit 1). An intermediate extraction also
omitted the final terminator and therefore did not reproduce the canonical
command identity. Corrected delimiter/index handling and reversal against the
whole command recovered the exact hashes above; final static comparisons
exited 0. None is a helper, controller, fixture or protocol failure.

The originally requested unexpected-failure report path was absent from this
branch and the fixed public tree. The first audit proposed this new path and
ended without a write or commit while its reservation was unresolved. That
was an orchestration/path limitation, not a runtime refusal or an executed
fixture. Root subsequently approved this exact sole new report; no further
confirmation was required.

The report is created on own branch from clean parent
`370c141412aa99eab42545d076f92022b537ab53`. All previous independent reports,
source, histories and failed evidence remain untouched. There is no source
materialization, alignment/merge, cleanup/delete operation, helper/collector/
controller/cell/harness execution, native/provider/HTTP/browser/Driver/Cargo
invocation, worker contact or runtime slot acquisition/release. Root retains
source reservations, runtime, interpretation, integration/publication and
status decisions.

For this report handoff, `python3 scripts/check-docs.py`, source/pin/archive
identity checks, new-report scope checks and `git diff --check` are the only
verification classes. The own-branch documentation checker passed in the
initial read-only audit. The source author's known protected-prefix link
false positive is separately reported in its immutable appendix, not silently
relabelled as a local checker result or repaired here. Final checks and the
one-file commit are recorded in the handoff; no runtime pass is claimed.

Final actual report checks passed: `python3 scripts/check-docs.py` exit 0;
`git diff --check` exit 0 plus a direct new-file trailing-whitespace scan;
all referenced 40-character commit pins resolve; all three fenced snippets
are closed; and exactly the reserved new report is untracked/changed before
staging. A separate static comparison verified all 19 archive/report SHA256
references, both complete diff reversals, nine unchanged function ASTs, all
five readable/literal collector identities, the later phase-only 120-byte
reversal, both exact published memory receipts and helper publication identity
(exit 0). These checks parsed data/source only. No candidate was evaluated.
