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
