# ENT-04 Microsoft Entra ID directory sync

[Implementation](../../src/cloud_directory.rs) and [tests](../../tests/cloud_directory.rs).

Entra sync imports users and selected groups through Microsoft Graph. It does not copy passwords, delete local users, or change user/group records until an authorized caller applies a reviewed plan. Validate paging, permissions, membership mapping and removal confirmation with a controlled Entra tenant before deployment; the repository tests use local mocks.

The token profile is OAuth 2.0 `client_credentials` against `token_url`, with `scope` defaulting to `https://graph.microsoft.com/.default`. The client secret is read from a 0600/0400 file on every plan and apply. Graph calls use `Authorization: Bearer` and do not follow redirects off the configured origin.

```toml
[entra_directories.corp]
tenant_id = "00000000-0000-0000-0000-000000000000"
token_url = "https://login.microsoftonline.com/00000000-0000-0000-0000-000000000000/oauth2/v2.0/token"
client_id = "entra-sync"
client_secret_file = "secrets/entra-client"
graph_url = "https://graph.microsoft.com"
scope = "https://graph.microsoft.com/.default"
username_prefix = ""

[entra_directories.corp.groups]
staff = "11111111-1111-1111-1111-111111111111"

[entra_directories.corp.attributes]
email = "mail"
display_name = "displayName"
external_id = "id"
```

`graph_url` is an origin with no path. Production is `https://graph.microsoft.com`. Loopback HTTP is accepted for tests. Relative secret paths are resolved from `riauth.toml`.

Users come from `GET /v1.0/users` with `$select` and `$top`, following `@odata.nextLink` only when it stays on that origin and under `/v1.0/`. Groups come from `/v1.0/groups`; members come from the plain `/v1.0/groups/{id}/members` endpoint, without advanced-query parameters. The connector follows member `@odata.nextLink` pages and reads `id`, the mapped mail attribute, `displayName`, and `accountEnabled`. `accountEnabled = false` disables the linked local user. The stable match key is the object id, not the mail address. `userPrincipalName` is not used unless you map email to it.

Run this plan example from the repository root, where `deployment-private/` is ignored by Git, or from a private operator directory:

```sh
mkdir -p deployment-private
riauth directory entra list
riauth directory entra plan corp --out deployment-private/entra-plan.json
jq '{changes, removal_impact}' deployment-private/entra-plan.json
riauth --if-revision "$(jq -r .revision deployment-private/entra-plan.json)" directory entra apply --plan deployment-private/entra-plan.json
```

The same operations are `GET /api/entra-directories`, `POST /api/entra-directories/{id}/plan`, `GET /api/entra-directory-plans/{id}`, and `POST /api/entra-directory-plans/{id}/apply`.

Each plan includes `removal_impact.disabled_users`, `missing_users`, `removed_memberships`, and `review_required`. A previously linked user missing from the snapshot, any mapped-group membership removal, a full linked-user disable, or a large partial disable requires explicit confirmation. When `review_required` is true, inspect the complete plan and apply with `riauth directory entra apply --plan deployment-private/entra-plan.json --confirm-removals`; the HTTP equivalent is `X-riAuth-Confirm-Cloud-Removals: <plan-id>` on the apply request. The header must contain that exact plan's ID. Apply still re-fetches the directory and checks the stored plan before changing accounts.

The agent needs `directory.read` and `directory.sync` on `entra/corp`, `user.write` for every affected username, and `group.members` for each allow-listed local group. `directory/corp` does not grant this sync. Plans are actor-bound, expire after five minutes, and omit access tokens and client secrets. Plan creation persists the plan and audit bookkeeping; user and group changes happen only at apply. Apply fetches the upstream directory again and rejects changes to the reviewed entries, directory configuration, or local revision. Generate and review a new plan after a conflict.

New users are explicit links stored by tenant id, directory id, and upstream object id. A matching email or username does not adopt an existing account. Username collisions abort the plan. Administrators, LDAP-linked users, and users linked to another directory or tenant are refused. The local username is chosen at first link from the mail local-part, or the object id when that name is not usable, plus `username_prefix`. Later mail changes update the mailbox and clear `email_verified`; they do not rename the account.

Allow-listed groups are matched by object id or mail. Apply makes membership of those local groups match upstream members who are linked to this directory. Unlinked users are not added. Groups outside the allow-list are left alone. Create the local groups before planning. Non-user members are ignored.

A completed sync that no longer returns a previously linked user disables that user, bumps the epoch, revokes that user's sessions, and queues back-channel logout. The user row is not deleted. `accountEnabled = false` does the same. A failed page, a repeated `@odata.nextLink`, a page cap, or a next link that leaves the configured host is not deletion: nobody is disabled. A successful Graph page must contain an array-valued `value` collection; null or missing collections and members without IDs fail the crawl. Credential failure, HTTP 401, and HTTP 429/5xx are retryable. Each directory allows 5 such failures in 15 minutes; the next call does not contact the upstream. One call does not retry internally. A successful fetch clears the counter. Secret-file replacement is picked up on the next plan or apply.

Application permissions for a live tenant should be limited to `User.Read.All`, `Group.Read.All`, and `GroupMember.Read.All`, with admin consent. Directory write permissions are not required. Two configured directories do not share links, even when an object id string is reused.

Disable invalidates local sessions and durably revokes child-agent credentials and Windows devices in the same transaction. It also enqueues the outbound SSF account-disabled event once for the transition. Re-enabling a directory user does not reactivate revoked credentials; reenroll or rotate them explicitly. See [parent ownership](ENT-02.md), [Windows revocation](ENT-13.md#revocation), and [Shared Signals](ENT-07.md#outbound-behavior).
