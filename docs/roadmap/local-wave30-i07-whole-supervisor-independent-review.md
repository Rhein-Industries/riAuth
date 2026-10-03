# I07 whole-device-trust supervisor independent source review

2026-10-03. Reservation `wave30_I07_whole_supervisor_independent_review`, project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, original I07 `34688b10-fa4b-4b83-8070-adfb3e55dc41`, existing f2 WT/shell only. Entry commit `5ec4c70cfeff5e9cadd2111d64ff10178a18a6f1` was clean. This ONE new report is the sole write. No candidate/module/function/VM/controller/child/OS wait/signal/native/Cargo/provider/HTTP/browser/Driver execution, private leaf creation, cache/process/capacity probe, private panic capture read, contact or status/integration action occurred. Original primary/acceptance/status and previous source/security/receipt/header/PAM/Group/nonrenewed-60-second contracts remain unchanged.

**Disposition: hold runtime and the final supervisor source.** The corrected source removes the poll-before-signal generation-ownership defect and preserves a real consuming wait status. Two certain final-grading gaps remain: it can print PASS after its declared 1,800-second interval, and its final below-9-GiB disk observation can be recorded without refusing the pass. The exact prospective source-only hunks below correct those specific acceptance gaps; they have not been applied or executed. No historic reused PID, signal, deadline overrun, low-disk execution, completed whole14 invocation or old-negative cause is inferred. Root's supplied current 8.14-GiB capacity is below the unchanged 12-GiB launch prerequisite; this is a supplied observation, not a fresh measurement by this review. Runtime remains HELD; no slot was acquired or released.

## Immutable pins, reads and preservation

Reviewed `4503633703efe617b72a2ffcbb080af76c82847e:docs/roadmap/local-wave30-i07-managed-device-plan.md`, exact parent `64e4f9636aa60fa8f28fdb527d35f6c76e62bf26`. The entire new immutable appendix, narrative, canonical manifest and encoded diff were read. Parent report 129,064 bytes is a complete prefix of the new **176,494-byte / 1,690-line** report, SHA-256 `eab3dd40d188eca385843f9af2555c2bf5027fa220968fe6f42f9486bc1941af`. The candidate fence opens at report line 1026, body is lines 1027–1380, closes 1381. Canonical body already ends LF: **20,011 bytes / 354 lines**, SHA-256 **`7f880d6ac9324d08b3a9022346d42836bdaf3258adeba34474a56daa8744d236`**. All 354 lines were read completely in two bounded displays, separately from hash/AST verification; neither display was truncated.

Decoded only the ONE standalone U+2420 blank-context line in the archived diff. Its **16,589-byte** canonical hash is `592e2864966216b59a217f1138bb516bf8a833f06c90295f3cf14be1089cba8b`. A static in-memory unified-diff reader applied all three hunks to the complete old **15,555-byte / 258-line** body, SHA-256 `e88102b020534c334d6452cd8b2a2edb45e4c54fd6a395d82ff5416cd4e275bc`, and reconstructed every candidate byte. The original five top-level helpers and entire whole14 grading/return suffix are byte-identical. This verifies an archive, not an executed patch or lifecycle. The dated64e consuming-poll design remains historical/source-only; its source defect is not attributed to the older actual failed single negative.

The controller's fixed accepted source is `5c20ca13effb28501976aef69e50c6738ab560b6`; its alignment HEAD is `715f6e9c37cf3e6a92f13f284fa54e4cb6d8d0fd`. Their complete trees independently equal **`75ffffe23b4d24f7b09256c3670a44ab693b1120`**. This is immutable object equality, not alignment or a live ROOT checkout observation. Future preflight requires that ancestry/equality, a clean tree, and only the one allowed report append beyond HEAD; other incoming changes must refuse rather than silently broaden the source pin.

| Accepted source input | Bytes | SHA-256 / identity |
| --- | ---: | --- |
| `tests/device_trust.rs` | 74,157 / 2,118 lines | `f5fc6b3d42f97bf7c4601d8228e713066af1e3a95eeee3aca60dfeeec4c7f5df`, 100644 blob `54bc47d08b5562ed6d836df55144c93b5ac8dd2d` |
| `src/device_trust_types.rs` | 3,338 | `9b8a931b68b2a453350904b32f5e6f7a98f3cecb88a0cebde41f7ed3f9803825` |
| `src/device_trust.rs` | 12,917 | `5b2960a196a678e13fc767d6e662d7a099e8a5da1d3bdb82be01dc6b1878993e` |
| `src/assembly/device_trust.rs` | 16,909 | `95c0a41470d4bf6cd68a6017e308ef090cf282e7f999eaaa4476fca8c2afea7b` |
| `Cargo.toml` | 4,020 | `58e5ef824ed96290179c9f76fea208dc37173caeee21b6ce8d37f8a6dd1abcb8` |
| `Cargo.lock` | 109,243 | `b5c9d11c001244b8017303ce8c20516e02946483845eee40720b0910758d4426` |
| `rust-toolchain.toml` | 86 | `887f9be066a15585a2c583578e84b0fcb541126d81546276bad3d2ff00d61167` |

The three provider production bodies and complete test bytes equal `19805cb9250f3b3d77da4c15b52c313794450143`, already independently body-reviewed in [the retained I07 source review](local-wave30-i07-provider-binding-independent-review.md). That report's full prior eleven-test/source reads plus three complete new bodies are reused only after object equality; they are not claimed as newly reread full bodies in this task. The retained provider kind/missing-null-unknown/default-explicit-local/both trusted writer/account-session-epoch-freshness/replay/device-ID/plain-client controls, claims timestamp boundary and same-kind-setting limits remain exactly those source conclusions. No universal device/config fingerprint or physical Google/Windows proof is added. Other source changes between the earlier source tree and fixed5c20 were not promoted into a new broad product audit.

A corrected source inventory identifies **14 `#[test]` ordinary `fn` definitions** (source, not discovery/pass): seven top-level original local tests; original Google challenge binding, failure contracts, config validation and provider-transition negative; new both-writer-direction, implicit/explicit-local equivalence and legacy/unknown re-verification. The original negative and every fixture byte remain exact. An initial `async fn` inventory returned no matches because these tests are synchronous; this discovery miss was corrected with the actual attribute/function syntax. No test was executed to resolve it. The older actual Cargo101/one failure/unwrap-error-on-Ok at line1821 and unreached later exact-error/plain assertions remain failed and separate from this expected fourteen count.

## Full controller body and whole14 grader

Static AST parse finds ten definitions: five old top-level helpers, `main`, and nested `sample_disk`, `observe_leader`, `signal_group`, `drain`. There is one explicit Cargo Popen (line220), one non-consuming waitid site (157), one consuming waitpid site (285), one guarded nonzero killpg sink (182), and one signal-zero helper site (59). No `child.poll`, `wait`, `kill`, `terminate` or `send_signal` remains. Future preflight also defines bounded Git/pgrep subprocess calls; this is one Cargo run, not a false assertion that the whole script defines only one subprocess of every kind.

| Complete candidate spans | Source conclusions |
| --- | --- |
| 1–23 | Fixed management ROOT, future private leaf, source/test pins, limits and exact expanded Cargo argv. |
| 25–65 | All five helper bodies fully read; exclusive 0600 JSON, stream/file and directory fsync, fixed Git cwd/timeouts, statvfs numeric capacity, signal-zero absence with permission refusal. |
| 67–115 | SIGCHLD setter first, capability/default checks, immediate leaf UID/0700/direct-symlink check; immutable ancestry/tree/test/manifests/report-prefix checks; owned warm target/fingerprint/features; two exact-name competitor checks and 12-GiB start checks; fsynced launch-source receipt and exclusive log. No preflight was run. |
| 116–215 | One clock origin, first-failure/cleanup/capture state; 2-second disk samples, poisoned non-consuming terminal observer, sole guarded signal sink, capped nonblocking reader and fixed dependency-download pattern scan. |
| 217–255 | Main-thread default-handler recheck, new-session Cargo launch/group proof, exclusive child receipt, active stop/terminal detection, TERM/KILL intent and finite loop bounds. |
| 256–332 | Guarded finally, bounded TERM grace/KILL while leader remains unreaped, irreversible signal-phase closure, sole matching actual consuming wait/status conversion, bounded EOF drain, absence/cleanup refusal, output flush/fsync. |
| 333–350 | Final resource/log readback, finite status persistence before grade, complete fourteen-result grader, grade persistence, final public packet and wrapper return. This complete suffix was read, not just counted or hashed. |
| 353–354 | Sole main guard; not invoked. |

The exact display command is **231 bytes** without final LF, SHA-256 `91db4786064acfadddbd0bfc2edcd9733de5cad035db362eb2118ab7ad6dd06b` (the old shell fence's added LF is a distinct 232-byte archive). It remains:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave30-o06-readiness" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test device_trust -- --test-threads=1
```

ARGV is the equivalent direct `env` invocation with the target expanded from the fixed ROOT, five explicit build assignments, locked/test-support/whole-device_trust target and serial test threads. No shell command is evaluated here. The pinned fingerprint expects default/essentials/platform/test-support, an existing owned target and rlib; it does not confer a build result. No filtered negative, retry, different feature/target, cold duplicate, download, deletion or substitute command is proposed.

The full grader at338–350 requires exactly one summary, executed passed+failed==14, zero ignored/measured/filtered, reaped absent ownership, EOF/nontruncation, no observed download/stop/internal/cleanup failure. PASS additionally requires actual converted Cargo0, summary `ok`,14 passed/0 failed, no fixed target panic locations and no unwrap-error-on-Ok marker. Consistent FAILED/Cargo101/nonzero-failed is `test_failure`; all partial/inconsistent data are `incomplete`. Public panic information is fixed path plus bounded numeric line/column, not panic/debug-response text. The source-only private raw reader was reviewed; no actual raw log/panic content was opened. Pattern scanning is an observation of those fixed log tokens, not proof of no network activity.

Wrapper exit0 is intentionally a cleanup/controller-success condition, not a fourteen-pass oracle: a joined test_failure or incomplete stop can still return0 under the original code. A future disposition must consume the retained Cargo numeric status and grade fields rather than interpreting wrapper0 as target PASS. The original result/grade files are fsynced before comparisons/printing; the prospective late-clock packet below makes their earlier-observation role explicit.

## Ownership correction: conditionally source-acceptable

The new signal path starts by installing/verifying default SIGCHLD, requires native wait/status APIs, and checks the handler before spawn. `observe_leader` requires the original direct Popen child, successful group proof, unreaped status, open signal phase, no previous observer failure and no Popen consuming cache. It uses P_PID/WEXITED/WNOHANG/WNOWAIT and accepts only no terminal result or the original PID's terminal CLD result. Any exception poisons eligibility permanently; there is no numeric PID fallback or successful retry after that poison.

Every nonzero signal goes through `signal_group`, after a fresh successful observer. Events distinguish refused/attempting/sent/absent/unconfirmed, and failure never asserts delivery. Normal terminal detection still requests TERM and later KILL before consuming status; future evidence must disclose those intents even for an already-terminal Cargo leader. Successful WNOWAIT leaves that leader waitable throughout the signal phase, providing the intended group-number reservation under a fresh main-thread, assertions-enabled, exclusive-reaper CPython launch. No native flags/live waitability or exact future interpreter behavior was tested here. The policy's `assert` guards require optimization to remain disabled in the eventual separately reviewed launch; no blanket interpreter/OS attestation is inferred.

At line280 the signal phase closes before line285's consuming wait. Only a matching `waitpid` PID/status sets `child_reaped`, stores the raw numeric wait status, converts exit and fills Popen's cache. ECHILD/unknown status cannot synthesize0 or pass. If conversion subsequently fails, cleanup/internal failure remains even though consumption occurred. Signal paths never reopen after reap. Post-reap operations are bounded drain and signal0/absence; a nonempty/reused/unknown numeric group fails instead of authorizing a late kill. The default inherited-handler, observation-failure/unverified-group and unjoinable-child paths retain incomplete cleanup; no guessed kill or resource release follows. No external reaper/foreign process/group was discovered or signalled.

The future RUN leaf is separately root-created0700 and currently supplied as absent. The script checks its immediate owner/mode/direct symlink and creates files with O_EXCL/0600 after umask077. This is not a full ancestor/inode/no-follow confinement audit; its intended trust boundary is the root-owned fresh exclusive leaf in the fixed worktree. It is not safe to borrow an unrelated/private existing leaf. This review created none and did not inspect its contents. Private log writes retain at most16MiB; JSON also has a16MiB cap; actual status/grade/public fields contain fixed metadata, hashes/numerics and fixed diagnostic labels, without exception text/protocol values. Resource checks are nominal samples and owned-file observations, not kernel quotas or guaranteed write headroom.

## F1 — final persistence can permit a late PASS

The candidate declares inclusive1800 seconds in the source archive. Its actual `started` origin is line116, after preflight/launch-source persistence; it is not a clock over interpreter startup and all preflight. Active1765, loop/cleanup1790 and final reap1798 checks occur before blocking/retention work. After log flush/fsync, final statvfs/readback, status serialization/fsync and grade serialization/fsync, lines349–350 make no fresh monotonic check. Grade predicates contain no final deadline predicate.

Source counterexample within even that narrower clock: child0/all14/EOF/absence and no failures are recorded before the deadlines. The status or grade persistence returns at started+1800.01. The original grader/public packet still says whole_target_pass and wrapper0, because the earlier clock checks and saved elapsed field are not a final refusal. No operation was executed or timed to construct this counterexample. It establishes an acceptance gap, not a hard syscall/output bound or an observed old overrun.

The smallest terminal seam is a fresh clock **after `grade_id=exclusive_json(...)`**, before stdout/return. At or after1800 it forces public completeness/PASS false and classification incomplete, uses fixed `inclusive_deadline` only if no earlier stop reason, and wrapper125. Persisted status/grade bytes remain earlier durable observations; they are explicitly marked as preceding the final clock. No subsequent filesystem receipt rewrite is attempted. Root's future outer collector must retain complete bounded terminal stdout and numeric wrapper exit before comparisons; reading only the earlier grade file cannot establish the final bounded verdict.

This corrects the stated final-acceptance example relative to the source's `started`. It does not manufacture a hard kernel/serialization/output bound, include formerly unclocked preflight, move any original clock, extend grace, or strengthen the original active-phase sampled checks into hard timing. The final clock is sampled before print; blocking/late stdout after a passing sample is still not a hard output-time guarantee. Those limits must remain explicit rather than being described as a whole-OS exact1800 bound.

## F2 — final below-threshold disk observation is not latched

`sample_disk` at142–149 latches the existing `disk_stop_threshold` when a sampled value is strictly below9GiB. After cleanup, line333 directly appends a final free-byte observation without that condition. The grade records disk_minimum_bytes, but neither completeness nor PASS checks that number. Thus all earlier samples can be above9GiB and the final sample8.75GiB, with stop_reason None and PASS still permitted. This is a direct source/data counterexample, not an observed disk-pressure run or guarantee that sampling stops before the8GiB floor.

The smallest correction is two lines immediately after the final sample: latch the SAME strict-below-9GiB stop reason if the original first reason is still None. It precedes immutable actual-status retention and grading, so the recorded failure is truthful and no prior failure is overwritten. Thresholds12/9/8 and all sampling/body/write policies remain exact. No capacity probe, larger runner, threshold reduction or cache deletion is needed for this source correction.

## Exact prospective source-only hunks and identities

These two hunks are DATA against the canonical20011-byte450 body, not source ownership or a materialized executable. They are the entire prospective scope. The old whole14 grader and consuming/signal/capture/source/command/security routines remain byte-exact; the late terminal override distinguishes saved observation from final verdict.

```diff
--- 4503633-whole-supervisor.py (DATA)
+++ prospective-final-acceptance-supervisor.py (DATA)
@@ -333,0 +334,2 @@
+    if samples[-1]['free_bytes']<9*GIB and stop_reason is None:
+        stop_reason='disk_stop_threshold'
@@ -349,2 +351,8 @@
-    print(json.dumps({'grade':grade,'grade_file':grade_id},sort_keys=True),flush=True)
-    return 0 if child_reaped and group_gone and internal_failure is None and not cleanup_errors else 125
+    final_elapsed=time.monotonic()-started
+    inclusive_deadline_met=final_elapsed<1800
+    if not inclusive_deadline_met:
+        if stop_reason is None:
+            stop_reason='inclusive_deadline'
+        grade=dict(grade,whole_target_complete=False,whole_target_pass=False,whole_target_classification='incomplete',stop_reason=stop_reason)
+    print(json.dumps({'grade':grade,'grade_file':grade_id,'grade_file_precedes_final_clock':True,'final_elapsed_seconds':round(final_elapsed,6),'inclusive_deadline_met':inclusive_deadline_met},sort_keys=True),flush=True)
+    return 0 if inclusive_deadline_met and child_reaped and group_gone and internal_failure is None and not cleanup_errors else 125
```

| In-memory prospective variant | Bytes / lines | SHA-256 |
| --- | ---: | --- |
| F1 terminal-clock only | 20,516 / 360 | `c6f0efee4dd937760f1213b5fc15321a7144e76caafc93792463d5ea4c82f245` |
| F2 final-disk latch only | 20,117 / 356 | `aa11dbc720dc408884ea498d920d8e1aa2ff135ae40a6287b40329b31f18b9b4` |
| Both exact hunks | 20,622 / 362 | `e636b01905550ed43ffae706036b34e9e35e7a91d7d7db2d86d757a19be26af8` |

The zero-context diff is1,140 bytes, SHA-256 `fd8296a5a36a5c3622aa032281fc6a689758f13a8218684ecd46ee6d6c1bc016`. Reconstruct by applying these exact unique spans to the hash-checked fence; no final LF is added. Reversing them recovers all20011 bytes. Independent normalized whole-AST inverse deletes only the final-disk If and replaces the five terminal statements with the original Print/Return, matching every original node excluding locations. All five old helpers and the unchanged whole14 grading/persistence suffix before Print are independently byte-compared. All variants were parsed as source only; no compile/import/eval/function/case was used.

## Actual checks, errors and handoff limits

- Complete354-line candidate read; immutable candidate/report/parent/fence/command identity; exact three-hunk decoded forward reconstruction; five old helper and whole14 grader byte comparisons; ownership/status/order AST inventory: passed.
- Source5c20/alignment715 complete tree equality; accepted provider/test19805 body equality; exact test hash/blob; corrected fourteen ordinary test-definition inventory; manifest/lock/toolchain hash extraction: passed. No discovery/typecheck/test execution is implied.
- Prospective F1/F2/full variants: AST parsing, full-byte reversal, independent whole-AST inverse and protected body/command/grader preservation: passed.
- One combined prior-narrative/identity display and a later prior-review text display were truncated. All candidate354 lines and full grader were separately fully read; decisive identities, diff reconstruction and body equality were resampled in bounded complete outputs. The initial async-test regex miss was corrected as recorded above. None is a product/compiler/runtime failure or a hidden PASS.
- CONTRIBUTING/SECURITY/checker objects equal their previously completely read pins. No applicable AGENTS instructions changed. No immutable history/existing report/private evidence was rewritten; old negative and initial64 poll source failure remain distinct and preserved.

Final repository docs/link/hygiene/whitespace and scope/clean commit receipts are appended below after checking this ONE new file. The report is not implementation, independently completed original integration evidence, a current whole14 pass or a runtime release. Root alone reviews/reserves corrective author source, exact launch/leaf/collector/runtime, integration/publication and original status. I08 author composition is outside this task and requires a separate later full immutable review.


Final actual static receipt: `python3 scripts/check-docs.py` exited **0** (`Markdown links and build-directory layout checked`); staged `python3 scripts/check-repo-hygiene.py` exited **0** (`Tracked-file hygiene checked (1067 files)`); `git diff --cached --check` exited **0**. The checked initial new-report body was 21,569 bytes / 121 lines, SHA-256 `ac48e4fbf071c5e0b4c46808a88c9cb4f4f3611c98b2bf3293a72c9997159a58`. Staged readback and one-new-report scope passed; all applicable ancestor/report-directory AGENTS paths were absent. Final receipt restaging/docs/whitespace and commit readback confirm only this new file and an otherwise clean tree. These are static results; whole14 runtime remains UNRUN and source corrections/runtime/leaf creation remain separately root-reserved.
