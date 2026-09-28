# SCIM directory provisioning

The inbound SCIM base is `/scim/v2`. `Users` and `Groups` support GET/POST/PUT/PATCH/DELETE, filtered list queries and POST `.search`. Metadata is available at `ServiceProviderConfig`, `ResourceTypes`, and `Schemas`. Responses use `application/scim+json`, SCIM errors, resource locations and resource ETags.

Use a dedicated agent with `user.read`, `user.write`, `group.read`, `group.write` and `group.members` permissions for its allowed names. Each provisioning operator owns the records it creates; another operator cannot take ownership, even through a matching username or external ID. Operator credential rotation preserves ownership. Provisioning never creates or modifies human administrators. Run these examples from the repository root, where `deployment-private/` is ignored by Git, or from a private operator directory outside the checkout.

```sh
mkdir -p deployment-private
riauth agent create directory --ttl 86400 \
  --permission 'user.read=*' --permission 'user.write=*' \
  --permission 'group.read=*' --permission 'group.write=*' \
  --permission 'group.members=*' --out deployment-private/directory-agent.json
riauth --agent-file deployment-private/directory-agent.json scim Users --filter 'userName eq "alice"'
riauth --agent-file deployment-private/directory-agent.json --idempotency-key directory-create-alice \
  scim Users --method POST --file deployment-private/alice.scim.json
riauth --agent-file deployment-private/directory-agent.json scim Users --id RESOURCE_ID
riauth --agent-file deployment-private/directory-agent.json --idempotency-key directory-update-alice \
  scim Users --id RESOURCE_ID --method PATCH --if-version '"RESOURCE_ETAG"' --file deployment-private/alice.patch.json
```

For agent credentials, PUT/PATCH/DELETE require `If-Match` with the target resource's exact quoted `meta.version` (also returned in the HTTP `ETag` header). POST creates a new resource without `If-Match`. A stale version returns HTTP 412, while a missing version on an agent update or delete returns HTTP 428. The version changes when the effective User or Group changes, including direct changes to its projected fields or durable membership; unrelated management writes leave it stable. Idempotency keys permit exact retries without duplicate creation or a second update, including when the first update changed the ETag. `riauth revision` and `--if-revision` apply to management API writes, not inbound SCIM. Operators must not reuse one provisioning credential across unrelated directories.

Supported user attributes are `userName`, `externalId`, `displayName`, `name`, `active`, `emails` and write-only `password`; `groups` is read-only. User PATCH supports `name.formatted`, `name.givenName` and `name.familyName`, with an optional matching core User schema URN prefix. Add and replace on those paths set the named string, creating `name` when absent; add and replace of a `name` object merge its supported sub-attributes and preserve the others. Remove drops the selected sub-attribute or the whole `name`; removing an unassigned name path returns `noTarget`. Nested name PATCH writes accept only the three advertised string sub-attributes, each 1–200 bytes without control characters. Existing extra name sub-attributes survive supported PATCH updates but are not individually PATCH-addressable. Other `name.*` paths and unrelated schema-qualified paths are not advertised. A newly provisioned account without a password has local password authentication disabled. Link it to an upstream source through a reviewed `source_links` manifest using the exact upstream subject, or establish credentials through the account lifecycle. Email addresses start unverified.

Supported group attributes are `displayName`, `externalId` and user `members`. Members must belong to the same provisioning operator. Atomic patches support add/replace/remove of supported attributes. User `emails` and Group `members` accept object or array values for unfiltered add/replace. Add appends distinct entries; replace sets the array. Filtered paths support `emails[<filter>]` and `members[value eq "ID"]`, optionally followed by `.value`, `.type` or `.primary` for emails, or `.value` for members. Email filters use the same bounded `value`, `type` and `primary` equality/presence predicates, `and`/`or` and parentheses as list filters; all predicates select the same email entry. Matched complex entries are updated by the supplied sub-attributes, while filtered remove without a sub-attribute removes matched entries. A filtered add or replace with no match returns `noTarget`; a filtered remove with no match succeeds without changing the resource. Member `display` is server-controlled: an explicit `.display` PATCH path is rejected, and an echoed `display` in a member object is ignored and rebuilt from the owned User. Email PATCH entries have unique values ignoring ASCII case, at most eight entries are accepted, and setting one `primary` to true clears it on the other entries. Member IDs must be unique and owner-scoped, with at most 1,000 members. PATCH paths are limited to 1,024 bytes, eight filter predicates and three nesting levels. Disabling or deleting a user revokes its sessions/grants, Windows device bindings/tickets and parent-owned agent credentials. Setting a password enforces the configured [password-history limit](enterprise/ENT-08.md). Deletion keeps a local disabled identity and a SCIM tombstone so an old identifier cannot silently acquire another account; reprovisioning a deleted name currently requires an explicit local migration.

`Users.groups` and `Groups.members` both project durable membership, within the SCIM operator's ownership boundary. Temporary access grants affect authorization but are excluded from both SCIM views, outbound provisioning and user CSV reports. Approval, expiry and revocation therefore preserve reciprocal directory membership. See [temporary access](enterprise/ENT-01.md).

Profile boundaries: names are immutable, nested groups and enterprise/custom schemas are not implemented. GET list and POST `.search` share a filter parser with up to eight atomic predicates, three levels of parentheses or valuePath nesting, and a 1,024-byte limit. Case-insensitive `and` binds before `or`; parentheses override precedence. `eq` supports `displayName`, `externalId` and `id` on both collections, plus `userName`, `emails.value` and `active` on Users. `pr` supports those attributes and User `emails`. User valuePath filters support `emails[value eq "..."]`, `emails[type eq "..."]`, and `emails[primary eq true|false]`, with `pr`, `and`, `or` and grouping inside the brackets. All predicates inside one `emails[...]` must match the same effective email entry. String equality values must be complete JSON quoted strings with JSON escapes; boolean equality takes unquoted JSON `true` or `false`. Attribute names and operators are ASCII case insensitive, and only ASCII spaces separate tokens. Email value and type, userName and displayName equality are ASCII case insensitive; externalId and id are exact. Presence means assigned and nonempty; `active pr` matches either boolean value, including `false`, and `emails pr` matches a nonempty email array. Unsupported attributes/operators, other valuePaths, `not`, malformed escapes and control characters return `invalidFilter`. Filters apply after ownership and read permission checks, before pagination; results are capped at 1,000 per page. Bulk is not advertised. SCIM records and source links are distinct: no automatic email-based linking occurs. Outbound SCIM is described below; [LDAP synchronization](ldap.md) has its own reviewed plan/apply workflow.

Sorting is available on GET list and POST `.search` with `sortBy` and optional `sortOrder=ascending|descending` (default ascending). Both collections support `id`, `externalId` and `displayName`; Users also support `userName`, `active`, `emails.value`, `name.formatted`, `name.givenName` and `name.familyName`. Email sorting uses the primary entry, or the first entry when none is primary. Missing or empty values sort last ascending and first descending; equal sort values use ascending resource `id` for stable pagination. String sorting uses Unicode lowercase comparison except exact `id` and `externalId`; `active` sorts false before true ascending. Unsupported fields or orders, and an order without a field, return `invalidValue`. Sorting runs over owner-scoped, readable, filtered resources before pagination. Unsorted pages retain only the requested page; sorted queries use the existing in-memory scoped result set.

GET User/Group, GET list and POST `.search` support `attributes` and `excludedAttributes`. GET uses a comma-separated path list; `.search` uses an array of paths. The two options are mutually exclusive. Selection accepts supported core attributes, the advertised `name`, `emails`, `groups` and `members` sub-attributes, `meta.resourceType`, `meta.location` and `meta.version`, and optional matching core-schema URN prefixes. `id` and `schemas` remain in every resource. `meta` is returned by default but may be omitted or narrowed; the single-resource HTTP `ETag` and `Location` headers still come from the full authorized resource. Projection runs after ownership and read authorization and after list filtering, sorting and pagination. Unknown paths and write-only `password` are rejected; projected complex values contain only advertised sub-attributes. The list is capped at 32 paths and 1,024 bytes.

See [SCIM protocol](https://www.rfc-editor.org/rfc/rfc7644.html) and [SCIM schemas](https://www.rfc-editor.org/rfc/rfc7643.html). The local HTTP test covers credential ownership, retries, filtering, atomic patch failure, group membership and session invalidation; it does not establish compatibility with every directory product.

## Outbound provisioning

Server-configured targets can receive selected users and optional groups through agent-reviewed, immutable plans:

```toml
[scim_targets.payroll]
url = "https://payroll.example.com/scim/v2"
token_file = "payroll-scim-token"
groups = ["payroll-users"]
export_groups = true
# ca_file = "private-ca.pem"

[scim_reconciliation_modes]
payroll = "guarded-automatic"
```

`scim_reconciliation_modes` is an optional, per-target controller policy. An
omitted target uses `manual-review`. `guarded-automatic` queues plans with no
removals; `automatic` also queues below-threshold user deactivations. Both stop
at the shared removal review threshold and return the exact plan for operator
review. A controller trigger returns `snapshot_in_progress`, `awaiting_review`,
`queued` (with a durable job), or `in_progress`; queued work is not reported as
delivered. Each `snapshot_in_progress` call advances a persisted source and
ownership-link cursor by at most 128 records per collection. It has no plan ID
and cannot be applied. The `provision plan` CLI repeats these calls until the
snapshot is complete; an interrupted command can be run again to resume. If an old
leased job becomes stale, it returns `awaiting_prior_delivery` with a current
plan until the in-flight request settles, so replacement delivery cannot race it.
The trigger is available to server-side callers through
`Core::provisioning_reconcile`. A configured P02 controller can schedule it or
queue a source event through the reconciliation API. Unknown modes and
policies for unconfigured targets are rejected at configuration validation.
Changing a mode invalidates pending plans and stales queued jobs before another
remote dispatch.

A scheduled or event controller job records `snapshot_in_progress` as a compact
progress outcome and returns to the queue for the next page. Normal pages do
not spend the controller's four-attempt failure budget. Each resumed claim
rechecks its lease, configured credential and live scoped authority before
advancing the durable snapshot.

`token_file` is a static bearer token. To acquire and refresh an OAuth access token instead, omit `token_file` and set `oauth`. A target must use exactly one of those modes. OAuth supports `client_credentials` or a configured `refresh_token` file with Basic/post client authentication; access tokens stay in process memory and a SCIM 401 triggers one refresh/retry. See [OAuth for outbound SCIM](enterprise/ENT-12.md) for the configuration, token lifetime and opaque-token limitations.

```sh
mkdir -p deployment-private
riauth provision targets
riauth provision plan payroll --out deployment-private/payroll-plan.json
riauth provision apply --plan deployment-private/payroll-plan.json
riauth provision jobs
```

Grant the agent `provisioner.read` and `provisioner.sync` on `provisioner/payroll`, plus the usual current revision for applying a direct mutation. These permissions authorize reading and delivering the target's selected identity data. The target endpoint and credential paths are configured on the server, outside the agent API. A plan is actor-bound and expires after one hour. It records the local revision, target configuration and exact desired resources; the CLI verifies its saved copy against the server before applying. Creating a new plan for the same actor and target supersedes earlier plan snapshots; active jobs retain their immutable copy. A retained completed job remains an idempotent apply result by its ID after its plan snapshot is removed.

The worker persists progress and leases across restart and multiple nodes. It creates users before groups, uses issuer-namespaced `externalId` values, and never adopts an account merely because its username or email matches. It reconciles username, display name, active state, primary email and selected group membership. Only changed managed attributes are patched, preserving other attributes. Administrators are excluded. A user who leaves the selected groups or is disabled is deactivated remotely by a subsequently reviewed/applied plan; remote accounts are not deleted. Departed groups are emptied through explicit removal review. Plans include `removal_impact`; when `review_required` is true, inspect the resources and use `provision apply --plan deployment-private/payroll-plan.json --confirm-removals`, or send `X-riAuth-Confirm-Removals: <exact-plan-id>` to the apply endpoint. Apply checks exact content, current authority and the previously delivered managed links. See [connector removal safeguards](removal-safeguards.md) for thresholds and upgrade behavior.

Planning scans local users and ownership links in storage pages, persisting the
cursor, source revision, user and target link generations, selected group fingerprint and
quotas after each page. A changed source, group or link starts a fresh scan;
an incomplete scan never yields a disable plan. A draft expires after one hour
without progress and is limited to 100,000 scanned records per collection and 2 MiB of staged
data. The selected group population is limited to 2,000 member IDs, including
inactive members. These quotas stop very large or rapidly changing inventories
for operator inspection rather than treating an unseen record as departed.
Successful logins and other writes to authentication-only user fields do not
restart a source scan; changes to username, display name, email, enabled or
administrator status do. A completed plan binds the target link generation and
computes removal impact from its bounded set of reviewed link keys. Applying it
checks that generation and the exact saved link contents without scanning links
for other targets. Plans saved before this binding must be replanned before apply.

This profile requires an authenticated SCIM endpoint with `externalId eq` filtering, Users/Groups creation, PATCH and ETags for conditional updates. A successful empty `204 No Content` PATCH is read back and verified before the job advances. Responses, redirects and request duration are bounded; all non-loopback endpoints require verified HTTPS. The selected population is limited to 2000 users. Including historical links and groups, a plan is limited to 2064 resources and 2 MiB serialized size; the retained plan store is limited to 32 plans and 16 MiB. Completed and stale jobs retain their progress and error summary without the resource bodies; at most 64 jobs and 32 MiB of job records are retained, with the oldest terminal records removed when capacity is needed. If the remote external ID is ambiguous, a linked remote ID changes, or a linked account disappears, the worker stops that item for review. Local configuration or agent-authority changes mark an in-progress plan stale, including partial progress. Inconsistent filtered page totals/metadata never dispatch a write. Membership replacement, and accepting a group as already up to date while it has reviewed managed or desired members, requires a complete explicit member array with bounded unique IDs (an omitted, `null` or `membersNextLink`-paginated `members` field is rejected, not treated as empty), and refuses to remove remote members outside the reviewed previous managed snapshot. Missing or malformed active state cannot authorize a disable. Resolve incomplete data or unexpected remote drift before replanning.

The job sends stable idempotency keys and checks for an existing external ID before creating an account. External delivery is **at least once**, not a transaction spanning both systems. For guaranteed duplicate suppression after an ambiguous POST, the target must honor idempotency keys or enforce the external ID's uniqueness. A target that cannot supply those guarantees needs an adapter or operator reconciliation; riAuth does not silently treat such delivery as exactly once. After a stale or partially applied job, inspect remote state and create a new plan.

### Delivery outcomes

Each job reports `delivery_state`:

| `delivery_state` | Meaning |
| --- | --- |
| `pending` | Items remain. A retry that failed before sending, or whose write the target refused with a 4xx other than 408, 425 or 429, changed nothing remotely. |
| `ambiguous` | The current item's write was sent, and the reply was lost, a 5xx, or not verified by read-back. The next attempt reads the resource before any new write. It clears the state when it observes the item again. |
| `succeeded` | Every item was delivered and verified. |
| `failed` | The job stopped without completing: stale authority or configuration, an operator stop, or 12 attempts on one item. |

A stopped job that was `ambiguous` keeps that state, so inspect the target before trusting a replacement plan. `item` gives the position and kind of the resource the latest failed attempt concerned. Its `local_id` appears only for a viewer with `provisioner.read` on the target and `user.read` or `group.read` on that user or group. This applies to the job list, apply, stop and controller responses alike. With backoff capped at an hour, 12 attempts on one item take about two hours. Then the job stops, is audited as `provisioner.stop`, and releases the target for a new reviewed plan. `riauth provision stop <job-id>` (`POST /api/provisioning/jobs/{id}/stop`) stops an unfinished job at once. It needs `provisioner.sync` on the target and is audited the same way. A job whose item is leased keeps that lease until it settles and reads as `ambiguous`. That item's worker still records its verified result. PATCH idempotency keys include the version sent in `If-Match`, so a retry that read a changed version is a new request.

Disabling or deleting a linked account records deactivation intent for each target; see [offboarding deactivation](#offboarding-deactivation). Provisioning tests exercise a second riAuth HTTP instance, conditional updates, preserved unmanaged attributes, groups, deactivation and permission revocation.

## Offboarding deactivation

Every transaction that disables or deletes an enabled account (an administrator's disable, inbound SCIM, LDAP or cloud synchronization, desired-state apply, an SSF account-disabled signal, or [scheduled offboarding](enterprise/ENT-10.md)) also writes one row in `provisioning_deactivations` for each outbound user link whose target last reported the account active. That intent commits or aborts with the local revocation. The row binds the target, target URL, remote ID, external ID and link, and it is the only record of that target's outcome. Both editions record and deliver these rows; only scheduling offboarding jobs requires Platform.

The delivery worker handles one due row at a time and never records a remote outcome before the target confirms it:

1. An account that is enabled again closes the row as `superseded`; nothing is sent. A removed link, or a link or target URL binding that changed, closes it as `stale` for inspection.
2. A link that a reviewed provisioning job has already written as inactive closes the row as `delivered` with outcome `reviewed_delivery`. Reviewed jobs write links only after a verified read-back.
3. Dispatch requires a [scoped controller](removal-safeguards.md) declared for `scim/<target>`. Its agent's live `provisioner.sync` authority on `provisioner/<target>` is checked, and its credential file is read afresh and must authenticate as that agent. A deactivation is a removal, so the target's reconciliation mode applies. `manual-review` and `guarded-automatic` hold the row until a reviewed plan delivers the disable. `automatic` dispatches unless the shared P03 floor, counted over every previously delivered active link whose account is now disabled or deleted, requires review. A reviewed job holding a live lease on the target delays dispatch briefly.
4. The claim binds the controller authority, controller and target configuration, and link digest. That binding, the lease, the link and the account state are rechecked immediately before the conditional request. The worker reads the linked account, requires its `id`, `externalId` and a boolean `active`, and sends `PATCH active=false` only if the target still reports it active. The request carries `If-Match` and an idempotency key derived from the row, its epoch and that exact version. A retry of the identical request reuses the key. A retry after the version changed sends a new key, because some targets reject a reused key with a different precondition. A `204` is read back. Only the leasing worker records `delivered` (`deactivated` or `already_inactive`), a retry, or after five attempts `failed`. A missing remote account or a changed remote binding is `stale`.

Rows also report `delivery_state`, derived in this order:

1. `ambiguous` while `uncertain` is set, whatever the `status`. A PATCH was sent and its effect is unknown.
2. `succeeded` for `delivered`.
3. `cancelled` for `superseded`.
4. `pending` for `pending` or `running`, covering holds and retries that changed nothing.
5. `failed` for `stale` or `failed`.

`uncertain` is cleared only by an attempt that reads the account, or by a reviewed job's verified link. A refused PATCH is retried as `pending`. An account enabled again after an unverified PATCH does not close as `superseded` straight away. The row stays `pending` and `ambiguous` until the controller reads the account once, without writing. It then closes as `superseded` with outcome `remote_inactive` (that PATCH was applied, so a reviewed plan must reactivate the account) or `remote_active`. If the account is disabled again during that read, the row resumes delivery. `riauth provision retry-deactivation <id>` (`POST /api/provisioning/deactivations/{id}/retry`) takes a failed or stale row, re-binds it to the current link and evaluates it again. If a reviewed plan has delivered the disable since then, the row closes as delivered; if the account was re-enabled, it is superseded. The retry needs `provisioner.sync` on the target and is audited as `provisioner.deactivate.retry`. Its response is the full row only for a caller who could list it (`provisioner.read` on the target and `user.read` on the account). Otherwise it is limited to `id`, `target`, `status`, `delivery_state`, `hold`, `attempts` and `next_attempt`.

Held rows stay `pending`; `hold` names the reason: `awaiting_controller`, `awaiting_controller_authority`, `target_unconfigured`, `manual_mode`, `guarded_removal`, `removal_review_required`, `awaiting_prior_delivery` or `retry`. They are re-evaluated at an interval that grows with age to one hour, and they count as pending in the delivery queue metrics. The backlog alert therefore also reports offboarded accounts that are still active downstream. Delivery never rewrites managed links, so reviewed plans still count the departure until they deliver it. A reviewed plan made while the account was enabled cannot reactivate it: its job goes stale at that account. A write that was dispatched before the disable and lands after it records new intent. Terminal rows are kept for 90 days.

`riauth provision deactivations` (`GET /api/provisioning/deactivations`) lists the newest 1000 rows. Agents need `provisioner.read` on the target and `user.read` on the account. Delivery remains at least once: two overlapping attempts can each send the same disable, and a remote change made outside riAuth after delivery is not observed.
