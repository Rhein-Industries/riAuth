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

## Descriptor fix and memory design — independent source review

Date: 2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Reservation: `wave30_D01_descriptor_memory_independent_review`; same existing
WT/branch. Entry HEAD is `a770027d009172a96e4776aa5504500b66678520`.
This appendix preserves all 17,324 prior bytes, SHA-256
`ca7a6be40a1e0aaa03ff924a64dee59a922bdb6f83dfc09b93ece72eec78455d`.
The dated F1 finding and earlier HOLD recommendation are retained above.

**Recommendation: accept the descriptor correction and prospective envelope
by source inspection for root's separately released ONE bounded memory
invocation. No new concrete source blocker was found.** The exact published
built-in descriptor change addresses F1's subclass-accessor mechanism. This
does not claim an executed observer/privacy/fallback/capture test, approve a
browser release, reinterpret an old failure or change any original row status.
No further source hunk is proposed by this review.

### Exact inputs and what was actually read

| Input | Immutable identity and review extent |
| --- | --- |
| Corrected helper | `f4ef05d8428b235511e81277ba6b4b72d5ec08ba:scripts/d01-confidential-browser-demo.py`: all 757 lines read, 35,749 bytes, SHA-256 `75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1`. All parser, flow, reply, catch, main, writer and cleanup bodies were read, not only the descriptor line. |
| Published helper | `58155aad7d80a0a73c0bf3902e146a6aa07ef675`: whole helper bytes equal f4ef05d and the supplied SHA. Identity check only; no moving main/worktree import or alignment. |
| Reviewed design | `0ec651d6f7b30e320a8840683bd1e2d2bab0e484:docs/roadmap/local-wave30-d01-unexpected-failure-plan.md`: whole document 410,731 bytes/6,488 lines, SHA-256 `9530a35876c00d4fd6e949bb51ab56b12b36e0d829358a1d3e8672273ca670bb`. The entire new descriptor-aware prose, both scoped diffs and every readable child/controller body were read. |
| Design's prior prefix | `a88c2ba82615190d477ebfbc7155cb7c4d5c4908`: all 119,102 preceding bytes/2,207 lines preserved, SHA-256 `88b376467f3893a61e8f16c8209402eb7d8fee7f2b6f9b336fa46b4a75c87f9e`. Whole prefix comparison, not a claim of rereading every prior prose line in this slice. |
| Readable child | All 629 lines, document lines 2387–3015, 30,840 bytes, SHA-256 `8bf973998f0e327d9ac7fbfa7bcbd7bb3e93c8c3debe9c1a2d8251e9fd840046`. Read in complete 1–210/211–420/421–629 chunks. |
| Readable controller | All 348 lines, document lines 3024–3371, 15,981 bytes, SHA-256 `0e93e11f71f794770a60700760510f9d2893fa2bdfd044d09a5cb4c20ef555de`. Both complete halves read. |
| Unchanged legacy logic | All 461 readable lines, document lines 611–1071. Removing only the documented display space in `new["handle_error"] (` restores 24,210 bytes/SHA-256 `98b19b4d422067b0d49ea7eb1035a719d88bda672dc36579c6cf5c47cb0924eb`. Its original guard/sink assertions and tail were read; no tail or definition was executed. |
| Complete serialized payload | All 161,014 decoded bytes/633 physical lines statically reconstructed, SHA-256 `b10cf8376edbb80b00c051305f31643b04714494260c4a49576a354361048779`. Base64 fence: all 217,513 ASCII bytes/2,825 lines consumed and strictly decoded in memory, SHA-256 `abfd936ec4445320901131449c1f3df271bf2eaf68c115f91b093709e639c53c`. Encoded archive identity/assembly is distinct from semantic body reading. |

The complete 148-line child diff and 55-line controller diff were read. Their
exact 7,343/3,457-byte SHA-256 values are respectively
`e493cfa6290798a9c334a240c4d7fbdf44f647972d4cd42a35f8a78d0814792b` and
`8f66e15a77481361006ce632187b7cc2aff51e25c2a1a400d3237cee3ca45151`.
Both were independently regenerated, then applied forward and in reverse as
string diffs; complete old/new fence equality passed. The old readable child
and controller were AST/byte-comparison inputs; unchanged bodies are also
present in the completely read corrected sources. No mutable author files,
private protocol/receipt input or new execution result was inspected.

### F1 correction and preserved production behavior

Corrected helper line 254 reads
`BaseException.__traceback__.__get__(error, BaseException)` directly. It avoids
the subclass's normal `__traceback__` getter and obtains the native traceback
slot; native traceback/frame/line access closes F1's counterfeit-frame and
changing-line route. It keeps exact exception class identities, the first
latch, ten code-object identities, 64-link scan, 1..1024 exact-int check and
null-on-over-cap behavior. The observer contains no exception message/args,
str/repr, class-name, trace text, filename, locals/globals, header/query/token/
cookie/Origin/sender read or formatting sink. All three outer BaseException
fallbacks and the inner fallback remain unchanged.

Actual byte reversal of this one-line change reconstructs the entire 35,711-byte
3c9 helper. Removing the same seven 62-line observer insertions from the fixed
helper reconstructs every original 32,723 byte of 470690ca. Therefore all
accepted/refused request predicates, four-refusal counter and strict postflow
gate, phase/pending deadlines, cryptographic/verifier/provider pins, original
first-failure assignments, replies, cleanup and evidence precedence are exact
old source. This is source preservation, not execution evidence.

Static literal schema enumeration gives a maximum 126-byte diagnostic and
160-byte complete holder. The class/site/function labels and native line
integer/null provide that finite alphabet; the observer's real three callers
pass literal sites and owned holders. This is not a generic guarantee for
arbitrarily substituted module globals/callers. The memory design checks the
holder's closed keys, exact bool/int bounds, label enums, paired null fields,
160-byte cap and sentinel absence before its fixed result is constructed.
Ready/final public stdout and generic responses still omit the diagnostic.

### Genuine capture and descriptor cases, all prospective

Child lines 147–189 explicitly obtain genuine `TracebackType` and `FrameType`
objects. The first capture calls only the selected unbound `Handler.reply`
body using memory header/write/budget sinks; a one-shot memory write raises
OSError. The bounded scan selects the actual reply frame and lasti by identity.
The second capture executes a `FunctionType` clone made with `code.replace()`:
its code is asserted equal but distinct, without replacing the trusted code.
The third captures only the fixture's own SentinelError frame. Actual native
type and identity assertions are present; **none was evaluated here**.

`native_trace` constructs native traceback links from those captured frames
and actual lasti. Supplied line annotations 1/1024/0/1025 and lengths 64/65 are
synthetic native chains, often repeating one frame. They are not natural
execution-line or recursive-stack evidence. The clone and untrusted frame
must yield null own fields. `observe` installs the chain through the built-in
descriptor's setter inside an active exception handler, then calls the fixed
observer. No custom trace/frame-shaped object is accepted in the new path.

The changing and raising subclass getters at child lines 39–54 are distinct
adversarial cases. Lines 216–224 require the exact native Handler.reply/line1
projection, fixed `other` label, unchanged formatting counters, traceback
getter count zero and no sentinel in the holder. Thus a future invocation
checks that both getters are never called, rather than treating a swallowed
getter error as a successful diagnostic. No subclass instance, native frame,
traceback, sentinel sink or capture path was constructed by this review.

Holder read/latch/storage failure cases retain the intended false/null or
latched true/null state and refuse a later overwrite. First-latch/secondary
reply failure compares unchanged outcomes. Six real selected-body site cases
cover parser, dispatch, reply, send_error, server process/error and copied main
generic-except body; main itself is never called, so that copied frame expects
null own-function. Known Failure/Halt/success stays unobserved. Missing function/
field and outer BaseException injections preserve terminal/failure/reply
assertions. Writer-tail cases copy the exact tail into a controlled wrapper,
compare old/new fixed public outputs and keep original failure/stage despite
an injected evidence-write failure; they do not open a real helper evidence FD.

### Case accounting and unchanged oracles

| Observer group | Named cases, statically counted only |
| --- | ---: |
| Exact builtins/subclasses/private other | 11 |
| Native identity/clone/untrusted/line/cap | 8 |
| Hostile traceback getters | 2 |
| Holder/latch/storage/invalid-site/no-active-error | 5 |
| First-only/secondary failure | 2 |
| Unexpected selected-body sites | 6 |
| Known Failure/Halt/success | 3 |
| Missing observer/field/BaseException/legacy fallback | 10 |
| Formatting/private-writer/public-output | 3 |
| Total observer names | 50 |

AST literal accounting verifies 50 unique cases and exact controller name/group
allowlists. Forty-eight case descriptors are identical to the old unexecuted
design. Two impossible counterfeit-trace fault cases are replaced by the two
meaningful getter cases; no padded case is added. The three native captures
have a separate `native_capture` phase and three counters incremented only
after successful capture; they are setup witnesses, not three more cases.

All 461 normalized legacy logic lines and original definition/assertion nodes
are retained. Original imports/timer/output/exit tail are excluded by the
definition-only selector; the new child supplies the corrected helper and
same ac3 baseline. The unchanged 78-name/controller allowlist and 12 groups
match. Independent source counting of the run body's finite lists/loops yields
78 cases, 44 baseline calls and 80 candidate calls; this is not an actual count
receipt. Planned legacy oracles still require one injected write failure and
one response timeout. Their selector omits the observer, so even a future
legacy pass alone would not prove the new observer.

AST comparison against the old design finds 24 identical shared definitions;
only observe/trace_case/fault_case/first_case change. FakeError/FrameTrap/
fake_trace are removed; six native/hostile definitions are added. Existing
site, known, fallback, writer, privacy and normalized flow/body/header/budget
comparisons remain exact. No arbitrary exception contents become case names,
labels, counts or fixed JSON values.

### Controller retention, bounds and truthful failure handling

All 348 controller lines were reviewed. There is one possible Popen, using
the current interpreter with `-I -B -`, the exact in-memory payload, a fresh
same-owner 0700 empty directory and exclusive/no-follow 0600 stdout/stderr
captures. Child-only RLIMIT_FSIZE is 16KiB; private reads enforce regular file,
owner/mode/nlink/size and a 16KiB cap. The child's fixed result is capped at
8KiB. No source file, main/server constructor, provider/native command or
listener is entered along the reviewed selected-body paths. The controller
defines an API and makes no module-level invocation.

The child arms a 30-second alarm; the controller uses a monotonic 33-second
communicate stop and 35-second envelope, then kills/reaps only its own child
with the remaining budget and records the actual numeric poll result (or null
when unproved). Guards can absorb BaseException, and OS/filesystem scheduling
can stall; these timers are not an unconditional real-time proof. Success
still requires child elapsed <=30, whole controller <=35, child exit0/reaped,
no stderr/cleanup failures and exact completed oracles. No timing/kill/reap
actually happened here. No unrelated process, directory or cache is deleted.

Controller lines 184–218 structurally validate closed v2 result keys, fixed
enums/names, exact integer/bool types, bounded counts, finite nonnegative elapsed
and lowercase 64-hex hash shape. Missing/malformed output is marked unavailable,
never fabricated. It preserves a prior controller failure over later result
unavailability. At lines 265–288 the actual exit, bounded capture byte counts/
hashes and complete structurally valid child JSON are put into the retention
record, written/flush/fsynced through exclusive save **before** lines 299–322
check success/exit/count/timing or returned helper/archive hash equality.
Input payload hash equality is a pre-spawn integrity check, separate from
those post-retention result-pin expectations. No raw stdout/stderr, exception
text, filenames, private input or locals are printed or copied into fixed JSON.

Retention failure returns truthful failure/unavailable evidence. Expectation
failure preserves any earlier first failure. The reviewed save uses an earlier
timing sample; lines 340–348 explicitly return the final post-save sample and
can demote a late result to failed. Root's separate invocation must retain
that final returned sample as well as the files, and select/review its fresh
directory/interpreter/environment/resource preflight. No current runtime
command, resource observation, whole-bound or cleanup proof is invented here.

### Actual static checks and remaining release boundary

EXIT0 source-only utilities independently checked the complete four-assignment
assembly (130,174-byte prefix), all payload/archive/controller hashes and lengths,
strict base64 decode/equality, complete a88 prefix and published/fixed helper
equality. They reconstructed the original 91,163-byte ce79 archive with SHA
`ce79aeb99fafaaeb265d6b7bff55f415e91825d83e9c650eed412825a5a9525f`.
AST parsing passed for the full payload, logic, controller, fixed helper,
unchanged legacy and original archive. Both scoped diffs passed whole forward/
reverse application. Case/group/definition/one-Popen/no-direct-main-or-server-
constructor and observer private-attribute checks passed. Only our static
data utilities ran: no candidate compile, import, exec, definition/case,
constructor, stub, observer, helper or controller execution occurred.

Original prepared180 actual `unexpected_failure` cause, original Authorization
sender and true first cleanup start remain UNKNOWN. The failed 61-of-76 attempt,
historical corrected 78-case legacy pass, all earlier unexecuted designs and
actual GUI/fixture failures retain their dated outcomes and limits. None is
observer-aware evidence. The F1 correction is a source fact; all 128 planned
cases, three captures and fullJSON/exit/timing/reap oracles for this descriptor-
aware envelope remain UNEXECUTED.
Real browser remains HELD. Closed rows, primary assignments and task/board
statuses are unchanged; no completion credit is recommended from this report.

Only this report was appended. No source/controller/other report import/edit,
alignment/merge/main/push, author/worker/orchestrator contact, new task/worker/
WT/managed shell, runtime slot, native/provider/product-CLI/network/socket/
listener/HTTPServer constructor/Driver/Cargo/PG/browser/helper execution or
deletion occurred. RiWork Cua.ai Driver MCP-only preference remains in force.
Final report-prefix, documentation, whitespace, scope and clean-state checks
are recorded below after validation; root owns any separate bounded release.

### Report-only validation receipt

Actual `python3 scripts/check-docs.py` EXIT0: Markdown links and build-directory
layout checked. Actual `python3 scripts/check-repo-hygiene.py` EXIT0: 945 tracked
files checked. `git diff --check` and `git diff --cached --check` EXIT0 with no
whitespace errors. Own static prefix/scope utility proved all 17,324 a770 bytes
and their SHA unchanged, only this reserved report appended, no other staged/
unstaged/untracked path, final newline/trailing whitespace correct and helper
still absent from the own branch. All source/data utilities exited0; none
constructed candidate frames/exceptions or evaluated archived code. No static
check failed or required a correction in this review.

The report-only commit and subsequent clean-state receipt provide the immutable
handoff. Recommend only the separately reviewed/released single memory
invocation anchored to b10cf837 payload and 0e93e11f controller, with full
retention and final timing sample. No invocation or broader campaign is
authorized or performed by this report; root owns release and disposition.
