# Wave 30 I08 managed deadline independent source review

2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Reservation: `wave30_I08_managed_deadline_independent_review`.
Supporting WT `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`, existing shell
`2173637e-bdb3-4eba-a128-178f57b41a34`, branch
`roadmap/sol-diagnostics-wave30`, clean entry HEAD
`09a3ea8f5d0718ffa201763780195447201bdfbd`.
Only this new report is owned. No alignment or source import occurred.

## Source-only disposition

**Accept the narrow managed deadline source and regression definition for
root's review/integration. No certain source, compile, security or test
blocker was found in the complete reviewed bodies.** This is independent
source review, not a C# compilation/typecheck, six-case result, tested timer
precision or Windows lifecycle pass. The candidate remains UNCOMPILED/UNRUN.
No corrective hunk or expanded test campaign is requested.

The change preserves the original five scenarios and adds a meaningful
successful-headers/partial-body stall case. Its independent observation guard
does not provide the operation token. An old body read without a deadline
cannot become a passing cancellation test through the fixture's later
disposal. The linked scope remains cooperative and **per HTTP request**;
it does not establish a hard wall-clock bound on the entire two-request
login, synchronous work or native provider cleanup.

Original I08 `94a9dc76-3b23-4a01-aeca-e401d1c1f573` and its primary/status
are not changed. Windows lifecycle/signing/SAM/DPAPI/ACL/COM/LSA/secure-desktop
evidence remains separate. Root alone may release one later managed run;
this review took/released no runtime slot and invoked no SDK or candidate.

## Immutable inputs and complete read coverage

| Role | Full immutable pin / observation |
| --- | --- |
| Published comparison base | `a6d361600a03713fc1b687f367e9db84efe43463` |
| Three-file source commit | `e31fbee66f1038cfc2412e17497bbf07f83e1314` |
| Source's actual parent | `d4a2319f8be5716550b84392c782d59efd4f8d1d` |
| Author's separate report commit | `a839b2a8bbc308e3ddedb2185f5936adab9ffdcd`, parent e31fbee |
| Author report path | `docs/roadmap/local-wave30-i08-windows-lifecycle-plan.md` at a839b2a8 |
| Author report identity | 37,937 bytes /558 lines / SHA-256 `2ffe2670605d6aaeae2e6043fa81d25b6b0e298df3e76f5f940c33cf34cf7e73` |
| Preserved original author prefix | 27,950 bytes / SHA-256 `a965fc6931bb308687b467fd655205d9aa0b89b522797f68e29cd10ab610fd58` |

Read the complete237-line DeviceHost.cs,232-line Program.cs and228-line
SelfTest.cs at e31fbee, including all protocol/caller guards, all six
scenario definitions, every old helper and all three new fixture types.
The complete108-line base SelfTest was also reread; full base DeviceHost
and Program bytes were read and compared through their exact inverse and
the complete candidate bodies. Source diff against the full a6 base was
read, rather than substituting a shorthand or the author's old parent.

Also read the complete19-line managed project,288-line WindowsStateStore.cs
and157-line WindowsLocalAccount.cs. Native caller context was read at
CredentialProvider.cpp:184–275 (`InvokeHost`),503–554 (account/serialization
approval), and the constants/field declarations; the other C++ bodies were
not fully reread. Relevant Windows README build, failure/online-only and
provider marker paragraphs were read. The complete558-line author report,
including the historical396-line audit and162-line source-only appendix,
was reread in bounded displays. No historical private capture, binary,
native library, server protocol or signed bundle was opened.

CONTRIBUTING and SECURITY were retained from the prior complete a6 read;
their e31 bytes were verified equal. No applicable AGENTS file was found
in the worktree/ancestors or owned Windows/docs directories. The explicit
source-only reservation excludes the contribution guide's runtime commands.

The source commit itself changes exactly three files,128 additions/one
deletion. All15 paths in the Windows subtree were compared by mode/blob;
only those three differ from a6. Their parent baselines equal the full a6
files. **The whole e31 branch tree is not a6 plus only this delta:** a
whole-tree name comparison identifies214 differing paths because that branch
predates later root publications. This is an integration/provenance boundary,
not an unexpected alteration in the three-file source commit. No old
production tree was imported or credited as current. Root can apply only
the reviewed source delta against the matching current files.

| Managed path | Base bytes / SHA-256 | Candidate bytes / SHA-256 |
| --- | --- | --- |
| `windows/RiAuth.DeviceHost/DeviceHost.cs` | 11791 / `b060bb09bf9fda05e1497d70053d621227dcce6adb0aef1195f0bf0e5f9121ba` | 12178 / `c87516ac0323e4d009e6d438cfdf2b74918db3d7c0ad9ab4b9b67fd2367c48ee` |
| `windows/RiAuth.DeviceHost/Program.cs` | 11073 / `59e065d324dd684615aea08f4c7f19551e178208a99fcf691f4d85e4381639b2` | 11078 / `a1fe254ebf65a2153fcf2a17728b4b1b2283ca3d6e991be92eff2bb26126a241` |
| `windows/RiAuth.DeviceHost/SelfTest.cs` | 6105 / `c78dd4496222c58a36e575ddb0bb21be8e8d8904748aeebb8280e12479f08156` | 11597 / `69bd5f5031bfdb9b974cb2e8201e6f2823a3f32e924eb53f5a38392807e1b38e` |

Candidate blobs are respectively
`3cd1bb86906ee5e01a68ce0adb05e48b1f7af19e`,
`6cf7f027aadc58697a04f846257428f8146081a6` and
`0ecc40250fe659c278109bf2833355018a6ca611`.
Unchanged context hashes: project
`e5af8331053b1167d6af93ea9ad657a0983e694fa5bb71f8f003370fbed499ea`;
state store `9999747d0183650a109368feb0d11644ff90ad30c23b0d0be6b0ca3a930d9ce0`;
local account `85641bd7baef27b419899ec76d9665c8d5cfb90b59641bafe5908524763f0d0c`;
native provider `864e7e6bd97244ebe4d2ce807975b217829db53209971c0ad6b998525e037fa2`.
Those identity checks do not attest a compiled or signed executable.

## Whole-request deadline, callers and failure outcome

| Check | Complete-source witness and conclusion |
| --- | --- |
| Finite scope before send | DeviceHost.cs:92–98 snapshots `http.Timeout`, refuses InfiniteTimeSpan/nonpositive values before constructing/sending the request, creates a disposable linked CTS, schedules CancelAfter once and replaces only the local token parameter. Production Program.cs:22–26 still uses15 seconds; the test uses1 second. No chunk/header renewal exists. |
| Stronger caller retained | CreateLinkedTokenSource receives the original caller token. Earlier/pre-cancelled caller cancellation still reaches the linked token; the finite request timer is additional. No weaker independent replacement token, CancellationToken.None override, retry or caller CTS disposal was introduced. This is source reasoning; no separate caller-cancellation case ran. |
| Same token through response | DeviceHost.cs:108,111,116,123 passes the reassigned token to SendAsync(ResponseHeadersRead), ReadAsStreamAsync, each stream ReadAsync and JsonDocument.ParseAsync. Status/64KiB/JSON guards remain exact. Starting before send closes the separate response-body wait seam; headers success does not end this scope's timer. |
| Lifetime and disposal | The linked CTS, request and response are using-scoped; the stream is await-using scoped, followed by the limited MemoryStream. Failure unwinds those scopes before SendAsync's returned task exposes the exception to its caller. The returned JsonDocument is owned/disposed by the existing higher-level caller. No timer/content ownership escapes into a successful caller document. |
| Fixed cancellation adapter | Program.cs:117–121 changes only TaskCanceledException to OperationCanceledException, covering the subtype and direct cancellation while retaining the identical fixed message and numeric exit3. It emits no exception message/body/proof. Invalid timeout configuration reaches the existing InvalidOperationException/error exit1 arm, before a request exists. |
| Approval/state gates | DeviceHostClient.LoginAsync:183–221 awaits complete login JSON before ticket validation/redeem, then complete redeemed JSON before type/device/user/epoch/expiry checks, Save and SID return. A stalled login cannot reach redemption; a stalled redeem cannot reach Save/return. Enroll and Revoke likewise Save/Purge only after complete validated response. The explicit operator-confirmed purge command is unchanged and is not a timeout fallback. |
| Native approval boundary | Program cp-login prints the fixed approved-marker/SID only after awaited online login and a second pinned-local-user check. InvokeHost requires normal process completion, exit0 and the exact bounded marker/SID; GetSerialization requires matching tile SID/local account before packing the separate Windows password. A managed cancellation exit3 is denial, never a new approval marker. None of that native path was executed here. |

The deadline is cancellation delivered through the existing awaited operations.
It does not asynchronously interrupt arbitrary synchronous code or assert a
hard process/IO bound. Login and redemption have separate request scopes;
a stronger caller can bound both, but the production caller's None token
does not create a single15-second two-request login deadline. The unchanged
native caller still has its own initial20-second wait and INFINITE cleanup/
writer joins. This managed change does not repair or certify those joins.

No automatic retry, offline ticket request, local rollback or server-side
transaction undo is added. An enrollment/revoke request might have committed
remotely before its response stalls; local retention is not remote rollback.
The actual Windows store's pre-operation Load may secure paths/ACLs. Zero
fixture Save/Purge is not proof of zero filesystem/native activity on Windows.
Existing bearer-on-stdin, revision/idempotency headers, identity binding,
secret retention, permission/receipt and shared revocation guards stay exact.

## Sixth self-test and old-implementation discrimination

The new call follows the original five blocks at SelfTest.cs:70. Its full
definition is lines111–150, observed store152–160, observed content162–170
and stalled stream172–227. These are definitions/oracles, not reached results.

The existing FakeHandler supplies success headers synchronously and does not
inject cancellation into content. The new stream first delivers only a
bounded incomplete JSON prefix, then awaits its private disposal task with
the **read's token**. The fixture calls LoginAsync with CancellationToken.None
and configures the client timeout to1 second. Its five-second WaitAsync is
applied to the refusal task, with no operation token passed or client disposal
at that point. It observes success/failure independently of the operation.

| Required oracle | Why it is meaningful in this full fixture |
| --- | --- |
| Headers succeeded and body progressed | The response is200; PrefixDelivered must be true. A pre-header transport exception, pre-body cancellation or immediately invalid JSON cannot masquerade as the intended stalled-read case. |
| Cancellation occurred at the blocked read | The stream sets CancellationObserved only in an OperationCanceledException catch whose read token is cancelled. ExpectFailure requires that exception family from the login. A guard TimeoutException or fixture-generated IOException does not match it. |
| No authorization/state result | Exactly one request, zero redeem/offline calls, zero observed Save/Purge, full record equality with the initial DeviceState and not IsCompletedSuccessfully. The complete login call is tested, not a copied timeout utility. Missing redeem or mutated state cannot be hidden by checking only a message. |
| Request-owned disposal | Both content.Disposed and stream.Disposed are checked before the test's explicit content.Dispose in finally, and before its using-scope cleanup. The response's ownership must have unwound; fixture cleanup cannot manufacture that assertion. |
| First failure preserved | Finally disposes only the synthetic content, then observes the refusal under an independent one-second cleanup guard. The suppressed exception is confined to that latter cleanup await. It cannot suppress or replace the earlier guard/check exception. The controlled stream Dispose is idempotent and completes its own task; it does not manufacture a cancellation token. |

**Counterfactual source reasoning, not a mutant execution:** reversing the
seven-line deadline leaves the original CancellationToken.None on the body
read. ResponseHeadersRead has already returned headers; the fake handler
does not cancel the stalled stream. After the partial prefix, its disposal
task remains pending. The independent five-second guard therefore fails
before any cleanup disposal. Finally may then release that old read with
the stream's fixed IOException, but this cannot satisfy
ExpectFailure<OperationCanceledException> or erase the prior guard failure.
The whole selftest cannot print its success line on that path. No precise
runtime elapsed time or numeric unhandled-failure exit is invented.

Conversely, a premature cancellation without the prefix/read observation
fails the new checks. A deadline/refusal that leaves content undisposed fails
before fixture cleanup. Synchronous guard/using setup is not offered as a
measured hard process ceiling; root's future outer supervisor is separate.
No private XML, credentials, state, device secret, ticket, assertion or raw
exception is added to diagnostic output by these new helpers.

The original five blocks remain byte-exact: denial, immediate transport
outage, mismatched assertion identity, stale epoch, and accepted redeemed
identity. Their existing MemoryStore/FakeHandler/Reply/ExpectFailure/Check
helpers are also exact. The new observed store adds Save/Purge counters
without weakening the original cases or replacing their stores.

## Independent inverse/static proofs and retained errors

All reconstruction was in memory as data. No candidate C#/PowerShell/C++/
Rust function, controller, helper, case or source import was executed.

- Removing the exact387-byte/seven-line deadline insertion reproduces all
  11,791 base DeviceHost bytes and its SHA-256.
- Reversing the sole catch type reproduces all11,073 base Program bytes.
- Removing the sole new invocation and the appended fixture block reproduces
  all6,105 base SelfTest bytes, including every old scenario/helper. The
  restored bytes equal both a6 and the source commit's parent file.
- Exactly four token-consuming sites, one CancelAfter, deadline-before-
  request/send order, independent guard/counter/full-state/no-approval checks
  and disposal-before-cleanup order were checked without evaluating them.
- Lexical delimiter checks on all three C# sources passed after ignoring
  strings/comments. This is **not** Roslyn, a C# AST/grammar/typecheck, API
  binding validation or proof of TreatWarningsAsErrors success.
- The full Windows subtree mode/blob comparison, exact three-file commit
  scope and author-report27950-byte prefix equality passed.

The project is net9.0 with implicit usings, nullable and warnings-as-errors;
Release sets self-contained single-file win-x64. Its19 lines were read as
source, without SDK inventory or an XML parser. No .NET availability,
framework binary identity, timer dispatch or host compatibility is claimed.
The intended APIs/types fit the source's declared framework; actual overload
binding, warnings and async disposal behavior still require compilation and
the separately released managed run.

The author report retains its original Expat XML-extension parse failure,
missing guessed revocation path, first bad shorthand lookup, first inverse
checker extra-blank-line assertion and report-link corrections as historical
inspection failures. Its historical Rust1/1 and11/11 assertions stay at their
recorded old bodies/pins, not current managed/native passes. They were read
as author-report attribution; old raw logs were not reopened.
This review's initial combined display truncated part of the report/store/
account text and a wide214-path listing; bounded rereads supplied the complete
report/store/account bodies and data-only comparison supplied the exact count.
No source/compile/runtime failure or repaired environment is claimed from
those display limits. No static utility failed in this review.

## Held execution and report handoff

The only prospective managed command retained from the reviewed source plan is:

```text
dotnet run --project windows/RiAuth.DeviceHost -- selftest
```

It remains **HELD**. Root must bind the reviewed source/current integration,
a usable authorized .NET9 environment and a bounded actual-result/cleanup
envelope before that one command. The selftest branch precedes native store
construction and uses the in-memory stores/fake transport; it cannot prove
SAM/DPAPI/ACL/COM/LSA, signing/install/update/uninstall, real connectivity,
Windows logon/unlock or secure-desktop behavior. There is no new universal
host/device campaign or original-I08 DONE inference here.

Actual report/static checks: `python3 scripts/check-docs.py` exit0 before
and with this new report; `python3 scripts/check-repo-hygiene.py` exit0 with
1,065 indexed files; `git diff --check` and `git diff --cached --check` exit0.
The checker sources compare byte-equal to the fixed a6 objects, SHA-256
`925418bb155b59554666603efce4af16160ee5e00df9c409d46a3381b390ae82`
and `a3b6855473e0d68308d0f7691bb2ccff766b49f21abd68d6c766e75725d31ece`.
UTF-8/newline/NUL/trailing-whitespace, immutable source/base/report pins,
whole-file inverses, protected Windows mode/blob and author-prefix proofs
passed. Staged scope is exactly this new report; no unrelated tracked or
untracked changes exist. The final report SHA/commit and clean readback are
returned separately after final checks; none is runtime evidence.

Only this new report is committed; existing source/test/helper/guide/report
bytes remain untouched. No dotnet/version/SDK/compiler/selftest/native,
HTTP/socket/service/browser/Driver/Cargo, query/download/dependency/install,
alignment/merge, private cleanup, main/push/status, worker contact or new
worker/task/WT/shell occurred. No runtime slot was taken/released. RiWork
Cua.ai Driver MCP-only preference persists for any later separately authorized
desktop work. Root alone reviews/integrates, releases runtime and changes
original task dispositions; all closed rows/contracts are preserved.


## 2026-10-03 — independent one-shot managed controller source review

Reservation `wave30_I08_managed_controller_independent_review`, project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, existing f2 worktree only. Entry commit `74278ed9e182966dbe7743ce1e574420c15af943` was clean. This appendix is the only change. The complete 18,446-byte / 244-line `1f04` report above, SHA-256 `47bdcb746c597107253f148d1857a691926e95782ed504bbf0c971017f62cdde`, remains byte-exact. Original I08 assignment/status and the earlier source-only managed deadline assessment are unchanged. No runtime slot was acquired or released.

**Disposition: hold the proposed controller.** The previously identified inherited-SIGCHLD proof gap is present. One additional certain source defect permits a terminal observation after the 120-second child deadline to reach PASS. The latter has a two-line prospective guard below. Neither finding is an observed runtime result or an explanation of an older failure. No managed code/controller/observer/library was run, and no separately reserved author correction or mutable worker text was read. A corrected whole immutable controller needs a separate review before root can release one run.

### Immutable objects and exact inputs

The entire new controller and surrounding author appendix were read from `9fdc157b7ef09d2998854cf92bea31ea26ce0bf5:docs/roadmap/local-wave30-i08-windows-lifecycle-plan.md`, not from an author working file. That commit has exact parent `a6fe92178f4a6a03874ba17f27b98e9bd764ca35`; its only change is the report append. The 51,619-byte / 772-line parent report is a complete prefix of the 98,172-byte / 1,715-line new report, SHA-256 `58d568b61f3589e0643d8d6d4f591b485b3f574e7f5b41ceac82d2434df83f42`. The new appendix starts at line 773. Its Python fence opens at line 797, contains controller lines 1–710 at report lines 798–1507, and closes at line 1508. The canonical body already ends in LF: **31,494 bytes / 710 lines**, SHA-256 **`3a3703e6fb7095f2cc3d152aa170c3b68c03acf688c9361f11517cac99078490`**. No LF was added during extraction.

The fixed production pin is `e31fbee66f1038cfc2412e17497bbf07f83e1314`. Each of the six literal `SOURCE_HASHES` entries was independently compared with that Git object's complete bytes. All six mode/blob identities are also unchanged in `a6fe` and `9fdc`; the source bodies reviewed in the original `1f04` assessment are therefore reusable. The full 228-line `SelfTest.cs`, 19-line project, and relevant `Program.Main` dispatch were reread in this review. This does not align this worktree or establish the future launch ROOT's mutable checkout.

| Fixed project input | Bytes | SHA-256 |
| --- | ---: | --- |
| `windows/RiAuth.DeviceHost/DeviceHost.cs` | 12,178 | `c87516ac0323e4d009e6d438cfdf2b74918db3d7c0ad9ab4b9b67fd2367c48ee` |
| `windows/RiAuth.DeviceHost/Program.cs` | 11,078 | `a1fe254ebf65a2153fcf2a17728b4b1b2283ca3d6e991be92eff2bb26126a241` |
| `windows/RiAuth.DeviceHost/SelfTest.cs` | 11,597 | `69bd5f5031bfdb9b974cb2e8201e6f2823a3f32e924eb53f5a38392807e1b38e` |
| `windows/RiAuth.DeviceHost/RiAuth.DeviceHost.csproj` | 795 | `e5af8331053b1167d6af93ea9ad657a0983e694fa5bb71f8f003370fbed499ea` |
| `windows/RiAuth.DeviceHost/WindowsLocalAccount.cs` | 7,223 | `85641bd7baef27b419899ec76d9665c8d5cfb90b59641bafe5908524763f0d0c` |
| `windows/RiAuth.DeviceHost/WindowsStateStore.cs` | 12,772 | `9999747d0183650a109368feb0d11644ff90ad30c23b0d0be6b0ca3a930d9ce0` |

`COMMAND` remains the exact **1,483-byte** no-final-LF literal, SHA-256 **`6217c5e88aa76979bb0f91fae2faabcea2e9ffcfff722983f8058947c827ab7d`**. Static `shlex` parsing produces 27 tokens: `env`, ten exact assignments, then 16 SDK argv tokens. The source removes `env` and the ten assignments from argv and passes them through the explicit environment dictionary. The one launch is `/usr/local/share/dotnet/dotnet run`, the fixed original ROOT project, `--configuration Debug`, the eight archived private-output/restore/build properties, and sole application arguments `-- selftest`. No alternate command or source substitution is proposed.

The 139-byte offline NuGet configuration is SHA-256 `3c32733585fa65053b7fd9e6539bd8b799814470dd25c8520cca740868968747`. Its cleared package/fallback sources and the explicit local package source are retained. `NuGetAudit=false`, shared compilation/build parallelism off, processor count 1, and build-server reuse disabled are preserved. These source settings are not a network sandbox. Preflight intentionally starts from `os.environ.copy()`, rejects eleven specified conflicting .NET/MSBuild keys, then installs ten exact assignments; it does not promise a hermetic closed environment.

The complete `a6fe` SDK-readiness appendix and `9fdc` post-controller limits were read as dated reports. Their five public SDK metadata hashes and thirteen file-size/mode checks are exactly the controller's declared manifest. No current SDK binary, version command, loader, compiler, or native module was executed or re-attested here. The design itself distinguishes metadata identity from executable/dylib content attestation.

### Complete controller body coverage

The 710-line body was read in full in bounded displays. Independent AST parsing finds **24 function definitions, one `subprocess.Popen` call at line 544, and zero `signal.signal` calls**. Parsing and literal extraction do not instantiate classes or call any proposed function.

| Complete spans, controller line numbers | What was checked |
| --- | --- |
| 1–112 | Imports, fixed ROOT/LEAF/SDK/product roles, command/config/source/SDK manifests, numeric limits, fixed exception and two ctypes structure definitions. Neither structure nor observer was instantiated. |
| 114–167: `canonical`, `digest`, `bounded_regular`, `free_bytes`, `check_directory`, `verify_sources` | ASCII canonical finite JSON, bounded regular-file reads, `O_NOFOLLOW`, UID and directory identity checks, exact six-file catalog/hash checks, and rejection of source `bin`/`obj`. |
| 169–237: `preflight` | Fixed cwd/Darwin/ARM64, WNOWAIT API presence, absent fresh leaf, owned nonsymlink ancestors, 9-GiB start, conflicting keys/ancestor config/launch settings refusal, one SDK directory and declared metadata, exact command parse and returned inputs. |
| 239–298: observer `__init__`, `members`, `info`, `sample` | ABI size guards, group-scoped bounded PID enumeration, UID/PGID/PID and start-generation validation, disappeared/zombie handling and numeric RSS aggregation. No process enumeration or libproc call was made. |
| 300–319: `tree_bytes`, `walk_error` | Original leaf device/inode/UID, regular-owned entries only, no symlink following, depth 32 / 4,096 entries, and larger logical-versus-block allocation sum. |
| 321–432: `run` setup, `latch`, `catch`, `cleanup_error`, `exclusive`, `write_all`, `ended`, `pump`, nested `sample`, `signal_group` | First fixed failure retained, closed class names and cleanup codes, exclusive 0600 output, bounded nonblocking capture, 144 samples, disk/RSS/tree refusal, non-consuming child observation, and signals limited to a verified not-yet-reaped group. |
| 434–517: `finish_group` | One nonrenewed 10-second cleanup interval; TERM, five-second grace then KILL; immediate escalation for caps; WNOWAIT observation before the actual `Popen.wait` path; unknown/reap/signal errors retained; post-reap signal-zero and libproc absence required. |
| 519–587 | Private 0700 fresh leaf/directories, fsynced fixed config, second start-capacity check, one `start_new_session=True` SDK child, PID=PGID=SID check, generation-bearing launch receipt, capture registration and finite observation loop. |
| 588–633 | Cleanup in `finally`, bounded extra pipe drains, pipe/selector closure, log fsync/close, and failures retained before any grade. |
| 635–710 | Numeric child exit, bounded readback/hash/EOF checks, source recheck and final resource check, capped result written/fsynced and directory fsynced before grading, closed public summary, and marker/exit/cleanup/durability conjunction. |

The local public SDK headers were read only at the selected relevant spans, and complete files were hashed for identity. Their definitions match the design's group selector 2, BSD-info flavor 3, task-info flavor 4, `SZOMB=5`, 16-byte command-name extent, and declared 136-/96-byte field layouts under the stated Darwin types/alignment. This is source/ABI metadata, not a live ctypes, libproc or kernel compatibility result. The installed CPython stdlib was read only at the relevant POSIX wait/signal code; its complete file was hashed, not imported as the candidate.

| Selected local source file | Complete-file SHA-256 |
| --- | --- |
| macOS SDK `sys/proc_info.h` | `e427fa96b348537b21552b9de71e01039410bcad2cedee5584c5e5fddafd70fc` |
| macOS SDK `libproc.h` | `246d87709fc6b9157ce5cf3c475656ac48e0e1ae8bbdc46cf45acd34294448cd` |
| macOS SDK `sys/wait.h` | `b77f7dd6f592eba8b0d51c15c7b975472fdad50c12ea296f74f062cbf98dcbf7` |
| macOS SDK `sys/signal.h` | `319fbc4555c1d39c95d8a5e3034956bfe8e0e1bd9142df95ab71ad3ac8bb1f54` |
| macOS SDK `sys/proc.h` | `04c812f91c7b608faf7b8710eb1da670fa9456cba9636b787cc5dfb829091dcb` |
| CPython 3.14.6 public `pyconfig.h` | `e875b545c5d65a9598f2b3f4a8c2c1a8dfb81fc7baace25acc22aafaf9e2b421` |
| CPython 3.14.6 stdlib `subprocess.py` | `6628ffdd65c093a6c08cae01ffe82877d3ced515aac9e7be0cff16512c30a7d9` |

The first wide header display was truncated; the decisive structure, flavor, function-signature and signal-flag spans were reread in smaller complete displays. None of the controller's 710-line displays was truncated. `sys/wait.h:168–174` defines WEXITED/WNOWAIT; `sys/signal.h:299–305` includes `SA_NOCLDWAIT`. Public `pyconfig.h:1218/1230` declares `HAVE_SIGACTION`/`HAVE_SIGINTERRUPT`; these do not observe live signal flags. Stdlib `subprocess.py:2005–2036`, `2039–2078`, and `2217–2247` establish the conditional status-fallback/polling behavior discussed below. No official web or remote query was needed; all witnesses are the supplied immutable source or selected installed public source.

### F1 — inherited waitability is not established before spawn

`preflight` checks availability of `waitid`, `WNOWAIT`, `WEXITED` and `P_PID` at lines 172–174. `ended()` at lines 377–380 uses `WEXITED | WNOHANG | WNOWAIT`; its comment assumes that the original leader PID remains reserved until cleanup. However, lines 519–546 contain no native SIGCHLD disposition reset before the only Popen. The AST confirms no such setter anywhere in the complete source. Importing `signal`, checking constants, and `start_new_session=True` do not establish the missing parent waitability precondition.

A permitted inherited ignored/no-zombie disposition would undermine that assumption. The installed public `subprocess.py` also shows why `Popen.wait()` is not an independent substitute for establishing waitability: `_try_wait` converts `ChildProcessError` into the child's PID with status 0 (lines 2039–2049), and `_internal_poll` can set returncode 0 after ECHILD (lines 2024–2033). This is a **conditional source counterexample**, not evidence that this controller or any historic run inherited those flags, used fallback status, lost a child, or signalled a reused PID. `ended()` can instead fail closed with a fixed observation failure; that still does not establish the promised ownership proof.

Root already reserved the minimal pre-spawn SIGCHLD setter correction with the author. This review neither duplicates that ownership nor predicts its final bytes. A subsequent complete immutable correction must show the reset before the only spawn, its pinned fresh single-reaper interpreter assumptions and refusal before spawn if it fails. No setter, signal, wait, process observation, or launch was invoked here.

### F2 — terminal acceptance can bypass the child deadline

At line 576 the loop samples `now`; lines 577–579 reject an already-expired child interval. It then samples resources at 580–582 and pumps captures at 583. Either work or ordinary scheduling can cross the deadline. `pump` clamps a negative duration to zero; it does not latch timeout. The `ended()` branch at 584–585 can then break without checking a fresh clock. Lines 692–695 grade exit 0 plus marker/durable evidence/clean ownership; they have no deadline predicate. The recorded `elapsed_through_cleanup` is evidence, not a timeout guard.

A source-only counterexample is: entry clock = start + 119.95 seconds, no failure; resource/read/wait observation returns at start + 120.05 seconds, all cap checks pass, `ended()` is true. The original branch breaks with `first_error is None`. A zero child exit, complete marker, durable result and successful later cleanup satisfy the existing PASS conjunction. No mock, controller function or syscall was executed to produce this example. It is sufficient to disprove deadline-enforced terminal acceptance even without asserting a slow syscall, exact exit time or hard kernel bound.

The smallest prospective correction is **two added lines inside the existing terminal branch**. It checks the clock after the non-consuming terminal observation and latches the existing fixed `child_timeout` through the existing first-failure function before cleanup. A previous cap/failure stays first. A nonterminal observation still returns to the existing top-of-loop guard. Command/source/SDK/config/cleanup/caps/public schema and all case code remain unchanged.

```diff
--- 9fdc157/controller.py (DATA)
+++ prospective-terminal-deadline/controller.py (DATA)
@@ -584,0 +585,2 @@
+                if time.monotonic() >= deadline:
+                    latch("child_timeout")
```

Reconstruct only from the exact 31,494-byte canonical fence above: replace the unique byte span `            if ended():\n                break\n` with `            if ended():\n                if time.monotonic() >= deadline:\n                    latch("child_timeout")\n                break\n`. This was performed **in memory as source bytes only**. The resulting **31,586-byte / 712-line** prospective DATA body has SHA-256 **`cb5bc2b1c5e9ba4a575e3cbde1004dc919e241f7e67ee4bb647f678cdc84c12d`**. Reversing that unique substitution reproduces the complete original fence, not just the hunk. Deleting only the new inner If from the parsed terminal If yields whole-AST equality excluding locations. The complete 24 original function definitions, constants/manifests and main guard otherwise match; no new function/class/import or callable payload is introduced.

This prospective body deliberately still has F1: it is **not a composed corrected controller and not ready for runtime**. It is not a replacement for the author's separately reserved setter change. Root must review a complete composed immutable follow-up. The clock guard rejects late observed acceptance; it does not create a hard scheduling/IO/kernel/output bound or redefine the separate cleanup grace as a whole-controller 130-second deadline.

### Other reviewed boundaries and conditional readiness

The launch chain is source-backed and narrow: one explicit Popen, `start_new_session=True`, direct PID/PGID/SID equality, observer UID/group/generation record before a 2-KiB launch receipt, then no consuming poll during the live observation loop. With the missing waitability assumption supplied, holding the unreaped direct leader is the intended protection against group-number reuse before nonzero signals. `signal_group` refuses unverified/reaped state. An unverified launch records failure and addresses only the direct Popen child, using Popen's guarded signal path; it does not invent a group to kill. After actual `wait`, only signal zero and group enumeration are used. An observation, signal, unknown-reap, nonempty-group or failed final-probe outcome cannot satisfy PASS. Terminal EPERM remains failure in this source; no exception relaxation or imported interpretation of older D01 EPERM is proposed.

Cleanup precedes numeric exit/marker grading and keeps earlier failures. It consumes the direct child's status with `Popen.wait`, then requires `reaped` and two group-absence observations. F1 means an arbitrary inherited launch cannot currently claim the status/leader proof; a later fresh-pinned/single-reaper correction must make that assumption explicit. The design observes one process group and generation-checks its sampled members. It does not prove cleanup of a descendant that escapes that group, offer a whole-OS process quarantine, or promise two final observations are a race-free universal discovery mechanism. No such broader campaign is required for this focused managed slice.

Private output is bounded and retained: fresh UID-owned 0700 leaf, 0600 exclusive config/log/receipt files, no inherited stdin, separate 64-KiB stdout/stderr caps, observed/retained byte counts and EOF flags, 512-KiB capped canonical result, source recheck, log/config/result/directory fsync, and grading only after successful result persistence. The durable result records `grade_performed: false` and actual pre-grade observations; the later public packet is closed and does not print child output, exception text, environment values, private paths or protocol material. Nonzero child status stays a failure, including C# cancellation exit 3; the public controller failure exit is 2. Result write/fsync failure gives UNGRADABLE and no retry, rather than substituting success.

Resource checks are source controls, **not measurements from this review**: start free bytes at least 9 GiB; stop at or below 8.5 GiB above the retained 8-GiB floor; sampled group RSS over 2 GiB refused; owned tree plus receipt allowance over 256 MiB refused; up to 256 PIDs, 144 samples, 4,096 entries and depth 32. Sampling/scan/write/fsync/library calls can take time, resource use can change between samples, and no kernel/quota/physical-pressure guarantee is asserted. Preflight, persistence and stdout serialization have no separate hard whole-process cap in this source. The 120-second child interval and nonrenewed 10-second cleanup interval are the declared cooperative limits; F2 specifically prevents the former from rejecting late observed success.

The six fixed managed source scenarios remain denial, outage, mismatched assertion identity, stale epoch, approved identity/SID, and headers-success/partial-body stall. `Program.Main` lines 13–18 dispatch the sole `selftest` argument and print the marker only after awaited `SelfTest.RunAsync` returns. That branch is before ordinary HTTP client, WindowsStateStore or account functions. `Debug` has `net9.0`; the Windows self-contained/RID settings are conditional on Release. The sixth source case supplies a one-second HttpClient deadline, a five-second observation guard that supplies no operation cancellation token, and checks disposal/no redeem/offline/Save/Purge before fixture cleanup. No case outcome is claimed here. There is no live session store, DPAPI/SAM/ACL/COM/LSA/secure desktop, signed/install artifact, physical Windows device, real issuer/provider or credential proof from this proposed mock execution.

### Actual static checks and handoff limits

- Full canonical controller extraction, exact LF/byte/line/hash identity, AST parse, 24-definition / one-Popen / zero-setter count: passed.
- Exact 1,483-byte command/hash, 27 parsed tokens, 139-byte config/hash and six fixed complete input hashes: passed. All six production mode/blob identities match `e31`, `a6fe`, and `9fdc`.
- Original source/body reuse was based on immutable object equality; the 228-line selftest, project and relevant Main branch were additionally reread. Root SDK-readiness metadata was read as dated evidence, not rerun or substituted with this host's state.
- The F2 prospective two-line DATA substitution, complete byte inverse and normalized whole-AST inverse: passed. No code object compilation, candidate import/eval, constructor, function or case was executed.
- Guidance `CONTRIBUTING.md` / `SECURITY.md` and existing checker hashes match the previously read versions. The user-authorized static/report scope takes precedence over broad build instructions.
- Repository docs/link checker, hygiene and whitespace/scope/prefix receipts are recorded in the final static receipt below after this append is checked.

No controller/private executable was materialized, SDK/native version/library call/process probe was used, and no source/config/helper/other-report/workflow was changed. No raw child output, actual store, private credentials, mutable author correction, or remote receipt was read. Runtime remains HELD; no slot was taken or released. The original managed source remains UNCOMPILED/UNRUN. Root owns the whole-correction review, future launch reservation, integration/publication and status; all other lane and closed-row contracts are preserved.


Static receipt for this appendix: `python3 scripts/check-docs.py` exited **0** (`Markdown links and build-directory layout checked`); `python3 scripts/check-repo-hygiene.py` exited **0** (`Tracked-file hygiene checked (1066 files)`); `git diff --check` exited **0**. Six complete `git ls-tree` mode/blob rows are identical across `e31` / `a6fe` / `9fdc`, all mode 100644. Before this final receipt, the review append was 20,923 bytes / 110 added lines, SHA-256 `70b58a50aa216a4073b2e7e97e49ffe7361410ea7f773f6e5646a9e6b3f1bfff`; its complete prefix was verified against immutable `1f04`. The only tracked diff is this report; index and untracked inventory were empty before committing. These are actual static results, not managed/runtime outcomes. The final receipt itself contains no new source or link. Report-only handoff remains **F1 + F2 held, runtime UNRUN**.
