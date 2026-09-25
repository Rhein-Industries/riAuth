# ENT-03 Google Workspace directory sync

[Implementation](../../src/cloud_directory.rs) and [tests](../../tests/cloud_directory.rs).

Workspace sync imports users and selected groups through the Admin SDK Directory API. It does not copy passwords, delete local users, or change user/group records until an authorized caller applies a reviewed plan. Validate the token broker and synchronization behavior with a controlled Workspace tenant before deployment; the repository tests use local mocks.

The supported token profile is OAuth 2.0 `client_credentials` (`grant_type`, `client_id`, `client_secret`, and `scope` when it is non-empty). The client secret is read from a 0600/0400 file on every plan and apply. Google's public token endpoint does not issue Admin SDK access tokens with this grant; point `token_url` at an endpoint that implements the profile. Directory calls then use `Authorization: Bearer`.

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

`directory_url` is an origin with no path. Production is `https://admin.googleapis.com`. Loopback HTTP is accepted for a local broker. Relative secret paths are resolved from `riauth.toml`.

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

Each plan includes `removal_impact.disabled_users`, `missing_users`, `removed_memberships`, and `review_required`. A previously linked user missing from the snapshot, any mapped-group membership removal, a full linked-user disable, or a large partial disable requires explicit confirmation. When `review_required` is true, inspect the complete plan and apply with `riauth directory workspace apply --plan deployment-private/workspace-plan.json --confirm-removals`; the HTTP equivalent is `X-riAuth-Confirm-Cloud-Removals: <plan-id>` on the apply request. The header must contain that exact plan's ID. Apply still re-fetches the directory and checks the stored plan before changing accounts.

The agent needs `directory.read` and `directory.sync` on `workspace/corp`, `user.write` for every affected username, and `group.members` for each allow-listed local group. `directory/corp` does not grant this sync. Plans are actor-bound, expire after five minutes, and omit access tokens and client secrets. Plan creation persists the plan and audit bookkeeping; user and group changes happen only at apply. Apply fetches the upstream directory again and rejects changes to the reviewed entries, directory configuration, or local revision. Generate and review a new plan after a conflict.

New users are explicit links stored by Workspace customer, directory id, and upstream id. A matching email or username does not adopt an existing account. Username collisions abort the plan. Administrators, LDAP-linked users, and users linked to another directory or tenant are refused. The local username is chosen at first link from the email local-part, or the external id when that name is not usable, plus `username_prefix`. Later email changes update the mailbox and clear `email_verified`; they do not rename the account or take over a different user.

Allow-listed groups are matched by upstream id or email. Apply makes membership of those local groups match upstream members who are linked to this directory. Unlinked users are not added. Groups outside the allow-list are left alone, including groups removed from the allow-list. Create the local groups before planning.

A completed sync that no longer returns a previously linked user disables that user, bumps the epoch, revokes that user's sessions, and queues back-channel logout. The user row is not deleted. Suspension does the same. A failed page, a repeated page, a page cap, or any other incomplete crawl is not deletion: nobody is disabled. A missing collection is accepted as empty only on the first Workspace page with the expected collection `kind`; null collections and members without IDs fail the crawl. Credential failure, HTTP 401, and HTTP 429/5xx are retryable. Each directory allows 5 such failures in 15 minutes; the next call does not contact the upstream. One call does not retry internally. A successful fetch clears the counter. Secret-file replacement is picked up on the next plan or apply.

Two configured directories do not share links, even when an upstream id string is reused. Reapplying an unchanged plan does not duplicate users.

Disable invalidates local sessions and durably revokes child-agent credentials and Windows devices in the same transaction. It also enqueues the outbound SSF account-disabled event once for the transition. Re-enabling a directory user does not reactivate revoked credentials; reenroll or rotate them explicitly. See [parent ownership](ENT-02.md), [Windows revocation](ENT-13.md#revocation), and [Shared Signals](ENT-07.md#outbound-behavior).
