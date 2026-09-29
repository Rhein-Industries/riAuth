# ENT-10 Scheduled user offboarding

[Implementation](../../src/offboarding.rs) and [tests](../../tests/offboarding.rs).

Offboarding schedules local revocation of one user. The job is stored in `offboard_jobs` before the API returns, and `Core::cleanup` (the server maintenance pass, about once a minute) runs it when `execute_at` is due. A restart does not drop the job.

Human administrators can schedule, reschedule, cancel, list and get jobs. An agent needs `user.offboard` on `user/<username>` (or `user.offboard=*`). Agents cannot offboard administrators. The last enabled administrator cannot be offboarded. A user can have only one scheduled or running job.

## Time contract

`execute_at` is an absolute Unix timestamp. Two request forms are accepted:

- RFC3339 with an explicit numeric offset or `Z`, such as `2027-03-01T00:00:00Z` or `2027-03-01T02:00:00+02:00`. Subseconds are truncated to the Unix second.
- A JSON integer number of Unix seconds. The CLI sends an all-digit `--execute-at` value as that integer.

Naive local times (`2027-03-01T00:00:00`) and numeric strings are rejected. The timezone label is not consulted to interpret them.

`timezone` is required. It must match `[A-Za-z0-9_+-]{1,64}(/[A-Za-z0-9_+-]{1,64}){0,2}` and is stored exactly as typed. riAuth does not ship a time zone database (`time` is built with formatting and parsing only), so the label is audit information. `UTC`, `Etc/GMT+1` and similar strings are labels, not conversions. Numeric offsets such as `+02:00` are not valid labels. Reschedule replaces both `execute_at` and `timezone` and keeps the job id. Changing the label alone does not move the instant, because the new request must supply the absolute instant again.

`execute_at` must be after the current time and no later than 366 days ahead.

## Execution

When the job runs, one transaction revokes local access and records downstream intent. `enabled` is cleared and `epoch` increases by one, so existing sessions and OAuth grants fail the epoch check. Unexpired temporary access grants, including future-dated ones, are revoked with the job's creator as `revoked_by`. The shared disable transition durably revokes child agents, Windows devices and outstanding sign-in tickets. It also queues relying-party logout, enqueues an outbound SSF account-disabled signal, and records one outbound SCIM deactivation row per linked target. Re-enabling the account does not restore those credentials. New jobs record the actions `user.disable`, `session.revoke`, `grant.revoke` and `downstream.deactivate`; stored jobs keep their original list.

The scheduler's authority is checked again at execution. A revoked/expired agent, an inactive administrator, a changed user id, or a user who is now the last administrator fails the job without disabling anyone. An agent-created job still refuses an administrator. The worker checks the agent's enabled flag, expiry, permission and parent-user state. A disabled or deleted parent rejects execution even for a legacy agent row still marked enabled.

Workers claim inside a write transaction from a due-time index that excludes completed jobs and unexpired leases. Each claim reads at most 128 due index entries; it does not scan or sort retained history. Existing schema-3 stores backfill this derived index at open, and restore rebuilds it. Cleanup processes at most eight jobs per pass; no production backlog deadline is implied. The claim sets `lease_owner` and `lease_until` to 60 seconds ahead and increments `attempts`. Another worker skips that lease. An expired lease can be reclaimed. Before committing revocation the worker reads the job again. Cancellation of a scheduled job wins immediately. Cancellation of a running job sets `cancel_requested`; if that flag is set, the worker marks the job cancelled and does not disable the user.

Retryable failures before revocation are retried at most five times, with backoff timestamps of 30, 60, 120 and 240 seconds. Authorization and identity failures can terminate the job immediately. The fifth failure is permanent (`status: failed`), is audited as `offboard.execute`, and leaves the user enabled. `last_error` is a short message with no secrets.

## Downstream SCIM

The job never calls a SCIM target inside its commit, and `status: done` means only that the local revocation committed. The same transaction records a [deactivation row](../scim.md#offboarding-deactivation) for each outbound link whose target last reported the account active. That includes an account that was already disabled when the job ran. The job result stores the local outcome in `result.local` and the row IDs in `result.downstream.targets`. Job views then add `downstream`, which reads each row live:

| `downstream.state` | Meaning |
| --- | --- |
| `pending` | At least one target has no outcome yet, including rows held for a controller or review |
| `delivered` | Every recorded target confirmed that the account is inactive |
| `resolved` | Every target was delivered, or an operator attested with evidence that nothing is left active (`delivery_state: resolved`). riAuth did not verify those targets. |
| `incomplete` | No target is pending and at least one is `stale`, `failed`, `superseded`, `dismissed` (operator waiver) or `expired` (retention elapsed) |

Each entry in `downstream.targets` shows that target's `delivery_state`, `status`, `hold`, `outcome`, `attempts`, `last_error`, `delivered_at`, `uncertain`, `resolution` and `dismissal`. A dismissal preserves any original ambiguity and never counts as remote completion. An agent sees an entry only with `provisioner.read` on `provisioner/<target>`. Hidden targets are counted in `hidden_targets` and still decide `state`, so a hidden pending target never reads as delivered. If the account had no active outbound link, `result.downstream.targets` is empty and `downstream` is omitted. Jobs completed by earlier releases keep `downstream: local-only`.

A target delivers automatically only when it has a `scim/<target>` scoped controller and `automatic` reconciliation mode, below the shared removal floor. Otherwise the row is held until a reviewed `riauth provision plan` / `provision apply` delivers the disable; the row then closes as `delivered`.

## API and CLI

Permission `user.offboard` on `user/<username>` is required. Humans must be administrators, as with other management calls.

```sh
riauth offboard schedule alice --execute-at 2027-03-01T00:00:00Z --timezone America/New_York
riauth offboard schedule alice --execute-at 1800000000 --timezone America/New_York
riauth offboard reschedule <job-id> --execute-at 2027-03-15T00:00:00+00:00 --timezone Europe/Berlin
riauth offboard cancel <job-id>
riauth offboard list
riauth offboard get <job-id>
riauth offboard diagnostics
```

HTTP:

- `POST /api/offboard/jobs` with `username`, `execute_at` and `timezone`
- `POST /api/offboard/jobs/{id}/reschedule` with `execute_at` and `timezone`
- `POST /api/offboard/jobs/{id}/cancel`
- `GET /api/offboard/jobs`
- `GET /api/offboard/jobs/{id}`
- `GET /api/operations/offboarding` for the redacted job aggregate; `operations.read` on `operations/offboarding`
- `GET /api/operations/offboarding/deactivations` for the redacted deactivation-delivery aggregate; the same permission

Running, done and cancelled jobs cannot be rescheduled. Cancelling a done or failed job conflicts. Cancelling an already cancelled job returns the job without a second audit event.

## Operator diagnostic

`GET /api/operations/offboarding` and `riauth offboard diagnostics` are a Platform read. A serving Essentials process cannot retain `offboard_jobs`, so the route is on the Platform router. Permission is `operations.read` on `operations/offboarding`. Counts include every job. Attention items include jobs the caller may `user.offboard`, and omit target names the caller may not `provisioner.read`. The state uses the same classification as `downstream.state`. The diagnostic omits the job result, dismissal and resolution evidence, target URLs, remote identifiers, lease owners, and hold text outside the known hold tokens. Each attention item and visible target reports `has_error` when a stored `last_error` is present. The aggregate leaves that text on the job read. `next_action` uses that presence and names the follow-up. `remote_completion_verified` is true only when `downstream_state` is `delivered`. `affects_readiness` is false. Doctor and `queues.offboard_jobs.failed` still count a failed job by `status: failed`; a done job with incomplete downstream work is visible on this read. The scan covers the offboard job bucket. The response lists at most 50 jobs and 32 non-succeeded visible targets per job. See [offboarding diagnostics](../roadmap/o06-offboarding-diagnostics.md).

`GET /api/operations/offboarding/deactivations` is the row-level companion on the same permission. It pages the deactivation bucket 128 rows at a time, does not load offboarding jobs or report job linkage, counts every stored deactivation, including a row no offboarding job records, and lists at most 50 redacted attention rows for `pending`, `failed`, `ambiguous`, or `dismissed` delivery. Item visibility uses `user.offboard` on the account's current username. The stored username is not the authorization key. A hidden target omits its name. `has_error` records a stored error, and the text stays on `GET /api/provisioning/deactivations`. `affects_readiness` is false. Essentials does not serve this route. Both editions serve `GET /api/operations/provisioning/deactivations` on `operations.read` for `operations/provisioning`. That read uses `provisioner.read` and `user.read`, withholds a missing account, and omits the target name. See [deactivation diagnostics](../roadmap/o06-deactivation-diagnostics.md) and [provisioning deactivation diagnostics](../roadmap/o06-provisioning-deactivation-diagnostics.md).

## Audit

`offboard.schedule`, `offboard.reschedule`, `offboard.cancel` and `offboard.execute` record the actor, the job id and the username (`<job-id>/<username>`). These actions bump the configuration revision. Job records and audit events do not contain passwords, tokens or SCIM credentials. Neither diagnostic read records an audit event.
