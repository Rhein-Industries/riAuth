# Connector removal safeguards

LDAP import, Google Workspace import, Microsoft Entra import and outbound SCIM
share the guard in `connector_guard`. It validates bounded pagination, removal
impact and exact plan content. LDAP, cloud and outbound SCIM apply now use its
common eligibility, authority binding and recomputed removal-impact gate.
Outbound SCIM, LDAP, Workspace and Entra each have an explicit per-connector
controller mode. No scheduler is installed by these modes.

`[ldap_reconciliation_modes]`, `[workspace_reconciliation_modes]`,
`[entra_reconciliation_modes]` and `[scim_reconciliation_modes]` map configured
connector IDs to `manual-review`, `guarded-automatic` or `automatic`. Omitted
entries are manual. Unknown values and references to absent connectors reject
configuration. `Core::directory_reconcile` and `Core::cloud_reconcile` are
controller triggers for a scoped caller; P02 can schedule them later. They
return `awaiting_review` with the exact plan or `applied` after the existing
local apply transaction. A still-bound pending plan retains its ID. A mode,
revision, authority or source change requires a new plan; a controller trigger
supersedes the previous unapplied plan for that connector and actor. Automatic
mode still stops at the fixed P03 review floor, and guarded automatic stops at
any removal. SCIM's `queued` decision reports a durable job, not remote
completion.

| Path | Snapshot checks | Destructive boundary |
| --- | --- | --- |
| LDAP users and mapped memberships | Critical paged-results control on every page; successful completion; bounded cookies, pages, rows, bytes and time; no referrals, duplicate DNs or stable IDs | Apply re-fetches entries, checks plan content, current authority, configuration and local revision, recomputes impact and requires confirmation before reconciliation |
| Workspace/Entra users, groups and members | Required collection shape; unique IDs across pages; bounded pages, rows, bytes and time; no empty continuation pages or repeated cursors; exact totals, when supplied, must agree and complete; next links stay on the same collection and origin | Same apply checks as LDAP; existing tenant and stable-identity ownership checks remain |
| Outbound SCIM filtered lookup | Explicit Resources array and exact totalResults; at most one matching externalId; optional startIndex must be 1 and itemsPerPage must match; continuation/error responses fail | No POST/PATCH or successful item advancement from incomplete lookup; a missing previously linked resource requires inspection |
| Outbound SCIM group/user update | Complete bounded, unique member-value arrays before membership replacement, and before accepting a group as already up to date whenever it has reviewed managed or desired members (an omitted, `null` or paginated `members` field never counts as empty); the post-write read-back must report the same explicit membership; explicit boolean active state before disabling | Remote member removals must belong to the exact reviewed previous managed link; unexpected remote membership and omission of retained managed members fail closed. ETags protect the conditional PATCH. Authority, revision and lease are checked again before dispatch |

## Review policy

Each plan reports `removal_impact`: `disabled_users`, `missing_users`,
`removed_memberships` and `review_required`. The fixed baseline policy requires
review for every missing imported user or managed membership removal. Disabling
explicitly present users requires review when all active linked users would be
disabled, at least five would be disabled and that is at least 20% of active
linked users, or at least two would be disabled and that is at least 50%.
Equality crosses a threshold. Outbound SCIM measures departures against its
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

Remote delivery remains at least once and can be partial or uncertain. Revocation
cannot roll back a remote write already accepted by a peer. Inspect job errors
and partial results before replanning; local apply does not mean downstream work
has completed. Desired-state reconciliation still needs controller-mode
integration before the full P01 scope is complete.
Two-build parity, PostgreSQL/concurrency contracts and controlled real-peer
acceptance remain integration gates; local fake-peer tests establish only the
paths they exercise.
