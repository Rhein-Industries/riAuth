# SCIM directory provisioning

The inbound SCIM base is `/scim/v2`. `Users` and `Groups` support GET/POST/PUT/PATCH/DELETE, filtered list queries and POST `.search`. Metadata is available at `ServiceProviderConfig`, `ResourceTypes`, and `Schemas`. Responses use `application/scim+json`, SCIM errors, resource locations and revision ETags.

Use a dedicated agent with `user.read`, `user.write`, `group.read`, `group.write` and `group.members` permissions for its allowed names. Each provisioning operator owns the records it creates; another operator cannot take ownership, even through a matching username or external ID. Operator credential rotation preserves ownership. Provisioning never creates or modifies human administrators. Run these examples from the repository root, where `deployment-private/` is ignored by Git, or from a private operator directory outside the checkout.

```sh
mkdir -p deployment-private
riauth agent create directory --ttl 86400 \
  --permission 'user.read=*' --permission 'user.write=*' \
  --permission 'group.read=*' --permission 'group.write=*' \
  --permission 'group.members=*' --out deployment-private/directory-agent.json
riauth --agent-file deployment-private/directory-agent.json scim Users --filter 'userName eq "alice"'
riauth --agent-file deployment-private/directory-agent.json --if-revision 12 --idempotency-key directory-create-alice \
  scim Users --method POST --file deployment-private/alice.scim.json
```

Agent mutations require `If-Match` with the current configuration revision, returned as the resource ETag or by `riauth revision`. Idempotency keys permit exact retries without duplicate creation. An unrelated configuration change can stale an ETag; fetch the current state and reconcile again. Integrations unable to supply these headers need an adapter. Operators must not reuse one provisioning credential across unrelated directories.

Supported user attributes are `userName`, `externalId`, `displayName`, `name`, `active`, `emails` and write-only `password`; `groups` is read-only. A newly provisioned account without a password has local password authentication disabled. Link it to an upstream source through a reviewed `source_links` manifest using the exact upstream subject, or establish credentials through the account lifecycle. Email addresses start unverified.

Supported group attributes are `displayName`, `externalId` and user `members`. Members must belong to the same provisioning operator. Atomic patches support add/replace/remove of supported attributes and `members[value eq "ID"]` removal. Disabling or deleting a user revokes its sessions/grants, Windows device bindings/tickets and parent-owned agent credentials. Setting a password enforces the configured [password-history limit](enterprise/ENT-08.md). Deletion keeps a local disabled identity and a SCIM tombstone so an old identifier cannot silently acquire another account; reprovisioning a deleted name currently requires an explicit local migration.

`Users.groups` and `Groups.members` both project durable membership, within the SCIM operator's ownership boundary. Temporary access grants affect authorization but are excluded from both SCIM views, outbound provisioning and user CSV reports. Approval, expiry and revocation therefore preserve reciprocal directory membership. See [temporary access](enterprise/ENT-01.md).

Profile boundaries: names are immutable, nested groups and enterprise/custom schemas are not implemented, filters support one `eq` expression on `userName`, `displayName`, `externalId` or `id`, results are capped at 1,000 per page, and bulk/sort are not advertised. SCIM records and source links are distinct: no automatic email-based linking occurs. Outbound SCIM is described below; [LDAP synchronization](ldap.md) has its own reviewed plan/apply workflow.

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
```

`token_file` is a static bearer token. To acquire and refresh an OAuth access token instead, omit `token_file` and set `oauth`. A target must use exactly one of those modes. OAuth supports `client_credentials` or a configured `refresh_token` file with Basic/post client authentication; access tokens stay in process memory and a SCIM 401 triggers one refresh/retry. See [OAuth for outbound SCIM](enterprise/ENT-12.md) for the configuration, token lifetime and opaque-token limitations.

```sh
mkdir -p deployment-private
riauth provision targets
riauth provision plan payroll --out deployment-private/payroll-plan.json
riauth provision apply --plan deployment-private/payroll-plan.json
riauth provision jobs
```

Grant the agent `provisioner.read` and `provisioner.sync` on `provisioner/payroll`, plus the usual current revision for applying a direct mutation. These permissions authorize reading and delivering the target's selected identity data. The target endpoint and credential paths are configured on the server, outside the agent API. A plan is actor-bound and expires after one hour. It records the local revision, target configuration and exact desired resources; the CLI verifies its saved copy against the server before applying. Creating a new plan for the same actor and target supersedes earlier plan snapshots; active jobs retain their immutable copy. A retained completed job remains an idempotent apply result by its ID after its plan snapshot is removed.

The worker persists progress and leases across restart and multiple nodes. It creates users before groups, uses issuer-namespaced `externalId` values, and never adopts an account merely because its username or email matches. It reconciles username, display name, active state, primary email and selected group membership. Only changed managed attributes are patched, preserving other attributes. Administrators are excluded. A user who leaves the selected groups or is disabled is deactivated remotely by a subsequently reviewed/applied plan; remote accounts are not deleted. Departed groups are emptied.

This profile requires an authenticated SCIM endpoint with `externalId eq` filtering, Users/Groups creation, PATCH and ETags for conditional updates. A successful empty `204 No Content` PATCH is read back and verified before the job advances. Responses, redirects and request duration are bounded; all non-loopback endpoints require verified HTTPS. The selected population is limited to 2000 users. Including historical links and groups, a plan is limited to 2064 resources and 2 MiB serialized size; the retained plan store is limited to 32 plans and 16 MiB. Completed and stale jobs retain their progress and error summary without the resource bodies; at most 64 jobs and 32 MiB of job records are retained, with the oldest terminal records removed when capacity is needed. If the remote external ID is ambiguous, a linked remote ID changes, or a linked account disappears, the worker stops that item for review. Local configuration or agent-authority changes mark an in-progress plan stale, including partial progress.

The job sends stable idempotency keys and checks for an existing external ID before creating an account. External delivery is **at least once**, not a transaction spanning both systems. For guaranteed duplicate suppression after an ambiguous POST, the target must honor idempotency keys or enforce the external ID's uniqueness. A target that cannot supply those guarantees needs an adapter or operator reconciliation; riAuth does not silently treat such delivery as exactly once. After a stale or partially applied job, inspect remote state and create a new plan. [Scheduled offboarding](enterprise/ENT-10.md) only performs local revocation; it does not automatically create or apply an outbound SCIM job. Provisioning tests exercise a second riAuth HTTP instance, conditional updates, preserved unmanaged attributes, groups, deactivation and permission revocation.
