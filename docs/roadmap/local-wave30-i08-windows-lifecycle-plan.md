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

## One-shot managed controller design — source only, execution held

2026-10-03. Reservation wave30_I08_managed_one_shot_controller_design;
project 891e7443-8dac-4c1b-897f-9e53cb59c7ee, same WT ed9/shell0164.
I07 now owns the sole validation lane. This is a prospective complete inline
Python outer controller for a future separate root release; its definitions,
imports, functions, native observer, child and main were NEVER evaluated.
No private path/config/output was created, and no slot was taken/released.

Preserved the entire a6fe92178f4a6a03874ba17f27b98e9bd764ca35 report:
51,619 bytes / 772 lines, SHA-256
f0df4fb2d5d4a0f9a51cf5dcbf9b4ecec463de29daa5c9bc9b43d3ca1fe45553.
The SDK-readiness appendix's old D01 lane statement remains dated history;
current execution is held for I07. Source stays exact e31fbee and its project.

### Complete prospective controller

The following fence is the whole payload, including its final newline:
31494 bytes / 710 lines, SHA-256
3a3703e6fb7095f2cc3d152aa170c3b68c03acf688c9361f11517cac99078490. Only Python AST/data/hash checks are authorized here.
Root must review the entire wrapper and separately release its one invocation;
the main guard is not an instruction to execute during this source review.

```python
# DESIGN ONLY: root must separately release this exact one-shot controller.
import ctypes
import hashlib
import json
import os
from pathlib import Path
import selectors
import shlex
import signal
import stat
import subprocess
import sys
import time

ROOT = Path("/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27")
LEAF = ROOT / "target/wave27/i08-managed"
PROJECT = ROOT / "windows/RiAuth.DeviceHost/RiAuth.DeviceHost.csproj"
SDK = Path("/usr/local/share/dotnet")
SOURCE_PIN = "e31fbee66f1038cfc2412e17497bbf07f83e1314"
COMMAND_SHA = "6217c5e88aa76979bb0f91fae2faabcea2e9ffcfff722983f8058947c827ab7d"
COMMAND = r"""env \
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
  -- selftest"""
CONFIG = b"""<configuration>
  <packageSources><clear /></packageSources>
  <fallbackPackageFolders><clear /></fallbackPackageFolders>
</configuration>
"""
SOURCE_HASHES = {
    "DeviceHost.cs": "c87516ac0323e4d009e6d438cfdf2b74918db3d7c0ad9ab4b9b67fd2367c48ee",
    "Program.cs": "a1fe254ebf65a2153fcf2a17728b4b1b2283ca3d6e991be92eff2bb26126a241",
    "SelfTest.cs": "69bd5f5031bfdb9b974cb2e8201e6f2823a3f32e924eb53f5a38392807e1b38e",
    "RiAuth.DeviceHost.csproj": "e5af8331053b1167d6af93ea9ad657a0983e694fa5bb71f8f003370fbed499ea",
    "WindowsLocalAccount.cs": "85641bd7baef27b419899ec76d9665c8d5cfb90b59641bafe5908524763f0d0c",
    "WindowsStateStore.cs": "9999747d0183650a109368feb0d11644ff90ad30c23b0d0be6b0ca3a930d9ce0",
}
METADATA_HASHES = {
    ".version": "835299a4fd4532244a680605ad2047c1d44d6f8a34834b1bb747fa74ca38e11a",
    "dotnet.runtimeconfig.json": "e90a7dd2984b3ade889585a0f593d097a9421f9a0d63d64ae8ffdd313e24daf0",
    "MSBuild.runtimeconfig.json": "e90a7dd2984b3ade889585a0f593d097a9421f9a0d63d64ae8ffdd313e24daf0",
    "Roslyn/bincore/csc.runtimeconfig.json": "e46be9b13a311147cbc2203dae66958ced66105c7369690ce4ab75fdbcebb561",
    "Microsoft.NETCoreSdk.BundledVersions.props": "887582b3c662e6de057c3e1a89daa500c8f9526f3418cc8f8fddef13a70989ee",
}
SDK_FILES = {
    "dotnet": (140128, 0o755),
    "sdk/9.0.200/dotnet.dll": (3394048, 0o644),
    "sdk/9.0.200/MSBuild.dll": (1035776, 0o644),
    "sdk/9.0.200/Roslyn/bincore/csc.dll": (132096, 0o644),
    "sdk/9.0.200/NuGet.targets": (74726, 0o644),
    "sdk/9.0.200/NuGet.Build.Tasks.dll": (233984, 0o644),
    "sdk/9.0.200/Sdks/Microsoft.NET.Sdk/Sdk/Sdk.props": (2432, 0o644),
    "sdk/9.0.200/Sdks/Microsoft.NET.Sdk/Sdk/Sdk.targets": (4767, 0o644),
    "host/fxr/9.0.2/libhostfxr.dylib": (401072, 0o755),
    "shared/Microsoft.NETCore.App/9.0.2/libhostpolicy.dylib": (420240, 0o755),
    "shared/Microsoft.NETCore.App/9.0.2/System.Private.CoreLib.dll": (16264704, 0o644),
    "packs/Microsoft.NETCore.App.Ref/9.0.2/ref/net9.0/System.Runtime.dll": (837120, 0o644),
    "packs/Microsoft.NETCore.App.Host.osx-arm64/9.0.2/runtimes/osx-arm64/native/apphost": (140896, 0o755),
}
GIB = 1024 ** 3
START_FREE = 9 * GIB
STOP_FREE = 17 * GIB // 2
RSS_CAP = 2 * GIB
TREE_CAP = 256 * 1024 ** 2
LOG_CAP = 64 * 1024
RECEIPT_CAP = 512 * 1024
ENTRY_CAP = 4096
PID_CAP = 256
SAMPLE_CAP = 144
CHILD_SECONDS = 120.0
CLEANUP_SECONDS = 10.0

class FixedFailure(Exception):
    pass

# Numeric Darwin layouts from the installed public headers; names are padding.
class BsdInfo(ctypes.Structure):
    _fields_ = [(n, ctypes.c_uint32) for n in (
        "flags", "status", "xstatus", "pid", "ppid", "uid", "gid",
        "ruid", "rgid", "svuid", "svgid", "reserved")] + [
        ("unused_names", ctypes.c_byte * 48),
        ("nfiles", ctypes.c_uint32), ("pgid", ctypes.c_uint32),
        ("jobc", ctypes.c_uint32), ("tdev", ctypes.c_uint32),
        ("tpgid", ctypes.c_uint32), ("nice", ctypes.c_int32),
        ("start_sec", ctypes.c_uint64), ("start_usec", ctypes.c_uint64)]

class TaskInfo(ctypes.Structure):
    _fields_ = [(n, ctypes.c_uint64) for n in (
        "virtual", "resident", "total_user", "total_system",
        "threads_user", "threads_system")] + [(n, ctypes.c_int32) for n in (
        "policy", "faults", "pageins", "cow_faults", "messages_sent",
        "messages_received", "syscalls_mach", "syscalls_unix", "csw",
        "threadnum", "numrunning", "priority")]

def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      allow_nan=False).encode("ascii")

def digest(data):
    return hashlib.sha256(data).hexdigest()

def bounded_regular(path, cap, owner=None):
    # Reject symlink components before reading only approved public inputs/logs.
    if path.resolve(strict=True) != path:
        raise FixedFailure("input_symlink")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_size > cap:
            raise FixedFailure("input_type_size")
        if owner is not None and info.st_uid != owner:
            raise FixedFailure("input_owner")
        data = bytearray()
        while len(data) <= cap:
            chunk = os.read(fd, min(4096, cap + 1 - len(data)))
            if not chunk:
                break
            data.extend(chunk)
        if len(data) > cap:
            raise FixedFailure("input_type_size")
        return bytes(data)
    finally:
        os.close(fd)

def free_bytes():
    info = os.statvfs(ROOT)
    return info.f_bavail * info.f_frsize

def check_directory(path, owner):
    info = path.lstat()
    if (not stat.S_ISDIR(info.st_mode) or info.st_uid != owner
            or path.resolve(strict=True) != path):
        raise FixedFailure("directory_identity")
    return info

def verify_sources():
    if sorted(item.name for item in PROJECT.parent.iterdir()) != sorted(SOURCE_HASHES):
        raise FixedFailure("project_input_catalog")
    observed = {}
    for name, expected in SOURCE_HASHES.items():
        value = digest(bounded_regular(PROJECT.parent / name, 65536))
        if value != expected:
            raise FixedFailure("source_identity")
        observed[name] = value
    for name in ("bin", "obj"):
        if os.path.lexists(PROJECT.parent / name):
            raise FixedFailure("project_output_present")
    return observed

def preflight():
    if Path.cwd() != ROOT or sys.platform != "darwin" or os.uname().machine != "arm64":
        raise FixedFailure("host_cwd_identity")
    if not all(hasattr(os, n) for n in ("waitid", "WNOWAIT", "WEXITED", "P_PID")):
        raise FixedFailure("wait_observer_unavailable")
    if os.path.lexists(LEAF):
        raise FixedFailure("private_leaf_present")
    for parent in (ROOT, ROOT / "target", ROOT / "target/wave27"):
        check_directory(parent, os.getuid())
    if free_bytes() < START_FREE:
        raise FixedFailure("start_disk")
    for key in ("DOTNET_ROOT", "DOTNET_ROOT_ARM64", "DOTNET_ROOT_X64",
                "DOTNET_MSBUILD_SDK_RESOLVER_SDKS_DIR",
                "DOTNET_MSBUILD_SDK_RESOLVER_SDKS_VER", "MSBuildSDKsPath",
                "MSBUILD_EXE_PATH", "DOTNET_CLI_HOME", "NUGET_PACKAGES",
                "DOTNET_STARTUP_HOOKS", "DOTNET_ADDITIONAL_DEPS"):
        if key in os.environ:
            raise FixedFailure("inherited_override")
    for parent in (PROJECT.parent, *PROJECT.parent.parents):
        for name in ("global.json", "Directory.Build.props", "Directory.Build.targets",
                     "Directory.Packages.props", "NuGet.Config", "nuget.config"):
            if os.path.lexists(parent / name):
                raise FixedFailure("inherited_configuration")
    if os.path.lexists(PROJECT.parent / "Properties/launchSettings.json"):
        raise FixedFailure("launch_settings_present")
    sources = verify_sources()
    if sorted(item.name for item in (SDK / "sdk").iterdir()) != ["9.0.200"]:
        raise FixedFailure("sdk_selection_metadata")
    for relative, (size, mode) in SDK_FILES.items():
        path = SDK / relative
        info = path.lstat()
        if (path.resolve(strict=True) != path or not stat.S_ISREG(info.st_mode)
                or info.st_uid != 0 or info.st_size != size
                or stat.S_IMODE(info.st_mode) != mode):
            raise FixedFailure("sdk_file_metadata")
    if not os.access(SDK / "dotnet", os.R_OK | os.X_OK):
        raise FixedFailure("sdk_executable_access")
    for relative in ("sdk/9.0.200", "host/fxr/9.0.2",
                     "shared/Microsoft.NETCore.App/9.0.2",
                     "packs/Microsoft.NETCore.App.Ref/9.0.2",
                     "packs/Microsoft.NETCore.App.Host.osx-arm64/9.0.2"):
        check_directory(SDK / relative, 0)
    metadata = {}
    for name, expected in METADATA_HASHES.items():
        value = digest(bounded_regular(SDK / "sdk/9.0.200" / name, 60000, 0))
        if value != expected:
            raise FixedFailure("sdk_metadata_identity")
        metadata[name] = value
    check_directory(Path("/Users/dominik/.nuget/packages"), os.getuid())
    if len(COMMAND.encode()) != 1483 or digest(COMMAND.encode()) != COMMAND_SHA:
        raise FixedFailure("command_identity")
    tokens = shlex.split(COMMAND.replace("\\\n", ""), posix=True)
    if (len(tokens) != 27 or tokens[0] != "env"
            or tokens[11:13] != ["/usr/local/share/dotnet/dotnet", "run"]
            or tokens[-2:] != ["--", "selftest"]):
        raise FixedFailure("command_tokens")
    assignments = dict(item.split("=", 1) for item in tokens[1:11])
    if len(assignments) != 10:
        raise FixedFailure("command_environment")
    env = os.environ.copy()
    env.update(assignments)
    inputs = {"source_pin": SOURCE_PIN, "sources": sources,
              "sdk_public_metadata": metadata, "sdk_file_metadata": SDK_FILES,
              "command_sha256": COMMAND_SHA, "command_bytes": 1483,
              "explicit_environment": assignments, "config_sha256": digest(CONFIG),
              "start_free": START_FREE, "stop_free": STOP_FREE,
              "rss_cap": RSS_CAP, "tree_cap": TREE_CAP, "log_cap": LOG_CAP,
              "child_seconds": CHILD_SECONDS, "cleanup_seconds": CLEANUP_SECONDS}
    return tokens[11:], env, inputs

class GroupObserver:
    def __init__(self):
        if ctypes.sizeof(BsdInfo) != 136 or ctypes.sizeof(TaskInfo) != 96:
            raise FixedFailure("process_abi")
        self.lib = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
        self.lib.proc_listpids.argtypes = (
            ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_int)
        self.lib.proc_listpids.restype = ctypes.c_int
        self.lib.proc_pidinfo.argtypes = (
            ctypes.c_int, ctypes.c_int, ctypes.c_uint64,
            ctypes.c_void_p, ctypes.c_int)
        self.lib.proc_pidinfo.restype = ctypes.c_int
        self.identities = {}

    def members(self, pgid):
        buf = (ctypes.c_int * (PID_CAP + 1))()
        ctypes.set_errno(0)
        count = self.lib.proc_listpids(2, pgid, buf, ctypes.sizeof(buf))
        if (count < 0 or (count == 0 and ctypes.get_errno() != 0)
                or count % 4 or count >= ctypes.sizeof(buf)):
            raise FixedFailure("group_list_bound")
        values = sorted({int(pid) for pid in buf[:count // 4] if pid > 0})
        if len(values) > PID_CAP:
            raise FixedFailure("group_member_cap")
        return values

    def info(self, pid, pgid):
        value = BsdInfo()
        count = self.lib.proc_pidinfo(pid, 3, 0, ctypes.byref(value), 136)
        if count != 136:
            try:
                if os.getpgid(pid) != pgid:
                    return None
            except ProcessLookupError:
                return None
            raise FixedFailure("process_identity_unreadable")
        if value.pid != pid or value.pgid != pgid or value.uid != os.getuid():
            raise FixedFailure("process_identity")
        identity = (int(value.start_sec), int(value.start_usec))
        if pid in self.identities and self.identities[pid] != identity:
            raise FixedFailure("process_identity_changed")
        self.identities[pid] = identity
        return value

    def sample(self, pgid):
        members = self.members(pgid)
        resident = 0
        for pid in members:
            identity = self.info(pid, pgid)
            if identity is None or identity.status == 5:  # SZOMB, not running.
                continue
            task = TaskInfo()
            count = self.lib.proc_pidinfo(pid, 4, 0, ctypes.byref(task), 96)
            if count != 96:
                again = self.info(pid, pgid)
                if again is None or again.status == 5:
                    continue
                raise FixedFailure("rss_unreadable")
            resident += int(task.resident)
        return members, resident

def tree_bytes(leaf_identity):
    info = check_directory(LEAF, os.getuid())
    if (info.st_dev, info.st_ino) != leaf_identity:
        raise FixedFailure("private_leaf_identity")
    total, entries = 0, 0
    def walk_error(_):
        raise FixedFailure("private_scan")
    for parent, dirs, files in os.walk(LEAF, followlinks=False, onerror=walk_error):
        if len(Path(parent).relative_to(LEAF).parts) > 32:
            raise FixedFailure("private_depth")
        for name in dirs + files:
            entries += 1
            if entries > ENTRY_CAP:
                raise FixedFailure("private_entry_cap")
            item = (Path(parent) / name).lstat()
            if (item.st_uid != os.getuid()
                    or not (stat.S_ISREG(item.st_mode) or stat.S_ISDIR(item.st_mode))):
                raise FixedFailure("private_entry_type")
            total += max(item.st_size, item.st_blocks * 512)
    return total

def run():
    first_error = None
    child = None
    pgid = None
    verified_group = False
    child_started_at = None
    cleanup_deadline = None
    reaped = False
    child_exit = None
    observer = None
    leaf_identity = None
    inputs = None
    input_hash = None
    samples = []
    logs = {}
    selector = selectors.DefaultSelector()
    cleanup = {"attempted": False, "term": False, "kill": False,
               "reaped": False, "group_empty": None, "errors": []}
    phase = "preflight"
    receipt_durable = False

    def latch(code, exc=None):
        nonlocal first_error
        if first_error is None:
            names = ((FixedFailure, "FixedFailure"), (OSError, "OSError"),
                     (ValueError, "ValueError"), (RuntimeError, "RuntimeError"),
                     (KeyboardInterrupt, "KeyboardInterrupt"), (SystemExit, "SystemExit"))
            kind = next((name for cls, name in names if type(exc) is cls), "Other")
            first_error = {"code": code, "class": kind if exc is not None else None}

    def catch(code, exc):
        if type(exc) is FixedFailure:
            latch(exc.args[0], exc)  # Only controller-owned fixed string literals.
        else:
            latch(code, exc)

    def cleanup_error(code):
        if code not in cleanup["errors"] and len(cleanup["errors"]) < 12:
            cleanup["errors"].append(code)

    def exclusive(name):
        fd = os.open(LEAF / name,
                     os.O_RDWR | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        if stat.S_IMODE(os.fstat(fd).st_mode) != 0o600:
            os.close(fd)
            raise FixedFailure("private_file_mode")
        return fd

    def write_all(fd, data):
        view = memoryview(data)
        while view:
            count = os.write(fd, view)
            if count <= 0:
                raise FixedFailure("private_write")
            view = view[count:]

    def ended():
        # No poll()/wait() before cleanup: WNOWAIT reserves the group-leader PID.
        return os.waitid(os.P_PID, child.pid,
                         os.WEXITED | os.WNOHANG | os.WNOWAIT) is not None

    def pump(timeout):
        for key, _ in selector.select(max(0.0, timeout)):
            name = key.data
            try:
                data = os.read(key.fileobj.fileno(), 4096)
            except BlockingIOError:
                continue
            if not data:
                selector.unregister(key.fileobj)
                key.fileobj.close()
                logs[name]["eof"] = True
                continue
            log = logs[name]
            log["seen"] += len(data)
            available = LOG_CAP - log["kept"]
            retained = data[:max(0, available)]
            if retained:
                write_all(log["fd"], retained)
                log["kept"] += len(retained)
            if len(data) > available:
                log["truncated"] = True
                latch(name + "_cap")

    def sample():
        members, resident = observer.sample(pgid)
        used = tree_bytes(leaf_identity)
        available = free_bytes()
        if len(samples) >= SAMPLE_CAP:
            raise FixedFailure("sample_cap")
        samples.append({"seconds": round(time.monotonic() - child_started_at, 6),
                        "members": members, "rss_bytes": resident,
                        "private_bytes": used, "free_bytes": available})
        if available <= STOP_FREE:
            raise FixedFailure("stop_disk")
        if resident > RSS_CAP:
            raise FixedFailure("rss_cap")
        if used + RECEIPT_CAP > TREE_CAP:
            raise FixedFailure("private_growth")
        return members

    def signal_group(sig):
        if not verified_group or reaped:
            cleanup_error("group_signal_ownership")
            return
        try:
            os.killpg(pgid, sig)
            cleanup["term" if sig == signal.SIGTERM else "kill"] = True
        except ProcessLookupError:
            pass
        except BaseException:
            cleanup_error("group_signal")

    def finish_group():
        nonlocal reaped, child_exit, cleanup_deadline
        if child is None:
            cleanup["group_empty"] = True
            return
        cleanup["attempted"] = True
        if cleanup_deadline is None:
            cleanup_deadline = time.monotonic() + CLEANUP_SECONDS
        if not verified_group:
            cleanup_error("group_unverified")
            # Only the unreaped direct Popen child is addressable in this branch.
            try:
                child.kill()
            except BaseException:
                cleanup_error("direct_kill")
        else:
            signal_group(signal.SIGTERM)
        kill_at = cleanup_deadline - 5.0
        if first_error is not None and first_error["code"] in (
                "stop_disk", "rss_cap", "private_growth", "stdout_cap", "stderr_cap"):
            kill_at = time.monotonic()
        next_sample = time.monotonic()
        while time.monotonic() < cleanup_deadline:
            now = time.monotonic()
            if not cleanup["kill"] and now >= kill_at:
                if verified_group:
                    signal_group(signal.SIGKILL)
                else:
                    try:
                        child.kill()
                        cleanup["kill"] = True
                    except BaseException:
                        cleanup_error("direct_kill")
            try:
                pump(min(0.1, cleanup_deadline - now))
            except BaseException as exc:
                catch("cleanup_capture", exc)
                cleanup_error("capture_drain")
            try:
                if now >= next_sample and verified_group:
                    next_sample = now + 1.0
                    sample()
                members = observer.members(pgid) if verified_group else []
                done = ended()
                if done and (not verified_group or not set(members) - {child.pid}):
                    # Send final KILL while the unreaped leader still reserves PGID.
                    if verified_group:
                        signal_group(signal.SIGKILL)
                    child_exit = child.wait(timeout=max(0.0, cleanup_deadline - time.monotonic()))
                    reaped = True
                    break
            except BaseException as exc:
                cleanup_error("group_observation")
                catch("cleanup_observation", exc)
                # Observation errors remove grace, not group ownership.
                kill_at = time.monotonic()
        if not reaped:
            if verified_group:
                signal_group(signal.SIGKILL)
            try:
                child_exit = child.wait(timeout=max(0.0, cleanup_deadline - time.monotonic()))
                reaped = True
            except BaseException:
                cleanup_error("direct_reap")
        cleanup["reaped"] = reaped
        # After reaping, never signal a potentially reused group number.
        try:
            os.killpg(pgid, 0)
            cleanup["group_empty"] = False
        except ProcessLookupError:
            cleanup["group_empty"] = True
        except BaseException:
            cleanup["group_empty"] = None
            cleanup_error("final_group_probe")
        if cleanup["group_empty"] is not True:
            cleanup_error("group_not_empty")
        if verified_group and observer is not None:
            try:
                if observer.members(pgid):
                    cleanup["group_empty"] = False
                    cleanup_error("final_group_members")
            except BaseException:
                cleanup["group_empty"] = None
                cleanup_error("final_group_members")

    try:
        argv, env, inputs = preflight()
        input_hash = digest(canonical(inputs))
        observer = GroupObserver()  # Future numeric observation only, no helper child.
        os.umask(0o077)  # This standalone controller retains 077 until exit.
        os.mkdir(LEAF, 0o700)  # Fails if any competing/existing leaf is present.
        leaf = check_directory(LEAF, os.getuid())
        if stat.S_IMODE(leaf.st_mode) != 0o700:
            raise FixedFailure("private_leaf_mode")
        leaf_identity = (leaf.st_dev, leaf.st_ino)
        for name in ("cli-home", "packages"):
            os.mkdir(LEAF / name, 0o700)
        fd = exclusive("nuget-offline.config")
        try:
            write_all(fd, CONFIG)
            os.fsync(fd)
        finally:
            os.close(fd)
        for name in ("stdout", "stderr"):
            logs[name] = {"fd": exclusive(name + ".log"), "seen": 0,
                          "kept": 0, "truncated": False, "eof": False}
        if free_bytes() < START_FREE:
            raise FixedFailure("start_disk")
        phase = "child"
        child_started_at = time.monotonic()
        child = subprocess.Popen(argv, cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                 bufsize=0, start_new_session=True, close_fds=True)
        pgid = child.pid
        verified_group = os.getpgid(pgid) == pgid and os.getsid(pgid) == pgid
        if not verified_group:
            raise FixedFailure("new_group_identity")
        for name, stream in (("stdout", child.stdout), ("stderr", child.stderr)):
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_READ, name)
        owner = observer.info(pgid, pgid)
        if owner is None:
            raise FixedFailure("new_group_owner_record")
        launch = canonical({"schema_version": 1, "input_sha256": input_hash,
                            "pid": child.pid, "pgid": pgid, "session": pgid,
                            "owner_uid": int(owner.uid),
                            "leader_start_sec": int(owner.start_sec),
                            "leader_start_usec": int(owner.start_usec),
                            "start_monotonic": child_started_at,
                            "child_deadline_monotonic": child_started_at + CHILD_SECONDS,
                            "grade_performed": False}) + b"\n"
        if len(launch) > 2048:
            raise FixedFailure("launch_receipt_cap")
        fd = exclusive("launch.json")
        try:
            write_all(fd, launch)
            os.fsync(fd)
        finally:
            os.close(fd)
        deadline = child_started_at + CHILD_SECONDS
        next_sample = time.monotonic()
        while first_error is None:
            now = time.monotonic()
            if now >= deadline:
                latch("child_timeout")
                break
            if now >= next_sample:
                sample()
                next_sample = time.monotonic() + 1.0
            pump(min(0.1, deadline - time.monotonic()))
            if ended():
                break
    except BaseException as exc:
        catch(phase + "_exception", exc)
    finally:
        phase = "cleanup"
        try:
            finish_group()
        except BaseException as exc:
            catch("cleanup_exception", exc)
            cleanup_error("cleanup_exception")
            # A final reserved-PID group KILL; the same deadline is not reset.
            if child is not None and not reaped:
                signal_group(signal.SIGKILL)
                try:
                    child_exit = child.wait(timeout=max(
                        0.0, (cleanup_deadline or time.monotonic()) - time.monotonic()))
                    reaped = True
                    cleanup["reaped"] = True
                except BaseException:
                    cleanup_error("direct_reap")
            cleanup["group_empty"] = None
        try:
            # Final bounded nonblocking drain after group cleanup, not a new grace.
            for _ in range(34):
                if not selector.get_map():
                    break
                pump(0.0)
        except BaseException as exc:
            catch("capture_finalize", exc)
        for key in list(selector.get_map().values()):
            try:
                selector.unregister(key.fileobj)
                key.fileobj.close()
            except BaseException:
                cleanup_error("pipe_close")
        try:
            selector.close()
        except BaseException:
            cleanup_error("selector_close")
        for log in logs.values():
            try:
                os.fsync(log["fd"])
            except BaseException as exc:
                catch("capture_fsync", exc)
            finally:
                try:
                    os.close(log["fd"])
                except BaseException:
                    cleanup_error("capture_close")

    # All serialization and grading are AFTER the cleanup finally above.
    if child_exit is not None and child_exit != 0:
        latch("child_nonzero")
    captured = {}
    post_sources = None
    try:
        if leaf_identity is None:
            raise FixedFailure("no_owned_receipt_workspace")
        for name, log in logs.items():
            data = bounded_regular(LEAF / (name + ".log"), LOG_CAP, os.getuid())
            captured[name] = data
            if len(data) != log["kept"]:
                latch("capture_identity")  # Keep actual bounded failed-output evidence.
        post_sources = verify_sources()
        tree = tree_bytes(leaf_identity)
        available = free_bytes()
        if available <= STOP_FREE:
            latch("stop_disk")
        if tree + RECEIPT_CAP > TREE_CAP:
            latch("private_growth")
        for name, log in logs.items():
            if not log["eof"]:
                latch(name + "_eof_unverified")
        elapsed = None if child_started_at is None else time.monotonic() - child_started_at
        receipt = {"schema_version": 1, "source_pin": SOURCE_PIN,
                   "command_sha256": COMMAND_SHA, "input_sha256": input_hash,
                   "inputs": inputs, "child_started": child is not None,
                   "child_pid": None if child is None else child.pid, "pgid": pgid,
                   "child_exit": child_exit, "elapsed_through_cleanup": elapsed,
                   "first_error": first_error, "cleanup": cleanup,
                   "post_sources": post_sources, "resource_samples": samples,
                   "final_private_bytes_before_receipt": tree, "final_free_bytes": available,
                   "outputs": {name: {"bytes": len(captured.get(name, b"")),
                              "sha256": digest(captured.get(name, b"")),
                              "bytes_observed": log["seen"], "bytes_kept_recorded": log["kept"],
                              "truncated": log["truncated"], "eof": log["eof"]}
                               for name, log in logs.items()},
                   "grade_performed": False}
        payload = canonical(receipt) + b"\n"
        if len(payload) > RECEIPT_CAP:
            raise FixedFailure("receipt_cap")
        fd = exclusive("result.json")
        try:
            write_all(fd, payload)
            os.fsync(fd)
        finally:
            os.close(fd)
        dir_fd = os.open(LEAF, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            os.fsync(dir_fd)
        finally:
            os.close(dir_fd)
        receipt_durable = True
    except BaseException as exc:
        catch("receipt_exception", exc)
        # No retry/overwrite and no grading. The preceding finally already
        # attempted group cleanup even if serialization/fsync now fails.
    ok = (receipt_durable and first_error is None and child is not None
          and child_exit == 0 and cleanup["reaped"]
          and cleanup["group_empty"] is True and not cleanup["errors"]
          and captured.get("stdout", b"").splitlines().count(b"selftest passed") == 1)
    # No exception text, raw output, source values, environment, or private path.
    status = "UNGRADABLE" if not receipt_durable else ("PASS" if ok else "FAIL")
    public = {"schema_version": 1, "status": status,
              "receipt_durable": receipt_durable, "input_sha256": input_hash,
              "first_error": first_error, "cleanup_errors": cleanup["errors"],
              "child_exit": child_exit, "reaped": cleanup["reaped"],
              "group_empty": cleanup["group_empty"]}
    try:
        print(json.dumps(public, sort_keys=True, separators=(",", ":")))
    except BaseException:
        return 2  # Durable private receipt remains; no raw exception is printed.
    return 0 if ok else 2

if __name__ == "__main__":
    sys.exit(run())
```

### Command, source and private-input preservation

The embedded COMMAND is the unchanged 1,483-byte a6fe command, SHA-256
6217c5e88aa76979bb0f91fae2faabcea2e9ffcfff722983f8058947c827ab7d.
The wrapper parses its literal line continuations into the same ten explicit
environment assignments and sixteen dotnet argv elements (27 original env
command tokens). It starts dotnet directly without a shell or an additional
env/helper/version process. The rest of the caller's inherited environment
is preserved; named SDK/build/configuration injection overrides cause a
preflight refusal instead of silently changing the reviewed command.

All six complete project inputs equal e31fbee66f1038cfc2412e17497bbf07f83e1314:
the previously bound DeviceHost.cs, Program.cs, SelfTest.cs and project, plus
WindowsLocalAccount.cs / WindowsStateStore.cs. The latter two hashes in the
wrapper were checked against complete e31 objects and current bytes; they
are additional identity checks, not product edits. The exact six-file project
catalog rejects added source/configuration inputs. Checks run before launch
and after owned cleanup. The project remains net9.0/normal Debug, with no
package/project references or Windows Release RID override. The production
15-second request timeout, original five mock bodies and sole added stalled
response case remain unchanged.

SDK identity is deliberately the reviewed installed metadata, not a claimed
binary attestation: sole SDK directory 9.0.200, public metadata hashes for
.version / CLI-MSBuild-compiler runtime configs / BundledVersions.props,
root-owned regular file sizes/modes for thirteen SDK/runtime/reference/
apphost inputs, exact runtime/pack directories, and executable access.
No DLL/native executable was read for its content, loaded or invoked here.
Source/SDK metadata mismatch, an existing output leaf, unexpected project
bin/obj or inherited ancestor/launch configuration stops before dotnet.
The wrapper does not install/select an alternative SDK or acquire packs.

Only at released execution does an absent leaf become exclusively owned,
mode0700, under the existing nonsymlink own target/wave27. The standalone
controller keeps its own umask077 through exit. Its public XML is the same
fixed empty-feed/fallback configuration from a6fe, mode0600/O_EXCL/NOFOLLOW;
CLI home and package output are private mode0700 subdirectories. The same
command uses only the existing local package directory as restore source.
No user NuGet configuration, credential content or additional feed is read
by this controller. SDK settings still request no audit, advertising, server
reuse, shared compiler or project/compiler parallel build, and one CPU.
This defines expected offline restore behavior; it is not an executed network
or whole-filesystem sandbox. No installation/download command is present.

### Owned execution, failure preservation and durable grading order

Exactly one Popen definition starts the reviewed command, with DEVNULL stdin,
two binary pipes and start_new_session=True. Its PID must be its group/session
leader. The future Darwin observer samples only numeric metadata of that
owned group; it spawns no monitoring process and never requests argv,
environment, path or protocol contents. The private bounded launch.json
captures input hash, own PID/group/session, UID, leader start generation and
absolute monotonic deadline. It supports root ownership review if the outer
controller later becomes unresponsive; it is not a runtime receipt today.

The controller uses waitid WNOHANG/WNOWAIT to observe leader exit without
reaping, reserving its PID until group cleanup. It checks each observed PID's
UID/group/start generation. TERM starts cleanup, KILL follows within the
same ten-second interval (immediately for resource/output caps); a final
KILL is issued while the leader PID is still reserved before normal reaping.
Unverified ownership permits only direct unreaped-child cleanup, records
uncertainty, and cannot pass. After reaping no signal is sent to a possibly
reused group number. Final killpg(0) and numeric group enumeration must both
show no group; false/unknown or cleanup error cannot pass.

The first controller-observed failure is latched once, using fixed codes and
a fixed class allowlist/Other. No exception text, dynamic type name, traceback,
locals or private path is printed. Original managed output is retained only
in bounded private logs; it is not parsed into an inferred source cause.
A nonzero child exit supplies fixed child_nonzero only when no earlier
controller failure was latched. Cross-process failure chronology is not
reconstructed from output. Cleanup results/errors stay separate and cannot
replace the first error. Timeout/cap/observation/source drift all refuse.

The SDK execution block has an unconditional cleanup finally. Group cleanup
and capped pipe drainage precede output fsync, bounded actual-log hashing,
post-source verification and JSON serialization. A receipt records command/
input hashes, pre-input identities, actual numeric child exit, aggregate
elapsed through cleanup, all retained samples, complete bounded output
sizes/hashes/observed/truncation/EOF flags, and reap/final-group facts.
No per-case runtime or timing is inferred. Partial-write counter mismatch
latches failure while retaining actual bounded output/hash evidence.

The exclusive final result.json (maximum 512 KiB) and its directory are fsynced
BEFORE any fixed success-marker grading. A serializer/read/fsync failure
does not retry/overwrite; output is UNGRADABLE/exit2, never pass. The preceding
finally has already attempted cleanup even on serialization failure.
A durable receipt says grade_performed=false and preserves raw observations.
Only afterwards may exit0, no first/cleanup error, verified reaping/empty
group, complete uncapped EOF logs and exactly one full stdout line selftest
passed produce public PASS/exit0. This is an aggregate six-defined-case
completion oracle tied to immutable source, not six measured case durations.
The public JSON never includes raw captured output, input values or paths.
All private build files/receipts remain for root review; this controller
does not delete caches, SDKs, sources, targets or evidence.

### Finite envelope and honest limitations

| Limit | Prospective implementation |
| --- | --- |
| Disk start | At least 9 GiB = 9,663,676,416 bytes before preparation and again before child start. |
| Disk stop | Stop at or below 8.5 GiB = 9,126,805,504 bytes available, leaving a 0.5 GiB margin above the preserved 8 GiB floor. |
| Child / cleanup | One absolute 120-second child interval; one absolute 10-second TERM/KILL/reap interval, never reset by partial output or observation failure. |
| Group RSS | One-second numeric group snapshots; observed sum above 2 GiB refuses. |
| Private growth | One-second metadata scan; observed growth plus 512 KiB final-receipt reserve above 256 MiB refuses. No file contents scanned. |
| Outputs | Each 0600 log retains at most 65,536 bytes, continuously enforced while draining pipes; any additional observed bytes latch failure. |
| Other bounds | At most 256 enumerated PIDs, 144 resource samples, 4,096 private entries/depth 32, 2 KiB launch JSON and 512 KiB final JSON. |

The 512 KiB receipt cap accommodates the bounded full sample/PID list plus
fixed provenance; 128 KiB was not sufficient for that worst-case data shape.
The source-data count bounds do not establish resource performance.
Scans account for the maximum logical/allocated size of each private entry;
they cover this exclusive leaf, not other filesystem writes. Output file
caps are receiver-enforced; RSS/disk/growth are sampled stop conditions,
not kernel quotas, continuous peaks or instantaneous allocation guarantees.
A fast writer, OS scheduling, other disk users or stalled filesystem/kernel
call can cross a sampled threshold before observation/termination. The 8.5 GiB
stop is a safety margin; preserving 8 GiB is not mathematically guaranteed by
one-second observations alone. Root must watch release-time capacity and
retain actual samples/any violation rather than claim a hard floor proof.

Native process observation layouts derive from selected installed public
Darwin declarations: PROC_PGRP_ONLY 2, PROC_PIDTBSDINFO 3 (136-byte numeric/
unused-name layout), PROC_PIDTASKINFO 4 (96-byte layout), and SZOMB 5.
The wrapper checks its ctypes sizes before any child. Public source headers
were read as selected declarations plus whole-byte hash witnesses; no
libproc library call, layout instantiation or candidate class was executed.

Process groups do not contain descendants that deliberately change group/
session, and disabled SDK reuse is a request rather than proof against such
escape. Sequential PID/RSS reads race with exits; snapshots are not atomic
whole-process peak measurements. The final empty group proves only this
group at that observation, not absent remote peers or escaped processes.
Uninterruptible kernel tasks may survive KILL or fail to reap within ten
seconds; those facts remain failed/unknown cleanup. Popen and native/file
syscalls themselves are not forcibly interruptible by the Python loop.
Root's future outer invocation/ownership watch must account for controller
stall/crash and use the private launch generation record before any external
cleanup; this source cannot guarantee hard wall-time cancellation of those
OS operations. It grants no authority to signal an unrelated/reused group.

### Actual source-only validation and corrections

AST/data review checked the whole wrapper and original command, rather than
running definitions/cases. Source-design corrections before this proposed
commit were: checked list-API errno; reserved-PID final group KILL; retained
over-limit growth observations for failure receipts; one-second cleanup
sampling even on a cap error; retained standalone 077; bounded selector/public
output error paths; sole-SDK selection; all-six project catalog binding;
512KiB receipt reserve; private launch generation; nonzero exit classification;
and failed-capture evidence retention. These were corrections to the pending report design only, not production fixes or executed failure/retry results.

Metadata-only Python attribute inspection found waitid/WNOWAIT/P_PID present;
it did not invoke them. New public SDK metadata hashes in the wrapper were
read from the same existing installation, with no secret/config contents.
Selected public header witnesses (whole bytes hashed, not compiled):

| Public header under existing CommandLineTools SDK usr/include | Bytes / SHA-256 |
| --- | --- |
| sys/proc_info.h | 31,262 / e427fa96b348537b21552b9de71e01039410bcad2cedee5584c5e5fddafd70fc |
| libproc.h | 7,575 / 246d87709fc6b9157ce5cf3c475656ac48e0e1ae8bbdc46cf45acd34294448cd |
| sys/wait.h | 10,352 / b77f7dd6f592eba8b0d51c15c7b975472fdad50c12ea296f74f062cbf98dcbf7 |

Static extraction/AST/identity checking passed for the then-complete payload,
all six current/e31 project files, five bounded public SDK metadata hashes,
thirteen SDK binary/build-input metadata entries, and the original 1,483-byte
command. Exactly one Popen and 24 Python function/method definitions were
identified. Those are syntax/source counts, not invoked child/case counts.
The controller was not imported, evaluated, compiled or executed; neither
were its functions, main, native observer or any SDK/self-test.

Final whole-payload identity, report-prefix/scope, documentation/hygiene and
whitespace readback are recorded below after the final text checks.
Native Windows/signing/lifecycle inputs remain separately requested and
unfulfilled by this managed design. All prior source/failure/metadata history
is preserved. No SDK installer/download, service, native Windows, Cargo,
provider/HTTP/browser/Driver, other-worker contact, alignment, main/push,
status or new worker/task/WT/shell operation occurred. Root alone decides
source acceptance, future one-shot release, integration and original gates.

Final actual static readback passed on the exact complete **31,494-byte /
710-line** wrapper (final newline included), SHA-256
**3a3703e6fb7095f2cc3d152aa170c3b68c03acf688c9361f11517cac99078490**.
Python AST parsing/data inspection found 24 function/method definitions and
one Popen; no candidate definition was evaluated. The embedded command
equals all 1,483 original bytes, SHA-256 6217c5e88aa76979bb0f91fae2faabcea2e9ffcfff722983f8058947c827ab7d,
with unchanged ten explicit environment assignments / sixteen SDK argv
elements. Structural checks covered session creation, no shell/second spawn/
poll/eval/exec, WNOWAIT use, absolute deadlines, cleanup-before-serialization
and durable-receipt-before-marker grading. These are source facts, not
executed control-flow, ABI, cleanup or six-case oracles.

All six current project files equal complete e31 Git blobs; the full prior
51,619-byte report prefix and command have exact forward/reversal readback.
The named SDK public metadata hashes and thirteen regular-file metadata
entries matched this existing installation. The selected sys/proc.h
declaration at 153 additionally confirms SZOMB=5; this was not a semantic
full-header review. Private leaf, project bin/obj remain absent. Only this
report was modified; there were no preexisting staged/untracked changes.

Markdown links/build-layout, tracked-file hygiene (960 files) and whitespace
checks passed. Final staged scope/whitespace and immutable commit readback
finish the report-only handoff. No SDK, controller, native observer, fixture,
compiler or runtime was executed and no preparation outputs were created.
Execution remains HELD with I07 owning validation; no lane was acquired or
released, and no original/native Windows gate or task status changed.

## Waitable-child correction — source design only, runtime still held

2026-10-03. Reservation wave30_I08_waitable_child_controller_correction,
project 891e7443-8dac-4c1b-897f-9e53cb59c7ee, same WT ed9/shell0164.
Root identified a concrete blocker in the archived controller: an inherited
ignored SIGCHLD can disable waitable child status. The installed Python
subprocess ECHILD fallback can then synthesize status0 instead of a real
numeric child exit; WNOWAIT/group-leader PID reservation also requires a
waitable child. This is source diagnosis, not an attributed native failure
or an experiment: no signal handler or process was inspected/changed here.

The **entire 9fdc157b7ef09d2998854cf92bea31ea26ce0bf5 prefix** is preserved:
98,172 bytes / 1,715 lines, SHA-256
58d568b61f3589e0643d8d6d4f591b485b3f574e7f5b41ceac82d2434df83f42.
The original 31,494-byte / 710-line / SHA-256 3a3703 controller remains archived
unchanged above, including its source-only limits and earlier review facts.
I07 is now released, but this lane acquired no validation/runtime slot.
Root and independent source review plus a later exact runtime release remain
required. None of the archived or corrected controller is imported/evaluated.

### Local child/signal source witness

Read selected complete relevant methods/branches at installed
/opt/homebrew/Cellar/python@3.14/3.14.6/Frameworks/Python.framework/Versions/3.14/lib/python3.14/subprocess.py:
lines 2010–2058, including _internal_poll ECHILD and _try_wait
ChildProcessError branches at 2027–2048, both substituting status zero when
waiting is disabled. Full-file hash identity **6628ffdd65c093a6c08cae01ffe82877d3ced515aac9e7be0cff16512c30a7d9**
matches root's supplied pin; this is not a full semantic review of that
module or an import/native waitpid experiment.

Read the complete short public signal.py wrapper as text (not imported),
**2495 bytes / 94 lines**, SHA-256
**0363c964c90ac0b3e515de5749205e6e6454051a1211058375d84d91eab6071a**. Its Handlers conversion declares SIG_DFL/SIG_IGN;
signal() wraps _signal.signal and getsignal() wraps _signal.getsignal,
converting known numeric values to canonical Handlers enum members.
The correction therefore verifies exact **is signal.SIG_DFL** identity,
rather than truthiness or a numerically equal foreign value.
This is Python wrapper source evidence; native _signal implementation,
installed dispositions and kernel flags remain unexecuted/unobserved.

### Exact narrow diff

Only a ten-line installer/verifier helper plus one call is added. The helper
calls signal.signal(SIGCHLD,SIG_DFL) exactly once and calls getsignal exactly
once. Unavailable/refused installation/observation raises fixed
sigchld_default_unavailable_or_refused; a returned nondefault handler raises
fixed sigchld_default_unverified. Both flow into the existing first-error
latch and unconditional cleanup, before any Popen/child. No exception message
or prior handler is stored/printed; no new retry/probe is introduced.

```diff
--- 9fdc157-controller.py
+++ waitable-child-controller.py
@@ -94,0 +95,10 @@
+def require_waitable_child():
+    try:
+        signal.signal(signal.SIGCHLD, signal.SIG_DFL)
+        if signal.getsignal(signal.SIGCHLD) is not signal.SIG_DFL:
+            raise FixedFailure("sigchld_default_unverified")
+    except FixedFailure:
+        raise
+    except BaseException:
+        raise FixedFailure("sigchld_default_unavailable_or_refused") from None
+
@@ -541,0 +552 @@
+        require_waitable_child()
```

The call sits in the existing standalone run main-thread path immediately
before phase/child-start setup and the sole Popen. An attempted non-main-thread
or otherwise refused signal installation is caught and refuses before child
creation; ordinary standalone main invokes the native signal API on its main
thread. No threading module or check/extra process is added. The controller
deliberately retains default SIGCHLD until exit; it never restores inherited
SIG_IGN before join/reap or after cleanup. This is intended only as the
standalone supervisor, not a borrowed-library signal policy.

### Complete corrected archived controller

The whole corrected payload is **31893 bytes / 721 lines**
(final newline included), SHA-256 **a833a87be91e401d86617467d2aaf2ee946a49daf7e385ec5c7f9d0fac696f34**. This full fence is data for
root source review, not an execution instruction or runtime release.

```python
# DESIGN ONLY: root must separately release this exact one-shot controller.
import ctypes
import hashlib
import json
import os
from pathlib import Path
import selectors
import shlex
import signal
import stat
import subprocess
import sys
import time

ROOT = Path("/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27")
LEAF = ROOT / "target/wave27/i08-managed"
PROJECT = ROOT / "windows/RiAuth.DeviceHost/RiAuth.DeviceHost.csproj"
SDK = Path("/usr/local/share/dotnet")
SOURCE_PIN = "e31fbee66f1038cfc2412e17497bbf07f83e1314"
COMMAND_SHA = "6217c5e88aa76979bb0f91fae2faabcea2e9ffcfff722983f8058947c827ab7d"
COMMAND = r"""env \
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
  -- selftest"""
CONFIG = b"""<configuration>
  <packageSources><clear /></packageSources>
  <fallbackPackageFolders><clear /></fallbackPackageFolders>
</configuration>
"""
SOURCE_HASHES = {
    "DeviceHost.cs": "c87516ac0323e4d009e6d438cfdf2b74918db3d7c0ad9ab4b9b67fd2367c48ee",
    "Program.cs": "a1fe254ebf65a2153fcf2a17728b4b1b2283ca3d6e991be92eff2bb26126a241",
    "SelfTest.cs": "69bd5f5031bfdb9b974cb2e8201e6f2823a3f32e924eb53f5a38392807e1b38e",
    "RiAuth.DeviceHost.csproj": "e5af8331053b1167d6af93ea9ad657a0983e694fa5bb71f8f003370fbed499ea",
    "WindowsLocalAccount.cs": "85641bd7baef27b419899ec76d9665c8d5cfb90b59641bafe5908524763f0d0c",
    "WindowsStateStore.cs": "9999747d0183650a109368feb0d11644ff90ad30c23b0d0be6b0ca3a930d9ce0",
}
METADATA_HASHES = {
    ".version": "835299a4fd4532244a680605ad2047c1d44d6f8a34834b1bb747fa74ca38e11a",
    "dotnet.runtimeconfig.json": "e90a7dd2984b3ade889585a0f593d097a9421f9a0d63d64ae8ffdd313e24daf0",
    "MSBuild.runtimeconfig.json": "e90a7dd2984b3ade889585a0f593d097a9421f9a0d63d64ae8ffdd313e24daf0",
    "Roslyn/bincore/csc.runtimeconfig.json": "e46be9b13a311147cbc2203dae66958ced66105c7369690ce4ab75fdbcebb561",
    "Microsoft.NETCoreSdk.BundledVersions.props": "887582b3c662e6de057c3e1a89daa500c8f9526f3418cc8f8fddef13a70989ee",
}
SDK_FILES = {
    "dotnet": (140128, 0o755),
    "sdk/9.0.200/dotnet.dll": (3394048, 0o644),
    "sdk/9.0.200/MSBuild.dll": (1035776, 0o644),
    "sdk/9.0.200/Roslyn/bincore/csc.dll": (132096, 0o644),
    "sdk/9.0.200/NuGet.targets": (74726, 0o644),
    "sdk/9.0.200/NuGet.Build.Tasks.dll": (233984, 0o644),
    "sdk/9.0.200/Sdks/Microsoft.NET.Sdk/Sdk/Sdk.props": (2432, 0o644),
    "sdk/9.0.200/Sdks/Microsoft.NET.Sdk/Sdk/Sdk.targets": (4767, 0o644),
    "host/fxr/9.0.2/libhostfxr.dylib": (401072, 0o755),
    "shared/Microsoft.NETCore.App/9.0.2/libhostpolicy.dylib": (420240, 0o755),
    "shared/Microsoft.NETCore.App/9.0.2/System.Private.CoreLib.dll": (16264704, 0o644),
    "packs/Microsoft.NETCore.App.Ref/9.0.2/ref/net9.0/System.Runtime.dll": (837120, 0o644),
    "packs/Microsoft.NETCore.App.Host.osx-arm64/9.0.2/runtimes/osx-arm64/native/apphost": (140896, 0o755),
}
GIB = 1024 ** 3
START_FREE = 9 * GIB
STOP_FREE = 17 * GIB // 2
RSS_CAP = 2 * GIB
TREE_CAP = 256 * 1024 ** 2
LOG_CAP = 64 * 1024
RECEIPT_CAP = 512 * 1024
ENTRY_CAP = 4096
PID_CAP = 256
SAMPLE_CAP = 144
CHILD_SECONDS = 120.0
CLEANUP_SECONDS = 10.0

class FixedFailure(Exception):
    pass

def require_waitable_child():
    try:
        signal.signal(signal.SIGCHLD, signal.SIG_DFL)
        if signal.getsignal(signal.SIGCHLD) is not signal.SIG_DFL:
            raise FixedFailure("sigchld_default_unverified")
    except FixedFailure:
        raise
    except BaseException:
        raise FixedFailure("sigchld_default_unavailable_or_refused") from None

# Numeric Darwin layouts from the installed public headers; names are padding.
class BsdInfo(ctypes.Structure):
    _fields_ = [(n, ctypes.c_uint32) for n in (
        "flags", "status", "xstatus", "pid", "ppid", "uid", "gid",
        "ruid", "rgid", "svuid", "svgid", "reserved")] + [
        ("unused_names", ctypes.c_byte * 48),
        ("nfiles", ctypes.c_uint32), ("pgid", ctypes.c_uint32),
        ("jobc", ctypes.c_uint32), ("tdev", ctypes.c_uint32),
        ("tpgid", ctypes.c_uint32), ("nice", ctypes.c_int32),
        ("start_sec", ctypes.c_uint64), ("start_usec", ctypes.c_uint64)]

class TaskInfo(ctypes.Structure):
    _fields_ = [(n, ctypes.c_uint64) for n in (
        "virtual", "resident", "total_user", "total_system",
        "threads_user", "threads_system")] + [(n, ctypes.c_int32) for n in (
        "policy", "faults", "pageins", "cow_faults", "messages_sent",
        "messages_received", "syscalls_mach", "syscalls_unix", "csw",
        "threadnum", "numrunning", "priority")]

def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      allow_nan=False).encode("ascii")

def digest(data):
    return hashlib.sha256(data).hexdigest()

def bounded_regular(path, cap, owner=None):
    # Reject symlink components before reading only approved public inputs/logs.
    if path.resolve(strict=True) != path:
        raise FixedFailure("input_symlink")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_size > cap:
            raise FixedFailure("input_type_size")
        if owner is not None and info.st_uid != owner:
            raise FixedFailure("input_owner")
        data = bytearray()
        while len(data) <= cap:
            chunk = os.read(fd, min(4096, cap + 1 - len(data)))
            if not chunk:
                break
            data.extend(chunk)
        if len(data) > cap:
            raise FixedFailure("input_type_size")
        return bytes(data)
    finally:
        os.close(fd)

def free_bytes():
    info = os.statvfs(ROOT)
    return info.f_bavail * info.f_frsize

def check_directory(path, owner):
    info = path.lstat()
    if (not stat.S_ISDIR(info.st_mode) or info.st_uid != owner
            or path.resolve(strict=True) != path):
        raise FixedFailure("directory_identity")
    return info

def verify_sources():
    if sorted(item.name for item in PROJECT.parent.iterdir()) != sorted(SOURCE_HASHES):
        raise FixedFailure("project_input_catalog")
    observed = {}
    for name, expected in SOURCE_HASHES.items():
        value = digest(bounded_regular(PROJECT.parent / name, 65536))
        if value != expected:
            raise FixedFailure("source_identity")
        observed[name] = value
    for name in ("bin", "obj"):
        if os.path.lexists(PROJECT.parent / name):
            raise FixedFailure("project_output_present")
    return observed

def preflight():
    if Path.cwd() != ROOT or sys.platform != "darwin" or os.uname().machine != "arm64":
        raise FixedFailure("host_cwd_identity")
    if not all(hasattr(os, n) for n in ("waitid", "WNOWAIT", "WEXITED", "P_PID")):
        raise FixedFailure("wait_observer_unavailable")
    if os.path.lexists(LEAF):
        raise FixedFailure("private_leaf_present")
    for parent in (ROOT, ROOT / "target", ROOT / "target/wave27"):
        check_directory(parent, os.getuid())
    if free_bytes() < START_FREE:
        raise FixedFailure("start_disk")
    for key in ("DOTNET_ROOT", "DOTNET_ROOT_ARM64", "DOTNET_ROOT_X64",
                "DOTNET_MSBUILD_SDK_RESOLVER_SDKS_DIR",
                "DOTNET_MSBUILD_SDK_RESOLVER_SDKS_VER", "MSBuildSDKsPath",
                "MSBUILD_EXE_PATH", "DOTNET_CLI_HOME", "NUGET_PACKAGES",
                "DOTNET_STARTUP_HOOKS", "DOTNET_ADDITIONAL_DEPS"):
        if key in os.environ:
            raise FixedFailure("inherited_override")
    for parent in (PROJECT.parent, *PROJECT.parent.parents):
        for name in ("global.json", "Directory.Build.props", "Directory.Build.targets",
                     "Directory.Packages.props", "NuGet.Config", "nuget.config"):
            if os.path.lexists(parent / name):
                raise FixedFailure("inherited_configuration")
    if os.path.lexists(PROJECT.parent / "Properties/launchSettings.json"):
        raise FixedFailure("launch_settings_present")
    sources = verify_sources()
    if sorted(item.name for item in (SDK / "sdk").iterdir()) != ["9.0.200"]:
        raise FixedFailure("sdk_selection_metadata")
    for relative, (size, mode) in SDK_FILES.items():
        path = SDK / relative
        info = path.lstat()
        if (path.resolve(strict=True) != path or not stat.S_ISREG(info.st_mode)
                or info.st_uid != 0 or info.st_size != size
                or stat.S_IMODE(info.st_mode) != mode):
            raise FixedFailure("sdk_file_metadata")
    if not os.access(SDK / "dotnet", os.R_OK | os.X_OK):
        raise FixedFailure("sdk_executable_access")
    for relative in ("sdk/9.0.200", "host/fxr/9.0.2",
                     "shared/Microsoft.NETCore.App/9.0.2",
                     "packs/Microsoft.NETCore.App.Ref/9.0.2",
                     "packs/Microsoft.NETCore.App.Host.osx-arm64/9.0.2"):
        check_directory(SDK / relative, 0)
    metadata = {}
    for name, expected in METADATA_HASHES.items():
        value = digest(bounded_regular(SDK / "sdk/9.0.200" / name, 60000, 0))
        if value != expected:
            raise FixedFailure("sdk_metadata_identity")
        metadata[name] = value
    check_directory(Path("/Users/dominik/.nuget/packages"), os.getuid())
    if len(COMMAND.encode()) != 1483 or digest(COMMAND.encode()) != COMMAND_SHA:
        raise FixedFailure("command_identity")
    tokens = shlex.split(COMMAND.replace("\\\n", ""), posix=True)
    if (len(tokens) != 27 or tokens[0] != "env"
            or tokens[11:13] != ["/usr/local/share/dotnet/dotnet", "run"]
            or tokens[-2:] != ["--", "selftest"]):
        raise FixedFailure("command_tokens")
    assignments = dict(item.split("=", 1) for item in tokens[1:11])
    if len(assignments) != 10:
        raise FixedFailure("command_environment")
    env = os.environ.copy()
    env.update(assignments)
    inputs = {"source_pin": SOURCE_PIN, "sources": sources,
              "sdk_public_metadata": metadata, "sdk_file_metadata": SDK_FILES,
              "command_sha256": COMMAND_SHA, "command_bytes": 1483,
              "explicit_environment": assignments, "config_sha256": digest(CONFIG),
              "start_free": START_FREE, "stop_free": STOP_FREE,
              "rss_cap": RSS_CAP, "tree_cap": TREE_CAP, "log_cap": LOG_CAP,
              "child_seconds": CHILD_SECONDS, "cleanup_seconds": CLEANUP_SECONDS}
    return tokens[11:], env, inputs

class GroupObserver:
    def __init__(self):
        if ctypes.sizeof(BsdInfo) != 136 or ctypes.sizeof(TaskInfo) != 96:
            raise FixedFailure("process_abi")
        self.lib = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
        self.lib.proc_listpids.argtypes = (
            ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_int)
        self.lib.proc_listpids.restype = ctypes.c_int
        self.lib.proc_pidinfo.argtypes = (
            ctypes.c_int, ctypes.c_int, ctypes.c_uint64,
            ctypes.c_void_p, ctypes.c_int)
        self.lib.proc_pidinfo.restype = ctypes.c_int
        self.identities = {}

    def members(self, pgid):
        buf = (ctypes.c_int * (PID_CAP + 1))()
        ctypes.set_errno(0)
        count = self.lib.proc_listpids(2, pgid, buf, ctypes.sizeof(buf))
        if (count < 0 or (count == 0 and ctypes.get_errno() != 0)
                or count % 4 or count >= ctypes.sizeof(buf)):
            raise FixedFailure("group_list_bound")
        values = sorted({int(pid) for pid in buf[:count // 4] if pid > 0})
        if len(values) > PID_CAP:
            raise FixedFailure("group_member_cap")
        return values

    def info(self, pid, pgid):
        value = BsdInfo()
        count = self.lib.proc_pidinfo(pid, 3, 0, ctypes.byref(value), 136)
        if count != 136:
            try:
                if os.getpgid(pid) != pgid:
                    return None
            except ProcessLookupError:
                return None
            raise FixedFailure("process_identity_unreadable")
        if value.pid != pid or value.pgid != pgid or value.uid != os.getuid():
            raise FixedFailure("process_identity")
        identity = (int(value.start_sec), int(value.start_usec))
        if pid in self.identities and self.identities[pid] != identity:
            raise FixedFailure("process_identity_changed")
        self.identities[pid] = identity
        return value

    def sample(self, pgid):
        members = self.members(pgid)
        resident = 0
        for pid in members:
            identity = self.info(pid, pgid)
            if identity is None or identity.status == 5:  # SZOMB, not running.
                continue
            task = TaskInfo()
            count = self.lib.proc_pidinfo(pid, 4, 0, ctypes.byref(task), 96)
            if count != 96:
                again = self.info(pid, pgid)
                if again is None or again.status == 5:
                    continue
                raise FixedFailure("rss_unreadable")
            resident += int(task.resident)
        return members, resident

def tree_bytes(leaf_identity):
    info = check_directory(LEAF, os.getuid())
    if (info.st_dev, info.st_ino) != leaf_identity:
        raise FixedFailure("private_leaf_identity")
    total, entries = 0, 0
    def walk_error(_):
        raise FixedFailure("private_scan")
    for parent, dirs, files in os.walk(LEAF, followlinks=False, onerror=walk_error):
        if len(Path(parent).relative_to(LEAF).parts) > 32:
            raise FixedFailure("private_depth")
        for name in dirs + files:
            entries += 1
            if entries > ENTRY_CAP:
                raise FixedFailure("private_entry_cap")
            item = (Path(parent) / name).lstat()
            if (item.st_uid != os.getuid()
                    or not (stat.S_ISREG(item.st_mode) or stat.S_ISDIR(item.st_mode))):
                raise FixedFailure("private_entry_type")
            total += max(item.st_size, item.st_blocks * 512)
    return total

def run():
    first_error = None
    child = None
    pgid = None
    verified_group = False
    child_started_at = None
    cleanup_deadline = None
    reaped = False
    child_exit = None
    observer = None
    leaf_identity = None
    inputs = None
    input_hash = None
    samples = []
    logs = {}
    selector = selectors.DefaultSelector()
    cleanup = {"attempted": False, "term": False, "kill": False,
               "reaped": False, "group_empty": None, "errors": []}
    phase = "preflight"
    receipt_durable = False

    def latch(code, exc=None):
        nonlocal first_error
        if first_error is None:
            names = ((FixedFailure, "FixedFailure"), (OSError, "OSError"),
                     (ValueError, "ValueError"), (RuntimeError, "RuntimeError"),
                     (KeyboardInterrupt, "KeyboardInterrupt"), (SystemExit, "SystemExit"))
            kind = next((name for cls, name in names if type(exc) is cls), "Other")
            first_error = {"code": code, "class": kind if exc is not None else None}

    def catch(code, exc):
        if type(exc) is FixedFailure:
            latch(exc.args[0], exc)  # Only controller-owned fixed string literals.
        else:
            latch(code, exc)

    def cleanup_error(code):
        if code not in cleanup["errors"] and len(cleanup["errors"]) < 12:
            cleanup["errors"].append(code)

    def exclusive(name):
        fd = os.open(LEAF / name,
                     os.O_RDWR | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        if stat.S_IMODE(os.fstat(fd).st_mode) != 0o600:
            os.close(fd)
            raise FixedFailure("private_file_mode")
        return fd

    def write_all(fd, data):
        view = memoryview(data)
        while view:
            count = os.write(fd, view)
            if count <= 0:
                raise FixedFailure("private_write")
            view = view[count:]

    def ended():
        # No poll()/wait() before cleanup: WNOWAIT reserves the group-leader PID.
        return os.waitid(os.P_PID, child.pid,
                         os.WEXITED | os.WNOHANG | os.WNOWAIT) is not None

    def pump(timeout):
        for key, _ in selector.select(max(0.0, timeout)):
            name = key.data
            try:
                data = os.read(key.fileobj.fileno(), 4096)
            except BlockingIOError:
                continue
            if not data:
                selector.unregister(key.fileobj)
                key.fileobj.close()
                logs[name]["eof"] = True
                continue
            log = logs[name]
            log["seen"] += len(data)
            available = LOG_CAP - log["kept"]
            retained = data[:max(0, available)]
            if retained:
                write_all(log["fd"], retained)
                log["kept"] += len(retained)
            if len(data) > available:
                log["truncated"] = True
                latch(name + "_cap")

    def sample():
        members, resident = observer.sample(pgid)
        used = tree_bytes(leaf_identity)
        available = free_bytes()
        if len(samples) >= SAMPLE_CAP:
            raise FixedFailure("sample_cap")
        samples.append({"seconds": round(time.monotonic() - child_started_at, 6),
                        "members": members, "rss_bytes": resident,
                        "private_bytes": used, "free_bytes": available})
        if available <= STOP_FREE:
            raise FixedFailure("stop_disk")
        if resident > RSS_CAP:
            raise FixedFailure("rss_cap")
        if used + RECEIPT_CAP > TREE_CAP:
            raise FixedFailure("private_growth")
        return members

    def signal_group(sig):
        if not verified_group or reaped:
            cleanup_error("group_signal_ownership")
            return
        try:
            os.killpg(pgid, sig)
            cleanup["term" if sig == signal.SIGTERM else "kill"] = True
        except ProcessLookupError:
            pass
        except BaseException:
            cleanup_error("group_signal")

    def finish_group():
        nonlocal reaped, child_exit, cleanup_deadline
        if child is None:
            cleanup["group_empty"] = True
            return
        cleanup["attempted"] = True
        if cleanup_deadline is None:
            cleanup_deadline = time.monotonic() + CLEANUP_SECONDS
        if not verified_group:
            cleanup_error("group_unverified")
            # Only the unreaped direct Popen child is addressable in this branch.
            try:
                child.kill()
            except BaseException:
                cleanup_error("direct_kill")
        else:
            signal_group(signal.SIGTERM)
        kill_at = cleanup_deadline - 5.0
        if first_error is not None and first_error["code"] in (
                "stop_disk", "rss_cap", "private_growth", "stdout_cap", "stderr_cap"):
            kill_at = time.monotonic()
        next_sample = time.monotonic()
        while time.monotonic() < cleanup_deadline:
            now = time.monotonic()
            if not cleanup["kill"] and now >= kill_at:
                if verified_group:
                    signal_group(signal.SIGKILL)
                else:
                    try:
                        child.kill()
                        cleanup["kill"] = True
                    except BaseException:
                        cleanup_error("direct_kill")
            try:
                pump(min(0.1, cleanup_deadline - now))
            except BaseException as exc:
                catch("cleanup_capture", exc)
                cleanup_error("capture_drain")
            try:
                if now >= next_sample and verified_group:
                    next_sample = now + 1.0
                    sample()
                members = observer.members(pgid) if verified_group else []
                done = ended()
                if done and (not verified_group or not set(members) - {child.pid}):
                    # Send final KILL while the unreaped leader still reserves PGID.
                    if verified_group:
                        signal_group(signal.SIGKILL)
                    child_exit = child.wait(timeout=max(0.0, cleanup_deadline - time.monotonic()))
                    reaped = True
                    break
            except BaseException as exc:
                cleanup_error("group_observation")
                catch("cleanup_observation", exc)
                # Observation errors remove grace, not group ownership.
                kill_at = time.monotonic()
        if not reaped:
            if verified_group:
                signal_group(signal.SIGKILL)
            try:
                child_exit = child.wait(timeout=max(0.0, cleanup_deadline - time.monotonic()))
                reaped = True
            except BaseException:
                cleanup_error("direct_reap")
        cleanup["reaped"] = reaped
        # After reaping, never signal a potentially reused group number.
        try:
            os.killpg(pgid, 0)
            cleanup["group_empty"] = False
        except ProcessLookupError:
            cleanup["group_empty"] = True
        except BaseException:
            cleanup["group_empty"] = None
            cleanup_error("final_group_probe")
        if cleanup["group_empty"] is not True:
            cleanup_error("group_not_empty")
        if verified_group and observer is not None:
            try:
                if observer.members(pgid):
                    cleanup["group_empty"] = False
                    cleanup_error("final_group_members")
            except BaseException:
                cleanup["group_empty"] = None
                cleanup_error("final_group_members")

    try:
        argv, env, inputs = preflight()
        input_hash = digest(canonical(inputs))
        observer = GroupObserver()  # Future numeric observation only, no helper child.
        os.umask(0o077)  # This standalone controller retains 077 until exit.
        os.mkdir(LEAF, 0o700)  # Fails if any competing/existing leaf is present.
        leaf = check_directory(LEAF, os.getuid())
        if stat.S_IMODE(leaf.st_mode) != 0o700:
            raise FixedFailure("private_leaf_mode")
        leaf_identity = (leaf.st_dev, leaf.st_ino)
        for name in ("cli-home", "packages"):
            os.mkdir(LEAF / name, 0o700)
        fd = exclusive("nuget-offline.config")
        try:
            write_all(fd, CONFIG)
            os.fsync(fd)
        finally:
            os.close(fd)
        for name in ("stdout", "stderr"):
            logs[name] = {"fd": exclusive(name + ".log"), "seen": 0,
                          "kept": 0, "truncated": False, "eof": False}
        if free_bytes() < START_FREE:
            raise FixedFailure("start_disk")
        require_waitable_child()
        phase = "child"
        child_started_at = time.monotonic()
        child = subprocess.Popen(argv, cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                 bufsize=0, start_new_session=True, close_fds=True)
        pgid = child.pid
        verified_group = os.getpgid(pgid) == pgid and os.getsid(pgid) == pgid
        if not verified_group:
            raise FixedFailure("new_group_identity")
        for name, stream in (("stdout", child.stdout), ("stderr", child.stderr)):
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_READ, name)
        owner = observer.info(pgid, pgid)
        if owner is None:
            raise FixedFailure("new_group_owner_record")
        launch = canonical({"schema_version": 1, "input_sha256": input_hash,
                            "pid": child.pid, "pgid": pgid, "session": pgid,
                            "owner_uid": int(owner.uid),
                            "leader_start_sec": int(owner.start_sec),
                            "leader_start_usec": int(owner.start_usec),
                            "start_monotonic": child_started_at,
                            "child_deadline_monotonic": child_started_at + CHILD_SECONDS,
                            "grade_performed": False}) + b"\n"
        if len(launch) > 2048:
            raise FixedFailure("launch_receipt_cap")
        fd = exclusive("launch.json")
        try:
            write_all(fd, launch)
            os.fsync(fd)
        finally:
            os.close(fd)
        deadline = child_started_at + CHILD_SECONDS
        next_sample = time.monotonic()
        while first_error is None:
            now = time.monotonic()
            if now >= deadline:
                latch("child_timeout")
                break
            if now >= next_sample:
                sample()
                next_sample = time.monotonic() + 1.0
            pump(min(0.1, deadline - time.monotonic()))
            if ended():
                break
    except BaseException as exc:
        catch(phase + "_exception", exc)
    finally:
        phase = "cleanup"
        try:
            finish_group()
        except BaseException as exc:
            catch("cleanup_exception", exc)
            cleanup_error("cleanup_exception")
            # A final reserved-PID group KILL; the same deadline is not reset.
            if child is not None and not reaped:
                signal_group(signal.SIGKILL)
                try:
                    child_exit = child.wait(timeout=max(
                        0.0, (cleanup_deadline or time.monotonic()) - time.monotonic()))
                    reaped = True
                    cleanup["reaped"] = True
                except BaseException:
                    cleanup_error("direct_reap")
            cleanup["group_empty"] = None
        try:
            # Final bounded nonblocking drain after group cleanup, not a new grace.
            for _ in range(34):
                if not selector.get_map():
                    break
                pump(0.0)
        except BaseException as exc:
            catch("capture_finalize", exc)
        for key in list(selector.get_map().values()):
            try:
                selector.unregister(key.fileobj)
                key.fileobj.close()
            except BaseException:
                cleanup_error("pipe_close")
        try:
            selector.close()
        except BaseException:
            cleanup_error("selector_close")
        for log in logs.values():
            try:
                os.fsync(log["fd"])
            except BaseException as exc:
                catch("capture_fsync", exc)
            finally:
                try:
                    os.close(log["fd"])
                except BaseException:
                    cleanup_error("capture_close")

    # All serialization and grading are AFTER the cleanup finally above.
    if child_exit is not None and child_exit != 0:
        latch("child_nonzero")
    captured = {}
    post_sources = None
    try:
        if leaf_identity is None:
            raise FixedFailure("no_owned_receipt_workspace")
        for name, log in logs.items():
            data = bounded_regular(LEAF / (name + ".log"), LOG_CAP, os.getuid())
            captured[name] = data
            if len(data) != log["kept"]:
                latch("capture_identity")  # Keep actual bounded failed-output evidence.
        post_sources = verify_sources()
        tree = tree_bytes(leaf_identity)
        available = free_bytes()
        if available <= STOP_FREE:
            latch("stop_disk")
        if tree + RECEIPT_CAP > TREE_CAP:
            latch("private_growth")
        for name, log in logs.items():
            if not log["eof"]:
                latch(name + "_eof_unverified")
        elapsed = None if child_started_at is None else time.monotonic() - child_started_at
        receipt = {"schema_version": 1, "source_pin": SOURCE_PIN,
                   "command_sha256": COMMAND_SHA, "input_sha256": input_hash,
                   "inputs": inputs, "child_started": child is not None,
                   "child_pid": None if child is None else child.pid, "pgid": pgid,
                   "child_exit": child_exit, "elapsed_through_cleanup": elapsed,
                   "first_error": first_error, "cleanup": cleanup,
                   "post_sources": post_sources, "resource_samples": samples,
                   "final_private_bytes_before_receipt": tree, "final_free_bytes": available,
                   "outputs": {name: {"bytes": len(captured.get(name, b"")),
                              "sha256": digest(captured.get(name, b"")),
                              "bytes_observed": log["seen"], "bytes_kept_recorded": log["kept"],
                              "truncated": log["truncated"], "eof": log["eof"]}
                               for name, log in logs.items()},
                   "grade_performed": False}
        payload = canonical(receipt) + b"\n"
        if len(payload) > RECEIPT_CAP:
            raise FixedFailure("receipt_cap")
        fd = exclusive("result.json")
        try:
            write_all(fd, payload)
            os.fsync(fd)
        finally:
            os.close(fd)
        dir_fd = os.open(LEAF, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            os.fsync(dir_fd)
        finally:
            os.close(dir_fd)
        receipt_durable = True
    except BaseException as exc:
        catch("receipt_exception", exc)
        # No retry/overwrite and no grading. The preceding finally already
        # attempted group cleanup even if serialization/fsync now fails.
    ok = (receipt_durable and first_error is None and child is not None
          and child_exit == 0 and cleanup["reaped"]
          and cleanup["group_empty"] is True and not cleanup["errors"]
          and captured.get("stdout", b"").splitlines().count(b"selftest passed") == 1)
    # No exception text, raw output, source values, environment, or private path.
    status = "UNGRADABLE" if not receipt_durable else ("PASS" if ok else "FAIL")
    public = {"schema_version": 1, "status": status,
              "receipt_durable": receipt_durable, "input_sha256": input_hash,
              "first_error": first_error, "cleanup_errors": cleanup["errors"],
              "child_exit": child_exit, "reaped": cleanup["reaped"],
              "group_empty": cleanup["group_empty"]}
    try:
        print(json.dumps(public, sort_keys=True, separators=(",", ":")))
    except BaseException:
        return 2  # Durable private receipt remains; no raw exception is printed.
    return 0 if ok else 2

if __name__ == "__main__":
    sys.exit(run())
```

### Whole-byte and AST preservation proof

Removing exactly the helper text and the single installer call reconstructs
ALL 31,494 original controller bytes, SHA-256
3a3703e6fb7095f2cc3d152aa170c3b68c03acf688c9361f11517cac99078490.
A second independent structural inverse removes only the new FunctionDef
and its one call Expr from the corrected parsed tree; the complete AST dump,
excluding source-position attributes, equals the entire archived AST.
No other node, import, constant, source hash, environment assignment, command,
configuration, serializer, cleanup, resource/sample cap, absolute deadline,
success marker or phase operation is replaced.

Both fixed code literals are confined to the new helper. The ten-line
insertion and one call are the entire unified diff. The existing signal
import is reused; no controller/module import or candidate function/native
signal evaluation occurred in these checks. One signal installation request
and one exact verification are source definitions, not observed call counts.

All six managed source pins, SDK metadata and exact 1,483-byte command/
public offline configuration are preserved. The old 120/10-second deadlines,
9 GiB / 8.5 GiB / 8 GiB disk policy, resource/log/receipt bounds, WNOWAIT cleanup
before serialization, private ownership and fsync-before-marker grading
remain byte/AST-identical after the narrow inverse. Production 15s and all
five original mocks plus sixth stall case remain unchanged. Numeric status
preservation and default disposition are still source expectations, not
verified OS or managed runtime outcomes.

The signal seam fails before SDK launch: successful private setup may
already exist, so the existing finally still closes/captures bounded logs and
can record the fixed first failure. It does not fabricate a child exit or
selftest pass. It adds no rule to accept ECHILD: any unexpected remaining
wait/ownership error still follows the existing refusal/cleanup path.
The prior process-group escape/reuse, uninterruptible syscall, one-second
sampling / 8 GiB-floor, metadata-only SDK trust, output-persistence and real
Windows limitations remain applicable; this narrow fix does not resolve or
erase them. No old source/failure/unknown-cause record is reclassified.

AST/data/byte/hash checks only: controller never compiled/imported/evaluated;
no SDK, version, selftest, native/signal/wait probe, helper, HTTP/Driver/Cargo,
extra child or lane launched. Private output leaf/project bin/obj remain
absent; no config/dependency/installation/download was created. Only this
report is written. Source/project/product/helper/guide/main/accepted/status/
history and other reports remain untouched; no merge, contact, new task/WT/
worker/shell or push. Final documentation/hygiene/whitespace, source identity,
prefix/scope and immutable report commit readback are recorded below.

Final source checks passed for the **31,893-byte / 721-line** corrected
payload (final newline included), SHA-256
**a833a87be91e401d86617467d2aaf2ee946a49daf7e385ec5c7f9d0fac696f34**.
Both complete byte and independent AST inverses recovered all old bytes/nodes.
The helper contains exactly one signal installation and one identity
verification, with its one call preceding the sole Popen in the main path.
All original command/configuration/source/SDK/budget/receipt definitions
remain exact; all six current source/project files equal e31.

Initial Markdown/hygiene checks passed; whitespace checking flagged the
single-space blank context line in the default unified diff representation.
The report now archives the SAME two additions with zero context, **506
bytes**, SHA-256 **9a322818218edc571f1402e1d0fdc4caae710cea2cfe6a85c8b7f6a6c8826f44**. This is a representation-only static
correction; the corrected controller hash did not change and no runtime
failure/case was executed. A tool-orchestration JavaScript parse failure
occurred before the first correction script ran (zero command/file changes);
the corrected static orchestration then applied the representation change.
The full old/new payloads remain available above for exact context/reversal.
Final report-prefix/scope/docs/hygiene/whitespace and staged/commit readback
complete the immutable handoff.

No signal API/native wait, controller/function/import/main, SDK or test
invocation occurred; no private output paths were created. The validation
lane remains unacquired despite I07's release. Root and independent review
and a separate future runtime release are required; original/native Windows
gates and every preserved source/failure/unknown record remain unchanged.

Final static validation after the diff representation correction passed:
zero-context diff forward application and exact reversal each reconstruct the
full archived old/new payloads; independent AST inverse matches the entire
old tree. The two hunks contain only ten helper lines plus one call. Whole
prefix preservation, six complete e31 source blobs, original command/config
identity, one Popen and absent private outputs passed. Markdown links/layout,
tracked hygiene (960 files) and git diff --check all passed. Final staged
whitespace/scope and commit readback are part of this report-only handoff.
These are static checks; no candidate or signal/SDK/runtime observation is
added. Root's source/independent reviews and later release remain pending.

## 2026-10-03 — wave30_I08_disable_sdk_certificate_bootstrap

Project 891e7443-8dac-4c1b-897f-9e53cb59c7ee; existing worktree
ed9ac424-59f4-4520-905b-919aea3521eb and existing shell only.
Runtime is **HELD**. The complete **139,859-byte / 2,606-line** report at
**3ae7c2040163f128cb979f447bc7839494288e5c**, SHA-256
**48c9ee846b65b7bd5322dad6f944ab50476d0c11aa8d5df708f3086fe70a31b2**, remains the exact prefix.
The old 1,483-byte command, initial controller and SIGCHLD-corrected
controller remain dated historical text. This appendix supplies the
certificate-disabled command/controller for root and independent review
before any separately released execution; it replaces none of those bytes.

### Primary-source witnesses and bounded conclusion

I read the complete five retained primary-source bodies below, **29,149
bytes / 681 lines** total, including the certificate gate and PATH factory.
Those body reads are distinct from the subsequent size/hash identity
checks. The root receipt is
`planning/evidence/wave30-i08-sdk-first-run-root-source-review.json`
under the explicit project orchestrator directory
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
The bodies are retained alongside it in
`planning/evidence/wave30-i08-sdk-first-run-sources`.

The initial receipt read was **1,645 bytes**, SHA-256
**76e18c2cbcf14de7126918001bc8e729b7beb929c135dbf2f98c52544b953e06**.
It initially labeled 90e8 as a tree. The current root-corrected receipt
is **2,073 bytes**, SHA-256
**7bccf827ed7a583b2a661ffaf8aca490a2eaa520d7aa957522229650a5c2942e**: SDK tag **v9.0.200**, source commit
**90e8b202f25b7c2bf3b883d421ad5b1cb477e8b0**, source tree
**c322ba339d3adb62d542ea33c1f5b3d9935e7988**. Its pin_label_correction
preserves the initial receipt identity and states all source body bytes
are unchanged. The commit matches the earlier public installed SDK
.version metadata identity. I checked the retained receipt and all
five body hashes; I did not query the recorded remote API, invoke the
SDK or independently attest the root's commit/tree lookup.

| Retained primary body | Bytes / lines | SHA-256 |
| --- | --- | --- |
| Program.cs | 16466 / 368 | ba0c8927d8141cba0cd0397cc85471ee9fd823a1e5dc1600bc55ba483545f607 |
| DotnetFirstTimeUseConfigurer.cs | 5124 / 112 | 5188ea10dd70d67abeecb767d554739f4c2be6eb337e23e530e66d60a747f3c7 |
| OsxBashEnvironmentPath.cs | 2949 / 81 | 262e46e3e41726d505e24251414e9da26ee291dfbcfc1c666916ce3c33e074a6 |
| DotnetFirstRunConfiguration.cs | 1004 / 32 | cf0209c2c0b92e6e56135d6ebac83a3a7c79a8cb450cd9e059bcd42a77f63bf4 |
| EnvironmentPathFactory.cs | 3606 / 88 | ffaa6fef0ef73a5eecb632de37583753a535a22df8edd840bb5d0252b488eb11 |

SDK Program.cs:171 reads DOTNET_GENERATE_ASPNET_CERTIFICATE with
defaultValue: true. Program.cs:190–195 passes that value into the
first-run configuration; DotnetFirstRunConfiguration.cs:25 stores
GenerateAspNetCertificate. Program.cs:197–204 and :314–338 construct
and call the first-use configurer. DotnetFirstTimeUseConfigurer.cs:
77–82 invokes GenerateAspNetCoreDevelopmentCertificate and creates
the certificate sentinel only when CanGenerateAspNetCertificate is true.
At :102–107 that predicate requires enabled configuration and an
absent sentinel (or is false under the compile-time exclusion).
Explicit false therefore prevents this identified generator invocation
regardless of sentinel presence. That is source inference: neither
the SDK nor generator ran here, and no actual certificate/keychain/
store mutation is claimed.

DOTNET_SKIP_FIRST_TIME_EXPERIENCE is not read in these five selected
bodies. Its old assignment is retained; it is not the certificate
protection. Other first-use paths, including NuGet migration and the
first-use sentinel at DotnetFirstTimeUseConfigurer.cs:51–74, remain
defined. This opt-out does not prove that every SDK first-use write
or effect is disabled. The implementation of the generator itself
is outside this supplied set; no side-effect conclusion is drawn
from its construction in Program.cs.

Program.cs:181 defaults the native-installer flag to false; only the
InstallSuccess branch at :182–187 changes it. The exact argv is run.
EnvironmentPathFactory.cs:21 starts with DoNothingEnvironmentPath,
and its macOS branch at :47 requires that native-installer flag to
select OsxBashEnvironmentPath. Its paths.d write at :45 is consequently
outside this run selection. No unrelated PATH setting, configuration
or argument is changed. This is source selection, not a PATH experiment.

### Exact future command and binding

The sole command addition is
`DOTNET_GENERATE_ASPNET_CERTIFICATE=false` before the sole SDK launch.
The corrected command is **1,528 bytes** with no terminal newline,
SHA-256 **8a5b54debc10ba837af3b319e964724a9f47909ef8bf95c88edb2c9a7f44c1c3**. Parsing shell tokens as data after removing
backslash-newline continuations gives **28 tokens**: env, **11**
explicit environment assignments and the same **16** SDK argv tokens.
Removing the new **45-byte** line recovers the entire old command,
SHA-256 **6217c5e88aa76979bb0f91fae2faabcea2e9ffcfff722983f8058947c827ab7d**. Every previous assignment and argument
remains exact: telemetry/no-logo/workload flags, disabled server/
shared-compiler reuse, one CPU, private outputs, offline restore/
empty feeds/fallback and selftest mode. The production 15-second
client timeout in unchanged e31 source is preserved.

The controller's command hash/byte count, token count, assignment/
executable/return slices and receipt command_bytes now bind these
identities. The guard requires the exact eleven-key allowlist and
the new false value, failing with the existing fixed
command_environment code before Popen if the binding is wrong.
The complete pinned command hash binds all old values too.
The fixed token count, unique dictionary length and exact key set
reject missing, duplicate or extra assignments. env.update overrides
an inherited certificate setting with false; that same env goes to
the sole Popen. inputs records the new command hash/byte count and
explicit eleven-entry environment, so its derived input hash binds
the opt-out before launch. No other inherited-environment policy changes.

The shell fence's presentation newline is excluded from its identity;
COMMAND below contains exactly the **1,528 bytes**. All fences are DATA,
not executed/imported/evaluated/compiled code.

```sh
env \
  DOTNET_ROOT=/usr/local/share/dotnet \
  DOTNET_CLI_HOME=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/cli-home \
  NUGET_PACKAGES=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/packages \
  DOTNET_CLI_TELEMETRY_OPTOUT=1 \
  DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1 \
  DOTNET_GENERATE_ASPNET_CERTIFICATE=false \
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

Zero-context command diff: **140 bytes**, SHA-256 **3c063ec0e74347b034e47e969c049964b2cf510506256c8d82f7f1e19418bcc3**.
Forward and inverse text application recover both whole command strings.

```diff
--- i08-managed-command-historical
+++ i08-managed-command-certificate-disabled
@@ -6,0 +7 @@
+  DOTNET_GENERATE_ASPNET_CERTIFICATE=false \
```

### Complete corrected controller and minimal inverse

Complete prospective controller: **32,459 bytes / 730 lines**, including
final newline, SHA-256 **81366c6e29fb324e66206e2936b2faafc67d91cd6d9585910b6d1c9867ae95ae**. It retains the SIGCHLD-default
requirement: one standalone main-thread signal installation and exact
verification, fixed refusal before the sole Popen, and no restoration
of ignored SIGCHLD before join/reap. require_waitable_child and the
whole run body remain byte-identical to 3ae7c204.

The only changes from the **31,893-byte / 721-line** controller,
SHA-256 **a833a87be91e401d86617467d2aaf2ee946a49daf7e385ec5c7f9d0fac696f34**, are the command/environment bindings above.
Zero-context controller diff: **1770 bytes**, SHA-256 **79f566ce6800a121b6b6373edc67daaffd91e49e89b3256d807c3184a5fd44df**.
Reversing its listed text changes recovers all old bytes. An independent
AST inverse restores only COMMAND/COMMAND_SHA, the two command-byte
constants, token count, three slices and environment guard, recovering
the complete old AST without locations. Other top-level assignments,
all class bodies and all functions except preflight remain exact.
Configuration, source/SDK identities, failure codes, standalone entry,
limits, cleanup and grading remain unchanged.

```diff
--- i08-managed-controller-historical
+++ i08-managed-controller-certificate-disabled
@@ -20 +20 @@
-COMMAND_SHA = "6217c5e88aa76979bb0f91fae2faabcea2e9ffcfff722983f8058947c827ab7d"
+COMMAND_SHA = "8a5b54debc10ba837af3b319e964724a9f47909ef8bf95c88edb2c9a7f44c1c3"
@@ -26,0 +27 @@
+  DOTNET_GENERATE_ASPNET_CERTIFICATE=false \
@@ -228 +229 @@
-    if len(COMMAND.encode()) != 1483 or digest(COMMAND.encode()) != COMMAND_SHA:
+    if len(COMMAND.encode()) != 1528 or digest(COMMAND.encode()) != COMMAND_SHA:
@@ -231,2 +232,2 @@
-    if (len(tokens) != 27 or tokens[0] != "env"
-            or tokens[11:13] != ["/usr/local/share/dotnet/dotnet", "run"]
+    if (len(tokens) != 28 or tokens[0] != "env"
+            or tokens[12:14] != ["/usr/local/share/dotnet/dotnet", "run"]
@@ -235,2 +236,10 @@
-    assignments = dict(item.split("=", 1) for item in tokens[1:11])
-    if len(assignments) != 10:
+    assignments = dict(item.split("=", 1) for item in tokens[1:12])
+    if (len(assignments) != 11
+            or set(assignments) != {
+                "DOTNET_ROOT", "DOTNET_CLI_HOME", "NUGET_PACKAGES",
+                "DOTNET_CLI_TELEMETRY_OPTOUT", "DOTNET_SKIP_FIRST_TIME_EXPERIENCE",
+                "DOTNET_GENERATE_ASPNET_CERTIFICATE",
+                "DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE", "DOTNET_NOLOGO",
+                "DOTNET_PROCESSOR_COUNT", "DOTNET_CLI_USE_MSBUILD_SERVER",
+                "MSBUILDDISABLENODEREUSE"}
+            or assignments.get("DOTNET_GENERATE_ASPNET_CERTIFICATE") != "false"):
@@ -242 +251 @@
-              "command_sha256": COMMAND_SHA, "command_bytes": 1483,
+              "command_sha256": COMMAND_SHA, "command_bytes": 1528,
@@ -247 +256 @@
-    return tokens[11:], env, inputs
+    return tokens[12:], env, inputs
```

```python
# DESIGN ONLY: root must separately release this exact one-shot controller.
import ctypes
import hashlib
import json
import os
from pathlib import Path
import selectors
import shlex
import signal
import stat
import subprocess
import sys
import time

ROOT = Path("/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27")
LEAF = ROOT / "target/wave27/i08-managed"
PROJECT = ROOT / "windows/RiAuth.DeviceHost/RiAuth.DeviceHost.csproj"
SDK = Path("/usr/local/share/dotnet")
SOURCE_PIN = "e31fbee66f1038cfc2412e17497bbf07f83e1314"
COMMAND_SHA = "8a5b54debc10ba837af3b319e964724a9f47909ef8bf95c88edb2c9a7f44c1c3"
COMMAND = r"""env \
  DOTNET_ROOT=/usr/local/share/dotnet \
  DOTNET_CLI_HOME=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/cli-home \
  NUGET_PACKAGES=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/packages \
  DOTNET_CLI_TELEMETRY_OPTOUT=1 \
  DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1 \
  DOTNET_GENERATE_ASPNET_CERTIFICATE=false \
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
  -- selftest"""
CONFIG = b"""<configuration>
  <packageSources><clear /></packageSources>
  <fallbackPackageFolders><clear /></fallbackPackageFolders>
</configuration>
"""
SOURCE_HASHES = {
    "DeviceHost.cs": "c87516ac0323e4d009e6d438cfdf2b74918db3d7c0ad9ab4b9b67fd2367c48ee",
    "Program.cs": "a1fe254ebf65a2153fcf2a17728b4b1b2283ca3d6e991be92eff2bb26126a241",
    "SelfTest.cs": "69bd5f5031bfdb9b974cb2e8201e6f2823a3f32e924eb53f5a38392807e1b38e",
    "RiAuth.DeviceHost.csproj": "e5af8331053b1167d6af93ea9ad657a0983e694fa5bb71f8f003370fbed499ea",
    "WindowsLocalAccount.cs": "85641bd7baef27b419899ec76d9665c8d5cfb90b59641bafe5908524763f0d0c",
    "WindowsStateStore.cs": "9999747d0183650a109368feb0d11644ff90ad30c23b0d0be6b0ca3a930d9ce0",
}
METADATA_HASHES = {
    ".version": "835299a4fd4532244a680605ad2047c1d44d6f8a34834b1bb747fa74ca38e11a",
    "dotnet.runtimeconfig.json": "e90a7dd2984b3ade889585a0f593d097a9421f9a0d63d64ae8ffdd313e24daf0",
    "MSBuild.runtimeconfig.json": "e90a7dd2984b3ade889585a0f593d097a9421f9a0d63d64ae8ffdd313e24daf0",
    "Roslyn/bincore/csc.runtimeconfig.json": "e46be9b13a311147cbc2203dae66958ced66105c7369690ce4ab75fdbcebb561",
    "Microsoft.NETCoreSdk.BundledVersions.props": "887582b3c662e6de057c3e1a89daa500c8f9526f3418cc8f8fddef13a70989ee",
}
SDK_FILES = {
    "dotnet": (140128, 0o755),
    "sdk/9.0.200/dotnet.dll": (3394048, 0o644),
    "sdk/9.0.200/MSBuild.dll": (1035776, 0o644),
    "sdk/9.0.200/Roslyn/bincore/csc.dll": (132096, 0o644),
    "sdk/9.0.200/NuGet.targets": (74726, 0o644),
    "sdk/9.0.200/NuGet.Build.Tasks.dll": (233984, 0o644),
    "sdk/9.0.200/Sdks/Microsoft.NET.Sdk/Sdk/Sdk.props": (2432, 0o644),
    "sdk/9.0.200/Sdks/Microsoft.NET.Sdk/Sdk/Sdk.targets": (4767, 0o644),
    "host/fxr/9.0.2/libhostfxr.dylib": (401072, 0o755),
    "shared/Microsoft.NETCore.App/9.0.2/libhostpolicy.dylib": (420240, 0o755),
    "shared/Microsoft.NETCore.App/9.0.2/System.Private.CoreLib.dll": (16264704, 0o644),
    "packs/Microsoft.NETCore.App.Ref/9.0.2/ref/net9.0/System.Runtime.dll": (837120, 0o644),
    "packs/Microsoft.NETCore.App.Host.osx-arm64/9.0.2/runtimes/osx-arm64/native/apphost": (140896, 0o755),
}
GIB = 1024 ** 3
START_FREE = 9 * GIB
STOP_FREE = 17 * GIB // 2
RSS_CAP = 2 * GIB
TREE_CAP = 256 * 1024 ** 2
LOG_CAP = 64 * 1024
RECEIPT_CAP = 512 * 1024
ENTRY_CAP = 4096
PID_CAP = 256
SAMPLE_CAP = 144
CHILD_SECONDS = 120.0
CLEANUP_SECONDS = 10.0

class FixedFailure(Exception):
    pass

def require_waitable_child():
    try:
        signal.signal(signal.SIGCHLD, signal.SIG_DFL)
        if signal.getsignal(signal.SIGCHLD) is not signal.SIG_DFL:
            raise FixedFailure("sigchld_default_unverified")
    except FixedFailure:
        raise
    except BaseException:
        raise FixedFailure("sigchld_default_unavailable_or_refused") from None

# Numeric Darwin layouts from the installed public headers; names are padding.
class BsdInfo(ctypes.Structure):
    _fields_ = [(n, ctypes.c_uint32) for n in (
        "flags", "status", "xstatus", "pid", "ppid", "uid", "gid",
        "ruid", "rgid", "svuid", "svgid", "reserved")] + [
        ("unused_names", ctypes.c_byte * 48),
        ("nfiles", ctypes.c_uint32), ("pgid", ctypes.c_uint32),
        ("jobc", ctypes.c_uint32), ("tdev", ctypes.c_uint32),
        ("tpgid", ctypes.c_uint32), ("nice", ctypes.c_int32),
        ("start_sec", ctypes.c_uint64), ("start_usec", ctypes.c_uint64)]

class TaskInfo(ctypes.Structure):
    _fields_ = [(n, ctypes.c_uint64) for n in (
        "virtual", "resident", "total_user", "total_system",
        "threads_user", "threads_system")] + [(n, ctypes.c_int32) for n in (
        "policy", "faults", "pageins", "cow_faults", "messages_sent",
        "messages_received", "syscalls_mach", "syscalls_unix", "csw",
        "threadnum", "numrunning", "priority")]

def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      allow_nan=False).encode("ascii")

def digest(data):
    return hashlib.sha256(data).hexdigest()

def bounded_regular(path, cap, owner=None):
    # Reject symlink components before reading only approved public inputs/logs.
    if path.resolve(strict=True) != path:
        raise FixedFailure("input_symlink")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_size > cap:
            raise FixedFailure("input_type_size")
        if owner is not None and info.st_uid != owner:
            raise FixedFailure("input_owner")
        data = bytearray()
        while len(data) <= cap:
            chunk = os.read(fd, min(4096, cap + 1 - len(data)))
            if not chunk:
                break
            data.extend(chunk)
        if len(data) > cap:
            raise FixedFailure("input_type_size")
        return bytes(data)
    finally:
        os.close(fd)

def free_bytes():
    info = os.statvfs(ROOT)
    return info.f_bavail * info.f_frsize

def check_directory(path, owner):
    info = path.lstat()
    if (not stat.S_ISDIR(info.st_mode) or info.st_uid != owner
            or path.resolve(strict=True) != path):
        raise FixedFailure("directory_identity")
    return info

def verify_sources():
    if sorted(item.name for item in PROJECT.parent.iterdir()) != sorted(SOURCE_HASHES):
        raise FixedFailure("project_input_catalog")
    observed = {}
    for name, expected in SOURCE_HASHES.items():
        value = digest(bounded_regular(PROJECT.parent / name, 65536))
        if value != expected:
            raise FixedFailure("source_identity")
        observed[name] = value
    for name in ("bin", "obj"):
        if os.path.lexists(PROJECT.parent / name):
            raise FixedFailure("project_output_present")
    return observed

def preflight():
    if Path.cwd() != ROOT or sys.platform != "darwin" or os.uname().machine != "arm64":
        raise FixedFailure("host_cwd_identity")
    if not all(hasattr(os, n) for n in ("waitid", "WNOWAIT", "WEXITED", "P_PID")):
        raise FixedFailure("wait_observer_unavailable")
    if os.path.lexists(LEAF):
        raise FixedFailure("private_leaf_present")
    for parent in (ROOT, ROOT / "target", ROOT / "target/wave27"):
        check_directory(parent, os.getuid())
    if free_bytes() < START_FREE:
        raise FixedFailure("start_disk")
    for key in ("DOTNET_ROOT", "DOTNET_ROOT_ARM64", "DOTNET_ROOT_X64",
                "DOTNET_MSBUILD_SDK_RESOLVER_SDKS_DIR",
                "DOTNET_MSBUILD_SDK_RESOLVER_SDKS_VER", "MSBuildSDKsPath",
                "MSBUILD_EXE_PATH", "DOTNET_CLI_HOME", "NUGET_PACKAGES",
                "DOTNET_STARTUP_HOOKS", "DOTNET_ADDITIONAL_DEPS"):
        if key in os.environ:
            raise FixedFailure("inherited_override")
    for parent in (PROJECT.parent, *PROJECT.parent.parents):
        for name in ("global.json", "Directory.Build.props", "Directory.Build.targets",
                     "Directory.Packages.props", "NuGet.Config", "nuget.config"):
            if os.path.lexists(parent / name):
                raise FixedFailure("inherited_configuration")
    if os.path.lexists(PROJECT.parent / "Properties/launchSettings.json"):
        raise FixedFailure("launch_settings_present")
    sources = verify_sources()
    if sorted(item.name for item in (SDK / "sdk").iterdir()) != ["9.0.200"]:
        raise FixedFailure("sdk_selection_metadata")
    for relative, (size, mode) in SDK_FILES.items():
        path = SDK / relative
        info = path.lstat()
        if (path.resolve(strict=True) != path or not stat.S_ISREG(info.st_mode)
                or info.st_uid != 0 or info.st_size != size
                or stat.S_IMODE(info.st_mode) != mode):
            raise FixedFailure("sdk_file_metadata")
    if not os.access(SDK / "dotnet", os.R_OK | os.X_OK):
        raise FixedFailure("sdk_executable_access")
    for relative in ("sdk/9.0.200", "host/fxr/9.0.2",
                     "shared/Microsoft.NETCore.App/9.0.2",
                     "packs/Microsoft.NETCore.App.Ref/9.0.2",
                     "packs/Microsoft.NETCore.App.Host.osx-arm64/9.0.2"):
        check_directory(SDK / relative, 0)
    metadata = {}
    for name, expected in METADATA_HASHES.items():
        value = digest(bounded_regular(SDK / "sdk/9.0.200" / name, 60000, 0))
        if value != expected:
            raise FixedFailure("sdk_metadata_identity")
        metadata[name] = value
    check_directory(Path("/Users/dominik/.nuget/packages"), os.getuid())
    if len(COMMAND.encode()) != 1528 or digest(COMMAND.encode()) != COMMAND_SHA:
        raise FixedFailure("command_identity")
    tokens = shlex.split(COMMAND.replace("\\\n", ""), posix=True)
    if (len(tokens) != 28 or tokens[0] != "env"
            or tokens[12:14] != ["/usr/local/share/dotnet/dotnet", "run"]
            or tokens[-2:] != ["--", "selftest"]):
        raise FixedFailure("command_tokens")
    assignments = dict(item.split("=", 1) for item in tokens[1:12])
    if (len(assignments) != 11
            or set(assignments) != {
                "DOTNET_ROOT", "DOTNET_CLI_HOME", "NUGET_PACKAGES",
                "DOTNET_CLI_TELEMETRY_OPTOUT", "DOTNET_SKIP_FIRST_TIME_EXPERIENCE",
                "DOTNET_GENERATE_ASPNET_CERTIFICATE",
                "DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE", "DOTNET_NOLOGO",
                "DOTNET_PROCESSOR_COUNT", "DOTNET_CLI_USE_MSBUILD_SERVER",
                "MSBUILDDISABLENODEREUSE"}
            or assignments.get("DOTNET_GENERATE_ASPNET_CERTIFICATE") != "false"):
        raise FixedFailure("command_environment")
    env = os.environ.copy()
    env.update(assignments)
    inputs = {"source_pin": SOURCE_PIN, "sources": sources,
              "sdk_public_metadata": metadata, "sdk_file_metadata": SDK_FILES,
              "command_sha256": COMMAND_SHA, "command_bytes": 1528,
              "explicit_environment": assignments, "config_sha256": digest(CONFIG),
              "start_free": START_FREE, "stop_free": STOP_FREE,
              "rss_cap": RSS_CAP, "tree_cap": TREE_CAP, "log_cap": LOG_CAP,
              "child_seconds": CHILD_SECONDS, "cleanup_seconds": CLEANUP_SECONDS}
    return tokens[12:], env, inputs

class GroupObserver:
    def __init__(self):
        if ctypes.sizeof(BsdInfo) != 136 or ctypes.sizeof(TaskInfo) != 96:
            raise FixedFailure("process_abi")
        self.lib = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
        self.lib.proc_listpids.argtypes = (
            ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_int)
        self.lib.proc_listpids.restype = ctypes.c_int
        self.lib.proc_pidinfo.argtypes = (
            ctypes.c_int, ctypes.c_int, ctypes.c_uint64,
            ctypes.c_void_p, ctypes.c_int)
        self.lib.proc_pidinfo.restype = ctypes.c_int
        self.identities = {}

    def members(self, pgid):
        buf = (ctypes.c_int * (PID_CAP + 1))()
        ctypes.set_errno(0)
        count = self.lib.proc_listpids(2, pgid, buf, ctypes.sizeof(buf))
        if (count < 0 or (count == 0 and ctypes.get_errno() != 0)
                or count % 4 or count >= ctypes.sizeof(buf)):
            raise FixedFailure("group_list_bound")
        values = sorted({int(pid) for pid in buf[:count // 4] if pid > 0})
        if len(values) > PID_CAP:
            raise FixedFailure("group_member_cap")
        return values

    def info(self, pid, pgid):
        value = BsdInfo()
        count = self.lib.proc_pidinfo(pid, 3, 0, ctypes.byref(value), 136)
        if count != 136:
            try:
                if os.getpgid(pid) != pgid:
                    return None
            except ProcessLookupError:
                return None
            raise FixedFailure("process_identity_unreadable")
        if value.pid != pid or value.pgid != pgid or value.uid != os.getuid():
            raise FixedFailure("process_identity")
        identity = (int(value.start_sec), int(value.start_usec))
        if pid in self.identities and self.identities[pid] != identity:
            raise FixedFailure("process_identity_changed")
        self.identities[pid] = identity
        return value

    def sample(self, pgid):
        members = self.members(pgid)
        resident = 0
        for pid in members:
            identity = self.info(pid, pgid)
            if identity is None or identity.status == 5:  # SZOMB, not running.
                continue
            task = TaskInfo()
            count = self.lib.proc_pidinfo(pid, 4, 0, ctypes.byref(task), 96)
            if count != 96:
                again = self.info(pid, pgid)
                if again is None or again.status == 5:
                    continue
                raise FixedFailure("rss_unreadable")
            resident += int(task.resident)
        return members, resident

def tree_bytes(leaf_identity):
    info = check_directory(LEAF, os.getuid())
    if (info.st_dev, info.st_ino) != leaf_identity:
        raise FixedFailure("private_leaf_identity")
    total, entries = 0, 0
    def walk_error(_):
        raise FixedFailure("private_scan")
    for parent, dirs, files in os.walk(LEAF, followlinks=False, onerror=walk_error):
        if len(Path(parent).relative_to(LEAF).parts) > 32:
            raise FixedFailure("private_depth")
        for name in dirs + files:
            entries += 1
            if entries > ENTRY_CAP:
                raise FixedFailure("private_entry_cap")
            item = (Path(parent) / name).lstat()
            if (item.st_uid != os.getuid()
                    or not (stat.S_ISREG(item.st_mode) or stat.S_ISDIR(item.st_mode))):
                raise FixedFailure("private_entry_type")
            total += max(item.st_size, item.st_blocks * 512)
    return total

def run():
    first_error = None
    child = None
    pgid = None
    verified_group = False
    child_started_at = None
    cleanup_deadline = None
    reaped = False
    child_exit = None
    observer = None
    leaf_identity = None
    inputs = None
    input_hash = None
    samples = []
    logs = {}
    selector = selectors.DefaultSelector()
    cleanup = {"attempted": False, "term": False, "kill": False,
               "reaped": False, "group_empty": None, "errors": []}
    phase = "preflight"
    receipt_durable = False

    def latch(code, exc=None):
        nonlocal first_error
        if first_error is None:
            names = ((FixedFailure, "FixedFailure"), (OSError, "OSError"),
                     (ValueError, "ValueError"), (RuntimeError, "RuntimeError"),
                     (KeyboardInterrupt, "KeyboardInterrupt"), (SystemExit, "SystemExit"))
            kind = next((name for cls, name in names if type(exc) is cls), "Other")
            first_error = {"code": code, "class": kind if exc is not None else None}

    def catch(code, exc):
        if type(exc) is FixedFailure:
            latch(exc.args[0], exc)  # Only controller-owned fixed string literals.
        else:
            latch(code, exc)

    def cleanup_error(code):
        if code not in cleanup["errors"] and len(cleanup["errors"]) < 12:
            cleanup["errors"].append(code)

    def exclusive(name):
        fd = os.open(LEAF / name,
                     os.O_RDWR | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        if stat.S_IMODE(os.fstat(fd).st_mode) != 0o600:
            os.close(fd)
            raise FixedFailure("private_file_mode")
        return fd

    def write_all(fd, data):
        view = memoryview(data)
        while view:
            count = os.write(fd, view)
            if count <= 0:
                raise FixedFailure("private_write")
            view = view[count:]

    def ended():
        # No poll()/wait() before cleanup: WNOWAIT reserves the group-leader PID.
        return os.waitid(os.P_PID, child.pid,
                         os.WEXITED | os.WNOHANG | os.WNOWAIT) is not None

    def pump(timeout):
        for key, _ in selector.select(max(0.0, timeout)):
            name = key.data
            try:
                data = os.read(key.fileobj.fileno(), 4096)
            except BlockingIOError:
                continue
            if not data:
                selector.unregister(key.fileobj)
                key.fileobj.close()
                logs[name]["eof"] = True
                continue
            log = logs[name]
            log["seen"] += len(data)
            available = LOG_CAP - log["kept"]
            retained = data[:max(0, available)]
            if retained:
                write_all(log["fd"], retained)
                log["kept"] += len(retained)
            if len(data) > available:
                log["truncated"] = True
                latch(name + "_cap")

    def sample():
        members, resident = observer.sample(pgid)
        used = tree_bytes(leaf_identity)
        available = free_bytes()
        if len(samples) >= SAMPLE_CAP:
            raise FixedFailure("sample_cap")
        samples.append({"seconds": round(time.monotonic() - child_started_at, 6),
                        "members": members, "rss_bytes": resident,
                        "private_bytes": used, "free_bytes": available})
        if available <= STOP_FREE:
            raise FixedFailure("stop_disk")
        if resident > RSS_CAP:
            raise FixedFailure("rss_cap")
        if used + RECEIPT_CAP > TREE_CAP:
            raise FixedFailure("private_growth")
        return members

    def signal_group(sig):
        if not verified_group or reaped:
            cleanup_error("group_signal_ownership")
            return
        try:
            os.killpg(pgid, sig)
            cleanup["term" if sig == signal.SIGTERM else "kill"] = True
        except ProcessLookupError:
            pass
        except BaseException:
            cleanup_error("group_signal")

    def finish_group():
        nonlocal reaped, child_exit, cleanup_deadline
        if child is None:
            cleanup["group_empty"] = True
            return
        cleanup["attempted"] = True
        if cleanup_deadline is None:
            cleanup_deadline = time.monotonic() + CLEANUP_SECONDS
        if not verified_group:
            cleanup_error("group_unverified")
            # Only the unreaped direct Popen child is addressable in this branch.
            try:
                child.kill()
            except BaseException:
                cleanup_error("direct_kill")
        else:
            signal_group(signal.SIGTERM)
        kill_at = cleanup_deadline - 5.0
        if first_error is not None and first_error["code"] in (
                "stop_disk", "rss_cap", "private_growth", "stdout_cap", "stderr_cap"):
            kill_at = time.monotonic()
        next_sample = time.monotonic()
        while time.monotonic() < cleanup_deadline:
            now = time.monotonic()
            if not cleanup["kill"] and now >= kill_at:
                if verified_group:
                    signal_group(signal.SIGKILL)
                else:
                    try:
                        child.kill()
                        cleanup["kill"] = True
                    except BaseException:
                        cleanup_error("direct_kill")
            try:
                pump(min(0.1, cleanup_deadline - now))
            except BaseException as exc:
                catch("cleanup_capture", exc)
                cleanup_error("capture_drain")
            try:
                if now >= next_sample and verified_group:
                    next_sample = now + 1.0
                    sample()
                members = observer.members(pgid) if verified_group else []
                done = ended()
                if done and (not verified_group or not set(members) - {child.pid}):
                    # Send final KILL while the unreaped leader still reserves PGID.
                    if verified_group:
                        signal_group(signal.SIGKILL)
                    child_exit = child.wait(timeout=max(0.0, cleanup_deadline - time.monotonic()))
                    reaped = True
                    break
            except BaseException as exc:
                cleanup_error("group_observation")
                catch("cleanup_observation", exc)
                # Observation errors remove grace, not group ownership.
                kill_at = time.monotonic()
        if not reaped:
            if verified_group:
                signal_group(signal.SIGKILL)
            try:
                child_exit = child.wait(timeout=max(0.0, cleanup_deadline - time.monotonic()))
                reaped = True
            except BaseException:
                cleanup_error("direct_reap")
        cleanup["reaped"] = reaped
        # After reaping, never signal a potentially reused group number.
        try:
            os.killpg(pgid, 0)
            cleanup["group_empty"] = False
        except ProcessLookupError:
            cleanup["group_empty"] = True
        except BaseException:
            cleanup["group_empty"] = None
            cleanup_error("final_group_probe")
        if cleanup["group_empty"] is not True:
            cleanup_error("group_not_empty")
        if verified_group and observer is not None:
            try:
                if observer.members(pgid):
                    cleanup["group_empty"] = False
                    cleanup_error("final_group_members")
            except BaseException:
                cleanup["group_empty"] = None
                cleanup_error("final_group_members")

    try:
        argv, env, inputs = preflight()
        input_hash = digest(canonical(inputs))
        observer = GroupObserver()  # Future numeric observation only, no helper child.
        os.umask(0o077)  # This standalone controller retains 077 until exit.
        os.mkdir(LEAF, 0o700)  # Fails if any competing/existing leaf is present.
        leaf = check_directory(LEAF, os.getuid())
        if stat.S_IMODE(leaf.st_mode) != 0o700:
            raise FixedFailure("private_leaf_mode")
        leaf_identity = (leaf.st_dev, leaf.st_ino)
        for name in ("cli-home", "packages"):
            os.mkdir(LEAF / name, 0o700)
        fd = exclusive("nuget-offline.config")
        try:
            write_all(fd, CONFIG)
            os.fsync(fd)
        finally:
            os.close(fd)
        for name in ("stdout", "stderr"):
            logs[name] = {"fd": exclusive(name + ".log"), "seen": 0,
                          "kept": 0, "truncated": False, "eof": False}
        if free_bytes() < START_FREE:
            raise FixedFailure("start_disk")
        require_waitable_child()
        phase = "child"
        child_started_at = time.monotonic()
        child = subprocess.Popen(argv, cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                 bufsize=0, start_new_session=True, close_fds=True)
        pgid = child.pid
        verified_group = os.getpgid(pgid) == pgid and os.getsid(pgid) == pgid
        if not verified_group:
            raise FixedFailure("new_group_identity")
        for name, stream in (("stdout", child.stdout), ("stderr", child.stderr)):
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_READ, name)
        owner = observer.info(pgid, pgid)
        if owner is None:
            raise FixedFailure("new_group_owner_record")
        launch = canonical({"schema_version": 1, "input_sha256": input_hash,
                            "pid": child.pid, "pgid": pgid, "session": pgid,
                            "owner_uid": int(owner.uid),
                            "leader_start_sec": int(owner.start_sec),
                            "leader_start_usec": int(owner.start_usec),
                            "start_monotonic": child_started_at,
                            "child_deadline_monotonic": child_started_at + CHILD_SECONDS,
                            "grade_performed": False}) + b"\n"
        if len(launch) > 2048:
            raise FixedFailure("launch_receipt_cap")
        fd = exclusive("launch.json")
        try:
            write_all(fd, launch)
            os.fsync(fd)
        finally:
            os.close(fd)
        deadline = child_started_at + CHILD_SECONDS
        next_sample = time.monotonic()
        while first_error is None:
            now = time.monotonic()
            if now >= deadline:
                latch("child_timeout")
                break
            if now >= next_sample:
                sample()
                next_sample = time.monotonic() + 1.0
            pump(min(0.1, deadline - time.monotonic()))
            if ended():
                break
    except BaseException as exc:
        catch(phase + "_exception", exc)
    finally:
        phase = "cleanup"
        try:
            finish_group()
        except BaseException as exc:
            catch("cleanup_exception", exc)
            cleanup_error("cleanup_exception")
            # A final reserved-PID group KILL; the same deadline is not reset.
            if child is not None and not reaped:
                signal_group(signal.SIGKILL)
                try:
                    child_exit = child.wait(timeout=max(
                        0.0, (cleanup_deadline or time.monotonic()) - time.monotonic()))
                    reaped = True
                    cleanup["reaped"] = True
                except BaseException:
                    cleanup_error("direct_reap")
            cleanup["group_empty"] = None
        try:
            # Final bounded nonblocking drain after group cleanup, not a new grace.
            for _ in range(34):
                if not selector.get_map():
                    break
                pump(0.0)
        except BaseException as exc:
            catch("capture_finalize", exc)
        for key in list(selector.get_map().values()):
            try:
                selector.unregister(key.fileobj)
                key.fileobj.close()
            except BaseException:
                cleanup_error("pipe_close")
        try:
            selector.close()
        except BaseException:
            cleanup_error("selector_close")
        for log in logs.values():
            try:
                os.fsync(log["fd"])
            except BaseException as exc:
                catch("capture_fsync", exc)
            finally:
                try:
                    os.close(log["fd"])
                except BaseException:
                    cleanup_error("capture_close")

    # All serialization and grading are AFTER the cleanup finally above.
    if child_exit is not None and child_exit != 0:
        latch("child_nonzero")
    captured = {}
    post_sources = None
    try:
        if leaf_identity is None:
            raise FixedFailure("no_owned_receipt_workspace")
        for name, log in logs.items():
            data = bounded_regular(LEAF / (name + ".log"), LOG_CAP, os.getuid())
            captured[name] = data
            if len(data) != log["kept"]:
                latch("capture_identity")  # Keep actual bounded failed-output evidence.
        post_sources = verify_sources()
        tree = tree_bytes(leaf_identity)
        available = free_bytes()
        if available <= STOP_FREE:
            latch("stop_disk")
        if tree + RECEIPT_CAP > TREE_CAP:
            latch("private_growth")
        for name, log in logs.items():
            if not log["eof"]:
                latch(name + "_eof_unverified")
        elapsed = None if child_started_at is None else time.monotonic() - child_started_at
        receipt = {"schema_version": 1, "source_pin": SOURCE_PIN,
                   "command_sha256": COMMAND_SHA, "input_sha256": input_hash,
                   "inputs": inputs, "child_started": child is not None,
                   "child_pid": None if child is None else child.pid, "pgid": pgid,
                   "child_exit": child_exit, "elapsed_through_cleanup": elapsed,
                   "first_error": first_error, "cleanup": cleanup,
                   "post_sources": post_sources, "resource_samples": samples,
                   "final_private_bytes_before_receipt": tree, "final_free_bytes": available,
                   "outputs": {name: {"bytes": len(captured.get(name, b"")),
                              "sha256": digest(captured.get(name, b"")),
                              "bytes_observed": log["seen"], "bytes_kept_recorded": log["kept"],
                              "truncated": log["truncated"], "eof": log["eof"]}
                               for name, log in logs.items()},
                   "grade_performed": False}
        payload = canonical(receipt) + b"\n"
        if len(payload) > RECEIPT_CAP:
            raise FixedFailure("receipt_cap")
        fd = exclusive("result.json")
        try:
            write_all(fd, payload)
            os.fsync(fd)
        finally:
            os.close(fd)
        dir_fd = os.open(LEAF, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            os.fsync(dir_fd)
        finally:
            os.close(dir_fd)
        receipt_durable = True
    except BaseException as exc:
        catch("receipt_exception", exc)
        # No retry/overwrite and no grading. The preceding finally already
        # attempted group cleanup even if serialization/fsync now fails.
    ok = (receipt_durable and first_error is None and child is not None
          and child_exit == 0 and cleanup["reaped"]
          and cleanup["group_empty"] is True and not cleanup["errors"]
          and captured.get("stdout", b"").splitlines().count(b"selftest passed") == 1)
    # No exception text, raw output, source values, environment, or private path.
    status = "UNGRADABLE" if not receipt_durable else ("PASS" if ok else "FAIL")
    public = {"schema_version": 1, "status": status,
              "receipt_durable": receipt_durable, "input_sha256": input_hash,
              "first_error": first_error, "cleanup_errors": cleanup["errors"],
              "child_exit": child_exit, "reaped": cleanup["reaped"],
              "group_empty": cleanup["group_empty"]}
    try:
        print(json.dumps(public, sort_keys=True, separators=(",", ":")))
    except BaseException:
        return 2  # Durable private receipt remains; no raw exception is printed.
    return 0 if ok else 2

if __name__ == "__main__":
    sys.exit(run())
```

### Actual static checks, corrections and retained limits

Complete byte and independent structural AST inverses passed. Both
zero-context diffs applied forward/reverse reconstruct whole strings.
AST parsing passed; this is not compilation/evaluation/import of the
candidate. The archived COMMAND equals the shell data and hash/count.
All eleven keys and the new false predicate were checked as AST/data;
all ten previous values and sixteen argv tokens are exact. There is
one Popen. The same signal-default helper/call precedes that Popen;
all original class/function bodies except preflight remain byte/AST
identical. This is source identity/control-flow evidence, not a signal
or native-wait experiment.

Two initial static validator attempts failed before any append/file
write: the first structural inverse incorrectly required every tokens
access to be a slice (tokens[0] is an index); the diagnostic then
assumed every slice bound was a Constant (tokens[-2:] uses UnaryOp).
The corrected validator changes only the three designated positive
constant slices and leaves unrelated indexes/negative bounds intact.
A third preparation check stopped before any write when its exact
initial receipt size assertion encountered the root's newly corrected
2,073-byte commit/tree-label receipt. I reread that public receipt,
recorded both identities and verified that all five bodies retain
their original hashes. The candidate itself did not change in these
corrections; these failures were static preparation checks, not
SDK/runtime results.

All six current project files equal their whole e31fbee66f1038cfc2412e17497bbf07f83e1314
blobs and retained hashes: DeviceHost.cs, Program.cs, SelfTest.cs,
RiAuth.DeviceHost.csproj, WindowsLocalAccount.cs and WindowsStateStore.cs.
The five old mocks and sixth stall fixture, production 15-second
deadline and all product/security semantics remain unchanged.
Private i08-managed and project bin/obj remain absent; no output,
config/cache/dependency path was created.

The 120-second child / 10-second cleanup deadlines, one-second
samples, 2-GiB RSS / 256-MiB growth limits, 9-GiB start / 8.5-GiB stop
margin over the 8-GiB floor, 64-KiB cap per stream, WNOWAIT reservation,
TERM/KILL/reap ordering, first error and separate cleanup status are
exact. Bounded full-output retention, numeric exit/elapsed/input hash/
resource samples, owned reap/group-empty facts and private fsync
still precede grading the exact selftest marker. Cleanup is still in
finally before serialization; there is no retry or second child.
These are prospective controls, not results: sampled limits are not
continuous peaks/quotas, escaped process groups and blocking native
operations retain their stated limits, and SDK metadata is not binary
attestation or a network sandbox.

All earlier source/preparation failures and unknowns remain in the
preserved prefix. No actual old certificate mutation or new SDK pass
is inferred. No SDK/version/compiler/selftest/native/keychain/
certificate/store, controller/function/main/helper/import, network/
installation/download/browser/Driver/service/runtime invocation
occurred. No lane was taken or released; no process/deletion probe
or new output path was used. Only this report is written; sources,
project/config/helper, other reports, tasks/status, main/accepted
and history remain unchanged. Root must review this whole controller
and independent report before a separate one-shot runtime release.
Native signed Windows inputs and original lifecycle gates remain
separate. Final docs/hygiene/whitespace/scope and immutable commit
readback are recorded below.

Final static checks for this appendix passed: complete command/controller
archives, pinned byte/hash/line/token/environment identities, whole-byte
and independent AST inverses, zero-context diff forward/reverse
application, unchanged sixteen SDK argv tokens, exact eleven-key guard
and false certificate binding. The SIGCHLD-default helper and whole run
body are preserved; one Popen remains. All six e31 source blobs remain
exact and private/project output paths are absent. The full 139,859-byte
3ae7c204 report prefix is unchanged, and Git scope contains only this
report with no untracked files or staged unrelated changes.

python3 scripts/check-docs.py passed (Markdown links/build-directory
layout); python3 scripts/check-repo-hygiene.py passed (960 tracked files);
git diff --check passed. These are source/data/document checks only.
The final append and staged scope/whitespace are checked before the
report-only commit and immutable object readback. The complete corrected
controller remains UNEXECUTED; runtime is HELD for root and independent
review and later separate release. No certificate or SDK mutation/pass
is claimed, and no original Windows gate is closed by this report.

## 2026-10-03 — wave30_I08_final_bootstrap_deadline_composition

Project **891e7443-8dac-4c1b-897f-9e53cb59c7ee**, existing worktree
**ed9ac424-59f4-4520-905b-919aea3521eb** and existing shell only.
This is the exact two-correction source/design composition authorized
after root review of 7569146, the independent bootstrap finding at
**5ec4c70cfeff5e9cadd2111d64ff10178a18a6f1** and terminal-clock finding
at **ef67fa761bb18f7a795675006c970ff8c1734d4e**.
The whole **188,549-byte / 3,622-line** report at
**75691462bae1b3298418025a8b1f004cedbf4187**, SHA-256
**a568cfff64f180a34dafa4ca4e0db047fddb7585f9abf53d3a481bc38fc7402a**, remains the exact byte prefix. Every old command,
controller, failure, corrected metadata label, source limit and
historical check remains dated text, unchanged.

**Runtime remains HELD.** This append archives a complete prospective
command/controller, not an execution or a release. Root and Sol3 must
independently review the complete immutable composition before any
separately released one-shot managed selftest. No lane was acquired;
the separately completed/reaped/released root memory lane supplies no
managed SDK/selftest, certificate/workload or native Windows evidence.

### Source witnesses, reads and identity boundaries

The root-private retained primary sources and receipts below were
read as source/JSON DATA, not executed. They are under
`/Users/dominik/.local/share/riwork/orchestrators/projects/891e7443-8dac-4c1b-897f-9e53cb59c7ee/planning/evidence`.
I fully reread **Program.cs (368 lines)** and
**WorkloadIntegrityChecker.cs (47 lines)**, **18,509 bytes / 415 lines**
total, and all three finite retained JSON receipts. I also reread the
entire immutable **32,459-byte / 730-line** 756 controller in three
bounded complete displays. Those full-body reads are distinct from
hash checks, parsed AST comparisons and selected independent-report
F4/F2 sections. No claim of a fresh SDK/native/remote query is made.

| Retained witness, relative to planning/evidence | Bytes / lines | SHA-256 |
| --- | --- | --- |
| wave30-i08-sdk-first-run-root-source-review.json | 2073 / 52 | 7bccf827ed7a583b2a661ffaf8aca490a2eaa520d7aa957522229650a5c2942e |
| wave30-i08-sdk-first-run-root-source-review-initial.json | 1645 / 44 | 76e18c2cbcf14de7126918001bc8e729b7beb929c135dbf2f98c52544b953e06 |
| wave30-i08-sdk-first-run-source-tree.json | 3111 / 78 | 65bf3341a7baeb8ae228808c96800b688fd0990072c5b6983f897e1bd58f26b9 |
| wave30-i08-sdk-first-run-sources/Program.cs | 16466 / 368 | ba0c8927d8141cba0cd0397cc85471ee9fd823a1e5dc1600bc55ba483545f607 |
| wave30-i08-sdk-first-run-sources/WorkloadIntegrityChecker.cs | 2043 / 47 | 6836dd126333ce03990af00ac29d0518677d9477bf24a23e404d0590601106a9 |

The current root receipt correctly distinguishes SDK v9.0.200 commit
**90e8b202f25b7c2bf3b883d421ad5b1cb477e8b0** from tree
**c322ba339d3adb62d542ea33c1f5b3d9935e7988**. The separate retained
initial receipt labeled the commit as source_tree, and the initial
source-tree-selection JSON likewise records 90e8 in its tree field.
They remain historical witnesses, not corrected current-tree claims.
Current/initial five source records match exactly, and the current
receipt preserves the initial receipt hash and false SDK/certificate
execution fields. WorkloadIntegrityChecker's retained complete body
hash matches the pinned primary-body identity recorded in immutable
5ec4c70; it is additional to that original five-file receipt.
I did not repeat the recorded tag/API lookup or re-attest installed
SDK binaries. The existing .version/SDK metadata guards are unchanged.

SDK Program.cs:175–177 reads DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK;
without an explicit value its default comes from CI detection.
Program.cs:190–195 passes that value into the first-run configuration.
At :323 it captures whether the first-use sentinel was missing
BEFORE calling Configure at :338. At :345–354 it invokes
WorkloadIntegrityChecker.RunFirstUseCheck only when that captured
first-use value is true and SkipWorkloadIntegrityCheck is false.
The catcher can consume a checker exception and continue, so a later
selftest marker alone would not establish that bootstrap was avoided.

The complete checker at :18–37 constructs the resolver, temporary
package directory/downloader and installer and reads installed-workload
records. At :39–43 it invokes InstallWorkloads inside a transaction
if there are installed records. This is source-defined behavior:
neither the record contents nor an actual repair/install/temporary
directory/network action on this host was observed. The explicit
true skip prevents entry into this identified checker branch without
relying on CI detection, private sentinel state or an installed-record
scan. It does not weaken the application's security contracts.

The certificate false value remains explicit and checked: Program.cs:
171 and the previously fully read first-run configurer :77–82/:102–107
gate the certificate generator as recorded in the preserved 756 append.
The obsolete first-experience key stays unchanged and is not relied
upon for either gate. Workload-update-notification disable is retained
but is not substituted for the integrity skip. The non-installer
macOS run still selects DoNothingEnvironmentPath as previously sourced;
no additional PATH/configuration policy is introduced. These two flags
are not a universal first-use-write bypass or a network sandbox.
All prior NuGet migration/profile/sentinel and other unmeasured
first-use limits remain.

### Exact composed command and environment guards

The only addition to the 756 command is the **46-byte** line
`DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK=true`, following the existing
certificate opt-out and preceding the same sole SDK launch.

Command identity: **1,574 bytes** with no terminal newline,
SHA-256 **0b1caf022448de4941695e561d765cda2409018dcdfabcb3a52c067968460c0d**. Shell token parsing as DATA after normalizing
backslash-newline continuations yields **29** tokens: env, **12**
explicit assignments, then the same **16** SDK argv tokens.
The prior command is **1,528 bytes**, SHA-256
**8a5b54debc10ba837af3b319e964724a9f47909ef8bf95c88edb2c9a7f44c1c3**. Removing exactly the added line recovers it in full;
all eleven prior key/value assignments and all sixteen arguments match.

The controller binds the new command hash and 1,574-byte count, token
count 29, tokens[1:13] assignment span, tokens[13:15] executable/run
span, and returned tokens[13:] argv. The exact-key guard is twelve
entries and adds only DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK. Its two
explicit value guards require certificate false and workload skip
true. Duplicate, missing, additional or wrong bindings cannot satisfy
the fixed token count, dictionary length, exact key set and pinned
whole-command hash; refusal retains the fixed command_environment
code before child creation. All earlier assignment values are bound
by the same pinned command, not guessed from inherited environment.

env.update installs these exact values over any inherited values;
the same env goes to the sole SDK Popen. inputs records the updated
command_bytes/hash and twelve-entry explicit_environment, so the
same canonical input hash/launch/result receipt path binds the new
skip before launch. No receipt schema/key/resource/source guard is
changed. This verifies source binding, not an observed child environment.

The shell fence's presentation newline is excluded from its identity.
The archived COMMAND constant below is exactly those 1,574 bytes.
This command is DATA and was never invoked.

```sh
env \
  DOTNET_ROOT=/usr/local/share/dotnet \
  DOTNET_CLI_HOME=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/cli-home \
  NUGET_PACKAGES=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/packages \
  DOTNET_CLI_TELEMETRY_OPTOUT=1 \
  DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1 \
  DOTNET_GENERATE_ASPNET_CERTIFICATE=false \
  DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK=true \
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

Zero-context command diff to 756: **142 bytes**, SHA-256
**66a382defe2745c223539de3049ce7eb49bc8ad80044bafc49400fc20787eb19**. Forward and inverse text applications reconstruct
both complete command strings.

```diff
--- i08-managed-command-7569146
+++ i08-managed-command-final-bootstrap-deadline
@@ -7,0 +8 @@
+  DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK=true \
```

### Exact terminal clock guard and preserved cleanup

Inside the existing main run loop's if ended() branch, immediately
before the unchanged break, the only algorithmic addition is:

```python
                if time.monotonic() >= deadline:
                    latch("child_timeout")
```

This is the previously reviewed **92-byte / two-line** correction,
applied after the SAME WNOWAIT terminal observation, with the SAME
child deadline assigned from child_started_at + CHILD_SECONDS.
In the composed body, if ended() is at controller line 607;
its new clock/latch are the next two lines, followed by the same break.
There is no deadline reset, new phase, consuming wait, retry or child.
The existing top-of-loop timeout and nonterminal path remain intact.

The fresh >= check prevents an observation that has reached/passed
the deadline from reaching cleanup/grading with no failure merely
because an earlier loop clock was timely. latch retains the first
failure: an earlier capture/resource error cannot be overwritten by
the new child_timeout. The unchanged PASS conjunction still requires
first_error is None. The same cleanup finally, original nonrenewed
10-second cleanup clock, numeric consuming child wait, source/output
readback and receipt fsync run before grading. No earlier failure is
cleared. These are static control-flow claims, not runtime scheduling,
clock/signal/wait/libproc experiments. The guard limits late observed
acceptance; it does not claim a hard kernel/IO/cancellation bound or
a whole-controller 130-second wall-clock guarantee.

### Complete composed controller, diff and inverse

The complete final prospective controller is **32,734 bytes /
735 lines** including the final newline, SHA-256
**88b97b52d91b88ee5c689c26a5773398acf8e9f389ce79c36d22382e8340527c**. The source baseline is the entire 756
**32,459-byte / 730-line** controller at SHA-256
**81366c6e29fb324e66206e2936b2faafc67d91cd6d9585910b6d1c9867ae95ae**. The composed body adds five net lines and
275 net bytes. There are still **25 function definitions**,
the same four class definitions and one Popen.

The complete zero-context controller diff is **1697 bytes**,
SHA-256 **7c9e24913d36cdbfc252f514bcc313421e4b01c3be30a30219c49bd5716e8fdc**. It contains only the command/environment binding
updates and the new terminal guard. Each text replacement was unique.
Reversing them recovers EVERY baseline byte, and applying this diff
forward/reverse independently reconstructs both entire bodies.

The independent AST inverse restores COMMAND/COMMAND_SHA, the two
command-byte constants, token count and three positive slices; changes
the environment count 12 back to 11, removes only the new allowlist key
and new true predicate; then deletes only the new inner terminal If.
It recovers the complete 756 AST excluding source locations. Unrelated
token indexes/negative slices, certificate false predicate, first
failure and all other nodes remain untouched.

All original top-level assignments except COMMAND/COMMAND_SHA are
whole-AST equal. All class/function bodies except preflight/run are
byte/AST identical, including nested latch/ended/pump/sample/
finish_group helpers and the SIGCHLD-default helper. Reversing only
the two-line terminal addition makes the whole run body byte-identical.
Thus source/SDK/catalog/config/privacy/resource/receipt/cleanup/grade
bodies are protected, rather than accepted on a count-only assertion.

```diff
--- i08-managed-controller-7569146
+++ i08-managed-controller-final-bootstrap-deadline
@@ -20 +20 @@
-COMMAND_SHA = "8a5b54debc10ba837af3b319e964724a9f47909ef8bf95c88edb2c9a7f44c1c3"
+COMMAND_SHA = "0b1caf022448de4941695e561d765cda2409018dcdfabcb3a52c067968460c0d"
@@ -27,0 +28 @@
+  DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK=true \
@@ -229 +230 @@
-    if len(COMMAND.encode()) != 1528 or digest(COMMAND.encode()) != COMMAND_SHA:
+    if len(COMMAND.encode()) != 1574 or digest(COMMAND.encode()) != COMMAND_SHA:
@@ -232,2 +233,2 @@
-    if (len(tokens) != 28 or tokens[0] != "env"
-            or tokens[12:14] != ["/usr/local/share/dotnet/dotnet", "run"]
+    if (len(tokens) != 29 or tokens[0] != "env"
+            or tokens[13:15] != ["/usr/local/share/dotnet/dotnet", "run"]
@@ -236,2 +237,2 @@
-    assignments = dict(item.split("=", 1) for item in tokens[1:12])
-    if (len(assignments) != 11
+    assignments = dict(item.split("=", 1) for item in tokens[1:13])
+    if (len(assignments) != 12
@@ -241,0 +243 @@
+                "DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK",
@@ -245 +247,2 @@
-            or assignments.get("DOTNET_GENERATE_ASPNET_CERTIFICATE") != "false"):
+            or assignments.get("DOTNET_GENERATE_ASPNET_CERTIFICATE") != "false"
+            or assignments.get("DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK") != "true"):
@@ -251 +254 @@
-              "command_sha256": COMMAND_SHA, "command_bytes": 1528,
+              "command_sha256": COMMAND_SHA, "command_bytes": 1574,
@@ -256 +259 @@
-    return tokens[12:], env, inputs
+    return tokens[13:], env, inputs
@@ -604,0 +608,2 @@
+                if time.monotonic() >= deadline:
+                    latch("child_timeout")
```

```python
# DESIGN ONLY: root must separately release this exact one-shot controller.
import ctypes
import hashlib
import json
import os
from pathlib import Path
import selectors
import shlex
import signal
import stat
import subprocess
import sys
import time

ROOT = Path("/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27")
LEAF = ROOT / "target/wave27/i08-managed"
PROJECT = ROOT / "windows/RiAuth.DeviceHost/RiAuth.DeviceHost.csproj"
SDK = Path("/usr/local/share/dotnet")
SOURCE_PIN = "e31fbee66f1038cfc2412e17497bbf07f83e1314"
COMMAND_SHA = "0b1caf022448de4941695e561d765cda2409018dcdfabcb3a52c067968460c0d"
COMMAND = r"""env \
  DOTNET_ROOT=/usr/local/share/dotnet \
  DOTNET_CLI_HOME=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/cli-home \
  NUGET_PACKAGES=/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27/target/wave27/i08-managed/packages \
  DOTNET_CLI_TELEMETRY_OPTOUT=1 \
  DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1 \
  DOTNET_GENERATE_ASPNET_CERTIFICATE=false \
  DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK=true \
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
  -- selftest"""
CONFIG = b"""<configuration>
  <packageSources><clear /></packageSources>
  <fallbackPackageFolders><clear /></fallbackPackageFolders>
</configuration>
"""
SOURCE_HASHES = {
    "DeviceHost.cs": "c87516ac0323e4d009e6d438cfdf2b74918db3d7c0ad9ab4b9b67fd2367c48ee",
    "Program.cs": "a1fe254ebf65a2153fcf2a17728b4b1b2283ca3d6e991be92eff2bb26126a241",
    "SelfTest.cs": "69bd5f5031bfdb9b974cb2e8201e6f2823a3f32e924eb53f5a38392807e1b38e",
    "RiAuth.DeviceHost.csproj": "e5af8331053b1167d6af93ea9ad657a0983e694fa5bb71f8f003370fbed499ea",
    "WindowsLocalAccount.cs": "85641bd7baef27b419899ec76d9665c8d5cfb90b59641bafe5908524763f0d0c",
    "WindowsStateStore.cs": "9999747d0183650a109368feb0d11644ff90ad30c23b0d0be6b0ca3a930d9ce0",
}
METADATA_HASHES = {
    ".version": "835299a4fd4532244a680605ad2047c1d44d6f8a34834b1bb747fa74ca38e11a",
    "dotnet.runtimeconfig.json": "e90a7dd2984b3ade889585a0f593d097a9421f9a0d63d64ae8ffdd313e24daf0",
    "MSBuild.runtimeconfig.json": "e90a7dd2984b3ade889585a0f593d097a9421f9a0d63d64ae8ffdd313e24daf0",
    "Roslyn/bincore/csc.runtimeconfig.json": "e46be9b13a311147cbc2203dae66958ced66105c7369690ce4ab75fdbcebb561",
    "Microsoft.NETCoreSdk.BundledVersions.props": "887582b3c662e6de057c3e1a89daa500c8f9526f3418cc8f8fddef13a70989ee",
}
SDK_FILES = {
    "dotnet": (140128, 0o755),
    "sdk/9.0.200/dotnet.dll": (3394048, 0o644),
    "sdk/9.0.200/MSBuild.dll": (1035776, 0o644),
    "sdk/9.0.200/Roslyn/bincore/csc.dll": (132096, 0o644),
    "sdk/9.0.200/NuGet.targets": (74726, 0o644),
    "sdk/9.0.200/NuGet.Build.Tasks.dll": (233984, 0o644),
    "sdk/9.0.200/Sdks/Microsoft.NET.Sdk/Sdk/Sdk.props": (2432, 0o644),
    "sdk/9.0.200/Sdks/Microsoft.NET.Sdk/Sdk/Sdk.targets": (4767, 0o644),
    "host/fxr/9.0.2/libhostfxr.dylib": (401072, 0o755),
    "shared/Microsoft.NETCore.App/9.0.2/libhostpolicy.dylib": (420240, 0o755),
    "shared/Microsoft.NETCore.App/9.0.2/System.Private.CoreLib.dll": (16264704, 0o644),
    "packs/Microsoft.NETCore.App.Ref/9.0.2/ref/net9.0/System.Runtime.dll": (837120, 0o644),
    "packs/Microsoft.NETCore.App.Host.osx-arm64/9.0.2/runtimes/osx-arm64/native/apphost": (140896, 0o755),
}
GIB = 1024 ** 3
START_FREE = 9 * GIB
STOP_FREE = 17 * GIB // 2
RSS_CAP = 2 * GIB
TREE_CAP = 256 * 1024 ** 2
LOG_CAP = 64 * 1024
RECEIPT_CAP = 512 * 1024
ENTRY_CAP = 4096
PID_CAP = 256
SAMPLE_CAP = 144
CHILD_SECONDS = 120.0
CLEANUP_SECONDS = 10.0

class FixedFailure(Exception):
    pass

def require_waitable_child():
    try:
        signal.signal(signal.SIGCHLD, signal.SIG_DFL)
        if signal.getsignal(signal.SIGCHLD) is not signal.SIG_DFL:
            raise FixedFailure("sigchld_default_unverified")
    except FixedFailure:
        raise
    except BaseException:
        raise FixedFailure("sigchld_default_unavailable_or_refused") from None

# Numeric Darwin layouts from the installed public headers; names are padding.
class BsdInfo(ctypes.Structure):
    _fields_ = [(n, ctypes.c_uint32) for n in (
        "flags", "status", "xstatus", "pid", "ppid", "uid", "gid",
        "ruid", "rgid", "svuid", "svgid", "reserved")] + [
        ("unused_names", ctypes.c_byte * 48),
        ("nfiles", ctypes.c_uint32), ("pgid", ctypes.c_uint32),
        ("jobc", ctypes.c_uint32), ("tdev", ctypes.c_uint32),
        ("tpgid", ctypes.c_uint32), ("nice", ctypes.c_int32),
        ("start_sec", ctypes.c_uint64), ("start_usec", ctypes.c_uint64)]

class TaskInfo(ctypes.Structure):
    _fields_ = [(n, ctypes.c_uint64) for n in (
        "virtual", "resident", "total_user", "total_system",
        "threads_user", "threads_system")] + [(n, ctypes.c_int32) for n in (
        "policy", "faults", "pageins", "cow_faults", "messages_sent",
        "messages_received", "syscalls_mach", "syscalls_unix", "csw",
        "threadnum", "numrunning", "priority")]

def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      allow_nan=False).encode("ascii")

def digest(data):
    return hashlib.sha256(data).hexdigest()

def bounded_regular(path, cap, owner=None):
    # Reject symlink components before reading only approved public inputs/logs.
    if path.resolve(strict=True) != path:
        raise FixedFailure("input_symlink")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_size > cap:
            raise FixedFailure("input_type_size")
        if owner is not None and info.st_uid != owner:
            raise FixedFailure("input_owner")
        data = bytearray()
        while len(data) <= cap:
            chunk = os.read(fd, min(4096, cap + 1 - len(data)))
            if not chunk:
                break
            data.extend(chunk)
        if len(data) > cap:
            raise FixedFailure("input_type_size")
        return bytes(data)
    finally:
        os.close(fd)

def free_bytes():
    info = os.statvfs(ROOT)
    return info.f_bavail * info.f_frsize

def check_directory(path, owner):
    info = path.lstat()
    if (not stat.S_ISDIR(info.st_mode) or info.st_uid != owner
            or path.resolve(strict=True) != path):
        raise FixedFailure("directory_identity")
    return info

def verify_sources():
    if sorted(item.name for item in PROJECT.parent.iterdir()) != sorted(SOURCE_HASHES):
        raise FixedFailure("project_input_catalog")
    observed = {}
    for name, expected in SOURCE_HASHES.items():
        value = digest(bounded_regular(PROJECT.parent / name, 65536))
        if value != expected:
            raise FixedFailure("source_identity")
        observed[name] = value
    for name in ("bin", "obj"):
        if os.path.lexists(PROJECT.parent / name):
            raise FixedFailure("project_output_present")
    return observed

def preflight():
    if Path.cwd() != ROOT or sys.platform != "darwin" or os.uname().machine != "arm64":
        raise FixedFailure("host_cwd_identity")
    if not all(hasattr(os, n) for n in ("waitid", "WNOWAIT", "WEXITED", "P_PID")):
        raise FixedFailure("wait_observer_unavailable")
    if os.path.lexists(LEAF):
        raise FixedFailure("private_leaf_present")
    for parent in (ROOT, ROOT / "target", ROOT / "target/wave27"):
        check_directory(parent, os.getuid())
    if free_bytes() < START_FREE:
        raise FixedFailure("start_disk")
    for key in ("DOTNET_ROOT", "DOTNET_ROOT_ARM64", "DOTNET_ROOT_X64",
                "DOTNET_MSBUILD_SDK_RESOLVER_SDKS_DIR",
                "DOTNET_MSBUILD_SDK_RESOLVER_SDKS_VER", "MSBuildSDKsPath",
                "MSBUILD_EXE_PATH", "DOTNET_CLI_HOME", "NUGET_PACKAGES",
                "DOTNET_STARTUP_HOOKS", "DOTNET_ADDITIONAL_DEPS"):
        if key in os.environ:
            raise FixedFailure("inherited_override")
    for parent in (PROJECT.parent, *PROJECT.parent.parents):
        for name in ("global.json", "Directory.Build.props", "Directory.Build.targets",
                     "Directory.Packages.props", "NuGet.Config", "nuget.config"):
            if os.path.lexists(parent / name):
                raise FixedFailure("inherited_configuration")
    if os.path.lexists(PROJECT.parent / "Properties/launchSettings.json"):
        raise FixedFailure("launch_settings_present")
    sources = verify_sources()
    if sorted(item.name for item in (SDK / "sdk").iterdir()) != ["9.0.200"]:
        raise FixedFailure("sdk_selection_metadata")
    for relative, (size, mode) in SDK_FILES.items():
        path = SDK / relative
        info = path.lstat()
        if (path.resolve(strict=True) != path or not stat.S_ISREG(info.st_mode)
                or info.st_uid != 0 or info.st_size != size
                or stat.S_IMODE(info.st_mode) != mode):
            raise FixedFailure("sdk_file_metadata")
    if not os.access(SDK / "dotnet", os.R_OK | os.X_OK):
        raise FixedFailure("sdk_executable_access")
    for relative in ("sdk/9.0.200", "host/fxr/9.0.2",
                     "shared/Microsoft.NETCore.App/9.0.2",
                     "packs/Microsoft.NETCore.App.Ref/9.0.2",
                     "packs/Microsoft.NETCore.App.Host.osx-arm64/9.0.2"):
        check_directory(SDK / relative, 0)
    metadata = {}
    for name, expected in METADATA_HASHES.items():
        value = digest(bounded_regular(SDK / "sdk/9.0.200" / name, 60000, 0))
        if value != expected:
            raise FixedFailure("sdk_metadata_identity")
        metadata[name] = value
    check_directory(Path("/Users/dominik/.nuget/packages"), os.getuid())
    if len(COMMAND.encode()) != 1574 or digest(COMMAND.encode()) != COMMAND_SHA:
        raise FixedFailure("command_identity")
    tokens = shlex.split(COMMAND.replace("\\\n", ""), posix=True)
    if (len(tokens) != 29 or tokens[0] != "env"
            or tokens[13:15] != ["/usr/local/share/dotnet/dotnet", "run"]
            or tokens[-2:] != ["--", "selftest"]):
        raise FixedFailure("command_tokens")
    assignments = dict(item.split("=", 1) for item in tokens[1:13])
    if (len(assignments) != 12
            or set(assignments) != {
                "DOTNET_ROOT", "DOTNET_CLI_HOME", "NUGET_PACKAGES",
                "DOTNET_CLI_TELEMETRY_OPTOUT", "DOTNET_SKIP_FIRST_TIME_EXPERIENCE",
                "DOTNET_GENERATE_ASPNET_CERTIFICATE",
                "DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK",
                "DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE", "DOTNET_NOLOGO",
                "DOTNET_PROCESSOR_COUNT", "DOTNET_CLI_USE_MSBUILD_SERVER",
                "MSBUILDDISABLENODEREUSE"}
            or assignments.get("DOTNET_GENERATE_ASPNET_CERTIFICATE") != "false"
            or assignments.get("DOTNET_SKIP_WORKLOAD_INTEGRITY_CHECK") != "true"):
        raise FixedFailure("command_environment")
    env = os.environ.copy()
    env.update(assignments)
    inputs = {"source_pin": SOURCE_PIN, "sources": sources,
              "sdk_public_metadata": metadata, "sdk_file_metadata": SDK_FILES,
              "command_sha256": COMMAND_SHA, "command_bytes": 1574,
              "explicit_environment": assignments, "config_sha256": digest(CONFIG),
              "start_free": START_FREE, "stop_free": STOP_FREE,
              "rss_cap": RSS_CAP, "tree_cap": TREE_CAP, "log_cap": LOG_CAP,
              "child_seconds": CHILD_SECONDS, "cleanup_seconds": CLEANUP_SECONDS}
    return tokens[13:], env, inputs

class GroupObserver:
    def __init__(self):
        if ctypes.sizeof(BsdInfo) != 136 or ctypes.sizeof(TaskInfo) != 96:
            raise FixedFailure("process_abi")
        self.lib = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
        self.lib.proc_listpids.argtypes = (
            ctypes.c_uint32, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_int)
        self.lib.proc_listpids.restype = ctypes.c_int
        self.lib.proc_pidinfo.argtypes = (
            ctypes.c_int, ctypes.c_int, ctypes.c_uint64,
            ctypes.c_void_p, ctypes.c_int)
        self.lib.proc_pidinfo.restype = ctypes.c_int
        self.identities = {}

    def members(self, pgid):
        buf = (ctypes.c_int * (PID_CAP + 1))()
        ctypes.set_errno(0)
        count = self.lib.proc_listpids(2, pgid, buf, ctypes.sizeof(buf))
        if (count < 0 or (count == 0 and ctypes.get_errno() != 0)
                or count % 4 or count >= ctypes.sizeof(buf)):
            raise FixedFailure("group_list_bound")
        values = sorted({int(pid) for pid in buf[:count // 4] if pid > 0})
        if len(values) > PID_CAP:
            raise FixedFailure("group_member_cap")
        return values

    def info(self, pid, pgid):
        value = BsdInfo()
        count = self.lib.proc_pidinfo(pid, 3, 0, ctypes.byref(value), 136)
        if count != 136:
            try:
                if os.getpgid(pid) != pgid:
                    return None
            except ProcessLookupError:
                return None
            raise FixedFailure("process_identity_unreadable")
        if value.pid != pid or value.pgid != pgid or value.uid != os.getuid():
            raise FixedFailure("process_identity")
        identity = (int(value.start_sec), int(value.start_usec))
        if pid in self.identities and self.identities[pid] != identity:
            raise FixedFailure("process_identity_changed")
        self.identities[pid] = identity
        return value

    def sample(self, pgid):
        members = self.members(pgid)
        resident = 0
        for pid in members:
            identity = self.info(pid, pgid)
            if identity is None or identity.status == 5:  # SZOMB, not running.
                continue
            task = TaskInfo()
            count = self.lib.proc_pidinfo(pid, 4, 0, ctypes.byref(task), 96)
            if count != 96:
                again = self.info(pid, pgid)
                if again is None or again.status == 5:
                    continue
                raise FixedFailure("rss_unreadable")
            resident += int(task.resident)
        return members, resident

def tree_bytes(leaf_identity):
    info = check_directory(LEAF, os.getuid())
    if (info.st_dev, info.st_ino) != leaf_identity:
        raise FixedFailure("private_leaf_identity")
    total, entries = 0, 0
    def walk_error(_):
        raise FixedFailure("private_scan")
    for parent, dirs, files in os.walk(LEAF, followlinks=False, onerror=walk_error):
        if len(Path(parent).relative_to(LEAF).parts) > 32:
            raise FixedFailure("private_depth")
        for name in dirs + files:
            entries += 1
            if entries > ENTRY_CAP:
                raise FixedFailure("private_entry_cap")
            item = (Path(parent) / name).lstat()
            if (item.st_uid != os.getuid()
                    or not (stat.S_ISREG(item.st_mode) or stat.S_ISDIR(item.st_mode))):
                raise FixedFailure("private_entry_type")
            total += max(item.st_size, item.st_blocks * 512)
    return total

def run():
    first_error = None
    child = None
    pgid = None
    verified_group = False
    child_started_at = None
    cleanup_deadline = None
    reaped = False
    child_exit = None
    observer = None
    leaf_identity = None
    inputs = None
    input_hash = None
    samples = []
    logs = {}
    selector = selectors.DefaultSelector()
    cleanup = {"attempted": False, "term": False, "kill": False,
               "reaped": False, "group_empty": None, "errors": []}
    phase = "preflight"
    receipt_durable = False

    def latch(code, exc=None):
        nonlocal first_error
        if first_error is None:
            names = ((FixedFailure, "FixedFailure"), (OSError, "OSError"),
                     (ValueError, "ValueError"), (RuntimeError, "RuntimeError"),
                     (KeyboardInterrupt, "KeyboardInterrupt"), (SystemExit, "SystemExit"))
            kind = next((name for cls, name in names if type(exc) is cls), "Other")
            first_error = {"code": code, "class": kind if exc is not None else None}

    def catch(code, exc):
        if type(exc) is FixedFailure:
            latch(exc.args[0], exc)  # Only controller-owned fixed string literals.
        else:
            latch(code, exc)

    def cleanup_error(code):
        if code not in cleanup["errors"] and len(cleanup["errors"]) < 12:
            cleanup["errors"].append(code)

    def exclusive(name):
        fd = os.open(LEAF / name,
                     os.O_RDWR | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        if stat.S_IMODE(os.fstat(fd).st_mode) != 0o600:
            os.close(fd)
            raise FixedFailure("private_file_mode")
        return fd

    def write_all(fd, data):
        view = memoryview(data)
        while view:
            count = os.write(fd, view)
            if count <= 0:
                raise FixedFailure("private_write")
            view = view[count:]

    def ended():
        # No poll()/wait() before cleanup: WNOWAIT reserves the group-leader PID.
        return os.waitid(os.P_PID, child.pid,
                         os.WEXITED | os.WNOHANG | os.WNOWAIT) is not None

    def pump(timeout):
        for key, _ in selector.select(max(0.0, timeout)):
            name = key.data
            try:
                data = os.read(key.fileobj.fileno(), 4096)
            except BlockingIOError:
                continue
            if not data:
                selector.unregister(key.fileobj)
                key.fileobj.close()
                logs[name]["eof"] = True
                continue
            log = logs[name]
            log["seen"] += len(data)
            available = LOG_CAP - log["kept"]
            retained = data[:max(0, available)]
            if retained:
                write_all(log["fd"], retained)
                log["kept"] += len(retained)
            if len(data) > available:
                log["truncated"] = True
                latch(name + "_cap")

    def sample():
        members, resident = observer.sample(pgid)
        used = tree_bytes(leaf_identity)
        available = free_bytes()
        if len(samples) >= SAMPLE_CAP:
            raise FixedFailure("sample_cap")
        samples.append({"seconds": round(time.monotonic() - child_started_at, 6),
                        "members": members, "rss_bytes": resident,
                        "private_bytes": used, "free_bytes": available})
        if available <= STOP_FREE:
            raise FixedFailure("stop_disk")
        if resident > RSS_CAP:
            raise FixedFailure("rss_cap")
        if used + RECEIPT_CAP > TREE_CAP:
            raise FixedFailure("private_growth")
        return members

    def signal_group(sig):
        if not verified_group or reaped:
            cleanup_error("group_signal_ownership")
            return
        try:
            os.killpg(pgid, sig)
            cleanup["term" if sig == signal.SIGTERM else "kill"] = True
        except ProcessLookupError:
            pass
        except BaseException:
            cleanup_error("group_signal")

    def finish_group():
        nonlocal reaped, child_exit, cleanup_deadline
        if child is None:
            cleanup["group_empty"] = True
            return
        cleanup["attempted"] = True
        if cleanup_deadline is None:
            cleanup_deadline = time.monotonic() + CLEANUP_SECONDS
        if not verified_group:
            cleanup_error("group_unverified")
            # Only the unreaped direct Popen child is addressable in this branch.
            try:
                child.kill()
            except BaseException:
                cleanup_error("direct_kill")
        else:
            signal_group(signal.SIGTERM)
        kill_at = cleanup_deadline - 5.0
        if first_error is not None and first_error["code"] in (
                "stop_disk", "rss_cap", "private_growth", "stdout_cap", "stderr_cap"):
            kill_at = time.monotonic()
        next_sample = time.monotonic()
        while time.monotonic() < cleanup_deadline:
            now = time.monotonic()
            if not cleanup["kill"] and now >= kill_at:
                if verified_group:
                    signal_group(signal.SIGKILL)
                else:
                    try:
                        child.kill()
                        cleanup["kill"] = True
                    except BaseException:
                        cleanup_error("direct_kill")
            try:
                pump(min(0.1, cleanup_deadline - now))
            except BaseException as exc:
                catch("cleanup_capture", exc)
                cleanup_error("capture_drain")
            try:
                if now >= next_sample and verified_group:
                    next_sample = now + 1.0
                    sample()
                members = observer.members(pgid) if verified_group else []
                done = ended()
                if done and (not verified_group or not set(members) - {child.pid}):
                    # Send final KILL while the unreaped leader still reserves PGID.
                    if verified_group:
                        signal_group(signal.SIGKILL)
                    child_exit = child.wait(timeout=max(0.0, cleanup_deadline - time.monotonic()))
                    reaped = True
                    break
            except BaseException as exc:
                cleanup_error("group_observation")
                catch("cleanup_observation", exc)
                # Observation errors remove grace, not group ownership.
                kill_at = time.monotonic()
        if not reaped:
            if verified_group:
                signal_group(signal.SIGKILL)
            try:
                child_exit = child.wait(timeout=max(0.0, cleanup_deadline - time.monotonic()))
                reaped = True
            except BaseException:
                cleanup_error("direct_reap")
        cleanup["reaped"] = reaped
        # After reaping, never signal a potentially reused group number.
        try:
            os.killpg(pgid, 0)
            cleanup["group_empty"] = False
        except ProcessLookupError:
            cleanup["group_empty"] = True
        except BaseException:
            cleanup["group_empty"] = None
            cleanup_error("final_group_probe")
        if cleanup["group_empty"] is not True:
            cleanup_error("group_not_empty")
        if verified_group and observer is not None:
            try:
                if observer.members(pgid):
                    cleanup["group_empty"] = False
                    cleanup_error("final_group_members")
            except BaseException:
                cleanup["group_empty"] = None
                cleanup_error("final_group_members")

    try:
        argv, env, inputs = preflight()
        input_hash = digest(canonical(inputs))
        observer = GroupObserver()  # Future numeric observation only, no helper child.
        os.umask(0o077)  # This standalone controller retains 077 until exit.
        os.mkdir(LEAF, 0o700)  # Fails if any competing/existing leaf is present.
        leaf = check_directory(LEAF, os.getuid())
        if stat.S_IMODE(leaf.st_mode) != 0o700:
            raise FixedFailure("private_leaf_mode")
        leaf_identity = (leaf.st_dev, leaf.st_ino)
        for name in ("cli-home", "packages"):
            os.mkdir(LEAF / name, 0o700)
        fd = exclusive("nuget-offline.config")
        try:
            write_all(fd, CONFIG)
            os.fsync(fd)
        finally:
            os.close(fd)
        for name in ("stdout", "stderr"):
            logs[name] = {"fd": exclusive(name + ".log"), "seen": 0,
                          "kept": 0, "truncated": False, "eof": False}
        if free_bytes() < START_FREE:
            raise FixedFailure("start_disk")
        require_waitable_child()
        phase = "child"
        child_started_at = time.monotonic()
        child = subprocess.Popen(argv, cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                 bufsize=0, start_new_session=True, close_fds=True)
        pgid = child.pid
        verified_group = os.getpgid(pgid) == pgid and os.getsid(pgid) == pgid
        if not verified_group:
            raise FixedFailure("new_group_identity")
        for name, stream in (("stdout", child.stdout), ("stderr", child.stderr)):
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_READ, name)
        owner = observer.info(pgid, pgid)
        if owner is None:
            raise FixedFailure("new_group_owner_record")
        launch = canonical({"schema_version": 1, "input_sha256": input_hash,
                            "pid": child.pid, "pgid": pgid, "session": pgid,
                            "owner_uid": int(owner.uid),
                            "leader_start_sec": int(owner.start_sec),
                            "leader_start_usec": int(owner.start_usec),
                            "start_monotonic": child_started_at,
                            "child_deadline_monotonic": child_started_at + CHILD_SECONDS,
                            "grade_performed": False}) + b"\n"
        if len(launch) > 2048:
            raise FixedFailure("launch_receipt_cap")
        fd = exclusive("launch.json")
        try:
            write_all(fd, launch)
            os.fsync(fd)
        finally:
            os.close(fd)
        deadline = child_started_at + CHILD_SECONDS
        next_sample = time.monotonic()
        while first_error is None:
            now = time.monotonic()
            if now >= deadline:
                latch("child_timeout")
                break
            if now >= next_sample:
                sample()
                next_sample = time.monotonic() + 1.0
            pump(min(0.1, deadline - time.monotonic()))
            if ended():
                if time.monotonic() >= deadline:
                    latch("child_timeout")
                break
    except BaseException as exc:
        catch(phase + "_exception", exc)
    finally:
        phase = "cleanup"
        try:
            finish_group()
        except BaseException as exc:
            catch("cleanup_exception", exc)
            cleanup_error("cleanup_exception")
            # A final reserved-PID group KILL; the same deadline is not reset.
            if child is not None and not reaped:
                signal_group(signal.SIGKILL)
                try:
                    child_exit = child.wait(timeout=max(
                        0.0, (cleanup_deadline or time.monotonic()) - time.monotonic()))
                    reaped = True
                    cleanup["reaped"] = True
                except BaseException:
                    cleanup_error("direct_reap")
            cleanup["group_empty"] = None
        try:
            # Final bounded nonblocking drain after group cleanup, not a new grace.
            for _ in range(34):
                if not selector.get_map():
                    break
                pump(0.0)
        except BaseException as exc:
            catch("capture_finalize", exc)
        for key in list(selector.get_map().values()):
            try:
                selector.unregister(key.fileobj)
                key.fileobj.close()
            except BaseException:
                cleanup_error("pipe_close")
        try:
            selector.close()
        except BaseException:
            cleanup_error("selector_close")
        for log in logs.values():
            try:
                os.fsync(log["fd"])
            except BaseException as exc:
                catch("capture_fsync", exc)
            finally:
                try:
                    os.close(log["fd"])
                except BaseException:
                    cleanup_error("capture_close")

    # All serialization and grading are AFTER the cleanup finally above.
    if child_exit is not None and child_exit != 0:
        latch("child_nonzero")
    captured = {}
    post_sources = None
    try:
        if leaf_identity is None:
            raise FixedFailure("no_owned_receipt_workspace")
        for name, log in logs.items():
            data = bounded_regular(LEAF / (name + ".log"), LOG_CAP, os.getuid())
            captured[name] = data
            if len(data) != log["kept"]:
                latch("capture_identity")  # Keep actual bounded failed-output evidence.
        post_sources = verify_sources()
        tree = tree_bytes(leaf_identity)
        available = free_bytes()
        if available <= STOP_FREE:
            latch("stop_disk")
        if tree + RECEIPT_CAP > TREE_CAP:
            latch("private_growth")
        for name, log in logs.items():
            if not log["eof"]:
                latch(name + "_eof_unverified")
        elapsed = None if child_started_at is None else time.monotonic() - child_started_at
        receipt = {"schema_version": 1, "source_pin": SOURCE_PIN,
                   "command_sha256": COMMAND_SHA, "input_sha256": input_hash,
                   "inputs": inputs, "child_started": child is not None,
                   "child_pid": None if child is None else child.pid, "pgid": pgid,
                   "child_exit": child_exit, "elapsed_through_cleanup": elapsed,
                   "first_error": first_error, "cleanup": cleanup,
                   "post_sources": post_sources, "resource_samples": samples,
                   "final_private_bytes_before_receipt": tree, "final_free_bytes": available,
                   "outputs": {name: {"bytes": len(captured.get(name, b"")),
                              "sha256": digest(captured.get(name, b"")),
                              "bytes_observed": log["seen"], "bytes_kept_recorded": log["kept"],
                              "truncated": log["truncated"], "eof": log["eof"]}
                               for name, log in logs.items()},
                   "grade_performed": False}
        payload = canonical(receipt) + b"\n"
        if len(payload) > RECEIPT_CAP:
            raise FixedFailure("receipt_cap")
        fd = exclusive("result.json")
        try:
            write_all(fd, payload)
            os.fsync(fd)
        finally:
            os.close(fd)
        dir_fd = os.open(LEAF, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            os.fsync(dir_fd)
        finally:
            os.close(dir_fd)
        receipt_durable = True
    except BaseException as exc:
        catch("receipt_exception", exc)
        # No retry/overwrite and no grading. The preceding finally already
        # attempted group cleanup even if serialization/fsync now fails.
    ok = (receipt_durable and first_error is None and child is not None
          and child_exit == 0 and cleanup["reaped"]
          and cleanup["group_empty"] is True and not cleanup["errors"]
          and captured.get("stdout", b"").splitlines().count(b"selftest passed") == 1)
    # No exception text, raw output, source values, environment, or private path.
    status = "UNGRADABLE" if not receipt_durable else ("PASS" if ok else "FAIL")
    public = {"schema_version": 1, "status": status,
              "receipt_durable": receipt_durable, "input_sha256": input_hash,
              "first_error": first_error, "cleanup_errors": cleanup["errors"],
              "child_exit": child_exit, "reaped": cleanup["reaped"],
              "group_empty": cleanup["group_empty"]}
    try:
        print(json.dumps(public, sort_keys=True, separators=(",", ":")))
    except BaseException:
        return 2  # Durable private receipt remains; no raw exception is printed.
    return 0 if ok else 2

if __name__ == "__main__":
    sys.exit(run())
```

### Actual static verification and limits

Whole byte and independent whole-AST inverses passed; both zero-context
diffs reconstruct full old/new command/controller strings in both
directions. AST parsing passed without compilation, import, eval,
constructor/function/case execution. Exact command bytes/hash,
29 tokens, twelve unique allowed keys, certificate false/workload
true predicates and unchanged sixteen SDK argv tokens passed.
All eleven earlier assignment values remain exact. The twelve-key
set and value predicates were checked as AST/DATA.

The entire SIGCHLD helper remains byte-identical, with one setter and
one exact get-signal verifier, and its single main-path call is still
before the sole Popen. No ignored disposition is restored before reap.
The terminal guard matches the reviewed two-line condition/call, occurs
once only in the main ended branch and preserves the existing break.
There is no new function/class/import, syscall/observer path, config,
source pin, child, receipt key or retry. Static source proofs do not
observe actual native signal flags, a waitable child or numeric status.

All six current project files remain equal to their whole blobs/hashes
at **e31fbee66f1038cfc2412e17497bbf07f83e1314**:
DeviceHost.cs, Program.cs, SelfTest.cs, RiAuth.DeviceHost.csproj,
WindowsLocalAccount.cs and WindowsStateStore.cs. The same production
15-second managed response deadline, five old mocks and sixth stall
fixture are untouched and remain uncompiled/unrun in this lane.
The five SDK metadata hashes and thirteen size/mode identities are
unchanged source definitions, not new native attestations.

The 120-second child/10-second cleanup clocks, 1-second sampled
2-GiB RSS/256-MiB private-growth controls, 9-GiB start/8.5-GiB stop
margin above the 8-GiB floor, 64-KiB-per-stream capture, 512-KiB
receipt and sample/entry/PID caps remain exact. WNOWAIT PID reservation,
generation/UID/group checks, no signals after reap, TERM/KILL/join and
final group-absence observations retain their earlier source limits.
Private full bounded output/exit/elapsed/input/resource facts,
first-failure and separate cleanup facts, source recheck and fsync
still precede the fixed selftest marker grade. Cleanup still occurs
before serialization failures. Sampled limits do not establish
continuous peaks/quotas, escaped-group cleanup, hard syscall deadlines
or a network sandbox. No stronger lifetime/cancellation assurance
is inferred from the terminal clock check.

The initial combined source/review display was output-truncated and
a second combined independent-report diff exceeded its output budget.
The full primary Program body was reread alone, the entire 756 body
was reread in three complete bounded parts, and the complete checker/
JSON bodies and decisive F4/F2 sections were read. No omitted display
was credited as a full body read. Composition/inverse assertions passed
on the first static preparation attempt; no candidate fix or runtime
retry occurred. Every prior actual static error remains in the full
756 prefix, including the earlier token-slice checker failures and
receipt-label correction.

The existing private target/wave27/i08-managed leaf and project
bin/obj are absent; no directory/config/output/dependency path was
created. No SDK/version/selftest/compiler/native, signal/wait/libproc
experiment, controller/function/helper/import/main/case, package/
certificate/workload/keychain/store action, network/provider/browser/
Driver/Cargo/service/process probe or runtime was used. No lane,
worker/task/WT/shell/status/main/push/merge/alignment/contact operation
was added. Only this report is written. No certificate/workload
mutation or actual managed PASS/Windows gate is inferred.
Root plus Sol3 complete immutable source review and a separate
explicit one-shot release remain required; original native signed
Windows inputs and historical failures/unknowns stay separate.
Final docs/hygiene/whitespace/scope and archived readback checks follow.

Final actual static receipt for this composition:
python3 scripts/check-docs.py exited 0 (Markdown links and build-directory
layout checked); python3 scripts/check-repo-hygiene.py exited 0 (960
tracked files); git diff --check exited 0. The append's complete shell,
minimal terminal fragment, command/controller diffs and 32,734-byte
controller archive were read back and matched the generated DATA
byte-for-byte. Command hash/1,574 bytes/29 tokens/12 keys/16 unchanged
argv tokens, both explicit bootstrap value guards, the unique new
terminal guard and preserved SIGCHLD setter/verifier passed. Full
byte/AST inverse and both diff directions passed from immutable 756.

The entire 188,549-byte 756 report prefix remains exact; all six e31
project blobs remain exact; private leaf and project bin/obj remain
absent. Git scope is this sole report, without untracked files or
unrelated staged content. Final staged scope/whitespace and immutable
report-only commit readback complete the handoff. All earlier failures,
dated source receipts and limits remain preserved. No source/SDK/
native/controller execution or runtime lane is credited. Root and
Sol3 whole immutable review are still required, and runtime remains
HELD until a separately authorized one-shot managed selftest release.
