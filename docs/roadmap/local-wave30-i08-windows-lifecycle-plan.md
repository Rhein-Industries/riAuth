# I08 Windows lifecycle: original-scope source audit and next seam

2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original task
`94a9dc76-3b23-4a01-aeca-e401d1c1f573`; reservation
`wave30_I08_windows_lifecycle_original_scope_source_audit`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. Clean starting/parent commit:
`d2a64aa9205f9faffb9c80d11771bff07c23dd56`.

**One concrete local gap remains: the managed device client's response-body
read has no request deadline when its production caller supplies
`CancellationToken.None`.** Recommend reserving only that deadline seam and
one focused synthetic regression before implementation. Installation,
enrollment, signed update, uninstall and recovery have substantial accepted
source implementations. Their Windows-native execution and signed delivery
remain separate missing evidence. This report does not complete I08, change
its primary/status, or reopen any closed row.

All product/source citations below are immutable published
`544d1340b80cd3e040dc13142cdcbc1d75fea4cb`. No alignment, source import,
implementation, product/helper/test execution, Cargo reservation or desktop
interaction occurred. The only written file is this new report.

## Original row and authority

Read the complete matching row from the explicit project's
`planning/current-tasks.json` export, not an inferred replacement task. Export:
244,354 bytes, SHA-256
`0b38ca3eb8810a4628436eb5083f6329de848f3f4ddaa972c850601ad45c0835`.
Its exact outcome is:

> Include installation, enrollment, signed updates, uninstallation,
> connectivity-loss behavior, revocation, and recovery.

The workstream goal is complete tested integrations; its gate requires a
working setup, lifecycle and failure-handling path for each advertised
integration. It explicitly requests relevant implementation, tests,
documentation and released-artifact evidence as applicable, with actual
verification and external prerequisites; a report alone cannot complete it.
Proposed dependencies are I07, Q01 and Q02. The dated export retains `todo`
and original worktree `1567d248-712c-4a97-8184-3058d8020c4f`. This bounded audit
in WT ed9 is authorized by the current user reservation, superseding the row's
old scheduling hold for this report only. No board/assignment reconciliation
was performed.

Read complete [CONTRIBUTING](../../CONTRIBUTING.md) and
[SECURITY](../../SECURITY.md). Their pinned blobs are
`64708527ea0d85b741c5d8ffe88d4955a734bb1f` and
`047208fb72e97fc942fe5d4d988b162c28f80d5c`, respectively. No applicable
`AGENTS.md` was found in the repository/ancestor discovery. The user's static
reservation excludes the broad contribution check commands and all runtime.
Windows/environment inputs have already been requested by the user; this
audit makes no independent request and contacts no other worker.

## Bodies actually read and immutable identities

The following are **complete body/fixture reads**, including all installer
functions and all assertions in both Rust files. Byte/hash checks identify
the same objects but are not substitutes for those reads. SHA-256 values:

| Pinned file | Bytes / lines | SHA-256 |
| --- | ---: | --- |
| `windows/Install-DeviceHost.ps1` | 82,469 / 1,524 | `b600c2b45a3530de0ea0f34758ea2f4d25dfc0b99182e483aadbbcad75a252ce` |
| `windows/New-DeviceHostBundle.ps1` | 9,373 / 223 | `0794280737cd7e5ea8b9317dfaf3b8ab393892c362554024a0ce4fd595ba04e7` |
| `windows/RiAuth.CredentialProvider/CredentialProvider.cpp` | 29,863 / 729 | `864e7e6bd97244ebe4d2ce807975b217829db53209971c0ad6b998525e037fa2` |
| `windows/RiAuth.DeviceHost/DeviceHost.cs` | 11,791 / 230 | `b060bb09bf9fda05e1497d70053d621227dcce6adb0aef1195f0bf0e5f9121ba` |
| `windows/RiAuth.DeviceHost/Program.cs` | 11,073 / 232 | `59e065d324dd684615aea08f4c7f19551e178208a99fcf691f4d85e4381639b2` |
| `windows/RiAuth.DeviceHost/WindowsStateStore.cs` | 12,772 / 288 | `9999747d0183650a109368feb0d11644ff90ad30c23b0d0be6b0ca3a930d9ce0` |
| `windows/RiAuth.DeviceHost/WindowsLocalAccount.cs` | 7,223 / 157 | `85641bd7baef27b419899ec76d9665c8d5cfb90b59641bafe5908524763f0d0c` |
| `windows/RiAuth.DeviceHost/SelfTest.cs` | 6,105 / 108 | `c78dd4496222c58a36e575ddb0bb21be8e8d8904748aeebb8280e12479f08156` |
| `windows/tests/test_manifest_binding.py` | 2,559 / 56 | `faef3f08fc3df94ef67d529dfc279351083b29f199873522ec21d0224fa6ea9d` |
| `src/windows_login.rs` | 8,468 / 278 | `8ae057d1fa833abb08d1dfdc8f2ca23444bb62ded891c4f5322eafbb069f9d1f` |
| `src/assembly/windows_login.rs` | 12,290 / 342 | `cacb4fba0e395af7d4cb5bf46786110c1d5969039e3f9e0e7f29f03cf12ef72e` |
| `src/identity/windows_credentials.rs` | 3,164 / 95 | `999518427de03cf6223c02911473d466e799baaf384521f944c27733a447fc88` |
| `crates/riauthctl/src/windows_device.rs` | 6,007 / 164 | `5f372b1f02113f5e269b63757ec785f0e1648b52817da2baa7d9b7ce76d72351` |
| `tests/windows_login.rs` | 36,240 / 1,254 | `11cf6497412d3ffa7a2689acbb50ea39878435fef8153819ef0b0d4d49d2fe8d` |
| `tests/m03_windows_device_e2e.rs` | 11,560 / 344 | `e6609616ea4a4155171bd2f1059bc48000b65c1d72ab825d5cfb41dc8d027336` |

Also read complete Windows README (136 lines), recovery guide (86), provider
README (36), CMake definition (16), DLL exports (4), managed project (19),
[ENT-13](../enterprise/ENT-13.md) (147), and shared identity module (162).
The managed Release configuration is `net9.0`, self-contained single-file
`win-x64`; the CMake definition refuses non-Windows or non-64-bit builds and
uses native Windows credential/LSA libraries. These are source definitions,
not successful builds or loaded DLL evidence.

Read the complete relevant shared bodies, rather than claiming to read every
unrelated method in large files:

- `src/management.rs`: Windows enrollment request preparation, ticket
  deletion, enrollment/rotation, issuance-marker envelope and device revoke;
  `issue_credential_once` at 3011–3062.
- `src/api.rs`: Windows routes at 754–770 and handlers at 3863–3919;
  `App::blocking` context propagation and complete `protect` middleware,
  including fingerprint/header parsing, rate admission and response guards.
- `src/cli.rs`: Windows command definition and complete dispatch arm at
  1520–1554. Standalone riauthctl's full Windows module was read separately.
- `src/core.rs`: complete `recover_admin` at 393–451. Recovery refuses other
  connected PostgreSQL clients, retains explicit factor-reset/elevation
  conditions, changes the password/epoch, queues logout and audits in the
  writer. It cannot restore a previously revoked Windows device or perform
  Windows logon.

Commit/stat and selected historical-report checks below are explicitly
metadata/evidence reads. They are not additional native executions.

## Mapping the complete original lifecycle

| Original outcome | Actual accepted path at the fixed source | Verification boundary |
| --- | --- | --- |
| Installation | Signed four-file bundle; independently verify installer before execution, then `Verify`/elevated `Install`; signed host and native x64 DLL, exact hashes, fixed CLSID/COM registration and signer pin. Staging is validated before publication; failure restores owned installation/registration. | Source path exists. No verified signed Windows installation/LogonUI receipt in the selected accepted evidence. |
| Enrollment | Administrator/SYSTEM managed host resolves an enabled local SAM account and pins its SID. HTTPS issuer, exact device/user, explicit `--replace`, bearer on stdin, revision/key. Server hashes the generated secret and commits mapping, old-ticket invalidation, audit and redacted issuance receipt together. Machine DPAPI and SYSTEM/Administrators ACL protect local state. | Server/managed protocol fixtures cover portions. SAM, DPAPI, real ACL enforcement and SID replacement need Windows-native evidence. |
| Signed updates | Common pinned signer, exact signed manifest/payload bytes, canonical version and current installed hashes. Same-version/different-bytes refusal; retained highest-version floor; downgrade requires explicit reason and successful warning event first. Write-ahead pending journal binds exact retry and verified old/new/stage/backup paths. | Complete source/journal review and one lexical manifest fixture. No real signer, registry, locked-DLL, interrupted-update or rollback execution claimed. |
| Uninstallation | Refuses pending update or any local-state entry; verifies installed signatures/hashes/registration, moves owned binaries to quarantine, removes only its registration, restores on failure. Retained locked DLL is reported for restart cleanup. | Source behavior, not a native removal receipt. Empty local state is an operator precondition, not proof that remote revocation occurred. |
| Connectivity loss | Host is online-only and never requests/stores an offline ticket. Denial/outage retains state; a successful login needs a fresh consumed ticket and identity/epoch/expiry checks. Provider requires approval before serializing the separate local Windows password to LSA. | Fail-closed decision source and mock assertions exist. Response-body wait gap below remains. No connectivity-loss Windows tile execution credited. |
| Revocation | Device revoke invalidates pending tickets in the writer. Every shared user-disable transition invokes device/ticket revocation; re-enable does not resurrect it. Local purge follows confirmed remote revoke, or explicit operator confirmation for recovery. | Protocol fixtures assert secret/ticket refusal, atomic state preservation and scoped authorization. Windows secure-desktop observation remains missing. |
| Recovery | System Windows provider/local password remains available; separate riAuth identity/MFA recovery; server recovery is offline and bumps epoch. Re-enrollment uses current authority, a new key and current revision. Failed update retries the same signed bundle; ambiguous/foreign state refuses rather than inventing ownership. | Reusable documented/source paths. No performed Windows recovery journey or signer-rotation guarantee. |

### Security and bundle boundaries retained

`Read-SignedBundle` opens the manifest once with shared-read access and a
1 MiB bound, reads one byte array, verifies that array with PowerShell 7.4
Authenticode `-Content`, then strict UTF-8 decodes/parses those same bytes.
Its closed manifest fields bind version, provider CLSID and all three final
signed payload hashes. Duplicate/unknown properties, wrong signer/hash,
unexpected bundle member, reparse path and invalid native DLL refuse.
The builder signs payload copies before recording their hashes and publishes
only the exact four expected files after signature/archive checks. That
source binding is not an actual Authenticode trust or emitted ZIP receipt.

The installer serializes with its existing global mutex, preserves verified
backup generations and the highest version across deliberate downgrade, and
does not silently repair foreign registration or ambiguous mixed generations.
Its own self-signature check cannot make substituted script execution safe:
the external signature check in the guide must precede execution. Old signed
installer code can lack the new version guard; application control/trusted
updater and signer-rotation policy remain expressly outside this source's
guarantees. Uninstall does not disable Windows system providers or remove
their registrations.

Machine DPAPI alone is not an unprivileged-file access boundary: the state
store additionally checks protected ACL/owner and path reparse conditions.
Private temporary creation, flush and replace precede publication; protected
ciphertext and serialized state are bounded. Byte arrays are cleared where
implemented; no guarantee of erasing managed strings/all process copies is
made. The pinned local SAM SID is revalidated for the tile and after online
approval, so a deleted/recreated account is not accepted by name alone.

The provider accepts logon/unlock scenarios, has no auto/default credential,
rejects caller credential serialization, and keeps the local Windows
password separate from riAuth password/OTP. Host invocation uses a fixed
installed path, restricted inherited pipe handles and an exact bounded
approval marker/SID. No successful marker is released before ticket
redemption and identity checks. `InvokeHost` has a 20-second initial process
wait, followed on failure by termination/reaping and thread joins using
`INFINITE`; that initial wait is not proof of a whole-operation hard deadline.
This audit neither changes that native path nor claims the managed seam
establishes such a deadline.

Server enrollment/revoke HTTP routes still require both retry headers,
including full administrators. Live authority and permission binding precede
receipt replay; enrollment receipts retain only the issuance marker and exact
retry returns 409 `credential_already_issued`. A lost enrollment response may
already have committed: recovery is fresh authorized enrollment with a new
key/current revision, not secret replay. Login failures charge the shared
username lockout in the intended committed error path; ticket redeem consumes
before live expiry/device/user/epoch checks. The proposed timeout fix cannot
roll back a server operation already received or justify automatic retries.

The server's optional offline MAC format is a separate protocol facility
(maximum 72 hours), with its documented symmetric-secret and disconnected
revocation limits. The shipped component source intentionally has no such
fallback. Connectivity loss denies this tile while ordinary Windows recovery
providers remain available; it is not machine-wide riAuth enforcement.

## Fixture definitions and historical executions

Read all **11** current `tests/windows_login.rs` functions: first issuance/
login, target cap and same-user rotation, bad-secret/disable/revoke, shared
lockout, TOTP, offline expiry/epoch/device binding, remapping/secret rotation,
agent-versus-admin scope, writer/ticket invalidation, temporary-authority
credential exposure, and HTTP retry/header/receipt behavior. The scoped
writer fixtures retain full snapshot checks on denied operations. Current
HTTP assertions include missing-pair 428, unauthorized 403, exact enrollment
retry 409/no secret/no rotation, fingerprint/revision conflict, idempotent
revoke, one revision/audit effect and invalidated credentials. No assertions
were changed or run here.

The managed self-test contains **five** scenario blocks: denied login,
transport exception, mismatched assertion identity, stale epoch and accepted
identity. It uses `FakeHandler` and `MemoryStore`; no Windows state store,
SAM, DPAPI, ACL, COM, LSA, secure desktop or real server is exercised. The
transport case throws immediately, so it does not cover stalled response
content after successful headers. The Python manifest fixture has **one**
source-contract method verifying lexical same-array ordering; its own text
requires Windows for actual Authenticode. AST parsing this file is not
execution of its test method.

The **one ignored**
`riauthctl_windows_device_is_one_service_with_the_server_cli` fixture runs
actual Rust server/client binaries on loopback when explicitly enabled. Its
assertions include first issuance into a private file, UNIX mode 0600,
409/no-secret/no-second-file retry, interface listing parity, server
offline/login requests, revoke refusal and one audit per committed change.
It is not the managed Windows executable or DLL, and UNIX mode 0600 is not
Windows ACL proof. The fixed Linux CI definition explicitly selects this
target with `--ignored --test-threads=1`; a definition is not a verified named
run. No new or retrospectively inferred pass is assigned to it here.

Accepted history was checked in the fixed
[closure audit](local-wave28-task-closure-audit.md#i08) / JSON's exact I08
record and `docs/roadmap/local-wave30-remaining-task-actionability.json`,
plus selected executed-evidence paragraphs at lines 273–293 of
`docs/roadmap/local-wave30-six-task-completion-review.md`. Those latter two
objects were read at the fixed pin; they are absent from this own branch,
so no misleading relative link or file import is used. Exact mappings:

| Source history | Accepted integration | Credit |
| --- | --- | --- |
| `c846a6063aa3ba5d1e458a3b52f94707f79293ca` | `aa81e50b42477dd1b0f80837e4c0789ea494c66f` | Online-gated native provider/local account source. |
| `2493f56e04950d29ad64ea857f81620e9b2365aa` | `5537d5685eb982eee0bf9f94ac8fba975fa59adb` | Signed bundle, install and recovery source. |
| `af1eb096782fa5ca32f9e6af63f3c1ad82df7b3c` | `edfac4f3a83647379ea6003096e66928a2dc47e3` | Version floor and interrupted-update journal source. |
| `26e566c184826e936c4ec28a8450cdb59133a7c1` | `65e85b9bc382f7a6719138ad87b1c53fc576e8e0` | Signature/parser byte binding and lexical fixture source. |

The eight inspected native/managed/installer product files are byte-identical
between accepted `65e85b9` and fixed `544d134`. No import was needed.
This comparison does not attest a compiled Windows artifact.

The accepted six-task review reports actual historical
`bc3c3248dfa55217845602f7f4ab9d5c6a331b8e` execution of the then-named
`agent_enrollment_fences_temporary_access_and_preserves_device_replay`,
**1/1**, and `36e20364458ffbb6e376c7b83d116f2788592469` execution of
`windows_login`, **11/11**. This audit read that accepted report and source
identities, not those original raw runtime logs. Subsequent
`b22996523f6ba4b9bf90619f3cfe99630926edc1` /
`b8794610ad53de6a85f61b878d7b4aa9bf1d7783` changed the issuance/retry
assertion and renamed it to
`agent_enrollment_fences_temporary_access_and_never_replays_device_secret`.
The old passes remain historical and are not relabeled as executions of the
current body. There is no verified named Windows-native peer invocation in
the selected accepted records. Unrelated LDAP/FreeRADIUS/Lasso/PG/Linux
container passes do not establish Windows native lifecycle or hardware.
No old failure is removed, reclassified as a pass or given a guessed cause.

## One proposed local seam: bounded response content

Exact source witness: `DeviceHost.cs:82–118`, especially 101/104/109/116;
production `Program.cs:22–26`, 40/48/67/77 and 117–121. `DeviceApi.SendAsync`
uses `ResponseHeadersRead`. The client timeout bounds the send through
headers; response content is then read separately. Production passes
`CancellationToken.None` to both operations and content reads. A peer can
send successful headers, then stall within the 64 KiB byte cap without EOF.
No body-read deadline is created in this call path. This is a source-derived
liveness defect, not an observed Windows incident or an authorization bypass.

Managed enroll/revoke/login commands have no native parent wait to settle
that stall. The provider's separate initial 20-second host wait does not
repair managed command behavior. Since the self-test's outage throws before
content is read, its defined refusal is insufficient for this particular
case. The existing README promises timeout denial/state retention; reaching
that outcome requires bounded content consumption too.

**Prospective ownership request, not an implemented/reserved code change:**

| File | Smallest proposed hunk |
| --- | --- |
| `windows/RiAuth.DeviceHost/DeviceHost.cs` | Only `DeviceApi.SendAsync`: create a disposable linked cancellation deadline before `http.SendAsync`, retain the existing finite production 15-second request budget, and pass that same token through send, content stream creation/read and bounded JSON parsing. Never reset the deadline per chunk. Preserve all headers, paths, status checks, 64 KiB cap, JSON/error rules and caller cancellation. |
| `windows/RiAuth.DeviceHost/Program.cs` | Only the fixed cancellation-to-exit-3 adapter at 117–121 if required: recognize the base cancellation exception as well as its task subtype, with the same fixed timeout text and no exception/body output. No new retry, success or purge behavior. |
| `windows/RiAuth.DeviceHost/SelfTest.cs` | One focused stalled-after-headers synthetic stream case and only its necessary bounded fixture implementation. Retain all five original scenarios. |

A future implementation should use the finite `HttpClient.Timeout` for the
same whole-request deadline, so production stays 15 seconds and the fixture
can use a short finite timeout. Infinite/nonpositive injected settings must
not create an unbounded path; settle the exact fail-closed handling in source
review. A linked token must not replace a stronger existing caller deadline.
This is one request deadline, not a promised whole two-request login/native
cleanup deadline, nor a transaction or remote-I/O fence.

The proposed test returns 2xx headers immediately, then stalls after a
bounded partial JSON prefix on a cancellation-aware synthetic content stream.
Invoke with `CancellationToken.None`, as production does, and a short finite
client timeout. An independent finite test guard must detect the old hanging
implementation; it must not supply the operation token or provide the oracle
being tested. Require cancellation, exactly zero redeem/offline calls,
unchanged complete `DeviceState`, zero local Save/Purge, no approval/SID
result and disposed content. Synthetic secrets must not be printed. This
does not assert remote server rollback. It adds neither device proof
transitions nor a duplicate I07 fixture.

Prospective **sole managed fixture command**, only after root ownership and
runtime release:

```text
dotnet run --project windows/RiAuth.DeviceHost -- selftest
```

No such command, SDK check, harness or candidate was executed here. Source
review should compare full protected method bodies and explicit cancellation
mapping before that focused test; a fake stream pass would establish only
managed deadline/refusal behavior. No broad Rust or Windows campaign is
requested for the proposed seam. Native evidence still needs its own release.

## Already-requested Windows input and bounded native follow-up

The smallest native prerequisite is **one authorized Windows x64 VM/host
with recoverable secure-desktop access and an approved trusted signing
identity**. Root can use the user's already requested inputs; no new user
question is issued. Necessary concrete binding is:

- Exact Windows host/build and disposable restore point; an operator who
  can recover through an existing system provider/local administrator even
  when riAuth is unreachable. Do not disable system providers.
- Available .NET 9 SDK, MSVC C++ Desktop/Windows SDK and elevated x64
  PowerShell 7.4+ for building and signed manifest verification. Availability
  and capacity were not probed here.
- Code-signing certificate/private key available on the authorized signing
  machine, trusted signer pin delivered separately, and exact first/target
  signed four-file bundles with numeric versions. Keys/credentials must not
  enter a report or upload. No release, certificate or bundle is invented.
- Reachable HTTPS Platform issuer with trusted TLS/clock, one authorized
  enrollment actor and disposable riAuth/local-SAM identity. Root holds
  credentials privately and binds exact tested binaries/source/configuration.

After the local seam is reviewed, root can separately reserve one finite
native lifecycle using the existing guide commands: signature-before-script
and Verify/Install; enrollment and logon/unlock; connection-loss denial with
system-provider recovery; same-signer update/version refusal and a bounded
interrupted-update recovery; device/user revoke with denied tile and
re-enrollment; remote-confirmed purge and uninstall/registration absence.
The exact interruption mechanism and cleanup receipts need a host-specific
plan before execution; this report authorizes none. Reuse the original
installer/recovery rules, preserve prior signed bundles and retain partial
failures. Capture only fixed outcomes/pins/redacted operational receipts,
never passwords, bearer/device secrets or private protocol/state contents.

That is original lifecycle evidence, not a requirement for every Windows
version, every host, physical hardware, all protocols, a new human study or
universal CI/release pass. Real SAM/DPAPI/ACL/COM/LSA/LogonUI behavior cannot
be inferred from the managed mock, Rust loopback fixture, lexical signed-byte
test or historical unrelated peer results. Desktop, if later authorized,
must use RiWork Cua.ai Driver MCP after reading descriptions/current state;
missing setup/permissions must be reported without switching providers.

## Actual static checks, failures and handoff

Performed Git-object reads, full-body reads listed above, immutable commit
resolution, whole-byte SHA-256/blob/line identities, selected JSON export/
evidence parsing, source function/assertion inventories and eight-file
accepted-baseline equality comparisons. Parsed the manifest fixture with
Python AST only: one test definition, **zero test-method executions**.
No PowerShell/.NET/C++/Rust compilation or typecheck occurred.

An optional managed-project XML static parse **failed** before reaching later
items in that inspection command: the local Python Expat extension could not
resolve `XML_SetAllocTrackerActivationThreshold`. No XML parse pass is claimed;
the 19-line project was read as source. No environment repair, tool/version
probe or alternate library loading was attempted. A separate Git/hash/AST/
JSON inspection completed the remaining intended source identities without
XML parsing. One attempted `src/identity/revocation.rs` lookup failed because
the file does not exist at the pin; discovery located and fully read the
actual shared transition in `src/identity.rs`. Truncated combined displays
were followed by smaller reads for the relevant body spans; they do not
count as full reads by themselves.

Before the report, `python3 scripts/check-docs.py` passed: “Markdown links and
build-directory layout checked”; `python3 scripts/check-repo-hygiene.py`
passed for **959** tracked files. Final report-only scope, reference/pin,
documentation, hygiene and whitespace results are recorded below after
validation. No checker cache or build directory was cleaned.

The first post-report Markdown check failed on two report-created relative
links to accepted reports absent from this own branch. Corrected only those
references to immutable pinned paths; no other report/file was imported.
The witness comparison passed for all **15** table rows (whole source bytes,
line count and SHA-256); all **16** full commit references resolved. Exact
scope check found one new report and no tracked edits. The initial XML parse
failure and historical execution limits above remain unchanged.

After correction, the Markdown check passed with “Markdown links and
build-directory layout checked”; staged hygiene passed for **960** files.
`git diff --cached --check` passed, and the staged scope contained exactly this
new report with no unstaged/untracked changes. A second whole-source witness
check verified the same 15 rows. These are static/report checks only; no
candidate, managed self-test, protocol fixture or Windows code was executed.

No source/test/helper/existing documentation was edited, no runtime slot was
taken/released, and A09 ARM `37101183416` was neither queried nor overlapped.
No new worker/task/WT/shell, alignment/merge, contact, main/push/status action
occurred. Receipt-secret, route-specific headers, PAM fallback, held Group,
shared admission/SCIM stamps and nonrenewed 60-second/paused-I/O limitations
remain unchanged. Primary I08 and all closed rows remain root-owned.

Recommend **one separately reserved managed deadline correction**, followed
by root's already-requested Windows host/signing input decision. Recommend
no original I08 DONE inference from this report. Root owns implementation
reservation, native/runtime evidence, integration/publication and status.

## Managed response deadline materialized — source only, runtime held

2026-10-03. Reservation `wave30_I08_managed_response_deadline`, same project,
WT ed9, branch and existing shell `0164baca-c6c1-4cb1-81db-6d424f0640ac`.
Root authorized exactly the three managed files below and this report append.
The approval ledger records `SOURCE_ONLY_RESERVED` and runtime `HELD`.
The original **27,950-byte / 396-line** report from
`d4a2319f8be5716550b84392c782d59efd4f8d1d`, SHA-256
`a965fc6931bb308687b467fd655205d9aa0b89b522797f68e29cd10ab610fd58`,
is retained byte for byte, including all old failures and historical limits.

**Source commit:** `e31fbee66f1038cfc2412e17497bbf07f83e1314`, parent
`d4a2319f8be5716550b84392c782d59efd4f8d1d`; exactly three files,
128 insertions / one deletion. The report is committed separately.
This closes the reserved source implementation seam subject to root review;
it supplies **no compilation, self-test or Windows runtime pass**.

### Reviewed baseline and exact source results

Resolved the user's published base shorthand to
`a6d361600a03713fc1b687f367e9db84efe43463`, the local published-main reference
at the initial source inspection. The first literal three-character `a6d`
Git lookup failed; full object discovery and the main reference resolved it
without fetching, alignment or edits. The older `a6df27e` object is unrelated
and was not used as this base.

Read the entire protected `DeviceHost.cs`, `Program.cs` and `SelfTest.cs`,
plus CONTRIBUTING, SECURITY and the project definition. Each of those three
current files matched the full reviewed a6d361 object before editing:

| File | Baseline blob | Baseline SHA-256 |
| --- | --- | --- |
| `windows/RiAuth.DeviceHost/DeviceHost.cs` | `1b9adc281aa2414a698558663e8b78a2f15a4f5b` | `b060bb09bf9fda05e1497d70053d621227dcce6adb0aef1195f0bf0e5f9121ba` |
| `windows/RiAuth.DeviceHost/Program.cs` | `6e518365a220f7ea3aceb260d12ca3f37f2d0a80` | `59e065d324dd684615aea08f4c7f19551e178208a99fcf691f4d85e4381639b2` |
| `windows/RiAuth.DeviceHost/SelfTest.cs` | `16529c0e757bf29d4986ba2d7ad33b59aa7f7969` | `c78dd4496222c58a36e575ddb0bb21be8e8d8904748aeebb8280e12479f08156` |

Committed candidate identities, read back from e31fbee:

| File | Bytes / lines | Candidate blob | Candidate SHA-256 |
| --- | ---: | --- | --- |
| `windows/RiAuth.DeviceHost/DeviceHost.cs` | 12,178 / 237 | `3cd1bb86906ee5e01a68ce0adb05e48b1f7af19e` | `c87516ac0323e4d009e6d438cfdf2b74918db3d7c0ad9ab4b9b67fd2367c48ee` |
| `windows/RiAuth.DeviceHost/Program.cs` | 11,078 / 232 | `6cf7f027aadc58697a04f846257428f8146081a6` | `a1fe254ebf65a2153fcf2a17728b4b1b2283ca3d6e991be92eff2bb26126a241` |
| `windows/RiAuth.DeviceHost/SelfTest.cs` | 11,597 / 228 | `0ecc40250fe659c278109bf2833355018a6ca611` | `69bd5f5031bfdb9b974cb2e8201e6f2823a3f32e924eb53f5a38392807e1b38e` |

### Production behavior and preservation proof

`DeviceApi.SendAsync` has one seven-line insertion. It snapshots
`HttpClient.Timeout`, rejects `Timeout.InfiniteTimeSpan` or a nonpositive
value **before creating/sending the request**, with fixed configuration text
`riAuth request timeout must be finite and positive`. It creates a disposable
`CancellationTokenSource` linked to the original caller token, calls
`CancelAfter(timeout)` once, and assigns that source's token to the existing
local `cancellation` parameter. A stronger/pre-cancelled caller remains
linked; there is no replacement by an independent weaker caller token.

The original send, content-stream creation, each stream read and JSON parse
all continue to use that **same** local token. The timer starts before send
and is never reset by headers or a chunk. Production's unchanged client
timeout remains 15 seconds. This is a cooperative whole-request cancellation
deadline; it is not a hard cancellation bound on arbitrary synchronous code,
the two-request login or the native provider's joins.

Deleting precisely that insertion reconstructs the **entire original
DeviceHost file**, proving all other helpers, headers/paths, status/64 KiB/
JSON guards, identity/epoch/expiry checks, ticket redemption, Save/Purge and
no-retry behavior remain byte-identical. All four token-consuming call sites
and the single pre-send `CancelAfter` were checked as source.

Program's sole change is `catch (TaskCanceledException)` to
`catch (OperationCanceledException)`. Reversing that substitution reconstructs
the entire original file, including the fixed timeout message and exit 3.
The fixed bad-configuration `InvalidOperationException` still reaches the
existing configuration/error exit-1 arm; it is not a successful operation or
an automatic retry. No raw cancellation exception or response is printed.

### One additive self-test definition

The original five scenario bodies and every original helper remain exactly
unchanged. Removing the one new invocation and appended local helpers
reconstructs the complete original 6,105-byte SelfTest file and its baseline
hash. Existing `MemoryStore`/`FakeHandler` implementations were not modified.

The new `StalledResponseMustTimeOutAsync` uses those existing fake request
dispatch semantics plus three strictly local fixtures: a Save/Purge-counting
store, disposal-observing `StreamContent`, and a cancellation-aware partial
stream. It returns 200 headers, delivers a bounded incomplete JSON prefix,
then waits for disposal with the read's cancellation token. It records an
actual cancellation exception at that stalled read, rather than supplying a
timeout result from the handler.

The operation is called with `CancellationToken.None`; its injected finite
client timeout is one second. An independent five-second `WaitAsync` guard
observes the refusal task and **does not supply an operation token**. It
requires both partial-prefix delivery and observed read cancellation, one
request, zero redeem/offline calls, zero Save/Purge, complete DeviceState
equality and no successfully completed login/approval result. It checks both
content and stream disposal **before fixture cleanup**, avoiding a
fixture-driven disposal oracle.

The finally block disposes only the owned synthetic content. This releases
an old implementation's uncancelled wait after a guard failure, then observes
the refusal task under a separate one-second cleanup guard. Cleanup exceptions
are suppressed there to preserve the first test failure; they cannot convert
the earlier independent guard/expectation into a pass. These are nominal
managed test guards, not measured hard process deadlines. All diagnostics are
fixed and no synthetic secret/state/body is printed. This adds no device-proof
transition or I07 case, and exercises no server/OS account state.

### Actual static checks and corrections

- Whole-byte baseline comparison, three full-file inverse proofs and exact
  source-only staged scope passed. The first SelfTest inverse checker failed
  because its reconstruction added an extra blank line before the original
  class close. Corrected the **checker boundary only**, then the full inverse
  matched the original hash; candidate source remained unchanged.
- Static lexical delimiter inspection passed for the three C# files after
  ignoring strings/comments. This is neither a C# grammar/typecheck nor a
  compiler/AST claim. Static guard checks verified the explicit cancellation,
  independent guards, complete-state/counter/no-approval assertions and
  disposal-before-cleanup ordering without evaluating any case or function.
- `git diff --check` and `git diff --cached --check` passed.
- `python3 scripts/check-docs.py` passed before and after the source changes:
  “Markdown links and build-directory layout checked.”
- `python3 scripts/check-repo-hygiene.py` passed, including staged source,
  for **960** tracked files. No checker/build cache was removed.
- Source commit readback matched the three candidate blob/hash identities;
  the source commit ended with a clean worktree. Append-only report-prefix,
  report scope and final documentation/whitespace checks follow below.

No `.NET` SDK/version/compiler/selftest, PowerShell/native/signature command,
HTTP/socket/provider/browser/Driver, helper, Cargo or other runtime was
invoked. No SDK availability inventory was needed or performed; availability
is not asserted. Source inspection must not be reported as an executed
six-case pass. The earlier parser failure and historical 1/1, 11/11 and
Windows-native evidence limits stay exactly as dated above.

### Runtime readiness and original acceptance residual

The immutable three-file source is ready for root's independent review.
The sole prospective managed command remains
`dotnet run --project windows/RiAuth.DeviceHost -- selftest`, **HELD** until
root separately releases it with a usable SDK and resource envelope. It
compiles/runs the original five mocks and the one new case; it does not install
or exercise Windows SAM/DPAPI/ACL/LSA/LogonUI, sign a bundle, test a Windows
update/uninstall or prove release readiness. No native or signed-artifact
credit is added by this slice.

Original I08 remains subject to its already-requested signed Windows host
inputs and actual complete lifecycle evidence. Root owns interpretation,
runtime release, integration/publication and status; this worker changed no
primary/task status or closed-row disposition. No main/accepted edit, push,
merge/alignment, new task/WT/worker/shell, dependency acquisition, installation
or other-worker contact occurred. No runtime lane was taken or released.

Final append validation: the entire 27,950-byte prefix matched d4a2319,
and all **25** full object references resolved. The source stayed byte-exact
to e31fbee; only this report had a pending tracked change, with no untracked
files or unrelated staged changes. Markdown checking, tracked hygiene
(960 files) and whitespace checking passed after the append. Final staged
scope/whitespace and commit readback are checked for the report-only handoff;
they add no runtime or native evidence.

## Existing managed SDK readiness — metadata only, runtime still held

2026-10-03. Reservation `wave30_I08_managed_sdk_readiness`; same project,
WT ed9 and shell0164. Clean starting commit
`a839b2a8bbc308e3ddedb2185f5936adab9ffdcd`. The preceding **37,937 bytes /
558 lines**, SHA-256
`2ffe2670605d6aaeae2e6043fa81d25b6b0e298df3e76f5f940c33cf34cf7e73`,
are preserved exactly. This appendix changes no source/project/configuration.
D01 owns validation; this lane took/released no runtime slot.

**A concrete matching installed SDK is available by metadata:**
`/usr/local/share/dotnet/dotnet`, SDK `9.0.200`, installed runtime `9.0.2`,
SDK RID `osx-arm64`, matching this host's `arm64` machine metadata. No SDK
installation or dependency acquisition is needed for the proposed managed
Debug fixture based on the inspected project/pack metadata. This is a
readiness recommendation for a separately released command, not proof that
the executable loads, the source compiles or any case passes.

### Executable, SDK, runtime and dependency inventory

Inspected only path/lstat/access metadata, directory/version names, bounded
SDK version/runtime-configuration metadata and the project manifest. No
executable or library content was read, loaded or invoked. Concrete findings:

| Existing input | Metadata actually observed |
| --- | --- |
| `/usr/local/share/dotnet/dotnet` | Nonsymlink regular file, 140,128 bytes, root-owned, mode 0755; readable/executable access check true. |
| `sdk/9.0.200` | Nonsymlink root-owned 0755 directory, readable/searchable; sole installed SDK directory under this root. |
| SDK `.version` | 88-byte regular metadata file: `9.0.200`, `osx-arm64`, build `9.0.200-rtm.25073.12`, source identity `90e8b202f25b7c2bf3b883d421ad5b1cb477e8b0`. These are file values, not `dotnet --version` output. |
| CLI/MSBuild/compiler components | Regular root-owned files present: `dotnet.dll`, `MSBuild.dll`, `Roslyn/bincore/csc.dll`, `NuGet.targets`, `NuGet.Build.Tasks.dll`, and `Sdks/Microsoft.NET.Sdk/Sdk/{Sdk.props,Sdk.targets}`. DLLs were statted only. |
| Bounded runtime config metadata | CLI, MSBuild and compiler JSON name `net9.0` / `Microsoft.NETCore.App` `9.0.2`; compiler metadata additionally permits major roll-forward. No resolver was executed. |
| Runtime/host | Installed `host/fxr/9.0.2` and `shared/Microsoft.NETCore.App/9.0.2`; regular `libhostfxr.dylib`, `libhostpolicy.dylib`, `System.Private.CoreLib.dll` present. No library loading/ABI proof. |
| Reference/apphost packs | Installed `Microsoft.NETCore.App.Ref/9.0.2` and `Microsoft.NETCore.App.Host.osx-arm64/9.0.2`; required `ref/net9.0/System.Runtime.dll` and `runtimes/osx-arm64/native/apphost` are regular files. |
| SDK bundled version metadata | `Microsoft.NETCoreSdk.BundledVersions.props`, 51,705 bytes, SHA-256 `887582b3c662e6de057c3e1a89daa500c8f9526f3418cc8f8fddef13a70989ee`: SDK/RID `9.0.200`/`osx-arm64`; net9 targeting and apphost pack versions both `9.0.2`. Inspected as text metadata only, not evaluated as MSBuild. |
| Other SDK roots | No executable at the inspected usual x64, `/usr/local/bin`, Homebrew, `/usr/share/dotnet` or user `.dotnet` candidates. The absolute usable candidate above avoids PATH dependence. |

The project's entire unchanged 19-line manifest targets `net9.0`, with
**zero PackageReference and zero ProjectReference** declarations. The
`win-x64`/self-contained/single-file settings are explicitly Release-only.
The planned command uses the normal Debug configuration, preserving the
managed mock route rather than trying to acquire/build a Windows Release
runtime. SDK metadata names exactly the reference/apphost pack versions
already present. No general package-cache completeness assertion follows.

The existing NuGet package directory `/Users/dominik/.nuget/packages` is
UID501/mode0755 with **115 immediate package directories**, counted by
directory metadata only. No package payload or secret was inspected. The
user NuGet configuration is a 205-byte regular mode0600 file; its content
was **not** read. Case-variant names resolve on this filesystem and are not
credited as two distinct configurations. The future command below uses a
public, private-workspace offline settings file instead of depending on that
unread user configuration or its feeds/credentials.

No `global.json`, `Directory.Build.props`, `Directory.Build.targets`,
`Directory.Packages.props` or ancestor/project NuGet settings file was present
along the project-to-root ancestor path. `Properties/launchSettings.json` is
absent. Relevant SDK-root/resolver/MSBuild/NuGet-package/CLI-home override
presence checks were false; no unrelated environment or credential values
were printed. No config was rewritten or canonicalized.

### Private output and resource readiness

The existing own-worktree `target` and `target/wave27` are nonsymlink
UID501 directories, mode0755, owner-writable/searchable. The proposed leaf
`target/wave27/i08-managed` is **absent**. Project `bin` and `obj` are absent.
No leaf/output/configuration was created by this audit, and no accepted/shared
build output is selected. The root caller must create an exclusive mode0700
leaf and use umask077 **only after separate preparation/runtime release**;
the existing 0755 parent is not itself described as private.

Available disk readings were **15,747,485,696** and **15,742,705,664** bytes,
about 14.66 GiB, versus the 8 GiB floor **8,589,934,592** bytes. These are
dated statvfs observations, not a reserved-capacity or future headroom promise.
Recheck at release and stop before available space approaches the floor.
No build-output/cache deletion or capacity remediation was performed.

### Immutable prospective ONE managed self-test command

This command is a **proposal and has not been invoked**. Root must separately
release the serialized lane and prepare its private output/settings/log
envelope. Run from this existing worktree, on protected source
`e31fbee66f1038cfc2412e17497bbf07f83e1314` with unchanged project; verify its
source hashes above before starting. No baseline/second invocation or
automatic correction/retry is requested.

```sh
env \
  DOTNET_ROOT=/usr/local/share/dotnet \
  DOTNET_CLI_HOME=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/cli-home \
  NUGET_PACKAGES=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/packages \
  DOTNET_CLI_TELEMETRY_OPTOUT=1 \
  DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1 \
  DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE=true \
  DOTNET_NOLOGO=1 \
  DOTNET_PROCESSOR_COUNT=1 \
  DOTNET_CLI_USE_MSBUILD_SERVER=0 \
  MSBUILDDISABLENODEREUSE=1 \
  /usr/local/share/dotnet/dotnet run \
  --project /Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/windows/RiAuth.DeviceHost/RiAuth.DeviceHost.csproj \
  --configuration Debug \
  --property:BaseIntermediateOutputPath=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/obj/ \
  --property:BaseOutputPath=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/bin/ \
  --property:RestoreConfigFile=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/nuget-offline.config \
  --property:RestoreSources=/Users/dominik/.nuget/packages \
  --property:NuGetAudit=false \
  --property:UseSharedCompilation=false \
  --property:BuildInParallel=false \
  --property:ConcurrentBuild=false \
  -- selftest
```

The future exclusive mode0600 `nuget-offline.config` contains only this fixed
public text; it has **not** been written or parsed by NuGet here:

```xml
<configuration>
  <packageSources><clear /></packageSources>
  <fallbackPackageFolders><clear /></fallbackPackageFolders>
</configuration>
```

Its explicit `RestoreConfigFile` excludes unread user feed/credential
configuration. The only proposed restore source is an existing local package
directory; auditing and workload-update notification are disabled. Because
this project has no package references and required SDK packs exist, restore
should generate local assets rather than fetch dependencies. That is a
source/metadata expectation, not a witnessed offline restore. Missing pack,
source query/download attempt, unexpected package acquisition, SDK selection
or CLI argument failure must stop this one attempt for root review; no
installer, new feed, retry or substitute SDK is authorized by this report.

The flags retain default Debug semantics and the unfiltered `selftest`
argument: all original five scenarios plus the single new sixth stalled
content case. They change output/restore/build-resource handling only; the
production 15-second setting and one-second synthetic request timeout are
unchanged. No Release/publish, Windows RID, test-filter, weaker assertion,
new timeout property or stronger-caller cancellation override is introduced.
Command parsing/loadability/build behavior remains unverified until release.

### Expected build and caller cleanup envelope

The planned one invocation performs an offline project restore, a normal
managed Debug compile and `selftest`. It can launch owned MSBuild/compiler/
apphost children; it needs no server, listener, browser, Windows/native device
API or real peer. Shared compiler/MSBuild reuse is disabled, processor count
is one, and project/compiler parallel build settings are false. These are
requested resource settings, not measured peak-thread/RSS guarantees.

Proposed outer envelope: **120 seconds** for the single child/build/fixture,
then **10 seconds** for caller-owned termination/reaping if necessary;
at most **2 GiB aggregate observed RSS** and **256 MiB private output** before
stop, with the unchanged 8 GiB disk floor. These are prospective stop limits,
not observed consumption or a claim that startup/build will finish inside
them. The sixth case separately retains its one-second request deadline,
five-second independent assertion guard and one-second task-observation guard.

Before execution the root caller must establish exclusive private output,
bounded mode0600 stdout/stderr capture (proposed 64 KiB each), child process
group identity and resource/disk observation. Record exact source/SDK
metadata, full bounded output hashes, numeric exit, elapsed duration and
cleanup/resource result **before grading**. Expected success is exit0 plus
the helper's fixed `selftest passed`, with all six defined cases reached;
no per-case timing/count or pass is measured today. On nonzero/timeout/cap/
unexpected download, preserve the first failure and do not retry.

Terminate/reap only the owned child/group, prove no owned compiler/server/
apphost remains, and retain the fixed receipt/log evidence before any optional
cleanup of this exclusive leaf. Never delete SDK, NuGet global cache, parent
targets, accepted outputs or another lane's processes. If ownership/reaping
cannot be established, keep the fixture held and report the concrete boundary.
This audit launches nothing, creates no output/settings file, and deletes
nothing; root owns the eventual controller and release, not this readiness
report.

### Actual evidence and remaining prerequisites

Observed no missing SDK/runtime/reference/apphost component in this bounded
inventory. Remaining local prerequisites are **root's runtime release**,
private caller-envelope preparation and release-time capacity recheck;
executable loadability, CLI parsing and compilation have intentionally not
been tested. SDK/version commands, libraries, compilers, self-test, native/
PowerShell, HTTP/provider/browser/Driver and Cargo remain **UNRUN** here.
No installation, download or dependency mutation occurred.

Protected DeviceHost/Program/SelfTest/project bytes still equal e31fbee;
all three source candidate hashes above and the unchanged project SHA-256
`e5af8331053b1167d6af93ea9ad657a0983e694fa5bb71f8f003370fbed499ea`
were checked. Report-prefix, metadata JSON/text, source identity, documentation,
whitespace and scope checks are static. User NuGet/private credential/artifact
contents were not read, and no real Windows build/lifecycle/signing result is
borrowed from these installed macOS components.

I08 native Windows/environment requirements remain exactly separate and
already requested through root. No original row, primary assignment, closed
row, source/configuration, main/accepted history or status was changed. No
merge/alignment, push, new worker/task/WT/shell or other-worker contact.

Actual readiness static checks: the single fenced command is **1,483 bytes**
excluding fence/final newline, SHA-256
`6217c5e88aa76979bb0f91fae2faabcea2e9ffcfff722983f8058947c827ab7d`.
Line-continuation normalization followed by Python `shlex` data parsing
verified 27 tokens, ten environment assignments, eight build properties,
Debug and the sole `selftest` argument; no command evaluation or CLI syntax
acceptance is inferred. All four source/project files equal their complete
e31fbee blobs; all 37,937 prior report bytes remain identical. The private
leaf and project bin/obj are still absent. A later metadata-only capacity
reading was **16,786,821,120 bytes** available; none is reserved.

`python3 scripts/check-docs.py`, `python3 scripts/check-repo-hygiene.py`
(960 files) and `git diff --check` passed. The unstaged scope is this report
alone, the starting index has no staged changes and no untracked files were
present. Staged whitespace/scope and immutable commit readback complete the
report-only handoff; they add no SDK/runtime evidence.
