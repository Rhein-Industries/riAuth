# Connector removal safeguards

LDAP import, Google Workspace import, Microsoft Entra import, outbound SCIM and
desired state share the plan guard in `connector_guard`. The remote connectors
also use its bounded pagination checks. Apply uses common eligibility,
authority binding and recomputed removal-impact checks.
Outbound SCIM, LDAP, Workspace, Entra and desired state have explicit
controller modes. A mode alone does not install a schedule; an explicit scoped
controller declaration does.

## Source crawl quotas

The optional instance-wide quota policy applies to LDAP planning and apply
validation, and to Workspace and Entra planning and apply validation. Omitted
values retain the conservative defaults. Overrides may only lower a bound:

```toml
[reconciliation_quotas.ldap]
pages_per_call = 2
max_pages_per_search = 10
max_users = 1000
max_snapshot_bytes = 2097152
draft_ttl_seconds = 180

[reconciliation_quotas.cloud]
pages_per_call = 2
max_pages_per_collection = 10
max_objects = 1000
max_page_bytes = 524288
max_snapshot_bytes = 2097152
draft_ttl_seconds = 180
```

| Quota | Default and maximum | Minimum |
| --- | ---: | ---: |
| LDAP `pages_per_call` | 4 | 1 |
| LDAP `max_pages_per_search` | 20 | 1 |
| LDAP `max_users` (also rows per group search) | 2,000 | 1 |
| LDAP `max_snapshot_bytes` | 4,194,304 | 65,536 |
| Cloud `pages_per_call` | 5 | 1 |
| Cloud `max_pages_per_collection` | 20 | 1 |
| Cloud `max_objects` (all returned object types per collection) | 2,000 | 1 |
| Cloud `max_page_bytes` (source response) | 1,048,576 | 4,096 |
| Cloud `max_snapshot_bytes` | 4,194,304 | 65,536 |
| Both `draft_ttl_seconds` | 300 | 30 |

Cloud `max_page_bytes` cannot exceed `max_snapshot_bytes`. Invalid or unknown
fields reject configuration. The current policy is bound into each source
fingerprint: changing it restarts an in-progress plan crawl and rejects an old
plan at apply, including a partially completed apply crawl. Each resumed call
enforces the current policy before saving another page. A quota reached before
source completion is an incomplete snapshot, never an empty source or removal
authorization. LDAP and cloud still cap a wire page at 200 rows. LDAP paged
cookies remain limited to 4,096 bytes; Workspace tokens to 2,048 bytes and
Graph next links to 8,192 bytes; token responses retain a fixed 1 MiB cap. The 30-second per-call source budget, 5-second
LDAP step timeout, five-minute plan expiry and 7 MiB plan-record cap remain
fixed. LDAP paged results and cloud list pagination do not provide a remote
point-in-time snapshot across calls.

`[ldap_reconciliation_modes]`, `[workspace_reconciliation_modes]`,
`[entra_reconciliation_modes]` and `[scim_reconciliation_modes]` map configured
connector IDs to `manual-review`, `guarded-automatic` or `automatic`. Omitted
entries are manual. Unknown values and references to absent connectors reject
configuration. `Core::directory_reconcile` and `Core::cloud_reconcile` are
controller triggers for a scoped caller and can run from a P02 durable job. They
return `awaiting_review` with the exact plan or `applied` after the existing
local apply transaction. A still-bound pending plan retains its ID. A mode,
revision, authority or source change requires a new plan; a controller trigger
supersedes the previous unapplied plan for that connector and actor. Automatic
mode still stops at the fixed P03 review floor, and guarded automatic stops at
any removal. SCIM's `queued` decision reports a durable job, not remote
completion.

Configure an independent controller for each connector that should run on a
schedule or accept source events. The credential file must be private and contain
the current `ri_agent_` token for the named agent. That agent needs the connector's
scoped `directory.sync` or `provisioner.sync` permission and any additional
permissions required by its existing apply path. Credential rotation changes the
file contents; no bearer token is stored in the database. For example:

```toml
[reconciliation_controllers."scim/payroll"]
agent_id = "payroll_controller"
credential_file = "secrets/payroll-controller-token"
interval_seconds = 3600
```

Keys may also be `ldap/<id>`, `workspace/<id>`, or `entra/<id>`; interval bounds
are 60–86400 seconds. Schedule cursors and jobs survive restart. The worker
claims one due job at a time with a lease and at most four attempts. It checks
the current controller/connector config, the agent's live scoped authority and
the current credential before invoking the P01 plan/apply path. A heartbeat
renews a long read, and the shared apply gate checks the current lease inside
the same write transaction that commits local changes or queues SCIM delivery.
An expired worker cannot apply after another worker reclaims its job. The event
endpoint is `POST /api/reconciliation/{kind}/{id}/events` with
`{"event_id":"stable-source-event-id"}` and that agent's bearer token. Repeating
an event ID returns the same retained job. Scoped status is available at
`GET /api/reconciliation/schedules` and `GET /api/reconciliation/jobs`.
`GET /api/operations/reconciliation` is the instance-wide redacted failure
read for `operations.read` on `operations/reconciliation`. It does not replace
those scoped reads. A schedule `next_run` there is the next enqueue time.
See [reconciliation diagnostics](roadmap/o06-reconciliation-diagnostics.md).
Controller outcomes distinguish `local_applied`, `downstream_queued`,
`pending_prior_delivery`, and `none` for review. The SCIM delivery worker owns
remote delivery and its separate job status; a controller's completed queue
decision never means the target received a change. Desired-state scheduling
still requires a durable, authorized manifest source.

Desired-state plans include the same exact review binding and a recomputed
removal impact. `state_reconciliation_mode` defaults to manual review.
`Core::state_reconcile` accepts an authorized manifest and reports a pending
plan or a committed local apply. It holds credential and trust changes for
review even in automatic mode. The desired-state CLI and HTTP apply path accept
the same exact plan-ID removal confirmation.

A manifest that names only groups stores a membership dependency digest with
its base revision. An unrelated audited write may advance `meta.revision`
while that plan remains usable. Live membership, the identity or stored
credentials of a current or proposed member, directory or cloud ownership,
credential exposure, elevation provenance, the reviewed-membership fence, and
human grant generation invalidate it with `Desired-state group dependencies
changed`. The same conflict is returned when the issuer, reviewed membership
groups, PAM approvers, directory configuration, or capabilities change. A
supplied If-Match must be the current management revision. Repeating the same
idempotency key and fingerprint returns the stored result. A different If-Match
or body returns `Idempotency key was used for a different request`. Manifests
that name another resource family, other than one client display-name change,
and manifests with `target_state_fingerprint`, still compare the global
revision. Removal confirmation is unchanged.

A manifest that names one existing client and changes only its display name
stores a client dependency digest with its base revision. An unrelated audited
write may advance `meta.revision` while that plan remains usable. The client
record, its credential version, the signing key it uses, issuer ownership,
referenced policy groups and users, referenced sources, listener bindings, and
device-trust configuration invalidate it with `Desired-state client name
dependencies changed`. A client that a proxy, LDAP, or RADIUS listener already
names stays on the global revision, because those listeners read file-backed
secrets. A supplied If-Match must be the current management revision. The same
idempotency key and fingerprint return the stored result. Scope, credential,
endpoint, status, and policy edits stay on the global revision, and removal
confirmation is unchanged.

| Path | Snapshot checks | Destructive boundary |
| --- | --- | --- |
| LDAP users and mapped memberships | Critical paged-results control on every page; successful completion; bounded cookies, pages, rows, bytes and time; no referrals, duplicate DNs or stable IDs | Apply resumes a durable plan-bound crawl, checks exact completed entries, plan content, current authority, configuration and local revision, recomputes impact and requires confirmation before reconciliation |
| Workspace/Entra users, groups and members | Required collection shape; unique IDs across pages; bounded pages, rows, bytes and time; no empty continuation pages or repeated cursors; exact totals, when supplied, must agree and complete; next links stay on the same collection and origin | Same apply checks as LDAP; existing tenant and stable-identity ownership checks remain |
| Desired-state named resources | Explicit manifest resources only; omission leaves them unchanged | Apply recomputes disable and membership impact before the first mutation, validates exact content and current actor authority, then commits atomically. Group-only manifests accept an unrelated management revision when membership, member identity, ownership, and membership policy still match. A single existing client's display-name change accepts an unrelated management revision when its credential, signing key, policy references, and issuer ownership still match. Every other manifest still requires the global revision. |
| Outbound SCIM filtered lookup | Explicit Resources array and exact totalResults; at most one matching externalId; optional startIndex must be 1 and itemsPerPage must match; continuation/error responses fail | No POST/PATCH or successful item advancement from incomplete lookup; a missing previously linked resource requires inspection |
| Outbound SCIM group/user update | Complete bounded, unique member-value arrays before membership replacement, and before accepting a group as already up to date whenever it has reviewed managed or desired members (an omitted, `null` or paginated `members` field never counts as empty); the post-write read-back must report the same explicit membership; explicit boolean active state before disabling | Remote member removals must belong to the exact reviewed previous managed link; unexpected remote membership and omission of retained managed members fail closed. ETags protect the conditional PATCH. Authority, revision and lease are checked again before dispatch |

## Review policy

Each plan reports `removal_impact`: `disabled_users`, `missing_users`,
`removed_memberships` and `review_required`. Desired-state plans also report
nonzero `disabled_clients`, `disabled_sources` and `disabled_passwords`; each
requires review. The fixed baseline policy requires review for every missing
imported user or managed membership removal. Disabling explicitly present users
requires review when all active linked users would be
disabled, at least five would be disabled and that is at least 20% of active
linked users, or at least two would be disabled and that is at least 50%.
Equality crosses a threshold.
Desired state uses all active local users as its baseline because a manifest
does not own a separate connector population.
Outbound SCIM measures departures against its
previously delivered active links and managed group members; it never deletes
remote accounts. An ordinary explicit disable below those thresholds, with no
membership removal, can use the ordinary apply operation.

A complete empty initial source with no existing links is valid. An empty source
that would remove linked identities requires review. Network, authentication,
LDAP search failures, malformed pages and detectable incomplete pagination are
errors; confirmation cannot turn them into authoritative empty sources.

Review the plan's entries/resources, changes, impact, target and revision. When
`review_required` is true, use `--confirm-removals` with `directory apply`,
`directory workspace apply`, `directory entra apply` or `provision apply`.
The HTTP equivalent is `X-riAuth-Confirm-Removals: <exact-plan-id>` on apply.
Cloud apply also accepts the existing `X-riAuth-Confirm-Cloud-Removals` header.
Duplicate, malformed or conflicting headers reject. Keep the same confirmation
and idempotency key for an exact retry.

The immutable plan ID identifies reviewed content, including its actor, expiry,
configuration, local revision and impact. `review.content_digest` commits to that
content and `review.authority_digest` commits to the actor's permission set, expiry, parent binding and relevant account epochs.
Apply verifies these commitments and live permissions; it does not accept edited
plan files or carry confirmation to a replacement plan. Outbound SCIM also binds
the previous delivered links used for removal review. Changes require a fresh
plan and review. Old unapplied plans without these commitments must be recreated;
old queued SCIM jobs become stale before dispatch. Already applied plans/jobs
retain their authorized idempotent result behavior.

These thresholds are fixed, shared safeguards, not new configuration fields.
The existing strict connector configuration validation still rejects unknown
settings. LDAP removes only memberships previously owned by that directory;
cloud import manages only allow-listed groups; SCIM patches only its managed
attributes. Local rejection leaves canonical identities, groups, credentials,
links and revision unchanged. Cloud fetch retry bookkeeping and SCIM job error
and retry bookkeeping can still change to record an unsuccessful attempt.

## Limits and follow-up

Peers without exact totals can silently omit data while claiming completion.
Missing users and membership changes therefore still require review even after
a syntactically complete import. LDAP paged-results counts are estimates and are
not treated as exact totals. No connector read freezes a remote directory.
Outbound SCIM plans review desired content and previously delivered managed
links; they do not fetch an entire remote tenant during planning. Remote drift
outside that reviewed membership is rejected rather than implicitly approved.

Offboarding deactivation rows (see [outbound SCIM](scim.md#offboarding-deactivation))
are removals under the same modes. `manual-review` and `guarded-automatic` hold
them for a reviewed plan. `automatic` dispatches under the target's scoped
controller unless the floor, counted over all previously delivered active links
whose accounts are now disabled or deleted, requires review. The delivery does
not rewrite managed links, so a reviewed plan still sees and rebaselines those
departures. A reviewed job stops as stale rather than dispatch an active state
for an account that is now disabled.

Remote delivery remains at least once and can be partial or uncertain. Revocation
cannot roll back a remote write already accepted by a peer. Inspect job errors
and partial results before replanning; local apply does not mean downstream work
has completed. Scheduled controller triggers and acceptance across supported
deployment modes remain follow-up scope.
Two-build parity, PostgreSQL/concurrency contracts and controlled real-peer
acceptance remain integration gates; local fake-peer tests establish only the
paths they exercise.
