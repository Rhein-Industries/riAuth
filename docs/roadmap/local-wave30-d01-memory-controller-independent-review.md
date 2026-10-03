# D01 memory controller cleanup and packet independent source review

Reservation `wave30_D01_memory_cleanup_packet_independent_review`, project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, existing Sol3 worktree
`f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. Entry was clean at
`16032bdb992067889f56cd072df1ce0f1812ebc9`. Only this new report is owned.
The review date is 2026-10-03. No source alignment or executable correction
was made, and no controller, proposed function, VM, harness, case, Node
child, process-control operation or native/browser runtime was executed.

**Decision: packet-bound correction and protected-source preservation are
accepted at source level; the live-child signaling proof has one unresolved
prerequisite, F1 below. Keep the proposed cleanup mechanism held pending
root's resolution of that prerequisite.** This is not runtime authorization
or a claim that the prerequisite failed on the actual host. The remote A09
slot remains root-owned; this review acquired or released no slot.

## Immutable inputs and complete read coverage

The independently read proposal is commit
`74a8fff613c0edd6b4f842f380bd7125b4582c85`, immutable file path
`docs/roadmap/local-wave30-d01-continuation-correction-plan.md`.
I read the complete 792-line / 46278-byte new appendix, its complete
6988-byte diff, all three complete changed function bodies, and all 85
lines of its archived static verifier. I reconstructed the entire proposed
controller as data from the complete e443 controller and those bodies.
The base64 payload is data, not a function to run: every encoded byte and
decoded payload byte was compared, and the complete 476-line decoded
harness logic was read, including all 25 case definitions. I also read the
32502-byte candidate's control/command/framing/cleanup bodies and decoded,
hash-checked and AST-parsed its five Python collector literals. No collector
was imported or called. Encoded data identity is distinguished from reading
the executable logic that it represents.

| Input or reconstruction | Bytes / lines | SHA-256 |
| --- | ---: | --- |
| Complete e443 report | 649834 / 6470 | `afff04fedb1dc87eddab804df0147ab00415013414c6446e72da92b44d0bb98d` |
| Complete 74a8fff report | 705687 / 7431 | `15835cc90eb3e1f1252d71eb679dd10bfd4edf36533a750b7b8e480860918cfc` |
| Preserved 515 report prefix | 659409 | `7e56a225b3d736cab46640a5f89301c30435baa7208b075b4f7b0fea555f655c` |
| e443 corrected-EOF controller | 209970 / 2124 | `4c62dfc156a6b6ad5e4528d48124c926210f0abffdd2f6d8538bded788be090e` |
| Whole proposed controller | 213188 / 2183 | `98c270c461afccce0483b003cd41178d268a16e49ce33abfa48da031d75d1117` |
| Complete prospective diff, four unified hunks / five edits | 6988 / 126 | `2b7107674313e9079b00909cb1dfe490cbdd7378587be08aeecf6fab0ec0e301` |
| Unchanged complete old25 payload | 132725 / 723 | `46c19af829c546cf0f32f0a587db084dd00d304ea9d508c98ccda0b2a6028866` |
| Unchanged harness logic, including its leading LF | 26197 / 476 | `639df7f15a1f8135b57ecbacb15062cdc75da100530e4c062e856e7ed2a87054` |

The exact base commit is `e4431d4f85eec7add72dec0183b9784917e451c3`.
Its controller fence starts at report line 4198. The new proposal begins
at line 6640; narrative is at 6641–6839, diff at 6840–6967, complete
`closed_packet` at 6977–7037, `group_cleanup` at 7041–7114, `main` at
7118–7252, preservation proof at 7254–7301, verifier at 7302–7388 and
checks/limits at 7390–7431. These are immutable report witness spans,
not links to a newly materialized executable.

I separately read public historical receipt text at proposal-report lines
6472–6585 and 6612–6640. The seven private captures were not reread, opened,
parsed, probed or modified. Their already recorded metadata is not new
independent capture verification here.

## F1: handler-only check does not establish the live branch's waitability premise

In reconstructed controller lines 1997–2021, `eligible` requires the exact
Popen handle's positive strict-integer PID and `returncode is None`.
The only signal-disposition guard is
`signal.getsignal(signal.SIGCHLD) != signal.SIG_DFL`. After a WNOWAIT
query returns None, `getpgid(child.pid) == child.pid` establishes live
group identity, sets `owned=True`, and permits the nonzero group signal.
There is no positive terminal WNOWAIT event on that branch. Its safety
therefore depends on the child remaining waitable if it exits between the
live identity query and the signal.

The proposed report explicitly states this premise at lines 6772–6774:
default SIGCHLD plus a single source reaper prevents exit from recycling
the PID. The source guard establishes the Python handler value, not the
separate no-automatic-reap condition. CPython 3.14.6's
`signal_getsignal_impl` returns its stored handler, and initialization
records the handler returned by `PyOS_getsig`; it neither exposes nor
tests native sigaction flags. `PyOS_getsig` returns `sa_handler` alone.
Thus `SIG_DFL` does not distinguish a default handler with the
`SA_NOCLDWAIT` flag. These are source facts, not observations of this
interpreter's signal state. [CPython signal source](https://github.com/python/cpython/blob/v3.14.6/Modules/signalmodule.c),
[CPython signal wrappers](https://github.com/python/cpython/blob/v3.14.6/Python/pylifecycle.c).

General current XNU `setsigvec` permits `SA_NOCLDWAIT` independently of
`SIG_IGN`; its exit path has a separate automatic-reap treatment for
`P_NOCLDWAIT`. This makes the omitted condition relevant to the source
proof: a live waitid query can return None while the later exit does not
leave the parent a waitable child. Successful later reaping cannot make
an earlier group signal safe retroactively. This is an inference about
the predicate's sufficiency, not evidence that the historical process
had those flags or that a reused PID was signaled.
[Apple signal source](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_sig.c),
[Apple exit/wait source](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_exit.c).

Counterevidence matters: current XNU `forkproc` copies only a selected
`p_flag` mask that excludes `P_NOCLDWAIT`. That supports the ordinary
fresh-process assumption for that source; it is not a pin of the exact
host kernel, full controller-launch path or native signal state after
Python startup. I do not claim inheritance actually occurs on this host
or demand a campaign across operating systems.
[Apple process creation source](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_fork.c).

**Smallest root decision:** establish the no-automatic-reap premise for
this exact fresh controller launch, or reserve a narrowly reviewed
pre-spawn installation of waitable default SIGCHLD before creating the
one child. Under the cited CPython sigaction implementation,
`signal.signal(SIGCHLD, SIG_DFL)` reaches `PyOS_setsig`, whose replacement
flags contain `SA_ONSTACK`, not `SA_NOCLDWAIT`; this is a source-backed
possible seam, not an approved diff, an executed call, or a claim about
the installed native extension's build. Merely repeating `getsignal`
does not resolve F1. Do not replace the original handle with a discovered
PID/group, signal after reap, weaken the event checks, or silently fix
this archived candidate. No source file or proposal byte was changed.

## Packet work bound and unchanged case oracles

`closed_packet`, reconstructed lines 1919–1977, changes only the bound
plumbing for `assertions_completed`. It retains the 50000 ceiling for
`privacy_checks_completed`. `true_int` is byte-exact and requires
`type(value) is int`, so either counter rejects bool, float, negative,
non-integer and above-bound data. The new integer bound is
`2501*(16*(262144+4424)+128)+(2501+75+len(PLAN)+1)*PAYLOAD_BYTES`,
which is exactly **11012655666** for the fixed old25 inputs. It remains
below JavaScript's exact integer limit; no counter implementation changes.

I accept it as a deliberately coarse bound for this closed immutable
workload. Harness logic lines 16, 109–135 and 141–194 show why assertions
are work units: `ensure` increments for each successful check, and
`quotedArgs` checks each scanned command character. Shell quoting can
expand one character to four. Source allowance includes the largest
4424-byte READBACK collector, two variable-argument allowances at the
fixed 262144-byte record cap, framing margin and two checks per expanded
character. The 2501 factor permits the final attempt whose command scan
precedes the global 2500-call refusal. The fixed payload-size allowance
per call/cell/case/startup unit covers the much smaller non-scanner loops
over source binding, diff, controlled pages, traces, closed maps and
finite case fixtures. No command argument comes from an external tool
or private file in this harness. Candidate command sites are lines
39–58, 198–202, 247–250, 282–290 and 355–365.

This is not a measured maximum or a general validator for arbitrary
payloads. The bound relies on the exact immutable candidate, finite
stub model and unchanged source binding; a different payload or
argument generator needs its own review. The fixed case definitions
still stop at the first failed assertion, and failure packets are still
failed even when their work counter fits. Raising this transport bound
does not turn 230223 checks into 230223 cases or establish a passing case.

The independent data audit verified collector bodies and identities:

| Collector | Bytes | SHA-256 |
| --- | ---: | --- |
| CLOCK | 114 | `cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d` |
| STOP | 1600 | `5de5244440b134689bedfb1235e805c65a8b726e753cbdac1d0831ef1c83d071` |
| READBACK | 4424 | `b7d3504245ae94a584b455fa65826fbaa7759ec7a50a7b81c2cf8284c9f9051f` |
| MARKER | 626 | `13c6549e0328ad695769938cbc0be7eb47d7a10ea9f00c4d897655603437a949` |
| PERSIST | 805 | `819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30` |

All old packet guards remain byte-exact after reversing that one cap
block: one final LF and no additional LF; UTF-8/duplicate-free JSON and
nonfinite-constant refusal; exactly 15 keys and schema/PLAN equality;
typed ordered attempted/completed prefixes, progress gap at most one,
exact unreached suffix and strict-integer completed-group counts; cells
0–75; closed 13-name stub catalog with positive strict-integer counts
and total at most 2500; literal scope booleans; finite nonbool elapsed
0–30000; closed case/check failure packet; exactly typed/pinned source
fields. Main still requires the complete source, all PLAN names, no
child failure and positive cells/assertion/privacy counts for a pass.
No existing guard or refusal oracle was weakened.

The unchanged PLAN has phase_binding 5, secret_lifetime 3,
helper_handoff 5, partial_framing 7 and cleanup_latch 5. Complete harness
logic retains first-failure stop, 75-cell/2500-call/512-trace limits,
memory-only stubs, fresh VM contexts, code-generation restrictions,
privacy markers, receipt-before-comparison checks, password clearing,
single-use refs and all guard/sink case bodies. The later 36-case
projection harness is excluded from this old25 ceiling and controller.
Nothing in this report credits either harness with a run or pass.

## Cleanup control flow, status and diagnostics

The complete changed helper is reconstructed lines 1978–2049; the
author's complete body is report lines 7042–7113. I checked both live
and terminal paths, every refusal exit, the reap loop and group query.
The following conclusions are source-derived and conditional on F1's
waitable-child premise for the live branch.

| Boundary | Independent source assessment |
| --- | --- |
| Spawn/owner | The only spawn remains the exact pinned Node command with private cwd, three pipes, closed PATH/LANG/LC_ALL environment and `start_new_session=True`. The controller retains the original Popen object. No replacement handle, child poll, reaper thread or handler is added. |
| Terminal ownership | `terminal` requires an actual non-null event with strict-integer exact `si_pid`, SIGCHLD signo, CLD_EXITED/KILLED/DUMPED code and status 0–255, with positive killed/dumped signal. Other event codes/types/statuses fail. A terminal WNOWAIT event retains the owned child; it is distinct from live `getpgid` success. |
| Exit race | A live getpgid ProcessLookupError requires a new exact WNOWAIT terminal event; ESRCH alone never authorizes a signal. Wrong group, absent event, invalid event, nondefault handler or cached returncode refuses. |
| Signal ordering | Exactly one literal SIGKILL group-call site exists, before every reaping wait. Its ProcessLookupError is tolerated without inventing status. All other signaling errors latch failure. There is no nonzero signal after any reap. |
| Reap after refusal | An eligible original child gets a separate bounded P_PID/WEXITED/WNOHANG loop even if ownership/signaling refused. It never signals there. Thus refusal can still collect an actual exit, while preserving the first refusal. No eligibility means no invented reap. |
| Numeric exit | Only a returned exact terminal event sets `code`, `child.returncode` and `reaped=True`; exited status maps directly, killed/dumped status maps to negative signal. None, ECHILD, other errors, unsupported/invalid status or deadline expiry cannot map to zero. |
| Group query | Only `owned and reaped` permits the later literal-zero killpg query. Only ProcessLookupError sets `empty=True`. Presence/permission/other errors refuse. A reused group can cause conservative refusal but receives no delivered signal from this query. This is a point observation, not a permanent guarantee. |
| Public diagnostic | Exactly seven keys: stage, errno, identity, waitid_code/status and reap_code/status. Stage is null or one of nine source literals; errno is null or strict int 1–4095; identity is 0/1/2; event fields are null or values already accepted by the terminal predicate. No error string/class/path/trace/locals or child payload is copied. First stage and first failure are latched independently. |

The live child-to-group binding follows the unchanged new-session launch.
The [official CPython child-launch source](https://github.com/python/cpython/blob/v3.14.6/Modules/_posixsubprocess.c)
calls setsid before exec when requested. This is tag-source evidence,
not a new measurement of the installed extension or exact host kernel.

The [official waitid documentation](https://docs.python.org/3/library/os.html#os.waitid)
distinguishes None from ECHILD and documents the event fields. The
[CPython 3.14.6 wait wrapper](https://github.com/python/cpython/blob/v3.14.6/Modules/posixmodule.c)
initializes `si_pid`, forwards syscall errors and creates integer result
fields. There is no synthetic exit-zero fallback in that wrapper.

General XNU waitid source filters the parent's exact P_PID child and
preserves it under WNOWAIT; reaping removes group membership and the
process hash entry. General getpgid uses live proc lookup and can return
ESRCH for a child no longer referenceable through that lookup. These
sources support the terminal fallback's reasoning; they do not identify
the earlier exception or pin this host's kernel. The source also contains
an internal wait-collision sleep despite WNOHANG, so no unconditional
kernel blocking bound follows from the flag. This controller relies on
the reviewed single-owner/no-concurrent-reaper premise.
[Apple wait/reap source](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_exit.c),
[Apple process lookup](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_proc.c),
[Apple getpgid/setsid](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_prot.c).

I independently read the installed POSIX subprocess bodies as source:
90732 bytes, SHA-256
`6628ffdd65c093a6c08cae01ffe82877d3ced515aac9e7be0cff16512c30a7d9`,
at the report-named Python 3.14.6 stdlib path. Coverage includes
`__del__` 1132–1145, wait 1274–1295, fork arguments 1920–1929 and
exit-status/poll/try_wait/wait bodies 1996–2090. `_try_wait` can substitute
`sts=0` on ChildProcessError; removing `child.wait` is therefore a real
source improvement. No such method ran in this review.
[Corresponding official subprocess source](https://github.com/python/cpython/blob/v3.14.6/Lib/subprocess.py).

Failure boundaries stay candid. A refusal may prevent signaling a live
child; if it never becomes terminal before the absolute outer deadline,
reap remains unconfirmed and root must retain the cleanup hold. An invalid
result from a reaping syscall might already have consumed the OS event;
it still cannot be certified as a reap. If `returncode` remains None,
Popen finalization can perform a later internal nonblocking wait. That
is not the reviewed helper's returned status or cleanup confirmation.
There is no delivered group signal after the helper's reap attempt.
The comment about no child operations after the final clock describes
explicit main statements; it does not cancel interpreter finalization
on failure or prove a hard end-to-end process bound.

## Deadlines, output caps, persistence and grading

Main remains reconstructed lines 2050–2182. The three plumbing edits are
only `cleanup_diagnostic=None` initialization, the extra cleanup return
value and its retained-receipt field. The entire rest of main, including
the single Popen expression and final clock tail, reverses byte-exactly.

The absolute child deadline remains startup+30 seconds and the outer
deadline startup+35 seconds. Select/read/write handling remains
nonblocking with finite chunks; stdout/stderr remain 65536/16384 bytes.
Normal EOF still records the corresponding flag before unregister/close;
finally drains only already available bytes and respects the same caps.
The reap loop checks the original absolute outer deadline and sleeps at
most min(5 ms, remaining), with no new timeout origin or retry budget.
This is finite source scheduling, not a guarantee that a kernel call,
filesystem operation or stdout sink will return within that interval.

Disk samples and the 8-GiB floor remain exact. Fresh private directory
0700/UID/nonsymlink checks and exclusive no-follow evidence files remain
unchanged. `save` caps each file at 262144 bytes, uses mode 0600, flushes,
fsyncs and closes. Both bounded raw child outputs and the actual-exit
receipt, now with finite cleanup diagnostic, are saved before parsing
or grading. Failure during retention cannot become a pass.

The review journal is explicitly provisional and saved before the
final clock. The final monotonic sample follows every explicit file
save/close and owned-child operation. Inclusive `final_ns <=
OUTER_DEADLINE`, `review_closed`, full uncapped/EOF retention and a null
first failure all remain required. A save that returns late cannot earn
PASS from the earlier provisional grade. No new explicit filesystem or
child operation appears after that clock; serialization/write/flush of
closed stdout still follow it. Finalization, blocking fsync and output
delivery remain limits, not hidden claims of a hard kernel bound.

## Protected source and independent static proofs

Whole forward reconstruction gives the exact candidate hash above.
Whole reverse replacement restores all 209970 baseline bytes. Python
module AST reversal substitutes only `closed_packet`, `group_cleanup`
and `main` nodes and restores the entire baseline AST, ignoring source
locations. The set of twelve top-level functions is unchanged. A
separate byte inverse of main's three diagnostic edits and the packet
cap block proves their remaining bodies exact, rather than relying only
on the broad three-function AST inverse.

| Protected function | SHA-256 of exact complete source span |
| --- | --- |
| latch | `957eba18cdfb5d25bd5ee510141c74c9f942978d4540086d9db62b1f4f0fbec9` |
| within | `0b2b1be269404c27b4b554272d3cf5aaf20c481563e6407467ff7dc736cebfd9` |
| assemble_payload | `8978fe5175b1b6f74e8733b7dba8d0706e9d020fad959dc3cd88bed37a95e60b` |
| regular_node | `d29f5a1303626b60e8e7c061a86ebdc56fef40d17ce87df77f3feb8c4d907aa9` |
| private_directory | `12296c9587b151b2d998c6f174cb3579b4580ac210cae840e5a208c781a3eabc` |
| save | `ae13bed122b92c6c27e1a63f828c87bc4ce4477b6079836513b36d547f631900` |
| encode | `89ce1b76ca343fda5622acb07b3fce0d0ce63242704cf3b9c53b09c1ca0ac68e` |
| duplicate_free | `3134f45bbdba498f6684c83430ebfe93793174940b606f45cd1d1c0f8844a64b` |
| true_int | `646a375f4d50ebd66ab07b30ecf5c325b6027601ecc27d022ee50f2c5249bf04` |

All sixteen assignment source spans and ASTs remain exact:
START_NS, ROOT, NODE, NODE_SHA, PAYLOAD_SHA, PAYLOAD_BYTES, PAYLOAD_B64,
PLAN, CHECK_NAMES, EXPECTED_SOURCE, CHILD_DEADLINE, OUTER_DEADLINE,
STDOUT_CAP, STDERR_CAP, FLOOR and first_failure. Imports and the module
entrypoint remain exact. The three complete changed spans independently
match their archive:

| Function | Bytes | SHA-256 |
| --- | ---: | --- |
| closed_packet | 3772 | `bcd1ae35c59bd0c48a4866a922aa21f98b8b0845134767428468e21abd5eef77` |
| group_cleanup | 3773 | `de0d06fd97cf4a15bb4a0443bdce42e9e27c4ea7cdfc3ab6ee9d80b796201231` |
| main | 8444 | `6a2b09bfd01a4082e649577d7c97a01f2cccd6658e941302cc017efa91a4e0f9` |

Static syscall-source inspection confirms exactly two killpg sites
(SIGKILL then literal zero), three helper waitid sites (two WNOWAIT,
then one reaping query), signal-before-reap-before-query ordering and
no child.poll/child.wait in the revised helper. These are AST/source
selectors, not syscall results or prospective test executions.

## Checks, historical boundaries and remaining ownership

The independent Git/read/data reconstruction audit exited 0. It parsed
the baseline and whole candidate as ASTs; checked the complete appendix
and immutable prefix; matched the exact generated diff; proved whole
byte/AST inverse, nine protected function hashes, sixteen assignment
source/AST identities, main/packet narrower inverses, payload/logic and
five collector identities, unchanged 25-row PLAN/groups and the integer
ceiling. Archived verifier source was read and parsed as data; it was
not executed. No compile, eval, module import of a reviewed program or
Popen/kill/wait/case invocation was part of that audit.

One optional parser-location search, `rg --files node_modules ...`,
exited 2 because this worktree has no node_modules directory. No tool
setup followed; this report claims no new Acorn parse or JavaScript
execution. Official source reads used only CPython/Python documentation
and Apple-owned source. Some GitHub rendered pages omitted relevant
code; the corresponding official raw source was then read. Missing
rendered search matches are not tests of host syscall availability.

The public historical 515 receipt remains FAILED with numeric Python
exit 1 and first failure `owned_cleanup_unconfirmed`; numeric Node
child exit and reap are unconfirmed. The child-reported 25 completions,
32 cells and 230223 assertions remain unaccepted claims. The old
50000 bound's packet mismatch is independently source-visible, but
does not explain the earlier cleanup exception. Its cause, original
Authorization sender and true historical cleanup-start instant remain
UNKNOWN. Neither general current XNU nor this unexecuted proposal
supplies lost historical observations.

All actual memory/runtime acceptance stays with root. F1 is restricted
to this controller's live ownership premise; no universal platform,
tenant, browser or security campaign is requested. No source or old
report was rewritten, no private capture content was read, and no
other worker was contacted. The original D01/D05 gates, primary
ownership, closed rows and A09 runtime allocation are unchanged.

The first docs checker exited 1 because this new report linked to the
immutable continuation report as though it were present in this worktree.
It is absent here. I corrected only this report's citation to its exact
commit/path; no source alignment or historical-file import followed.
That was this report's link error, not a checker defect or a pre-existing
failure. The first whitespace and tracked/untracked scope checks exited
0, and AST-only parsing of the baseline, three changed bodies and complete
archived verifier also exited 0.

After that citation correction, `python3 scripts/check-docs.py` exited 0
with no link or build-layout errors. `git diff --check` and the independent
sole-new-report/tracked-byte/whitespace audit exited 0. These checks run
the existing documentation checker and source-data audit only. No reviewed
program was materialized or executed. This report provides source review
and a held premise, not permission to execute the proposed controller.
