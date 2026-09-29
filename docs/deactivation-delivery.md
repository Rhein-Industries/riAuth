# Deactivation delivery

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D04
`ec76d0c5-2efe-4005-bb49-1f3b54878146`.

Use this page while the store is still serving and an outbound deactivation
is incomplete or ambiguous. That includes a row that no offboarding job
records. The read below counts every stored deactivation and lists the
attention rows this caller may see. It leaves the row unchanged.

The detailed list and the follow-up writes stay in
[outbound provisioning](scim.md#outbound-provisioning) and
[offboarding deactivation](scim.md#offboarding-deactivation). Stopping a
provisioning job stays in
[connector dependency incidents](connector-incidents.md#outbound-scim).
Outbound Shared Signals is a separate queue. Its incident read is
[SSF delivery](ssf-delivery.md).

## Where the read lives

The read is `Core::offboarding_deactivation_diagnostics` in
[src/offboarding.rs](../src/offboarding.rs). Platform registers

`GET /api/operations/offboarding/deactivations`

in `platform_routes` in [src/api.rs](../src/api.rs), compiled with the
`platform` feature. The handler is
`session_handler!(offboarding_deactivation_diagnostics)`. The body is
`riauth.offboarding-deactivation-diagnostics/v1`.

Essentials keeps the shared `provisioning_deactivations` bucket and
`GET /api/provisioning/deactivations`. The redacted aggregate is the
Platform route above.

`riauth offboard diagnostics` calls `GET /api/operations/offboarding` and
returns `riauth.offboarding-diagnostics/v1`. That is the scheduled-job
aggregate. Its hidden-target token is `inspect_hidden_targets`. There is
no `riauth` or `riauthctl` command for the deactivation aggregate.

## Authorization and the request

Send the GET to the configured issuer with one trailing slash removed, then
`/api/operations/offboarding/deactivations`. A path on the issuer stays in
that URL: the server nests its routes under the issuer path. Use a client
that already trusts this deployment.

The header is one `Authorization: Bearer` value. A missing or non-bearer
header is HTTP 401 `authentication_required`. A rejected session is HTTP
401 `invalid_token` (`Authentication required or session expired`). A caller
who fails the permission check is HTTP 403 `access_denied`.

The route calls `management` with `operations.read` on
`operations/offboarding`. A human with `user.admin` passes every
edition-available action. An agent needs `operations.read` on
`operations/offboarding`, or `operations.read=*`. A delegated human is
refused. Current grants are `help_desk`, `application_owner`,
`directory_operator`, `auditor`, and `security_administrator`, and
`HumanGrant::allows` has no `operations.read` arm. `directory.sync`,
`provisioner.sync`, `provisioner.read`, and `user.offboard` leave the same
check failed. `operations.read` on `operations/reconciliation` does too.

Keep the bearer token out of shell history, tickets, and shared logs. A
saved CLI session file contains `issuer`, `token`, and `expires_at`. Leave
that file out of copied output. This page has no shell example: the read
has no `riauth` subcommand, and a copied command would have to carry the
token.

Item visibility is a second check, after the route check:

- The account still exists and the caller has `user.offboard` on
  `user/<current username>` for that `user_id`. The stored username is
  historical and is not the authorization key. The item's `username` is the
  current username. `recorded_username_matches` compares it with the stored
  name. The response has no separate stored-username field.
- The account is missing and the caller is a non-agent, non-delegated
  administrator. The item has `account_present: false` and neither username
  field.
- An agent, including one with `user.offboard=*`, leaves a missing account
  in the withheld counts. The item is omitted. `visible_account` uses the
  same branch for a delegated principal; current grants refuse that
  principal at the route.
- `target` requires `provisioner.read` on `provisioner/<target>`. A hidden
  target omits `target`, sets `target_hidden` true, and sets `next_action`
  to `inspect_hidden_target` before any other token. The row still has a
  target.

`counts.withheld` counts every row this caller may not see, including rows
that are not attention rows. `counts.withheld_attention` is the attention
subset. An operations-only caller can see the counts and an empty `items`
list.

## How to read the body

`checked_at` is the server's unix time when the read runs.
`limits.attention_items` is 50. `listed` is the length of `items`.
`truncated` is true when the number of visible attention rows is greater
than 50. Withheld rows stay in the counts and leave `truncated` false.

`counts.deactivations` is every stored row. These status counts partition
that total: `pending`, `running`, `delivered`, `superseded`, `stale`,
`failed`, `dismissed`. These delivery counts also partition that total, by
`Deactivation::delivery_state`: `delivery_pending`, `delivery_failed`,
`delivery_ambiguous`, `delivery_dismissed`, `delivery_succeeded`,
`delivery_resolved`, `delivery_cancelled`. The two partitions describe
different facts. A `failed` status with a satisfied operator resolution is
`delivery_resolved`. A `stale` status that is not uncertain is
`delivery_failed`. `counts.failed` is status `Failed` only. A delivered row
that is still uncertain increments both `counts.delivered` and
`counts.delivery_ambiguous`.

Attention is delivery state `pending`, `failed`, `ambiguous`, or
`dismissed`. `succeeded`, `resolved`, and `cancelled` stay in the counts and
are omitted from `items`. `counts.attention` includes withheld attention
rows. Visible items are ordered by severity and then id: failed, ambiguous,
dismissed, pending.

`delivery_state` is chosen in this order. `uncertain: true` is `ambiguous`
and outranks status, hold, and a stored resolution. Status `Dismissed` is
then `dismissed`. A satisfied resolution is then `resolved`. Otherwise
`Delivered` is `succeeded`, `Superseded` is `cancelled`, `Pending` or
`Running` is `pending`, and `Stale` or `Failed` is `failed`.

Remote completion is delivery state `succeeded`. Those rows are counted and
omitted from `items`. Every listed item sets `remote_completion_verified`
to false, including a listed row whose `status` is `delivered` while
`delivery_state` is `ambiguous`. A resolved row is an operator attestation
and is omitted from `items`. A dismissed row is a waiver. The body has no
`healthy` field and sets `affects_readiness` to false.

`doctor.healthy` is true when at least one enabled administrator exists.
The doctor body has no deactivation field. `/livez` reports process
liveness and performs no database read. `/readyz` reports the storage
readiness check and, on an authentication role, refuses a saturated
application-worker pool. This read writes neither answer. The scan does add
to the existing process counter `riauth_storage_scanned_records_total`.
That counter counts storage scans for the process. It is not a
deactivation-completion series, and this read adds no deactivation series
and no dashboard.

The read is a storage read. It writes no audit event, changes no queue
index, and neither claims nor dispatches a deactivation. It loads no
offboarding job. The body has no job-linkage field and no
`counts.unreferenced` field. `riauth offboard get` reads one scheduled
job. The redacted job aggregate is `GET /api/operations/offboarding`.

Each listed item can carry `id`, `account_present`, `target_hidden`,
`status`, `delivery_state`, `hold`, `hold_recognized`, `outcome`,
`attempts`, `next_attempt`, `has_error`, `uncertain`, `delivered_at`,
`remote_completion_verified`, `has_unlinked_create`,
`dispatch_recovery_count`, and `next_action`, plus `username` and
`recorded_username_matches` when the account is visible, plus `target` when
the target is visible. `has_error` is true when a stored `last_error` is
present, including an empty string. `has_unlinked_create` is presence.
`dispatch_recovery_count` is the recovery-list length. `outcome` is
`deactivated`, `already_inactive`, `reviewed_delivery`, `remote_active`, or
`remote_inactive`. Any other outcome is null.

`hold` is null when the stored hold is absent or not one of:
`unlinked_create_requires_settlement`, `recovered_dispatch`,
`awaiting_dispatch_ack`, `target_unconfigured`, `awaiting_controller`,
`awaiting_controller_authority`, `awaiting_prior_delivery`, `manual_mode`,
`removal_review_required`, `guarded_removal`, `retry`. An absent hold has
`hold_recognized: true`. Any other hold has `hold_recognized: false`. A
known hold selects `next_action` ahead of `has_error`. The wording of a
stored error does not select the token.

The aggregate omits stored error text, target URLs, remote and external
identifiers, link material, leases, actors, resolution and dismissal
records, dispatch-recovery payloads, and unlinked-create payloads. Those
fields stay on `GET /api/provisioning/deactivations` for a caller with
`provisioner.read` on the target and `user.read` on the account.
`riauth provision deactivations` lists the newest 1000 rows
([src/cli.rs](../src/cli.rs)).

## Next action

Read `next_action` from the item. Apply the row only through the existing
provision commands. This GET does not call them.

| Token | When the item carries it | Follow-up, which this read does not perform |
| --- | --- | --- |
| `inspect_hidden_target` | `target_hidden` is true. This token replaces every other token. | Obtain `provisioner.read` on `provisioner/<target>` before treating the row as target-less. The name stays omitted until then. |
| `attest_remote_state` | `delivery_state` is `ambiguous`. `uncertain` is true, so this outranks status and hold. | Inspect the target. `riauth provision resolve-deactivation` records `observed` and `evidence` and performs no remote request. `applied` or `absent` later reads as `resolved`, which is omitted from `items` and is separate from `succeeded`. The command is in [resolving ambiguity](scim.md#resolving-ambiguity). |
| `waiver_is_not_remote_delivery` | `delivery_state` is `dismissed`. | The waiver is the local record. Dismissal performs no remote request. A row that was already uncertain stays `ambiguous` and uses `attest_remote_state`. See [dismissing an undeliverable intent](scim.md#dismissing-an-undeliverable-intent). |
| `inspect_and_replan` | `delivery_state` is `failed` and `status` is `stale`. | The link may be gone. `riauth provision retry-deactivation` then conflicts with `The outbound link was removed; there is nothing to deactivate`. Inspect the target and replan. The retry command itself does not call the target. |
| `retry_or_replan_deactivation` | `delivery_state` is `failed` and `status` is `failed`. | `riauth provision retry-deactivation` rebinds a failed or stale row to the current outbound link and queues evaluation. The command does not call the target. The worker may call it on a later pass. It does not create a user. An unlinked Create is refused: `An unlinked Create cannot be retried or rebound from a later link; settle its original request and resolve the offboarding intent explicitly`. |
| `review_provisioning_plan` | `delivery_state` is `pending` and `hold` is `manual_mode`, `removal_review_required`, `guarded_removal`, `awaiting_controller`, or `target_unconfigured`. | Automatic delivery is waiting on review mode, the shared removal floor, a missing controller configuration, or a target that is absent from configuration. This read does not apply a plan. |
| `restore_controller_authority` | `delivery_state` is `pending` and `hold` is `awaiting_controller_authority`. | The scoped controller agent is missing, inactive, or lacks `provisioner.sync` on `provisioner/<target>`. This read does not change that agent. |
| `wait_for_provisioning_job` | `delivery_state` is `pending` and `hold` is `awaiting_prior_delivery`. | A reviewed provisioning job for that target still has an unsettled lease. Leave that job to finish. |
| `wait_for_dispatch_settlement` | `delivery_state` is `pending` and `hold` is `awaiting_dispatch_ack`, `unlinked_create_requires_settlement`, or `recovered_dispatch`. | A dispatch still needs settlement. The listed item still has `remote_completion_verified: false`. The same stored hold with `uncertain: true` is `ambiguous` and uses `attest_remote_state`. Dispatch recovery is the separate administrator action in [recovering an abandoned dispatch](scim.md#recovering-an-abandoned-dispatch). |
| `wait_for_retry` | `delivery_state` is `pending` and `hold` is `retry`. | The worker will retry. `next_attempt` is the due time. This read does not send the disable. |
| `inspect_deactivation` | `delivery_state` is `pending`, no hold above selected a token, and `has_error` is true. | A stored `last_error` is present and the hold is absent or unrecognized. The text is on `riauth provision deactivations`. |
| `wait_for_deactivation` | `delivery_state` is `pending`, no hold above selected a token, and `has_error` is false. | The worker has the row. `next_attempt` is the due time. `remote_completion_verified` on the item is false. |

The same helper also returns `attestation_is_not_remote_delivery`,
`delivery_record_expired`, and `account_changed_before_delivery` for a job
aggregate's target rollup. A listed deactivation item is `pending`,
`failed`, `ambiguous`, or `dismissed`, so the table above plus
`inspect_hidden_target` is the whole item vocabulary.

## Page and value limits

The read scans bucket `provisioning_deactivations` with
`store::maintenance::PAGE`, which is 128. The next page starts after the
last key of the previous page. A page that does not advance that cursor
fails the read: the HTTP response is 500 `server_error` with message
`Internal server error`, and the server log records
`Deactivation diagnostic page did not advance`. A short page ends the scan,
so a bucket whose length is an exact multiple of 128 takes one extra empty
page read.

The response keeps at most 50 visible attention rows, ranked by delivery
severity and then id, and drops a worse row as soon as that list is full.
`truncated` reports that the visible attention count exceeded 50. The
retained list is the most severe visible rows. The response has no resume
cursor.

`tx.scan` decodes each stored deactivation in full while its page is
current. The value includes error text, URLs, remote identifiers, evidence,
and recovery payloads. This scan has no byte budget. Snapshot paging's byte
budget is a different path. One page can therefore hold 128 large values.
That page is released before the next page is read. One user record is read
per deactivation and is not accumulated. Job records are not read. A listed
item copies the current username and, when visible, the target name.

Names created through the API are 1–64 characters from ASCII letters,
digits, `.`, `-`, `_`, and `@`, and `.` and `..` are rejected. A value
written directly into the store is decoded as stored. This read does not
apply that name limit again.

The handler waits up to two seconds for one of eight application workers.
After admission, the scan runs through the whole bucket on that worker.
The two-second readiness-probe deadline is a separate check. Cleanup is
also a separate pass: it removes a terminal row, other than a dismissed row
or a row that still holds a lease, a dispatch recovery, or an unlinked
create, 90 days after `next_attempt`. This diagnostic does not delete those
rows.

## What this page did not run

This page records the source behavior of the read. It sent no request to
the route. It called no connector and deployed no dashboard. It did not
re-run the deactivation diagnostic tests, and it did not start PostgreSQL.
Essentials does not serve this Platform route. Both editions serve
`GET /api/operations/provisioning/deactivations`, documented in
[provisioning deactivation diagnostics](roadmap/o06-provisioning-deactivation-diagnostics.md).
That read withholds a missing account and omits the target name. `doctor`, `/readyz`,
`/livez`, and the existing Prometheus counters keep the behavior described
above. The [A01 coverage inventory](roadmap/coverage-inventory.md) still
describes D04 at revision `96e23e2`.
