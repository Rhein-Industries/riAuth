# O06 deactivation diagnostics

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

This page records the Platform deactivation-delivery read added here. It is a description of the current source and one local test, not a dashboard deployment or a change to deactivation dispatch.

## What an operator can read

Platform `GET /api/operations/offboarding/deactivations` returns `riauth.offboarding-deactivation-diagnostics/v1`.

The caller needs `operations.read` on `operations/offboarding`, the same resource as the scheduled-offboarding aggregate. `directory.sync`, `provisioner.sync`, `provisioner.read`, or `user.offboard` alone does not pass. `operations.read` on `operations/reconciliation` does not pass. There is no `riauth` or `riauthctl` command for this read. `riauth offboard diagnostics` remains `GET /api/operations/offboarding` and still returns `riauth.offboarding-diagnostics/v1`.

The route is on the Platform router. Essentials keeps the shared `provisioning_deactivations` rows and `GET /api/provisioning/deactivations`, which still returns the stored row to a caller who can read that account and target. A serving Essentials process does not expose this redacted aggregate.

Counts include every stored deactivation row. The response lists at most 50 attention rows the caller may see. `truncated` is true when that visible list is longer than 50. `counts.attention` includes withheld rows. `counts.withheld` counts every row the caller may not see, and `counts.withheld_attention` is the attention subset. The scan reads the whole `provisioning_deactivations` bucket. It is not the maintenance retention pass. Cleanup still removes a terminal row, other than a dismissed row or a row that still holds a lease, a dispatch recovery, or an unlinked create, 90 days after `next_attempt`.

Attention is the live delivery state `pending`, `failed`, `ambiguous`, or `dismissed`. `succeeded`, `resolved`, and `cancelled` are counted and omitted from `items`. Rows are sorted by severity and then id: failed delivery, ambiguous, dismissed, then pending.

Status counts (`pending`, `running`, `delivered`, `superseded`, `stale`, `failed`, `dismissed`) partition `counts.deactivations`. Delivery counts (`delivery_pending`, `delivery_failed`, `delivery_ambiguous`, `delivery_dismissed`, `delivery_succeeded`, `delivery_resolved`, `delivery_cancelled`) also partition that total. A `failed` status with a satisfied operator resolution is `delivery_resolved` and is omitted from `items`. A `stale` status without uncertainty is `delivery_failed`. `counts.failed` and `counts.delivery_failed` therefore describe different rows.

`counts.unreferenced` counts rows whose id is absent from every offboarding job's `result.downstream.targets[].delivery`. `counts.unreferenced_attention` is the attention subset. A referenced failed row is still listed. The job aggregate stays job-centric and does not gain these rows.

An item is visible when the account still exists and the caller has `user.offboard` on `user/<current username>` for that `user_id`. The stored username is historical and is not the authorization key. When the account exists, `username` is the current username and `recorded_username_matches` compares it with the stored name. The stored name is omitted when it differs. When the account is missing, a non-agent, non-delegated administrator sees `account_present: false` and neither username field. An agent, including one with `user.offboard=*`, leaves that row in `withheld`.

`target` requires `provisioner.read` on `provisioner/<target>`. A hidden target omits `target`, sets `target_hidden` true, and uses `next_action` `inspect_hidden_target`.

Each listed item reports `has_error` when a stored `last_error` is present, including an empty string. `hold` is one of `unlinked_create_requires_settlement`, `recovered_dispatch`, `awaiting_dispatch_ack`, `target_unconfigured`, `awaiting_controller`, `awaiting_controller_authority`, `awaiting_prior_delivery`, `manual_mode`, `removal_review_required`, `guarded_removal`, or `retry`. Any other hold is null and `hold_recognized` is false. A known hold selects `next_action` ahead of error presence. `outcome` is one of `deactivated`, `already_inactive`, `reviewed_delivery`, `remote_active`, or `remote_inactive`. `remote_completion_verified` is false on every listed item: a succeeded delivery is omitted. `referenced_by_offboard_job`, `has_unlinked_create`, and `dispatch_recovery_count` are presence and length only.

`next_action` is a fixed token: `inspect_hidden_target`, `attest_remote_state`, `waiver_is_not_remote_delivery`, `inspect_and_replan`, `retry_or_replan_deactivation`, `review_provisioning_plan`, `restore_controller_authority`, `wait_for_provisioning_job`, `wait_for_dispatch_settlement`, `wait_for_retry`, `inspect_deactivation`, or `wait_for_deactivation`. The wording of a stored error does not select the token.

The aggregate omits stored error text, target URLs, remote and external identifiers, link material, leases, actors, resolution and dismissal records, dispatch-recovery payloads, and unlinked-create payloads. `GET /api/provisioning/deactivations` still returns those fields to a caller who may read the row. `affects_readiness` is false. The body has no `healthy` field. `doctor.healthy`, `/readyz`, and `/livez` keep their existing answers. The read does not write an audit event, does not change queue indexes, and does not publish a Prometheus series. It does not claim or dispatch a deactivation.

## What remains open

- Dashboards.
- Connector lag. A schedule's `next_run` is the next enqueue time. This read does not measure it. Reconciliation controller failures stay on [reconciliation diagnostics](o06-reconciliation-diagnostics.md).
- Node mismatch, which remains [O03](coverage-inventory.md).
- Storage pressure and key problems.
- Provisioning-job error text, which stays on the provisioning job read.
- Mail deliveries, which stay on the mail read.
- `doctor`, `/readyz`, `/livez`, Prometheus, and Grafana. This slice adds no series and no dashboard JSON.
- `riauthctl`, and any new `riauth` diagnostics subcommand. The existing `riauth offboard diagnostics` command remains the job aggregate.
- An Essentials redacted deactivation aggregate. Essentials keeps the detailed deactivation list.
- Deactivation dispatch and the connector due cursor. This read leaves both on their existing write paths.
- A production deadline for scanning the whole deactivation bucket.

## Local check

`tests/offboarding_deactivation_diagnostics.rs::offboarding_deactivation_diagnostics_reports_incomplete_delivery_without_secrets` plants deactivation rows and one done offboarding job directly. It does not call the deactivation worker. Stored errors, holds, outcomes, URLs, remote identifiers, leases, actors, resolution evidence, dismissal evidence, dispatch recoveries, and unlinked creates contain URL and token text. One stored username is a URL for an account that has since been renamed, and one failed row names an account that is gone. The aggregate reports `has_error`, a fixed `next_action`, and the current username when the account is present. The detailed deactivation list still returns the planted `last_error`. The test checks that the aggregate contains neither those needles nor the stored-text fields, that status and delivery counts partition the bucket, that an operations-only agent sees counts and no names, that `user.offboard=*` still withholds the missing account, that a hidden target omits its name, that `provisioner.read` or `user.offboard` alone is `access_denied`, that the job aggregate stays on its own schema, that doctor stays healthy and gains no deactivation field, that queue pending and failed counts and the audit length stay unchanged across the read, that the HTTP body matches the Core body aside from `checked_at`, and that the 50-row cap holds.

This worktree ran that test with a private target:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --test offboarding_deactivation_diagnostics \
  offboarding_deactivation_diagnostics_reports_incomplete_delivery_without_secrets -- \
  --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 0 filtered out, finished in 3.92s. The test profile finished in 1m 11s. `/tmp/riauth-o06-target` was 3.7G afterward. Free space on `/System/Volumes/Data` was 21215168 KiB before the command and 20979116 KiB after. `python3 scripts/check-docs.py` reported that markdown links and the build-directory layout were checked. The linker printed `__eh_frame section too large` while linking the test binary; the test still passed.

The existing job-aggregate test was run again on the same private target, with the same Cargo settings:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --test offboarding \
  offboarding_diagnostics_reports_incomplete_and_failed_without_secrets -- \
  --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 29 filtered out, finished in 2.85s. The test profile finished in 4.43s. The linker printed the same `__eh_frame` warning. The full `tests/offboarding.rs` suite was not run.
