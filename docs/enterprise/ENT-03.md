# ENT-03 Google Workspace directory sync

[Implementation](../../src/cloud_directory.rs) and [tests](../../tests/cloud_directory.rs).

Workspace sync imports users and selected groups through the Admin SDK Directory API. It does not copy passwords, delete local users, or change user/group records until an authorized caller applies a reviewed plan. Validate authorization and synchronization behavior with a controlled Workspace tenant before deployment; the repository tests use local mocks.

The supported direct mode follows [Google's service-account JWT bearer flow](https://developers.google.com/identity/protocols/oauth2/service-account). A Workspace super administrator must grant domain-wide delegation to the service account's **numeric client ID** in the Admin console. Configure a dedicated Workspace user with only the Directory read privileges needed by this connector as `delegated_subject`. The signed assertion has that user as `sub`; Google constrains access by that user's privileges and the granted OAuth scopes. For users only, the connector requests `admin.directory.user.readonly`. When mapped groups are configured, it also requests `admin.directory.group.readonly` and `admin.directory.group.member.readonly` as documented in [Directory API scopes](https://developers.google.com/workspace/admin/directory/v1/guides/authorizing). Authorize exactly those scopes in the Admin console.

```toml
[workspace_directories.corp]
customer_id = "C01234567"
domain = "example.com"
directory_url = "https://admin.googleapis.com"
username_prefix = ""

[workspace_directories.corp.direct_auth]
key_file = "secrets/workspace-service-account.json"
delegated_subject = "directory-reader@example.com"

[workspace_directories.corp.groups]
staff = "staff@example.com"
```

The key file is Google's service-account JSON file, stored outside source control with owner-only permissions (0600 or 0400). Relative paths resolve from `riauth.toml`. The connector verifies its `type` and official `token_uri`, then reads the key afresh for each plan and apply so key replacement takes effect without process restart. Standard Platform builds accept only the exact `https://admin.googleapis.com` Directory origin and `https://oauth2.googleapis.com/token` token endpoint in direct mode. The direct HTTP client ignores ambient system and environment proxies, and does not follow redirects. Only builds with the explicit `test-support` feature accept a fake peer using the same literal HTTP loopback IP and port for both endpoints; do not enable that feature in shipped builds. Assertions use RS256, expire after one hour, and are exchanged for a bearer token once per bounded sync. A token response with less than 60 seconds of lifetime is rejected. Authentication failures count against the same per-directory retry budget as broker failures. The key file and access token are never included in a plan. Protect and rotate service-account keys according to [Google's key guidance](https://docs.cloud.google.com/iam/docs/best-practices-for-managing-service-account-keys).

The broker mode remains available for deployments that provide tokens through a separate trusted endpoint. It retains the normal HTTP client's proxy behavior.

In broker mode, the token profile is OAuth 2.0 `client_credentials` (`grant_type`, `client_id`, `client_secret`, and `scope` when it is non-empty). The client secret is read from a 0600/0400 file on every plan and apply. Google's public token endpoint does not issue Admin SDK access tokens with this grant; point `token_url` at an endpoint that implements the profile. Directory calls then use `Authorization: Bearer`.

```toml
[workspace_directories.corp]
customer_id = "C01234567"
domain = "example.com"
token_url = "https://directory-broker.example.com/token"
client_id = "workspace-sync"
client_secret_file = "secrets/workspace-client"
directory_url = "https://admin.googleapis.com"
scope = ""
username_prefix = ""

[workspace_directories.corp.groups]
staff = "staff@example.com"

[workspace_directories.corp.attributes]
email = "primaryEmail"
display_name = "name.fullName"
external_id = "id"
```

`directory_url` is an origin with no path. Production is `https://admin.googleapis.com`. Loopback HTTP is accepted for a local peer. Relative secret paths are resolved from `riauth.toml`.

Users come from `GET /admin/directory/v1/users` with `customer`, `domain`, and `maxResults`, following `nextPageToken` on that same URL. Groups come from `/admin/directory/v1/groups`, and members from `/admin/directory/v1/groups/{id}/members`. The connector reads `id`, `primaryEmail`, `name.fullName`, and `suspended`. `orgUnitPath` may be present and is not imported. `suspended = true` disables the linked local user. Attribute names are configurable; the stable match key is the external id, not the email.

Run this plan example from the repository root, where `deployment-private/` is ignored by Git, or from a private operator directory:

```sh
mkdir -p deployment-private
riauth directory workspace list
riauth directory workspace plan corp --out deployment-private/workspace-plan.json
jq '{changes, removal_impact}' deployment-private/workspace-plan.json
riauth --if-revision "$(jq -r .revision deployment-private/workspace-plan.json)" directory workspace apply --plan deployment-private/workspace-plan.json
```

The same operations are `GET /api/workspace-directories`, `POST /api/workspace-directories/{id}/plan`, `GET /api/workspace-directory-plans/{id}`, and `POST /api/workspace-directory-plans/{id}/apply`.

Workspace plan and apply requests read at most five source pages and return `decision: snapshot_in_progress` with a durable `snapshot_id` until the user, group, and selected membership pages finish. Repeat the same request, including the plan ID and removal confirmation header on apply, to resume; the CLI does this automatically. Apply progress also includes `operation: apply_validation` and `plan_id`. The apply cursor is bound to the exact stored plan and review commitment. Controller validation of a pending Workspace plan uses the planning cursor and retains the plan ID only when the completed source still matches. Each draft retains a page token, parsed users, selected group IDs, and the current collection's seen IDs. It expires after five idle minutes. Planning restarts on authority, revision, or configuration changes; apply rejects those changes and requires a new plan. Each collection is limited to 20 pages and 2,000 objects, each response to 1 MiB and 200 rows, and the entire crawl and serialized draft to 4 MiB. Plans are limited to 7 MiB so their backup records fit below the 8 MiB frame ceiling with headroom. An unfinished or invalid crawl cannot produce a plan or change local users and groups. Each request has a 30-second source budget; the five-minute plan expiry still bounds the whole apply validation.

The optional [source crawl quotas](../removal-safeguards.md#source-crawl-quotas) can tighten these page, object, byte and draft-expiry bounds. A quota change requires a new plan.

Each plan includes `removal_impact.disabled_users`, `missing_users`, `removed_memberships`, and `review_required`. A previously linked user missing from the snapshot, any mapped-group membership removal, a full linked-user disable, or a large partial disable requires explicit confirmation. When `review_required` is true, inspect the complete plan and apply with `riauth directory workspace apply --plan deployment-private/workspace-plan.json --confirm-removals`; the HTTP equivalent is `X-riAuth-Confirm-Cloud-Removals: <plan-id>` on the apply request. The header must contain that exact plan's ID. The shared `X-riAuth-Confirm-Removals` header is also accepted. See [connector removal safeguards](../removal-safeguards.md) for exact thresholds, content/authority commitments and upgrade behavior. Apply still re-fetches the directory and checks the stored plan before changing accounts.

The agent needs `directory.read` and `directory.sync` on `workspace/corp`, `user.write` for every affected username, and `group.members` for each allow-listed local group. `directory/corp` does not grant this sync. Plans are actor-bound, expire after five minutes, and omit access tokens and client secrets. Plan creation persists the plan and audit bookkeeping; user and group changes happen only after a complete apply crawl matches every reviewed entry. Apply rejects changes to the reviewed entries, directory configuration, local revision, or actor authority. Generate and review a new plan after a conflict. Workspace list pagination does not provide a point-in-time transaction across users, groups, and memberships; upstream changes that occur during the crawl and leave the final staged entries unchanged cannot be detected.

New users are explicit links stored by Workspace customer, directory id, and upstream id. A matching email or username does not adopt an existing account. Username collisions abort the plan. Administrators, LDAP-linked users, and users linked to another directory or tenant are refused. The local username is chosen at first link from the email local-part, or the external id when that name is not usable, plus `username_prefix`. Later email changes update the mailbox and clear `email_verified`; they do not rename the account or take over a different user.

Allow-listed groups are matched by upstream id or email. Apply makes membership of those local groups match upstream members who are linked to this directory. Unlinked users are not added. Groups outside the allow-list are left alone, including groups removed from the allow-list. Create the local groups before planning.

A completed sync that no longer returns a previously linked user disables that user, bumps the epoch, revokes that user's sessions, and queues back-channel logout. The user row is not deleted. Suspension does the same. A failed page, a repeated page, a page cap, or any other incomplete crawl is not deletion: nobody is disabled. A missing collection is accepted as empty only on the first Workspace page with the expected collection `kind`; null collections and members without IDs fail the crawl. Credential failure, HTTP 401, and HTTP 429/5xx are retryable. Each directory allows 5 such failures in 15 minutes; the next call does not contact the upstream. One call does not retry internally. A successful fetch clears the counter. Secret-file replacement is picked up on the next plan or apply.

Two configured directories do not share links, even when an upstream id string is reused. Reapplying an unchanged plan does not duplicate users.

Disable invalidates local sessions and durably revokes child-agent credentials and Windows devices in the same transaction. It also enqueues the outbound SSF account-disabled event once for the transition. Re-enabling a directory user does not reactivate revoked credentials; reenroll or rotate them explicitly. See [parent ownership](ENT-02.md), [Windows revocation](ENT-13.md#revocation), and [Shared Signals](ENT-07.md#outbound-behavior).
