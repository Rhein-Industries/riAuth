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
