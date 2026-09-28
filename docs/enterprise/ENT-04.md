# ENT-04 Microsoft Entra ID directory sync

[Implementation](../../src/cloud_directory.rs) and [tests](../../tests/cloud_directory.rs).

Entra sync imports users and selected groups through Microsoft Graph. It does not copy passwords, delete local users, or change user/group records until an authorized caller applies a reviewed plan. Validate paging, permissions, membership mapping and removal confirmation with a controlled Entra tenant before deployment; the repository tests use local mocks.

The token profile is OAuth 2.0 `client_credentials` against `token_url`, with `scope` defaulting to `https://graph.microsoft.com/.default`. The existing client-secret mode reads a 0600/0400 file on every plan and apply. A [certificate credential](https://learn.microsoft.com/en-us/entra/identity-platform/certificate-credentials) can instead sign a fresh PS256 client assertion with a matching RSA private key. Graph calls use `Authorization: Bearer` and do not follow redirects off the configured origin.

```toml
[entra_directories.corp]
tenant_id = "00000000-0000-0000-0000-000000000000"
token_url = "https://login.microsoftonline.com/00000000-0000-0000-0000-000000000000/oauth2/v2.0/token"
client_id = "22222222-2222-2222-2222-222222222222"
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

`graph_url` is an origin with no path. Production is `https://graph.microsoft.com`. Loopback HTTP is accepted for tests. Relative credential paths are resolved from `riauth.toml`.

For certificate mode, replace `client_secret_file` with `certificate_file = "secrets/entra-cert.pem"` and `private_key_file = "secrets/entra-key.pem"`. Configure exactly one credential mode. The PEM certificate must contain one valid 2048–8192 bit RSA certificate registered on the Entra application; the matching PEM private key must have owner-only permissions. The connector binds `aud` to the configured tenant-specific v2 token URL and sends the certificate's SHA-256 thumbprint as `x5t#S256`. For production certificate mode, `token_url` and `graph_url` must be a matching [Microsoft cloud pair](https://learn.microsoft.com/en-us/graph/deployments), and `scope` must be that Graph origin plus `/.default`.

To rotate, register the new certificate on the Entra app while the old one remains valid, then replace the certificate and private-key files. Each plan and apply reloads both files. A mismatched pair fails before the token request, with no fallback to the old secret; an existing plan still re-fetches Graph before account changes. Remove the old certificate from Entra after the new pair works. Validate the overlap and expiry behavior in a controlled tenant.

Users come from `GET /v1.0/users` with `$select` and `$top`, following `@odata.nextLink` only when it stays on that origin and the same collection path. Groups come from `/v1.0/groups`; each selected group's users come from [`/v1.0/groups/{id}/transitiveMembers`](https://learn.microsoft.com/en-us/graph/api/group-list-transitivemembers?view=graph-rest-1.0), which includes users inherited through nested groups. For each Entra collection, the connector requests `$count=true`, sends `ConsistencyLevel: eventual` on every page, and requires the first-page count to match the complete result. Missing or mismatched counts, failed pages, and unknown member types abort the snapshot. Only Graph user objects enter local groups. Graph's advanced-query index can lag a recent change, so inspect removal plans and validate timing in a controlled tenant.

The user snapshot reads `id`, the mapped mail attribute, `displayName`, and `accountEnabled`; `accountEnabled = false` disables the linked local user. Entra `attributes.external_id` must be `id`, because [UPN and email can change](https://learn.microsoft.com/en-us/entra/identity/hybrid/connect/howto-troubleshoot-upn-changes). `userPrincipalName` is not used unless you map email to it. Pages are capped at 20 per collection, 2,000 objects, 1 MiB per page, 4 MiB per collection, and 30 seconds per fetch.

Run this plan example from the repository root, where `deployment-private/` is ignored by Git, or from a private operator directory:

```sh
mkdir -p deployment-private
riauth directory entra list
riauth directory entra plan corp --out deployment-private/entra-plan.json
jq '{changes, removal_impact}' deployment-private/entra-plan.json
riauth --if-revision "$(jq -r .revision deployment-private/entra-plan.json)" directory entra apply --plan deployment-private/entra-plan.json
```

The same operations are `GET /api/entra-directories`, `POST /api/entra-directories/{id}/plan`, `GET /api/entra-directory-plans/{id}`, and `POST /api/entra-directory-plans/{id}/apply`.

Each plan includes `removal_impact.disabled_users`, `missing_users`, `removed_memberships`, and `review_required`. A previously linked user missing from the snapshot, any mapped-group membership removal, a full linked-user disable, or a large partial disable requires explicit confirmation. When `review_required` is true, inspect the complete plan and apply with `riauth directory entra apply --plan deployment-private/entra-plan.json --confirm-removals`; the HTTP equivalent is `X-riAuth-Confirm-Cloud-Removals: <plan-id>` on the apply request. The header must contain that exact plan's ID. The shared `X-riAuth-Confirm-Removals` header is also accepted. See [connector removal safeguards](../removal-safeguards.md) for exact thresholds, content/authority commitments and upgrade behavior. Apply still re-fetches the directory and checks the stored plan before changing accounts.

The agent needs `directory.read` and `directory.sync` on `entra/corp`, `user.write` for every affected username, and `group.members` for each allow-listed local group. `directory/corp` does not grant this sync. Plans are actor-bound, expire after five minutes, and omit access tokens and client secrets. Plan creation persists the plan and audit bookkeeping; user and group changes happen only at apply. Apply fetches the upstream directory again and rejects changes to the reviewed entries, directory configuration, or local revision. Generate and review a new plan after a conflict.

New users are explicit links stored by tenant id, directory id, and upstream object id. A matching email or username does not adopt an existing account. Username collisions abort the plan. Administrators, LDAP-linked users, and users linked to another directory or tenant are refused. The local username is chosen at first link from the mail local-part, or the object id when that name is not usable, plus `username_prefix`. Later mail changes update the mailbox and clear `email_verified`; they do not rename the account.

Allow-listed groups are matched by object id or mail. Apply makes membership of those local groups match upstream members who are linked to this directory. Unlinked users are not added. Groups outside the allow-list are left alone. Create the local groups before planning. Non-user members are ignored.

A completed sync that no longer returns a previously linked user disables that user, bumps the epoch, revokes that user's sessions, and queues back-channel logout. The user row is not deleted. `accountEnabled = false` does the same. A failed page, a repeated `@odata.nextLink`, a page cap, or a next link that leaves the configured host is not deletion: nobody is disabled. A successful Graph page must contain an array-valued `value` collection; null or missing collections and members without IDs fail the crawl. Credential failure, HTTP 401, and HTTP 429/5xx are retryable. Each directory allows 5 such failures in 15 minutes; the next call does not contact the upstream. One call does not retry internally. A successful fetch clears the counter. Credential-file replacement is picked up on the next plan or apply.

Application permissions for a live tenant should be limited to `User.Read.All`, `Group.Read.All`, and `GroupMember.Read.All`, with admin consent. [Hidden-membership groups](https://learn.microsoft.com/en-us/graph/api/group-list-transitivemembers?view=graph-rest-1.0) additionally require `Member.Read.Hidden`. Directory write permissions are not required. Two configured directories do not share links, even when an object id string is reused.

Disable invalidates local sessions and durably revokes child-agent credentials and Windows devices in the same transaction. It also enqueues the outbound SSF account-disabled event once for the transition. Re-enabling a directory user does not reactivate revoked credentials; reenroll or rotate them explicitly. See [parent ownership](ENT-02.md), [Windows revocation](ENT-13.md#revocation), and [Shared Signals](ENT-07.md#outbound-behavior).
