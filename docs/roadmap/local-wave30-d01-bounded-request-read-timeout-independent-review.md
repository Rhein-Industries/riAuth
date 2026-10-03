# D01 bounded initial request-line timeout: independent source review

Date: 2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Reservation `wave30_D01_bounded_request_timeout_independent_review`.
Existing WT f2e8500e / shell2173637e; entry and parent source
`3414b9f0ce063920dcafba061286ccfcd1bd5f1f`. ONLY this new report is owned.

**Source-only acceptance: no concrete blocker found in the exact reviewed
candidate.** Its first three guarded initial-read timeouts reject and discard
entire connections; its fourth is terminal before observation. The bounded
classification accepts no partial request and cannot satisfy a success check.
This independently reviewed policy differs from my historical 3414 terminal
classification proposal, which remains unapplied and unchanged. Neither
proposal diagnoses an empty connection, sender, browser preconnect or kernel.

Adoption remains HELD on root's separate source reservation, frozen helper and
controller/archive/composition pin review, and exact focused memory design and
validation. Candidate functions, expressions and cases were not executed. No
runtime, signal, socket, Driver, browser, native provider, CLI or Cargo ran; no
slot was acquired/released. Root alone integrates and decides runtime/status.

## Immutable inputs and source/body coverage

The entire 1083-line plan and all 777 candidate lines were read. An initially
truncated middle callback span and template tail were explicitly reread.
Candidate extraction takes its unique full Python fence as-is; the source
already ends LF and no LF was added. Repository files were read through fixed
Git objects, with no alignment or protected-source import into this WT.

| Input | Fixed identity | Bytes / LF | SHA-256 |
| --- | --- | ---: | --- |
| Complete authored plan | `d0f4f39d7e7bb43e83c858f13a4f321031c07d25:docs/roadmap/local-wave30-d01-bounded-request-read-timeout-plan.md` | 57359 / 1083 | `a67d73dd8914c0570680a1ff57bd12bb39715d04c1726390b3d0df562055f198` |
| Full baseline helper | `6f1c2940de4b6f456354812f7d8c24a93100d3fb:scripts/d01-confidential-browser-demo.py` | 35749 / 757 | `75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1` |
| Full candidate fence | authored immutable plan above | 36884 / 777 | `37d32db6fbd8c2c9b676f691d115ecfd1af893724144361971e86f73f573b406` |
| Exact authored four-hunk diff | same plan | 1486 / 28 | `11f7af8c63652869b4dcdbc446527c96aa7fda378ed9ba0f04af242cb001ca85` |
| Independently generated inverse | this review, static DATA | 1450 / 28 | `38350a93eee203028b138093849bc6cfac6a1bfaa336fa9802bc0d71fece7038` |

Baseline helper Git mode is 100644, blob
`a01b1f3f81f0978eb0a02ed339c7248cae485350`; plan mode 100644, blob
`31aa4af812c412feac5974646a7808d11fbd9b67`. Own actual helper equals the
baseline hash and remains untouched. The original CONTRIBUTING and SECURITY
bodies were reread; their published hashes are respectively
7e7dd7b756f734a8105cad5b96dffa8a51977c64f3ecd70b8182fa3de6ba6737 and
2556771d58d09a2a4754e7484645b7e948b84286ef0d21cc66169b920a31c4d8.
No applicable ancestor AGENTS.md exists. The explicit static-only reservation
controls this phase; private-data and actual-environment disclosure remain.

The complete safe 1638-byte root receipt at fixed 6f was reread; SHA-256
7561321eb839f96727fccad204912869bbda8934144e6158f802e523cab02f6b.
It retains FAILED, exact TimeoutError/Handler.handle_one_request/line468,
protected-before403, no confidential journey pass and false whole60 cleanup.
Root's nine-PID/port/lab absence and worker's Driver/session/window facts are
supplied previous evidence, not probes performed here. Historical first event,
sender/request/partial-byte content and exact cause remain UNKNOWN. No new
counter existed in that executed source, so no retrospective count is inferred.
No private captures or credentials were read for this review.

## Exact bounded classification and security witnesses

The new read Try is candidate lines469–486. It encloses only the original
assignment/call at line470: rfile.readline(MAX_REQUEST_LINE + 1). It has one
TimeoutError clause, no else/finally and no other read. The clause tests exactly
nine AST terms, without evaluating those terms in this review:

| Required term | Rejected states keep the old path |
| --- | --- |
| type(timeout) is TimeoutError | Subclasses and other exceptions receive no new classification. |
| attempted is False | True, integer0 or any other value fails the exact identity guard. |
| pending is None | Any pending object fails. |
| cookie is None | Any established cookie fails. |
| subject is None | Any established subject fails. |
| done is False | Terminal or nonexact state fails. |
| failure is None | Any prior failure fails. |
| type(preflow_request_timeouts) is int | Boolean and custom numeric values fail. |
| 0 <= preflow_request_timeouts < 4 | Negative, 4 and larger counts fail. |

On any mismatch, a bare raise reaches the unchanged original terminal
Exception handler. Exact int0..3 increments once to int1..4. At counts1–3,
only this counter and the existing request stage change; the branch does not
alter done/failure, checks/statuses, attempted, pending, cookie, subject or the
Authorization refusal count. At count4, lines480–485 set failure request_timeout
and done True BEFORE invoking the original protected observer. The clause ends
with a value-less return at486. There is no parse/reply/route/secret/header/path
access in this clause, no exception-content access and no resumed read.

The counter has exactly its new initialization, guarded increment, record
initialization and final record copy. With the reviewed trusted Demo writes,
it remains int0..4. The main while-not-done loop cannot start a fifth request
after count4. A manually supplied counter4 is also rejected by the guard,
without increment. Direct field corruption is not an accepted production
state or a promise of well-typed evidence; synthetic invalid-counter tests
must assert refusal and zero recovery increment, not fabricate normal state.

The return occurs before the original request-line length guard, HeaderReader,
parse_request, Host/Authorization/framing/target checks, GET/POST dispatch,
cookies/begin/callback or reply. A partial request is discarded, never accepted
without its guards. Any later successfully read request on a fresh connection
still traverses the identical original guards. Header/parser and reply timeouts
fall outside this dedicated Try and retain their old terminal outcomes.
There is no added body-read path: nonzero content length and prohibited framing
remain refused, with only the original None/0-length path permitted.

The old Authorization counter remains independent: first-three preflow 403
refusals, fourth terminal, and post-flow Authorization refusal are unchanged.
Only the unchanged valid POST/login/Origin/content-type/cookie path can begin
the one flow. Existing nonce/state/S256, confidential exchange, native signature
and token checks, UserInfo subject, and fresh protected cookie stay mandatory.
The new integer is outside the checks dictionary. Main still rejects demo.failure
before requiring all original checks; count or diagnostic can neither supply a
check nor make a failed/fourth-timeout fixture pass.

## Entire stream disposal; native and cleanup limits

Candidate Handler462–547 sets close_connection True before any read. The
new return leaves that flag True, so BaseHTTPRequestHandler.handle has no
same-stream second iteration. Each accepted connection constructs a separate
handler. BaseRequestHandler's original finally runs StreamRequestHandler.finish,
which closes its read/write file objects; DemoServer.process_request's original
finally invokes shutdown_request and then clears active. TCPServer's original
shutdown/close disposes of the socket. Any partial buffered bytes stay inside
that disposed stream; no field, reader or buffer is transferred to a new one.
The design never tries to repair or reuse a timed-out buffered file.

These are source paths, not actual socket-close or host-kernel evidence. If
finish/shutdown raises, active clearing is not unconditionally guaranteed:
shutdown_request precedes active=None. Existing server/main terminal handling
and cleanup flags remain responsible; a failed close is not credited as a
successful rejection cleanup. The new catch does not intercept such failures.
A future finite test must distinguish normal disposal from injected close
failure, instead of asserting active cleared after every failing shutdown.

Relevant complete installed CPython3.14.6 source bodies were reread, without
import/version/library/runtime calls: BaseHTTPRequestHandler.handle and
StreamRequestHandler/BaseRequestHandler setup/finish, socketserver request
handling and TCP closure. Source fingerprints match the previously pinned
read: http/server.py 53122 bytes SHA
b917e19d333ae8aa1995063c9bddf5a15391f8f7caf32e41a22067824d643429;
socketserver.py 28065 bytes SHA
ecbbe1a633801460399a8f10b39007aa0e13cdbdad507e39414000f097b769b5;
socket.py 37536 bytes SHA
873ff8912562c0375e75a7301024e61e878eef101b566dc0f3b6644c81874c8e.
These installed-source identities do not attest the earlier fixture's entire
runtime stack. [Pinned CPython handler lifetime source](https://github.com/python/cpython/blob/v3.14.6/Lib/socketserver.py).

No empty/idle/sender/Chrome inference is needed for this classification: it
rejects the whole timed-out request, including any partial bytes, under the
explicit finite preflow bound. Its first-three counter is rejection metadata,
not an accepted request, route, authentication, cookie or journey attempt.

## Budget, interrupts and observation

All Budget bodies, Handler.timeout=5, request and response limit5, validated
absolute helper deadline1..600, pending180 and disk stop8.5GiB remain exact.
The branch does not construct a Budget, mutate deadlines, sleep, retry the
read, renew its operation or restart the absolute start. Returning follows the
existing context unwinding; the next listener loop has its original global/disk
tick. A fresh accepted connection uses the original per-operation limit,
subject to the same absolute deadline. A phase/global/pending Halt from entry,
read or exit is not caught by TimeoutError or ordinary Exception; it remains
terminal. Native timeout and alarm ordering can still terminate before all
three classified refusals; this design does not promise three recoveries or
fourth-tag precedence over an independent deadline failure.

Counts1–3 call no observer and introduce no BaseException catch on a newly
continuing path. Thus observer-time interruption cannot be swallowed and then
allow one of those paths to continue. Count4 retains the existing observer's
BaseException fallback, which may suppress an exception occurring within the
observation itself. It runs only after terminal state is established, so it
cannot create continuation or a passing result. This is a precise preservation
claim, not a claim the old fallback never catches Halt. Direct interruption
elsewhere keeps its original unwind/cleanup path.

The first three update only safe private count metadata; they do not latch,
clear or call the original unexpected observer. A later genuine fatal event
therefore can still become its first finite observation. Count4 invokes the
same first-only builtin-descriptor/code-identity/class/64-frame/1024-line
projection; an existing observation remains unchanged. The original handler,
server and main observation calls also remain. The exception variable is not
saved or formatted. No request line, headers, cookies, URL, subject, private
values, exception text/args, traceback/path/locals or sender data is emitted.
The new record key lives in original exclusive0600 helper/private outer evidence;
existing stdout and public projection shapes do not expose the counter.

## Exact composed/controller boundaries and adoption prerequisites

The full original controller shell template at immutable
`2e29c30d01ccd41a2aac3914e3ac20afa38d324f` was read, including its tail:
17326 bytes /225 logical lines /224 LF, SHA
5eb6ab2d3d23b49fd431c41f1932082fe0c6779ad724ac77b7fded10926b26b0.
Its finite_observation/retain_observation, helper result/exit loop, source hash
check, actual child stopping/reap, private persistence and final numeric exit
bodies are unchanged. Nested helper_result retains the new integer privately.
Neither observed=true nor the count changes result or helper_exit. Nonzero
helper exit still refuses; a zero exit still requires the actual original
helper checks and browser final checkpoint. No template function was run.

The complete113395-byte/1241-LF a270 DATA source was reconstructed from the
unique 251a29 fence plus the reviewed two entry-pin literals; SHA
`a270153f635393085ccd553a2ba18c4ee66f844d17444013c19ed95c3201809e`.
Its unchanged33701-byte suffix remains505b1c0fec96e248dc67db46379bd801fa06a2678462aa094fb2fc1bc5ea398f.
Complete five collectors CLOCK, STOP, READBACK, MARKER, PERSIST and the full
L_ARCHIVE_SOURCE were decoded/read as source DATA and never called. Relevant
complete composed bodies read: finiteObservation793–810, diagnostic811–828,
latch829–835, observeController836–887, pollController888–900,
cleanup926–1026, page1044–1076, entry1086–1119 and final output branches.
These retain numerical exit before comparisons, first-failure latch before
await, safe finite diagnostic only, exact owned join/readback and page proof.
The earlier successful protected-before page cannot erase a later helper
failure. No count/observation creates an application or journey success.

Original prepare180, active840, inclusive900 and cleanup30/15/5 allocations
remain. The cleanup collector marker is observational, not a proven first
true event; source allows essential join after60 and explicitly retains
first_event_unproven/whole_cleanup_within60_proven false. Independent actual
absence does not convert that false proof into a whole60 guarantee.
Current source/memory/browser rows are not changed by this review.

There is a concrete, acknowledged adoption prerequisite, rather than a
candidate source blocker. Old a270 remains bound to helper75b/35749:

| Frozen witness | Required separate source adoption before any future journey |
| --- | --- |
| L_CONTROLLER_TEMPLATE line122 | Expected helper hash75b would reject37d. Adopt only the reviewed immutable helper identity. |
| L_ARCHIVE_SOURCE lines33–38 | stat size35749, bounded read35750, length35749 and hash75b must all match the new36884-byte helper. Rebinding hash alone is insufficient. |
| Archive source template check | The old controller-template hash5eb must match the separately reviewed changed template, not a mutable computed trust value. |
| lReleaseValid line36 / pSeedValid line426 | Both strict controller_template_sha256 literals expect5eb; truthful changed-template release/seed input must be separately reviewed. |
| Complete composed/embedded source receipts and future tests | Record exact revised source hashes; preserve unchanged entry projection and all original policy bytes except root-reserved binding hunks. |

No binding literal, input, archived template or source file was changed here.
Root must review the exact dependency chain, sizes/read cap/hash/readback and
canonical source receipts before separately authorizing a launch. Pin rejection
is not an excuse to loosen identity validation. My3414 diagnostic candidate
is historical/unapplied and is not the reviewed37d source.

## Whole source and AST protection proofs

Independent zero-context patch application reconstructs the entire candidate
from the exact baseline with four hunks. An independently generated inverse
reconstructs all baseline bytes, including its final LF. The canonical fence
and both patch identities are verified; no source LF normalization is needed.
AST inversion performs only four declared changes: remove the new Demo counter
initialization, replace the one-read Try by its sole original Assign, remove the
one main record key/value, and remove the one final counter copy. The resulting
whole AST equals the baseline with only source location attributes excluded.

All35 functions are present in the same order, with no additions. The changed
ones are Demo.__init__272–284, Handler.handle_one_request462–547 and
main627–773. All32 others are byte- and AST-identical:

```text
Failure.__init__ Halt.__init__ require private_directory private_bytes load_verifier
client_secret Budget.__init__ Budget.__enter__ Budget.interrupt Budget.tick
Budget.limit Budget.close observe_unexpected_failure Demo.invoke Demo.setup
Demo.begin Demo.callback Demo.clear HeaderReader.__init__ HeaderReader.readline
DemoServer.__init__ DemoServer.process_request DemoServer.handle_error
Handler.log_message Handler.record_request_reason Handler.require_request
Handler.send_error Handler.cookies Handler.get Handler.reply QuietParser.error
```

The remaining original statements/handlers of the changed Handler and main
are recovered by whole-AST inversion; all corresponding unaffected bytes are
also recovered by exact patch inverse. Top-level constants/imports, verifier/
provider/crypto pins, fixed HTML, private-file protections, route/header/auth,
receipt/removal/audit protections, cleanup and every original success check
remain intact. The read Try body is exactly the original call/assignment.
Structural checks confirm nine exact guard terms, bare raise on mismatch,
int increment, terminal assignment BEFORE fourth observer, no first-three
observer, no parser/reply/read/route call in catch, and final value-less return.
They parse/compare syntax DATA; no condition/function was evaluated.

## Focused future memory checks: required, not authored or executed

The plan's focused scenarios are appropriate. A future separately reviewed
finite envelope must bind the exact complete37d source, preserve original
accepted/refused guard/sink cases, and retain actual bounded result/exit before
grading. It must not import/main/run the helper, construct an HTTP server,
open a socket or call crypto/provider. Main remains definitions-only; counter
initialization/final-copy checks can inspect source or carefully selected
owned sinks, not invoke an entire fixture. No target count or runtime command
is released by this review.

| Finite risk | Meaningful required oracle |
| --- | --- |
| Partial-buffer prefix then exact initial-read TimeoutError | No parser/header/route/reply/content observation; close flag staysTrue, buffer not reused, no private sentinel serialization. Never label it empty. |
| Guarded1/2/3 then fourth on fresh handlers/same Demo | Exact count bound; first3 no checks/statuses/flow/observer mutation; fourth done/failure BEFORE an observer that can throw; no fifth listener continuation. |
| Every nonexact state separately | Each guard identity/range independently refuses; no broad truthiness or bool-as-int allowance. |
| Subclass/other exception; parser/header/reply timeout | Old terminal handling/class projection; no counter increment. No fictitious new body-read test. |
| Halt at entry/read/context exit and close errors | No recovered continuation; unchanged tag/terminal/cleanup behavior; injected shutdown failure must remain unconfirmed/failed, not falsely active-cleared. |
| Later fatal after recovered refusal, prior observer, observer absent/failing | Genuine first fatal or prior latch retained; actual nonzero/helper_failed survives. First3 must never call observer fallback. |
| Successful/EOF request and original auth/framing/cookie guards | Original outcomes preserved; timeout refusal supplies no journey flag. Fourth Authorization and all postflow rules unchanged. |
| Private count/failure-before-Demo and privacy traps | Initial0/copy under valid Demo lifecycle; no raw exception args/str/repr/trace/sentinel/header/URL/state output. |
| Frozen source pin and real completion boundaries | Old75b rejects37d; revised truthful pins require separate review. observed=true/count alone neither latches failure nor passes journey; actual checks/exit/page still required. |

Native disposal and real Driver journey remain a later distinct acceptance:
source control flow and fake sinks cannot prove actual connection close,
cryptographic interoperability, sender or whole cleanup timing. Existing real
failure and whole memory successes remain dated, separate outcomes.

## Actual static checks and limitations

Entry docs and tracked-file hygiene exited0. Candidate/plan/patch identities,
full four-hunk forward and inverse application, complete byte/AST inverse,
35-function inventory,32 complete protected bodies and dedicated catch/guard
structural checks passed. No proposed code object/import/function/guard/case/
VM/harness/main/controller/collector was executed. No compiler or typecheck ran.

One auxiliary AST-call check initially exited1: it assumed ast.walk returns
calls in source execution order. Actual traversal was observer,type(timeout),
type(counter), though the statement tree already proved terminal-before-observer.
This was a review-check ordering error. Comparing the exact call multiset and
retaining the separate ordered statement-tree assertions produced exit0 with
no candidate/data changes. It is not a repository checker defect or a runtime
failure. Author's reported earlier whitespace failure/re-encoding remains
retained in the immutable plan; it is not an execution result of this review.

No concrete candidate correction or weakened guard is proposed. The future
memory design and frozen identity adoption are required gates, with runtime
HELD. No worker contact, alignment, source materialization, private transfer,
process probe, signal, provider/native/HTTP/CLI/Cargo/Driver call, resource
acquisition, new task/worker/WT, main/push or task-status mutation occurred.


Final precommit static receipt: docs checker exited0; tracked-file hygiene
exited0; whitespace check exited0. Sole new path/empty index/current3414 parent,
actual helper75b unchanged, historical report SHA
 af63ea51acfda1e4f9a7a9c31b0d88d157891edd0afcd6617806b471fc8b65e3
unchanged, and new report0644 checks exited0. Cached new-file whitespace and
sole-path checks follow staging; no proposed execution is included.

Cached whitespace/sole-report checks exited0; tracked hygiene with the staged
new report exited0 (1060 files); final docs checker exited0. These checks
introduce no source or runtime acceptance beyond this independent review.
