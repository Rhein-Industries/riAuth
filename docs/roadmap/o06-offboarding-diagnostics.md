# O06 offboarding diagnostics

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

This page records the scheduled-offboarding read added here. It is a description of the current source and one local test, not a dashboard deployment or a live directory outage.

## What an operator can read

Platform `GET /api/operations/offboarding` and `riauth offboard diagnostics` return `riauth.offboarding-diagnostics/v1`.

The caller needs `operations.read` on `operations/offboarding`. Counts include every stored offboarding job. An attention item is included when that caller also has `user.offboard` on `user/<stored username>`. Jobs outside that permission stay in `counts.withheld` and `counts.withheld_attention` and contribute no username. Target names also require `provisioner.read` on `provisioner/<target>`. A hidden target still decides `downstream_state`. When any target is hidden, `next_action` is `inspect_hidden_targets`.

Attention items are:

- `status: failed`
- `status: done` with live downstream state `pending` or `incomplete`
- `status: scheduled` or `running` when `last_error` is already set

Cancelled jobs, future jobs without an error, done jobs that are `delivered` or `resolved`, and done jobs with no recorded targets are counted and omitted from `items`. The response lists at most 50 items, sorted by severity and then job id, and at most 32 non-succeeded visible targets on each item. `truncated` and `targets_omitted` report the remainder. The scan reads the whole `offboard_jobs` bucket; there is no incomplete-downstream index.

`status: done` means the local revocation committed. `remote_completion_verified` is true only when every recorded target's live state is `delivered`. `resolved` counts an operator attestation and stays separate from `delivered`. `affects_readiness` is false. `doctor.healthy`, `/readyz`, and `/livez` keep their existing answers.

The item omits the stored job result, dismissal and resolution records, target URLs, remote identifiers, external identifiers, lease owners, and hold text outside these tokens: `unlinked_create_requires_settlement`, `recovered_dispatch`, `awaiting_dispatch_ack`, `target_unconfigured`, `awaiting_controller`, `awaiting_controller_authority`, `awaiting_prior_delivery`, `manual_mode`, `removal_review_required`, `guarded_removal`, and `retry`. An unknown hold is null and `hold_recognized` is false. Each attention item and each visible target reports `has_error` when a stored `last_error` is present. The aggregate leaves that text, including a URL or token, on the per-job read. `outcome` is one of `deactivated`, `already_inactive`, `reviewed_delivery`, `remote_active`, or `remote_inactive`.

`next_action` is a fixed token chosen from status, downstream state, known holds, and error presence. The token set includes `inspect_local_failure`, `wait_for_local_retry`, `wait_for_worker`, `confirm_waiver_not_delivery`, `review_provisioning_plan`, `attest_remote_state`, and `inspect_deactivation`. The wording of a stored error does not select the token. Per-job `GET /api/offboard/jobs/{id}` still returns the stored `last_error` and the full downstream record for a caller who may read that job. This aggregate is the redacted view.

A serving Essentials process cannot retain `offboard_jobs`. The route is on the Platform router. `riauthctl` has no offboarding command.

The read does not write an audit event. It is not on the maintenance hot path and it does not publish a Prometheus series.

## What remains open

- Dashboards.
- Connector lag. A schedule's `next_run` advances when a job is enqueued, which is not a measure of sync completion.
- Node mismatch, which remains [O03](coverage-inventory.md).
- Storage pressure and key problems.
- Failed jobs outside scheduled offboarding. Provisioning jobs and mail deliveries keep their existing reads. Reconciliation controller failures have a separate redacted aggregate, [reconciliation diagnostics](o06-reconciliation-diagnostics.md). This offboarding read does not list them.
- `doctor` and `queues.offboard_jobs.failed`. The failed queue counts `status: failed` only, so a done job with incomplete downstream work is still absent there.
- Deactivation rows that no offboarding job references. This read does not walk `provisioning_deactivations` on its own.
- A production backlog deadline for the full job scan.

## Local check

`tests/offboarding.rs::offboarding_diagnostics_reports_incomplete_and_failed_without_secrets` builds one future job, one local failure after five retryable attempts, one waived downstream target, one verified delivery, one attestation, one local retry, and one held target. Stored job and deactivation errors are then replaced with URL and token text. The aggregate reports `has_error`, and the per-job read still returns that text. The test checks that the aggregate body contains neither the planted URL or token nor a `last_error` field. It also checks the count partitions, withheld items for an operations-only agent, hidden target names, `access_denied` without `operations.read`, unchanged `offboard.execute` audit count, unchanged doctor health, the offboard queue's failed count, and the 50-item cap. Cloned failed jobs carry the same planted text and stay inside that check.

This worktree ran that test with a private target:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --test offboarding \
  offboarding_diagnostics_reports_incomplete_and_failed_without_secrets -- \
  --test-threads=1
```

Result: `ok`. 1 passed, 0 failed, 29 filtered out, finished in 2.96s. The test profile finished in 1m 14s. `/tmp/riauth-o06-target` was 3.3G afterward. `python3 scripts/check-docs.py` reported that markdown links and the build-directory layout were checked. The linker printed `__eh_frame section too large` while linking the test binary; the test still passed.
