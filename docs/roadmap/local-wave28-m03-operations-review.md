# Wave28 M03 operations review

Date: 2026-10-02. Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` (M03).
Review worktree: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`, starting HEAD
`88c50b025e5b3c65876944011b95fb05b30e08cd`.

Accept the local backend proof and the operation slices identified below.
Hold offboarding schedule/reschedule for a validation mismatch (F1), and SSF
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

## Integration matrix

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
403 `revision_optional` response, preserving its bearer policy for approvers
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
