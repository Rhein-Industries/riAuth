# SSF delivery

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D04
`ec76d0c5-2efe-4005-bb49-1f3b54878146`.

Use this page while the store is still serving and an outbound Shared Signals
delivery is stopped, retrying, or cancelled. The read below counts every
stored delivery and lists the attention rows this caller may see. It leaves
the row unchanged.

Enqueue rules for account disable, session revoke, and credential change stay
in [outbound behavior](enterprise/ENT-07.md#outbound-behavior). The serving
commands that call `signals::enqueue` for `session-revoked`, for one session
or for `user revoke-sessions`, are in
[credential compromise](credential-compromise.md). That page separates the
call from a zero-or-more `ssf_deliveries` count. The design
record for this read is
[SSF delivery diagnostics](roadmap/o06-ssf-delivery-diagnostics.md). Mail,
logout, and deactivation delivery stay on their own reads.
[Deactivation delivery](deactivation-delivery.md) is the outbound SCIM
deactivation aggregate.

## Where the read lives

The read is `Core::ssf_delivery_diagnostics` in
[src/assembly/ssf.rs](../src/assembly/ssf.rs). Platform registers

`GET /api/operations/ssf`

in `platform_routes` in [src/api.rs](../src/api.rs), compiled with the
`platform` feature. The handler is
`session_handler!(ssf_delivery_diagnostics, ssf_delivery_diagnostics)`. The
body is `riauth.ssf-delivery-diagnostics/v1`.

Essentials does not register that route. `edition::validate_store` refuses
an Essentials open while `ssf_streams`, `ssf_deliveries`, or `ssf_jti` still
holds a row.

There is no `riauth` or `riauthctl` command for this GET. `riauth ssf stream
list` calls `GET /api/ssf/admin/streams` and returns stream configuration. It
is not this aggregate.

## Authorization and the request

Send the GET to the configured issuer with one trailing slash removed, then
`/api/operations/ssf`. A path on the issuer stays in that URL: the server
nests its routes under the issuer path. Use a client that already trusts
this deployment.

The header is one `Authorization: Bearer` value. A missing header is HTTP
401 `authentication_required` (`Bearer authentication required`). A
non-bearer scheme, a token with whitespace, or a rejected session is HTTP
401 `invalid_token` (`Authentication required or session expired`). A
duplicate `Authorization` header is HTTP 400 `invalid_request`. A caller who
fails the permission check is HTTP 403 `access_denied`.

The route calls `management` with `operations.read` on `operations/ssf`. A
human with `user.admin` passes every edition-available action. An agent
needs `operations.read` on `operations/ssf`, or `operations.read=*`. These
grants leave the route check failed: `ssf.configure`, `ssf.manage`,
`operations.read` on `operations/mail`, `operations.read` on
`operations/offboarding`, and `operations.read` on
`operations/reconciliation`.

A delegated human is refused. Current grants are `help_desk`,
`application_owner`, `directory_operator`, `auditor`, and
`security_administrator`, and `HumanGrant::allows` in
[src/delegation.rs](../src/delegation.rs) has no `operations.read`,
`ssf.configure`, or `ssf.manage` arm. A non-administrator with no active
grants is HTTP 403 `access_denied` when the principal is loaded, before the
route action is checked.

Keep the bearer token out of shell history, tickets, and shared logs. A
saved CLI session file contains `issuer`, `token`, and `expires_at`. Leave
that file out of copied output. This page has no shell example: the read
has no `riauth` subcommand, and a copied command would have to carry the
token.

Item visibility is a second check, after the route check. The resource is
`ssf/` plus the stored `stream_id`. The caller needs `ssf.configure` or
`ssf.manage` on that resource. An agent resource of `*` matches. A full
administrator sees every row. Either stream permission is enough for that
row; the route permission is still required first. `ssf.configure` on one
stream does not reveal another stream's row.

`counts.withheld` counts every row this caller may not see, including
pending and delivered rows. `counts.withheld_attention` is the attention
subset. Those rows are omitted, so the stream id is absent from `items`. An
operations-only caller can see the counts and an empty `items` list.
Withheld rows leave `truncated` false.

## How to read the body

`checked_at` is the server's unix time when the read runs.
`limits.attention_items` is 50. `listed` is the length of `items`.
`truncated` is true when the number of visible attention rows is greater
than 50. Withheld rows stay in the counts and leave `truncated` false.

`counts.deliveries` is every stored row. These state counts partition that
total: `pending`, `retrying`, `stopped`, `cancelled`, `delivered`.
`counts.attention` includes withheld attention rows.

Classification uses the stored flags, in this order. `delivered_at` is
`delivered`, including a row that also has `stopped` and `last_failed`.
Otherwise `stopped` together with `last_failed` is `stopped`. Otherwise
`stopped` is `cancelled`. Otherwise `last_failed` is `retrying`. The
remaining rows are `pending`.

Attention is `retrying`, `stopped`, and `cancelled`. `pending` and
`delivered` stay in the counts and are omitted from `items`. Visible items
are ordered by severity and then id: stopped, retrying, cancelled.

Each listed item carries `id`, `stream_id`, `delivery_state`, `attempts`,
`next_attempt`, `created_at`, `last_failed`, `last_status`, `event`, and
`next_action`. `last_status` is the stored HTTP status, or null when none
was recorded. `last_failed` is the stored flag.

`event` is a fixed token. The stored event string maps as follows:

| Token | Stored event |
| --- | --- |
| `account_disabled` | `https://schemas.openid.net/secevent/risc/event-type/account-disabled` |
| `session_revoked` | `https://schemas.openid.net/secevent/caep/event-type/session-revoked` |
| `credential_change` | `https://schemas.openid.net/secevent/caep/event-type/credential-change` |
| `unknown` | any other stored string |

The aggregate omits the endpoint URL (`uri`), the raw event string, the
subject, the audience, the credential type, and the SET `jti`. The receiver
authorization header lives on the stream record. This read does not load
`ssf_streams`, so that header is neither read nor returned. The SET body is
built at send time and is not stored on the delivery record. A failed POST
can log `stream_id`, `attempt`, and `status` with the message `SSF delivery
not accepted`. That log uses status `0` when no HTTP status was recorded.
The stored `last_status` stays null in that case. The log omits the endpoint
and the token. The log line is not this response.

`GET /api/ssf/admin/streams` (`riauth ssf stream list`) is a separate read.
A full administrator sees every stream. An agent needs `ssf.manage` on
`ssf/<stream id>` for that stream; `ssf.configure` does not pass that list.
The view includes `delivery.endpoint_url` and omits the authorization
header. `GET /api/ssf/streams` is the receiver configuration read. An owned
standard stream is visible there with `ssf.configure` on `ssf/<stream id>`,
and that view also includes the endpoint URL and omits the authorization
header. This diagnostic does not call either route. An item can therefore
name a `stream_id` whose endpoint this response does not contain.

## Next action

Read `next_action` from the item. This GET does not claim, finish, cancel,
or POST the row. The listed vocabulary is these three tokens.

| Token | When the item carries it | What the flags mean |
| --- | --- | --- |
| `inspect_receiver` | `delivery_state` is `stopped`: `stopped` and `last_failed` are both set, and `delivered_at` is absent. | The worker will not claim the row again. `finish_delivery` in [src/ssf.rs](../src/ssf.rs) sets both flags for a recorded status outside 200–299, 429, and 500–599, including a 3xx returned without following the redirect, and for a retryable result once `attempts` has reached 5. `claim_deliveries` sets both flags, without another POST, when `attempts` is already at least 5 or when `created_at + 86400` is below the claim time. A push URL that fails `push_url` at claim sets both flags and adds one to `attempts`. `cancel_pending` only sets `stopped`. A row that already had `last_failed` therefore becomes `stopped` when a stream is deleted, when its endpoint, authorization header, or delivered events change, when its subject bindings change, or when claim finds the stream missing or `endpoint_url` different from the stored `uri`. `last_status` null on this token means no HTTP status is stored. That includes the age stop, the attempt-cap stop before POST, a rejected push URL, and a transport or signing failure that later reached the attempt cap. |
| `wait_for_retry` | `delivery_state` is `retrying`: `last_failed` is set, `stopped` is clear, and `delivered_at` is absent. | `finish_delivery` recorded 429, 500–599, or no status, and `attempts` was still below 5. No status covers a transport error, the five-second client timeout, and a signing failure. `next_attempt` is the due time. After that finish the delay is `2^attempts` seconds, capped at 3600. Claim parks `next_attempt` at 60 seconds ahead, stores an owner `lease`, and leaves `dispatch_started` empty. `begin_dispatch` sets `dispatch_started` before the POST while that lease is current and `next_attempt` is still ahead. Until finish runs, that in-flight row is still `pending` and is omitted from `items`. The item omits `lease` and `dispatch_started`. This read does not send the POST. |
| `delivery_cancelled` | `delivery_state` is `cancelled`: `stopped` is set, `last_failed` is clear, and `delivered_at` is absent. | `cancel_pending`, the missing-stream and endpoint-mismatch arms of `claim_deliveries`, and the same arms of `begin_dispatch` set `stopped` and left `last_failed` false. The pin clears the owner and does not POST. The local queue will not claim the row again. The read does not load the stream, so these writers look the same on the item. A finish that arrives after `stopped` is set is ignored, so an in-flight POST can still have reached the receiver while `last_status` stays at its previous value. `attempts` can be greater than zero. This read does not delete the stream or change its endpoint. |

`MAX_ATTEMPTS` is 5. Retryable statuses are 429, 500–599, and a missing
status. Other recorded statuses stop on that attempt. HTTP 200–299 sets
`delivered_at` and clears `last_failed`. Those rows are counted as
`delivered` and omitted from `items`.

Removing a stream, replacing its subject bindings, or changing its endpoint,
authorization header, or delivered events calls `cancel_pending` for that
stream. Pending rows gain `stopped` and keep the `last_failed` they already
had. That write is separate from this GET.

## Remote receipt and readiness

`counts.delivered` means `delivered_at` is set. The worker sets that after
an HTTP status in 200–299. The response body is not stored and is not
interpreted. The count records that HTTP status. Receiver processing, and
whether the receiver kept the SET, stay outside this read.

A row absent from `items` can be pending, delivered, withheld, past the
50-row visible cap, or already removed by cleanup. `delivery_cancelled` means
the local row is stopped without `last_failed`. It can follow a POST whose
completion was ignored. `inspect_receiver` covers receiver statuses and the
local stops in the table above. The token alone does not identify which of
those writers ran.

The SET `jti` is omitted. Outbound delivery is at-least-once, and the same
`jti` is reused across retries, as
[outbound behavior](enterprise/ENT-07.md#outbound-behavior) describes.
Deduplication by the receiver is not visible in this body.

`affects_readiness` is false. The body has no `healthy` field.
`doctor.healthy` is true when at least one enabled administrator exists.
The doctor body has `pending_logout_deliveries` and has no SSF field.
`/livez` reports process liveness and performs no database read. `/readyz`
reports the storage readiness check and, on an authentication role, refuses
a saturated application-worker pool. This read writes neither answer.

`queues.ssf_deliveries` is a different classifier, in `queue_state` in
[src/store/maintenance.rs](../src/store/maintenance.rs). `failed` is true
when `stopped` is true, `last_failed` is true, `next_attempt` is
`u64::MAX`, an `error` field is present, or `last_status` is outside
200–299. That mixes retrying, stopped, and cancelled, and it includes a
delivered row that still has `stopped`, `last_failed`, or a `last_status`
outside 200–299. `pending` is a row with no `delivered_at` and without
`stopped`, so a retrying row is pending there. This read does not rewrite
those gauges. Treat `counts.stopped` and the queue failed gauge as
different numbers.

The scan does add to the existing process counter
`riauth_storage_scanned_records_total`. That counter counts storage scans
for the process. It is not a delivery-receipt series. This read adds no SSF
Prometheus series and no dashboard.

The read is a storage read. It writes no audit event, changes no queue
index, and neither claims nor dispatches a delivery.

## Page and value limits

The read scans bucket `ssf_deliveries` with `store::maintenance::PAGE`,
which is 128. The next page starts after the last key of the previous page.
A page that does not advance that cursor fails the read: the HTTP response
is 500 `server_error` with message `Internal server error`, and the server
log records `SSF delivery diagnostic page did not advance`. A short page
ends the scan, so a bucket whose length is an exact multiple of 128 takes
one extra empty page read.

The response keeps at most 50 visible attention rows, ranked by severity
and then id, and drops a worse row as soon as that list is full.
`truncated` reports that the visible attention count exceeded 50. The
retained list is the most severe visible rows. The response has no resume
cursor.

`tx.scan` decodes each stored delivery in full while its page is current.
The value includes the endpoint URL, subject, audience, JTI, credential
type, and raw event string. This scan has no byte budget. One page can
therefore hold 128 large values. That page is released before the next page
is read. Stream records and user records are not read. A listed item copies
`stream_id`.

Administrator-created stream ids pass `validate_name`: 1–64 ASCII letters,
digits, `.`, `-`, `_`, and `@`, and `.` and `..` are rejected.
Receiver-created stream ids are a UUID from `crypto::id`. This read uses
the stored id as the authorization resource and does not apply that name
limit again. A value written directly into the store is decoded as stored.
The page cursor keeps one stored key.

The handler waits up to two seconds for one of eight application workers.
A pool that stays busy is HTTP 503 `temporarily_unavailable` (`Server busy;
retry shortly`). That status is admission, not a delivery state. After
admission, the scan runs through the whole bucket on that worker. The
two-second readiness-probe deadline is a separate check.

Cleanup is also a separate pass. `ssf::cleanup` deletes a delivery when
`created_at + 7 * 86400` is below the cleanup time, and it deletes an
expired `ssf_jti`. `maintenance_page` advances at most 128 records per
collection on each pass. This diagnostic does not delete those rows. A later
read can omit a row that cleanup has removed.

## What this page did not run

This page records the source behavior of the read. It sent no request to
`GET /api/operations/ssf`. It called no receiver and deployed no dashboard.
It did not re-run `tests/ssf_delivery_diagnostics.rs`, and it did not start
PostgreSQL. The design note linked above records an earlier local run of
that file. This page does not repeat that run.
[ENT-07](enterprise/ENT-07.md) records that Apple Business Manager was not
tested, and `tests/ssf.rs` uses local keys and a local HTTP listener.
Essentials has no route for this aggregate. `doctor`, `/readyz`, `/livez`,
and the existing Prometheus counters keep the behavior described above. The
[A01 coverage inventory](roadmap/coverage-inventory.md) still describes D04
at revision `96e23e2`.
