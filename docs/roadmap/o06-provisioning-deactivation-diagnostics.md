# O06 provisioning deactivation diagnostics

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

Essentials and Platform both serve one redacted deactivation aggregate. It does not use `user.offboard`. A missing account is withheld for every caller, and the response does not include the target name.

## What an operator can read

`GET /api/operations/provisioning/deactivations` returns `riauth.provisioning-deactivation-diagnostics/v1`.

The caller needs `operations.read` on `operations/provisioning`. `operations.read` on `operations/health` or `operations/offboarding` does not pass. `provisioner.read` or `user.read` alone does not pass. Essentials and Platform both serve the route, on the same router as `GET /api/operations/provisioning`. `operations.read`, `provisioner.read`, and `user.read` are not platform-only actions. There is no `riauth` or `riauthctl` command for this read.

`GET /api/provisioning/deactivations` remains the detailed list. It still returns the stored target, stored username, `last_error`, URLs, and remote identifiers to a caller who can read that row. A missing account is omitted from that list. This aggregate does not replace that read.

Counts include every stored deactivation. The read scans `provisioning_deactivations` one storage page at a time. The page size is `store::maintenance::PAGE`, 128. The next page starts after the last key of the previous page. A page that does not advance that cursor is an internal error. A short page ends the scan, so a bucket whose length is an exact multiple of 128 takes one extra empty page read. The response retains at most 50 attention rows the caller may see, ranked by delivery severity and then id, and drops a worse row as soon as that list is full. `truncated` is true only when the visible attention count is greater than 50. Withheld rows stay in the counts and do not set `truncated`. `counts.attention` includes withheld rows. `counts.withheld` counts every row the caller may not see, including rows that are not attention. `counts.withheld_attention` is the attention subset.

Attention is the live delivery state `pending`, `failed`, `ambiguous`, or `dismissed`. `succeeded`, `resolved`, and `cancelled` are counted and omitted from `items`. Rows are sorted by severity and then id: failed delivery, ambiguous, dismissed, then pending.

Status counts (`pending`, `running`, `delivered`, `superseded`, `stale`, `failed`, `dismissed`) partition `counts.deactivations`. Delivery counts (`delivery_pending`, `delivery_failed`, `delivery_ambiguous`, `delivery_dismissed`, `delivery_succeeded`, `delivery_resolved`, `delivery_cancelled`) also partition that total. A `stale` status without uncertainty is `delivery_failed`, so its `next_action` is `inspect_and_replan`.

An item is visible only when `readable_user` returns the live account. That is the same check as `row_readable` on the detailed list: `provisioner.read` on `provisioner/<stored target>`, then a `users` row whose id is the stored `user_id`, then `user.read` on `user/<current username>`. The stored username is historical and is not an authorization key. A missing account returns no user, including for a full administrator, so the row is withheld and neither the stored username nor the target is copied. A row that fails `provisioner.read` is withheld in full. It is not listed with a hidden target, and this response has no `inspect_hidden_target` token. `user.offboard` is not consulted. Essentials still rejects `user.offboard` when an agent is created.

When the row is visible, `username` is the current username. `recorded_username_matches` is true only when that current name equals the stored name. The stored name itself is omitted. `hold` is one of `unlinked_create_requires_settlement`, `recovered_dispatch`, `awaiting_dispatch_ack`, `target_unconfigured`, `awaiting_controller`, `awaiting_controller_authority`, `awaiting_prior_delivery`, `manual_mode`, `removal_review_required`, `guarded_removal`, or `retry`. Any other hold is null and `hold_recognized` is false. An absent hold is null and `hold_recognized` is true. `outcome` is one of `deactivated`, `already_inactive`, `reviewed_delivery`, `remote_active`, or `remote_inactive`. Any other outcome is null. `has_error` is true when `last_error` is present, including an empty string. `remote_completion_verified` is false on every item. `has_unlinked_create` and `dispatch_recovery_count` are presence and length only.

`next_action` is a fixed token from the delivery state, status, known hold, and `has_error`: `attest_remote_state`, `waiver_is_not_remote_delivery`, `attestation_is_not_remote_delivery`, `delivery_record_expired`, `account_changed_before_delivery`, `inspect_and_replan`, `retry_or_replan_deactivation`, `review_provisioning_plan`, `restore_controller_authority`, `wait_for_provisioning_job`, `wait_for_dispatch_settlement`, `wait_for_retry`, `inspect_deactivation`, or `wait_for_deactivation`. The wording of a stored error does not select the token. Attention items are only the four states above, so the tokens for succeeded, resolved, and cancelled delivery are not listed.

`id` is the storage key, so the detailed read can be opened. A row created by deactivation delivery uses the digest of the link and the disabling epoch. A key written directly into the store is not checked again here. The aggregate does not copy `target`, `target_url`, `remote_id`, `external_id`, the stored username, `last_error`, link material, leases, actors, resolution, dismissal, dispatch-recovery payloads, or unlinked-create payloads.

`affects_readiness` is false. The body has no `healthy` field. `doctor.healthy`, `/readyz`, and `/livez` keep their existing answers. The read does not write an audit event, does not change queue indexes, and does not publish a Prometheus series. It does not claim or dispatch a deactivation.

Platform `GET /api/operations/offboarding/deactivations` is a different route. It remains on the Platform router, still requires `user.offboard` on the current username, and a full administrator can still see a missing account there as `account_present: false` with the stored target name. Essentials does not serve that route. `riauth offboard diagnostics` remains the job aggregate. See [deactivation diagnostics](o06-deactivation-diagnostics.md).

## Memory

The retained response is the fixed counters plus at most 50 redacted items. Each stored deactivation value is still decoded in full while its page is current. That value includes error text, URLs, remote identifiers, evidence, and recovery payloads. The store does not size-cap the value for this read, so one page can hold 128 large values. That page is released before the next page is read. The page cursor keeps one stored key. A row that passes `provisioner.read` reads one user record. That record is not accumulated. A listed item copies the current username and the storage key. It does not copy the target name. API-created names are 1–64 ASCII characters; a value written directly into the store is not re-capped here.

## What remains open

- A Grafana dashboard document at [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json). Setup and limits are in [operations](../operations.md). The file repeats the alert PromQL, has not been imported into Grafana, and leaves connector lag, key problems, and readiness unplotted.
- Connector lag. A schedule can store `last_completed_at`, the local unix time when its then-current `last_job` was stored completed. That time can remain after `last_job` changes. It is not a remote high-water mark and not downstream delivery completion. `next_run` still moves when a job is enqueued. This read does not report the field. It is on [reconciliation diagnostics](o06-reconciliation-diagnostics.md).
- Node mismatch, which remains [O03](coverage-inventory.md).
- Storage occupancy and key health have no shared store contract. The audit is [storage and key diagnostics](o06-storage-key-contract.md). `signing_errors` is not key health.
- The Platform deactivation read still uses `user.offboard` and can show a missing account's target name to a full administrator. That behavior stays on [deactivation diagnostics](o06-deactivation-diagnostics.md).
- Mail delivery status, which stays on `GET /api/operations/mail`. Logout delivery state, which stays on `GET /api/operations/logout`.
- `doctor`, `/readyz`, `/livez`, Prometheus, and Grafana. This slice adds no series and no dashboard JSON.
- `riauthctl`, and any new `riauth` diagnostics subcommand.
- Deactivation dispatch. This read leaves the write path unchanged. Stored error text and the target name stay on `GET /api/provisioning/deactivations`.
- A production deadline for walking every deactivation value. The retained list is at most 50 rows, and the read is paged, but each stored value is still decoded in full and has no size cap on this path.

## Local check

The focused test is `tests/provisioning_deactivation_diagnostics.rs`. It plants 132 deactivation rows, which is not a multiple of 128: one delivered row, 128 failed rows for one live account on one target, one failed row whose account is gone, one failed row for another account, and one failed row on another target. Stored names, errors, holds, outcomes, URLs, remote identifiers, leases, actors, dispatch recoveries, and unlinked creates contain URL and token text. The aggregate reports the current username, `recorded_username_matches`, `has_error`, a fixed `next_action`, and the storage key. It omits the target name. The detailed list still returns the planted `last_error` and target for a readable row and omits the missing account. On Platform, `GET /api/operations/offboarding/deactivations` still returns the missing account's target name to a full administrator. Essentials still rejects `user.offboard` at agent creation.

This worktree ran the redb test on the Platform feature set:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --test provisioning_deactivation_diagnostics -- \
  --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 1 ignored, finished in 2.26s. The test profile finished in 4m 15s. `/tmp/riauth-o06-target` was 3.3G afterward. Free space on `/System/Volumes/Data` was 14240680 KiB before the command and 11351940 KiB after. The linker printed `__eh_frame section too large` while linking the test binary; the test still passed.

The same redb test then ran on Essentials:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --no-default-features --features essentials \
  --test provisioning_deactivation_diagnostics -- \
  --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 1 ignored, finished in 2.03s. The test profile finished in 1m 11s. `/tmp/riauth-o06-target` was 4.6G afterward. Free space was 11376668 KiB before the command and 9905988 KiB after. That build printed existing unused-method warnings, including `allowed_by` in `src/session_protocol.rs`. The test still passed.

The ignored test then ran against a disposable PostgreSQL cluster under `/tmp/riauth-o06-pg.nB0QtD` (loopback trust, empty unix sockets, an ephemeral port, marker `riauth disposable contract cluster`). The Platform Cargo settings were used, with `RIAUTH_TEST_CONTRACT_PG_ROOT` set to that directory:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
RIAUTH_TEST_CONTRACT_PG_ROOT=/tmp/riauth-o06-pg.nB0QtD \
cargo test --locked --offline --test provisioning_deactivation_diagnostics \
  postgres_provisioning_deactivation_diagnostics_pages_exact_counts -- \
  --ignored --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 0 ignored, 1 filtered out, finished in 1.62s. The test profile finished in 0.53s. Free space was 9911792 KiB before the command and 9871380 KiB after. The linker printed `__eh_frame section too large` again; the test still passed. The cluster was then stopped with `pg_ctl -m immediate` and the directory was removed.
