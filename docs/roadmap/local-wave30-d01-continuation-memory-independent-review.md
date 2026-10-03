# D01 continuation memory envelope — independent source review

Date: 2026-10-03, Europe/Vaduz. Reservation
`wave30_D01_continuation_memory_independent_review`; project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, existing isolated worktree
`ed9ac424-59f4-4520-905b-919aea3521eb` only. Clean entry parent:
`e04cb470dab863ebd1b59d57db13771bad428e8b`. This new report is the sole
write. No alignment, source edit, artifact/private-input read, worker contact
or runtime reservation occurred.

## Disposition: hold the exact controller for one correction

The complete harness is a source-acceptable bounded consumer of the exact old
9f cell, with meaningful F2/F3/F4 and security-negative cases. The controller
has one concrete reachable bookkeeping defect: normal EOF is consumed and
the stream closed without marking `stream_eof[kind]`. A normally completed
child therefore cannot satisfy the subsequent full-output retention gate.
Hold the exact assembled invocation pending the one-line correction below
and review of its new controller identity. No candidate/case/controller has
been executed by this review; this is source diagnosis, not a failed runtime.

The correction does not require a candidate, case, oracle, timeout, cleanup,
privacy, source binding or product change. It records the actual empty read
already observed on the existing out/err branch. It must not replace the EOF
gate with an unconditional success or weaken output-cap handling.

### F1: normal output EOF is forgotten before retention grading

Immutable controller line 2043, document line 3976, is:

```python
                    if not part:selector.unregister(stream);stream.close();continue
```

`stream_eof` starts as `{"out":False,"err":False}` in `main`. On the normal
successful path, each stdout/stderr pipe reaches an empty `os.read`, is removed
from the selector and closed here. When both streams and stdin are removed
and WNOWAIT has observed the child exit, the loop ends. The finally drain
only visits streams **still registered** in that selector. Its separate empty
read at controller line 2067/document line 4000 sets the flag, but it cannot
visit a pipe already removed by the normal branch.

The receipt records the false flags. After all bounded raw outputs and the
receipt are fsynced/closed, controller lines 2088–2089/document lines 4021–4022
require both flags and latch `child_output_incomplete` otherwise. Even a full
valid 25-case child packet with exit 0 and complete owned cleanup is refused
on this normal path. This is an archive-source inference; no actual child
outcome, timing or historical failure is attributed to it.

Smallest proposed source seam, controller only:

```diff
@@ -2043 +2043 @@
-                    if not part:selector.unregister(stream);stream.close();continue
+                    if not part:stream_eof[kind]=True;selector.unregister(stream);stream.close();continue
```

The branch is inside the out/err arm, so `kind` is an existing EOF-map key.
Assignment occurs only after the actual read returned empty, before the
unchanged unregister/close/continue. In-memory text reversal restores the
whole immutable controller; Python AST parsing of this prospective seam
passed. The corrected controller was neither materialized nor executed.
Root/source owner must regenerate and review its exact archive hash before
any separately released invocation. The original 36ac controller hash remains
the reviewed defective source identity, not a corrected/pass identity.

## Exact objects, body reads and encoded-data checks

The reviewed object is
`36ac893435b3aa05556d43e8576851ced20c176e:docs/roadmap/local-wave30-d01-continuation-correction-plan.md`.
Its new appendix begins at line 963. I read the complete new design prose,
the complete 476-line readable harness and all controller executable statements
and guards. Encoded constants were parsed/decoded as source DATA and compared
to immutable objects; I did not pretend that reading a base64 line is a body
review. A first combined display included oversized encoded lines and was
truncated; subsequent reads selected all readable harness/controller spans
and separately inspected their complete data bindings.

| Exact archived unit | Bytes / LF lines | SHA-256 |
| --- | --- | --- |
| Complete report | 427496 / 4127 | `edf20d1280f3edf24389e3b7367bdccc5bcf475a87bab12c0d3c9e045a3d4071` |
| Full Node payload, document lines 1200–1922 | 132725 / 723 | `46c19af829c546cf0f32f0a587db084dd00d304ea9d508c98ccda0b2a6028866` |
| Complete readable harness suffix | 26197 / 476 | `639df7f15a1f8135b57ecbacb15062cdc75da100530e4c062e856e7ed2a87054` |
| Full Python controller, document lines 1934–4057 | 209948 / 2124 | `e3bde0116783eccd01bc6c9c3183c06ade99af21d95800fa47327de9c1ca2207` |
| Decoded whole 9f candidate | 32502 / 485 | `7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8` |
| Decoded whole c5 baseline | 31581 / 467 | `d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d` |
| Decoded four-hunk diff | 2243 / 52 | `395948657075c7fb27503b507d18303085755af9e531303197b964adc7ecf144` |

The first 61754 report bytes equal the entire original
`9f3a4a372b53fc6d73d5df45ad1a8d217ad60879` report, SHA-256
`fd32648eac52c94add32138f4721a96e16f56ce4d6858964363516160410a1db`.
Its complete cell was previously fully read in my
[continuation source review](local-wave30-d01-continuation-correction-independent-review.md).
The decoded candidate equals that exact readable cell; the decoded baseline
equals the complete c5 archive at
`c5d4d173857d040947a8cb3780c47c7141f98336:docs/roadmap/local-wave30-d01-user-browser-review.md`,
document lines 9142–9608. The decoded diff equals the readable original diff.

All three base64 source strings are canonical round trips with their declared
bytes/hashes. Four unique reverse edit anchors reconstruct the whole c5
source. An independent coordinate/count/line-checking diff applicator also
reconstructs the whole 9f source from c5 using all four hunks. The controller's
complete `PAYLOAD_B64` literal decodes to the exact 132725-byte readable Node
archive, not another harness. Its literal PLAN, CHECK_NAMES and EXPECTED_SOURCE
match the complete Node BINDING data, with no alternate source import.

I parsed both full cells, the full Node payload and the full prospective async
wrapper with Acorn. The wrapper's complete statement-array AST equals the
candidate's complete module statement-array AST, ignoring source locations
and normalizing BigInt data only. Candidate AST SHA-256 is
`786f743a839085292b57dc0c695a97c338d57eef9ab314168ff081da3e100a74`;
c5 baseline AST SHA-256 is
`28a72ac4b2b27a5c6f6aeee63883557580e4b63e9de097c9339e9dd9fd33d750`.
No VM or async wrapper was constructed or evaluated in this audit.

The five decoded collector bodies equal the exact full candidate literals and
their c5 declarations. Each was Python AST parsed only:

| Collector | Bytes | SHA-256 |
| --- | ---: | --- |
| CLOCK | 114 | `cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d` |
| STOP | 1600 | `5de5244440b134689bedfb1235e805c65a8b726e753cbdac1d0831ef1c83d071` |
| READBACK | 4424 | `b7d3504245ae94a584b455fa65826fbaa7759ec7a50a7b81c2cf8284c9f9051f` |
| MARKER | 626 | `13c6549e0328ad695769938cbc0be7eb47d7a10ea9f00c4d897655603437a949` |
| PERSIST | 805 | `819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30` |

## Complete harness: guards and meaningful case oracles

The harness creates one `vm.Script` containing the entire fixed candidate.
Each cell gets a fresh VM context and the same cloned synthetic store; strings
and wasm code generation are disabled. Only node:vm, node:crypto and
node:perf_hooks are imported by the prospective payload. AST inspection found
no dynamic import or require in it. This is a reviewed-source memory consumer,
not a sandbox guarantee for arbitrary code or a product integration run.

Tools are finite local stubs. Collector commands are decoded as quoted DATA,
matched against the exact five literal bodies and never sent to Python/shell.
Store/load keys, synthetic handles, target/tab/session, ref namespace, click
route, replacement input, username/password routing, owned kill/end/windows,
marker/stop/readback/persist shapes and pending-command refusals are checked.
Host assertion objects are recorded in a WeakMap and remembered by the bridge
even if the candidate catches them; a host failure stops later cells/cases.
Injected private exceptions are expected synthetic inputs, not host assertions.

The source limits are 75 cells, 2500 counted tool calls and 512 trace entries
per state, with persistent real-clock checks. The exact candidate's synchronous
VM timeout and async deadline race share remaining real budget. A synthetic
Date.now controls only candidate boundary scenarios, never the outer budget.
There is no reset of the retained candidate start to make a second cell pass.

All 25 SwitchCase names exactly match the declared ordered plan. The groups
are phase_binding 5, secret_lifetime 3, helper_handoff 5, partial_framing 7,
cleanup_latch 5. These are declared cases, not executed passes:

| Case | Meaningful checked behavior |
| --- | --- |
| phase_entry_then_continuation | Full entry keeps numeric receipt before readiness, changes phase, and permits a second full cell with unchanged start. |
| password_survives_until_password_dispatch | Username, click and snapshot cells preserve password/no clears; a fourth password cell routes the exact synthetic value and consumes it once. |
| missing_password_refuses_without_input | Missing private input cannot reach browser input; fixed refusal and cleanup remain. |
| unowned_context_refuses_without_operations | Original context refusal is returned with zero stub operations. |
| undeclared_ref_refuses_input | Ref outside the accepted snapshot cannot dispatch input. |
| stale_ref_cannot_cross_cells | A consumed ref is replaced by the new snapshot and cannot dispatch in a second cell. |
| invalid_kind_repeated_cleanup_clears_before_await | Natural repeated cleanup clears twice; one stop protocol observes null before/after its await. |
| helper_zero_final_snapshot_then_cleanup | Queued zero exit permits exactly one bound final snapshot with full protected predicates, no input/navigation, then cleanup. |
| helper_zero_missing_page_still_fails | Zero exit without protected page proof refuses independently of helper success. |
| helper_nonzero_final_still_refuses | Nonzero exit refuses before the final snapshot. |
| helper_zero_other_kind_still_refuses | Zero exit with click decision refuses before any browser action. |
| helper_zero_prepared_phase_still_refuses | Zero exit in prepared_entry refuses without navigation/snapshot/input despite a final decision. |
| partial_line_drains_before_output_and_next_cell | Post-decision split line is processed before return; a fresh second cell sees no raw partial carry. |
| partial_exact_cap_is_accepted | A split 16384-character line is framed at the exact cap; unknown private fields never survive projection. |
| partial_over_cap_refuses | A 16385-character line retains fixed invalid-observation refusal. |
| partial_joined_controller_refuses | Partial data alongside controller completion preserves original completion refusal. |
| partial_unavailable_controller_refuses | Private polling exception with a pending fragment becomes fixed unavailability and cannot invent release. |
| partial_deadline_prevents_extra_poll | At active deadline no extra active poll occurs; essential cleanup join is counted separately. |
| partial_completed_late_still_refuses | Completion at the exact active deadline still refuses after the drain. |
| retained_start_budget_refuses_next_cell | A second cell at the retained active deadline cannot take another snapshot. |
| inclusive_cleanup_deadline_does_not_invent_join | At the inclusive whole limit, controller join remains unproved and release false. |
| first_page_failure_survives_later_helper_error | Later cleanup helper error cannot overwrite the first page refusal. |
| missing_absence_proof_prevents_release | Incomplete absence proof independently withholds release. |
| cleanup_exception_is_private | Driver exception becomes fixed cleanup error; separate complete synthetic absence can prove release without journey. |
| pending_metadata_command_prevents_release | An unjoined metadata receipt withholds release despite other absence fields. |

These consumers would distinguish the reported old F2/F3/F4 behavior from the
corrected full cell; they are not merely mirror-function comparisons or count
assertions. Case completion is appended only after its assertions return.
The first unexpected failure stops the ordered plan; attempted/completed/
unreached and group totals reflect progress. Counters are not extra cases or
performance evidence. All cases explicitly avoid whole60/journey success;
release evidence is distinct from the first refusal.

## Privacy and controller protocol

Synthetic password, partial-content and exception sentinels remain internal.
Public text, owned/observation store values, metadata and traces are scanned
for all three. Type tracing retains kind/password_matches, not typed text.
The pending private-input store is intentionally excluded until consumption;
STOP checks clearing before its first await and after it. Host failure catches
read no arbitrary message, stack, repr or string conversion. The final packet
projects only finite source identities, plan/progress, counts and fixed failure
labels. A final sentinel check refuses rather than serializing a private value.

The controller has one prospective Popen site, new owned process group, empty
private cwd and PATH/LANG/LC_ALL only; NODE_OPTIONS/NODE_PATH and credentials
are not forwarded. Its launcher path/hash are declared pinned inputs, not a
binary/dylib/artifact attestation newly performed here. No launcher/artifact
was read for this review. The controller rehash/check behavior itself was
reviewed as source only.

START_NS precedes assembly/preflight; fixed START+30 and START+35 deadlines
are never reset. Nonblocking stdin writes and stdout/stderr reads are capped
at 65536/16384 bytes; evidence writes are capped at 262144. Future work requires
the exact own source worktree, an existing owned nonsymlink 0700 private
directory and fresh 16-hex public basename. It creates one exclusive 0700
subdirectory and four exclusive 0600 files, with no overwrite/deletion. The
8 GiB capacity floor is checked before launch and sampled during child work;
actual capacity/peak/runtime remain unmeasured.

The controller refuses missing WNOWAIT APIs before launch and keeps the owned
leader unreaped until checking its group identity and signaling only that
created group. It joins/reaps and checks group emptiness; no newly discovered
PID/group is signaled. Availability of these APIs and actual cleanup are not
proved by AST parsing. The finally branch performs only bounded immediately
available pipe drain. F1 concerns recording EOF on the earlier normal branch,
not removal of exit/group cleanup or output caps.

Actual child exit, bounded raw stdout/stderr and hashes are written/flushed/
fsynced/closed before JSON parsing or grading. The receipt preserves first
failure and raw bounded failed output even if the packet refuses. Closed
packet validation rejects duplicate keys/nonfinite JSON and requires exact
top-level keys, plan/progress prefixes, derived group counts, actual integer
types, finite counters/elapsed time, fixed failure catalog, scope booleans and
exact source key/value/types. Boolean 1 substitution cannot satisfy source
identity or integer counters. The finite check catalog has 82 labels; static
ensure labels are covered, with unknown-case fallback mapped to unexpected.

After grading, review.json is explicitly provisional until fsync/close and
the final clock sample. Success also requires child exit 0, full 25-case
packet, no first failure, reaped/empty group, retained output and inclusive
within35. No evidence/child operation follows that final sample. The final
public return and interpreter transport are not hard-cancelled by this source;
blocking filesystem/preflight work can exceed a deadline and must fail rather
than become timing proof. F1 currently prevents the normal full-output gate
despite the otherwise correct receipt-before-grade ordering.

## Checks performed and preserved limits

Independent source/data checks exited 0 for report/payload/logic/controller
sizes/hashes, entire old prefix equality, canonical decoded sources, complete
controller-payload equality, all four diff hunks/anchors, case/group/catalog
bindings, full-wrapper Acorn AST identity and five exact collector literals.
Full controller and collector Python AST parsing passed; no compile/eval,
imports, VM construction, case or function under review ran.

One initial EOF AST selector exited 1 because it also selected the launcher's
file-hash `if not part` branch outside main. Narrowing this independent query
to the already parsed `main` AST then passed: normal EOF line 2043 has no flag
assignment, cleanup EOF line 2067 has one. This was a corrected static selector,
not a runtime failure or a source/case correction. Prospective one-line seam
parse and full text inverse passed without materialization. The author's
earlier unused-draft selector and oversized-response assembly issues remain
dated source-preparation failures in its preserved report.

The [public projection review](local-wave30-d01-public-projection-independent-review.md)
and its e04 commit are untouched. This harness runs **old 9f**, not later Sol3
projection or Sol4 preparation/private-input composition. It cannot validate
actual typed role/name/ref association, browser behavior, partial ownership,
native collectors, real PID/listener absence, RP callback/issuer/nonce/token
delivery, credential authorization or whole D01/D05 outcome.

Historical 18-case old-cell cleanup memory, 128-case descriptor helper memory
(50 observer/78 legacy), and earlier 78 legacy results retain their own pins
and scopes. They are not any of these 25 cases. Earlier failed designs and
c88 prepared-180 actual failure remain failed; sender, cause, lost values,
true first-cleanup anchor and whole60 remain UNKNOWN/unproved. The exact
840/900 synthetic boundary tests do not prove real cancellation. No deployed
HA, full-lifetime external IO fence, paused-process, tenant/release or GUI
journey claim follows. Existing security/receipt/header/PAM, Group hold,
nonrenewed60s and closed original rows remain unchanged.

No candidate/harness/controller/function/case/helper/import/module main,
memory VM/stub, native library, product CLI, Driver/browser, HTTP/network,
PG/service or Cargo runtime ran. Only independent parser/data programs and
documentation checks executed. No runtime slot was acquired/released and
no author/worker was contacted. Root alone owns any correction reservation,
updated immutable source review, one future invocation and integration/status.

`python3 scripts/check-docs.py` passed at entry with no preexisting flags;
the author's five private-target-layout flags remain its dated worktree
result, not this worktree's result. Report-aware docs, exact reference checks,
LF/trailing-whitespace, Git scope/whitespace and clean post-commit checks
accompany the handoff. The final commit adds only this report; every other
tracked file and historical report remains equal to entry parent e04.
