# D01 preparation appendix independent source review

Project: 891e7443-8dac-4c1b-897f-9e53cb59c7ee. Reservation:
wave30_D01_preparation_independent_review. Existing WT
f2e8500e-2e56-47e3-b60e-9f81bbc8cff2; clean entry parent
48288c1f10f24d47bb86b57edd0682014e9b64ee. This review creates only this report.

One concrete blocker remains in the reviewed preparation candidate: its blank-window
selector requires one total owned window, rather than one uniquely matching
about:blank window among validated auxiliary windows. The finding follows directly
from the source. It is not an explanation of any historical browser failure.
The bridge, source preservation, ownership and deadline checks below are static
findings; this review grants no adapter, preparation, browser journey or runtime
credit.

Read the entire ff18b723921b4a6049c73a678d2fd963962ab5c0 appendix beginning at
report line11192, including the complete1197-line candidate, four concrete
interfaces,64-line private carrier,53-line archive preflight,40-line partial
readback,242-line launcher,488-line preparation prefix,467-line c5 suffix,
author parser/check results, and historical limitations. Also read the complete
decoded225-line2e29 controller command, rather than merely comparing its hash.
Function and literal comparisons were performed independently using source
extraction and AST parsing. No author checker or function under review executed.

Immutable input identities:

| Input | Bytes / lines where applicable | SHA256 |
| --- | --- | --- |
| ff18b723 full user-browser review report | 949008 /13304 LF lines | 9d99eeb9503db9246b02cf03f0c49f77e110866d1d0d3cfd0635b72ac6bc9732 |
| Protected504a0c2a4cee19358580db2653451e065baf11ce report prefix | 772698 | d333b8c3e57e648eac1101d38c24c4b460e47cb1e262a6f3114d74f028a1bcb4 |
| ff18b723 full candidate, report fence line11918 | 111182 /1197 | 94ca708e5d1bf8ff01b87ae105250cc562d95e219e11a50e65bbeff226982f2b |
| Literal launcher, candidate1–242 | 41004 /242 | 57f397498e59719a6039815ca6ff59784322e7aecf4555894718a308529525ee |
| Concrete preparation prefix, candidate243–730 | 38597 /488 | 99f99bb386ce4461545fcd76c54d333d95bc92082f9a93789e826682777917b4 |
| Interfaces including literal carrier, candidate248–369 | 10955 /122 | 1c5ded39f080bcc25a812facd1358acbbd6056f97e6166fd374c2d61a9342a95 |
| c5d4d173857d040947a8cb3780c47c7141f98336 literal cell | 31581 /467 | d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d |
| Private carrier, plain fence line11325 and decoded JS literal | 3459 /64 | e621f4bd9fded5d72ce9a7cee7c6ae5027c5032682ccfaec1d9ce9a39d99dea1 |
| Archived prefix diff, report fence line11646 | 26518 /269, seven unified hunks | 25b6bce79ea11fbd8395488e1178a46073d51f8361c213b202d54e7776c42fef |
| 2e29c30d01ccd41a2aac3914e3ac20afa38d324f controller command | 17326 /225 | 5eb6ab2d3d23b49fd431c41f1932082fe0c6779ad724ac77b7fded10926b26b0 |
| f4ef05d8428b235511e81277ba6b4b72d5ec08ba descriptor helper | 35749 /757 | 75bfb8a63d68aee6ee6b2e500959c0e7d80a6331f1c690022c4300134c7d34f1 |
| Root public blank Driver contract JSON | 3893, mode0600 | b8145f029796a2063dfe167f026fe299d85afcaedbec5a7abacbeb5bf1a5ea27 |

The report path at each report commit above is
docs/roadmap/local-wave30-d01-user-browser-review.md. The c5 suffix in ff18b723
is the exact entire31581-byte c5 cell, not a selectively preserved function list.
The504a0 report is an exact prefix of ff18b723. Applying the archived seven
prefix-diff hunks to the25920-byte old prefix reproduces the38597-byte new prefix.
Applying the inverse hunks to that new prefix reproduces the old prefix; appending
the unchanged c5 cell reproduces the full57501-byte504a0 candidate, SHA256
401af55e346a9f55da682ca2de1762d8416da509816c52ef47e844c1fb8e6aeb.

The controller literal equals the complete2e29 sh fence after excluding the one
fence-delimiter LF: the source command and both here-document delimiters remain
unchanged. Its223-line Python body parses; its body is17304bytes/SHA256
7080e3a5601f1b2501f6a88336c803e1f4c530aa13a6e35674faaa3e3cd202f7.
This body hash is distinct from the full command hash. The helper Git object and
the existing WT helper agree exactly. Source-pin equality is not a fresh binary,
provider-version or native fixture check; none of those inputs executed here.

Read the unchanged controller's historical native pins in its source: riauth
7abf745c10691a012a1918c83089168d0e0d88b42764dbf8ed538b90c828b606;
riauth-maintenance
86490c7f71de6b7ae9d4dabdf9060a8757100f3aedbdaa670284d9e1206a2a95;
riauthctl bfbbb322f1a66d0ac9998beb9fb5838097cea0442cea3fd3a1d057fcb3f600cf.
Its BIN directory remains the retained wave27 d01-essentials-c01c39a
aarch64-apple-darwin/debug artifact directory. The verifier source is pinned to
9cefe7a56425bb73c17753e8766d92320b77da3b:scripts/recovery-drill-oidc.py and
f6dd1aa0b71de4793c9b86ee799012bb31a8fc2d6182976094b44df6ef04de3d;
the provider binary pin is
67a83dd6d6d747d50c5d296dffb23e32bae9a2c588c93ae2d77e4c607b455c72.
These are read-only source declarations, not newly measured artifact identities.
The fixed local-demo/Local demo creation argv, protected secret-file receipt,
helper invocation, provider probe and existing native calls are unchanged.

F1 witness and smallest prospective seam:

Candidate482–492 defines pWindows. It first validates an array of at most32
windows, every row's exact prepared PID, positive integer window ID, bounded title,
and globally unique window IDs. Candidate489–491 then requires the entire array
to contain exactly one row with title about:blank. Candidate667–669 calls
pWindows(raw,true), latches prepared_window_ambiguous on null and takes the one
returned window ID for exact PID/window binding.

Consequently, a list satisfying all existing row/ownership checks and containing
one exact about:blank title plus an auxiliary title is necessarily refused.
This violates the specified unique matching window contract. No Driver call,
synthetic case or function execution is needed to establish that implication.
The root public receipt records the selected blank window and later eleven
remaining owned windows; it does not include the initial raw list_windows array.
I do not claim that it proves a particular initial array or an actual invocation
of this candidate.

Propose only this three-line replacement against the immutable1197-line candidate.
All row/ownership/ID/title/cap checks execute before the fixed-title selection.
The false branch preserves the original all-window cleanup projection. Zero or
multiple exact blank matches remain refused. Selection uses an exact public title,
without geometry, rank, arbitrary regex, unrelated owner or a replacement listener.
No candidate or source file has been edited.

```diff
--- ff18b723-1197-line-candidate
+++ prospective-unique-owned-blank-selection
@@ -489,3 +489,3 @@
-  if(requireBlank&&(s.windows.length!==1||s.windows[0].title!=="about:blank"))
-    return null;
-  return s.windows.map(w=>({pid:w.pid,window_id:w.window_id}));
+  const rows=requireBlank?s.windows.filter(w=>w.title==="about:blank"):s.windows;
+  if(requireBlank&&rows.length!==1)return null;
+  return rows.map(w=>({pid:w.pid,window_id:w.window_id}));
```

The prospective variant exists only as an in-memory string for static comparison:
111211bytes/1197lines, SHA256
b1f8a15dc99b26f6671466fef7ac9103e1525c5ad9e23829aa13b4a2c8ef7c49.
Replacing those three added lines with the three original lines reproduces the
entire111182-byte reviewed candidate. After omitting only parser location offsets,
normalizing BigInt literal values for serialization and restoring the one pWindows
AST node, the complete prospective AST equals the original AST. All other function,
collector, launcher, interface, deadline and cleanup bytes remain exact. This is
a proposal for root reservation and subsequent focused validation, not an implemented
fix or a new universal gate.

Concrete interfaces and private transfer findings:

| Witness, candidate-relative lines | Source-derived result and limits |
| --- | --- |
| 244–247,251–333 | Four concrete bindings exist. Prepare decoding checks the launched isolated profile, spawned-by-driver ownership and matching positive owner PID. Bind decoding checks exact mutation-allowed native CDP identity, fixed about:blank title, one active blank tab, bounded target/tab capabilities and inert root shape. The bind reply lacks a PID/window field; exact requested PID/window supplies that part of the binding, not an invented returned field. |
| 259–271,311–333 | Snapshot decoding requires complete semantic_v2 viewport state, matching target/tab, an inert root, empty interactive refs, and all seven exact omission counters zero. The surveyed root role is not promoted into an interactive ref. This is a source contract derived from one closed blank survey, not proof of unobserved provider variants. |
| 334–348 | The one carrier call uses actual tools.exec_command fields exit_code, output and session_id. Its command contains public source and the owned lab path, never the password. login:false/tty:false,10000ms yield and1000 output-token cap remain explicit. The returned ToolResult is held transiently; numeric exit, returned session handle and output character count are retained through pKeep before comparisons or payload parsing. |
| 349–367 | A pending child is refused and its actual positive session handle retained for cleanup joining. A completed child requires exit0 and at most256 ASCII carrier characters; the decoded value is separately bounded30–128 ASCII bytes. The exact three-key schema is riauth.d01-private-input/v1, bytes, value; bytes must be a safe integer and equal the fixed-alphabet value length. result/raw/packet/value/request references are cleared, including exceptional paths, before a later await. |
| 689–698 | The synchronous, one-use sink alone stores the validated value in the private ephemeral d01_fresh_password_input slot. It does not return the value, print it, send it to text/notify, include it in public projection or write another durable password record. Transfer immediately precedes the literal c5 cell without model yield. |
| 520 onward;1140–1152 | Cleanup clears the private input before its first await; c5 password dispatch validates the same alphabet, uses the slot solely for Driver typing, then clears it. Pending carrier joins discard raw late output and retain only numeric/size metadata. Clearing references is not a claim to erase immutable strings from memory. |

The plain64-line carrier and its decoded JS literal are byte identical. Its
unchanged48-line reader opens only browser-password relative to a verified0700
same-UID lab directory, with no-follow/nonblocking reads; checks0600 same-UID
regular file, one link, size30–128; caps the read at129bytes; compares file identity,
timestamps, size and directory identity before/after; decodes ASCII and accepts
only the fixed alphabet. The mutable bytearray is overwritten in finally.
The added wrapper has a5s alarm and one compact stdout success packet; a closed
failure packet and exit1 contain no error text. The carrier schema naturally exceeds
the password's128-byte bound, which explains the separate256-character stdout cap.
No actual secret file was read here. The existing controller's owned temporary
password file and reviewed client receipt-secret behavior remain unchanged.

Launcher, seed, ownership and timing findings:

| Witness | Source-derived result and boundary |
| --- | --- |
| Launcher22–39,176–210; archive literal line3 | Exactly one root-released input and one fresh inclusive START feed the archived controller. Archive preflight checks private-directory ownership/mode, fresh output names, helper pin, free space and unoccupied ports; three exact START/output substitutions are reversed against the retained controller template and AST before exclusive0600 archival. No second controller or post-seed model pause is introduced. |
| Launcher43–97,201–216 | Numeric child exit/session and received wall time are recorded before comparison; controller output is kept as a bounded16384-character internal buffer. The gate must be one canonical four-key browser_prepare_required/guard_pid/server_pid/lab event. Its two positive distinct PIDs and direct-child fixed-prefix lab come from the unchanged controller, not from an exec session ID or a release boolean. Pending/completed/refused results do not manufacture a gate. |
| Launcher217–232 and pSeedValid407–443 | AST extraction independently proves the producer and consumer have the same exact25 distinct keys. The consumer checks the pins, phase, workspace, positive handles, distinct paths, canonical gate relation and pending launcher receipt. Metadata/runtime booleans alone never establish PID/lab/Driver ownership. |
| Controller62–108,138–179 | Lab ownership begins at the actual fresh mkdtemp; children are exact Popen objects retained in children/names. Controller gate exposes os.getpid and the actual server PID. Listener owners must equal that Popen PID. Only CLI children use setsid plus the controlling-TTY setup; launcher session handles are not PIDs or process-group IDs. stop_child signals the exact owned Popen object and waits. No arbitrary PID/PGID/group-kill authority is inferred by the candidate. |
| Launcher98–170 | Before a valid gate, an exec handle cannot reveal the guard PID, lab or all child ownership. The failure path joins what it actually owns, records owned_stop_path_unknown where necessary, and cannot claim resource release. After a known gate it can request stop only in that exact lab and compare canonical child exits with fresh PID/port/lab absence. A numeric controller exit alone is not complete absence proof. |
| pPrepareEntry644–710 | Prepare-derived PID precedes validated window selection; the exact PID/window bind precedes the matching target/tab snapshot. Raw Driver result and decoded projection references are cleared between operations. The known opaque binding capabilities, gate PIDs/lab and returned exec handle enter c5 without rebind or fabricated ownership. F1 is the one proposed selection correction above. |
| pCleanup520–643 | Stop clock persistence precedes the one cooperative background close. Escalation requires the exact closed unverifiable/refused synthetic/background/delivery_failed/foreground profile and fresh still-owned PID-filtered windows; then only the known browser PID is killed. Ending a named session is separately validated and does not prove browser death. Joins and fresh PID/window/port/lab absence plus canonical child exits are necessary for PROVEN; unknown ownership/readback stays UNKNOWN. |
| Controller4–5,14–23,162–174; launcher/prefix/c5 deadlines | One inclusive900s envelope retains the840s active boundary and60s cleanup allocation. The prepare marker waits at most180s and does not reset START; the unchanged helper limit is600s inside the active budget. Source loops stop ordinary work at their deadlines. The8.5GiB stop threshold protects the intended8GiB floor through sampled disk checks; it is not a guaranteed continuous host floor or a separate proven8GiB stop. Driver/call limits and cleanup deadlines are cooperative observations, not hard kernel cancellation. |

Seed key list independently extracted from both ASTs:

```text
runtime_release single_controller fresh_paths_preflight start_epoch_ms
workspace exec_session guard_pid server_pid lab outer_out event_out cleanup_out
session controller_template_sha256 controller_payload_sha256 entry_source_sha256
helper_source_sha256 gate_event controller_pending interfaces_source_sha256 phase
driver_contract_receipt_sha256 payload_archive_out provider_out launcher_receipt
```

Complete c5 function source equality was checked for finiteObservation, diagnostic,
latch, observeController, pollController, timedDriver, cleanup, checked, page,
navigateAndSnapshot, entry, continuation and publicPredicate. The five CLOCK,
STOP, READBACK, MARKER and PERSIST collector literals remain byte exact because
the entire c5 suffix is exact. All fourteen embedded Python source literals and
the separately extracted controller body parsed; no collector was called.

The separate actual shape input is the root's0600 public JSON at
planning/evidence/wave30-d01-root-blank-driver-contract.json in the project
orchestrator directory. Read its complete body and the distinct published
6e949b97573d7a4cc42cf5ac1ec43c5fc3963599 report notes at11253–11261. The receipt
describes an already closed isolated blank survey, a spawned owner PID, selected
window and exact binding, semantic_v2 inert root, seven zero omissions, unverifiable
cooperative close and root-owned exact-PID cleanup. Its actual prepared PID,
window, snapshot refs and binding capabilities are historical receipt values;
they are not fresh preparation seed values. Neither this receipt nor the published
note is an execution result for the four new interfaces.

No Sol6 public-decision adaptation or my independent public-projection proposal
at48288c1f10f24d47bb86b57edd0682014e9b64ee has been composed into ff18b723.
The unchanged c5 page/publicPredicate behavior remains the original body. Old
memory envelopes and prior real failures cannot count as tests of this new
launcher/private transfer/window adapter or of either separate projection proposal.
F1 does not infer a prior exception class, sender, lost value, historical cleanup
start, browser outcome or successful credential/consent/callback flow.

Actual static checks and limitations:

1. Immutable Git extraction, hashes, whole historical-prefix equality, forward and
   inverse seven-hunk reconstruction, full c5 suffix/function equality and exact
   plain/literal private carrier comparison passed.
2. Installed Node was used only as an Acorn source parser and AST/source comparison
   tool. Original and prospective full1197-line modules parse; the exact25 seed
   fields agree; masking only pWindows gives whole normalized AST equality.
   Python ast.parse checked embedded source strings, controller body and pinned
   helper; no reviewed module was imported or executed.
3. The first independent AST comparison failed in the static checker because JSON
   serialization did not support Acorn's BigInt literal value. The checker was
   corrected to serialize that literal value explicitly; it then passed. The
   next static attempt wrongly submitted the full shell here-document command
   to Python ast.parse and received SyntaxError at its shell header. Extracting
   only the223-line Python body corrected that checker operation; parsing passed.
   Neither failure was a candidate/runtime result.
4. Worktree check-docs and whitespace/scope validation are recorded below after
   creation of this report. The author appendix records its immutable baseline
   docs check exit1 at user-browser report line2813: the scanner treats the fenced
   call as a link whose target is SimpleNamespace(demo=d. Independently read the
   scanner and that source line; this review does not rerun the checker in the
   author WT, rewrite its protected prefix or characterize the scanner as broken.
5. No candidate VM/eval, harness/case execution, reviewed function call, adapter, controller,
   helper, HTTPServer construction, native/library/product/CLI/HTTP/network/Cargo,
   Driver/browser, secret-file read, runtime release or slot change occurred.
   Desktop preference remains RiWork Cua.ai Driver only for a future separately
   authorized fixture. Root owns review, integration, any follow-up source
   reservation and separate finite validation/runtime release. D01 primary42,
   D05, original A09 and closed rows remain unchanged.

Actual worktree checks: python3 scripts/check-docs.py exited0 with
Markdown links and build-directory layout checked; git diff --check exited0.
An independent byte/scope check confirmed LF-only text, final LF, no trailing
whitespace, no tracked-file change and exactly this one new report. The current
worktree docs result is separate from the immutable author's recorded exit1.


## 2026-10-03 — Independent review of the unique blank-window correction

Reservation: wave30_D01_unique_blank_window_independent_review, project
891e7443-8dac-4c1b-897f-9e53cb59c7ee, existing WT
f2e8500e-2e56-47e3-b60e-9f81bbc8cff2. Entry parent is
e906e75112ec7313a749cdc3f122e778cf6f2152. This phase appends only this report.
The complete20118-byte e906 prefix, SHA256
b66047cd443b139fe7244e4aae785ceb9ca3468acb6ed9b18c4cc32246f512aa,
remains exact, including the original F1 finding, prospective compact variant,
static checker failures/corrections and source-only limits.

Source acceptance: Sol4's immutable44d32c824218512d4bac6aca073dde6d260bdf37
correction resolves F1. The corrected branch selects exactly one literal
about:blank title only after every existing owner, row-cap, positive-ID,
unique-ID and bounded-title check has passed. The unchanged false branch
projects all validated owned windows for cleanup. No further concrete blocker
was found in this authorized delta. This accepts the source correction, not
preparation execution, a browser journey, unrelated projection work or runtime.

Read the complete correction appendix from user-browser report line13306:
its finding/provenance, both16-line diffs, corrected branch and callers,
complete83-line author static checker, retained result and limitations.
The preceding full ff18 source was read for e906. This phase independently
compared the entire corrected source against it, parsed both full programs,
reversed the sole branch edit and checked all33 other top-level function
declarations by source range. Thus the earlier body review covers every
unchanged byte; acceptance is not based only on a declared hash or function count.
The author checker was read, not executed; the independent checks below used
a separately written parser/comparison tool without evaluating either candidate.

| Immutable source/archive | Bytes / lines | SHA256 |
| --- | --- | --- |
| Full44d32c user-browser report | 1074063 /14803 | b6c235997b3db79889e51432f7887e5189570dc8f16e3ccd611e26f9a6d6bb1d |
| Preserved ff18b723 report prefix | 949008 /13304 | 9d99eeb9503db9246b02cf03f0c49f77e110866d1d0d3cfd0635b72ac6bc9732 |
| ff18b723 original candidate | 111182 /1197 | 94ca708e5d1bf8ff01b87ae105250cc562d95e219e11a50e65bbeff226982f2b |
| Canonical44d32c candidate, fence at13411 | 111275 /1200 | dfd04b3d914cf332d5ceba13d0a587dc7db2ca26b82c71e453fe85b5f2a43d9d |
| Archived forward diff, fence at13361 | 726 /16 | d0cdf11049f806797f18022ed38ea6de1b76dda3576d19c71a194adc3e2aa741 |
| Archived inverse diff, fence at13383 | 726 /16 | 8d68cfd81eec4a88c3d1a45f99f024026fe16f6e3153af883b35a5c46356ddb7 |

Both report objects use docs/roadmap/local-wave30-d01-user-browser-review.md.
The entire949008-byte ff18 report is an exact prefix of44d32c. The canonical
candidate consists of the1200 source lines ending with the already present LF
after the final closing brace. The following empty Markdown separator line
is not candidate source. An initial general fence inventory included that
separator; canonical extraction excluded exactly that observed separator,
asserted the boundary and added no LF or other byte. It produced the declared
111275-byte hash and matches the independently reconstructed source exactly.

The accepted source is the explicit requireBlank branch in44d32c, not the
different compact prospective variant in the dated e906 prefix. Its only
difference from ff18 is the following archived edit, which adds93bytes and
three lines. No new source file has been materialized here.

```diff
--- ff18-concrete-candidate
+++ unique-blank-projection-candidate
@@ -486,8 +486,11 @@
   if(!s.windows.every(w=>w!==null&&typeof w==="object"&&w.pid===pBrowserPid&&
      pPositive(w.window_id)&&typeof w.title==="string"&&w.title.length<=512)||
      new Set(s.windows.map(w=>w.window_id)).size!==s.windows.length)return null;
-  if(requireBlank&&(s.windows.length!==1||s.windows[0].title!=="about:blank"))
-    return null;
+  if(requireBlank) {
+    const blanks=s.windows.filter(w=>w.title==="about:blank");
+    if(blanks.length!==1)return null;
+    return blanks.map(w=>({pid:w.pid,window_id:w.window_id}));
+  }
   return s.windows.map(w=>({pid:w.pid,window_id:w.window_id}));
 }
 function pPrepareProjection(value) {
```

Independent full-source and AST proof:

- The exact old two-line branch occurs once; the exact new five-line branch
  occurs once. Forward replacement reproduces the complete canonical corrected
  candidate, and inverse replacement reproduces the entire ff18 candidate.
  Every byte before and after the branch is identical. Independently applying
  both archived unified diffs, including all context lines, gives the same results.
- Acorn parsed both complete programs, exit0. Both retain34 top-level function
  declarations in the same order. All33 declarations other than pWindows have
  byte-identical source ranges. All top-level string literals also match.
- pWindows still has five top-level statements. Statements0,1,2 and4 are
  AST-identical: result extraction, initial shape/cap refusal, complete owner/
  ID/title/uniqueness refusal, and the original all-window cleanup return.
  Only statement3 changes, to an if(requireBlank) with no else and exactly
  three body statements: fixed-title filter, exact-one refusal and projection.
- Restoring only statement3 to the original AST gives complete whole-program
  normalized AST equality. Normalization excludes source positions and
  preserves BigInt literal values as explicit decimal data. Original and
  reversed AST SHA256:
  ba23e634f3be23cc12e66cb1c99abe7189b387ab04487a602998256bf03b2676.
  Corrected AST SHA256:
  5b26f32ee26088e36df0d8182e251db5b17ab2e2a77ebc60e10e2f4f7156d081.
  These independently computed values agree with the author's static receipt.
- The entire41004-byte242-line launcher and31581-byte467-line c5 suffix are
  byte exact. All four interfaces, the private carrier and seed producer,
  controller/source pins, private transfer handling, collectors, latches,
  deadlines and cleanup bodies remain unchanged. Fourteen unchanged embedded
  Python literals and the separate223-line controller body parsed without
  executing or importing their code.

Corrected-candidate witness spans and logical consequences, not executed cases:

| Witness / input condition | Source consequence |
| --- | --- |
| pWindows482–488: tool error, missing/non-array windows, more than32 rows, invalid row/PID/ID/title or any duplicate window ID | Null before selection, for either caller mode. An invalid auxiliary row cannot be bypassed by a valid blank match. |
| pWindows489–493: requireBlank=true and zero exact about:blank titles | Null; no substitute title, rank, index fallback or geometry. |
| Same branch: two or more exact blank titles with otherwise valid unique IDs | Null; ambiguity is still refused. |
| Same branch: one exact blank title plus validated auxiliary windows | Only that blank's existing pid/window_id pair is projected. No title or arbitrary extra row field is copied. |
| pWindows494: requireBlank=false | Original all-window projection, preserving validated owned auxiliary windows. A valid empty array still returns an empty array. |
| pPrepareEntry670–675 | Null still latches prepared_window_ambiguous. The sole returned match supplies the exact existing PID/window bind; windows[0] now refers to a proven unique match, not ranked selection. |
| pCleanup550–556 and566 onward | Cleanup still calls pWindows(false), detects any remaining owned windows and retains its exact-PID escalation and final-empty-window checks. Filtering does not hide auxiliary windows from cleanup or turn an unknown owner/result into absence. |

Reread the full pinned public root blank contract at
8a2c4a192d04a43cd930141506a5215cca343542:
docs/roadmap/evidence/wave30-d01-root-blank-driver-contract.json,
3893bytes/SHA256
b8145f029796a2063dfe167f026fe299d85afcaedbec5a7abacbeb5bf1a5ea27.
It records the Driver-spawned PID, selected about:blank window, exact native
binding and inert blank snapshot. After unverifiable synthetic/background
cooperative close, it records eleven remaining PID windows, root's exact
owned-PID kill, named-session end, empty final window list and PID absence.
That body supports retaining all validated owned windows in the cleanup branch.

The44d32c prose additionally attributes an initial ten-window count to root.
The pinned contract itself does not contain the initial raw window array;
I have not independently observed or queried that array. Source acceptance
does not depend on treating the author's attributed count as fresh evidence:
the corrected predicate handles a unique validated blank among auxiliary
windows, while the old predicate necessarily rejects that arrangement.
Historical receipt IDs are not future ownership handles. The closed survey,
root's reported parser/inverse exit0 and this independent parser result are
separate inputs; none is an adapter or candidate execution.

This phase leaves the full900s inclusive/840s active/180s preparation/60s
cleanup allocation and sampled8.5GiB stop/intended8GiB floor exactly as in
ff18. Pre-gate PID/lab/process-group ownership and release-UNKNOWN boundaries,
private output/reference clearing and actual child-join requirements retain
their original limits. It does not repair or test those separate seams.
The Sol6 adaptation and my separate public-decision projection are still not
composed into this corrected candidate. Old memory results and real browser
failures remain dated; no changed memory case, private bridge or browser journey
ran. Historical cause, sender and true first cleanup start remain UNKNOWN.
Primary and closed-row decisions remain root-owned and unchanged.

Actual checks: the independent full-source/Acorn/AST/diff/literal proof exited0
on its first validation invocation in this phase; canonical extraction added
no LF. The recorded root parser exit0 was supplied evidence and was not
reinterpreted as candidate execution. Current WT documentation, whitespace,
prefix and one-file scope results follow after this append. The author44d32c
report still records its protected-prefix check-docs exit1 for the previously
described SimpleNamespace(demo=d target; this review does not rewrite that
prefix, change the checker or call it a checker defect.

No candidate VM/eval, case, reviewed function, launcher/controller/private
transfer, Driver/browser, native/product/Cargo, HTTP/network, external query,
secret-file read or runtime occurred. No slot was acquired or released.
This source acceptance creates no runtime authorization. Root alone reviews
and integrates the report, reserves any source follow-up and releases any
subsequent focused validation or fixture.

Append-only WT documentation/whitespace/prefix/scope results:
python3 scripts/check-docs.py exited0 with Markdown links and build-directory
layout checked; git diff --check exited0. An independent byte check confirmed
the entire20118-byte e906 prefix, LF-only text, final LF, no trailing whitespace,
the exact726-byte forward-diff hash above, this sole changed report and no new
files. The current WT result is separate from the author's immutable exit1.


## wave30_D01_complete_composition_independent_review — 2026-10-03

### Disposition and scope

Recommend **source acceptance only** of the exact corrected composition with
SHA-256 `a270153f635393085ccd553a2ba18c4ee66f844d17444013c19ed95c3201809e`.
I found no concrete additional composition blocker after reading the entire
module, all embedded bodies, and both literal changes. This recommendation
requires neither a product edit nor a new source reservation. It does not
release runtime or establish D01/D05 completion. The changed-projection memory
review and any real journey remain separately owned and authorized by root.

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing Sol2 worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`; review parent
`f25f151d086d99f5d0f1619193c60b1a3540e5b9`.
`runtime_run_by_this_review: false`. No runtime lane was taken or released.

The report path was absent in this branch and filesystem. As explicitly
reserved, I created only this path from the complete published
`ffda21202dc60021fc955078bbc9e4585444a7d0` object, then appended this section.
Its original 31,346 bytes / 399 LF characters have SHA-256
`c602e6a1b4eebd2994e109557c6fdd3b815e699e0557b46d00e3da8c651385c2`.
They equal the whole report at
`99d740c126cfe1a28b28d629c026e87ddb3aba46` and remain unmodified here.
Those dated authorship, findings, corrections, executions and limitations retain
their original meaning. No other file or source history was imported.

### Immutable bodies and exact construction

The composition and correction are in the immutable
`docs/roadmap/local-wave30-d01-user-browser-review.md` objects below, rather
than the moving author worktree. That source report is absent in this branch;
its Git objects were read without importing it:

| Object or body | Exact identity and review |
| --- | --- |
| Full fence at `251a29c097fc8b9a1fc1dd083bab97060242bdf0` | Canonical uncorrected module: 113,395 bytes / 1,241 LF characters; SHA-256 `484bd441ef0f3db547d75f6138f2943b07a3ef090c30271a9cdc3d7914dda02e`. |
| Literal correction at `b21135a5c804de6ef9308b305d47b5021af71092` | Only module lines 37 and 428 change: the entry suffix pin in the release validator and seed validator. Entire preceding 251a29c report remains a byte prefix of this object. |
| Corrected entire module | 113,395 bytes / 1,241 LF characters; SHA-256 `a270153f635393085ccd553a2ba18c4ee66f844d17444013c19ed95c3201809e`. |
| Uncorrected preparation prefix | 79,694 bytes / 733 LF characters; SHA-256 `98c765d6f29992ad9a56148af9c2562e95c4be9cb273a84a59f5a3e02cd7bedf`. Exact prefix of the accepted 44d32 module. |
| Corrected preparation prefix | Same byte and LF lengths; SHA-256 `52bdc1bec3f527996032045a6a466e39e326a40d6cae393e1264d0ca9f2bd11d`. Reversing the two pins restores every prefix byte. |
| Complete projection suffix | 33,701 bytes / 508 LF characters; SHA-256 `505b1c0fec96e248dc67db46379bd801fa06a2678462aa094fb2fc1bc5ea398f`. Exact suffix from `48288c1f10f24d47bb86b57edd0682014e9b64ee`, independently reviewed in `e04cb470dab863ebd1b59d57db13771bad428e8b`. |
| Accepted preparation baseline | `44d32c824218512d4bac6aca073dde6d260bdf37`: whole 111,275-byte / 1,200-LF module, SHA-256 `dfd04b3d914cf332d5ceba13d0a587dc7db2ca26b82c71e453fe85b5f2a43d9d`. Its original suffix is 31,581 bytes with SHA-256 `d727878009b955e6c6e309ffa90fe30c3644089b4a0fb7ebbe4ebd7264fd3d9d`. |

Canonical fence extraction distinguished the single Markdown delimiter LF from
the source by exact expected hash; no whitespace normalization was used for byte
equality. I reconstructed the corrected module in memory from the entire
251a29c fence, applying exactly those two full-line replacements. The exact
502-byte / 8-LF forward and inverse diff fences were independently identified:
`4afbcb5ef4fa85f1a8079206cc4e0d9536a53b4298f5fabc5fb0e89279f0a417`
and `7c35ea8bdc9acd3a4d9f4f50d316fa814376b2cd63233c66846aea3994782db5`.

I read all corrected module lines 1–1,241. The long literal rows were read as
complete decoded source bodies, not treated as a body review merely because a
hash matched. This covers the entire launcher/preparation prefix and entire
projection suffix, including their failure and cleanup paths. I also read the
complete original published report, the composition prose and full 149-line
static checker, and the correction prose/diffs and complete 79-line checker.
The archived checkers were read, not executed. Full body reads are distinct from
the Git-object and hash comparisons above. No entire million-byte author report
or unrelated source stack is claimed as semantically audited.

### Forward, inverse and lexical checks actually performed

Independent source-only Python programs read fixed Git objects and used the
installed Node internal Acorn parser with `--expose-internals`. Parsed source
was data; no candidate expression, function, tool operation or archived checker
was evaluated. Python `ast.parse` likewise parsed embedded bodies without
compiling, importing or running them.

The complete corrected module parses as an ECMAScript module. Normalized AST
comparison removes source-position metadata while retaining operators, literal
values/raw spellings and structure. Its two changed binary comparisons are
exactly `c.entry_source_sha256 ===` at line 37 and
`s.entry_source_sha256 !==` at line 428. Restoring the old value/raw spelling at
those two nodes reproduces the entire uncorrected AST. The corrected full AST
also equals the concatenated preparation-prefix AST and projection-suffix AST.
Reversing the pins and replacing the entire suffix with the accepted original
suffix reproduces the complete accepted 44d32 AST and bytes.

The module has 35 top-level function declarations in the same order; 33 retain
identical complete AST bodies. Only `lReleaseValid` and `pSeedValid` change, at
their one pin literal each. There are 109 distinct top-level bindings with no
duplicate names. A separate scope walk covered function parameters/defaults,
block/catch/loop scopes, binding patterns, lexical declarations and references;
it found no duplicate declaration or unresolved reference outside the explicit
host/builtin allowance. The manual read also checked initialization before the
actual call sites. These are static checks, not proof of host availability,
TypeScript typing, tool response shapes or execution.

The launcher creates 25 seed keys; they exactly equal the seed validator's
25-key list. The initial owned context contains 24 keys. Its phase, helper PID,
readiness flags, public paths, positive distinct PIDs and bound handles satisfy
the suffix's initial `prepared_entry` gate by source construction. Eleven tool
method names are referenced: the two terminal operations and the nine existing
RiWork Cua Driver operations. Enabled metadata exists for all eleven. Relevant
Driver descriptors and the published blank-state contract were reviewed as
contract context; no Driver method or fresh state call was made here.

### Embedded source preservation and parsing

All 15 named Python literals were decoded, compared and parsed. They contain
nine distinct complete bodies; repeated bodies are byte-identical:

| Body | Bytes; exact SHA-256; bindings |
| --- | --- |
| Controller command | 17,326; `5eb6ab2d3d23b49fd431c41f1932082fe0c6779ad724ac77b7fded10926b26b0`; `L_CONTROLLER_TEMPLATE`. Full 225-line command read; only its shell delimiter was removed for Python AST parsing. |
| Archive preflight | 3,505; `8054cc77d8c76a8ac78e388a86fe090d6353bb4ec039d3ac078ea86f74bcfa35`; `L_ARCHIVE_SOURCE`. |
| Failed-launch readback | 2,551; `7156a5ce75e3eec855c72dd080b1e9df7f59602ede6337ee74fb737625ac4f7b`; `L_FAILED_READBACK`. |
| Private carrier | 3,459; `e621f4bd9fded5d72ce9a7cee7c6ae5027c5032682ccfaec1d9ce9a39d99dea1`; `P_PRIVATE_CARRIER`. |
| Clock | 114; `cfb3f3154cb778388724728b6f7cb804bd7e142844ded7c044afc8dd3135ab4d`; `P_CLOCK`, `CLOCK`. |
| Stop marker | 1,600; `5de5244440b134689bedfb1235e805c65a8b726e753cbdac1d0831ef1c83d071`; `L_STOP`, `P_STOP`, `STOP`. |
| Completion readback | 4,424; `b7d3504245ae94a584b455fa65826fbaa7759ec7a50a7b81c2cf8284c9f9051f`; `P_READBACK`, `READBACK`. |
| Prepared marker | 626; `13c6549e0328ad695769938cbc0be7eb47d7a10ea9f00c4d897655603437a949`; `MARKER`. |
| Public receipt persistence | 805; `819a7785625ce2f78a9cb03b5f242dbfa6728f9f507b242e3c048887ad01ea30`; `L_PERSIST`, `P_PERSIST`, `PERSIST`. |

These comparisons use the literal values, not candidate evaluation. The
controller command is `python3 -u -` with a heredoc; the other eight unique
literals are Python source bodies passed by the surrounding code. Whole byte
inverse already proves preservation of these bodies and all unchanged launcher,
preparation and suffix code. Their parsing does not attest installed tools,
helper artifacts, filesystem permissions or provider availability.

### Reachable handoff, authority and privacy

The corrected `entry_source_sha256` denotes the **projection suffix** 505b, not
the full composition a270. Both validators now agree with the appended literal
cell. The launcher copies this truthful release pin into the seed. It neither
rewrites an actual release object nor upgrades runtime permission by deriving a
new pin from executed source. No release/seed/input object was supplied or
mutated during this review. The two stale “c5” comments retain their accepted
bytes; the executable pins and complete appended body identify the new suffix.

Source lines 22–39 require the exact 14-key release, fixed helper/controller/
Driver-contract/entry pins, explicit runtime permission and six distinct public
basenames. The archive preflight precedes the one controller launch. Its source
checks freshness, ownership/mode/size, exact archived controller transformations
and artifact pin before proceeding. Actual tool receipts are retained before
comparisons. A gate is accepted only for that pending owned exec with exact
fields, positive distinct guard/server PIDs and the bounded direct lab path.
The 25-key seed at lines 217–231 is stored only after that observation.

Preparation rejects a mismatched seed or pre-existing context/password/
observation state (lines 407–443 and 717–724). It obtains one Driver-owned PID,
then validates the entire bounded window list before choosing the unique blank
window. An ambiguity is refused, rather than ranked by geometry or position.
It binds the exact session/PID/window and validates the corresponding inert
blank snapshot, tab/target and namespace before transferring a credential.
The four required interfaces are fixed function bindings, not caller-supplied
arbitrary hooks. Their callers and argument/result shapes agree in the source.
Provider compatibility beyond the accepted observed blank contract remains
unmeasured and fail-closed.

The private reader is relative to a verified owned directory FD. It checks
no-follow directory/file identities, regular file, owner, exact modes, one link,
bounded size and unchanged name/FD metadata around the bounded read. The carrier
runs once, retains numeric exit/session/size before parsing, validates an exact
three-key packet and fixed ASCII alphabet, and calls only the synchronous sink.
Its pending child remains an owned cleanup obligation; late output is discarded.
The wrapper has a bounded alarm and fixed failure packet. Raw result/value
references are cleared; the reader clears its bytearray. This is reference and
buffer clearing in source, not a guarantee that immutable strings disappear
from process memory or that a tool transport cannot retain its own data.

The sink's one-use flag stores only the validated password. It creates the
24-key prepared context and immediately enters the literal suffix in the same
cell, without text/yield/model wait or an RP HTTP probe. The suffix revalidates
owned handles, phase and distinct PIDs. It writes the prepared marker before
readiness, and sets `browser_decision` only after successful entry observations.
The complete initial module is not a replayable continuation wrapper: its seed/
freshness gates refuse existing state. Later decisions use the exact suffix
against that already owned context, as specified by the composition design.

Password survives non-password decisions until actual password dispatch; it is
then cleared, and suffix cleanup clears it before any await or idempotency
return. Decision actions require a currently owned snapshot ref, clear that ref
set before dispatch, and use the same bound session/target/tab. The public
projection retains typed role/name/action/ref association from one snapshot,
preserves duplicate ref multiplicity, and emits only the fixed twelve public
role/name pairs. Arbitrary account text, OTP/password, raw URLs, query/header
values, sender identity, private errors and snapshot prose are not projected.

The projection is not an independent authorization engine: root must resolve
ambiguity and pin the printed role/name/ref action and expected result before
a continuation. The dispatcher validates freshness; it does not itself infer
role/action authority from a bare ref. A confirmed or unverifiable dispatch
never earns a page outcome or journey credit. The post-action snapshot and
unchanged page predicates remain required; `protected_after` is a read-only
snapshot of the callback-following page, not another RP request. These boundaries
match the independently accepted projection review.

### Failure, cleanup and timing boundaries

The original first-failure latches remain first-only, with numeric observations
retained before comparison. Controller diagnostics pass the existing closed
site/class/function/line schema; malformed observations produce fixed labels.
No exception text, repr, arbitrary type name, private path or protocol content
is added to public output. Source preservation covers the helper/controller
pins, request predicates, readiness and native continuation semantics; this
review did not reread or execute an unrelated production stack.

The launcher distinguishes a proved owned gate from missing ownership. Before
that proof it does not guess a PID, lab or stop path; a joined command alone
cannot prove child release. Preparation cleanup uses its actual owned browser
PID/window/session, cooperative quit and checked ownership before fallback kill.
Suffix cleanup retains the same bounded stop/join/readback sequence. All three
retain unknown or unjoined ownership honestly. Release requires the canonical
owned child exits, fresh PID/port/lab absence and Driver teardown checks, not
just one numeric process exit. Public receipt files use exclusive creation and
bounded fixed metadata. No existing receipt or private file was touched here.

Controller preparation waits, helper observation, per-tool budgets, 16,384-char
partial buffers, bounded join counts and the original start clock remain.
Partial newline drain stays in the same cell and owned exec; active work refuses
at 840 seconds, while cleanup joins retain the original inclusive 900-second
boundary. Nothing resets the deadline to a new continuation or cleanup clock.
Receipts are recorded before elapsed/result comparisons; final absence is
observed separately from the failure timestamp. An awaited tool can overrun a
budget before returning: these checks are cooperative observation boundaries,
not hard cancellation guarantees. Output continues to state
`whole_cleanup_within60_proven: false` and `journey_credit: false`.

### Actual static checks and preparation corrections

Actual successful checks in this slice were immutable Git reads; exact fence,
prefix/suffix and forward/inverse hashes; whole byte and Acorn AST comparisons;
lexical/reference and seed/context checks; all 15 Python literal AST parses;
enabled tool-name metadata comparison; Markdown checker; scope and whitespace
checks. The checks ran only review/parser programs. No archived parser/checker,
composed source, helper, controller, collector, case or tool stub executed.

Several review-program preparations failed and were corrected before the final
static results. Ordinary `require('acorn')` could not resolve a module; the
installed internal parser was used instead. A seed checker looked for a
nonexistent `want` variable and raised `TypeError`; it was changed to select the
one 25-element validator array. An initial command-prefix filter encountered a
partial command string (`IndexError`) and later matched zero bodies, causing a
count assertion to fail. These source literals are mostly Python bodies, and
the controller wrapper includes `-u`. Final extraction selected the fifteen
named literal bindings, removed only the controller shell delimiter and parsed
all bodies, yielding nine distinct hashes. Zero-match outputs establish no
embedded-body coverage. These are static checker corrections, not candidate
runtime failures or repairs; no reviewed source byte changed.

`python3 scripts/check-docs.py` passed on the clean parent before this report
was created. Its first post-append run flagged this append's link to the source
report, which exists in immutable objects but is absent in this branch; the
independent local-link assertion also failed. I replaced only that new link
with the exact immutable source path/pins above. The preserved published prefix
was not edited and no other file was imported. Final documentation, explicit
prefix/pin/link, staged scope and whitespace checks passed. No lint, build,
test, benchmark, provider or runtime success follows from those checks.

### Historical evidence and remaining gates

The published descriptor-memory receipt credits the previously executed
128/128 memory cases (50 observer, 78 legacy) only. It does not validate this
composition, changed projection, preparation interface or real browser journey.
The published blank Driver survey establishes its dated blank contract only;
its resources were already cleaned. This slice made zero Driver calls and
created no new browser handles. The separately released old-25 memory work is
not observed or credited here.

All prior failures retain their pins and meanings. In particular the c88
prepared-180 unexpected failure's origin, sender, lost values, true first event,
first cleanup clock and whole-60 result remain **UNKNOWN**. Source acceptance
of a correction does not retrospectively attribute that failure. Earlier
failed designs, legacy memory runs and the EOF review are preserved separately.

The smallest next gate is root's review/release of the changed-projection memory
composition, followed only by a separately authorized real fixture/journey when
its prerequisites are established. There is no recommendation for an unrelated
suite, provider campaign or mandatory new host gate. This append acquires no
source ownership and changes no original task status, closed row, credential,
receipt/header/PAM, held Group or shared-admission/paused-IO contract.
