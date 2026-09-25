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

When the job runs it clears the local account and queues relying-party logout: `enabled` is cleared, `epoch` increases by one, and `logout::queue_user` fans out relying-party logout. Existing sessions and OAuth grants fail the epoch check. Temporary PAM grants are separate stored entitlements and retain their own expiry/revocation rules. The job records actions `user.disable`, `session.revoke`, `grant.revoke` and `downstream.local-only`. The shared disable transition also durably revokes child agents, Windows devices and outstanding sign-in tickets, and enqueues an outbound SSF account-disabled signal. Re-enabling the account does not restore those credentials.

The scheduler's authority is checked again at execution. A revoked/expired agent, an inactive administrator, a changed user id, or a user who is now the last administrator fails the job without disabling anyone. An agent-created job still refuses an administrator. The worker checks the agent's enabled flag, expiry, permission and parent-user state. A disabled or deleted parent rejects execution even for a legacy agent row still marked enabled.

Workers claim inside a write transaction from a due-time index that excludes completed jobs and unexpired leases. Each claim reads at most 128 due index entries; it does not scan or sort retained history. Existing schema-3 stores backfill this derived index at open, and restore rebuilds it. Cleanup processes at most eight jobs per pass; no production backlog deadline is implied. The claim sets `lease_owner` and `lease_until` to 60 seconds ahead and increments `attempts`. Another worker skips that lease. An expired lease can be reclaimed. Before committing revocation the worker reads the job again. Cancellation of a scheduled job wins immediately. Cancellation of a running job sets `cancel_requested`; if that flag is set, the worker marks the job cancelled and does not disable the user.

Retryable failures before revocation are retried at most five times, with backoff timestamps of 30, 60, 120 and 240 seconds. Authorization and identity failures can terminate the job immediately. The fifth failure is permanent (`status: failed`), is audited as `offboard.execute`, and leaves the user enabled. `last_error` is a short message with no secrets.

## Downstream SCIM

Offboarding does **not** call outbound SCIM, even when `[scim_targets]` is configured. There is no single-user deprovision function. Outbound deactivation is the existing reviewed plan: a disabled local user that already has a provisioning link is sent as `active: false` when an operator runs `riauth provision plan` and `riauth provision apply`. The job result is always `downstream: local-only`. `scim_targets_configured` only reports whether targets exist. Do not treat that field as a delivery receipt.

## API and CLI

Permission `user.offboard` on `user/<username>` is required. Humans must be administrators, as with other management calls.

```sh
riauth offboard schedule alice --execute-at 2027-03-01T00:00:00Z --timezone America/New_York
riauth offboard schedule alice --execute-at 1800000000 --timezone America/New_York
riauth offboard reschedule <job-id> --execute-at 2027-03-15T00:00:00+00:00 --timezone Europe/Berlin
riauth offboard cancel <job-id>
riauth offboard list
riauth offboard get <job-id>
```

HTTP:

- `POST /api/offboard/jobs` with `username`, `execute_at` and `timezone`
- `POST /api/offboard/jobs/{id}/reschedule` with `execute_at` and `timezone`
- `POST /api/offboard/jobs/{id}/cancel`
- `GET /api/offboard/jobs`
- `GET /api/offboard/jobs/{id}`

Running, done and cancelled jobs cannot be rescheduled. Cancelling a done or failed job conflicts. Cancelling an already cancelled job returns the job without a second audit event.

## Audit

`offboard.schedule`, `offboard.reschedule`, `offboard.cancel` and `offboard.execute` record the actor, the job id and the username (`<job-id>/<username>`). These actions bump the configuration revision. Job records and audit events do not contain passwords, tokens or SCIM credentials.
