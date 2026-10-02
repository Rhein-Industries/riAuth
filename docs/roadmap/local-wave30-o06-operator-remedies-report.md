# O06 operator remedies: docs slice evidence

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, O06 task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6`. This report covers the slice
committed on worktree `9c54c023-a95f-48e9-a908-6f97a5677c98`, branch
`roadmap/local-management-wave27`.

The slice is documentation only, as root assigned after the
[independent review](local-wave30-o06-independent-review.md). It covers:

- finding 5: action-token remedies and stale wording;
- finding 6: the `security-agreement-record` command correction only;
- finding 4: the `doctor` overstatement only.

O06 stays open. Lane lead: Claude Opus 5.5. One read-only Sonnet reviewer
checked the change and reported `claude-sonnet-5-5`.

## Commit and baseline

| Commit | Content |
| --- | --- |
| `31837b6cd06be545ec4301b2f4285aa8456c1eaa` | The seven documentation edits below |
| this report | This file only, in its own commit |

**Baseline.** Before editing, the seven target files were byte-identical to
published `da5ff7dcfc3442c302955344229168872911b0ec`. There was no merge or
reset. The patch ID of `31837b6` equals the patch ID of
`git diff da5ff7d HEAD` over those files, so the change applies unchanged to
the published baseline.

**Source.** Every fact was read from Git objects at `da5ff7d`.

## Changes

| File | Edit |
| --- | --- |
| `docs/operations.md` | New section `## Diagnostic next actions`, placed after "Diagnostics and recovery" and before "Background capacity and overload". It is separate from the storage and key paragraphs and adds three tables, described below. |
| `docs/connector-incidents.md` | New section `## Reconciliation controllers`, placed before `## LDAP`, described below. |
| `docs/availability.md` | Only the `security-agreement-record` sentence. It now gives `--confirm-authentication-policy --confirm-rate-limits` (`src/cli/local.rs:361-366`), says the command writes format 3 from the four integers plus effective HTTP rate limits (`FORMAT = 3`, `src/node_security.rs:28`), and says the earlier release refuses format 3. The stop-every-process, backup and rollback wording is unchanged. |
| `docs/disaster-recovery.md` | Only the Administrator row. The row no longer claims "signing-key health". `doctor` reports the schema, the enabled-administrator count behind `healthy`, the active signing key id once the stored active key yields its public JWK, and whether `database_key_file` is configured (`src/operations.rs:82-111`, `src/crypto.rs:415-425`). It checks no remote signer, other signing domain or key file contents, and the next row proves signing. |
| `docs/deactivation-delivery.md` | Only the edition sentence. Both editions serve the shared redacted aggregate `GET /api/operations/provisioning/deactivations` on `operations.read` for `operations/provisioning` (`src/api.rs:589-592`, before the Platform merge). The Platform route is the offboarding-scoped aggregate. |
| `docs/roadmap/o06-offboarding-diagnostics.md` | Only the riauthctl sentence. `riauthctl offboard diagnostics` reads the same route, alongside `list`, `get`, `schedule`, `reschedule` and `cancel` (`crates/riauthctl/src/offboard.rs:17-44,64`). |
| `docs/roadmap/o06-offboarding-overdue.md` | Only the caller name. `Core::cleanup`, through `cleanup_pass`, calls `crate::offboarding::cleanup` (`src/core.rs:912-925,1042`). `Core::maintenance` does not exist. |

### The three tables in `docs/operations.md`

Each table has the columns token, emitted when, read the cause, already true,
and action.

- **Reconciliation** (`GET /api/operations/reconciliation`):
  - tokens: `inspect_connector_and_replan`, `refresh_authority_and_replan`,
    `wait_for_retry`, `wait_for_worker`, `inspect_controller` and
    `check_worker_duty`;
  - a note that `review_local_result` is never an attention row.
- **Scheduled offboarding jobs** (`GET /api/operations/offboarding`):
  - tokens: `inspect_local_failure`, `wait_for_local_retry`,
    `wait_for_worker`, `check_worker_duty`, `inspect_hidden_targets` and
    `confirm_waiver_not_delivery`;
  - a pointer for the job-level deactivation tokens.
- **Outbound provisioning jobs** (`GET /api/operations/provisioning`):
  - tokens: `inspect_provisioning_job`, `review_ambiguous_delivery` and
    `wait_for_retry`.

### The new section in `docs/connector-incidents.md`

- the reconciliation reads and their scope permissions;
- what still answers during an incident;
- the fact that a failed or stale job is not retried, while the next due run
  enqueues a new one;
- a six-step procedure;
- links to the connector sections.

## Source behind the tables

Reconciliation, from `src/reconciliation.rs` at `da5ff7d`:

- **Token selection:** `job_next_action` and `job_needs_attention`
  (`:546-563`), and `schedule_item` (`:645-653`).
- **Attempts:** `MAX_ATTEMPTS = 4` (`:32`), with backoff `30 << (attempts-1)`
  (`:1366-1368`).
- **Failure and staleness:**
  - `failed`, including after the lease expires on the final attempt
    (`:1172-1174`);
  - `stale` for `access_denied`, `invalid_token`, `not_found` or a
    reconciliation conflict (`:1360-1363`);
  - `stale` when configuration or authority is invalidated (`:1024-1045`).
- **Scheduling:**
  - the scheduler enqueues only when no job is queued or running for the
    scope and fingerprint (`:1082-1090`), and the job id is random;
  - `scoped_agent` (`:368-385`), and `ensure_capacity` with "Too many active
    reconciliation jobs" (`:718`), retried after 30 seconds (`:1111-1113`).
- **Reads:**
  - scoped reads are filtered by `action_resource` (`:327-341`, `:902-929`);
  - the event route accepts only the controller agent's own token
    (`:874-886`).

Offboarding:

- **Token selection:** `job_next_action` (`src/offboarding.rs:507-574`),
  `needs_attention` (`:482-492`) and `target_next_action` (`:419-453`).
- **No cancel or reschedule:** `offboard_cancel` refuses `failed`
  (`:993-995`). `offboard_reschedule` accepts only `scheduled`
  (`:966-967`). A failed job is not `active()`
  (`src/offboarding_types.rs:48-50`), so a new schedule is allowed
  (`src/offboarding.rs:907-916`).
- **Exhaustion:** `finalize_exhausted` writes "Offboarding stopped after 5
  attempts" (`:1198-1201,1352-1359`).
- **Already-disabled users:** `guard_schedule` does not refuse a disabled user
  (`:194-199`).
- **Commands:** `riauth offboard get` maps to `GET /api/offboard/jobs/{id}`
  (`src/cli.rs:2502-2506`). `riauth user disable` sends `enabled: false` and
  needs the global `--idempotency-key` and `--if-revision`
  (`src/cli.rs:136-143,2640,2679-2681`).
- **Docs agreement:** the fifth failure is permanent, the user stays enabled,
  and the lease is 60 seconds (`docs/enterprise/ENT-10.md`).

Provisioning:

- **Token selection:** `provisioning_attention_action`
  (`src/provisioning.rs:497-504`).
- **Delivery states:** the `docs/scim.md` delivery outcome table.
- **Resolve:** `provisioning_resolve` (`src/provisioning.rs:1220-1255`):
  - it needs `provisioner.sync` and read access to the item;
  - it applies only to a stopped job whose item is ambiguous;
  - it conflicts while the item is still in flight;
  - it makes no remote request.
- **Commands:** `riauth provision plan <target> --out`,
  `apply --plan [--confirm-removals]`, `jobs`, `stop` and
  `resolve --observed --evidence` (`src/cli.rs:578-612,1750-1754`).

## Rules kept

- **No cancel or reschedule for failed jobs.** No failed offboarding job is
  told to be cancelled or rescheduled. The table says it cannot be either,
  stays listed, and keeps `riauth_queue_failed{queue="offboard_jobs"}` above
  zero.
- **No unverified delivery.** A waiver, an attestation and missing proof are
  never presented as verified delivery. `confirm_waiver_not_delivery` keeps
  the job `incomplete`.
- **No agreement bypass.** The only node-policy text is the reviewed offline
  `security-agreement-record` command, kept within its existing stopped,
  backed-up and rollback scope.
- **Nothing invented.** No service, log line, metric, remote health check or
  command was added. Every command and route named exists at `da5ff7d`.

## Independent review

A read-only Sonnet reviewer (`claude-sonnet-5-5`, Git objects only, no build
or remote call) checked the uncommitted diff against `da5ff7d`. It found no
hard-rule violation and no invented surface.

Its findings and how each was resolved before the commit:

1. **`riauth user disable` was missing its required global flags.** Fixed by
   giving the full command, `riauth --idempotency-key <key> --if-revision
   <revision> user disable <username>`, with the revision from
   `riauth revision`.
2. **"Jobs already queued still run" was overstated for `inspect_controller`.**
   Now limited to the capacity conflict. After an agent failure, that
   controller's queued and running jobs have gone `stale`.
3. **`inspect_local_failure` missed the lease-exhaust failure.** Added
   "its 60-second lease expired after the fifth attempt", and the
   already-true cell now says "the job disabled nothing".
4. **Not every offboarding job-level token was covered.** Added rows for
   `inspect_hidden_targets` and for the job-level deactivation tokens, and
   added the hidden-target condition to `confirm_waiver_not_delivery`.
5. **"Items it delivered stay delivered" overstated what riAuth knows.** Now
   "riAuth does not undo items it delivered before the stop".
6. **`provision resolve` lacked its preconditions.** Added the permission,
   item read access and the in-flight conflict.
7. **The `check_worker_duty` already-true cell was vacuous.** It now states
   that no completion was recorded within the limit, and that the clock is
   local controller time, not remote lag.
8. **A table cell held an escaped pipe inside a code span.** Replaced with
   prose naming the three `--observed` values.

Points the reviewer reported but that were not changed:

- **No production path stores the error those tokens need.** The reviewer
  found no production pass in this commit that leaves a stored error on a
  `scheduled` or `running` offboarding job; only tests drive that path. The
  table states each token's emission condition, which matches the code, so
  those rows were kept as written.
- **`availability.md` still describes only the four integers.** The rest of
  that paragraph does not mention the rate limits compared at open. That text
  is outside this slice's single-sentence correction, so it was left for
  root.

## Checks actually run

| Check | Result |
| --- | --- |
| `python3 scripts/check-docs.py` | passed, before and after the review fixes, and for this report |
| `git diff --check` | clean |
| Table shape | each of the 15 rows of the three new tables has five cells |
| Anchors | `operations.md#diagnostic-next-actions`, `connector-incidents.md#what-still-answers`, `#ldap`, `#outbound-scim`, `#workspace-and-entra` and `scim.md#delivery-outcomes` resolve to real headings |
| Patch identity | `31837b6` equals the diff from `da5ff7d` over the seven files |

**Not done:** no Rust, Cargo, test, build, service or desktop work; nothing
to main; no push; no status, task, worktree or cache-deletion change.

## Limits and overlap

- **Remedy tables only.** These tables document existing remedies; none of
  the review's code findings 1–4, 6 or 7 is fixed by documentation.
- **Scope of findings 3 and 6.** The never-started controller case
  (finding 3) is not documented here; that belongs to the finding 3 code
  slice. The `/readyz` cause (finding 6) is also left to its code slice.
- **Overlap with other docs work.** Root owns integration and any overlap
  with the Sol lane's documentation, including the storage and key paragraphs
  of `docs/operations.md`.
