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
