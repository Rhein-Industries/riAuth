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

## Waitable SIGCHLD installation design — source only

Reservation `wave30_D01_waitable_sigchld_install_design`, 2026-10-03,
same project and existing Sol3 worktree. Root accepted the original F1
as a source-proof gap, not actual historical signal flags or cause.
The entire 24731-byte / 371-line cb7f883 report remains the exact prefix,
SHA-256 `d800cc8c7ec8d55c35a1276cf437651bad32a2921d2a5647e6b263c302197973`.
Entry was clean at `cb7f8833086f470859706c9ea11138f8a8b81e2f`.
Only this appendix is owned. The earlier hold remains dated evidence.

**Assessment: the one pre-spawn setter resolves F1 at source level for
the explicitly pinned, fresh, sole-child CPython process described below.**
It replaces the native signal flags rather than merely reading the
cached handler. A successful setter followed by unchanged single-owner
control flow establishes the missing no-automatic-reap premise before
the child exists. This is conditional source acceptance of the seam,
not an invocation, live signal readback, kernel certification, completed
cleanup or memory pass. All materialization and runtime remain held.

### Exact one-statement proposal and whole-source identity

The input is the complete 213188-byte / 2183-line controller reconstructed
from immutable 74a8fff and e443 as in the original review:
SHA-256 `98c270c461afccce0483b003cd41178d268a16e49ce33abfa48da031d75d1117`.
Insert exactly this 53-byte source line at new controller line 2071,
immediately before the only Popen assignment, which moves to line 2072:

```python
        signal.signal(signal.SIGCHLD,signal.SIG_DFL)
```

The full resulting source is **213241 bytes / 2184 lines**, SHA-256
`e109d30d3f481d9f0bacee5b34ca9ae1bae5457001c94ba931fd2f7520b60b54`.
It is reconstructed in memory only; no executable controller file was
created. The complete exact 534-byte / 10-line diff is SHA-256
`af937c81b65f3dd6bf06f8ef1dea8ab399978ecf983609d068ec1c7d296f75d4`:

```diff
--- controller-74a8fff-98c270.py
+++ DESIGN-waitable-sigchld.py
@@ -2068,6 +2068,7 @@
         if free<FLOOR:raise ValueError("disk_floor")
         evidence=private_directory(private,"d01-continuation-memory-"+sys.argv[1])
         phase="spawn"
+        signal.signal(signal.SIGCHLD,signal.SIG_DFL)
         child=subprocess.Popen([str(NODE),"--input-type=module","-"],cwd=evidence,
             env={"PATH":"/usr/bin:/bin","LANG":"C","LC_ALL":"C"},
             stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,
```

There is one added statement and no replacement, moved statement, import,
constant, source-binding or payload edit. No child has been spawned at
that statement. The dedicated controller process must have no foreign
children: changing process-wide SIGCHLD in an existing embedded/shared
interpreter is outside this design. There is no saved previous handler,
restoration step, new wait fallback or additional signal operation.

### Why this setter supplies the missing premise

I independently reread the official CPython v3.14.6 implementation.
The local `signal.py` wrapper passes integer enum values to `_signal.signal`.
Native `signal_signal_impl` checks the main interpreter/main thread,
selects SIG_DFL, calls `PyOS_setsig` at line 497 and propagates an error
before updating its stored handler. On success its cached handler is
set to the same default value. The existing cleanup `getsignal` guard
is therefore retained as a consistency check, not the original proof
that native flags were absent.
[CPython signal implementation](https://github.com/python/cpython/blob/v3.14.6/Modules/signalmodule.c).

With HAVE_SIGACTION, `PyOS_setsig`, lines 3361–3377, installs a replacement
sigaction with default handler, empty handler mask and `SA_ONSTACK` flags.
It does not preserve inherited `SA_NOCLDWAIT` or a previous ignored
handler. A failed native installation returns SIG_ERR; a successful one
supplies the premise F1 lacked. The macro HAVE_SIGINTERRUPT belongs to
the alternate no-sigaction branch; it does not change this replacement.
[CPython signal wrapper implementation](https://github.com/python/cpython/blob/v3.14.6/Python/pylifecycle.c).

General current XNU `setsigvec`, lines 635–645, clears P_NOCLDWAIT when
neither SA_NOCLDWAIT nor SIG_IGN is supplied. That supports the inference
from this setter to ordinary waitable-child semantics. This is primary
source reasoning, not a claim that Apple main is the exact host-kernel
revision or that the old515 process had P_NOCLDWAIT.
[Apple signal implementation](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_sig.c).

The unchanged child call still requests a new session; its single-owner
parent performs only WNOWAIT observations before the one permitted
nonzero group signal. With successful installation, no concurrent
reaper/disposition changer and the retained original child, exit cannot
permit PID reuse before this parent's explicit reap. Terminal event
validation, cached-returncode refusal and the exact getpgid/terminal
fallback paths remain unchanged. This setter addresses only F1; it is
not an alternative status proof or authority to discover another group.

If the setter raises an ordinary Exception, `phase` is already the fixed
`spawn` value, so the unchanged catch latches `controller_spawn_failed`.
The Popen assignment was not reached: child remains None and spawn_count
remains zero; finally skips group_cleanup. The existing evidence path
can retain the refusal, with no child exit/reap/empty-group fabrication.
Secondary grading failures cannot overwrite that first failure. Existing
BaseException propagation, including any pending interrupt raised before
installation, remains unchanged; no new receipt guarantee is claimed
for that outer boundary. A setter that returns late does not gain a new
deadline or a hard syscall bound.

### Installed metadata pins and exact future-launch assumptions

No proposed-controller launch, reviewed native-module import or version
command was invoked. Local Python served only as the static source/data
audit tool. File metadata, bytes for hashing, installed Python source/build
metadata and official source were read. Let F be the versioned directory:
`/opt/homebrew/Cellar/python@3.14/3.14.6/Frameworks/Python.framework/Versions/3.14`.
All files below were regular nonsymlinks at the named paths, UID 501.

| Path relative to F | Bytes / mode | Independently read SHA-256 |
| --- | ---: | --- |
| bin/python3.14 | 34640 / 0755 | `4f00ea2ad53d62437a6a3946b73c73614a97e8accdc5b96dc095ea1a0d9c6a56` |
| Python | 5468672 / 0755 | `15436055aa2c02ed0218ac0e02b3a27a92102f88cd59ab5094896b9306317336` |
| include/python3.14/pyconfig.h | 60669 / 0644 | `e875b545c5d65a9598f2b3f4a8c2c1a8dfb81fc7baace25acc22aafaf9e2b421` |
| lib/python3.14/signal.py | 2495 / 0644 | `0363c964c90ac0b3e515de5749205e6e6454051a1211058375d84d91eab6071a` |
| lib/python3.14/config-3.14-darwin/Makefile | 223264 / 0644 | `516889e804bfce1dd5c7e2c9d94c0a30c2ca6c0f3c758f56d30c123dceeec3d3` |
| lib/python3.14/config-3.14-darwin/Setup | 11083 / 0644 | `7db652648cd3f0f9afee585595fa80b52d53cf414514eb856803fc5dddf0f13b` |
| lib/python3.14/subprocess.py | 90732 / 0644 | `6628ffdd65c093a6c08cae01ffe82877d3ced515aac9e7be0cff16512c30a7d9` |
| lib/python3.14/lib-dynload/_posixsubprocess.cpython-314-darwin.so | 54784 / 0755 | `ededa2d9c463ba4d1fd39ac1cf36a7f4393c1179543dec22fb8b2389983abef8` |

Header lines 1218 and 1230 independently match HAVE_SIGACTION=1 and
HAVE_SIGINTERRUPT=1. The full installed signal.py source was read, not
imported. Makefile lines 24–27 list `_signal` among built modules, omit
it from MODSHARED_NAMES and include Modules/signalmodule.o in MODOBJS;
the existing posix/wait support is also listed there. This supports a
builtin `_signal` expectation whose native pin is the framework Python
image, not an invented `_signal.so` artifact. It is build metadata, not
an observation of a loaded module. The Makefile framework-install path
uses the opt location; the named opt framework Python path currently
resolves to F/Python. No loaded-image inspection was performed.

A later root-reviewed launch must bind these assumptions explicitly:

1. Use that exact absolute versioned interpreter, with its framework
   image, signal.py, pyconfig/build metadata and unchanged subprocess/
   _posixsubprocess identities rechecked as metadata before launch. Do
   not substitute PATH-selected python3, a newer installation, a virtual
   environment or an embedded interpreter. The actual loader must use
   the pinned framework image, not an unreviewed DYLD override.
2. Treat the mapping from that installed native build to the cited
   v3.14.6 sigaction implementation as a declared source/build assumption.
   A header and file hash alone do not prove how every native instruction
   was compiled. Any distribution patch that changes this setter's
   semantics requires review; no disassembly, import or native probe has
   established instruction equivalence here.
3. Run in a fresh main interpreter on its main thread, dedicated to the
   controller, with no existing children or external reaper and no hook,
   embedding/native actor or thread changing signal disposition after
   installation. The complete controller has only its one Popen and no
   such actor. A separately reviewed launch may use `-I -S` and a closed
   parent environment to exclude Python startup/path customizations and
   loader overrides; that is future launcher design, not a second edit
   to this controller or an authorized invocation. Otherwise equivalent
   startup isolation needs an explicit source-backed launch contract.
4. Preserve the old25 ROOT/private directory/input nonce/Node binary
   pin and every existing 30/35-second, output, resource, ownership and
   evidence control. Retain ordinary waitable-child/sigaction semantics
   for the selected host, without claiming an exact kernel pin from
   current XNU source. Setter failure must stop before Popen; it cannot
   authorize fallback, a retry, foreign-child cleanup or signal discovery.

There is no new FFI, library setup, native diagnostic, alternate wait
implementation or required tool execution in this design. The current
metadata does not release any of those future launch prerequisites.

### Whole-byte, statement and AST preservation

The data-only forward construction contains the one 53-byte addition
exactly once. Removing exactly that addition restores every 98c270
byte. The complete twelve-function set stays unchanged: eleven source
spans are byte-exact, including the previously changed closed_packet
and group_cleanup; removing the one line restores main byte-exactly.
New main is 8497 bytes, SHA-256
`d0f1b32db1ea73595e8c58612850fbb0690fabdc772eccc8ad012e0190541461`.

At AST level the sole inserted Expr is the exact
`signal.signal(signal.SIGCHLD, signal.SIG_DFL)` call, immediately before
the unchanged sole Popen Assign in main's existing try body. Deleting
that Expr restores the entire module AST excluding locations. There
is no other changed node, call, branch or exception handler.

All sixteen assignment source spans/ASTs, imports and entrypoint remain
exact. The complete old25 payload remains 132725 bytes with SHA-256
`46c19af829c546cf0f32f0a587db084dd00d304ea9d508c98ccda0b2a6028866`.
The revised assertion ceiling, strict packet integers/nonbools, twelve
function behavior outside the setter, 30/35-second absolute clocks,
EOF flags, pre-reap signal order, actual terminal/status-returning reap,
first-failure latch, finite cleanup diagnostic, 8-GiB floor, output/file
caps, fsync-before-grade and final-clock tail remain exact. The setter
does not add an after-reap signal or remove the cached default guard.

This is the complete reconstruction/protection checker as data. The
independent static audit used these source/data operations; no proposed
controller statement or imported reviewed module was executed:

```python
import ast, base64, difflib, hashlib, re, subprocess

PATH = "docs/roadmap/local-wave30-d01-continuation-correction-plan.md"
PROPOSAL = "74a8fff613c0edd6b4f842f380bd7125b4582c85"
def sha(raw):
    return hashlib.sha256(raw).hexdigest()
def fences(raw):
    ticks = bytes([96]) * 3
    return re.findall(b"^" + ticks + rb"([^\n]*)\n(.*?)^" + ticks + b"$", raw, re.M | re.S)
def span(raw, node):
    return b"".join(raw.splitlines(keepends=True)[node.lineno-1:node.end_lineno])
archives = fences(subprocess.check_output(["git", "show", PROPOSAL + ":" + PATH]))
original = next(v for _, v in archives if len(v) == 209970)
original_tree = ast.parse(original)
functions = {n.name: n for n in original_tree.body if isinstance(n, ast.FunctionDef)}
base = original
for name, size in (("closed_packet", 3772), ("group_cleanup", 3773), ("main", 8444)):
    body = next(v for _, v in archives if len(v) == size and v.startswith(("def " + name + "(").encode()))
    old = span(original, functions[name])
    assert base.count(old) == 1
    base = base.replace(old, body)
assert len(base) == 213188 and sha(base) == "98c270c461afccce0483b003cd41178d268a16e49ce33abfa48da031d75d1117"
anchor = b'        child=subprocess.Popen([str(NODE),"--input-type=module","-"],cwd=evidence,\n'
addition = b'        signal.signal(signal.SIGCHLD,signal.SIG_DFL)\n'
assert base.count(anchor) == 1 and addition not in base
candidate = base.replace(anchor, addition + anchor)
assert len(candidate) == 213241 and sha(candidate) == "e109d30d3f481d9f0bacee5b34ca9ae1bae5457001c94ba931fd2f7520b60b54"
assert candidate.count(addition) == 1 and candidate.replace(addition, b"") == base
diff = "".join(difflib.unified_diff(base.decode().splitlines(keepends=True),
    candidate.decode().splitlines(keepends=True), fromfile="controller-74a8fff-98c270.py",
    tofile="DESIGN-waitable-sigchld.py", n=3)).encode()
assert len(diff) == 534 and sha(diff) == "af937c81b65f3dd6bf06f8ef1dea8ab399978ecf983609d068ec1c7d296f75d4"
before, after = ast.parse(base), ast.parse(candidate)
old_functions = {n.name: n for n in before.body if isinstance(n, ast.FunctionDef)}
new_functions = {n.name: n for n in after.body if isinstance(n, ast.FunctionDef)}
assert len(new_functions) == 12 and old_functions.keys() == new_functions.keys()
for name in old_functions:
    old, new = span(base, old_functions[name]), span(candidate, new_functions[name])
    assert (new.replace(addition, b"") if name == "main" else new) == old
def assignments(tree):
    return {n.targets[0].id: n for n in tree.body if isinstance(n, ast.Assign)}
old_assignments, new_assignments = assignments(before), assignments(after)
assert len(old_assignments) == 16 and old_assignments.keys() == new_assignments.keys()
for name, node in old_assignments.items():
    assert span(base, node) == span(candidate, new_assignments[name])
    assert ast.dump(node, include_attributes=False) == ast.dump(new_assignments[name], include_attributes=False)
payload = base64.b64decode(new_assignments["PAYLOAD_B64"].value.value, validate=True)
assert len(payload) == 132725 and sha(payload) == "46c19af829c546cf0f32f0a587db084dd00d304ea9d508c98ccda0b2a6028866"
main_try = next(n for n in new_functions["main"].body if isinstance(n, ast.Try))
statement = ast.parse("signal.signal(signal.SIGCHLD,signal.SIG_DFL)").body[0]
matches = [i for i, n in enumerate(main_try.body)
           if ast.dump(n, include_attributes=False) == ast.dump(statement, include_attributes=False)]
assert len(matches) == 1
i = matches[0]
assert isinstance(main_try.body[i+1], ast.Assign)
assert ast.unparse(main_try.body[i+1].value.func) == "subprocess.Popen"
del main_try.body[i]
assert ast.dump(after, include_attributes=False) == ast.dump(before, include_attributes=False)
```

### Actual static checks and boundaries

Immutable source reads and independent data-only reconstruction exited
0: exact old98 and newe109 hashes, complete diff hash, byte inverse,
sole Expr AST inverse, eleven unchanged full functions plus restored
main, sixteen assignment bytes/ASTs and complete payload matched.
Installed metadata/source/hash reads exited 0 and reproduced root's
pyconfig identity. No build, native import, version probe, signal call,
process-control call, Node/VM/case/harness or executable-file creation
was part of these checks. Official signal.py page reads returned an
internal error/cache miss; no claim relies on those failed fetches.
Its installed source body was read directly, and the native reasoning
uses the successfully read official C sources.

New36 DATA binding remains separately owned. This one-line proposal
patches only the old25 controller code as data. A future composition
must separately review the exact payload/binding/packet changes and
all source identities; this appendix cannot certify that composition,
its work ceiling or a case outcome. No other worker was contacted.

Old515 remains FAILED, with cleanup cause and numeric child exit UNKNOWN;
no earlier flag state, cause, sender or true cleanup-start instant is
reconstructed. Root owns source review, publication and any separate
single finite memory release. A09 run 37087561409 retains the sole
validation lane; no local slot was acquired or released. All runtime
is still held, including any setter/controller materialization.

Final independent appendix checks exited 0 for the exact 24731-byte prefix,
sole-file append, complete archived diff equality/hash, source reconstruction,
byte inverse, candidate/checker AST syntax and trailing whitespace.
`python3 scripts/check-docs.py` and `git diff --check` both exited 0, with
no link or build-layout error. The appended source fences were parsed as
data; neither the controller nor the archived checker was executed.

## 2026-10-03 exact composed-controller outer-launch design — UNRUN

Reservation `wave30_D01_exact_composed_memory_launcher_design` owns only this
appendix. The entire `ec88cd1445d14e8cbbd1abf1660a04cbefbac0d7` report prefix is
preserved: 42397 bytes, 665 lines, SHA-256
`83666ce65c0f49ad2f9050bf18e868f8b24554d57187ea9430e58e0cb219596a`.
No launcher/controller file has been materialized and no process under review
has run. The following is complete proposed source DATA for one future launch.

### Exact input and smallest launch boundary

The sole controller input is the final Python fence, with its existing final LF,
in `560525a07eea0a696ffeb36599308795348dab47`,
`docs/roadmap/local-wave30-d01-root-user-review.md`: 242290 bytes, 2451 lines,
SHA-256 `d29c5c015cfa6f7b2ae3a88fea221e7da7bc3699e03f1dfba4f29e29e2c5778a`.
That immutable whole report hashes to
`ceb787321c0ebfd55cc86cfba0cfae6dd87a24b8115bf5c11f31c36286b0f412`.
Its accepted six DATA bindings and exact 53-byte SIGCHLD setter are left
unchanged. The embedded new36 payload is 151979 bytes, SHA-256
`a5d564be3b8a27abaead1b85b7f87cf2e7a0fe755144e48429b4a505118f8697`;
the source declares 36 ordered cases and 96 check labels. None is an actual
executed case or assertion count. Old25 is never selected or launched.

Use one fresh outer CPython process to feed those exact controller bytes through
one nonblocking stdin pipe to one exact CPython child. That child alone owns its
reviewed one Node child. This reuses the prior actual outer-capture contract:
private finite raw stdout/stderr and numeric actual status retained before
expectations. The old actual outer implementation was not archived here; this
new source does not claim to reproduce its code or to fix old515. Seven small
accepted controller functions are reused byte-exactly as complete functions,
including the status-returning WNOWAIT/reap helper; their outer child is the
direct controller, not Node.

The outer has a five-second preflight clock. Its capture limit is forty seconds
from the Popen attempt, with cleanup ending at absolute fifty seconds and a
final decision clock at absolute fifty-five. These are outer cooperative
limits. The controller's own unchanged thirty-second Node and thirty-five-second
total clocks remain authoritative; its closed return must report `within35`
and elapsed time at most 35. A longer capture window does not grant the controller
a longer passing deadline. Source/hash reads, process creation, filesystem
calls, fsync, waits, interpreter startup and stdout delivery are not hard kernel
or I/O bounds. No quota, guaranteed free space or guaranteed write safety is
claimed. The final clock covers explicit evidence persistence, and no explicit
file or child operation follows it; serialization, stdout and interpreter
finalization are outside that final observation.

Capture keeps at most 65536 stdout bytes and 16384 stderr bytes. On overflow it
retains the exact prefix, bytes actually seen and an explicit capped/incomplete
state, then refuses; it never labels that prefix complete output. EOF on both
streams, all 242290 input bytes sent, no cap and actual status are prerequisites
for a completed capture. Up to 32 resource samples are taken at a two-second
cadence during capture. They are actual samples, not a proof of between-sample
headroom. Start requires 8.5 GiB; any observed free space below 8 GiB stops the
owned capture. The controller retains its own 8-GiB check. No cache deletion,
threshold lowering, helper retry or baseline run is proposed.

### Exact pins, startup assumptions and staging prerequisites

The versioned interpreter, framework image, header, signal.py, Makefile, Setup,
subprocess.py and _posixsubprocess pins are precisely the eight ec88 pins.
Metadata-only rereads during this design reproduced all eight sizes, modes,
UIDs and hashes. Node reproduced the accepted hash and size, with freshly
observed mode **0555**, UID 501; the source below pins that actual mode rather
than treating it as the interpreter's 0755 mode. No version command, loaded-image
inspection, native import or live signal probe occurred.

Before any future invocation, root must separately authorize and stage both
DATA files named in the manifest as exclusive regular nonsymlink mode-0600
UID-501 files inside the original module-boundaries worktree's existing
UID-501 mode-0700 `deployment-private`. Both proposed DATA paths are currently
absent, observed by metadata only. Root must extract the controller fence
without adding a newline, rehash the full controller and full launcher and
verify modes/ownership before starting the pinned interpreter. No staging
is authorized by this report. The launcher reopens controller DATA relative
to the validated private-directory descriptor with O_NOFOLLOW, validates its
exact size/mode/UID/hash and post-read identity, and validates all nine native/
metadata pins again before its single child attempt. It refuses a reused nonce
or existing inner fixture and exclusively creates its fresh outer capture.
The fresh nonce must be exactly sixteen lowercase hexadecimal characters;
no fixed nonce or automatic retry is embedded.

Root's future entry must be on the original ROOT, through the exact absolute
CPython 3.14.6 executable, in a fresh main interpreter on its main thread,
`-I -S`, with exactly PATH=/usr/bin:/bin, LANG=C and LC_ALL=C. Do not inherit
DYLD/PYTHON hooks, a venv, embedded/native actor, foreign child, reaper,
signal-disposition changer or concurrent installation/fixture mutator.
The closed environment is checked again in the launcher and passed unchanged
to its controller. The opt framework resolution must still select the pinned
versioned image. The native/build mapping and ordinary waitable-child host
semantics remain the declared ec88 source assumptions, not live observations
or an exact XNU revision. Eight file pins are not a hash of the complete
transitive Python/Node dependency closure or a proof of which image the loader
used. Root must retain the stable trusted installation/loader premise from
ec88; this proposal introduces no native FFI, fallback or module probe.

The outer also installs the same SIGCHLD default immediately before its only
Popen, while it has no child. Setter refusal before that attempt means no
launcher-created child. A failed Popen with no returned handle is conservatively
unknown ownership after an attempt; it is never converted into proof that no
process existed. Both levels retain the same single-reaper/no-other-actor premise.

### Status retention, ownership and unresolved abnormal-child boundary

The private `controller.stdout`, `controller.stderr`, `resources.json` and
`retained.json` are exclusive mode-0600, flushed/fsynced/closed and the capture
directory is fsynced before JSON parsing or expectations. They preserve the
actual controller terminal code, waitid/reap diagnostic, partial/complete
stream state, hashes, timing, resources and the first capture/cleanup failure.
A nonzero or unknown actual code takes precedence over packet comparisons.
There is no Popen.wait/ECHILD-to-zero fallback. Evidence-write failure refuses
grading and is reported as unconfirmed durability. The subsequent review is
explicitly provisional until the closed final-clock stdout and numeric outer
exit, which root must retain before comparing results. Filesystem fsync is the
requested persistence operation, not a power-loss or hard latency guarantee.

The reused cleanup helper only authorizes a nonzero signal for its original
unreaped, session-leading direct Popen child. Live getpgid or validated terminal
WNOWAIT observation, PID/event/type checks and cached returncode guard precede
the group signal. It obtains real consuming waitid status even on refusal
paths. A signal-zero existence query after reap is not permission for another
nonzero signal. There is no foreign-child discovery, wait fallback, broad kill
or reused-PID delivery after reap. If exact reaping/group absence cannot be
established, that uncertainty and first failure remain; no automatic lane
release is permitted.

Node intentionally starts another session. Killing or reaping the outer-owned
controller cannot prove or perform nested Node cleanup. After timeout, abnormal
controller status, incomplete/malformed output, pre-grade failure or missing
handle, nested cleanup remains **UNKNOWN**, even if the direct controller group
is absent. The outer neither learns a nested PID from raw output nor signals it.
Only a complete zero-exit closed passing return may be labeled
`controller_reported_clean`; `independent_nested_ownership_proof` remains false.
That label describes the source-bound controller's own proof, not independent
outer custody of Node. Root must review the actual controller evidence before
releasing the lane. An abnormal controller branch requires a separate
root-owned ownership/cleanup decision; no safe grandchild cleanup channel is
invented here. Thus normal bounded capture has a concrete design, while
independent cleanup after abnormal controller termination remains an explicit
unresolved boundary and may block later lane release.

### Full launcher source DATA

Exact source: 21320 bytes, 380 lines, SHA-256
`40302bd5b6f10b1c3f7f2c0cebf4b7e7dd8d77810a71180fd98af23835babacb`. Its complete native pin tuple, closed argv/env,
limits and failure handling are included; this fence is not an executable file.

```python
# DESIGN DATA ONLY: one future exact composed-controller launch; never run in this source phase.
import time
START_NS=time.monotonic_ns()
import hashlib,json,math,os,pathlib,re,selectors,shutil,signal,stat,subprocess,sys
ROOT=pathlib.Path("/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27")
PRIVATE=ROOT/"deployment-private"
F=pathlib.Path("/opt/homebrew/Cellar/python@3.14/3.14.6/Frameworks/Python.framework/Versions/3.14")
PY=F/"bin/python3.14"
NODE=pathlib.Path("/opt/homebrew/Cellar/node/26.7.0/bin/node")
SOURCE_NAME="d01-composed-controller-d29c5c-source.py"
SOURCE_SHA="d29c5c015cfa6f7b2ae3a88fea221e7da7bc3699e03f1dfba4f29e29e2c5778a"
SOURCE_BYTES=242290
ENV={"PATH":"/usr/bin:/bin","LANG":"C","LC_ALL":"C"}
# UID/mode/size/hash pins are source/metadata checks, not loaded-image observations.
PINS=(
    ("python_executable",PY,34640,0o755,"4f00ea2ad53d62437a6a3946b73c73614a97e8accdc5b96dc095ea1a0d9c6a56"),
    ("python_framework",F/"Python",5468672,0o755,"15436055aa2c02ed0218ac0e02b3a27a92102f88cd59ab5094896b9306317336"),
    ("python_build_header",F/"include/python3.14/pyconfig.h",60669,0o644,"e875b545c5d65a9598f2b3f4a8c2c1a8dfb81fc7baace25acc22aafaf9e2b421"),
    ("signal_stdlib",F/"lib/python3.14/signal.py",2495,0o644,"0363c964c90ac0b3e515de5749205e6e6454051a1211058375d84d91eab6071a"),
    ("python_build_makefile",F/"lib/python3.14/config-3.14-darwin/Makefile",223264,0o644,"516889e804bfce1dd5c7e2c9d94c0a30c2ca6c0f3c758f56d30c123dceeec3d3"),
    ("python_build_setup",F/"lib/python3.14/config-3.14-darwin/Setup",11083,0o644,"7db652648cd3f0f9afee585595fa80b52d53cf414514eb856803fc5dddf0f13b"),
    ("subprocess_stdlib",F/"lib/python3.14/subprocess.py",90732,0o644,"6628ffdd65c093a6c08cae01ffe82877d3ced515aac9e7be0cff16512c30a7d9"),
    ("posixsubprocess_native",F/"lib/python3.14/lib-dynload/_posixsubprocess.cpython-314-darwin.so",54784,0o755,"ededa2d9c463ba4d1fd39ac1cf36a7f4393c1179543dec22fb8b2389983abef8"),
    ("node_executable",NODE,50320,0o555,"1ef99ea25fe70c9b67e7efe768ef8ee22148d3cabc703db6131b57aeb617d040"),
)
PRECHECK_DEADLINE=START_NS+5_000_000_000
OUTER_DEADLINE=START_NS+50_000_000_000
FINAL_DEADLINE=START_NS+55_000_000_000
START_FREE=17*1024**3//2
FLOOR=8*1024**3
STDOUT_CAP=65536
STDERR_CAP=16384
SAMPLE_CAP=32
first_failure=None

def latch(tag):
    global first_failure
    if first_failure is None:first_failure=tag


def private_directory(parent,name):
    fd=os.open(parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:
        info=os.fstat(fd)
        if info.st_uid!=os.getuid() or stat.S_IMODE(info.st_mode)!=0o700:
            raise ValueError("private_directory")
        os.mkdir(name,0o700,dir_fd=fd)
    finally:os.close(fd)
    path=parent/name
    info=path.lstat()
    if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():
        raise ValueError("private_directory")
    return path


def save(parent,name,raw):
    if len(raw)>262144:raise ValueError("evidence_cap")
    fd=os.open(parent/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    try:
        os.fchmod(fd,0o600)
        with os.fdopen(fd,"wb",closefd=False) as out:
            out.write(raw);out.flush();os.fsync(out.fileno())
    finally:os.close(fd)


def encode(value):
    return (json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=True)+"\n").encode("ascii")


def duplicate_free(pairs):
    result={}
    for key,value in pairs:
        if key in result:raise ValueError("duplicate_json")
        result[key]=value
    return result


def true_int(value,low,high):
    return type(value) is int and low<=value<=high


def group_cleanup(child):
    # Only the exact unreaped Popen child can authorize a nonzero group signal.
    reaped=False;empty=False;code=None;owned=False;stage="child_identity"
    diagnostic={"stage":None,"errno":None,"identity":0,
                "waitid_code":None,"waitid_status":None,
                "reap_code":None,"reap_status":None}
    def failed(at,error=None,label="owned_cleanup_unconfirmed"):
        if diagnostic["stage"] is None:
            diagnostic["stage"]=at
            number=getattr(error,"errno",None)
            diagnostic["errno"]=number if true_int(number,1,4095) else None
        latch(label)
    def terminal(info):
        return (info is not None and type(info.si_pid) is int and info.si_pid==child.pid and
                type(info.si_signo) is int and info.si_signo==signal.SIGCHLD and
                type(info.si_code) is int and
                info.si_code in (os.CLD_EXITED,os.CLD_KILLED,os.CLD_DUMPED) and
                true_int(info.si_status,0,255) and
                (info.si_code==os.CLD_EXITED or info.si_status>0))
    eligible=true_int(child.pid,1,2147483647) and child.returncode is None
    try:
        if not eligible:raise ValueError("child_identity")
        stage="signal_disposition"
        if signal.getsignal(signal.SIGCHLD)!=signal.SIG_DFL:
            raise ValueError("signal_disposition")
        stage="waitid_identity"
        observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
        if observation is None:
            stage="live_group_identity"
            try:
                owned=os.getpgid(child.pid)==child.pid
                if owned:diagnostic["identity"]=1
            except ProcessLookupError:
                stage="exited_group_identity"
                observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
        if observation is not None:
            if not terminal(observation):raise ValueError("waitid_identity")
            owned=True;diagnostic["identity"]=2
            diagnostic["waitid_code"]=observation.si_code
            diagnostic["waitid_status"]=observation.si_status
        if not owned or child.returncode is not None:raise ValueError("group_identity")
        stage="signal_owned_group"
        try:os.killpg(child.pid,signal.SIGKILL)
        except ProcessLookupError:pass
    except Exception as error:
        failed(stage,error)
    # Reap this exact child even after identity/signal refusal; never signal here.
    if eligible:
        stage="reap_owned_child"
        try:
            while time.monotonic_ns()<OUTER_DEADLINE:
                observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG)
                if observation is not None:
                    if not terminal(observation):raise ValueError("reap_identity")
                    code=(observation.si_status if observation.si_code==os.CLD_EXITED
                          else -observation.si_status)
                    child.returncode=code;reaped=True
                    diagnostic["reap_code"]=observation.si_code
                    diagnostic["reap_status"]=observation.si_status
                    break
                remaining=(OUTER_DEADLINE-time.monotonic_ns())/1e9
                if remaining>0:time.sleep(min(.005,remaining))
            if not reaped:failed("reap_deadline")
        except Exception as error:
            failed(stage,error)
    if owned and reaped:
        # Signal zero is only an existence query; no signal is delivered after reap.
        try:os.killpg(child.pid,0)
        except ProcessLookupError:empty=True
        except Exception as error:failed("group_absence",error)
        if not empty:failed("group_absence",label="owned_group_not_empty")
    return code,reaped,empty,diagnostic


def pinned_file(path,size,mode,digest,dir_fd=None,keep=False):
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW,dir_fd=dir_fd)
    raw=bytearray();count=0;hasher=hashlib.sha256()
    try:
        before=os.fstat(fd)
        if (not stat.S_ISREG(before.st_mode) or before.st_uid!=501 or
                stat.S_IMODE(before.st_mode)!=mode or before.st_size!=size):
            raise ValueError("file_metadata")
        while True:
            if time.monotonic_ns()>=PRECHECK_DEADLINE:raise TimeoutError()
            part=os.read(fd,131072)
            if not part:break
            count+=len(part)
            if count>size:raise ValueError("file_size")
            hasher.update(part)
            if keep:raw.extend(part)
        after=os.fstat(fd)
        if ((before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns,
             before.st_uid,before.st_mode)!=(after.st_dev,after.st_ino,after.st_size,
             after.st_mtime_ns,after.st_ctime_ns,after.st_uid,after.st_mode) or
                count!=size or hasher.hexdigest()!=digest):
            raise ValueError("file_identity")
    finally:os.close(fd)
    return bytes(raw),{"bytes":count,"mode":mode,"uid":501,"sha256":hasher.hexdigest()}

def directory_sync(path):
    fd=os.open(path,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:
        info=os.fstat(fd)
        if info.st_uid!=501 or stat.S_IMODE(info.st_mode)!=0o700:
            raise ValueError("directory_identity")
        os.fsync(fd)
    finally:os.close(fd)

def accepted_return(raw):
    packet=json.loads(raw.decode("ascii"),object_pairs_hook=duplicate_free)
    keys={"schema","result","first_failure","child_exit","child_reaped","owned_group_empty",
          "full_packet_retained_before_grade","review_closed","within35","elapsed_seconds",
          "completed_cases","child_failure","actual_product_or_driver"}
    if type(packet) is not dict or set(packet)!=keys or not raw.endswith(b"\n"):
        return False
    elapsed=packet["elapsed_seconds"]
    return (packet["schema"]=="riauth.d01-continuation-memory-return/v1" and
            packet["result"]=="passed" and packet["first_failure"] is None and
            true_int(packet["child_exit"],0,0) and packet["child_reaped"] is True and
            packet["owned_group_empty"] is True and
            packet["full_packet_retained_before_grade"] is True and
            packet["review_closed"] is True and packet["within35"] is True and
            type(elapsed) in (int,float) and math.isfinite(elapsed) and 0<=elapsed<=35 and
            true_int(packet["completed_cases"],36,36) and packet["child_failure"] is None and
            packet["actual_product_or_driver"] is False)

def main():
    evidence=None;child=None;selector=None;source=b"";verified={}
    raw={"out":bytearray(),"err":bytearray()};caps={"out":STDOUT_CAP,"err":STDERR_CAP}
    eof={"out":False,"err":False};seen={"out":0,"err":0};output_capped=False
    code=None;reaped=False;empty=False;diagnostic=None;spawn_attempts=0
    sent=0;spawn_ns=None;capture_ns=None;capture_deadline=None;phase="preflight"
    samples=[];minimum_free=None;last_disk=0;retained=False;review_closed=False;grade=False
    nested="not_spawned"
    try:
        os.umask(0o077)
        if len(sys.argv)!=2 or re.fullmatch(r"[0-9a-f]{16}",sys.argv[1]) is None:
            raise ValueError("nonce")
        if os.getuid()!=501 or pathlib.Path.cwd()!=ROOT or ROOT.resolve(strict=True)!=ROOT:
            raise ValueError("workspace")
        if (pathlib.Path(sys.executable).resolve(strict=True)!=PY or
                sys.flags.isolated!=1 or sys.flags.no_site!=1 or dict(os.environ)!=ENV):
            raise ValueError("interpreter_environment")
        info=ROOT.lstat()
        if not stat.S_ISDIR(info.st_mode) or info.st_uid!=501:raise ValueError("workspace")
        private_fd=os.open(PRIVATE,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        try:
            info=os.fstat(private_fd)
            if info.st_uid!=501 or stat.S_IMODE(info.st_mode)!=0o700:
                raise ValueError("private_directory")
            try:os.stat("d01-continuation-memory-"+sys.argv[1],dir_fd=private_fd,follow_symlinks=False)
            except FileNotFoundError:pass
            else:raise ValueError("nonce_reused")
            evidence=private_directory(PRIVATE,"d01-composed-outer-"+sys.argv[1])
            source,verified["controller_source"]=pinned_file(
                SOURCE_NAME,SOURCE_BYTES,0o600,SOURCE_SHA,dir_fd=private_fd,keep=True)
        finally:os.close(private_fd)
        for label,path,size,mode,digest in PINS:
            unused,verified[label]=pinned_file(path,size,mode,digest)
        opt=pathlib.Path("/opt/homebrew/opt/python@3.14/Frameworks/Python.framework/Versions/3.14/Python")
        if opt.resolve(strict=True)!=(F/"Python"):raise ValueError("framework_resolution")
        if not all(hasattr(os,name) for name in
                   ("waitid","P_PID","WEXITED","WNOHANG","WNOWAIT","CLD_EXITED","CLD_KILLED","CLD_DUMPED")):
            raise ValueError("owned_wait_unavailable")
        free=shutil.disk_usage(ROOT).free;minimum_free=free
        samples.append({"elapsed_ns":time.monotonic_ns()-START_NS,"free_bytes":free})
        if free<START_FREE:raise ValueError("start_free")
        if time.monotonic_ns()>=PRECHECK_DEADLINE:raise TimeoutError()
        phase="spawn"
        signal.signal(signal.SIGCHLD,signal.SIG_DFL)
        if time.monotonic_ns()>=PRECHECK_DEADLINE:raise TimeoutError()
        spawn_ns=time.monotonic_ns();capture_deadline=spawn_ns+40_000_000_000
        spawn_attempts=1;nested="unknown"
        child=subprocess.Popen([str(PY),"-I","-S","-",sys.argv[1]],cwd=ROOT,env=ENV,
            stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,
            start_new_session=True,bufsize=0)
        selector=selectors.DefaultSelector()
        for stream,kind,events in ((child.stdin,"in",selectors.EVENT_WRITE),
                                   (child.stdout,"out",selectors.EVENT_READ),
                                   (child.stderr,"err",selectors.EVENT_READ)):
            os.set_blocking(stream.fileno(),False);selector.register(stream,events,kind)
        phase="capture";exit_seen=False
        while selector.get_map() or not exit_seen:
            now=time.monotonic_ns()
            if now>=capture_deadline:latch("capture_deadline");break
            if now-last_disk>=2_000_000_000:
                if len(samples)>=SAMPLE_CAP:latch("resource_sample_cap");break
                free=shutil.disk_usage(ROOT).free;minimum_free=min(minimum_free,free);last_disk=now
                samples.append({"elapsed_ns":now-START_NS,"free_bytes":free})
                if free<FLOOR:latch("disk_floor");break
            observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
            if observation is not None:exit_seen=True
            for key,events in selector.select(min(.02,max(0,(capture_deadline-now)/1e9))):
                stream,kind=key.fileobj,key.data
                if kind=="in":
                    try:written=os.write(stream.fileno(),source[sent:sent+16384])
                    except BlockingIOError:continue
                    except BrokenPipeError:latch("controller_input_closed");break
                    sent+=written
                    if sent==SOURCE_BYTES:selector.unregister(stream);stream.close()
                else:
                    try:part=os.read(stream.fileno(),4096)
                    except BlockingIOError:continue
                    if not part:eof[kind]=True;selector.unregister(stream);stream.close();continue
                    seen[kind]+=len(part)
                    raw[kind].extend(part[:max(0,caps[kind]-len(raw[kind]))])
                    if seen[kind]>caps[kind]:
                        output_capped=True;latch("controller_output_cap");break
            if first_failure is not None:break
    except BaseException:latch("outer_"+phase+"_failed")
    finally:
        if child is not None:
            try:code,reaped,empty,diagnostic=group_cleanup(child)
            except BaseException:latch("outer_cleanup_failed")
            # Bounded nonblocking available-data drain, not a claim of total produced bytes.
            for kind,stream in (("out",child.stdout),("err",child.stderr)):
                try:
                    if stream is not None and not stream.closed:
                        os.set_blocking(stream.fileno(),False)
                        for attempt in range(caps[kind]//4096+1):
                            if time.monotonic_ns()>=OUTER_DEADLINE:
                                latch("drain_deadline");break
                            try:part=os.read(stream.fileno(),4096)
                            except BlockingIOError:break
                            if not part:eof[kind]=True;break
                            seen[kind]+=len(part)
                            raw[kind].extend(part[:max(0,caps[kind]-len(raw[kind]))])
                            if seen[kind]>caps[kind]:
                                output_capped=True;latch("controller_output_cap");break
                except BaseException:latch("outer_drain_failed")
            for stream in (child.stdin,child.stdout,child.stderr):
                try:
                    if stream is not None and not stream.closed:stream.close()
                except BaseException:latch("outer_pipe_close_failed")
        if selector is not None:
            try:selector.close()
            except BaseException:latch("outer_selector_close_failed")
        capture_ns=time.monotonic_ns()
    complete=sent==SOURCE_BYTES and all(eof.values()) and not output_capped
    receipt={"schema":"riauth.d01-composed-outer-capture/v1","controller_source_sha256":SOURCE_SHA,
        "controller_source_bytes":SOURCE_BYTES,"verified_files":verified,"spawn_attempts":spawn_attempts,
        "controller_handle_acquired":child is not None,"controller_pid":None if child is None else child.pid,
        "controller_exit":code,"controller_reaped":reaped,"controller_group_empty":empty,
        "cleanup_diagnostic":diagnostic,"nested_node_cleanup":"unknown" if spawn_attempts else "not_spawned",
        "input_sent":sent,"stdout_bytes":len(raw["out"]),"stderr_bytes":len(raw["err"]),
        "stdout_sha256":hashlib.sha256(raw["out"]).hexdigest(),
        "stderr_sha256":hashlib.sha256(raw["err"]).hexdigest(),"bytes_seen":seen,"stream_eof":eof,
        "output_capped":output_capped,"complete_bounded_capture":complete,"first_failure":first_failure,
        "spawn_elapsed_ns":None if spawn_ns is None else spawn_ns-START_NS,
        "capture_end_elapsed_ns":capture_ns-START_NS,"minimum_free_bytes":minimum_free,
        "actual_product_or_driver":False}
    try:
        if evidence is None:raise ValueError("evidence")
        # These files and actual exit/status are fsynced/closed BEFORE parsing or grading.
        save(evidence,"controller.stdout",bytes(raw["out"]))
        save(evidence,"controller.stderr",bytes(raw["err"]))
        save(evidence,"resources.json",encode({"schema":"riauth.d01-composed-outer-resources/v1",
                                              "samples":samples,"minimum_free_bytes":minimum_free}))
        save(evidence,"retained.json",encode(receipt));directory_sync(evidence);retained=True
    except BaseException:latch("outer_evidence_unconfirmed")
    # A nonzero/unknown actual status always precedes packet expectations.
    if spawn_attempts!=1 or child is None:latch("controller_not_acquired")
    if not reaped or not empty:latch("controller_cleanup_unconfirmed")
    if code is None:latch("controller_exit_unknown")
    elif code!=0:latch("controller_nonzero")
    if not complete:latch("controller_capture_incomplete")
    if raw["err"]:latch("controller_stderr_nonempty")
    if retained and first_failure is None:
        try:
            if not accepted_return(bytes(raw["out"])):latch("controller_return_refused")
        except BaseException:latch("controller_return_refused")
        if first_failure is None:
            nested="controller_reported_clean";grade=True
    try:
        if not retained:raise ValueError("retention")
        save(evidence,"review.json",encode({"schema":"riauth.d01-composed-outer-review/v1",
            "retained_before_grade":retained,"grade_before_final_clock":grade,
            "first_failure":first_failure,"controller_exit":code,"controller_reaped":reaped,
            "controller_group_empty":empty,"nested_node_cleanup":nested,
            "independent_nested_ownership_proof":False}))
        directory_sync(evidence);review_closed=True
    except BaseException:latch("outer_review_unconfirmed")
    final_ns=time.monotonic_ns()
    # No explicit evidence write/close/mutation or child operation after this clock.
    within55=final_ns<FINAL_DEADLINE
    if not within55:latch("outer_final_deadline")
    passed=grade and retained and review_closed and within55 and first_failure is None
    summary={"schema":"riauth.d01-composed-outer-return/v1","result":"passed" if passed else "failed",
        "first_failure":first_failure,"controller_exit":code,"controller_reaped":reaped,
        "controller_group_empty":empty,"nested_node_cleanup":nested,
        "independent_nested_ownership_proof":False,"retained_before_grade":retained,
        "review_closed":review_closed,"complete_bounded_capture":complete,
        "within55":within55,"elapsed_seconds":(final_ns-START_NS)/1e9,
        "stdout_bytes":len(raw["out"]),"stderr_bytes":len(raw["err"]),
        "stdout_sha256":hashlib.sha256(raw["out"]).hexdigest(),
        "stderr_sha256":hashlib.sha256(raw["err"]).hexdigest(),"actual_product_or_driver":False}
    sys.stdout.write(json.dumps(summary,sort_keys=True,separators=(",",":"))+"\n");sys.stdout.flush()
    return 0 if passed else 1
if __name__=="__main__":sys.exit(main())
```

### Closed launch manifest and one future reservation

This is the exact manifest; the named nonce is a prerequisite placeholder,
not a value selected or generated in this design. The only proposed future
reservation is `wave30_D01_one_exact_composed_memory_outer`, after root's full
source/independent reviews, staging, metadata revalidation and explicit release.
It would permit one installed pinned Python invocation of the above unchanged
launcher, one unchanged composed-controller child and its one reviewed Node
memory child. No old25/baseline, retries, additional targets, browser or native
fixture are included. There is no runtime slot acquired, used or released here.

```json
{
  "schema": "riauth.d01-exact-composed-memory-launch-design/v1",
  "reservation": "wave30_D01_exact_composed_memory_launcher_design",
  "phase": "source_design_only_unrun",
  "controller_archive": {
    "commit": "560525a07eea0a696ffeb36599308795348dab47",
    "path": "docs/roadmap/local-wave30-d01-root-user-review.md",
    "selector": "final python fence including its existing final LF",
    "bytes": 242290,
    "lines": 2451,
    "sha256": "d29c5c015cfa6f7b2ae3a88fea221e7da7bc3699e03f1dfba4f29e29e2c5778a"
  },
  "launcher": {
    "future_data_path": "/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/deployment-private/d01-composed-memory-outer.py",
    "mode": "0600",
    "uid": 501,
    "bytes": 21320,
    "lines": 380,
    "sha256": "40302bd5b6f10b1c3f7f2c0cebf4b7e7dd8d77810a71180fd98af23835babacb"
  },
  "controller_data": {
    "future_data_path": "/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/deployment-private/d01-composed-controller-d29c5c-source.py",
    "mode": "0600",
    "uid": 501,
    "bytes": 242290,
    "sha256": "d29c5c015cfa6f7b2ae3a88fea221e7da7bc3699e03f1dfba4f29e29e2c5778a",
    "currently_absent": true
  },
  "root": "/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27",
  "private_directory": {
    "name": "deployment-private",
    "mode": "0700",
    "uid": 501,
    "nofollow": true
  },
  "argv": [
    "/opt/homebrew/Cellar/python@3.14/3.14.6/Frameworks/Python.framework/Versions/3.14/bin/python3.14",
    "-I",
    "-S",
    "/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/deployment-private/d01-composed-memory-outer.py",
    "ROOT_SELECTED_FRESH_16_LOWERCASE_HEX_NONCE"
  ],
  "controller_argv": [
    "/opt/homebrew/Cellar/python@3.14/3.14.6/Frameworks/Python.framework/Versions/3.14/bin/python3.14",
    "-I",
    "-S",
    "-",
    "SAME_NONCE"
  ],
  "environment": {
    "PATH": "/usr/bin:/bin",
    "LANG": "C",
    "LC_ALL": "C"
  },
  "source_payload": {
    "bytes": 151979,
    "sha256": "a5d564be3b8a27abaead1b85b7f87cf2e7a0fe755144e48429b4a505118f8697",
    "declared_cases": 36,
    "declared_checks": 96,
    "candidate_bytes": 33701,
    "candidate_sha256": "505b1c0fec96e248dc67db46379bd801fa06a2678462aa094fb2fc1bc5ea398f"
  },
  "budgets": {
    "preflight_seconds": 5,
    "controller_node_seconds": 30,
    "controller_total_seconds": 35,
    "outer_capture_seconds_from_spawn": 40,
    "outer_cleanup_absolute_seconds": 50,
    "outer_final_absolute_seconds": 55,
    "stdout_bytes": 65536,
    "stderr_bytes": 16384,
    "per_saved_file_bytes": 262144,
    "resource_sample_seconds": 2,
    "resource_sample_count": 32,
    "start_free_bytes": 9126805504,
    "floor_bytes": 8589934592
  },
  "children": {
    "controller_spawn_attempts_max": 1,
    "node_spawn_from_unchanged_controller_max": 1,
    "separate_node_session": true,
    "no_nested_signal_authority": true
  },
  "durability_order": [
    "actual status and bounded raw output captured",
    "private exclusive mode0600 stdout/stderr/resources/status files fsynced and closed",
    "capture directory fsynced",
    "packet parsing/expectations",
    "provisional review fsynced and closed",
    "capture directory fsynced",
    "final monotonic clock",
    "closed stdout summary and integer outer exit"
  ],
  "abnormal_nested_cleanup": "UNKNOWN; no discovery, nested signal, retry, or automatic validation-lane release",
  "native_pins": "complete PINS tuple below; eight exact ec88 CPython pins plus same Node hash, freshly observed mode0555",
  "runtime_authorized": false
}
```

The future entry argv in the manifest is launched with a cleared environment
containing only its three stated keys and cwd equal to ROOT. Root's existing
invocation owner must retain actual numeric launcher/tool completion and the
closed stdout before comparisons. Missing final stdout, unavailable numeric
status, termination of the outer process or inability to retain tool completion
is a failure/UNKNOWN cleanup boundary, never a synthesized zero or automatic
release. No second watchdog, owned-grandchild authority or hard timeout-kill
mechanism is silently authorized. The source has finite cooperative clocks;
an unreturning kernel/loader/I/O operation still needs root's explicit handling
and cannot be made a guaranteed wall-time bound by this proposal.

### Actual source-only checks

The complete composed-controller fence was read as bytes, its complete twelve
function bodies/controls and new bindings were read, and its Python AST parsed.
Base64 decoding and reading JSON string constants were DATA operations, not
execution of controller assignments. The whole controller hash/size matched,
its payload hash/size and 36/96 declarations matched, and the one accepted
SIGCHLD setter remained present. No controller source was changed.

The proposed launcher AST parsed successfully as DATA. Static checks found:
exactly one Popen with fixed absolute interpreter, -I/-S/stdin argv, original
ROOT, closed ENV and start_new_session; exactly one outer default setter before
it; no wait/poll/waitpid fallback, eval/exec, deletion or extra process launcher;
only the two unchanged killpg sites within the accepted cleanup function.
The seven reused function source spans match the composed input byte-exactly.
Their identities are:
```json
{
  "duplicate_free": {
    "bytes": 173,
    "sha256": "3134f45bbdba498f6684c83430ebfe93793174940b606f45cd1d1c0f8844a64b"
  },
  "encode": {
    "bytes": 125,
    "sha256": "89ce1b76ca343fda5622acb07b3fce0d0ce63242704cf3b9c53b09c1ca0ac68e"
  },
  "group_cleanup": {
    "bytes": 3773,
    "sha256": "de0d06fd97cf4a15bb4a0443bdce42e9e27c4ea7cdfc3ab6ee9d80b796201231"
  },
  "latch": {
    "bytes": 88,
    "sha256": "957eba18cdfb5d25bd5ee510141c74c9f942978d4540086d9db62b1f4f0fbec9"
  },
  "private_directory": {
    "bytes": 532,
    "sha256": "12296c9587b151b2d998c6f174cb3579b4580ac210cae840e5a208c781a3eabc"
  },
  "save": {
    "bytes": 342,
    "sha256": "ae13bed122b92c6c27e1a63f828c87bc4ce4477b6079836513b36d547f631900"
  },
  "true_int": {
    "bytes": 81,
    "sha256": "646a375f4d50ebd66ab07b30ecf5c325b6027601ecc27d022ee50f2c5249bf04"
  }
}
```
The unchanged controller source hash protects every function/policy byte;
this outer proposal is additive DATA and not a patch to that source. The
new source has eleven complete functions. Its final explicit monotonic sample
is source line 364; no save, directory-sync or owned-child
call follows it in main. The capture status/raw-file writes and directory fsync
precede the only accepted-return parser. Source budget, environment, argv,
integer/nonbool zero/case checks and actual-before-expectation ordering were
checked statically, not by calling any function.

One initial audit extraction exited 1: ast.literal_eval was incorrectly applied
to PLAN's json.loads AST Call (ValueError, source line 2123). The corrected
extractor read the literal JSON argument as DATA and exited 0. This was an audit
parser mistake, not a controller failure or evidence of a historical cause.
All subsequent immutable source/hash/AST/metadata checks exited 0. Local Python
was used as a static audit tool; neither this proposed launcher nor controller,
helper, observer, VM, case, Node, native module or signal/process-control
function was invoked.

The original old515 failure, child exit/cause UNKNOWN, sender UNKNOWN and true
historical cleanup-start UNKNOWN remain unchanged. No old25/new36 memory pass,
real browser journey, physical resource guarantee or native release is claimed.
A09 run 37087561409 owns validation under the supplied root reservation;
all runtime remains HELD. Root alone reviews, integrates, publishes and
separately decides any future execution and release.

Final actual static receipt: canonical report-fence extraction reproduced the
complete 21320-byte/380-line launcher and its SHA, manifest and reused function
proofs. The full ec88 prefix and sole-file append matched; 724 insertions and
zero removals preceded this receipt. Documentation links/build layout and
whitespace/scope checks exited 0, with no link or build-layout errors. This
receipt adds no executable source, invocation, runtime outcome or slot credit.

## 2026-10-03 pinned-platform startup environment correction — SOURCE DESIGN ONLY

Reservation `wave30_D01_platform_startup_environment_diagnostic` owns only
this append. The entire `ecb7552978ed50e1afcdee53c83b04c2bad921c2` prefix stays
byte-exact: 82139 bytes, 1396 lines, SHA-256
`28e32a5bd4149cd20cf404c7ce002a0ae1ea6209634229b9f3e14a11803e3169`.
All older designs, static audit errors and failed outcomes remain historical
evidence. Root's newly supplied outer preflight FAILURE is appended below;
old515 remains FAILED with child exit and cause UNKNOWN. Nothing in this
proposal credits old25/new36 memory or a browser journey.

### Actual receipts and exact source read

Both complete public receipts were read at the project orchestrator's
`planning/evidence` under
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Their independently computed byte/hash identities are:

```json
{
  "wave30-d01-new36-outer-preflight-f18ba1c-tool.json": {
    "bytes": 808,
    "sha256": "3d590e14a9bc654b8eee691f4951e9be71b6cfc5e5895ebc1c56c529ad963923"
  },
  "wave30-d01-pinned-startup-diagnostic.json": {
    "bytes": 660,
    "sha256": "d0ad0d2d88f2eea5a2cdfbe76e78bc10275616d4a1f857b00a55798c73f1b691"
  }
}
```

The first receipt retains actual tool exit **1**, nonce
`f18ba1c0174eaf5a` as supplied by root, and a complete closed outer summary:
`outer_preflight_failed`, elapsed 0.012147583, complete capture false,
controller exit NULL, controller reaped false, controller group empty false,
nested cleanup `not_spawned`, retention false and review closed false.
Child stdout/stderr lengths are zero. Those lengths describe the nonexistent
controller capture, not an absence of the retained outer summary. No
per-predicate diagnostic was retained in this first invocation.
Root supplied the separate ownership observations: no outer/inner evidence
directory, no controller/Node child, and validation lane released unused.
Those are root's actual observations, not a newly executed child/process
inspection here.

The separately authorized startup diagnostic retained actual exit **0**,
elapsed 0.028264416847378016, empty stderr, no timeout and diagnostic process
reaped. Its source SHA is
`69ced507a0e09440e339317977ab96d91b581e7b029083c1485709d70334d9fc`.
It is explicitly not a memory retry. Root specified the same pinned interpreter,
imports, closed three-key caller environment and cwd, with no memory/controller/
Node invocation. Its receipt reports cwd/executable/root resolution true,
UID 501, isolated 1 and no_site 1; each PATH/LANG/LC_ALL value matches.
Environment equality is **false**, key count **4**, the sole allowlisted extra
name is exactly `__CF_USER_TEXT_ENCODING`, and other-extra-key count is zero.
No value for the extra key was captured, read or inferred in this review.
Root reports native hash match; the diagnostic receipt does not contain an
independent loaded-image attestation or a digest of its complete dependency
closure. The receipt pins its diagnostic source, but does not embed that
source body; this review neither reconstructed nor executed the diagnostic.

The complete original 380-line launcher was read from the immutable ecb report
and from root's staged
`/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27/deployment-private/d01-composed-memory-outer.py`.
The staged file was a regular nonsymlink, UID 501, mode 0600, 21320 bytes;
O_NOFOLLOW readback matched every archived byte and SHA
`40302bd5b6f10b1c3f7f2c0cebf4b7e7dd8d77810a71180fd98af23835babacb`.
This was a source DATA read, not import/execution or a private capture read.

The original failure remains FAILURE with its exact cause **UNKNOWN**.
Several predicates share `outer_preflight_failed`; a later diagnostic does
not retroactively identify which failed in that process. The current source
comparison does, however, necessarily reject the later diagnostic's reported
four-key shape even when all three required values match. That source/observed
startup incompatibility is the narrowly supported reason for this proposal.
No broken loader, platform origin of the key, benign encoding value, native
cause or memory-candidate defect is attested.

### One exact launcher-only hunk

Replace only `dict(os.environ)` at original launcher source line 223 with:

```text
{key:os.environ[key] for key in os.environ if key!="__CF_USER_TEXT_ENCODING"}
```

The observation compares the remaining dictionary to unchanged ENV.
It permits only that exact case-sensitive key to be present or absent; the
comprehension skips its value lookup entirely. It does not capture, check,
normalize or copy the uncaptured value. It is not a prefix/regex filter,
general platform exception, fallback or allowance for any other key name.
Missing or changed PATH/LANG/LC_ALL still makes the dictionaries unequal,
and every other extra key remains in the compared dictionary and refuses.
DYLD/PYTHON names, similarly spelled keys and unexpected caller additions
gain no exception. These are source-derived properties, not executed
predicate cases.

Both the root-selected caller environment and the literal child ENV remain
**exactly three keys**. No new environment variable is supplied to the child,
no parent value is inherited, and os.environ is not mutated. An observed
platform-added key is tolerated only by this one comparison, without making
a claim about its origin or actual value. Existing flags, executable, UID,
cwd, root resolution, native/source pins and every subsequent refusal guard
remain exact.

Exact one-hunk unified diff: 717 bytes, SHA-256
`f0606caf8202e8fc35cb40bc850b1ca7fa5a2cc481bffbf5542a84ad559fb175`.

```diff
--- ecb7552-outer-40302bd5.py
+++ DESIGN-platform-text-encoding-outer.py
@@ -220,7 +220,7 @@
         if os.getuid()!=501 or pathlib.Path.cwd()!=ROOT or ROOT.resolve(strict=True)!=ROOT:
             raise ValueError("workspace")
         if (pathlib.Path(sys.executable).resolve(strict=True)!=PY or
-                sys.flags.isolated!=1 or sys.flags.no_site!=1 or dict(os.environ)!=ENV):
+                sys.flags.isolated!=1 or sys.flags.no_site!=1 or {key:os.environ[key] for key in os.environ if key!="__CF_USER_TEXT_ENCODING"}!=ENV):
             raise ValueError("interpreter_environment")
         info=ROOT.lstat()
         if not stat.S_ISDIR(info.st_mode) or info.st_uid!=501:raise ValueError("workspace")
```

### Complete proposed candidate DATA

Candidate: 21381 bytes, 380 lines, SHA-256
`8a4da1306765f331f18b4fd56cb71dff03395f40fb2246475be5e47b035e1e1b`.
The 61-byte increase is solely the expression substitution. The full source
is archived for immutable review; it is not a script or private DATA-file edit.

```python
# DESIGN DATA ONLY: one future exact composed-controller launch; never run in this source phase.
import time
START_NS=time.monotonic_ns()
import hashlib,json,math,os,pathlib,re,selectors,shutil,signal,stat,subprocess,sys
ROOT=pathlib.Path("/Users/dominik/orca/projects/riAuth-public-preview-local-module-boundaries-wave27")
PRIVATE=ROOT/"deployment-private"
F=pathlib.Path("/opt/homebrew/Cellar/python@3.14/3.14.6/Frameworks/Python.framework/Versions/3.14")
PY=F/"bin/python3.14"
NODE=pathlib.Path("/opt/homebrew/Cellar/node/26.7.0/bin/node")
SOURCE_NAME="d01-composed-controller-d29c5c-source.py"
SOURCE_SHA="d29c5c015cfa6f7b2ae3a88fea221e7da7bc3699e03f1dfba4f29e29e2c5778a"
SOURCE_BYTES=242290
ENV={"PATH":"/usr/bin:/bin","LANG":"C","LC_ALL":"C"}
# UID/mode/size/hash pins are source/metadata checks, not loaded-image observations.
PINS=(
    ("python_executable",PY,34640,0o755,"4f00ea2ad53d62437a6a3946b73c73614a97e8accdc5b96dc095ea1a0d9c6a56"),
    ("python_framework",F/"Python",5468672,0o755,"15436055aa2c02ed0218ac0e02b3a27a92102f88cd59ab5094896b9306317336"),
    ("python_build_header",F/"include/python3.14/pyconfig.h",60669,0o644,"e875b545c5d65a9598f2b3f4a8c2c1a8dfb81fc7baace25acc22aafaf9e2b421"),
    ("signal_stdlib",F/"lib/python3.14/signal.py",2495,0o644,"0363c964c90ac0b3e515de5749205e6e6454051a1211058375d84d91eab6071a"),
    ("python_build_makefile",F/"lib/python3.14/config-3.14-darwin/Makefile",223264,0o644,"516889e804bfce1dd5c7e2c9d94c0a30c2ca6c0f3c758f56d30c123dceeec3d3"),
    ("python_build_setup",F/"lib/python3.14/config-3.14-darwin/Setup",11083,0o644,"7db652648cd3f0f9afee585595fa80b52d53cf414514eb856803fc5dddf0f13b"),
    ("subprocess_stdlib",F/"lib/python3.14/subprocess.py",90732,0o644,"6628ffdd65c093a6c08cae01ffe82877d3ced515aac9e7be0cff16512c30a7d9"),
    ("posixsubprocess_native",F/"lib/python3.14/lib-dynload/_posixsubprocess.cpython-314-darwin.so",54784,0o755,"ededa2d9c463ba4d1fd39ac1cf36a7f4393c1179543dec22fb8b2389983abef8"),
    ("node_executable",NODE,50320,0o555,"1ef99ea25fe70c9b67e7efe768ef8ee22148d3cabc703db6131b57aeb617d040"),
)
PRECHECK_DEADLINE=START_NS+5_000_000_000
OUTER_DEADLINE=START_NS+50_000_000_000
FINAL_DEADLINE=START_NS+55_000_000_000
START_FREE=17*1024**3//2
FLOOR=8*1024**3
STDOUT_CAP=65536
STDERR_CAP=16384
SAMPLE_CAP=32
first_failure=None

def latch(tag):
    global first_failure
    if first_failure is None:first_failure=tag


def private_directory(parent,name):
    fd=os.open(parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:
        info=os.fstat(fd)
        if info.st_uid!=os.getuid() or stat.S_IMODE(info.st_mode)!=0o700:
            raise ValueError("private_directory")
        os.mkdir(name,0o700,dir_fd=fd)
    finally:os.close(fd)
    path=parent/name
    info=path.lstat()
    if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode)!=0o700 or info.st_uid!=os.getuid():
        raise ValueError("private_directory")
    return path


def save(parent,name,raw):
    if len(raw)>262144:raise ValueError("evidence_cap")
    fd=os.open(parent/name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    try:
        os.fchmod(fd,0o600)
        with os.fdopen(fd,"wb",closefd=False) as out:
            out.write(raw);out.flush();os.fsync(out.fileno())
    finally:os.close(fd)


def encode(value):
    return (json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=True)+"\n").encode("ascii")


def duplicate_free(pairs):
    result={}
    for key,value in pairs:
        if key in result:raise ValueError("duplicate_json")
        result[key]=value
    return result


def true_int(value,low,high):
    return type(value) is int and low<=value<=high


def group_cleanup(child):
    # Only the exact unreaped Popen child can authorize a nonzero group signal.
    reaped=False;empty=False;code=None;owned=False;stage="child_identity"
    diagnostic={"stage":None,"errno":None,"identity":0,
                "waitid_code":None,"waitid_status":None,
                "reap_code":None,"reap_status":None}
    def failed(at,error=None,label="owned_cleanup_unconfirmed"):
        if diagnostic["stage"] is None:
            diagnostic["stage"]=at
            number=getattr(error,"errno",None)
            diagnostic["errno"]=number if true_int(number,1,4095) else None
        latch(label)
    def terminal(info):
        return (info is not None and type(info.si_pid) is int and info.si_pid==child.pid and
                type(info.si_signo) is int and info.si_signo==signal.SIGCHLD and
                type(info.si_code) is int and
                info.si_code in (os.CLD_EXITED,os.CLD_KILLED,os.CLD_DUMPED) and
                true_int(info.si_status,0,255) and
                (info.si_code==os.CLD_EXITED or info.si_status>0))
    eligible=true_int(child.pid,1,2147483647) and child.returncode is None
    try:
        if not eligible:raise ValueError("child_identity")
        stage="signal_disposition"
        if signal.getsignal(signal.SIGCHLD)!=signal.SIG_DFL:
            raise ValueError("signal_disposition")
        stage="waitid_identity"
        observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
        if observation is None:
            stage="live_group_identity"
            try:
                owned=os.getpgid(child.pid)==child.pid
                if owned:diagnostic["identity"]=1
            except ProcessLookupError:
                stage="exited_group_identity"
                observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
        if observation is not None:
            if not terminal(observation):raise ValueError("waitid_identity")
            owned=True;diagnostic["identity"]=2
            diagnostic["waitid_code"]=observation.si_code
            diagnostic["waitid_status"]=observation.si_status
        if not owned or child.returncode is not None:raise ValueError("group_identity")
        stage="signal_owned_group"
        try:os.killpg(child.pid,signal.SIGKILL)
        except ProcessLookupError:pass
    except Exception as error:
        failed(stage,error)
    # Reap this exact child even after identity/signal refusal; never signal here.
    if eligible:
        stage="reap_owned_child"
        try:
            while time.monotonic_ns()<OUTER_DEADLINE:
                observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG)
                if observation is not None:
                    if not terminal(observation):raise ValueError("reap_identity")
                    code=(observation.si_status if observation.si_code==os.CLD_EXITED
                          else -observation.si_status)
                    child.returncode=code;reaped=True
                    diagnostic["reap_code"]=observation.si_code
                    diagnostic["reap_status"]=observation.si_status
                    break
                remaining=(OUTER_DEADLINE-time.monotonic_ns())/1e9
                if remaining>0:time.sleep(min(.005,remaining))
            if not reaped:failed("reap_deadline")
        except Exception as error:
            failed(stage,error)
    if owned and reaped:
        # Signal zero is only an existence query; no signal is delivered after reap.
        try:os.killpg(child.pid,0)
        except ProcessLookupError:empty=True
        except Exception as error:failed("group_absence",error)
        if not empty:failed("group_absence",label="owned_group_not_empty")
    return code,reaped,empty,diagnostic


def pinned_file(path,size,mode,digest,dir_fd=None,keep=False):
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW,dir_fd=dir_fd)
    raw=bytearray();count=0;hasher=hashlib.sha256()
    try:
        before=os.fstat(fd)
        if (not stat.S_ISREG(before.st_mode) or before.st_uid!=501 or
                stat.S_IMODE(before.st_mode)!=mode or before.st_size!=size):
            raise ValueError("file_metadata")
        while True:
            if time.monotonic_ns()>=PRECHECK_DEADLINE:raise TimeoutError()
            part=os.read(fd,131072)
            if not part:break
            count+=len(part)
            if count>size:raise ValueError("file_size")
            hasher.update(part)
            if keep:raw.extend(part)
        after=os.fstat(fd)
        if ((before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns,before.st_ctime_ns,
             before.st_uid,before.st_mode)!=(after.st_dev,after.st_ino,after.st_size,
             after.st_mtime_ns,after.st_ctime_ns,after.st_uid,after.st_mode) or
                count!=size or hasher.hexdigest()!=digest):
            raise ValueError("file_identity")
    finally:os.close(fd)
    return bytes(raw),{"bytes":count,"mode":mode,"uid":501,"sha256":hasher.hexdigest()}

def directory_sync(path):
    fd=os.open(path,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
    try:
        info=os.fstat(fd)
        if info.st_uid!=501 or stat.S_IMODE(info.st_mode)!=0o700:
            raise ValueError("directory_identity")
        os.fsync(fd)
    finally:os.close(fd)

def accepted_return(raw):
    packet=json.loads(raw.decode("ascii"),object_pairs_hook=duplicate_free)
    keys={"schema","result","first_failure","child_exit","child_reaped","owned_group_empty",
          "full_packet_retained_before_grade","review_closed","within35","elapsed_seconds",
          "completed_cases","child_failure","actual_product_or_driver"}
    if type(packet) is not dict or set(packet)!=keys or not raw.endswith(b"\n"):
        return False
    elapsed=packet["elapsed_seconds"]
    return (packet["schema"]=="riauth.d01-continuation-memory-return/v1" and
            packet["result"]=="passed" and packet["first_failure"] is None and
            true_int(packet["child_exit"],0,0) and packet["child_reaped"] is True and
            packet["owned_group_empty"] is True and
            packet["full_packet_retained_before_grade"] is True and
            packet["review_closed"] is True and packet["within35"] is True and
            type(elapsed) in (int,float) and math.isfinite(elapsed) and 0<=elapsed<=35 and
            true_int(packet["completed_cases"],36,36) and packet["child_failure"] is None and
            packet["actual_product_or_driver"] is False)

def main():
    evidence=None;child=None;selector=None;source=b"";verified={}
    raw={"out":bytearray(),"err":bytearray()};caps={"out":STDOUT_CAP,"err":STDERR_CAP}
    eof={"out":False,"err":False};seen={"out":0,"err":0};output_capped=False
    code=None;reaped=False;empty=False;diagnostic=None;spawn_attempts=0
    sent=0;spawn_ns=None;capture_ns=None;capture_deadline=None;phase="preflight"
    samples=[];minimum_free=None;last_disk=0;retained=False;review_closed=False;grade=False
    nested="not_spawned"
    try:
        os.umask(0o077)
        if len(sys.argv)!=2 or re.fullmatch(r"[0-9a-f]{16}",sys.argv[1]) is None:
            raise ValueError("nonce")
        if os.getuid()!=501 or pathlib.Path.cwd()!=ROOT or ROOT.resolve(strict=True)!=ROOT:
            raise ValueError("workspace")
        if (pathlib.Path(sys.executable).resolve(strict=True)!=PY or
                sys.flags.isolated!=1 or sys.flags.no_site!=1 or {key:os.environ[key] for key in os.environ if key!="__CF_USER_TEXT_ENCODING"}!=ENV):
            raise ValueError("interpreter_environment")
        info=ROOT.lstat()
        if not stat.S_ISDIR(info.st_mode) or info.st_uid!=501:raise ValueError("workspace")
        private_fd=os.open(PRIVATE,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
        try:
            info=os.fstat(private_fd)
            if info.st_uid!=501 or stat.S_IMODE(info.st_mode)!=0o700:
                raise ValueError("private_directory")
            try:os.stat("d01-continuation-memory-"+sys.argv[1],dir_fd=private_fd,follow_symlinks=False)
            except FileNotFoundError:pass
            else:raise ValueError("nonce_reused")
            evidence=private_directory(PRIVATE,"d01-composed-outer-"+sys.argv[1])
            source,verified["controller_source"]=pinned_file(
                SOURCE_NAME,SOURCE_BYTES,0o600,SOURCE_SHA,dir_fd=private_fd,keep=True)
        finally:os.close(private_fd)
        for label,path,size,mode,digest in PINS:
            unused,verified[label]=pinned_file(path,size,mode,digest)
        opt=pathlib.Path("/opt/homebrew/opt/python@3.14/Frameworks/Python.framework/Versions/3.14/Python")
        if opt.resolve(strict=True)!=(F/"Python"):raise ValueError("framework_resolution")
        if not all(hasattr(os,name) for name in
                   ("waitid","P_PID","WEXITED","WNOHANG","WNOWAIT","CLD_EXITED","CLD_KILLED","CLD_DUMPED")):
            raise ValueError("owned_wait_unavailable")
        free=shutil.disk_usage(ROOT).free;minimum_free=free
        samples.append({"elapsed_ns":time.monotonic_ns()-START_NS,"free_bytes":free})
        if free<START_FREE:raise ValueError("start_free")
        if time.monotonic_ns()>=PRECHECK_DEADLINE:raise TimeoutError()
        phase="spawn"
        signal.signal(signal.SIGCHLD,signal.SIG_DFL)
        if time.monotonic_ns()>=PRECHECK_DEADLINE:raise TimeoutError()
        spawn_ns=time.monotonic_ns();capture_deadline=spawn_ns+40_000_000_000
        spawn_attempts=1;nested="unknown"
        child=subprocess.Popen([str(PY),"-I","-S","-",sys.argv[1]],cwd=ROOT,env=ENV,
            stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,
            start_new_session=True,bufsize=0)
        selector=selectors.DefaultSelector()
        for stream,kind,events in ((child.stdin,"in",selectors.EVENT_WRITE),
                                   (child.stdout,"out",selectors.EVENT_READ),
                                   (child.stderr,"err",selectors.EVENT_READ)):
            os.set_blocking(stream.fileno(),False);selector.register(stream,events,kind)
        phase="capture";exit_seen=False
        while selector.get_map() or not exit_seen:
            now=time.monotonic_ns()
            if now>=capture_deadline:latch("capture_deadline");break
            if now-last_disk>=2_000_000_000:
                if len(samples)>=SAMPLE_CAP:latch("resource_sample_cap");break
                free=shutil.disk_usage(ROOT).free;minimum_free=min(minimum_free,free);last_disk=now
                samples.append({"elapsed_ns":now-START_NS,"free_bytes":free})
                if free<FLOOR:latch("disk_floor");break
            observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
            if observation is not None:exit_seen=True
            for key,events in selector.select(min(.02,max(0,(capture_deadline-now)/1e9))):
                stream,kind=key.fileobj,key.data
                if kind=="in":
                    try:written=os.write(stream.fileno(),source[sent:sent+16384])
                    except BlockingIOError:continue
                    except BrokenPipeError:latch("controller_input_closed");break
                    sent+=written
                    if sent==SOURCE_BYTES:selector.unregister(stream);stream.close()
                else:
                    try:part=os.read(stream.fileno(),4096)
                    except BlockingIOError:continue
                    if not part:eof[kind]=True;selector.unregister(stream);stream.close();continue
                    seen[kind]+=len(part)
                    raw[kind].extend(part[:max(0,caps[kind]-len(raw[kind]))])
                    if seen[kind]>caps[kind]:
                        output_capped=True;latch("controller_output_cap");break
            if first_failure is not None:break
    except BaseException:latch("outer_"+phase+"_failed")
    finally:
        if child is not None:
            try:code,reaped,empty,diagnostic=group_cleanup(child)
            except BaseException:latch("outer_cleanup_failed")
            # Bounded nonblocking available-data drain, not a claim of total produced bytes.
            for kind,stream in (("out",child.stdout),("err",child.stderr)):
                try:
                    if stream is not None and not stream.closed:
                        os.set_blocking(stream.fileno(),False)
                        for attempt in range(caps[kind]//4096+1):
                            if time.monotonic_ns()>=OUTER_DEADLINE:
                                latch("drain_deadline");break
                            try:part=os.read(stream.fileno(),4096)
                            except BlockingIOError:break
                            if not part:eof[kind]=True;break
                            seen[kind]+=len(part)
                            raw[kind].extend(part[:max(0,caps[kind]-len(raw[kind]))])
                            if seen[kind]>caps[kind]:
                                output_capped=True;latch("controller_output_cap");break
                except BaseException:latch("outer_drain_failed")
            for stream in (child.stdin,child.stdout,child.stderr):
                try:
                    if stream is not None and not stream.closed:stream.close()
                except BaseException:latch("outer_pipe_close_failed")
        if selector is not None:
            try:selector.close()
            except BaseException:latch("outer_selector_close_failed")
        capture_ns=time.monotonic_ns()
    complete=sent==SOURCE_BYTES and all(eof.values()) and not output_capped
    receipt={"schema":"riauth.d01-composed-outer-capture/v1","controller_source_sha256":SOURCE_SHA,
        "controller_source_bytes":SOURCE_BYTES,"verified_files":verified,"spawn_attempts":spawn_attempts,
        "controller_handle_acquired":child is not None,"controller_pid":None if child is None else child.pid,
        "controller_exit":code,"controller_reaped":reaped,"controller_group_empty":empty,
        "cleanup_diagnostic":diagnostic,"nested_node_cleanup":"unknown" if spawn_attempts else "not_spawned",
        "input_sent":sent,"stdout_bytes":len(raw["out"]),"stderr_bytes":len(raw["err"]),
        "stdout_sha256":hashlib.sha256(raw["out"]).hexdigest(),
        "stderr_sha256":hashlib.sha256(raw["err"]).hexdigest(),"bytes_seen":seen,"stream_eof":eof,
        "output_capped":output_capped,"complete_bounded_capture":complete,"first_failure":first_failure,
        "spawn_elapsed_ns":None if spawn_ns is None else spawn_ns-START_NS,
        "capture_end_elapsed_ns":capture_ns-START_NS,"minimum_free_bytes":minimum_free,
        "actual_product_or_driver":False}
    try:
        if evidence is None:raise ValueError("evidence")
        # These files and actual exit/status are fsynced/closed BEFORE parsing or grading.
        save(evidence,"controller.stdout",bytes(raw["out"]))
        save(evidence,"controller.stderr",bytes(raw["err"]))
        save(evidence,"resources.json",encode({"schema":"riauth.d01-composed-outer-resources/v1",
                                              "samples":samples,"minimum_free_bytes":minimum_free}))
        save(evidence,"retained.json",encode(receipt));directory_sync(evidence);retained=True
    except BaseException:latch("outer_evidence_unconfirmed")
    # A nonzero/unknown actual status always precedes packet expectations.
    if spawn_attempts!=1 or child is None:latch("controller_not_acquired")
    if not reaped or not empty:latch("controller_cleanup_unconfirmed")
    if code is None:latch("controller_exit_unknown")
    elif code!=0:latch("controller_nonzero")
    if not complete:latch("controller_capture_incomplete")
    if raw["err"]:latch("controller_stderr_nonempty")
    if retained and first_failure is None:
        try:
            if not accepted_return(bytes(raw["out"])):latch("controller_return_refused")
        except BaseException:latch("controller_return_refused")
        if first_failure is None:
            nested="controller_reported_clean";grade=True
    try:
        if not retained:raise ValueError("retention")
        save(evidence,"review.json",encode({"schema":"riauth.d01-composed-outer-review/v1",
            "retained_before_grade":retained,"grade_before_final_clock":grade,
            "first_failure":first_failure,"controller_exit":code,"controller_reaped":reaped,
            "controller_group_empty":empty,"nested_node_cleanup":nested,
            "independent_nested_ownership_proof":False}))
        directory_sync(evidence);review_closed=True
    except BaseException:latch("outer_review_unconfirmed")
    final_ns=time.monotonic_ns()
    # No explicit evidence write/close/mutation or child operation after this clock.
    within55=final_ns<FINAL_DEADLINE
    if not within55:latch("outer_final_deadline")
    passed=grade and retained and review_closed and within55 and first_failure is None
    summary={"schema":"riauth.d01-composed-outer-return/v1","result":"passed" if passed else "failed",
        "first_failure":first_failure,"controller_exit":code,"controller_reaped":reaped,
        "controller_group_empty":empty,"nested_node_cleanup":nested,
        "independent_nested_ownership_proof":False,"retained_before_grade":retained,
        "review_closed":review_closed,"complete_bounded_capture":complete,
        "within55":within55,"elapsed_seconds":(final_ns-START_NS)/1e9,
        "stdout_bytes":len(raw["out"]),"stderr_bytes":len(raw["err"]),
        "stdout_sha256":hashlib.sha256(raw["out"]).hexdigest(),
        "stderr_sha256":hashlib.sha256(raw["err"]).hexdigest(),"actual_product_or_driver":False}
    sys.stdout.write(json.dumps(summary,sort_keys=True,separators=(",",":"))+"\n");sys.stdout.flush()
    return 0 if passed else 1
if __name__=="__main__":sys.exit(main())
```

### Whole-byte and AST inverse proof

Static forward construction required exactly one `dict(os.environ)` occurrence
in the pinned original. Substituting the exact dictionary-comprehension bytes
once constructed the complete candidate. Replacing that sole expression with
the original reproduces all 21320 original bytes and SHA 40302bd5 exactly.
Both complete module ASTs parsed as DATA.

The only new DictComp has key `key`, value `os.environ[key]`, one generator
over `os.environ`, and one exact `key != "__CF_USER_TEXT_ENCODING"` condition.
Replacing only that AST node with the original `dict(os.environ)` Call
reproduces the complete original module AST excluding locations. The outer
comparison to ENV and all enclosing OR predicates, handler branches,
short-circuit order, imports and entrypoint remain unchanged.
No reviewed function or class was called, and the proposed comprehension or
predicate was not evaluated during the proof.

The complete eleven-function set is unchanged. Main is changed only by that
expression, with candidate main SHA-256 `e721fff658e5f3b2dfbc506806be597859a07dbe1c56f251052f7817b44f2485`;
its byte inverse restores the whole original main. All ten other complete
function spans and ASTs are byte-identical:

```json
{
  "latch": {
    "bytes": 88,
    "sha256": "957eba18cdfb5d25bd5ee510141c74c9f942978d4540086d9db62b1f4f0fbec9"
  },
  "private_directory": {
    "bytes": 532,
    "sha256": "12296c9587b151b2d998c6f174cb3579b4580ac210cae840e5a208c781a3eabc"
  },
  "save": {
    "bytes": 342,
    "sha256": "ae13bed122b92c6c27e1a63f828c87bc4ce4477b6079836513b36d547f631900"
  },
  "encode": {
    "bytes": 125,
    "sha256": "89ce1b76ca343fda5622acb07b3fce0d0ce63242704cf3b9c53b09c1ca0ac68e"
  },
  "duplicate_free": {
    "bytes": 173,
    "sha256": "3134f45bbdba498f6684c83430ebfe93793174940b606f45cd1d1c0f8844a64b"
  },
  "true_int": {
    "bytes": 81,
    "sha256": "646a375f4d50ebd66ab07b30ecf5c325b6027601ecc27d022ee50f2c5249bf04"
  },
  "group_cleanup": {
    "bytes": 3773,
    "sha256": "de0d06fd97cf4a15bb4a0443bdce42e9e27c4ea7cdfc3ab6ee9d80b796201231"
  },
  "pinned_file": {
    "bytes": 1228,
    "sha256": "b9860dcbdd16a6ace5ccc082c7520a733947112b50484665ece4227240a7dfd0"
  },
  "directory_sync": {
    "bytes": 286,
    "sha256": "739e063a7283f01457ab73c8a4f0461a297a31e171fe5760eb30ce1d00c2d18c"
  },
  "accepted_return": {
    "bytes": 1182,
    "sha256": "c1ed36f6f66c9ee3e241c710717371ae329c8b4d033bc846ccd81effab34b6e6"
  }
}
```

All twenty top-level assignments preserve bytes and ASTs, including ENV,
PINS, source SHA/size, ROOT, Node path, startup/capture/cleanup/final clocks,
resource floor and start threshold, output/sample caps and first-failure
initial state. Exact Popen, default SIGCHLD setter, waitid/killpg, save,
directory-sync and accepted-return call ASTs match the original.
The sole child call still passes ENV without inheritance and requests a
fresh session. The unchanged controller input is 242290 bytes/SHA
`d29c5c015cfa6f7b2ae3a88fea221e7da7bc3699e03f1dfba4f29e29e2c5778a`.
Neither it nor Node/harness/source DATA was edited or executed here.

Thus the one-child limit, actual wait status rather than fallback zero,
owned pre-reap signals, no broad discovery/post-reap delivery, unknown
nested-Node cleanup on abnormal controller failure, complete/capped capture
distinction, fsync-before-expectations, first-failure latch, provisional review
and final-clock decision remain unchanged. Controller thirty/thirty-five-second
limits and outer five/forty/fifty/fifty-five-second controls are not widened.
No write-safety, hard I/O deadline, native loader or successful child launch
claim follows from allowing this metadata name.

### Actual static scope and future release boundary

Complete receipt/source/hash reads and data-only candidate construction,
Python AST syntax, whole-byte inverse, sole-node normalized AST inverse,
ten protected full-function spans, restored main, twenty unchanged assignments
and protected call AST comparisons exited 0. No new extraction/check error
occurred in this source phase; older errors remain in the untouched prefix.
Static Python/Git tools ran; the reviewed launcher/controller/diagnostic/helper/
observer/memory candidate was not imported or executed. No native/version
probe, invocation of proposed signal/wait/kill controls, provider/browser/Driver
call, runtime retry, cache deletion or other worker contact occurred.

Root's startup diagnostic exit 0 is limited to its recorded startup check.
The earlier outer invocation remains exit 1, with no completed memory packet;
old515 stays failed and its child exit/cause remain UNKNOWN. Historical sender
and true cleanup-start remain UNKNOWN. This proposal does not infer a memory
pass, new assertion count, exact prior preflight cause or fixture completion.

Only after root's immutable full/independent review may root separately change
its private launcher DATA to the exact candidate hash, verify readback and
staging/source/native metadata, and release ONE fresh original command/nonce.
Existing d29 controller DATA, closed caller/child ENV, pins and all ownership/
resource/evidence rules remain the same. No source materialization or invocation
is authorized here; no local validation slot was acquired, used or released.
Runtime remains **HELD**. Root owns integration, any future private-file change,
execution reservation, receipts, cleanup decision and status.

Final actual appendix checks: canonical fence/diff extraction reproduced the
21381-byte candidate, 717-byte exact diff and both hashes; whole-byte and sole
AST-node inverses matched. Entire 82139-byte ecb prefix and report-only scope
matched. Documentation links/build layout, whitespace and scope checks exited
0, with no link or build-layout errors. No executable/private DATA file was
changed and no reviewed runtime was invoked.

## 2026-10-03 terminal-child signal permission deferral — SOURCE DESIGN ONLY

Reservation `wave30_D01_terminal_signal_permission_cleanup_design` owns only
this appendix. The complete `ab8d7e7325bc961a0d538950456dc1b909f193b3` prefix is
preserved: 116152 bytes, 2027 lines, SHA-256
`d515a32069e3824859534ca391dbd0af966e01d78811afaf3b0c348c3c2a1302`.
The new actual whole envelope remains **FAILED**. Old515's native cause/child
exit and the first generic preflight failure's exact cause remain UNKNOWN.

### Actual failed run and approved closed evidence

Root separately released nonce `fd49b9cead44f00b` with outer 8a4 and unchanged
controller d29. The complete public project-orchestrator receipts were read:

| planning/evidence receipt | Bytes | SHA-256 |
| --- | ---: | --- |
| wave30-d01-new36-env-outer-fd49b9c-tool.json | 801 | `abb3a7110efe2a9fa7ae958410ce3c272a7ec7a3a5883df88299b44f42abedd2` |
| wave30-d01-new36-env-cleanup-root-verification.json | 4822 | `afae835c42ed0784e316b84534212701e82524ec976542ef7523feae9b113458` |

Tool/outer exit is **1**, elapsed 0.197602042. The captured controller exit is
**1**; complete capture, reaped and direct group empty are true. Outer's
first failure is `owned_cleanup_unconfirmed`, with diagnostic
`signal_owned_group` / errno 1 / identity 2, WNOWAIT code 1/status 1 and
real consuming-reap code 1/status 1. The closed controller return is also
failed, despite inner Node's actual exit **0**, reaped/group empty true,
WNOWAIT code 1/status 0 and consuming-reap code 1/status 0. Its first
failure is the same signal permission diagnostic. Event code 1 and exit
status are separate recorded fields; Node did not exit 1.

The packet records 36 completed cases, 51 cells, 311714 assertions,
2294 privacy checks, no child case failure and no unreached cases.
That is actual packet evidence inside a **FAILED whole envelope**, not a
whole-memory acceptance or a browser/product result. Both provisional
reviews have grade_before_final_clock false. The original diagnostic records
errno and event/status fields, not an exception class, delivered signal,
kernel policy or OS-defect cause; none is retrospectively inferred.

Root's verification records group absence for original PIDs/groups 31721 and
31722, whole_envelope_pass false and release at
2026-10-03T03:24:45.281923+00:00. Root also supplied fresh exact ps absence.
These are root's retained/supplied observations; this source phase did not
query a PID, group or native process. Root released the lane on actual
consuming status and absence proof before this report, not on a retry or
another nonzero signal.

Only the six explicitly approved closed metadata/packet files in the exact
nonce's two directories were read. Directories were UID 501/mode 0700;
files were regular nonsymlinks, UID 501/mode 0600. No other private capture,
credential, log or source-directory inventory was read. Their byte/hash pins:

```json
{
  "d01-composed-outer-fd49b9cead44f00b/retained.json": {
    "bytes": 2375,
    "sha256": "c9bd2a1e4e4a5924afc50b8aa5df211a24dcaa8380c1d0d0626d6ae7467e8037",
    "mode": "0600",
    "uid": 501
  },
  "d01-composed-outer-fd49b9cead44f00b/review.json": {
    "bytes": 305,
    "sha256": "1d8ce26bd1b00fabd5ba178c6bb7409e06b0b6669b4e7667c0ebbc61177b8e67",
    "mode": "0600",
    "uid": 501
  },
  "d01-composed-outer-fd49b9cead44f00b/controller.stdout": {
    "bytes": 359,
    "sha256": "27a9b954dedb3a0716704adef11b3c2aaedc9d9c727aad83bca1b686c8f505db",
    "mode": "0600",
    "uid": 501
  },
  "d01-continuation-memory-fd49b9cead44f00b/retained.json": {
    "bytes": 748,
    "sha256": "0030115fd975abc7e5771fb03a4b584aa056a278f515002e195546cd3cdfe8a5",
    "mode": "0600",
    "uid": 501
  },
  "d01-continuation-memory-fd49b9cead44f00b/review.json": {
    "bytes": 7116,
    "sha256": "0a89e9fadafad045a9ea9632b056ef425eaa5f0f998cbc4baf205e680e8fcd79",
    "mode": "0600",
    "uid": 501
  },
  "d01-continuation-memory-fd49b9cead44f00b/child.stdout": {
    "bytes": 6862,
    "sha256": "7bd435978f008a4a904e3c2d00dc2b0e49a6a1323874d080dd56ba436d78bbe3",
    "mode": "0600",
    "uid": 501
  }
}
```

The complete original Node packet below is 6862 bytes including its existing
final LF, SHA-256
`7bd435978f008a4a904e3c2d00dc2b0e49a6a1323874d080dd56ba436d78bbe3`.
The packet's successful case observations do not override either failed
envelope or claim the new cleanup proposal passed.

```json
{"schema":"riauth.d01-continuation-memory/v1","source":{"candidate_sha256":"505b1c0fec96e248dc67db46379bd801fa06a2678462aa094fb2fc1bc5ea398f","candidate_bytes":33701,"baseline_sha256":"7ab23019e838512a84600d4069fef2e419b9bb7f07ff5aaa2c7477af956f9ca8","baseline_bytes":32502,"diff_sha256":"b6f884994720949a44bfb417ff68de56e71e04f04d7357686ffc71452a873ed2","logic_sha256":"6ae4fd802a5515aade80b0b46cffe38030d9b4a46db427b32ed47fb9e20e583c","full_inverse":true},"planned":[{"name":"phase_entry_then_continuation","group":"phase_binding"},{"name":"password_survives_until_password_dispatch","group":"secret_lifetime"},{"name":"missing_password_refuses_without_input","group":"secret_lifetime"},{"name":"unowned_context_refuses_without_operations","group":"phase_binding"},{"name":"undeclared_ref_refuses_input","group":"phase_binding"},{"name":"stale_ref_cannot_cross_cells","group":"phase_binding"},{"name":"invalid_kind_repeated_cleanup_clears_before_await","group":"secret_lifetime"},{"name":"helper_zero_final_snapshot_then_cleanup","group":"helper_handoff"},{"name":"helper_zero_missing_page_still_fails","group":"helper_handoff"},{"name":"helper_nonzero_final_still_refuses","group":"helper_handoff"},{"name":"helper_zero_other_kind_still_refuses","group":"helper_handoff"},{"name":"helper_zero_prepared_phase_still_refuses","group":"helper_handoff"},{"name":"partial_line_drains_before_output_and_next_cell","group":"partial_framing"},{"name":"partial_exact_cap_is_accepted","group":"partial_framing"},{"name":"partial_over_cap_refuses","group":"partial_framing"},{"name":"partial_joined_controller_refuses","group":"partial_framing"},{"name":"partial_unavailable_controller_refuses","group":"partial_framing"},{"name":"partial_deadline_prevents_extra_poll","group":"partial_framing"},{"name":"partial_completed_late_still_refuses","group":"partial_framing"},{"name":"retained_start_budget_refuses_next_cell","group":"phase_binding"},{"name":"inclusive_cleanup_deadline_does_not_invent_join","group":"cleanup_latch"},{"name":"first_page_failure_survives_later_helper_error","group":"cleanup_latch"},{"name":"missing_absence_proof_prevents_release","group":"cleanup_latch"},{"name":"cleanup_exception_is_private","group":"cleanup_latch"},{"name":"pending_metadata_command_prevents_release","group":"cleanup_latch"},{"name":"projection_exact_pairs_and_private_field_omission","group":"public_projection"},{"name":"projection_otp_observed_without_action_or_value","group":"public_projection"},{"name":"projection_required_consent_allow_roundtrip","group":"public_projection"},{"name":"projection_optional_consent_continue_roundtrip","group":"public_projection"},{"name":"projection_role_and_label_rejections","group":"public_projection"},{"name":"projection_incomplete_and_error_rejections","group":"public_projection"},{"name":"projection_missing_or_wrong_advertised_action","group":"public_projection"},{"name":"projection_malformed_refs","group":"public_projection"},{"name":"projection_disjoint_rows_do_not_infer_association","group":"public_projection"},{"name":"projection_duplicate_actions_preserve_multiplicity","group":"public_projection"},{"name":"projection_latest_success_replaces_previous_snapshot","group":"public_projection"}],"attempted":["phase_entry_then_continuation","password_survives_until_password_dispatch","missing_password_refuses_without_input","unowned_context_refuses_without_operations","undeclared_ref_refuses_input","stale_ref_cannot_cross_cells","invalid_kind_repeated_cleanup_clears_before_await","helper_zero_final_snapshot_then_cleanup","helper_zero_missing_page_still_fails","helper_nonzero_final_still_refuses","helper_zero_other_kind_still_refuses","helper_zero_prepared_phase_still_refuses","partial_line_drains_before_output_and_next_cell","partial_exact_cap_is_accepted","partial_over_cap_refuses","partial_joined_controller_refuses","partial_unavailable_controller_refuses","partial_deadline_prevents_extra_poll","partial_completed_late_still_refuses","retained_start_budget_refuses_next_cell","inclusive_cleanup_deadline_does_not_invent_join","first_page_failure_survives_later_helper_error","missing_absence_proof_prevents_release","cleanup_exception_is_private","pending_metadata_command_prevents_release","projection_exact_pairs_and_private_field_omission","projection_otp_observed_without_action_or_value","projection_required_consent_allow_roundtrip","projection_optional_consent_continue_roundtrip","projection_role_and_label_rejections","projection_incomplete_and_error_rejections","projection_missing_or_wrong_advertised_action","projection_malformed_refs","projection_disjoint_rows_do_not_infer_association","projection_duplicate_actions_preserve_multiplicity","projection_latest_success_replaces_previous_snapshot"],"completed":["phase_entry_then_continuation","password_survives_until_password_dispatch","missing_password_refuses_without_input","unowned_context_refuses_without_operations","undeclared_ref_refuses_input","stale_ref_cannot_cross_cells","invalid_kind_repeated_cleanup_clears_before_await","helper_zero_final_snapshot_then_cleanup","helper_zero_missing_page_still_fails","helper_nonzero_final_still_refuses","helper_zero_other_kind_still_refuses","helper_zero_prepared_phase_still_refuses","partial_line_drains_before_output_and_next_cell","partial_exact_cap_is_accepted","partial_over_cap_refuses","partial_joined_controller_refuses","partial_unavailable_controller_refuses","partial_deadline_prevents_extra_poll","partial_completed_late_still_refuses","retained_start_budget_refuses_next_cell","inclusive_cleanup_deadline_does_not_invent_join","first_page_failure_survives_later_helper_error","missing_absence_proof_prevents_release","cleanup_exception_is_private","pending_metadata_command_prevents_release","projection_exact_pairs_and_private_field_omission","projection_otp_observed_without_action_or_value","projection_required_consent_allow_roundtrip","projection_optional_consent_continue_roundtrip","projection_role_and_label_rejections","projection_incomplete_and_error_rejections","projection_missing_or_wrong_advertised_action","projection_malformed_refs","projection_disjoint_rows_do_not_infer_association","projection_duplicate_actions_preserve_multiplicity","projection_latest_success_replaces_previous_snapshot"],"completed_groups":{"phase_binding":5,"secret_lifetime":3,"helper_handoff":5,"partial_framing":7,"cleanup_latch":5,"public_projection":11},"cells":51,"stub_counts":{"collector_marker":2,"controller_poll":107,"navigate":2,"snapshot":41,"type":2,"click":4,"collector_stop":27,"collector_clock":213,"kill":27,"end_session":27,"windows":27,"collector_readback":27,"collector_persist":27},"assertions_completed":311714,"privacy_checks_completed":2294,"first_failure":null,"unreached":[],"elapsed_ms":59.467375,"full_candidate_only":true,"actual_tools_or_product":false}
```

### Smallest shared hunk and sufficiency boundary

Both original staged sources were read fully as DATA via O_NOFOLLOW,
verified regular UID-501/mode-0600 and matched their immutable originals:
controller d29 from `560525a07eea0a696ffeb36599308795348dab47`,
`docs/roadmap/local-wave30-d01-root-user-review.md` final Python fence;
outer 8a4 from immutable ab8's candidate fence in this report.
Their group_cleanup bodies match all 3773 bytes / 72 lines / SHA-256
`de0d06fd97cf4a15bb4a0443bdce42e9e27c4ea7cdfc3ab6ee9d80b796201231`.

Insert only the following six lines after the existing ProcessLookupError
pass around the pre-reap SIGKILL attempt. Addition: 303 bytes, SHA-256
`0671bad33042dc7708d25f22dd1e8332193adbe15496f5a8addc6e4727717453`.

```python
        except PermissionError as error:
            number=error.errno
            if type(number) is not int or number!=1 or not terminal(observation):raise
            if diagnostic["stage"] is None:
                diagnostic["stage"]="signal_owned_group"
                diagnostic["errno"]=number
```

Only PermissionError, strict type-int/nonbool errno 1, and the original
already validated terminal WNOWAIT event permit deferral. The captured
number is read once. No permission exception is generally ignored.
The unchanged preceding flow checked eligible PID/cached returncode,
installed default SIGCHLD guard, exact original child/event/PID/signo/code/
status types, and the ownership guard before this signal attempt.
The only observations reaching the branch were made with WNOWAIT;
a live-path None cannot satisfy terminal. No new group search, ownership
transfer, reaper, fallback or delivery authority is introduced.

The branch records the original fixed stage/errno without latching that
one observation as a final cleanup failure. It neither sets code/reaped/
empty nor claims that a signal was delivered. Those results still come
only from the unchanged consuming waitid and group-absence block. The
diagnostic may now describe a deferred observation; stage/errno alone is
not a success verdict. If a later reap/absence operation fails, first
failure is latched by the unchanged code and result flags remain refusing,
even though the first diagnostic remains the earlier permission observation.

Both complete caller bodies were read: controller main refuses nonzero/
unknown child status, incomplete/capped output, invalid/source-mismatched
packet, child failure, missing counts, unreaped/nonempty cleanup, evidence
failure and final deadline; outer main additionally retains actual status
before comparisons and requires the exact closed zero-exit passing return.
Neither caller accepts merely a present child, diagnostic or WNOWAIT event.
Any preexisting first failure stays latched; this branch never clears it.

Counterexamples were assessed by source paths, not executed:

- Live/unknown event, wrong PID/nonterminal event or cached returncode refusal
  cannot take the new terminal-only deferral; original ownership/refusal
  paths remain. A live child that exits between observation and the signal
  still has None here and conservatively fails.
- Permission errno 13, any other errno, bool True, float/string 1 or missing
  errno rethrows to the original failure path. Other exception classes
  retain the original outer catch.
- ECHILD, no consuming result before deadline, wrong consuming event/PID or
  another reap failure leaves status unknown/reaped false and fails. There
  is no Popen.wait fallback or synthesized zero.
- A surviving group member after leader reap, or a permission/unknown result
  from signal-zero query, leaves empty false and fails. The terminal leader
  alone cannot prove all group members gone.
- A real nonzero child result remains nonzero and both callers fail; an
  existing capture/packet/deadline failure remains first even if cleanup
  subsequently proves reaped and empty.
- After reap only the existing zero-signal existence query remains. A reused
  PID/group cannot authorize a new nonzero signal; presence or uncertainty
  refuses instead of triggering discovery/delivery.

Under the existing fresh single-reaper/waitable-child premise, I found no
new authority/refusal bypass in this six-line branch. This is conditional
source sufficiency, not execution or an exact host/kernel attestation.
General current Python docs explain that WNOWAIT preserves waitability for
later status retrieval, and PermissionError spans more than EPERM, so the
numeric restriction matters.
[Python waitid/WNOWAIT documentation](https://docs.python.org/3.14/library/os.html#os.WNOWAIT),
[Python PermissionError documentation](https://docs.python.org/3.14/builtins/exceptions.html#PermissionError).
Those pages currently identify 3.14.8; they support general API reasoning,
not the installed 3.14.6 native instruction mapping or an errno cause.
The ec88 interpreter/build/loader and no-foreign-reaper assumptions remain.

### Complete changed function DATA

Candidate shared body: 4076 bytes, 78 lines, SHA-256
`c6c97888c986a89d7ac2b608f6bd0ebcb000279c90b1e1022a822b0c4e2fe75a`.
It preserves every original byte outside the six-line insertion.

```python
def group_cleanup(child):
    # Only the exact unreaped Popen child can authorize a nonzero group signal.
    reaped=False;empty=False;code=None;owned=False;stage="child_identity"
    diagnostic={"stage":None,"errno":None,"identity":0,
                "waitid_code":None,"waitid_status":None,
                "reap_code":None,"reap_status":None}
    def failed(at,error=None,label="owned_cleanup_unconfirmed"):
        if diagnostic["stage"] is None:
            diagnostic["stage"]=at
            number=getattr(error,"errno",None)
            diagnostic["errno"]=number if true_int(number,1,4095) else None
        latch(label)
    def terminal(info):
        return (info is not None and type(info.si_pid) is int and info.si_pid==child.pid and
                type(info.si_signo) is int and info.si_signo==signal.SIGCHLD and
                type(info.si_code) is int and
                info.si_code in (os.CLD_EXITED,os.CLD_KILLED,os.CLD_DUMPED) and
                true_int(info.si_status,0,255) and
                (info.si_code==os.CLD_EXITED or info.si_status>0))
    eligible=true_int(child.pid,1,2147483647) and child.returncode is None
    try:
        if not eligible:raise ValueError("child_identity")
        stage="signal_disposition"
        if signal.getsignal(signal.SIGCHLD)!=signal.SIG_DFL:
            raise ValueError("signal_disposition")
        stage="waitid_identity"
        observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
        if observation is None:
            stage="live_group_identity"
            try:
                owned=os.getpgid(child.pid)==child.pid
                if owned:diagnostic["identity"]=1
            except ProcessLookupError:
                stage="exited_group_identity"
                observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG|os.WNOWAIT)
        if observation is not None:
            if not terminal(observation):raise ValueError("waitid_identity")
            owned=True;diagnostic["identity"]=2
            diagnostic["waitid_code"]=observation.si_code
            diagnostic["waitid_status"]=observation.si_status
        if not owned or child.returncode is not None:raise ValueError("group_identity")
        stage="signal_owned_group"
        try:os.killpg(child.pid,signal.SIGKILL)
        except ProcessLookupError:pass
        except PermissionError as error:
            number=error.errno
            if type(number) is not int or number!=1 or not terminal(observation):raise
            if diagnostic["stage"] is None:
                diagnostic["stage"]="signal_owned_group"
                diagnostic["errno"]=number
    except Exception as error:
        failed(stage,error)
    # Reap this exact child even after identity/signal refusal; never signal here.
    if eligible:
        stage="reap_owned_child"
        try:
            while time.monotonic_ns()<OUTER_DEADLINE:
                observation=os.waitid(os.P_PID,child.pid,os.WEXITED|os.WNOHANG)
                if observation is not None:
                    if not terminal(observation):raise ValueError("reap_identity")
                    code=(observation.si_status if observation.si_code==os.CLD_EXITED
                          else -observation.si_status)
                    child.returncode=code;reaped=True
                    diagnostic["reap_code"]=observation.si_code
                    diagnostic["reap_status"]=observation.si_status
                    break
                remaining=(OUTER_DEADLINE-time.monotonic_ns())/1e9
                if remaining>0:time.sleep(min(.005,remaining))
            if not reaped:failed("reap_deadline")
        except Exception as error:
            failed(stage,error)
    if owned and reaped:
        # Signal zero is only an existence query; no signal is delivered after reap.
        try:os.killpg(child.pid,0)
        except ProcessLookupError:empty=True
        except Exception as error:failed("group_absence",error)
        if not empty:failed("group_absence",label="owned_group_not_empty")
    return code,reaped,empty,diagnostic
```

### Reconstructible full-module candidates and exact diffs

No 242-KB embedded source is duplicated. Apply the one exact module hunk
to the named immutable original and require the complete resulting hash.
These are shared-cleanup-only variants, with every original input/native
pin, environment, assignment, case/data binding and control preserved.

```json
{
  "controller": {
    "original_bytes": 242290,
    "original_lines": 2451,
    "original_sha256": "d29c5c015cfa6f7b2ae3a88fea221e7da7bc3699e03f1dfba4f29e29e2c5778a",
    "candidate_bytes": 242593,
    "candidate_lines": 2457,
    "candidate_sha256": "4ff0525f366d9847f40b5d4abf140b6e1e07fc9167353d93c0302a8fa0fc218f",
    "diff_bytes": 669,
    "diff_sha256": "d288d814ebdabc6a184f6506cd39b6f168cc1808f42b955ae1d75c33f2a2ae1c",
    "unchanged_assignment_count": 16,
    "whole_byte_inverse": true,
    "whole_ast_inverse": true
  },
  "outer": {
    "original_bytes": 21381,
    "original_lines": 380,
    "original_sha256": "8a4da1306765f331f18b4fd56cb71dff03395f40fb2246475be5e47b035e1e1b",
    "candidate_bytes": 21684,
    "candidate_lines": 386,
    "candidate_sha256": "9942573e2a96960fe878057d9d833e7333f86d6294f4103adefacfe8eb9a070b",
    "diff_bytes": 657,
    "diff_sha256": "3a1249ca87efbc0ba21890f409e80c9978dcb7fa1b61df60d04d69fb35cbe840",
    "unchanged_assignment_count": 20,
    "whole_byte_inverse": true,
    "whole_ast_inverse": true
  }
}
```

Controller exact diff:

```diff
--- controller-original.py
+++ controller-DESIGN-terminal-eperm.py
@@ -2286,6 +2286,12 @@
         stage="signal_owned_group"
         try:os.killpg(child.pid,signal.SIGKILL)
         except ProcessLookupError:pass
+        except PermissionError as error:
+            number=error.errno
+            if type(number) is not int or number!=1 or not terminal(observation):raise
+            if diagnostic["stage"] is None:
+                diagnostic["stage"]="signal_owned_group"
+                diagnostic["errno"]=number
     except Exception as error:
         failed(stage,error)
     # Reap this exact child even after identity/signal refusal; never signal here.
```

Outer exact diff:

```diff
--- outer-original.py
+++ outer-DESIGN-terminal-eperm.py
@@ -123,6 +123,12 @@
         stage="signal_owned_group"
         try:os.killpg(child.pid,signal.SIGKILL)
         except ProcessLookupError:pass
+        except PermissionError as error:
+            number=error.errno
+            if type(number) is not int or number!=1 or not terminal(observation):raise
+            if diagnostic["stage"] is None:
+                diagnostic["stage"]="signal_owned_group"
+                diagnostic["errno"]=number
     except Exception as error:
         failed(stage,error)
     # Reap this exact child even after identity/signal refusal; never signal here.
```

Full-byte inverse removes the one exact 303-byte insertion and restores
each entire original module hash. The sole added AST node is the second
ExceptHandler on the existing inner killpg Try, after ProcessLookupError.
It matches the exact six-line branch AST and contains no latch call.
Removing that handler reproduces both complete module ASTs, excluding
locations. All eleven other controller functions and ten other outer
functions retain exact full spans/ASTs, including both unchanged caller
main bodies. Their preserved identities are:

```json
{
  "controller": {
    "latch": {
      "bytes": 88,
      "sha256": "957eba18cdfb5d25bd5ee510141c74c9f942978d4540086d9db62b1f4f0fbec9"
    },
    "within": {
      "bytes": 62,
      "sha256": "0b2b1be269404c27b4b554272d3cf5aaf20c481563e6407467ff7dc736cebfd9"
    },
    "assemble_payload": {
      "bytes": 240,
      "sha256": "8978fe5175b1b6f74e8733b7dba8d0706e9d020fad959dc3cd88bed37a95e60b"
    },
    "regular_node": {
      "bytes": 511,
      "sha256": "d29f5a1303626b60e8e7c061a86ebdc56fef40d17ce87df77f3feb8c4d907aa9"
    },
    "private_directory": {
      "bytes": 532,
      "sha256": "12296c9587b151b2d998c6f174cb3579b4580ac210cae840e5a208c781a3eabc"
    },
    "save": {
      "bytes": 342,
      "sha256": "ae13bed122b92c6c27e1a63f828c87bc4ce4477b6079836513b36d547f631900"
    },
    "encode": {
      "bytes": 125,
      "sha256": "89ce1b76ca343fda5622acb07b3fce0d0ce63242704cf3b9c53b09c1ca0ac68e"
    },
    "duplicate_free": {
      "bytes": 173,
      "sha256": "3134f45bbdba498f6684c83430ebfe93793174940b606f45cd1d1c0f8844a64b"
    },
    "true_int": {
      "bytes": 81,
      "sha256": "646a375f4d50ebd66ab07b30ecf5c325b6027601ecc27d022ee50f2c5249bf04"
    },
    "closed_packet": {
      "bytes": 3772,
      "sha256": "bcd1ae35c59bd0c48a4866a922aa21f98b8b0845134767428468e21abd5eef77"
    },
    "main": {
      "bytes": 8497,
      "sha256": "d0f1b32db1ea73595e8c58612850fbb0690fabdc772eccc8ad012e0190541461"
    }
  },
  "outer": {
    "latch": {
      "bytes": 88,
      "sha256": "957eba18cdfb5d25bd5ee510141c74c9f942978d4540086d9db62b1f4f0fbec9"
    },
    "private_directory": {
      "bytes": 532,
      "sha256": "12296c9587b151b2d998c6f174cb3579b4580ac210cae840e5a208c781a3eabc"
    },
    "save": {
      "bytes": 342,
      "sha256": "ae13bed122b92c6c27e1a63f828c87bc4ce4477b6079836513b36d547f631900"
    },
    "encode": {
      "bytes": 125,
      "sha256": "89ce1b76ca343fda5622acb07b3fce0d0ce63242704cf3b9c53b09c1ca0ac68e"
    },
    "duplicate_free": {
      "bytes": 173,
      "sha256": "3134f45bbdba498f6684c83430ebfe93793174940b606f45cd1d1c0f8844a64b"
    },
    "true_int": {
      "bytes": 81,
      "sha256": "646a375f4d50ebd66ab07b30ecf5c325b6027601ecc27d022ee50f2c5249bf04"
    },
    "pinned_file": {
      "bytes": 1228,
      "sha256": "b9860dcbdd16a6ace5ccc082c7520a733947112b50484665ece4227240a7dfd0"
    },
    "directory_sync": {
      "bytes": 286,
      "sha256": "739e063a7283f01457ab73c8a4f0461a297a31e171fe5760eb30ce1d00c2d18c"
    },
    "accepted_return": {
      "bytes": 1182,
      "sha256": "c1ed36f6f66c9ee3e241c710717371ae329c8b4d033bc846ccd81effab34b6e6"
    },
    "main": {
      "bytes": 11205,
      "sha256": "e721fff658e5f3b2dfbc506806be597859a07dbe1c56f251052f7817b44f2485"
    }
  }
}
```

Controller's sixteen and outer's twenty top-level assignments are exact.
The whole-module inverse also protects imports, entrypoint, SIGCHLD setter,
reap guards, timing/IO/resource caps, schemas, first-failure/evidence order,
51-cell/36-case payload code and actual-status-returning logic. The
Node payload remains 151979 bytes / a5d564; no case/function code within
that payload was changed or evaluated. No nonzero post-reap signal,
errno-wide exception handler, merely-presence acceptance or assertion
relaxation is added.

### Concrete paired-runtime binding blocker — not silently changed

This reservation keeps outer SOURCE_BYTES **242290** and SOURCE_SHA **d29c5**
exact. The proposed controller is now **242593** bytes / SHA-256
`4ff0525f366d9847f40b5d4abf140b6e1e07fc9167353d93c0302a8fa0fc218f`.
The shared-hunk-only outer consequently cannot consume that changed controller:
its unchanged pinned_file size/hash guard must refuse before child creation.
Running that outer with old d29 instead would retain old d29's cleanup
behavior and cannot validate the proposed pair.

This is a certain composition blocker, not an ownership failure or permission
to weaken the source pin. A later separately reviewed root DATA-binding seam
must bind SOURCE_BYTES to 242593 and SOURCE_SHA to the complete 4ff0525f
candidate, with explicitly staged matching input bytes; all native/build pins
and other controls can remain unchanged. These two values are stated as
future prerequisites only, not included in either candidate/diff above.
No third source hunk, private file change or runnable composed pair is
authorized or claimed here. Root must resolve that binding gate before
reserving any new ONE paired invocation.

### Actual static checks and preserved limits

Immutable/private source DATA reads, complete approved receipts/closed captures,
candidate AST parsing, exact six-line handler-shape checks, both whole-byte
and complete normalized-AST inverses, preserved function spans and all
36 assignment spans/ASTs exited 0. Source variants have hashes as above;
private originals were not rewritten. No proposed helper/handler/controller/
launcher/harness/case was called, imported or executed; ordinary Git/Python
static tools and the two official primary documentation pages were used.
No reviewed process/signal/group query, native/version probe or diagnostic
repeat occurred; no runtime/Node/VM/Cargo/Driver/provider/browser/service
operation was performed.
No other worker was contacted, cache deleted or source alignment performed.

The actual corrected-environment envelope stays FAILED, with actual Node
packet counts retained rather than treated as overall acceptance. Old515's
native cause/child exit, old generic preflight's exact cause, historical sender
and true cleanup-start remain UNKNOWN. All other original row/security/fixture
gates remain unchanged. This append acquired, used and released no validation
slot. Runtime remains **HELD**; root owns both body reviews, the separate
source-pin binding decision, integration/publication, any future single
invocation, actual evidence, cleanup release and statuses.

Final actual appendix checks: canonical fences reproduced the six-line
303-byte insertion, 4076-byte function, both exact module diffs/candidate
hashes and complete byte/AST inverses. The original 6862-byte actual Node
packet hash and entire 116152-byte ab8 prefix matched. Documentation links/
build layout, whitespace and report-only scope checks exited 0 without
errors. The paired source-pin blocker remains explicit; no private DATA
writer or reviewed runtime was invoked.
