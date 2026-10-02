# Wave 30 D01 observer independent source review

Date: 2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Reservation: `wave30_D01_unexpected_observer_independent_source_review`.
Existing WT `ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`; clean entry HEAD
`fc2b1bce8d1a8b36868fcc390e075d78116f4d39`.
Only this new report is owned/written. No source import or alignment.

## Finding and recommendation

**F1: hold the observer's claimed fixed-shape/privacy guarantee for one narrow
source correction.** At candidate line254, `error.__traceback__` uses ordinary
attribute dispatch on an arbitrary BaseException subclass. Its overridden
accessor can return a counterfeit trace. Line259 validates `tb_lineno`, but
line260 reads it again when storing `own_line`. A changing accessor can return
an integer for the type/range checks and an unchecked value for the assignment.
Thus the new private diagnostic can contain an arbitrary string/value, rather
than only a line integer or null. The finite-shape/160-byte argument does not
cover this path. A failed-accessor exception is caught safely; a successfully
returned unsafe value is not rejected.

This is a source-derived mechanism, not an executed reproduction, an observed
provider error or the cause of the earlier D01 request failure. One minimal
proposed hunk is given below for root's separate ownership/review. No helper,
observer, harness/stub, collector or candidate function was executed here.
No other concrete blocker was identified in the seven additive hunks. Whole
baseline byte/AST preservation passed; it does not establish behavioral
equivalence of the newly executed diagnostic under adversarial accessors.

I04's original DONE disposition at root publication dc0 and all closed rows
remain closed. D01/D05 status and primary assignments are unchanged. Real
browser work and an observer-aware memory envelope remain separately HELD;
root alone decides source correction, envelope review, release and disposition.

## Immutable objects and full-body reads

| Role | Exact source and actual review |
| --- | --- |
| Baseline | `470690cad0cd93c9f25c5bc40b982e1b91679b49`, `scripts/d01-confidential-browser-demo.py`: 32,723 bytes/695 lines, blob `3d3379399126423a6a5588a6bd55ea4ac1723ebd`, SHA-256 `7fbc23e56dbc4999b94672ec4b29b0d33596c53b4d637ea99f850451bd63fbf0`. All original bodies were read within the complete candidate and compared through whole reversal; this is not only a hash lookup. |
| Candidate | `3c9c63bb217bfc20614e62ef7116e6e2abad0170`: all757 lines read in bounded1–260/261–520/521–757 chunks, including every original flow/parser/reply/main/cleanup body and all seven additions. 35,711 bytes, blob `d980f87959b6a14126c52c07a24bf859ccd26766`, SHA-256 `c0c022ecd91d92c19058b31d413864c980fe6b0ddfbd1303a45be629afd900e9`. |
| Published identity | `56bd0829514ed8014cc9563fc7b0e727dba46d1d` helper is byte-identical to3c9, including the supplied SHA-256. No current mutable main/source-worker file was inspected. |
| Complete design | `1e50d3e1d368ffa98c8a56e7b9b8614f6d3d27b0`, `docs/roadmap/local-wave30-d01-unexpected-failure-plan.md`: all379 lines read, 24,372 bytes/SHA-256 `85409506c980b1cd41eae70d300fdbdc5d95e51de12c6753b21b46b1bb4c7988`. An initially truncated combined read was completed with bounded chunks. |
| Complete static appendix | `6ff577800c704d406f7ffd2aa342e719b337ed6e`: all101 added lines read. Whole480-line/30,530-byte report SHA-256 `fe08d3aad7122e090dce67796cfb87fd2ed7cc7f9cecc561e13d55195308a018`; its complete design prefix matches1e50 exactly. |

The observer definition plus following blank lines is2,263 bytes/SHA-256
`4825addc25de47496cc6532bd417e3029256c5c130ead4e06d8cd65d85e53487`.
Repository guidance was read; no applicable AGENTS file was found in the
worktree/ancestors. Its broad build guidance is superseded by this explicit
source-only reservation. The helper is absent from this own branch and remains
absent. No private receipt, request/protocol input, author workspace, native
artifact or separately prepared observer-memory design was opened.

## F1 exact information flow and smallest proposed hunk

The paper counterexample requires only an exception subclass with an overridden
`__traceback__` accessor. It returns one trace-shaped object whose `tb_frame`
exposes one actual allowlisted code object and whose `tb_next` is null. Its
`tb_lineno` accessor returns a small valid integer on the first two reads, then
an arbitrary ASCII string on the final assignment read. The condition's
chained comparison evaluates its line operand once; the final assignment is
a separate attribute read. The later loop comparisons need not match another
code object, so they do not overwrite that stored value. No failure is raised,
the residual trace is null, and line264 stores the unsafe value in the holder.
The unchanged line732 JSON writer can therefore serialize that value. The
new observer never explicitly calls str/repr, but that alone cannot enforce
its advertised output type or exclude raw private text obtained this way.

This is constructible Python attribute-dispatch behavior identified by source;
no exception instance, fake traceback, sink, sentinel harness or truth table
was constructed/run in this review. No actual private value or past caller is
asserted. A subclass accessor which merely raises follows the separate safe
fallback described below and is not this counterexample.

Proposed **one-line** change, against exact3c9/published56bd line254 only:

```diff
@@ -254 +254 @@
-        trace = error.__traceback__ if isinstance(error, BaseException) else None
+        trace = BaseException.__traceback__.__get__(error, BaseException) if isinstance(error, BaseException) else None
```

Read the built-in BaseException traceback descriptor directly, bypassing the
subclass's attribute dispatch. Its native traceback/frame/line objects supply
the existing bounded identity/line scan. Keep the same first latch, class
labels,64-link cap, ten own codes, line limits, inner fallback and three outer
guards. This is a proposed source seam, not an edit, compiled artifact or tested
fix. Root's separately prepared memory envelope should cover a hostile subclass
accessor without invoking it, as well as its already planned cases; no new
runtime reservation or broad campaign is requested here.

## First observation, trace boundaries and privacy review

| Mechanism | Source conclusion and practical boundary |
| --- | --- |
| First-only latch | Lines227–230 refuse a prior observation or invalid site and set observed:true before retrieving the exception, codes or trace. With the owned plain holder, subsequent handler/server/main observations cannot overwrite it. A failure after latching can leave diagnostic:null permanently. A failure before the latch can leave unobserved/null. “First” means first successful latch at these catch sites, not proof of the earliest external exception or original sender. |
| Class labels | Lines232–240 use `type(error) is kind` for exactly AttributeError, TypeError, ValueError, KeyError, OSError, BrokenPipeError, ConnectionResetError and TimeoutError; everything else is the literal `other`. No subclass name, `__name__`, message, args, str/repr, cause/context or exception object enters the diagnostic. Exact identity avoids treating OSError subclasses as OSError. |
| Code trust | Lines242–253 list exactly HeaderReader.readline, DemoServer.process_request, Demo.begin/callback/invoke, Handler.handle_one_request/send_error/get/reply and main. Matching is code-object `is`, with fixed labels rather than co_name/co_filename. On a genuine traceback this is a closed trust check, not name/path matching. Ordinary subclass access at254 currently admits the counterfeit-trace issue F1; the code-object comparison alone does not authenticate a custom frame-shaped object. |
| Trace cap | Lines255–263 scan at most64 links and640 fixed code comparisons. The last matching valid own frame wins. An unexhausted trace after64 clears both fields; no trusted frame or only out-of-range lines leave null fields. A later invalid line does not itself clear a previously valid own frame. This is a fixed scan count on ordinary traces, not a separate timing proof for arbitrary Python accessors. |
| Line limits | Line259 requires exact int (not bool) and1..1024. Genuine traceback line fields are stable. F1's repeated final read defeats the guarantee for custom accessor-backed traces; there is no final shape validation before storage/serialization. |
| Privacy sinks | Lines264–265 store only site/class/own-function/own-line in the intended shape. The function has no explicit formatting, I/O, logging, import, exception chaining, locals/frame-global/filename/header/query/cookie/token/sender access. F1 remains an indirect unchecked-value sink. Class/subclass str/repr traps are never explicitly invoked by the observer; hostile attribute-read/write failure is swallowed without formatting its exception. |
| Storage/output | Demo's owned plain dict at283 is shared with record at657. Before Demo exists, the main record owns its initial dict at616. No sidecar or extra file/write is added. Normal evidence uses the unchanged exclusive0600 FD at642 and ASCII JSON writer731–734. Observer output is absent from ready stdout666 and final stdout746–753, generic HTML and replies. The literal allowed schema has a static maximum160 bytes; that bound is conditional on field types and does not establish privacy for F1. |

## All three guarded sites and unchanged outcomes

| Existing catch site | Additive guard and exact old continuation |
| --- | --- |
| HTTPServer `handle_error`427 | New428–431 wraps function lookup, `self.demo`/holder access, call and projection in `try/except BaseException: pass`. Original432 still sets unexpected_failure/done:true. Process_request419–425 still shuts down the request and clears active in finally. The new observer sends no reply or shutdown. |
| Handler unexpected `except Exception`523 | New524–527 protects lookup/argument evaluation/call, then original528–529 still sets unexpected_failure/done:true and attempts the same generic400 reply. A later unexpected reply error can reach the server guard; the prior latch remains first. Known PreflowAuthorizationRefusal/Failure branches516–522 were not moved or modified. |
| Main unexpected `except Exception`679 | New680–683 protects record access/lookup/call, then original684–685 still sets the same tag and stage. Earlier Failure/Halt/KeyboardInterrupt clauses674–678 stay in the original order. Finally686–727 and the existing evidence writer/result exit remain unchanged. |

Observer-local BaseException catch266–268 protects projection and diagnostic
write failures after the latch. Site guards also protect a missing observer
name in the legacy selector namespace, argument/property failure or an observer
which itself raises BaseException. This supports the intended source fallback;
no such injection was performed and no preservation truth-table pass is claimed.
An unexpected error which was never observed cannot acquire a retrospective
diagnostic. Known failures and normal success do not call the observer. These
new guards do not change the original Failure/Halt handling around the request
or flow; they protect only the new observation calls.

## Baseline preservation and actual static checks

Seven insertion-only spans in final source are224–270 (47 lines),283 (1),
428–431 (4),524–527 (4),616 (1),657 (1),680–683 (4): exactly62 additions.
Removing them reconstructs every32,723 baseline byte and all695 original lines,
including final newline. Published56bd matches all35,711 candidate bytes.
No baseline-only preparation import was performed in this WT.

Actual stdlib AST parsing, with **no compile/exec/import of candidate code**,
removed exactly the new definition, three guarded calls, two holder/shared
assignments and one record dict field. The resulting whole normalized AST is
identical to470, including all35 original functions and their exception headers,
statement/call order and finally bodies. Static guards also confirmed the three
literal sites, single range64, no observer import/raise/with or explicit
formatting/I/O calls, and no forbidden args/cause/context/locals/globals/code
name/filename attributes. Literal JSON-size enumeration computed160 bytes;
it does not execute the observer or discharge F1. Complete design-prefix/
101-line-append equality and source absence checks passed. No static check
failed here; the initial combined design output was truncated and reread.

All original acceptance/refusal predicates remain exact old source nodes:

- Request-line/header/count/body/Host/target/method/Origin/cookie constraints,
  secret short circuit, no arbitrary credentials, generic replies and status
  bookkeeping are unchanged. First three preflow Authorization refusals return
  403 while live; fourth increments the same counter to4 and marks terminal
  request_invalid. Postflow Authorization is still terminal400. Counter or
  allowance changes are not part of this observer slice.
- One attempted flow, state/nonce/S256, exact issuer/client/callback, flow-cookie
  and callback consumption, confidential exchange, pinned verifier/native
  provider identity, token/JWKS claims/access hash and userinfo subject checks
  remain unchanged. The helper still performs no riAuth login or approval.
- Pending180 seconds, argument ceiling600 seconds, phase timeouts, disk
  thresholds, original first-failure assignments, replies and cleanup/evidence
  failure precedence remain the baseline. Byte/AST equality is source proof,
  not executed protection or a cleanup timing measurement.

The pinned verifier/provider references were read in the full helper, not
queried, loaded or executed: verifier9cefe7a/blob3be747d0/SHA-256
`f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d`,
OpenSSL SHA-256
`67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72`
and the existing3.6.4 banner literal. This is identity/predicate review, not a
fresh provider/verifier/version result. Controllers and actual protocol paths
were not executed or rewritten.

## Historical failure and unexecuted boundaries

The complete immutable1e50 design records one prepared180 helper EXIT1 at
unexpected_failure/request and outer rp_nonzero/browser_checkpoint. It records
protected-before403, zero Authorization refusals, null request-invalid reason
and null authorization redirect/callback/token/userinfo/protected-after statuses.
The original inner cause and original Authorization sender remain **UNKNOWN**;
F1 is not retrospectively attributed to that unobserved exception. Five CLI
prerequisites and prior cleanup statements are historical receipt attribution,
not commands or resources checked by this review.

Historical helper/outer elapsed10.096s/111.287s and cleanup action observations
do not establish true first cleanup start or the whole60-second bound. The
recorded first_event_proven:false/whole_cleanup_within60_proven:false remains
failed/unproved. Earlier61-of-76 failure and corrected78-case legacy memory
pass retain their own limits; their old selector omits this observer and they
are not observer evidence. No new observer-aware envelope, memory case or
real-browser run was read or executed here. Original failures are not relabeled.

Source inspection and static preservation are complete with F1 reported.
Observer behavior, failure/accessor preservation cases and browser completion
are **UNEXECUTED here**; real browser remains HELD. No runtime slot was acquired
or released, no deletion/cleanup/service operation was performed, and no worker
or orchestrator was contacted. No source/controller/other-report, task, board,
primary assignment, main or accepted tree changed. No new task/WT/worker/shell,
network/socket/listener/HTTPServer constructor, helper/module main/import,
native/product-CLI/provider/version/Cargo/PG/browser/Driver operation or
merge/push.

## Report validation and handoff

Actual report-only checks, all EXIT0:

- `python3 scripts/check-docs.py`: Markdown links and build-directory layout
  checked.
- `python3 scripts/check-repo-hygiene.py`: tracked-file hygiene checked,
  945 files with this report staged.
- `git diff --cached --check`: no whitespace errors. The staged scope is
  exactly this one added report; no existing tracked file changed. Entry HEAD
  remains fc2b1bce until the report-only commit.
- A stdlib/static Git-object utility rechecked exact candidate/published byte
  identity, baseline and candidate SHA-256 values, the three F1 line witnesses,
  all62-line reversal, complete design-prefix equality, all four report hash
  references, final newline/trailing whitespace and own-branch helper absence.
  It parsed/read immutable data only and executed no candidate code.

The report-only commit and subsequent clean-state check are handed off by their
immutable hash. Root owns the proposed single-line source reservation and any
independently reviewed observer-aware bounded release. I recommend holding the
observer's claimed fixed shape for F1. Source-preservation checks passed
independently of that new diagnostic's unexecuted behavior.
