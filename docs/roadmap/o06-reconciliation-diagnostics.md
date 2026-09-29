# O06 reconciliation diagnostics

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

This page records the reconciliation-controller read added here. It is a description of the current source and one local test, not a dashboard deployment or a measured connector lag.

## What an operator can read

`GET /api/operations/reconciliation` returns `riauth.reconciliation-diagnostics/v1`.

The caller needs `operations.read` on `operations/reconciliation`. `directory.sync` or `provisioner.sync` alone does not pass. Essentials and Platform both serve the route. Workspace and Entra controllers still require the Platform build; an Essentials store can report LDAP and SCIM controller rows that it retained.

Counts include every stored schedule and every retained reconciliation job. The service keeps at most 256 jobs. The response lists at most 50 attention rows. `truncated` is true when more rows need attention. There is no per-user withholding: a controller scope is the identity of the failure, and an operations reader sees it without that connector's sync permission.

Attention rows are:

- a schedule with `last_error` present, including an empty string
- `status: failed`
- `status: stale`, except a job whose stored error is exactly the fixed sentence written when a schedule is disabled before dispatch
- `status: queued` or `running` when `last_error` is already set

Completed jobs, disabled schedules with no stored error, and queued or running jobs with no stored error are counted and omitted from `items`. A stale job created by disabling a schedule before dispatch stays in `counts.stale` and is omitted. The response does not copy that sentence. Rows are sorted by severity and then id: failed jobs, other stale jobs, queued or running retries, then schedules.

Each row sets `record` to `schedule` or `job`. A schedule row has `scope`, `enabled`, `interval_seconds`, `next_run`, `last_job`, `agent_id`, `has_error`, and `next_action`. A job row has `id`, `scope`, `status`, `origin`, `attempts`, `next_attempt`, `has_error`, and `next_action`. `has_error` is true when a stored `last_error` is present. `next_action` is one of `inspect_controller`, `inspect_connector_and_replan`, `refresh_authority_and_replan`, `wait_for_retry`, or `wait_for_worker`. The wording of a stored error does not select the token.

`next_run` is the next time the scheduler enqueues a periodic job. It is not a measurement of how long a sync took or how far a connector is behind. This read does not report connector lag.

The aggregate omits stored error text, `last_outcome`, job `outcome`, authority bindings, lease owner and deadline, actor, configuration fingerprint, and credential paths. `GET /api/reconciliation/schedules` and `GET /api/reconciliation/jobs` still return those fields to a caller who may sync that controller. `affects_readiness` is false. `doctor.healthy`, `/readyz`, and `/livez` keep their existing answers. The read does not write an audit event, does not publish a Prometheus series, and has no `riauth` or `riauthctl` command.

## What remains open

- Dashboards.
- Connector lag. `next_run` advances when a job is enqueued, which is not sync completion, and this read does not add a completion time.
- Node mismatch, which remains [O03](coverage-inventory.md).
- Storage pressure and key problems.
- Provisioning job errors, which stay on the provisioning job read, including its stored error text.
- Mail deliveries, which stay on the mail read.
- Deactivation delivery. Incomplete and failed rows, including rows no offboarding job records, are [deactivation diagnostics](o06-deactivation-diagnostics.md). That read pages the deactivation bucket and does not load offboarding jobs.
- The separate Platform offboarding job aggregate. This reconciliation read does not extend it.
- `doctor`, `/readyz`, `/livez`, Prometheus, and Grafana. Controller failures do not change those answers, and this slice adds no series and no dashboard JSON.
- A production backlog deadline beyond the existing 256-job retention and the 50-row response cap.

## Local check

`tests/reconciliation_diagnostics.rs::reconciliation_diagnostics_reports_controller_failures_without_secrets` plants schedules and jobs whose stored errors, outcomes, authority, lease, actor and fingerprint contain a URL and token text. The aggregate reports `has_error` and a fixed `next_action`. The schedule and job reads still return the planted text. The test checks that the aggregate contains neither those needles nor the stored-text fields, that a sync-only agent is denied, that an operations reader sees the controller scope, that doctor stays healthy, that the read is not audited, and that the 50-row cap holds.

This worktree ran that test with a private target:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --test reconciliation_diagnostics \
  reconciliation_diagnostics_reports_controller_failures_without_secrets -- \
  --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 0 filtered out, finished in 2.00s. The test profile finished in 1m 17s. `/tmp/riauth-o06-target` was 3.5G afterward. `python3 scripts/check-docs.py` reported that markdown links and the build-directory layout were checked. The linker printed `__eh_frame section too large` while linking the test binary; the test still passed.
