# I08 final managed controller: independent source review

2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`;
reservation `wave30_I08_final_composition_independent_review`.
Existing supporting WT `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`, clean entry
`640affad29ea7849005a1da142c88245434370b8`. Sole write is this new report.
The dated independent report identifies original I08 as
`94a9dc76-3b23-4a01-aeca-e401d1c1f573`; no live task query/status change occurred.
I07 review ownership is separate; no worker was contacted or spawned.

**HOLD: one new certain cleanup-ownership blocker, F5 below.** The complete
final controller does correct all four dated findings, but a consuming wait
can reap the child and then raise KeyboardInterrupt without setting the
controller's local `reaped` flag. Later group KILL remains enabled on that path.
This is a source counterexample, not an observed signal or historical cause.
The proposed correction is confined to the destruction/consuming-wait boundary;
no expanded test campaign, managed-source or SDK-policy change is requested.
No runtime release, compilation, six-case PASS, attestation or lifecycle
completion is inferred.

Runtime remains **HELD**. The latest user coordination reports shared free space
below the controller's 9-GiB start requirement; this audit did not remeasure
capacity or acquire/release a lane. Root alone can later reserve one exact SDK
invocation after fresh capacity and source/host/input/ownership checks. No native
Windows/physical-device/signing/install/secure-desktop gate is credited here.

## Immutable inputs and complete read coverage

Author input: `38baa1d5a006f7ceebbb00ba2269e4248c46efb1`,
`docs/roadmap/local-wave30-i08-windows-lifecycle-plan.md`.
Its exact parent is `75691462bae1b3298418025a8b1f004cedbf4187`.
The full report is 241272 bytes / 4713 lines, SHA256
`cd3946a3bf0bddfc75649844173226095e3718624c59225a1c820396b7035e8b`;
its entire 188549-byte parent is an exact prefix. The new 52723-byte /
1091-line appendix was read completely: all prose, the entire final controller,
complete command, two-line terminal fragment and both diffs. The older author
prefix was checked by identity and selected archive extraction, not represented
as a fresh complete semantic review of all its dated bodies.

| Input | Bytes / SHA256 | Independent coverage |
| --- | --- | --- |
| Final controller, fence index14 | 32734 / `88b97b52d91b88ee5c689c26a5773398acf8e9f389ce79c36d22382e8340527c` | All 735 lines read in three bounded displays, complete Python AST and grader review; no import/compilation/evaluation. |
| Parent controller, fence index9 | 32459 / `81366c6e29fb324e66206e2936b2faafc67d91cd6d9585910b6d1c9867ae95ae` | Whole identity to the parent archive, complete byte reconstruction and independent structural AST inverse; final preserved bodies reviewed through the entire final source. No extra claim of a separate old-body read. |
| Dated review `ef67fa761bb18f7a795675006c970ff8c1734d4e` | 40238 / `7e7e07b0d3f8c12f4e374e1d936b9ee9fb1795233a4a7b72a242294e89c63cca` | Entire 357-line managed-deadline/controller report, including F1/F2, read independently. Earlier author's/runtime receipts stay attributed. |
| Dated continuation `5ec4c70cfeff5e9cadd2111d64ff10178a18a6f1` | 57895 / `a002d7ca3da89f72bdd17762832ca8e19cfb60baf809dacb17ee0fc05ee81273` | Entire 429 lines covered by exact ef67 prefix plus complete new 17657-byte appendix; F3/F4 and first-use limits read. |
| Final command, excluding presentation LF | 1574 / `0b1caf022448de4941695e561d765cda2409018dcdfabcb3a52c067968460c0d` | Complete command read, Constant/fence equality, shlex DATA tokenization and whole inverse to the prior command. |
| Offline config | 139 / `3c32733585fa65053b7fd9e6539bd8b799814470dd25c8520cca740868968747` | Complete source literal/fence comparison; cleared package/fallback sources remain. No restore/config experiment. |

The dated reviews are at
`docs/roadmap/local-wave30-i08-managed-deadline-independent-review.md`, not
the author's plan path. An initial metadata loop used the latter path at ef67
and got Git absent-path/128; immutable changed-path discovery corrected it.
An initial combined two-report display was truncated. Separate bounded reads
then supplied all 357 ef67 lines and all 72 continuation lines. No clipped
display was credited as a complete read or source/runtime failure.

## F1–F4 composition and exact inverse

Line numbers below are **final controller lines**, not Markdown line numbers.

| Dated finding | Exact final witness | Source conclusion |
| --- | --- | --- |
| F1: inherited SIGCHLD waitability | `require_waitable_child`97–105 requests SIG_DFL, checks the canonical getter and emits fixed refusal codes. Its sole call564 precedes sole Popen567. | Corrected under the fresh standalone main-thread CPython/single-reaper assumptions. Setter failure refuses before SDK creation. No ignored handler is restored, and constants/API availability alone are not offered as waitability proof. Actual native flags/waitability remain unobserved. |
| F2: timely loop clock followed by late terminal observation | `if ended()`607 now checks `time.monotonic() >= deadline`608 and latches `child_timeout`609 before the same break610. Deadline596 is the same nonrenewed start+120. | Corrected late-observed acceptance. Prior capture/resource failure remains first; PASS requires no first_error. This is not a hard syscall/IO or whole-controller130-second guarantee. |
| F3: first-use certificate creation | Command27 explicitly sets `DOTNET_GENERATE_ASPNET_CERTIFICATE=false`; exact twelve-key environment guard247 verifies it before env.update251 and Popen567. | Corrected identified generator branch under the retained primary SDK source. NOLOGO/obsolete first-experience key are not substituted for this control. No certificate/keychain mutation was observed. |
| F4: first-use workload repair | Command28 explicitly sets `DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK=true`; key243/value248, command hash/length/tokens, canonical inputs252–258 and actual-env handoff567 bind it. | Corrected identified checker entry without relying on CI, private sentinel state or an installed-workload record scan. No workload/package action was observed. |

The final command has 29 DATA tokens: `env`, twelve distinct exact assignments,
then sixteen SDK arguments. The assignment slice1:13, executable/run13:15 and
returned argv13: are consistent. The sole argv remains the same Debug net9
project, eight archived properties, and `-- selftest`. The original eleven
assignment values and all sixteen SDK argv tokens match the parent exactly.
Removing just the 46-byte workload-skip command line restores the full prior
1528-byte command. The shell fence equals the exact COMMAND Constant plus only
its presentation newline. Duplicate/missing/extra/wrong environment bindings
cannot satisfy the exact command hash/token count/key/value guards.

Independently applying all ten zero-context controller diff hunks forward and
backward recovers every old/new byte. The 1697-byte diff SHA256 is
`7c9e24913d36cdbfc252f514bcc313421e4b01c3be30a30219c49bd5716e8fdc`.
An independent **structural** AST inverse restored only COMMAND/COMMAND_SHA,
the four affected integer constants, the three positive slices, the key set,
the added true predicate and the inner terminal If; the whole AST equals the
parent excluding locations. It did not substitute the complete old preflight
or run subtree to manufacture equality.

There are still 25 function definitions, four classes and one Popen. Every
top-level definition except preflight/run is byte-identical to the parent.
Every nested run helper is also byte-identical: latch/catch/cleanup_error,
exclusive/write_all/ended/pump/sample/signal_group/finish_group. The run-only
delta is the two-line terminal guard. Source/SDK manifests, config, limits,
privacy/first-error/cleanup/receipt/grade bodies remain protected.

## F5 — exceptional consuming wait leaves group destruction enabled after reap

This is distinct from F1. Default SIGCHLD/single reaper makes status waitable;
it does not make a successful consuming wait incapable of raising an exception
before the caller updates its own bookkeeping. The controller explicitly
handles KeyboardInterrupt, so this is a supported failure path rather than an
invented malicious input or SDK behavior.

Newly read **public stdlib text witness** for the candidate path, without a
controller wait/signal experiment:
`/opt/homebrew/Cellar/python@3.14/3.14.6/Frameworks/Python.framework/Versions/3.14/lib/python3.14/subprocess.py`.
Whole file: 90732 bytes, SHA256
`6628ffdd65c093a6c08cae01ffe82877d3ced515aac9e7be0cff16512c30a7d9`,
equal to the selected interpreter-source identity recorded in ef67. Coverage is
the complete relevant `wait`1274–1295, `_handle_exitstatus`1996–2003,
`_try_wait`2039–2049, POSIX `_wait`2052–2090, `_internal_poll`2005–2036 and
`send_signal`2217–2247/kill2254–2257 spans; not a fresh full90732-byte semantic
review or interpreter attestation.

`Popen.wait` catches KeyboardInterrupt, performs another bounded `_wait` at1292
and then **re-raises** at1295 even when that inner wait succeeded. POSIX `_wait`
uses the real waitpid at2042/2067 and records terminal returncode at2070 via
`_handle_exitstatus`. Thus the exact child can be consumed, `child.returncode`
can hold its actual status, and `child.wait()` still does not return normally
to the controller. No fallback ECHILD/ignored disposition is needed for this
counterexample.

Finite source witness:

1. The verified SDK leader is terminal, the group has no other observed members,
   and cleanup is still within its original ten-second interval. `done`499 is
   true, final reserved-PID KILL503 precedes the first consuming wait504.
2. KeyboardInterrupt occurs in that `child.wait` path. The pinned stdlib's
   interrupt handler consumes the terminal child, sets its cached returncode,
   and re-raises. The assignments `child_exit = ...`504 and `reaped=True`505
   therefore do not complete. Locally, verified_group remains true and reaped
   remains false despite actual kernel consumption.
3. The broad exception arm507–511 records `group_observation`/the original
   failure and moves kill_at to now. `ended()` thereafter can raise ECHILD
   because the exact child has already been reaped. That error is caught too;
   it does not revoke the stale destructive authority.
4. At the end of the unchanged cleanup interval, `if not reaped`512 is still
   true. `signal_group(SIGKILL)`514 checks only verified_group/local reaped at445,
   then attempts nonzero `os.killpg(pgid,sig)`449 **after actual consumption**.
   The later wait516 can return the cached status and only then set reaped517.

If the number has not been reused, that late killpg may merely get ESRCH. The
certain defect is the destructive attempt after the leader no longer reserves
the number. If reused, the old ownership proof no longer authorizes its new
group. No actual PID reuse, signal delivery, process loss or runtime schedule
was observed here. The recorded first failure prevents PASS, so this is an
unsafe cleanup-on-refusal boundary, **not a demonstrated false selftest PASS**.

Smallest prospective owner reservation: add an irreversible destructive-group
seal before entering **any** consuming `child.wait` site504/516/624, and make
`signal_group` refuse after that seal in addition to its existing guards.
Keep all final required group KILL attempts before the first possible consuming
operation. Once consumption may have occurred, retain the original error and
use only safe status completion/absence queries; never re-enable group signals
or renew cleanup grace. Unknown status/absence must remain failed, with no
synthetic0/reap. Normal returned status can still update the existing fields.

A cached-returncode guard alone is weaker: native waitpid consumption can precede
Python's cached-status assignment, with an interrupt between them. The seal
must therefore precede the operation that **may** consume, not wait for an
after-the-fact local success flag. This is a bounded source proposal for root
and the existing owner, not code materialization, a signal experiment, extra
SDK run or permission to bypass unknown cleanup. Command, F1–F4, markers,
managed cases, manifests, thresholds and receipt/privacy semantics stay exact.

## Complete ownership, capture, persistence and grader review

All spans below were read in full, including every failure/finally branch.

| Final span | Reviewed result |
| --- | --- |
| 1–105 | Fixed original-owner ROOT/LEAF/project/SDK, source pin, full command/config/manifests, limits, fixed exception and SIGCHLD-default control. No namespace/current-worktree fallback or alternate SDK launch. |
| 107–179 | Darwin layout definitions and nofollow/bounded public input reads, UID/directory identity, exact six-file source catalog/hash checks and source-bin/obj refusal. No dynamic source import or accepting only a version label. |
| 181–259 | Fixed Darwin/arm64/cwd, wait API presence, fresh leaf, owner ancestors, capacity, eleven conflicting inherited keys, ancestor configs/launch settings, selected SDK directory and metadata guards, exact command/env/input binding. |
| 261–341 | Full libproc observer and private tree scan: bounded group enumeration, PID/PGID/effective UID, start-generation consistency, vanished/zombie handling, numeric RSS, leaf dev/inode and nofollow owned-entry/depth/count/growth refusal. |
| 343–454 | Closed first failure/class projection, exclusive private logs, bounded writes/capture/EOF, WNOWAIT non-consuming terminal observation, resource sampling and verified-unreaped group-only signaling. |
| 456–539 | Complete TERM/KILL/grace/escalation/consuming-reap/final absence flow, including unverified direct-child branch, observation errors and timeout/refusal. Normal-return order is preserved; F5 identifies the exceptional after-consumption signal defect. |
| 541–610 | Exclusive owned0700 leaf and directories, private config/logs, config/launch fsync, second capacity check, setter, one session child, PID=PGID=SID and generation/UID launch record, capture registration and both child clock guards. |
| 611–658 | First-failure-preserving cleanup finally, same cleanup clock during fallback, bounded tail drain, stream/selector closure and log fsync/close before grading. |
| 660–735 | Full numeric-exit/source/log/EOF/resource readback, capped durable pre-grade receipt, directory fsync, exact marker/ownership/durability conjunction and closed public output/exit2 failure. |

The 136-/96-byte ctypes layouts, group selector2, BSD flavor3/task flavor4 and
zombie5 are explicit selected-Darwin source assumptions. The dated independent
review supplies its selected public-header/stdlib source witnesses. I read
the complete current observer definitions but did not newly read/hash every
installed header, instantiate a ctypes structure, load libproc or test its ABI.
No live UID/generation/RSS/group/kernel result is inferred from those constants.

Default SIGCHLD precedes the only child. The controller contains no consuming
poll in its observation loop; `ended`401–402 uses WEXITED/WNOHANG/WNOWAIT.
The direct unreaped leader reserves the intended PGID until cleanup under the
single-reaper assumption. Initial group verification571 and owner info577–579
bind the launch; sampled members must match UID/group and their recorded start
generation. Disappeared members are not assigned an invented identity/RSS.

`signal_group`444–454 refuses unverified/reaped state. Verified cleanup sends
TERM472, then KILL482/503/514 only while the leader remains unreaped; final
KILL precedes consuming child.wait504. Other consuming waits516/624 are cleanup
paths. The unverified branch addresses only the direct Popen child via kill468/
485, never fabricates a group authority; its fixed cleanup error prevents PASS.
Popen's guarded direct-child signaling can internally observe that child in
this branch, rather than supplying a verified-group proof. On a normal wait
return followed by the local reaped assignment, only non-delivering
killpg(pgid,0)523 and member enumeration remain. F5 defeats that unconditional
claim when a consuming wait instead re-raises. Unknown reap/signal/absence or
nonempty group cannot pass; failure grading does not prevent F5's late signal.
No live signal trace or escaped-descendant containment is claimed.

The cleanup clock is assigned once at463 and never renewed. TERM grace lasts
to deadline−5; resource/log cap failures remove grace. Earlier first failures
are not overwritten by cleanup errors or terminal timeout. Normal EOF is
recorded by the capture pump and is not equated with process exit. Both64KiB
stream caps retain bounded actual bytes, seen/kept/truncated and EOF facts.
The final34 pump iterations add no cleanup grace; missing EOF refuses.

Private fresh leaf acquisition precedes `leaf_identity`; failed preflight/mkdir
cannot reach a result save merely because the constant LEAF exists. With no
acquired identity the post-cleanup receipt path refuses at666–667. Files use
O_EXCL/O_NOFOLLOW/0600 under the owned0700 leaf. Log/config/result/launch and
directory fsyncs have failure paths; cleanup is attempted before serialization
or final persistence failures, with no overwrite/retry.

Numeric child_exit, complete bounded private output, full input hash/catalog,
resource samples, original first failure and separate cleanup facts enter
result.json before grading. Post_sources verifies the complete source catalog
again; retained log length must match kept bytes, truncation/caps already latch,
and each EOF flag must be true. The 512KiB result explicitly records
`grade_performed:false`. Only after its write/fsync/close and leaf-directory
fsync is receipt_durable set712 and `ok` evaluated717–720.

PASS requires durable receipt, no first_error, a started child with numeric0,
successful reap, true group absence, no cleanup errors and exactly one complete
stdout line `selftest passed`. Zero/multiple markers, nonzero child, incomplete
capture, changed source, unknown cleanup or failed fsync cannot pass. Marker
grading is justified by the fixed source dispatch described below, not arbitrary
child text or six invented result records. SDK/build messages may coexist in
private logs; the public packet does not echo them or environment values.
Public output contains fixed status/field/error names, hashes, numeric status
and cleanup flags. Raw stdout/stderr remain private bounded evidence. Failed
printing returns2; no raw exception fallback is emitted.

## Fixed managed source and retained SDK bootstrap evidence

All six complete files at `e31fbee66f1038cfc2412e17497bbf07f83e1314` hash-match
SOURCE_HASHES and are whole-byte equal at final38baa. No stale branch was merged.

| Project file | Bytes / SHA256 | This independent read |
| --- | --- | --- |
| DeviceHost.cs | 12178 / `c87516ac0323e4d009e6d438cfdf2b74918db3d7c0ad9ab4b9b67fd2367c48ee` | Complete237-line body. |
| Program.cs | 11078 / `a1fe254ebf65a2153fcf2a17728b4b1b2283ca3d6e991be92eff2bb26126a241` | Complete232-line body, including all dispatch/catches/options. |
| SelfTest.cs | 11597 / `69bd5f5031bfdb9b974cb2e8201e6f2823a3f32e924eb53f5a38392807e1b38e` | Complete228-line body, all six definitions/helpers/oracles. |
| RiAuth.DeviceHost.csproj | 795 / `e5af8331053b1167d6af93ea9ad657a0983e694fa5bb71f8f003370fbed499ea` | Complete19-line project source. |
| WindowsLocalAccount.cs | 7223 / `85641bd7baef27b419899ec76d9665c8d5cfb90b59641bafe5908524763f0d0c` | Complete-byte hash/equality only; earlier review's native body coverage remains attributed to that review. |
| WindowsStateStore.cs | 12772 / `9999747d0183650a109368feb0d11644ff90ad30c23b0d0be6b0ca3a930d9ce0` | Complete-byte hash/equality only; no fresh complete native-store body review or invocation. |

Program13–18 requires exactly the sole selftest argument, awaits the complete
SelfTest and only then prints the one marker/returns0, before ordinary HTTP/
Windows store/account setup. The project remains net9 Debug; Windows RID/
self-contained/single-file settings are Release-only. All sources are
UNCOMPILED/UNRUN here; AST/lexical reading does not establish C# type binding,
warnings-as-errors or SDK compatibility.

The six mocks are denial, outage, wrong identity, stale epoch, accepted redeemed
identity/SID and successful-headers/partial-body stall. The sixth independent
5-second observation guard does not supply the operation token. It requires
prefix delivery, read-token cancellation, one request, zero redeem/offline/
Save/Purge, full initial state, no successful login and request-owned content/
stream disposal before fixture cleanup. Cleanup cannot erase an earlier failed
oracle or turn an old uncancelled read into the expected cancellation result.
DeviceApi92–123 retains the one finite linked per-request timer through send,
stream reads and JSON parse; Program's ordinary request timeout stays15 seconds
and cancellation yields the fixed denial/retained-state exit3. This is not a
single15-second two-request login bound or a native Windows cleanup guarantee.

Read all three retained public root JSON bodies as DATA: current source receipt
2073 bytes/`7bccf827ed7a583b2a661ffaf8aca490a2eaa520d7aa957522229650a5c2942e`,
initial1645/`76e18c2cbcf14de7126918001bc8e729b7beb929c135dbf2f98c52544b953e06`
and source-selection3111/`65bf3341a7baeb8ae228808c96800b688fd0990072c5b6983f897e1bd58f26b9`.
They are under root planning/evidence, not SDK execution receipts. The corrected
source identity is SDK v9.0.200 commit
`90e8b202f25b7c2bf3b883d421ad5b1cb477e8b0`, tree
`c322ba339d3adb62d542ea33c1f5b3d9935e7988`. Initial mislabeled commit-as-tree
fields remain historical and are not repeated as a verified current tree.

Retained primary SDK bodies newly read **in full**, with hashes matching the
receipts or the supplied independent pinned-source record:

| SDK source | Bytes / SHA256 |
| --- | --- |
| Program.cs | 16466 / `ba0c8927d8141cba0cd0397cc85471ee9fd823a1e5dc1600bc55ba483545f607` |
| DotnetFirstTimeUseConfigurer.cs | 5124 / `5188ea10dd70d67abeecb767d554739f4c2be6eb337e23e530e66d60a747f3c7` |
| DotnetFirstRunConfiguration.cs | 1004 / `cf0209c2c0b92e6e56135d6ebac83a3a7c79a8cb450cd9e059bcd42a77f63bf4` |
| WorkloadIntegrityChecker.cs | 2043 / `6836dd126333ce03990af00ac29d0518677d9477bf24a23e404d0590601106a9` |
| EnvironmentPathFactory.cs | 3606 / `ffaa6fef0ef73a5eecb632de37583753a535a22df8edd840bb5d0252b488eb11` |

Program171/175–195 reads the two explicit booleans into the stored configuration.
Configurer77–82/102–107 excludes certificate generation with false. Program323/
345–354's captured first-use condition excludes checker invocation with true;
the checker18–43 otherwise constructs resolver/downloader/installer and can
install recorded workloads. Those branches are blocked in the selected source
recipe without claiming they ever ran here. macOS non-installer run selects the
DoNothing environment path; this is not a universal Windows/Linux PATH claim.

First-use NuGet migration remains at Configurer57–58, alongside private notice/
toolpath sentinels. Its implementation and all SDK/cache/temporary writes are
not newly audited. Offline NuGet sources and explicit flags are **not a network
sandbox or a universal first-use-write bypass**. Preflight copies the ambient
environment after rejecting eleven selected overrides and installs the twelve
fixed bindings; no hermetic environment claim is made. Five SDK metadata hashes
and thirteen file size/mode/owner checks are source controls, not installed
binary/dylib content attestation. No remote lookup, package, SDK, certificate,
workload, native or keychain action was performed.

## Resource limits, actual static checks and handoff

Exact policies remain 9GiB launch, stop at/below8.5GiB over the retained8GiB
floor, sampled2GiB group RSS, 256MiB leaf growth including receipt allowance,
one-second sampling, 256 PIDs, 144 samples, 4096 entries/depth32, two64KiB logs,
512KiB result, child120 seconds and nonrenewed cleanup10 seconds. Scheduling,
scan/write/fsync/native calls can stall or drain between samples. No hard IO/
kernel quota, continuous peak, escaped-group cleanup or whole-controller130s
bound is asserted. Post-child persistence/public printing has no separate hard
outer clock in this design; its source never promised one. This is not a new
acceptance gate or justification to reduce any existing margin.

Actual checks: immutable Git/fence/byte/hash extraction, all specified full
source/prose reads plus the selected public CPython source spans/hash, Python
AST/Constant and shlex DATA parsing, whole forward/
reverse controller diff, independent structural AST inverse, complete protected
top-level/nested body byte comparisons, command/config identities and all six
source blob comparisons. No candidate definition, class, controller, grader,
selftest, SDK or native probe executed. The initial wrong Git path and truncated
combined display were corrected only by source reads, as recorded above; no
trial runtime or author-source correction occurred.
One report-only patch initially failed its context check without writing;
the corrected narrow report patches succeeded. This is not a controller failure.

CONTRIBUTING/SECURITY equal their previously fully read hashes
`7e7dd7b756f734a8105cad5b96dffa8a51977c64f3ecd70b8182fa3de6ba6737` /
`2556771d58d09a2a4754e7484645b7e948b84286ef0d21cc66169b920a31c4d8`.
The explicit sole-report/static scope supersedes broad contribution commands.
UTF-8/fence/newline/whitespace/source pins and one-file committed scope are
checked; no broad docs/runtime campaign or other file import is required.

Root's next bounded source seam is F5's destruction seal around the existing
consuming-wait sites, followed by complete immutable correction/inverse review.
The single already-archived SDK invocation stays HELD for that review and fresh
authorized capacity/host/source/leaf/SDK/environment checks in the original
owner ROOT. Neither this WT's branch nor this report supplies those live
prerequisites. No runtime slot was acquired/released.

D01 report640 remains whole-byte unchanged, SHA256
`95366b5eec0628d08debcdee2cdf3b88fa3e189ca83963609e722c2600be5bc7`.
No production/test/helper/controller/workflow/other-report/private leaf was
edited or materialized. No SDK/version/compiler/selftest/package/libproc/
signal/wait/provider/HTTP/browser/Driver/Cargo/runtime/deletion/contact/new
worker/task/WT/shell/merge/main/push/status operation occurred. Root alone owns
later review, exact sole SDK release, integration and original disposition.
All receipt/header/PAM/Group/nonrenewed60s contracts, closed rows and historical
failures remain; future desktop remains RiWork Cua.ai Driver MCP only.
