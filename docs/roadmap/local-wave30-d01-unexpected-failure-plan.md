# Wave 30 D01 unexpected-failure finite observation plan

Date: 2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`;
existing WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`.
Reservation: `wave30_D01_unexpected_failure_finite_source_design`.
Only this new plan is written. Existing branch HEAD at entry is
`7ecc79bde1627f4729c28f918d0d4cf5effe4c9f`; no alignment or source import.

## Observed failure and limits

The two explicitly authorized finite REDACTED documents were read completely,
hashed and mode-checked. No unredacted log, credential or protocol file was read.
Both files are regular 0600 files under the management WT's deployment-private
directory. These observations concern retained receipts, not new process/port
or desktop queries.

| Authorized document | Bytes | SHA-256 |
| --- | ---: | --- |
| d01-confidential-browser-budget180.redacted.json | 4099 | 83549d45aec2900fae09b1ec2ea4bd7792945a5508bde085568b59506a08cd9f |
| d01-confidential-browser-budget180-cleanup.redacted.json | 2612 | 2ac06f21da0aff2874094788cd227397f3a67b64dafa2a434e5d5a6e8ba47dda |

Exactly one helper invocation failed, exit1. Helper failure is
unexpected_failure/request; outer failure is rp_nonzero/browser_checkpoint.
Protected-without-cookie returned403 and its check is true. The actual app
page's generic “Local demo could not complete this request” observation is
supplied by root; the retained helper record has no exception class or trace.
Authorization refusal count is ZERO and request_invalid_reason is null.
Authorization redirect, callback, token exchange, userinfo and protected-after
statuses are null; password, consent and callback remain UNRUN. Five CLI
prerequisites exited0. Helper elapsed10.096s and outer elapsed111.287s are
receipt values; they are not a new timing measurement or proof of cleanup start.

The result reports closed listener/connection, cleared private references,
removed verifier temporaries/lab, reaped owned children and absent ports.
Separate cleanup receipt reports session ended, window count0, owned PIDs/lab/
ports absent. It also explicitly reports first_event_proven:false,
first_failure:first_event_unproven and whole_cleanup_within60_proven:false.
The true initial cleanup event is unknown. Its finite recorded action durations
cannot prove the entire60s bound from an unretained true start.

No failure cause, exception class, sender or source bug follows from the generic
tag/page. Broken pipe, browser cancellation and any particular defect remain
unproven. The original sender remains UNKNOWN. All earlier provider, parser,
capacity, Authorization refusal, lost-output and cleanup-timing failures retain
their original dates and limitations. This plan grants no journey/runtime pass,
cause correction, slot or change to D01/D05 or any closed row.

## Complete source read and protected inputs

Published immutable pin:
`94054b3c9b674e445893b52c1d7de29703fca73c`.
All695 lines of `scripts/d01-confidential-browser-demo.py` were read in three
bounded chunks, including DemoServer.handle_error, Handler.handle_one_request,
send_error/get/reply and the complete main failure/evidence/cleanup handling.
Source is32723 bytes, SHA-256
`7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0`,
matching accepted implementation
`470690cad0cd93c9f25c5bc40b982e1b91679b49`.
The helper is absent from this own WT; it was not imported, aligned or created.

Fixed verifier read from immutable
`9cefe7a56425bb73c17753e8766d92320b77da3b`, blob
`3be747d03146f1bcaa3ec012ee8d173b61fa737d`,22540 bytes/SHA-256
`f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d`.
Relevant bodies read: require/equal/JSON/base64/request (1–165),
discovery/callback_fields/verify_id_token (273–364). The fixed issuer/endpoints,
state/issuer callback binding, RS256/JWKS certificate-independent verification,
audience/nonce/time/access hash and private native-verifier temporaries remain
as defined. Verifier/helper/module import, provider/version and protocol
execution did not occur.

Historical root review source was read at lines54–105; selected loader,
constructor and failure-sink source of the archived corrected78-case harness
was read at report2440–2560 and2800–2898. Its existing source is preserved.
The root receipt identifies full prior payload91163B/SHA
`ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f`
and prior741B result SHA
`81c7a85c6b278bb60412be566a0bc07732fbc50c1ead9aa6030d07c14648dc5f`.
Those are historical controlled78-case evidence, not a run of this proposal.
The failed61-of-76 verification and its unretained individual values remain
failed; the later corrected78-case pass does not imply browser success.

Full existing user report at94054b3 is305159B/SHA-256
`27c898ec2d0da620a56474ce05d52c2090ef9df765e9c051ee101f99ebddd754`;
root report SHA-256
`41fa78e099d6b3a68ffb8caa7a8006841e2441037ac510f51d74f479402ac50a`.
These whole-report identities protect history; this support phase does not
claim a full reread of every prior embedded controller or historical receipt.

## One proposed helper-only diagnostic reservation

Proposed future source ownership is ONLY the exact additive hunk set below in
`scripts/d01-confidential-browser-demo.py`, plus a separate append to this new
plan after root approval. No verifier, product, existing report, controller,
guide, harness, CLI, workflow, config, credential or transport changes.

One observer function, one Demo-owned private holder, one shared record field
and three guarded observation calls are sufficient. The calls precede unchanged
failure handling at the existing Handler except Exception, HTTPServer
handle_error and main except Exception sites. No new catch alters the existing
request/flow body. The observer does not send replies, set failure/done/stage,
open files, print, invoke a budget, load code or perform provider/network work.

Private field: unexpected_failure_observation, exactly
{observed: bool, diagnostic: null or {site, exception_class, own_function,
own_line}}. First observation latches observed:true before inspecting an error;
a later catch cannot overwrite it. A projection/storage failure can retain
observed:true/diagnostic:null. Failure before the latch or invocation failure
leaves the existing null/unobserved holder and the original outcome intact;
no result asserts a successfully retained observation in that case.

Site is one of handler/server/main. Exception labels are eight exact built-in
classes plus other, tested by type identity; subclasses and arbitrary class
names are not copied. No str/repr/error text, cause/context, arguments, locals,
paths, filenames, URL/header/query/cookie/Origin/sender, protocol or secret value
is read into output. No chain of traceback frames is serialized.

At most64 traceback links are scanned. Only code-object identity matches against
ten explicit own helper functions can contribute the last trusted function
label and an integer source line1..1024. Untrusted frames never provide a name
or path. If64 links do not exhaust the trace, the frame fields are null; an
out-of-range line also cannot establish a frame. This gives a boundary witness,
not an explanation of external library internals or the initiating sender.

The exact bounded holder is at most160 ASCII JSON bytes for its fixed schema/
enums under the existing default encoder (static literal-size calculation).
It uses the existing exclusive0600 evidence FD and existing finite serialization
path. No sidecar or extra file/write is added. The stdout ready/result objects,
HTML content and failure tags have no observer field and remain byte-preserved.

Coverage witnesses, old published lines → proposed final lines:

| Boundary | Exact witness | Meaning of retained own frame |
| --- | --- | --- |
| Parser | handle_one_request422 →474 | The helper called the HTTP parser; external parser frames remain unreported. |
| Dispatch | handle_one_request451 →503; get492–517 unchanged body | Actual dispatch/get/flow ownership can distinguish a route-body exception from the parser call boundary. |
| Reply | reply528/543 →584/599 | A native Python write/send failure can identify the own reply boundary only if it is actually observed. |
| Parser error reply | send_error404–407, own code allowlisted | Unexpected send_error/reply failures remain covered by existing Handler or server catch, without changing send_error outcomes. |
| HTTPServer | process_request374 →422; handle_error379 unchanged assignment | Setup/finish/shutdown exceptions retain the server catch site and only a matching own boundary. |
| Outer main | except Exception621 →679 | Escaped unexpected setup/request failures retain the original main handling; known Failure/Halt paths are unchanged. |

Observer calls themselves are guarded, including function lookup and argument
evaluation, with no-op BaseException handlers. Projection/storage errors do not
replace the original failure or stop its reply/cleanup path. Known Failure/Halt,
PreflowAuthorizationRefusal, response timeouts, terminaldone, original failure
assignments and bare propagation remain as before. This is an observation seam,
not a cause fix.

The unchanged78-case loader deliberately selects only require and certain
classes/methods; it omits a new observer function. It also hoists all top-level
assignments before those classes. The final proposal therefore introduces NO
new top-level code-object assignment, and guards the optional observer lookup.
This source constraint was found by reading the loader, not by running it.
The old harness remains archived byte-exact; its counters, sinks, guards and
short-circuit expectations can remain intact. New diagnostic validation would
need a separately reviewed observer-aware memory envelope; no claim is made
that the old harness exercises the observer or that any new suite has passed.

## Exact prospective diff — NOT MATERIALIZED

Final candidate35711 bytes /757 lines, SHA-256
`c0c022ecd91d92c19058b31d413864c980fe6b0ddfbd1303a45be629afd900e9`.
Observer2263 bytes, SHA-256
`4825addc25de47496cc6532bd417e3029256c5c130ead4e06d8cd65d85e53487`.
The complete following zero-context diff has SHA-256
`68c4c00de5e5945b6205ee7e377f28af9645bfc4523fb098acb7333e354ce891`.
Its equivalent context3 serialization SHA-256 is
`e11e6055f03acfbcc13f9d669976d28779fd9ea2bd084f32d7aab6b95d180f47`.
Hashes include exact UTF-8 text and final newline. Earlier in-memory sketches
are superseded; only this final diff is proposed for root review.

```diff
--- a/scripts/d01-confidential-browser-demo.py
+++ b/scripts/d01-confidential-browser-demo.py
@@ -223,0 +224,47 @@
+def observe_unexpected_failure(holder, site):
+    """First-only finite private observation; never replace the original outcome."""
+    try:
+        if holder["observed"] or site not in ("handler", "server", "main"):
+            return
+        holder["observed"] = True
+        error = sys.exc_info()[1]
+        label = "other"
+        for kind, name in ((AttributeError, "AttributeError"),
+                           (TypeError, "TypeError"), (ValueError, "ValueError"),
+                           (KeyError, "KeyError"), (OSError, "OSError"),
+                           (BrokenPipeError, "BrokenPipeError"),
+                           (ConnectionResetError, "ConnectionResetError"),
+                           (TimeoutError, "TimeoutError")):
+            if type(error) is kind:
+                label = name
+                break
+        own_function = own_line = None
+        codes = (
+            (HeaderReader.readline.__code__, "HeaderReader.readline"),
+            (DemoServer.process_request.__code__, "DemoServer.process_request"),
+            (Demo.begin.__code__, "Demo.begin"),
+            (Demo.callback.__code__, "Demo.callback"),
+            (Demo.invoke.__code__, "Demo.invoke"),
+            (Handler.handle_one_request.__code__, "Handler.handle_one_request"),
+            (Handler.send_error.__code__, "Handler.send_error"),
+            (Handler.get.__code__, "Handler.get"),
+            (Handler.reply.__code__, "Handler.reply"),
+            (main.__code__, "main"),
+        )
+        trace = error.__traceback__ if isinstance(error, BaseException) else None
+        for _ in range(64):
+            if trace is None:
+                break
+            for code, name in codes:
+                if trace.tb_frame.f_code is code and type(trace.tb_lineno) is int and 1 <= trace.tb_lineno <= 1024:
+                    own_function, own_line = name, trace.tb_lineno
+            trace = trace.tb_next
+        if trace is not None:
+            own_function = own_line = None
+        holder["diagnostic"] = {"site": site, "exception_class": label,
+                                "own_function": own_function, "own_line": own_line}
+    except BaseException:
+        # Projection/storage failures retain the first latch and original handler.
+        pass
+
+
@@ -235,0 +283 @@
+        self.unexpected_failure_observation = {"observed": False, "diagnostic": None}
@@ -379,0 +428,4 @@
+        try:
+            observe_unexpected_failure(self.demo.unexpected_failure_observation, "server")
+        except BaseException:
+            pass
@@ -471,0 +524,4 @@
+            try:
+                observe_unexpected_failure(demo.unexpected_failure_observation, "handler")
+            except BaseException:
+                pass
@@ -559,0 +616 @@
+              "unexpected_failure_observation": {"observed": False, "diagnostic": None},
@@ -599,0 +657 @@
+        record["unexpected_failure_observation"] = demo.unexpected_failure_observation
@@ -621,0 +680,4 @@
+        try:
+            observe_unexpected_failure(record["unexpected_failure_observation"], "main")
+        except BaseException:
+            pass
```

## Actual static preservation proofs

The candidate was constructed in memory against pinned94054b3, never written
as a helper. All seven diff hunks contain insertions only:62 added lines.
Removing final spans224–270,283,428–431,524–527,616,657 and680–683 reconstructs
the entire32723-byte baseline, including all695 original lines and final
newline. Reversing the exact construction replacements agrees byte-for-byte.

AST parse and in-memory code-object compile passed for old/final strings only;
neither code object was exec'd and no candidate/helper module was imported.
A transformer removes only the new observer definition, three exact guarded
observer calls, new holder assignment/shared-record assignment and new record
dict field. Whole normalized AST then equals the entire baseline AST with
location attributes excluded. All35 original functions, exception headers,
short-circuit conditions, cleanup/finally branches and call order remain.
All original top-level assignments are AST-identical; imports are unchanged.

Additional AST checks verify the observer has no import/raise/context manager,
exec/eval/compile/str/repr/print/open call, no path/frame-locals/args/cause/context
access, one range(64) scan, three exact guarded call sites and ten fixed own code
labels. All file I/O, native/verifier calls and private evidence writer are
baseline nodes, not new observer operations. Static enum/schema-size checking
uses literal data only; it does not evaluate the observer.

Protected first-three/fourth-terminal Authorization short circuit, all Host/
header/body/method/target/cookie bounds, postflow400 refusal, no secret reads
for refused requests, original status/check bookkeeping, discovery signature
and printed issuer remain in the exact old AST. Source preserve proofs apply to
the proposed addition; they do not prove its future execution or availability
of any native/browser resource.

Static-method errors retained: one shell search failed due to an unquoted zsh
Git-path glob; quoting the immutable pathspec corrected the read. A broad
historical search was truncated; relevant loader/history spans were reread
in bounded chunks. First in-memory reversal code attempted to join nested
lists and raised TypeError; flattened joining corrected the proof. A separate
JSON embedding check initially failed because its Python string consumed JSON
escapes; a raw JSON literal corrected the static checker input. These were
inspection/construction failures, not helper/runtime failures. An overbroad
AST guard also initially confused the allowed Handler.get function attribute
with a dictionary.get call; restricting it to actual Call nodes passed. Final
frame trust uses explicit code-object is comparisons, never equality or names.
The initial context3 fence reconstruction passed its source/hash checks but
the subsequent report whitespace assertion found diff-prefixed blank context
lines. The final fence uses the exact zero-context serialization to preserve
all insertions without trailing report whitespace; candidate bytes are unchanged.
Source reads also identified the old-loader lookup/hoisting constraints; the final unexecuted
proposal keeps those old source nodes intact rather than altering the harness.

## Exact finite validation design — separate future release

No validation harness or memory candidate is executed now. Root would first
review the final helper diff, then a complete finite observer-aware memory
envelope and its source pins before releasing ONE controlled check. The
existing78-case harness source must remain archived unchanged, with old
normalization/guards and first-failure behavior preserved. Its source-only
selector omission is covered by guarded lookup; it is not an observer pass.

The proposed controlled check uses AST-selected own definitions in memory,
stub parser/read/reply/dispatch/budget sinks and literal private sentinels only.
No module main, HTTPServer constructor, socket/listener, real budget/alarm
installation from the helper, native/provider/CLI/Driver, file creation or
network operation. Any outer harness alarm/process wrapper is a separate root
reservation, with proposed30s whole bound, no retry, capped private result and
complete fixed counts/exit retained before interpretation.

The following is a finite case design, not actual results:

| Case/group | Fixed injection and required outcome |
| --- | --- |
| Closed class labels | Eight exact built-in exception instances plus one custom sentinel subclass; exact eight labels/other, no dynamic class name or error text. |
| Parser boundary | Inject sentinel ValueError from controlled parser called by original handle_one_request; handler site/own caller line474, unchanged unexpected_failure/done/400 behavior. |
| Dispatch boundary | Original get path reaches a stub equality operation raising TypeError; retain Handler.get frame and original failure/reply sequence; no begin/callback/credential side effect. |
| Reply boundary | Original reply with one-shot OSError write sink failure; last own reply frame, original recovery reply or propagated error class unchanged compared with baseline. |
| send_error boundary | Controlled parser rejection enters original send_error and its reply write fails; observe only actual unexpected error, preserve original request-invalid/override/reply behavior exactly. |
| Server boundary | Original process_request/finish stub raises custom private-sentinel error; shutdown/finally and active reset unchanged, server catch owns fixed failure/done. |
| Main boundary | AST-selected original main unexpected-except block with stub record/demo only, without calling main; site main and old tag/stage assignment match; no setup/provider/write invoked. |
| First-only | Capture A, then force a second unexpected reply/server error B; exact first holder remains A while original later failure handling still runs. |
| Private sentinel | Custom exception has str/repr traps and sentinel-bearing message/attributes; no trap invoked and no sentinel/string/class/path/locals appears in projected JSON or captured public result. |
| Untrusted/bounded trace | Synthetic finite trace samples: no trusted code, line1, line1024, line0, line1025,64 links and65 links; exact own labels/bounds or null;65-link frame fields null. No full trace serialization. |
| Projection read failure | Controlled traceback accessor raises a private-sentinel exception after first latch; observed:true/diagnostic:null, original request/reply outcome survives; later observation cannot overwrite. |
| Projection write failure | Bounded dict test sink raises on diagnostic assignment after observed latch; same null/latch and original outcome; no formatting of the write exception. |
| Optional observer failure | Selected legacy namespace omits observer, and separate injected observer/argument lookup failures include a BaseException; guarded call cannot replace original server/handler/main tags, replies or propagation. |
| Existing refusal/success paths | Preserve the complete78-case acceptance predicates, count1..3 live403/count4 terminal, postflow400, request bounds, secret short circuit and write/timeout propagation. Successful/known Failure/Halt paths create no observation. |
| Existing evidence write failure | AST-selected baseline writer with finite in-memory write/FD sinks fails; old evidence_write_failed handling/first failure/public result survives, no diagnostic appears on stdout. Do not call helper main or open an actual FD. |

Class/trace boundary groups have the explicitly enumerated finite samples above;
future reviewed harness must advertise their exact implemented counts and stop
at its first failed assertion, with all unattempted cases stated. No larger
tenant/profile/transport campaign, browser retry, sender probe, new refusal
allowance or automatic cause correction is proposed. These cases can prove
projection/outcome preservation only, not OAuth/native/browser acceptance.

## Preserved actual controller and handoff

Prepared180s controller/source, inclusive START, START+840 active stop and
START+900 total deadline remain unchanged. Existing helper's180s pending time,
600s argument ceiling, disk thresholds, cleanup order and all original source
nodes remain unchanged. The helper diagnostic does not reset a budget or fix
the true-cleanup-start evidence gap. Actual cleanup still takes priority; no
new resource measurement/claim or desktop ownership exists in this support phase.

The only next proposed source reservation is the seven exact helper insertions
above, after root full review. Source materialization, finite memory execution
and one real fixture each require separate root decisions. Source performer
Sol4's actual-report ownership is not duplicated or contacted. No other worker/
task/WT/shell, source edit, import, native/library/provider/version/protocol,
HTTP/browser/Driver action, download/query/dispatch/Cargo/build/service, merge,
main/push/status or cache deletion occurred. RiWork Cua.ai Driver preference
persists for any separately authorized desktop work; none was used here.

Repository docs/hygiene/whitespace, report-only scope and clean commit results
are recorded after authoring below. This source plan supplies no runtime pass
or original-row closure.

## Actual report/static receipt

The final report fence independently reconstructs all35711 candidate bytes,
SHA-256 c0c022ecd91d92c19058b31d413864c980fe6b0ddfbd1303a45be629afd900e9,
from accepted94054b3/470 baseline using the exact seven zero-context hunks.
The final zero-context reader initially assumed explicit counts even for a
one-line insertion header and rejected that valid header; accepting the
standard omitted-one count corrected this inspection-only parser. Final
reconstruction, hash, AST parse/in-memory compile, whole byte/normalized-AST
reversal, finite schema/guard checks and both regular0600 receipt re-hashes
passed. No code object, observer, harness, helper or module main was executed.

Actual repository checks: docs checker EXIT0; staged repository hygiene EXIT0
(966 tracked files); Git whitespace EXIT0. Sole staged scope is this new plan,
no other tracked edits or untracked files, and the helper remains absent in
this WT. No pre-existing docs/build-layout error remained in these checks.
Report source is ready for immutable root review; proposed helper ownership,
source materialization, memory execution and real fixture remain separate
HELD decisions. Failure cause/sender and true cleanup-start bound remain unknown.

## Approved observer materialization — actual source-only receipt

Date: 2026-10-02. Reservation:
`wave30_D01_unexpected_failure_observer_materialization`.
Root fully reviewed the preceding immutable1e50d3e plan and exact seven-hunk
candidate. This phase materializes that approved source only. No observer,
helper, old78-case/new memory harness, module main, native/provider/CLI/HTTP/
socket/listener/Driver/browser/Cargo or runtime operation was performed.
No runtime slot was taken or released; no runtime/cause result is inferred.

### Three separate commit roles

1. BASELINE-ONLY preparation commit
   `bf2aff3f00d3adb02b7b900ac5aeb4279dd8579e` imports only the formerly absent
   `scripts/d01-confidential-browser-demo.py` from published
   `69f46cf75390cde70a992eb528c7ee5f769aeeca`.
   Its entire32723 bytes/695 lines match7fbc SHA-256 exactly; Git mode100644,
   baseline blob `3d3379399126423a6a5588a6bd55ea4ac1723ebd`.
   This matches both fixed94054b3 and accepted470 full helper bytes too.
   Root must NOT integrate this preparation commit. No other published path
   or tree was imported, and branch/history were not aligned.
2. Source-only commit
   `3c9c63bb217bfc20614e62ef7116e6e2abad0170` contains precisely the seven
   reviewed insertions, one file/62 added lines, no deletions or other delta.
   Helper35711 bytes/757 lines, mode100644, blob
   `d980f87959b6a14126c52c07a24bf859ccd26766`, SHA-256
   `c0c022ecd91d92c19058b31d413864c980fe6b0ddfbd1303a45be629afd900e9`.
   Its parent is the local preparation commit, but its additive source delta
   is the root integration unit against the already-published baseline.
3. A separate report-only evidence commit appends this receipt. Its immutable
   hash is returned in the handoff after committing. Integrate only the
   source/evidence deltas if root approves, preserving other accepted docs.

The existing plan is retained as a complete24372-byte/379-line prefix,
SHA-256 `85409506c980b1cd41eae70d300fdbdc5d95e51de12c6753b21b46b1bb4c7988`.
Its earlier helper-absent/source-proposed statements are dated design evidence;
this appendix records actual materialization, not a retroactive runtime pass.
Every old failure, unknown cause/sender, source-phase inspection error and
cleanup first-event limitation remains intact.

### Actual preservation and finite source checks

The implementation was reconstructed only from the exact reviewed immutable
zero-context fence, SHA-256
`68c4c00de5e5945b6205ee7e377f28af9645bfc4523fb098acb7333e354ce891`.
Actual final spans224–270 (47 lines),283 (1),428–431 (4),524–527 (4),616 (1),
657 (1),680–683 (4) match the approved seven additions and total62 lines.
Removing those spans reconstructs the entire local/published32723-byte baseline,
including all695 original lines, every old function/branch and final newline.
No replaced/deleted baseline line or mode change exists.

AST parse/in-memory code-object compile passed for the actual baseline and
materialized candidate, without executing/importing either. Normalization
removed exactly one new observer definition, three exact guarded observation
calls, two holder/shared-record assignments and one record dict field.
The resulting full AST equals the complete old AST with locations excluded,
including all35 original functions and every original short-circuit predicate,
exception header, assignment/call order and finally/cleanup branch.

Thus firstfail/failure tags, terminaldone, replies/send_error, request parser,
strict Host/body/cookie/target guards, first-three403/fourth-terminal refusal,
postflow400 Authorization refusal, credential short circuit, crypto/verifier
pins, printed issuer and all original budgets/deadlines remain exact old nodes.
This is byte/source preservation proof, not execution of those predicates.
Only the explicitly approved private observation holder/calls are additive.

Actual observer AST checks confirm three guarded call sites, no observer import/
raise/I/O/print/str/repr/exec/eval/compile, no error arguments/cause/context,
locals/path/filename read into output, one range64 scan and ten closed own
function labels. Frame matching uses explicit code-object is; maximum640 fixed
comparisons. Literal enum/schema calculation confirms at most160 ASCII JSON
bytes in the holder, within the reviewed256-byte planning bound. No projection
truth table or candidate function was executed. First-only latch/projection-
write-failure behavior remains to be checked by a separately reviewed finite
memory envelope; source checking does not award it an actual behavioral pass.

Actual source-stage repository checks all EXIT0: docs link/build-layout checker,
tracked hygiene (967 files), Git staged whitespace and sole-helper staged scope.
Source commit's exact file/hash/mode/parent were checked; it was clean before
this report appendix. No current-phase static command failed. No pre-existing
docs/build-layout error remained. Final report-stage docs/hygiene/whitespace
checks also EXIT0 (967 files); full-prefix, sole-report append, unchanged
committed source and no-unstaged/untracked scope checks passed.

### Remaining authority and evidence boundaries

No validation harness was implemented or executed. A future observer-aware
finite envelope still needs its complete exact design, immutable source review
and separate runtime release; the unchanged archived78-case source is not
redefined or newly run. Prepared180 controller/source, original inclusive
START/840 active/900 total and all cleanup thresholds/ordering remain unchanged.
This source materialization does not cure an unknown request, recover an
exception class/trace, identify a sender or prove true historic cleanup start.
Cause, sender and that original whole-cleanup bound remain UNKNOWN.

The other worker's cleanup design/report lane is not contacted or duplicated.
No product/verifier/controller/guide/D05/workflow/other file was edited; no
worker/task/worktree/managed shell, merge/main/push/status or desktop action.
Original gates, root publication/integration authority and closed rows remain
unchanged. RiWork Cua.ai Driver preference persists; no desktop was used.

## Observer-aware finite memory envelope — DESIGN ONLY

Date: 2026-10-02. Reservation:
`wave30_D01_observer_aware_memory_envelope_design`.
Only this appendix is written. Full prior480-line/30530-byte report is preserved,
SHA-256 `fe08d3aad7122e090dce67796cfb87fd2ed7cc7f9cecc561e13d55195308a018`.
Helper remains the exact35711-byte c0c022 source at
`3c9c63bb217bfc20614e62ef7116e6e2abad0170`, mode100644. No alignment/import,
helper/source/controller/verifier/guide/workflow/harness file edit occurred.

This is complete prospective source, not an invocation or a result. No helper,
observer, old/new harness, selected definition, module main, server constructor,
native/provider/CLI/HTTP/network/socket/listener/signal/Driver/browser/Cargo
operation was executed. Static tools alone read Git/source and performed AST
parse, in-memory code-object compile and byte/hash checks. No runtime slot was
acquired or released. Root must review this entire envelope and separately
release ONE bounded memory invocation before execution.

### Fixed source roles and exact assembly

| Input role | Immutable source | Whole-byte identity |
| --- | --- | --- |
| Original legacy baseline | `7461ab50f5a5fdf0aa5d7bd6c4b309575e7a4238:scripts/d01-confidential-browser-demo.py` |31162B/671 lines; SHA ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd|
| Accepted unchanged helper | `470690cad0cd93c9f25c5bc40b982e1b91679b49:scripts/d01-confidential-browser-demo.py` |32723B/695 lines; SHA7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0|
| Current observer helper | `3c9c63bb217bfc20614e62ef7116e6e2abad0170:scripts/d01-confidential-browser-demo.py` |35711B/757 lines; SHAc0c022ecd91d92c19058b31d413864c980fe6b0ddfbd1303a45be629afd900e9|
| Legacy readable logic |Complete461-line archive below, normalized exactly once|24210B/SHA98b19b4d422067b0d49ea7eb1035a719d88bda672dc36579c6cf5c47cb0924eb|
| New readable child logic |Complete576-line archive below, literal bytes|28019B/SHA1fa1c0eeb2f9b0ad61979bd838c45f955d56f8caab961d2f07c826870b3188dd|

The old ce79 payload remains EXACT91163 bytes/463 physical lines, SHA
`ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f`.
It is still the two serialized source lines BASELINE_TEXT then SOURCE_TEXT
(legacy baseline then accepted7fbc), followed by unchanged legacy logic.
The actual old741-byte result SHA
`81c7a85c6b278bb60412be566a0bc07732fbc50c1ead9aa6030d07c14648dc5f`
and its78/44/80 counts remain historical evidence, never rewritten or credited
to this new envelope. The failed61-of-76 attempt and every real-fixture failure
also retain their original outcomes/unknowns.

New child payload assembly is exact, in this order, with no separator other
than the shown real LF at the end of each assignment:

```text
LEGACY_BASELINE_TEXT=<json.dumps(whole legacy baseline text, ensure_ascii=True)>\n
UNCHANGED_TEXT=<json.dumps(whole accepted7fbc text, ensure_ascii=True)>\n
SOURCE_TEXT=<json.dumps(whole c0c022 text, ensure_ascii=True)>\n
LEGACY_LOGIC=<json.dumps(whole normalized legacy logic below, ensure_ascii=True)>\n
<entire literal new child-logic fence below including its final LF>
```

Angle-bracket expressions are assembly instructions, not literal bytes.
Use Python json.dumps default separators, no indentation/sort transformation;
retain all whole-source final newlines. Four assignment lines total130136B.
Complete new child payload is **158155 bytes/580 physical lines**, SHA-256
**`b7bdeb6f9e24b8c537cfdca7506f570a6e2e180efa2f560dad2c6c1693aa0822`**.
This exact full payload was statically reconstructed/hashed/parsed/compiled
BEFORE any execution. Child/helper source roles are explicit; an updated
SOURCE_SHA supplied to the legacy definitions is not an edit to their archive.

Complete controller-source archive below is15438B/341 lines, SHA-256
`ba20f87f0f4fe11c593ac3e5b433bde4e215e2703e0e11be3633facc78dcf0cc`.
It has a frozen50+78-name allowlist and exactly one possible Popen. Its API is
bounded_memory_envelope(payload_bytes, root_selected_fresh_private_directory).
The payload bytes must match the full SHA above; root must separately reserve
the fresh directory, exact invocation/interpreter/environment, resource
preflight and output retention. No actual directory or command is launched
or chosen by this design.

### Planned cases — all unexecuted

| Layer/group | Planned |
| --- | ---: |
| Original legacy guard/sink cases |78|
| Exact builtin classes, two subclasses, private other |11|
| Code identity, lines1/1024/0/1025,64/65 links, untrusted |8|
| Trace/frame/holder/latch/storage failures, invalid site, no active error |7|
| First observation and secondary reply/server failure |2|
| Parser/dispatch/reply/send_error/server/main-except sites |6|
| Known Failure/Halt/success without observation |3|
| Missing observer/field/BaseException guards and old selector |10|
| str/repr traps, private writer/public stdout success/failure |3|
| Observer subtotal |50|
| TOTAL |128|

The old run(), verify(), guard/sink factories and every old assertion are loaded
as their exact archived definition AST nodes. Original import/timer/print/exit
tail is NOT executed. The new envelope explicitly supplies current c0c022 as
SOURCE_TEXT/SOURCE_SHA and the same original ac3 baseline; its old78 checks
keep identical guard/outcome assertions and expected44/80 calls, injected
write failure1 and response timeout1. That legacy selector still omits the
observer and does not validate diagnostics. This is a new prospective reuse,
not a retroactive change to the historical ce79 run/results.

The separate observer-aware selector adds observe_unexpected_failure and full
Demo/DemoServer/main definitions, with require/Failure/Halt/Preflow/HeaderReader/
Handler and original constants. All ten own trusted codeobjects exist.
Original main is defined and NEVER called; DemoServer is defined and NEVER
constructed. HTTPServer is never constructed. Real setup/begin/callback/native
work is never invoked. Old DemoSink overrides flow work; new parser/read/reply/
budget/writer sinks use only deterministic memory fixtures. Server methods
are invoked unbound with an owned SimpleNamespace; they create no server.

Actual handler outcomes are compared using the unchanged old normalized()
contract, including all original flow/check/status/reply/header/body/budget/
call values. Main-except comparisons omit only the one explicitly new holder.
The main case executes only the copied generic-except body; its codeobject is
not the original main codeobject, so own_function:null is required there.
The private writer/print/return tail is copied exactly into a fresh selected
wrapper, not moved into a module-level return and not invoked through main.

Fake traceback objects use only controlled selected codeobject identity and
bounded integers. The equal-but-distinct clone must not match. No traceback
text, filename, URL/header/query/cookie/Origin/sender, locals, exception args,
raw message/class name, secret or real protocol input is serialized. The
sentinel is a source-defined fake string; custom str/repr traps must remain0.
Accessor/storage failures cannot replace original outcomes. Failed latch/read
writes may leave observed:false: the test does not falsely claim an original
exception was retained in that case. A latched diagnostic-write failure stays
observed:true/null and blocks a later overwrite.

### Complete unchanged legacy logic archive

Marker: D01_OBSERVER_LEGACY_78_READABLE_ARCHIVE_V1.
The one handle_error indexing call has the same publication space used in the
existing root report. Remove ONLY the space in `new["handle_error"] (` when
reconstructing this old archive. That restores24210B/SHA98b19b4 exactly.
No other display/source normalization applies. The full old timer/output tail
is retained below for archive identity but excluded from new selected execution.

```python
import ast, contextlib, copy, hashlib, http.server, io, json, re, signal, time, urllib.parse
from types import SimpleNamespace

started=time.monotonic()
SOURCE_SHA="7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0"
BASE_COMMIT="7461ab50f5a5fdf0aa5d7bd6c4b309575e7a4238"
groups={}
completed=0
case="source_pin"
handler_calls={"baseline":0,"candidate":0}
expected_write_failures=0
expected_timeouts=0

class VerificationDeadline(BaseException):
    pass

def deadline(signum,frame):
    raise VerificationDeadline

def verify(name,group,body):
    global case,completed
    case=name
    body()
    completed+=1
    groups[group]=groups.get(group,0)+1

class BudgetSink:
    def __init__(self):
        self.events=[]
        self.pending_deadline=None
        self.fail_response=False
    @contextlib.contextmanager
    def limit(self,seconds,tag):
        self.events.append((seconds,tag,"enter"))
        if tag=="response_timeout" and self.fail_response:
            raise self.halt("response_timeout")
        try:
            yield
        finally:
            self.events.append((seconds,tag,"exit"))

class WriteFailure(Exception):
    pass

class WriteSink(io.BytesIO):
    def __init__(self):
        super().__init__()
        self.fail=False
    def write(self,value):
        if self.fail:
            raise WriteFailure
        return super().write(value)

def load_selected(text,version):
    tree=ast.parse(text)
    constants=[node for node in tree.body if isinstance(node,ast.Assign)]
    assert all(not any(isinstance(item,ast.Call) for item in ast.walk(node)) for node in constants)
    selected=constants[:]
    for node in tree.body:
        if isinstance(node,ast.ClassDef) and node.name in {
                "Failure","Halt","HeaderReader","Handler","PreflowAuthorizationRefusal"}:
            selected.append(node)
        elif isinstance(node,ast.FunctionDef) and node.name=="require":
            selected.append(node)
        elif isinstance(node,ast.ClassDef) and node.name=="Demo":
            node=copy.deepcopy(node)
            node.body=[method for method in node.body if isinstance(method,ast.FunctionDef)
                       and method.name in {"__init__","clear"}]
            selected.append(node)
    ns={"http":http,"re":re,"urllib":urllib}
    exec(compile(ast.fix_missing_locations(ast.Module(body=selected,type_ignores=[])),
                 "<selected-request-methods-"+version+">","exec"),ns)
    main=next(node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name=="main")
    ns["record_node"]=copy.deepcopy(next(node for node in main.body if isinstance(node,ast.Assign)
        and any(isinstance(t,ast.Name) and t.id=="record" for t in node.targets)))
    main_try=next(node for node in main.body if isinstance(node,ast.Try))
    demo_final=next(node for node in main_try.finalbody if isinstance(node,ast.If)
        and isinstance(node.test,ast.Compare) and isinstance(node.test.left,ast.Name)
        and node.test.left.id=="demo")
    ns["copy_nodes"]=[copy.deepcopy(node) for node in demo_final.body
                     if isinstance(node,ast.Assign) and isinstance(node.targets[0],ast.Subscript)
                     and isinstance(node.targets[0].value,ast.Name)
                     and node.targets[0].value.id=="record"]
    loop_index=next(i for i,node in enumerate(main_try.body) if isinstance(node,ast.While))
    ns["gate_nodes"]=copy.deepcopy(main_try.body[loop_index+1:loop_index+4])
    server=next(node for node in tree.body if isinstance(node,ast.ClassDef) and node.name=="DemoServer")
    error=copy.deepcopy(next(node for node in server.body if isinstance(node,ast.FunctionDef)
                            and node.name=="handle_error"))
    exec(compile(ast.Module(body=[error],type_ignores=[]),"<server-error-method>","exec"),ns)
    return ns

def make_demo(ns):
    names=("credential_private_validated","provider_identity_verified","discovery_verified",
        "protected_without_cookie_denied","authorization_redirect_issued",
        "state_issuer_flow_cookie_verified","confidential_s256_exchange_verified",
        "rs256_jwks_issuer_audience_nonce_time_access_hash_verified",
        "userinfo_subject_verified","protected_with_fresh_cookie_accepted")
    class DemoSink(ns["Demo"]):
        def __getattribute__(self,name):
            if name=="secret":
                access=object.__getattribute__(self,"calls")
                access["credential"]+=1
            return object.__getattribute__(self,name)
        def begin(self):
            self.calls["begin"]+=1
            ns["require"](not self.attempted,"flow_already_started")
            self.stage="flow"
            self.attempted=True
            self.pending={}
            return ns["ISSUER"]+"/authorize","a"*43
        def callback(self,query,cookie):
            self.calls["callback"]+=1
            ns["require"](self.pending is not None,"callback_already_consumed")
            self.pending=None
            self.cookie="b"*43
            self.subject="synthetic"
            return self.cookie
    budget=BudgetSink()
    budget.halt=ns["Halt"]
    demo=DemoSink(SimpleNamespace(equal=lambda left,right:left==right),None,object(),budget,
                  {name:i<3 for i,name in enumerate(names)},
                  {name:None for name in ("authorization_redirect","callback","token_exchange",
                   "userinfo","protected_before","protected_after")})
    demo.calls={"get":0,"cookies":0,"begin":0,"callback":0,"credential":0}
    return demo

def flow_snapshot(demo):
    return (demo.attempted,demo.pending,demo.cookie,demo.subject,
            demo.failure,demo.done,copy.deepcopy(demo.checks),copy.deepcopy(demo.statuses))

def build_wire(ns,method="GET",target="/",headers=(),host="normal"):
    prefix=[] if host=="absent" else [("Host",ns["AUTHORITY"] if host=="normal" else "invalid")]
    return ((method+" "+target+" HTTP/1.1\r\n"
            +"".join(name+": "+value+"\r\n" for name,value in prefix+list(headers))
            +"\r\n").encode("ascii"))

def request(ns,demo,version,wire,*,write_fail=False,timeout=False,parser_path=None):
    handler_calls[version]+=1
    class HandlerSink(ns["Handler"]):
        def parse_request(self):
            parsed=super().parse_request()
            if parsed and parser_path is not None:
                self.path=parser_path
            return parsed
        def get(self,target):
            demo.calls["get"]+=1
            return super().get(target)
        def cookies(self):
            demo.calls["cookies"]+=1
            return super().cookies()
        def reply(self,status,text,**options):
            self.responses.append((status,tuple(sorted(options))))
            return super().reply(status,text,**options)
        def send_response(self,status,message=None):
            self.sent_statuses.append(status)
        def send_header(self,name,value):
            self.sent_headers.append((name,value))
        def end_headers(self):
            self.header_ends+=1
    handler=object.__new__(HandlerSink)
    handler.server=SimpleNamespace(demo=demo)
    handler.rfile=io.BytesIO(wire)
    handler.wfile=WriteSink()
    handler.wfile.fail=write_fail
    handler.responses=[]
    handler.sent_statuses=[]
    handler.sent_headers=[]
    handler.header_ends=0
    demo.budget.fail_response=timeout
    handler.handle_one_request()
    return handler

def normalized(ns,demo,handler):
    return (flow_snapshot(demo),demo.stage,demo.request_invalid_reason,demo.calls,
            handler.responses,handler.sent_statuses,handler.sent_headers,handler.wfile.getvalue(),
            demo.budget.events)

def record_copy(ns,demo):
    env=dict(ns)
    env["demo"]=demo
    exec(compile(ast.Module(body=[ns["record_node"]],type_ignores=[]),"<record-initialization>","exec"),env)
    exec(compile(ast.Module(body=ns["copy_nodes"],type_ignores=[]),"<record-final-copy>","exec"),env)
    return env["record"]

def gate_fails(ns,demo):
    env=dict(ns)
    env["demo"]=demo
    env["record"]={"checks":demo.checks,"result":"failed"}
    try:
        exec(compile(ast.Module(body=ns["gate_nodes"],type_ignores=[]),"<existing-outcome-gate>","exec"),env)
    except ns["Failure"] as failure:
        assert failure.tag in {"request_invalid","unexpected_failure"}
    else:
        raise AssertionError
    assert env["record"]["result"]=="failed"

def refused(ns,demo,handler,before,count,terminal):
    assert handler.responses==[(403,())] and handler.sent_statuses==[403]
    assert handler.header_ends==1
    assert all(name not in {"Location","Set-Cookie"} for name,value in handler.sent_headers)
    assert b"<form" not in handler.wfile.getvalue()
    assert demo.calls=={"get":0,"cookies":0,"begin":0,"callback":0,"credential":0}
    after=flow_snapshot(demo)
    assert after[:4]==before[:4] and after[6:]==before[6:]
    assert demo.preflow_authorization_refusals==count
    assert demo.request_invalid_reason=="authorization"
    assert demo.failure==("request_invalid" if terminal else before[4])
    assert demo.done==(True if terminal else before[5])
    assert all(not value for key,value in demo.checks.items() if key not in
               {"credential_private_validated","provider_identity_verified","discovery_verified"})
    assert all(value is None for value in demo.statuses.values())
    copied=record_copy(ns,demo)
    assert copied["preflow_authorization_refusals"]==count
    assert copied["request_invalid_reason"]=="authorization"
    gate_fails(ns,demo)

def run():
    global expected_write_failures,expected_timeouts
    text=SOURCE_TEXT
    assert hashlib.sha256(text.encode()).hexdigest()==SOURCE_SHA
    baseline_text=BASELINE_TEXT
    assert hashlib.sha256(baseline_text.encode()).hexdigest()=="ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd"
    old=load_selected(baseline_text,"baseline")
    new=load_selected(text,"candidate")
    demo=make_demo(new)
    assert record_copy(new,demo)["preflow_authorization_refusals"]==0
    verify("initial_count_copy","record_copy",lambda:gate_fails(new,demo))
    for count in range(1,5):
        def check(count=count):
            before=flow_snapshot(demo)
            handler=request(new,demo,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
            refused(new,demo,handler,before,count,count==4)
        verify("bounded_refusal_"+str(count),"bounded_sequence",check)
    for label,headers,method,target in [
        ("empty",[("Authorization","")],"GET","/"),
        ("duplicate",[("Authorization","ignored"),("Authorization","")],"GET","/"),
        ("post",[("Authorization","ignored")],"POST","/login"),
        ("protected",[("Authorization","ignored"),("Cookie","d01_local_demo="+"b"*43)],"GET","/protected"),
        ("callback",[("Authorization","ignored")],"GET","/callback"),
        ("other_method",[("Authorization","ignored")],"PUT","/")]:
        def check(headers=headers,method=method,target=target):
            d=make_demo(new);before=flow_snapshot(d)
            h=request(new,d,"candidate",build_wire(new,method,target,headers))
            refused(new,d,h,before,1,False)
            d.clear()
            assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
        verify("presence_"+label,"authorization_presence",check)
    for field,value in [("attempted",True),("pending",{}),("cookie","b"*43),
                        ("subject","synthetic"),("attempted",0),("attempted",None)]:
        def check(field=field,value=value):
            d=make_demo(new);setattr(d,field,value);before=flow_snapshot(d)
            h=request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
            after=flow_snapshot(d)
            assert h.responses==[(400,())] and d.failure=="request_invalid" and d.done
            assert d.preflow_authorization_refusals==0 and d.request_invalid_reason=="authorization"
            assert after[:4]==before[:4] and after[6:]==before[6:]
            assert all(v==0 for v in d.calls.values())
        verify("strict_state_"+field+"_"+str(groups.get("postflow",0)+1),"postflow",check)
    def exhausted():
        d=make_demo(new);d.preflow_authorization_refusals=4
        h=request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
        assert h.responses==[(400,())] and d.done and d.failure=="request_invalid"
        assert d.preflow_authorization_refusals==4 and all(v==0 for v in d.calls.values())
    verify("defensive_exhaustion","postflow",exhausted)
    login=[("Origin",new["ORIGIN"]),("Content-Type","application/x-www-form-urlencoded"),("Content-Length","0")]
    specifications=[
        ("root","GET","/",[],None),("protected_before","GET","/protected",[],None),
        ("unavailable","GET","/unavailable",[],None),("root_query","GET","/?x",[],None),
        ("protected_query","GET","/protected?x",[],None),
        ("callback_empty","GET","/callback",[],None),
        ("callback_ready","GET","/callback",[],{"pending":{}}),
        ("login","POST","/login",login,None),
        ("login_repeated","POST","/login",login,{"attempted":True}),
        ("protected_after","GET","/protected",[("Cookie","d01_local_demo="+"b"*43)],
            {"cookie":"b"*43,"subject":"synthetic","protected_before":True}),
        ("protected_before_missing","GET","/protected",[("Cookie","d01_local_demo="+"b"*43)],
            {"cookie":"b"*43,"subject":"synthetic"}),
        ("protected_cookie_mismatch","GET","/protected",[("Cookie","d01_local_demo="+"c"*43)],
            {"cookie":"b"*43,"subject":"synthetic"}),
        ("method","PUT","/",[],None),
        ("post_target","POST","/wrong",login,None),("post_query","POST","/login?x",login,None),
        ("origin_missing","POST","/login",[("Content-Type","application/x-www-form-urlencoded")],None),
        ("origin_wrong","POST","/login",[("Origin","invalid"),("Content-Type","application/x-www-form-urlencoded")],None),
        ("origin_duplicate","POST","/login",login+[("Origin",new["ORIGIN"])],None),
        ("type_missing","POST","/login",[("Origin",new["ORIGIN"])],None),
        ("type_wrong","POST","/login",[("Origin",new["ORIGIN"]),("Content-Type","text/plain")],None),
        ("type_duplicate","POST","/login",login+[("Content-Type","application/x-www-form-urlencoded")],None),
        ("cookie_duplicate","GET","/",[("Cookie","other=x"),("Cookie","other=x")],None),
        ("cookie_shape","GET","/",[("Cookie","d01_demo_flow=x")],None),
        ("cookie_own_duplicate","GET","/",[("Cookie","d01_demo_flow="+"a"*43+"; d01_demo_flow="+"a"*43)],None),
        ("cookie_unrelated","GET","/",[("Cookie","unrelated=x")],None)]
    structural=[
        ("host_absent","/","absent",[]),("host_wrong","/","wrong",[]),
        ("host_duplicate","/","normal",[("Host",new["AUTHORITY"])]),
        ("transfer","/","normal",[("Transfer-Encoding","chunked")]),
        ("expect","/","normal",[("Expect","100-continue")]),
        ("length_nonzero","/","normal",[("Content-Length","1")]),
        ("length_duplicate","/","normal",[("Content-Length","0"),("Content-Length","0")]),
        ("length_format","/","normal",[("Content-Length","00")]),
        ("scheme","http://localhost:3000/","normal",[]),
        ("netloc","//localhost:3000/","normal",[]),("fragment","/#x","normal",[]),
        ("parse_target","http://[","normal",[])]
    for label,method,target,headers,initial in specifications:
        def check(method=method,target=target,headers=headers,initial=initial):
            outcomes=[]
            for ns,version in [(old,"baseline"),(new,"candidate")]:
                d=make_demo(ns)
                for field,value in (initial or {}).items():
                    if field=="protected_before":d.checks["protected_without_cookie_denied"]=value
                    else:setattr(d,field,copy.deepcopy(value))
                h=request(ns,d,version,build_wire(ns,method,target,headers))
                outcomes.append(normalized(ns,d,h))
                if version=="candidate":
                    assert d.preflow_authorization_refusals==0
            assert outcomes[0]==outcomes[1]
        verify("header_free_"+label,"header_free_routes_guards",check)
    for label,target,host,headers in structural:
        def check(target=target,host=host,headers=headers):
            outcomes=[]
            for ns,version in [(old,"baseline"),(new,"candidate")]:
                d=make_demo(ns)
                h=request(ns,d,version,build_wire(ns,target=target,headers=headers,host=host))
                outcomes.append(normalized(ns,d,h))
                if target.startswith("//"):
                    assert h.path=="/"+target.lstrip("/")
                    assert not d.done and d.failure is None and h.responses==[(404,())]
                else:
                    assert d.done and d.failure is not None and h.responses==[(400,())]
                if version=="candidate":assert d.preflow_authorization_refusals==0
            assert outcomes[0]==outcomes[1]
        verify("header_free_bound_"+label,"header_free_bounds",check)
        def auth_check(target=target,host=host,headers=headers):
            d=make_demo(new);before=flow_snapshot(d)
            h=request(new,d,"candidate",build_wire(new,target=target,host=host,
                headers=[("Authorization","ignored")]+headers))
            if target.startswith("//"):
                assert h.path=="/"+target.lstrip("/")
                refused(new,d,h,before,1,False)
            else:
                assert d.done and d.failure is not None and h.responses==[(400,())]
                assert d.preflow_authorization_refusals==0
                assert d.request_invalid_reason==("host" if host!="normal" or any(n=="Host" for n,v in headers)
                                                 else "authorization")
                after=flow_snapshot(d)
                assert after[:4]==before[:4] and after[6:]==before[6:]
                assert all(v==0 for v in d.calls.values())
        verify("authorization_bound_"+label,"authorization_bounds",auth_check)
    for authorization in (False,True):
        def controlled(authorization=authorization):
            outcomes=[]
            parser_path="//"+new["AUTHORITY"]+"/"
            for ns,version in [(old,"baseline"),(new,"candidate")]:
                d=make_demo(ns);before=flow_snapshot(d)
                headers=[("Authorization","ignored")] if authorization else []
                h=request(ns,d,version,build_wire(ns,headers=headers),parser_path=parser_path)
                assert h.path==parser_path
                assert h.responses==[(400,())] and d.done and d.failure=="request_invalid"
                assert d.request_invalid_reason==("authorization" if authorization else "target_netloc")
                after=flow_snapshot(d)
                assert after[:4]==before[:4] and after[6:]==before[6:]
                assert all(v==0 for v in d.calls.values())
                if version=="candidate":assert d.preflow_authorization_refusals==0
                outcomes.append(normalized(ns,d,h))
            assert outcomes[0]==outcomes[1]
        verify("controlled_parser_result_"+("authorization" if authorization else "header_free"),
               "controlled_parser_result",controlled)
    raw_bounds=[
        ("request_line",b"GET /"+b"x"*8192+b" HTTP/1.1\r\n\r\n"),
        ("header_line",b"GET / HTTP/1.1\r\nHost: localhost:3000\r\nX: "+b"x"*4096+b"\r\n\r\n"),
        ("header_count",b"GET / HTTP/1.1\r\nHost: localhost:3000\r\n"+b"X: x\r\n"*33+b"\r\n"),
        ("header_total",b"GET / HTTP/1.1\r\nHost: localhost:3000\r\n"+(b"X: "+b"x"*3000+b"\r\n")*3+b"\r\n"),
        ("parser",b"GET / HTTP/9.0\r\nHost: localhost:3000\r\n\r\n")]
    for label,wire in raw_bounds:
        def check(wire=wire):
            outcomes=[]
            for ns,version in [(old,"baseline"),(new,"candidate")]:
                d=make_demo(ns)
                h=request(ns,d,version,wire)
                outcomes.append(normalized(ns,d,h))
                assert d.done and d.failure in {"request_invalid","request_limit"}
                assert all(v==0 for v in d.calls.values())
            assert outcomes[0]==outcomes[1]
        verify("raw_bound_"+label,"parser_header_limits",check)
    def persist():
        d=make_demo(new)
        request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
        request(new,d,"candidate",build_wire(new))
        assert d.preflow_authorization_refusals==1
        request(new,d,"candidate",build_wire(new,"POST","/login",login))
        assert d.attempted and d.preflow_authorization_refusals==1
        before=flow_snapshot(d)
        request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
        after=flow_snapshot(d)
        assert after[:4]==before[:4] and after[6:]==before[6:]
        assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
        copied=record_copy(new,d)
        d.clear()
        assert copied["preflow_authorization_refusals"]==1
        assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
    verify("counter_across_header_free_flow_cleanup","counter_persistence",persist)
    def first_only():
        d=make_demo(new);d.request_invalid_reason="http_parse"
        h=request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]))
        assert h.responses==[(403,())] and d.request_invalid_reason=="http_parse"
        assert d.preflow_authorization_refusals==1
        d.clear()
        assert d.request_invalid_reason=="http_parse" and d.preflow_authorization_refusals==1
    verify("first_reason_preserved","first_reason",first_only)
    def write_failure():
        global expected_write_failures
        d=make_demo(new);before=flow_snapshot(d)
        try:
            request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]),write_fail=True)
        except WriteFailure:
            expected_write_failures+=1
        else:
            raise AssertionError
        assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
        assert flow_snapshot(d)==before and all(v==0 for v in d.calls.values())
        new["handle_error"] (SimpleNamespace(demo=d),None,None)
        assert d.failure=="unexpected_failure" and d.done
        assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
        assert record_copy(new,d)["preflow_authorization_refusals"]==1
    verify("403_write_failure_propagates","response_failures",write_failure)
    def response_timeout():
        global expected_timeouts
        d=make_demo(new)
        try:
            request(new,d,"candidate",build_wire(new,headers=[("Authorization","ignored")]),timeout=True)
        except new["Halt"] as failure:
            assert failure.tag=="response_timeout"
            expected_timeouts+=1
        else:
            raise AssertionError
        assert d.preflow_authorization_refusals==1 and d.request_invalid_reason=="authorization"
        assert all(v==0 for v in d.calls.values())
        gate_fails(new,d)
    verify("403_response_timeout_propagates","response_failures",response_timeout)

result="failed"
failure=None
signal.signal(signal.SIGALRM,deadline)
signal.setitimer(signal.ITIMER_REAL,30)
try:
    run()
    result="passed"
except VerificationDeadline:
    failure="verification_deadline"
except AssertionError:
    failure="assertion_failed"
except BaseException:
    failure="harness_exception"
finally:
    signal.setitimer(signal.ITIMER_REAL,0)
elapsed=round(time.monotonic()-started,6)
print(json.dumps({"result":result,"failure_tag":failure,"failed_case":case if failure else None,
 "completed_cases":completed,"groups":groups,"handler_calls":handler_calls,
 "expected_write_failures":expected_write_failures,"expected_response_timeouts":expected_timeouts,
 "elapsed_seconds":elapsed,"deadline_seconds":30,"helper_sha256":SOURCE_SHA,
 "main_invoked":False,"network_listener_native_provider_cli_driver_browser_cargo":False},sort_keys=True))
raise SystemExit(0 if result=="passed" else 1)
```

### Complete new child logic archive

Marker: D01_OBSERVER_MEMORY_LOGIC_V1.
This fence is literal28019-byte source, final newline included. Indexing calls
already include intentional legal spaces before parentheses; do NOT normalize
them. No code was executed in this design phase.

```python
import ast, contextlib, copy, hashlib, http.server, io, json, re, signal, sys, time, urllib.parse
from types import SimpleNamespace

SENTINEL = "D01_PRIVATE_MEMORY_SENTINEL"
EXPECTED_LEGACY = {"record_copy": 1, "bounded_sequence": 4, "authorization_presence": 6,
    "postflow": 7, "header_free_routes_guards": 25, "header_free_bounds": 12,
    "authorization_bounds": 12, "controlled_parser_result": 2, "parser_header_limits": 5,
    "counter_persistence": 1, "first_reason": 1, "response_failures": 2}
EXPECTED_OBSERVER = {"classes": 11, "trace": 8, "faults": 7, "first": 2,
    "sites": 6, "known": 3, "fallback": 10, "privacy": 3}
SOURCE_HASH = "c0c022ecd91d92c19058b31d413864c980fe6b0ddfbd1303a45be629afd900e9"
UNCHANGED_HASH = "7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0"
BASELINE_HASH = "ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd"
LEGACY_LOGIC_HASH = "98b19b4d422067b0d49ea7eb1035a719d88bda672dc36579c6cf5c47cb0924eb"

class EnvelopeDeadline(BaseException):
    pass

class PrivateTrap(BaseException):
    pass

class GuardFailure(BaseException):
    pass

class SentinelError(Exception):
    def __str__(self):
        trap_counts["str"] += 1
        raise PrivateTrap
    def __repr__(self):
        trap_counts["repr"] += 1
        raise PrivateTrap

class ValueSubclass(ValueError):
    pass

class OSSubclass(OSError):
    pass

class FakeError(SentinelError):
    def __init__(self, trace=None, fail=False):
        self.trace, self.fail = trace, fail
        super().__init__(SENTINEL)
    def __getattribute__(self, name):
        if name == "__traceback__":
            if object.__getattribute__(self, "fail"):
                raise PrivateTrap
            return object.__getattribute__(self, "trace")
        return object.__getattribute__(self, name)

class FrameTrap:
    @property
    def tb_frame(self):
        raise PrivateTrap

class FaultHolder(dict):
    def __init__(self, mode):
        super().__init__(observed=False, diagnostic=None)
        self.mode = mode
    def __getitem__(self, key):
        if self.mode == "read" and key == "observed":
            raise PrivateTrap
        return dict.__getitem__(self, key)
    def __setitem__(self, key, value):
        if self.mode == key:
            raise PrivateTrap
        return dict.__setitem__(self, key, value)

def empty_holder():
    return {"observed": False, "diagnostic": None}

def legacy_namespace():
    tree = ast.parse(LEGACY_LOGIC)
    definitions = [n for n in tree.body if isinstance(n, (ast.FunctionDef, ast.ClassDef))]
    namespace = {"ast": ast, "contextlib": contextlib, "copy": copy, "hashlib": hashlib,
        "http": http, "io": io, "json": json, "re": re, "signal": signal, "time": time,
        "urllib": urllib, "SimpleNamespace": SimpleNamespace,
        "SOURCE_TEXT": SOURCE_TEXT, "BASELINE_TEXT": LEGACY_BASELINE_TEXT,
        "SOURCE_SHA": SOURCE_HASH, "BASE_COMMIT": "7461ab50f5a5fdf0aa5d7bd6c4b309575e7a4238",
        "groups": {}, "completed": 0, "case": "source_pin",
        "handler_calls": {"baseline": 0, "candidate": 0},
        "expected_write_failures": 0, "expected_timeouts": 0}
    # Original function/class nodes are unmodified; no original top-level tail runs.
    exec(compile(ast.Module(body=definitions, type_ignores=[]), "<legacy-definitions>", "exec"), namespace)
    return namespace

def selected_namespace(text):
    tree = ast.parse(text)
    classes = {"Failure", "Halt", "PreflowAuthorizationRefusal", "HeaderReader",
               "Demo", "DemoServer", "Handler"}
    functions = {"require", "observe_unexpected_failure", "main"}
    selected = [n for n in tree.body if isinstance(n, ast.Assign)]
    assert all(not any(isinstance(x, ast.Call) for x in ast.walk(n)) for n in selected)
    selected += [n for n in tree.body if
                 isinstance(n, ast.ClassDef) and n.name in classes or
                 isinstance(n, ast.FunctionDef) and n.name in functions]
    namespace = {"http": http, "contextlib": contextlib, "hashlib": hashlib, "re": re,
                 "sys": sys, "time": time, "urllib": urllib}
    exec(compile(ast.Module(body=selected, type_ignores=[]), "<observer-definitions>", "exec"), namespace)
    namespace["source_ast"] = tree
    # All ten trusted codeobjects exist. Neither main nor a server constructor is called.
    codes = (namespace["HeaderReader"].readline.__code__,
        namespace["DemoServer"].process_request.__code__,
        namespace["Demo"].begin.__code__, namespace["Demo"].callback.__code__,
        namespace["Demo"].invoke.__code__, namespace["Handler"].handle_one_request.__code__,
        namespace["Handler"].send_error.__code__, namespace["Handler"].get.__code__,
        namespace["Handler"].reply.__code__, namespace["main"].__code__)
    assert len(codes) == 10
    return namespace

def new_demo(namespace):
    return legacy["make_demo"] (namespace)

def holder_of(demo):
    return demo.unexpected_failure_observation

def checked_holder(holder):
    value = {key: value for key, value in dict.items(holder)}
    assert set(value) == {"observed", "diagnostic"} and type(value["observed"]) is bool
    diagnostic = value["diagnostic"]
    if diagnostic is not None:
        assert value["observed"] is True
        assert set(diagnostic) == {"site", "exception_class", "own_function", "own_line"}
        assert diagnostic["site"] in {"handler", "server", "main"}
        assert diagnostic["exception_class"] in {"AttributeError", "TypeError", "ValueError",
            "KeyError", "OSError", "BrokenPipeError", "ConnectionResetError", "TimeoutError", "other"}
        assert diagnostic["own_function"] in {None, "HeaderReader.readline",
            "DemoServer.process_request", "Demo.begin", "Demo.callback", "Demo.invoke",
            "Handler.handle_one_request", "Handler.send_error", "Handler.get", "Handler.reply", "main"}
        assert (diagnostic["own_function"] is None) == (diagnostic["own_line"] is None)
        assert diagnostic["own_line"] is None or type(diagnostic["own_line"]) is int and 1 <= diagnostic["own_line"] <= 1024
    raw = json.dumps(value, sort_keys=True).encode("ascii")
    assert len(raw) <= 160 and SENTINEL.encode() not in raw
    return value

def observe(namespace, error, holder=None, site="handler"):
    holder = empty_holder() if holder is None else holder
    try:
        raise error
    except BaseException:
        namespace["observe_unexpected_failure"] (holder, site)
    return checked_holder(holder)

def fake_trace(namespace, line, count=1, trusted=True, cloned=False):
    code = namespace["Handler"].reply.__code__ if trusted else fake_trace.__code__
    if cloned:
        code = code.replace()
        assert code is not namespace["Handler"].reply.__code__
    trace = None
    for _ in range(count):
        trace = SimpleNamespace(tb_frame=SimpleNamespace(f_code=code), tb_lineno=line, tb_next=trace)
    return trace

def class_case(name):
    classes = {"AttributeError": AttributeError, "TypeError": TypeError, "ValueError": ValueError,
        "KeyError": KeyError, "OSError": OSError, "BrokenPipeError": BrokenPipeError,
        "ConnectionResetError": ConnectionResetError, "TimeoutError": TimeoutError,
        "ValueSubclass": ValueSubclass, "OSSubclass": OSSubclass, "SentinelError": SentinelError}
    error = classes[name] (SENTINEL)
    before = dict(trap_counts)
    value = observe(aware, error)
    assert value["observed"] and value["diagnostic"]["exception_class"] == (name if name in {
        "AttributeError", "TypeError", "ValueError", "KeyError", "OSError",
        "BrokenPipeError", "ConnectionResetError", "TimeoutError"} else "other")
    assert value["diagnostic"]["own_function"] is None
    assert trap_counts == before

def trace_case(kind):
    options = {"line1": (1, 1, True, False), "line1024": (1024, 1, True, False),
        "line0": (0, 1, True, False), "line1025": (1025, 1, True, False),
        "links64": (512, 64, True, False), "links65": (512, 65, True, False),
        "untrusted": (512, 1, False, False), "cloned": (512, 1, True, True)}
    line, count, trusted, cloned = options[kind]
    value = observe(aware, FakeError(fake_trace(aware, line, count, trusted, cloned)))
    valid = line in (1, 1024, 512) and count <= 64 and trusted and not cloned
    assert value["diagnostic"]["own_function"] == ("Handler.reply" if valid else None)
    assert value["diagnostic"]["own_line"] == (line if valid else None)

def fault_case(kind):
    if kind == "accessor":
        value = observe(aware, FakeError(fail=True))
        assert value == {"observed": True, "diagnostic": None}
    elif kind == "frame_accessor":
        value = observe(aware, FakeError(FrameTrap()))
        assert value == {"observed": True, "diagnostic": None}
    elif kind in ("read", "observed", "diagnostic"):
        holder = FaultHolder(kind)
        value = observe(aware, SentinelError(SENTINEL), holder)
        assert value == {"observed": kind == "diagnostic", "diagnostic": None}
        if kind == "diagnostic":
            holder.mode = None
            assert observe(aware, ValueError(SENTINEL), holder) == value
    elif kind == "invalid_site":
        assert observe(aware, ValueError(SENTINEL), site=SENTINEL) == empty_holder()
    else:
        holder = empty_holder()
        aware["observe_unexpected_failure"] (holder, "server")
        assert checked_holder(holder) == {"observed": True, "diagnostic": {
            "site": "server", "exception_class": "other", "own_function": None, "own_line": None}}

class MemoryWrite(io.BytesIO):
    def __init__(self, once=False):
        super().__init__()
        self.once = once
    def write(self, raw):
        if self.once:
            self.once = False
            raise OSError(SENTINEL)
        return super().write(raw)

def memory_handler(namespace, demo, fault=None):
    class Sink(namespace["Handler"]):
        def parse_request(self):
            if fault == "parser":
                raise ValueError(SENTINEL)
            if fault == "send_error":
                self.send_error(400)
                return False
            return super().parse_request()
        def record_request_reason(self, reason):
            if fault == "send_error" and reason == "http_parse":
                raise TypeError(SENTINEL)
            return super().record_request_reason(reason)
        def cookies(self):
            demo.calls["cookies"] += 1
            if fault == "dispatch":
                raise TypeError(SENTINEL)
            return super().cookies()
        def get(self, target):
            demo.calls["get"] += 1
            return super().get(target)
        def reply(self, status, text, **options):
            self.responses.append((status, tuple(sorted(options))))
            return super().reply(status, text, **options)
        def send_response(self, status, message=None):
            self.sent_statuses.append(status)
        def send_header(self, name, value):
            self.sent_headers.append((name, value))
        def end_headers(self):
            self.header_ends += 1
    handler = object.__new__(Sink)
    handler.server = SimpleNamespace(demo=demo)
    handler.rfile = io.BytesIO(legacy["build_wire"] (namespace))
    handler.wfile = MemoryWrite(once=fault == "reply")
    handler.responses, handler.sent_statuses, handler.sent_headers = [], [], []
    handler.header_ends = 0
    return handler

def main_handler(namespace, record, demo):
    function = next(n for n in namespace["source_ast"].body if isinstance(n, ast.FunctionDef) and n.name == "main")
    block = next(n for n in function.body if isinstance(n, ast.Try))
    handler = next(n for n in block.handlers if isinstance(n.type, ast.Name) and n.type.id == "Exception")
    environment = dict(namespace, record=record, demo=demo, stage="arguments")
    try:
        raise ValueError(SENTINEL)
    except Exception:
        exec(compile(ast.Module(body=copy.deepcopy(handler.body), type_ignores=[]),
                     "<main-except-only>", "exec"), environment)

def site_outcome(namespace, site):
    demo = new_demo(namespace)
    demo.stage = "request"
    if site in {"parser", "dispatch", "reply", "send_error"}:
        handler = memory_handler(namespace, demo, site)
        handler.handle_one_request()
        return legacy["normalized"] (namespace, demo, handler), demo
    if site == "server":
        events = []
        def finish(request, address):
            events.append("finish")
            raise KeyError(SENTINEL)
        def shutdown(request):
            events.append("shutdown")
        server = SimpleNamespace(demo=demo, active=None, finish_request=finish, shutdown_request=shutdown)
        try:
            namespace["DemoServer"].process_request(server, None, None)
        except Exception:
            namespace["DemoServer"].handle_error(server, None, None)
        assert server.active is None and events == ["finish", "shutdown"]
        return (legacy["flow_snapshot"] (demo), demo.stage, demo.calls, events), demo
    record = {"failure_tag": None, "failure_stage": None,
              "unexpected_failure_observation": holder_of(demo) if hasattr(demo, "unexpected_failure_observation") else empty_holder()}
    main_handler(namespace, record, demo)
    assert record["failure_tag"] == "unexpected_failure" and record["failure_stage"] == "request"
    return {k: v for k, v in record.items() if k != "unexpected_failure_observation"}, demo

def site_case(site):
    before, ignored = site_outcome(reference, site)
    after, demo = site_outcome(aware, site)
    assert before == after
    value = checked_holder(holder_of(demo))
    labels = {"parser": ("handler", "ValueError", "Handler.handle_one_request"),
        "dispatch": ("handler", "TypeError", "Handler.get"),
        "reply": ("handler", "OSError", "Handler.reply"),
        "send_error": ("handler", "TypeError", "Handler.send_error"),
        "server": ("server", "KeyError", "DemoServer.process_request"),
        "main": ("main", "ValueError", None)}
    site_label, class_label, own = labels[site]
    assert value["observed"] and value["diagnostic"]["site"] == site_label
    assert value["diagnostic"]["exception_class"] == class_label
    assert value["diagnostic"]["own_function"] == own

def first_case(kind):
    if kind == "direct":
        holder = empty_holder()
        first = observe(aware, FakeError(fake_trace(aware, 1)), holder)
        assert observe(aware, ValueError(SENTINEL), holder, "server") == first
        return
    outcomes = []
    for namespace in (reference, aware):
        demo = new_demo(namespace)
        handler = memory_handler(namespace, demo, "parser")
        handler.wfile.once = True
        try:
            handler.handle_one_request()
        except OSError:
            # The original reply failure escapes; the existing server sink runs.
            namespace["DemoServer"].handle_error(SimpleNamespace(demo=demo), None, None)
        else:
            raise AssertionError
        assert demo.failure == "unexpected_failure" and demo.done
        if namespace is aware:
            value = checked_holder(holder_of(demo))
            assert value["diagnostic"]["exception_class"] == "ValueError"
            assert value["diagnostic"]["site"] == "handler"
        outcomes.append(legacy["normalized"] (namespace, demo, handler))
    assert outcomes[0] == outcomes[1]

def known_case(kind):
    outcomes = []
    for namespace in (reference, aware):
        demo = new_demo(namespace)
        handler = memory_handler(namespace, demo)
        if kind == "failure":
            handler.rfile = io.BytesIO(legacy["build_wire"] (namespace, host="wrong"))
        elif kind == "halt":
            demo.budget.fail_response = True
        try:
            handler.handle_one_request()
        except namespace["Halt"] as error:
            assert kind == "halt" and error.tag == "response_timeout"
        else:
            assert kind != "halt"
        if namespace is aware:
            assert checked_holder(holder_of(demo)) == empty_holder()
        outcomes.append(legacy["normalized"] (namespace, demo, handler))
    assert outcomes[0] == outcomes[1]

def fallback_case(mode, site):
    if mode == "legacy":
        namespace = legacy["load_selected"] (SOURCE_TEXT, "candidate")
        demo = legacy["make_demo"] (namespace)
        del demo.unexpected_failure_observation
        namespace["handle_error"] (SimpleNamespace(demo=demo), None, None)
        assert demo.failure == "unexpected_failure" and demo.done
        assert "observe_unexpected_failure" not in namespace
        return
    namespace = selected_namespace(SOURCE_TEXT)
    if mode == "absent":
        del namespace["observe_unexpected_failure"]
    elif mode == "guard":
        def fail(*args):
            raise GuardFailure
        namespace["observe_unexpected_failure"] = fail
    demo = new_demo(namespace)
    demo.stage = "request"
    if mode == "field":
        del demo.unexpected_failure_observation
    if site == "handler":
        handler = memory_handler(namespace, demo, "parser")
        handler.handle_one_request()
        assert demo.failure == "unexpected_failure" and demo.done
        assert handler.responses == [(400, ())]
    elif site == "server":
        namespace["DemoServer"].handle_error(SimpleNamespace(demo=demo), None, None)
        assert demo.failure == "unexpected_failure" and demo.done
    else:
        record = {"failure_tag": None, "failure_stage": None}
        if mode != "field":
            record["unexpected_failure_observation"] = holder_of(demo)
        main_handler(namespace, record, demo)
        assert record["failure_tag"] == "unexpected_failure" and record["failure_stage"] == "request"
    if mode != "field":
        assert checked_holder(holder_of(demo)) == empty_holder()

def writer_trial(namespace, fail):
    tree = namespace["source_ast"]
    main = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "main")
    start = next(i for i, n in enumerate(main.body) if isinstance(n, ast.Assign)
                 and any(isinstance(t, ast.Name) and t.id == "written" for t in n.targets))
    # Preserve the exact writer/print/return tail inside a fresh wrapper, never main.
    wrapper = copy.deepcopy(main)
    wrapper.name = "selected_evidence_tail"
    wrapper.body = copy.deepcopy(main.body[start:])
    sink = io.BytesIO()
    class Writer:
        def __enter__(self):
            return self
        def __exit__(self, *args):
            return False
        def write(self, raw):
            if fail:
                raise OSError(SENTINEL)
            return sink.write(raw)
        def flush(self):
            pass
        def fileno(self):
            return 1
    stdout = io.StringIO()
    def safe_print(raw, **options):
        stdout.write(raw + "\n")
    holder = empty_holder()
    if "observe_unexpected_failure" in namespace:
        observe(namespace, SentinelError(SENTINEL), holder)
    record = {"result": "failed", "failure_tag": "unexpected_failure",
              "failure_stage": "request", "cleanup": {},
              "unexpected_failure_observation": holder}
    environment = dict(namespace, os=SimpleNamespace(fdopen=lambda *a: Writer(),
        fsync=lambda *a: None, close=lambda *a: None), json=json, print=safe_print,
        evidence_fd=1, record=record)
    exec(compile(ast.Module(body=[wrapper], type_ignores=[]), "<evidence-tail-only>", "exec"), environment)
    assert environment["selected_evidence_tail"] () == 1
    public = json.loads(stdout.getvalue())
    assert set(public) == {"result", "failure_tag", "cleanup_failure_tag",
                          "evidence_failure_tag", "evidence_written"}
    assert "unexpected_failure_observation" not in public
    assert SENTINEL not in stdout.getvalue() and SENTINEL.encode() not in sink.getvalue()
    assert public["failure_tag"] == "unexpected_failure" and public["result"] == "failed"
    assert public["evidence_written"] is (not fail)
    if fail:
        assert public["evidence_failure_tag"] == "evidence_write_failed"
        assert record["failure_stage"] == "request"
    else:
        assert checked_holder(json.loads(sink.getvalue())["unexpected_failure_observation"]) == holder
    return public

def privacy_case(kind):
    before = dict(trap_counts)
    if kind == "sentinel":
        value = observe(aware, SentinelError(SENTINEL))
        assert value["diagnostic"]["exception_class"] == "other"
    else:
        assert writer_trial(reference, kind == "writer_failure") == writer_trial(aware, kind == "writer_failure")
    assert trap_counts == before

CASES = (
    ("class_attribute", "classes", "class", ("AttributeError",)),
    ("class_type", "classes", "class", ("TypeError",)),
    ("class_value", "classes", "class", ("ValueError",)),
    ("class_key", "classes", "class", ("KeyError",)),
    ("class_os", "classes", "class", ("OSError",)),
    ("class_broken_pipe", "classes", "class", ("BrokenPipeError",)),
    ("class_connection_reset", "classes", "class", ("ConnectionResetError",)),
    ("class_timeout", "classes", "class", ("TimeoutError",)),
    ("class_value_subclass", "classes", "class", ("ValueSubclass",)),
    ("class_os_subclass", "classes", "class", ("OSSubclass",)),
    ("class_private_other", "classes", "class", ("SentinelError",)),
    ("trace_line1", "trace", "trace", ("line1",)),
    ("trace_line1024", "trace", "trace", ("line1024",)),
    ("trace_line0", "trace", "trace", ("line0",)),
    ("trace_line1025", "trace", "trace", ("line1025",)),
    ("trace_64", "trace", "trace", ("links64",)),
    ("trace_65", "trace", "trace", ("links65",)),
    ("trace_untrusted", "trace", "trace", ("untrusted",)),
    ("trace_equal_clone", "trace", "trace", ("cloned",)),
    ("fault_trace_accessor", "faults", "fault", ("accessor",)),
    ("fault_frame_accessor", "faults", "fault", ("frame_accessor",)),
    ("fault_holder_read", "faults", "fault", ("read",)),
    ("fault_latch_write", "faults", "fault", ("observed",)),
    ("fault_diagnostic_write", "faults", "fault", ("diagnostic",)),
    ("fault_invalid_site", "faults", "fault", ("invalid_site",)),
    ("fault_no_active_exception", "faults", "fault", ("no_exception",)),
    ("first_direct", "first", "first", ("direct",)),
    ("first_second_reply_failure", "first", "first", ("reply",)),
    ("site_parser", "sites", "site", ("parser",)),
    ("site_dispatch", "sites", "site", ("dispatch",)),
    ("site_reply", "sites", "site", ("reply",)),
    ("site_send_error", "sites", "site", ("send_error",)),
    ("site_server", "sites", "site", ("server",)),
    ("site_main_block", "sites", "site", ("main",)),
    ("known_failure", "known", "known", ("failure",)),
    ("known_halt", "known", "known", ("halt",)),
    ("known_success", "known", "known", ("success",)),
    ("fallback_absent_handler", "fallback", "fallback", ("absent", "handler")),
    ("fallback_absent_server", "fallback", "fallback", ("absent", "server")),
    ("fallback_absent_main", "fallback", "fallback", ("absent", "main")),
    ("fallback_field_handler", "fallback", "fallback", ("field", "handler")),
    ("fallback_field_server", "fallback", "fallback", ("field", "server")),
    ("fallback_field_main", "fallback", "fallback", ("field", "main")),
    ("fallback_baseexception_handler", "fallback", "fallback", ("guard", "handler")),
    ("fallback_baseexception_server", "fallback", "fallback", ("guard", "server")),
    ("fallback_baseexception_main", "fallback", "fallback", ("guard", "main")),
    ("fallback_legacy_selector_field", "fallback", "fallback", ("legacy", "server")),
    ("privacy_str_repr", "privacy", "privacy", ("sentinel",)),
    ("privacy_writer_success", "privacy", "privacy", ("writer_success",)),
    ("privacy_writer_failure", "privacy", "privacy", ("writer_failure",)),
)
RUNNERS = {"class": class_case, "trace": trace_case, "fault": fault_case,
    "first": first_case, "site": site_case, "known": known_case,
    "fallback": fallback_case, "privacy": privacy_case}

started = time.monotonic()
legacy = None
observer_completed = 0
observer_groups = {key: 0 for key in EXPECTED_OBSERVER}
trap_counts = {"str": 0, "repr": 0}
case = "source_archive"
failure = None
result = "failed"

def deadline(signum, frame):
    raise EnvelopeDeadline

try:
    signal.signal(signal.SIGALRM, deadline)
    signal.setitimer(signal.ITIMER_REAL, 30)
    assert hashlib.sha256(SOURCE_TEXT.encode()).hexdigest() == SOURCE_HASH
    assert hashlib.sha256(UNCHANGED_TEXT.encode()).hexdigest() == UNCHANGED_HASH
    assert hashlib.sha256(LEGACY_BASELINE_TEXT.encode()).hexdigest() == BASELINE_HASH
    assert hashlib.sha256(LEGACY_LOGIC.encode()).hexdigest() == LEGACY_LOGIC_HASH
    old_archive = ("BASELINE_TEXT=" + json.dumps(LEGACY_BASELINE_TEXT, ensure_ascii=True) + "\n"
                   + "SOURCE_TEXT=" + json.dumps(UNCHANGED_TEXT, ensure_ascii=True) + "\n" + LEGACY_LOGIC)
    assert len(old_archive.encode()) == 91163
    assert hashlib.sha256(old_archive.encode()).hexdigest() == "ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f"
    assert len(CASES) == 50 and len({n for n, g, k, a in CASES}) == 50
    planned = {key: sum(group == key for name, group, kind, args in CASES) for key in EXPECTED_OBSERVER}
    assert planned == EXPECTED_OBSERVER
    legacy = legacy_namespace()
    case = "legacy_78"
    legacy["run"] ()
    assert legacy["completed"] == 78 and legacy["groups"] == EXPECTED_LEGACY
    assert legacy["handler_calls"] == {"baseline": 44, "candidate": 80}
    assert legacy["expected_write_failures"] == 1 and legacy["expected_timeouts"] == 1
    case = "observer_selector"
    reference = selected_namespace(UNCHANGED_TEXT)
    aware = selected_namespace(SOURCE_TEXT)
    for name, group, kind, args in CASES:
        case = name
        RUNNERS[kind] (*args)
        observer_completed += 1
        observer_groups[group] += 1
    case = "observer_totals"
    assert observer_completed == 50 and observer_groups == EXPECTED_OBSERVER
    assert trap_counts == {"str": 0, "repr": 0}
    result = "passed"
except EnvelopeDeadline:
    failure = "verification_deadline"
except AssertionError:
    failure = "assertion_failed"
except BaseException:
    failure = "harness_exception"
finally:
    signal.setitimer(signal.ITIMER_REAL, 0)

legacy_completed = 0 if legacy is None else legacy["completed"]
legacy_groups = {key: 0 if legacy is None else legacy["groups"].get(key, 0) for key in EXPECTED_LEGACY}
failed_case = None if failure is None else (
    "legacy_" + legacy["case"] if case == "legacy_78" and legacy is not None else case)
receipt = {"schema": "riauth.d01-observer-memory/v1", "result": result,
    "failure_tag": failure, "failed_case": failed_case, "planned_cases": 128,
    "legacy_planned": 78, "observer_planned": 50,
    "legacy_completed": legacy_completed, "observer_completed": observer_completed,
    "completed_cases": legacy_completed + observer_completed,
    "legacy_groups": legacy_groups, "observer_groups": observer_groups,
    "legacy_handler_calls": {"baseline": 0, "candidate": 0} if legacy is None else legacy["handler_calls"],
    "expected_write_failures": 0 if legacy is None else legacy["expected_write_failures"],
    "expected_response_timeouts": 0 if legacy is None else legacy["expected_timeouts"],
    "private_format_traps": trap_counts, "elapsed_seconds": round(time.monotonic() - started, 6),
    "deadline_seconds": 30, "helper_sha256": SOURCE_HASH, "unchanged_sha256": UNCHANGED_HASH,
    "legacy_archive_sha256": "ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f",
    "main_invoked": False, "server_constructed": False,
    "network_listener_native_provider_cli_driver_browser_cargo": False}
raw = (json.dumps(receipt, sort_keys=True) + "\n").encode("ascii")
assert len(raw) <= 8192 and SENTINEL.encode() not in raw
sys.stdout.buffer.write(raw)
sys.stdout.buffer.flush()
raise SystemExit(0 if result == "passed" else 1)
```

### Complete prospective controller archive

Marker: D01_OBSERVER_MEMORY_CONTROLLER_V1.
This literal15438-byte source defines the future controller; it is NOT called
here. Child30s uses a future SIGALRM. Controller35s reserves33s for communicate
and remaining2s for owned kill/reap; captures have child-only16KiB RLIMIT_FSIZE
and private exclusive0600 files. No existing signals/limits were changed now.

The first complete structurally safe fixed child JSON and numeric exit are
saved to d01-memory-retained.json BEFORE success/count/oracle expectations.
An empty/malformed result is explicitly unavailable, never an invented receipt.
Raw captures remain bounded private files and are never printed/uploaded here.
Review expectations go into a separate exclusive d01-memory-reviewed.json;
the original retained receipt is never overwritten. First failure remains
separate from cleanup/retention errors. Only the owned child may be killed/
reaped. Reaping failure remains unproven and cannot release an alleged resource.

The returned post-review-save elapsed/within35 sample must also be retained
by the future root caller. Earlier saved elapsed samples cannot independently
prove the whole35s. This is a finite budget and stop/reap design, not a guarantee
against an OS scheduling/Popen/filesystem stall. No current capacity, timing or
availability is claimed. Root future release requires its fresh8GiB floor/
private resource preflight; no source phase samples, lowers thresholds or
deletes caches. Captures/receipts stay owned and retained until root review;
this code performs no directory/cache deletion.

```python
import hashlib, json, math, os, pathlib, resource, stat, subprocess, sys, time

PAYLOAD_SHA256 = "b7bdeb6f9e24b8c537cfdca7506f570a6e2e180efa2f560dad2c6c1693aa0822"
CAP = 16384
PLANNED_LEGACY = {"record_copy": 1, "bounded_sequence": 4, "authorization_presence": 6,
    "postflow": 7, "header_free_routes_guards": 25, "header_free_bounds": 12,
    "authorization_bounds": 12, "controlled_parser_result": 2, "parser_header_limits": 5,
    "counter_persistence": 1, "first_reason": 1, "response_failures": 2}
PLANNED_OBSERVER = {"classes": 11, "trace": 8, "faults": 7, "first": 2,
    "sites": 6, "known": 3, "fallback": 10, "privacy": 3}

OBSERVER_NAMES = (
    "class_attribute",
    "class_type",
    "class_value",
    "class_key",
    "class_os",
    "class_broken_pipe",
    "class_connection_reset",
    "class_timeout",
    "class_value_subclass",
    "class_os_subclass",
    "class_private_other",
    "trace_line1",
    "trace_line1024",
    "trace_line0",
    "trace_line1025",
    "trace_64",
    "trace_65",
    "trace_untrusted",
    "trace_equal_clone",
    "fault_trace_accessor",
    "fault_frame_accessor",
    "fault_holder_read",
    "fault_latch_write",
    "fault_diagnostic_write",
    "fault_invalid_site",
    "fault_no_active_exception",
    "first_direct",
    "first_second_reply_failure",
    "site_parser",
    "site_dispatch",
    "site_reply",
    "site_send_error",
    "site_server",
    "site_main_block",
    "known_failure",
    "known_halt",
    "known_success",
    "fallback_absent_handler",
    "fallback_absent_server",
    "fallback_absent_main",
    "fallback_field_handler",
    "fallback_field_server",
    "fallback_field_main",
    "fallback_baseexception_handler",
    "fallback_baseexception_server",
    "fallback_baseexception_main",
    "fallback_legacy_selector_field",
    "privacy_str_repr",
    "privacy_writer_success",
    "privacy_writer_failure",
)

LEGACY_NAMES = (
    "initial_count_copy",
    "bounded_refusal_1",
    "bounded_refusal_2",
    "bounded_refusal_3",
    "bounded_refusal_4",
    "presence_empty",
    "presence_duplicate",
    "presence_post",
    "presence_protected",
    "presence_callback",
    "presence_other_method",
    "strict_state_attempted_1",
    "strict_state_pending_2",
    "strict_state_cookie_3",
    "strict_state_subject_4",
    "strict_state_attempted_5",
    "strict_state_attempted_6",
    "defensive_exhaustion",
    "header_free_root",
    "header_free_protected_before",
    "header_free_unavailable",
    "header_free_root_query",
    "header_free_protected_query",
    "header_free_callback_empty",
    "header_free_callback_ready",
    "header_free_login",
    "header_free_login_repeated",
    "header_free_protected_after",
    "header_free_protected_before_missing",
    "header_free_protected_cookie_mismatch",
    "header_free_method",
    "header_free_post_target",
    "header_free_post_query",
    "header_free_origin_missing",
    "header_free_origin_wrong",
    "header_free_origin_duplicate",
    "header_free_type_missing",
    "header_free_type_wrong",
    "header_free_type_duplicate",
    "header_free_cookie_duplicate",
    "header_free_cookie_shape",
    "header_free_cookie_own_duplicate",
    "header_free_cookie_unrelated",
    "header_free_bound_host_absent",
    "header_free_bound_host_wrong",
    "header_free_bound_host_duplicate",
    "header_free_bound_transfer",
    "header_free_bound_expect",
    "header_free_bound_length_nonzero",
    "header_free_bound_length_duplicate",
    "header_free_bound_length_format",
    "header_free_bound_scheme",
    "header_free_bound_netloc",
    "header_free_bound_fragment",
    "header_free_bound_parse_target",
    "authorization_bound_host_absent",
    "authorization_bound_host_wrong",
    "authorization_bound_host_duplicate",
    "authorization_bound_transfer",
    "authorization_bound_expect",
    "authorization_bound_length_nonzero",
    "authorization_bound_length_duplicate",
    "authorization_bound_length_format",
    "authorization_bound_scheme",
    "authorization_bound_netloc",
    "authorization_bound_fragment",
    "authorization_bound_parse_target",
    "controlled_parser_result_header_free",
    "controlled_parser_result_authorization",
    "raw_bound_request_line",
    "raw_bound_header_line",
    "raw_bound_header_count",
    "raw_bound_header_total",
    "raw_bound_parser",
    "counter_across_header_free_flow_cleanup",
    "first_reason_preserved",
    "403_write_failure_propagates",
    "403_response_timeout_propagates",
)

class ControllerFailure(Exception):
    pass

def require(condition):
    if not condition:
        raise ControllerFailure

def exclusive(directory, name):
    descriptor = os.open(directory / name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    os.fchmod(descriptor, 0o600)
    return descriptor

def private_read(directory, name):
    descriptor = os.open(directory / name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        info = os.fstat(descriptor)
        require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid()
                and stat.S_IMODE(info.st_mode) == 0o600 and info.st_nlink == 1
                and 0 <= info.st_size <= CAP)
        with os.fdopen(descriptor, "rb", closefd=False) as stream:
            raw = stream.read(CAP + 1)
        require(len(raw) <= CAP)
        return raw
    finally:
        os.close(descriptor)

def save(directory, name, document):
    raw = (json.dumps(document, sort_keys=True) + "\n").encode("ascii")
    require(len(raw) <= CAP)
    descriptor = exclusive(directory, name)
    with os.fdopen(descriptor, "wb") as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())

def child_limits():
    resource.setrlimit(resource.RLIMIT_FSIZE, (CAP, CAP))

def safe_document(raw, observer_names, legacy_names):
    document = json.loads(raw)
    keys = {"schema", "result", "failure_tag", "failed_case", "planned_cases",
        "legacy_planned", "observer_planned", "legacy_completed", "observer_completed",
        "completed_cases", "legacy_groups", "observer_groups", "legacy_handler_calls",
        "expected_write_failures", "expected_response_timeouts", "private_format_traps",
        "elapsed_seconds", "deadline_seconds", "helper_sha256", "unchanged_sha256",
        "legacy_archive_sha256", "main_invoked", "server_constructed",
        "network_listener_native_provider_cli_driver_browser_cargo"}
    require(type(document) is dict and set(document) == keys)
    require(document["schema"] == "riauth.d01-observer-memory/v1")
    require(document["result"] in {"passed", "failed"})
    require(document["failure_tag"] in {None, "verification_deadline", "assertion_failed", "harness_exception"})
    names = set(observer_names) | {"source_archive", "legacy_source_pin", "observer_selector", "observer_totals"} | {
        "legacy_" + name for name in legacy_names}
    require(document["failed_case"] is None or document["failed_case"] in names)
    for key in ("planned_cases", "legacy_planned", "observer_planned", "legacy_completed",
                "observer_completed", "completed_cases", "deadline_seconds",
                "expected_write_failures", "expected_response_timeouts"):
        require(type(document[key]) is int and 0 <= document[key] <= 128)
    for key, names in (("legacy_groups", set(PLANNED_LEGACY)), ("observer_groups", set(PLANNED_OBSERVER)),
                      ("legacy_handler_calls", {"baseline", "candidate"}),
                      ("private_format_traps", {"str", "repr"})):
        require(type(document[key]) is dict and set(document[key]) == names)
        require(all(type(value) is int and 0 <= value <= 128 for value in document[key].values()))
    for key in ("main_invoked", "server_constructed", "network_listener_native_provider_cli_driver_browser_cargo"):
        require(type(document[key]) is bool)
    require(type(document["elapsed_seconds"]) in (int, float)
            and math.isfinite(document["elapsed_seconds"]) and document["elapsed_seconds"] >= 0)
    for key, value in (("helper_sha256", "c0c022ecd91d92c19058b31d413864c980fe6b0ddfbd1303a45be629afd900e9"),
                       ("unchanged_sha256", "7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0"),
                       ("legacy_archive_sha256", "ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f")):
        require(document[key] == value)
    return document

def bounded_memory_envelope(payload, directory):
    started = time.monotonic()
    stop = started + 33
    deadline = started + 35
    child = None
    output_fd = error_fd = None
    record = {"schema": "riauth.d01-observer-memory-controller/v1",
        "payload_sha256": PAYLOAD_SHA256, "controller_seconds": 35, "child_seconds": 30,
        "spawn_count": 0, "child_exit": None, "child_reaped": False,
        "first_failure": None, "cleanup_failures": [], "child_result": None,
        "stdout_bytes": None, "stderr_bytes": None, "stdout_sha256": None,
        "stderr_sha256": None, "elapsed_seconds": None, "within35": False}
    try:
        require(hashlib.sha256(payload).hexdigest() == PAYLOAD_SHA256)
        directory = pathlib.Path(directory)
        descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            info = os.fstat(descriptor)
            require(stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid()
                    and stat.S_IMODE(info.st_mode) == 0o700 and not os.listdir(descriptor))
        finally:
            os.close(descriptor)
        output_fd = exclusive(directory, "d01-memory-child.stdout")
        error_fd = exclusive(directory, "d01-memory-child.stderr")
        child = subprocess.Popen([sys.executable, "-I", "-B", "-"], stdin=subprocess.PIPE,
            stdout=output_fd, stderr=error_fd, cwd=directory,
            env={"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C"},
            preexec_fn=child_limits)
        record["spawn_count"] = 1
        child.communicate(input=payload, timeout=max(0.001, stop - time.monotonic()))
    except subprocess.TimeoutExpired:
        record["first_failure"] = "controller_timeout"
    except BaseException:
        record["first_failure"] = "controller_failure"
    finally:
        if child is not None:
            if child.poll() is None:
                try:
                    child.kill()
                except BaseException:
                    record["cleanup_failures"].append("owned_kill_failed")
                try:
                    child.communicate(timeout=max(0.001, deadline - time.monotonic()))
                except BaseException:
                    record["cleanup_failures"].append("owned_reap_unproven")
            record["child_exit"] = child.poll()
            record["child_reaped"] = record["child_exit"] is not None
        for descriptor in (output_fd, error_fd):
            if descriptor is not None:
                try:
                    os.close(descriptor)
                except OSError:
                    record["cleanup_failures"].append("capture_close_failed")
    # Capture/exit and the complete structurally safe fixed JSON are retained
    # before success/count/oracle expectations. Raw captures are never printed.
    try:
        stdout = private_read(directory, "d01-memory-child.stdout")
        stderr = private_read(directory, "d01-memory-child.stderr")
        record["stdout_bytes"], record["stderr_bytes"] = len(stdout), len(stderr)
        record["stdout_sha256"] = hashlib.sha256(stdout).hexdigest()
        record["stderr_sha256"] = hashlib.sha256(stderr).hexdigest()
        try:
            record["child_result"] = safe_document(stdout, OBSERVER_NAMES, LEGACY_NAMES)
        except BaseException:
            if record["first_failure"] is None:
                record["first_failure"] = "fixed_result_unavailable"
        record["elapsed_seconds"] = round(time.monotonic() - started, 6)
        record["within35"] = record["elapsed_seconds"] <= 35
        save(directory, "d01-memory-retained.json", record)
    except BaseException:
        # Missing/malformed capture never becomes an invented child result.
        return {"result": "failed", "failure_tag": record["first_failure"] or "retention_failed",
                "evidence_failure_tag": "retention_failed", "child_exit": record["child_exit"],
                "child_reaped": record["child_reaped"]}
    outcome = {"schema": "riauth.d01-observer-memory-review/v1", "result": "failed",
        "failure_tag": record["first_failure"], "retained_before_expectations": True,
        "child_exit": record["child_exit"], "child_reaped": record["child_reaped"],
        "within35": record["within35"]}
    try:
        require(record["first_failure"] is None and record["child_reaped"]
                and record["child_exit"] == 0 and record["within35"]
                and not record["cleanup_failures"] and not stderr)
        document = record["child_result"]
        require(document is not None and document["result"] == "passed"
                and document["failure_tag"] is None and document["failed_case"] is None)
        require(document["planned_cases"] == 128 and document["legacy_planned"] == 78
                and document["observer_planned"] == 50 and document["completed_cases"] == 128
                and document["legacy_completed"] == 78 and document["observer_completed"] == 50)
        require(document["legacy_groups"] == PLANNED_LEGACY
                and document["observer_groups"] == PLANNED_OBSERVER
                and document["legacy_handler_calls"] == {"baseline": 44, "candidate": 80}
                and document["expected_write_failures"] == 1
                and document["expected_response_timeouts"] == 1
                and document["private_format_traps"] == {"str": 0, "repr": 0}
                and document["deadline_seconds"] == 30 and document["elapsed_seconds"] <= 30
                and not document["main_invoked"] and not document["server_constructed"]
                and not document["network_listener_native_provider_cli_driver_browser_cargo"])
        outcome["result"] = "passed"
    except BaseException:
        if outcome["failure_tag"] is None:
            outcome["failure_tag"] = "expectations_failed"
    outcome["elapsed_seconds_before_review_save"] = round(time.monotonic() - started, 6)
    outcome["within35"] = outcome["elapsed_seconds_before_review_save"] <= 35
    if not outcome["within35"]:
        outcome["result"] = "failed"
        if outcome["failure_tag"] is None:
            outcome["failure_tag"] = "controller_budget_exceeded"
    try:
        save(directory, "d01-memory-reviewed.json", outcome)
    except BaseException:
        outcome["result"] = "failed"
        outcome["evidence_failure_tag"] = "review_write_failed"
        if outcome["failure_tag"] is None:
            outcome["failure_tag"] = "review_write_failed"
    # Caller must retain this post-save completion sample too; the saved review
    # is an earlier sample and cannot independently prove a whole-controller bound.
    outcome["elapsed_seconds"] = round(time.monotonic() - started, 6)
    outcome["within35"] = outcome["elapsed_seconds"] <= 35
    if not outcome["within35"]:
        outcome["result"] = "failed"
        if outcome["failure_tag"] is None:
            outcome["failure_tag"] = "controller_budget_exceeded"
    return outcome
```

### Actual source-only checks and remaining gate

Performed only static tools: whole old ce79 archive reconstruction/hash/AST
parse/in-memory compile; new full child payload reconstruction/hash/AST parse/
compile; literal50-name/group counts and128 planned total; controller source
parse/compile, frozen128-name allowlist and exactly one future spawn node.
Definitions-only helper selector and exact writer/print/return wrapper AST
compile passed without executing any definition; writer tail normalizes equal
to the unchanged7fbc tail. Current c0c022 source hash remains exact.

One archive check initially serialized SOURCE_TEXT before BASELINE_TEXT and
failed the old ce79 hash assertion; the published explicit baseline-first order
corrected that static assembly. No harness ran. Static body review also caught
that a writer-tail return must remain inside a function; the final source uses
the complete copied tail in its own wrapper. These are design/static findings,
not recovered actual exceptions, helper defects or new runtime evidence.

Final report reconstruction/prefix/docs/hygiene/whitespace/scope receipts are
recorded below after authoring. No actual case passes/fails, imports, timers,
signals, subprocess child, callback, browser result or resource ownership is
credited. The original cause/sender/true historic cleanup start remain UNKNOWN.
Prepared180 controller, inclusive START+840/900, original real cleanup design
and source performer lanes remain unchanged; no contact/duplication occurred.
The real browser fixture stays HELD. Root owns full review and any later exact
bounded-memory release. No worker/task/WT/managed shell, source/main/push/status
change or desktop occurred; RiWork Cua.ai Driver preference persists.

### Final design/static receipt

The final archived child is unchanged from the assembly above:158155B,
SHA-256 `b7bdeb6f9e24b8c537cfdca7506f570a6e2e180efa2f560dad2c6c1693aa0822`.
Its complete28019-byte logic has50 unique case names, eight observer groups
totalling50 and12 unchanged legacy groups totalling78; planned total128.
The final controller is15438B/341 lines, SHA-256
`ba20f87f0f4fe11c593ac3e5b433bde4e215e2703e0e11be3633facc78dcf0cc`.
Its literal observer-name allowlist equals the50-case name column, and its
legacy allowlist has78 names. Exactly one future Popen node is present.

Static checks actually performed, with no archived code evaluated:

- Fence extraction reconstructed normalized legacy24210B/461 lines,
  new logic28019B/576 lines and final controller15438B/341 lines, each with
  its exact stated SHA. Full old payload reconstructs ce79/91163B unchanged;
  four-line new source embedding reconstructs b7bdeb/158155B/580 lines.
  AST parse and in-memory compile of these strings passed.
- Full30530-byte prior report prefix is byte-identical to6ff5778, SHA
  `fe08d3aad7122e090dce67796cfb87fd2ed7cc7f9cecc561e13d55195308a018`.
  Current helper is byte-identical to3c9c63bb/c0c022, mode100644, with no diff.
  Removing exactly the62 inserted source lines in the seven previously
  reviewed spans reconstructs all32723B/695 lines of7fbc. Whole reversed AST
  equals that baseline AST with location attributes excluded.
- Both selected-definition ASTs compile without evaluation. All ten trusted
  own function definitions exist; main/server constructors have no call in
  the new harness. The copied writer/print/return tail is AST-identical to
  the unchanged7fbc tail, and its fresh function wrapper compiles only.
  The complete original legacy definition nodes are selected unchanged;
  the historical timer/output/exit tail is excluded by the new selector.
- `python3 scripts/check-docs.py` exited0: Markdown links and build-directory
  layout checked. `python3 scripts/check-repo-hygiene.py` exited0:
  tracked-file hygiene checked967 files. `git diff --check HEAD` exited0.
  Scope check found only this report changed and no untracked files.

Additional static/design findings are retained candidly. One final checker
assertion mistakenly used CASES column1 (group) instead of column0 (case name)
when testing name uniqueness, and exited1. Correcting that checker index
passed; neither the archived case table nor child payload changed. Inspection
also found the draft controller's review-write failure could overwrite a
previous failure tag. The final controller keeps that tag and records a
separate evidence_failure_tag, setting review_write_failed as first failure
only when no prior tag exists. This is a correction to this unexecuted design,
not a helper/source correction or evidence of the historic exception. The
earlier wrong old-payload serialization order and writer-tail wrapper findings
remain recorded above. No harness, observer, helper, selected definition,
controller, imported helper/module main, HTTPServer constructor or signal
handler executed; static tools alone ran.

No current case result, numeric child exit, elapsed bound, resource sample,
native/browser outcome or runtime-slot event exists. The30s/35s envelope is
prospective; its complete fixed result and numeric exit must first be retained
on the separately authorized run before expectations are applied. Root must
retain the controller's final returned completion sample as described above.
Historical cause, sender and true cleanup start remain UNKNOWN. Source c0c022,
prepared180/START+840 inclusive900 and the real browser HELD gate are unchanged.
One report-only commit is the handoff; root owns full review and any later
exact bounded-memory release.

## Built-in traceback descriptor — source-only correction

Date: 2026-10-02. Reservation:
`wave30_D01_builtin_traceback_descriptor`. Existing WT f2e8500e only.
This appendix preserves the entire de9bb9c report:113555B/2114 lines, SHA-256
`6659902907691170d51690c410943987a93bc2e366d363b4939ab5e72553eeaa`.
All earlier proposals, failed/static phases, archived logic and historical
results remain dated records. The de9 memory design is **UNEXECUTED and HELD**;
this source correction does not authorize its unchanged payload.

### Review read and exact source change

Read all218 lines of the immutable independent report at
`a770027d009172a96e4776aa5504500b66678520`, path
`docs/roadmap/local-wave30-d01-observer-independent-review.md`.
Its F1 witness is source-derived: ordinary exception-subclass traceback
attribute dispatch can supply a counterfeit trace with changing line accessors,
allowing an unchecked final value into the intended finite diagnostic.
No observer, subclass accessor, fake traceback or reproduction was executed.
F1 is not attributed to the earlier real request failure or original sender.

Applied exactly the root-approved line254 replacement to c0c022. Dedicated
source commit is `f4ef05d8428b235511e81277ba6b4b72d5ec08ba`, parent
`de9bb9c4dbb3184af042e606d5c14a652700e4b1`. Its only path is
`scripts/d01-confidential-browser-demo.py`, exactly one deletion/one insertion:

```diff
@@ -254 +254 @@
-        trace = error.__traceback__ if isinstance(error, BaseException) else None
+        trace = BaseException.__traceback__.__get__(error, BaseException) if isinstance(error, BaseException) else None
```

Corrected helper identity:35749B/757 lines, mode100644, Git blob
`a01b1f3f81f0978eb0a02ed339c7248cae485350`, SHA-256
`75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1`.
This explicit built-in descriptor selection reads the native traceback rather
than dispatching the exception subclass's overridden accessor. The existing
isinstance guard and else None are unchanged. Every other observer/helper byte
is exact c0c022: first latch, eight exact class identities plus other, ten code
identities, line1..1024 check,64-link scan, inner/outer BaseException catches,
private holder/evidence wiring, replies, failure precedence and original
acceptance/refusal/cleanup predicates. No import or dependency was added.

### Actual static proof and checks

The original line occurs once at254 in the pinned3c9 helper; the new line occurs
once at254 in the corrected file. Whole-file replacement equality passed.
Reversing precisely that new line reconstructs all35711 c0c022 bytes, including
the final newline. AST parse and in-memory compile passed without evaluating
the code object. The exact new assignment RHS equals the approved expression;
its guard/else AST nodes equal the old nodes. Restoring only that RHS yields
whole normalized AST equality to c0c022, with location attributes excluded.
Thus all other definitions, statements, branches and call order are unchanged.

Source-stage `python3 scripts/check-docs.py` exited0 (Markdown links and
build-directory layout checked), `git diff --check HEAD` and
`git diff --cached --check` exited0. Staged scope was exactly the helper's1/1
delta; the complete de9 report was unchanged at source commit. Entry was clean;
no untracked files or unrelated staged/unstaged paths were present. An AGENTS
file enumeration returned1 because there were no matches; ancestor/local
guidance checks found no applicable AGENTS file. No source/static proof or docs
checker failed. Final append-prefix/scope/docs/whitespace checks are performed
before the separate report-only commit and clean handoff.

### Memory and actual-fixture boundaries

The full de9 child logic/controller remain archived unchanged above, with
payload b7bdeb/158155B and controller ba20f8/15438B. Their c0c022 source pin is
historical; neither archive is a corrected-source envelope. Their custom
FakeError/accessor and fake trace/frame cases depend on the old attribute
dispatch and require a subsequent descriptor-aware design review. In
particular, their fake code/line/count and accessor-failure expectations cannot
be credited against this descriptor selection. No revised selector, case,
payload, controller, harness file or validation count is designed/materialized
by this correction. No inferred pass/fail or retrospective cause is claimed.

No observer/helper/harness import or execution, module main, server constructor,
native/provider/HTTP/network/CLI/Driver/browser/Cargo/CC operation occurred.
No runtime slot was acquired or released. Historic cause, original sender and
true cleanup start remain UNKNOWN; all actual failures and old78 results retain
their original boundaries. Prepared180/START+840 inclusive900 and the real
browser HELD gate remain unchanged. No source alignment, other path, task,
worker, WT, managed shell, main/push/status mutation or contact occurred.
RiWork Cua.ai Driver preference persists. Root separately owns corrected finite
envelope review, integration/publication and any bounded-memory/real-fixture
release; this source-only change grants no runtime or completion credit.

Final report-phase checks actually passed: whole de9 prefix equality,
corrected-source byte/AST reversal, docs checker EXIT0 and whitespace EXIT0.
The report delta is append-only and its only staged path is this report;
source f4ef05d remains exact. The post-commit handoff separately checks clean
tracked/staged/untracked scope and supplies the immutable report commit.

## Descriptor-aware finite memory envelope — DESIGN ONLY

Date: 2026-10-02. Reservation:
`wave30_D01_descriptor_aware_memory_design`. Existing WT f2e8500e only.
The entire preceding a88c2ba report remains exact119102 bytes/2207 lines,
SHA-256 `88b376467f3893a61e8f16c8209402eb7d8fee7f2b6f9b336fa46b4a75c87f9e`.
Both old design/source privacy-observation dates, every historical failure and
the complete de9 archives remain preserved. This appendix supersedes the old
custom-trace mechanics for a future review; it does not execute either design.

### Immutable inputs, complete payload and archive identities

| Role | Immutable source/identity |
| --- | --- |
| Corrected helper | `f4ef05d8428b235511e81277ba6b4b72d5ec08ba:scripts/d01-confidential-browser-demo.py`,35749B/757 lines, mode100644, SHA75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1 |
| Original legacy baseline | `7461ab50f5a5fdf0aa5d7bd6c4b309575e7a4238:scripts/d01-confidential-browser-demo.py`,31162B/671 lines, SHAac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd |
| Accepted unchanged helper | `470690cad0cd93c9f25c5bc40b982e1b91679b49:scripts/d01-confidential-browser-demo.py`,32723B/695 lines, SHA7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0 |
| Unchanged legacy logic | Earlier D01_OBSERVER_LEGACY_78_READABLE_ARCHIVE_V1 fence, normalize its one documented display space only;24210B/461 lines, SHA98b19b4d422067b0d49ea7eb1035a719d88bda672dc36579c6cf5c47cb0924eb |
| Corrected readable child logic | Literal fence below;30840B/629 lines, SHA8bf973998f0e327d9ac7fbfa7bcbd7bb3e93c8c3debe9c1a2d8251e9fd840046 |
| Corrected controller | Literal fence below;15981B/348 lines, SHA0e93e11f71f794770a60700760510f9d2893fa2bdfd044d09a5cb4c20ef555de |
| Entire serialized child payload | 161014B/633 physical lines, SHAb10cf8376edbb80b00c051305f31643b04714494260c4a49576a354361048779 |
| Complete base64 archive | 217513 ASCII bytes/2825 lines including final LF, SHAabfd936ec4445320901131449c1f3df271bf2eaf68c115f91b093709e639c53c; decoding yields exactly the entire payload above |

All source strings retain their full final LF. Exact payload assembly is four
real assignment lines in this order, followed immediately by the entire
literal corrected child-logic fence including its final LF:

```text
LEGACY_BASELINE_TEXT=<json.dumps(whole ac3 baseline text, ensure_ascii=True)>\n
UNCHANGED_TEXT=<json.dumps(whole accepted7fbc text, ensure_ascii=True)>\n
SOURCE_TEXT=<json.dumps(whole corrected75bfb8a6 text, ensure_ascii=True)>\n
LEGACY_LOGIC=<json.dumps(whole normalized98b19 legacy logic, ensure_ascii=True)>\n
<entire literal corrected child-logic fence including final LF>
```

Use Python json.dumps default separators, no indentation/sort transformation.
Angle brackets are assembly instructions, not literal bytes. The assignment
prefix is exactly130174 bytes. This whole assembly was reconstructed,
hashed, AST-parsed and compiled in memory only before any execution. The full
base64 fence is a complete byte archive, not an extra executable/helper file:
decode with base64.b64decode("".join(fence.splitlines()), validate=True), compare
the whole decoded SHA/bytes to the above assembly, then parse/compile only.
No input assignment, selected definition or archived code was evaluated.

The original ce79 payload remains91163B and SHA
`ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f`;
its historical78-case/44-baseline/80-candidate-call result remains original
evidence. Old de9 b7bdeb/158155B and ba20f8/15438B remain separately dated
UNEXECUTED/HELD records, never corrected in place. None is an actual result
of this descriptor-aware envelope.

### Truthful frame capture and changed trace mechanics

Only a future, separately authorized memory envelope calls the capture code.
After the unchanged78 legacy cases, selected_namespace defines corrected
require/observer/main and the original classes including full Demo/DemoServer.
All ten trusted own codeobjects exist. Original main is NEVER called and no
DemoServer/HTTPServer constructor runs. DemoSink keeps setup/flow work stubbed.

Capture path1 calls the selected Handler.reply body unbound once with a
memory Handler/Demo fixture, fixed200/body text, memory headers/write sink and
BudgetSink. Its one-shot MemoryWrite.write raises builtin OSError. The capture
function catches that error, gets its actual traceback using the built-in
BaseException descriptor, scans at most64 genuine native links, and takes the
actual Handler.reply frame/lasti by code identity. It never reads frame locals,
globals, filenames, raw trace text or exception text.

Capture path2 creates a distinct-but-equal reply code object using code.replace
and FunctionType, without replacing the trusted Handler.reply function/code.
It runs that clone body once with the same bounded sinks and captures its
genuine frame the same way. That actual cloned code must fail the observer's
identity check despite structural equality. Capture path3 raises/catches only
the fixed SentinelError inside capture_fixture_frame and takes that genuine
untrusted fixture frame. No fixture credential, listener, native provider,
CLI or real HTTP operation participates. These three completed capture paths
have explicit counters; they are setup witnesses, not additional padded cases.

native_trace constructs only types.TracebackType objects with those genuine
types.FrameType instances and actual captured lasti values. Every supplied
frame is authentic; no SimpleNamespace/frame-shaped object is accepted.
Controlled line annotations1/1024/0/1025 and chain lengths64/65 exercise the
existing range/cap algorithm. These annotations/chains are constructed native
tracebacks, NOT claims that the helper naturally executed at those line
numbers or recursively raised64/65 frames. The same genuine frame may occur
in multiple native links. This avoids recursive stack/resource campaigns.

Inside the active exception handler, observe installs the controlled native
chain through BaseException.__traceback__.__set__, then calls the unchanged
selected observer. It never uses the subclass getter to install/read a trace.
The old fake_trace/FakeError/FrameTrap definitions and counterfeit-accessor
branches are removed from this prospective logic only; their old archives stay.

### Planned coverage and exact unchanged guards

| Observer group | Planned named cases |
| --- | ---: |
| Exact builtin class identities, two subclasses, private other |11|
| Native code identity/clone/untrusted, line1/1024/0/1025,64/65 cap |8|
| Hostile changing/private and raising traceback getters |2|
| Holder read/latch/storage failure, invalid site, no active error |5|
| First latch and second reply/server failure |2|
| Parser/dispatch/reply/send_error/server/main-except unexpected sites |6|
| Known Failure/Halt/success with no observation |3|
| Missing observer/field, outer BaseException guards, legacy fallback |10|
| str/repr traps and private writer/public stdout success/failure |3|
| Observer named-case total |50|
| Original legacy guard/sink named cases |78|
| Total planned named cases |128|

There is no target-count padding.48 old case names/arguments remain; the two
obsolete fault_trace_accessor/fault_frame_accessor cases are replaced by two
meaningful descriptor_changing_getter/descriptor_raising_getter cases.
Native capture setup has its separate native_capture failure label and three
counters. All50 case descriptors and their group counts are literal and
statically counted before any execution. The count coincidentally remains50.

The changing subclass getter would alternate private SENTINEL/native trace if
called; the other getter would raise a PrivateTrap carrying the fake sentinel.
Both must be NEVER invoked: the traceback-access counter stays0, str/repr
counters stay0, the result uses the actual native Handler.reply frame and the
fixed other class label, and no sentinel enters the checked holder or result.
This is a future assertion, not an executed bypass/privacy proof.

Original legacy run/verify/load_selected/make_demo/request/guard/sink definition
nodes and every original assertion remain exact98b19 logic. Their original
import/timer/output/exit tail is not evaluated. The new envelope supplies the
corrected helper as SOURCE_TEXT/SOURCE_SHA and the same ac3 baseline; it expects
exact44/80 legacy handler calls, one injected403 write failure and one response
timeout. That legacy selector still omits the observer. Neither historical78
results nor the reused legacy cases establish observer coverage.

Twenty-four shared top-level definition ASTs are unchanged from de9, including
class_case, site/fallback/known/privacy cases, memory handler/write sinks,
selectors and exact writer-tail extraction. Only observe, trace_case,
fault_case and first_case have changed bodies: native attachment/boundaries,
removal of impossible custom native-frame failures, and native first-latch
mechanics. Existing actual handler outcome comparisons, terminal/failure/reply
assertions, legacy normalized snapshots and first failure remain. Main is
defined but only its generic-except body/writer tail is copied into controlled
memory execution in the future, never called as main. Server methods are
called unbound with SimpleNamespace, never through a server constructor.

### Future envelope, evidence-before-oracles and limits

Child30s SIGALRM, controller35s monotonic budget/33s communicate stop/remaining
owned kill-reap, one child, first-failure stop and exclusive private0600
captures/receipts in a fresh0700 directory remain unchanged. Child captures
have child-only16KiB RLIMIT_FSIZE; its fixed public result is capped8192B.
Raw stdout/stderr captures stay private, bounded and unprinted; no raw error,
trace, path, URL/header/query/cookie/Origin/sender, locals or protocol is added.
No cleanup touches an unrelated process, directory or cache.

The complete structurally safe v2 fixed JSON and actual numeric child exit are
saved and fsynced to d01-memory-retained.json BEFORE success/exit/count/timing/
source-pin comparisons. Structural validation enforces closed keys/enums/names,
finite numeric fields and64-lowercase-hex hash shape; expected hash equality now
happens AFTER the retained save. Thus a structurally valid wrong-pin/failing
child receipt is retained before rejecting it. Missing/malformed output remains
unavailable, never a fabricated complete child result. Separate reviewed JSON
preserves first failure, cleanup failures and the earlier review-write handling.

Changes to the controller are limited to new payload/receipt version, case/
group names, native-capture and getter counters, native_capture failure label,
corrected helper pin and moving expected hash equality after retention.
Private captures/permissions/caps, command, owned reap, first-failure handling,
capture hashes, elapsed samples and absence of directory/cache deletion stay.
The caller must retain the final returned post-save timing sample too; the
saved review is an earlier sample. OS/Popen/filesystem scheduling stalls are
not claimed impossible. Root's later exact invocation must select its fresh
directory/interpreter/environment/resource preflight and prove its whole bound.
No current timing/resource/capacity or cleanup proof is invented.

### Complete corrected readable child logic

Marker: D01_DESCRIPTOR_MEMORY_LOGIC_V2.
This literal code is archived only, never executed or imported here.

```python
import ast, contextlib, copy, hashlib, http.server, io, json, re, signal, sys, time, urllib.parse
from types import SimpleNamespace, FrameType, TracebackType, FunctionType

SENTINEL = "D01_PRIVATE_MEMORY_SENTINEL"
EXPECTED_LEGACY = {"record_copy": 1, "bounded_sequence": 4, "authorization_presence": 6,
    "postflow": 7, "header_free_routes_guards": 25, "header_free_bounds": 12,
    "authorization_bounds": 12, "controlled_parser_result": 2, "parser_header_limits": 5,
    "counter_persistence": 1, "first_reason": 1, "response_failures": 2}
EXPECTED_OBSERVER = {"classes": 11, "trace": 8, "faults": 5, "descriptor": 2,
    "first": 2, "sites": 6, "known": 3, "fallback": 10, "privacy": 3}
SOURCE_HASH = "75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1"
UNCHANGED_HASH = "7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0"
BASELINE_HASH = "ac3c350d0ea5f1c7448d533524e5db2f99ee37ad61be313c6f36e0c4025a2fcd"
LEGACY_LOGIC_HASH = "98b19b4d422067b0d49ea7eb1035a719d88bda672dc36579c6cf5c47cb0924eb"

class EnvelopeDeadline(BaseException):
    pass

class PrivateTrap(BaseException):
    pass

class GuardFailure(BaseException):
    pass

class SentinelError(Exception):
    def __str__(self):
        trap_counts["str"] += 1
        raise PrivateTrap
    def __repr__(self):
        trap_counts["repr"] += 1
        raise PrivateTrap

class ValueSubclass(ValueError):
    pass

class OSSubclass(OSError):
    pass

class ChangingTraceback(SentinelError):
    def __init__(self, native):
        self.native = native
        super().__init__(SENTINEL)
    def __getattribute__(self, name):
        if name == "__traceback__":
            trap_counts["traceback"] += 1
            return SENTINEL if trap_counts["traceback"] % 2 else object.__getattribute__(self, "native")
        return object.__getattribute__(self, name)

class RaisingTraceback(SentinelError):
    def __getattribute__(self, name):
        if name == "__traceback__":
            trap_counts["traceback"] += 1
            raise PrivateTrap(SENTINEL)
        return object.__getattribute__(self, name)

class FaultHolder(dict):
    def __init__(self, mode):
        super().__init__(observed=False, diagnostic=None)
        self.mode = mode
    def __getitem__(self, key):
        if self.mode == "read" and key == "observed":
            raise PrivateTrap
        return dict.__getitem__(self, key)
    def __setitem__(self, key, value):
        if self.mode == key:
            raise PrivateTrap
        return dict.__setitem__(self, key, value)

def empty_holder():
    return {"observed": False, "diagnostic": None}

def legacy_namespace():
    tree = ast.parse(LEGACY_LOGIC)
    definitions = [n for n in tree.body if isinstance(n, (ast.FunctionDef, ast.ClassDef))]
    namespace = {"ast": ast, "contextlib": contextlib, "copy": copy, "hashlib": hashlib,
        "http": http, "io": io, "json": json, "re": re, "signal": signal, "time": time,
        "urllib": urllib, "SimpleNamespace": SimpleNamespace,
        "SOURCE_TEXT": SOURCE_TEXT, "BASELINE_TEXT": LEGACY_BASELINE_TEXT,
        "SOURCE_SHA": SOURCE_HASH, "BASE_COMMIT": "7461ab50f5a5fdf0aa5d7bd6c4b309575e7a4238",
        "groups": {}, "completed": 0, "case": "source_pin",
        "handler_calls": {"baseline": 0, "candidate": 0},
        "expected_write_failures": 0, "expected_timeouts": 0}
    # Original function/class nodes are unmodified; no original top-level tail runs.
    exec(compile(ast.Module(body=definitions, type_ignores=[]), "<legacy-definitions>", "exec"), namespace)
    return namespace

def selected_namespace(text):
    tree = ast.parse(text)
    classes = {"Failure", "Halt", "PreflowAuthorizationRefusal", "HeaderReader",
               "Demo", "DemoServer", "Handler"}
    functions = {"require", "observe_unexpected_failure", "main"}
    selected = [n for n in tree.body if isinstance(n, ast.Assign)]
    assert all(not any(isinstance(x, ast.Call) for x in ast.walk(n)) for n in selected)
    selected += [n for n in tree.body if
                 isinstance(n, ast.ClassDef) and n.name in classes or
                 isinstance(n, ast.FunctionDef) and n.name in functions]
    namespace = {"http": http, "contextlib": contextlib, "hashlib": hashlib, "re": re,
                 "sys": sys, "time": time, "urllib": urllib}
    exec(compile(ast.Module(body=selected, type_ignores=[]), "<observer-definitions>", "exec"), namespace)
    namespace["source_ast"] = tree
    # All ten trusted codeobjects exist. Neither main nor a server constructor is called.
    codes = (namespace["HeaderReader"].readline.__code__,
        namespace["DemoServer"].process_request.__code__,
        namespace["Demo"].begin.__code__, namespace["Demo"].callback.__code__,
        namespace["Demo"].invoke.__code__, namespace["Handler"].handle_one_request.__code__,
        namespace["Handler"].send_error.__code__, namespace["Handler"].get.__code__,
        namespace["Handler"].reply.__code__, namespace["main"].__code__)
    assert len(codes) == 10
    return namespace

def new_demo(namespace):
    return legacy["make_demo"] (namespace)

def holder_of(demo):
    return demo.unexpected_failure_observation

def checked_holder(holder):
    value = {key: value for key, value in dict.items(holder)}
    assert set(value) == {"observed", "diagnostic"} and type(value["observed"]) is bool
    diagnostic = value["diagnostic"]
    if diagnostic is not None:
        assert value["observed"] is True
        assert set(diagnostic) == {"site", "exception_class", "own_function", "own_line"}
        assert diagnostic["site"] in {"handler", "server", "main"}
        assert diagnostic["exception_class"] in {"AttributeError", "TypeError", "ValueError",
            "KeyError", "OSError", "BrokenPipeError", "ConnectionResetError", "TimeoutError", "other"}
        assert diagnostic["own_function"] in {None, "HeaderReader.readline",
            "DemoServer.process_request", "Demo.begin", "Demo.callback", "Demo.invoke",
            "Handler.handle_one_request", "Handler.send_error", "Handler.get", "Handler.reply", "main"}
        assert (diagnostic["own_function"] is None) == (diagnostic["own_line"] is None)
        assert diagnostic["own_line"] is None or type(diagnostic["own_line"]) is int and 1 <= diagnostic["own_line"] <= 1024
    raw = json.dumps(value, sort_keys=True).encode("ascii")
    assert len(raw) <= 160 and SENTINEL.encode() not in raw
    return value

def observe(namespace, error, holder=None, site="handler", native=None):
    holder = empty_holder() if holder is None else holder
    try:
        raise error
    except BaseException:
        if native is not None:
            assert type(native) is TracebackType
            BaseException.__traceback__.__set__(error, native)
        namespace["observe_unexpected_failure"] (holder, site)
    return checked_holder(holder)

def capture_reply_frame(namespace, cloned=False):
    original = namespace["Handler"].reply
    function = original
    if cloned:
        function = FunctionType(original.__code__.replace(), original.__globals__,
                                "untrusted_reply_clone", original.__defaults__)
        function.__kwdefaults__ = dict(original.__kwdefaults__)
        assert function.__code__ == original.__code__ and function.__code__ is not original.__code__
    handler = memory_handler(namespace, new_demo(namespace))
    handler.wfile.once = True
    try:
        # Only this response body runs; all I/O and budget operations are memory sinks.
        function(handler, 200, "memory frame capture")
    except OSError as error:
        trace = BaseException.__traceback__.__get__(error, BaseException)
        for _ in range(64):
            if trace is None:
                break
            assert type(trace) is TracebackType and type(trace.tb_frame) is FrameType
            if trace.tb_frame.f_code is function.__code__:
                return trace.tb_frame, trace.tb_lasti
            trace = trace.tb_next
    raise AssertionError

def capture_fixture_frame():
    try:
        raise SentinelError(SENTINEL)
    except SentinelError as error:
        trace = BaseException.__traceback__.__get__(error, BaseException)
        assert type(trace) is TracebackType and type(trace.tb_frame) is FrameType
        assert trace.tb_frame.f_code is capture_fixture_frame.__code__ and trace.tb_next is None
        return trace.tb_frame, trace.tb_lasti

def native_trace(line, count=1, trusted=True, cloned=False):
    key = "cloned" if cloned else "trusted" if trusted else "untrusted"
    frame, lasti = native_frames[key]
    assert type(frame) is FrameType and type(lasti) is int
    trace = None
    for _ in range(count):
        # Native traceback with a genuine captured frame and a controlled line annotation.
        trace = TracebackType(trace, frame, lasti, line)
        assert type(trace) is TracebackType
    return trace

def class_case(name):
    classes = {"AttributeError": AttributeError, "TypeError": TypeError, "ValueError": ValueError,
        "KeyError": KeyError, "OSError": OSError, "BrokenPipeError": BrokenPipeError,
        "ConnectionResetError": ConnectionResetError, "TimeoutError": TimeoutError,
        "ValueSubclass": ValueSubclass, "OSSubclass": OSSubclass, "SentinelError": SentinelError}
    error = classes[name] (SENTINEL)
    before = dict(trap_counts)
    value = observe(aware, error)
    assert value["observed"] and value["diagnostic"]["exception_class"] == (name if name in {
        "AttributeError", "TypeError", "ValueError", "KeyError", "OSError",
        "BrokenPipeError", "ConnectionResetError", "TimeoutError"} else "other")
    assert value["diagnostic"]["own_function"] is None
    assert trap_counts == before

def trace_case(kind):
    options = {"line1": (1, 1, True, False), "line1024": (1024, 1, True, False),
        "line0": (0, 1, True, False), "line1025": (1025, 1, True, False),
        "links64": (512, 64, True, False), "links65": (512, 65, True, False),
        "untrusted": (512, 1, False, False), "cloned": (512, 1, True, True)}
    line, count, trusted, cloned = options[kind]
    value = observe(aware, SentinelError(SENTINEL), native=native_trace(line, count, trusted, cloned))
    valid = line in (1, 1024, 512) and count <= 64 and trusted and not cloned
    assert value["diagnostic"]["own_function"] == ("Handler.reply" if valid else None)
    assert value["diagnostic"]["own_line"] == (line if valid else None)

def descriptor_case(kind):
    native = native_trace(1)
    error = ChangingTraceback(native) if kind == "changing" else RaisingTraceback(SENTINEL)
    before = dict(trap_counts)
    value = observe(aware, error, native=native)
    assert value == {"observed": True, "diagnostic": {"site": "handler",
        "exception_class": "other", "own_function": "Handler.reply", "own_line": 1}}
    assert trap_counts == before and trap_counts["traceback"] == 0
    assert SENTINEL not in json.dumps(value, sort_keys=True)

def fault_case(kind):
    if kind in ("read", "observed", "diagnostic"):
        holder = FaultHolder(kind)
        value = observe(aware, SentinelError(SENTINEL), holder)
        assert value == {"observed": kind == "diagnostic", "diagnostic": None}
        if kind == "diagnostic":
            holder.mode = None
            assert observe(aware, ValueError(SENTINEL), holder) == value
    elif kind == "invalid_site":
        assert observe(aware, ValueError(SENTINEL), site=SENTINEL) == empty_holder()
    else:
        holder = empty_holder()
        aware["observe_unexpected_failure"] (holder, "server")
        assert checked_holder(holder) == {"observed": True, "diagnostic": {
            "site": "server", "exception_class": "other", "own_function": None, "own_line": None}}

class MemoryWrite(io.BytesIO):
    def __init__(self, once=False):
        super().__init__()
        self.once = once
    def write(self, raw):
        if self.once:
            self.once = False
            raise OSError(SENTINEL)
        return super().write(raw)

def memory_handler(namespace, demo, fault=None):
    class Sink(namespace["Handler"]):
        def parse_request(self):
            if fault == "parser":
                raise ValueError(SENTINEL)
            if fault == "send_error":
                self.send_error(400)
                return False
            return super().parse_request()
        def record_request_reason(self, reason):
            if fault == "send_error" and reason == "http_parse":
                raise TypeError(SENTINEL)
            return super().record_request_reason(reason)
        def cookies(self):
            demo.calls["cookies"] += 1
            if fault == "dispatch":
                raise TypeError(SENTINEL)
            return super().cookies()
        def get(self, target):
            demo.calls["get"] += 1
            return super().get(target)
        def reply(self, status, text, **options):
            self.responses.append((status, tuple(sorted(options))))
            return super().reply(status, text, **options)
        def send_response(self, status, message=None):
            self.sent_statuses.append(status)
        def send_header(self, name, value):
            self.sent_headers.append((name, value))
        def end_headers(self):
            self.header_ends += 1
    handler = object.__new__(Sink)
    handler.server = SimpleNamespace(demo=demo)
    handler.rfile = io.BytesIO(legacy["build_wire"] (namespace))
    handler.wfile = MemoryWrite(once=fault == "reply")
    handler.responses, handler.sent_statuses, handler.sent_headers = [], [], []
    handler.header_ends = 0
    return handler

def main_handler(namespace, record, demo):
    function = next(n for n in namespace["source_ast"].body if isinstance(n, ast.FunctionDef) and n.name == "main")
    block = next(n for n in function.body if isinstance(n, ast.Try))
    handler = next(n for n in block.handlers if isinstance(n.type, ast.Name) and n.type.id == "Exception")
    environment = dict(namespace, record=record, demo=demo, stage="arguments")
    try:
        raise ValueError(SENTINEL)
    except Exception:
        exec(compile(ast.Module(body=copy.deepcopy(handler.body), type_ignores=[]),
                     "<main-except-only>", "exec"), environment)

def site_outcome(namespace, site):
    demo = new_demo(namespace)
    demo.stage = "request"
    if site in {"parser", "dispatch", "reply", "send_error"}:
        handler = memory_handler(namespace, demo, site)
        handler.handle_one_request()
        return legacy["normalized"] (namespace, demo, handler), demo
    if site == "server":
        events = []
        def finish(request, address):
            events.append("finish")
            raise KeyError(SENTINEL)
        def shutdown(request):
            events.append("shutdown")
        server = SimpleNamespace(demo=demo, active=None, finish_request=finish, shutdown_request=shutdown)
        try:
            namespace["DemoServer"].process_request(server, None, None)
        except Exception:
            namespace["DemoServer"].handle_error(server, None, None)
        assert server.active is None and events == ["finish", "shutdown"]
        return (legacy["flow_snapshot"] (demo), demo.stage, demo.calls, events), demo
    record = {"failure_tag": None, "failure_stage": None,
              "unexpected_failure_observation": holder_of(demo) if hasattr(demo, "unexpected_failure_observation") else empty_holder()}
    main_handler(namespace, record, demo)
    assert record["failure_tag"] == "unexpected_failure" and record["failure_stage"] == "request"
    return {k: v for k, v in record.items() if k != "unexpected_failure_observation"}, demo

def site_case(site):
    before, ignored = site_outcome(reference, site)
    after, demo = site_outcome(aware, site)
    assert before == after
    value = checked_holder(holder_of(demo))
    labels = {"parser": ("handler", "ValueError", "Handler.handle_one_request"),
        "dispatch": ("handler", "TypeError", "Handler.get"),
        "reply": ("handler", "OSError", "Handler.reply"),
        "send_error": ("handler", "TypeError", "Handler.send_error"),
        "server": ("server", "KeyError", "DemoServer.process_request"),
        "main": ("main", "ValueError", None)}
    site_label, class_label, own = labels[site]
    assert value["observed"] and value["diagnostic"]["site"] == site_label
    assert value["diagnostic"]["exception_class"] == class_label
    assert value["diagnostic"]["own_function"] == own

def first_case(kind):
    if kind == "direct":
        holder = empty_holder()
        first = observe(aware, SentinelError(SENTINEL), holder, native=native_trace(1))
        assert observe(aware, ValueError(SENTINEL), holder, "server") == first
        return
    outcomes = []
    for namespace in (reference, aware):
        demo = new_demo(namespace)
        handler = memory_handler(namespace, demo, "parser")
        handler.wfile.once = True
        try:
            handler.handle_one_request()
        except OSError:
            # The original reply failure escapes; the existing server sink runs.
            namespace["DemoServer"].handle_error(SimpleNamespace(demo=demo), None, None)
        else:
            raise AssertionError
        assert demo.failure == "unexpected_failure" and demo.done
        if namespace is aware:
            value = checked_holder(holder_of(demo))
            assert value["diagnostic"]["exception_class"] == "ValueError"
            assert value["diagnostic"]["site"] == "handler"
        outcomes.append(legacy["normalized"] (namespace, demo, handler))
    assert outcomes[0] == outcomes[1]

def known_case(kind):
    outcomes = []
    for namespace in (reference, aware):
        demo = new_demo(namespace)
        handler = memory_handler(namespace, demo)
        if kind == "failure":
            handler.rfile = io.BytesIO(legacy["build_wire"] (namespace, host="wrong"))
        elif kind == "halt":
            demo.budget.fail_response = True
        try:
            handler.handle_one_request()
        except namespace["Halt"] as error:
            assert kind == "halt" and error.tag == "response_timeout"
        else:
            assert kind != "halt"
        if namespace is aware:
            assert checked_holder(holder_of(demo)) == empty_holder()
        outcomes.append(legacy["normalized"] (namespace, demo, handler))
    assert outcomes[0] == outcomes[1]

def fallback_case(mode, site):
    if mode == "legacy":
        namespace = legacy["load_selected"] (SOURCE_TEXT, "candidate")
        demo = legacy["make_demo"] (namespace)
        del demo.unexpected_failure_observation
        namespace["handle_error"] (SimpleNamespace(demo=demo), None, None)
        assert demo.failure == "unexpected_failure" and demo.done
        assert "observe_unexpected_failure" not in namespace
        return
    namespace = selected_namespace(SOURCE_TEXT)
    if mode == "absent":
        del namespace["observe_unexpected_failure"]
    elif mode == "guard":
        def fail(*args):
            raise GuardFailure
        namespace["observe_unexpected_failure"] = fail
    demo = new_demo(namespace)
    demo.stage = "request"
    if mode == "field":
        del demo.unexpected_failure_observation
    if site == "handler":
        handler = memory_handler(namespace, demo, "parser")
        handler.handle_one_request()
        assert demo.failure == "unexpected_failure" and demo.done
        assert handler.responses == [(400, ())]
    elif site == "server":
        namespace["DemoServer"].handle_error(SimpleNamespace(demo=demo), None, None)
        assert demo.failure == "unexpected_failure" and demo.done
    else:
        record = {"failure_tag": None, "failure_stage": None}
        if mode != "field":
            record["unexpected_failure_observation"] = holder_of(demo)
        main_handler(namespace, record, demo)
        assert record["failure_tag"] == "unexpected_failure" and record["failure_stage"] == "request"
    if mode != "field":
        assert checked_holder(holder_of(demo)) == empty_holder()

def writer_trial(namespace, fail):
    tree = namespace["source_ast"]
    main = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "main")
    start = next(i for i, n in enumerate(main.body) if isinstance(n, ast.Assign)
                 and any(isinstance(t, ast.Name) and t.id == "written" for t in n.targets))
    # Preserve the exact writer/print/return tail inside a fresh wrapper, never main.
    wrapper = copy.deepcopy(main)
    wrapper.name = "selected_evidence_tail"
    wrapper.body = copy.deepcopy(main.body[start:])
    sink = io.BytesIO()
    class Writer:
        def __enter__(self):
            return self
        def __exit__(self, *args):
            return False
        def write(self, raw):
            if fail:
                raise OSError(SENTINEL)
            return sink.write(raw)
        def flush(self):
            pass
        def fileno(self):
            return 1
    stdout = io.StringIO()
    def safe_print(raw, **options):
        stdout.write(raw + "\n")
    holder = empty_holder()
    if "observe_unexpected_failure" in namespace:
        observe(namespace, SentinelError(SENTINEL), holder)
    record = {"result": "failed", "failure_tag": "unexpected_failure",
              "failure_stage": "request", "cleanup": {},
              "unexpected_failure_observation": holder}
    environment = dict(namespace, os=SimpleNamespace(fdopen=lambda *a: Writer(),
        fsync=lambda *a: None, close=lambda *a: None), json=json, print=safe_print,
        evidence_fd=1, record=record)
    exec(compile(ast.Module(body=[wrapper], type_ignores=[]), "<evidence-tail-only>", "exec"), environment)
    assert environment["selected_evidence_tail"] () == 1
    public = json.loads(stdout.getvalue())
    assert set(public) == {"result", "failure_tag", "cleanup_failure_tag",
                          "evidence_failure_tag", "evidence_written"}
    assert "unexpected_failure_observation" not in public
    assert SENTINEL not in stdout.getvalue() and SENTINEL.encode() not in sink.getvalue()
    assert public["failure_tag"] == "unexpected_failure" and public["result"] == "failed"
    assert public["evidence_written"] is (not fail)
    if fail:
        assert public["evidence_failure_tag"] == "evidence_write_failed"
        assert record["failure_stage"] == "request"
    else:
        assert checked_holder(json.loads(sink.getvalue())["unexpected_failure_observation"]) == holder
    return public

def privacy_case(kind):
    before = dict(trap_counts)
    if kind == "sentinel":
        value = observe(aware, SentinelError(SENTINEL))
        assert value["diagnostic"]["exception_class"] == "other"
    else:
        assert writer_trial(reference, kind == "writer_failure") == writer_trial(aware, kind == "writer_failure")
    assert trap_counts == before

CASES = (
    ("class_attribute", "classes", "class", ("AttributeError",)),
    ("class_type", "classes", "class", ("TypeError",)),
    ("class_value", "classes", "class", ("ValueError",)),
    ("class_key", "classes", "class", ("KeyError",)),
    ("class_os", "classes", "class", ("OSError",)),
    ("class_broken_pipe", "classes", "class", ("BrokenPipeError",)),
    ("class_connection_reset", "classes", "class", ("ConnectionResetError",)),
    ("class_timeout", "classes", "class", ("TimeoutError",)),
    ("class_value_subclass", "classes", "class", ("ValueSubclass",)),
    ("class_os_subclass", "classes", "class", ("OSSubclass",)),
    ("class_private_other", "classes", "class", ("SentinelError",)),
    ("trace_line1", "trace", "trace", ("line1",)),
    ("trace_line1024", "trace", "trace", ("line1024",)),
    ("trace_line0", "trace", "trace", ("line0",)),
    ("trace_line1025", "trace", "trace", ("line1025",)),
    ("trace_64", "trace", "trace", ("links64",)),
    ("trace_65", "trace", "trace", ("links65",)),
    ("trace_untrusted", "trace", "trace", ("untrusted",)),
    ("trace_equal_clone", "trace", "trace", ("cloned",)),
    ("descriptor_changing_getter", "descriptor", "descriptor", ("changing",)),
    ("descriptor_raising_getter", "descriptor", "descriptor", ("raising",)),
    ("fault_holder_read", "faults", "fault", ("read",)),
    ("fault_latch_write", "faults", "fault", ("observed",)),
    ("fault_diagnostic_write", "faults", "fault", ("diagnostic",)),
    ("fault_invalid_site", "faults", "fault", ("invalid_site",)),
    ("fault_no_active_exception", "faults", "fault", ("no_exception",)),
    ("first_direct", "first", "first", ("direct",)),
    ("first_second_reply_failure", "first", "first", ("reply",)),
    ("site_parser", "sites", "site", ("parser",)),
    ("site_dispatch", "sites", "site", ("dispatch",)),
    ("site_reply", "sites", "site", ("reply",)),
    ("site_send_error", "sites", "site", ("send_error",)),
    ("site_server", "sites", "site", ("server",)),
    ("site_main_block", "sites", "site", ("main",)),
    ("known_failure", "known", "known", ("failure",)),
    ("known_halt", "known", "known", ("halt",)),
    ("known_success", "known", "known", ("success",)),
    ("fallback_absent_handler", "fallback", "fallback", ("absent", "handler")),
    ("fallback_absent_server", "fallback", "fallback", ("absent", "server")),
    ("fallback_absent_main", "fallback", "fallback", ("absent", "main")),
    ("fallback_field_handler", "fallback", "fallback", ("field", "handler")),
    ("fallback_field_server", "fallback", "fallback", ("field", "server")),
    ("fallback_field_main", "fallback", "fallback", ("field", "main")),
    ("fallback_baseexception_handler", "fallback", "fallback", ("guard", "handler")),
    ("fallback_baseexception_server", "fallback", "fallback", ("guard", "server")),
    ("fallback_baseexception_main", "fallback", "fallback", ("guard", "main")),
    ("fallback_legacy_selector_field", "fallback", "fallback", ("legacy", "server")),
    ("privacy_str_repr", "privacy", "privacy", ("sentinel",)),
    ("privacy_writer_success", "privacy", "privacy", ("writer_success",)),
    ("privacy_writer_failure", "privacy", "privacy", ("writer_failure",)),
)
RUNNERS = {"class": class_case, "trace": trace_case, "descriptor": descriptor_case, "fault": fault_case,
    "first": first_case, "site": site_case, "known": known_case,
    "fallback": fallback_case, "privacy": privacy_case}

started = time.monotonic()
legacy = None
observer_completed = 0
observer_groups = {key: 0 for key in EXPECTED_OBSERVER}
trap_counts = {"str": 0, "repr": 0, "traceback": 0}
native_frames = {}
native_captures = {"trusted": 0, "cloned": 0, "untrusted": 0}
case = "source_archive"
failure = None
result = "failed"

def deadline(signum, frame):
    raise EnvelopeDeadline

try:
    signal.signal(signal.SIGALRM, deadline)
    signal.setitimer(signal.ITIMER_REAL, 30)
    assert hashlib.sha256(SOURCE_TEXT.encode()).hexdigest() == SOURCE_HASH
    assert hashlib.sha256(UNCHANGED_TEXT.encode()).hexdigest() == UNCHANGED_HASH
    assert hashlib.sha256(LEGACY_BASELINE_TEXT.encode()).hexdigest() == BASELINE_HASH
    assert hashlib.sha256(LEGACY_LOGIC.encode()).hexdigest() == LEGACY_LOGIC_HASH
    old_archive = ("BASELINE_TEXT=" + json.dumps(LEGACY_BASELINE_TEXT, ensure_ascii=True) + "\n"
                   + "SOURCE_TEXT=" + json.dumps(UNCHANGED_TEXT, ensure_ascii=True) + "\n" + LEGACY_LOGIC)
    assert len(old_archive.encode()) == 91163
    assert hashlib.sha256(old_archive.encode()).hexdigest() == "ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f"
    assert len(CASES) == 50 and len({n for n, g, k, a in CASES}) == 50
    planned = {key: sum(group == key for name, group, kind, args in CASES) for key in EXPECTED_OBSERVER}
    assert planned == EXPECTED_OBSERVER
    legacy = legacy_namespace()
    case = "legacy_78"
    legacy["run"] ()
    assert legacy["completed"] == 78 and legacy["groups"] == EXPECTED_LEGACY
    assert legacy["handler_calls"] == {"baseline": 44, "candidate": 80}
    assert legacy["expected_write_failures"] == 1 and legacy["expected_timeouts"] == 1
    case = "observer_selector"
    reference = selected_namespace(UNCHANGED_TEXT)
    aware = selected_namespace(SOURCE_TEXT)
    case = "native_capture"
    native_frames["trusted"] = capture_reply_frame(aware)
    native_captures["trusted"] += 1
    native_frames["cloned"] = capture_reply_frame(aware, cloned=True)
    native_captures["cloned"] += 1
    native_frames["untrusted"] = capture_fixture_frame()
    native_captures["untrusted"] += 1
    for name, group, kind, args in CASES:
        case = name
        RUNNERS[kind] (*args)
        observer_completed += 1
        observer_groups[group] += 1
    case = "observer_totals"
    assert observer_completed == 50 and observer_groups == EXPECTED_OBSERVER
    assert trap_counts == {"str": 0, "repr": 0, "traceback": 0}
    assert native_captures == {"trusted": 1, "cloned": 1, "untrusted": 1}
    result = "passed"
except EnvelopeDeadline:
    failure = "verification_deadline"
except AssertionError:
    failure = "assertion_failed"
except BaseException:
    failure = "harness_exception"
finally:
    signal.setitimer(signal.ITIMER_REAL, 0)

legacy_completed = 0 if legacy is None else legacy["completed"]
legacy_groups = {key: 0 if legacy is None else legacy["groups"].get(key, 0) for key in EXPECTED_LEGACY}
failed_case = None if failure is None else (
    "legacy_" + legacy["case"] if case == "legacy_78" and legacy is not None else case)
receipt = {"schema": "riauth.d01-observer-memory/v2", "result": result,
    "failure_tag": failure, "failed_case": failed_case, "planned_cases": 128,
    "legacy_planned": 78, "observer_planned": 50,
    "legacy_completed": legacy_completed, "observer_completed": observer_completed,
    "completed_cases": legacy_completed + observer_completed,
    "legacy_groups": legacy_groups, "observer_groups": observer_groups,
    "legacy_handler_calls": {"baseline": 0, "candidate": 0} if legacy is None else legacy["handler_calls"],
    "expected_write_failures": 0 if legacy is None else legacy["expected_write_failures"],
    "expected_response_timeouts": 0 if legacy is None else legacy["expected_timeouts"],
    "private_format_traps": trap_counts, "native_captures": native_captures,
    "elapsed_seconds": round(time.monotonic() - started, 6),
    "deadline_seconds": 30, "helper_sha256": SOURCE_HASH, "unchanged_sha256": UNCHANGED_HASH,
    "legacy_archive_sha256": "ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f",
    "main_invoked": False, "server_constructed": False,
    "network_listener_native_provider_cli_driver_browser_cargo": False}
raw = (json.dumps(receipt, sort_keys=True) + "\n").encode("ascii")
assert len(raw) <= 8192 and SENTINEL.encode() not in raw
sys.stdout.buffer.write(raw)
sys.stdout.buffer.flush()
raise SystemExit(0 if result == "passed" else 1)
```

### Complete corrected readable controller

Marker: D01_DESCRIPTOR_MEMORY_CONTROLLER_V2.
This literal source defines a future API only; no call is made here.

```python
import hashlib, json, math, os, pathlib, resource, stat, subprocess, sys, time

PAYLOAD_SHA256 = "b10cf8376edbb80b00c051305f31643b04714494260c4a49576a354361048779"
CAP = 16384
PLANNED_LEGACY = {"record_copy": 1, "bounded_sequence": 4, "authorization_presence": 6,
    "postflow": 7, "header_free_routes_guards": 25, "header_free_bounds": 12,
    "authorization_bounds": 12, "controlled_parser_result": 2, "parser_header_limits": 5,
    "counter_persistence": 1, "first_reason": 1, "response_failures": 2}
PLANNED_OBSERVER = {"classes": 11, "trace": 8, "faults": 5, "descriptor": 2,
    "first": 2, "sites": 6, "known": 3, "fallback": 10, "privacy": 3}

OBSERVER_NAMES = (
    "class_attribute",
    "class_type",
    "class_value",
    "class_key",
    "class_os",
    "class_broken_pipe",
    "class_connection_reset",
    "class_timeout",
    "class_value_subclass",
    "class_os_subclass",
    "class_private_other",
    "trace_line1",
    "trace_line1024",
    "trace_line0",
    "trace_line1025",
    "trace_64",
    "trace_65",
    "trace_untrusted",
    "trace_equal_clone",
    "descriptor_changing_getter",
    "descriptor_raising_getter",
    "fault_holder_read",
    "fault_latch_write",
    "fault_diagnostic_write",
    "fault_invalid_site",
    "fault_no_active_exception",
    "first_direct",
    "first_second_reply_failure",
    "site_parser",
    "site_dispatch",
    "site_reply",
    "site_send_error",
    "site_server",
    "site_main_block",
    "known_failure",
    "known_halt",
    "known_success",
    "fallback_absent_handler",
    "fallback_absent_server",
    "fallback_absent_main",
    "fallback_field_handler",
    "fallback_field_server",
    "fallback_field_main",
    "fallback_baseexception_handler",
    "fallback_baseexception_server",
    "fallback_baseexception_main",
    "fallback_legacy_selector_field",
    "privacy_str_repr",
    "privacy_writer_success",
    "privacy_writer_failure",
)

LEGACY_NAMES = (
    "initial_count_copy",
    "bounded_refusal_1",
    "bounded_refusal_2",
    "bounded_refusal_3",
    "bounded_refusal_4",
    "presence_empty",
    "presence_duplicate",
    "presence_post",
    "presence_protected",
    "presence_callback",
    "presence_other_method",
    "strict_state_attempted_1",
    "strict_state_pending_2",
    "strict_state_cookie_3",
    "strict_state_subject_4",
    "strict_state_attempted_5",
    "strict_state_attempted_6",
    "defensive_exhaustion",
    "header_free_root",
    "header_free_protected_before",
    "header_free_unavailable",
    "header_free_root_query",
    "header_free_protected_query",
    "header_free_callback_empty",
    "header_free_callback_ready",
    "header_free_login",
    "header_free_login_repeated",
    "header_free_protected_after",
    "header_free_protected_before_missing",
    "header_free_protected_cookie_mismatch",
    "header_free_method",
    "header_free_post_target",
    "header_free_post_query",
    "header_free_origin_missing",
    "header_free_origin_wrong",
    "header_free_origin_duplicate",
    "header_free_type_missing",
    "header_free_type_wrong",
    "header_free_type_duplicate",
    "header_free_cookie_duplicate",
    "header_free_cookie_shape",
    "header_free_cookie_own_duplicate",
    "header_free_cookie_unrelated",
    "header_free_bound_host_absent",
    "header_free_bound_host_wrong",
    "header_free_bound_host_duplicate",
    "header_free_bound_transfer",
    "header_free_bound_expect",
    "header_free_bound_length_nonzero",
    "header_free_bound_length_duplicate",
    "header_free_bound_length_format",
    "header_free_bound_scheme",
    "header_free_bound_netloc",
    "header_free_bound_fragment",
    "header_free_bound_parse_target",
    "authorization_bound_host_absent",
    "authorization_bound_host_wrong",
    "authorization_bound_host_duplicate",
    "authorization_bound_transfer",
    "authorization_bound_expect",
    "authorization_bound_length_nonzero",
    "authorization_bound_length_duplicate",
    "authorization_bound_length_format",
    "authorization_bound_scheme",
    "authorization_bound_netloc",
    "authorization_bound_fragment",
    "authorization_bound_parse_target",
    "controlled_parser_result_header_free",
    "controlled_parser_result_authorization",
    "raw_bound_request_line",
    "raw_bound_header_line",
    "raw_bound_header_count",
    "raw_bound_header_total",
    "raw_bound_parser",
    "counter_across_header_free_flow_cleanup",
    "first_reason_preserved",
    "403_write_failure_propagates",
    "403_response_timeout_propagates",
)

class ControllerFailure(Exception):
    pass

def require(condition):
    if not condition:
        raise ControllerFailure

def exclusive(directory, name):
    descriptor = os.open(directory / name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    os.fchmod(descriptor, 0o600)
    return descriptor

def private_read(directory, name):
    descriptor = os.open(directory / name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        info = os.fstat(descriptor)
        require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid()
                and stat.S_IMODE(info.st_mode) == 0o600 and info.st_nlink == 1
                and 0 <= info.st_size <= CAP)
        with os.fdopen(descriptor, "rb", closefd=False) as stream:
            raw = stream.read(CAP + 1)
        require(len(raw) <= CAP)
        return raw
    finally:
        os.close(descriptor)

def save(directory, name, document):
    raw = (json.dumps(document, sort_keys=True) + "\n").encode("ascii")
    require(len(raw) <= CAP)
    descriptor = exclusive(directory, name)
    with os.fdopen(descriptor, "wb") as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())

def child_limits():
    resource.setrlimit(resource.RLIMIT_FSIZE, (CAP, CAP))

def safe_document(raw, observer_names, legacy_names):
    document = json.loads(raw)
    keys = {"schema", "result", "failure_tag", "failed_case", "planned_cases",
        "legacy_planned", "observer_planned", "legacy_completed", "observer_completed",
        "completed_cases", "legacy_groups", "observer_groups", "legacy_handler_calls",
        "expected_write_failures", "expected_response_timeouts", "private_format_traps", "native_captures",
        "elapsed_seconds", "deadline_seconds", "helper_sha256", "unchanged_sha256",
        "legacy_archive_sha256", "main_invoked", "server_constructed",
        "network_listener_native_provider_cli_driver_browser_cargo"}
    require(type(document) is dict and set(document) == keys)
    require(document["schema"] == "riauth.d01-observer-memory/v2")
    require(document["result"] in {"passed", "failed"})
    require(document["failure_tag"] in {None, "verification_deadline", "assertion_failed", "harness_exception"})
    names = set(observer_names) | {"source_archive", "legacy_source_pin", "observer_selector", "native_capture", "observer_totals"} | {
        "legacy_" + name for name in legacy_names}
    require(document["failed_case"] is None or document["failed_case"] in names)
    for key in ("planned_cases", "legacy_planned", "observer_planned", "legacy_completed",
                "observer_completed", "completed_cases", "deadline_seconds",
                "expected_write_failures", "expected_response_timeouts"):
        require(type(document[key]) is int and 0 <= document[key] <= 128)
    for key, names in (("legacy_groups", set(PLANNED_LEGACY)), ("observer_groups", set(PLANNED_OBSERVER)),
                      ("legacy_handler_calls", {"baseline", "candidate"}),
                      ("private_format_traps", {"str", "repr", "traceback"}),
                      ("native_captures", {"trusted", "cloned", "untrusted"})):
        require(type(document[key]) is dict and set(document[key]) == names)
        require(all(type(value) is int and 0 <= value <= 128 for value in document[key].values()))
    for key in ("main_invoked", "server_constructed", "network_listener_native_provider_cli_driver_browser_cargo"):
        require(type(document[key]) is bool)
    require(type(document["elapsed_seconds"]) in (int, float)
            and math.isfinite(document["elapsed_seconds"]) and document["elapsed_seconds"] >= 0)
    for key in ("helper_sha256", "unchanged_sha256", "legacy_archive_sha256"):
        value = document[key]
        require(type(value) is str and len(value) == 64
                and all(character in "0123456789abcdef" for character in value))
    return document

def bounded_memory_envelope(payload, directory):
    started = time.monotonic()
    stop = started + 33
    deadline = started + 35
    child = None
    output_fd = error_fd = None
    record = {"schema": "riauth.d01-observer-memory-controller/v2",
        "payload_sha256": PAYLOAD_SHA256, "controller_seconds": 35, "child_seconds": 30,
        "spawn_count": 0, "child_exit": None, "child_reaped": False,
        "first_failure": None, "cleanup_failures": [], "child_result": None,
        "stdout_bytes": None, "stderr_bytes": None, "stdout_sha256": None,
        "stderr_sha256": None, "elapsed_seconds": None, "within35": False}
    try:
        require(hashlib.sha256(payload).hexdigest() == PAYLOAD_SHA256)
        directory = pathlib.Path(directory)
        descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            info = os.fstat(descriptor)
            require(stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid()
                    and stat.S_IMODE(info.st_mode) == 0o700 and not os.listdir(descriptor))
        finally:
            os.close(descriptor)
        output_fd = exclusive(directory, "d01-memory-child.stdout")
        error_fd = exclusive(directory, "d01-memory-child.stderr")
        child = subprocess.Popen([sys.executable, "-I", "-B", "-"], stdin=subprocess.PIPE,
            stdout=output_fd, stderr=error_fd, cwd=directory,
            env={"PATH": "/usr/bin:/bin", "LANG": "C", "LC_ALL": "C"},
            preexec_fn=child_limits)
        record["spawn_count"] = 1
        child.communicate(input=payload, timeout=max(0.001, stop - time.monotonic()))
    except subprocess.TimeoutExpired:
        record["first_failure"] = "controller_timeout"
    except BaseException:
        record["first_failure"] = "controller_failure"
    finally:
        if child is not None:
            if child.poll() is None:
                try:
                    child.kill()
                except BaseException:
                    record["cleanup_failures"].append("owned_kill_failed")
                try:
                    child.communicate(timeout=max(0.001, deadline - time.monotonic()))
                except BaseException:
                    record["cleanup_failures"].append("owned_reap_unproven")
            record["child_exit"] = child.poll()
            record["child_reaped"] = record["child_exit"] is not None
        for descriptor in (output_fd, error_fd):
            if descriptor is not None:
                try:
                    os.close(descriptor)
                except OSError:
                    record["cleanup_failures"].append("capture_close_failed")
    # Capture/exit and the complete structurally safe fixed JSON are retained
    # before success/count/oracle expectations. Raw captures are never printed.
    try:
        stdout = private_read(directory, "d01-memory-child.stdout")
        stderr = private_read(directory, "d01-memory-child.stderr")
        record["stdout_bytes"], record["stderr_bytes"] = len(stdout), len(stderr)
        record["stdout_sha256"] = hashlib.sha256(stdout).hexdigest()
        record["stderr_sha256"] = hashlib.sha256(stderr).hexdigest()
        try:
            record["child_result"] = safe_document(stdout, OBSERVER_NAMES, LEGACY_NAMES)
        except BaseException:
            if record["first_failure"] is None:
                record["first_failure"] = "fixed_result_unavailable"
        record["elapsed_seconds"] = round(time.monotonic() - started, 6)
        record["within35"] = record["elapsed_seconds"] <= 35
        save(directory, "d01-memory-retained.json", record)
    except BaseException:
        # Missing/malformed capture never becomes an invented child result.
        return {"result": "failed", "failure_tag": record["first_failure"] or "retention_failed",
                "evidence_failure_tag": "retention_failed", "child_exit": record["child_exit"],
                "child_reaped": record["child_reaped"]}
    outcome = {"schema": "riauth.d01-observer-memory-review/v2", "result": "failed",
        "failure_tag": record["first_failure"], "retained_before_expectations": True,
        "child_exit": record["child_exit"], "child_reaped": record["child_reaped"],
        "within35": record["within35"]}
    try:
        require(record["first_failure"] is None and record["child_reaped"]
                and record["child_exit"] == 0 and record["within35"]
                and not record["cleanup_failures"] and not stderr)
        document = record["child_result"]
        require(document is not None)
        for key, value in (("helper_sha256", "75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1"),
                           ("unchanged_sha256", "7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0"),
                           ("legacy_archive_sha256", "ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f")):
            require(document[key] == value)
        require(document["result"] == "passed"
                and document["failure_tag"] is None and document["failed_case"] is None)
        require(document["planned_cases"] == 128 and document["legacy_planned"] == 78
                and document["observer_planned"] == 50 and document["completed_cases"] == 128
                and document["legacy_completed"] == 78 and document["observer_completed"] == 50)
        require(document["legacy_groups"] == PLANNED_LEGACY
                and document["observer_groups"] == PLANNED_OBSERVER
                and document["legacy_handler_calls"] == {"baseline": 44, "candidate": 80}
                and document["expected_write_failures"] == 1
                and document["expected_response_timeouts"] == 1
                and document["private_format_traps"] == {"str": 0, "repr": 0, "traceback": 0}
                and document["native_captures"] == {"trusted": 1, "cloned": 1, "untrusted": 1}
                and document["deadline_seconds"] == 30 and document["elapsed_seconds"] <= 30
                and not document["main_invoked"] and not document["server_constructed"]
                and not document["network_listener_native_provider_cli_driver_browser_cargo"])
        outcome["result"] = "passed"
    except BaseException:
        if outcome["failure_tag"] is None:
            outcome["failure_tag"] = "expectations_failed"
    outcome["elapsed_seconds_before_review_save"] = round(time.monotonic() - started, 6)
    outcome["within35"] = outcome["elapsed_seconds_before_review_save"] <= 35
    if not outcome["within35"]:
        outcome["result"] = "failed"
        if outcome["failure_tag"] is None:
            outcome["failure_tag"] = "controller_budget_exceeded"
    try:
        save(directory, "d01-memory-reviewed.json", outcome)
    except BaseException:
        outcome["result"] = "failed"
        outcome["evidence_failure_tag"] = "review_write_failed"
        if outcome["failure_tag"] is None:
            outcome["failure_tag"] = "review_write_failed"
    # Caller must retain this post-save completion sample too; the saved review
    # is an earlier sample and cannot independently prove a whole-controller bound.
    outcome["elapsed_seconds"] = round(time.monotonic() - started, 6)
    outcome["within35"] = outcome["elapsed_seconds"] <= 35
    if not outcome["within35"]:
        outcome["result"] = "failed"
        if outcome["failure_tag"] is None:
            outcome["failure_tag"] = "controller_budget_exceeded"
    return outcome
```

### Exact complete serialized payload byte archive

Marker: D01_DESCRIPTOR_SERIALIZED_PAYLOAD_BASE64_V2.
Every decoded byte is part of the four assignments plus the child logic above.
Encoding is ASCII base64 wrapped76 columns with a final LF; do not evaluate
the decoded module in this source/design phase.

```text
TEVHQUNZX0JBU0VMSU5FX1RFWFQ9IiMhL3Vzci9iaW4vZW52IHB5dGhvbjNcblwiXCJcIk9uZSBk
aXNwb3NhYmxlIGNvbmZpZGVudGlhbCBsb2NhbC1kZW1vIGFwcGxpY2F0aW9uOyBicm93c2VyIGF1
dGhlbnRpY2F0aW9uIG9ubHkuXG5cblB5dGhvbiAzLjExKywgUE9TSVggYWxhcm1zLCB0aGUgcGlu
bmVkIHJlY292ZXJ5IHZlcmlmaWVyIGFuZCBuYXRpdmUgT3BlblNTTCBhcmVcbnByZXJlcXVpc2l0
ZXMuIEltcG9ydCBzdGFydHMgbm90aGluZy4gVGhlIGV4dGVybmFsIG93bmVyIHN1cHBsaWVzIGEg
cHJpdmF0ZSBsYWIsXG5yZWdpc3RlcnMgdGhlIHByaW50ZWQgY2xpZW50LCBkcml2ZXMgdGhlIGJy
b3dzZXIgdGhyb3VnaCBSaVdvcmsgQ3VhLmFpIERyaXZlcixcbmFuZCBvd25zIHRoZSBJZFAvYnJv
d3NlciBsaWZlY3ljbGUgYW5kIHRoZSBvdmVyYWxsIDE1LW1pbnV0ZSBjbGVhbnVwIGRlYWRsaW5l
LlxuVGhpcyBoZWxwZXIgbmVpdGhlciBsb2dzIGluIHRvIHJpQXV0aCBub3IgYXBwcm92ZXMgYW4g
YXV0aG9yaXphdGlvbiByZXF1ZXN0LlxuXCJcIlwiXG5cbmltcG9ydCBhcmdwYXJzZVxuaW1wb3J0
IGNvbnRleHRsaWJcbmltcG9ydCBoYXNobGliXG5pbXBvcnQgaHR0cC5zZXJ2ZXJcbmltcG9ydCBq
c29uXG5pbXBvcnQgb3NcbmltcG9ydCByZVxuaW1wb3J0IHNlY3JldHNcbmltcG9ydCBzaHV0aWxc
bmltcG9ydCBzaWduYWxcbmltcG9ydCBzdGF0XG5pbXBvcnQgc3VicHJvY2Vzc1xuaW1wb3J0IHN5
c1xuaW1wb3J0IHRpbWVcbmltcG9ydCB0eXBlc1xuaW1wb3J0IHVybGxpYi5wYXJzZVxuZnJvbSBw
YXRobGliIGltcG9ydCBQYXRoXG5cblxuSVNTVUVSID0gXCJodHRwOi8vbG9jYWxob3N0OjkwMDBc
IlxuQ0xJRU5UX0lEID0gXCJsb2NhbC1kZW1vXCJcbk9SSUdJTiA9IFwiaHR0cDovL2xvY2FsaG9z
dDozMDAwXCJcbkFVVEhPUklUWSA9IFwibG9jYWxob3N0OjMwMDBcIlxuQ0FMTEJBQ0sgPSBPUklH
SU4gKyBcIi9jYWxsYmFja1wiXG5GTE9XX0NPT0tJRSA9IFwiZDAxX2RlbW9fZmxvd1wiXG5BUFBf
Q09PS0lFID0gXCJkMDFfbG9jYWxfZGVtb1wiXG5WRVJJRklFUl9DT01NSVQgPSBcIjljZWZlN2E1
NjQyNWJiNzNjMTc3NTNlODc2NmQ5MjMyMGI3N2RhM2JcIlxuVkVSSUZJRVJfQkxPQiA9IFwiM2Jl
NzQ3ZDAzMTQ2ZjFiY2FhM2VjMDEyZWU4ZDE3M2I2MWZhNzM3ZFwiXG5WRVJJRklFUl9TSEEyNTYg
PSBcImY2ZGQxYWEwYjcxZGU0NzkzYzliODZlZTc5OTAxMmJiMzFhOGZjMmQ2MTgyOTc2MDk0YjQ0
ZGY2ZWYwNGRlM2RcIlxuT1BFTlNTTF9TSEEyNTYgPSBcIjY3YTgzZGQ2ZDZkNzQ3ZDUwYzVkMjk2
ZGZmYjIzZTMyYmFlOWEyYzU4OGM5M2FlMmQ3N2U0YzYwN2I0NTVjNzJcIlxuT1BFTlNTTF9WRVJT
SU9OID0gXCJPcGVuU1NMIDMuNi40IDI1IEF1ZyAyMDI2IChMaWJyYXJ5OiBPcGVuU1NMIDMuNi40
IDI1IEF1ZyAyMDI2KVwiXG5NQVhfRklMRSA9IDI1NiAqIDEwMjRcbk1BWF9SRVFVRVNUX0xJTkUg
PSA4MTkyXG5NQVhfSEVBREVSUyA9IDgxOTJcbk1BWF9IRUFERVJfTElORSA9IDQwOTZcbk1BWF9I
RUFERVJfQ09VTlQgPSAzMlxuTUFYX0NPT0tJRSA9IDQwOTZcblJFUVVFU1RfSU5WQUxJRF9SRUFT
T05TID0ge1xuICAgIFwiaHR0cF9wYXJzZVwiLCBcImhvc3RcIiwgXCJhdXRob3JpemF0aW9uXCIs
IFwidHJhbnNmZXJfZW5jb2RpbmdcIiwgXCJleHBlY3RcIixcbiAgICBcImNvbnRlbnRfbGVuZ3Ro
XCIsIFwidGFyZ2V0X3NjaGVtZVwiLCBcInRhcmdldF9uZXRsb2NcIiwgXCJ0YXJnZXRfZnJhZ21l
bnRcIixcbiAgICBcIm1ldGhvZFwiLCBcInBvc3RfdGFyZ2V0XCIsIFwib3JpZ2luXCIsIFwiY29u
dGVudF90eXBlXCIsIFwiY29va2llX2hlYWRlcl9jb3VudFwiLFxuICAgIFwib3duX2Nvb2tpZV9z
aGFwZVwiLFxufVxuU1RPUF9GUkVFX0JZVEVTID0gMTcgKiAxMDI0KiozIC8vIDJcblBFTkRJTkdf
U0VDT05EUyA9IDE4MFxuRkFJTFVSRV9UQUdTID0ge1xuICAgIFwiYXJndW1lbnRzX2ludmFsaWRc
IiwgXCJwYXRoc19pbnZhbGlkXCIsIFwicHJpdmF0ZV9maWxlX2ludmFsaWRcIixcbiAgICBcImNy
ZWRlbnRpYWxfaW52YWxpZFwiLCBcInZlcmlmaWVyX2hhc2hfbWlzbWF0Y2hcIiwgXCJ2ZXJpZmll
cl9pbXBvcnRfZmFpbGVkXCIsXG4gICAgXCJwcm92aWRlcl9mYWlsZWRcIiwgXCJkaXNjb3Zlcnlf
ZmFpbGVkXCIsIFwibGlzdGVuZXJfZmFpbGVkXCIsXG4gICAgXCJyZXF1ZXN0X2ludmFsaWRcIiwg
XCJyZXF1ZXN0X2xpbWl0XCIsIFwicmVxdWVzdF90aW1lb3V0XCIsIFwicmVzcG9uc2VfdGltZW91
dFwiLFxuICAgIFwiZmxvd19hbHJlYWR5X3N0YXJ0ZWRcIiwgXCJjYWxsYmFja19pbnZhbGlkXCIs
IFwiY2FsbGJhY2tfYWxyZWFkeV9jb25zdW1lZFwiLFxuICAgIFwidG9rZW5fZXhjaGFuZ2VfZmFp
bGVkXCIsIFwidG9rZW5fdmFsaWRhdGlvbl9mYWlsZWRcIiwgXCJ1c2VyaW5mb19mYWlsZWRcIixc
biAgICBcInByb3RlY3RlZF9iZWZvcmVfdW5vYnNlcnZlZFwiLCBcImZpeHR1cmVfZGVhZGxpbmVc
IiwgXCJwZW5kaW5nX2RlYWRsaW5lXCIsXG4gICAgXCJkaXNrX21hcmdpblwiLCBcImRpc2tfb2Jz
ZXJ2YXRpb25fZmFpbGVkXCIsIFwiaW50ZXJydXB0ZWRcIiwgXCJjbGVhbnVwX2ZhaWxlZFwiLFxu
ICAgIFwiZXZpZGVuY2Vfd3JpdGVfZmFpbGVkXCIsIFwidW5leHBlY3RlZF9mYWlsdXJlXCIsXG59
XG5cblxuY2xhc3MgRmFpbHVyZShFeGNlcHRpb24pOlxuICAgIGRlZiBfX2luaXRfXyhzZWxmLCB0
YWcpOlxuICAgICAgICBzZWxmLnRhZyA9IHRhZyBpZiB0YWcgaW4gRkFJTFVSRV9UQUdTIGVsc2Ug
XCJ1bmV4cGVjdGVkX2ZhaWx1cmVcIlxuICAgICAgICBzdXBlcigpLl9faW5pdF9fKHNlbGYudGFn
KVxuXG5cbmNsYXNzIEhhbHQoQmFzZUV4Y2VwdGlvbik6XG4gICAgXCJcIlwiUGFzcyB0aHJvdWdo
IEhUVFAvdmVyaWZpZXIgRXhjZXB0aW9uIGhhbmRsZXJzIHRvIG93bmVkIGZpbmFsbHkgY2xlYW51
cC5cIlwiXCJcblxuICAgIGRlZiBfX2luaXRfXyhzZWxmLCB0YWcpOlxuICAgICAgICBzZWxmLnRh
ZyA9IHRhZyBpZiB0YWcgaW4gRkFJTFVSRV9UQUdTIGVsc2UgXCJ1bmV4cGVjdGVkX2ZhaWx1cmVc
IlxuXG5cbmRlZiByZXF1aXJlKGNvbmRpdGlvbiwgdGFnKTpcbiAgICBpZiBub3QgY29uZGl0aW9u
OlxuICAgICAgICByYWlzZSBGYWlsdXJlKHRhZylcblxuXG5kZWYgcHJpdmF0ZV9kaXJlY3Rvcnko
cGF0aCk6XG4gICAgcGF0aCA9IFBhdGgob3MucGF0aC5hYnNwYXRoKHBhdGgpKVxuICAgIGZkID0g
b3Mub3BlbihwYXRoLCBvcy5PX1JET05MWSB8IG9zLk9fRElSRUNUT1JZIHwgb3MuT19OT0ZPTExP
VylcbiAgICB0cnk6XG4gICAgICAgIGluZm8gPSBvcy5mc3RhdChmZClcbiAgICAgICAgcmVxdWly
ZShzdGF0LlNfSVNESVIoaW5mby5zdF9tb2RlKSBhbmQgaW5mby5zdF91aWQgPT0gb3MuZ2V0dWlk
KClcbiAgICAgICAgICAgICAgICBhbmQgc3RhdC5TX0lNT0RFKGluZm8uc3RfbW9kZSkgPT0gMG83
MDAsIFwicGF0aHNfaW52YWxpZFwiKVxuICAgIGZpbmFsbHk6XG4gICAgICAgIG9zLmNsb3NlKGZk
KVxuICAgIHJldHVybiBwYXRoXG5cblxuZGVmIHByaXZhdGVfYnl0ZXMocGF0aCwgbGltaXQ9TUFY
X0ZJTEUpOlxuICAgIGZkID0gb3Mub3BlbihwYXRoLCBvcy5PX1JET05MWSB8IG9zLk9fTk9GT0xM
T1cgfCBvcy5PX05PTkJMT0NLKVxuICAgIHRyeTpcbiAgICAgICAgaW5mbyA9IG9zLmZzdGF0KGZk
KVxuICAgICAgICByZXF1aXJlKHN0YXQuU19JU1JFRyhpbmZvLnN0X21vZGUpIGFuZCBpbmZvLnN0
X3VpZCA9PSBvcy5nZXR1aWQoKVxuICAgICAgICAgICAgICAgIGFuZCBzdGF0LlNfSU1PREUoaW5m
by5zdF9tb2RlKSA9PSAwbzYwMFxuICAgICAgICAgICAgICAgIGFuZCAwIDwgaW5mby5zdF9zaXpl
IDw9IGxpbWl0LCBcInByaXZhdGVfZmlsZV9pbnZhbGlkXCIpXG4gICAgICAgIHdpdGggb3MuZmRv
cGVuKGZkLCBcInJiXCIsIGNsb3NlZmQ9RmFsc2UpIGFzIHNvdXJjZTpcbiAgICAgICAgICAgIHJh
dyA9IHNvdXJjZS5yZWFkKGxpbWl0ICsgMSlcbiAgICAgICAgcmVxdWlyZSgwIDwgbGVuKHJhdykg
PD0gbGltaXQsIFwicHJpdmF0ZV9maWxlX2ludmFsaWRcIilcbiAgICAgICAgcmV0dXJuIHJhd1xu
ICAgIGZpbmFsbHk6XG4gICAgICAgIG9zLmNsb3NlKGZkKVxuXG5cbmRlZiBsb2FkX3ZlcmlmaWVy
KHBhdGgpOlxuICAgIHJhdyA9IHByaXZhdGVfYnl0ZXMocGF0aClcbiAgICByZXF1aXJlKGhhc2hs
aWIuc2hhMjU2KHJhdykuaGV4ZGlnZXN0KCkgPT0gVkVSSUZJRVJfU0hBMjU2LFxuICAgICAgICAg
ICAgXCJ2ZXJpZmllcl9oYXNoX21pc21hdGNoXCIpXG4gICAgIyBFeGVjdXRlIHByZWNpc2VseSB0
aGUgdmVyaWZpZWQgYnl0ZXMsIHdpdGhvdXQgcmVyZWFkaW5nIG9yIHdyaXRpbmcgcHljYWNoZS5c
biAgICBtb2R1bGUgPSB0eXBlcy5Nb2R1bGVUeXBlKFwiZDAxX3Bpbm5lZF9yZWNvdmVyeV92ZXJp
ZmllclwiKVxuICAgIG1vZHVsZS5fX2ZpbGVfXyA9IHN0cihwYXRoKVxuICAgIHRyeTpcbiAgICAg
ICAgZXhlYyhjb21waWxlKHJhdywgc3RyKHBhdGgpLCBcImV4ZWNcIiksIG1vZHVsZS5fX2RpY3Rf
XylcbiAgICBleGNlcHQgRXhjZXB0aW9uOlxuICAgICAgICByYWlzZSBGYWlsdXJlKFwidmVyaWZp
ZXJfaW1wb3J0X2ZhaWxlZFwiKSBmcm9tIE5vbmVcbiAgICByZXR1cm4gbW9kdWxlXG5cblxuZGVm
IGNsaWVudF9zZWNyZXQobW9kdWxlLCBwYXRoKTpcbiAgICB0cnk6XG4gICAgICAgIHNhdmVkID0g
bW9kdWxlLmpzb25fb2JqZWN0KHByaXZhdGVfYnl0ZXMocGF0aCkpXG4gICAgZXhjZXB0IEZhaWx1
cmU6XG4gICAgICAgIHJhaXNlXG4gICAgZXhjZXB0IEV4Y2VwdGlvbjpcbiAgICAgICAgcmFpc2Ug
RmFpbHVyZShcImNyZWRlbnRpYWxfaW52YWxpZFwiKSBmcm9tIE5vbmVcbiAgICBjbGllbnQgPSBz
YXZlZC5nZXQoXCJjbGllbnRcIilcbiAgICByZXF1aXJlKGlzaW5zdGFuY2UoY2xpZW50LCBkaWN0
KSwgXCJjcmVkZW50aWFsX2ludmFsaWRcIilcbiAgICBzZXR0aW5ncywgc2NvcGVzID0gY2xpZW50
LmdldChcInNldHRpbmdzXCIpLCBjbGllbnQuZ2V0KFwic2NvcGVzXCIpXG4gICAgcmVxdWlyZShj
bGllbnQuZ2V0KFwiY2xpZW50X2lkXCIpID09IENMSUVOVF9JRFxuICAgICAgICAgICAgYW5kIGNs
aWVudC5nZXQoXCJjb25maWRlbnRpYWxcIikgaXMgVHJ1ZVxuICAgICAgICAgICAgYW5kIGNsaWVu
dC5nZXQoXCJzZXJ2aWNlXCIpIGlzIEZhbHNlIGFuZCBjbGllbnQuZ2V0KFwiZW5hYmxlZFwiKSBp
cyBUcnVlXG4gICAgICAgICAgICBhbmQgY2xpZW50LmdldChcInJlZGlyZWN0X3VyaXNcIikgPT0g
W0NBTExCQUNLXVxuICAgICAgICAgICAgYW5kIGlzaW5zdGFuY2Uoc2NvcGVzLCBsaXN0KSBhbmQg
bGVuKHNjb3BlcykgPT0gMlxuICAgICAgICAgICAgYW5kIGFsbChpc2luc3RhbmNlKHZhbHVlLCBz
dHIpIGZvciB2YWx1ZSBpbiBzY29wZXMpXG4gICAgICAgICAgICBhbmQgc2V0KHNjb3BlcykgPT0g
e1wib3BlbmlkXCIsIFwicHJvZmlsZVwifVxuICAgICAgICAgICAgYW5kIGlzaW5zdGFuY2Uoc2V0
dGluZ3MsIGRpY3QpXG4gICAgICAgICAgICBhbmQgc2V0dGluZ3MuZ2V0KFwidG9rZW5fZW5kcG9p
bnRfYXV0aF9tZXRob2RcIikgaXMgTm9uZSxcbiAgICAgICAgICAgIFwiY3JlZGVudGlhbF9pbnZh
bGlkXCIpXG4gICAgc2VjcmV0ID0gc2F2ZWQuZ2V0KFwiY2xpZW50X3NlY3JldFwiKVxuICAgIHJl
cXVpcmUoaXNpbnN0YW5jZShzZWNyZXQsIHN0cikgYW5kIDAgPCBsZW4oc2VjcmV0KSA8PSA0MDk2
XG4gICAgICAgICAgICBhbmQgYWxsKDMyIDw9IG9yZCh2YWx1ZSkgPD0gMTI2IGZvciB2YWx1ZSBp
biBzZWNyZXQpLFxuICAgICAgICAgICAgXCJjcmVkZW50aWFsX2ludmFsaWRcIilcbiAgICBzYXZl
ZC5jbGVhcigpXG4gICAgcmV0dXJuIHNlY3JldFxuXG5cbmNsYXNzIEJ1ZGdldDpcbiAgICBcIlwi
XCJPbmUgbWFpbi10aHJlYWQgYWxhcm0gYm91bmRzIGJsb2NraW5nIHdvcmsgYW5kIHNhbXBsZXMg
ZnJlZSBzcGFjZS5cIlwiXCJcblxuICAgIGRlZiBfX2luaXRfXyhzZWxmLCBsYWIsIHN0YXJ0ZWQs
IHNlY29uZHMpOlxuICAgICAgICBzZWxmLmxhYiwgc2VsZi5kZWFkbGluZSA9IGxhYiwgc3RhcnRl
ZCArIHNlY29uZHNcbiAgICAgICAgc2VsZi5waGFzZV9kZWFkbGluZSA9IHNlbGYucGVuZGluZ19k
ZWFkbGluZSA9IE5vbmVcbiAgICAgICAgc2VsZi5waGFzZV90YWcgPSBcImZpeHR1cmVfZGVhZGxp
bmVcIlxuICAgICAgICBzZWxmLm1pbmltdW1fZnJlZSA9IE5vbmVcbiAgICAgICAgc2VsZi5zYW1w
bGVzID0gMFxuICAgICAgICBzZWxmLnByZXZpb3VzID0ge31cblxuICAgIGRlZiBfX2VudGVyX18o
c2VsZik6XG4gICAgICAgIHJlcXVpcmUoaGFzYXR0cihzaWduYWwsIFwic2V0aXRpbWVyXCIpIGFu
ZCBoYXNhdHRyKHNpZ25hbCwgXCJTSUdBTFJNXCIpXG4gICAgICAgICAgICAgICAgYW5kIHNpZ25h
bC5nZXRpdGltZXIoc2lnbmFsLklUSU1FUl9SRUFMKSA9PSAoMC4wLCAwLjApLFxuICAgICAgICAg
ICAgICAgIFwiYXJndW1lbnRzX2ludmFsaWRcIilcbiAgICAgICAgZm9yIG5hbWUgaW4gKHNpZ25h
bC5TSUdBTFJNLCBzaWduYWwuU0lHSU5ULCBzaWduYWwuU0lHVEVSTSk6XG4gICAgICAgICAgICBz
ZWxmLnByZXZpb3VzW25hbWVdID0gc2lnbmFsLmdldHNpZ25hbChuYW1lKVxuICAgICAgICAgICAg
c2lnbmFsLnNpZ25hbChuYW1lLCBzZWxmLnRpY2sgaWYgbmFtZSA9PSBzaWduYWwuU0lHQUxSTSBl
bHNlIHNlbGYuaW50ZXJydXB0KVxuICAgICAgICBzZWxmLnRpY2soKVxuICAgICAgICByZXR1cm4g
c2VsZlxuXG4gICAgZGVmIGludGVycnVwdChzZWxmLCBzaWdudW0sIGZyYW1lKTpcbiAgICAgICAg
cmFpc2UgSGFsdChcImludGVycnVwdGVkXCIpXG5cbiAgICBkZWYgdGljayhzZWxmLCBzaWdudW09
Tm9uZSwgZnJhbWU9Tm9uZSk6XG4gICAgICAgIHRyeTpcbiAgICAgICAgICAgIGZyZWUgPSBzaHV0
aWwuZGlza191c2FnZShzZWxmLmxhYikuZnJlZVxuICAgICAgICBleGNlcHQgRXhjZXB0aW9uOlxu
ICAgICAgICAgICAgcmFpc2UgSGFsdChcImRpc2tfb2JzZXJ2YXRpb25fZmFpbGVkXCIpIGZyb20g
Tm9uZVxuICAgICAgICBzZWxmLnNhbXBsZXMgKz0gMVxuICAgICAgICBzZWxmLm1pbmltdW1fZnJl
ZSA9IGZyZWUgaWYgc2VsZi5taW5pbXVtX2ZyZWUgaXMgTm9uZSBlbHNlIG1pbihzZWxmLm1pbmlt
dW1fZnJlZSwgZnJlZSlcbiAgICAgICAgcmVxdWlyZV9mcmVlID0gZnJlZSA+PSBTVE9QX0ZSRUVf
QllURVNcbiAgICAgICAgaWYgbm90IHJlcXVpcmVfZnJlZTpcbiAgICAgICAgICAgIHJhaXNlIEhh
bHQoXCJkaXNrX21hcmdpblwiKVxuICAgICAgICBub3cgPSB0aW1lLm1vbm90b25pYygpXG4gICAg
ICAgIGRlYWRsaW5lcyA9IFsoc2VsZi5kZWFkbGluZSwgXCJmaXh0dXJlX2RlYWRsaW5lXCIpXVxu
ICAgICAgICBpZiBzZWxmLnBoYXNlX2RlYWRsaW5lIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAg
ZGVhZGxpbmVzLmFwcGVuZCgoc2VsZi5waGFzZV9kZWFkbGluZSwgc2VsZi5waGFzZV90YWcpKVxu
ICAgICAgICBpZiBzZWxmLnBlbmRpbmdfZGVhZGxpbmUgaXMgbm90IE5vbmU6XG4gICAgICAgICAg
ICBkZWFkbGluZXMuYXBwZW5kKChzZWxmLnBlbmRpbmdfZGVhZGxpbmUsIFwicGVuZGluZ19kZWFk
bGluZVwiKSlcbiAgICAgICAgZGVhZGxpbmUsIHRhZyA9IG1pbihkZWFkbGluZXMpXG4gICAgICAg
IGlmIG5vdyA+PSBkZWFkbGluZTpcbiAgICAgICAgICAgIHJhaXNlIEhhbHQodGFnKVxuICAgICAg
ICBzaWduYWwuc2V0aXRpbWVyKHNpZ25hbC5JVElNRVJfUkVBTCwgbWluKDEuMCwgZGVhZGxpbmUg
LSBub3cpKVxuXG4gICAgQGNvbnRleHRsaWIuY29udGV4dG1hbmFnZXJcbiAgICBkZWYgbGltaXQo
c2VsZiwgc2Vjb25kcywgdGFnKTpcbiAgICAgICAgb2xkID0gc2VsZi5waGFzZV9kZWFkbGluZSwg
c2VsZi5waGFzZV90YWdcbiAgICAgICAgZGVhZGxpbmUgPSB0aW1lLm1vbm90b25pYygpICsgc2Vj
b25kc1xuICAgICAgICBpZiBvbGRbMF0gaXMgTm9uZSBvciBkZWFkbGluZSA8IG9sZFswXTpcbiAg
ICAgICAgICAgIHNlbGYucGhhc2VfZGVhZGxpbmUsIHNlbGYucGhhc2VfdGFnID0gZGVhZGxpbmUs
IHRhZ1xuICAgICAgICB0cnk6XG4gICAgICAgICAgICBzZWxmLnRpY2soKVxuICAgICAgICAgICAg
eWllbGRcbiAgICAgICAgZmluYWxseTpcbiAgICAgICAgICAgIHNlbGYucGhhc2VfZGVhZGxpbmUs
IHNlbGYucGhhc2VfdGFnID0gb2xkXG4gICAgICAgICAgICAjIERvIG5vdCByZXBsYWNlIGFuIGlu
LWZsaWdodCBIYWx0IHdpdGggYW5vdGhlciBhbGFybSBkdXJpbmcgdW53aW5kaW5nLlxuICAgICAg
ICAgICAgaWYgc3lzLmV4Y19pbmZvKClbMF0gaXMgTm9uZTpcbiAgICAgICAgICAgICAgICBzZWxm
LnRpY2soKVxuXG4gICAgZGVmIGNsb3NlKHNlbGYpOlxuICAgICAgICBpZiBzZWxmLnByZXZpb3Vz
OlxuICAgICAgICAgICAgc2lnbmFsLnNldGl0aW1lcihzaWduYWwuSVRJTUVSX1JFQUwsIDApXG4g
ICAgICAgICAgICBmb3IgbmFtZSwgaGFuZGxlciBpbiBzZWxmLnByZXZpb3VzLml0ZW1zKCk6XG4g
ICAgICAgICAgICAgICAgc2lnbmFsLnNpZ25hbChuYW1lLCBoYW5kbGVyKVxuICAgICAgICAgICAg
c2VsZi5wcmV2aW91cy5jbGVhcigpXG5cblxuY2xhc3MgRGVtbzpcbiAgICBkZWYgX19pbml0X18o
c2VsZiwgbW9kdWxlLCB3b3Jrc3BhY2UsIHNlY3JldCwgYnVkZ2V0LCBjaGVja3MsIHN0YXR1c2Vz
KTpcbiAgICAgICAgc2VsZi5tb2R1bGUsIHNlbGYud29ya3NwYWNlLCBzZWxmLnNlY3JldCA9IG1v
ZHVsZSwgd29ya3NwYWNlLCBzZWNyZXRcbiAgICAgICAgc2VsZi5pc3N1ZXIsIHNlbGYuY2xpZW50
X2lkID0gSVNTVUVSLCBDTElFTlRfSURcbiAgICAgICAgc2VsZi5idWRnZXQsIHNlbGYuY2hlY2tz
ID0gYnVkZ2V0LCBjaGVja3NcbiAgICAgICAgc2VsZi5zdGF0dXNlcyA9IHN0YXR1c2VzXG4gICAg
ICAgIHNlbGYuc3RhZ2UgPSBcInNldHVwXCJcbiAgICAgICAgc2VsZi5wZW5kaW5nID0gc2VsZi5j
b29raWUgPSBzZWxmLnN1YmplY3QgPSBOb25lXG4gICAgICAgIHNlbGYuYXR0ZW1wdGVkID0gc2Vs
Zi5kb25lID0gRmFsc2VcbiAgICAgICAgc2VsZi5mYWlsdXJlID0gTm9uZVxuICAgICAgICBzZWxm
LnJlcXVlc3RfaW52YWxpZF9yZWFzb24gPSBOb25lXG5cbiAgICBkZWYgaW52b2tlKHNlbGYsIHN0
YWdlLCBzZWNvbmRzLCBmdW5jdGlvbiwgKmFyZ3MsICoqa3dhcmdzKTpcbiAgICAgICAgc2VsZi5z
dGFnZSA9IHN0YWdlXG4gICAgICAgIHRyeTpcbiAgICAgICAgICAgIHdpdGggc2VsZi5idWRnZXQu
bGltaXQoc2Vjb25kcywgc3RhZ2UgKyBcIl9mYWlsZWRcIik6XG4gICAgICAgICAgICAgICAgcmV0
dXJuIGZ1bmN0aW9uKCphcmdzLCAqKmt3YXJncylcbiAgICAgICAgZXhjZXB0IEZhaWx1cmU6XG4g
ICAgICAgICAgICByYWlzZVxuICAgICAgICBleGNlcHQgRXhjZXB0aW9uOlxuICAgICAgICAgICAg
IyBJbXBvcnRlZCBmYWlsdXJlcyBhcmUgbmV2ZXIgZm9ybWF0dGVkIG9yIGNvcGllZCBpbnRvIHRo
ZSByZWNvcmQuXG4gICAgICAgICAgICByYWlzZSBGYWlsdXJlKHN0YWdlICsgXCJfZmFpbGVkXCIp
IGZyb20gTm9uZVxuXG4gICAgZGVmIHNldHVwKHNlbGYsIGV4ZWN1dGFibGUpOlxuICAgICAgICBk
ZWYgcHJvdmlkZXIoKTpcbiAgICAgICAgICAgIHNlbGYub3BlbnNzbCA9IFBhdGgoZXhlY3V0YWJs
ZSkucmVzb2x2ZShzdHJpY3Q9VHJ1ZSlcbiAgICAgICAgICAgIGluZm8gPSBzZWxmLm9wZW5zc2wu
c3RhdCgpXG4gICAgICAgICAgICByZXF1aXJlKHN0cihQYXRoKGV4ZWN1dGFibGUpKSA9PSBcIi9v
cHQvaG9tZWJyZXcvYmluL29wZW5zc2xcIlxuICAgICAgICAgICAgICAgICAgICBhbmQgc3RhdC5T
X0lTUkVHKGluZm8uc3RfbW9kZSkgYW5kIDAgPCBpbmZvLnN0X3NpemUgPD0gMzIgKiAxMDI0ICog
MTAyNCxcbiAgICAgICAgICAgICAgICAgICAgXCJwcm92aWRlcl9mYWlsZWRcIilcbiAgICAgICAg
ICAgIHdpdGggc2VsZi5vcGVuc3NsLm9wZW4oXCJyYlwiKSBhcyBzb3VyY2U6XG4gICAgICAgICAg
ICAgICAgcmVxdWlyZShoYXNobGliLmZpbGVfZGlnZXN0KHNvdXJjZSwgXCJzaGEyNTZcIikuaGV4
ZGlnZXN0KCkgPT0gT1BFTlNTTF9TSEEyNTYsXG4gICAgICAgICAgICAgICAgICAgICAgICBcInBy
b3ZpZGVyX2ZhaWxlZFwiKVxuICAgICAgICAgICAgc2VsZi5vcGVuc3NsX2Vudmlyb25tZW50ID0g
e2tleTogdmFsdWUgZm9yIGtleSwgdmFsdWUgaW4gb3MuZW52aXJvbi5pdGVtcygpXG4gICAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgaWYga2V5IGluIHtcIlBBVEhcIiwgXCJI
T01FXCIsIFwiVE1QRElSXCIsIFwiTEFOR1wiLCBcIkxDX0FMTFwifX1cbiAgICAgICAgICAgIHJl
c3VsdCA9IHN1YnByb2Nlc3MucnVuKFtzdHIoc2VsZi5vcGVuc3NsKSwgXCJ2ZXJzaW9uXCJdLCBj
YXB0dXJlX291dHB1dD1UcnVlLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg
dGltZW91dD01LCBlbnY9c2VsZi5vcGVuc3NsX2Vudmlyb25tZW50KVxuICAgICAgICAgICAgcmVx
dWlyZShyZXN1bHQucmV0dXJuY29kZSA9PSAwXG4gICAgICAgICAgICAgICAgICAgIGFuZCBsZW4o
cmVzdWx0LnN0ZG91dCkgPD0gMjU2IGFuZCBsZW4ocmVzdWx0LnN0ZGVycikgPD0gNDA5NlxuICAg
ICAgICAgICAgICAgICAgICBhbmQgcmVzdWx0LnN0ZG91dC5kZWNvZGUoXCJhc2NpaVwiKS5zdHJp
cCgpID09IE9QRU5TU0xfVkVSU0lPTixcbiAgICAgICAgICAgICAgICAgICAgXCJwcm92aWRlcl9m
YWlsZWRcIilcbiAgICAgICAgc2VsZi5pbnZva2UoXCJwcm92aWRlclwiLCA1LCBwcm92aWRlcilc
biAgICAgICAgc2VsZi5jaGVja3NbXCJwcm92aWRlcl9pZGVudGl0eV92ZXJpZmllZFwiXSA9IFRy
dWVcbiAgICAgICAgZG9jdW1lbnQgPSBzZWxmLmludm9rZShcImRpc2NvdmVyeVwiLCA1LFxuICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgIHNlbGYubW9kdWxlLkxvY2FsUmVseWluZ1BhcnR5
LmRpc2NvdmVyeSwgc2VsZilcbiAgICAgICAgbWV0aG9kcyA9IGRvY3VtZW50LmdldChcInRva2Vu
X2VuZHBvaW50X2F1dGhfbWV0aG9kc19zdXBwb3J0ZWRcIilcbiAgICAgICAgcmVxdWlyZShpc2lu
c3RhbmNlKG1ldGhvZHMsIGxpc3QpIGFuZCBcImNsaWVudF9zZWNyZXRfcG9zdFwiIGluIG1ldGhv
ZHNcbiAgICAgICAgICAgICAgICBhbmQgYWxsKGlzaW5zdGFuY2UodmFsdWUsIHN0cikgZm9yIHZh
bHVlIGluIG1ldGhvZHMpLCBcImRpc2NvdmVyeV9mYWlsZWRcIilcbiAgICAgICAgc2VsZi5kaXNj
b3ZlcnlfZG9jdW1lbnQgPSBkb2N1bWVudFxuICAgICAgICBzZWxmLmNoZWNrc1tcImRpc2NvdmVy
eV92ZXJpZmllZFwiXSA9IFRydWVcblxuICAgIGRlZiBiZWdpbihzZWxmKTpcbiAgICAgICAgc2Vs
Zi5zdGFnZSA9IFwiZmxvd1wiXG4gICAgICAgIHJlcXVpcmUobm90IHNlbGYuYXR0ZW1wdGVkLCBc
ImZsb3dfYWxyZWFkeV9zdGFydGVkXCIpXG4gICAgICAgIHNlbGYuYXR0ZW1wdGVkID0gVHJ1ZVxu
ICAgICAgICBzZWxmLnBlbmRpbmcgPSB7XCJzdGF0ZVwiOiBzZWNyZXRzLnRva2VuX3VybHNhZmUo
MzIpLCBcIm5vbmNlXCI6IHNlY3JldHMudG9rZW5fdXJsc2FmZSgzMiksXG4gICAgICAgICAgICAg
ICAgICAgICAgICBcInZlcmlmaWVyXCI6IHNlY3JldHMudG9rZW5fdXJsc2FmZSg0OCksXG4gICAg
ICAgICAgICAgICAgICAgICAgICBcImZsb3dfY29va2llXCI6IHNlY3JldHMudG9rZW5fdXJsc2Fm
ZSgzMiksXG4gICAgICAgICAgICAgICAgICAgICAgICBcInN0YXJ0ZWRfYXRcIjogaW50KHRpbWUu
dGltZSgpKX1cbiAgICAgICAgc2VsZi5idWRnZXQucGVuZGluZ19kZWFkbGluZSA9IHRpbWUubW9u
b3RvbmljKCkgKyBQRU5ESU5HX1NFQ09ORFNcbiAgICAgICAgc2VsZi5idWRnZXQudGljaygpXG4g
ICAgICAgIGZpZWxkcyA9IHtcInJlc3BvbnNlX3R5cGVcIjogXCJjb2RlXCIsIFwiY2xpZW50X2lk
XCI6IENMSUVOVF9JRCxcbiAgICAgICAgICAgICAgICAgIFwicmVkaXJlY3RfdXJpXCI6IENBTExC
QUNLLCBcInNjb3BlXCI6IFwib3BlbmlkIHByb2ZpbGVcIixcbiAgICAgICAgICAgICAgICAgIFwi
c3RhdGVcIjogc2VsZi5wZW5kaW5nW1wic3RhdGVcIl0sIFwibm9uY2VcIjogc2VsZi5wZW5kaW5n
W1wibm9uY2VcIl0sXG4gICAgICAgICAgICAgICAgICBcImNvZGVfY2hhbGxlbmdlXCI6IHNlbGYu
bW9kdWxlLmI2NHVybChcbiAgICAgICAgICAgICAgICAgICAgICBoYXNobGliLnNoYTI1NihzZWxm
LnBlbmRpbmdbXCJ2ZXJpZmllclwiXS5lbmNvZGUoXCJhc2NpaVwiKSkuZGlnZXN0KCkpLFxuICAg
ICAgICAgICAgICAgICAgXCJjb2RlX2NoYWxsZW5nZV9tZXRob2RcIjogXCJTMjU2XCJ9XG4gICAg
ICAgIHJldHVybiAoc2VsZi5kaXNjb3ZlcnlfZG9jdW1lbnRbXCJhdXRob3JpemF0aW9uX2VuZHBv
aW50XCJdICsgXCI/XCJcbiAgICAgICAgICAgICAgICArIHVybGxpYi5wYXJzZS51cmxlbmNvZGUo
ZmllbGRzKSwgc2VsZi5wZW5kaW5nW1wiZmxvd19jb29raWVcIl0pXG5cbiAgICBkZWYgY2FsbGJh
Y2soc2VsZiwgcXVlcnksIGZsb3dfY29va2llKTpcbiAgICAgICAgc2VsZi5zdGFnZSA9IFwiY2Fs
bGJhY2tcIlxuICAgICAgICByZXF1aXJlKHNlbGYucGVuZGluZyBpcyBub3QgTm9uZSwgXCJjYWxs
YmFja19hbHJlYWR5X2NvbnN1bWVkXCIpXG4gICAgICAgIHBlbmRpbmcsIHNlbGYucGVuZGluZyA9
IHNlbGYucGVuZGluZywgTm9uZVxuICAgICAgICBzZWxmLmJ1ZGdldC5wZW5kaW5nX2RlYWRsaW5l
ID0gTm9uZVxuICAgICAgICByYXcgPSB0b2tlbnMgPSBhY2Nlc3MgPSBzdWJqZWN0ID0gY29kZSA9
IGZvcm0gPSBOb25lXG4gICAgICAgIHRyeTpcbiAgICAgICAgICAgIHJlcXVpcmUoc2VsZi5tb2R1
bGUuZXF1YWwoZmxvd19jb29raWUsIHBlbmRpbmdbXCJmbG93X2Nvb2tpZVwiXSksIFwiY2FsbGJh
Y2tfaW52YWxpZFwiKVxuICAgICAgICAgICAgdHJ5OlxuICAgICAgICAgICAgICAgIGNvZGUgPSBz
ZWxmLm1vZHVsZS5Mb2NhbFJlbHlpbmdQYXJ0eS5jYWxsYmFja19maWVsZHMoc2VsZiwgcXVlcnks
IHBlbmRpbmdbXCJzdGF0ZVwiXSlcbiAgICAgICAgICAgIGV4Y2VwdCBFeGNlcHRpb246XG4gICAg
ICAgICAgICAgICAgcmFpc2UgRmFpbHVyZShcImNhbGxiYWNrX2ludmFsaWRcIikgZnJvbSBOb25l
XG4gICAgICAgICAgICBzZWxmLmNoZWNrc1tcInN0YXRlX2lzc3Vlcl9mbG93X2Nvb2tpZV92ZXJp
ZmllZFwiXSA9IFRydWVcbiAgICAgICAgICAgIGZvcm0gPSB7XCJncmFudF90eXBlXCI6IFwiYXV0
aG9yaXphdGlvbl9jb2RlXCIsIFwiY2xpZW50X2lkXCI6IENMSUVOVF9JRCxcbiAgICAgICAgICAg
ICAgICAgICAgXCJjbGllbnRfc2VjcmV0XCI6IHNlbGYuc2VjcmV0LCBcInJlZGlyZWN0X3VyaVwi
OiBDQUxMQkFDSyxcbiAgICAgICAgICAgICAgICAgICAgXCJjb2RlXCI6IGNvZGUsIFwiY29kZV92
ZXJpZmllclwiOiBwZW5kaW5nW1widmVyaWZpZXJcIl19XG4gICAgICAgICAgICBzdGF0dXMsIF8s
IHJhdyA9IHNlbGYuaW52b2tlKFwidG9rZW5fZXhjaGFuZ2VcIiwgNSwgc2VsZi5tb2R1bGUucmVx
dWVzdCxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgc2VsZi5kaXNj
b3ZlcnlfZG9jdW1lbnRbXCJ0b2tlbl9lbmRwb2ludFwiXSxcbiAgICAgICAgICAgICAgICAgICAg
ICAgICAgICAgICAgICAgICAgICAgbWV0aG9kPVwiUE9TVFwiLCBmb3JtPWZvcm0pXG4gICAgICAg
ICAgICBzZWxmLnN0YXR1c2VzW1widG9rZW5fZXhjaGFuZ2VcIl0gPSBzdGF0dXNcbiAgICAgICAg
ICAgIHJlcXVpcmUoc3RhdHVzID09IDIwMCwgXCJ0b2tlbl9leGNoYW5nZV9mYWlsZWRcIilcbiAg
ICAgICAgICAgIHRva2VucyA9IHNlbGYubW9kdWxlLmpzb25fb2JqZWN0KHJhdylcbiAgICAgICAg
ICAgIGFjY2Vzcywgc2NvcGUgPSB0b2tlbnMuZ2V0KFwiYWNjZXNzX3Rva2VuXCIpLCB0b2tlbnMu
Z2V0KFwic2NvcGVcIilcbiAgICAgICAgICAgIHJlcXVpcmUoaXNpbnN0YW5jZShhY2Nlc3MsIHN0
cikgYW5kIDAgPCBsZW4oYWNjZXNzKSA8PSBzZWxmLm1vZHVsZS5NQVhfVE9LRU5cbiAgICAgICAg
ICAgICAgICAgICAgYW5kIHJlLmZ1bGxtYXRjaChyXCJbQS1aYS16MC05Xy4tXStcIiwgYWNjZXNz
KSBpcyBub3QgTm9uZVxuICAgICAgICAgICAgICAgICAgICBhbmQgdG9rZW5zLmdldChcInRva2Vu
X3R5cGVcIikgPT0gXCJCZWFyZXJcIiBhbmQgaXNpbnN0YW5jZShzY29wZSwgc3RyKVxuICAgICAg
ICAgICAgICAgICAgICBhbmQgc2V0KHNjb3BlLnNwbGl0KCkpID09IHtcIm9wZW5pZFwiLCBcInBy
b2ZpbGVcIn0sIFwidG9rZW5fZXhjaGFuZ2VfZmFpbGVkXCIpXG4gICAgICAgICAgICBzZWxmLmNo
ZWNrc1tcImNvbmZpZGVudGlhbF9zMjU2X2V4Y2hhbmdlX3ZlcmlmaWVkXCJdID0gVHJ1ZVxuICAg
ICAgICAgICAgIyBLZWVwIHRoZSBwaW5uZWQgaW1wbGVtZW50YXRpb247IGNhcCBjb21iaW5lZCBK
V0tTL3NpZ25hdHVyZSB3b3JrXG4gICAgICAgICAgICAjIGF0IGZpdmUgcmVhbCBzZWNvbmRzIGFz
IHdlbGwgYXMgaXRzIGV4aXN0aW5nIHNvY2tldC9wcm9jZXNzIGNhcHMuXG4gICAgICAgICAgICBz
dWJqZWN0ID0gc2VsZi5pbnZva2UoXCJ0b2tlbl92YWxpZGF0aW9uXCIsIDUsXG4gICAgICAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgc2VsZi5tb2R1bGUuTG9jYWxSZWx5aW5nUGFydHkudmVy
aWZ5X2lkX3Rva2VuLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgIHNlbGYsIHRv
a2Vucy5nZXQoXCJpZF90b2tlblwiKSwgYWNjZXNzLCBwZW5kaW5nLFxuICAgICAgICAgICAgICAg
ICAgICAgICAgICAgICAgICAgIHNlbGYuZGlzY292ZXJ5X2RvY3VtZW50KVxuICAgICAgICAgICAg
c2VsZi5jaGVja3NbXCJyczI1Nl9qd2tzX2lzc3Vlcl9hdWRpZW5jZV9ub25jZV90aW1lX2FjY2Vz
c19oYXNoX3ZlcmlmaWVkXCJdID0gVHJ1ZVxuICAgICAgICAgICAgc3RhdHVzLCBfLCByYXcgPSBz
ZWxmLmludm9rZShcInVzZXJpbmZvXCIsIDUsIHNlbGYubW9kdWxlLnJlcXVlc3QsXG4gICAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgIHNlbGYuZGlzY292ZXJ5X2RvY3VtZW50
W1widXNlcmluZm9fZW5kcG9pbnRcIl0sIGJlYXJlcj1hY2Nlc3MpXG4gICAgICAgICAgICBzZWxm
LnN0YXR1c2VzW1widXNlcmluZm9cIl0gPSBzdGF0dXNcbiAgICAgICAgICAgIHJlcXVpcmUoc3Rh
dHVzID09IDIwMCBhbmQgc2VsZi5tb2R1bGUuZXF1YWwoc2VsZi5tb2R1bGUuanNvbl9vYmplY3Qo
cmF3KS5nZXQoXCJzdWJcIiksIHN1YmplY3QpLFxuICAgICAgICAgICAgICAgICAgICBcInVzZXJp
bmZvX2ZhaWxlZFwiKVxuICAgICAgICAgICAgc2VsZi5jaGVja3NbXCJ1c2VyaW5mb19zdWJqZWN0
X3ZlcmlmaWVkXCJdID0gVHJ1ZVxuICAgICAgICAgICAgc2VsZi5jb29raWUsIHNlbGYuc3ViamVj
dCA9IHNlY3JldHMudG9rZW5fdXJsc2FmZSgzMiksIHN1YmplY3RcbiAgICAgICAgICAgIHJldHVy
biBzZWxmLmNvb2tpZVxuICAgICAgICBmaW5hbGx5OlxuICAgICAgICAgICAgcGVuZGluZy5jbGVh
cigpXG4gICAgICAgICAgICBpZiBmb3JtIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAgICAgIGZv
cm0uY2xlYXIoKVxuICAgICAgICAgICAgcmF3ID0gdG9rZW5zID0gYWNjZXNzID0gc3ViamVjdCA9
IGNvZGUgPSBmb3JtID0gTm9uZVxuICAgICAgICAgICAgc2VsZi5zZWNyZXQgPSBOb25lXG5cbiAg
ICBkZWYgY2xlYXIoc2VsZik6XG4gICAgICAgIGlmIHNlbGYucGVuZGluZyBpcyBub3QgTm9uZTpc
biAgICAgICAgICAgIHNlbGYucGVuZGluZy5jbGVhcigpXG4gICAgICAgIHNlbGYucGVuZGluZyA9
IHNlbGYuY29va2llID0gc2VsZi5zdWJqZWN0ID0gc2VsZi5zZWNyZXQgPSBOb25lXG4gICAgICAg
IHNlbGYuYnVkZ2V0LnBlbmRpbmdfZGVhZGxpbmUgPSBOb25lXG5cblxuY2xhc3MgSGVhZGVyUmVh
ZGVyOlxuICAgIGRlZiBfX2luaXRfXyhzZWxmLCBzb3VyY2UpOlxuICAgICAgICBzZWxmLnNvdXJj
ZSwgc2VsZi5yZW1haW5pbmcsIHNlbGYubGluZXMgPSBzb3VyY2UsIE1BWF9IRUFERVJTLCAwXG5c
biAgICBkZWYgcmVhZGxpbmUoc2VsZiwgc2l6ZT0tMSk6XG4gICAgICAgIGxpbmUgPSBzZWxmLnNv
dXJjZS5yZWFkbGluZShtaW4oTUFYX0hFQURFUl9MSU5FICsgMSwgc2VsZi5yZW1haW5pbmcgKyAx
KSlcbiAgICAgICAgc2VsZi5yZW1haW5pbmcgLT0gbGVuKGxpbmUpXG4gICAgICAgIHNlbGYubGlu
ZXMgKz0gMVxuICAgICAgICByZXF1aXJlKGxlbihsaW5lKSA8PSBNQVhfSEVBREVSX0xJTkUgYW5k
IHNlbGYucmVtYWluaW5nID49IDBcbiAgICAgICAgICAgICAgICBhbmQgc2VsZi5saW5lcyA8PSBN
QVhfSEVBREVSX0NPVU5UICsgMSwgXCJyZXF1ZXN0X2xpbWl0XCIpXG4gICAgICAgIHJldHVybiBs
aW5lXG5cblxuY2xhc3MgRGVtb1NlcnZlcihodHRwLnNlcnZlci5IVFRQU2VydmVyKTpcbiAgICBk
ZWYgX19pbml0X18oc2VsZiwgZGVtbyk6XG4gICAgICAgIHNlbGYuZGVtbywgc2VsZi5hY3RpdmUg
PSBkZW1vLCBOb25lXG4gICAgICAgIHN1cGVyKCkuX19pbml0X18oKFwiMTI3LjAuMC4xXCIsIDMw
MDApLCBIYW5kbGVyKVxuICAgICAgICBzZWxmLnRpbWVvdXQgPSAwLjJcblxuICAgIGRlZiBwcm9j
ZXNzX3JlcXVlc3Qoc2VsZiwgcmVxdWVzdCwgYWRkcmVzcyk6XG4gICAgICAgIHNlbGYuYWN0aXZl
ID0gcmVxdWVzdFxuICAgICAgICB0cnk6XG4gICAgICAgICAgICBzZWxmLmZpbmlzaF9yZXF1ZXN0
KHJlcXVlc3QsIGFkZHJlc3MpXG4gICAgICAgIGZpbmFsbHk6XG4gICAgICAgICAgICBzZWxmLnNo
dXRkb3duX3JlcXVlc3QocmVxdWVzdClcbiAgICAgICAgICAgIHNlbGYuYWN0aXZlID0gTm9uZVxu
XG4gICAgZGVmIGhhbmRsZV9lcnJvcihzZWxmLCByZXF1ZXN0LCBhZGRyZXNzKTpcbiAgICAgICAg
c2VsZi5kZW1vLmZhaWx1cmUsIHNlbGYuZGVtby5kb25lID0gXCJ1bmV4cGVjdGVkX2ZhaWx1cmVc
IiwgVHJ1ZVxuXG5cbmNsYXNzIEhhbmRsZXIoaHR0cC5zZXJ2ZXIuQmFzZUhUVFBSZXF1ZXN0SGFu
ZGxlcik6XG4gICAgdGltZW91dCA9IDVcbiAgICBwcm90b2NvbF92ZXJzaW9uID0gXCJIVFRQLzEu
MFwiXG5cbiAgICBkZWYgbG9nX21lc3NhZ2Uoc2VsZiwgZm9ybWF0LCAqYXJncyk6XG4gICAgICAg
IHBhc3NcblxuICAgIGRlZiByZWNvcmRfcmVxdWVzdF9yZWFzb24oc2VsZiwgcmVhc29uKTpcbiAg
ICAgICAgZGVtbyA9IHNlbGYuc2VydmVyLmRlbW9cbiAgICAgICAgaWYgZGVtby5yZXF1ZXN0X2lu
dmFsaWRfcmVhc29uIGlzIE5vbmUgYW5kIHJlYXNvbiBpbiBSRVFVRVNUX0lOVkFMSURfUkVBU09O
UzpcbiAgICAgICAgICAgIGRlbW8ucmVxdWVzdF9pbnZhbGlkX3JlYXNvbiA9IHJlYXNvblxuXG4g
ICAgZGVmIHJlcXVpcmVfcmVxdWVzdChzZWxmLCBjb25kaXRpb24sIHJlYXNvbik6XG4gICAgICAg
IGlmIG5vdCBjb25kaXRpb246XG4gICAgICAgICAgICBzZWxmLnJlY29yZF9yZXF1ZXN0X3JlYXNv
bihyZWFzb24pXG4gICAgICAgICAgICByYWlzZSBGYWlsdXJlKFwicmVxdWVzdF9pbnZhbGlkXCIp
XG5cbiAgICBkZWYgc2VuZF9lcnJvcihzZWxmLCBjb2RlLCBtZXNzYWdlPU5vbmUsIGV4cGxhaW49
Tm9uZSk6XG4gICAgICAgIHNlbGYucmVjb3JkX3JlcXVlc3RfcmVhc29uKFwiaHR0cF9wYXJzZVwi
KVxuICAgICAgICBzZWxmLnNlcnZlci5kZW1vLmZhaWx1cmUsIHNlbGYuc2VydmVyLmRlbW8uZG9u
ZSA9IFwicmVxdWVzdF9pbnZhbGlkXCIsIFRydWVcbiAgICAgICAgc2VsZi5yZXBseShjb2RlLCBc
IkxvY2FsIGRlbW8gY291bGQgbm90IGNvbXBsZXRlIHRoaXMgcmVxdWVzdC5cIilcblxuICAgIGRl
ZiBoYW5kbGVfb25lX3JlcXVlc3Qoc2VsZik6XG4gICAgICAgIGRlbW8gPSBzZWxmLnNlcnZlci5k
ZW1vXG4gICAgICAgIHNlbGYucmVxdWVzdGxpbmUsIHNlbGYucmVxdWVzdF92ZXJzaW9uLCBzZWxm
LmNvbW1hbmQgPSBcIlwiLCBcIkhUVFAvMS4wXCIsIE5vbmVcbiAgICAgICAgc2VsZi5jbG9zZV9j
b25uZWN0aW9uID0gVHJ1ZVxuICAgICAgICB0cnk6XG4gICAgICAgICAgICBkZW1vLnN0YWdlID0g
XCJyZXF1ZXN0XCJcbiAgICAgICAgICAgIHdpdGggZGVtby5idWRnZXQubGltaXQoNSwgXCJyZXF1
ZXN0X3RpbWVvdXRcIik6XG4gICAgICAgICAgICAgICAgc2VsZi5yYXdfcmVxdWVzdGxpbmUgPSBz
ZWxmLnJmaWxlLnJlYWRsaW5lKE1BWF9SRVFVRVNUX0xJTkUgKyAxKVxuICAgICAgICAgICAgICAg
IGlmIG5vdCBzZWxmLnJhd19yZXF1ZXN0bGluZTpcbiAgICAgICAgICAgICAgICAgICAgcmV0dXJu
XG4gICAgICAgICAgICAgICAgcmVxdWlyZShsZW4oc2VsZi5yYXdfcmVxdWVzdGxpbmUpIDw9IE1B
WF9SRVFVRVNUX0xJTkUsIFwicmVxdWVzdF9saW1pdFwiKVxuICAgICAgICAgICAgICAgIG9yaWdp
bmFsLCBzZWxmLnJmaWxlID0gc2VsZi5yZmlsZSwgSGVhZGVyUmVhZGVyKHNlbGYucmZpbGUpXG4g
ICAgICAgICAgICAgICAgdHJ5OlxuICAgICAgICAgICAgICAgICAgICBpZiBub3Qgc2VsZi5wYXJz
ZV9yZXF1ZXN0KCk6XG4gICAgICAgICAgICAgICAgICAgICAgICByZXR1cm5cbiAgICAgICAgICAg
ICAgICBmaW5hbGx5OlxuICAgICAgICAgICAgICAgICAgICBzZWxmLnJmaWxlID0gb3JpZ2luYWxc
biAgICAgICAgICAgICAgICBzZWxmLmNsb3NlX2Nvbm5lY3Rpb24gPSBUcnVlXG4gICAgICAgICAg
ICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qoc2VsZi5oZWFkZXJzLmdldF9hbGwoXCJIb3N0XCIp
ID09IFtBVVRIT1JJVFldLCBcImhvc3RcIilcbiAgICAgICAgICAgICAgICBzZWxmLnJlcXVpcmVf
cmVxdWVzdChzZWxmLmhlYWRlcnMuZ2V0X2FsbChcIkF1dGhvcml6YXRpb25cIikgaXMgTm9uZSwg
XCJhdXRob3JpemF0aW9uXCIpXG4gICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qo
c2VsZi5oZWFkZXJzLmdldF9hbGwoXCJUcmFuc2Zlci1FbmNvZGluZ1wiKSBpcyBOb25lLCBcInRy
YW5zZmVyX2VuY29kaW5nXCIpXG4gICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qo
c2VsZi5oZWFkZXJzLmdldF9hbGwoXCJFeHBlY3RcIikgaXMgTm9uZSwgXCJleHBlY3RcIilcbiAg
ICAgICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVzdChzZWxmLmhlYWRlcnMuZ2V0X2FsbChc
IkNvbnRlbnQtTGVuZ3RoXCIpIGluIChOb25lLCBbXCIwXCJdKSwgXCJjb250ZW50X2xlbmd0aFwi
KVxuICAgICAgICAgICAgICAgIHRhcmdldCA9IHVybGxpYi5wYXJzZS51cmxzcGxpdChzZWxmLnBh
dGgpXG4gICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qobm90IHRhcmdldC5zY2hl
bWUsIFwidGFyZ2V0X3NjaGVtZVwiKVxuICAgICAgICAgICAgICAgIHNlbGYucmVxdWlyZV9yZXF1
ZXN0KG5vdCB0YXJnZXQubmV0bG9jLCBcInRhcmdldF9uZXRsb2NcIilcbiAgICAgICAgICAgICAg
ICBzZWxmLnJlcXVpcmVfcmVxdWVzdChub3QgdGFyZ2V0LmZyYWdtZW50LCBcInRhcmdldF9mcmFn
bWVudFwiKVxuICAgICAgICAgICAgaWYgc2VsZi5jb21tYW5kID09IFwiR0VUXCI6XG4gICAgICAg
ICAgICAgICAgc2VsZi5nZXQodGFyZ2V0KVxuICAgICAgICAgICAgZWxpZiBzZWxmLmNvbW1hbmQg
PT0gXCJQT1NUXCI6XG4gICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3QodGFyZ2V0
LnBhdGggPT0gXCIvbG9naW5cIiBhbmQgbm90IHRhcmdldC5xdWVyeSwgXCJwb3N0X3RhcmdldFwi
KVxuICAgICAgICAgICAgICAgIHNlbGYucmVxdWlyZV9yZXF1ZXN0KHNlbGYuaGVhZGVycy5nZXRf
YWxsKFwiT3JpZ2luXCIpID09IFtPUklHSU5dLCBcIm9yaWdpblwiKVxuICAgICAgICAgICAgICAg
IHNlbGYucmVxdWlyZV9yZXF1ZXN0KHNlbGYuaGVhZGVycy5nZXRfYWxsKFwiQ29udGVudC1UeXBl
XCIpID09IFtcImFwcGxpY2F0aW9uL3gtd3d3LWZvcm0tdXJsZW5jb2RlZFwiXSwgXCJjb250ZW50
X3R5cGVcIilcbiAgICAgICAgICAgICAgICBzZWxmLmNvb2tpZXMoKVxuICAgICAgICAgICAgICAg
IGxvY2F0aW9uLCBjb29raWUgPSBkZW1vLmJlZ2luKClcbiAgICAgICAgICAgICAgICBzZWxmLnJl
cGx5KDMwMywgXCJcIiwgbG9jYXRpb249bG9jYXRpb24sIGNvb2tpZT0oRkxPV19DT09LSUUsIGNv
b2tpZSkpXG4gICAgICAgICAgICAgICAgZGVtby5zdGF0dXNlc1tcImF1dGhvcml6YXRpb25fcmVk
aXJlY3RcIl0gPSAzMDNcbiAgICAgICAgICAgICAgICBkZW1vLmNoZWNrc1tcImF1dGhvcml6YXRp
b25fcmVkaXJlY3RfaXNzdWVkXCJdID0gVHJ1ZVxuICAgICAgICAgICAgZWxzZTpcbiAgICAgICAg
ICAgICAgICBzZWxmLnJlY29yZF9yZXF1ZXN0X3JlYXNvbihcIm1ldGhvZFwiKVxuICAgICAgICAg
ICAgICAgIHJhaXNlIEZhaWx1cmUoXCJyZXF1ZXN0X2ludmFsaWRcIilcbiAgICAgICAgZXhjZXB0
IEZhaWx1cmUgYXMgZmFpbHVyZTpcbiAgICAgICAgICAgIGRlbW8uZmFpbHVyZSwgZGVtby5kb25l
ID0gZmFpbHVyZS50YWcsIFRydWVcbiAgICAgICAgICAgIHNlbGYucmVwbHkoNDAwLCBcIkxvY2Fs
IGRlbW8gY291bGQgbm90IGNvbXBsZXRlIHRoaXMgcmVxdWVzdC5cIilcbiAgICAgICAgZXhjZXB0
IEV4Y2VwdGlvbjpcbiAgICAgICAgICAgIGRlbW8uZmFpbHVyZSwgZGVtby5kb25lID0gXCJ1bmV4
cGVjdGVkX2ZhaWx1cmVcIiwgVHJ1ZVxuICAgICAgICAgICAgc2VsZi5yZXBseSg0MDAsIFwiTG9j
YWwgZGVtbyBjb3VsZCBub3QgY29tcGxldGUgdGhpcyByZXF1ZXN0LlwiKVxuXG4gICAgZGVmIGNv
b2tpZXMoc2VsZik6XG4gICAgICAgIGhlYWRlcnMgPSBzZWxmLmhlYWRlcnMuZ2V0X2FsbChcIkNv
b2tpZVwiLCBbXSlcbiAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3QobGVuKGhlYWRlcnMpIDw9
IDEsIFwiY29va2llX2hlYWRlcl9jb3VudFwiKVxuICAgICAgICByYXcgPSBoZWFkZXJzWzBdIGlm
IGhlYWRlcnMgZWxzZSBcIlwiXG4gICAgICAgIHJlcXVpcmUobGVuKHJhdykgPD0gTUFYX0NPT0tJ
RSwgXCJyZXF1ZXN0X2xpbWl0XCIpXG4gICAgICAgIHBpZWNlcyA9IHJhdy5zcGxpdChcIjtcIikg
aWYgcmF3IGVsc2UgW11cbiAgICAgICAgcmVxdWlyZShsZW4ocGllY2VzKSA8PSBNQVhfSEVBREVS
X0NPVU5ULCBcInJlcXVlc3RfbGltaXRcIilcbiAgICAgICAgb3duID0ge31cbiAgICAgICAgZm9y
IHBpZWNlIGluIHBpZWNlczpcbiAgICAgICAgICAgIG5hbWUsIHNlcGFyYXRvciwgdmFsdWUgPSBw
aWVjZS5zdHJpcCgpLnBhcnRpdGlvbihcIj1cIilcbiAgICAgICAgICAgIGlmIG5hbWUgaW4gKEZM
T1dfQ09PS0lFLCBBUFBfQ09PS0lFKTpcbiAgICAgICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVx
dWVzdChzZXBhcmF0b3IgYW5kIG5hbWUgbm90IGluIG93blxuICAgICAgICAgICAgICAgICAgICAg
ICAgICAgICAgICAgICAgIGFuZCByZS5mdWxsbWF0Y2goclwiW0EtWmEtejAtOV8tXXs0M31cIiwg
dmFsdWUpIGlzIG5vdCBOb25lLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg
IFwib3duX2Nvb2tpZV9zaGFwZVwiKVxuICAgICAgICAgICAgICAgIG93bltuYW1lXSA9IHZhbHVl
XG4gICAgICAgIHJldHVybiBvd25cblxuICAgIGRlZiBnZXQoc2VsZiwgdGFyZ2V0KTpcbiAgICAg
ICAgZGVtbywgY29va2llcyA9IHNlbGYuc2VydmVyLmRlbW8sIHNlbGYuY29va2llcygpXG4gICAg
ICAgIGlmIHRhcmdldC5wYXRoID09IFwiL2NhbGxiYWNrXCI6XG4gICAgICAgICAgICBjb29raWUg
PSBkZW1vLmNhbGxiYWNrKHRhcmdldC5xdWVyeSwgY29va2llcy5nZXQoRkxPV19DT09LSUUpKVxu
ICAgICAgICAgICAgc2VsZi5yZXBseSgzMDMsIFwiXCIsIGxvY2F0aW9uPVwiL3Byb3RlY3RlZFwi
LCBjb29raWU9KEFQUF9DT09LSUUsIGNvb2tpZSksIGNsZWFyX2Zsb3c9VHJ1ZSlcbiAgICAgICAg
ICAgIGRlbW8uc3RhdHVzZXNbXCJjYWxsYmFja1wiXSA9IDMwM1xuICAgICAgICBlbGlmIHRhcmdl
dC5wYXRoID09IFwiL3Byb3RlY3RlZFwiIGFuZCBub3QgdGFyZ2V0LnF1ZXJ5OlxuICAgICAgICAg
ICAgZGVtby5zdGFnZSA9IFwicHJvdGVjdGVkXCJcbiAgICAgICAgICAgIGF1dGhlbnRpY2F0ZWQg
PSAoZGVtby5zdWJqZWN0IGlzIG5vdCBOb25lIGFuZCBkZW1vLmNvb2tpZSBpcyBub3QgTm9uZVxu
ICAgICAgICAgICAgICAgICAgICAgICAgICAgICBhbmQgZGVtby5tb2R1bGUuZXF1YWwoY29va2ll
cy5nZXQoQVBQX0NPT0tJRSksIGRlbW8uY29va2llKSlcbiAgICAgICAgICAgIHNlbGYucmVwbHko
MjAwIGlmIGF1dGhlbnRpY2F0ZWQgZWxzZSA0MDMsXG4gICAgICAgICAgICAgICAgICAgICAgIFwi
U2lnbmVkIGluLiBQcm90ZWN0ZWQgYXBwbGljYXRpb24gYWNjZXNzIGlzIGF2YWlsYWJsZS5cIiBp
ZiBhdXRoZW50aWNhdGVkXG4gICAgICAgICAgICAgICAgICAgICAgIGVsc2UgXCJTaWduIGluIHJl
cXVpcmVkLlwiLCBwcm90ZWN0ZWQ9YXV0aGVudGljYXRlZClcbiAgICAgICAgICAgIGlmIGF1dGhl
bnRpY2F0ZWQ6XG4gICAgICAgICAgICAgICAgZGVtby5jaGVja3NbXCJwcm90ZWN0ZWRfd2l0aF9m
cmVzaF9jb29raWVfYWNjZXB0ZWRcIl0gPSBUcnVlXG4gICAgICAgICAgICAgICAgZGVtby5zdGF0
dXNlc1tcInByb3RlY3RlZF9hZnRlclwiXSA9IDIwMFxuICAgICAgICAgICAgICAgIGlmIG5vdCBk
ZW1vLmNoZWNrc1tcInByb3RlY3RlZF93aXRob3V0X2Nvb2tpZV9kZW5pZWRcIl06XG4gICAgICAg
ICAgICAgICAgICAgIGRlbW8uZmFpbHVyZSA9IFwicHJvdGVjdGVkX2JlZm9yZV91bm9ic2VydmVk
XCJcbiAgICAgICAgICAgICAgICBkZW1vLmRvbmUgPSBUcnVlXG4gICAgICAgICAgICBlbGlmIG5v
dCBkZW1vLmF0dGVtcHRlZCBhbmQgQVBQX0NPT0tJRSBub3QgaW4gY29va2llczpcbiAgICAgICAg
ICAgICAgICBkZW1vLmNoZWNrc1tcInByb3RlY3RlZF93aXRob3V0X2Nvb2tpZV9kZW5pZWRcIl0g
PSBUcnVlXG4gICAgICAgICAgICAgICAgZGVtby5zdGF0dXNlc1tcInByb3RlY3RlZF9iZWZvcmVc
Il0gPSA0MDNcbiAgICAgICAgZWxpZiB0YXJnZXQucGF0aCA9PSBcIi9cIiBhbmQgbm90IHRhcmdl
dC5xdWVyeTpcbiAgICAgICAgICAgIHNlbGYucmVwbHkoMjAwLCBcIlVzZSBTaWduIGluIHRvIG9w
ZW4gdGhpcyBsb2NhbCBhcHBsaWNhdGlvbi5cIiwgc3RhcnQ9VHJ1ZSlcbiAgICAgICAgZWxzZTpc
biAgICAgICAgICAgIHNlbGYucmVwbHkoNDA0LCBcIlRoaXMgcGFnZSBpcyB1bmF2YWlsYWJsZS5c
IilcblxuICAgIGRlZiByZXBseShzZWxmLCBzdGF0dXMsIHRleHQsICosIGxvY2F0aW9uPU5vbmUs
IGNvb2tpZT1Ob25lLCBjbGVhcl9mbG93PUZhbHNlLFxuICAgICAgICAgICAgICBzdGFydD1GYWxz
ZSwgcHJvdGVjdGVkPUZhbHNlKTpcbiAgICAgICAgaGVhZGluZyA9IFwiUHJvdGVjdGVkIGFwcGxp
Y2F0aW9uIGFjY2Vzc1wiIGlmIHByb3RlY3RlZCBlbHNlIFwiTG9jYWwgZGVtb1wiXG4gICAgICAg
IGZvcm0gPSAoJzxmb3JtIG1ldGhvZD1cInBvc3RcIiBhY3Rpb249XCIvbG9naW5cIj48YnV0dG9u
IHR5cGU9XCJzdWJtaXRcIj5TaWduIGluPC9idXR0b24+PC9mb3JtPidcbiAgICAgICAgICAgICAg
ICBpZiBzdGFydCBlbHNlIFwiXCIpXG4gICAgICAgIHJhdyA9ICgnPCFkb2N0eXBlIGh0bWw+PGh0
bWwgbGFuZz1cImVuXCI+PG1ldGEgY2hhcnNldD1cInV0Zi04XCI+J1xuICAgICAgICAgICAgICAg
Jzx0aXRsZT5Mb2NhbCBkZW1vPC90aXRsZT48aDE+JyArIGhlYWRpbmcgKyAnPC9oMT48cD4nICsg
dGV4dCArICc8L3A+J1xuICAgICAgICAgICAgICAgKyBmb3JtICsgJzwvaHRtbD4nKS5lbmNvZGUo
XCJ1dGYtOFwiKVxuICAgICAgICB3aXRoIHNlbGYuc2VydmVyLmRlbW8uYnVkZ2V0LmxpbWl0KDUs
IFwicmVzcG9uc2VfdGltZW91dFwiKTpcbiAgICAgICAgICAgIHNlbGYuc2VuZF9yZXNwb25zZShz
dGF0dXMpXG4gICAgICAgICAgICBzZWxmLnNlbmRfaGVhZGVyKFwiQ29udGVudC1UeXBlXCIsIFwi
dGV4dC9odG1sOyBjaGFyc2V0PXV0Zi04XCIpXG4gICAgICAgICAgICBzZWxmLnNlbmRfaGVhZGVy
KFwiQ29udGVudC1MZW5ndGhcIiwgc3RyKGxlbihyYXcpKSlcbiAgICAgICAgICAgIHNlbGYuc2Vu
ZF9oZWFkZXIoXCJDb25uZWN0aW9uXCIsIFwiY2xvc2VcIilcbiAgICAgICAgICAgIHNlbGYuc2Vu
ZF9oZWFkZXIoXCJDYWNoZS1Db250cm9sXCIsIFwibm8tc3RvcmVcIilcbiAgICAgICAgICAgIHNl
bGYuc2VuZF9oZWFkZXIoXCJSZWZlcnJlci1Qb2xpY3lcIiwgXCJuby1yZWZlcnJlclwiKVxuICAg
ICAgICAgICAgc2VsZi5zZW5kX2hlYWRlcihcIkNvbnRlbnQtU2VjdXJpdHktUG9saWN5XCIsIFwi
ZGVmYXVsdC1zcmMgJ25vbmUnOyBmb3JtLWFjdGlvbiAnc2VsZic7IGZyYW1lLWFuY2VzdG9ycyAn
bm9uZSdcIilcbiAgICAgICAgICAgIGlmIGxvY2F0aW9uIGlzIG5vdCBOb25lOlxuICAgICAgICAg
ICAgICAgIHNlbGYuc2VuZF9oZWFkZXIoXCJMb2NhdGlvblwiLCBsb2NhdGlvbilcbiAgICAgICAg
ICAgIGlmIGNvb2tpZSBpcyBub3QgTm9uZTpcbiAgICAgICAgICAgICAgICBzZWxmLnNlbmRfaGVh
ZGVyKFwiU2V0LUNvb2tpZVwiLCBjb29raWVbMF0gKyBcIj1cIiArIGNvb2tpZVsxXVxuICAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgKyBcIjsgSHR0cE9ubHk7IFNhbWVTaXRlPUxheDsg
UGF0aD0vXCIpXG4gICAgICAgICAgICBpZiBjbGVhcl9mbG93OlxuICAgICAgICAgICAgICAgIHNl
bGYuc2VuZF9oZWFkZXIoXCJTZXQtQ29va2llXCIsIEZMT1dfQ09PS0lFICsgXCI9OyBNYXgtQWdl
PTA7IEh0dHBPbmx5OyBTYW1lU2l0ZT1MYXg7IFBhdGg9L1wiKVxuICAgICAgICAgICAgc2VsZi5l
bmRfaGVhZGVycygpXG4gICAgICAgICAgICBzZWxmLndmaWxlLndyaXRlKHJhdylcbiAgICAgICAg
ICAgIHNlbGYud2ZpbGUuZmx1c2goKVxuICAgICAgICBzZWxmLmNsb3NlX2Nvbm5lY3Rpb24gPSBU
cnVlXG5cblxuY2xhc3MgUXVpZXRQYXJzZXIoYXJncGFyc2UuQXJndW1lbnRQYXJzZXIpOlxuICAg
IGRlZiBlcnJvcihzZWxmLCBtZXNzYWdlKTpcbiAgICAgICAgcmFpc2UgRmFpbHVyZShcImFyZ3Vt
ZW50c19pbnZhbGlkXCIpXG5cblxuZGVmIG1haW4oKTpcbiAgICBzdGFydGVkID0gdGltZS5tb25v
dG9uaWMoKVxuICAgIGJ1ZGdldCA9IGRlbW8gPSBzZXJ2ZXIgPSBldmlkZW5jZV9mZCA9IHdvcmtz
cGFjZSA9IHNlY3JldCA9IE5vbmVcbiAgICBzdGFnZSA9IFwiYXJndW1lbnRzXCJcbiAgICByZWNv
cmQgPSB7XCJzY2hlbWFcIjogXCJyaWF1dGguZDAxLWNvbmZpZGVudGlhbC1icm93c2VyL3YxXCIs
IFwicmVzdWx0XCI6IFwiZmFpbGVkXCIsXG4gICAgICAgICAgICAgIFwiZmFpbHVyZV9zdGFnZVwi
OiBOb25lLCBcImZhaWx1cmVfdGFnXCI6IE5vbmUsIFwiY2hlY2tzXCI6IHt9LCBcImNsZWFudXBc
Ijoge30sXG4gICAgICAgICAgICAgIFwicmVxdWVzdF9pbnZhbGlkX3JlYXNvblwiOiBOb25lLFxu
ICAgICAgICAgICAgICBcInNvdXJjZVwiOiB7XCJ2ZXJpZmllcl9jb21taXRcIjogVkVSSUZJRVJf
Q09NTUlULCBcInZlcmlmaWVyX2Jsb2JcIjogVkVSSUZJRVJfQkxPQixcbiAgICAgICAgICAgICAg
ICAgICAgICAgICBcInZlcmlmaWVyX2V4cGVjdGVkX3NoYTI1NlwiOiBWRVJJRklFUl9TSEEyNTZ9
LFxuICAgICAgICAgICAgICBcInByb3ZpZGVyXCI6IE5vbmUsXG4gICAgICAgICAgICAgIFwiaHR0
cF9zdGF0dXNlc1wiOiB7bmFtZTogTm9uZSBmb3IgbmFtZSBpblxuICAgICAgICAgICAgICAgICAg
ICAgICAgICAgICAgICAoXCJhdXRob3JpemF0aW9uX3JlZGlyZWN0XCIsIFwiY2FsbGJhY2tcIiwg
XCJ0b2tlbl9leGNoYW5nZVwiLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgXCJ1
c2VyaW5mb1wiLCBcInByb3RlY3RlZF9iZWZvcmVcIiwgXCJwcm90ZWN0ZWRfYWZ0ZXJcIil9fVxu
ICAgIHRyeTpcbiAgICAgICAgcGFyc2VyID0gUXVpZXRQYXJzZXIoYWRkX2hlbHA9RmFsc2UsIGFs
bG93X2FiYnJldj1GYWxzZSlcbiAgICAgICAgZm9yIG5hbWUgaW4gKFwid29ya3NwYWNlXCIsIFwi
c2VjcmV0LWZpbGVcIiwgXCJ2ZXJpZmllci1oZWxwZXJcIiwgXCJvcGVuc3NsXCIsIFwiZXZpZGVu
Y2VcIik6XG4gICAgICAgICAgICBwYXJzZXIuYWRkX2FyZ3VtZW50KFwiLS1cIiArIG5hbWUsIHJl
cXVpcmVkPVRydWUpXG4gICAgICAgIHBhcnNlci5hZGRfYXJndW1lbnQoXCItLWRlYWRsaW5lLXNl
Y29uZHNcIiwgdHlwZT1pbnQsIGRlZmF1bHQ9NjAwKVxuICAgICAgICBhcmdzID0gcGFyc2VyLnBh
cnNlX2FyZ3MoKVxuICAgICAgICByZXF1aXJlKHN5cy52ZXJzaW9uX2luZm8gPj0gKDMsIDExKSBh
bmQgMSA8PSBhcmdzLmRlYWRsaW5lX3NlY29uZHMgPD0gNjAwLFxuICAgICAgICAgICAgICAgIFwi
YXJndW1lbnRzX2ludmFsaWRcIilcbiAgICAgICAgc3RhZ2UgPSBcInBhdGhzXCJcbiAgICAgICAg
d29ya3NwYWNlID0gcHJpdmF0ZV9kaXJlY3RvcnkoYXJncy53b3Jrc3BhY2UpXG4gICAgICAgIGxh
YiA9IHByaXZhdGVfZGlyZWN0b3J5KHdvcmtzcGFjZS5wYXJlbnQpXG4gICAgICAgIHJlcXVpcmUo
bm90IGFueSh3b3Jrc3BhY2UuaXRlcmRpcigpKSwgXCJwYXRoc19pbnZhbGlkXCIpXG4gICAgICAg
IHNlY3JldF9wYXRoLCB2ZXJpZmllcl9wYXRoLCBldmlkZW5jZV9wYXRoID0gW1BhdGgob3MucGF0
aC5hYnNwYXRoKHZhbHVlKSkgZm9yIHZhbHVlIGluXG4gICAgICAgICAgICAgICAgICAgICAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAoYXJncy5zZWNyZXRfZmlsZSwgYXJncy52ZXJpZmll
cl9oZWxwZXIsIGFyZ3MuZXZpZGVuY2UpXVxuICAgICAgICByZXF1aXJlKHByaXZhdGVfZGlyZWN0
b3J5KHNlY3JldF9wYXRoLnBhcmVudCkucGFyZW50ID09IGxhYlxuICAgICAgICAgICAgICAgIGFu
ZCB2ZXJpZmllcl9wYXRoLnBhcmVudCA9PSBsYWIgYW5kIGV2aWRlbmNlX3BhdGgucGFyZW50ID09
IGxhYlxuICAgICAgICAgICAgICAgIGFuZCBsZW4oe3NlY3JldF9wYXRoLCB2ZXJpZmllcl9wYXRo
LCBldmlkZW5jZV9wYXRofSkgPT0gMywgXCJwYXRoc19pbnZhbGlkXCIpXG4gICAgICAgIGJ1ZGdl
dCA9IEJ1ZGdldChsYWIsIHN0YXJ0ZWQsIGFyZ3MuZGVhZGxpbmVfc2Vjb25kcylcbiAgICAgICAg
YnVkZ2V0Ll9fZW50ZXJfXygpXG4gICAgICAgIGV2aWRlbmNlX2ZkID0gb3Mub3BlbihldmlkZW5j
ZV9wYXRoLCBvcy5PX1dST05MWSB8IG9zLk9fQ1JFQVQgfCBvcy5PX0VYQ0wgfCBvcy5PX05PRk9M
TE9XLCAwbzYwMClcbiAgICAgICAgb3MuZmNobW9kKGV2aWRlbmNlX2ZkLCAwbzYwMClcbiAgICAg
ICAgc3RhZ2UgPSBcInZlcmlmaWVyXCJcbiAgICAgICAgbW9kdWxlID0gbG9hZF92ZXJpZmllcih2
ZXJpZmllcl9wYXRoKVxuICAgICAgICByZWNvcmRbXCJzb3VyY2VcIl1bXCJ2ZXJpZmllcl9zaGEy
NTZcIl0gPSBWRVJJRklFUl9TSEEyNTZcbiAgICAgICAgcmVjb3JkW1wic291cmNlXCJdW1wiaGVs
cGVyX3NoYTI1NlwiXSA9IGhhc2hsaWIuc2hhMjU2KFBhdGgoX19maWxlX18pLnJlYWRfYnl0ZXMo
KSkuaGV4ZGlnZXN0KClcbiAgICAgICAgc3RhZ2UgPSBcImNyZWRlbnRpYWxcIlxuICAgICAgICBz
ZWNyZXQgPSBjbGllbnRfc2VjcmV0KG1vZHVsZSwgc2VjcmV0X3BhdGgpXG4gICAgICAgIGZvciBu
YW1lIGluIChcImNyZWRlbnRpYWxfcHJpdmF0ZV92YWxpZGF0ZWRcIiwgXCJwcm92aWRlcl9pZGVu
dGl0eV92ZXJpZmllZFwiLCBcImRpc2NvdmVyeV92ZXJpZmllZFwiLFxuICAgICAgICAgICAgICAg
ICAgICAgXCJwcm90ZWN0ZWRfd2l0aG91dF9jb29raWVfZGVuaWVkXCIsIFwiYXV0aG9yaXphdGlv
bl9yZWRpcmVjdF9pc3N1ZWRcIixcbiAgICAgICAgICAgICAgICAgICAgIFwic3RhdGVfaXNzdWVy
X2Zsb3dfY29va2llX3ZlcmlmaWVkXCIsIFwiY29uZmlkZW50aWFsX3MyNTZfZXhjaGFuZ2VfdmVy
aWZpZWRcIixcbiAgICAgICAgICAgICAgICAgICAgIFwicnMyNTZfandrc19pc3N1ZXJfYXVkaWVu
Y2Vfbm9uY2VfdGltZV9hY2Nlc3NfaGFzaF92ZXJpZmllZFwiLFxuICAgICAgICAgICAgICAgICAg
ICAgXCJ1c2VyaW5mb19zdWJqZWN0X3ZlcmlmaWVkXCIsIFwicHJvdGVjdGVkX3dpdGhfZnJlc2hf
Y29va2llX2FjY2VwdGVkXCIpOlxuICAgICAgICAgICAgcmVjb3JkW1wiY2hlY2tzXCJdW25hbWVd
ID0gbmFtZSA9PSBcImNyZWRlbnRpYWxfcHJpdmF0ZV92YWxpZGF0ZWRcIlxuICAgICAgICBkZW1v
ID0gRGVtbyhtb2R1bGUsIHdvcmtzcGFjZSwgc2VjcmV0LCBidWRnZXQsIHJlY29yZFtcImNoZWNr
c1wiXSwgcmVjb3JkW1wiaHR0cF9zdGF0dXNlc1wiXSlcbiAgICAgICAgc2VjcmV0ID0gTm9uZVxu
ICAgICAgICBkZW1vLnNldHVwKGFyZ3Mub3BlbnNzbClcbiAgICAgICAgcmVjb3JkW1wicHJvdmlk
ZXJcIl0gPSB7XCJzaGEyNTZcIjogT1BFTlNTTF9TSEEyNTYsIFwidmVyc2lvblwiOiBPUEVOU1NM
X1ZFUlNJT059XG4gICAgICAgIGRlbW8uc3RhZ2UgPSBcImxpc3RlbmVyXCJcbiAgICAgICAgdHJ5
OlxuICAgICAgICAgICAgc2VydmVyID0gRGVtb1NlcnZlcihkZW1vKVxuICAgICAgICBleGNlcHQg
RXhjZXB0aW9uOlxuICAgICAgICAgICAgcmFpc2UgRmFpbHVyZShcImxpc3RlbmVyX2ZhaWxlZFwi
KSBmcm9tIE5vbmVcbiAgICAgICAgcHJpbnQoanNvbi5kdW1wcyh7XCJyZWFkeVwiOiBUcnVlLCBc
InBvcnRcIjogMzAwMCwgXCJwaWRcIjogb3MuZ2V0cGlkKCl9KSwgZmx1c2g9VHJ1ZSlcbiAgICAg
ICAgd2hpbGUgbm90IGRlbW8uZG9uZTpcbiAgICAgICAgICAgIGJ1ZGdldC50aWNrKClcbiAgICAg
ICAgICAgIHNlcnZlci5oYW5kbGVfcmVxdWVzdCgpXG4gICAgICAgIGlmIGRlbW8uZmFpbHVyZSBp
cyBub3QgTm9uZTpcbiAgICAgICAgICAgIHJhaXNlIEZhaWx1cmUoZGVtby5mYWlsdXJlKVxuICAg
ICAgICByZXF1aXJlKGFsbChyZWNvcmRbXCJjaGVja3NcIl0udmFsdWVzKCkpLCBcInVuZXhwZWN0
ZWRfZmFpbHVyZVwiKVxuICAgICAgICByZWNvcmRbXCJyZXN1bHRcIl0gPSBcInBhc3NlZFwiXG4g
ICAgZXhjZXB0IChGYWlsdXJlLCBIYWx0KSBhcyBmYWlsdXJlOlxuICAgICAgICByZWNvcmRbXCJm
YWlsdXJlX3RhZ1wiXSA9IGZhaWx1cmUudGFnXG4gICAgICAgIHJlY29yZFtcImZhaWx1cmVfc3Rh
Z2VcIl0gPSBkZW1vLnN0YWdlIGlmIGRlbW8gaXMgbm90IE5vbmUgZWxzZSBzdGFnZVxuICAgIGV4
Y2VwdCBLZXlib2FyZEludGVycnVwdDpcbiAgICAgICAgcmVjb3JkW1wiZmFpbHVyZV90YWdcIl0s
IHJlY29yZFtcImZhaWx1cmVfc3RhZ2VcIl0gPSBcImludGVycnVwdGVkXCIsIHN0YWdlXG4gICAg
ZXhjZXB0IEV4Y2VwdGlvbjpcbiAgICAgICAgcmVjb3JkW1wiZmFpbHVyZV90YWdcIl0gPSBcInVu
ZXhwZWN0ZWRfZmFpbHVyZVwiXG4gICAgICAgIHJlY29yZFtcImZhaWx1cmVfc3RhZ2VcIl0gPSBk
ZW1vLnN0YWdlIGlmIGRlbW8gaXMgbm90IE5vbmUgZWxzZSBzdGFnZVxuICAgIGZpbmFsbHk6XG4g
ICAgICAgIGNsZWFudXBfZmFpbGVkID0gRmFsc2VcbiAgICAgICAgaWYgYnVkZ2V0IGlzIG5vdCBO
b25lOlxuICAgICAgICAgICAgdHJ5OlxuICAgICAgICAgICAgICAgIGJ1ZGdldC5jbG9zZSgpXG4g
ICAgICAgICAgICBleGNlcHQgQmFzZUV4Y2VwdGlvbjpcbiAgICAgICAgICAgICAgICBjbGVhbnVw
X2ZhaWxlZCA9IFRydWVcbiAgICAgICAgICAgIHJlY29yZFtcIm1pbmltdW1fZnJlZV9ieXRlc1wi
XSwgcmVjb3JkW1wiZGlza19zYW1wbGVzXCJdID0gYnVkZ2V0Lm1pbmltdW1fZnJlZSwgYnVkZ2V0
LnNhbXBsZXNcbiAgICAgICAgaWYgc2VydmVyIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAgdHJ5
OlxuICAgICAgICAgICAgICAgIGlmIHNlcnZlci5hY3RpdmUgaXMgbm90IE5vbmU6XG4gICAgICAg
ICAgICAgICAgICAgIHNlcnZlci5zaHV0ZG93bl9yZXF1ZXN0KHNlcnZlci5hY3RpdmUpXG4gICAg
ICAgICAgICAgICAgICAgIHNlcnZlci5hY3RpdmUgPSBOb25lXG4gICAgICAgICAgICBleGNlcHQg
QmFzZUV4Y2VwdGlvbjpcbiAgICAgICAgICAgICAgICBjbGVhbnVwX2ZhaWxlZCA9IFRydWVcbiAg
ICAgICAgICAgIHRyeTpcbiAgICAgICAgICAgICAgICBzZXJ2ZXIuc2VydmVyX2Nsb3NlKClcbiAg
ICAgICAgICAgIGV4Y2VwdCBCYXNlRXhjZXB0aW9uOlxuICAgICAgICAgICAgICAgIGNsZWFudXBf
ZmFpbGVkID0gVHJ1ZVxuICAgICAgICAgICAgcmVjb3JkW1wiY2xlYW51cFwiXVtcImxpc3RlbmVy
X2Nsb3NlZFwiXSA9IHNlcnZlci5zb2NrZXQuZmlsZW5vKCkgPT0gLTFcbiAgICAgICAgICAgIHJl
Y29yZFtcImNsZWFudXBcIl1bXCJjb25uZWN0aW9uX2Nsb3NlZFwiXSA9IHNlcnZlci5hY3RpdmUg
aXMgTm9uZVxuICAgICAgICBpZiBkZW1vIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAgcmVjb3Jk
W1wicmVxdWVzdF9pbnZhbGlkX3JlYXNvblwiXSA9IGRlbW8ucmVxdWVzdF9pbnZhbGlkX3JlYXNv
blxuICAgICAgICAgICAgdHJ5OlxuICAgICAgICAgICAgICAgIGRlbW8uY2xlYXIoKVxuICAgICAg
ICAgICAgICAgIHJlY29yZFtcImNsZWFudXBcIl1bXCJwcml2YXRlX3JlZmVyZW5jZXNfY2xlYXJl
ZFwiXSA9IFRydWVcbiAgICAgICAgICAgIGV4Y2VwdCBCYXNlRXhjZXB0aW9uOlxuICAgICAgICAg
ICAgICAgIGNsZWFudXBfZmFpbGVkID0gVHJ1ZVxuICAgICAgICAgICAgICAgIHJlY29yZFtcImNs
ZWFudXBcIl1bXCJwcml2YXRlX3JlZmVyZW5jZXNfY2xlYXJlZFwiXSA9IEZhbHNlXG4gICAgICAg
IHNlY3JldCA9IE5vbmVcbiAgICAgICAgdHJ5OlxuICAgICAgICAgICAgaWYgd29ya3NwYWNlIGlz
IG5vdCBOb25lOlxuICAgICAgICAgICAgICAgIHJlY29yZFtcImNsZWFudXBcIl1bXCJ2ZXJpZmll
cl90ZW1wb3Jhcmllc19yZW1vdmVkXCJdID0gbm90IGFueSh3b3Jrc3BhY2UuaXRlcmRpcigpKVxu
ICAgICAgICAgICAgaWYgY2xlYW51cF9mYWlsZWQgb3IgKHJlY29yZFtcInJlc3VsdFwiXSA9PSBc
InBhc3NlZFwiIGFuZCBub3QgYWxsKHJlY29yZFtcImNsZWFudXBcIl0udmFsdWVzKCkpKTpcbiAg
ICAgICAgICAgICAgICByYWlzZSBGYWlsdXJlKFwiY2xlYW51cF9mYWlsZWRcIilcbiAgICAgICAg
ZXhjZXB0IEJhc2VFeGNlcHRpb246XG4gICAgICAgICAgICByZWNvcmRbXCJyZXN1bHRcIl0gPSBc
ImZhaWxlZFwiXG4gICAgICAgICAgICByZWNvcmRbXCJjbGVhbnVwXCJdW1wiZmFpbHVyZV90YWdc
Il0gPSBcImNsZWFudXBfZmFpbGVkXCJcbiAgICAgICAgICAgIGlmIHJlY29yZFtcImZhaWx1cmVf
dGFnXCJdIGlzIE5vbmU6XG4gICAgICAgICAgICAgICAgcmVjb3JkW1wiZmFpbHVyZV90YWdcIl0s
IHJlY29yZFtcImZhaWx1cmVfc3RhZ2VcIl0gPSBcImNsZWFudXBfZmFpbGVkXCIsIFwiY2xlYW51
cFwiXG4gICAgICAgIHJlY29yZFtcImVsYXBzZWRfc2Vjb25kc1wiXSA9IHJvdW5kKHRpbWUubW9u
b3RvbmljKCkgLSBzdGFydGVkLCAzKVxuICAgIHdyaXR0ZW4gPSBGYWxzZVxuICAgIGlmIGV2aWRl
bmNlX2ZkIGlzIG5vdCBOb25lOlxuICAgICAgICB0cnk6XG4gICAgICAgICAgICB3aXRoIG9zLmZk
b3BlbihldmlkZW5jZV9mZCwgXCJ3YlwiKSBhcyBvdXRwdXQ6XG4gICAgICAgICAgICAgICAgb3V0
cHV0LndyaXRlKChqc29uLmR1bXBzKHJlY29yZCwgc29ydF9rZXlzPVRydWUpICsgXCJcXG5cIiku
ZW5jb2RlKFwiYXNjaWlcIikpXG4gICAgICAgICAgICAgICAgb3V0cHV0LmZsdXNoKClcbiAgICAg
ICAgICAgICAgICBvcy5mc3luYyhvdXRwdXQuZmlsZW5vKCkpXG4gICAgICAgICAgICB3cml0dGVu
ID0gVHJ1ZVxuICAgICAgICBleGNlcHQgRXhjZXB0aW9uOlxuICAgICAgICAgICAgcmVjb3JkW1wi
cmVzdWx0XCJdID0gXCJmYWlsZWRcIlxuICAgICAgICAgICAgcmVjb3JkW1wiZXZpZGVuY2VfZmFp
bHVyZV90YWdcIl0gPSBcImV2aWRlbmNlX3dyaXRlX2ZhaWxlZFwiXG4gICAgICAgICAgICBpZiBy
ZWNvcmRbXCJmYWlsdXJlX3RhZ1wiXSBpcyBOb25lOlxuICAgICAgICAgICAgICAgIHJlY29yZFtc
ImZhaWx1cmVfdGFnXCJdLCByZWNvcmRbXCJmYWlsdXJlX3N0YWdlXCJdID0gXCJldmlkZW5jZV93
cml0ZV9mYWlsZWRcIiwgXCJldmlkZW5jZVwiXG4gICAgICAgIGZpbmFsbHk6XG4gICAgICAgICAg
ICB0cnk6XG4gICAgICAgICAgICAgICAgb3MuY2xvc2UoZXZpZGVuY2VfZmQpXG4gICAgICAgICAg
ICBleGNlcHQgT1NFcnJvcjpcbiAgICAgICAgICAgICAgICBwYXNzXG4gICAgdHJ5OlxuICAgICAg
ICBwcmludChqc29uLmR1bXBzKHtcInJlc3VsdFwiOiByZWNvcmRbXCJyZXN1bHRcIl0sIFwiZmFp
bHVyZV90YWdcIjogcmVjb3JkW1wiZmFpbHVyZV90YWdcIl0sXG4gICAgICAgICAgICAgICAgICAg
ICAgICAgIFwiY2xlYW51cF9mYWlsdXJlX3RhZ1wiOiByZWNvcmRbXCJjbGVhbnVwXCJdLmdldChc
ImZhaWx1cmVfdGFnXCIpLFxuICAgICAgICAgICAgICAgICAgICAgICAgICBcImV2aWRlbmNlX2Zh
aWx1cmVfdGFnXCI6IHJlY29yZC5nZXQoXCJldmlkZW5jZV9mYWlsdXJlX3RhZ1wiKSxcbiAgICAg
ICAgICAgICAgICAgICAgICAgICAgXCJldmlkZW5jZV93cml0dGVuXCI6IHdyaXR0ZW59KSwgZmx1
c2g9VHJ1ZSlcbiAgICBleGNlcHQgRXhjZXB0aW9uOlxuICAgICAgICByZXR1cm4gMVxuICAgIHJl
dHVybiAwIGlmIHJlY29yZFtcInJlc3VsdFwiXSA9PSBcInBhc3NlZFwiIGFuZCB3cml0dGVuIGVs
c2UgMVxuXG5cbmlmIF9fbmFtZV9fID09IFwiX19tYWluX19cIjpcbiAgICByYWlzZSBTeXN0ZW1F
eGl0KG1haW4oKSlcbiIKVU5DSEFOR0VEX1RFWFQ9IiMhL3Vzci9iaW4vZW52IHB5dGhvbjNcblwi
XCJcIk9uZSBkaXNwb3NhYmxlIGNvbmZpZGVudGlhbCBsb2NhbC1kZW1vIGFwcGxpY2F0aW9uOyBi
cm93c2VyIGF1dGhlbnRpY2F0aW9uIG9ubHkuXG5cblB5dGhvbiAzLjExKywgUE9TSVggYWxhcm1z
LCB0aGUgcGlubmVkIHJlY292ZXJ5IHZlcmlmaWVyIGFuZCBuYXRpdmUgT3BlblNTTCBhcmVcbnBy
ZXJlcXVpc2l0ZXMuIEltcG9ydCBzdGFydHMgbm90aGluZy4gVGhlIGV4dGVybmFsIG93bmVyIHN1
cHBsaWVzIGEgcHJpdmF0ZSBsYWIsXG5yZWdpc3RlcnMgdGhlIHByaW50ZWQgY2xpZW50LCBkcml2
ZXMgdGhlIGJyb3dzZXIgdGhyb3VnaCBSaVdvcmsgQ3VhLmFpIERyaXZlcixcbmFuZCBvd25zIHRo
ZSBJZFAvYnJvd3NlciBsaWZlY3ljbGUgYW5kIHRoZSBvdmVyYWxsIDE1LW1pbnV0ZSBjbGVhbnVw
IGRlYWRsaW5lLlxuVGhpcyBoZWxwZXIgbmVpdGhlciBsb2dzIGluIHRvIHJpQXV0aCBub3IgYXBw
cm92ZXMgYW4gYXV0aG9yaXphdGlvbiByZXF1ZXN0LlxuXCJcIlwiXG5cbmltcG9ydCBhcmdwYXJz
ZVxuaW1wb3J0IGNvbnRleHRsaWJcbmltcG9ydCBoYXNobGliXG5pbXBvcnQgaHR0cC5zZXJ2ZXJc
bmltcG9ydCBqc29uXG5pbXBvcnQgb3NcbmltcG9ydCByZVxuaW1wb3J0IHNlY3JldHNcbmltcG9y
dCBzaHV0aWxcbmltcG9ydCBzaWduYWxcbmltcG9ydCBzdGF0XG5pbXBvcnQgc3VicHJvY2Vzc1xu
aW1wb3J0IHN5c1xuaW1wb3J0IHRpbWVcbmltcG9ydCB0eXBlc1xuaW1wb3J0IHVybGxpYi5wYXJz
ZVxuZnJvbSBwYXRobGliIGltcG9ydCBQYXRoXG5cblxuSVNTVUVSID0gXCJodHRwOi8vbG9jYWxo
b3N0OjkwMDBcIlxuQ0xJRU5UX0lEID0gXCJsb2NhbC1kZW1vXCJcbk9SSUdJTiA9IFwiaHR0cDov
L2xvY2FsaG9zdDozMDAwXCJcbkFVVEhPUklUWSA9IFwibG9jYWxob3N0OjMwMDBcIlxuQ0FMTEJB
Q0sgPSBPUklHSU4gKyBcIi9jYWxsYmFja1wiXG5GTE9XX0NPT0tJRSA9IFwiZDAxX2RlbW9fZmxv
d1wiXG5BUFBfQ09PS0lFID0gXCJkMDFfbG9jYWxfZGVtb1wiXG5WRVJJRklFUl9DT01NSVQgPSBc
IjljZWZlN2E1NjQyNWJiNzNjMTc3NTNlODc2NmQ5MjMyMGI3N2RhM2JcIlxuVkVSSUZJRVJfQkxP
QiA9IFwiM2JlNzQ3ZDAzMTQ2ZjFiY2FhM2VjMDEyZWU4ZDE3M2I2MWZhNzM3ZFwiXG5WRVJJRklF
Ul9TSEEyNTYgPSBcImY2ZGQxYWEwYjcxZGU0NzkzYzliODZlZTc5OTAxMmJiMzFhOGZjMmQ2MTgy
OTc2MDk0YjQ0ZGY2ZWYwNGRlM2RcIlxuT1BFTlNTTF9TSEEyNTYgPSBcIjY3YTgzZGQ2ZDZkNzQ3
ZDUwYzVkMjk2ZGZmYjIzZTMyYmFlOWEyYzU4OGM5M2FlMmQ3N2U0YzYwN2I0NTVjNzJcIlxuT1BF
TlNTTF9WRVJTSU9OID0gXCJPcGVuU1NMIDMuNi40IDI1IEF1ZyAyMDI2IChMaWJyYXJ5OiBPcGVu
U1NMIDMuNi40IDI1IEF1ZyAyMDI2KVwiXG5NQVhfRklMRSA9IDI1NiAqIDEwMjRcbk1BWF9SRVFV
RVNUX0xJTkUgPSA4MTkyXG5NQVhfSEVBREVSUyA9IDgxOTJcbk1BWF9IRUFERVJfTElORSA9IDQw
OTZcbk1BWF9IRUFERVJfQ09VTlQgPSAzMlxuTUFYX0NPT0tJRSA9IDQwOTZcblJFUVVFU1RfSU5W
QUxJRF9SRUFTT05TID0ge1xuICAgIFwiaHR0cF9wYXJzZVwiLCBcImhvc3RcIiwgXCJhdXRob3Jp
emF0aW9uXCIsIFwidHJhbnNmZXJfZW5jb2RpbmdcIiwgXCJleHBlY3RcIixcbiAgICBcImNvbnRl
bnRfbGVuZ3RoXCIsIFwidGFyZ2V0X3NjaGVtZVwiLCBcInRhcmdldF9uZXRsb2NcIiwgXCJ0YXJn
ZXRfZnJhZ21lbnRcIixcbiAgICBcIm1ldGhvZFwiLCBcInBvc3RfdGFyZ2V0XCIsIFwib3JpZ2lu
XCIsIFwiY29udGVudF90eXBlXCIsIFwiY29va2llX2hlYWRlcl9jb3VudFwiLFxuICAgIFwib3du
X2Nvb2tpZV9zaGFwZVwiLFxufVxuU1RPUF9GUkVFX0JZVEVTID0gMTcgKiAxMDI0KiozIC8vIDJc
blBFTkRJTkdfU0VDT05EUyA9IDE4MFxuRkFJTFVSRV9UQUdTID0ge1xuICAgIFwiYXJndW1lbnRz
X2ludmFsaWRcIiwgXCJwYXRoc19pbnZhbGlkXCIsIFwicHJpdmF0ZV9maWxlX2ludmFsaWRcIixc
biAgICBcImNyZWRlbnRpYWxfaW52YWxpZFwiLCBcInZlcmlmaWVyX2hhc2hfbWlzbWF0Y2hcIiwg
XCJ2ZXJpZmllcl9pbXBvcnRfZmFpbGVkXCIsXG4gICAgXCJwcm92aWRlcl9mYWlsZWRcIiwgXCJk
aXNjb3ZlcnlfZmFpbGVkXCIsIFwibGlzdGVuZXJfZmFpbGVkXCIsXG4gICAgXCJyZXF1ZXN0X2lu
dmFsaWRcIiwgXCJyZXF1ZXN0X2xpbWl0XCIsIFwicmVxdWVzdF90aW1lb3V0XCIsIFwicmVzcG9u
c2VfdGltZW91dFwiLFxuICAgIFwiZmxvd19hbHJlYWR5X3N0YXJ0ZWRcIiwgXCJjYWxsYmFja19p
bnZhbGlkXCIsIFwiY2FsbGJhY2tfYWxyZWFkeV9jb25zdW1lZFwiLFxuICAgIFwidG9rZW5fZXhj
aGFuZ2VfZmFpbGVkXCIsIFwidG9rZW5fdmFsaWRhdGlvbl9mYWlsZWRcIiwgXCJ1c2VyaW5mb19m
YWlsZWRcIixcbiAgICBcInByb3RlY3RlZF9iZWZvcmVfdW5vYnNlcnZlZFwiLCBcImZpeHR1cmVf
ZGVhZGxpbmVcIiwgXCJwZW5kaW5nX2RlYWRsaW5lXCIsXG4gICAgXCJkaXNrX21hcmdpblwiLCBc
ImRpc2tfb2JzZXJ2YXRpb25fZmFpbGVkXCIsIFwiaW50ZXJydXB0ZWRcIiwgXCJjbGVhbnVwX2Zh
aWxlZFwiLFxuICAgIFwiZXZpZGVuY2Vfd3JpdGVfZmFpbGVkXCIsIFwidW5leHBlY3RlZF9mYWls
dXJlXCIsXG59XG5cblxuY2xhc3MgRmFpbHVyZShFeGNlcHRpb24pOlxuICAgIGRlZiBfX2luaXRf
XyhzZWxmLCB0YWcpOlxuICAgICAgICBzZWxmLnRhZyA9IHRhZyBpZiB0YWcgaW4gRkFJTFVSRV9U
QUdTIGVsc2UgXCJ1bmV4cGVjdGVkX2ZhaWx1cmVcIlxuICAgICAgICBzdXBlcigpLl9faW5pdF9f
KHNlbGYudGFnKVxuXG5cbmNsYXNzIEhhbHQoQmFzZUV4Y2VwdGlvbik6XG4gICAgXCJcIlwiUGFz
cyB0aHJvdWdoIEhUVFAvdmVyaWZpZXIgRXhjZXB0aW9uIGhhbmRsZXJzIHRvIG93bmVkIGZpbmFs
bHkgY2xlYW51cC5cIlwiXCJcblxuICAgIGRlZiBfX2luaXRfXyhzZWxmLCB0YWcpOlxuICAgICAg
ICBzZWxmLnRhZyA9IHRhZyBpZiB0YWcgaW4gRkFJTFVSRV9UQUdTIGVsc2UgXCJ1bmV4cGVjdGVk
X2ZhaWx1cmVcIlxuXG5cbmRlZiByZXF1aXJlKGNvbmRpdGlvbiwgdGFnKTpcbiAgICBpZiBub3Qg
Y29uZGl0aW9uOlxuICAgICAgICByYWlzZSBGYWlsdXJlKHRhZylcblxuXG5kZWYgcHJpdmF0ZV9k
aXJlY3RvcnkocGF0aCk6XG4gICAgcGF0aCA9IFBhdGgob3MucGF0aC5hYnNwYXRoKHBhdGgpKVxu
ICAgIGZkID0gb3Mub3BlbihwYXRoLCBvcy5PX1JET05MWSB8IG9zLk9fRElSRUNUT1JZIHwgb3Mu
T19OT0ZPTExPVylcbiAgICB0cnk6XG4gICAgICAgIGluZm8gPSBvcy5mc3RhdChmZClcbiAgICAg
ICAgcmVxdWlyZShzdGF0LlNfSVNESVIoaW5mby5zdF9tb2RlKSBhbmQgaW5mby5zdF91aWQgPT0g
b3MuZ2V0dWlkKClcbiAgICAgICAgICAgICAgICBhbmQgc3RhdC5TX0lNT0RFKGluZm8uc3RfbW9k
ZSkgPT0gMG83MDAsIFwicGF0aHNfaW52YWxpZFwiKVxuICAgIGZpbmFsbHk6XG4gICAgICAgIG9z
LmNsb3NlKGZkKVxuICAgIHJldHVybiBwYXRoXG5cblxuZGVmIHByaXZhdGVfYnl0ZXMocGF0aCwg
bGltaXQ9TUFYX0ZJTEUpOlxuICAgIGZkID0gb3Mub3BlbihwYXRoLCBvcy5PX1JET05MWSB8IG9z
Lk9fTk9GT0xMT1cgfCBvcy5PX05PTkJMT0NLKVxuICAgIHRyeTpcbiAgICAgICAgaW5mbyA9IG9z
LmZzdGF0KGZkKVxuICAgICAgICByZXF1aXJlKHN0YXQuU19JU1JFRyhpbmZvLnN0X21vZGUpIGFu
ZCBpbmZvLnN0X3VpZCA9PSBvcy5nZXR1aWQoKVxuICAgICAgICAgICAgICAgIGFuZCBzdGF0LlNf
SU1PREUoaW5mby5zdF9tb2RlKSA9PSAwbzYwMFxuICAgICAgICAgICAgICAgIGFuZCAwIDwgaW5m
by5zdF9zaXplIDw9IGxpbWl0LCBcInByaXZhdGVfZmlsZV9pbnZhbGlkXCIpXG4gICAgICAgIHdp
dGggb3MuZmRvcGVuKGZkLCBcInJiXCIsIGNsb3NlZmQ9RmFsc2UpIGFzIHNvdXJjZTpcbiAgICAg
ICAgICAgIHJhdyA9IHNvdXJjZS5yZWFkKGxpbWl0ICsgMSlcbiAgICAgICAgcmVxdWlyZSgwIDwg
bGVuKHJhdykgPD0gbGltaXQsIFwicHJpdmF0ZV9maWxlX2ludmFsaWRcIilcbiAgICAgICAgcmV0
dXJuIHJhd1xuICAgIGZpbmFsbHk6XG4gICAgICAgIG9zLmNsb3NlKGZkKVxuXG5cbmRlZiBsb2Fk
X3ZlcmlmaWVyKHBhdGgpOlxuICAgIHJhdyA9IHByaXZhdGVfYnl0ZXMocGF0aClcbiAgICByZXF1
aXJlKGhhc2hsaWIuc2hhMjU2KHJhdykuaGV4ZGlnZXN0KCkgPT0gVkVSSUZJRVJfU0hBMjU2LFxu
ICAgICAgICAgICAgXCJ2ZXJpZmllcl9oYXNoX21pc21hdGNoXCIpXG4gICAgIyBFeGVjdXRlIHBy
ZWNpc2VseSB0aGUgdmVyaWZpZWQgYnl0ZXMsIHdpdGhvdXQgcmVyZWFkaW5nIG9yIHdyaXRpbmcg
cHljYWNoZS5cbiAgICBtb2R1bGUgPSB0eXBlcy5Nb2R1bGVUeXBlKFwiZDAxX3Bpbm5lZF9yZWNv
dmVyeV92ZXJpZmllclwiKVxuICAgIG1vZHVsZS5fX2ZpbGVfXyA9IHN0cihwYXRoKVxuICAgIHRy
eTpcbiAgICAgICAgZXhlYyhjb21waWxlKHJhdywgc3RyKHBhdGgpLCBcImV4ZWNcIiksIG1vZHVs
ZS5fX2RpY3RfXylcbiAgICBleGNlcHQgRXhjZXB0aW9uOlxuICAgICAgICByYWlzZSBGYWlsdXJl
KFwidmVyaWZpZXJfaW1wb3J0X2ZhaWxlZFwiKSBmcm9tIE5vbmVcbiAgICByZXR1cm4gbW9kdWxl
XG5cblxuZGVmIGNsaWVudF9zZWNyZXQobW9kdWxlLCBwYXRoKTpcbiAgICB0cnk6XG4gICAgICAg
IHNhdmVkID0gbW9kdWxlLmpzb25fb2JqZWN0KHByaXZhdGVfYnl0ZXMocGF0aCkpXG4gICAgZXhj
ZXB0IEZhaWx1cmU6XG4gICAgICAgIHJhaXNlXG4gICAgZXhjZXB0IEV4Y2VwdGlvbjpcbiAgICAg
ICAgcmFpc2UgRmFpbHVyZShcImNyZWRlbnRpYWxfaW52YWxpZFwiKSBmcm9tIE5vbmVcbiAgICBj
bGllbnQgPSBzYXZlZC5nZXQoXCJjbGllbnRcIilcbiAgICByZXF1aXJlKGlzaW5zdGFuY2UoY2xp
ZW50LCBkaWN0KSwgXCJjcmVkZW50aWFsX2ludmFsaWRcIilcbiAgICBzZXR0aW5ncywgc2NvcGVz
ID0gY2xpZW50LmdldChcInNldHRpbmdzXCIpLCBjbGllbnQuZ2V0KFwic2NvcGVzXCIpXG4gICAg
cmVxdWlyZShjbGllbnQuZ2V0KFwiY2xpZW50X2lkXCIpID09IENMSUVOVF9JRFxuICAgICAgICAg
ICAgYW5kIGNsaWVudC5nZXQoXCJjb25maWRlbnRpYWxcIikgaXMgVHJ1ZVxuICAgICAgICAgICAg
YW5kIGNsaWVudC5nZXQoXCJzZXJ2aWNlXCIpIGlzIEZhbHNlIGFuZCBjbGllbnQuZ2V0KFwiZW5h
YmxlZFwiKSBpcyBUcnVlXG4gICAgICAgICAgICBhbmQgY2xpZW50LmdldChcInJlZGlyZWN0X3Vy
aXNcIikgPT0gW0NBTExCQUNLXVxuICAgICAgICAgICAgYW5kIGlzaW5zdGFuY2Uoc2NvcGVzLCBs
aXN0KSBhbmQgbGVuKHNjb3BlcykgPT0gMlxuICAgICAgICAgICAgYW5kIGFsbChpc2luc3RhbmNl
KHZhbHVlLCBzdHIpIGZvciB2YWx1ZSBpbiBzY29wZXMpXG4gICAgICAgICAgICBhbmQgc2V0KHNj
b3BlcykgPT0ge1wib3BlbmlkXCIsIFwicHJvZmlsZVwifVxuICAgICAgICAgICAgYW5kIGlzaW5z
dGFuY2Uoc2V0dGluZ3MsIGRpY3QpXG4gICAgICAgICAgICBhbmQgc2V0dGluZ3MuZ2V0KFwidG9r
ZW5fZW5kcG9pbnRfYXV0aF9tZXRob2RcIikgaXMgTm9uZSxcbiAgICAgICAgICAgIFwiY3JlZGVu
dGlhbF9pbnZhbGlkXCIpXG4gICAgc2VjcmV0ID0gc2F2ZWQuZ2V0KFwiY2xpZW50X3NlY3JldFwi
KVxuICAgIHJlcXVpcmUoaXNpbnN0YW5jZShzZWNyZXQsIHN0cikgYW5kIDAgPCBsZW4oc2VjcmV0
KSA8PSA0MDk2XG4gICAgICAgICAgICBhbmQgYWxsKDMyIDw9IG9yZCh2YWx1ZSkgPD0gMTI2IGZv
ciB2YWx1ZSBpbiBzZWNyZXQpLFxuICAgICAgICAgICAgXCJjcmVkZW50aWFsX2ludmFsaWRcIilc
biAgICBzYXZlZC5jbGVhcigpXG4gICAgcmV0dXJuIHNlY3JldFxuXG5cbmNsYXNzIEJ1ZGdldDpc
biAgICBcIlwiXCJPbmUgbWFpbi10aHJlYWQgYWxhcm0gYm91bmRzIGJsb2NraW5nIHdvcmsgYW5k
IHNhbXBsZXMgZnJlZSBzcGFjZS5cIlwiXCJcblxuICAgIGRlZiBfX2luaXRfXyhzZWxmLCBsYWIs
IHN0YXJ0ZWQsIHNlY29uZHMpOlxuICAgICAgICBzZWxmLmxhYiwgc2VsZi5kZWFkbGluZSA9IGxh
Yiwgc3RhcnRlZCArIHNlY29uZHNcbiAgICAgICAgc2VsZi5waGFzZV9kZWFkbGluZSA9IHNlbGYu
cGVuZGluZ19kZWFkbGluZSA9IE5vbmVcbiAgICAgICAgc2VsZi5waGFzZV90YWcgPSBcImZpeHR1
cmVfZGVhZGxpbmVcIlxuICAgICAgICBzZWxmLm1pbmltdW1fZnJlZSA9IE5vbmVcbiAgICAgICAg
c2VsZi5zYW1wbGVzID0gMFxuICAgICAgICBzZWxmLnByZXZpb3VzID0ge31cblxuICAgIGRlZiBf
X2VudGVyX18oc2VsZik6XG4gICAgICAgIHJlcXVpcmUoaGFzYXR0cihzaWduYWwsIFwic2V0aXRp
bWVyXCIpIGFuZCBoYXNhdHRyKHNpZ25hbCwgXCJTSUdBTFJNXCIpXG4gICAgICAgICAgICAgICAg
YW5kIHNpZ25hbC5nZXRpdGltZXIoc2lnbmFsLklUSU1FUl9SRUFMKSA9PSAoMC4wLCAwLjApLFxu
ICAgICAgICAgICAgICAgIFwiYXJndW1lbnRzX2ludmFsaWRcIilcbiAgICAgICAgZm9yIG5hbWUg
aW4gKHNpZ25hbC5TSUdBTFJNLCBzaWduYWwuU0lHSU5ULCBzaWduYWwuU0lHVEVSTSk6XG4gICAg
ICAgICAgICBzZWxmLnByZXZpb3VzW25hbWVdID0gc2lnbmFsLmdldHNpZ25hbChuYW1lKVxuICAg
ICAgICAgICAgc2lnbmFsLnNpZ25hbChuYW1lLCBzZWxmLnRpY2sgaWYgbmFtZSA9PSBzaWduYWwu
U0lHQUxSTSBlbHNlIHNlbGYuaW50ZXJydXB0KVxuICAgICAgICBzZWxmLnRpY2soKVxuICAgICAg
ICByZXR1cm4gc2VsZlxuXG4gICAgZGVmIGludGVycnVwdChzZWxmLCBzaWdudW0sIGZyYW1lKTpc
biAgICAgICAgcmFpc2UgSGFsdChcImludGVycnVwdGVkXCIpXG5cbiAgICBkZWYgdGljayhzZWxm
LCBzaWdudW09Tm9uZSwgZnJhbWU9Tm9uZSk6XG4gICAgICAgIHRyeTpcbiAgICAgICAgICAgIGZy
ZWUgPSBzaHV0aWwuZGlza191c2FnZShzZWxmLmxhYikuZnJlZVxuICAgICAgICBleGNlcHQgRXhj
ZXB0aW9uOlxuICAgICAgICAgICAgcmFpc2UgSGFsdChcImRpc2tfb2JzZXJ2YXRpb25fZmFpbGVk
XCIpIGZyb20gTm9uZVxuICAgICAgICBzZWxmLnNhbXBsZXMgKz0gMVxuICAgICAgICBzZWxmLm1p
bmltdW1fZnJlZSA9IGZyZWUgaWYgc2VsZi5taW5pbXVtX2ZyZWUgaXMgTm9uZSBlbHNlIG1pbihz
ZWxmLm1pbmltdW1fZnJlZSwgZnJlZSlcbiAgICAgICAgcmVxdWlyZV9mcmVlID0gZnJlZSA+PSBT
VE9QX0ZSRUVfQllURVNcbiAgICAgICAgaWYgbm90IHJlcXVpcmVfZnJlZTpcbiAgICAgICAgICAg
IHJhaXNlIEhhbHQoXCJkaXNrX21hcmdpblwiKVxuICAgICAgICBub3cgPSB0aW1lLm1vbm90b25p
YygpXG4gICAgICAgIGRlYWRsaW5lcyA9IFsoc2VsZi5kZWFkbGluZSwgXCJmaXh0dXJlX2RlYWRs
aW5lXCIpXVxuICAgICAgICBpZiBzZWxmLnBoYXNlX2RlYWRsaW5lIGlzIG5vdCBOb25lOlxuICAg
ICAgICAgICAgZGVhZGxpbmVzLmFwcGVuZCgoc2VsZi5waGFzZV9kZWFkbGluZSwgc2VsZi5waGFz
ZV90YWcpKVxuICAgICAgICBpZiBzZWxmLnBlbmRpbmdfZGVhZGxpbmUgaXMgbm90IE5vbmU6XG4g
ICAgICAgICAgICBkZWFkbGluZXMuYXBwZW5kKChzZWxmLnBlbmRpbmdfZGVhZGxpbmUsIFwicGVu
ZGluZ19kZWFkbGluZVwiKSlcbiAgICAgICAgZGVhZGxpbmUsIHRhZyA9IG1pbihkZWFkbGluZXMp
XG4gICAgICAgIGlmIG5vdyA+PSBkZWFkbGluZTpcbiAgICAgICAgICAgIHJhaXNlIEhhbHQodGFn
KVxuICAgICAgICBzaWduYWwuc2V0aXRpbWVyKHNpZ25hbC5JVElNRVJfUkVBTCwgbWluKDEuMCwg
ZGVhZGxpbmUgLSBub3cpKVxuXG4gICAgQGNvbnRleHRsaWIuY29udGV4dG1hbmFnZXJcbiAgICBk
ZWYgbGltaXQoc2VsZiwgc2Vjb25kcywgdGFnKTpcbiAgICAgICAgb2xkID0gc2VsZi5waGFzZV9k
ZWFkbGluZSwgc2VsZi5waGFzZV90YWdcbiAgICAgICAgZGVhZGxpbmUgPSB0aW1lLm1vbm90b25p
YygpICsgc2Vjb25kc1xuICAgICAgICBpZiBvbGRbMF0gaXMgTm9uZSBvciBkZWFkbGluZSA8IG9s
ZFswXTpcbiAgICAgICAgICAgIHNlbGYucGhhc2VfZGVhZGxpbmUsIHNlbGYucGhhc2VfdGFnID0g
ZGVhZGxpbmUsIHRhZ1xuICAgICAgICB0cnk6XG4gICAgICAgICAgICBzZWxmLnRpY2soKVxuICAg
ICAgICAgICAgeWllbGRcbiAgICAgICAgZmluYWxseTpcbiAgICAgICAgICAgIHNlbGYucGhhc2Vf
ZGVhZGxpbmUsIHNlbGYucGhhc2VfdGFnID0gb2xkXG4gICAgICAgICAgICAjIERvIG5vdCByZXBs
YWNlIGFuIGluLWZsaWdodCBIYWx0IHdpdGggYW5vdGhlciBhbGFybSBkdXJpbmcgdW53aW5kaW5n
LlxuICAgICAgICAgICAgaWYgc3lzLmV4Y19pbmZvKClbMF0gaXMgTm9uZTpcbiAgICAgICAgICAg
ICAgICBzZWxmLnRpY2soKVxuXG4gICAgZGVmIGNsb3NlKHNlbGYpOlxuICAgICAgICBpZiBzZWxm
LnByZXZpb3VzOlxuICAgICAgICAgICAgc2lnbmFsLnNldGl0aW1lcihzaWduYWwuSVRJTUVSX1JF
QUwsIDApXG4gICAgICAgICAgICBmb3IgbmFtZSwgaGFuZGxlciBpbiBzZWxmLnByZXZpb3VzLml0
ZW1zKCk6XG4gICAgICAgICAgICAgICAgc2lnbmFsLnNpZ25hbChuYW1lLCBoYW5kbGVyKVxuICAg
ICAgICAgICAgc2VsZi5wcmV2aW91cy5jbGVhcigpXG5cblxuY2xhc3MgRGVtbzpcbiAgICBkZWYg
X19pbml0X18oc2VsZiwgbW9kdWxlLCB3b3Jrc3BhY2UsIHNlY3JldCwgYnVkZ2V0LCBjaGVja3Ms
IHN0YXR1c2VzKTpcbiAgICAgICAgc2VsZi5tb2R1bGUsIHNlbGYud29ya3NwYWNlLCBzZWxmLnNl
Y3JldCA9IG1vZHVsZSwgd29ya3NwYWNlLCBzZWNyZXRcbiAgICAgICAgc2VsZi5pc3N1ZXIsIHNl
bGYuY2xpZW50X2lkID0gSVNTVUVSLCBDTElFTlRfSURcbiAgICAgICAgc2VsZi5idWRnZXQsIHNl
bGYuY2hlY2tzID0gYnVkZ2V0LCBjaGVja3NcbiAgICAgICAgc2VsZi5zdGF0dXNlcyA9IHN0YXR1
c2VzXG4gICAgICAgIHNlbGYuc3RhZ2UgPSBcInNldHVwXCJcbiAgICAgICAgc2VsZi5wZW5kaW5n
ID0gc2VsZi5jb29raWUgPSBzZWxmLnN1YmplY3QgPSBOb25lXG4gICAgICAgIHNlbGYuYXR0ZW1w
dGVkID0gc2VsZi5kb25lID0gRmFsc2VcbiAgICAgICAgc2VsZi5mYWlsdXJlID0gTm9uZVxuICAg
ICAgICBzZWxmLnJlcXVlc3RfaW52YWxpZF9yZWFzb24gPSBOb25lXG4gICAgICAgIHNlbGYucHJl
Zmxvd19hdXRob3JpemF0aW9uX3JlZnVzYWxzID0gMFxuXG4gICAgZGVmIGludm9rZShzZWxmLCBz
dGFnZSwgc2Vjb25kcywgZnVuY3Rpb24sICphcmdzLCAqKmt3YXJncyk6XG4gICAgICAgIHNlbGYu
c3RhZ2UgPSBzdGFnZVxuICAgICAgICB0cnk6XG4gICAgICAgICAgICB3aXRoIHNlbGYuYnVkZ2V0
LmxpbWl0KHNlY29uZHMsIHN0YWdlICsgXCJfZmFpbGVkXCIpOlxuICAgICAgICAgICAgICAgIHJl
dHVybiBmdW5jdGlvbigqYXJncywgKiprd2FyZ3MpXG4gICAgICAgIGV4Y2VwdCBGYWlsdXJlOlxu
ICAgICAgICAgICAgcmFpc2VcbiAgICAgICAgZXhjZXB0IEV4Y2VwdGlvbjpcbiAgICAgICAgICAg
ICMgSW1wb3J0ZWQgZmFpbHVyZXMgYXJlIG5ldmVyIGZvcm1hdHRlZCBvciBjb3BpZWQgaW50byB0
aGUgcmVjb3JkLlxuICAgICAgICAgICAgcmFpc2UgRmFpbHVyZShzdGFnZSArIFwiX2ZhaWxlZFwi
KSBmcm9tIE5vbmVcblxuICAgIGRlZiBzZXR1cChzZWxmLCBleGVjdXRhYmxlKTpcbiAgICAgICAg
ZGVmIHByb3ZpZGVyKCk6XG4gICAgICAgICAgICBzZWxmLm9wZW5zc2wgPSBQYXRoKGV4ZWN1dGFi
bGUpLnJlc29sdmUoc3RyaWN0PVRydWUpXG4gICAgICAgICAgICBpbmZvID0gc2VsZi5vcGVuc3Ns
LnN0YXQoKVxuICAgICAgICAgICAgcmVxdWlyZShzdHIoUGF0aChleGVjdXRhYmxlKSkgPT0gXCIv
b3B0L2hvbWVicmV3L2Jpbi9vcGVuc3NsXCJcbiAgICAgICAgICAgICAgICAgICAgYW5kIHN0YXQu
U19JU1JFRyhpbmZvLnN0X21vZGUpIGFuZCAwIDwgaW5mby5zdF9zaXplIDw9IDMyICogMTAyNCAq
IDEwMjQsXG4gICAgICAgICAgICAgICAgICAgIFwicHJvdmlkZXJfZmFpbGVkXCIpXG4gICAgICAg
ICAgICB3aXRoIHNlbGYub3BlbnNzbC5vcGVuKFwicmJcIikgYXMgc291cmNlOlxuICAgICAgICAg
ICAgICAgIHJlcXVpcmUoaGFzaGxpYi5maWxlX2RpZ2VzdChzb3VyY2UsIFwic2hhMjU2XCIpLmhl
eGRpZ2VzdCgpID09IE9QRU5TU0xfU0hBMjU2LFxuICAgICAgICAgICAgICAgICAgICAgICAgXCJw
cm92aWRlcl9mYWlsZWRcIilcbiAgICAgICAgICAgIHNlbGYub3BlbnNzbF9lbnZpcm9ubWVudCA9
IHtrZXk6IHZhbHVlIGZvciBrZXksIHZhbHVlIGluIG9zLmVudmlyb24uaXRlbXMoKVxuICAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgIGlmIGtleSBpbiB7XCJQQVRIXCIsIFwi
SE9NRVwiLCBcIlRNUERJUlwiLCBcIkxBTkdcIiwgXCJMQ19BTExcIn19XG4gICAgICAgICAgICBy
ZXN1bHQgPSBzdWJwcm9jZXNzLnJ1bihbc3RyKHNlbGYub3BlbnNzbCksIFwidmVyc2lvblwiXSwg
Y2FwdHVyZV9vdXRwdXQ9VHJ1ZSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg
IHRpbWVvdXQ9NSwgZW52PXNlbGYub3BlbnNzbF9lbnZpcm9ubWVudClcbiAgICAgICAgICAgIHJl
cXVpcmUocmVzdWx0LnJldHVybmNvZGUgPT0gMFxuICAgICAgICAgICAgICAgICAgICBhbmQgbGVu
KHJlc3VsdC5zdGRvdXQpIDw9IDI1NiBhbmQgbGVuKHJlc3VsdC5zdGRlcnIpIDw9IDQwOTZcbiAg
ICAgICAgICAgICAgICAgICAgYW5kIHJlc3VsdC5zdGRvdXQuZGVjb2RlKFwiYXNjaWlcIikuc3Ry
aXAoKSA9PSBPUEVOU1NMX1ZFUlNJT04sXG4gICAgICAgICAgICAgICAgICAgIFwicHJvdmlkZXJf
ZmFpbGVkXCIpXG4gICAgICAgIHNlbGYuaW52b2tlKFwicHJvdmlkZXJcIiwgNSwgcHJvdmlkZXIp
XG4gICAgICAgIHNlbGYuY2hlY2tzW1wicHJvdmlkZXJfaWRlbnRpdHlfdmVyaWZpZWRcIl0gPSBU
cnVlXG4gICAgICAgIGRvY3VtZW50ID0gc2VsZi5pbnZva2UoXCJkaXNjb3ZlcnlcIiwgNSxcbiAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgICBzZWxmLm1vZHVsZS5Mb2NhbFJlbHlpbmdQYXJ0
eS5kaXNjb3ZlcnksIHNlbGYpXG4gICAgICAgIG1ldGhvZHMgPSBkb2N1bWVudC5nZXQoXCJ0b2tl
bl9lbmRwb2ludF9hdXRoX21ldGhvZHNfc3VwcG9ydGVkXCIpXG4gICAgICAgIHJlcXVpcmUoaXNp
bnN0YW5jZShtZXRob2RzLCBsaXN0KSBhbmQgXCJjbGllbnRfc2VjcmV0X3Bvc3RcIiBpbiBtZXRo
b2RzXG4gICAgICAgICAgICAgICAgYW5kIGFsbChpc2luc3RhbmNlKHZhbHVlLCBzdHIpIGZvciB2
YWx1ZSBpbiBtZXRob2RzKSwgXCJkaXNjb3ZlcnlfZmFpbGVkXCIpXG4gICAgICAgIHNlbGYuZGlz
Y292ZXJ5X2RvY3VtZW50ID0gZG9jdW1lbnRcbiAgICAgICAgc2VsZi5jaGVja3NbXCJkaXNjb3Zl
cnlfdmVyaWZpZWRcIl0gPSBUcnVlXG5cbiAgICBkZWYgYmVnaW4oc2VsZik6XG4gICAgICAgIHNl
bGYuc3RhZ2UgPSBcImZsb3dcIlxuICAgICAgICByZXF1aXJlKG5vdCBzZWxmLmF0dGVtcHRlZCwg
XCJmbG93X2FscmVhZHlfc3RhcnRlZFwiKVxuICAgICAgICBzZWxmLmF0dGVtcHRlZCA9IFRydWVc
biAgICAgICAgc2VsZi5wZW5kaW5nID0ge1wic3RhdGVcIjogc2VjcmV0cy50b2tlbl91cmxzYWZl
KDMyKSwgXCJub25jZVwiOiBzZWNyZXRzLnRva2VuX3VybHNhZmUoMzIpLFxuICAgICAgICAgICAg
ICAgICAgICAgICAgXCJ2ZXJpZmllclwiOiBzZWNyZXRzLnRva2VuX3VybHNhZmUoNDgpLFxuICAg
ICAgICAgICAgICAgICAgICAgICAgXCJmbG93X2Nvb2tpZVwiOiBzZWNyZXRzLnRva2VuX3VybHNh
ZmUoMzIpLFxuICAgICAgICAgICAgICAgICAgICAgICAgXCJzdGFydGVkX2F0XCI6IGludCh0aW1l
LnRpbWUoKSl9XG4gICAgICAgIHNlbGYuYnVkZ2V0LnBlbmRpbmdfZGVhZGxpbmUgPSB0aW1lLm1v
bm90b25pYygpICsgUEVORElOR19TRUNPTkRTXG4gICAgICAgIHNlbGYuYnVkZ2V0LnRpY2soKVxu
ICAgICAgICBmaWVsZHMgPSB7XCJyZXNwb25zZV90eXBlXCI6IFwiY29kZVwiLCBcImNsaWVudF9p
ZFwiOiBDTElFTlRfSUQsXG4gICAgICAgICAgICAgICAgICBcInJlZGlyZWN0X3VyaVwiOiBDQUxM
QkFDSywgXCJzY29wZVwiOiBcIm9wZW5pZCBwcm9maWxlXCIsXG4gICAgICAgICAgICAgICAgICBc
InN0YXRlXCI6IHNlbGYucGVuZGluZ1tcInN0YXRlXCJdLCBcIm5vbmNlXCI6IHNlbGYucGVuZGlu
Z1tcIm5vbmNlXCJdLFxuICAgICAgICAgICAgICAgICAgXCJjb2RlX2NoYWxsZW5nZVwiOiBzZWxm
Lm1vZHVsZS5iNjR1cmwoXG4gICAgICAgICAgICAgICAgICAgICAgaGFzaGxpYi5zaGEyNTYoc2Vs
Zi5wZW5kaW5nW1widmVyaWZpZXJcIl0uZW5jb2RlKFwiYXNjaWlcIikpLmRpZ2VzdCgpKSxcbiAg
ICAgICAgICAgICAgICAgIFwiY29kZV9jaGFsbGVuZ2VfbWV0aG9kXCI6IFwiUzI1NlwifVxuICAg
ICAgICByZXR1cm4gKHNlbGYuZGlzY292ZXJ5X2RvY3VtZW50W1wiYXV0aG9yaXphdGlvbl9lbmRw
b2ludFwiXSArIFwiP1wiXG4gICAgICAgICAgICAgICAgKyB1cmxsaWIucGFyc2UudXJsZW5jb2Rl
KGZpZWxkcyksIHNlbGYucGVuZGluZ1tcImZsb3dfY29va2llXCJdKVxuXG4gICAgZGVmIGNhbGxi
YWNrKHNlbGYsIHF1ZXJ5LCBmbG93X2Nvb2tpZSk6XG4gICAgICAgIHNlbGYuc3RhZ2UgPSBcImNh
bGxiYWNrXCJcbiAgICAgICAgcmVxdWlyZShzZWxmLnBlbmRpbmcgaXMgbm90IE5vbmUsIFwiY2Fs
bGJhY2tfYWxyZWFkeV9jb25zdW1lZFwiKVxuICAgICAgICBwZW5kaW5nLCBzZWxmLnBlbmRpbmcg
PSBzZWxmLnBlbmRpbmcsIE5vbmVcbiAgICAgICAgc2VsZi5idWRnZXQucGVuZGluZ19kZWFkbGlu
ZSA9IE5vbmVcbiAgICAgICAgcmF3ID0gdG9rZW5zID0gYWNjZXNzID0gc3ViamVjdCA9IGNvZGUg
PSBmb3JtID0gTm9uZVxuICAgICAgICB0cnk6XG4gICAgICAgICAgICByZXF1aXJlKHNlbGYubW9k
dWxlLmVxdWFsKGZsb3dfY29va2llLCBwZW5kaW5nW1wiZmxvd19jb29raWVcIl0pLCBcImNhbGxi
YWNrX2ludmFsaWRcIilcbiAgICAgICAgICAgIHRyeTpcbiAgICAgICAgICAgICAgICBjb2RlID0g
c2VsZi5tb2R1bGUuTG9jYWxSZWx5aW5nUGFydHkuY2FsbGJhY2tfZmllbGRzKHNlbGYsIHF1ZXJ5
LCBwZW5kaW5nW1wic3RhdGVcIl0pXG4gICAgICAgICAgICBleGNlcHQgRXhjZXB0aW9uOlxuICAg
ICAgICAgICAgICAgIHJhaXNlIEZhaWx1cmUoXCJjYWxsYmFja19pbnZhbGlkXCIpIGZyb20gTm9u
ZVxuICAgICAgICAgICAgc2VsZi5jaGVja3NbXCJzdGF0ZV9pc3N1ZXJfZmxvd19jb29raWVfdmVy
aWZpZWRcIl0gPSBUcnVlXG4gICAgICAgICAgICBmb3JtID0ge1wiZ3JhbnRfdHlwZVwiOiBcImF1
dGhvcml6YXRpb25fY29kZVwiLCBcImNsaWVudF9pZFwiOiBDTElFTlRfSUQsXG4gICAgICAgICAg
ICAgICAgICAgIFwiY2xpZW50X3NlY3JldFwiOiBzZWxmLnNlY3JldCwgXCJyZWRpcmVjdF91cmlc
IjogQ0FMTEJBQ0ssXG4gICAgICAgICAgICAgICAgICAgIFwiY29kZVwiOiBjb2RlLCBcImNvZGVf
dmVyaWZpZXJcIjogcGVuZGluZ1tcInZlcmlmaWVyXCJdfVxuICAgICAgICAgICAgc3RhdHVzLCBf
LCByYXcgPSBzZWxmLmludm9rZShcInRva2VuX2V4Y2hhbmdlXCIsIDUsIHNlbGYubW9kdWxlLnJl
cXVlc3QsXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgIHNlbGYuZGlz
Y292ZXJ5X2RvY3VtZW50W1widG9rZW5fZW5kcG9pbnRcIl0sXG4gICAgICAgICAgICAgICAgICAg
ICAgICAgICAgICAgICAgICAgICAgIG1ldGhvZD1cIlBPU1RcIiwgZm9ybT1mb3JtKVxuICAgICAg
ICAgICAgc2VsZi5zdGF0dXNlc1tcInRva2VuX2V4Y2hhbmdlXCJdID0gc3RhdHVzXG4gICAgICAg
ICAgICByZXF1aXJlKHN0YXR1cyA9PSAyMDAsIFwidG9rZW5fZXhjaGFuZ2VfZmFpbGVkXCIpXG4g
ICAgICAgICAgICB0b2tlbnMgPSBzZWxmLm1vZHVsZS5qc29uX29iamVjdChyYXcpXG4gICAgICAg
ICAgICBhY2Nlc3MsIHNjb3BlID0gdG9rZW5zLmdldChcImFjY2Vzc190b2tlblwiKSwgdG9rZW5z
LmdldChcInNjb3BlXCIpXG4gICAgICAgICAgICByZXF1aXJlKGlzaW5zdGFuY2UoYWNjZXNzLCBz
dHIpIGFuZCAwIDwgbGVuKGFjY2VzcykgPD0gc2VsZi5tb2R1bGUuTUFYX1RPS0VOXG4gICAgICAg
ICAgICAgICAgICAgIGFuZCByZS5mdWxsbWF0Y2goclwiW0EtWmEtejAtOV8uLV0rXCIsIGFjY2Vz
cykgaXMgbm90IE5vbmVcbiAgICAgICAgICAgICAgICAgICAgYW5kIHRva2Vucy5nZXQoXCJ0b2tl
bl90eXBlXCIpID09IFwiQmVhcmVyXCIgYW5kIGlzaW5zdGFuY2Uoc2NvcGUsIHN0cilcbiAgICAg
ICAgICAgICAgICAgICAgYW5kIHNldChzY29wZS5zcGxpdCgpKSA9PSB7XCJvcGVuaWRcIiwgXCJw
cm9maWxlXCJ9LCBcInRva2VuX2V4Y2hhbmdlX2ZhaWxlZFwiKVxuICAgICAgICAgICAgc2VsZi5j
aGVja3NbXCJjb25maWRlbnRpYWxfczI1Nl9leGNoYW5nZV92ZXJpZmllZFwiXSA9IFRydWVcbiAg
ICAgICAgICAgICMgS2VlcCB0aGUgcGlubmVkIGltcGxlbWVudGF0aW9uOyBjYXAgY29tYmluZWQg
SldLUy9zaWduYXR1cmUgd29ya1xuICAgICAgICAgICAgIyBhdCBmaXZlIHJlYWwgc2Vjb25kcyBh
cyB3ZWxsIGFzIGl0cyBleGlzdGluZyBzb2NrZXQvcHJvY2VzcyBjYXBzLlxuICAgICAgICAgICAg
c3ViamVjdCA9IHNlbGYuaW52b2tlKFwidG9rZW5fdmFsaWRhdGlvblwiLCA1LFxuICAgICAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgIHNlbGYubW9kdWxlLkxvY2FsUmVseWluZ1BhcnR5LnZl
cmlmeV9pZF90b2tlbixcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBzZWxmLCB0
b2tlbnMuZ2V0KFwiaWRfdG9rZW5cIiksIGFjY2VzcywgcGVuZGluZyxcbiAgICAgICAgICAgICAg
ICAgICAgICAgICAgICAgICAgICBzZWxmLmRpc2NvdmVyeV9kb2N1bWVudClcbiAgICAgICAgICAg
IHNlbGYuY2hlY2tzW1wicnMyNTZfandrc19pc3N1ZXJfYXVkaWVuY2Vfbm9uY2VfdGltZV9hY2Nl
c3NfaGFzaF92ZXJpZmllZFwiXSA9IFRydWVcbiAgICAgICAgICAgIHN0YXR1cywgXywgcmF3ID0g
c2VsZi5pbnZva2UoXCJ1c2VyaW5mb1wiLCA1LCBzZWxmLm1vZHVsZS5yZXF1ZXN0LFxuICAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBzZWxmLmRpc2NvdmVyeV9kb2N1bWVu
dFtcInVzZXJpbmZvX2VuZHBvaW50XCJdLCBiZWFyZXI9YWNjZXNzKVxuICAgICAgICAgICAgc2Vs
Zi5zdGF0dXNlc1tcInVzZXJpbmZvXCJdID0gc3RhdHVzXG4gICAgICAgICAgICByZXF1aXJlKHN0
YXR1cyA9PSAyMDAgYW5kIHNlbGYubW9kdWxlLmVxdWFsKHNlbGYubW9kdWxlLmpzb25fb2JqZWN0
KHJhdykuZ2V0KFwic3ViXCIpLCBzdWJqZWN0KSxcbiAgICAgICAgICAgICAgICAgICAgXCJ1c2Vy
aW5mb19mYWlsZWRcIilcbiAgICAgICAgICAgIHNlbGYuY2hlY2tzW1widXNlcmluZm9fc3ViamVj
dF92ZXJpZmllZFwiXSA9IFRydWVcbiAgICAgICAgICAgIHNlbGYuY29va2llLCBzZWxmLnN1Ympl
Y3QgPSBzZWNyZXRzLnRva2VuX3VybHNhZmUoMzIpLCBzdWJqZWN0XG4gICAgICAgICAgICByZXR1
cm4gc2VsZi5jb29raWVcbiAgICAgICAgZmluYWxseTpcbiAgICAgICAgICAgIHBlbmRpbmcuY2xl
YXIoKVxuICAgICAgICAgICAgaWYgZm9ybSBpcyBub3QgTm9uZTpcbiAgICAgICAgICAgICAgICBm
b3JtLmNsZWFyKClcbiAgICAgICAgICAgIHJhdyA9IHRva2VucyA9IGFjY2VzcyA9IHN1YmplY3Qg
PSBjb2RlID0gZm9ybSA9IE5vbmVcbiAgICAgICAgICAgIHNlbGYuc2VjcmV0ID0gTm9uZVxuXG4g
ICAgZGVmIGNsZWFyKHNlbGYpOlxuICAgICAgICBpZiBzZWxmLnBlbmRpbmcgaXMgbm90IE5vbmU6
XG4gICAgICAgICAgICBzZWxmLnBlbmRpbmcuY2xlYXIoKVxuICAgICAgICBzZWxmLnBlbmRpbmcg
PSBzZWxmLmNvb2tpZSA9IHNlbGYuc3ViamVjdCA9IHNlbGYuc2VjcmV0ID0gTm9uZVxuICAgICAg
ICBzZWxmLmJ1ZGdldC5wZW5kaW5nX2RlYWRsaW5lID0gTm9uZVxuXG5cbmNsYXNzIEhlYWRlclJl
YWRlcjpcbiAgICBkZWYgX19pbml0X18oc2VsZiwgc291cmNlKTpcbiAgICAgICAgc2VsZi5zb3Vy
Y2UsIHNlbGYucmVtYWluaW5nLCBzZWxmLmxpbmVzID0gc291cmNlLCBNQVhfSEVBREVSUywgMFxu
XG4gICAgZGVmIHJlYWRsaW5lKHNlbGYsIHNpemU9LTEpOlxuICAgICAgICBsaW5lID0gc2VsZi5z
b3VyY2UucmVhZGxpbmUobWluKE1BWF9IRUFERVJfTElORSArIDEsIHNlbGYucmVtYWluaW5nICsg
MSkpXG4gICAgICAgIHNlbGYucmVtYWluaW5nIC09IGxlbihsaW5lKVxuICAgICAgICBzZWxmLmxp
bmVzICs9IDFcbiAgICAgICAgcmVxdWlyZShsZW4obGluZSkgPD0gTUFYX0hFQURFUl9MSU5FIGFu
ZCBzZWxmLnJlbWFpbmluZyA+PSAwXG4gICAgICAgICAgICAgICAgYW5kIHNlbGYubGluZXMgPD0g
TUFYX0hFQURFUl9DT1VOVCArIDEsIFwicmVxdWVzdF9saW1pdFwiKVxuICAgICAgICByZXR1cm4g
bGluZVxuXG5cbmNsYXNzIERlbW9TZXJ2ZXIoaHR0cC5zZXJ2ZXIuSFRUUFNlcnZlcik6XG4gICAg
ZGVmIF9faW5pdF9fKHNlbGYsIGRlbW8pOlxuICAgICAgICBzZWxmLmRlbW8sIHNlbGYuYWN0aXZl
ID0gZGVtbywgTm9uZVxuICAgICAgICBzdXBlcigpLl9faW5pdF9fKChcIjEyNy4wLjAuMVwiLCAz
MDAwKSwgSGFuZGxlcilcbiAgICAgICAgc2VsZi50aW1lb3V0ID0gMC4yXG5cbiAgICBkZWYgcHJv
Y2Vzc19yZXF1ZXN0KHNlbGYsIHJlcXVlc3QsIGFkZHJlc3MpOlxuICAgICAgICBzZWxmLmFjdGl2
ZSA9IHJlcXVlc3RcbiAgICAgICAgdHJ5OlxuICAgICAgICAgICAgc2VsZi5maW5pc2hfcmVxdWVz
dChyZXF1ZXN0LCBhZGRyZXNzKVxuICAgICAgICBmaW5hbGx5OlxuICAgICAgICAgICAgc2VsZi5z
aHV0ZG93bl9yZXF1ZXN0KHJlcXVlc3QpXG4gICAgICAgICAgICBzZWxmLmFjdGl2ZSA9IE5vbmVc
blxuICAgIGRlZiBoYW5kbGVfZXJyb3Ioc2VsZiwgcmVxdWVzdCwgYWRkcmVzcyk6XG4gICAgICAg
IHNlbGYuZGVtby5mYWlsdXJlLCBzZWxmLmRlbW8uZG9uZSA9IFwidW5leHBlY3RlZF9mYWlsdXJl
XCIsIFRydWVcblxuXG5jbGFzcyBQcmVmbG93QXV0aG9yaXphdGlvblJlZnVzYWwoRXhjZXB0aW9u
KTpcbiAgICBwYXNzXG5cblxuY2xhc3MgSGFuZGxlcihodHRwLnNlcnZlci5CYXNlSFRUUFJlcXVl
c3RIYW5kbGVyKTpcbiAgICB0aW1lb3V0ID0gNVxuICAgIHByb3RvY29sX3ZlcnNpb24gPSBcIkhU
VFAvMS4wXCJcblxuICAgIGRlZiBsb2dfbWVzc2FnZShzZWxmLCBmb3JtYXQsICphcmdzKTpcbiAg
ICAgICAgcGFzc1xuXG4gICAgZGVmIHJlY29yZF9yZXF1ZXN0X3JlYXNvbihzZWxmLCByZWFzb24p
OlxuICAgICAgICBkZW1vID0gc2VsZi5zZXJ2ZXIuZGVtb1xuICAgICAgICBpZiBkZW1vLnJlcXVl
c3RfaW52YWxpZF9yZWFzb24gaXMgTm9uZSBhbmQgcmVhc29uIGluIFJFUVVFU1RfSU5WQUxJRF9S
RUFTT05TOlxuICAgICAgICAgICAgZGVtby5yZXF1ZXN0X2ludmFsaWRfcmVhc29uID0gcmVhc29u
XG5cbiAgICBkZWYgcmVxdWlyZV9yZXF1ZXN0KHNlbGYsIGNvbmRpdGlvbiwgcmVhc29uKTpcbiAg
ICAgICAgaWYgbm90IGNvbmRpdGlvbjpcbiAgICAgICAgICAgIHNlbGYucmVjb3JkX3JlcXVlc3Rf
cmVhc29uKHJlYXNvbilcbiAgICAgICAgICAgIHJhaXNlIEZhaWx1cmUoXCJyZXF1ZXN0X2ludmFs
aWRcIilcblxuICAgIGRlZiBzZW5kX2Vycm9yKHNlbGYsIGNvZGUsIG1lc3NhZ2U9Tm9uZSwgZXhw
bGFpbj1Ob25lKTpcbiAgICAgICAgc2VsZi5yZWNvcmRfcmVxdWVzdF9yZWFzb24oXCJodHRwX3Bh
cnNlXCIpXG4gICAgICAgIHNlbGYuc2VydmVyLmRlbW8uZmFpbHVyZSwgc2VsZi5zZXJ2ZXIuZGVt
by5kb25lID0gXCJyZXF1ZXN0X2ludmFsaWRcIiwgVHJ1ZVxuICAgICAgICBzZWxmLnJlcGx5KGNv
ZGUsIFwiTG9jYWwgZGVtbyBjb3VsZCBub3QgY29tcGxldGUgdGhpcyByZXF1ZXN0LlwiKVxuXG4g
ICAgZGVmIGhhbmRsZV9vbmVfcmVxdWVzdChzZWxmKTpcbiAgICAgICAgZGVtbyA9IHNlbGYuc2Vy
dmVyLmRlbW9cbiAgICAgICAgc2VsZi5yZXF1ZXN0bGluZSwgc2VsZi5yZXF1ZXN0X3ZlcnNpb24s
IHNlbGYuY29tbWFuZCA9IFwiXCIsIFwiSFRUUC8xLjBcIiwgTm9uZVxuICAgICAgICBzZWxmLmNs
b3NlX2Nvbm5lY3Rpb24gPSBUcnVlXG4gICAgICAgIHRyeTpcbiAgICAgICAgICAgIGRlbW8uc3Rh
Z2UgPSBcInJlcXVlc3RcIlxuICAgICAgICAgICAgd2l0aCBkZW1vLmJ1ZGdldC5saW1pdCg1LCBc
InJlcXVlc3RfdGltZW91dFwiKTpcbiAgICAgICAgICAgICAgICBzZWxmLnJhd19yZXF1ZXN0bGlu
ZSA9IHNlbGYucmZpbGUucmVhZGxpbmUoTUFYX1JFUVVFU1RfTElORSArIDEpXG4gICAgICAgICAg
ICAgICAgaWYgbm90IHNlbGYucmF3X3JlcXVlc3RsaW5lOlxuICAgICAgICAgICAgICAgICAgICBy
ZXR1cm5cbiAgICAgICAgICAgICAgICByZXF1aXJlKGxlbihzZWxmLnJhd19yZXF1ZXN0bGluZSkg
PD0gTUFYX1JFUVVFU1RfTElORSwgXCJyZXF1ZXN0X2xpbWl0XCIpXG4gICAgICAgICAgICAgICAg
b3JpZ2luYWwsIHNlbGYucmZpbGUgPSBzZWxmLnJmaWxlLCBIZWFkZXJSZWFkZXIoc2VsZi5yZmls
ZSlcbiAgICAgICAgICAgICAgICB0cnk6XG4gICAgICAgICAgICAgICAgICAgIGlmIG5vdCBzZWxm
LnBhcnNlX3JlcXVlc3QoKTpcbiAgICAgICAgICAgICAgICAgICAgICAgIHJldHVyblxuICAgICAg
ICAgICAgICAgIGZpbmFsbHk6XG4gICAgICAgICAgICAgICAgICAgIHNlbGYucmZpbGUgPSBvcmln
aW5hbFxuICAgICAgICAgICAgICAgIHNlbGYuY2xvc2VfY29ubmVjdGlvbiA9IFRydWVcbiAgICAg
ICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVzdChzZWxmLmhlYWRlcnMuZ2V0X2FsbChcIkhv
c3RcIikgPT0gW0FVVEhPUklUWV0sIFwiaG9zdFwiKVxuICAgICAgICAgICAgICAgIGlmIChzZWxm
LmhlYWRlcnMuZ2V0X2FsbChcIkF1dGhvcml6YXRpb25cIikgaXMgbm90IE5vbmVcbiAgICAgICAg
ICAgICAgICAgICAgICAgIGFuZCBkZW1vLmF0dGVtcHRlZCBpcyBGYWxzZSBhbmQgZGVtby5wZW5k
aW5nIGlzIE5vbmVcbiAgICAgICAgICAgICAgICAgICAgICAgIGFuZCBkZW1vLmNvb2tpZSBpcyBO
b25lIGFuZCBkZW1vLnN1YmplY3QgaXMgTm9uZVxuICAgICAgICAgICAgICAgICAgICAgICAgYW5k
IGRlbW8ucHJlZmxvd19hdXRob3JpemF0aW9uX3JlZnVzYWxzIDwgNCk6XG4gICAgICAgICAgICAg
ICAgICAgIHNlbGYucmVjb3JkX3JlcXVlc3RfcmVhc29uKFwiYXV0aG9yaXphdGlvblwiKVxuICAg
ICAgICAgICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVzdChzZWxmLmhlYWRlcnMuZ2V0X2Fs
bChcIlRyYW5zZmVyLUVuY29kaW5nXCIpIGlzIE5vbmUsIFwidHJhbnNmZXJfZW5jb2RpbmdcIilc
biAgICAgICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qoc2VsZi5oZWFkZXJzLmdl
dF9hbGwoXCJFeHBlY3RcIikgaXMgTm9uZSwgXCJleHBlY3RcIilcbiAgICAgICAgICAgICAgICAg
ICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qoc2VsZi5oZWFkZXJzLmdldF9hbGwoXCJDb250ZW50LUxl
bmd0aFwiKSBpbiAoTm9uZSwgW1wiMFwiXSksIFwiY29udGVudF9sZW5ndGhcIilcbiAgICAgICAg
ICAgICAgICAgICAgdGFyZ2V0ID0gdXJsbGliLnBhcnNlLnVybHNwbGl0KHNlbGYucGF0aClcbiAg
ICAgICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qobm90IHRhcmdldC5zY2hlbWUs
IFwidGFyZ2V0X3NjaGVtZVwiKVxuICAgICAgICAgICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVx
dWVzdChub3QgdGFyZ2V0Lm5ldGxvYywgXCJ0YXJnZXRfbmV0bG9jXCIpXG4gICAgICAgICAgICAg
ICAgICAgIHNlbGYucmVxdWlyZV9yZXF1ZXN0KG5vdCB0YXJnZXQuZnJhZ21lbnQsIFwidGFyZ2V0
X2ZyYWdtZW50XCIpXG4gICAgICAgICAgICAgICAgICAgIGRlbW8ucHJlZmxvd19hdXRob3JpemF0
aW9uX3JlZnVzYWxzICs9IDFcbiAgICAgICAgICAgICAgICAgICAgcmFpc2UgUHJlZmxvd0F1dGhv
cml6YXRpb25SZWZ1c2FsXG4gICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qoc2Vs
Zi5oZWFkZXJzLmdldF9hbGwoXCJBdXRob3JpemF0aW9uXCIpIGlzIE5vbmUsIFwiYXV0aG9yaXph
dGlvblwiKVxuICAgICAgICAgICAgICAgIHNlbGYucmVxdWlyZV9yZXF1ZXN0KHNlbGYuaGVhZGVy
cy5nZXRfYWxsKFwiVHJhbnNmZXItRW5jb2RpbmdcIikgaXMgTm9uZSwgXCJ0cmFuc2Zlcl9lbmNv
ZGluZ1wiKVxuICAgICAgICAgICAgICAgIHNlbGYucmVxdWlyZV9yZXF1ZXN0KHNlbGYuaGVhZGVy
cy5nZXRfYWxsKFwiRXhwZWN0XCIpIGlzIE5vbmUsIFwiZXhwZWN0XCIpXG4gICAgICAgICAgICAg
ICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qoc2VsZi5oZWFkZXJzLmdldF9hbGwoXCJDb250ZW50LUxl
bmd0aFwiKSBpbiAoTm9uZSwgW1wiMFwiXSksIFwiY29udGVudF9sZW5ndGhcIilcbiAgICAgICAg
ICAgICAgICB0YXJnZXQgPSB1cmxsaWIucGFyc2UudXJsc3BsaXQoc2VsZi5wYXRoKVxuICAgICAg
ICAgICAgICAgIHNlbGYucmVxdWlyZV9yZXF1ZXN0KG5vdCB0YXJnZXQuc2NoZW1lLCBcInRhcmdl
dF9zY2hlbWVcIilcbiAgICAgICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVzdChub3QgdGFy
Z2V0Lm5ldGxvYywgXCJ0YXJnZXRfbmV0bG9jXCIpXG4gICAgICAgICAgICAgICAgc2VsZi5yZXF1
aXJlX3JlcXVlc3Qobm90IHRhcmdldC5mcmFnbWVudCwgXCJ0YXJnZXRfZnJhZ21lbnRcIilcbiAg
ICAgICAgICAgIGlmIHNlbGYuY29tbWFuZCA9PSBcIkdFVFwiOlxuICAgICAgICAgICAgICAgIHNl
bGYuZ2V0KHRhcmdldClcbiAgICAgICAgICAgIGVsaWYgc2VsZi5jb21tYW5kID09IFwiUE9TVFwi
OlxuICAgICAgICAgICAgICAgIHNlbGYucmVxdWlyZV9yZXF1ZXN0KHRhcmdldC5wYXRoID09IFwi
L2xvZ2luXCIgYW5kIG5vdCB0YXJnZXQucXVlcnksIFwicG9zdF90YXJnZXRcIilcbiAgICAgICAg
ICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVzdChzZWxmLmhlYWRlcnMuZ2V0X2FsbChcIk9yaWdp
blwiKSA9PSBbT1JJR0lOXSwgXCJvcmlnaW5cIilcbiAgICAgICAgICAgICAgICBzZWxmLnJlcXVp
cmVfcmVxdWVzdChzZWxmLmhlYWRlcnMuZ2V0X2FsbChcIkNvbnRlbnQtVHlwZVwiKSA9PSBbXCJh
cHBsaWNhdGlvbi94LXd3dy1mb3JtLXVybGVuY29kZWRcIl0sIFwiY29udGVudF90eXBlXCIpXG4g
ICAgICAgICAgICAgICAgc2VsZi5jb29raWVzKClcbiAgICAgICAgICAgICAgICBsb2NhdGlvbiwg
Y29va2llID0gZGVtby5iZWdpbigpXG4gICAgICAgICAgICAgICAgc2VsZi5yZXBseSgzMDMsIFwi
XCIsIGxvY2F0aW9uPWxvY2F0aW9uLCBjb29raWU9KEZMT1dfQ09PS0lFLCBjb29raWUpKVxuICAg
ICAgICAgICAgICAgIGRlbW8uc3RhdHVzZXNbXCJhdXRob3JpemF0aW9uX3JlZGlyZWN0XCJdID0g
MzAzXG4gICAgICAgICAgICAgICAgZGVtby5jaGVja3NbXCJhdXRob3JpemF0aW9uX3JlZGlyZWN0
X2lzc3VlZFwiXSA9IFRydWVcbiAgICAgICAgICAgIGVsc2U6XG4gICAgICAgICAgICAgICAgc2Vs
Zi5yZWNvcmRfcmVxdWVzdF9yZWFzb24oXCJtZXRob2RcIilcbiAgICAgICAgICAgICAgICByYWlz
ZSBGYWlsdXJlKFwicmVxdWVzdF9pbnZhbGlkXCIpXG4gICAgICAgIGV4Y2VwdCBQcmVmbG93QXV0
aG9yaXphdGlvblJlZnVzYWw6XG4gICAgICAgICAgICBpZiBkZW1vLnByZWZsb3dfYXV0aG9yaXph
dGlvbl9yZWZ1c2FscyA9PSA0OlxuICAgICAgICAgICAgICAgIGRlbW8uZmFpbHVyZSwgZGVtby5k
b25lID0gXCJyZXF1ZXN0X2ludmFsaWRcIiwgVHJ1ZVxuICAgICAgICAgICAgc2VsZi5yZXBseSg0
MDMsIFwiTG9jYWwgZGVtbyBjb3VsZCBub3QgY29tcGxldGUgdGhpcyByZXF1ZXN0LlwiKVxuICAg
ICAgICBleGNlcHQgRmFpbHVyZSBhcyBmYWlsdXJlOlxuICAgICAgICAgICAgZGVtby5mYWlsdXJl
LCBkZW1vLmRvbmUgPSBmYWlsdXJlLnRhZywgVHJ1ZVxuICAgICAgICAgICAgc2VsZi5yZXBseSg0
MDAsIFwiTG9jYWwgZGVtbyBjb3VsZCBub3QgY29tcGxldGUgdGhpcyByZXF1ZXN0LlwiKVxuICAg
ICAgICBleGNlcHQgRXhjZXB0aW9uOlxuICAgICAgICAgICAgZGVtby5mYWlsdXJlLCBkZW1vLmRv
bmUgPSBcInVuZXhwZWN0ZWRfZmFpbHVyZVwiLCBUcnVlXG4gICAgICAgICAgICBzZWxmLnJlcGx5
KDQwMCwgXCJMb2NhbCBkZW1vIGNvdWxkIG5vdCBjb21wbGV0ZSB0aGlzIHJlcXVlc3QuXCIpXG5c
biAgICBkZWYgY29va2llcyhzZWxmKTpcbiAgICAgICAgaGVhZGVycyA9IHNlbGYuaGVhZGVycy5n
ZXRfYWxsKFwiQ29va2llXCIsIFtdKVxuICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVzdChsZW4o
aGVhZGVycykgPD0gMSwgXCJjb29raWVfaGVhZGVyX2NvdW50XCIpXG4gICAgICAgIHJhdyA9IGhl
YWRlcnNbMF0gaWYgaGVhZGVycyBlbHNlIFwiXCJcbiAgICAgICAgcmVxdWlyZShsZW4ocmF3KSA8
PSBNQVhfQ09PS0lFLCBcInJlcXVlc3RfbGltaXRcIilcbiAgICAgICAgcGllY2VzID0gcmF3LnNw
bGl0KFwiO1wiKSBpZiByYXcgZWxzZSBbXVxuICAgICAgICByZXF1aXJlKGxlbihwaWVjZXMpIDw9
IE1BWF9IRUFERVJfQ09VTlQsIFwicmVxdWVzdF9saW1pdFwiKVxuICAgICAgICBvd24gPSB7fVxu
ICAgICAgICBmb3IgcGllY2UgaW4gcGllY2VzOlxuICAgICAgICAgICAgbmFtZSwgc2VwYXJhdG9y
LCB2YWx1ZSA9IHBpZWNlLnN0cmlwKCkucGFydGl0aW9uKFwiPVwiKVxuICAgICAgICAgICAgaWYg
bmFtZSBpbiAoRkxPV19DT09LSUUsIEFQUF9DT09LSUUpOlxuICAgICAgICAgICAgICAgIHNlbGYu
cmVxdWlyZV9yZXF1ZXN0KHNlcGFyYXRvciBhbmQgbmFtZSBub3QgaW4gb3duXG4gICAgICAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgYW5kIHJlLmZ1bGxtYXRjaChyXCJbQS1aYS16MC05
Xy1dezQzfVwiLCB2YWx1ZSkgaXMgbm90IE5vbmUsXG4gICAgICAgICAgICAgICAgICAgICAgICAg
ICAgICAgICAgICAgXCJvd25fY29va2llX3NoYXBlXCIpXG4gICAgICAgICAgICAgICAgb3duW25h
bWVdID0gdmFsdWVcbiAgICAgICAgcmV0dXJuIG93blxuXG4gICAgZGVmIGdldChzZWxmLCB0YXJn
ZXQpOlxuICAgICAgICBkZW1vLCBjb29raWVzID0gc2VsZi5zZXJ2ZXIuZGVtbywgc2VsZi5jb29r
aWVzKClcbiAgICAgICAgaWYgdGFyZ2V0LnBhdGggPT0gXCIvY2FsbGJhY2tcIjpcbiAgICAgICAg
ICAgIGNvb2tpZSA9IGRlbW8uY2FsbGJhY2sodGFyZ2V0LnF1ZXJ5LCBjb29raWVzLmdldChGTE9X
X0NPT0tJRSkpXG4gICAgICAgICAgICBzZWxmLnJlcGx5KDMwMywgXCJcIiwgbG9jYXRpb249XCIv
cHJvdGVjdGVkXCIsIGNvb2tpZT0oQVBQX0NPT0tJRSwgY29va2llKSwgY2xlYXJfZmxvdz1UcnVl
KVxuICAgICAgICAgICAgZGVtby5zdGF0dXNlc1tcImNhbGxiYWNrXCJdID0gMzAzXG4gICAgICAg
IGVsaWYgdGFyZ2V0LnBhdGggPT0gXCIvcHJvdGVjdGVkXCIgYW5kIG5vdCB0YXJnZXQucXVlcnk6
XG4gICAgICAgICAgICBkZW1vLnN0YWdlID0gXCJwcm90ZWN0ZWRcIlxuICAgICAgICAgICAgYXV0
aGVudGljYXRlZCA9IChkZW1vLnN1YmplY3QgaXMgbm90IE5vbmUgYW5kIGRlbW8uY29va2llIGlz
IG5vdCBOb25lXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgIGFuZCBkZW1vLm1vZHVsZS5l
cXVhbChjb29raWVzLmdldChBUFBfQ09PS0lFKSwgZGVtby5jb29raWUpKVxuICAgICAgICAgICAg
c2VsZi5yZXBseSgyMDAgaWYgYXV0aGVudGljYXRlZCBlbHNlIDQwMyxcbiAgICAgICAgICAgICAg
ICAgICAgICAgXCJTaWduZWQgaW4uIFByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3MgaXMgYXZh
aWxhYmxlLlwiIGlmIGF1dGhlbnRpY2F0ZWRcbiAgICAgICAgICAgICAgICAgICAgICAgZWxzZSBc
IlNpZ24gaW4gcmVxdWlyZWQuXCIsIHByb3RlY3RlZD1hdXRoZW50aWNhdGVkKVxuICAgICAgICAg
ICAgaWYgYXV0aGVudGljYXRlZDpcbiAgICAgICAgICAgICAgICBkZW1vLmNoZWNrc1tcInByb3Rl
Y3RlZF93aXRoX2ZyZXNoX2Nvb2tpZV9hY2NlcHRlZFwiXSA9IFRydWVcbiAgICAgICAgICAgICAg
ICBkZW1vLnN0YXR1c2VzW1wicHJvdGVjdGVkX2FmdGVyXCJdID0gMjAwXG4gICAgICAgICAgICAg
ICAgaWYgbm90IGRlbW8uY2hlY2tzW1wicHJvdGVjdGVkX3dpdGhvdXRfY29va2llX2RlbmllZFwi
XTpcbiAgICAgICAgICAgICAgICAgICAgZGVtby5mYWlsdXJlID0gXCJwcm90ZWN0ZWRfYmVmb3Jl
X3Vub2JzZXJ2ZWRcIlxuICAgICAgICAgICAgICAgIGRlbW8uZG9uZSA9IFRydWVcbiAgICAgICAg
ICAgIGVsaWYgbm90IGRlbW8uYXR0ZW1wdGVkIGFuZCBBUFBfQ09PS0lFIG5vdCBpbiBjb29raWVz
OlxuICAgICAgICAgICAgICAgIGRlbW8uY2hlY2tzW1wicHJvdGVjdGVkX3dpdGhvdXRfY29va2ll
X2RlbmllZFwiXSA9IFRydWVcbiAgICAgICAgICAgICAgICBkZW1vLnN0YXR1c2VzW1wicHJvdGVj
dGVkX2JlZm9yZVwiXSA9IDQwM1xuICAgICAgICBlbGlmIHRhcmdldC5wYXRoID09IFwiL1wiIGFu
ZCBub3QgdGFyZ2V0LnF1ZXJ5OlxuICAgICAgICAgICAgc2VsZi5yZXBseSgyMDAsIFwiVXNlIFNp
Z24gaW4gdG8gb3BlbiB0aGlzIGxvY2FsIGFwcGxpY2F0aW9uLlwiLCBzdGFydD1UcnVlKVxuICAg
ICAgICBlbHNlOlxuICAgICAgICAgICAgc2VsZi5yZXBseSg0MDQsIFwiVGhpcyBwYWdlIGlzIHVu
YXZhaWxhYmxlLlwiKVxuXG4gICAgZGVmIHJlcGx5KHNlbGYsIHN0YXR1cywgdGV4dCwgKiwgbG9j
YXRpb249Tm9uZSwgY29va2llPU5vbmUsIGNsZWFyX2Zsb3c9RmFsc2UsXG4gICAgICAgICAgICAg
IHN0YXJ0PUZhbHNlLCBwcm90ZWN0ZWQ9RmFsc2UpOlxuICAgICAgICBoZWFkaW5nID0gXCJQcm90
ZWN0ZWQgYXBwbGljYXRpb24gYWNjZXNzXCIgaWYgcHJvdGVjdGVkIGVsc2UgXCJMb2NhbCBkZW1v
XCJcbiAgICAgICAgZm9ybSA9ICgnPGZvcm0gbWV0aG9kPVwicG9zdFwiIGFjdGlvbj1cIi9sb2dp
blwiPjxidXR0b24gdHlwZT1cInN1Ym1pdFwiPlNpZ24gaW48L2J1dHRvbj48L2Zvcm0+J1xuICAg
ICAgICAgICAgICAgIGlmIHN0YXJ0IGVsc2UgXCJcIilcbiAgICAgICAgcmF3ID0gKCc8IWRvY3R5
cGUgaHRtbD48aHRtbCBsYW5nPVwiZW5cIj48bWV0YSBjaGFyc2V0PVwidXRmLThcIj4nXG4gICAg
ICAgICAgICAgICAnPHRpdGxlPkxvY2FsIGRlbW88L3RpdGxlPjxoMT4nICsgaGVhZGluZyArICc8
L2gxPjxwPicgKyB0ZXh0ICsgJzwvcD4nXG4gICAgICAgICAgICAgICArIGZvcm0gKyAnPC9odG1s
PicpLmVuY29kZShcInV0Zi04XCIpXG4gICAgICAgIHdpdGggc2VsZi5zZXJ2ZXIuZGVtby5idWRn
ZXQubGltaXQoNSwgXCJyZXNwb25zZV90aW1lb3V0XCIpOlxuICAgICAgICAgICAgc2VsZi5zZW5k
X3Jlc3BvbnNlKHN0YXR1cylcbiAgICAgICAgICAgIHNlbGYuc2VuZF9oZWFkZXIoXCJDb250ZW50
LVR5cGVcIiwgXCJ0ZXh0L2h0bWw7IGNoYXJzZXQ9dXRmLThcIilcbiAgICAgICAgICAgIHNlbGYu
c2VuZF9oZWFkZXIoXCJDb250ZW50LUxlbmd0aFwiLCBzdHIobGVuKHJhdykpKVxuICAgICAgICAg
ICAgc2VsZi5zZW5kX2hlYWRlcihcIkNvbm5lY3Rpb25cIiwgXCJjbG9zZVwiKVxuICAgICAgICAg
ICAgc2VsZi5zZW5kX2hlYWRlcihcIkNhY2hlLUNvbnRyb2xcIiwgXCJuby1zdG9yZVwiKVxuICAg
ICAgICAgICAgc2VsZi5zZW5kX2hlYWRlcihcIlJlZmVycmVyLVBvbGljeVwiLCBcIm5vLXJlZmVy
cmVyXCIpXG4gICAgICAgICAgICBzZWxmLnNlbmRfaGVhZGVyKFwiQ29udGVudC1TZWN1cml0eS1Q
b2xpY3lcIiwgXCJkZWZhdWx0LXNyYyAnbm9uZSc7IGZvcm0tYWN0aW9uICdzZWxmJzsgZnJhbWUt
YW5jZXN0b3JzICdub25lJ1wiKVxuICAgICAgICAgICAgaWYgbG9jYXRpb24gaXMgbm90IE5vbmU6
XG4gICAgICAgICAgICAgICAgc2VsZi5zZW5kX2hlYWRlcihcIkxvY2F0aW9uXCIsIGxvY2F0aW9u
KVxuICAgICAgICAgICAgaWYgY29va2llIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAgICAgIHNl
bGYuc2VuZF9oZWFkZXIoXCJTZXQtQ29va2llXCIsIGNvb2tpZVswXSArIFwiPVwiICsgY29va2ll
WzFdXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICArIFwiOyBIdHRwT25seTsgU2Ft
ZVNpdGU9TGF4OyBQYXRoPS9cIilcbiAgICAgICAgICAgIGlmIGNsZWFyX2Zsb3c6XG4gICAgICAg
ICAgICAgICAgc2VsZi5zZW5kX2hlYWRlcihcIlNldC1Db29raWVcIiwgRkxPV19DT09LSUUgKyBc
Ij07IE1heC1BZ2U9MDsgSHR0cE9ubHk7IFNhbWVTaXRlPUxheDsgUGF0aD0vXCIpXG4gICAgICAg
ICAgICBzZWxmLmVuZF9oZWFkZXJzKClcbiAgICAgICAgICAgIHNlbGYud2ZpbGUud3JpdGUocmF3
KVxuICAgICAgICAgICAgc2VsZi53ZmlsZS5mbHVzaCgpXG4gICAgICAgIHNlbGYuY2xvc2VfY29u
bmVjdGlvbiA9IFRydWVcblxuXG5jbGFzcyBRdWlldFBhcnNlcihhcmdwYXJzZS5Bcmd1bWVudFBh
cnNlcik6XG4gICAgZGVmIGVycm9yKHNlbGYsIG1lc3NhZ2UpOlxuICAgICAgICByYWlzZSBGYWls
dXJlKFwiYXJndW1lbnRzX2ludmFsaWRcIilcblxuXG5kZWYgbWFpbigpOlxuICAgIHN0YXJ0ZWQg
PSB0aW1lLm1vbm90b25pYygpXG4gICAgYnVkZ2V0ID0gZGVtbyA9IHNlcnZlciA9IGV2aWRlbmNl
X2ZkID0gd29ya3NwYWNlID0gc2VjcmV0ID0gTm9uZVxuICAgIHN0YWdlID0gXCJhcmd1bWVudHNc
IlxuICAgIHJlY29yZCA9IHtcInNjaGVtYVwiOiBcInJpYXV0aC5kMDEtY29uZmlkZW50aWFsLWJy
b3dzZXIvdjFcIiwgXCJyZXN1bHRcIjogXCJmYWlsZWRcIixcbiAgICAgICAgICAgICAgXCJmYWls
dXJlX3N0YWdlXCI6IE5vbmUsIFwiZmFpbHVyZV90YWdcIjogTm9uZSwgXCJjaGVja3NcIjoge30s
IFwiY2xlYW51cFwiOiB7fSxcbiAgICAgICAgICAgICAgXCJyZXF1ZXN0X2ludmFsaWRfcmVhc29u
XCI6IE5vbmUsIFwicHJlZmxvd19hdXRob3JpemF0aW9uX3JlZnVzYWxzXCI6IDAsXG4gICAgICAg
ICAgICAgIFwic291cmNlXCI6IHtcInZlcmlmaWVyX2NvbW1pdFwiOiBWRVJJRklFUl9DT01NSVQs
IFwidmVyaWZpZXJfYmxvYlwiOiBWRVJJRklFUl9CTE9CLFxuICAgICAgICAgICAgICAgICAgICAg
ICAgIFwidmVyaWZpZXJfZXhwZWN0ZWRfc2hhMjU2XCI6IFZFUklGSUVSX1NIQTI1Nn0sXG4gICAg
ICAgICAgICAgIFwicHJvdmlkZXJcIjogTm9uZSxcbiAgICAgICAgICAgICAgXCJodHRwX3N0YXR1
c2VzXCI6IHtuYW1lOiBOb25lIGZvciBuYW1lIGluXG4gICAgICAgICAgICAgICAgICAgICAgICAg
ICAgICAgIChcImF1dGhvcml6YXRpb25fcmVkaXJlY3RcIiwgXCJjYWxsYmFja1wiLCBcInRva2Vu
X2V4Y2hhbmdlXCIsXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBcInVzZXJpbmZv
XCIsIFwicHJvdGVjdGVkX2JlZm9yZVwiLCBcInByb3RlY3RlZF9hZnRlclwiKX19XG4gICAgdHJ5
OlxuICAgICAgICBwYXJzZXIgPSBRdWlldFBhcnNlcihhZGRfaGVscD1GYWxzZSwgYWxsb3dfYWJi
cmV2PUZhbHNlKVxuICAgICAgICBmb3IgbmFtZSBpbiAoXCJ3b3Jrc3BhY2VcIiwgXCJzZWNyZXQt
ZmlsZVwiLCBcInZlcmlmaWVyLWhlbHBlclwiLCBcIm9wZW5zc2xcIiwgXCJldmlkZW5jZVwiKTpc
biAgICAgICAgICAgIHBhcnNlci5hZGRfYXJndW1lbnQoXCItLVwiICsgbmFtZSwgcmVxdWlyZWQ9
VHJ1ZSlcbiAgICAgICAgcGFyc2VyLmFkZF9hcmd1bWVudChcIi0tZGVhZGxpbmUtc2Vjb25kc1wi
LCB0eXBlPWludCwgZGVmYXVsdD02MDApXG4gICAgICAgIGFyZ3MgPSBwYXJzZXIucGFyc2VfYXJn
cygpXG4gICAgICAgIHJlcXVpcmUoc3lzLnZlcnNpb25faW5mbyA+PSAoMywgMTEpIGFuZCAxIDw9
IGFyZ3MuZGVhZGxpbmVfc2Vjb25kcyA8PSA2MDAsXG4gICAgICAgICAgICAgICAgXCJhcmd1bWVu
dHNfaW52YWxpZFwiKVxuICAgICAgICBzdGFnZSA9IFwicGF0aHNcIlxuICAgICAgICB3b3Jrc3Bh
Y2UgPSBwcml2YXRlX2RpcmVjdG9yeShhcmdzLndvcmtzcGFjZSlcbiAgICAgICAgbGFiID0gcHJp
dmF0ZV9kaXJlY3Rvcnkod29ya3NwYWNlLnBhcmVudClcbiAgICAgICAgcmVxdWlyZShub3QgYW55
KHdvcmtzcGFjZS5pdGVyZGlyKCkpLCBcInBhdGhzX2ludmFsaWRcIilcbiAgICAgICAgc2VjcmV0
X3BhdGgsIHZlcmlmaWVyX3BhdGgsIGV2aWRlbmNlX3BhdGggPSBbUGF0aChvcy5wYXRoLmFic3Bh
dGgodmFsdWUpKSBmb3IgdmFsdWUgaW5cbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg
ICAgICAgICAgICAgICAgICAgIChhcmdzLnNlY3JldF9maWxlLCBhcmdzLnZlcmlmaWVyX2hlbHBl
ciwgYXJncy5ldmlkZW5jZSldXG4gICAgICAgIHJlcXVpcmUocHJpdmF0ZV9kaXJlY3Rvcnkoc2Vj
cmV0X3BhdGgucGFyZW50KS5wYXJlbnQgPT0gbGFiXG4gICAgICAgICAgICAgICAgYW5kIHZlcmlm
aWVyX3BhdGgucGFyZW50ID09IGxhYiBhbmQgZXZpZGVuY2VfcGF0aC5wYXJlbnQgPT0gbGFiXG4g
ICAgICAgICAgICAgICAgYW5kIGxlbih7c2VjcmV0X3BhdGgsIHZlcmlmaWVyX3BhdGgsIGV2aWRl
bmNlX3BhdGh9KSA9PSAzLCBcInBhdGhzX2ludmFsaWRcIilcbiAgICAgICAgYnVkZ2V0ID0gQnVk
Z2V0KGxhYiwgc3RhcnRlZCwgYXJncy5kZWFkbGluZV9zZWNvbmRzKVxuICAgICAgICBidWRnZXQu
X19lbnRlcl9fKClcbiAgICAgICAgZXZpZGVuY2VfZmQgPSBvcy5vcGVuKGV2aWRlbmNlX3BhdGgs
IG9zLk9fV1JPTkxZIHwgb3MuT19DUkVBVCB8IG9zLk9fRVhDTCB8IG9zLk9fTk9GT0xMT1csIDBv
NjAwKVxuICAgICAgICBvcy5mY2htb2QoZXZpZGVuY2VfZmQsIDBvNjAwKVxuICAgICAgICBzdGFn
ZSA9IFwidmVyaWZpZXJcIlxuICAgICAgICBtb2R1bGUgPSBsb2FkX3ZlcmlmaWVyKHZlcmlmaWVy
X3BhdGgpXG4gICAgICAgIHJlY29yZFtcInNvdXJjZVwiXVtcInZlcmlmaWVyX3NoYTI1NlwiXSA9
IFZFUklGSUVSX1NIQTI1NlxuICAgICAgICByZWNvcmRbXCJzb3VyY2VcIl1bXCJoZWxwZXJfc2hh
MjU2XCJdID0gaGFzaGxpYi5zaGEyNTYoUGF0aChfX2ZpbGVfXykucmVhZF9ieXRlcygpKS5oZXhk
aWdlc3QoKVxuICAgICAgICBzdGFnZSA9IFwiY3JlZGVudGlhbFwiXG4gICAgICAgIHNlY3JldCA9
IGNsaWVudF9zZWNyZXQobW9kdWxlLCBzZWNyZXRfcGF0aClcbiAgICAgICAgZm9yIG5hbWUgaW4g
KFwiY3JlZGVudGlhbF9wcml2YXRlX3ZhbGlkYXRlZFwiLCBcInByb3ZpZGVyX2lkZW50aXR5X3Zl
cmlmaWVkXCIsIFwiZGlzY292ZXJ5X3ZlcmlmaWVkXCIsXG4gICAgICAgICAgICAgICAgICAgICBc
InByb3RlY3RlZF93aXRob3V0X2Nvb2tpZV9kZW5pZWRcIiwgXCJhdXRob3JpemF0aW9uX3JlZGly
ZWN0X2lzc3VlZFwiLFxuICAgICAgICAgICAgICAgICAgICAgXCJzdGF0ZV9pc3N1ZXJfZmxvd19j
b29raWVfdmVyaWZpZWRcIiwgXCJjb25maWRlbnRpYWxfczI1Nl9leGNoYW5nZV92ZXJpZmllZFwi
LFxuICAgICAgICAgICAgICAgICAgICAgXCJyczI1Nl9qd2tzX2lzc3Vlcl9hdWRpZW5jZV9ub25j
ZV90aW1lX2FjY2Vzc19oYXNoX3ZlcmlmaWVkXCIsXG4gICAgICAgICAgICAgICAgICAgICBcInVz
ZXJpbmZvX3N1YmplY3RfdmVyaWZpZWRcIiwgXCJwcm90ZWN0ZWRfd2l0aF9mcmVzaF9jb29raWVf
YWNjZXB0ZWRcIik6XG4gICAgICAgICAgICByZWNvcmRbXCJjaGVja3NcIl1bbmFtZV0gPSBuYW1l
ID09IFwiY3JlZGVudGlhbF9wcml2YXRlX3ZhbGlkYXRlZFwiXG4gICAgICAgIGRlbW8gPSBEZW1v
KG1vZHVsZSwgd29ya3NwYWNlLCBzZWNyZXQsIGJ1ZGdldCwgcmVjb3JkW1wiY2hlY2tzXCJdLCBy
ZWNvcmRbXCJodHRwX3N0YXR1c2VzXCJdKVxuICAgICAgICBzZWNyZXQgPSBOb25lXG4gICAgICAg
IGRlbW8uc2V0dXAoYXJncy5vcGVuc3NsKVxuICAgICAgICByZWNvcmRbXCJwcm92aWRlclwiXSA9
IHtcInNoYTI1NlwiOiBPUEVOU1NMX1NIQTI1NiwgXCJ2ZXJzaW9uXCI6IE9QRU5TU0xfVkVSU0lP
Tn1cbiAgICAgICAgZGVtby5zdGFnZSA9IFwibGlzdGVuZXJcIlxuICAgICAgICB0cnk6XG4gICAg
ICAgICAgICBzZXJ2ZXIgPSBEZW1vU2VydmVyKGRlbW8pXG4gICAgICAgIGV4Y2VwdCBFeGNlcHRp
b246XG4gICAgICAgICAgICByYWlzZSBGYWlsdXJlKFwibGlzdGVuZXJfZmFpbGVkXCIpIGZyb20g
Tm9uZVxuICAgICAgICBwcmludChqc29uLmR1bXBzKHtcInJlYWR5XCI6IFRydWUsIFwicG9ydFwi
OiAzMDAwLCBcInBpZFwiOiBvcy5nZXRwaWQoKX0pLCBmbHVzaD1UcnVlKVxuICAgICAgICB3aGls
ZSBub3QgZGVtby5kb25lOlxuICAgICAgICAgICAgYnVkZ2V0LnRpY2soKVxuICAgICAgICAgICAg
c2VydmVyLmhhbmRsZV9yZXF1ZXN0KClcbiAgICAgICAgaWYgZGVtby5mYWlsdXJlIGlzIG5vdCBO
b25lOlxuICAgICAgICAgICAgcmFpc2UgRmFpbHVyZShkZW1vLmZhaWx1cmUpXG4gICAgICAgIHJl
cXVpcmUoYWxsKHJlY29yZFtcImNoZWNrc1wiXS52YWx1ZXMoKSksIFwidW5leHBlY3RlZF9mYWls
dXJlXCIpXG4gICAgICAgIHJlY29yZFtcInJlc3VsdFwiXSA9IFwicGFzc2VkXCJcbiAgICBleGNl
cHQgKEZhaWx1cmUsIEhhbHQpIGFzIGZhaWx1cmU6XG4gICAgICAgIHJlY29yZFtcImZhaWx1cmVf
dGFnXCJdID0gZmFpbHVyZS50YWdcbiAgICAgICAgcmVjb3JkW1wiZmFpbHVyZV9zdGFnZVwiXSA9
IGRlbW8uc3RhZ2UgaWYgZGVtbyBpcyBub3QgTm9uZSBlbHNlIHN0YWdlXG4gICAgZXhjZXB0IEtl
eWJvYXJkSW50ZXJydXB0OlxuICAgICAgICByZWNvcmRbXCJmYWlsdXJlX3RhZ1wiXSwgcmVjb3Jk
W1wiZmFpbHVyZV9zdGFnZVwiXSA9IFwiaW50ZXJydXB0ZWRcIiwgc3RhZ2VcbiAgICBleGNlcHQg
RXhjZXB0aW9uOlxuICAgICAgICByZWNvcmRbXCJmYWlsdXJlX3RhZ1wiXSA9IFwidW5leHBlY3Rl
ZF9mYWlsdXJlXCJcbiAgICAgICAgcmVjb3JkW1wiZmFpbHVyZV9zdGFnZVwiXSA9IGRlbW8uc3Rh
Z2UgaWYgZGVtbyBpcyBub3QgTm9uZSBlbHNlIHN0YWdlXG4gICAgZmluYWxseTpcbiAgICAgICAg
Y2xlYW51cF9mYWlsZWQgPSBGYWxzZVxuICAgICAgICBpZiBidWRnZXQgaXMgbm90IE5vbmU6XG4g
ICAgICAgICAgICB0cnk6XG4gICAgICAgICAgICAgICAgYnVkZ2V0LmNsb3NlKClcbiAgICAgICAg
ICAgIGV4Y2VwdCBCYXNlRXhjZXB0aW9uOlxuICAgICAgICAgICAgICAgIGNsZWFudXBfZmFpbGVk
ID0gVHJ1ZVxuICAgICAgICAgICAgcmVjb3JkW1wibWluaW11bV9mcmVlX2J5dGVzXCJdLCByZWNv
cmRbXCJkaXNrX3NhbXBsZXNcIl0gPSBidWRnZXQubWluaW11bV9mcmVlLCBidWRnZXQuc2FtcGxl
c1xuICAgICAgICBpZiBzZXJ2ZXIgaXMgbm90IE5vbmU6XG4gICAgICAgICAgICB0cnk6XG4gICAg
ICAgICAgICAgICAgaWYgc2VydmVyLmFjdGl2ZSBpcyBub3QgTm9uZTpcbiAgICAgICAgICAgICAg
ICAgICAgc2VydmVyLnNodXRkb3duX3JlcXVlc3Qoc2VydmVyLmFjdGl2ZSlcbiAgICAgICAgICAg
ICAgICAgICAgc2VydmVyLmFjdGl2ZSA9IE5vbmVcbiAgICAgICAgICAgIGV4Y2VwdCBCYXNlRXhj
ZXB0aW9uOlxuICAgICAgICAgICAgICAgIGNsZWFudXBfZmFpbGVkID0gVHJ1ZVxuICAgICAgICAg
ICAgdHJ5OlxuICAgICAgICAgICAgICAgIHNlcnZlci5zZXJ2ZXJfY2xvc2UoKVxuICAgICAgICAg
ICAgZXhjZXB0IEJhc2VFeGNlcHRpb246XG4gICAgICAgICAgICAgICAgY2xlYW51cF9mYWlsZWQg
PSBUcnVlXG4gICAgICAgICAgICByZWNvcmRbXCJjbGVhbnVwXCJdW1wibGlzdGVuZXJfY2xvc2Vk
XCJdID0gc2VydmVyLnNvY2tldC5maWxlbm8oKSA9PSAtMVxuICAgICAgICAgICAgcmVjb3JkW1wi
Y2xlYW51cFwiXVtcImNvbm5lY3Rpb25fY2xvc2VkXCJdID0gc2VydmVyLmFjdGl2ZSBpcyBOb25l
XG4gICAgICAgIGlmIGRlbW8gaXMgbm90IE5vbmU6XG4gICAgICAgICAgICByZWNvcmRbXCJyZXF1
ZXN0X2ludmFsaWRfcmVhc29uXCJdID0gZGVtby5yZXF1ZXN0X2ludmFsaWRfcmVhc29uXG4gICAg
ICAgICAgICByZWNvcmRbXCJwcmVmbG93X2F1dGhvcml6YXRpb25fcmVmdXNhbHNcIl0gPSBkZW1v
LnByZWZsb3dfYXV0aG9yaXphdGlvbl9yZWZ1c2Fsc1xuICAgICAgICAgICAgdHJ5OlxuICAgICAg
ICAgICAgICAgIGRlbW8uY2xlYXIoKVxuICAgICAgICAgICAgICAgIHJlY29yZFtcImNsZWFudXBc
Il1bXCJwcml2YXRlX3JlZmVyZW5jZXNfY2xlYXJlZFwiXSA9IFRydWVcbiAgICAgICAgICAgIGV4
Y2VwdCBCYXNlRXhjZXB0aW9uOlxuICAgICAgICAgICAgICAgIGNsZWFudXBfZmFpbGVkID0gVHJ1
ZVxuICAgICAgICAgICAgICAgIHJlY29yZFtcImNsZWFudXBcIl1bXCJwcml2YXRlX3JlZmVyZW5j
ZXNfY2xlYXJlZFwiXSA9IEZhbHNlXG4gICAgICAgIHNlY3JldCA9IE5vbmVcbiAgICAgICAgdHJ5
OlxuICAgICAgICAgICAgaWYgd29ya3NwYWNlIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAgICAg
IHJlY29yZFtcImNsZWFudXBcIl1bXCJ2ZXJpZmllcl90ZW1wb3Jhcmllc19yZW1vdmVkXCJdID0g
bm90IGFueSh3b3Jrc3BhY2UuaXRlcmRpcigpKVxuICAgICAgICAgICAgaWYgY2xlYW51cF9mYWls
ZWQgb3IgKHJlY29yZFtcInJlc3VsdFwiXSA9PSBcInBhc3NlZFwiIGFuZCBub3QgYWxsKHJlY29y
ZFtcImNsZWFudXBcIl0udmFsdWVzKCkpKTpcbiAgICAgICAgICAgICAgICByYWlzZSBGYWlsdXJl
KFwiY2xlYW51cF9mYWlsZWRcIilcbiAgICAgICAgZXhjZXB0IEJhc2VFeGNlcHRpb246XG4gICAg
ICAgICAgICByZWNvcmRbXCJyZXN1bHRcIl0gPSBcImZhaWxlZFwiXG4gICAgICAgICAgICByZWNv
cmRbXCJjbGVhbnVwXCJdW1wiZmFpbHVyZV90YWdcIl0gPSBcImNsZWFudXBfZmFpbGVkXCJcbiAg
ICAgICAgICAgIGlmIHJlY29yZFtcImZhaWx1cmVfdGFnXCJdIGlzIE5vbmU6XG4gICAgICAgICAg
ICAgICAgcmVjb3JkW1wiZmFpbHVyZV90YWdcIl0sIHJlY29yZFtcImZhaWx1cmVfc3RhZ2VcIl0g
PSBcImNsZWFudXBfZmFpbGVkXCIsIFwiY2xlYW51cFwiXG4gICAgICAgIHJlY29yZFtcImVsYXBz
ZWRfc2Vjb25kc1wiXSA9IHJvdW5kKHRpbWUubW9ub3RvbmljKCkgLSBzdGFydGVkLCAzKVxuICAg
IHdyaXR0ZW4gPSBGYWxzZVxuICAgIGlmIGV2aWRlbmNlX2ZkIGlzIG5vdCBOb25lOlxuICAgICAg
ICB0cnk6XG4gICAgICAgICAgICB3aXRoIG9zLmZkb3BlbihldmlkZW5jZV9mZCwgXCJ3YlwiKSBh
cyBvdXRwdXQ6XG4gICAgICAgICAgICAgICAgb3V0cHV0LndyaXRlKChqc29uLmR1bXBzKHJlY29y
ZCwgc29ydF9rZXlzPVRydWUpICsgXCJcXG5cIikuZW5jb2RlKFwiYXNjaWlcIikpXG4gICAgICAg
ICAgICAgICAgb3V0cHV0LmZsdXNoKClcbiAgICAgICAgICAgICAgICBvcy5mc3luYyhvdXRwdXQu
ZmlsZW5vKCkpXG4gICAgICAgICAgICB3cml0dGVuID0gVHJ1ZVxuICAgICAgICBleGNlcHQgRXhj
ZXB0aW9uOlxuICAgICAgICAgICAgcmVjb3JkW1wicmVzdWx0XCJdID0gXCJmYWlsZWRcIlxuICAg
ICAgICAgICAgcmVjb3JkW1wiZXZpZGVuY2VfZmFpbHVyZV90YWdcIl0gPSBcImV2aWRlbmNlX3dy
aXRlX2ZhaWxlZFwiXG4gICAgICAgICAgICBpZiByZWNvcmRbXCJmYWlsdXJlX3RhZ1wiXSBpcyBO
b25lOlxuICAgICAgICAgICAgICAgIHJlY29yZFtcImZhaWx1cmVfdGFnXCJdLCByZWNvcmRbXCJm
YWlsdXJlX3N0YWdlXCJdID0gXCJldmlkZW5jZV93cml0ZV9mYWlsZWRcIiwgXCJldmlkZW5jZVwi
XG4gICAgICAgIGZpbmFsbHk6XG4gICAgICAgICAgICB0cnk6XG4gICAgICAgICAgICAgICAgb3Mu
Y2xvc2UoZXZpZGVuY2VfZmQpXG4gICAgICAgICAgICBleGNlcHQgT1NFcnJvcjpcbiAgICAgICAg
ICAgICAgICBwYXNzXG4gICAgdHJ5OlxuICAgICAgICBwcmludChqc29uLmR1bXBzKHtcInJlc3Vs
dFwiOiByZWNvcmRbXCJyZXN1bHRcIl0sIFwiZmFpbHVyZV90YWdcIjogcmVjb3JkW1wiZmFpbHVy
ZV90YWdcIl0sXG4gICAgICAgICAgICAgICAgICAgICAgICAgIFwiY2xlYW51cF9mYWlsdXJlX3Rh
Z1wiOiByZWNvcmRbXCJjbGVhbnVwXCJdLmdldChcImZhaWx1cmVfdGFnXCIpLFxuICAgICAgICAg
ICAgICAgICAgICAgICAgICBcImV2aWRlbmNlX2ZhaWx1cmVfdGFnXCI6IHJlY29yZC5nZXQoXCJl
dmlkZW5jZV9mYWlsdXJlX3RhZ1wiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgXCJldmlk
ZW5jZV93cml0dGVuXCI6IHdyaXR0ZW59KSwgZmx1c2g9VHJ1ZSlcbiAgICBleGNlcHQgRXhjZXB0
aW9uOlxuICAgICAgICByZXR1cm4gMVxuICAgIHJldHVybiAwIGlmIHJlY29yZFtcInJlc3VsdFwi
XSA9PSBcInBhc3NlZFwiIGFuZCB3cml0dGVuIGVsc2UgMVxuXG5cbmlmIF9fbmFtZV9fID09IFwi
X19tYWluX19cIjpcbiAgICByYWlzZSBTeXN0ZW1FeGl0KG1haW4oKSlcbiIKU09VUkNFX1RFWFQ9
IiMhL3Vzci9iaW4vZW52IHB5dGhvbjNcblwiXCJcIk9uZSBkaXNwb3NhYmxlIGNvbmZpZGVudGlh
bCBsb2NhbC1kZW1vIGFwcGxpY2F0aW9uOyBicm93c2VyIGF1dGhlbnRpY2F0aW9uIG9ubHkuXG5c
blB5dGhvbiAzLjExKywgUE9TSVggYWxhcm1zLCB0aGUgcGlubmVkIHJlY292ZXJ5IHZlcmlmaWVy
IGFuZCBuYXRpdmUgT3BlblNTTCBhcmVcbnByZXJlcXVpc2l0ZXMuIEltcG9ydCBzdGFydHMgbm90
aGluZy4gVGhlIGV4dGVybmFsIG93bmVyIHN1cHBsaWVzIGEgcHJpdmF0ZSBsYWIsXG5yZWdpc3Rl
cnMgdGhlIHByaW50ZWQgY2xpZW50LCBkcml2ZXMgdGhlIGJyb3dzZXIgdGhyb3VnaCBSaVdvcmsg
Q3VhLmFpIERyaXZlcixcbmFuZCBvd25zIHRoZSBJZFAvYnJvd3NlciBsaWZlY3ljbGUgYW5kIHRo
ZSBvdmVyYWxsIDE1LW1pbnV0ZSBjbGVhbnVwIGRlYWRsaW5lLlxuVGhpcyBoZWxwZXIgbmVpdGhl
ciBsb2dzIGluIHRvIHJpQXV0aCBub3IgYXBwcm92ZXMgYW4gYXV0aG9yaXphdGlvbiByZXF1ZXN0
LlxuXCJcIlwiXG5cbmltcG9ydCBhcmdwYXJzZVxuaW1wb3J0IGNvbnRleHRsaWJcbmltcG9ydCBo
YXNobGliXG5pbXBvcnQgaHR0cC5zZXJ2ZXJcbmltcG9ydCBqc29uXG5pbXBvcnQgb3NcbmltcG9y
dCByZVxuaW1wb3J0IHNlY3JldHNcbmltcG9ydCBzaHV0aWxcbmltcG9ydCBzaWduYWxcbmltcG9y
dCBzdGF0XG5pbXBvcnQgc3VicHJvY2Vzc1xuaW1wb3J0IHN5c1xuaW1wb3J0IHRpbWVcbmltcG9y
dCB0eXBlc1xuaW1wb3J0IHVybGxpYi5wYXJzZVxuZnJvbSBwYXRobGliIGltcG9ydCBQYXRoXG5c
blxuSVNTVUVSID0gXCJodHRwOi8vbG9jYWxob3N0OjkwMDBcIlxuQ0xJRU5UX0lEID0gXCJsb2Nh
bC1kZW1vXCJcbk9SSUdJTiA9IFwiaHR0cDovL2xvY2FsaG9zdDozMDAwXCJcbkFVVEhPUklUWSA9
IFwibG9jYWxob3N0OjMwMDBcIlxuQ0FMTEJBQ0sgPSBPUklHSU4gKyBcIi9jYWxsYmFja1wiXG5G
TE9XX0NPT0tJRSA9IFwiZDAxX2RlbW9fZmxvd1wiXG5BUFBfQ09PS0lFID0gXCJkMDFfbG9jYWxf
ZGVtb1wiXG5WRVJJRklFUl9DT01NSVQgPSBcIjljZWZlN2E1NjQyNWJiNzNjMTc3NTNlODc2NmQ5
MjMyMGI3N2RhM2JcIlxuVkVSSUZJRVJfQkxPQiA9IFwiM2JlNzQ3ZDAzMTQ2ZjFiY2FhM2VjMDEy
ZWU4ZDE3M2I2MWZhNzM3ZFwiXG5WRVJJRklFUl9TSEEyNTYgPSBcImY2ZGQxYWEwYjcxZGU0Nzkz
YzliODZlZTc5OTAxMmJiMzFhOGZjMmQ2MTgyOTc2MDk0YjQ0ZGY2ZWYwNGRlM2RcIlxuT1BFTlNT
TF9TSEEyNTYgPSBcIjY3YTgzZGQ2ZDZkNzQ3ZDUwYzVkMjk2ZGZmYjIzZTMyYmFlOWEyYzU4OGM5
M2FlMmQ3N2U0YzYwN2I0NTVjNzJcIlxuT1BFTlNTTF9WRVJTSU9OID0gXCJPcGVuU1NMIDMuNi40
IDI1IEF1ZyAyMDI2IChMaWJyYXJ5OiBPcGVuU1NMIDMuNi40IDI1IEF1ZyAyMDI2KVwiXG5NQVhf
RklMRSA9IDI1NiAqIDEwMjRcbk1BWF9SRVFVRVNUX0xJTkUgPSA4MTkyXG5NQVhfSEVBREVSUyA9
IDgxOTJcbk1BWF9IRUFERVJfTElORSA9IDQwOTZcbk1BWF9IRUFERVJfQ09VTlQgPSAzMlxuTUFY
X0NPT0tJRSA9IDQwOTZcblJFUVVFU1RfSU5WQUxJRF9SRUFTT05TID0ge1xuICAgIFwiaHR0cF9w
YXJzZVwiLCBcImhvc3RcIiwgXCJhdXRob3JpemF0aW9uXCIsIFwidHJhbnNmZXJfZW5jb2Rpbmdc
IiwgXCJleHBlY3RcIixcbiAgICBcImNvbnRlbnRfbGVuZ3RoXCIsIFwidGFyZ2V0X3NjaGVtZVwi
LCBcInRhcmdldF9uZXRsb2NcIiwgXCJ0YXJnZXRfZnJhZ21lbnRcIixcbiAgICBcIm1ldGhvZFwi
LCBcInBvc3RfdGFyZ2V0XCIsIFwib3JpZ2luXCIsIFwiY29udGVudF90eXBlXCIsIFwiY29va2ll
X2hlYWRlcl9jb3VudFwiLFxuICAgIFwib3duX2Nvb2tpZV9zaGFwZVwiLFxufVxuU1RPUF9GUkVF
X0JZVEVTID0gMTcgKiAxMDI0KiozIC8vIDJcblBFTkRJTkdfU0VDT05EUyA9IDE4MFxuRkFJTFVS
RV9UQUdTID0ge1xuICAgIFwiYXJndW1lbnRzX2ludmFsaWRcIiwgXCJwYXRoc19pbnZhbGlkXCIs
IFwicHJpdmF0ZV9maWxlX2ludmFsaWRcIixcbiAgICBcImNyZWRlbnRpYWxfaW52YWxpZFwiLCBc
InZlcmlmaWVyX2hhc2hfbWlzbWF0Y2hcIiwgXCJ2ZXJpZmllcl9pbXBvcnRfZmFpbGVkXCIsXG4g
ICAgXCJwcm92aWRlcl9mYWlsZWRcIiwgXCJkaXNjb3ZlcnlfZmFpbGVkXCIsIFwibGlzdGVuZXJf
ZmFpbGVkXCIsXG4gICAgXCJyZXF1ZXN0X2ludmFsaWRcIiwgXCJyZXF1ZXN0X2xpbWl0XCIsIFwi
cmVxdWVzdF90aW1lb3V0XCIsIFwicmVzcG9uc2VfdGltZW91dFwiLFxuICAgIFwiZmxvd19hbHJl
YWR5X3N0YXJ0ZWRcIiwgXCJjYWxsYmFja19pbnZhbGlkXCIsIFwiY2FsbGJhY2tfYWxyZWFkeV9j
b25zdW1lZFwiLFxuICAgIFwidG9rZW5fZXhjaGFuZ2VfZmFpbGVkXCIsIFwidG9rZW5fdmFsaWRh
dGlvbl9mYWlsZWRcIiwgXCJ1c2VyaW5mb19mYWlsZWRcIixcbiAgICBcInByb3RlY3RlZF9iZWZv
cmVfdW5vYnNlcnZlZFwiLCBcImZpeHR1cmVfZGVhZGxpbmVcIiwgXCJwZW5kaW5nX2RlYWRsaW5l
XCIsXG4gICAgXCJkaXNrX21hcmdpblwiLCBcImRpc2tfb2JzZXJ2YXRpb25fZmFpbGVkXCIsIFwi
aW50ZXJydXB0ZWRcIiwgXCJjbGVhbnVwX2ZhaWxlZFwiLFxuICAgIFwiZXZpZGVuY2Vfd3JpdGVf
ZmFpbGVkXCIsIFwidW5leHBlY3RlZF9mYWlsdXJlXCIsXG59XG5cblxuY2xhc3MgRmFpbHVyZShF
eGNlcHRpb24pOlxuICAgIGRlZiBfX2luaXRfXyhzZWxmLCB0YWcpOlxuICAgICAgICBzZWxmLnRh
ZyA9IHRhZyBpZiB0YWcgaW4gRkFJTFVSRV9UQUdTIGVsc2UgXCJ1bmV4cGVjdGVkX2ZhaWx1cmVc
IlxuICAgICAgICBzdXBlcigpLl9faW5pdF9fKHNlbGYudGFnKVxuXG5cbmNsYXNzIEhhbHQoQmFz
ZUV4Y2VwdGlvbik6XG4gICAgXCJcIlwiUGFzcyB0aHJvdWdoIEhUVFAvdmVyaWZpZXIgRXhjZXB0
aW9uIGhhbmRsZXJzIHRvIG93bmVkIGZpbmFsbHkgY2xlYW51cC5cIlwiXCJcblxuICAgIGRlZiBf
X2luaXRfXyhzZWxmLCB0YWcpOlxuICAgICAgICBzZWxmLnRhZyA9IHRhZyBpZiB0YWcgaW4gRkFJ
TFVSRV9UQUdTIGVsc2UgXCJ1bmV4cGVjdGVkX2ZhaWx1cmVcIlxuXG5cbmRlZiByZXF1aXJlKGNv
bmRpdGlvbiwgdGFnKTpcbiAgICBpZiBub3QgY29uZGl0aW9uOlxuICAgICAgICByYWlzZSBGYWls
dXJlKHRhZylcblxuXG5kZWYgcHJpdmF0ZV9kaXJlY3RvcnkocGF0aCk6XG4gICAgcGF0aCA9IFBh
dGgob3MucGF0aC5hYnNwYXRoKHBhdGgpKVxuICAgIGZkID0gb3Mub3BlbihwYXRoLCBvcy5PX1JE
T05MWSB8IG9zLk9fRElSRUNUT1JZIHwgb3MuT19OT0ZPTExPVylcbiAgICB0cnk6XG4gICAgICAg
IGluZm8gPSBvcy5mc3RhdChmZClcbiAgICAgICAgcmVxdWlyZShzdGF0LlNfSVNESVIoaW5mby5z
dF9tb2RlKSBhbmQgaW5mby5zdF91aWQgPT0gb3MuZ2V0dWlkKClcbiAgICAgICAgICAgICAgICBh
bmQgc3RhdC5TX0lNT0RFKGluZm8uc3RfbW9kZSkgPT0gMG83MDAsIFwicGF0aHNfaW52YWxpZFwi
KVxuICAgIGZpbmFsbHk6XG4gICAgICAgIG9zLmNsb3NlKGZkKVxuICAgIHJldHVybiBwYXRoXG5c
blxuZGVmIHByaXZhdGVfYnl0ZXMocGF0aCwgbGltaXQ9TUFYX0ZJTEUpOlxuICAgIGZkID0gb3Mu
b3BlbihwYXRoLCBvcy5PX1JET05MWSB8IG9zLk9fTk9GT0xMT1cgfCBvcy5PX05PTkJMT0NLKVxu
ICAgIHRyeTpcbiAgICAgICAgaW5mbyA9IG9zLmZzdGF0KGZkKVxuICAgICAgICByZXF1aXJlKHN0
YXQuU19JU1JFRyhpbmZvLnN0X21vZGUpIGFuZCBpbmZvLnN0X3VpZCA9PSBvcy5nZXR1aWQoKVxu
ICAgICAgICAgICAgICAgIGFuZCBzdGF0LlNfSU1PREUoaW5mby5zdF9tb2RlKSA9PSAwbzYwMFxu
ICAgICAgICAgICAgICAgIGFuZCAwIDwgaW5mby5zdF9zaXplIDw9IGxpbWl0LCBcInByaXZhdGVf
ZmlsZV9pbnZhbGlkXCIpXG4gICAgICAgIHdpdGggb3MuZmRvcGVuKGZkLCBcInJiXCIsIGNsb3Nl
ZmQ9RmFsc2UpIGFzIHNvdXJjZTpcbiAgICAgICAgICAgIHJhdyA9IHNvdXJjZS5yZWFkKGxpbWl0
ICsgMSlcbiAgICAgICAgcmVxdWlyZSgwIDwgbGVuKHJhdykgPD0gbGltaXQsIFwicHJpdmF0ZV9m
aWxlX2ludmFsaWRcIilcbiAgICAgICAgcmV0dXJuIHJhd1xuICAgIGZpbmFsbHk6XG4gICAgICAg
IG9zLmNsb3NlKGZkKVxuXG5cbmRlZiBsb2FkX3ZlcmlmaWVyKHBhdGgpOlxuICAgIHJhdyA9IHBy
aXZhdGVfYnl0ZXMocGF0aClcbiAgICByZXF1aXJlKGhhc2hsaWIuc2hhMjU2KHJhdykuaGV4ZGln
ZXN0KCkgPT0gVkVSSUZJRVJfU0hBMjU2LFxuICAgICAgICAgICAgXCJ2ZXJpZmllcl9oYXNoX21p
c21hdGNoXCIpXG4gICAgIyBFeGVjdXRlIHByZWNpc2VseSB0aGUgdmVyaWZpZWQgYnl0ZXMsIHdp
dGhvdXQgcmVyZWFkaW5nIG9yIHdyaXRpbmcgcHljYWNoZS5cbiAgICBtb2R1bGUgPSB0eXBlcy5N
b2R1bGVUeXBlKFwiZDAxX3Bpbm5lZF9yZWNvdmVyeV92ZXJpZmllclwiKVxuICAgIG1vZHVsZS5f
X2ZpbGVfXyA9IHN0cihwYXRoKVxuICAgIHRyeTpcbiAgICAgICAgZXhlYyhjb21waWxlKHJhdywg
c3RyKHBhdGgpLCBcImV4ZWNcIiksIG1vZHVsZS5fX2RpY3RfXylcbiAgICBleGNlcHQgRXhjZXB0
aW9uOlxuICAgICAgICByYWlzZSBGYWlsdXJlKFwidmVyaWZpZXJfaW1wb3J0X2ZhaWxlZFwiKSBm
cm9tIE5vbmVcbiAgICByZXR1cm4gbW9kdWxlXG5cblxuZGVmIGNsaWVudF9zZWNyZXQobW9kdWxl
LCBwYXRoKTpcbiAgICB0cnk6XG4gICAgICAgIHNhdmVkID0gbW9kdWxlLmpzb25fb2JqZWN0KHBy
aXZhdGVfYnl0ZXMocGF0aCkpXG4gICAgZXhjZXB0IEZhaWx1cmU6XG4gICAgICAgIHJhaXNlXG4g
ICAgZXhjZXB0IEV4Y2VwdGlvbjpcbiAgICAgICAgcmFpc2UgRmFpbHVyZShcImNyZWRlbnRpYWxf
aW52YWxpZFwiKSBmcm9tIE5vbmVcbiAgICBjbGllbnQgPSBzYXZlZC5nZXQoXCJjbGllbnRcIilc
biAgICByZXF1aXJlKGlzaW5zdGFuY2UoY2xpZW50LCBkaWN0KSwgXCJjcmVkZW50aWFsX2ludmFs
aWRcIilcbiAgICBzZXR0aW5ncywgc2NvcGVzID0gY2xpZW50LmdldChcInNldHRpbmdzXCIpLCBj
bGllbnQuZ2V0KFwic2NvcGVzXCIpXG4gICAgcmVxdWlyZShjbGllbnQuZ2V0KFwiY2xpZW50X2lk
XCIpID09IENMSUVOVF9JRFxuICAgICAgICAgICAgYW5kIGNsaWVudC5nZXQoXCJjb25maWRlbnRp
YWxcIikgaXMgVHJ1ZVxuICAgICAgICAgICAgYW5kIGNsaWVudC5nZXQoXCJzZXJ2aWNlXCIpIGlz
IEZhbHNlIGFuZCBjbGllbnQuZ2V0KFwiZW5hYmxlZFwiKSBpcyBUcnVlXG4gICAgICAgICAgICBh
bmQgY2xpZW50LmdldChcInJlZGlyZWN0X3VyaXNcIikgPT0gW0NBTExCQUNLXVxuICAgICAgICAg
ICAgYW5kIGlzaW5zdGFuY2Uoc2NvcGVzLCBsaXN0KSBhbmQgbGVuKHNjb3BlcykgPT0gMlxuICAg
ICAgICAgICAgYW5kIGFsbChpc2luc3RhbmNlKHZhbHVlLCBzdHIpIGZvciB2YWx1ZSBpbiBzY29w
ZXMpXG4gICAgICAgICAgICBhbmQgc2V0KHNjb3BlcykgPT0ge1wib3BlbmlkXCIsIFwicHJvZmls
ZVwifVxuICAgICAgICAgICAgYW5kIGlzaW5zdGFuY2Uoc2V0dGluZ3MsIGRpY3QpXG4gICAgICAg
ICAgICBhbmQgc2V0dGluZ3MuZ2V0KFwidG9rZW5fZW5kcG9pbnRfYXV0aF9tZXRob2RcIikgaXMg
Tm9uZSxcbiAgICAgICAgICAgIFwiY3JlZGVudGlhbF9pbnZhbGlkXCIpXG4gICAgc2VjcmV0ID0g
c2F2ZWQuZ2V0KFwiY2xpZW50X3NlY3JldFwiKVxuICAgIHJlcXVpcmUoaXNpbnN0YW5jZShzZWNy
ZXQsIHN0cikgYW5kIDAgPCBsZW4oc2VjcmV0KSA8PSA0MDk2XG4gICAgICAgICAgICBhbmQgYWxs
KDMyIDw9IG9yZCh2YWx1ZSkgPD0gMTI2IGZvciB2YWx1ZSBpbiBzZWNyZXQpLFxuICAgICAgICAg
ICAgXCJjcmVkZW50aWFsX2ludmFsaWRcIilcbiAgICBzYXZlZC5jbGVhcigpXG4gICAgcmV0dXJu
IHNlY3JldFxuXG5cbmNsYXNzIEJ1ZGdldDpcbiAgICBcIlwiXCJPbmUgbWFpbi10aHJlYWQgYWxh
cm0gYm91bmRzIGJsb2NraW5nIHdvcmsgYW5kIHNhbXBsZXMgZnJlZSBzcGFjZS5cIlwiXCJcblxu
ICAgIGRlZiBfX2luaXRfXyhzZWxmLCBsYWIsIHN0YXJ0ZWQsIHNlY29uZHMpOlxuICAgICAgICBz
ZWxmLmxhYiwgc2VsZi5kZWFkbGluZSA9IGxhYiwgc3RhcnRlZCArIHNlY29uZHNcbiAgICAgICAg
c2VsZi5waGFzZV9kZWFkbGluZSA9IHNlbGYucGVuZGluZ19kZWFkbGluZSA9IE5vbmVcbiAgICAg
ICAgc2VsZi5waGFzZV90YWcgPSBcImZpeHR1cmVfZGVhZGxpbmVcIlxuICAgICAgICBzZWxmLm1p
bmltdW1fZnJlZSA9IE5vbmVcbiAgICAgICAgc2VsZi5zYW1wbGVzID0gMFxuICAgICAgICBzZWxm
LnByZXZpb3VzID0ge31cblxuICAgIGRlZiBfX2VudGVyX18oc2VsZik6XG4gICAgICAgIHJlcXVp
cmUoaGFzYXR0cihzaWduYWwsIFwic2V0aXRpbWVyXCIpIGFuZCBoYXNhdHRyKHNpZ25hbCwgXCJT
SUdBTFJNXCIpXG4gICAgICAgICAgICAgICAgYW5kIHNpZ25hbC5nZXRpdGltZXIoc2lnbmFsLklU
SU1FUl9SRUFMKSA9PSAoMC4wLCAwLjApLFxuICAgICAgICAgICAgICAgIFwiYXJndW1lbnRzX2lu
dmFsaWRcIilcbiAgICAgICAgZm9yIG5hbWUgaW4gKHNpZ25hbC5TSUdBTFJNLCBzaWduYWwuU0lH
SU5ULCBzaWduYWwuU0lHVEVSTSk6XG4gICAgICAgICAgICBzZWxmLnByZXZpb3VzW25hbWVdID0g
c2lnbmFsLmdldHNpZ25hbChuYW1lKVxuICAgICAgICAgICAgc2lnbmFsLnNpZ25hbChuYW1lLCBz
ZWxmLnRpY2sgaWYgbmFtZSA9PSBzaWduYWwuU0lHQUxSTSBlbHNlIHNlbGYuaW50ZXJydXB0KVxu
ICAgICAgICBzZWxmLnRpY2soKVxuICAgICAgICByZXR1cm4gc2VsZlxuXG4gICAgZGVmIGludGVy
cnVwdChzZWxmLCBzaWdudW0sIGZyYW1lKTpcbiAgICAgICAgcmFpc2UgSGFsdChcImludGVycnVw
dGVkXCIpXG5cbiAgICBkZWYgdGljayhzZWxmLCBzaWdudW09Tm9uZSwgZnJhbWU9Tm9uZSk6XG4g
ICAgICAgIHRyeTpcbiAgICAgICAgICAgIGZyZWUgPSBzaHV0aWwuZGlza191c2FnZShzZWxmLmxh
YikuZnJlZVxuICAgICAgICBleGNlcHQgRXhjZXB0aW9uOlxuICAgICAgICAgICAgcmFpc2UgSGFs
dChcImRpc2tfb2JzZXJ2YXRpb25fZmFpbGVkXCIpIGZyb20gTm9uZVxuICAgICAgICBzZWxmLnNh
bXBsZXMgKz0gMVxuICAgICAgICBzZWxmLm1pbmltdW1fZnJlZSA9IGZyZWUgaWYgc2VsZi5taW5p
bXVtX2ZyZWUgaXMgTm9uZSBlbHNlIG1pbihzZWxmLm1pbmltdW1fZnJlZSwgZnJlZSlcbiAgICAg
ICAgcmVxdWlyZV9mcmVlID0gZnJlZSA+PSBTVE9QX0ZSRUVfQllURVNcbiAgICAgICAgaWYgbm90
IHJlcXVpcmVfZnJlZTpcbiAgICAgICAgICAgIHJhaXNlIEhhbHQoXCJkaXNrX21hcmdpblwiKVxu
ICAgICAgICBub3cgPSB0aW1lLm1vbm90b25pYygpXG4gICAgICAgIGRlYWRsaW5lcyA9IFsoc2Vs
Zi5kZWFkbGluZSwgXCJmaXh0dXJlX2RlYWRsaW5lXCIpXVxuICAgICAgICBpZiBzZWxmLnBoYXNl
X2RlYWRsaW5lIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAgZGVhZGxpbmVzLmFwcGVuZCgoc2Vs
Zi5waGFzZV9kZWFkbGluZSwgc2VsZi5waGFzZV90YWcpKVxuICAgICAgICBpZiBzZWxmLnBlbmRp
bmdfZGVhZGxpbmUgaXMgbm90IE5vbmU6XG4gICAgICAgICAgICBkZWFkbGluZXMuYXBwZW5kKChz
ZWxmLnBlbmRpbmdfZGVhZGxpbmUsIFwicGVuZGluZ19kZWFkbGluZVwiKSlcbiAgICAgICAgZGVh
ZGxpbmUsIHRhZyA9IG1pbihkZWFkbGluZXMpXG4gICAgICAgIGlmIG5vdyA+PSBkZWFkbGluZTpc
biAgICAgICAgICAgIHJhaXNlIEhhbHQodGFnKVxuICAgICAgICBzaWduYWwuc2V0aXRpbWVyKHNp
Z25hbC5JVElNRVJfUkVBTCwgbWluKDEuMCwgZGVhZGxpbmUgLSBub3cpKVxuXG4gICAgQGNvbnRl
eHRsaWIuY29udGV4dG1hbmFnZXJcbiAgICBkZWYgbGltaXQoc2VsZiwgc2Vjb25kcywgdGFnKTpc
biAgICAgICAgb2xkID0gc2VsZi5waGFzZV9kZWFkbGluZSwgc2VsZi5waGFzZV90YWdcbiAgICAg
ICAgZGVhZGxpbmUgPSB0aW1lLm1vbm90b25pYygpICsgc2Vjb25kc1xuICAgICAgICBpZiBvbGRb
MF0gaXMgTm9uZSBvciBkZWFkbGluZSA8IG9sZFswXTpcbiAgICAgICAgICAgIHNlbGYucGhhc2Vf
ZGVhZGxpbmUsIHNlbGYucGhhc2VfdGFnID0gZGVhZGxpbmUsIHRhZ1xuICAgICAgICB0cnk6XG4g
ICAgICAgICAgICBzZWxmLnRpY2soKVxuICAgICAgICAgICAgeWllbGRcbiAgICAgICAgZmluYWxs
eTpcbiAgICAgICAgICAgIHNlbGYucGhhc2VfZGVhZGxpbmUsIHNlbGYucGhhc2VfdGFnID0gb2xk
XG4gICAgICAgICAgICAjIERvIG5vdCByZXBsYWNlIGFuIGluLWZsaWdodCBIYWx0IHdpdGggYW5v
dGhlciBhbGFybSBkdXJpbmcgdW53aW5kaW5nLlxuICAgICAgICAgICAgaWYgc3lzLmV4Y19pbmZv
KClbMF0gaXMgTm9uZTpcbiAgICAgICAgICAgICAgICBzZWxmLnRpY2soKVxuXG4gICAgZGVmIGNs
b3NlKHNlbGYpOlxuICAgICAgICBpZiBzZWxmLnByZXZpb3VzOlxuICAgICAgICAgICAgc2lnbmFs
LnNldGl0aW1lcihzaWduYWwuSVRJTUVSX1JFQUwsIDApXG4gICAgICAgICAgICBmb3IgbmFtZSwg
aGFuZGxlciBpbiBzZWxmLnByZXZpb3VzLml0ZW1zKCk6XG4gICAgICAgICAgICAgICAgc2lnbmFs
LnNpZ25hbChuYW1lLCBoYW5kbGVyKVxuICAgICAgICAgICAgc2VsZi5wcmV2aW91cy5jbGVhcigp
XG5cblxuZGVmIG9ic2VydmVfdW5leHBlY3RlZF9mYWlsdXJlKGhvbGRlciwgc2l0ZSk6XG4gICAg
XCJcIlwiRmlyc3Qtb25seSBmaW5pdGUgcHJpdmF0ZSBvYnNlcnZhdGlvbjsgbmV2ZXIgcmVwbGFj
ZSB0aGUgb3JpZ2luYWwgb3V0Y29tZS5cIlwiXCJcbiAgICB0cnk6XG4gICAgICAgIGlmIGhvbGRl
cltcIm9ic2VydmVkXCJdIG9yIHNpdGUgbm90IGluIChcImhhbmRsZXJcIiwgXCJzZXJ2ZXJcIiwg
XCJtYWluXCIpOlxuICAgICAgICAgICAgcmV0dXJuXG4gICAgICAgIGhvbGRlcltcIm9ic2VydmVk
XCJdID0gVHJ1ZVxuICAgICAgICBlcnJvciA9IHN5cy5leGNfaW5mbygpWzFdXG4gICAgICAgIGxh
YmVsID0gXCJvdGhlclwiXG4gICAgICAgIGZvciBraW5kLCBuYW1lIGluICgoQXR0cmlidXRlRXJy
b3IsIFwiQXR0cmlidXRlRXJyb3JcIiksXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAoVHlw
ZUVycm9yLCBcIlR5cGVFcnJvclwiKSwgKFZhbHVlRXJyb3IsIFwiVmFsdWVFcnJvclwiKSxcbiAg
ICAgICAgICAgICAgICAgICAgICAgICAgIChLZXlFcnJvciwgXCJLZXlFcnJvclwiKSwgKE9TRXJy
b3IsIFwiT1NFcnJvclwiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIChCcm9rZW5QaXBl
RXJyb3IsIFwiQnJva2VuUGlwZUVycm9yXCIpLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAg
KENvbm5lY3Rpb25SZXNldEVycm9yLCBcIkNvbm5lY3Rpb25SZXNldEVycm9yXCIpLFxuICAgICAg
ICAgICAgICAgICAgICAgICAgICAgKFRpbWVvdXRFcnJvciwgXCJUaW1lb3V0RXJyb3JcIikpOlxu
ICAgICAgICAgICAgaWYgdHlwZShlcnJvcikgaXMga2luZDpcbiAgICAgICAgICAgICAgICBsYWJl
bCA9IG5hbWVcbiAgICAgICAgICAgICAgICBicmVha1xuICAgICAgICBvd25fZnVuY3Rpb24gPSBv
d25fbGluZSA9IE5vbmVcbiAgICAgICAgY29kZXMgPSAoXG4gICAgICAgICAgICAoSGVhZGVyUmVh
ZGVyLnJlYWRsaW5lLl9fY29kZV9fLCBcIkhlYWRlclJlYWRlci5yZWFkbGluZVwiKSxcbiAgICAg
ICAgICAgIChEZW1vU2VydmVyLnByb2Nlc3NfcmVxdWVzdC5fX2NvZGVfXywgXCJEZW1vU2VydmVy
LnByb2Nlc3NfcmVxdWVzdFwiKSxcbiAgICAgICAgICAgIChEZW1vLmJlZ2luLl9fY29kZV9fLCBc
IkRlbW8uYmVnaW5cIiksXG4gICAgICAgICAgICAoRGVtby5jYWxsYmFjay5fX2NvZGVfXywgXCJE
ZW1vLmNhbGxiYWNrXCIpLFxuICAgICAgICAgICAgKERlbW8uaW52b2tlLl9fY29kZV9fLCBcIkRl
bW8uaW52b2tlXCIpLFxuICAgICAgICAgICAgKEhhbmRsZXIuaGFuZGxlX29uZV9yZXF1ZXN0Ll9f
Y29kZV9fLCBcIkhhbmRsZXIuaGFuZGxlX29uZV9yZXF1ZXN0XCIpLFxuICAgICAgICAgICAgKEhh
bmRsZXIuc2VuZF9lcnJvci5fX2NvZGVfXywgXCJIYW5kbGVyLnNlbmRfZXJyb3JcIiksXG4gICAg
ICAgICAgICAoSGFuZGxlci5nZXQuX19jb2RlX18sIFwiSGFuZGxlci5nZXRcIiksXG4gICAgICAg
ICAgICAoSGFuZGxlci5yZXBseS5fX2NvZGVfXywgXCJIYW5kbGVyLnJlcGx5XCIpLFxuICAgICAg
ICAgICAgKG1haW4uX19jb2RlX18sIFwibWFpblwiKSxcbiAgICAgICAgKVxuICAgICAgICB0cmFj
ZSA9IEJhc2VFeGNlcHRpb24uX190cmFjZWJhY2tfXy5fX2dldF9fKGVycm9yLCBCYXNlRXhjZXB0
aW9uKSBpZiBpc2luc3RhbmNlKGVycm9yLCBCYXNlRXhjZXB0aW9uKSBlbHNlIE5vbmVcbiAgICAg
ICAgZm9yIF8gaW4gcmFuZ2UoNjQpOlxuICAgICAgICAgICAgaWYgdHJhY2UgaXMgTm9uZTpcbiAg
ICAgICAgICAgICAgICBicmVha1xuICAgICAgICAgICAgZm9yIGNvZGUsIG5hbWUgaW4gY29kZXM6
XG4gICAgICAgICAgICAgICAgaWYgdHJhY2UudGJfZnJhbWUuZl9jb2RlIGlzIGNvZGUgYW5kIHR5
cGUodHJhY2UudGJfbGluZW5vKSBpcyBpbnQgYW5kIDEgPD0gdHJhY2UudGJfbGluZW5vIDw9IDEw
MjQ6XG4gICAgICAgICAgICAgICAgICAgIG93bl9mdW5jdGlvbiwgb3duX2xpbmUgPSBuYW1lLCB0
cmFjZS50Yl9saW5lbm9cbiAgICAgICAgICAgIHRyYWNlID0gdHJhY2UudGJfbmV4dFxuICAgICAg
ICBpZiB0cmFjZSBpcyBub3QgTm9uZTpcbiAgICAgICAgICAgIG93bl9mdW5jdGlvbiA9IG93bl9s
aW5lID0gTm9uZVxuICAgICAgICBob2xkZXJbXCJkaWFnbm9zdGljXCJdID0ge1wic2l0ZVwiOiBz
aXRlLCBcImV4Y2VwdGlvbl9jbGFzc1wiOiBsYWJlbCxcbiAgICAgICAgICAgICAgICAgICAgICAg
ICAgICAgICAgXCJvd25fZnVuY3Rpb25cIjogb3duX2Z1bmN0aW9uLCBcIm93bl9saW5lXCI6IG93
bl9saW5lfVxuICAgIGV4Y2VwdCBCYXNlRXhjZXB0aW9uOlxuICAgICAgICAjIFByb2plY3Rpb24v
c3RvcmFnZSBmYWlsdXJlcyByZXRhaW4gdGhlIGZpcnN0IGxhdGNoIGFuZCBvcmlnaW5hbCBoYW5k
bGVyLlxuICAgICAgICBwYXNzXG5cblxuY2xhc3MgRGVtbzpcbiAgICBkZWYgX19pbml0X18oc2Vs
ZiwgbW9kdWxlLCB3b3Jrc3BhY2UsIHNlY3JldCwgYnVkZ2V0LCBjaGVja3MsIHN0YXR1c2VzKTpc
biAgICAgICAgc2VsZi5tb2R1bGUsIHNlbGYud29ya3NwYWNlLCBzZWxmLnNlY3JldCA9IG1vZHVs
ZSwgd29ya3NwYWNlLCBzZWNyZXRcbiAgICAgICAgc2VsZi5pc3N1ZXIsIHNlbGYuY2xpZW50X2lk
ID0gSVNTVUVSLCBDTElFTlRfSURcbiAgICAgICAgc2VsZi5idWRnZXQsIHNlbGYuY2hlY2tzID0g
YnVkZ2V0LCBjaGVja3NcbiAgICAgICAgc2VsZi5zdGF0dXNlcyA9IHN0YXR1c2VzXG4gICAgICAg
IHNlbGYuc3RhZ2UgPSBcInNldHVwXCJcbiAgICAgICAgc2VsZi5wZW5kaW5nID0gc2VsZi5jb29r
aWUgPSBzZWxmLnN1YmplY3QgPSBOb25lXG4gICAgICAgIHNlbGYuYXR0ZW1wdGVkID0gc2VsZi5k
b25lID0gRmFsc2VcbiAgICAgICAgc2VsZi5mYWlsdXJlID0gTm9uZVxuICAgICAgICBzZWxmLnJl
cXVlc3RfaW52YWxpZF9yZWFzb24gPSBOb25lXG4gICAgICAgIHNlbGYucHJlZmxvd19hdXRob3Jp
emF0aW9uX3JlZnVzYWxzID0gMFxuICAgICAgICBzZWxmLnVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNl
cnZhdGlvbiA9IHtcIm9ic2VydmVkXCI6IEZhbHNlLCBcImRpYWdub3N0aWNcIjogTm9uZX1cblxu
ICAgIGRlZiBpbnZva2Uoc2VsZiwgc3RhZ2UsIHNlY29uZHMsIGZ1bmN0aW9uLCAqYXJncywgKipr
d2FyZ3MpOlxuICAgICAgICBzZWxmLnN0YWdlID0gc3RhZ2VcbiAgICAgICAgdHJ5OlxuICAgICAg
ICAgICAgd2l0aCBzZWxmLmJ1ZGdldC5saW1pdChzZWNvbmRzLCBzdGFnZSArIFwiX2ZhaWxlZFwi
KTpcbiAgICAgICAgICAgICAgICByZXR1cm4gZnVuY3Rpb24oKmFyZ3MsICoqa3dhcmdzKVxuICAg
ICAgICBleGNlcHQgRmFpbHVyZTpcbiAgICAgICAgICAgIHJhaXNlXG4gICAgICAgIGV4Y2VwdCBF
eGNlcHRpb246XG4gICAgICAgICAgICAjIEltcG9ydGVkIGZhaWx1cmVzIGFyZSBuZXZlciBmb3Jt
YXR0ZWQgb3IgY29waWVkIGludG8gdGhlIHJlY29yZC5cbiAgICAgICAgICAgIHJhaXNlIEZhaWx1
cmUoc3RhZ2UgKyBcIl9mYWlsZWRcIikgZnJvbSBOb25lXG5cbiAgICBkZWYgc2V0dXAoc2VsZiwg
ZXhlY3V0YWJsZSk6XG4gICAgICAgIGRlZiBwcm92aWRlcigpOlxuICAgICAgICAgICAgc2VsZi5v
cGVuc3NsID0gUGF0aChleGVjdXRhYmxlKS5yZXNvbHZlKHN0cmljdD1UcnVlKVxuICAgICAgICAg
ICAgaW5mbyA9IHNlbGYub3BlbnNzbC5zdGF0KClcbiAgICAgICAgICAgIHJlcXVpcmUoc3RyKFBh
dGgoZXhlY3V0YWJsZSkpID09IFwiL29wdC9ob21lYnJldy9iaW4vb3BlbnNzbFwiXG4gICAgICAg
ICAgICAgICAgICAgIGFuZCBzdGF0LlNfSVNSRUcoaW5mby5zdF9tb2RlKSBhbmQgMCA8IGluZm8u
c3Rfc2l6ZSA8PSAzMiAqIDEwMjQgKiAxMDI0LFxuICAgICAgICAgICAgICAgICAgICBcInByb3Zp
ZGVyX2ZhaWxlZFwiKVxuICAgICAgICAgICAgd2l0aCBzZWxmLm9wZW5zc2wub3BlbihcInJiXCIp
IGFzIHNvdXJjZTpcbiAgICAgICAgICAgICAgICByZXF1aXJlKGhhc2hsaWIuZmlsZV9kaWdlc3Qo
c291cmNlLCBcInNoYTI1NlwiKS5oZXhkaWdlc3QoKSA9PSBPUEVOU1NMX1NIQTI1NixcbiAgICAg
ICAgICAgICAgICAgICAgICAgIFwicHJvdmlkZXJfZmFpbGVkXCIpXG4gICAgICAgICAgICBzZWxm
Lm9wZW5zc2xfZW52aXJvbm1lbnQgPSB7a2V5OiB2YWx1ZSBmb3Iga2V5LCB2YWx1ZSBpbiBvcy5l
bnZpcm9uLml0ZW1zKClcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBp
ZiBrZXkgaW4ge1wiUEFUSFwiLCBcIkhPTUVcIiwgXCJUTVBESVJcIiwgXCJMQU5HXCIsIFwiTENf
QUxMXCJ9fVxuICAgICAgICAgICAgcmVzdWx0ID0gc3VicHJvY2Vzcy5ydW4oW3N0cihzZWxmLm9w
ZW5zc2wpLCBcInZlcnNpb25cIl0sIGNhcHR1cmVfb3V0cHV0PVRydWUsXG4gICAgICAgICAgICAg
ICAgICAgICAgICAgICAgICAgICAgICB0aW1lb3V0PTUsIGVudj1zZWxmLm9wZW5zc2xfZW52aXJv
bm1lbnQpXG4gICAgICAgICAgICByZXF1aXJlKHJlc3VsdC5yZXR1cm5jb2RlID09IDBcbiAgICAg
ICAgICAgICAgICAgICAgYW5kIGxlbihyZXN1bHQuc3Rkb3V0KSA8PSAyNTYgYW5kIGxlbihyZXN1
bHQuc3RkZXJyKSA8PSA0MDk2XG4gICAgICAgICAgICAgICAgICAgIGFuZCByZXN1bHQuc3Rkb3V0
LmRlY29kZShcImFzY2lpXCIpLnN0cmlwKCkgPT0gT1BFTlNTTF9WRVJTSU9OLFxuICAgICAgICAg
ICAgICAgICAgICBcInByb3ZpZGVyX2ZhaWxlZFwiKVxuICAgICAgICBzZWxmLmludm9rZShcInBy
b3ZpZGVyXCIsIDUsIHByb3ZpZGVyKVxuICAgICAgICBzZWxmLmNoZWNrc1tcInByb3ZpZGVyX2lk
ZW50aXR5X3ZlcmlmaWVkXCJdID0gVHJ1ZVxuICAgICAgICBkb2N1bWVudCA9IHNlbGYuaW52b2tl
KFwiZGlzY292ZXJ5XCIsIDUsXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgc2VsZi5t
b2R1bGUuTG9jYWxSZWx5aW5nUGFydHkuZGlzY292ZXJ5LCBzZWxmKVxuICAgICAgICBtZXRob2Rz
ID0gZG9jdW1lbnQuZ2V0KFwidG9rZW5fZW5kcG9pbnRfYXV0aF9tZXRob2RzX3N1cHBvcnRlZFwi
KVxuICAgICAgICByZXF1aXJlKGlzaW5zdGFuY2UobWV0aG9kcywgbGlzdCkgYW5kIFwiY2xpZW50
X3NlY3JldF9wb3N0XCIgaW4gbWV0aG9kc1xuICAgICAgICAgICAgICAgIGFuZCBhbGwoaXNpbnN0
YW5jZSh2YWx1ZSwgc3RyKSBmb3IgdmFsdWUgaW4gbWV0aG9kcyksIFwiZGlzY292ZXJ5X2ZhaWxl
ZFwiKVxuICAgICAgICBzZWxmLmRpc2NvdmVyeV9kb2N1bWVudCA9IGRvY3VtZW50XG4gICAgICAg
IHNlbGYuY2hlY2tzW1wiZGlzY292ZXJ5X3ZlcmlmaWVkXCJdID0gVHJ1ZVxuXG4gICAgZGVmIGJl
Z2luKHNlbGYpOlxuICAgICAgICBzZWxmLnN0YWdlID0gXCJmbG93XCJcbiAgICAgICAgcmVxdWly
ZShub3Qgc2VsZi5hdHRlbXB0ZWQsIFwiZmxvd19hbHJlYWR5X3N0YXJ0ZWRcIilcbiAgICAgICAg
c2VsZi5hdHRlbXB0ZWQgPSBUcnVlXG4gICAgICAgIHNlbGYucGVuZGluZyA9IHtcInN0YXRlXCI6
IHNlY3JldHMudG9rZW5fdXJsc2FmZSgzMiksIFwibm9uY2VcIjogc2VjcmV0cy50b2tlbl91cmxz
YWZlKDMyKSxcbiAgICAgICAgICAgICAgICAgICAgICAgIFwidmVyaWZpZXJcIjogc2VjcmV0cy50
b2tlbl91cmxzYWZlKDQ4KSxcbiAgICAgICAgICAgICAgICAgICAgICAgIFwiZmxvd19jb29raWVc
Ijogc2VjcmV0cy50b2tlbl91cmxzYWZlKDMyKSxcbiAgICAgICAgICAgICAgICAgICAgICAgIFwi
c3RhcnRlZF9hdFwiOiBpbnQodGltZS50aW1lKCkpfVxuICAgICAgICBzZWxmLmJ1ZGdldC5wZW5k
aW5nX2RlYWRsaW5lID0gdGltZS5tb25vdG9uaWMoKSArIFBFTkRJTkdfU0VDT05EU1xuICAgICAg
ICBzZWxmLmJ1ZGdldC50aWNrKClcbiAgICAgICAgZmllbGRzID0ge1wicmVzcG9uc2VfdHlwZVwi
OiBcImNvZGVcIiwgXCJjbGllbnRfaWRcIjogQ0xJRU5UX0lELFxuICAgICAgICAgICAgICAgICAg
XCJyZWRpcmVjdF91cmlcIjogQ0FMTEJBQ0ssIFwic2NvcGVcIjogXCJvcGVuaWQgcHJvZmlsZVwi
LFxuICAgICAgICAgICAgICAgICAgXCJzdGF0ZVwiOiBzZWxmLnBlbmRpbmdbXCJzdGF0ZVwiXSwg
XCJub25jZVwiOiBzZWxmLnBlbmRpbmdbXCJub25jZVwiXSxcbiAgICAgICAgICAgICAgICAgIFwi
Y29kZV9jaGFsbGVuZ2VcIjogc2VsZi5tb2R1bGUuYjY0dXJsKFxuICAgICAgICAgICAgICAgICAg
ICAgIGhhc2hsaWIuc2hhMjU2KHNlbGYucGVuZGluZ1tcInZlcmlmaWVyXCJdLmVuY29kZShcImFz
Y2lpXCIpKS5kaWdlc3QoKSksXG4gICAgICAgICAgICAgICAgICBcImNvZGVfY2hhbGxlbmdlX21l
dGhvZFwiOiBcIlMyNTZcIn1cbiAgICAgICAgcmV0dXJuIChzZWxmLmRpc2NvdmVyeV9kb2N1bWVu
dFtcImF1dGhvcml6YXRpb25fZW5kcG9pbnRcIl0gKyBcIj9cIlxuICAgICAgICAgICAgICAgICsg
dXJsbGliLnBhcnNlLnVybGVuY29kZShmaWVsZHMpLCBzZWxmLnBlbmRpbmdbXCJmbG93X2Nvb2tp
ZVwiXSlcblxuICAgIGRlZiBjYWxsYmFjayhzZWxmLCBxdWVyeSwgZmxvd19jb29raWUpOlxuICAg
ICAgICBzZWxmLnN0YWdlID0gXCJjYWxsYmFja1wiXG4gICAgICAgIHJlcXVpcmUoc2VsZi5wZW5k
aW5nIGlzIG5vdCBOb25lLCBcImNhbGxiYWNrX2FscmVhZHlfY29uc3VtZWRcIilcbiAgICAgICAg
cGVuZGluZywgc2VsZi5wZW5kaW5nID0gc2VsZi5wZW5kaW5nLCBOb25lXG4gICAgICAgIHNlbGYu
YnVkZ2V0LnBlbmRpbmdfZGVhZGxpbmUgPSBOb25lXG4gICAgICAgIHJhdyA9IHRva2VucyA9IGFj
Y2VzcyA9IHN1YmplY3QgPSBjb2RlID0gZm9ybSA9IE5vbmVcbiAgICAgICAgdHJ5OlxuICAgICAg
ICAgICAgcmVxdWlyZShzZWxmLm1vZHVsZS5lcXVhbChmbG93X2Nvb2tpZSwgcGVuZGluZ1tcImZs
b3dfY29va2llXCJdKSwgXCJjYWxsYmFja19pbnZhbGlkXCIpXG4gICAgICAgICAgICB0cnk6XG4g
ICAgICAgICAgICAgICAgY29kZSA9IHNlbGYubW9kdWxlLkxvY2FsUmVseWluZ1BhcnR5LmNhbGxi
YWNrX2ZpZWxkcyhzZWxmLCBxdWVyeSwgcGVuZGluZ1tcInN0YXRlXCJdKVxuICAgICAgICAgICAg
ZXhjZXB0IEV4Y2VwdGlvbjpcbiAgICAgICAgICAgICAgICByYWlzZSBGYWlsdXJlKFwiY2FsbGJh
Y2tfaW52YWxpZFwiKSBmcm9tIE5vbmVcbiAgICAgICAgICAgIHNlbGYuY2hlY2tzW1wic3RhdGVf
aXNzdWVyX2Zsb3dfY29va2llX3ZlcmlmaWVkXCJdID0gVHJ1ZVxuICAgICAgICAgICAgZm9ybSA9
IHtcImdyYW50X3R5cGVcIjogXCJhdXRob3JpemF0aW9uX2NvZGVcIiwgXCJjbGllbnRfaWRcIjog
Q0xJRU5UX0lELFxuICAgICAgICAgICAgICAgICAgICBcImNsaWVudF9zZWNyZXRcIjogc2VsZi5z
ZWNyZXQsIFwicmVkaXJlY3RfdXJpXCI6IENBTExCQUNLLFxuICAgICAgICAgICAgICAgICAgICBc
ImNvZGVcIjogY29kZSwgXCJjb2RlX3ZlcmlmaWVyXCI6IHBlbmRpbmdbXCJ2ZXJpZmllclwiXX1c
biAgICAgICAgICAgIHN0YXR1cywgXywgcmF3ID0gc2VsZi5pbnZva2UoXCJ0b2tlbl9leGNoYW5n
ZVwiLCA1LCBzZWxmLm1vZHVsZS5yZXF1ZXN0LFxuICAgICAgICAgICAgICAgICAgICAgICAgICAg
ICAgICAgICAgICAgICBzZWxmLmRpc2NvdmVyeV9kb2N1bWVudFtcInRva2VuX2VuZHBvaW50XCJd
LFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBtZXRob2Q9XCJQT1NU
XCIsIGZvcm09Zm9ybSlcbiAgICAgICAgICAgIHNlbGYuc3RhdHVzZXNbXCJ0b2tlbl9leGNoYW5n
ZVwiXSA9IHN0YXR1c1xuICAgICAgICAgICAgcmVxdWlyZShzdGF0dXMgPT0gMjAwLCBcInRva2Vu
X2V4Y2hhbmdlX2ZhaWxlZFwiKVxuICAgICAgICAgICAgdG9rZW5zID0gc2VsZi5tb2R1bGUuanNv
bl9vYmplY3QocmF3KVxuICAgICAgICAgICAgYWNjZXNzLCBzY29wZSA9IHRva2Vucy5nZXQoXCJh
Y2Nlc3NfdG9rZW5cIiksIHRva2Vucy5nZXQoXCJzY29wZVwiKVxuICAgICAgICAgICAgcmVxdWly
ZShpc2luc3RhbmNlKGFjY2Vzcywgc3RyKSBhbmQgMCA8IGxlbihhY2Nlc3MpIDw9IHNlbGYubW9k
dWxlLk1BWF9UT0tFTlxuICAgICAgICAgICAgICAgICAgICBhbmQgcmUuZnVsbG1hdGNoKHJcIltB
LVphLXowLTlfLi1dK1wiLCBhY2Nlc3MpIGlzIG5vdCBOb25lXG4gICAgICAgICAgICAgICAgICAg
IGFuZCB0b2tlbnMuZ2V0KFwidG9rZW5fdHlwZVwiKSA9PSBcIkJlYXJlclwiIGFuZCBpc2luc3Rh
bmNlKHNjb3BlLCBzdHIpXG4gICAgICAgICAgICAgICAgICAgIGFuZCBzZXQoc2NvcGUuc3BsaXQo
KSkgPT0ge1wib3BlbmlkXCIsIFwicHJvZmlsZVwifSwgXCJ0b2tlbl9leGNoYW5nZV9mYWlsZWRc
IilcbiAgICAgICAgICAgIHNlbGYuY2hlY2tzW1wiY29uZmlkZW50aWFsX3MyNTZfZXhjaGFuZ2Vf
dmVyaWZpZWRcIl0gPSBUcnVlXG4gICAgICAgICAgICAjIEtlZXAgdGhlIHBpbm5lZCBpbXBsZW1l
bnRhdGlvbjsgY2FwIGNvbWJpbmVkIEpXS1Mvc2lnbmF0dXJlIHdvcmtcbiAgICAgICAgICAgICMg
YXQgZml2ZSByZWFsIHNlY29uZHMgYXMgd2VsbCBhcyBpdHMgZXhpc3Rpbmcgc29ja2V0L3Byb2Nl
c3MgY2Fwcy5cbiAgICAgICAgICAgIHN1YmplY3QgPSBzZWxmLmludm9rZShcInRva2VuX3ZhbGlk
YXRpb25cIiwgNSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBzZWxmLm1vZHVs
ZS5Mb2NhbFJlbHlpbmdQYXJ0eS52ZXJpZnlfaWRfdG9rZW4sXG4gICAgICAgICAgICAgICAgICAg
ICAgICAgICAgICAgICAgc2VsZiwgdG9rZW5zLmdldChcImlkX3Rva2VuXCIpLCBhY2Nlc3MsIHBl
bmRpbmcsXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgc2VsZi5kaXNjb3Zlcnlf
ZG9jdW1lbnQpXG4gICAgICAgICAgICBzZWxmLmNoZWNrc1tcInJzMjU2X2p3a3NfaXNzdWVyX2F1
ZGllbmNlX25vbmNlX3RpbWVfYWNjZXNzX2hhc2hfdmVyaWZpZWRcIl0gPSBUcnVlXG4gICAgICAg
ICAgICBzdGF0dXMsIF8sIHJhdyA9IHNlbGYuaW52b2tlKFwidXNlcmluZm9cIiwgNSwgc2VsZi5t
b2R1bGUucmVxdWVzdCxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg
c2VsZi5kaXNjb3ZlcnlfZG9jdW1lbnRbXCJ1c2VyaW5mb19lbmRwb2ludFwiXSwgYmVhcmVyPWFj
Y2VzcylcbiAgICAgICAgICAgIHNlbGYuc3RhdHVzZXNbXCJ1c2VyaW5mb1wiXSA9IHN0YXR1c1xu
ICAgICAgICAgICAgcmVxdWlyZShzdGF0dXMgPT0gMjAwIGFuZCBzZWxmLm1vZHVsZS5lcXVhbChz
ZWxmLm1vZHVsZS5qc29uX29iamVjdChyYXcpLmdldChcInN1YlwiKSwgc3ViamVjdCksXG4gICAg
ICAgICAgICAgICAgICAgIFwidXNlcmluZm9fZmFpbGVkXCIpXG4gICAgICAgICAgICBzZWxmLmNo
ZWNrc1tcInVzZXJpbmZvX3N1YmplY3RfdmVyaWZpZWRcIl0gPSBUcnVlXG4gICAgICAgICAgICBz
ZWxmLmNvb2tpZSwgc2VsZi5zdWJqZWN0ID0gc2VjcmV0cy50b2tlbl91cmxzYWZlKDMyKSwgc3Vi
amVjdFxuICAgICAgICAgICAgcmV0dXJuIHNlbGYuY29va2llXG4gICAgICAgIGZpbmFsbHk6XG4g
ICAgICAgICAgICBwZW5kaW5nLmNsZWFyKClcbiAgICAgICAgICAgIGlmIGZvcm0gaXMgbm90IE5v
bmU6XG4gICAgICAgICAgICAgICAgZm9ybS5jbGVhcigpXG4gICAgICAgICAgICByYXcgPSB0b2tl
bnMgPSBhY2Nlc3MgPSBzdWJqZWN0ID0gY29kZSA9IGZvcm0gPSBOb25lXG4gICAgICAgICAgICBz
ZWxmLnNlY3JldCA9IE5vbmVcblxuICAgIGRlZiBjbGVhcihzZWxmKTpcbiAgICAgICAgaWYgc2Vs
Zi5wZW5kaW5nIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAgc2VsZi5wZW5kaW5nLmNsZWFyKClc
biAgICAgICAgc2VsZi5wZW5kaW5nID0gc2VsZi5jb29raWUgPSBzZWxmLnN1YmplY3QgPSBzZWxm
LnNlY3JldCA9IE5vbmVcbiAgICAgICAgc2VsZi5idWRnZXQucGVuZGluZ19kZWFkbGluZSA9IE5v
bmVcblxuXG5jbGFzcyBIZWFkZXJSZWFkZXI6XG4gICAgZGVmIF9faW5pdF9fKHNlbGYsIHNvdXJj
ZSk6XG4gICAgICAgIHNlbGYuc291cmNlLCBzZWxmLnJlbWFpbmluZywgc2VsZi5saW5lcyA9IHNv
dXJjZSwgTUFYX0hFQURFUlMsIDBcblxuICAgIGRlZiByZWFkbGluZShzZWxmLCBzaXplPS0xKTpc
biAgICAgICAgbGluZSA9IHNlbGYuc291cmNlLnJlYWRsaW5lKG1pbihNQVhfSEVBREVSX0xJTkUg
KyAxLCBzZWxmLnJlbWFpbmluZyArIDEpKVxuICAgICAgICBzZWxmLnJlbWFpbmluZyAtPSBsZW4o
bGluZSlcbiAgICAgICAgc2VsZi5saW5lcyArPSAxXG4gICAgICAgIHJlcXVpcmUobGVuKGxpbmUp
IDw9IE1BWF9IRUFERVJfTElORSBhbmQgc2VsZi5yZW1haW5pbmcgPj0gMFxuICAgICAgICAgICAg
ICAgIGFuZCBzZWxmLmxpbmVzIDw9IE1BWF9IRUFERVJfQ09VTlQgKyAxLCBcInJlcXVlc3RfbGlt
aXRcIilcbiAgICAgICAgcmV0dXJuIGxpbmVcblxuXG5jbGFzcyBEZW1vU2VydmVyKGh0dHAuc2Vy
dmVyLkhUVFBTZXJ2ZXIpOlxuICAgIGRlZiBfX2luaXRfXyhzZWxmLCBkZW1vKTpcbiAgICAgICAg
c2VsZi5kZW1vLCBzZWxmLmFjdGl2ZSA9IGRlbW8sIE5vbmVcbiAgICAgICAgc3VwZXIoKS5fX2lu
aXRfXygoXCIxMjcuMC4wLjFcIiwgMzAwMCksIEhhbmRsZXIpXG4gICAgICAgIHNlbGYudGltZW91
dCA9IDAuMlxuXG4gICAgZGVmIHByb2Nlc3NfcmVxdWVzdChzZWxmLCByZXF1ZXN0LCBhZGRyZXNz
KTpcbiAgICAgICAgc2VsZi5hY3RpdmUgPSByZXF1ZXN0XG4gICAgICAgIHRyeTpcbiAgICAgICAg
ICAgIHNlbGYuZmluaXNoX3JlcXVlc3QocmVxdWVzdCwgYWRkcmVzcylcbiAgICAgICAgZmluYWxs
eTpcbiAgICAgICAgICAgIHNlbGYuc2h1dGRvd25fcmVxdWVzdChyZXF1ZXN0KVxuICAgICAgICAg
ICAgc2VsZi5hY3RpdmUgPSBOb25lXG5cbiAgICBkZWYgaGFuZGxlX2Vycm9yKHNlbGYsIHJlcXVl
c3QsIGFkZHJlc3MpOlxuICAgICAgICB0cnk6XG4gICAgICAgICAgICBvYnNlcnZlX3VuZXhwZWN0
ZWRfZmFpbHVyZShzZWxmLmRlbW8udW5leHBlY3RlZF9mYWlsdXJlX29ic2VydmF0aW9uLCBcInNl
cnZlclwiKVxuICAgICAgICBleGNlcHQgQmFzZUV4Y2VwdGlvbjpcbiAgICAgICAgICAgIHBhc3Nc
biAgICAgICAgc2VsZi5kZW1vLmZhaWx1cmUsIHNlbGYuZGVtby5kb25lID0gXCJ1bmV4cGVjdGVk
X2ZhaWx1cmVcIiwgVHJ1ZVxuXG5cbmNsYXNzIFByZWZsb3dBdXRob3JpemF0aW9uUmVmdXNhbChF
eGNlcHRpb24pOlxuICAgIHBhc3NcblxuXG5jbGFzcyBIYW5kbGVyKGh0dHAuc2VydmVyLkJhc2VI
VFRQUmVxdWVzdEhhbmRsZXIpOlxuICAgIHRpbWVvdXQgPSA1XG4gICAgcHJvdG9jb2xfdmVyc2lv
biA9IFwiSFRUUC8xLjBcIlxuXG4gICAgZGVmIGxvZ19tZXNzYWdlKHNlbGYsIGZvcm1hdCwgKmFy
Z3MpOlxuICAgICAgICBwYXNzXG5cbiAgICBkZWYgcmVjb3JkX3JlcXVlc3RfcmVhc29uKHNlbGYs
IHJlYXNvbik6XG4gICAgICAgIGRlbW8gPSBzZWxmLnNlcnZlci5kZW1vXG4gICAgICAgIGlmIGRl
bW8ucmVxdWVzdF9pbnZhbGlkX3JlYXNvbiBpcyBOb25lIGFuZCByZWFzb24gaW4gUkVRVUVTVF9J
TlZBTElEX1JFQVNPTlM6XG4gICAgICAgICAgICBkZW1vLnJlcXVlc3RfaW52YWxpZF9yZWFzb24g
PSByZWFzb25cblxuICAgIGRlZiByZXF1aXJlX3JlcXVlc3Qoc2VsZiwgY29uZGl0aW9uLCByZWFz
b24pOlxuICAgICAgICBpZiBub3QgY29uZGl0aW9uOlxuICAgICAgICAgICAgc2VsZi5yZWNvcmRf
cmVxdWVzdF9yZWFzb24ocmVhc29uKVxuICAgICAgICAgICAgcmFpc2UgRmFpbHVyZShcInJlcXVl
c3RfaW52YWxpZFwiKVxuXG4gICAgZGVmIHNlbmRfZXJyb3Ioc2VsZiwgY29kZSwgbWVzc2FnZT1O
b25lLCBleHBsYWluPU5vbmUpOlxuICAgICAgICBzZWxmLnJlY29yZF9yZXF1ZXN0X3JlYXNvbihc
Imh0dHBfcGFyc2VcIilcbiAgICAgICAgc2VsZi5zZXJ2ZXIuZGVtby5mYWlsdXJlLCBzZWxmLnNl
cnZlci5kZW1vLmRvbmUgPSBcInJlcXVlc3RfaW52YWxpZFwiLCBUcnVlXG4gICAgICAgIHNlbGYu
cmVwbHkoY29kZSwgXCJMb2NhbCBkZW1vIGNvdWxkIG5vdCBjb21wbGV0ZSB0aGlzIHJlcXVlc3Qu
XCIpXG5cbiAgICBkZWYgaGFuZGxlX29uZV9yZXF1ZXN0KHNlbGYpOlxuICAgICAgICBkZW1vID0g
c2VsZi5zZXJ2ZXIuZGVtb1xuICAgICAgICBzZWxmLnJlcXVlc3RsaW5lLCBzZWxmLnJlcXVlc3Rf
dmVyc2lvbiwgc2VsZi5jb21tYW5kID0gXCJcIiwgXCJIVFRQLzEuMFwiLCBOb25lXG4gICAgICAg
IHNlbGYuY2xvc2VfY29ubmVjdGlvbiA9IFRydWVcbiAgICAgICAgdHJ5OlxuICAgICAgICAgICAg
ZGVtby5zdGFnZSA9IFwicmVxdWVzdFwiXG4gICAgICAgICAgICB3aXRoIGRlbW8uYnVkZ2V0Lmxp
bWl0KDUsIFwicmVxdWVzdF90aW1lb3V0XCIpOlxuICAgICAgICAgICAgICAgIHNlbGYucmF3X3Jl
cXVlc3RsaW5lID0gc2VsZi5yZmlsZS5yZWFkbGluZShNQVhfUkVRVUVTVF9MSU5FICsgMSlcbiAg
ICAgICAgICAgICAgICBpZiBub3Qgc2VsZi5yYXdfcmVxdWVzdGxpbmU6XG4gICAgICAgICAgICAg
ICAgICAgIHJldHVyblxuICAgICAgICAgICAgICAgIHJlcXVpcmUobGVuKHNlbGYucmF3X3JlcXVl
c3RsaW5lKSA8PSBNQVhfUkVRVUVTVF9MSU5FLCBcInJlcXVlc3RfbGltaXRcIilcbiAgICAgICAg
ICAgICAgICBvcmlnaW5hbCwgc2VsZi5yZmlsZSA9IHNlbGYucmZpbGUsIEhlYWRlclJlYWRlcihz
ZWxmLnJmaWxlKVxuICAgICAgICAgICAgICAgIHRyeTpcbiAgICAgICAgICAgICAgICAgICAgaWYg
bm90IHNlbGYucGFyc2VfcmVxdWVzdCgpOlxuICAgICAgICAgICAgICAgICAgICAgICAgcmV0dXJu
XG4gICAgICAgICAgICAgICAgZmluYWxseTpcbiAgICAgICAgICAgICAgICAgICAgc2VsZi5yZmls
ZSA9IG9yaWdpbmFsXG4gICAgICAgICAgICAgICAgc2VsZi5jbG9zZV9jb25uZWN0aW9uID0gVHJ1
ZVxuICAgICAgICAgICAgICAgIHNlbGYucmVxdWlyZV9yZXF1ZXN0KHNlbGYuaGVhZGVycy5nZXRf
YWxsKFwiSG9zdFwiKSA9PSBbQVVUSE9SSVRZXSwgXCJob3N0XCIpXG4gICAgICAgICAgICAgICAg
aWYgKHNlbGYuaGVhZGVycy5nZXRfYWxsKFwiQXV0aG9yaXphdGlvblwiKSBpcyBub3QgTm9uZVxu
ICAgICAgICAgICAgICAgICAgICAgICAgYW5kIGRlbW8uYXR0ZW1wdGVkIGlzIEZhbHNlIGFuZCBk
ZW1vLnBlbmRpbmcgaXMgTm9uZVxuICAgICAgICAgICAgICAgICAgICAgICAgYW5kIGRlbW8uY29v
a2llIGlzIE5vbmUgYW5kIGRlbW8uc3ViamVjdCBpcyBOb25lXG4gICAgICAgICAgICAgICAgICAg
ICAgICBhbmQgZGVtby5wcmVmbG93X2F1dGhvcml6YXRpb25fcmVmdXNhbHMgPCA0KTpcbiAgICAg
ICAgICAgICAgICAgICAgc2VsZi5yZWNvcmRfcmVxdWVzdF9yZWFzb24oXCJhdXRob3JpemF0aW9u
XCIpXG4gICAgICAgICAgICAgICAgICAgIHNlbGYucmVxdWlyZV9yZXF1ZXN0KHNlbGYuaGVhZGVy
cy5nZXRfYWxsKFwiVHJhbnNmZXItRW5jb2RpbmdcIikgaXMgTm9uZSwgXCJ0cmFuc2Zlcl9lbmNv
ZGluZ1wiKVxuICAgICAgICAgICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVzdChzZWxmLmhl
YWRlcnMuZ2V0X2FsbChcIkV4cGVjdFwiKSBpcyBOb25lLCBcImV4cGVjdFwiKVxuICAgICAgICAg
ICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVzdChzZWxmLmhlYWRlcnMuZ2V0X2FsbChcIkNv
bnRlbnQtTGVuZ3RoXCIpIGluIChOb25lLCBbXCIwXCJdKSwgXCJjb250ZW50X2xlbmd0aFwiKVxu
ICAgICAgICAgICAgICAgICAgICB0YXJnZXQgPSB1cmxsaWIucGFyc2UudXJsc3BsaXQoc2VsZi5w
YXRoKVxuICAgICAgICAgICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVzdChub3QgdGFyZ2V0
LnNjaGVtZSwgXCJ0YXJnZXRfc2NoZW1lXCIpXG4gICAgICAgICAgICAgICAgICAgIHNlbGYucmVx
dWlyZV9yZXF1ZXN0KG5vdCB0YXJnZXQubmV0bG9jLCBcInRhcmdldF9uZXRsb2NcIilcbiAgICAg
ICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qobm90IHRhcmdldC5mcmFnbWVudCwg
XCJ0YXJnZXRfZnJhZ21lbnRcIilcbiAgICAgICAgICAgICAgICAgICAgZGVtby5wcmVmbG93X2F1
dGhvcml6YXRpb25fcmVmdXNhbHMgKz0gMVxuICAgICAgICAgICAgICAgICAgICByYWlzZSBQcmVm
bG93QXV0aG9yaXphdGlvblJlZnVzYWxcbiAgICAgICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVx
dWVzdChzZWxmLmhlYWRlcnMuZ2V0X2FsbChcIkF1dGhvcml6YXRpb25cIikgaXMgTm9uZSwgXCJh
dXRob3JpemF0aW9uXCIpXG4gICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qoc2Vs
Zi5oZWFkZXJzLmdldF9hbGwoXCJUcmFuc2Zlci1FbmNvZGluZ1wiKSBpcyBOb25lLCBcInRyYW5z
ZmVyX2VuY29kaW5nXCIpXG4gICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qoc2Vs
Zi5oZWFkZXJzLmdldF9hbGwoXCJFeHBlY3RcIikgaXMgTm9uZSwgXCJleHBlY3RcIilcbiAgICAg
ICAgICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVzdChzZWxmLmhlYWRlcnMuZ2V0X2FsbChcIkNv
bnRlbnQtTGVuZ3RoXCIpIGluIChOb25lLCBbXCIwXCJdKSwgXCJjb250ZW50X2xlbmd0aFwiKVxu
ICAgICAgICAgICAgICAgIHRhcmdldCA9IHVybGxpYi5wYXJzZS51cmxzcGxpdChzZWxmLnBhdGgp
XG4gICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3Qobm90IHRhcmdldC5zY2hlbWUs
IFwidGFyZ2V0X3NjaGVtZVwiKVxuICAgICAgICAgICAgICAgIHNlbGYucmVxdWlyZV9yZXF1ZXN0
KG5vdCB0YXJnZXQubmV0bG9jLCBcInRhcmdldF9uZXRsb2NcIilcbiAgICAgICAgICAgICAgICBz
ZWxmLnJlcXVpcmVfcmVxdWVzdChub3QgdGFyZ2V0LmZyYWdtZW50LCBcInRhcmdldF9mcmFnbWVu
dFwiKVxuICAgICAgICAgICAgaWYgc2VsZi5jb21tYW5kID09IFwiR0VUXCI6XG4gICAgICAgICAg
ICAgICAgc2VsZi5nZXQodGFyZ2V0KVxuICAgICAgICAgICAgZWxpZiBzZWxmLmNvbW1hbmQgPT0g
XCJQT1NUXCI6XG4gICAgICAgICAgICAgICAgc2VsZi5yZXF1aXJlX3JlcXVlc3QodGFyZ2V0LnBh
dGggPT0gXCIvbG9naW5cIiBhbmQgbm90IHRhcmdldC5xdWVyeSwgXCJwb3N0X3RhcmdldFwiKVxu
ICAgICAgICAgICAgICAgIHNlbGYucmVxdWlyZV9yZXF1ZXN0KHNlbGYuaGVhZGVycy5nZXRfYWxs
KFwiT3JpZ2luXCIpID09IFtPUklHSU5dLCBcIm9yaWdpblwiKVxuICAgICAgICAgICAgICAgIHNl
bGYucmVxdWlyZV9yZXF1ZXN0KHNlbGYuaGVhZGVycy5nZXRfYWxsKFwiQ29udGVudC1UeXBlXCIp
ID09IFtcImFwcGxpY2F0aW9uL3gtd3d3LWZvcm0tdXJsZW5jb2RlZFwiXSwgXCJjb250ZW50X3R5
cGVcIilcbiAgICAgICAgICAgICAgICBzZWxmLmNvb2tpZXMoKVxuICAgICAgICAgICAgICAgIGxv
Y2F0aW9uLCBjb29raWUgPSBkZW1vLmJlZ2luKClcbiAgICAgICAgICAgICAgICBzZWxmLnJlcGx5
KDMwMywgXCJcIiwgbG9jYXRpb249bG9jYXRpb24sIGNvb2tpZT0oRkxPV19DT09LSUUsIGNvb2tp
ZSkpXG4gICAgICAgICAgICAgICAgZGVtby5zdGF0dXNlc1tcImF1dGhvcml6YXRpb25fcmVkaXJl
Y3RcIl0gPSAzMDNcbiAgICAgICAgICAgICAgICBkZW1vLmNoZWNrc1tcImF1dGhvcml6YXRpb25f
cmVkaXJlY3RfaXNzdWVkXCJdID0gVHJ1ZVxuICAgICAgICAgICAgZWxzZTpcbiAgICAgICAgICAg
ICAgICBzZWxmLnJlY29yZF9yZXF1ZXN0X3JlYXNvbihcIm1ldGhvZFwiKVxuICAgICAgICAgICAg
ICAgIHJhaXNlIEZhaWx1cmUoXCJyZXF1ZXN0X2ludmFsaWRcIilcbiAgICAgICAgZXhjZXB0IFBy
ZWZsb3dBdXRob3JpemF0aW9uUmVmdXNhbDpcbiAgICAgICAgICAgIGlmIGRlbW8ucHJlZmxvd19h
dXRob3JpemF0aW9uX3JlZnVzYWxzID09IDQ6XG4gICAgICAgICAgICAgICAgZGVtby5mYWlsdXJl
LCBkZW1vLmRvbmUgPSBcInJlcXVlc3RfaW52YWxpZFwiLCBUcnVlXG4gICAgICAgICAgICBzZWxm
LnJlcGx5KDQwMywgXCJMb2NhbCBkZW1vIGNvdWxkIG5vdCBjb21wbGV0ZSB0aGlzIHJlcXVlc3Qu
XCIpXG4gICAgICAgIGV4Y2VwdCBGYWlsdXJlIGFzIGZhaWx1cmU6XG4gICAgICAgICAgICBkZW1v
LmZhaWx1cmUsIGRlbW8uZG9uZSA9IGZhaWx1cmUudGFnLCBUcnVlXG4gICAgICAgICAgICBzZWxm
LnJlcGx5KDQwMCwgXCJMb2NhbCBkZW1vIGNvdWxkIG5vdCBjb21wbGV0ZSB0aGlzIHJlcXVlc3Qu
XCIpXG4gICAgICAgIGV4Y2VwdCBFeGNlcHRpb246XG4gICAgICAgICAgICB0cnk6XG4gICAgICAg
ICAgICAgICAgb2JzZXJ2ZV91bmV4cGVjdGVkX2ZhaWx1cmUoZGVtby51bmV4cGVjdGVkX2ZhaWx1
cmVfb2JzZXJ2YXRpb24sIFwiaGFuZGxlclwiKVxuICAgICAgICAgICAgZXhjZXB0IEJhc2VFeGNl
cHRpb246XG4gICAgICAgICAgICAgICAgcGFzc1xuICAgICAgICAgICAgZGVtby5mYWlsdXJlLCBk
ZW1vLmRvbmUgPSBcInVuZXhwZWN0ZWRfZmFpbHVyZVwiLCBUcnVlXG4gICAgICAgICAgICBzZWxm
LnJlcGx5KDQwMCwgXCJMb2NhbCBkZW1vIGNvdWxkIG5vdCBjb21wbGV0ZSB0aGlzIHJlcXVlc3Qu
XCIpXG5cbiAgICBkZWYgY29va2llcyhzZWxmKTpcbiAgICAgICAgaGVhZGVycyA9IHNlbGYuaGVh
ZGVycy5nZXRfYWxsKFwiQ29va2llXCIsIFtdKVxuICAgICAgICBzZWxmLnJlcXVpcmVfcmVxdWVz
dChsZW4oaGVhZGVycykgPD0gMSwgXCJjb29raWVfaGVhZGVyX2NvdW50XCIpXG4gICAgICAgIHJh
dyA9IGhlYWRlcnNbMF0gaWYgaGVhZGVycyBlbHNlIFwiXCJcbiAgICAgICAgcmVxdWlyZShsZW4o
cmF3KSA8PSBNQVhfQ09PS0lFLCBcInJlcXVlc3RfbGltaXRcIilcbiAgICAgICAgcGllY2VzID0g
cmF3LnNwbGl0KFwiO1wiKSBpZiByYXcgZWxzZSBbXVxuICAgICAgICByZXF1aXJlKGxlbihwaWVj
ZXMpIDw9IE1BWF9IRUFERVJfQ09VTlQsIFwicmVxdWVzdF9saW1pdFwiKVxuICAgICAgICBvd24g
PSB7fVxuICAgICAgICBmb3IgcGllY2UgaW4gcGllY2VzOlxuICAgICAgICAgICAgbmFtZSwgc2Vw
YXJhdG9yLCB2YWx1ZSA9IHBpZWNlLnN0cmlwKCkucGFydGl0aW9uKFwiPVwiKVxuICAgICAgICAg
ICAgaWYgbmFtZSBpbiAoRkxPV19DT09LSUUsIEFQUF9DT09LSUUpOlxuICAgICAgICAgICAgICAg
IHNlbGYucmVxdWlyZV9yZXF1ZXN0KHNlcGFyYXRvciBhbmQgbmFtZSBub3QgaW4gb3duXG4gICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgYW5kIHJlLmZ1bGxtYXRjaChyXCJbQS1a
YS16MC05Xy1dezQzfVwiLCB2YWx1ZSkgaXMgbm90IE5vbmUsXG4gICAgICAgICAgICAgICAgICAg
ICAgICAgICAgICAgICAgICAgXCJvd25fY29va2llX3NoYXBlXCIpXG4gICAgICAgICAgICAgICAg
b3duW25hbWVdID0gdmFsdWVcbiAgICAgICAgcmV0dXJuIG93blxuXG4gICAgZGVmIGdldChzZWxm
LCB0YXJnZXQpOlxuICAgICAgICBkZW1vLCBjb29raWVzID0gc2VsZi5zZXJ2ZXIuZGVtbywgc2Vs
Zi5jb29raWVzKClcbiAgICAgICAgaWYgdGFyZ2V0LnBhdGggPT0gXCIvY2FsbGJhY2tcIjpcbiAg
ICAgICAgICAgIGNvb2tpZSA9IGRlbW8uY2FsbGJhY2sodGFyZ2V0LnF1ZXJ5LCBjb29raWVzLmdl
dChGTE9XX0NPT0tJRSkpXG4gICAgICAgICAgICBzZWxmLnJlcGx5KDMwMywgXCJcIiwgbG9jYXRp
b249XCIvcHJvdGVjdGVkXCIsIGNvb2tpZT0oQVBQX0NPT0tJRSwgY29va2llKSwgY2xlYXJfZmxv
dz1UcnVlKVxuICAgICAgICAgICAgZGVtby5zdGF0dXNlc1tcImNhbGxiYWNrXCJdID0gMzAzXG4g
ICAgICAgIGVsaWYgdGFyZ2V0LnBhdGggPT0gXCIvcHJvdGVjdGVkXCIgYW5kIG5vdCB0YXJnZXQu
cXVlcnk6XG4gICAgICAgICAgICBkZW1vLnN0YWdlID0gXCJwcm90ZWN0ZWRcIlxuICAgICAgICAg
ICAgYXV0aGVudGljYXRlZCA9IChkZW1vLnN1YmplY3QgaXMgbm90IE5vbmUgYW5kIGRlbW8uY29v
a2llIGlzIG5vdCBOb25lXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgIGFuZCBkZW1vLm1v
ZHVsZS5lcXVhbChjb29raWVzLmdldChBUFBfQ09PS0lFKSwgZGVtby5jb29raWUpKVxuICAgICAg
ICAgICAgc2VsZi5yZXBseSgyMDAgaWYgYXV0aGVudGljYXRlZCBlbHNlIDQwMyxcbiAgICAgICAg
ICAgICAgICAgICAgICAgXCJTaWduZWQgaW4uIFByb3RlY3RlZCBhcHBsaWNhdGlvbiBhY2Nlc3Mg
aXMgYXZhaWxhYmxlLlwiIGlmIGF1dGhlbnRpY2F0ZWRcbiAgICAgICAgICAgICAgICAgICAgICAg
ZWxzZSBcIlNpZ24gaW4gcmVxdWlyZWQuXCIsIHByb3RlY3RlZD1hdXRoZW50aWNhdGVkKVxuICAg
ICAgICAgICAgaWYgYXV0aGVudGljYXRlZDpcbiAgICAgICAgICAgICAgICBkZW1vLmNoZWNrc1tc
InByb3RlY3RlZF93aXRoX2ZyZXNoX2Nvb2tpZV9hY2NlcHRlZFwiXSA9IFRydWVcbiAgICAgICAg
ICAgICAgICBkZW1vLnN0YXR1c2VzW1wicHJvdGVjdGVkX2FmdGVyXCJdID0gMjAwXG4gICAgICAg
ICAgICAgICAgaWYgbm90IGRlbW8uY2hlY2tzW1wicHJvdGVjdGVkX3dpdGhvdXRfY29va2llX2Rl
bmllZFwiXTpcbiAgICAgICAgICAgICAgICAgICAgZGVtby5mYWlsdXJlID0gXCJwcm90ZWN0ZWRf
YmVmb3JlX3Vub2JzZXJ2ZWRcIlxuICAgICAgICAgICAgICAgIGRlbW8uZG9uZSA9IFRydWVcbiAg
ICAgICAgICAgIGVsaWYgbm90IGRlbW8uYXR0ZW1wdGVkIGFuZCBBUFBfQ09PS0lFIG5vdCBpbiBj
b29raWVzOlxuICAgICAgICAgICAgICAgIGRlbW8uY2hlY2tzW1wicHJvdGVjdGVkX3dpdGhvdXRf
Y29va2llX2RlbmllZFwiXSA9IFRydWVcbiAgICAgICAgICAgICAgICBkZW1vLnN0YXR1c2VzW1wi
cHJvdGVjdGVkX2JlZm9yZVwiXSA9IDQwM1xuICAgICAgICBlbGlmIHRhcmdldC5wYXRoID09IFwi
L1wiIGFuZCBub3QgdGFyZ2V0LnF1ZXJ5OlxuICAgICAgICAgICAgc2VsZi5yZXBseSgyMDAsIFwi
VXNlIFNpZ24gaW4gdG8gb3BlbiB0aGlzIGxvY2FsIGFwcGxpY2F0aW9uLlwiLCBzdGFydD1UcnVl
KVxuICAgICAgICBlbHNlOlxuICAgICAgICAgICAgc2VsZi5yZXBseSg0MDQsIFwiVGhpcyBwYWdl
IGlzIHVuYXZhaWxhYmxlLlwiKVxuXG4gICAgZGVmIHJlcGx5KHNlbGYsIHN0YXR1cywgdGV4dCwg
KiwgbG9jYXRpb249Tm9uZSwgY29va2llPU5vbmUsIGNsZWFyX2Zsb3c9RmFsc2UsXG4gICAgICAg
ICAgICAgIHN0YXJ0PUZhbHNlLCBwcm90ZWN0ZWQ9RmFsc2UpOlxuICAgICAgICBoZWFkaW5nID0g
XCJQcm90ZWN0ZWQgYXBwbGljYXRpb24gYWNjZXNzXCIgaWYgcHJvdGVjdGVkIGVsc2UgXCJMb2Nh
bCBkZW1vXCJcbiAgICAgICAgZm9ybSA9ICgnPGZvcm0gbWV0aG9kPVwicG9zdFwiIGFjdGlvbj1c
Ii9sb2dpblwiPjxidXR0b24gdHlwZT1cInN1Ym1pdFwiPlNpZ24gaW48L2J1dHRvbj48L2Zvcm0+
J1xuICAgICAgICAgICAgICAgIGlmIHN0YXJ0IGVsc2UgXCJcIilcbiAgICAgICAgcmF3ID0gKCc8
IWRvY3R5cGUgaHRtbD48aHRtbCBsYW5nPVwiZW5cIj48bWV0YSBjaGFyc2V0PVwidXRmLThcIj4n
XG4gICAgICAgICAgICAgICAnPHRpdGxlPkxvY2FsIGRlbW88L3RpdGxlPjxoMT4nICsgaGVhZGlu
ZyArICc8L2gxPjxwPicgKyB0ZXh0ICsgJzwvcD4nXG4gICAgICAgICAgICAgICArIGZvcm0gKyAn
PC9odG1sPicpLmVuY29kZShcInV0Zi04XCIpXG4gICAgICAgIHdpdGggc2VsZi5zZXJ2ZXIuZGVt
by5idWRnZXQubGltaXQoNSwgXCJyZXNwb25zZV90aW1lb3V0XCIpOlxuICAgICAgICAgICAgc2Vs
Zi5zZW5kX3Jlc3BvbnNlKHN0YXR1cylcbiAgICAgICAgICAgIHNlbGYuc2VuZF9oZWFkZXIoXCJD
b250ZW50LVR5cGVcIiwgXCJ0ZXh0L2h0bWw7IGNoYXJzZXQ9dXRmLThcIilcbiAgICAgICAgICAg
IHNlbGYuc2VuZF9oZWFkZXIoXCJDb250ZW50LUxlbmd0aFwiLCBzdHIobGVuKHJhdykpKVxuICAg
ICAgICAgICAgc2VsZi5zZW5kX2hlYWRlcihcIkNvbm5lY3Rpb25cIiwgXCJjbG9zZVwiKVxuICAg
ICAgICAgICAgc2VsZi5zZW5kX2hlYWRlcihcIkNhY2hlLUNvbnRyb2xcIiwgXCJuby1zdG9yZVwi
KVxuICAgICAgICAgICAgc2VsZi5zZW5kX2hlYWRlcihcIlJlZmVycmVyLVBvbGljeVwiLCBcIm5v
LXJlZmVycmVyXCIpXG4gICAgICAgICAgICBzZWxmLnNlbmRfaGVhZGVyKFwiQ29udGVudC1TZWN1
cml0eS1Qb2xpY3lcIiwgXCJkZWZhdWx0LXNyYyAnbm9uZSc7IGZvcm0tYWN0aW9uICdzZWxmJzsg
ZnJhbWUtYW5jZXN0b3JzICdub25lJ1wiKVxuICAgICAgICAgICAgaWYgbG9jYXRpb24gaXMgbm90
IE5vbmU6XG4gICAgICAgICAgICAgICAgc2VsZi5zZW5kX2hlYWRlcihcIkxvY2F0aW9uXCIsIGxv
Y2F0aW9uKVxuICAgICAgICAgICAgaWYgY29va2llIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAg
ICAgIHNlbGYuc2VuZF9oZWFkZXIoXCJTZXQtQ29va2llXCIsIGNvb2tpZVswXSArIFwiPVwiICsg
Y29va2llWzFdXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICArIFwiOyBIdHRwT25s
eTsgU2FtZVNpdGU9TGF4OyBQYXRoPS9cIilcbiAgICAgICAgICAgIGlmIGNsZWFyX2Zsb3c6XG4g
ICAgICAgICAgICAgICAgc2VsZi5zZW5kX2hlYWRlcihcIlNldC1Db29raWVcIiwgRkxPV19DT09L
SUUgKyBcIj07IE1heC1BZ2U9MDsgSHR0cE9ubHk7IFNhbWVTaXRlPUxheDsgUGF0aD0vXCIpXG4g
ICAgICAgICAgICBzZWxmLmVuZF9oZWFkZXJzKClcbiAgICAgICAgICAgIHNlbGYud2ZpbGUud3Jp
dGUocmF3KVxuICAgICAgICAgICAgc2VsZi53ZmlsZS5mbHVzaCgpXG4gICAgICAgIHNlbGYuY2xv
c2VfY29ubmVjdGlvbiA9IFRydWVcblxuXG5jbGFzcyBRdWlldFBhcnNlcihhcmdwYXJzZS5Bcmd1
bWVudFBhcnNlcik6XG4gICAgZGVmIGVycm9yKHNlbGYsIG1lc3NhZ2UpOlxuICAgICAgICByYWlz
ZSBGYWlsdXJlKFwiYXJndW1lbnRzX2ludmFsaWRcIilcblxuXG5kZWYgbWFpbigpOlxuICAgIHN0
YXJ0ZWQgPSB0aW1lLm1vbm90b25pYygpXG4gICAgYnVkZ2V0ID0gZGVtbyA9IHNlcnZlciA9IGV2
aWRlbmNlX2ZkID0gd29ya3NwYWNlID0gc2VjcmV0ID0gTm9uZVxuICAgIHN0YWdlID0gXCJhcmd1
bWVudHNcIlxuICAgIHJlY29yZCA9IHtcInNjaGVtYVwiOiBcInJpYXV0aC5kMDEtY29uZmlkZW50
aWFsLWJyb3dzZXIvdjFcIiwgXCJyZXN1bHRcIjogXCJmYWlsZWRcIixcbiAgICAgICAgICAgICAg
XCJmYWlsdXJlX3N0YWdlXCI6IE5vbmUsIFwiZmFpbHVyZV90YWdcIjogTm9uZSwgXCJjaGVja3Nc
Ijoge30sIFwiY2xlYW51cFwiOiB7fSxcbiAgICAgICAgICAgICAgXCJyZXF1ZXN0X2ludmFsaWRf
cmVhc29uXCI6IE5vbmUsIFwicHJlZmxvd19hdXRob3JpemF0aW9uX3JlZnVzYWxzXCI6IDAsXG4g
ICAgICAgICAgICAgIFwidW5leHBlY3RlZF9mYWlsdXJlX29ic2VydmF0aW9uXCI6IHtcIm9ic2Vy
dmVkXCI6IEZhbHNlLCBcImRpYWdub3N0aWNcIjogTm9uZX0sXG4gICAgICAgICAgICAgIFwic291
cmNlXCI6IHtcInZlcmlmaWVyX2NvbW1pdFwiOiBWRVJJRklFUl9DT01NSVQsIFwidmVyaWZpZXJf
YmxvYlwiOiBWRVJJRklFUl9CTE9CLFxuICAgICAgICAgICAgICAgICAgICAgICAgIFwidmVyaWZp
ZXJfZXhwZWN0ZWRfc2hhMjU2XCI6IFZFUklGSUVSX1NIQTI1Nn0sXG4gICAgICAgICAgICAgIFwi
cHJvdmlkZXJcIjogTm9uZSxcbiAgICAgICAgICAgICAgXCJodHRwX3N0YXR1c2VzXCI6IHtuYW1l
OiBOb25lIGZvciBuYW1lIGluXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgIChcImF1
dGhvcml6YXRpb25fcmVkaXJlY3RcIiwgXCJjYWxsYmFja1wiLCBcInRva2VuX2V4Y2hhbmdlXCIs
XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBcInVzZXJpbmZvXCIsIFwicHJvdGVj
dGVkX2JlZm9yZVwiLCBcInByb3RlY3RlZF9hZnRlclwiKX19XG4gICAgdHJ5OlxuICAgICAgICBw
YXJzZXIgPSBRdWlldFBhcnNlcihhZGRfaGVscD1GYWxzZSwgYWxsb3dfYWJicmV2PUZhbHNlKVxu
ICAgICAgICBmb3IgbmFtZSBpbiAoXCJ3b3Jrc3BhY2VcIiwgXCJzZWNyZXQtZmlsZVwiLCBcInZl
cmlmaWVyLWhlbHBlclwiLCBcIm9wZW5zc2xcIiwgXCJldmlkZW5jZVwiKTpcbiAgICAgICAgICAg
IHBhcnNlci5hZGRfYXJndW1lbnQoXCItLVwiICsgbmFtZSwgcmVxdWlyZWQ9VHJ1ZSlcbiAgICAg
ICAgcGFyc2VyLmFkZF9hcmd1bWVudChcIi0tZGVhZGxpbmUtc2Vjb25kc1wiLCB0eXBlPWludCwg
ZGVmYXVsdD02MDApXG4gICAgICAgIGFyZ3MgPSBwYXJzZXIucGFyc2VfYXJncygpXG4gICAgICAg
IHJlcXVpcmUoc3lzLnZlcnNpb25faW5mbyA+PSAoMywgMTEpIGFuZCAxIDw9IGFyZ3MuZGVhZGxp
bmVfc2Vjb25kcyA8PSA2MDAsXG4gICAgICAgICAgICAgICAgXCJhcmd1bWVudHNfaW52YWxpZFwi
KVxuICAgICAgICBzdGFnZSA9IFwicGF0aHNcIlxuICAgICAgICB3b3Jrc3BhY2UgPSBwcml2YXRl
X2RpcmVjdG9yeShhcmdzLndvcmtzcGFjZSlcbiAgICAgICAgbGFiID0gcHJpdmF0ZV9kaXJlY3Rv
cnkod29ya3NwYWNlLnBhcmVudClcbiAgICAgICAgcmVxdWlyZShub3QgYW55KHdvcmtzcGFjZS5p
dGVyZGlyKCkpLCBcInBhdGhzX2ludmFsaWRcIilcbiAgICAgICAgc2VjcmV0X3BhdGgsIHZlcmlm
aWVyX3BhdGgsIGV2aWRlbmNlX3BhdGggPSBbUGF0aChvcy5wYXRoLmFic3BhdGgodmFsdWUpKSBm
b3IgdmFsdWUgaW5cbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg
ICAgICAgIChhcmdzLnNlY3JldF9maWxlLCBhcmdzLnZlcmlmaWVyX2hlbHBlciwgYXJncy5ldmlk
ZW5jZSldXG4gICAgICAgIHJlcXVpcmUocHJpdmF0ZV9kaXJlY3Rvcnkoc2VjcmV0X3BhdGgucGFy
ZW50KS5wYXJlbnQgPT0gbGFiXG4gICAgICAgICAgICAgICAgYW5kIHZlcmlmaWVyX3BhdGgucGFy
ZW50ID09IGxhYiBhbmQgZXZpZGVuY2VfcGF0aC5wYXJlbnQgPT0gbGFiXG4gICAgICAgICAgICAg
ICAgYW5kIGxlbih7c2VjcmV0X3BhdGgsIHZlcmlmaWVyX3BhdGgsIGV2aWRlbmNlX3BhdGh9KSA9
PSAzLCBcInBhdGhzX2ludmFsaWRcIilcbiAgICAgICAgYnVkZ2V0ID0gQnVkZ2V0KGxhYiwgc3Rh
cnRlZCwgYXJncy5kZWFkbGluZV9zZWNvbmRzKVxuICAgICAgICBidWRnZXQuX19lbnRlcl9fKClc
biAgICAgICAgZXZpZGVuY2VfZmQgPSBvcy5vcGVuKGV2aWRlbmNlX3BhdGgsIG9zLk9fV1JPTkxZ
IHwgb3MuT19DUkVBVCB8IG9zLk9fRVhDTCB8IG9zLk9fTk9GT0xMT1csIDBvNjAwKVxuICAgICAg
ICBvcy5mY2htb2QoZXZpZGVuY2VfZmQsIDBvNjAwKVxuICAgICAgICBzdGFnZSA9IFwidmVyaWZp
ZXJcIlxuICAgICAgICBtb2R1bGUgPSBsb2FkX3ZlcmlmaWVyKHZlcmlmaWVyX3BhdGgpXG4gICAg
ICAgIHJlY29yZFtcInNvdXJjZVwiXVtcInZlcmlmaWVyX3NoYTI1NlwiXSA9IFZFUklGSUVSX1NI
QTI1NlxuICAgICAgICByZWNvcmRbXCJzb3VyY2VcIl1bXCJoZWxwZXJfc2hhMjU2XCJdID0gaGFz
aGxpYi5zaGEyNTYoUGF0aChfX2ZpbGVfXykucmVhZF9ieXRlcygpKS5oZXhkaWdlc3QoKVxuICAg
ICAgICBzdGFnZSA9IFwiY3JlZGVudGlhbFwiXG4gICAgICAgIHNlY3JldCA9IGNsaWVudF9zZWNy
ZXQobW9kdWxlLCBzZWNyZXRfcGF0aClcbiAgICAgICAgZm9yIG5hbWUgaW4gKFwiY3JlZGVudGlh
bF9wcml2YXRlX3ZhbGlkYXRlZFwiLCBcInByb3ZpZGVyX2lkZW50aXR5X3ZlcmlmaWVkXCIsIFwi
ZGlzY292ZXJ5X3ZlcmlmaWVkXCIsXG4gICAgICAgICAgICAgICAgICAgICBcInByb3RlY3RlZF93
aXRob3V0X2Nvb2tpZV9kZW5pZWRcIiwgXCJhdXRob3JpemF0aW9uX3JlZGlyZWN0X2lzc3VlZFwi
LFxuICAgICAgICAgICAgICAgICAgICAgXCJzdGF0ZV9pc3N1ZXJfZmxvd19jb29raWVfdmVyaWZp
ZWRcIiwgXCJjb25maWRlbnRpYWxfczI1Nl9leGNoYW5nZV92ZXJpZmllZFwiLFxuICAgICAgICAg
ICAgICAgICAgICAgXCJyczI1Nl9qd2tzX2lzc3Vlcl9hdWRpZW5jZV9ub25jZV90aW1lX2FjY2Vz
c19oYXNoX3ZlcmlmaWVkXCIsXG4gICAgICAgICAgICAgICAgICAgICBcInVzZXJpbmZvX3N1Ympl
Y3RfdmVyaWZpZWRcIiwgXCJwcm90ZWN0ZWRfd2l0aF9mcmVzaF9jb29raWVfYWNjZXB0ZWRcIik6
XG4gICAgICAgICAgICByZWNvcmRbXCJjaGVja3NcIl1bbmFtZV0gPSBuYW1lID09IFwiY3JlZGVu
dGlhbF9wcml2YXRlX3ZhbGlkYXRlZFwiXG4gICAgICAgIGRlbW8gPSBEZW1vKG1vZHVsZSwgd29y
a3NwYWNlLCBzZWNyZXQsIGJ1ZGdldCwgcmVjb3JkW1wiY2hlY2tzXCJdLCByZWNvcmRbXCJodHRw
X3N0YXR1c2VzXCJdKVxuICAgICAgICByZWNvcmRbXCJ1bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2
YXRpb25cIl0gPSBkZW1vLnVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvblxuICAgICAgICBz
ZWNyZXQgPSBOb25lXG4gICAgICAgIGRlbW8uc2V0dXAoYXJncy5vcGVuc3NsKVxuICAgICAgICBy
ZWNvcmRbXCJwcm92aWRlclwiXSA9IHtcInNoYTI1NlwiOiBPUEVOU1NMX1NIQTI1NiwgXCJ2ZXJz
aW9uXCI6IE9QRU5TU0xfVkVSU0lPTn1cbiAgICAgICAgZGVtby5zdGFnZSA9IFwibGlzdGVuZXJc
IlxuICAgICAgICB0cnk6XG4gICAgICAgICAgICBzZXJ2ZXIgPSBEZW1vU2VydmVyKGRlbW8pXG4g
ICAgICAgIGV4Y2VwdCBFeGNlcHRpb246XG4gICAgICAgICAgICByYWlzZSBGYWlsdXJlKFwibGlz
dGVuZXJfZmFpbGVkXCIpIGZyb20gTm9uZVxuICAgICAgICBwcmludChqc29uLmR1bXBzKHtcInJl
YWR5XCI6IFRydWUsIFwicG9ydFwiOiAzMDAwLCBcInBpZFwiOiBvcy5nZXRwaWQoKX0pLCBmbHVz
aD1UcnVlKVxuICAgICAgICB3aGlsZSBub3QgZGVtby5kb25lOlxuICAgICAgICAgICAgYnVkZ2V0
LnRpY2soKVxuICAgICAgICAgICAgc2VydmVyLmhhbmRsZV9yZXF1ZXN0KClcbiAgICAgICAgaWYg
ZGVtby5mYWlsdXJlIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAgcmFpc2UgRmFpbHVyZShkZW1v
LmZhaWx1cmUpXG4gICAgICAgIHJlcXVpcmUoYWxsKHJlY29yZFtcImNoZWNrc1wiXS52YWx1ZXMo
KSksIFwidW5leHBlY3RlZF9mYWlsdXJlXCIpXG4gICAgICAgIHJlY29yZFtcInJlc3VsdFwiXSA9
IFwicGFzc2VkXCJcbiAgICBleGNlcHQgKEZhaWx1cmUsIEhhbHQpIGFzIGZhaWx1cmU6XG4gICAg
ICAgIHJlY29yZFtcImZhaWx1cmVfdGFnXCJdID0gZmFpbHVyZS50YWdcbiAgICAgICAgcmVjb3Jk
W1wiZmFpbHVyZV9zdGFnZVwiXSA9IGRlbW8uc3RhZ2UgaWYgZGVtbyBpcyBub3QgTm9uZSBlbHNl
IHN0YWdlXG4gICAgZXhjZXB0IEtleWJvYXJkSW50ZXJydXB0OlxuICAgICAgICByZWNvcmRbXCJm
YWlsdXJlX3RhZ1wiXSwgcmVjb3JkW1wiZmFpbHVyZV9zdGFnZVwiXSA9IFwiaW50ZXJydXB0ZWRc
Iiwgc3RhZ2VcbiAgICBleGNlcHQgRXhjZXB0aW9uOlxuICAgICAgICB0cnk6XG4gICAgICAgICAg
ICBvYnNlcnZlX3VuZXhwZWN0ZWRfZmFpbHVyZShyZWNvcmRbXCJ1bmV4cGVjdGVkX2ZhaWx1cmVf
b2JzZXJ2YXRpb25cIl0sIFwibWFpblwiKVxuICAgICAgICBleGNlcHQgQmFzZUV4Y2VwdGlvbjpc
biAgICAgICAgICAgIHBhc3NcbiAgICAgICAgcmVjb3JkW1wiZmFpbHVyZV90YWdcIl0gPSBcInVu
ZXhwZWN0ZWRfZmFpbHVyZVwiXG4gICAgICAgIHJlY29yZFtcImZhaWx1cmVfc3RhZ2VcIl0gPSBk
ZW1vLnN0YWdlIGlmIGRlbW8gaXMgbm90IE5vbmUgZWxzZSBzdGFnZVxuICAgIGZpbmFsbHk6XG4g
ICAgICAgIGNsZWFudXBfZmFpbGVkID0gRmFsc2VcbiAgICAgICAgaWYgYnVkZ2V0IGlzIG5vdCBO
b25lOlxuICAgICAgICAgICAgdHJ5OlxuICAgICAgICAgICAgICAgIGJ1ZGdldC5jbG9zZSgpXG4g
ICAgICAgICAgICBleGNlcHQgQmFzZUV4Y2VwdGlvbjpcbiAgICAgICAgICAgICAgICBjbGVhbnVw
X2ZhaWxlZCA9IFRydWVcbiAgICAgICAgICAgIHJlY29yZFtcIm1pbmltdW1fZnJlZV9ieXRlc1wi
XSwgcmVjb3JkW1wiZGlza19zYW1wbGVzXCJdID0gYnVkZ2V0Lm1pbmltdW1fZnJlZSwgYnVkZ2V0
LnNhbXBsZXNcbiAgICAgICAgaWYgc2VydmVyIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAgdHJ5
OlxuICAgICAgICAgICAgICAgIGlmIHNlcnZlci5hY3RpdmUgaXMgbm90IE5vbmU6XG4gICAgICAg
ICAgICAgICAgICAgIHNlcnZlci5zaHV0ZG93bl9yZXF1ZXN0KHNlcnZlci5hY3RpdmUpXG4gICAg
ICAgICAgICAgICAgICAgIHNlcnZlci5hY3RpdmUgPSBOb25lXG4gICAgICAgICAgICBleGNlcHQg
QmFzZUV4Y2VwdGlvbjpcbiAgICAgICAgICAgICAgICBjbGVhbnVwX2ZhaWxlZCA9IFRydWVcbiAg
ICAgICAgICAgIHRyeTpcbiAgICAgICAgICAgICAgICBzZXJ2ZXIuc2VydmVyX2Nsb3NlKClcbiAg
ICAgICAgICAgIGV4Y2VwdCBCYXNlRXhjZXB0aW9uOlxuICAgICAgICAgICAgICAgIGNsZWFudXBf
ZmFpbGVkID0gVHJ1ZVxuICAgICAgICAgICAgcmVjb3JkW1wiY2xlYW51cFwiXVtcImxpc3RlbmVy
X2Nsb3NlZFwiXSA9IHNlcnZlci5zb2NrZXQuZmlsZW5vKCkgPT0gLTFcbiAgICAgICAgICAgIHJl
Y29yZFtcImNsZWFudXBcIl1bXCJjb25uZWN0aW9uX2Nsb3NlZFwiXSA9IHNlcnZlci5hY3RpdmUg
aXMgTm9uZVxuICAgICAgICBpZiBkZW1vIGlzIG5vdCBOb25lOlxuICAgICAgICAgICAgcmVjb3Jk
W1wicmVxdWVzdF9pbnZhbGlkX3JlYXNvblwiXSA9IGRlbW8ucmVxdWVzdF9pbnZhbGlkX3JlYXNv
blxuICAgICAgICAgICAgcmVjb3JkW1wicHJlZmxvd19hdXRob3JpemF0aW9uX3JlZnVzYWxzXCJd
ID0gZGVtby5wcmVmbG93X2F1dGhvcml6YXRpb25fcmVmdXNhbHNcbiAgICAgICAgICAgIHRyeTpc
biAgICAgICAgICAgICAgICBkZW1vLmNsZWFyKClcbiAgICAgICAgICAgICAgICByZWNvcmRbXCJj
bGVhbnVwXCJdW1wicHJpdmF0ZV9yZWZlcmVuY2VzX2NsZWFyZWRcIl0gPSBUcnVlXG4gICAgICAg
ICAgICBleGNlcHQgQmFzZUV4Y2VwdGlvbjpcbiAgICAgICAgICAgICAgICBjbGVhbnVwX2ZhaWxl
ZCA9IFRydWVcbiAgICAgICAgICAgICAgICByZWNvcmRbXCJjbGVhbnVwXCJdW1wicHJpdmF0ZV9y
ZWZlcmVuY2VzX2NsZWFyZWRcIl0gPSBGYWxzZVxuICAgICAgICBzZWNyZXQgPSBOb25lXG4gICAg
ICAgIHRyeTpcbiAgICAgICAgICAgIGlmIHdvcmtzcGFjZSBpcyBub3QgTm9uZTpcbiAgICAgICAg
ICAgICAgICByZWNvcmRbXCJjbGVhbnVwXCJdW1widmVyaWZpZXJfdGVtcG9yYXJpZXNfcmVtb3Zl
ZFwiXSA9IG5vdCBhbnkod29ya3NwYWNlLml0ZXJkaXIoKSlcbiAgICAgICAgICAgIGlmIGNsZWFu
dXBfZmFpbGVkIG9yIChyZWNvcmRbXCJyZXN1bHRcIl0gPT0gXCJwYXNzZWRcIiBhbmQgbm90IGFs
bChyZWNvcmRbXCJjbGVhbnVwXCJdLnZhbHVlcygpKSk6XG4gICAgICAgICAgICAgICAgcmFpc2Ug
RmFpbHVyZShcImNsZWFudXBfZmFpbGVkXCIpXG4gICAgICAgIGV4Y2VwdCBCYXNlRXhjZXB0aW9u
OlxuICAgICAgICAgICAgcmVjb3JkW1wicmVzdWx0XCJdID0gXCJmYWlsZWRcIlxuICAgICAgICAg
ICAgcmVjb3JkW1wiY2xlYW51cFwiXVtcImZhaWx1cmVfdGFnXCJdID0gXCJjbGVhbnVwX2ZhaWxl
ZFwiXG4gICAgICAgICAgICBpZiByZWNvcmRbXCJmYWlsdXJlX3RhZ1wiXSBpcyBOb25lOlxuICAg
ICAgICAgICAgICAgIHJlY29yZFtcImZhaWx1cmVfdGFnXCJdLCByZWNvcmRbXCJmYWlsdXJlX3N0
YWdlXCJdID0gXCJjbGVhbnVwX2ZhaWxlZFwiLCBcImNsZWFudXBcIlxuICAgICAgICByZWNvcmRb
XCJlbGFwc2VkX3NlY29uZHNcIl0gPSByb3VuZCh0aW1lLm1vbm90b25pYygpIC0gc3RhcnRlZCwg
MylcbiAgICB3cml0dGVuID0gRmFsc2VcbiAgICBpZiBldmlkZW5jZV9mZCBpcyBub3QgTm9uZTpc
biAgICAgICAgdHJ5OlxuICAgICAgICAgICAgd2l0aCBvcy5mZG9wZW4oZXZpZGVuY2VfZmQsIFwi
d2JcIikgYXMgb3V0cHV0OlxuICAgICAgICAgICAgICAgIG91dHB1dC53cml0ZSgoanNvbi5kdW1w
cyhyZWNvcmQsIHNvcnRfa2V5cz1UcnVlKSArIFwiXFxuXCIpLmVuY29kZShcImFzY2lpXCIpKVxu
ICAgICAgICAgICAgICAgIG91dHB1dC5mbHVzaCgpXG4gICAgICAgICAgICAgICAgb3MuZnN5bmMo
b3V0cHV0LmZpbGVubygpKVxuICAgICAgICAgICAgd3JpdHRlbiA9IFRydWVcbiAgICAgICAgZXhj
ZXB0IEV4Y2VwdGlvbjpcbiAgICAgICAgICAgIHJlY29yZFtcInJlc3VsdFwiXSA9IFwiZmFpbGVk
XCJcbiAgICAgICAgICAgIHJlY29yZFtcImV2aWRlbmNlX2ZhaWx1cmVfdGFnXCJdID0gXCJldmlk
ZW5jZV93cml0ZV9mYWlsZWRcIlxuICAgICAgICAgICAgaWYgcmVjb3JkW1wiZmFpbHVyZV90YWdc
Il0gaXMgTm9uZTpcbiAgICAgICAgICAgICAgICByZWNvcmRbXCJmYWlsdXJlX3RhZ1wiXSwgcmVj
b3JkW1wiZmFpbHVyZV9zdGFnZVwiXSA9IFwiZXZpZGVuY2Vfd3JpdGVfZmFpbGVkXCIsIFwiZXZp
ZGVuY2VcIlxuICAgICAgICBmaW5hbGx5OlxuICAgICAgICAgICAgdHJ5OlxuICAgICAgICAgICAg
ICAgIG9zLmNsb3NlKGV2aWRlbmNlX2ZkKVxuICAgICAgICAgICAgZXhjZXB0IE9TRXJyb3I6XG4g
ICAgICAgICAgICAgICAgcGFzc1xuICAgIHRyeTpcbiAgICAgICAgcHJpbnQoanNvbi5kdW1wcyh7
XCJyZXN1bHRcIjogcmVjb3JkW1wicmVzdWx0XCJdLCBcImZhaWx1cmVfdGFnXCI6IHJlY29yZFtc
ImZhaWx1cmVfdGFnXCJdLFxuICAgICAgICAgICAgICAgICAgICAgICAgICBcImNsZWFudXBfZmFp
bHVyZV90YWdcIjogcmVjb3JkW1wiY2xlYW51cFwiXS5nZXQoXCJmYWlsdXJlX3RhZ1wiKSxcbiAg
ICAgICAgICAgICAgICAgICAgICAgICAgXCJldmlkZW5jZV9mYWlsdXJlX3RhZ1wiOiByZWNvcmQu
Z2V0KFwiZXZpZGVuY2VfZmFpbHVyZV90YWdcIiksXG4gICAgICAgICAgICAgICAgICAgICAgICAg
IFwiZXZpZGVuY2Vfd3JpdHRlblwiOiB3cml0dGVufSksIGZsdXNoPVRydWUpXG4gICAgZXhjZXB0
IEV4Y2VwdGlvbjpcbiAgICAgICAgcmV0dXJuIDFcbiAgICByZXR1cm4gMCBpZiByZWNvcmRbXCJy
ZXN1bHRcIl0gPT0gXCJwYXNzZWRcIiBhbmQgd3JpdHRlbiBlbHNlIDFcblxuXG5pZiBfX25hbWVf
XyA9PSBcIl9fbWFpbl9fXCI6XG4gICAgcmFpc2UgU3lzdGVtRXhpdChtYWluKCkpXG4iCkxFR0FD
WV9MT0dJQz0iaW1wb3J0IGFzdCwgY29udGV4dGxpYiwgY29weSwgaGFzaGxpYiwgaHR0cC5zZXJ2
ZXIsIGlvLCBqc29uLCByZSwgc2lnbmFsLCB0aW1lLCB1cmxsaWIucGFyc2VcbmZyb20gdHlwZXMg
aW1wb3J0IFNpbXBsZU5hbWVzcGFjZVxuXG5zdGFydGVkPXRpbWUubW9ub3RvbmljKClcblNPVVJD
RV9TSEE9XCI3ZmJjMjNlNTZkYmM0OTk5Yjk0NjcyZWM0YjI5YjBkMzM1OTZjNTNiNGQ2MzdlYTk5
Zjg1MDQ1MWJkNjNmYmYwXCJcbkJBU0VfQ09NTUlUPVwiNzQ2MWFiNTBmNWE1ZmRmMGFhNWQ3YmQ2
YzRiMzA5NTc1ZTdhNDIzOFwiXG5ncm91cHM9e31cbmNvbXBsZXRlZD0wXG5jYXNlPVwic291cmNl
X3BpblwiXG5oYW5kbGVyX2NhbGxzPXtcImJhc2VsaW5lXCI6MCxcImNhbmRpZGF0ZVwiOjB9XG5l
eHBlY3RlZF93cml0ZV9mYWlsdXJlcz0wXG5leHBlY3RlZF90aW1lb3V0cz0wXG5cbmNsYXNzIFZl
cmlmaWNhdGlvbkRlYWRsaW5lKEJhc2VFeGNlcHRpb24pOlxuICAgIHBhc3NcblxuZGVmIGRlYWRs
aW5lKHNpZ251bSxmcmFtZSk6XG4gICAgcmFpc2UgVmVyaWZpY2F0aW9uRGVhZGxpbmVcblxuZGVm
IHZlcmlmeShuYW1lLGdyb3VwLGJvZHkpOlxuICAgIGdsb2JhbCBjYXNlLGNvbXBsZXRlZFxuICAg
IGNhc2U9bmFtZVxuICAgIGJvZHkoKVxuICAgIGNvbXBsZXRlZCs9MVxuICAgIGdyb3Vwc1tncm91
cF09Z3JvdXBzLmdldChncm91cCwwKSsxXG5cbmNsYXNzIEJ1ZGdldFNpbms6XG4gICAgZGVmIF9f
aW5pdF9fKHNlbGYpOlxuICAgICAgICBzZWxmLmV2ZW50cz1bXVxuICAgICAgICBzZWxmLnBlbmRp
bmdfZGVhZGxpbmU9Tm9uZVxuICAgICAgICBzZWxmLmZhaWxfcmVzcG9uc2U9RmFsc2VcbiAgICBA
Y29udGV4dGxpYi5jb250ZXh0bWFuYWdlclxuICAgIGRlZiBsaW1pdChzZWxmLHNlY29uZHMsdGFn
KTpcbiAgICAgICAgc2VsZi5ldmVudHMuYXBwZW5kKChzZWNvbmRzLHRhZyxcImVudGVyXCIpKVxu
ICAgICAgICBpZiB0YWc9PVwicmVzcG9uc2VfdGltZW91dFwiIGFuZCBzZWxmLmZhaWxfcmVzcG9u
c2U6XG4gICAgICAgICAgICByYWlzZSBzZWxmLmhhbHQoXCJyZXNwb25zZV90aW1lb3V0XCIpXG4g
ICAgICAgIHRyeTpcbiAgICAgICAgICAgIHlpZWxkXG4gICAgICAgIGZpbmFsbHk6XG4gICAgICAg
ICAgICBzZWxmLmV2ZW50cy5hcHBlbmQoKHNlY29uZHMsdGFnLFwiZXhpdFwiKSlcblxuY2xhc3Mg
V3JpdGVGYWlsdXJlKEV4Y2VwdGlvbik6XG4gICAgcGFzc1xuXG5jbGFzcyBXcml0ZVNpbmsoaW8u
Qnl0ZXNJTyk6XG4gICAgZGVmIF9faW5pdF9fKHNlbGYpOlxuICAgICAgICBzdXBlcigpLl9faW5p
dF9fKClcbiAgICAgICAgc2VsZi5mYWlsPUZhbHNlXG4gICAgZGVmIHdyaXRlKHNlbGYsdmFsdWUp
OlxuICAgICAgICBpZiBzZWxmLmZhaWw6XG4gICAgICAgICAgICByYWlzZSBXcml0ZUZhaWx1cmVc
biAgICAgICAgcmV0dXJuIHN1cGVyKCkud3JpdGUodmFsdWUpXG5cbmRlZiBsb2FkX3NlbGVjdGVk
KHRleHQsdmVyc2lvbik6XG4gICAgdHJlZT1hc3QucGFyc2UodGV4dClcbiAgICBjb25zdGFudHM9
W25vZGUgZm9yIG5vZGUgaW4gdHJlZS5ib2R5IGlmIGlzaW5zdGFuY2Uobm9kZSxhc3QuQXNzaWdu
KV1cbiAgICBhc3NlcnQgYWxsKG5vdCBhbnkoaXNpbnN0YW5jZShpdGVtLGFzdC5DYWxsKSBmb3Ig
aXRlbSBpbiBhc3Qud2Fsayhub2RlKSkgZm9yIG5vZGUgaW4gY29uc3RhbnRzKVxuICAgIHNlbGVj
dGVkPWNvbnN0YW50c1s6XVxuICAgIGZvciBub2RlIGluIHRyZWUuYm9keTpcbiAgICAgICAgaWYg
aXNpbnN0YW5jZShub2RlLGFzdC5DbGFzc0RlZikgYW5kIG5vZGUubmFtZSBpbiB7XG4gICAgICAg
ICAgICAgICAgXCJGYWlsdXJlXCIsXCJIYWx0XCIsXCJIZWFkZXJSZWFkZXJcIixcIkhhbmRsZXJc
IixcIlByZWZsb3dBdXRob3JpemF0aW9uUmVmdXNhbFwifTpcbiAgICAgICAgICAgIHNlbGVjdGVk
LmFwcGVuZChub2RlKVxuICAgICAgICBlbGlmIGlzaW5zdGFuY2Uobm9kZSxhc3QuRnVuY3Rpb25E
ZWYpIGFuZCBub2RlLm5hbWU9PVwicmVxdWlyZVwiOlxuICAgICAgICAgICAgc2VsZWN0ZWQuYXBw
ZW5kKG5vZGUpXG4gICAgICAgIGVsaWYgaXNpbnN0YW5jZShub2RlLGFzdC5DbGFzc0RlZikgYW5k
IG5vZGUubmFtZT09XCJEZW1vXCI6XG4gICAgICAgICAgICBub2RlPWNvcHkuZGVlcGNvcHkobm9k
ZSlcbiAgICAgICAgICAgIG5vZGUuYm9keT1bbWV0aG9kIGZvciBtZXRob2QgaW4gbm9kZS5ib2R5
IGlmIGlzaW5zdGFuY2UobWV0aG9kLGFzdC5GdW5jdGlvbkRlZilcbiAgICAgICAgICAgICAgICAg
ICAgICAgYW5kIG1ldGhvZC5uYW1lIGluIHtcIl9faW5pdF9fXCIsXCJjbGVhclwifV1cbiAgICAg
ICAgICAgIHNlbGVjdGVkLmFwcGVuZChub2RlKVxuICAgIG5zPXtcImh0dHBcIjpodHRwLFwicmVc
IjpyZSxcInVybGxpYlwiOnVybGxpYn1cbiAgICBleGVjKGNvbXBpbGUoYXN0LmZpeF9taXNzaW5n
X2xvY2F0aW9ucyhhc3QuTW9kdWxlKGJvZHk9c2VsZWN0ZWQsdHlwZV9pZ25vcmVzPVtdKSksXG4g
ICAgICAgICAgICAgICAgIFwiPHNlbGVjdGVkLXJlcXVlc3QtbWV0aG9kcy1cIit2ZXJzaW9uK1wi
PlwiLFwiZXhlY1wiKSxucylcbiAgICBtYWluPW5leHQobm9kZSBmb3Igbm9kZSBpbiB0cmVlLmJv
ZHkgaWYgaXNpbnN0YW5jZShub2RlLGFzdC5GdW5jdGlvbkRlZikgYW5kIG5vZGUubmFtZT09XCJt
YWluXCIpXG4gICAgbnNbXCJyZWNvcmRfbm9kZVwiXT1jb3B5LmRlZXBjb3B5KG5leHQobm9kZSBm
b3Igbm9kZSBpbiBtYWluLmJvZHkgaWYgaXNpbnN0YW5jZShub2RlLGFzdC5Bc3NpZ24pXG4gICAg
ICAgIGFuZCBhbnkoaXNpbnN0YW5jZSh0LGFzdC5OYW1lKSBhbmQgdC5pZD09XCJyZWNvcmRcIiBm
b3IgdCBpbiBub2RlLnRhcmdldHMpKSlcbiAgICBtYWluX3RyeT1uZXh0KG5vZGUgZm9yIG5vZGUg
aW4gbWFpbi5ib2R5IGlmIGlzaW5zdGFuY2Uobm9kZSxhc3QuVHJ5KSlcbiAgICBkZW1vX2ZpbmFs
PW5leHQobm9kZSBmb3Igbm9kZSBpbiBtYWluX3RyeS5maW5hbGJvZHkgaWYgaXNpbnN0YW5jZShu
b2RlLGFzdC5JZilcbiAgICAgICAgYW5kIGlzaW5zdGFuY2Uobm9kZS50ZXN0LGFzdC5Db21wYXJl
KSBhbmQgaXNpbnN0YW5jZShub2RlLnRlc3QubGVmdCxhc3QuTmFtZSlcbiAgICAgICAgYW5kIG5v
ZGUudGVzdC5sZWZ0LmlkPT1cImRlbW9cIilcbiAgICBuc1tcImNvcHlfbm9kZXNcIl09W2NvcHku
ZGVlcGNvcHkobm9kZSkgZm9yIG5vZGUgaW4gZGVtb19maW5hbC5ib2R5XG4gICAgICAgICAgICAg
ICAgICAgICBpZiBpc2luc3RhbmNlKG5vZGUsYXN0LkFzc2lnbikgYW5kIGlzaW5zdGFuY2Uobm9k
ZS50YXJnZXRzWzBdLGFzdC5TdWJzY3JpcHQpXG4gICAgICAgICAgICAgICAgICAgICBhbmQgaXNp
bnN0YW5jZShub2RlLnRhcmdldHNbMF0udmFsdWUsYXN0Lk5hbWUpXG4gICAgICAgICAgICAgICAg
ICAgICBhbmQgbm9kZS50YXJnZXRzWzBdLnZhbHVlLmlkPT1cInJlY29yZFwiXVxuICAgIGxvb3Bf
aW5kZXg9bmV4dChpIGZvciBpLG5vZGUgaW4gZW51bWVyYXRlKG1haW5fdHJ5LmJvZHkpIGlmIGlz
aW5zdGFuY2Uobm9kZSxhc3QuV2hpbGUpKVxuICAgIG5zW1wiZ2F0ZV9ub2Rlc1wiXT1jb3B5LmRl
ZXBjb3B5KG1haW5fdHJ5LmJvZHlbbG9vcF9pbmRleCsxOmxvb3BfaW5kZXgrNF0pXG4gICAgc2Vy
dmVyPW5leHQobm9kZSBmb3Igbm9kZSBpbiB0cmVlLmJvZHkgaWYgaXNpbnN0YW5jZShub2RlLGFz
dC5DbGFzc0RlZikgYW5kIG5vZGUubmFtZT09XCJEZW1vU2VydmVyXCIpXG4gICAgZXJyb3I9Y29w
eS5kZWVwY29weShuZXh0KG5vZGUgZm9yIG5vZGUgaW4gc2VydmVyLmJvZHkgaWYgaXNpbnN0YW5j
ZShub2RlLGFzdC5GdW5jdGlvbkRlZilcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICBhbmQg
bm9kZS5uYW1lPT1cImhhbmRsZV9lcnJvclwiKSlcbiAgICBleGVjKGNvbXBpbGUoYXN0Lk1vZHVs
ZShib2R5PVtlcnJvcl0sdHlwZV9pZ25vcmVzPVtdKSxcIjxzZXJ2ZXItZXJyb3ItbWV0aG9kPlwi
LFwiZXhlY1wiKSxucylcbiAgICByZXR1cm4gbnNcblxuZGVmIG1ha2VfZGVtbyhucyk6XG4gICAg
bmFtZXM9KFwiY3JlZGVudGlhbF9wcml2YXRlX3ZhbGlkYXRlZFwiLFwicHJvdmlkZXJfaWRlbnRp
dHlfdmVyaWZpZWRcIixcImRpc2NvdmVyeV92ZXJpZmllZFwiLFxuICAgICAgICBcInByb3RlY3Rl
ZF93aXRob3V0X2Nvb2tpZV9kZW5pZWRcIixcImF1dGhvcml6YXRpb25fcmVkaXJlY3RfaXNzdWVk
XCIsXG4gICAgICAgIFwic3RhdGVfaXNzdWVyX2Zsb3dfY29va2llX3ZlcmlmaWVkXCIsXCJjb25m
aWRlbnRpYWxfczI1Nl9leGNoYW5nZV92ZXJpZmllZFwiLFxuICAgICAgICBcInJzMjU2X2p3a3Nf
aXNzdWVyX2F1ZGllbmNlX25vbmNlX3RpbWVfYWNjZXNzX2hhc2hfdmVyaWZpZWRcIixcbiAgICAg
ICAgXCJ1c2VyaW5mb19zdWJqZWN0X3ZlcmlmaWVkXCIsXCJwcm90ZWN0ZWRfd2l0aF9mcmVzaF9j
b29raWVfYWNjZXB0ZWRcIilcbiAgICBjbGFzcyBEZW1vU2luayhuc1tcIkRlbW9cIl0pOlxuICAg
ICAgICBkZWYgX19nZXRhdHRyaWJ1dGVfXyhzZWxmLG5hbWUpOlxuICAgICAgICAgICAgaWYgbmFt
ZT09XCJzZWNyZXRcIjpcbiAgICAgICAgICAgICAgICBhY2Nlc3M9b2JqZWN0Ll9fZ2V0YXR0cmli
dXRlX18oc2VsZixcImNhbGxzXCIpXG4gICAgICAgICAgICAgICAgYWNjZXNzW1wiY3JlZGVudGlh
bFwiXSs9MVxuICAgICAgICAgICAgcmV0dXJuIG9iamVjdC5fX2dldGF0dHJpYnV0ZV9fKHNlbGYs
bmFtZSlcbiAgICAgICAgZGVmIGJlZ2luKHNlbGYpOlxuICAgICAgICAgICAgc2VsZi5jYWxsc1tc
ImJlZ2luXCJdKz0xXG4gICAgICAgICAgICBuc1tcInJlcXVpcmVcIl0obm90IHNlbGYuYXR0ZW1w
dGVkLFwiZmxvd19hbHJlYWR5X3N0YXJ0ZWRcIilcbiAgICAgICAgICAgIHNlbGYuc3RhZ2U9XCJm
bG93XCJcbiAgICAgICAgICAgIHNlbGYuYXR0ZW1wdGVkPVRydWVcbiAgICAgICAgICAgIHNlbGYu
cGVuZGluZz17fVxuICAgICAgICAgICAgcmV0dXJuIG5zW1wiSVNTVUVSXCJdK1wiL2F1dGhvcml6
ZVwiLFwiYVwiKjQzXG4gICAgICAgIGRlZiBjYWxsYmFjayhzZWxmLHF1ZXJ5LGNvb2tpZSk6XG4g
ICAgICAgICAgICBzZWxmLmNhbGxzW1wiY2FsbGJhY2tcIl0rPTFcbiAgICAgICAgICAgIG5zW1wi
cmVxdWlyZVwiXShzZWxmLnBlbmRpbmcgaXMgbm90IE5vbmUsXCJjYWxsYmFja19hbHJlYWR5X2Nv
bnN1bWVkXCIpXG4gICAgICAgICAgICBzZWxmLnBlbmRpbmc9Tm9uZVxuICAgICAgICAgICAgc2Vs
Zi5jb29raWU9XCJiXCIqNDNcbiAgICAgICAgICAgIHNlbGYuc3ViamVjdD1cInN5bnRoZXRpY1wi
XG4gICAgICAgICAgICByZXR1cm4gc2VsZi5jb29raWVcbiAgICBidWRnZXQ9QnVkZ2V0U2luaygp
XG4gICAgYnVkZ2V0LmhhbHQ9bnNbXCJIYWx0XCJdXG4gICAgZGVtbz1EZW1vU2luayhTaW1wbGVO
YW1lc3BhY2UoZXF1YWw9bGFtYmRhIGxlZnQscmlnaHQ6bGVmdD09cmlnaHQpLE5vbmUsb2JqZWN0
KCksYnVkZ2V0LFxuICAgICAgICAgICAgICAgICAge25hbWU6aTwzIGZvciBpLG5hbWUgaW4gZW51
bWVyYXRlKG5hbWVzKX0sXG4gICAgICAgICAgICAgICAgICB7bmFtZTpOb25lIGZvciBuYW1lIGlu
IChcImF1dGhvcml6YXRpb25fcmVkaXJlY3RcIixcImNhbGxiYWNrXCIsXCJ0b2tlbl9leGNoYW5n
ZVwiLFxuICAgICAgICAgICAgICAgICAgIFwidXNlcmluZm9cIixcInByb3RlY3RlZF9iZWZvcmVc
IixcInByb3RlY3RlZF9hZnRlclwiKX0pXG4gICAgZGVtby5jYWxscz17XCJnZXRcIjowLFwiY29v
a2llc1wiOjAsXCJiZWdpblwiOjAsXCJjYWxsYmFja1wiOjAsXCJjcmVkZW50aWFsXCI6MH1cbiAg
ICByZXR1cm4gZGVtb1xuXG5kZWYgZmxvd19zbmFwc2hvdChkZW1vKTpcbiAgICByZXR1cm4gKGRl
bW8uYXR0ZW1wdGVkLGRlbW8ucGVuZGluZyxkZW1vLmNvb2tpZSxkZW1vLnN1YmplY3QsXG4gICAg
ICAgICAgICBkZW1vLmZhaWx1cmUsZGVtby5kb25lLGNvcHkuZGVlcGNvcHkoZGVtby5jaGVja3Mp
LGNvcHkuZGVlcGNvcHkoZGVtby5zdGF0dXNlcykpXG5cbmRlZiBidWlsZF93aXJlKG5zLG1ldGhv
ZD1cIkdFVFwiLHRhcmdldD1cIi9cIixoZWFkZXJzPSgpLGhvc3Q9XCJub3JtYWxcIik6XG4gICAg
cHJlZml4PVtdIGlmIGhvc3Q9PVwiYWJzZW50XCIgZWxzZSBbKFwiSG9zdFwiLG5zW1wiQVVUSE9S
SVRZXCJdIGlmIGhvc3Q9PVwibm9ybWFsXCIgZWxzZSBcImludmFsaWRcIildXG4gICAgcmV0dXJu
ICgobWV0aG9kK1wiIFwiK3RhcmdldCtcIiBIVFRQLzEuMVxcclxcblwiXG4gICAgICAgICAgICAr
XCJcIi5qb2luKG5hbWUrXCI6IFwiK3ZhbHVlK1wiXFxyXFxuXCIgZm9yIG5hbWUsdmFsdWUgaW4g
cHJlZml4K2xpc3QoaGVhZGVycykpXG4gICAgICAgICAgICArXCJcXHJcXG5cIikuZW5jb2RlKFwi
YXNjaWlcIikpXG5cbmRlZiByZXF1ZXN0KG5zLGRlbW8sdmVyc2lvbix3aXJlLCosd3JpdGVfZmFp
bD1GYWxzZSx0aW1lb3V0PUZhbHNlLHBhcnNlcl9wYXRoPU5vbmUpOlxuICAgIGhhbmRsZXJfY2Fs
bHNbdmVyc2lvbl0rPTFcbiAgICBjbGFzcyBIYW5kbGVyU2luayhuc1tcIkhhbmRsZXJcIl0pOlxu
ICAgICAgICBkZWYgcGFyc2VfcmVxdWVzdChzZWxmKTpcbiAgICAgICAgICAgIHBhcnNlZD1zdXBl
cigpLnBhcnNlX3JlcXVlc3QoKVxuICAgICAgICAgICAgaWYgcGFyc2VkIGFuZCBwYXJzZXJfcGF0
aCBpcyBub3QgTm9uZTpcbiAgICAgICAgICAgICAgICBzZWxmLnBhdGg9cGFyc2VyX3BhdGhcbiAg
ICAgICAgICAgIHJldHVybiBwYXJzZWRcbiAgICAgICAgZGVmIGdldChzZWxmLHRhcmdldCk6XG4g
ICAgICAgICAgICBkZW1vLmNhbGxzW1wiZ2V0XCJdKz0xXG4gICAgICAgICAgICByZXR1cm4gc3Vw
ZXIoKS5nZXQodGFyZ2V0KVxuICAgICAgICBkZWYgY29va2llcyhzZWxmKTpcbiAgICAgICAgICAg
IGRlbW8uY2FsbHNbXCJjb29raWVzXCJdKz0xXG4gICAgICAgICAgICByZXR1cm4gc3VwZXIoKS5j
b29raWVzKClcbiAgICAgICAgZGVmIHJlcGx5KHNlbGYsc3RhdHVzLHRleHQsKipvcHRpb25zKTpc
biAgICAgICAgICAgIHNlbGYucmVzcG9uc2VzLmFwcGVuZCgoc3RhdHVzLHR1cGxlKHNvcnRlZChv
cHRpb25zKSkpKVxuICAgICAgICAgICAgcmV0dXJuIHN1cGVyKCkucmVwbHkoc3RhdHVzLHRleHQs
KipvcHRpb25zKVxuICAgICAgICBkZWYgc2VuZF9yZXNwb25zZShzZWxmLHN0YXR1cyxtZXNzYWdl
PU5vbmUpOlxuICAgICAgICAgICAgc2VsZi5zZW50X3N0YXR1c2VzLmFwcGVuZChzdGF0dXMpXG4g
ICAgICAgIGRlZiBzZW5kX2hlYWRlcihzZWxmLG5hbWUsdmFsdWUpOlxuICAgICAgICAgICAgc2Vs
Zi5zZW50X2hlYWRlcnMuYXBwZW5kKChuYW1lLHZhbHVlKSlcbiAgICAgICAgZGVmIGVuZF9oZWFk
ZXJzKHNlbGYpOlxuICAgICAgICAgICAgc2VsZi5oZWFkZXJfZW5kcys9MVxuICAgIGhhbmRsZXI9
b2JqZWN0Ll9fbmV3X18oSGFuZGxlclNpbmspXG4gICAgaGFuZGxlci5zZXJ2ZXI9U2ltcGxlTmFt
ZXNwYWNlKGRlbW89ZGVtbylcbiAgICBoYW5kbGVyLnJmaWxlPWlvLkJ5dGVzSU8od2lyZSlcbiAg
ICBoYW5kbGVyLndmaWxlPVdyaXRlU2luaygpXG4gICAgaGFuZGxlci53ZmlsZS5mYWlsPXdyaXRl
X2ZhaWxcbiAgICBoYW5kbGVyLnJlc3BvbnNlcz1bXVxuICAgIGhhbmRsZXIuc2VudF9zdGF0dXNl
cz1bXVxuICAgIGhhbmRsZXIuc2VudF9oZWFkZXJzPVtdXG4gICAgaGFuZGxlci5oZWFkZXJfZW5k
cz0wXG4gICAgZGVtby5idWRnZXQuZmFpbF9yZXNwb25zZT10aW1lb3V0XG4gICAgaGFuZGxlci5o
YW5kbGVfb25lX3JlcXVlc3QoKVxuICAgIHJldHVybiBoYW5kbGVyXG5cbmRlZiBub3JtYWxpemVk
KG5zLGRlbW8saGFuZGxlcik6XG4gICAgcmV0dXJuIChmbG93X3NuYXBzaG90KGRlbW8pLGRlbW8u
c3RhZ2UsZGVtby5yZXF1ZXN0X2ludmFsaWRfcmVhc29uLGRlbW8uY2FsbHMsXG4gICAgICAgICAg
ICBoYW5kbGVyLnJlc3BvbnNlcyxoYW5kbGVyLnNlbnRfc3RhdHVzZXMsaGFuZGxlci5zZW50X2hl
YWRlcnMsaGFuZGxlci53ZmlsZS5nZXR2YWx1ZSgpLFxuICAgICAgICAgICAgZGVtby5idWRnZXQu
ZXZlbnRzKVxuXG5kZWYgcmVjb3JkX2NvcHkobnMsZGVtbyk6XG4gICAgZW52PWRpY3QobnMpXG4g
ICAgZW52W1wiZGVtb1wiXT1kZW1vXG4gICAgZXhlYyhjb21waWxlKGFzdC5Nb2R1bGUoYm9keT1b
bnNbXCJyZWNvcmRfbm9kZVwiXV0sdHlwZV9pZ25vcmVzPVtdKSxcIjxyZWNvcmQtaW5pdGlhbGl6
YXRpb24+XCIsXCJleGVjXCIpLGVudilcbiAgICBleGVjKGNvbXBpbGUoYXN0Lk1vZHVsZShib2R5
PW5zW1wiY29weV9ub2Rlc1wiXSx0eXBlX2lnbm9yZXM9W10pLFwiPHJlY29yZC1maW5hbC1jb3B5
PlwiLFwiZXhlY1wiKSxlbnYpXG4gICAgcmV0dXJuIGVudltcInJlY29yZFwiXVxuXG5kZWYgZ2F0
ZV9mYWlscyhucyxkZW1vKTpcbiAgICBlbnY9ZGljdChucylcbiAgICBlbnZbXCJkZW1vXCJdPWRl
bW9cbiAgICBlbnZbXCJyZWNvcmRcIl09e1wiY2hlY2tzXCI6ZGVtby5jaGVja3MsXCJyZXN1bHRc
IjpcImZhaWxlZFwifVxuICAgIHRyeTpcbiAgICAgICAgZXhlYyhjb21waWxlKGFzdC5Nb2R1bGUo
Ym9keT1uc1tcImdhdGVfbm9kZXNcIl0sdHlwZV9pZ25vcmVzPVtdKSxcIjxleGlzdGluZy1vdXRj
b21lLWdhdGU+XCIsXCJleGVjXCIpLGVudilcbiAgICBleGNlcHQgbnNbXCJGYWlsdXJlXCJdIGFz
IGZhaWx1cmU6XG4gICAgICAgIGFzc2VydCBmYWlsdXJlLnRhZyBpbiB7XCJyZXF1ZXN0X2ludmFs
aWRcIixcInVuZXhwZWN0ZWRfZmFpbHVyZVwifVxuICAgIGVsc2U6XG4gICAgICAgIHJhaXNlIEFz
c2VydGlvbkVycm9yXG4gICAgYXNzZXJ0IGVudltcInJlY29yZFwiXVtcInJlc3VsdFwiXT09XCJm
YWlsZWRcIlxuXG5kZWYgcmVmdXNlZChucyxkZW1vLGhhbmRsZXIsYmVmb3JlLGNvdW50LHRlcm1p
bmFsKTpcbiAgICBhc3NlcnQgaGFuZGxlci5yZXNwb25zZXM9PVsoNDAzLCgpKV0gYW5kIGhhbmRs
ZXIuc2VudF9zdGF0dXNlcz09WzQwM11cbiAgICBhc3NlcnQgaGFuZGxlci5oZWFkZXJfZW5kcz09
MVxuICAgIGFzc2VydCBhbGwobmFtZSBub3QgaW4ge1wiTG9jYXRpb25cIixcIlNldC1Db29raWVc
In0gZm9yIG5hbWUsdmFsdWUgaW4gaGFuZGxlci5zZW50X2hlYWRlcnMpXG4gICAgYXNzZXJ0IGJc
Ijxmb3JtXCIgbm90IGluIGhhbmRsZXIud2ZpbGUuZ2V0dmFsdWUoKVxuICAgIGFzc2VydCBkZW1v
LmNhbGxzPT17XCJnZXRcIjowLFwiY29va2llc1wiOjAsXCJiZWdpblwiOjAsXCJjYWxsYmFja1wi
OjAsXCJjcmVkZW50aWFsXCI6MH1cbiAgICBhZnRlcj1mbG93X3NuYXBzaG90KGRlbW8pXG4gICAg
YXNzZXJ0IGFmdGVyWzo0XT09YmVmb3JlWzo0XSBhbmQgYWZ0ZXJbNjpdPT1iZWZvcmVbNjpdXG4g
ICAgYXNzZXJ0IGRlbW8ucHJlZmxvd19hdXRob3JpemF0aW9uX3JlZnVzYWxzPT1jb3VudFxuICAg
IGFzc2VydCBkZW1vLnJlcXVlc3RfaW52YWxpZF9yZWFzb249PVwiYXV0aG9yaXphdGlvblwiXG4g
ICAgYXNzZXJ0IGRlbW8uZmFpbHVyZT09KFwicmVxdWVzdF9pbnZhbGlkXCIgaWYgdGVybWluYWwg
ZWxzZSBiZWZvcmVbNF0pXG4gICAgYXNzZXJ0IGRlbW8uZG9uZT09KFRydWUgaWYgdGVybWluYWwg
ZWxzZSBiZWZvcmVbNV0pXG4gICAgYXNzZXJ0IGFsbChub3QgdmFsdWUgZm9yIGtleSx2YWx1ZSBp
biBkZW1vLmNoZWNrcy5pdGVtcygpIGlmIGtleSBub3QgaW5cbiAgICAgICAgICAgICAgIHtcImNy
ZWRlbnRpYWxfcHJpdmF0ZV92YWxpZGF0ZWRcIixcInByb3ZpZGVyX2lkZW50aXR5X3ZlcmlmaWVk
XCIsXCJkaXNjb3ZlcnlfdmVyaWZpZWRcIn0pXG4gICAgYXNzZXJ0IGFsbCh2YWx1ZSBpcyBOb25l
IGZvciB2YWx1ZSBpbiBkZW1vLnN0YXR1c2VzLnZhbHVlcygpKVxuICAgIGNvcGllZD1yZWNvcmRf
Y29weShucyxkZW1vKVxuICAgIGFzc2VydCBjb3BpZWRbXCJwcmVmbG93X2F1dGhvcml6YXRpb25f
cmVmdXNhbHNcIl09PWNvdW50XG4gICAgYXNzZXJ0IGNvcGllZFtcInJlcXVlc3RfaW52YWxpZF9y
ZWFzb25cIl09PVwiYXV0aG9yaXphdGlvblwiXG4gICAgZ2F0ZV9mYWlscyhucyxkZW1vKVxuXG5k
ZWYgcnVuKCk6XG4gICAgZ2xvYmFsIGV4cGVjdGVkX3dyaXRlX2ZhaWx1cmVzLGV4cGVjdGVkX3Rp
bWVvdXRzXG4gICAgdGV4dD1TT1VSQ0VfVEVYVFxuICAgIGFzc2VydCBoYXNobGliLnNoYTI1Nih0
ZXh0LmVuY29kZSgpKS5oZXhkaWdlc3QoKT09U09VUkNFX1NIQVxuICAgIGJhc2VsaW5lX3RleHQ9
QkFTRUxJTkVfVEVYVFxuICAgIGFzc2VydCBoYXNobGliLnNoYTI1NihiYXNlbGluZV90ZXh0LmVu
Y29kZSgpKS5oZXhkaWdlc3QoKT09XCJhYzNjMzUwZDBlYTVmMWM3NDQ4ZDUzMzUyNGU1ZGIyZjk5
ZWUzN2FkNjFiZTMxM2M2ZjM2ZTBjNDAyNWEyZmNkXCJcbiAgICBvbGQ9bG9hZF9zZWxlY3RlZChi
YXNlbGluZV90ZXh0LFwiYmFzZWxpbmVcIilcbiAgICBuZXc9bG9hZF9zZWxlY3RlZCh0ZXh0LFwi
Y2FuZGlkYXRlXCIpXG4gICAgZGVtbz1tYWtlX2RlbW8obmV3KVxuICAgIGFzc2VydCByZWNvcmRf
Y29weShuZXcsZGVtbylbXCJwcmVmbG93X2F1dGhvcml6YXRpb25fcmVmdXNhbHNcIl09PTBcbiAg
ICB2ZXJpZnkoXCJpbml0aWFsX2NvdW50X2NvcHlcIixcInJlY29yZF9jb3B5XCIsbGFtYmRhOmdh
dGVfZmFpbHMobmV3LGRlbW8pKVxuICAgIGZvciBjb3VudCBpbiByYW5nZSgxLDUpOlxuICAgICAg
ICBkZWYgY2hlY2soY291bnQ9Y291bnQpOlxuICAgICAgICAgICAgYmVmb3JlPWZsb3dfc25hcHNo
b3QoZGVtbylcbiAgICAgICAgICAgIGhhbmRsZXI9cmVxdWVzdChuZXcsZGVtbyxcImNhbmRpZGF0
ZVwiLGJ1aWxkX3dpcmUobmV3LGhlYWRlcnM9WyhcIkF1dGhvcml6YXRpb25cIixcImlnbm9yZWRc
IildKSlcbiAgICAgICAgICAgIHJlZnVzZWQobmV3LGRlbW8saGFuZGxlcixiZWZvcmUsY291bnQs
Y291bnQ9PTQpXG4gICAgICAgIHZlcmlmeShcImJvdW5kZWRfcmVmdXNhbF9cIitzdHIoY291bnQp
LFwiYm91bmRlZF9zZXF1ZW5jZVwiLGNoZWNrKVxuICAgIGZvciBsYWJlbCxoZWFkZXJzLG1ldGhv
ZCx0YXJnZXQgaW4gW1xuICAgICAgICAoXCJlbXB0eVwiLFsoXCJBdXRob3JpemF0aW9uXCIsXCJc
IildLFwiR0VUXCIsXCIvXCIpLFxuICAgICAgICAoXCJkdXBsaWNhdGVcIixbKFwiQXV0aG9yaXph
dGlvblwiLFwiaWdub3JlZFwiKSwoXCJBdXRob3JpemF0aW9uXCIsXCJcIildLFwiR0VUXCIsXCIv
XCIpLFxuICAgICAgICAoXCJwb3N0XCIsWyhcIkF1dGhvcml6YXRpb25cIixcImlnbm9yZWRcIild
LFwiUE9TVFwiLFwiL2xvZ2luXCIpLFxuICAgICAgICAoXCJwcm90ZWN0ZWRcIixbKFwiQXV0aG9y
aXphdGlvblwiLFwiaWdub3JlZFwiKSwoXCJDb29raWVcIixcImQwMV9sb2NhbF9kZW1vPVwiK1wi
YlwiKjQzKV0sXCJHRVRcIixcIi9wcm90ZWN0ZWRcIiksXG4gICAgICAgIChcImNhbGxiYWNrXCIs
WyhcIkF1dGhvcml6YXRpb25cIixcImlnbm9yZWRcIildLFwiR0VUXCIsXCIvY2FsbGJhY2tcIiks
XG4gICAgICAgIChcIm90aGVyX21ldGhvZFwiLFsoXCJBdXRob3JpemF0aW9uXCIsXCJpZ25vcmVk
XCIpXSxcIlBVVFwiLFwiL1wiKV06XG4gICAgICAgIGRlZiBjaGVjayhoZWFkZXJzPWhlYWRlcnMs
bWV0aG9kPW1ldGhvZCx0YXJnZXQ9dGFyZ2V0KTpcbiAgICAgICAgICAgIGQ9bWFrZV9kZW1vKG5l
dyk7YmVmb3JlPWZsb3dfc25hcHNob3QoZClcbiAgICAgICAgICAgIGg9cmVxdWVzdChuZXcsZCxc
ImNhbmRpZGF0ZVwiLGJ1aWxkX3dpcmUobmV3LG1ldGhvZCx0YXJnZXQsaGVhZGVycykpXG4gICAg
ICAgICAgICByZWZ1c2VkKG5ldyxkLGgsYmVmb3JlLDEsRmFsc2UpXG4gICAgICAgICAgICBkLmNs
ZWFyKClcbiAgICAgICAgICAgIGFzc2VydCBkLnByZWZsb3dfYXV0aG9yaXphdGlvbl9yZWZ1c2Fs
cz09MSBhbmQgZC5yZXF1ZXN0X2ludmFsaWRfcmVhc29uPT1cImF1dGhvcml6YXRpb25cIlxuICAg
ICAgICB2ZXJpZnkoXCJwcmVzZW5jZV9cIitsYWJlbCxcImF1dGhvcml6YXRpb25fcHJlc2VuY2Vc
IixjaGVjaylcbiAgICBmb3IgZmllbGQsdmFsdWUgaW4gWyhcImF0dGVtcHRlZFwiLFRydWUpLChc
InBlbmRpbmdcIix7fSksKFwiY29va2llXCIsXCJiXCIqNDMpLFxuICAgICAgICAgICAgICAgICAg
ICAgICAgKFwic3ViamVjdFwiLFwic3ludGhldGljXCIpLChcImF0dGVtcHRlZFwiLDApLChcImF0
dGVtcHRlZFwiLE5vbmUpXTpcbiAgICAgICAgZGVmIGNoZWNrKGZpZWxkPWZpZWxkLHZhbHVlPXZh
bHVlKTpcbiAgICAgICAgICAgIGQ9bWFrZV9kZW1vKG5ldyk7c2V0YXR0cihkLGZpZWxkLHZhbHVl
KTtiZWZvcmU9Zmxvd19zbmFwc2hvdChkKVxuICAgICAgICAgICAgaD1yZXF1ZXN0KG5ldyxkLFwi
Y2FuZGlkYXRlXCIsYnVpbGRfd2lyZShuZXcsaGVhZGVycz1bKFwiQXV0aG9yaXphdGlvblwiLFwi
aWdub3JlZFwiKV0pKVxuICAgICAgICAgICAgYWZ0ZXI9Zmxvd19zbmFwc2hvdChkKVxuICAgICAg
ICAgICAgYXNzZXJ0IGgucmVzcG9uc2VzPT1bKDQwMCwoKSldIGFuZCBkLmZhaWx1cmU9PVwicmVx
dWVzdF9pbnZhbGlkXCIgYW5kIGQuZG9uZVxuICAgICAgICAgICAgYXNzZXJ0IGQucHJlZmxvd19h
dXRob3JpemF0aW9uX3JlZnVzYWxzPT0wIGFuZCBkLnJlcXVlc3RfaW52YWxpZF9yZWFzb249PVwi
YXV0aG9yaXphdGlvblwiXG4gICAgICAgICAgICBhc3NlcnQgYWZ0ZXJbOjRdPT1iZWZvcmVbOjRd
IGFuZCBhZnRlcls2Ol09PWJlZm9yZVs2Ol1cbiAgICAgICAgICAgIGFzc2VydCBhbGwodj09MCBm
b3IgdiBpbiBkLmNhbGxzLnZhbHVlcygpKVxuICAgICAgICB2ZXJpZnkoXCJzdHJpY3Rfc3RhdGVf
XCIrZmllbGQrXCJfXCIrc3RyKGdyb3Vwcy5nZXQoXCJwb3N0Zmxvd1wiLDApKzEpLFwicG9zdGZs
b3dcIixjaGVjaylcbiAgICBkZWYgZXhoYXVzdGVkKCk6XG4gICAgICAgIGQ9bWFrZV9kZW1vKG5l
dyk7ZC5wcmVmbG93X2F1dGhvcml6YXRpb25fcmVmdXNhbHM9NFxuICAgICAgICBoPXJlcXVlc3Qo
bmV3LGQsXCJjYW5kaWRhdGVcIixidWlsZF93aXJlKG5ldyxoZWFkZXJzPVsoXCJBdXRob3JpemF0
aW9uXCIsXCJpZ25vcmVkXCIpXSkpXG4gICAgICAgIGFzc2VydCBoLnJlc3BvbnNlcz09Wyg0MDAs
KCkpXSBhbmQgZC5kb25lIGFuZCBkLmZhaWx1cmU9PVwicmVxdWVzdF9pbnZhbGlkXCJcbiAgICAg
ICAgYXNzZXJ0IGQucHJlZmxvd19hdXRob3JpemF0aW9uX3JlZnVzYWxzPT00IGFuZCBhbGwodj09
MCBmb3IgdiBpbiBkLmNhbGxzLnZhbHVlcygpKVxuICAgIHZlcmlmeShcImRlZmVuc2l2ZV9leGhh
dXN0aW9uXCIsXCJwb3N0Zmxvd1wiLGV4aGF1c3RlZClcbiAgICBsb2dpbj1bKFwiT3JpZ2luXCIs
bmV3W1wiT1JJR0lOXCJdKSwoXCJDb250ZW50LVR5cGVcIixcImFwcGxpY2F0aW9uL3gtd3d3LWZv
cm0tdXJsZW5jb2RlZFwiKSwoXCJDb250ZW50LUxlbmd0aFwiLFwiMFwiKV1cbiAgICBzcGVjaWZp
Y2F0aW9ucz1bXG4gICAgICAgIChcInJvb3RcIixcIkdFVFwiLFwiL1wiLFtdLE5vbmUpLChcInBy
b3RlY3RlZF9iZWZvcmVcIixcIkdFVFwiLFwiL3Byb3RlY3RlZFwiLFtdLE5vbmUpLFxuICAgICAg
ICAoXCJ1bmF2YWlsYWJsZVwiLFwiR0VUXCIsXCIvdW5hdmFpbGFibGVcIixbXSxOb25lKSwoXCJy
b290X3F1ZXJ5XCIsXCJHRVRcIixcIi8/eFwiLFtdLE5vbmUpLFxuICAgICAgICAoXCJwcm90ZWN0
ZWRfcXVlcnlcIixcIkdFVFwiLFwiL3Byb3RlY3RlZD94XCIsW10sTm9uZSksXG4gICAgICAgIChc
ImNhbGxiYWNrX2VtcHR5XCIsXCJHRVRcIixcIi9jYWxsYmFja1wiLFtdLE5vbmUpLFxuICAgICAg
ICAoXCJjYWxsYmFja19yZWFkeVwiLFwiR0VUXCIsXCIvY2FsbGJhY2tcIixbXSx7XCJwZW5kaW5n
XCI6e319KSxcbiAgICAgICAgKFwibG9naW5cIixcIlBPU1RcIixcIi9sb2dpblwiLGxvZ2luLE5v
bmUpLFxuICAgICAgICAoXCJsb2dpbl9yZXBlYXRlZFwiLFwiUE9TVFwiLFwiL2xvZ2luXCIsbG9n
aW4se1wiYXR0ZW1wdGVkXCI6VHJ1ZX0pLFxuICAgICAgICAoXCJwcm90ZWN0ZWRfYWZ0ZXJcIixc
IkdFVFwiLFwiL3Byb3RlY3RlZFwiLFsoXCJDb29raWVcIixcImQwMV9sb2NhbF9kZW1vPVwiK1wi
YlwiKjQzKV0sXG4gICAgICAgICAgICB7XCJjb29raWVcIjpcImJcIio0MyxcInN1YmplY3RcIjpc
InN5bnRoZXRpY1wiLFwicHJvdGVjdGVkX2JlZm9yZVwiOlRydWV9KSxcbiAgICAgICAgKFwicHJv
dGVjdGVkX2JlZm9yZV9taXNzaW5nXCIsXCJHRVRcIixcIi9wcm90ZWN0ZWRcIixbKFwiQ29va2ll
XCIsXCJkMDFfbG9jYWxfZGVtbz1cIitcImJcIio0MyldLFxuICAgICAgICAgICAge1wiY29va2ll
XCI6XCJiXCIqNDMsXCJzdWJqZWN0XCI6XCJzeW50aGV0aWNcIn0pLFxuICAgICAgICAoXCJwcm90
ZWN0ZWRfY29va2llX21pc21hdGNoXCIsXCJHRVRcIixcIi9wcm90ZWN0ZWRcIixbKFwiQ29va2ll
XCIsXCJkMDFfbG9jYWxfZGVtbz1cIitcImNcIio0MyldLFxuICAgICAgICAgICAge1wiY29va2ll
XCI6XCJiXCIqNDMsXCJzdWJqZWN0XCI6XCJzeW50aGV0aWNcIn0pLFxuICAgICAgICAoXCJtZXRo
b2RcIixcIlBVVFwiLFwiL1wiLFtdLE5vbmUpLFxuICAgICAgICAoXCJwb3N0X3RhcmdldFwiLFwi
UE9TVFwiLFwiL3dyb25nXCIsbG9naW4sTm9uZSksKFwicG9zdF9xdWVyeVwiLFwiUE9TVFwiLFwi
L2xvZ2luP3hcIixsb2dpbixOb25lKSxcbiAgICAgICAgKFwib3JpZ2luX21pc3NpbmdcIixcIlBP
U1RcIixcIi9sb2dpblwiLFsoXCJDb250ZW50LVR5cGVcIixcImFwcGxpY2F0aW9uL3gtd3d3LWZv
cm0tdXJsZW5jb2RlZFwiKV0sTm9uZSksXG4gICAgICAgIChcIm9yaWdpbl93cm9uZ1wiLFwiUE9T
VFwiLFwiL2xvZ2luXCIsWyhcIk9yaWdpblwiLFwiaW52YWxpZFwiKSwoXCJDb250ZW50LVR5cGVc
IixcImFwcGxpY2F0aW9uL3gtd3d3LWZvcm0tdXJsZW5jb2RlZFwiKV0sTm9uZSksXG4gICAgICAg
IChcIm9yaWdpbl9kdXBsaWNhdGVcIixcIlBPU1RcIixcIi9sb2dpblwiLGxvZ2luK1soXCJPcmln
aW5cIixuZXdbXCJPUklHSU5cIl0pXSxOb25lKSxcbiAgICAgICAgKFwidHlwZV9taXNzaW5nXCIs
XCJQT1NUXCIsXCIvbG9naW5cIixbKFwiT3JpZ2luXCIsbmV3W1wiT1JJR0lOXCJdKV0sTm9uZSks
XG4gICAgICAgIChcInR5cGVfd3JvbmdcIixcIlBPU1RcIixcIi9sb2dpblwiLFsoXCJPcmlnaW5c
IixuZXdbXCJPUklHSU5cIl0pLChcIkNvbnRlbnQtVHlwZVwiLFwidGV4dC9wbGFpblwiKV0sTm9u
ZSksXG4gICAgICAgIChcInR5cGVfZHVwbGljYXRlXCIsXCJQT1NUXCIsXCIvbG9naW5cIixsb2dp
bitbKFwiQ29udGVudC1UeXBlXCIsXCJhcHBsaWNhdGlvbi94LXd3dy1mb3JtLXVybGVuY29kZWRc
IildLE5vbmUpLFxuICAgICAgICAoXCJjb29raWVfZHVwbGljYXRlXCIsXCJHRVRcIixcIi9cIixb
KFwiQ29va2llXCIsXCJvdGhlcj14XCIpLChcIkNvb2tpZVwiLFwib3RoZXI9eFwiKV0sTm9uZSks
XG4gICAgICAgIChcImNvb2tpZV9zaGFwZVwiLFwiR0VUXCIsXCIvXCIsWyhcIkNvb2tpZVwiLFwi
ZDAxX2RlbW9fZmxvdz14XCIpXSxOb25lKSxcbiAgICAgICAgKFwiY29va2llX293bl9kdXBsaWNh
dGVcIixcIkdFVFwiLFwiL1wiLFsoXCJDb29raWVcIixcImQwMV9kZW1vX2Zsb3c9XCIrXCJhXCIq
NDMrXCI7IGQwMV9kZW1vX2Zsb3c9XCIrXCJhXCIqNDMpXSxOb25lKSxcbiAgICAgICAgKFwiY29v
a2llX3VucmVsYXRlZFwiLFwiR0VUXCIsXCIvXCIsWyhcIkNvb2tpZVwiLFwidW5yZWxhdGVkPXhc
IildLE5vbmUpXVxuICAgIHN0cnVjdHVyYWw9W1xuICAgICAgICAoXCJob3N0X2Fic2VudFwiLFwi
L1wiLFwiYWJzZW50XCIsW10pLChcImhvc3Rfd3JvbmdcIixcIi9cIixcIndyb25nXCIsW10pLFxu
ICAgICAgICAoXCJob3N0X2R1cGxpY2F0ZVwiLFwiL1wiLFwibm9ybWFsXCIsWyhcIkhvc3RcIixu
ZXdbXCJBVVRIT1JJVFlcIl0pXSksXG4gICAgICAgIChcInRyYW5zZmVyXCIsXCIvXCIsXCJub3Jt
YWxcIixbKFwiVHJhbnNmZXItRW5jb2RpbmdcIixcImNodW5rZWRcIildKSxcbiAgICAgICAgKFwi
ZXhwZWN0XCIsXCIvXCIsXCJub3JtYWxcIixbKFwiRXhwZWN0XCIsXCIxMDAtY29udGludWVcIild
KSxcbiAgICAgICAgKFwibGVuZ3RoX25vbnplcm9cIixcIi9cIixcIm5vcm1hbFwiLFsoXCJDb250
ZW50LUxlbmd0aFwiLFwiMVwiKV0pLFxuICAgICAgICAoXCJsZW5ndGhfZHVwbGljYXRlXCIsXCIv
XCIsXCJub3JtYWxcIixbKFwiQ29udGVudC1MZW5ndGhcIixcIjBcIiksKFwiQ29udGVudC1MZW5n
dGhcIixcIjBcIildKSxcbiAgICAgICAgKFwibGVuZ3RoX2Zvcm1hdFwiLFwiL1wiLFwibm9ybWFs
XCIsWyhcIkNvbnRlbnQtTGVuZ3RoXCIsXCIwMFwiKV0pLFxuICAgICAgICAoXCJzY2hlbWVcIixc
Imh0dHA6Ly9sb2NhbGhvc3Q6MzAwMC9cIixcIm5vcm1hbFwiLFtdKSxcbiAgICAgICAgKFwibmV0
bG9jXCIsXCIvL2xvY2FsaG9zdDozMDAwL1wiLFwibm9ybWFsXCIsW10pLChcImZyYWdtZW50XCIs
XCIvI3hcIixcIm5vcm1hbFwiLFtdKSxcbiAgICAgICAgKFwicGFyc2VfdGFyZ2V0XCIsXCJodHRw
Oi8vW1wiLFwibm9ybWFsXCIsW10pXVxuICAgIGZvciBsYWJlbCxtZXRob2QsdGFyZ2V0LGhlYWRl
cnMsaW5pdGlhbCBpbiBzcGVjaWZpY2F0aW9uczpcbiAgICAgICAgZGVmIGNoZWNrKG1ldGhvZD1t
ZXRob2QsdGFyZ2V0PXRhcmdldCxoZWFkZXJzPWhlYWRlcnMsaW5pdGlhbD1pbml0aWFsKTpcbiAg
ICAgICAgICAgIG91dGNvbWVzPVtdXG4gICAgICAgICAgICBmb3IgbnMsdmVyc2lvbiBpbiBbKG9s
ZCxcImJhc2VsaW5lXCIpLChuZXcsXCJjYW5kaWRhdGVcIildOlxuICAgICAgICAgICAgICAgIGQ9
bWFrZV9kZW1vKG5zKVxuICAgICAgICAgICAgICAgIGZvciBmaWVsZCx2YWx1ZSBpbiAoaW5pdGlh
bCBvciB7fSkuaXRlbXMoKTpcbiAgICAgICAgICAgICAgICAgICAgaWYgZmllbGQ9PVwicHJvdGVj
dGVkX2JlZm9yZVwiOmQuY2hlY2tzW1wicHJvdGVjdGVkX3dpdGhvdXRfY29va2llX2RlbmllZFwi
XT12YWx1ZVxuICAgICAgICAgICAgICAgICAgICBlbHNlOnNldGF0dHIoZCxmaWVsZCxjb3B5LmRl
ZXBjb3B5KHZhbHVlKSlcbiAgICAgICAgICAgICAgICBoPXJlcXVlc3QobnMsZCx2ZXJzaW9uLGJ1
aWxkX3dpcmUobnMsbWV0aG9kLHRhcmdldCxoZWFkZXJzKSlcbiAgICAgICAgICAgICAgICBvdXRj
b21lcy5hcHBlbmQobm9ybWFsaXplZChucyxkLGgpKVxuICAgICAgICAgICAgICAgIGlmIHZlcnNp
b249PVwiY2FuZGlkYXRlXCI6XG4gICAgICAgICAgICAgICAgICAgIGFzc2VydCBkLnByZWZsb3df
YXV0aG9yaXphdGlvbl9yZWZ1c2Fscz09MFxuICAgICAgICAgICAgYXNzZXJ0IG91dGNvbWVzWzBd
PT1vdXRjb21lc1sxXVxuICAgICAgICB2ZXJpZnkoXCJoZWFkZXJfZnJlZV9cIitsYWJlbCxcImhl
YWRlcl9mcmVlX3JvdXRlc19ndWFyZHNcIixjaGVjaylcbiAgICBmb3IgbGFiZWwsdGFyZ2V0LGhv
c3QsaGVhZGVycyBpbiBzdHJ1Y3R1cmFsOlxuICAgICAgICBkZWYgY2hlY2sodGFyZ2V0PXRhcmdl
dCxob3N0PWhvc3QsaGVhZGVycz1oZWFkZXJzKTpcbiAgICAgICAgICAgIG91dGNvbWVzPVtdXG4g
ICAgICAgICAgICBmb3IgbnMsdmVyc2lvbiBpbiBbKG9sZCxcImJhc2VsaW5lXCIpLChuZXcsXCJj
YW5kaWRhdGVcIildOlxuICAgICAgICAgICAgICAgIGQ9bWFrZV9kZW1vKG5zKVxuICAgICAgICAg
ICAgICAgIGg9cmVxdWVzdChucyxkLHZlcnNpb24sYnVpbGRfd2lyZShucyx0YXJnZXQ9dGFyZ2V0
LGhlYWRlcnM9aGVhZGVycyxob3N0PWhvc3QpKVxuICAgICAgICAgICAgICAgIG91dGNvbWVzLmFw
cGVuZChub3JtYWxpemVkKG5zLGQsaCkpXG4gICAgICAgICAgICAgICAgaWYgdGFyZ2V0LnN0YXJ0
c3dpdGgoXCIvL1wiKTpcbiAgICAgICAgICAgICAgICAgICAgYXNzZXJ0IGgucGF0aD09XCIvXCIr
dGFyZ2V0LmxzdHJpcChcIi9cIilcbiAgICAgICAgICAgICAgICAgICAgYXNzZXJ0IG5vdCBkLmRv
bmUgYW5kIGQuZmFpbHVyZSBpcyBOb25lIGFuZCBoLnJlc3BvbnNlcz09Wyg0MDQsKCkpXVxuICAg
ICAgICAgICAgICAgIGVsc2U6XG4gICAgICAgICAgICAgICAgICAgIGFzc2VydCBkLmRvbmUgYW5k
IGQuZmFpbHVyZSBpcyBub3QgTm9uZSBhbmQgaC5yZXNwb25zZXM9PVsoNDAwLCgpKV1cbiAgICAg
ICAgICAgICAgICBpZiB2ZXJzaW9uPT1cImNhbmRpZGF0ZVwiOmFzc2VydCBkLnByZWZsb3dfYXV0
aG9yaXphdGlvbl9yZWZ1c2Fscz09MFxuICAgICAgICAgICAgYXNzZXJ0IG91dGNvbWVzWzBdPT1v
dXRjb21lc1sxXVxuICAgICAgICB2ZXJpZnkoXCJoZWFkZXJfZnJlZV9ib3VuZF9cIitsYWJlbCxc
ImhlYWRlcl9mcmVlX2JvdW5kc1wiLGNoZWNrKVxuICAgICAgICBkZWYgYXV0aF9jaGVjayh0YXJn
ZXQ9dGFyZ2V0LGhvc3Q9aG9zdCxoZWFkZXJzPWhlYWRlcnMpOlxuICAgICAgICAgICAgZD1tYWtl
X2RlbW8obmV3KTtiZWZvcmU9Zmxvd19zbmFwc2hvdChkKVxuICAgICAgICAgICAgaD1yZXF1ZXN0
KG5ldyxkLFwiY2FuZGlkYXRlXCIsYnVpbGRfd2lyZShuZXcsdGFyZ2V0PXRhcmdldCxob3N0PWhv
c3QsXG4gICAgICAgICAgICAgICAgaGVhZGVycz1bKFwiQXV0aG9yaXphdGlvblwiLFwiaWdub3Jl
ZFwiKV0raGVhZGVycykpXG4gICAgICAgICAgICBpZiB0YXJnZXQuc3RhcnRzd2l0aChcIi8vXCIp
OlxuICAgICAgICAgICAgICAgIGFzc2VydCBoLnBhdGg9PVwiL1wiK3RhcmdldC5sc3RyaXAoXCIv
XCIpXG4gICAgICAgICAgICAgICAgcmVmdXNlZChuZXcsZCxoLGJlZm9yZSwxLEZhbHNlKVxuICAg
ICAgICAgICAgZWxzZTpcbiAgICAgICAgICAgICAgICBhc3NlcnQgZC5kb25lIGFuZCBkLmZhaWx1
cmUgaXMgbm90IE5vbmUgYW5kIGgucmVzcG9uc2VzPT1bKDQwMCwoKSldXG4gICAgICAgICAgICAg
ICAgYXNzZXJ0IGQucHJlZmxvd19hdXRob3JpemF0aW9uX3JlZnVzYWxzPT0wXG4gICAgICAgICAg
ICAgICAgYXNzZXJ0IGQucmVxdWVzdF9pbnZhbGlkX3JlYXNvbj09KFwiaG9zdFwiIGlmIGhvc3Qh
PVwibm9ybWFsXCIgb3IgYW55KG49PVwiSG9zdFwiIGZvciBuLHYgaW4gaGVhZGVycylcbiAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICBlbHNlIFwiYXV0aG9y
aXphdGlvblwiKVxuICAgICAgICAgICAgICAgIGFmdGVyPWZsb3dfc25hcHNob3QoZClcbiAgICAg
ICAgICAgICAgICBhc3NlcnQgYWZ0ZXJbOjRdPT1iZWZvcmVbOjRdIGFuZCBhZnRlcls2Ol09PWJl
Zm9yZVs2Ol1cbiAgICAgICAgICAgICAgICBhc3NlcnQgYWxsKHY9PTAgZm9yIHYgaW4gZC5jYWxs
cy52YWx1ZXMoKSlcbiAgICAgICAgdmVyaWZ5KFwiYXV0aG9yaXphdGlvbl9ib3VuZF9cIitsYWJl
bCxcImF1dGhvcml6YXRpb25fYm91bmRzXCIsYXV0aF9jaGVjaylcbiAgICBmb3IgYXV0aG9yaXph
dGlvbiBpbiAoRmFsc2UsVHJ1ZSk6XG4gICAgICAgIGRlZiBjb250cm9sbGVkKGF1dGhvcml6YXRp
b249YXV0aG9yaXphdGlvbik6XG4gICAgICAgICAgICBvdXRjb21lcz1bXVxuICAgICAgICAgICAg
cGFyc2VyX3BhdGg9XCIvL1wiK25ld1tcIkFVVEhPUklUWVwiXStcIi9cIlxuICAgICAgICAgICAg
Zm9yIG5zLHZlcnNpb24gaW4gWyhvbGQsXCJiYXNlbGluZVwiKSwobmV3LFwiY2FuZGlkYXRlXCIp
XTpcbiAgICAgICAgICAgICAgICBkPW1ha2VfZGVtbyhucyk7YmVmb3JlPWZsb3dfc25hcHNob3Qo
ZClcbiAgICAgICAgICAgICAgICBoZWFkZXJzPVsoXCJBdXRob3JpemF0aW9uXCIsXCJpZ25vcmVk
XCIpXSBpZiBhdXRob3JpemF0aW9uIGVsc2UgW11cbiAgICAgICAgICAgICAgICBoPXJlcXVlc3Qo
bnMsZCx2ZXJzaW9uLGJ1aWxkX3dpcmUobnMsaGVhZGVycz1oZWFkZXJzKSxwYXJzZXJfcGF0aD1w
YXJzZXJfcGF0aClcbiAgICAgICAgICAgICAgICBhc3NlcnQgaC5wYXRoPT1wYXJzZXJfcGF0aFxu
ICAgICAgICAgICAgICAgIGFzc2VydCBoLnJlc3BvbnNlcz09Wyg0MDAsKCkpXSBhbmQgZC5kb25l
IGFuZCBkLmZhaWx1cmU9PVwicmVxdWVzdF9pbnZhbGlkXCJcbiAgICAgICAgICAgICAgICBhc3Nl
cnQgZC5yZXF1ZXN0X2ludmFsaWRfcmVhc29uPT0oXCJhdXRob3JpemF0aW9uXCIgaWYgYXV0aG9y
aXphdGlvbiBlbHNlIFwidGFyZ2V0X25ldGxvY1wiKVxuICAgICAgICAgICAgICAgIGFmdGVyPWZs
b3dfc25hcHNob3QoZClcbiAgICAgICAgICAgICAgICBhc3NlcnQgYWZ0ZXJbOjRdPT1iZWZvcmVb
OjRdIGFuZCBhZnRlcls2Ol09PWJlZm9yZVs2Ol1cbiAgICAgICAgICAgICAgICBhc3NlcnQgYWxs
KHY9PTAgZm9yIHYgaW4gZC5jYWxscy52YWx1ZXMoKSlcbiAgICAgICAgICAgICAgICBpZiB2ZXJz
aW9uPT1cImNhbmRpZGF0ZVwiOmFzc2VydCBkLnByZWZsb3dfYXV0aG9yaXphdGlvbl9yZWZ1c2Fs
cz09MFxuICAgICAgICAgICAgICAgIG91dGNvbWVzLmFwcGVuZChub3JtYWxpemVkKG5zLGQsaCkp
XG4gICAgICAgICAgICBhc3NlcnQgb3V0Y29tZXNbMF09PW91dGNvbWVzWzFdXG4gICAgICAgIHZl
cmlmeShcImNvbnRyb2xsZWRfcGFyc2VyX3Jlc3VsdF9cIisoXCJhdXRob3JpemF0aW9uXCIgaWYg
YXV0aG9yaXphdGlvbiBlbHNlIFwiaGVhZGVyX2ZyZWVcIiksXG4gICAgICAgICAgICAgICBcImNv
bnRyb2xsZWRfcGFyc2VyX3Jlc3VsdFwiLGNvbnRyb2xsZWQpXG4gICAgcmF3X2JvdW5kcz1bXG4g
ICAgICAgIChcInJlcXVlc3RfbGluZVwiLGJcIkdFVCAvXCIrYlwieFwiKjgxOTIrYlwiIEhUVFAv
MS4xXFxyXFxuXFxyXFxuXCIpLFxuICAgICAgICAoXCJoZWFkZXJfbGluZVwiLGJcIkdFVCAvIEhU
VFAvMS4xXFxyXFxuSG9zdDogbG9jYWxob3N0OjMwMDBcXHJcXG5YOiBcIitiXCJ4XCIqNDA5Niti
XCJcXHJcXG5cXHJcXG5cIiksXG4gICAgICAgIChcImhlYWRlcl9jb3VudFwiLGJcIkdFVCAvIEhU
VFAvMS4xXFxyXFxuSG9zdDogbG9jYWxob3N0OjMwMDBcXHJcXG5cIitiXCJYOiB4XFxyXFxuXCIq
MzMrYlwiXFxyXFxuXCIpLFxuICAgICAgICAoXCJoZWFkZXJfdG90YWxcIixiXCJHRVQgLyBIVFRQ
LzEuMVxcclxcbkhvc3Q6IGxvY2FsaG9zdDozMDAwXFxyXFxuXCIrKGJcIlg6IFwiK2JcInhcIioz
MDAwK2JcIlxcclxcblwiKSozK2JcIlxcclxcblwiKSxcbiAgICAgICAgKFwicGFyc2VyXCIsYlwi
R0VUIC8gSFRUUC85LjBcXHJcXG5Ib3N0OiBsb2NhbGhvc3Q6MzAwMFxcclxcblxcclxcblwiKV1c
biAgICBmb3IgbGFiZWwsd2lyZSBpbiByYXdfYm91bmRzOlxuICAgICAgICBkZWYgY2hlY2sod2ly
ZT13aXJlKTpcbiAgICAgICAgICAgIG91dGNvbWVzPVtdXG4gICAgICAgICAgICBmb3IgbnMsdmVy
c2lvbiBpbiBbKG9sZCxcImJhc2VsaW5lXCIpLChuZXcsXCJjYW5kaWRhdGVcIildOlxuICAgICAg
ICAgICAgICAgIGQ9bWFrZV9kZW1vKG5zKVxuICAgICAgICAgICAgICAgIGg9cmVxdWVzdChucyxk
LHZlcnNpb24sd2lyZSlcbiAgICAgICAgICAgICAgICBvdXRjb21lcy5hcHBlbmQobm9ybWFsaXpl
ZChucyxkLGgpKVxuICAgICAgICAgICAgICAgIGFzc2VydCBkLmRvbmUgYW5kIGQuZmFpbHVyZSBp
biB7XCJyZXF1ZXN0X2ludmFsaWRcIixcInJlcXVlc3RfbGltaXRcIn1cbiAgICAgICAgICAgICAg
ICBhc3NlcnQgYWxsKHY9PTAgZm9yIHYgaW4gZC5jYWxscy52YWx1ZXMoKSlcbiAgICAgICAgICAg
IGFzc2VydCBvdXRjb21lc1swXT09b3V0Y29tZXNbMV1cbiAgICAgICAgdmVyaWZ5KFwicmF3X2Jv
dW5kX1wiK2xhYmVsLFwicGFyc2VyX2hlYWRlcl9saW1pdHNcIixjaGVjaylcbiAgICBkZWYgcGVy
c2lzdCgpOlxuICAgICAgICBkPW1ha2VfZGVtbyhuZXcpXG4gICAgICAgIHJlcXVlc3QobmV3LGQs
XCJjYW5kaWRhdGVcIixidWlsZF93aXJlKG5ldyxoZWFkZXJzPVsoXCJBdXRob3JpemF0aW9uXCIs
XCJpZ25vcmVkXCIpXSkpXG4gICAgICAgIHJlcXVlc3QobmV3LGQsXCJjYW5kaWRhdGVcIixidWls
ZF93aXJlKG5ldykpXG4gICAgICAgIGFzc2VydCBkLnByZWZsb3dfYXV0aG9yaXphdGlvbl9yZWZ1
c2Fscz09MVxuICAgICAgICByZXF1ZXN0KG5ldyxkLFwiY2FuZGlkYXRlXCIsYnVpbGRfd2lyZShu
ZXcsXCJQT1NUXCIsXCIvbG9naW5cIixsb2dpbikpXG4gICAgICAgIGFzc2VydCBkLmF0dGVtcHRl
ZCBhbmQgZC5wcmVmbG93X2F1dGhvcml6YXRpb25fcmVmdXNhbHM9PTFcbiAgICAgICAgYmVmb3Jl
PWZsb3dfc25hcHNob3QoZClcbiAgICAgICAgcmVxdWVzdChuZXcsZCxcImNhbmRpZGF0ZVwiLGJ1
aWxkX3dpcmUobmV3LGhlYWRlcnM9WyhcIkF1dGhvcml6YXRpb25cIixcImlnbm9yZWRcIildKSlc
biAgICAgICAgYWZ0ZXI9Zmxvd19zbmFwc2hvdChkKVxuICAgICAgICBhc3NlcnQgYWZ0ZXJbOjRd
PT1iZWZvcmVbOjRdIGFuZCBhZnRlcls2Ol09PWJlZm9yZVs2Ol1cbiAgICAgICAgYXNzZXJ0IGQu
cHJlZmxvd19hdXRob3JpemF0aW9uX3JlZnVzYWxzPT0xIGFuZCBkLnJlcXVlc3RfaW52YWxpZF9y
ZWFzb249PVwiYXV0aG9yaXphdGlvblwiXG4gICAgICAgIGNvcGllZD1yZWNvcmRfY29weShuZXcs
ZClcbiAgICAgICAgZC5jbGVhcigpXG4gICAgICAgIGFzc2VydCBjb3BpZWRbXCJwcmVmbG93X2F1
dGhvcml6YXRpb25fcmVmdXNhbHNcIl09PTFcbiAgICAgICAgYXNzZXJ0IGQucHJlZmxvd19hdXRo
b3JpemF0aW9uX3JlZnVzYWxzPT0xIGFuZCBkLnJlcXVlc3RfaW52YWxpZF9yZWFzb249PVwiYXV0
aG9yaXphdGlvblwiXG4gICAgdmVyaWZ5KFwiY291bnRlcl9hY3Jvc3NfaGVhZGVyX2ZyZWVfZmxv
d19jbGVhbnVwXCIsXCJjb3VudGVyX3BlcnNpc3RlbmNlXCIscGVyc2lzdClcbiAgICBkZWYgZmly
c3Rfb25seSgpOlxuICAgICAgICBkPW1ha2VfZGVtbyhuZXcpO2QucmVxdWVzdF9pbnZhbGlkX3Jl
YXNvbj1cImh0dHBfcGFyc2VcIlxuICAgICAgICBoPXJlcXVlc3QobmV3LGQsXCJjYW5kaWRhdGVc
IixidWlsZF93aXJlKG5ldyxoZWFkZXJzPVsoXCJBdXRob3JpemF0aW9uXCIsXCJpZ25vcmVkXCIp
XSkpXG4gICAgICAgIGFzc2VydCBoLnJlc3BvbnNlcz09Wyg0MDMsKCkpXSBhbmQgZC5yZXF1ZXN0
X2ludmFsaWRfcmVhc29uPT1cImh0dHBfcGFyc2VcIlxuICAgICAgICBhc3NlcnQgZC5wcmVmbG93
X2F1dGhvcml6YXRpb25fcmVmdXNhbHM9PTFcbiAgICAgICAgZC5jbGVhcigpXG4gICAgICAgIGFz
c2VydCBkLnJlcXVlc3RfaW52YWxpZF9yZWFzb249PVwiaHR0cF9wYXJzZVwiIGFuZCBkLnByZWZs
b3dfYXV0aG9yaXphdGlvbl9yZWZ1c2Fscz09MVxuICAgIHZlcmlmeShcImZpcnN0X3JlYXNvbl9w
cmVzZXJ2ZWRcIixcImZpcnN0X3JlYXNvblwiLGZpcnN0X29ubHkpXG4gICAgZGVmIHdyaXRlX2Zh
aWx1cmUoKTpcbiAgICAgICAgZ2xvYmFsIGV4cGVjdGVkX3dyaXRlX2ZhaWx1cmVzXG4gICAgICAg
IGQ9bWFrZV9kZW1vKG5ldyk7YmVmb3JlPWZsb3dfc25hcHNob3QoZClcbiAgICAgICAgdHJ5Olxu
ICAgICAgICAgICAgcmVxdWVzdChuZXcsZCxcImNhbmRpZGF0ZVwiLGJ1aWxkX3dpcmUobmV3LGhl
YWRlcnM9WyhcIkF1dGhvcml6YXRpb25cIixcImlnbm9yZWRcIildKSx3cml0ZV9mYWlsPVRydWUp
XG4gICAgICAgIGV4Y2VwdCBXcml0ZUZhaWx1cmU6XG4gICAgICAgICAgICBleHBlY3RlZF93cml0
ZV9mYWlsdXJlcys9MVxuICAgICAgICBlbHNlOlxuICAgICAgICAgICAgcmFpc2UgQXNzZXJ0aW9u
RXJyb3JcbiAgICAgICAgYXNzZXJ0IGQucHJlZmxvd19hdXRob3JpemF0aW9uX3JlZnVzYWxzPT0x
IGFuZCBkLnJlcXVlc3RfaW52YWxpZF9yZWFzb249PVwiYXV0aG9yaXphdGlvblwiXG4gICAgICAg
IGFzc2VydCBmbG93X3NuYXBzaG90KGQpPT1iZWZvcmUgYW5kIGFsbCh2PT0wIGZvciB2IGluIGQu
Y2FsbHMudmFsdWVzKCkpXG4gICAgICAgIG5ld1tcImhhbmRsZV9lcnJvclwiXShTaW1wbGVOYW1l
c3BhY2UoZGVtbz1kKSxOb25lLE5vbmUpXG4gICAgICAgIGFzc2VydCBkLmZhaWx1cmU9PVwidW5l
eHBlY3RlZF9mYWlsdXJlXCIgYW5kIGQuZG9uZVxuICAgICAgICBhc3NlcnQgZC5wcmVmbG93X2F1
dGhvcml6YXRpb25fcmVmdXNhbHM9PTEgYW5kIGQucmVxdWVzdF9pbnZhbGlkX3JlYXNvbj09XCJh
dXRob3JpemF0aW9uXCJcbiAgICAgICAgYXNzZXJ0IHJlY29yZF9jb3B5KG5ldyxkKVtcInByZWZs
b3dfYXV0aG9yaXphdGlvbl9yZWZ1c2Fsc1wiXT09MVxuICAgIHZlcmlmeShcIjQwM193cml0ZV9m
YWlsdXJlX3Byb3BhZ2F0ZXNcIixcInJlc3BvbnNlX2ZhaWx1cmVzXCIsd3JpdGVfZmFpbHVyZSlc
biAgICBkZWYgcmVzcG9uc2VfdGltZW91dCgpOlxuICAgICAgICBnbG9iYWwgZXhwZWN0ZWRfdGlt
ZW91dHNcbiAgICAgICAgZD1tYWtlX2RlbW8obmV3KVxuICAgICAgICB0cnk6XG4gICAgICAgICAg
ICByZXF1ZXN0KG5ldyxkLFwiY2FuZGlkYXRlXCIsYnVpbGRfd2lyZShuZXcsaGVhZGVycz1bKFwi
QXV0aG9yaXphdGlvblwiLFwiaWdub3JlZFwiKV0pLHRpbWVvdXQ9VHJ1ZSlcbiAgICAgICAgZXhj
ZXB0IG5ld1tcIkhhbHRcIl0gYXMgZmFpbHVyZTpcbiAgICAgICAgICAgIGFzc2VydCBmYWlsdXJl
LnRhZz09XCJyZXNwb25zZV90aW1lb3V0XCJcbiAgICAgICAgICAgIGV4cGVjdGVkX3RpbWVvdXRz
Kz0xXG4gICAgICAgIGVsc2U6XG4gICAgICAgICAgICByYWlzZSBBc3NlcnRpb25FcnJvclxuICAg
ICAgICBhc3NlcnQgZC5wcmVmbG93X2F1dGhvcml6YXRpb25fcmVmdXNhbHM9PTEgYW5kIGQucmVx
dWVzdF9pbnZhbGlkX3JlYXNvbj09XCJhdXRob3JpemF0aW9uXCJcbiAgICAgICAgYXNzZXJ0IGFs
bCh2PT0wIGZvciB2IGluIGQuY2FsbHMudmFsdWVzKCkpXG4gICAgICAgIGdhdGVfZmFpbHMobmV3
LGQpXG4gICAgdmVyaWZ5KFwiNDAzX3Jlc3BvbnNlX3RpbWVvdXRfcHJvcGFnYXRlc1wiLFwicmVz
cG9uc2VfZmFpbHVyZXNcIixyZXNwb25zZV90aW1lb3V0KVxuXG5yZXN1bHQ9XCJmYWlsZWRcIlxu
ZmFpbHVyZT1Ob25lXG5zaWduYWwuc2lnbmFsKHNpZ25hbC5TSUdBTFJNLGRlYWRsaW5lKVxuc2ln
bmFsLnNldGl0aW1lcihzaWduYWwuSVRJTUVSX1JFQUwsMzApXG50cnk6XG4gICAgcnVuKClcbiAg
ICByZXN1bHQ9XCJwYXNzZWRcIlxuZXhjZXB0IFZlcmlmaWNhdGlvbkRlYWRsaW5lOlxuICAgIGZh
aWx1cmU9XCJ2ZXJpZmljYXRpb25fZGVhZGxpbmVcIlxuZXhjZXB0IEFzc2VydGlvbkVycm9yOlxu
ICAgIGZhaWx1cmU9XCJhc3NlcnRpb25fZmFpbGVkXCJcbmV4Y2VwdCBCYXNlRXhjZXB0aW9uOlxu
ICAgIGZhaWx1cmU9XCJoYXJuZXNzX2V4Y2VwdGlvblwiXG5maW5hbGx5OlxuICAgIHNpZ25hbC5z
ZXRpdGltZXIoc2lnbmFsLklUSU1FUl9SRUFMLDApXG5lbGFwc2VkPXJvdW5kKHRpbWUubW9ub3Rv
bmljKCktc3RhcnRlZCw2KVxucHJpbnQoanNvbi5kdW1wcyh7XCJyZXN1bHRcIjpyZXN1bHQsXCJm
YWlsdXJlX3RhZ1wiOmZhaWx1cmUsXCJmYWlsZWRfY2FzZVwiOmNhc2UgaWYgZmFpbHVyZSBlbHNl
IE5vbmUsXG4gXCJjb21wbGV0ZWRfY2FzZXNcIjpjb21wbGV0ZWQsXCJncm91cHNcIjpncm91cHMs
XCJoYW5kbGVyX2NhbGxzXCI6aGFuZGxlcl9jYWxscyxcbiBcImV4cGVjdGVkX3dyaXRlX2ZhaWx1
cmVzXCI6ZXhwZWN0ZWRfd3JpdGVfZmFpbHVyZXMsXCJleHBlY3RlZF9yZXNwb25zZV90aW1lb3V0
c1wiOmV4cGVjdGVkX3RpbWVvdXRzLFxuIFwiZWxhcHNlZF9zZWNvbmRzXCI6ZWxhcHNlZCxcImRl
YWRsaW5lX3NlY29uZHNcIjozMCxcImhlbHBlcl9zaGEyNTZcIjpTT1VSQ0VfU0hBLFxuIFwibWFp
bl9pbnZva2VkXCI6RmFsc2UsXCJuZXR3b3JrX2xpc3RlbmVyX25hdGl2ZV9wcm92aWRlcl9jbGlf
ZHJpdmVyX2Jyb3dzZXJfY2FyZ29cIjpGYWxzZX0sc29ydF9rZXlzPVRydWUpKVxucmFpc2UgU3lz
dGVtRXhpdCgwIGlmIHJlc3VsdD09XCJwYXNzZWRcIiBlbHNlIDEpXG4iCmltcG9ydCBhc3QsIGNv
bnRleHRsaWIsIGNvcHksIGhhc2hsaWIsIGh0dHAuc2VydmVyLCBpbywganNvbiwgcmUsIHNpZ25h
bCwgc3lzLCB0aW1lLCB1cmxsaWIucGFyc2UKZnJvbSB0eXBlcyBpbXBvcnQgU2ltcGxlTmFtZXNw
YWNlLCBGcmFtZVR5cGUsIFRyYWNlYmFja1R5cGUsIEZ1bmN0aW9uVHlwZQoKU0VOVElORUwgPSAi
RDAxX1BSSVZBVEVfTUVNT1JZX1NFTlRJTkVMIgpFWFBFQ1RFRF9MRUdBQ1kgPSB7InJlY29yZF9j
b3B5IjogMSwgImJvdW5kZWRfc2VxdWVuY2UiOiA0LCAiYXV0aG9yaXphdGlvbl9wcmVzZW5jZSI6
IDYsCiAgICAicG9zdGZsb3ciOiA3LCAiaGVhZGVyX2ZyZWVfcm91dGVzX2d1YXJkcyI6IDI1LCAi
aGVhZGVyX2ZyZWVfYm91bmRzIjogMTIsCiAgICAiYXV0aG9yaXphdGlvbl9ib3VuZHMiOiAxMiwg
ImNvbnRyb2xsZWRfcGFyc2VyX3Jlc3VsdCI6IDIsICJwYXJzZXJfaGVhZGVyX2xpbWl0cyI6IDUs
CiAgICAiY291bnRlcl9wZXJzaXN0ZW5jZSI6IDEsICJmaXJzdF9yZWFzb24iOiAxLCAicmVzcG9u
c2VfZmFpbHVyZXMiOiAyfQpFWFBFQ1RFRF9PQlNFUlZFUiA9IHsiY2xhc3NlcyI6IDExLCAidHJh
Y2UiOiA4LCAiZmF1bHRzIjogNSwgImRlc2NyaXB0b3IiOiAyLAogICAgImZpcnN0IjogMiwgInNp
dGVzIjogNiwgImtub3duIjogMywgImZhbGxiYWNrIjogMTAsICJwcml2YWN5IjogM30KU09VUkNF
X0hBU0ggPSAiNzViZmI4YTYzZDY4YWVlNmVlNmIyZTUwMDk1OWMwZTdkODBhNjMzMWYxYzY5MDAy
MmM0MzAwMTM0YzdkMzRmMSIKVU5DSEFOR0VEX0hBU0ggPSAiN2ZiYzIzZTU2ZGJjNDk5OWI5NDY3
MmVjNGIyOWIwZDMzNTk2YzUzYjRkNjM3ZWE5OWY4NTA0NTFiZDYzZmJmMCIKQkFTRUxJTkVfSEFT
SCA9ICJhYzNjMzUwZDBlYTVmMWM3NDQ4ZDUzMzUyNGU1ZGIyZjk5ZWUzN2FkNjFiZTMxM2M2ZjM2
ZTBjNDAyNWEyZmNkIgpMRUdBQ1lfTE9HSUNfSEFTSCA9ICI5OGIxOWI0ZDQyMjA2N2IwZDQ5ZWE3
ZWIxMDM1YTcxOWQ4OGJkYTY3MmRjMzY1NzljNmNmNWM0N2NiMDkyNGViIgoKY2xhc3MgRW52ZWxv
cGVEZWFkbGluZShCYXNlRXhjZXB0aW9uKToKICAgIHBhc3MKCmNsYXNzIFByaXZhdGVUcmFwKEJh
c2VFeGNlcHRpb24pOgogICAgcGFzcwoKY2xhc3MgR3VhcmRGYWlsdXJlKEJhc2VFeGNlcHRpb24p
OgogICAgcGFzcwoKY2xhc3MgU2VudGluZWxFcnJvcihFeGNlcHRpb24pOgogICAgZGVmIF9fc3Ry
X18oc2VsZik6CiAgICAgICAgdHJhcF9jb3VudHNbInN0ciJdICs9IDEKICAgICAgICByYWlzZSBQ
cml2YXRlVHJhcAogICAgZGVmIF9fcmVwcl9fKHNlbGYpOgogICAgICAgIHRyYXBfY291bnRzWyJy
ZXByIl0gKz0gMQogICAgICAgIHJhaXNlIFByaXZhdGVUcmFwCgpjbGFzcyBWYWx1ZVN1YmNsYXNz
KFZhbHVlRXJyb3IpOgogICAgcGFzcwoKY2xhc3MgT1NTdWJjbGFzcyhPU0Vycm9yKToKICAgIHBh
c3MKCmNsYXNzIENoYW5naW5nVHJhY2ViYWNrKFNlbnRpbmVsRXJyb3IpOgogICAgZGVmIF9faW5p
dF9fKHNlbGYsIG5hdGl2ZSk6CiAgICAgICAgc2VsZi5uYXRpdmUgPSBuYXRpdmUKICAgICAgICBz
dXBlcigpLl9faW5pdF9fKFNFTlRJTkVMKQogICAgZGVmIF9fZ2V0YXR0cmlidXRlX18oc2VsZiwg
bmFtZSk6CiAgICAgICAgaWYgbmFtZSA9PSAiX190cmFjZWJhY2tfXyI6CiAgICAgICAgICAgIHRy
YXBfY291bnRzWyJ0cmFjZWJhY2siXSArPSAxCiAgICAgICAgICAgIHJldHVybiBTRU5USU5FTCBp
ZiB0cmFwX2NvdW50c1sidHJhY2ViYWNrIl0gJSAyIGVsc2Ugb2JqZWN0Ll9fZ2V0YXR0cmlidXRl
X18oc2VsZiwgIm5hdGl2ZSIpCiAgICAgICAgcmV0dXJuIG9iamVjdC5fX2dldGF0dHJpYnV0ZV9f
KHNlbGYsIG5hbWUpCgpjbGFzcyBSYWlzaW5nVHJhY2ViYWNrKFNlbnRpbmVsRXJyb3IpOgogICAg
ZGVmIF9fZ2V0YXR0cmlidXRlX18oc2VsZiwgbmFtZSk6CiAgICAgICAgaWYgbmFtZSA9PSAiX190
cmFjZWJhY2tfXyI6CiAgICAgICAgICAgIHRyYXBfY291bnRzWyJ0cmFjZWJhY2siXSArPSAxCiAg
ICAgICAgICAgIHJhaXNlIFByaXZhdGVUcmFwKFNFTlRJTkVMKQogICAgICAgIHJldHVybiBvYmpl
Y3QuX19nZXRhdHRyaWJ1dGVfXyhzZWxmLCBuYW1lKQoKY2xhc3MgRmF1bHRIb2xkZXIoZGljdCk6
CiAgICBkZWYgX19pbml0X18oc2VsZiwgbW9kZSk6CiAgICAgICAgc3VwZXIoKS5fX2luaXRfXyhv
YnNlcnZlZD1GYWxzZSwgZGlhZ25vc3RpYz1Ob25lKQogICAgICAgIHNlbGYubW9kZSA9IG1vZGUK
ICAgIGRlZiBfX2dldGl0ZW1fXyhzZWxmLCBrZXkpOgogICAgICAgIGlmIHNlbGYubW9kZSA9PSAi
cmVhZCIgYW5kIGtleSA9PSAib2JzZXJ2ZWQiOgogICAgICAgICAgICByYWlzZSBQcml2YXRlVHJh
cAogICAgICAgIHJldHVybiBkaWN0Ll9fZ2V0aXRlbV9fKHNlbGYsIGtleSkKICAgIGRlZiBfX3Nl
dGl0ZW1fXyhzZWxmLCBrZXksIHZhbHVlKToKICAgICAgICBpZiBzZWxmLm1vZGUgPT0ga2V5Ogog
ICAgICAgICAgICByYWlzZSBQcml2YXRlVHJhcAogICAgICAgIHJldHVybiBkaWN0Ll9fc2V0aXRl
bV9fKHNlbGYsIGtleSwgdmFsdWUpCgpkZWYgZW1wdHlfaG9sZGVyKCk6CiAgICByZXR1cm4geyJv
YnNlcnZlZCI6IEZhbHNlLCAiZGlhZ25vc3RpYyI6IE5vbmV9CgpkZWYgbGVnYWN5X25hbWVzcGFj
ZSgpOgogICAgdHJlZSA9IGFzdC5wYXJzZShMRUdBQ1lfTE9HSUMpCiAgICBkZWZpbml0aW9ucyA9
IFtuIGZvciBuIGluIHRyZWUuYm9keSBpZiBpc2luc3RhbmNlKG4sIChhc3QuRnVuY3Rpb25EZWYs
IGFzdC5DbGFzc0RlZikpXQogICAgbmFtZXNwYWNlID0geyJhc3QiOiBhc3QsICJjb250ZXh0bGli
IjogY29udGV4dGxpYiwgImNvcHkiOiBjb3B5LCAiaGFzaGxpYiI6IGhhc2hsaWIsCiAgICAgICAg
Imh0dHAiOiBodHRwLCAiaW8iOiBpbywgImpzb24iOiBqc29uLCAicmUiOiByZSwgInNpZ25hbCI6
IHNpZ25hbCwgInRpbWUiOiB0aW1lLAogICAgICAgICJ1cmxsaWIiOiB1cmxsaWIsICJTaW1wbGVO
YW1lc3BhY2UiOiBTaW1wbGVOYW1lc3BhY2UsCiAgICAgICAgIlNPVVJDRV9URVhUIjogU09VUkNF
X1RFWFQsICJCQVNFTElORV9URVhUIjogTEVHQUNZX0JBU0VMSU5FX1RFWFQsCiAgICAgICAgIlNP
VVJDRV9TSEEiOiBTT1VSQ0VfSEFTSCwgIkJBU0VfQ09NTUlUIjogIjc0NjFhYjUwZjVhNWZkZjBh
YTVkN2JkNmM0YjMwOTU3NWU3YTQyMzgiLAogICAgICAgICJncm91cHMiOiB7fSwgImNvbXBsZXRl
ZCI6IDAsICJjYXNlIjogInNvdXJjZV9waW4iLAogICAgICAgICJoYW5kbGVyX2NhbGxzIjogeyJi
YXNlbGluZSI6IDAsICJjYW5kaWRhdGUiOiAwfSwKICAgICAgICAiZXhwZWN0ZWRfd3JpdGVfZmFp
bHVyZXMiOiAwLCAiZXhwZWN0ZWRfdGltZW91dHMiOiAwfQogICAgIyBPcmlnaW5hbCBmdW5jdGlv
bi9jbGFzcyBub2RlcyBhcmUgdW5tb2RpZmllZDsgbm8gb3JpZ2luYWwgdG9wLWxldmVsIHRhaWwg
cnVucy4KICAgIGV4ZWMoY29tcGlsZShhc3QuTW9kdWxlKGJvZHk9ZGVmaW5pdGlvbnMsIHR5cGVf
aWdub3Jlcz1bXSksICI8bGVnYWN5LWRlZmluaXRpb25zPiIsICJleGVjIiksIG5hbWVzcGFjZSkK
ICAgIHJldHVybiBuYW1lc3BhY2UKCmRlZiBzZWxlY3RlZF9uYW1lc3BhY2UodGV4dCk6CiAgICB0
cmVlID0gYXN0LnBhcnNlKHRleHQpCiAgICBjbGFzc2VzID0geyJGYWlsdXJlIiwgIkhhbHQiLCAi
UHJlZmxvd0F1dGhvcml6YXRpb25SZWZ1c2FsIiwgIkhlYWRlclJlYWRlciIsCiAgICAgICAgICAg
ICAgICJEZW1vIiwgIkRlbW9TZXJ2ZXIiLCAiSGFuZGxlciJ9CiAgICBmdW5jdGlvbnMgPSB7InJl
cXVpcmUiLCAib2JzZXJ2ZV91bmV4cGVjdGVkX2ZhaWx1cmUiLCAibWFpbiJ9CiAgICBzZWxlY3Rl
ZCA9IFtuIGZvciBuIGluIHRyZWUuYm9keSBpZiBpc2luc3RhbmNlKG4sIGFzdC5Bc3NpZ24pXQog
ICAgYXNzZXJ0IGFsbChub3QgYW55KGlzaW5zdGFuY2UoeCwgYXN0LkNhbGwpIGZvciB4IGluIGFz
dC53YWxrKG4pKSBmb3IgbiBpbiBzZWxlY3RlZCkKICAgIHNlbGVjdGVkICs9IFtuIGZvciBuIGlu
IHRyZWUuYm9keSBpZgogICAgICAgICAgICAgICAgIGlzaW5zdGFuY2UobiwgYXN0LkNsYXNzRGVm
KSBhbmQgbi5uYW1lIGluIGNsYXNzZXMgb3IKICAgICAgICAgICAgICAgICBpc2luc3RhbmNlKG4s
IGFzdC5GdW5jdGlvbkRlZikgYW5kIG4ubmFtZSBpbiBmdW5jdGlvbnNdCiAgICBuYW1lc3BhY2Ug
PSB7Imh0dHAiOiBodHRwLCAiY29udGV4dGxpYiI6IGNvbnRleHRsaWIsICJoYXNobGliIjogaGFz
aGxpYiwgInJlIjogcmUsCiAgICAgICAgICAgICAgICAgInN5cyI6IHN5cywgInRpbWUiOiB0aW1l
LCAidXJsbGliIjogdXJsbGlifQogICAgZXhlYyhjb21waWxlKGFzdC5Nb2R1bGUoYm9keT1zZWxl
Y3RlZCwgdHlwZV9pZ25vcmVzPVtdKSwgIjxvYnNlcnZlci1kZWZpbml0aW9ucz4iLCAiZXhlYyIp
LCBuYW1lc3BhY2UpCiAgICBuYW1lc3BhY2VbInNvdXJjZV9hc3QiXSA9IHRyZWUKICAgICMgQWxs
IHRlbiB0cnVzdGVkIGNvZGVvYmplY3RzIGV4aXN0LiBOZWl0aGVyIG1haW4gbm9yIGEgc2VydmVy
IGNvbnN0cnVjdG9yIGlzIGNhbGxlZC4KICAgIGNvZGVzID0gKG5hbWVzcGFjZVsiSGVhZGVyUmVh
ZGVyIl0ucmVhZGxpbmUuX19jb2RlX18sCiAgICAgICAgbmFtZXNwYWNlWyJEZW1vU2VydmVyIl0u
cHJvY2Vzc19yZXF1ZXN0Ll9fY29kZV9fLAogICAgICAgIG5hbWVzcGFjZVsiRGVtbyJdLmJlZ2lu
Ll9fY29kZV9fLCBuYW1lc3BhY2VbIkRlbW8iXS5jYWxsYmFjay5fX2NvZGVfXywKICAgICAgICBu
YW1lc3BhY2VbIkRlbW8iXS5pbnZva2UuX19jb2RlX18sIG5hbWVzcGFjZVsiSGFuZGxlciJdLmhh
bmRsZV9vbmVfcmVxdWVzdC5fX2NvZGVfXywKICAgICAgICBuYW1lc3BhY2VbIkhhbmRsZXIiXS5z
ZW5kX2Vycm9yLl9fY29kZV9fLCBuYW1lc3BhY2VbIkhhbmRsZXIiXS5nZXQuX19jb2RlX18sCiAg
ICAgICAgbmFtZXNwYWNlWyJIYW5kbGVyIl0ucmVwbHkuX19jb2RlX18sIG5hbWVzcGFjZVsibWFp
biJdLl9fY29kZV9fKQogICAgYXNzZXJ0IGxlbihjb2RlcykgPT0gMTAKICAgIHJldHVybiBuYW1l
c3BhY2UKCmRlZiBuZXdfZGVtbyhuYW1lc3BhY2UpOgogICAgcmV0dXJuIGxlZ2FjeVsibWFrZV9k
ZW1vIl0gKG5hbWVzcGFjZSkKCmRlZiBob2xkZXJfb2YoZGVtbyk6CiAgICByZXR1cm4gZGVtby51
bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24KCmRlZiBjaGVja2VkX2hvbGRlcihob2xkZXIp
OgogICAgdmFsdWUgPSB7a2V5OiB2YWx1ZSBmb3Iga2V5LCB2YWx1ZSBpbiBkaWN0Lml0ZW1zKGhv
bGRlcil9CiAgICBhc3NlcnQgc2V0KHZhbHVlKSA9PSB7Im9ic2VydmVkIiwgImRpYWdub3N0aWMi
fSBhbmQgdHlwZSh2YWx1ZVsib2JzZXJ2ZWQiXSkgaXMgYm9vbAogICAgZGlhZ25vc3RpYyA9IHZh
bHVlWyJkaWFnbm9zdGljIl0KICAgIGlmIGRpYWdub3N0aWMgaXMgbm90IE5vbmU6CiAgICAgICAg
YXNzZXJ0IHZhbHVlWyJvYnNlcnZlZCJdIGlzIFRydWUKICAgICAgICBhc3NlcnQgc2V0KGRpYWdu
b3N0aWMpID09IHsic2l0ZSIsICJleGNlcHRpb25fY2xhc3MiLCAib3duX2Z1bmN0aW9uIiwgIm93
bl9saW5lIn0KICAgICAgICBhc3NlcnQgZGlhZ25vc3RpY1sic2l0ZSJdIGluIHsiaGFuZGxlciIs
ICJzZXJ2ZXIiLCAibWFpbiJ9CiAgICAgICAgYXNzZXJ0IGRpYWdub3N0aWNbImV4Y2VwdGlvbl9j
bGFzcyJdIGluIHsiQXR0cmlidXRlRXJyb3IiLCAiVHlwZUVycm9yIiwgIlZhbHVlRXJyb3IiLAog
ICAgICAgICAgICAiS2V5RXJyb3IiLCAiT1NFcnJvciIsICJCcm9rZW5QaXBlRXJyb3IiLCAiQ29u
bmVjdGlvblJlc2V0RXJyb3IiLCAiVGltZW91dEVycm9yIiwgIm90aGVyIn0KICAgICAgICBhc3Nl
cnQgZGlhZ25vc3RpY1sib3duX2Z1bmN0aW9uIl0gaW4ge05vbmUsICJIZWFkZXJSZWFkZXIucmVh
ZGxpbmUiLAogICAgICAgICAgICAiRGVtb1NlcnZlci5wcm9jZXNzX3JlcXVlc3QiLCAiRGVtby5i
ZWdpbiIsICJEZW1vLmNhbGxiYWNrIiwgIkRlbW8uaW52b2tlIiwKICAgICAgICAgICAgIkhhbmRs
ZXIuaGFuZGxlX29uZV9yZXF1ZXN0IiwgIkhhbmRsZXIuc2VuZF9lcnJvciIsICJIYW5kbGVyLmdl
dCIsICJIYW5kbGVyLnJlcGx5IiwgIm1haW4ifQogICAgICAgIGFzc2VydCAoZGlhZ25vc3RpY1si
b3duX2Z1bmN0aW9uIl0gaXMgTm9uZSkgPT0gKGRpYWdub3N0aWNbIm93bl9saW5lIl0gaXMgTm9u
ZSkKICAgICAgICBhc3NlcnQgZGlhZ25vc3RpY1sib3duX2xpbmUiXSBpcyBOb25lIG9yIHR5cGUo
ZGlhZ25vc3RpY1sib3duX2xpbmUiXSkgaXMgaW50IGFuZCAxIDw9IGRpYWdub3N0aWNbIm93bl9s
aW5lIl0gPD0gMTAyNAogICAgcmF3ID0ganNvbi5kdW1wcyh2YWx1ZSwgc29ydF9rZXlzPVRydWUp
LmVuY29kZSgiYXNjaWkiKQogICAgYXNzZXJ0IGxlbihyYXcpIDw9IDE2MCBhbmQgU0VOVElORUwu
ZW5jb2RlKCkgbm90IGluIHJhdwogICAgcmV0dXJuIHZhbHVlCgpkZWYgb2JzZXJ2ZShuYW1lc3Bh
Y2UsIGVycm9yLCBob2xkZXI9Tm9uZSwgc2l0ZT0iaGFuZGxlciIsIG5hdGl2ZT1Ob25lKToKICAg
IGhvbGRlciA9IGVtcHR5X2hvbGRlcigpIGlmIGhvbGRlciBpcyBOb25lIGVsc2UgaG9sZGVyCiAg
ICB0cnk6CiAgICAgICAgcmFpc2UgZXJyb3IKICAgIGV4Y2VwdCBCYXNlRXhjZXB0aW9uOgogICAg
ICAgIGlmIG5hdGl2ZSBpcyBub3QgTm9uZToKICAgICAgICAgICAgYXNzZXJ0IHR5cGUobmF0aXZl
KSBpcyBUcmFjZWJhY2tUeXBlCiAgICAgICAgICAgIEJhc2VFeGNlcHRpb24uX190cmFjZWJhY2tf
Xy5fX3NldF9fKGVycm9yLCBuYXRpdmUpCiAgICAgICAgbmFtZXNwYWNlWyJvYnNlcnZlX3VuZXhw
ZWN0ZWRfZmFpbHVyZSJdIChob2xkZXIsIHNpdGUpCiAgICByZXR1cm4gY2hlY2tlZF9ob2xkZXIo
aG9sZGVyKQoKZGVmIGNhcHR1cmVfcmVwbHlfZnJhbWUobmFtZXNwYWNlLCBjbG9uZWQ9RmFsc2Up
OgogICAgb3JpZ2luYWwgPSBuYW1lc3BhY2VbIkhhbmRsZXIiXS5yZXBseQogICAgZnVuY3Rpb24g
PSBvcmlnaW5hbAogICAgaWYgY2xvbmVkOgogICAgICAgIGZ1bmN0aW9uID0gRnVuY3Rpb25UeXBl
KG9yaWdpbmFsLl9fY29kZV9fLnJlcGxhY2UoKSwgb3JpZ2luYWwuX19nbG9iYWxzX18sCiAgICAg
ICAgICAgICAgICAgICAgICAgICAgICAgICAgInVudHJ1c3RlZF9yZXBseV9jbG9uZSIsIG9yaWdp
bmFsLl9fZGVmYXVsdHNfXykKICAgICAgICBmdW5jdGlvbi5fX2t3ZGVmYXVsdHNfXyA9IGRpY3Qo
b3JpZ2luYWwuX19rd2RlZmF1bHRzX18pCiAgICAgICAgYXNzZXJ0IGZ1bmN0aW9uLl9fY29kZV9f
ID09IG9yaWdpbmFsLl9fY29kZV9fIGFuZCBmdW5jdGlvbi5fX2NvZGVfXyBpcyBub3Qgb3JpZ2lu
YWwuX19jb2RlX18KICAgIGhhbmRsZXIgPSBtZW1vcnlfaGFuZGxlcihuYW1lc3BhY2UsIG5ld19k
ZW1vKG5hbWVzcGFjZSkpCiAgICBoYW5kbGVyLndmaWxlLm9uY2UgPSBUcnVlCiAgICB0cnk6CiAg
ICAgICAgIyBPbmx5IHRoaXMgcmVzcG9uc2UgYm9keSBydW5zOyBhbGwgSS9PIGFuZCBidWRnZXQg
b3BlcmF0aW9ucyBhcmUgbWVtb3J5IHNpbmtzLgogICAgICAgIGZ1bmN0aW9uKGhhbmRsZXIsIDIw
MCwgIm1lbW9yeSBmcmFtZSBjYXB0dXJlIikKICAgIGV4Y2VwdCBPU0Vycm9yIGFzIGVycm9yOgog
ICAgICAgIHRyYWNlID0gQmFzZUV4Y2VwdGlvbi5fX3RyYWNlYmFja19fLl9fZ2V0X18oZXJyb3Is
IEJhc2VFeGNlcHRpb24pCiAgICAgICAgZm9yIF8gaW4gcmFuZ2UoNjQpOgogICAgICAgICAgICBp
ZiB0cmFjZSBpcyBOb25lOgogICAgICAgICAgICAgICAgYnJlYWsKICAgICAgICAgICAgYXNzZXJ0
IHR5cGUodHJhY2UpIGlzIFRyYWNlYmFja1R5cGUgYW5kIHR5cGUodHJhY2UudGJfZnJhbWUpIGlz
IEZyYW1lVHlwZQogICAgICAgICAgICBpZiB0cmFjZS50Yl9mcmFtZS5mX2NvZGUgaXMgZnVuY3Rp
b24uX19jb2RlX186CiAgICAgICAgICAgICAgICByZXR1cm4gdHJhY2UudGJfZnJhbWUsIHRyYWNl
LnRiX2xhc3RpCiAgICAgICAgICAgIHRyYWNlID0gdHJhY2UudGJfbmV4dAogICAgcmFpc2UgQXNz
ZXJ0aW9uRXJyb3IKCmRlZiBjYXB0dXJlX2ZpeHR1cmVfZnJhbWUoKToKICAgIHRyeToKICAgICAg
ICByYWlzZSBTZW50aW5lbEVycm9yKFNFTlRJTkVMKQogICAgZXhjZXB0IFNlbnRpbmVsRXJyb3Ig
YXMgZXJyb3I6CiAgICAgICAgdHJhY2UgPSBCYXNlRXhjZXB0aW9uLl9fdHJhY2ViYWNrX18uX19n
ZXRfXyhlcnJvciwgQmFzZUV4Y2VwdGlvbikKICAgICAgICBhc3NlcnQgdHlwZSh0cmFjZSkgaXMg
VHJhY2ViYWNrVHlwZSBhbmQgdHlwZSh0cmFjZS50Yl9mcmFtZSkgaXMgRnJhbWVUeXBlCiAgICAg
ICAgYXNzZXJ0IHRyYWNlLnRiX2ZyYW1lLmZfY29kZSBpcyBjYXB0dXJlX2ZpeHR1cmVfZnJhbWUu
X19jb2RlX18gYW5kIHRyYWNlLnRiX25leHQgaXMgTm9uZQogICAgICAgIHJldHVybiB0cmFjZS50
Yl9mcmFtZSwgdHJhY2UudGJfbGFzdGkKCmRlZiBuYXRpdmVfdHJhY2UobGluZSwgY291bnQ9MSwg
dHJ1c3RlZD1UcnVlLCBjbG9uZWQ9RmFsc2UpOgogICAga2V5ID0gImNsb25lZCIgaWYgY2xvbmVk
IGVsc2UgInRydXN0ZWQiIGlmIHRydXN0ZWQgZWxzZSAidW50cnVzdGVkIgogICAgZnJhbWUsIGxh
c3RpID0gbmF0aXZlX2ZyYW1lc1trZXldCiAgICBhc3NlcnQgdHlwZShmcmFtZSkgaXMgRnJhbWVU
eXBlIGFuZCB0eXBlKGxhc3RpKSBpcyBpbnQKICAgIHRyYWNlID0gTm9uZQogICAgZm9yIF8gaW4g
cmFuZ2UoY291bnQpOgogICAgICAgICMgTmF0aXZlIHRyYWNlYmFjayB3aXRoIGEgZ2VudWluZSBj
YXB0dXJlZCBmcmFtZSBhbmQgYSBjb250cm9sbGVkIGxpbmUgYW5ub3RhdGlvbi4KICAgICAgICB0
cmFjZSA9IFRyYWNlYmFja1R5cGUodHJhY2UsIGZyYW1lLCBsYXN0aSwgbGluZSkKICAgICAgICBh
c3NlcnQgdHlwZSh0cmFjZSkgaXMgVHJhY2ViYWNrVHlwZQogICAgcmV0dXJuIHRyYWNlCgpkZWYg
Y2xhc3NfY2FzZShuYW1lKToKICAgIGNsYXNzZXMgPSB7IkF0dHJpYnV0ZUVycm9yIjogQXR0cmli
dXRlRXJyb3IsICJUeXBlRXJyb3IiOiBUeXBlRXJyb3IsICJWYWx1ZUVycm9yIjogVmFsdWVFcnJv
ciwKICAgICAgICAiS2V5RXJyb3IiOiBLZXlFcnJvciwgIk9TRXJyb3IiOiBPU0Vycm9yLCAiQnJv
a2VuUGlwZUVycm9yIjogQnJva2VuUGlwZUVycm9yLAogICAgICAgICJDb25uZWN0aW9uUmVzZXRF
cnJvciI6IENvbm5lY3Rpb25SZXNldEVycm9yLCAiVGltZW91dEVycm9yIjogVGltZW91dEVycm9y
LAogICAgICAgICJWYWx1ZVN1YmNsYXNzIjogVmFsdWVTdWJjbGFzcywgIk9TU3ViY2xhc3MiOiBP
U1N1YmNsYXNzLCAiU2VudGluZWxFcnJvciI6IFNlbnRpbmVsRXJyb3J9CiAgICBlcnJvciA9IGNs
YXNzZXNbbmFtZV0gKFNFTlRJTkVMKQogICAgYmVmb3JlID0gZGljdCh0cmFwX2NvdW50cykKICAg
IHZhbHVlID0gb2JzZXJ2ZShhd2FyZSwgZXJyb3IpCiAgICBhc3NlcnQgdmFsdWVbIm9ic2VydmVk
Il0gYW5kIHZhbHVlWyJkaWFnbm9zdGljIl1bImV4Y2VwdGlvbl9jbGFzcyJdID09IChuYW1lIGlm
IG5hbWUgaW4gewogICAgICAgICJBdHRyaWJ1dGVFcnJvciIsICJUeXBlRXJyb3IiLCAiVmFsdWVF
cnJvciIsICJLZXlFcnJvciIsICJPU0Vycm9yIiwKICAgICAgICAiQnJva2VuUGlwZUVycm9yIiwg
IkNvbm5lY3Rpb25SZXNldEVycm9yIiwgIlRpbWVvdXRFcnJvciJ9IGVsc2UgIm90aGVyIikKICAg
IGFzc2VydCB2YWx1ZVsiZGlhZ25vc3RpYyJdWyJvd25fZnVuY3Rpb24iXSBpcyBOb25lCiAgICBh
c3NlcnQgdHJhcF9jb3VudHMgPT0gYmVmb3JlCgpkZWYgdHJhY2VfY2FzZShraW5kKToKICAgIG9w
dGlvbnMgPSB7ImxpbmUxIjogKDEsIDEsIFRydWUsIEZhbHNlKSwgImxpbmUxMDI0IjogKDEwMjQs
IDEsIFRydWUsIEZhbHNlKSwKICAgICAgICAibGluZTAiOiAoMCwgMSwgVHJ1ZSwgRmFsc2UpLCAi
bGluZTEwMjUiOiAoMTAyNSwgMSwgVHJ1ZSwgRmFsc2UpLAogICAgICAgICJsaW5rczY0IjogKDUx
MiwgNjQsIFRydWUsIEZhbHNlKSwgImxpbmtzNjUiOiAoNTEyLCA2NSwgVHJ1ZSwgRmFsc2UpLAog
ICAgICAgICJ1bnRydXN0ZWQiOiAoNTEyLCAxLCBGYWxzZSwgRmFsc2UpLCAiY2xvbmVkIjogKDUx
MiwgMSwgVHJ1ZSwgVHJ1ZSl9CiAgICBsaW5lLCBjb3VudCwgdHJ1c3RlZCwgY2xvbmVkID0gb3B0
aW9uc1traW5kXQogICAgdmFsdWUgPSBvYnNlcnZlKGF3YXJlLCBTZW50aW5lbEVycm9yKFNFTlRJ
TkVMKSwgbmF0aXZlPW5hdGl2ZV90cmFjZShsaW5lLCBjb3VudCwgdHJ1c3RlZCwgY2xvbmVkKSkK
ICAgIHZhbGlkID0gbGluZSBpbiAoMSwgMTAyNCwgNTEyKSBhbmQgY291bnQgPD0gNjQgYW5kIHRy
dXN0ZWQgYW5kIG5vdCBjbG9uZWQKICAgIGFzc2VydCB2YWx1ZVsiZGlhZ25vc3RpYyJdWyJvd25f
ZnVuY3Rpb24iXSA9PSAoIkhhbmRsZXIucmVwbHkiIGlmIHZhbGlkIGVsc2UgTm9uZSkKICAgIGFz
c2VydCB2YWx1ZVsiZGlhZ25vc3RpYyJdWyJvd25fbGluZSJdID09IChsaW5lIGlmIHZhbGlkIGVs
c2UgTm9uZSkKCmRlZiBkZXNjcmlwdG9yX2Nhc2Uoa2luZCk6CiAgICBuYXRpdmUgPSBuYXRpdmVf
dHJhY2UoMSkKICAgIGVycm9yID0gQ2hhbmdpbmdUcmFjZWJhY2sobmF0aXZlKSBpZiBraW5kID09
ICJjaGFuZ2luZyIgZWxzZSBSYWlzaW5nVHJhY2ViYWNrKFNFTlRJTkVMKQogICAgYmVmb3JlID0g
ZGljdCh0cmFwX2NvdW50cykKICAgIHZhbHVlID0gb2JzZXJ2ZShhd2FyZSwgZXJyb3IsIG5hdGl2
ZT1uYXRpdmUpCiAgICBhc3NlcnQgdmFsdWUgPT0geyJvYnNlcnZlZCI6IFRydWUsICJkaWFnbm9z
dGljIjogeyJzaXRlIjogImhhbmRsZXIiLAogICAgICAgICJleGNlcHRpb25fY2xhc3MiOiAib3Ro
ZXIiLCAib3duX2Z1bmN0aW9uIjogIkhhbmRsZXIucmVwbHkiLCAib3duX2xpbmUiOiAxfX0KICAg
IGFzc2VydCB0cmFwX2NvdW50cyA9PSBiZWZvcmUgYW5kIHRyYXBfY291bnRzWyJ0cmFjZWJhY2si
XSA9PSAwCiAgICBhc3NlcnQgU0VOVElORUwgbm90IGluIGpzb24uZHVtcHModmFsdWUsIHNvcnRf
a2V5cz1UcnVlKQoKZGVmIGZhdWx0X2Nhc2Uoa2luZCk6CiAgICBpZiBraW5kIGluICgicmVhZCIs
ICJvYnNlcnZlZCIsICJkaWFnbm9zdGljIik6CiAgICAgICAgaG9sZGVyID0gRmF1bHRIb2xkZXIo
a2luZCkKICAgICAgICB2YWx1ZSA9IG9ic2VydmUoYXdhcmUsIFNlbnRpbmVsRXJyb3IoU0VOVElO
RUwpLCBob2xkZXIpCiAgICAgICAgYXNzZXJ0IHZhbHVlID09IHsib2JzZXJ2ZWQiOiBraW5kID09
ICJkaWFnbm9zdGljIiwgImRpYWdub3N0aWMiOiBOb25lfQogICAgICAgIGlmIGtpbmQgPT0gImRp
YWdub3N0aWMiOgogICAgICAgICAgICBob2xkZXIubW9kZSA9IE5vbmUKICAgICAgICAgICAgYXNz
ZXJ0IG9ic2VydmUoYXdhcmUsIFZhbHVlRXJyb3IoU0VOVElORUwpLCBob2xkZXIpID09IHZhbHVl
CiAgICBlbGlmIGtpbmQgPT0gImludmFsaWRfc2l0ZSI6CiAgICAgICAgYXNzZXJ0IG9ic2VydmUo
YXdhcmUsIFZhbHVlRXJyb3IoU0VOVElORUwpLCBzaXRlPVNFTlRJTkVMKSA9PSBlbXB0eV9ob2xk
ZXIoKQogICAgZWxzZToKICAgICAgICBob2xkZXIgPSBlbXB0eV9ob2xkZXIoKQogICAgICAgIGF3
YXJlWyJvYnNlcnZlX3VuZXhwZWN0ZWRfZmFpbHVyZSJdIChob2xkZXIsICJzZXJ2ZXIiKQogICAg
ICAgIGFzc2VydCBjaGVja2VkX2hvbGRlcihob2xkZXIpID09IHsib2JzZXJ2ZWQiOiBUcnVlLCAi
ZGlhZ25vc3RpYyI6IHsKICAgICAgICAgICAgInNpdGUiOiAic2VydmVyIiwgImV4Y2VwdGlvbl9j
bGFzcyI6ICJvdGhlciIsICJvd25fZnVuY3Rpb24iOiBOb25lLCAib3duX2xpbmUiOiBOb25lfX0K
CmNsYXNzIE1lbW9yeVdyaXRlKGlvLkJ5dGVzSU8pOgogICAgZGVmIF9faW5pdF9fKHNlbGYsIG9u
Y2U9RmFsc2UpOgogICAgICAgIHN1cGVyKCkuX19pbml0X18oKQogICAgICAgIHNlbGYub25jZSA9
IG9uY2UKICAgIGRlZiB3cml0ZShzZWxmLCByYXcpOgogICAgICAgIGlmIHNlbGYub25jZToKICAg
ICAgICAgICAgc2VsZi5vbmNlID0gRmFsc2UKICAgICAgICAgICAgcmFpc2UgT1NFcnJvcihTRU5U
SU5FTCkKICAgICAgICByZXR1cm4gc3VwZXIoKS53cml0ZShyYXcpCgpkZWYgbWVtb3J5X2hhbmRs
ZXIobmFtZXNwYWNlLCBkZW1vLCBmYXVsdD1Ob25lKToKICAgIGNsYXNzIFNpbmsobmFtZXNwYWNl
WyJIYW5kbGVyIl0pOgogICAgICAgIGRlZiBwYXJzZV9yZXF1ZXN0KHNlbGYpOgogICAgICAgICAg
ICBpZiBmYXVsdCA9PSAicGFyc2VyIjoKICAgICAgICAgICAgICAgIHJhaXNlIFZhbHVlRXJyb3Io
U0VOVElORUwpCiAgICAgICAgICAgIGlmIGZhdWx0ID09ICJzZW5kX2Vycm9yIjoKICAgICAgICAg
ICAgICAgIHNlbGYuc2VuZF9lcnJvcig0MDApCiAgICAgICAgICAgICAgICByZXR1cm4gRmFsc2UK
ICAgICAgICAgICAgcmV0dXJuIHN1cGVyKCkucGFyc2VfcmVxdWVzdCgpCiAgICAgICAgZGVmIHJl
Y29yZF9yZXF1ZXN0X3JlYXNvbihzZWxmLCByZWFzb24pOgogICAgICAgICAgICBpZiBmYXVsdCA9
PSAic2VuZF9lcnJvciIgYW5kIHJlYXNvbiA9PSAiaHR0cF9wYXJzZSI6CiAgICAgICAgICAgICAg
ICByYWlzZSBUeXBlRXJyb3IoU0VOVElORUwpCiAgICAgICAgICAgIHJldHVybiBzdXBlcigpLnJl
Y29yZF9yZXF1ZXN0X3JlYXNvbihyZWFzb24pCiAgICAgICAgZGVmIGNvb2tpZXMoc2VsZik6CiAg
ICAgICAgICAgIGRlbW8uY2FsbHNbImNvb2tpZXMiXSArPSAxCiAgICAgICAgICAgIGlmIGZhdWx0
ID09ICJkaXNwYXRjaCI6CiAgICAgICAgICAgICAgICByYWlzZSBUeXBlRXJyb3IoU0VOVElORUwp
CiAgICAgICAgICAgIHJldHVybiBzdXBlcigpLmNvb2tpZXMoKQogICAgICAgIGRlZiBnZXQoc2Vs
ZiwgdGFyZ2V0KToKICAgICAgICAgICAgZGVtby5jYWxsc1siZ2V0Il0gKz0gMQogICAgICAgICAg
ICByZXR1cm4gc3VwZXIoKS5nZXQodGFyZ2V0KQogICAgICAgIGRlZiByZXBseShzZWxmLCBzdGF0
dXMsIHRleHQsICoqb3B0aW9ucyk6CiAgICAgICAgICAgIHNlbGYucmVzcG9uc2VzLmFwcGVuZCgo
c3RhdHVzLCB0dXBsZShzb3J0ZWQob3B0aW9ucykpKSkKICAgICAgICAgICAgcmV0dXJuIHN1cGVy
KCkucmVwbHkoc3RhdHVzLCB0ZXh0LCAqKm9wdGlvbnMpCiAgICAgICAgZGVmIHNlbmRfcmVzcG9u
c2Uoc2VsZiwgc3RhdHVzLCBtZXNzYWdlPU5vbmUpOgogICAgICAgICAgICBzZWxmLnNlbnRfc3Rh
dHVzZXMuYXBwZW5kKHN0YXR1cykKICAgICAgICBkZWYgc2VuZF9oZWFkZXIoc2VsZiwgbmFtZSwg
dmFsdWUpOgogICAgICAgICAgICBzZWxmLnNlbnRfaGVhZGVycy5hcHBlbmQoKG5hbWUsIHZhbHVl
KSkKICAgICAgICBkZWYgZW5kX2hlYWRlcnMoc2VsZik6CiAgICAgICAgICAgIHNlbGYuaGVhZGVy
X2VuZHMgKz0gMQogICAgaGFuZGxlciA9IG9iamVjdC5fX25ld19fKFNpbmspCiAgICBoYW5kbGVy
LnNlcnZlciA9IFNpbXBsZU5hbWVzcGFjZShkZW1vPWRlbW8pCiAgICBoYW5kbGVyLnJmaWxlID0g
aW8uQnl0ZXNJTyhsZWdhY3lbImJ1aWxkX3dpcmUiXSAobmFtZXNwYWNlKSkKICAgIGhhbmRsZXIu
d2ZpbGUgPSBNZW1vcnlXcml0ZShvbmNlPWZhdWx0ID09ICJyZXBseSIpCiAgICBoYW5kbGVyLnJl
c3BvbnNlcywgaGFuZGxlci5zZW50X3N0YXR1c2VzLCBoYW5kbGVyLnNlbnRfaGVhZGVycyA9IFtd
LCBbXSwgW10KICAgIGhhbmRsZXIuaGVhZGVyX2VuZHMgPSAwCiAgICByZXR1cm4gaGFuZGxlcgoK
ZGVmIG1haW5faGFuZGxlcihuYW1lc3BhY2UsIHJlY29yZCwgZGVtbyk6CiAgICBmdW5jdGlvbiA9
IG5leHQobiBmb3IgbiBpbiBuYW1lc3BhY2VbInNvdXJjZV9hc3QiXS5ib2R5IGlmIGlzaW5zdGFu
Y2UobiwgYXN0LkZ1bmN0aW9uRGVmKSBhbmQgbi5uYW1lID09ICJtYWluIikKICAgIGJsb2NrID0g
bmV4dChuIGZvciBuIGluIGZ1bmN0aW9uLmJvZHkgaWYgaXNpbnN0YW5jZShuLCBhc3QuVHJ5KSkK
ICAgIGhhbmRsZXIgPSBuZXh0KG4gZm9yIG4gaW4gYmxvY2suaGFuZGxlcnMgaWYgaXNpbnN0YW5j
ZShuLnR5cGUsIGFzdC5OYW1lKSBhbmQgbi50eXBlLmlkID09ICJFeGNlcHRpb24iKQogICAgZW52
aXJvbm1lbnQgPSBkaWN0KG5hbWVzcGFjZSwgcmVjb3JkPXJlY29yZCwgZGVtbz1kZW1vLCBzdGFn
ZT0iYXJndW1lbnRzIikKICAgIHRyeToKICAgICAgICByYWlzZSBWYWx1ZUVycm9yKFNFTlRJTkVM
KQogICAgZXhjZXB0IEV4Y2VwdGlvbjoKICAgICAgICBleGVjKGNvbXBpbGUoYXN0Lk1vZHVsZShi
b2R5PWNvcHkuZGVlcGNvcHkoaGFuZGxlci5ib2R5KSwgdHlwZV9pZ25vcmVzPVtdKSwKICAgICAg
ICAgICAgICAgICAgICAgIjxtYWluLWV4Y2VwdC1vbmx5PiIsICJleGVjIiksIGVudmlyb25tZW50
KQoKZGVmIHNpdGVfb3V0Y29tZShuYW1lc3BhY2UsIHNpdGUpOgogICAgZGVtbyA9IG5ld19kZW1v
KG5hbWVzcGFjZSkKICAgIGRlbW8uc3RhZ2UgPSAicmVxdWVzdCIKICAgIGlmIHNpdGUgaW4geyJw
YXJzZXIiLCAiZGlzcGF0Y2giLCAicmVwbHkiLCAic2VuZF9lcnJvciJ9OgogICAgICAgIGhhbmRs
ZXIgPSBtZW1vcnlfaGFuZGxlcihuYW1lc3BhY2UsIGRlbW8sIHNpdGUpCiAgICAgICAgaGFuZGxl
ci5oYW5kbGVfb25lX3JlcXVlc3QoKQogICAgICAgIHJldHVybiBsZWdhY3lbIm5vcm1hbGl6ZWQi
XSAobmFtZXNwYWNlLCBkZW1vLCBoYW5kbGVyKSwgZGVtbwogICAgaWYgc2l0ZSA9PSAic2VydmVy
IjoKICAgICAgICBldmVudHMgPSBbXQogICAgICAgIGRlZiBmaW5pc2gocmVxdWVzdCwgYWRkcmVz
cyk6CiAgICAgICAgICAgIGV2ZW50cy5hcHBlbmQoImZpbmlzaCIpCiAgICAgICAgICAgIHJhaXNl
IEtleUVycm9yKFNFTlRJTkVMKQogICAgICAgIGRlZiBzaHV0ZG93bihyZXF1ZXN0KToKICAgICAg
ICAgICAgZXZlbnRzLmFwcGVuZCgic2h1dGRvd24iKQogICAgICAgIHNlcnZlciA9IFNpbXBsZU5h
bWVzcGFjZShkZW1vPWRlbW8sIGFjdGl2ZT1Ob25lLCBmaW5pc2hfcmVxdWVzdD1maW5pc2gsIHNo
dXRkb3duX3JlcXVlc3Q9c2h1dGRvd24pCiAgICAgICAgdHJ5OgogICAgICAgICAgICBuYW1lc3Bh
Y2VbIkRlbW9TZXJ2ZXIiXS5wcm9jZXNzX3JlcXVlc3Qoc2VydmVyLCBOb25lLCBOb25lKQogICAg
ICAgIGV4Y2VwdCBFeGNlcHRpb246CiAgICAgICAgICAgIG5hbWVzcGFjZVsiRGVtb1NlcnZlciJd
LmhhbmRsZV9lcnJvcihzZXJ2ZXIsIE5vbmUsIE5vbmUpCiAgICAgICAgYXNzZXJ0IHNlcnZlci5h
Y3RpdmUgaXMgTm9uZSBhbmQgZXZlbnRzID09IFsiZmluaXNoIiwgInNodXRkb3duIl0KICAgICAg
ICByZXR1cm4gKGxlZ2FjeVsiZmxvd19zbmFwc2hvdCJdIChkZW1vKSwgZGVtby5zdGFnZSwgZGVt
by5jYWxscywgZXZlbnRzKSwgZGVtbwogICAgcmVjb3JkID0geyJmYWlsdXJlX3RhZyI6IE5vbmUs
ICJmYWlsdXJlX3N0YWdlIjogTm9uZSwKICAgICAgICAgICAgICAidW5leHBlY3RlZF9mYWlsdXJl
X29ic2VydmF0aW9uIjogaG9sZGVyX29mKGRlbW8pIGlmIGhhc2F0dHIoZGVtbywgInVuZXhwZWN0
ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbiIpIGVsc2UgZW1wdHlfaG9sZGVyKCl9CiAgICBtYWluX2hh
bmRsZXIobmFtZXNwYWNlLCByZWNvcmQsIGRlbW8pCiAgICBhc3NlcnQgcmVjb3JkWyJmYWlsdXJl
X3RhZyJdID09ICJ1bmV4cGVjdGVkX2ZhaWx1cmUiIGFuZCByZWNvcmRbImZhaWx1cmVfc3RhZ2Ui
XSA9PSAicmVxdWVzdCIKICAgIHJldHVybiB7azogdiBmb3IgaywgdiBpbiByZWNvcmQuaXRlbXMo
KSBpZiBrICE9ICJ1bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24ifSwgZGVtbwoKZGVmIHNp
dGVfY2FzZShzaXRlKToKICAgIGJlZm9yZSwgaWdub3JlZCA9IHNpdGVfb3V0Y29tZShyZWZlcmVu
Y2UsIHNpdGUpCiAgICBhZnRlciwgZGVtbyA9IHNpdGVfb3V0Y29tZShhd2FyZSwgc2l0ZSkKICAg
IGFzc2VydCBiZWZvcmUgPT0gYWZ0ZXIKICAgIHZhbHVlID0gY2hlY2tlZF9ob2xkZXIoaG9sZGVy
X29mKGRlbW8pKQogICAgbGFiZWxzID0geyJwYXJzZXIiOiAoImhhbmRsZXIiLCAiVmFsdWVFcnJv
ciIsICJIYW5kbGVyLmhhbmRsZV9vbmVfcmVxdWVzdCIpLAogICAgICAgICJkaXNwYXRjaCI6ICgi
aGFuZGxlciIsICJUeXBlRXJyb3IiLCAiSGFuZGxlci5nZXQiKSwKICAgICAgICAicmVwbHkiOiAo
ImhhbmRsZXIiLCAiT1NFcnJvciIsICJIYW5kbGVyLnJlcGx5IiksCiAgICAgICAgInNlbmRfZXJy
b3IiOiAoImhhbmRsZXIiLCAiVHlwZUVycm9yIiwgIkhhbmRsZXIuc2VuZF9lcnJvciIpLAogICAg
ICAgICJzZXJ2ZXIiOiAoInNlcnZlciIsICJLZXlFcnJvciIsICJEZW1vU2VydmVyLnByb2Nlc3Nf
cmVxdWVzdCIpLAogICAgICAgICJtYWluIjogKCJtYWluIiwgIlZhbHVlRXJyb3IiLCBOb25lKX0K
ICAgIHNpdGVfbGFiZWwsIGNsYXNzX2xhYmVsLCBvd24gPSBsYWJlbHNbc2l0ZV0KICAgIGFzc2Vy
dCB2YWx1ZVsib2JzZXJ2ZWQiXSBhbmQgdmFsdWVbImRpYWdub3N0aWMiXVsic2l0ZSJdID09IHNp
dGVfbGFiZWwKICAgIGFzc2VydCB2YWx1ZVsiZGlhZ25vc3RpYyJdWyJleGNlcHRpb25fY2xhc3Mi
XSA9PSBjbGFzc19sYWJlbAogICAgYXNzZXJ0IHZhbHVlWyJkaWFnbm9zdGljIl1bIm93bl9mdW5j
dGlvbiJdID09IG93bgoKZGVmIGZpcnN0X2Nhc2Uoa2luZCk6CiAgICBpZiBraW5kID09ICJkaXJl
Y3QiOgogICAgICAgIGhvbGRlciA9IGVtcHR5X2hvbGRlcigpCiAgICAgICAgZmlyc3QgPSBvYnNl
cnZlKGF3YXJlLCBTZW50aW5lbEVycm9yKFNFTlRJTkVMKSwgaG9sZGVyLCBuYXRpdmU9bmF0aXZl
X3RyYWNlKDEpKQogICAgICAgIGFzc2VydCBvYnNlcnZlKGF3YXJlLCBWYWx1ZUVycm9yKFNFTlRJ
TkVMKSwgaG9sZGVyLCAic2VydmVyIikgPT0gZmlyc3QKICAgICAgICByZXR1cm4KICAgIG91dGNv
bWVzID0gW10KICAgIGZvciBuYW1lc3BhY2UgaW4gKHJlZmVyZW5jZSwgYXdhcmUpOgogICAgICAg
IGRlbW8gPSBuZXdfZGVtbyhuYW1lc3BhY2UpCiAgICAgICAgaGFuZGxlciA9IG1lbW9yeV9oYW5k
bGVyKG5hbWVzcGFjZSwgZGVtbywgInBhcnNlciIpCiAgICAgICAgaGFuZGxlci53ZmlsZS5vbmNl
ID0gVHJ1ZQogICAgICAgIHRyeToKICAgICAgICAgICAgaGFuZGxlci5oYW5kbGVfb25lX3JlcXVl
c3QoKQogICAgICAgIGV4Y2VwdCBPU0Vycm9yOgogICAgICAgICAgICAjIFRoZSBvcmlnaW5hbCBy
ZXBseSBmYWlsdXJlIGVzY2FwZXM7IHRoZSBleGlzdGluZyBzZXJ2ZXIgc2luayBydW5zLgogICAg
ICAgICAgICBuYW1lc3BhY2VbIkRlbW9TZXJ2ZXIiXS5oYW5kbGVfZXJyb3IoU2ltcGxlTmFtZXNw
YWNlKGRlbW89ZGVtbyksIE5vbmUsIE5vbmUpCiAgICAgICAgZWxzZToKICAgICAgICAgICAgcmFp
c2UgQXNzZXJ0aW9uRXJyb3IKICAgICAgICBhc3NlcnQgZGVtby5mYWlsdXJlID09ICJ1bmV4cGVj
dGVkX2ZhaWx1cmUiIGFuZCBkZW1vLmRvbmUKICAgICAgICBpZiBuYW1lc3BhY2UgaXMgYXdhcmU6
CiAgICAgICAgICAgIHZhbHVlID0gY2hlY2tlZF9ob2xkZXIoaG9sZGVyX29mKGRlbW8pKQogICAg
ICAgICAgICBhc3NlcnQgdmFsdWVbImRpYWdub3N0aWMiXVsiZXhjZXB0aW9uX2NsYXNzIl0gPT0g
IlZhbHVlRXJyb3IiCiAgICAgICAgICAgIGFzc2VydCB2YWx1ZVsiZGlhZ25vc3RpYyJdWyJzaXRl
Il0gPT0gImhhbmRsZXIiCiAgICAgICAgb3V0Y29tZXMuYXBwZW5kKGxlZ2FjeVsibm9ybWFsaXpl
ZCJdIChuYW1lc3BhY2UsIGRlbW8sIGhhbmRsZXIpKQogICAgYXNzZXJ0IG91dGNvbWVzWzBdID09
IG91dGNvbWVzWzFdCgpkZWYga25vd25fY2FzZShraW5kKToKICAgIG91dGNvbWVzID0gW10KICAg
IGZvciBuYW1lc3BhY2UgaW4gKHJlZmVyZW5jZSwgYXdhcmUpOgogICAgICAgIGRlbW8gPSBuZXdf
ZGVtbyhuYW1lc3BhY2UpCiAgICAgICAgaGFuZGxlciA9IG1lbW9yeV9oYW5kbGVyKG5hbWVzcGFj
ZSwgZGVtbykKICAgICAgICBpZiBraW5kID09ICJmYWlsdXJlIjoKICAgICAgICAgICAgaGFuZGxl
ci5yZmlsZSA9IGlvLkJ5dGVzSU8obGVnYWN5WyJidWlsZF93aXJlIl0gKG5hbWVzcGFjZSwgaG9z
dD0id3JvbmciKSkKICAgICAgICBlbGlmIGtpbmQgPT0gImhhbHQiOgogICAgICAgICAgICBkZW1v
LmJ1ZGdldC5mYWlsX3Jlc3BvbnNlID0gVHJ1ZQogICAgICAgIHRyeToKICAgICAgICAgICAgaGFu
ZGxlci5oYW5kbGVfb25lX3JlcXVlc3QoKQogICAgICAgIGV4Y2VwdCBuYW1lc3BhY2VbIkhhbHQi
XSBhcyBlcnJvcjoKICAgICAgICAgICAgYXNzZXJ0IGtpbmQgPT0gImhhbHQiIGFuZCBlcnJvci50
YWcgPT0gInJlc3BvbnNlX3RpbWVvdXQiCiAgICAgICAgZWxzZToKICAgICAgICAgICAgYXNzZXJ0
IGtpbmQgIT0gImhhbHQiCiAgICAgICAgaWYgbmFtZXNwYWNlIGlzIGF3YXJlOgogICAgICAgICAg
ICBhc3NlcnQgY2hlY2tlZF9ob2xkZXIoaG9sZGVyX29mKGRlbW8pKSA9PSBlbXB0eV9ob2xkZXIo
KQogICAgICAgIG91dGNvbWVzLmFwcGVuZChsZWdhY3lbIm5vcm1hbGl6ZWQiXSAobmFtZXNwYWNl
LCBkZW1vLCBoYW5kbGVyKSkKICAgIGFzc2VydCBvdXRjb21lc1swXSA9PSBvdXRjb21lc1sxXQoK
ZGVmIGZhbGxiYWNrX2Nhc2UobW9kZSwgc2l0ZSk6CiAgICBpZiBtb2RlID09ICJsZWdhY3kiOgog
ICAgICAgIG5hbWVzcGFjZSA9IGxlZ2FjeVsibG9hZF9zZWxlY3RlZCJdIChTT1VSQ0VfVEVYVCwg
ImNhbmRpZGF0ZSIpCiAgICAgICAgZGVtbyA9IGxlZ2FjeVsibWFrZV9kZW1vIl0gKG5hbWVzcGFj
ZSkKICAgICAgICBkZWwgZGVtby51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24KICAgICAg
ICBuYW1lc3BhY2VbImhhbmRsZV9lcnJvciJdIChTaW1wbGVOYW1lc3BhY2UoZGVtbz1kZW1vKSwg
Tm9uZSwgTm9uZSkKICAgICAgICBhc3NlcnQgZGVtby5mYWlsdXJlID09ICJ1bmV4cGVjdGVkX2Zh
aWx1cmUiIGFuZCBkZW1vLmRvbmUKICAgICAgICBhc3NlcnQgIm9ic2VydmVfdW5leHBlY3RlZF9m
YWlsdXJlIiBub3QgaW4gbmFtZXNwYWNlCiAgICAgICAgcmV0dXJuCiAgICBuYW1lc3BhY2UgPSBz
ZWxlY3RlZF9uYW1lc3BhY2UoU09VUkNFX1RFWFQpCiAgICBpZiBtb2RlID09ICJhYnNlbnQiOgog
ICAgICAgIGRlbCBuYW1lc3BhY2VbIm9ic2VydmVfdW5leHBlY3RlZF9mYWlsdXJlIl0KICAgIGVs
aWYgbW9kZSA9PSAiZ3VhcmQiOgogICAgICAgIGRlZiBmYWlsKCphcmdzKToKICAgICAgICAgICAg
cmFpc2UgR3VhcmRGYWlsdXJlCiAgICAgICAgbmFtZXNwYWNlWyJvYnNlcnZlX3VuZXhwZWN0ZWRf
ZmFpbHVyZSJdID0gZmFpbAogICAgZGVtbyA9IG5ld19kZW1vKG5hbWVzcGFjZSkKICAgIGRlbW8u
c3RhZ2UgPSAicmVxdWVzdCIKICAgIGlmIG1vZGUgPT0gImZpZWxkIjoKICAgICAgICBkZWwgZGVt
by51bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRpb24KICAgIGlmIHNpdGUgPT0gImhhbmRsZXIi
OgogICAgICAgIGhhbmRsZXIgPSBtZW1vcnlfaGFuZGxlcihuYW1lc3BhY2UsIGRlbW8sICJwYXJz
ZXIiKQogICAgICAgIGhhbmRsZXIuaGFuZGxlX29uZV9yZXF1ZXN0KCkKICAgICAgICBhc3NlcnQg
ZGVtby5mYWlsdXJlID09ICJ1bmV4cGVjdGVkX2ZhaWx1cmUiIGFuZCBkZW1vLmRvbmUKICAgICAg
ICBhc3NlcnQgaGFuZGxlci5yZXNwb25zZXMgPT0gWyg0MDAsICgpKV0KICAgIGVsaWYgc2l0ZSA9
PSAic2VydmVyIjoKICAgICAgICBuYW1lc3BhY2VbIkRlbW9TZXJ2ZXIiXS5oYW5kbGVfZXJyb3Io
U2ltcGxlTmFtZXNwYWNlKGRlbW89ZGVtbyksIE5vbmUsIE5vbmUpCiAgICAgICAgYXNzZXJ0IGRl
bW8uZmFpbHVyZSA9PSAidW5leHBlY3RlZF9mYWlsdXJlIiBhbmQgZGVtby5kb25lCiAgICBlbHNl
OgogICAgICAgIHJlY29yZCA9IHsiZmFpbHVyZV90YWciOiBOb25lLCAiZmFpbHVyZV9zdGFnZSI6
IE5vbmV9CiAgICAgICAgaWYgbW9kZSAhPSAiZmllbGQiOgogICAgICAgICAgICByZWNvcmRbInVu
ZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbiJdID0gaG9sZGVyX29mKGRlbW8pCiAgICAgICAg
bWFpbl9oYW5kbGVyKG5hbWVzcGFjZSwgcmVjb3JkLCBkZW1vKQogICAgICAgIGFzc2VydCByZWNv
cmRbImZhaWx1cmVfdGFnIl0gPT0gInVuZXhwZWN0ZWRfZmFpbHVyZSIgYW5kIHJlY29yZFsiZmFp
bHVyZV9zdGFnZSJdID09ICJyZXF1ZXN0IgogICAgaWYgbW9kZSAhPSAiZmllbGQiOgogICAgICAg
IGFzc2VydCBjaGVja2VkX2hvbGRlcihob2xkZXJfb2YoZGVtbykpID09IGVtcHR5X2hvbGRlcigp
CgpkZWYgd3JpdGVyX3RyaWFsKG5hbWVzcGFjZSwgZmFpbCk6CiAgICB0cmVlID0gbmFtZXNwYWNl
WyJzb3VyY2VfYXN0Il0KICAgIG1haW4gPSBuZXh0KG4gZm9yIG4gaW4gdHJlZS5ib2R5IGlmIGlz
aW5zdGFuY2UobiwgYXN0LkZ1bmN0aW9uRGVmKSBhbmQgbi5uYW1lID09ICJtYWluIikKICAgIHN0
YXJ0ID0gbmV4dChpIGZvciBpLCBuIGluIGVudW1lcmF0ZShtYWluLmJvZHkpIGlmIGlzaW5zdGFu
Y2UobiwgYXN0LkFzc2lnbikKICAgICAgICAgICAgICAgICBhbmQgYW55KGlzaW5zdGFuY2UodCwg
YXN0Lk5hbWUpIGFuZCB0LmlkID09ICJ3cml0dGVuIiBmb3IgdCBpbiBuLnRhcmdldHMpKQogICAg
IyBQcmVzZXJ2ZSB0aGUgZXhhY3Qgd3JpdGVyL3ByaW50L3JldHVybiB0YWlsIGluc2lkZSBhIGZy
ZXNoIHdyYXBwZXIsIG5ldmVyIG1haW4uCiAgICB3cmFwcGVyID0gY29weS5kZWVwY29weShtYWlu
KQogICAgd3JhcHBlci5uYW1lID0gInNlbGVjdGVkX2V2aWRlbmNlX3RhaWwiCiAgICB3cmFwcGVy
LmJvZHkgPSBjb3B5LmRlZXBjb3B5KG1haW4uYm9keVtzdGFydDpdKQogICAgc2luayA9IGlvLkJ5
dGVzSU8oKQogICAgY2xhc3MgV3JpdGVyOgogICAgICAgIGRlZiBfX2VudGVyX18oc2VsZik6CiAg
ICAgICAgICAgIHJldHVybiBzZWxmCiAgICAgICAgZGVmIF9fZXhpdF9fKHNlbGYsICphcmdzKToK
ICAgICAgICAgICAgcmV0dXJuIEZhbHNlCiAgICAgICAgZGVmIHdyaXRlKHNlbGYsIHJhdyk6CiAg
ICAgICAgICAgIGlmIGZhaWw6CiAgICAgICAgICAgICAgICByYWlzZSBPU0Vycm9yKFNFTlRJTkVM
KQogICAgICAgICAgICByZXR1cm4gc2luay53cml0ZShyYXcpCiAgICAgICAgZGVmIGZsdXNoKHNl
bGYpOgogICAgICAgICAgICBwYXNzCiAgICAgICAgZGVmIGZpbGVubyhzZWxmKToKICAgICAgICAg
ICAgcmV0dXJuIDEKICAgIHN0ZG91dCA9IGlvLlN0cmluZ0lPKCkKICAgIGRlZiBzYWZlX3ByaW50
KHJhdywgKipvcHRpb25zKToKICAgICAgICBzdGRvdXQud3JpdGUocmF3ICsgIlxuIikKICAgIGhv
bGRlciA9IGVtcHR5X2hvbGRlcigpCiAgICBpZiAib2JzZXJ2ZV91bmV4cGVjdGVkX2ZhaWx1cmUi
IGluIG5hbWVzcGFjZToKICAgICAgICBvYnNlcnZlKG5hbWVzcGFjZSwgU2VudGluZWxFcnJvcihT
RU5USU5FTCksIGhvbGRlcikKICAgIHJlY29yZCA9IHsicmVzdWx0IjogImZhaWxlZCIsICJmYWls
dXJlX3RhZyI6ICJ1bmV4cGVjdGVkX2ZhaWx1cmUiLAogICAgICAgICAgICAgICJmYWlsdXJlX3N0
YWdlIjogInJlcXVlc3QiLCAiY2xlYW51cCI6IHt9LAogICAgICAgICAgICAgICJ1bmV4cGVjdGVk
X2ZhaWx1cmVfb2JzZXJ2YXRpb24iOiBob2xkZXJ9CiAgICBlbnZpcm9ubWVudCA9IGRpY3QobmFt
ZXNwYWNlLCBvcz1TaW1wbGVOYW1lc3BhY2UoZmRvcGVuPWxhbWJkYSAqYTogV3JpdGVyKCksCiAg
ICAgICAgZnN5bmM9bGFtYmRhICphOiBOb25lLCBjbG9zZT1sYW1iZGEgKmE6IE5vbmUpLCBqc29u
PWpzb24sIHByaW50PXNhZmVfcHJpbnQsCiAgICAgICAgZXZpZGVuY2VfZmQ9MSwgcmVjb3JkPXJl
Y29yZCkKICAgIGV4ZWMoY29tcGlsZShhc3QuTW9kdWxlKGJvZHk9W3dyYXBwZXJdLCB0eXBlX2ln
bm9yZXM9W10pLCAiPGV2aWRlbmNlLXRhaWwtb25seT4iLCAiZXhlYyIpLCBlbnZpcm9ubWVudCkK
ICAgIGFzc2VydCBlbnZpcm9ubWVudFsic2VsZWN0ZWRfZXZpZGVuY2VfdGFpbCJdICgpID09IDEK
ICAgIHB1YmxpYyA9IGpzb24ubG9hZHMoc3Rkb3V0LmdldHZhbHVlKCkpCiAgICBhc3NlcnQgc2V0
KHB1YmxpYykgPT0geyJyZXN1bHQiLCAiZmFpbHVyZV90YWciLCAiY2xlYW51cF9mYWlsdXJlX3Rh
ZyIsCiAgICAgICAgICAgICAgICAgICAgICAgICAgImV2aWRlbmNlX2ZhaWx1cmVfdGFnIiwgImV2
aWRlbmNlX3dyaXR0ZW4ifQogICAgYXNzZXJ0ICJ1bmV4cGVjdGVkX2ZhaWx1cmVfb2JzZXJ2YXRp
b24iIG5vdCBpbiBwdWJsaWMKICAgIGFzc2VydCBTRU5USU5FTCBub3QgaW4gc3Rkb3V0LmdldHZh
bHVlKCkgYW5kIFNFTlRJTkVMLmVuY29kZSgpIG5vdCBpbiBzaW5rLmdldHZhbHVlKCkKICAgIGFz
c2VydCBwdWJsaWNbImZhaWx1cmVfdGFnIl0gPT0gInVuZXhwZWN0ZWRfZmFpbHVyZSIgYW5kIHB1
YmxpY1sicmVzdWx0Il0gPT0gImZhaWxlZCIKICAgIGFzc2VydCBwdWJsaWNbImV2aWRlbmNlX3dy
aXR0ZW4iXSBpcyAobm90IGZhaWwpCiAgICBpZiBmYWlsOgogICAgICAgIGFzc2VydCBwdWJsaWNb
ImV2aWRlbmNlX2ZhaWx1cmVfdGFnIl0gPT0gImV2aWRlbmNlX3dyaXRlX2ZhaWxlZCIKICAgICAg
ICBhc3NlcnQgcmVjb3JkWyJmYWlsdXJlX3N0YWdlIl0gPT0gInJlcXVlc3QiCiAgICBlbHNlOgog
ICAgICAgIGFzc2VydCBjaGVja2VkX2hvbGRlcihqc29uLmxvYWRzKHNpbmsuZ2V0dmFsdWUoKSlb
InVuZXhwZWN0ZWRfZmFpbHVyZV9vYnNlcnZhdGlvbiJdKSA9PSBob2xkZXIKICAgIHJldHVybiBw
dWJsaWMKCmRlZiBwcml2YWN5X2Nhc2Uoa2luZCk6CiAgICBiZWZvcmUgPSBkaWN0KHRyYXBfY291
bnRzKQogICAgaWYga2luZCA9PSAic2VudGluZWwiOgogICAgICAgIHZhbHVlID0gb2JzZXJ2ZShh
d2FyZSwgU2VudGluZWxFcnJvcihTRU5USU5FTCkpCiAgICAgICAgYXNzZXJ0IHZhbHVlWyJkaWFn
bm9zdGljIl1bImV4Y2VwdGlvbl9jbGFzcyJdID09ICJvdGhlciIKICAgIGVsc2U6CiAgICAgICAg
YXNzZXJ0IHdyaXRlcl90cmlhbChyZWZlcmVuY2UsIGtpbmQgPT0gIndyaXRlcl9mYWlsdXJlIikg
PT0gd3JpdGVyX3RyaWFsKGF3YXJlLCBraW5kID09ICJ3cml0ZXJfZmFpbHVyZSIpCiAgICBhc3Nl
cnQgdHJhcF9jb3VudHMgPT0gYmVmb3JlCgpDQVNFUyA9ICgKICAgICgiY2xhc3NfYXR0cmlidXRl
IiwgImNsYXNzZXMiLCAiY2xhc3MiLCAoIkF0dHJpYnV0ZUVycm9yIiwpKSwKICAgICgiY2xhc3Nf
dHlwZSIsICJjbGFzc2VzIiwgImNsYXNzIiwgKCJUeXBlRXJyb3IiLCkpLAogICAgKCJjbGFzc192
YWx1ZSIsICJjbGFzc2VzIiwgImNsYXNzIiwgKCJWYWx1ZUVycm9yIiwpKSwKICAgICgiY2xhc3Nf
a2V5IiwgImNsYXNzZXMiLCAiY2xhc3MiLCAoIktleUVycm9yIiwpKSwKICAgICgiY2xhc3Nfb3Mi
LCAiY2xhc3NlcyIsICJjbGFzcyIsICgiT1NFcnJvciIsKSksCiAgICAoImNsYXNzX2Jyb2tlbl9w
aXBlIiwgImNsYXNzZXMiLCAiY2xhc3MiLCAoIkJyb2tlblBpcGVFcnJvciIsKSksCiAgICAoImNs
YXNzX2Nvbm5lY3Rpb25fcmVzZXQiLCAiY2xhc3NlcyIsICJjbGFzcyIsICgiQ29ubmVjdGlvblJl
c2V0RXJyb3IiLCkpLAogICAgKCJjbGFzc190aW1lb3V0IiwgImNsYXNzZXMiLCAiY2xhc3MiLCAo
IlRpbWVvdXRFcnJvciIsKSksCiAgICAoImNsYXNzX3ZhbHVlX3N1YmNsYXNzIiwgImNsYXNzZXMi
LCAiY2xhc3MiLCAoIlZhbHVlU3ViY2xhc3MiLCkpLAogICAgKCJjbGFzc19vc19zdWJjbGFzcyIs
ICJjbGFzc2VzIiwgImNsYXNzIiwgKCJPU1N1YmNsYXNzIiwpKSwKICAgICgiY2xhc3NfcHJpdmF0
ZV9vdGhlciIsICJjbGFzc2VzIiwgImNsYXNzIiwgKCJTZW50aW5lbEVycm9yIiwpKSwKICAgICgi
dHJhY2VfbGluZTEiLCAidHJhY2UiLCAidHJhY2UiLCAoImxpbmUxIiwpKSwKICAgICgidHJhY2Vf
bGluZTEwMjQiLCAidHJhY2UiLCAidHJhY2UiLCAoImxpbmUxMDI0IiwpKSwKICAgICgidHJhY2Vf
bGluZTAiLCAidHJhY2UiLCAidHJhY2UiLCAoImxpbmUwIiwpKSwKICAgICgidHJhY2VfbGluZTEw
MjUiLCAidHJhY2UiLCAidHJhY2UiLCAoImxpbmUxMDI1IiwpKSwKICAgICgidHJhY2VfNjQiLCAi
dHJhY2UiLCAidHJhY2UiLCAoImxpbmtzNjQiLCkpLAogICAgKCJ0cmFjZV82NSIsICJ0cmFjZSIs
ICJ0cmFjZSIsICgibGlua3M2NSIsKSksCiAgICAoInRyYWNlX3VudHJ1c3RlZCIsICJ0cmFjZSIs
ICJ0cmFjZSIsICgidW50cnVzdGVkIiwpKSwKICAgICgidHJhY2VfZXF1YWxfY2xvbmUiLCAidHJh
Y2UiLCAidHJhY2UiLCAoImNsb25lZCIsKSksCiAgICAoImRlc2NyaXB0b3JfY2hhbmdpbmdfZ2V0
dGVyIiwgImRlc2NyaXB0b3IiLCAiZGVzY3JpcHRvciIsICgiY2hhbmdpbmciLCkpLAogICAgKCJk
ZXNjcmlwdG9yX3JhaXNpbmdfZ2V0dGVyIiwgImRlc2NyaXB0b3IiLCAiZGVzY3JpcHRvciIsICgi
cmFpc2luZyIsKSksCiAgICAoImZhdWx0X2hvbGRlcl9yZWFkIiwgImZhdWx0cyIsICJmYXVsdCIs
ICgicmVhZCIsKSksCiAgICAoImZhdWx0X2xhdGNoX3dyaXRlIiwgImZhdWx0cyIsICJmYXVsdCIs
ICgib2JzZXJ2ZWQiLCkpLAogICAgKCJmYXVsdF9kaWFnbm9zdGljX3dyaXRlIiwgImZhdWx0cyIs
ICJmYXVsdCIsICgiZGlhZ25vc3RpYyIsKSksCiAgICAoImZhdWx0X2ludmFsaWRfc2l0ZSIsICJm
YXVsdHMiLCAiZmF1bHQiLCAoImludmFsaWRfc2l0ZSIsKSksCiAgICAoImZhdWx0X25vX2FjdGl2
ZV9leGNlcHRpb24iLCAiZmF1bHRzIiwgImZhdWx0IiwgKCJub19leGNlcHRpb24iLCkpLAogICAg
KCJmaXJzdF9kaXJlY3QiLCAiZmlyc3QiLCAiZmlyc3QiLCAoImRpcmVjdCIsKSksCiAgICAoImZp
cnN0X3NlY29uZF9yZXBseV9mYWlsdXJlIiwgImZpcnN0IiwgImZpcnN0IiwgKCJyZXBseSIsKSks
CiAgICAoInNpdGVfcGFyc2VyIiwgInNpdGVzIiwgInNpdGUiLCAoInBhcnNlciIsKSksCiAgICAo
InNpdGVfZGlzcGF0Y2giLCAic2l0ZXMiLCAic2l0ZSIsICgiZGlzcGF0Y2giLCkpLAogICAgKCJz
aXRlX3JlcGx5IiwgInNpdGVzIiwgInNpdGUiLCAoInJlcGx5IiwpKSwKICAgICgic2l0ZV9zZW5k
X2Vycm9yIiwgInNpdGVzIiwgInNpdGUiLCAoInNlbmRfZXJyb3IiLCkpLAogICAgKCJzaXRlX3Nl
cnZlciIsICJzaXRlcyIsICJzaXRlIiwgKCJzZXJ2ZXIiLCkpLAogICAgKCJzaXRlX21haW5fYmxv
Y2siLCAic2l0ZXMiLCAic2l0ZSIsICgibWFpbiIsKSksCiAgICAoImtub3duX2ZhaWx1cmUiLCAi
a25vd24iLCAia25vd24iLCAoImZhaWx1cmUiLCkpLAogICAgKCJrbm93bl9oYWx0IiwgImtub3du
IiwgImtub3duIiwgKCJoYWx0IiwpKSwKICAgICgia25vd25fc3VjY2VzcyIsICJrbm93biIsICJr
bm93biIsICgic3VjY2VzcyIsKSksCiAgICAoImZhbGxiYWNrX2Fic2VudF9oYW5kbGVyIiwgImZh
bGxiYWNrIiwgImZhbGxiYWNrIiwgKCJhYnNlbnQiLCAiaGFuZGxlciIpKSwKICAgICgiZmFsbGJh
Y2tfYWJzZW50X3NlcnZlciIsICJmYWxsYmFjayIsICJmYWxsYmFjayIsICgiYWJzZW50IiwgInNl
cnZlciIpKSwKICAgICgiZmFsbGJhY2tfYWJzZW50X21haW4iLCAiZmFsbGJhY2siLCAiZmFsbGJh
Y2siLCAoImFic2VudCIsICJtYWluIikpLAogICAgKCJmYWxsYmFja19maWVsZF9oYW5kbGVyIiwg
ImZhbGxiYWNrIiwgImZhbGxiYWNrIiwgKCJmaWVsZCIsICJoYW5kbGVyIikpLAogICAgKCJmYWxs
YmFja19maWVsZF9zZXJ2ZXIiLCAiZmFsbGJhY2siLCAiZmFsbGJhY2siLCAoImZpZWxkIiwgInNl
cnZlciIpKSwKICAgICgiZmFsbGJhY2tfZmllbGRfbWFpbiIsICJmYWxsYmFjayIsICJmYWxsYmFj
ayIsICgiZmllbGQiLCAibWFpbiIpKSwKICAgICgiZmFsbGJhY2tfYmFzZWV4Y2VwdGlvbl9oYW5k
bGVyIiwgImZhbGxiYWNrIiwgImZhbGxiYWNrIiwgKCJndWFyZCIsICJoYW5kbGVyIikpLAogICAg
KCJmYWxsYmFja19iYXNlZXhjZXB0aW9uX3NlcnZlciIsICJmYWxsYmFjayIsICJmYWxsYmFjayIs
ICgiZ3VhcmQiLCAic2VydmVyIikpLAogICAgKCJmYWxsYmFja19iYXNlZXhjZXB0aW9uX21haW4i
LCAiZmFsbGJhY2siLCAiZmFsbGJhY2siLCAoImd1YXJkIiwgIm1haW4iKSksCiAgICAoImZhbGxi
YWNrX2xlZ2FjeV9zZWxlY3Rvcl9maWVsZCIsICJmYWxsYmFjayIsICJmYWxsYmFjayIsICgibGVn
YWN5IiwgInNlcnZlciIpKSwKICAgICgicHJpdmFjeV9zdHJfcmVwciIsICJwcml2YWN5IiwgInBy
aXZhY3kiLCAoInNlbnRpbmVsIiwpKSwKICAgICgicHJpdmFjeV93cml0ZXJfc3VjY2VzcyIsICJw
cml2YWN5IiwgInByaXZhY3kiLCAoIndyaXRlcl9zdWNjZXNzIiwpKSwKICAgICgicHJpdmFjeV93
cml0ZXJfZmFpbHVyZSIsICJwcml2YWN5IiwgInByaXZhY3kiLCAoIndyaXRlcl9mYWlsdXJlIiwp
KSwKKQpSVU5ORVJTID0geyJjbGFzcyI6IGNsYXNzX2Nhc2UsICJ0cmFjZSI6IHRyYWNlX2Nhc2Us
ICJkZXNjcmlwdG9yIjogZGVzY3JpcHRvcl9jYXNlLCAiZmF1bHQiOiBmYXVsdF9jYXNlLAogICAg
ImZpcnN0IjogZmlyc3RfY2FzZSwgInNpdGUiOiBzaXRlX2Nhc2UsICJrbm93biI6IGtub3duX2Nh
c2UsCiAgICAiZmFsbGJhY2siOiBmYWxsYmFja19jYXNlLCAicHJpdmFjeSI6IHByaXZhY3lfY2Fz
ZX0KCnN0YXJ0ZWQgPSB0aW1lLm1vbm90b25pYygpCmxlZ2FjeSA9IE5vbmUKb2JzZXJ2ZXJfY29t
cGxldGVkID0gMApvYnNlcnZlcl9ncm91cHMgPSB7a2V5OiAwIGZvciBrZXkgaW4gRVhQRUNURURf
T0JTRVJWRVJ9CnRyYXBfY291bnRzID0geyJzdHIiOiAwLCAicmVwciI6IDAsICJ0cmFjZWJhY2si
OiAwfQpuYXRpdmVfZnJhbWVzID0ge30KbmF0aXZlX2NhcHR1cmVzID0geyJ0cnVzdGVkIjogMCwg
ImNsb25lZCI6IDAsICJ1bnRydXN0ZWQiOiAwfQpjYXNlID0gInNvdXJjZV9hcmNoaXZlIgpmYWls
dXJlID0gTm9uZQpyZXN1bHQgPSAiZmFpbGVkIgoKZGVmIGRlYWRsaW5lKHNpZ251bSwgZnJhbWUp
OgogICAgcmFpc2UgRW52ZWxvcGVEZWFkbGluZQoKdHJ5OgogICAgc2lnbmFsLnNpZ25hbChzaWdu
YWwuU0lHQUxSTSwgZGVhZGxpbmUpCiAgICBzaWduYWwuc2V0aXRpbWVyKHNpZ25hbC5JVElNRVJf
UkVBTCwgMzApCiAgICBhc3NlcnQgaGFzaGxpYi5zaGEyNTYoU09VUkNFX1RFWFQuZW5jb2RlKCkp
LmhleGRpZ2VzdCgpID09IFNPVVJDRV9IQVNICiAgICBhc3NlcnQgaGFzaGxpYi5zaGEyNTYoVU5D
SEFOR0VEX1RFWFQuZW5jb2RlKCkpLmhleGRpZ2VzdCgpID09IFVOQ0hBTkdFRF9IQVNICiAgICBh
c3NlcnQgaGFzaGxpYi5zaGEyNTYoTEVHQUNZX0JBU0VMSU5FX1RFWFQuZW5jb2RlKCkpLmhleGRp
Z2VzdCgpID09IEJBU0VMSU5FX0hBU0gKICAgIGFzc2VydCBoYXNobGliLnNoYTI1NihMRUdBQ1lf
TE9HSUMuZW5jb2RlKCkpLmhleGRpZ2VzdCgpID09IExFR0FDWV9MT0dJQ19IQVNICiAgICBvbGRf
YXJjaGl2ZSA9ICgiQkFTRUxJTkVfVEVYVD0iICsganNvbi5kdW1wcyhMRUdBQ1lfQkFTRUxJTkVf
VEVYVCwgZW5zdXJlX2FzY2lpPVRydWUpICsgIlxuIgogICAgICAgICAgICAgICAgICAgKyAiU09V
UkNFX1RFWFQ9IiArIGpzb24uZHVtcHMoVU5DSEFOR0VEX1RFWFQsIGVuc3VyZV9hc2NpaT1UcnVl
KSArICJcbiIgKyBMRUdBQ1lfTE9HSUMpCiAgICBhc3NlcnQgbGVuKG9sZF9hcmNoaXZlLmVuY29k
ZSgpKSA9PSA5MTE2MwogICAgYXNzZXJ0IGhhc2hsaWIuc2hhMjU2KG9sZF9hcmNoaXZlLmVuY29k
ZSgpKS5oZXhkaWdlc3QoKSA9PSAiY2U3OWFlYjk5ZmFmYWFlYjI2NWQ2YjdiZmY1NWY0MTVlOTE4
MjVkODNlOWM2NTBlZWQ0MTI4MjVhNWE5NTI1ZiIKICAgIGFzc2VydCBsZW4oQ0FTRVMpID09IDUw
IGFuZCBsZW4oe24gZm9yIG4sIGcsIGssIGEgaW4gQ0FTRVN9KSA9PSA1MAogICAgcGxhbm5lZCA9
IHtrZXk6IHN1bShncm91cCA9PSBrZXkgZm9yIG5hbWUsIGdyb3VwLCBraW5kLCBhcmdzIGluIENB
U0VTKSBmb3Iga2V5IGluIEVYUEVDVEVEX09CU0VSVkVSfQogICAgYXNzZXJ0IHBsYW5uZWQgPT0g
RVhQRUNURURfT0JTRVJWRVIKICAgIGxlZ2FjeSA9IGxlZ2FjeV9uYW1lc3BhY2UoKQogICAgY2Fz
ZSA9ICJsZWdhY3lfNzgiCiAgICBsZWdhY3lbInJ1biJdICgpCiAgICBhc3NlcnQgbGVnYWN5WyJj
b21wbGV0ZWQiXSA9PSA3OCBhbmQgbGVnYWN5WyJncm91cHMiXSA9PSBFWFBFQ1RFRF9MRUdBQ1kK
ICAgIGFzc2VydCBsZWdhY3lbImhhbmRsZXJfY2FsbHMiXSA9PSB7ImJhc2VsaW5lIjogNDQsICJj
YW5kaWRhdGUiOiA4MH0KICAgIGFzc2VydCBsZWdhY3lbImV4cGVjdGVkX3dyaXRlX2ZhaWx1cmVz
Il0gPT0gMSBhbmQgbGVnYWN5WyJleHBlY3RlZF90aW1lb3V0cyJdID09IDEKICAgIGNhc2UgPSAi
b2JzZXJ2ZXJfc2VsZWN0b3IiCiAgICByZWZlcmVuY2UgPSBzZWxlY3RlZF9uYW1lc3BhY2UoVU5D
SEFOR0VEX1RFWFQpCiAgICBhd2FyZSA9IHNlbGVjdGVkX25hbWVzcGFjZShTT1VSQ0VfVEVYVCkK
ICAgIGNhc2UgPSAibmF0aXZlX2NhcHR1cmUiCiAgICBuYXRpdmVfZnJhbWVzWyJ0cnVzdGVkIl0g
PSBjYXB0dXJlX3JlcGx5X2ZyYW1lKGF3YXJlKQogICAgbmF0aXZlX2NhcHR1cmVzWyJ0cnVzdGVk
Il0gKz0gMQogICAgbmF0aXZlX2ZyYW1lc1siY2xvbmVkIl0gPSBjYXB0dXJlX3JlcGx5X2ZyYW1l
KGF3YXJlLCBjbG9uZWQ9VHJ1ZSkKICAgIG5hdGl2ZV9jYXB0dXJlc1siY2xvbmVkIl0gKz0gMQog
ICAgbmF0aXZlX2ZyYW1lc1sidW50cnVzdGVkIl0gPSBjYXB0dXJlX2ZpeHR1cmVfZnJhbWUoKQog
ICAgbmF0aXZlX2NhcHR1cmVzWyJ1bnRydXN0ZWQiXSArPSAxCiAgICBmb3IgbmFtZSwgZ3JvdXAs
IGtpbmQsIGFyZ3MgaW4gQ0FTRVM6CiAgICAgICAgY2FzZSA9IG5hbWUKICAgICAgICBSVU5ORVJT
W2tpbmRdICgqYXJncykKICAgICAgICBvYnNlcnZlcl9jb21wbGV0ZWQgKz0gMQogICAgICAgIG9i
c2VydmVyX2dyb3Vwc1tncm91cF0gKz0gMQogICAgY2FzZSA9ICJvYnNlcnZlcl90b3RhbHMiCiAg
ICBhc3NlcnQgb2JzZXJ2ZXJfY29tcGxldGVkID09IDUwIGFuZCBvYnNlcnZlcl9ncm91cHMgPT0g
RVhQRUNURURfT0JTRVJWRVIKICAgIGFzc2VydCB0cmFwX2NvdW50cyA9PSB7InN0ciI6IDAsICJy
ZXByIjogMCwgInRyYWNlYmFjayI6IDB9CiAgICBhc3NlcnQgbmF0aXZlX2NhcHR1cmVzID09IHsi
dHJ1c3RlZCI6IDEsICJjbG9uZWQiOiAxLCAidW50cnVzdGVkIjogMX0KICAgIHJlc3VsdCA9ICJw
YXNzZWQiCmV4Y2VwdCBFbnZlbG9wZURlYWRsaW5lOgogICAgZmFpbHVyZSA9ICJ2ZXJpZmljYXRp
b25fZGVhZGxpbmUiCmV4Y2VwdCBBc3NlcnRpb25FcnJvcjoKICAgIGZhaWx1cmUgPSAiYXNzZXJ0
aW9uX2ZhaWxlZCIKZXhjZXB0IEJhc2VFeGNlcHRpb246CiAgICBmYWlsdXJlID0gImhhcm5lc3Nf
ZXhjZXB0aW9uIgpmaW5hbGx5OgogICAgc2lnbmFsLnNldGl0aW1lcihzaWduYWwuSVRJTUVSX1JF
QUwsIDApCgpsZWdhY3lfY29tcGxldGVkID0gMCBpZiBsZWdhY3kgaXMgTm9uZSBlbHNlIGxlZ2Fj
eVsiY29tcGxldGVkIl0KbGVnYWN5X2dyb3VwcyA9IHtrZXk6IDAgaWYgbGVnYWN5IGlzIE5vbmUg
ZWxzZSBsZWdhY3lbImdyb3VwcyJdLmdldChrZXksIDApIGZvciBrZXkgaW4gRVhQRUNURURfTEVH
QUNZfQpmYWlsZWRfY2FzZSA9IE5vbmUgaWYgZmFpbHVyZSBpcyBOb25lIGVsc2UgKAogICAgImxl
Z2FjeV8iICsgbGVnYWN5WyJjYXNlIl0gaWYgY2FzZSA9PSAibGVnYWN5Xzc4IiBhbmQgbGVnYWN5
IGlzIG5vdCBOb25lIGVsc2UgY2FzZSkKcmVjZWlwdCA9IHsic2NoZW1hIjogInJpYXV0aC5kMDEt
b2JzZXJ2ZXItbWVtb3J5L3YyIiwgInJlc3VsdCI6IHJlc3VsdCwKICAgICJmYWlsdXJlX3RhZyI6
IGZhaWx1cmUsICJmYWlsZWRfY2FzZSI6IGZhaWxlZF9jYXNlLCAicGxhbm5lZF9jYXNlcyI6IDEy
OCwKICAgICJsZWdhY3lfcGxhbm5lZCI6IDc4LCAib2JzZXJ2ZXJfcGxhbm5lZCI6IDUwLAogICAg
ImxlZ2FjeV9jb21wbGV0ZWQiOiBsZWdhY3lfY29tcGxldGVkLCAib2JzZXJ2ZXJfY29tcGxldGVk
Ijogb2JzZXJ2ZXJfY29tcGxldGVkLAogICAgImNvbXBsZXRlZF9jYXNlcyI6IGxlZ2FjeV9jb21w
bGV0ZWQgKyBvYnNlcnZlcl9jb21wbGV0ZWQsCiAgICAibGVnYWN5X2dyb3VwcyI6IGxlZ2FjeV9n
cm91cHMsICJvYnNlcnZlcl9ncm91cHMiOiBvYnNlcnZlcl9ncm91cHMsCiAgICAibGVnYWN5X2hh
bmRsZXJfY2FsbHMiOiB7ImJhc2VsaW5lIjogMCwgImNhbmRpZGF0ZSI6IDB9IGlmIGxlZ2FjeSBp
cyBOb25lIGVsc2UgbGVnYWN5WyJoYW5kbGVyX2NhbGxzIl0sCiAgICAiZXhwZWN0ZWRfd3JpdGVf
ZmFpbHVyZXMiOiAwIGlmIGxlZ2FjeSBpcyBOb25lIGVsc2UgbGVnYWN5WyJleHBlY3RlZF93cml0
ZV9mYWlsdXJlcyJdLAogICAgImV4cGVjdGVkX3Jlc3BvbnNlX3RpbWVvdXRzIjogMCBpZiBsZWdh
Y3kgaXMgTm9uZSBlbHNlIGxlZ2FjeVsiZXhwZWN0ZWRfdGltZW91dHMiXSwKICAgICJwcml2YXRl
X2Zvcm1hdF90cmFwcyI6IHRyYXBfY291bnRzLCAibmF0aXZlX2NhcHR1cmVzIjogbmF0aXZlX2Nh
cHR1cmVzLAogICAgImVsYXBzZWRfc2Vjb25kcyI6IHJvdW5kKHRpbWUubW9ub3RvbmljKCkgLSBz
dGFydGVkLCA2KSwKICAgICJkZWFkbGluZV9zZWNvbmRzIjogMzAsICJoZWxwZXJfc2hhMjU2Ijog
U09VUkNFX0hBU0gsICJ1bmNoYW5nZWRfc2hhMjU2IjogVU5DSEFOR0VEX0hBU0gsCiAgICAibGVn
YWN5X2FyY2hpdmVfc2hhMjU2IjogImNlNzlhZWI5OWZhZmFhZWIyNjVkNmI3YmZmNTVmNDE1ZTkx
ODI1ZDgzZTljNjUwZWVkNDEyODI1YTVhOTUyNWYiLAogICAgIm1haW5faW52b2tlZCI6IEZhbHNl
LCAic2VydmVyX2NvbnN0cnVjdGVkIjogRmFsc2UsCiAgICAibmV0d29ya19saXN0ZW5lcl9uYXRp
dmVfcHJvdmlkZXJfY2xpX2RyaXZlcl9icm93c2VyX2NhcmdvIjogRmFsc2V9CnJhdyA9IChqc29u
LmR1bXBzKHJlY2VpcHQsIHNvcnRfa2V5cz1UcnVlKSArICJcbiIpLmVuY29kZSgiYXNjaWkiKQph
c3NlcnQgbGVuKHJhdykgPD0gODE5MiBhbmQgU0VOVElORUwuZW5jb2RlKCkgbm90IGluIHJhdwpz
eXMuc3Rkb3V0LmJ1ZmZlci53cml0ZShyYXcpCnN5cy5zdGRvdXQuYnVmZmVyLmZsdXNoKCkKcmFp
c2UgU3lzdGVtRXhpdCgwIGlmIHJlc3VsdCA9PSAicGFzc2VkIiBlbHNlIDEpCg==
```

### Scoped zero-context diffs against immutable de9 archives

These virtual paths name archive fences, not existing/new harness files.
Both diffs are deterministic difflib.unified_diff outputs with n=0 and final LF.
Child diff:7343B, SHAe493cfa6290798a9c334a240c4d7fbdf44f647972d4cd42a35f8a78d0814792b.
Controller diff:3457B, SHA8f66e15a77481361006ce632187b7cc2aff51e25c2a1a400d3237cee3ca45151.
Whole forward/reverse application must match the exact original/corrected fence
bytes; only those scopes change. No actual source/workflow/helper file changes.

Marker: D01_DESCRIPTOR_MEMORY_LOGIC_DIFF_V2.

```diff
--- de9/observer-memory-logic.py
+++ proposed/descriptor-memory-logic.py
@@ -2 +2 @@
-from types import SimpleNamespace
+from types import SimpleNamespace, FrameType, TracebackType, FunctionType
@@ -9,3 +9,3 @@
-EXPECTED_OBSERVER = {"classes": 11, "trace": 8, "faults": 7, "first": 2,
-    "sites": 6, "known": 3, "fallback": 10, "privacy": 3}
-SOURCE_HASH = "c0c022ecd91d92c19058b31d413864c980fe6b0ddfbd1303a45be629afd900e9"
+EXPECTED_OBSERVER = {"classes": 11, "trace": 8, "faults": 5, "descriptor": 2,
+    "first": 2, "sites": 6, "known": 3, "fallback": 10, "privacy": 3}
+SOURCE_HASH = "75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1"
@@ -39,3 +39,3 @@
-class FakeError(SentinelError):
-    def __init__(self, trace=None, fail=False):
-        self.trace, self.fail = trace, fail
+class ChangingTraceback(SentinelError):
+    def __init__(self, native):
+        self.native = native
@@ -45,3 +45,2 @@
-            if object.__getattribute__(self, "fail"):
-                raise PrivateTrap
-            return object.__getattribute__(self, "trace")
+            trap_counts["traceback"] += 1
+            return SENTINEL if trap_counts["traceback"] % 2 else object.__getattribute__(self, "native")
@@ -50,4 +49,6 @@
-class FrameTrap:
-    @property
-    def tb_frame(self):
-        raise PrivateTrap
+class RaisingTraceback(SentinelError):
+    def __getattribute__(self, name):
+        if name == "__traceback__":
+            trap_counts["traceback"] += 1
+            raise PrivateTrap(SENTINEL)
+        return object.__getattribute__(self, name)
@@ -135 +136 @@
-def observe(namespace, error, holder=None, site="handler"):
+def observe(namespace, error, holder=None, site="handler", native=None):
@@ -139,0 +141,3 @@
+        if native is not None:
+            assert type(native) is TracebackType
+            BaseException.__traceback__.__set__(error, native)
@@ -143,2 +147,3 @@
-def fake_trace(namespace, line, count=1, trusted=True, cloned=False):
-    code = namespace["Handler"].reply.__code__ if trusted else fake_trace.__code__
+def capture_reply_frame(namespace, cloned=False):
+    original = namespace["Handler"].reply
+    function = original
@@ -146,2 +151,33 @@
-        code = code.replace()
-        assert code is not namespace["Handler"].reply.__code__
+        function = FunctionType(original.__code__.replace(), original.__globals__,
+                                "untrusted_reply_clone", original.__defaults__)
+        function.__kwdefaults__ = dict(original.__kwdefaults__)
+        assert function.__code__ == original.__code__ and function.__code__ is not original.__code__
+    handler = memory_handler(namespace, new_demo(namespace))
+    handler.wfile.once = True
+    try:
+        # Only this response body runs; all I/O and budget operations are memory sinks.
+        function(handler, 200, "memory frame capture")
+    except OSError as error:
+        trace = BaseException.__traceback__.__get__(error, BaseException)
+        for _ in range(64):
+            if trace is None:
+                break
+            assert type(trace) is TracebackType and type(trace.tb_frame) is FrameType
+            if trace.tb_frame.f_code is function.__code__:
+                return trace.tb_frame, trace.tb_lasti
+            trace = trace.tb_next
+    raise AssertionError
+
+def capture_fixture_frame():
+    try:
+        raise SentinelError(SENTINEL)
+    except SentinelError as error:
+        trace = BaseException.__traceback__.__get__(error, BaseException)
+        assert type(trace) is TracebackType and type(trace.tb_frame) is FrameType
+        assert trace.tb_frame.f_code is capture_fixture_frame.__code__ and trace.tb_next is None
+        return trace.tb_frame, trace.tb_lasti
+
+def native_trace(line, count=1, trusted=True, cloned=False):
+    key = "cloned" if cloned else "trusted" if trusted else "untrusted"
+    frame, lasti = native_frames[key]
+    assert type(frame) is FrameType and type(lasti) is int
@@ -150 +186,3 @@
-        trace = SimpleNamespace(tb_frame=SimpleNamespace(f_code=code), tb_lineno=line, tb_next=trace)
+        # Native traceback with a genuine captured frame and a controlled line annotation.
+        trace = TracebackType(trace, frame, lasti, line)
+        assert type(trace) is TracebackType
@@ -173 +211 @@
-    value = observe(aware, FakeError(fake_trace(aware, line, count, trusted, cloned)))
+    value = observe(aware, SentinelError(SENTINEL), native=native_trace(line, count, trusted, cloned))
@@ -177,0 +216,10 @@
+def descriptor_case(kind):
+    native = native_trace(1)
+    error = ChangingTraceback(native) if kind == "changing" else RaisingTraceback(SENTINEL)
+    before = dict(trap_counts)
+    value = observe(aware, error, native=native)
+    assert value == {"observed": True, "diagnostic": {"site": "handler",
+        "exception_class": "other", "own_function": "Handler.reply", "own_line": 1}}
+    assert trap_counts == before and trap_counts["traceback"] == 0
+    assert SENTINEL not in json.dumps(value, sort_keys=True)
+
@@ -179,7 +227 @@
-    if kind == "accessor":
-        value = observe(aware, FakeError(fail=True))
-        assert value == {"observed": True, "diagnostic": None}
-    elif kind == "frame_accessor":
-        value = observe(aware, FakeError(FrameTrap()))
-        assert value == {"observed": True, "diagnostic": None}
-    elif kind in ("read", "observed", "diagnostic"):
+    if kind in ("read", "observed", "diagnostic"):
@@ -305 +347 @@
-        first = observe(aware, FakeError(fake_trace(aware, 1)), holder)
+        first = observe(aware, SentinelError(SENTINEL), holder, native=native_trace(1))
@@ -465,2 +507,2 @@
-    ("fault_trace_accessor", "faults", "fault", ("accessor",)),
-    ("fault_frame_accessor", "faults", "fault", ("frame_accessor",)),
+    ("descriptor_changing_getter", "descriptor", "descriptor", ("changing",)),
+    ("descriptor_raising_getter", "descriptor", "descriptor", ("raising",)),
@@ -497 +539 @@
-RUNNERS = {"class": class_case, "trace": trace_case, "fault": fault_case,
+RUNNERS = {"class": class_case, "trace": trace_case, "descriptor": descriptor_case, "fault": fault_case,
@@ -505 +547,3 @@
-trap_counts = {"str": 0, "repr": 0}
+trap_counts = {"str": 0, "repr": 0, "traceback": 0}
+native_frames = {}
+native_captures = {"trusted": 0, "cloned": 0, "untrusted": 0}
@@ -535,0 +580,7 @@
+    case = "native_capture"
+    native_frames["trusted"] = capture_reply_frame(aware)
+    native_captures["trusted"] += 1
+    native_frames["cloned"] = capture_reply_frame(aware, cloned=True)
+    native_captures["cloned"] += 1
+    native_frames["untrusted"] = capture_fixture_frame()
+    native_captures["untrusted"] += 1
@@ -543 +594,2 @@
-    assert trap_counts == {"str": 0, "repr": 0}
+    assert trap_counts == {"str": 0, "repr": 0, "traceback": 0}
+    assert native_captures == {"trusted": 1, "cloned": 1, "untrusted": 1}
@@ -558 +610 @@
-receipt = {"schema": "riauth.d01-observer-memory/v1", "result": result,
+receipt = {"schema": "riauth.d01-observer-memory/v2", "result": result,
@@ -567 +619,2 @@
-    "private_format_traps": trap_counts, "elapsed_seconds": round(time.monotonic() - started, 6),
+    "private_format_traps": trap_counts, "native_captures": native_captures,
+    "elapsed_seconds": round(time.monotonic() - started, 6),
```

Marker: D01_DESCRIPTOR_MEMORY_CONTROLLER_DIFF_V2.

```diff
--- de9/observer-memory-controller.py
+++ proposed/descriptor-memory-controller.py
@@ -3 +3 @@
-PAYLOAD_SHA256 = "b7bdeb6f9e24b8c537cfdca7506f570a6e2e180efa2f560dad2c6c1693aa0822"
+PAYLOAD_SHA256 = "b10cf8376edbb80b00c051305f31643b04714494260c4a49576a354361048779"
@@ -9,2 +9,2 @@
-PLANNED_OBSERVER = {"classes": 11, "trace": 8, "faults": 7, "first": 2,
-    "sites": 6, "known": 3, "fallback": 10, "privacy": 3}
+PLANNED_OBSERVER = {"classes": 11, "trace": 8, "faults": 5, "descriptor": 2,
+    "first": 2, "sites": 6, "known": 3, "fallback": 10, "privacy": 3}
@@ -32,2 +32,2 @@
-    "fault_trace_accessor",
-    "fault_frame_accessor",
+    "descriptor_changing_getter",
+    "descriptor_raising_getter",
@@ -189 +189 @@
-        "expected_write_failures", "expected_response_timeouts", "private_format_traps",
+        "expected_write_failures", "expected_response_timeouts", "private_format_traps", "native_captures",
@@ -194 +194 @@
-    require(document["schema"] == "riauth.d01-observer-memory/v1")
+    require(document["schema"] == "riauth.d01-observer-memory/v2")
@@ -197 +197 @@
-    names = set(observer_names) | {"source_archive", "legacy_source_pin", "observer_selector", "observer_totals"} | {
+    names = set(observer_names) | {"source_archive", "legacy_source_pin", "observer_selector", "native_capture", "observer_totals"} | {
@@ -206 +206,2 @@
-                      ("private_format_traps", {"str", "repr"})):
+                      ("private_format_traps", {"str", "repr", "traceback"}),
+                      ("native_captures", {"trusted", "cloned", "untrusted"})):
@@ -213,4 +214,4 @@
-    for key, value in (("helper_sha256", "c0c022ecd91d92c19058b31d413864c980fe6b0ddfbd1303a45be629afd900e9"),
-                       ("unchanged_sha256", "7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0"),
-                       ("legacy_archive_sha256", "ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f")):
-        require(document[key] == value)
+    for key in ("helper_sha256", "unchanged_sha256", "legacy_archive_sha256"):
+        value = document[key]
+        require(type(value) is str and len(value) == 64
+                and all(character in "0123456789abcdef" for character in value))
@@ -225 +226 @@
-    record = {"schema": "riauth.d01-observer-memory-controller/v1",
+    record = {"schema": "riauth.d01-observer-memory-controller/v2",
@@ -293 +294 @@
-    outcome = {"schema": "riauth.d01-observer-memory-review/v1", "result": "failed",
+    outcome = {"schema": "riauth.d01-observer-memory-review/v2", "result": "failed",
@@ -302 +303,6 @@
-        require(document is not None and document["result"] == "passed"
+        require(document is not None)
+        for key, value in (("helper_sha256", "75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1"),
+                           ("unchanged_sha256", "7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0"),
+                           ("legacy_archive_sha256", "ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f")):
+            require(document[key] == value)
+        require(document["result"] == "passed"
@@ -312 +318,2 @@
-                and document["private_format_traps"] == {"str": 0, "repr": 0}
+                and document["private_format_traps"] == {"str": 0, "repr": 0, "traceback": 0}
+                and document["native_captures"] == {"trusted": 1, "cloned": 1, "untrusted": 1}
```

### Static-only evidence and remaining gates

In-memory assembly/hash/base64 round-trip/AST parse/code-object compile and
literal case/group/name counting passed without evaluating any archived code.
Full reconstructed payload is161014B/SHAb10cf8376edbb80b00c051305f31643b04714494260c4a49576a354361048779;
readable controller is15981B/SHA0e93e11f71f794770a60700760510f9d2893fa2bdfd044d09a5cb4c20ef555de.
Static AST comparison identified exactly the changed/removed/added definitions
described above and24 unaffected shared definitions. One possible Popen node
exists, with no call to main/HTTPServer/DemoServer in the new logic.
Final report-fence reconstruction, diff reversal, immutable prefix/helper pins,
docs/whitespace/scope and clean receipts are recorded after authoring.

No helper/observer/harness/selected definition/module-main execution or import,
server constructor, HTTP/network/socket/listener/native/provider/CLI/Driver/
browser/Cargo/PG operation or runtime slot occurred. No new worker/task/WT/
managed shell, alignment, main/push/status mutation, contact or cache deletion.
Real browser remains HELD. Original cause, sender and first cleanup start stay
UNKNOWN; all real failures and legacy results retain their original limits.
Root full review AND independent design review precede any separate release
of ONE exact bounded memory invocation. No trial run or pass is inferred.
RiWork Cua.ai Driver preference persists.

### Final descriptor-aware static receipt

Actual static checks passed before this report-only handoff. Fence extraction
reconstructed30840B/629-line logic,15981B/348-line controller,217513B/2825-line
base64 archive and7343B/3457B zero-context diffs with their exact stated hashes.
Decoding and independent four-assignment assembly agreed across all161014
payload bytes and633 physical lines, SHA-256
`b10cf8376edbb80b00c051305f31643b04714494260c4a49576a354361048779`.
Whole forward AND reverse application of both diffs agreed with immutable
de9 and corrected fence bytes. This parser applies string diffs only.

AST parse and in-memory compile passed for corrected logic/controller/full
payload, the exact old ce79 archive, unchanged legacy definition nodes and
selected corrected-helper definitions. No code object was evaluated. Literal
counting proved50 unique observer names/nine groups and78 legacy cases, total128;
controller allowlists equal the corrected50 names plus unchanged78 names.
Comparing definition ASTs proved exactly four changed shared definitions,
three removed fake-trace definitions, six added native/hostile definitions and
24 identical shared definitions.48 retained case descriptors are identical.
There is one possible future Popen node, no direct main/HTTPServer/DemoServer
call and no SimpleNamespace fake trace/frame construction in the new logic.

All119102 prior bytes remain identical to a88c2ba, including both older design
and F1/source-correction appendices. Actual helper bytes remain exactly
f4ef05d/75bfb8a6 with no diff. Old ce79/91163B reconstruction passed unchanged.
`python3 scripts/check-docs.py` exited0: Markdown links and build-directory
layout checked. `git diff --check HEAD` exited0. Scope/prefix checks found only
this append-only report delta and no untracked or other staged/unstaged paths.
No static tool/check failed in this phase; no actual envelope result exists.

The separate report-only commit and post-commit clean check are the handoff.
No native traceback/frame, exception instance, observer, capture path, old/new
case or controller was constructed/executed by these checks. No slot was
acquired/released. Root full and independent design review, then separate exact
ONE bounded-memory release, remain required; real browser remains HELD.
Original cause/sender/first cleanup remain UNKNOWN. No completion credit.


## Root actual descriptor-aware memory verification, 2026-10-02

Root read the complete0178404 independent review: no new source blocker.
After both full source reviews, root released and ran exactly one archived
b10cf837 payload through exact0e93e11f controller. All128cases PASSED:
78legacy/50observer,44baseline/80candidate Handler calls, three genuine native
Python frame captures, str/repr/traceback-getter counters all0. One injected
write failure and one timeout completed. Child exit0/reaped, stderr0; full1398B
stdout SHA006d33cc7314d7c483f99dcb8ed59d3b6b537571b5908ee4178d14c21094024f
and numeric exit were retained/fsynced before comparisons. Child elapsed0.101654s;
final post-save controller0.202287s, root function0.202353875s, within30/35bounds.
No cleanup failure or signal stop was needed; no independent PID/group audit
is claimed because that controller does not retain a child PID. Exclusive0600
captures/retained/review/final-return files remain private. [Root actual receipt](evidence/wave30-d01-descriptor-memory-0ec651d.json)
contains the complete fixed child JSON and source/hash/timing/mode evidence.
No helper main/server construction, native provider/CLI/network/listener/Driver/
browser/Cargo ran. Memory source/guard/privacy acceptance only; no journey or
whole D01/D05 completion. Historical failures/unknowns are preserved unchanged.
The next real fixture remains held for complete controller/cleanup source review
and a fresh separately released bounded attempt. No slot was acquired/released.
