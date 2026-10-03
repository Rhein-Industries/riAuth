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
