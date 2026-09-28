# Agent interface — riauth.cli/v1

riAuth is managed through the CLI. It does not require an embedded LLM or MCP server. Server-side permissions apply equally to CLI calls and direct HTTP requests.

## Bootstrap and permissions

A human administrator can grant the permissions needed by [the complete example](../examples/identity.toml). Run these examples from the repository root, where `deployment-private/` is ignored by Git; outside the checkout, use a private operator directory:

```sh
mkdir -p deployment-private
riauth agent create deployer \
  --permission user.read=user/alice --permission user.write=user/alice \
  --permission group.read=group/staff --permission group.write=group/staff \
  --permission group.members=group/staff \
  --permission client.read=client/reports --permission client.write=client/reports \
  --permission client.rotate=client/reports \
  --permission state.read=state/revision \
  --permission audit.read=audit/events \
  --ttl 86400 --out deployment-private/deployer.json
```

Select the credential with `--agent-file` or `RIAUTH_AGENT_FILE`. Its issuer and expiration are checked locally and by the server. Selecting it never falls back to a human session. Credentials expire after 60 seconds to 30 days. A human administrator can rotate with `agent rotate deployer --out deployment-private/replacement.json` or revoke with `agent revoke deployer`. Rotation immediately invalidates the previous token and preserves permissions.

`agent create` accepts optional `--parent USERNAME` (HTTP field `parent`). The parent must already exist, be enabled, and must not be an administrator. The stored link is that user's id (`parent_user`) and does not change on rotation. Ownership adds no permissions and does not grant the parent's sessions: the explicit permission list remains the ceiling. Disabling or deleting the parent immediately stops new agent authentication and durably revokes owned credentials in the same transaction. This applies to administrative, SCIM, manifest, LDAP/cloud-directory, inbound SSF and scheduled-offboarding transitions. Re-enabling the parent cannot restore revoked credentials; inherited active child credentials in older disabled-user snapshots are revoked on re-enable as well. Scheduled offboarding revalidates the creator’s current parent authority before execution. Omit `--parent` for an unowned agent; that previous behavior is unchanged. Human administrators still create, rotate, and revoke agents.

`riauth capabilities` enumerates the local artifact's compiled features, actions and resource kinds without opening a database; its `features` list is empty because it cannot establish instance usability. The running server's `GET /api/capabilities` reports `feature_states` and lists only locally usable profiles in `features`. Neither report grants an agent permission. Permissions use an exact `kind/name`, or `*` for every resource of that action. There are no implicit permission hierarchies. `client.write` does not include `client.rotate`. Group creation and membership changes have separate permissions. Agents cannot create administrators, alter existing administrator users, issue agent credentials, or authenticate/consent as end users.

`device.enroll=device/<id>` enrolls, lists, and revokes that Windows device. `device.enroll=*` covers every device id. Agents cannot enroll or replace a device for an administrator. The credential provider package is not part of this service; see [enterprise/ENT-13.md](enterprise/ENT-13.md).
`directory.read` and `directory.sync` authorize LDAP on `directory/<id>`. The same actions authorize Google Workspace on `workspace/<id>` and Microsoft Entra ID on `entra/<id>`. A resource of `*` matches every directory resource for that action, including cloud directories. See [Workspace](enterprise/ENT-03.md) and [Entra](enterprise/ENT-04.md).

Additional scoped enterprise actions are:

| Action/resource | Purpose |
| --- | --- |
| `mtls.read=user/<username>`, `mtls.bind=user/<username>` | Inspect or change HTTPS client-certificate bindings; separate from RADIUS `certificate.read` / `certificate.write`. |
| `user.offboard=user/<username>` | Schedule, inspect, reschedule or cancel offboarding. Per-target SCIM deactivation details also need `provisioner.read` on that target. |
| `ssf.configure=ssf/<id>` or `ssf.configure=*` | Read or change an owned outbound Shared Signals stream; `*` permits creation with a generated ID. |
| `ssf.manage=ssf/<id>` | Register pinned inbound signing trust or bind approved local subjects for the selected Shared Signals stream. |
| `audit.read=audit/events` | Audit review, audit CSV export and the Events Map; does not grant user CSV access. |
| `user.read=user/<username>` | Include the selected user in user inventory and CSV exports. |

PAM requester and approver actions use human sessions and configured approvers;
agent permissions do not grant a human session. See the [PAM contract](enterprise/ENT-01.md).

Grant `operations.backup=operations/backup` only to a backup custodian: backups contain the **entire instance**, including private signing keys, MFA secrets and credential records. This permission is broader than resource-scoped inventory access. `operations.read=operations/health` allows doctor, `operations.read=operations/metrics` allows metrics, and `operations.read=operations/logout` allows outbox inspection. `key.rotate=key/signing` permits signing-key rotation; `session.revoke=session/<id>` permits that session's revocation.

## Delegated human administration (M04 first slice)

A full human administrator can replace another enabled, non-administrator's delegated grants with `PUT /api/users/{username}/delegated-grants` using a JSON array. An empty array also revokes grants from a disabled or promoted account. `GET` on the same path reads them. The same routes are available to an administrator's signed-in browser at `/api/admin/users/{username}/delegated-grants` behind the portal read or write guards. A grant is tied to the exact named target and its stable stored ID; `*` and kind-wide scopes are rejected. Changes are live on the next management request and are audited with the assigning actor and scopes. A grant holder cannot change grants, even their own.

```json
[
  {"role":"help_desk","scope":"user/alice"},
  {"role":"application_owner","scope":"client/reports"}
]
```

`help_desk` may read its selected ordinary account and update its display name, email, enabled state, password, MFA reset, or sessions through the existing user update API. It cannot create an account; change administrator status, attributes, subjects or email verification; update itself; or update an administrator, another delegated person or a configured privileged-access approver. `application_owner` may read its selected client and update its name, redirect URIs, portal presentation, allowed browser origins and post-logout redirect URIs. It cannot create clients, rotate credentials, change sign-in policy, scopes, enabled state or other provider settings. Both roles can read the configuration revision for conditional writes. Their direct HTTP mutations require `If-Match`; the CLI's `--if-revision` sends the same header. The existing CLI user and client commands, direct API routes and browser administration routes all call the same server management methods. The browser page has not yet been tailored to hide unsupported controls.

The full administrator role and scoped agents retain their current behavior. Directory operator, auditor and security administrator human roles are later M04 work; grants do not confer those permissions. Desired-state plan/apply remains unavailable to delegated humans in this slice.

## Desired state

1. Read `riauth --json schema manifest` and `riauth --json capabilities`.
2. Write a JSON or TOML manifest with `api_version = "riauth/v1"`.
3. Run local `validate --file examples/identity.toml`. This checks structure without reading secrets; dependency and permission checks occur during server planning.
4. Run `plan --file examples/identity.toml --out deployment-private/plan.json`. Inspect its redacted changes and `removal_impact`.
5. Run `apply --plan deployment-private/plan.json --non-interactive --json --run-id <run>` using the same agent. Add `--confirm-removals` after reviewing a plan whose `removal_impact.review_required` is true.
6. Use `get`, `inventory`, `explain`, and `audit` to verify the result.

Include global `--agent-file deployment-private/deployer.json` on remote commands, or set its environment equivalent. Authorized automation may execute this workflow unattended; end-user consent remains a separate interaction.

Manifests reconcile named users, groups, clients, upstream sources and explicit source links. Other enterprise records, such as certificate bindings, SSF streams, Windows devices and offboarding jobs, use their dedicated APIs/commands. Omitted resources remain unchanged. Membership of an included group is authoritative. Users and clients are disabled with `enabled = false`; absence is not deletion. Client ID/type and existing user ID are immutable. Last-administrator protection is enforced.

Use `password_ref` or `password_hash_ref`, paired with `password_version`, and `secret_ref` paired with `secret_version`. References are `env:NAME` or `file:PATH`; the CLI resolves them on the agent's computer only when applying a credential change. File paths resolve from the CLI working directory; prefer absolute paths. Password hashes must come from an authorized offline export. Client secrets must be 32–1024 bytes. Do not store secrets in user attributes or literal claim mappings: those are public configuration.

An unchanged credential version preserves the existing credential. Changing the version requests rotation. Replanning an already reconciled manifest yields no changes, and applying that plan performs no resource mutations. `export --out deployment-private/current.json` contains visible state and stable IDs but omits credential references/hashes; add references before using an export to create users or confidential clients on another instance.

A manifest may set `issuer` to the riAuth issuer it was prepared for. Planning and applying then fail unless the instance's issuer is exactly that canonical URL, including any trailing slash. Converted Authentik manifests always set it; `export` never does.

Plans expire after 15 minutes and are bound to issuer, principal, current authority, controller mode, complete plan content and global configuration revision. They record explicit user, client, source and password disables and included-group membership removals. Missing manifest resources are unchanged. Every membership or client/source/password disable needs exact plan-ID confirmation; explicit user disables use the shared P03 threshold. The HTTP apply header is `X-riAuth-Confirm-Removals: <exact-plan-id>`. Apply recomputes impact and checks its review binding before reconciling. Applying is one database transaction: failed validation, missing secrets or stale revision commits no resource changes. A repeated successful apply returns its original result to a currently authorized actor, including after loss of the first HTTP response. Applied receipts remain available for roughly one day after plan expiry. The CLI checks applied status before rereading secrets. Unapplied plans created before the review binding was introduced must be replanned.

The optional `state_reconciliation_mode` setting accepts `manual-review`, `guarded-automatic` or `automatic` and controls the server-side `Core::state_reconcile` trigger. Omission stays manual. Guarded automatic stops on every removal. Automatic permits below-threshold explicit user disables, but both modes stop at the shared review floor. Passwordless non-admin user creation and group additions are eligible for automatic apply; other user updates, credential, client, source, source-link and administrator changes remain review-only. Automated apply accepts only changes that need no supplied secret. Scheduling and a remote trigger are separate work.

## Direct mutation retries

Prefer plan/apply for batches. User/client/group CRUD, client-secret rotation, signing-key rotation and agent credential management also support persisted idempotency receipts:

```sh
riauth --agent-file deployment-private/deployer.json --json revision
riauth --agent-file deployment-private/deployer.json \
  --if-revision 42 --idempotency-key reports-rotation-20260909 \
  --output-file deployment-private/reports-v2.json client rotate-secret reports
```

Use the actual returned revision. Direct agent mutations require `--if-revision`; its HTTP equivalent is `If-Match: "42"`. A stale revision fails. Retry the same logical operation with the same key, body and path; the server returns the committed result without rotating again, even though the original revision is now stale. A different request with that key fails. Authentication and the agent's current permissions are checked on replay.

Receipts return the original result for 24 hours. Expired receipts reject reuse for another six days, then are cleaned up. Never reuse an operation key for new work. After an uncertain operation outside the receipt window, inspect state before making a new plan. Failed transactions do not reserve the key. End-user login, OAuth exchanges and device approval do not use this mechanism; follow their protocol retry rules.

## Output, secrets and errors

With `--json`, success is:

```json
{"schema_version":"riauth.cli/v1","ok":true,"data":{"changed":true}}
```

Failure is:

```json
{"schema_version":"riauth.cli/v1","ok":false,"error":{"code":"conflict","message":"Configuration revision changed","http_status":409,"retryable":false},"exit_code":5}
```

Remote requests have a 30-second deadline by default, configurable with `--request-timeout SECONDS` or `RIAUTH_REQUEST_TIMEOUT` (1..=86,400), including body receipt. CSV reports write one bounded page at a time and atomically publish the complete private file; `rows` counts logical records, including quoted newlines.

Prompts/progress go to stderr. `--non-interactive` and non-TTY input never prompt. Missing required secrets fail. `schema` exposes JSON Schemas for manifests, plans, apply requests, users, clients, provider settings, agents, Authentik migration input, migration inventories, migration report findings and the CLI envelope. Help/version output remains ordinary CLI text.

| Exit | Meaning |
| --- | --- |
| 0 | Command succeeded; inspect fields such as migration `ready_for_plan` |
| 1 | Operation/local validation failure |
| 2 | CLI syntax/usage error |
| 3 | Authentication failure |
| 4 | Permission denied |
| 5 | Conflict / stale state |
| 6 | Rate limited or temporarily unavailable; bounded retry |

`--output-file` writes the complete result into a new mode-0600 file and returns only its path. Client creation/rotation requires it or explicit `--show-secrets`. Agent create/rotate and recovery-code generation require their own `--out` sink. Existing destinations are rejected. Preserve operation keys when retrying a credential delivery failure, using a new output path. OAuth token commands intentionally deliver requested tokens and also support `--output-file`.

## Inventory and explanations

`get client reports`, `get user alice`, and `get group staff` perform exact authorized lookup. `inventory clients --limit 100` returns `items`, `next_cursor` and `revision`; pass the opaque cursor through `--after` until null. Supported collections are users, groups, clients, sources and audit. `get source <id>` performs the corresponding exact source lookup. `--filter` matches resource-name substrings for users, groups, clients and sources. For audit inventory it matches an exact run ID only; the shared `events` resource name is never used as a match. Audit review and `report audit --run-id` also use exact run matching. Cursors are encrypted and bound to principal, collection, filter and revision and expire after one hour. Restart enumeration after a conflict. Lists disclose only permitted resources.

`explain reports alice --scope 'openid profile groups'` requires read access to both user and client. It reports policy reasons and claim previews. `--mfa` simulates an authenticated MFA session; it never creates one. Claims use attributes and bounded declarative mappings, never executable code.

`--run-id` is recorded with authenticated actor, generated request ID and redacted changes. Plan application additionally records its plan ID. Audit is retained for 90 days. `report audit --run-id <run> --out deployment-private/audit.csv` exports an exact automation run; ordinary `audit --limit 100` shows recent events. [Audit review](enterprise/ENT-09.md) exposes exact actor/target/run filters and action prefixes. [CSV reports](enterprise/ENT-15.md) walk every page and write a new private file. Before/after changes cover user, group, client, agent, source and Windows-device records; other event families retain attribution without a universal object diff.

Operational recovery and backup instructions are in [operations.md](operations.md). Configuration changes are atomic locally. Logout delivery is asynchronous, durable and retried; external applications must implement logout-token verification and deduplication.

Plan changes include `secret_references` for only the credentials that will change. The CLI resolves those references for user passwords/hashes, TOTP factors, application secrets and source secrets. Explicit source links and password disabling require no secret value. Unchanged credential versions do not require their files/environment variables to remain available; retrying an applied plan returns its stored result before reading any references. Recreate pending plans produced by an older binary if they lack this metadata for a credential change.

The readers for saved CLI/agent credentials, referenced plan secrets, imported keys, Vault/SMTP/SCIM/cloud-directory/alert credentials and PostgreSQL connection files require bounded regular UTF-8 files with owner-only permissions on Unix (`0600` or `0400`). Validation and reading use the same open descriptor. The CLI reports a missing or invalid private file without printing its contents.
