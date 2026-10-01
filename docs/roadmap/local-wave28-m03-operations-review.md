# Wave28 M03 operations review

Latest static review: management snapshot
`fa7511f3ac8854df77bfd3a12d950eed2f1d9e73`, compared with accepted main
`148fafd4825c6cf803faf4ae869e3089e1462982`, plus the separately pinned F3
correction `a162347cf0b5d55d2957066fff26ca2145f6197e`. F1, F2 and F3 are corrected; the
connector-independent operations and corrected backup/SSF slices are recommended
for acceptance within the port boundary at the end of this report. The initial
review below records the earlier snapshot; the correction follow-up supersedes
its F1/F2 holds and F3 residual.

Date: 2026-10-02. Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` (M03).
Review worktree: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`, starting HEAD
`88c50b025e5b3c65876944011b95fb05b30e08cd`.

Initial recommendation at `9471b9a`: accept the local backend proof and the
operation slices identified below. Hold offboarding schedule/reschedule for
a validation mismatch (F1), and SSF
deletion for a missing secret-response guard (F2). Neither finding was
reproduced by running a product binary in this review. No task closure or board
change is requested.

## Original acceptance and pinned inputs

The existing task's original acceptance is: "GUI, CLI, and API must share
authorization, validation, transactions, idempotency, and audit behavior."
Its completion gate is: "The same change has the same permission checks and
outcome regardless of which interface submits it." This review does not narrow
that acceptance to the new client verbs or to one storage fixture.

Source objects came from the existing management worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`. Every implementation read used `git show`
with an immutable full commit hash, rather than that worktree's mutable files.
The accepted server comparison was
`62cdc6e66064f33d2496ebc867066aae9c10cf10`; the supplied accepted main baseline
was `e5bbc5cdc86df1e5c5ac8b676ec00640bf6902c4`. Publishing after that baseline
is outside this snapshot review.

| Alias | Full source object |
| --- | --- |
| backend proof | `4b9851aa2ad896c83a194f31fd0649e130a479eb` |
| operations | `bbbfd638c9e1d90d110792cab64cef08facc3dad` |
| mixed follow-up | `ee2b2ab3d74c0b523d5f281d1bfa19b99ed1737b` |
| SSF/backup | `7de863506ae25be56b7d42b464d6a62f02c99dba` |
| corrective | `9471b9aa3b14c703b95f9a4848016c3cd091d05d` |
| held workflow API | `32f9aed565ccf1c3f5dd242aa5df561d38185122` |

The source report was
`9471b9aa3b14c703b95f9a4848016c3cd091d05d:docs/roadmap/local-wave27-management-report.md`.
Its last edit is `ce7853f078f27d65ba6524488c31c610d380a6fe`, before the corrective
commit. Its statement that five backup/SSF findings remain unfixed is therefore
historical; the actual corrective code supersedes it. Reported runs below are
worker-reported historical evidence, not independently rerun results or raw-log
verification.

## Initial integration matrix

ACCEPT means a review recommendation for the stated slice, with the evidence
limits below. It is not acceptance of an entire cumulative source branch.

| Commit | Recommendation | Exact scope and condition |
| --- | --- | --- |
| `4b9851a` | ACCEPT | `tests/m03_riauthctl_backends_postgres.rs` and additive `scripts/test-postgres.sh` support for its target. Preserve other accepted script targets. This is local source-built binary evidence for one sequence on four storage modes. |
| `bbbfd63` | PARTIAL ACCEPT; HOLD schedule/reschedule | Accept removal confirmation, temporary access, offboarding list/cancel, directory and SCIM operations with their transport, plan helpers and narrow tests/docs. Hold the offboarding schedule/reschedule dispatch and corresponding coverage claims until F1 is corrected. F3 is a low robustness follow-up for directory/SCIM plan output. Use the operation follow-up hunks from `ee2b2ab` with this slice. |
| `ee2b2ab` | ACCEPT operation hunks only; HOLD whole commit | Accept `directory::plan_bound`, `provision::plan_bound`, the pre-Remote plan flag refusal in `main::run`, SCIM fallback mock corrections, the plan-flag refusal test and the exact removal-refusal assertion. Exclude all workflow and connector export/status changes. |
| `7de8635` | HOLD standalone; PARTIAL ACCEPT with `9471b9a` | The original archive parser, signal handling, non-UTF-8 backup output and short-secret response check need the corrective commit. With it, accept backup/archive and SSF create/list, plus their necessary Cargo, transport, dispatch and focused test changes. Hold SSF deletion for F2. Scope the documentation as below. |
| `9471b9a` | ACCEPT corrective changes; retain F2 hold | Static review supports the typed archive frames/strict config, early backup path check, Unix signal registration, bounded local SSF header validation, exact-secret echo check and best-effort zeroization wording. This is not a fresh test-pass claim. The commit does not fix SSF deletion. |
| `32f9aed` and workflow parts of `ee2b2ab` | HOLD | Explicit user hold pending live activation replay/sealing correction. No workflow API/CLI acceptance is given here. |

For `ee2b2ab`, the included test hunks are the `ops_server` 404 responses and
the retained-job assertions in `m03_parity_ops.rs`, followed by
`plan_bound_commands_refuse_a_revision_or_request_key_instead_of_ignoring_it`.
Stop before `export_server` and its connector tests. Include the exact
409/conflict/removal-review assertion in `tests/m03_state_removal_e2e.rs`.
In `main.rs`, exclude `mod workflow`, the workflow command enum and dispatch.
Exclude its `management.rs` connector-status changes, `workflow.rs`,
`m03_parity_workflow.rs` and `m03_workflow_approval_e2e.rs`. Extract only the
relevant operation sentences from mixed README/capability/operations docs.
The connector cumulative stack belongs to the revisions lane.

Root should port these as focused slices or wait for owner corrections. Do not
copy cumulative stale files over the accepted registration/key/invitation/
Windows/certificate/source/email work or CI/A03 changes.

## Concrete findings

### F1 — offboarding validation differs across interfaces (medium, hold)

At the corrective object, `crates/riauthctl/src/offboard.rs:72,90,115-119`
still calls `label`, which limits the entire timezone label to 64 bytes.
Accepted `src/offboarding.rs:75-89` accepts one to three slash-separated ASCII
components, each 1–64 bytes, with grammar
`[A-Za-z0-9_+-]{1,64}(/[A-Za-z0-9_+-]{1,64}){0,2}`.

A label formed from 32 ASCII `a` characters, `/`, and 32 ASCII `b` characters
has length 65. It satisfies the shared server's syntax but riauthctl refuses it
before sending schedule/reschedule. With otherwise valid scheduling inputs,
this changes the interface's validation outcome. This is a static counterexample
to the original M03 acceptance, not a live scheduling result.

Align the local syntax check with the shared server grammar, or leave syntax
validation to that writer. Owner verification should cover a valid multi-part
label longer than 64 bytes and rejected malformed components. No offboarding
source correction was made by this reviewer.

### F2 — SSF deletion can print an unexpected authorization header (medium, hold)

At the corrective object, `crates/riauthctl/src/ssf.rs:78-86` checks only
`deleted == true` and the exact requested `stream_id`, then returns the whole
response. Create and list invoke the recursive `carries` guard; delete does not.
The final emitter at `crates/riauthctl/src/main.rs:639-687` recursively redacts
token/secret/password/private-key/credential fields, but not
`authorization_header`.

For deletion of `one`, this synthetic response passes both checks and leaves
the header unchanged for emission:

```json
{"deleted":true,"stream_id":"one","delivery":{"authorization_header":"synthetic-review-marker"}}
```

The accepted shared writer at `src/management/ssf_streams.rs:483-503` returns
only `deleted` and `stream_id`. No current server leak or captured CLI disclosure
is claimed. The defect is a client guard missing from its documented promise
to refuse responses containing that known secret field. The mock at
`m03_parity_ssf_backup.rs:1091-1097` also never injects a delete-response header;
the existing leakage test at lines 1334–1359 exercises create/list only.

Before accepting deletion, refuse `carries(&deleted, None)` with a fixed error,
or emit only the validated deletion fields. Add a narrow mock case for a nested
authorization header on an otherwise matching deletion response and assert
that no marker appears on either output stream. A broad credential redaction
change is not necessary to fix this operation slice.

### F3 — a non-UTF-8 plan output can fail after saving (low, follow-up)

At the corrective object, `crates/riauthctl/src/plans.rs:108-117` saves the plan,
then builds `json!({"plan_file": out})`. Directory planning calls this sequence
at `directory.rs:95-102,150-161`; provisioning planning uses it too. On Unix, a
Path can contain invalid UTF-8. Such a path is accepted by file I/O but cannot
serialize as a JSON string, and `json!` panics on a serialization error.
Thus the client can fail after server planning and private-file publication.
The backup command fixes the analogous problem with an early `out.to_str()`
check; these plan paths do not use it.

This is a static inferred edge case, not a product reproduction. It does not
bypass removal confirmation or publish credentials publicly. Apply the same
early validation, or return a fallible path summary, and cover refusal before
any request. It remains recorded rather than represented as fixed by `9471b9a`.

## Security and shared-writer review

**Request pinning and failure output.** `Remote` verifies the discovery issuer,
binds a saved session/agent to the primary management issuer, refuses foreign
management bases and redirects, and disables ambient proxies. The new operation
paths are primary `/api/...` paths and validate interpolated identifiers. HTTP
failures expose status and allowlisted error codes with fixed messages, rather
than reflecting arbitrary response descriptions. Success output still depends
on each operation's response validation; F2 is the concrete remaining gap.

**Removal and plan replay.** Desired-state, directory/cloud and SCIM apply
accept an exact plan ID for `--confirm-removals`; a mismatched ID is refused
before a request. The transport sends both removal confirmation headers only
when provided. Directory/cloud apply compares the private plan with the server
copy, allowing only the server's changed `applied` marker. Cloud apply checks
the provider kind. SCIM's retained-job fallback permits only a terminal job
matching plan, target and revision after the saved plan is unavailable.
Accepted `src/provisioning.rs` still reauthorizes the live caller and exact
target/owner before returning an existing job; that path is read-only and does
not repeat external delivery. No local fallback implies unconfirmed removal.
Plan/apply loops have a 1,024-page quota and tell the caller to resume when it
is exhausted. The `ee2b2ab` operation hunks refuse revision/idempotency flags
that plan-bound commands would otherwise silently ignore.

**Mutations and authority.** The new adapters enter the existing API routes
and Core/shared transaction writers for access, offboarding, directory and
provisioning. Direct writes carry a revision and idempotency key. PAM's
revision-fetch fallback omits the revision only for the server's explicit
403 revision-read refusal while the access adapter has enabled the local
`revision_optional` flag, preserving its bearer policy for approvers
without `state.read`; it does not reinterpret another failure as permission.
The browser's stricter requirement for both headers remains an existing M03
policy difference, not resolved by this client addition. The SSF shared writer
reauthorizes before receipt replay and keeps mutation, receipt and audit in
one store write. Client-side checks do not replace the server gates.

**Private files and cancellation.** Plan/session helpers use bounded reads
and Unix private-mode checks; new outputs use exclusive private temporary
files and hard-link publication without replacement. Backup creates a 0600
random partial beside the destination, probes hard-link support before
transfer, syncs received data, verifies it and then publishes with a no-replace
hard link. A destination created by another process is preserved. The
corrective commit rejects non-UTF-8 backup output before network/file work
and registers Unix SIGINT/SIGTERM/SIGHUP before creating the partial. Dropping
the operation removes the partial on a best-effort basis and cancellation
signals the blocking verifier's per-frame checks.

This is cooperative cancellation during transfer/verification, not a promise
that every signal wins against successful completion and synchronous
publication. There is no final cancellation check between the selected result
and the hard link. `Drop` ignores unlink errors, and probe cleanup can fail.
Therefore README wording that *only* SIGKILL or power loss can leave a partial
is too absolute; describe ordinary cleanup as best-effort. The inspected signal
tests stall the transfer, not verification or the publication boundary.
Only Unix code enforces 0600/private-mode bits; non-Unix code does not establish
Windows owner-only ACLs and handles only Ctrl-C from its first poll. No Windows
permission/signal result or Linux runtime result is claimed by this review.

**Archive bounds and parsing.** At the corrective object, the reader limits
plaintext frames to 8 MiB and archives to 4 GiB, with a caller-selectable lower
quota and a server header that can only lower it further. Length/quota checks
precede frame allocation. AES-256-GCM associated data binds format, stream,
index and kind; record names must be strictly ordered and non-repeated; the
trailer checks frame/record counts and the transcript; trailing bytes fail.
Typed header/record/trailer envelopes refuse unknown and repeated fields.
The recursive strict configuration visitor refuses repeated object keys at
any depth. Arbitrary record payloads remain `Value`, as in the server codec:
the duplicate-key claim must be scoped to envelopes/config, not every nested
record object. These byte bounds do not establish a total process-memory cap.
`verified: true` means authenticated framing/structure, not guaranteed restore
compatibility: typed server configuration and supported schema range remain
the restore writer's decisions. The transfer's per-read timeout is not a
whole-operation deadline.

**Credentials and zeroization.** SSF create checks the bounded private input,
ID and header before network requests; invalid/empty/non-string/control header
values get fixed errors. Create refuses the header field or an exact whole-string
secret echo; list refuses the header field. The corrective exact comparison
avoids false positives from a short secret appearing inside ordinary metadata.
This does not claim detection of an arbitrary transformed/concatenated secret
in an untrusted response. Backup output is metadata, with no raw archive bytes
or key printed by the normal server path. Encoded/decoded key buffers and
explicit SSF secret copies use `Zeroizing`/explicit wiping. HTTP body copies,
ordinary JSON Values and parser/crypto intermediates are not all zeroized.
In particular backup's `json!` request body contains an ordinary String copy
of `encryption_key`. Retain the corrective best-effort wording; do not assert
complete erasure. The serving host receives the backup key as part of the
existing export protocol and must be trusted accordingly.

## Evidence actually available

| Source-reported execution | Evidence scope |
| --- | --- |
| `RIAUTH_PG_TEST_TARGET=m03_riauthctl_backends_postgres scripts/test-postgres.sh` | Two ignored tests reported passed: the redb pair and all four storage modes, using source-built debug/test-support binaries and disposable loopback PostgreSQL 16 primary/standby. |
| After `bbbfd63`: `cargo test --features test-support --test m03_state_removal_e2e --test m03_pam_e2e -- --ignored` | One removal and one temporary-access real-binary test reported passed. |
| After `ee2b2ab`: riauthctl CI command twice; real removal test rerun | `m03_parity_ops` 10 reported passed, including connector tests outside this review; one real removal test reported passed with the exact refusal assertion. Workflow/LDAP reported runs do not release their holds. |
| After `7de8635`: riauthctl CI command twice; terminal-usb check; client clippy; `m03_backup_e2e -- --ignored` | SSF/backup mock suite 12 reported passed; check/clippy reported clean; one real backup/restore test reported passed. This predates `9471b9a`. |
| Corrective `9471b9a` definitions inspected | Added short-secret, invalid-header, non-UTF-8 backup-output and SIGTERM/SIGHUP cases, plus duplicate-field/config and framing fault cases. The pinned source report supplies no post-correction execution result; these are static test definitions here. |

The backend proof compares normalized selected outputs, revision and audit
counts for one login/user/group/reviewed-grant/agent/registration sequence. It
checks issued credentials against stored values, marker-only issuance receipts,
and encrypted PostgreSQL records. It does not establish parity for every
management operation, all concurrent races or all arbitrary stored secrets.
Its PostgreSQL cleanup targets only a UUID-named disposable database after
checking a private primary marker/data-directory and the published test role.
No test database was started by this review.

Directory/cloud, provisioning, offboarding and SSF coverage in the source report
is client-mock evidence. The real backup fixture shows one produced archive
accepted by offline restore; it does not prove all malformed archives, deployed
multi-node PostgreSQL, TLS/failover or external tenant behavior. Source-built
native binaries and synthetic peers are not official release artifacts or
real tenants. This review adds no installed-artifact, Windows-host, independent
reviewer or deployment/migration gate evidence.

## Reviewer checks and handoff

Performed: repository guidance and original task-detail reads; immutable
`git show`/`git ls-tree`/targeted diff inspection of the listed commits,
operation files, relevant test definitions and accepted server writers;
working-tree status checks. Documentation/diff verification results are recorded
below before commit. No product build, product test, browser scan, cloud write,
source edit, task/status change, merge or push was performed. W07 stayed untouched.

Only this report is authored in this worktree. Root owns integration, board
decisions and publishing. F1 and F2 need owner corrections with narrow evidence;
F3 and the documentation/zeroization/OS limits remain explicit. M07 workflow
API/CLI stays held until its live activation replay/sealing correction is
reviewed; connector stack decisions remain with the revisions lane. Broader
original M03 acceptance and artifact/interface coverage remain open.

Documentation verification: `python3 scripts/check-docs.py` passed
(`Markdown links and build-directory layout checked`). `git diff --cached
--check` passed; the staged diff contains only this new Markdown report.
No production checks were run by this reviewer.

## Correction follow-up at fa7511f

The original M03 task details and current worktree state were read again before
this follow-up. The acceptance and completion gate above still apply. All
implementation reads again used immutable objects; the management worktree's
mutable client files were not read or edited. The supplied accepted main was
used as a comparison object without merging it into this report-only branch.

| Input | Full pinned commit |
| --- | --- |
| Cumulative management snapshot | `fa7511f3ac8854df77bfd3a12d950eed2f1d9e73` |
| Accepted main | `148fafd4825c6cf803faf4ae869e3089e1462982` |
| Earlier review | `1210f6b50e3c18453ff4886d011ab7efd63dd785` |
| Initial corrective | `9471b9aa3b14c703b95f9a4848016c3cd091d05d` |
| Signal/privacy follow-up | `e2e5e8c64fc072c545ff8a8e4f1609f5aa3c8b4c` |
| F1 correction | `174215399559bd77aeb16fc95f26a52ca5b81d7e` |
| F2 correction | `07a754dfc9a19a2680a1b2896ce7abf1ae5b09b2` |
| Separately committed F3 correction | `a162347cf0b5d55d2957066fff26ca2145f6197e` |

F3 was committed while this review was being finished. Its full hash was
resolved before reading its six-file diff and relevant cumulative helpers.
That diff against `fa7511f` contains only the UTF-8 plan-output correction,
its focused test definition and README wording; it adds no workflow or
connector-status change. The port checklist keeps `fa7511f` as its baseline
and explicitly names the F3 hunks to add.

### Findings disposition and acceptance recommendation

| Source slice | Static recommendation | Evidence and residual |
| --- | --- | --- |
| Connector-independent `bbbfd63`, with `1742153` and `a162347` | ACCEPT corrected operation slice | F1 is corrected at cumulative `offboard.rs:121-135`: the same component count, byte bound and ASCII alphabet as accepted `src/offboarding.rs:75-89`. The new mock definition covers the 65-byte counterexample, three 64-byte components, reschedule and local malformed-input refusals. Include the separately reviewed F3 correction for plan output. |
| Operation-only `ee2b2ab` | ACCEPT specified operation hunks | Preserve the prior plan-bound flag refusal and exact removal/retained-job test corrections. Workflow/client-connector hunks remain held. |
| `7de8635` plus `9471b9a` plus `e2e5e8c` | ACCEPT cumulative backup/archive and SSF create/list | Keep all corrections together. Archive framing/config parsing is unchanged from reviewed `9471b9a`; early UTF-8 backup refusal, size limits and no-replace private publication remain. Unix INT/TERM cancellation and inherited HUP disposition are scoped below. |
| `07a754d` | ACCEPT SSF delete correction; release F2 hold | At cumulative `ssf.rs:82-100`, recursively refuse the known header field, validate `deleted: true` and the requested `stream_id`, then reconstruct only those two fields. Extra response fields cannot reach stdout. New definitions cover nested/mixed-case header refusal and allowlisted output. |
| `fa7511f` itself | Do not port as product code | It adds a workflow activation proposal, not an accepted replay correction. It is only the immutable review snapshot here. |
| `a162347` | ACCEPT F3 correction | Every LDAP/Workspace/Entra/SCIM and desired-state plan command checks UTF-8 output before requests or output checks/writes. The shared save helper also checks; summaries serialize a validated `&str` and return `Result` rather than panicking. The five-command Unix refusal and valid-summary definitions were inspected, not run. No client edits or duplicate tests were made by this reviewer. |
| Workflow/client-connector slices | HOLD | The replay API and live activation/sealing correction have not been accepted for this review. No workflow or connector-status acceptance is implied by operation acceptance. |

No additional blocking code defect was found within the corrected bounded
backup/SSF operation slice. This recommendation does not turn the original M03
task into done. Whole `bbbfd63`, `ee2b2ab`, `7de8635` or management-branch copies
are not approved as an integration method: use the port boundaries below and
include the separately reviewed F3 hunks.

The accepted and source server objects for `src/offboarding.rs`,
`src/management/ssf_streams.rs` and `src/provisioning.rs` have identical blob
IDs in the two snapshots. The current SSF shared delete writer still returns
`{"deleted":true,"stream_id":id}` at lines 483–503, after live authorization,
receipt replay/preconditions and one transaction. `id` is not the response key.
There was no observed server leak in either review.

The normal CLI emitter still does not generically mask `authorization_header`;
the corrected SSF operation guard and deletion allowlist provide that protection.
`e2e5e8c` strengthens `carries`: case-insensitive header-field keys, exact secret
keys/values and substring matches in values for secrets at least eight bytes.
It is a conservative heuristic: a short secret coinciding with an ordinary
property name can cause a refusal after creation, and arbitrary encodings or
transformed secrets are not covered. Fixed errors describe an uncertain already
committed create/delete without reflecting response bodies or secret values.
Best-effort zeroization and non-Unix private-permission limits remain as recorded
above; the crate retains `unsafe_code = "forbid"`.

One wording correction to the initial review is made above: PAM fallback tests
the HTTP 403 status when its local `revision_optional` flag is enabled; it does
not require a server error code named `revision_optional`. Other status failures
remain errors, and the server still authorizes the mutation independently.

### Signals and publication boundary

At cumulative `backup.rs:74,118-154`, registration occurs before partial-file
creation and covers SIGINT/SIGTERM only. No SIGHUP listener is installed. This
preserves the inherited ignore used by `nohup`; with the ordinary default HUP
disposition, a HUP can end the process and leave a private partial. The new
fixtures model both cases with an explicit HUP signal and use TERM to clean up
the surviving `nohup` transfer. They do not exercise a real terminal closure.

The prepublication `pending()` poll at lines 97–102 consumes an already visible
notification before the hard link. Tokio's signal API documents process-wide
handler installation and cancel-safe `recv`; the timeout implementation polls
its inner future before the timer, supporting a zero-duration poll for an
immediately ready notification. These are library facts, not host execution
evidence. [Tokio 1.53.1 Signal API](https://docs.rs/tokio/1.53.1/tokio/signal/unix/struct.Signal.html),
[Tokio timeout source](https://docs.rs/tokio/latest/src/tokio/time/timeout.rs.html).

Residual documentation condition for port: say "checks for a visible pending
notification before publication", rather than promising that every signal
arriving before the hard link wins. Notification delivery can lag, and a signal
can arrive after the last poll. The unit definition at `backup.rs:348-363`
loops/sleeps until the notification becomes visible; it does not establish
one-poll observation of every delivered OS signal. Publication and synchronous
cleanup remain outside cooperative polling, and unlink errors are ignored.
Describe ordinary partial cleanup as best-effort. This is a low contract-wording
limit, not a claim that an unverified archive is published or that an existing
destination is replaced. No Unix-host/Windows signal result was executed here.

### Bounded source-to-port checklist

Each row is a permissible source boundary for root, not an instruction to replace
an accepted cumulative file. Base the target on reviewed main `148fafd`; preserve
its accepted equivalents, CI/A03 and credential writers. Review any later target
delta before applying these hunks. Directory operations below operate the
already configured directory/provisioning routes; connector definition/export
and activation support are excluded.

| Destination | Pinned source boundary | Port requirement |
| --- | --- | --- |
| `crates/riauthctl/src/access.rs`, `directory.rs`, `offboard.rs`, `plans.rs`, `provision.rs` | New connector-independent operation modules from cumulative `fa7511f` (`bbbfd63`, operation helpers from `ee2b2ab`, corrected `1742153`), plus `a162347` in directory/plans/provision | Include exact confirmation, bounded plan paging, stored-plan matching, typed recovery input and PAM policy. `offboard.rs` must include F1 correction. Add F3's early output checks and fallible validated-string summary/callers together. |
| `crates/riauthctl/src/management.rs` | Only `bbbfd63` removal-confirmation signature/check/header hunks in `apply`, plus `a162347` UTF-8 check/string summary in `plan` | Retain other accepted `plan`/`export` and credential behavior. Exclude `ee2b2ab` connector-status extraction/helper and cumulative export changes. |
| `crates/riauthctl/src/transport.rs` | `bbbfd63` confirmation headers and optional-revision/plan methods; `7de8635` streaming client/`open_stream` hunks | Update all `RequestHeaders` constructors explicitly. Preserve primary-issuer/session binding, no redirects/proxies and bounded safe error handling. Streaming gets connect/read timeout, not a whole-transfer deadline. |
| `crates/riauthctl/src/archive.rs`, `backup.rs`, `ssf.rs` | Cumulative `fa7511f`, incorporating `7de8635` → `9471b9a` → `e2e5e8c`, and `07a754d` for SSF | Treat the corrections as a unit. Preserve framing/quotas, typed envelopes/strict config, no-replace publication, UTF-8 backup refusal, INT/TERM-only listeners, widened secret guard and two-field delete output. |
| `crates/riauthctl/src/main.rs` | Operation modules/enums/dispatch and apply confirmation from `bbbfd63`; pre-Remote flag check only from `ee2b2ab`; SSF/backup routing and 32-byte–4-GiB parser bound from `7de8635` | Exclude `mod workflow`, Workflow enum and dispatch. Preserve accepted command paths and emitter behavior. |
| `crates/riauthctl/Cargo.toml`, `Cargo.lock` | `7de8635` adds direct `aws-lc-rs`/`base64`, Tokio `signal` and its lock wiring | Reconcile only required dependency/feature entries; keep accepted locked graph changes and `unsafe_code = "forbid"`. The corrective commits add no dependency changes. |
| `crates/riauthctl/tests/m03_parity_ops.rs` | Operation fixtures/tests from `bbbfd63`, retained-job/flag corrections from `ee2b2ab`, timezone definition from `1742153`, non-UTF-8 plan definition from `a162347` | Include through the plan-bound refusal test, stopping before `export_server` (line 1670 at `fa7511f`, line 1807 with F3). Exclude connector export fixtures/tests after that boundary. |
| `crates/riauthctl/tests/m03_parity_ssf_backup.rs` | Cumulative `fa7511f`, from `7de8635` and `9471b9a`/`e2e5e8c`/`07a754d` | Include corrected archive faults, local input/privacy checks, nested/mixed-case deletion response cases and INT/TERM/default-HUP/nohup definitions. They are definitions, not this review's execution results. |
| `tests/m03_pam_e2e.rs`, `tests/m03_state_removal_e2e.rs`, `tests/m03_backup_e2e.rs` | PAM/removal tests from `bbbfd63`, exact refusal from `ee2b2ab`, real backup/restore test from `7de8635` | Preserve ignored real-binary setup and explicit evidence limits. Do not include workflow/connector e2e tests from mixed commits. |
| `crates/riauthctl/README.md`, `docs/capability-matrix.md`, `docs/operations.md` | Only command/confirmation/private-file/backup/SSF statements belonging to these operation slices and their corrections, including F3's plan-output sentence | Preserve accepted other-lane text, exclude workflow/client-connector claims, and scope pending-signal/cleanup and duplicate-key promises as above. Do not copy historical source-report closure assertions into product docs. |

Not in this port: `src/cli/workflows.rs`, server workflow API/assembly or sealing
changes, `crates/riauthctl/src/workflow.rs`, workflow mock/e2e files, connector
export/status code/tests, or the activation proposal added by `fa7511f`.
The already reviewed `4b9851a` backend-proof recommendation remains limited to
its two files and local evidence; it does not authorize the connector stack.
W07 implementation is untouched.

### Follow-up evidence and remaining gates

Pinned `fa7511f:docs/roadmap/local-wave28-management-report.md`, last edited by
`3d10007316fb4c93f56cd993e6638b0de3c27f16`, supplies newer historical evidence
than the initial review's wave27 report: after `9471b9a`, the worker reports
17 SSF/backup mock tests and one real backup/restore pass; after `e2e5e8c`, it
reports unit 1, SSF/backup 19, clean client clippy/fmt/terminal-usb check and one
real backup/restore pass. Those are source-reported results, not fresh checks
or independent log verification. The pinned report predates `1742153` and
`07a754d`; their new test definitions were inspected but no run result for
those commits or `a162347` is asserted here. Earlier SIGHUP-cleanup claims in that report
are historical and superseded by its own `e2e5e8c` section.

Performed in this follow-up: task/guidance/status reads, full-hash resolution,
immutable correction/cumulative-source reads, accepted-server blob comparisons,
focused patch/test-definition inspection and primary library documentation
review. No build or product test was run, and no Cargo target was used.
Only this Markdown report was edited. `git diff --check` and
`git diff --cached --check` passed; the staged scope is this report only.
No source, main, accepted worktree, task status,
merge or push was changed.

F3 is closed by static review of `a162347`. Low documentation limits, broader
interface/header policy decisions, official released artifacts and external acceptance remain
explicit. Root owns porting, validation and board reconciliation. Acceptance of
this static operation review is not whole-task or external completion.
