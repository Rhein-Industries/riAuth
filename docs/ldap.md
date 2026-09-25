# LDAP directories

riAuth imports LDAP identities and authenticates their passwords through LDAP. Administration and login use the normal JSON API and CLI. No directory password hashes are copied. Configure a named directory on every riAuth node with the same settings:

```toml
[directories.staff]
url = "ldap://directory.example.test:389"
transport = "starttls"
bind_dn = "cn=riauth,ou=services,dc=example,dc=test"
password_file = "secrets/ldap-password"
ca_file = "directory-ca.pem"
user_base = "ou=people,dc=example,dc=test"
user_filter = "(objectClass=inetOrgPerson)"
id_attribute = "entryUUID"
username_attribute = "uid"
display_attribute = "cn"
email_attribute = "mail"
username_prefix = ""

[directories.staff.group_user_filters]
staff = "(memberOf=cn=staff,ou=groups,dc=example,dc=test)"
```

Create the local groups first. Each group filter searches **users** under `user_base`, intersected with `user_filter`; LDAP must provide the corresponding user attribute, such as AD's `memberOf` or OpenLDAP's memberof overlay. Use explicit filters for nested membership if supported by the directory. Map AD's binary `objectGUID` as the stable ID and `sAMAccountName` as the username; exclude disabled accounts in `user_filter`. Binary and text stable IDs are stored without lossy conversion. DNs and emails are never used to adopt existing accounts.

`starttls` requires a successful TLS upgrade before binding. `ldaps` requires an `ldaps://` URL. Certificate chain and hostname validation stay enabled; `ca_file` extends the trust roots. Plain `loopback` transport is restricted to a literal loopback address for tests. Credential files require owner-only permissions (0600 or 0400), contain just the password, and resolve relative to `riauth.toml`. The service bind needs read/search rights to every selected entry and attribute. Anonymous and empty-password binds are rejected. Run the plan example from the repository root, where `deployment-private/` is ignored by Git, or from a private operator directory.

```sh
mkdir -p deployment-private
riauth directory list
riauth directory plan staff --out deployment-private/ldap-plan.json
# Review entries, create/update/disable actions, group changes and revision.
riauth --if-revision "$(jq -r .revision deployment-private/ldap-plan.json)" directory apply --plan deployment-private/ldap-plan.json
riauth login alice --mfa
```

The agent needs `directory.read` and `directory.sync` on `directory/staff`, `user.write` for every affected old/new username, and `group.members` for each affected group. It cannot take over an administrator or a local password account. A username collision aborts the complete plan. New accounts have local passwords disabled and unverified email. Renames retain the local ID, pairwise seed, factors and subject overrides. Removed users are disabled; sync changes revoke existing sessions and grants. Disabling a linked user also revokes that user's Windows device bindings/tickets and parent-owned agents and queues an account-disabled event for configured Shared Signals streams. Only memberships previously owned by this directory are removed; unrelated memberships are preserved.

Plans last five minutes, are immutable and actor-bound, and include the local revision and directory configuration fingerprint. Apply fetches the complete LDAP result again and refuses changed data, changed permissions or stale local state. Local application is atomic and supports the usual conditional/idempotent mutation headers. Keep the same idempotency key when retrying an HTTP apply after an ambiguous response. LDAP searches themselves are not a transactional snapshot: concurrent directory changes can require another plan. Searches reject referrals, partial/error results, duplicate identities, multivalued mapped attributes and limits exceeded (2,000 users, 32 groups, 4 MiB of returned attributes, bounded network/overall time). They do not treat an error as an empty directory. A deliberately complete empty result produces a reviewable disable plan.

Normal `login`, request approval and OIDC step-up dispatch imported accounts to the directory. Each password login rechecks the selected entry's immutable ID and eligibility before binding with the submitted password. Local TOTP/recovery codes still apply; replay is rejected. Directory outages never fall back to a local password. A password login gives the directory 0.8 seconds in total for the connection (including TLS), the service bind, the entry re-check and the user bind; a directory that has not answered by then counts as an outage (`directory_unavailable` for `riauth login`, the uniform browser 401 on the sign-in pages), so a hung directory cannot make directory-bound usernames stand out by response time or hold the four credential permits. Synchronization (plan) keeps its 5-second step timeouts. Existing sessions are invalidated by synchronization or local revocation; riAuth does not continuously query LDAP on every token use. Schedule reviewed plan/apply runs at the required removal interval. Removing a configured directory invalidates its identities on that node; synchronize configuration across nodes.

`scripts/test-ldap.sh` starts a private OpenLDAP server with a disposable CA, uses actual STARTTLS, and tests import, a 205-entry paged result, rename continuity, MFA/recovery replay, collision protection, revoked permissions, stale plans, failed searches and removal. Linux requires `slapd`/`ldap-utils`; macOS uses Homebrew OpenLDAP because Apple's bundled daemon uses a different TLS setup. It does not modify an existing directory. For applications that bind and search riAuth, see the [LDAP provider](ldap-provider.md).

For cloud directories, [Google Workspace](enterprise/ENT-03.md) and [Microsoft Entra](enterprise/ENT-04.md) use separate `directory workspace` / `directory entra` commands and source-specific permissions. These imports do not enable LDAP password authentication or infer upstream SSO links from email. Configure explicit source links or another supported authentication method.

Protocol references: [LDAP operations (RFC 4511)](https://www.rfc-editor.org/rfc/rfc4511.html), [authentication (RFC 4513)](https://www.rfc-editor.org/rfc/rfc4513.html), [paged results (RFC 2696)](https://www.rfc-editor.org/rfc/rfc2696.html), [ldap3](https://docs.rs/ldap3/0.12.1/ldap3/).
