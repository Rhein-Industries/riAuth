# SCIM directory provisioning

The inbound SCIM base is `/scim/v2`. `Users` and `Groups` support GET/POST/PUT/PATCH/DELETE, filtered list queries and POST `.search`. Metadata is available at `ServiceProviderConfig`, `ResourceTypes`, and `Schemas`. Responses use `application/scim+json`, SCIM errors, resource locations and resource ETags.

Use a dedicated agent with `user.read`, `user.write`, `group.read`, `group.write` and `group.members` permissions for its allowed names. Each provisioning operator owns the records it creates; another operator cannot take ownership, even through a matching username or external ID. Operator credential rotation preserves ownership. Provisioning never creates or modifies human administrators. Run these examples from the repository root, where `deployment-private/` is ignored by Git, or from a private operator directory outside the checkout. A human administrator must read the current numeric revision with `riauth revision` before creating the agent; use a unique, stable key for that one creation.

```sh
mkdir -p deployment-private
riauth --if-revision '<revision>' --idempotency-key directory-agent-create-001 agent create directory --ttl 86400 \
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

The agent credential is disclosed only on the first successful creation. A server-side exact retry of agent creation returns 409 `credential_already_issued` without the token; a CLI retry needs a new unused `--out` path if the first file exists. Inspect the agent and rotate its credential if the file was never written. The management revision flags on `agent create` do not replace the SCIM resource ETag used by later provisioning writes.

For SCIM writes made with agent credentials, PUT/PATCH/DELETE require `If-Match` with the target resource's exact quoted `meta.version` (also returned in the HTTP `ETag` header). SCIM POST creates a new resource without `If-Match`. A stale version returns HTTP 412, while a missing version on an agent update or delete returns HTTP 428. The version changes when the effective User or Group changes, including direct changes to its projected fields or durable membership; unrelated management writes leave it stable. A management change publishes that version for the one live SCIM group bound to the directory group. The write reads SCIM records 128 at a time and retains that group alone; a second live group with the same identity fails closed. GET, If-Match and write responses share that version. Its `groups` and `members` arrays are read 128 records at a time, so a conditional update does not retain the opposite collection. A User PATCH starts from that same projected `active` value, display name, and email, and leaves any of them unchanged when the patch does not target it. Idempotency keys permit exact retries without duplicate creation or a second update, including when the first update changed the ETag. `riauth revision` and `--if-revision` apply to management API writes, not inbound SCIM. Operators must not reuse one provisioning credential across unrelated directories.

Supported user attributes are `userName`, `externalId`, `displayName`, `name`, `active`, `emails` and write-only `password`; `groups` is read-only. User PATCH supports `name.formatted`, `name.givenName` and `name.familyName`, with an optional matching core User schema URN prefix. Add and replace on those paths set the named string, creating `name` when absent; add and replace of a `name` object merge its supported sub-attributes and preserve the others. Remove drops the selected sub-attribute or the whole `name`; removing an unassigned name path returns `noTarget`. Nested name PATCH writes accept only the three advertised string sub-attributes, each 1–200 bytes without control characters. Existing extra name sub-attributes survive supported PATCH updates but are not individually PATCH-addressable. Other `name.*` paths and unrelated schema-qualified paths are not advertised. A newly provisioned account without a password has local password authentication disabled. Link it to an upstream source through a reviewed `source_links` manifest using the exact upstream subject, or establish credentials through the account lifecycle. Email addresses start unverified.

Supported group attributes are `displayName`, `externalId` and user `members`. Members must belong to the same provisioning operator. Atomic patches support add/replace/remove of supported attributes. User `emails` and Group `members` accept object or array values for unfiltered add/replace. Add appends distinct entries; replace sets the array. Filtered paths support `emails[<filter>]` and `members[value eq "ID"]`, optionally followed by `.value`, `.type` or `.primary` for emails, or `.value` for members. Email filters use the same bounded `value`, `type` and `primary` equality/presence predicates, `and`/`or` and parentheses as list filters; all predicates select the same email entry. Matched complex entries are updated by the supplied sub-attributes, while filtered remove without a sub-attribute removes matched entries. A filtered add or replace with no match returns `noTarget`; a filtered remove with no match succeeds without changing the resource. Member `display` is server-controlled: an explicit `.display` PATCH path is rejected, and an echoed `display` in a member object is ignored and rebuilt from the owned User. Across POST, PUT and PATCH, email entries have unique values ignoring ASCII case and are limited to eight. `type` must be a nonempty string of at most 64 bytes without control characters, and `primary` must be a boolean. Full writes normalize recognized email sub-attribute names case-insensitively and reject conflicting spellings; unrelated extra sub-attributes are preserved. A PATCH that sets one `primary` to true clears it on the other entries. Member IDs must be unique and owner-scoped, with at most 1,000 members. A membership update reads SCIM users 128 records at a time and retains that directory group's other members, not the operator's user collection. PATCH paths are limited to 1,024 bytes, eight filter predicates and three nesting levels. Disabling or deleting a user revokes its sessions/grants, Windows device bindings/tickets and parent-owned agent credentials. Setting a password enforces the configured [password-history limit](enterprise/ENT-08.md). Deletion keeps a local disabled identity and a SCIM tombstone so an old identifier cannot silently acquire another account; reprovisioning a deleted name currently requires an explicit local migration.

User deletion removes a durable group membership only when the operator also has `group.members` authority for that exact group. Other memberships remain on the disabled local identity, including access-denial groups if the identity is later re-enabled. Deletion still revokes sessions and retains its tombstone. Full User and Group writes also validate that the complete stored record can be decoded; excessive stored JSON complexity returns `invalidValue` and rolls back the entire write. Supported opaque metadata that fits that bound remains available.

`Users.groups` and `Groups.members` both project durable membership, within the SCIM operator's ownership boundary. Temporary access grants affect authorization but are excluded from both SCIM views, outbound provisioning and user CSV reports. Approval, expiry and revocation therefore preserve reciprocal directory membership. See [temporary access](enterprise/ENT-01.md).

Profile boundaries: names are immutable, nested groups and enterprise/custom schemas are not implemented. GET list and POST `.search` share a filter parser with up to eight atomic predicates, three levels of parentheses or valuePath nesting, and a 1,024-byte limit. Case-insensitive `and` binds before `or`; parentheses override precedence. `eq` supports `displayName`, `externalId` and `id` on both collections, plus `userName`, `emails.value` and `active` on Users. `pr` supports those attributes and User `emails`. User valuePath filters support `emails[value eq "..."]`, `emails[type eq "..."]`, and `emails[primary eq true|false]`, with `pr`, `and`, `or` and grouping inside the brackets. All predicates inside one `emails[...]` must match the same effective email entry. String equality values must be complete JSON quoted strings with JSON escapes; boolean equality takes unquoted JSON `true` or `false`. Attribute names and operators are ASCII case insensitive, and only ASCII spaces separate tokens. Email value and type, userName and displayName equality are ASCII case insensitive; externalId and id are exact. Presence means assigned and nonempty; `active pr` matches either boolean value, including `false`, and `emails pr` matches a nonempty email array. Unsupported attributes/operators, other valuePaths, `not`, malformed escapes and control characters return `invalidFilter`. Filters apply after ownership and read permission checks, before pagination; results are capped at 1,000 per page. Bulk is not advertised. SCIM records and source links are distinct: no automatic email-based linking occurs. Outbound SCIM is described below; [LDAP synchronization](ldap.md) has its own reviewed plan/apply workflow.

Sorting is available on GET list and POST `.search` with `sortBy` and optional `sortOrder=ascending|descending` (default ascending). Both collections support `id`, `externalId` and `displayName`; Users also support `userName`, `active`, `emails.value`, `name.formatted`, `name.givenName` and `name.familyName`. Email sorting uses the primary entry, or the first entry when none is primary. Missing or empty values sort last ascending and first descending; equal sort values use ascending resource `id` for stable pagination. String sorting uses Unicode lowercase comparison except exact `id` and `externalId`; `active` sorts false before true ascending. Unsupported fields or orders, and an order without a field, return `invalidValue`. Sorting runs over owner-scoped, readable, filtered resources before pagination. Unsorted and `sortBy=id` pages use store-key cursor scans and retain only the requested page, in either ID direction. Other sorts retain at most 4,096 candidate resources: for a positive `count`, `startIndex - 1 + min(count, 1000)` must be at most 4,096. A larger window returns HTTP 400 with SCIM `invalidValue` before scanning the collection; reduce the page range or use `sortBy=id`. A zero-count request retains no candidates and may use any `startIndex`. Exact `totalResults` still requires scanning the whole collection under the same read snapshot, including for empty or ID-sorted pages.

GET User/Group, GET list and POST `.search` support `attributes` and `excludedAttributes`. GET uses a comma-separated path list; `.search` uses an array of paths. The two options are mutually exclusive. Selection accepts supported core attributes, the advertised `name`, `emails`, `groups` and `members` sub-attributes, `meta.resourceType`, `meta.location` and `meta.version`, and optional matching core-schema URN prefixes. `id` and `schemas` remain in every resource. `meta` is returned by default but may be omitted or narrowed; the single-resource HTTP `ETag` and `Location` headers still come from the full authorized resource. Projection runs after ownership and read authorization and after list filtering, sorting and pagination. Unknown paths and write-only `password` are rejected; projected complex values contain only advertised sub-attributes. The list is capped at 32 paths and 1,024 bytes.

See [SCIM protocol](https://www.rfc-editor.org/rfc/rfc7644.html) and [SCIM schemas](https://www.rfc-editor.org/rfc/rfc7643.html). The local HTTP test covers credential ownership, retries, filtering, atomic patch failure, group membership and session invalidation; it does not establish compatibility with every directory product.

The [inbound SCIM recipe](recipes/platform-inbound-scim.md) names `scim_http_provisioning_is_owned_atomic_retriable_and_deprovisions_sessions`, separates its HTTP checks from its in-process calls, and records the source behavior that function leaves unasserted. That test was not run for the recipe. No named SCIM client is connected.

## Outbound provisioning

Before delivery, an operator with `provisioner.sync` on `provisioner/payroll` can send an authenticated, empty `POST /api/provisioning/targets/payroll/test-connection` to riAuth (use the saved operator bearer; no `If-Match` or `Idempotency-Key` is required). The server uses only that configured target's URL, CA and private static/OAuth credential and requests `Users?startIndex=1&count=1`; it accepts a valid partial first page without following it. HTTP 200 returns `connected`, `checked_at`, fixed `component`, `safety: "no_scim_writes"` and `next_action`, plus fixed `error` on a failed check; authorization refusal remains a refusal after an in-flight reply. A failure advises `check_scim_configuration` or `check_scim_credential_and_users_access`: inspect the server configuration/private credential and the peer's TLS, token grant and Users-read authority before trying again. This uses the existing shared connector/target budget and may update existing OAuth cache/freshness metadata and admission bookkeeping; it creates no connection receipt, plan, delivery job, link or identity change. A readable file or successful local plan does not prove connectivity, and a passing probe proves neither a full crawl, Groups/filter/write support, mapping correctness nor remote delivery. Replace private credential files on the server under their existing permissions to rotate them; the next probe rereads them. No credential, path, target URL or returned user data is accepted from or disclosed to the caller.

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

The [outbound SCIM recipe](recipes/platform-outbound-scim.md) records `outbound_scim_plans_provision_groups_preserve_remote_attributes_disable_departures_and_stop_stale_jobs` in [tests/identity/policy.rs](../tests/identity/policy.rs). It separates that function's loopback assertions from the 12-attempt item stop, the controller's four-attempt budget, and a named SaaS directory. The test was not run for the recipe.

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

`GET /api/operations/provisioning` counts every stored job and lists at most 50 redacted attention rows for `failed`, `ambiguous`, and `pending` jobs that already have `error`. It requires `operations.read` on `operations/provisioning`, and each listed row also requires `provisioner.read` on that target. The aggregate reports `has_error` and a fixed `next_action` and leaves the stored error text on `GET /api/provisioning/jobs`. See [provisioning job diagnostics](roadmap/o06-provisioning-job-diagnostics.md).

A stopped job that was `ambiguous` keeps that state, so inspect the target before trusting a replacement plan. An operator can record what the target shows with `riauth provision resolve <job-id> --observed applied|not_applied|absent --evidence <reference>` (`POST /api/provisioning/jobs/{id}/resolve`). See [resolving ambiguity](#resolving-ambiguity). `item` gives the position and kind of the resource the latest failed attempt concerned. Its `local_id` appears only for a viewer with `provisioner.read` on the target and `user.read` or `group.read` on that user or group. This applies to the job list, apply, stop and controller responses alike. With backoff capped at an hour, 12 attempts on one item take about two hours. Then the job stops, is audited as `provisioner.stop`, and releases the target for a new reviewed plan. `riauth provision stop <job-id>` (`POST /api/provisioning/jobs/{id}/stop`) stops an unfinished job at once. It needs `provisioner.sync` on the target and is audited the same way. A job whose item is leased keeps that lease until it settles and reads as `ambiguous`. That item's worker still records its verified result. PATCH idempotency keys include the version sent in `If-Match`, so a retry that read a changed version is a new request.

Reviewed workers persist `dispatch_started` under their lease before the first
OAuth or SCIM request. The fence remains through token acquisition, 401 refresh,
every retry, response handling and verification reads. Each send checks the
owned lease; each write also rechecks current authority, source and lease expiry
after credential acquisition. Verification reads can finish under the owned
fence after stop or expiry. During ordinary delivery, only the owning worker's
outcome transaction clears the fence. A 60-second lease or any elapsed grace
cannot prove settlement.
Acknowledgement means that worker has finished the attempt and cannot send
again. A timed-out request can still have an unknown remote outcome; it remains
ambiguous and is never reported as success.

An expired lease explicitly recorded as unstarted can be reclaimed safely: the
old worker cannot acquire the fence after expiry or replacement. A started or
legacy untracked lease is held for acknowledgement, retained through cleanup,
and blocks replacement delivery and dismissal. A claimant marks such an expired
attempt stopped and ambiguous without taking its lease. If that worker crashed,
the hold persists until [audited dispatch recovery](#recovering-an-abandoned-dispatch)
records external quiescence. There is no time-only force-clear. Stop pre-fence worker
versions before upgrading; mixed-version workers cannot enforce this protocol.

Disabling or deleting a linked account records deactivation intent for each target; see [offboarding deactivation](#offboarding-deactivation). Provisioning tests exercise a second riAuth HTTP instance, conditional updates, preserved unmanaged attributes, groups, deactivation and permission revocation.

## Offboarding deactivation

Every transaction that disables or deletes an enabled account (an administrator's disable, inbound SCIM, LDAP or cloud synchronization, desired-state apply, an SSF account-disabled signal, or [scheduled offboarding](enterprise/ENT-10.md)) also writes one row in `provisioning_deactivations` for each outbound user link whose target last reported the account active. That intent commits or aborts with the local revocation. The row binds the target, target URL, remote ID, external ID and link, and it is the only record of that target's outcome. Both editions record and deliver these rows; only scheduling offboarding jobs requires Platform.

The delivery worker handles one due row at a time and never records a remote outcome before the target confirms it:

1. An account that is enabled again closes the row as `superseded`; nothing is sent. A removed link, or a link or target URL binding that changed, closes it as `stale` for inspection.
2. A link that a reviewed provisioning job has already written as inactive closes the row as `delivered` with outcome `reviewed_delivery`. Reviewed jobs write links only after a verified read-back.
3. Dispatch requires a [scoped controller](removal-safeguards.md) declared for `scim/<target>`. Its agent's live `provisioner.sync` authority on `provisioner/<target>` is checked, and its credential file is read afresh and must authenticate as that agent. A deactivation is a removal, so the target's reconciliation mode applies. `manual-review` and `guarded-automatic` hold the row until a reviewed plan delivers the disable. `automatic` dispatches unless the shared P03 floor, counted over every previously delivered active link whose account is now disabled or deleted, requires review. A reviewed job holding a live lease or unacknowledged dispatch on the target delays dispatch until it settles.
4. The claim binds the controller authority, controller and target configuration, and link digest. That binding, the lease, the link and the account state are rechecked immediately before the conditional request. The worker reads the linked account, requires its `id`, `externalId` and a boolean `active`, and sends `PATCH active=false` only if the target still reports it active. The request carries `If-Match` and an idempotency key derived from the row, its epoch and that exact version. A retry of the identical request reuses the key. A retry after the version changed sends a new key, because some targets reject a reused key with a different precondition. A `204` is read back. Only the leasing worker records `delivered` (`deactivated` or `already_inactive`), a retry, or after five attempts `failed`. A missing remote account or a changed remote binding is `stale`.

Deactivation attempts persist their own `dispatch_started` pin before the first
OAuth or SCIM send. The pin covers refresh, retries and verification; every
actual send rechecks ownership and every PATCH rechecks live admission after
authentication. The 180-second lease limits admission, not settlement. An
expired started or legacy lease becomes `failed`, `uncertain: true`, with
`hold: awaiting_dispatch_ack`, retaining the lease even on the final attempt.
Another claimant cannot take it. Retry, resolution, dismissal, intent enqueueing
and cleanup cannot clear it. The owning worker can still acknowledge its result;
otherwise use the explicit recovery protocol below. No network operation or
wait is added to the transaction that revokes the account locally.

Rows also report `delivery_state`, derived in this order:

1. `ambiguous` while `uncertain` is set, whatever the `status`. A PATCH was sent and its effect is unknown.
   Dismissing the row preserves this ambiguity.
2. `dismissed` for an operator waiver without an uncertain write. This never means remote success.
3. `resolved` when an operator attested `applied` or `absent` for an ambiguity no attempt could settle. This is never `succeeded`.
4. `succeeded` for `delivered`.
5. `cancelled` for `superseded`.
6. `pending` for `pending` or `running`, covering holds and retries that changed nothing.
7. `failed` for `stale` or `failed`.

`uncertain` is cleared only by an attempt that reads the account, or by a reviewed job's verified link. A refused PATCH is retried as `pending`. An account enabled again after an unverified PATCH does not close as `superseded` straight away. The row stays `pending` and `ambiguous` until the controller reads the account once, without writing. It then closes as `superseded` with outcome `remote_inactive` (that PATCH was applied, so a reviewed plan must reactivate the account) or `remote_active`. If the account is disabled again during that read, the row resumes delivery. `riauth provision retry-deactivation <id>` (`POST /api/provisioning/deactivations/{id}/retry`) takes a failed or stale row, re-binds it to the current link and evaluates it again. If a reviewed plan has delivered the disable since then, the row closes as delivered; if the account was re-enabled, it is superseded. The retry needs `provisioner.sync` on the target and is audited as `provisioner.deactivate.retry`. Its response is the full row only for a caller who could list it (`provisioner.read` on the target and `user.read` on the account). Otherwise it is limited to `id`, `target`, `status`, `delivery_state`, `hold`, `attempts` and `next_attempt`.

Ordinary held rows stay `pending`; `hold` names the reason: `awaiting_controller`, `awaiting_controller_authority`, `target_unconfigured`, `manual_mode`, `guarded_removal`, `removal_review_required`, `awaiting_prior_delivery` or `retry`. They are re-evaluated at an interval that grows with age to one hour, and they count as pending in the delivery queue metrics. The backlog alert therefore also reports offboarded accounts that are still active downstream. Delivery never rewrites managed links, so reviewed plans still count the departure until they deliver it. A reviewed plan made while the account was enabled cannot reactivate it: its job goes stale at that account. A write that was dispatched before the disable and lands after it records new intent. Terminal rows are kept for 90 days, except dismissed rows, pinned rows and rows with dispatch recovery history, which remain retained.

### Operator view

Administrators can open **Delivery outcomes** at `/admin#/deliveries` to inspect
retained deactivation requests, separately from the current local account state.
The view distinguishes queued, held and leased attempts, ambiguity, verified
completion, operator attestations and audited dismissals. Each person's detail
page links to their visible requests. Results are a manually refreshed snapshot
of up to 1,000 visible retained rows; an empty list does not establish remote
completion. Deleted local identities currently fail closed and are not listed.

Eligible held, stale or failed requests offer **Dismiss further attempts**, with
a required reason, evidence and acknowledgment. The browser uses the existing
management operation through guarded same-origin routes, with the reviewed row
revision and an idempotency key. The server checks current authority, state and
leases and retains the audit. Dismissal never labels the remote account inactive.
Retry, ambiguity resolution and reviewed provisioning job controls remain in the
management API and CLI.

### Resolving ambiguity

#### Unlinked user Creates

An initial user Create can commit at a provider while its response is lost.
Before sending that Create, riAuth persists `unlinked_create` provenance under
the same durable dispatch fence: source job, immutable local user ID, target
URL, external ID and request idempotency key. A disable or delete records an
offboarding obligation from that provenance in the local revocation transaction,
even when no verified link exists. There is no network or target-admission wait
in that transaction. An unknown Create stops its job; another job cannot POST
the same user to that target while this source remains unresolved, even if an
external-ID lookup temporarily returns no account.

The deactivation entrypoint also examines one retained source job per pass,
using a separate durable cursor and fixed sweep boundary. It reconstructs
missing obligations for already disabled or deleted users, including legacy
ambiguous jobs with compacted snapshots. Missing legacy bindings remain unknown;
they are never guessed from a username or the current target URL. This local
reconciliation runs independently of reviewed provisioning and its capacity.
It leaves existing connector due cursors and target permits intact.

These obligations are `stale`, `ambiguous`, and held as
`unlinked_create_requires_settlement`. They have no verified `remote_id` and
cannot be retried or rebound to a later link. Each source job has its own row,
separate from any subsequently verified linked account. A later successful
Create response can queue ordinary linked deactivation, but does not erase an
unlinked obligation already recorded during its send. Retention and job capacity
eviction preserve the source provenance and these rows. Original Create
resolution, local disable, an empty lookup, an elapsed lease, or a client timeout
is never reported as remote offboarding success.

Reconciliation of an unlinked row is an explicit administrator action:

1. Stop or fence all old workers that could resume the source request. If a
   dispatch pin remains, first use the audited abandoned-pin recovery protocol
   below; this resolution cannot bypass it.
2. Establish with the provider that **every original request has settled or
   been cancelled and cannot commit later**. An unreachable provider, a single
   GET, an empty search or a rotated local credential is insufficient.
3. Discover every matching remote identity using retained external ID/request
   evidence and provider records, account for duplicates and delayed visibility,
   and verify every account inactive or absent. Apply any required disable at
   the provider before attesting `applied` (offboarding satisfied) or `absent`.
   An active identity leaves the obligation unresolved.
4. Submit the fresh row revision, an idempotency key, bounded evidence, and both
   settlement attestations to the existing resolve endpoint:

```json
{
  "observed": "applied",
  "evidence": "OPS-92: source requests settled; all matching accounts disabled",
  "create_settlement": {
    "revision": "<deactivation revision>",
    "workers_quiesced": true,
    "remote_requests_settled": true
  }
}
```

The route is `POST /api/provisioning/deactivations/{id}/resolve`, with the usual
management `If-Match` and `Idempotency-Key` headers. The CLI equivalent is
`riauth --idempotency-key OPS-92 provision resolve-deactivation <id> --observed applied --evidence "OPS-92: source settled; all matches disabled" --revision <revision> --workers-quiesced --remote-requests-settled`.
Scoped agents cannot supply this infrastructure/provider attestation; an
administrator must still pass current immutable-user read authorization.

The operation retains the original ambiguity evidence and source record,
records the proof in the row and audit, stops the source plan, and discharges
that source's prospective obligation. It reports `resolved`, never `delivered`
or `succeeded`, and performs no remote request. Receipt replay rechecks current
user readability. An explicit audited dismissal remains a waiver, preserves
ambiguity, and does not authorize a replacement Create. If external settlement
or complete remote discovery cannot be established, keep the obligation open
or record that waiver; do not claim remote revocation. riAuth cannot verify
these external facts automatically. Already deleted source jobs or lost legacy
local-user identity cannot be reconstructed from nothing and require external
inventory review. Source and row retention can exhaust the existing job cap;
there is no automatic evidence archival or unlinked-resolution UI in this slice.

#### Other ambiguous deliveries

Some ambiguity cannot be settled by any attempt: a deactivation row that is `stale`, for example because its link was removed, or `failed`, or a stopped job whose current item stayed ambiguous. For these, an operator records what the target shows:

```sh
riauth provision resolve-deactivation <id> --observed applied --evidence "Checked the payroll console; ticket OPS-42"
riauth provision resolve <job-id> --observed not_applied --evidence "Payroll shows the old name; ticket OPS-43"
```

The HTTP routes are `POST /api/provisioning/deactivations/{id}/resolve` and `POST /api/provisioning/jobs/{id}/resolve`, each with body `{"observed": ..., "evidence": ...}`.

- **`observed`:** `applied` (the write took effect), `not_applied` (the target shows the earlier state) or `absent` (the remote resource is gone).
- **`evidence`:** 1-280 characters including surrounding whitespace, without control characters or credentials, naming where the check was made. Whitespace-only evidence is rejected.
- **Authorization:** `provisioner.sync` on the target. The caller must also be able to read what it attests about: `provisioner.read` on the target, plus `user.read` on the current account, or `user.read` / `group.read` on the job's actual local item. A missing job item can be recovered only from the retained plan at its cursor; without a matching local identity, resolution is denied and evidence stays redacted. A resolution is accepted only while the record is ambiguous. Stop and resolve retain a started or untracked job lease until its worker acknowledges settlement; no elapsed grace can release it. Resolution remains available after that acknowledgement.

A resolution clears `uncertain` and records the observation, evidence, actor and time. It changes neither the record's original target, remote identity, epoch, status, outcome nor error. Nothing is written to the target. The audit event (`provisioner.deactivate.resolve` or `provisioner.resolve`) keeps the same evidence in `details.context`.

The effect depends on the record:

- **Deactivation, `applied` or `absent`:** the row reads as `resolved`, never as delivered or succeeded, and it can no longer be retried.
- **Deactivation, `not_applied`:** the row reads as `failed`. It can be retried if its link still exists.
- **Stopped job:** it stays stopped and reads as `failed`. A fresh reviewed plan delivers the remaining work.

### Recovering an abandoned dispatch

A crashed worker or an older worker without dispatch tracking can leave a pin
that never receives acknowledgement. Recovery is an **administrator-only**
management action; scoped agents cannot attest infrastructure or provider
quiescence. It requires all of the following external work, recorded in evidence:

1. Terminate or fence every old worker that could resume the attempt, including
   suspended, partitioned and legacy nodes, and prevent those processes from
   restarting. Pausing a scheduler or waiting for lease expiry is insufficient.
2. Establish with the provider that prior requests have finished or were
   cancelled and cannot commit later. Revoke old remote credentials or otherwise
   fence old request authority as needed. A local credential-file change, a
   single GET, a timeout or provider unreachability is not this proof.
3. Read the pinned record again and submit its exact revision, reason and
   evidence with an idempotency key and both explicit attestations below.

riAuth checks authority, identity readability, revision, expiry, pin state and
the required attestations. It cannot independently verify that external workers
and provider requests have been quiesced. If either fact cannot be established,
keep the pin; do not attest it. Expiry only identifies a recovery candidate.

```sh
riauth --idempotency-key OPS-82 provision recover-deactivation-dispatch <id> \
  --revision <revision> --reason worker_lost \
  --workers-quiesced --remote-requests-settled \
  --evidence "OPS-82: old nodes fenced; provider confirmed prior requests drained"
```

For reviewed jobs, use `provision recover-dispatch <job-id>` and the listing's
`state_revision` (the separate `revision` is the plan's configuration revision).
The routes are `POST /api/provisioning/deactivations/{id}/recover-dispatch` and
`POST /api/provisioning/jobs/{id}/recover-dispatch`, with body:

```json
{"revision":"...","reason":"worker_lost","workers_quiesced":true,"remote_requests_settled":true,"evidence":"OPS-82"}
```

Reasons are `worker_lost` and `legacy_untracked`; the latter requires a lease
without dispatch tracking. Evidence has the same 1–280 character limit and
credential restrictions as resolution. A missing idempotency key, live lease,
stale revision, unstarted lease or missing attestation is rejected. Receipt
replays recheck the current identity's readability and never repeat recovery.

Recovery retires the old owner, leaves `uncertain: true`, and records the old
pin/state, reviewed revision, reason, evidence, administrator and time in
`dispatch_recoveries` and an audit event (`provisioner.deactivate.recover_dispatch`
or `provisioner.recover_dispatch`, under `details.context.attestation`). The
deactivation stays `stale` with `hold: recovered_dispatch`; a reviewed job stays
stopped with its cursor unchanged. Neither is queued, resolved, dismissed or
reported delivered by recovery. Inspect the remote identity before a separate
resolution, dismissal, explicit retry or newly reviewed plan. Retries read the
remote account before writing. Recovery history survives retention and
re-enqueueing; at 16 entries per record, further recovery fails closed without
discarding evidence. Reviewed jobs with recovery history are exempt from
capacity eviction; if retained evidence fills the job capacity, new jobs are
refused instead of deleting that history.

### Dismissing an undeliverable intent

When a remote identity is gone or remote state is permanently unverifiable, an
operator can waive further attempts on one held, failed or stale deactivation:

```sh
riauth --idempotency-key OPS-42 provision dismiss-deactivation <id> \
  --revision <row-revision> --reason permanently_unverifiable \
  --evidence "Retired payroll tenant cannot be inspected; exception OPS-42"
```

Read `revision` from `provision deactivations` immediately before deciding. It
binds every persisted field of that row, including worker changes. The HTTP
route is `POST /api/provisioning/deactivations/{id}/dismiss`, with body
`{"revision":"...","reason":"permanently_unverifiable","evidence":"..."}`
and a required `Idempotency-Key` header. Reasons are `remote_absent` and
`permanently_unverifiable`; evidence follows the same 1–280 character and
credential restrictions as resolution. Both humans and agents need the row
revision and key. Agents also supply the usual configuration `If-Match`
(`--if-revision`). An exact retry replays its receipt without another audit
event; changing the request under the same key conflicts.

Dismissal needs `provisioner.sync` and `provisioner.read` on the target plus
`user.read` on the named account. Running rows, delivered or superseded rows,
satisfied resolutions, and targets with a live or settling reviewed-job lease
cannot be dismissed. A started or untracked reviewed-job dispatch fences
dismissal until its worker acknowledges settlement, whether the job is active
or stale. Stopping that job, lease expiry and cleanup cannot bypass the fence.
Deactivation listings, full retry responses, resolutions and dismissals load the
current account by immutable `user_id` before checking its read scope. The stored
username is historical evidence: renaming or reusing it never transfers access.
If that local identity no longer exists, these account-detail operations fail
closed; the durable intent remains stored.

Full delivery-action responses, including idempotency receipt replays, recheck
the returned identity's current read scope before exposing account details or
operator evidence. A rename or deletion can therefore deny a formerly readable
receipt without repeating or undoing its committed action. The saved receipt
remains unchanged and replays exactly if that identity becomes readable again.
Write-only retry receipts remain minimal; they do not gain account details.

The row becomes `status: dismissed` and leaves the automatic queue. Its original
identity, epoch, hold, attempts, error, outcome and `uncertain` flag stay intact.
The waiver records reason, evidence, actor, time, prior status and reviewed row
revision, also audited as `provisioner.deactivate.dismiss`. A previously
ambiguous row stays `delivery_state: ambiguous`; otherwise it reads as
`dismissed`. The offboarding summary remains `incomplete` once no targets are
pending. That aggregate is [offboarding diagnostics](roadmap/o06-offboarding-diagnostics.md). The redacted row read is [deactivation diagnostics](roadmap/o06-deactivation-diagnostics.md). Dismissal performs no remote request or managed-link update.

The intent and waiver survive cleanup and restart. Retry and resolve reject a
dismissed row; repeated enqueue for the same disable preserves it. A new disable
epoch records a separate intent, and reviewed provisioning plans remain
available for subsequent reconciliation.

`riauth provision deactivations` (`GET /api/provisioning/deactivations`) lists the newest 1000 rows. Agents need `provisioner.read` on the target and `user.read` on the account. Delivery remains at least once: two overlapping attempts can each send the same disable, and a remote change made outside riAuth after delivery is not observed.
