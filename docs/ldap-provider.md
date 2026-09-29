# LDAP provider

riAuth can serve a read-only LDAPv3 directory alongside its HTTP/OIDC listener in the same Rust process. It supports simple binds, LDAPS, mandatory STARTTLS, root DSE discovery, Who Am I, equality/presence/substring/boolean search filters, and bounded RFC 2696 paging. User and group administration stays in the CLI and agent API.

Create a policy client with `settings.ldap` through `client create --file` or a manifest:

```json
{
  "client_id": "legacy-directory",
  "name": "Legacy application directory",
  "confidential": false,
  "service": false,
  "redirect_uris": [],
  "scopes": ["openid", "profile", "email", "groups"],
  "allowed_groups": ["staff"],
  "require_mfa": true,
  "settings": {
    "ldap": {"base_dn": "dc=example,dc=test", "search_groups": ["staff"]}
  }
}
```

The local groups must exist. `allowed_groups`, the client policy, configured MFA and default assurance values control user binds. A client requiring device trust rejects password binds: LDAP creates a new session and does not implement the session-bound device challenge protocol, so trust from another browser or CLI session cannot be inherited. `search_groups` independently selects directory visibility for a scoped service credential. Email and group attributes require the corresponding scopes. A user bind can search only its own selected entry and its memberships. A service bind can search enabled users selected by durable membership in the configured groups. It cannot retrieve password hashes, factor secrets, recovery codes or arbitrary user attributes.

LDAP checks two small Group metadata records in the search snapshot: the indexed name and digest of its `name` and `members`, and a source-side digest refreshed by every Group import. A raw import that changes those fields without rebuilding its binding fails closed. Service searches page through a Group-to-member index; long stale IDs use compact scan markers and trigger a paged User scan. Ordinary search binding and member selection no longer load the encoded `Group.members` value. This assumes Group writes go through riAuth's transactional store; changes made directly to database records outside that store bypass the digest update. Index rebuilds and Group writes still process full Group records, one Group at a time during rebuild. RFC 2696 paging limits response pages, not those remaining source-record costs.

Configure a listener in `riauth.toml`:

```toml
[ldap_listeners.legacy]
listen = "0.0.0.0:1636"
client_id = "legacy-directory"
allowed_peers = ["192.0.2.20"]
ldaps = true
tls_cert_file = "ldap-fullchain.pem"
tls_key_file = "secrets/ldap-key.pem"
```

The allowed peer 192.0.2.20 is a documentation placeholder; use the application server's actual address.

Run the normal `riauth serve`. With `ldaps = false`, the port speaks LDAP and requires STARTTLS before binds or directory searches. Certificate files resolve relative to the config file and are reloaded every minute; failed reloads retain the active certificate. `local_unencrypted = true` is restricted to explicit loopback test listeners with no TLS files. Peer IPs are always explicit; LDAP does not trust HTTP forwarding headers. Listener startup succeeds before the corresponding client is provisioned; requests fail closed until that client exists and is enabled.

Service applications bind as `cn=riauth-agent,dc=example,dc=test`, using a dedicated agent token as the password. That token needs only `ldap.search` on `client/legacy-directory`. Normal agent expiry, rotation and revocation apply to every subsequent search. Users bind as `uid=alice,ou=users,dc=example,dc=test`, with their password. If TOTP is configured, append `;123456` or `;ri_recovery_...`; one-time proofs cannot be replayed. Passkey-only assurance cannot be satisfied by a simple password bind. Directory-backed users authenticate through the configured upstream LDAP source.

Entries use these DNs:

| Entry | DN / attributes |
| --- | --- |
| User | `uid=alice,ou=users,<base>`; inetOrgPerson, uid, cn, sn, displayName, optional mail/memberOf, operational entryUUID |
| Group | `cn=staff,ou=groups,<base>`; groupOfNames, cn, member containing only visible user DNs |
| Root DSE | Empty base, base scope; namingContexts, supportedLDAPVersion, supportedExtension, supportedControl |

`entryUUID` is the local immutable user ID, independent of the OIDC subject mode. `sn` currently uses the display name. Empty groups are omitted. DNs use the constrained `dc=...` base and riAuth's supported account/group names; collisions under ASCII case-insensitive DN comparison are rejected. Attribute filtering uses Unicode lowercase comparisons for this profile, rather than a complete LDAP schema matching-rule engine. Unsupported comparison, ordering, approximate, extensible-match, alias, write, SASL and password-modify operations return protocol errors. POSIX/AD schema emulation is not advertised.

Active [temporary access grants](enterprise/ENT-01.md) affect user-bind policy. Directory projections use durable membership throughout: user `memberOf`, group `member`, and `search_groups` selection exclude temporary grants before and after their expiry or revocation. A user with only a temporary grant to a search group can authenticate if policy allows but still has no searchable user entry. The user/group directory views remain reciprocal.

Each connection has at most one active paging cursor. Cursors are bound to the exact search, local configuration revision and connection authentication, expire after five minutes and are consumed by continuation. Changed queries, rebinds, stale revisions or reused cursors cannot continue a search. Restart paging after a configuration change. The provider limits selected users to 2,000, search output to 4 MiB, pages to 500 entries, BER requests to 32 KiB, filter depth/nodes, connections to 128 per listener/eight per allowed peer, and operations to 600/minute per peer/provider. Idle, handshake and write timeouts are enforced. Failed rebinds clear prior authentication. Normal connection closure revokes its transient user session.

The automated network test uses independent `ldap3` clients against actual LDAPS and STARTTLS listeners. It covers service scope, root discovery, user/factor authentication, paging, credential isolation, failed rebinds and live agent revocation. The OpenLDAP source test is separate; no third-party application's LDAP compatibility is assumed from these fixtures. The [LDAP-provider recipe](recipes/platform-ldap-provider.md) records the fixture listener, the statuses that ldap3 test asserts, and the difference between that crate, the OpenLDAP import, and a deployment directory client. The [LDAP import recipe](recipes/ldap-import.md) records the client fixture.

A recorded local run points OpenLDAP `ldapsearch` 2.7.1 at one loopback STARTTLS listener. The binary was `/opt/homebrew/opt/openldap/bin/ldapsearch`, banner `ldapsearch 2.7.1 (Sep  8 2026 21:55:18)`. The command was `ldapsearch -x -ZZ -H ldap://127.0.0.1:62030 -D cn=riauth-agent,dc=riauth,dc=test -y <token-file> -b dc=riauth,dc=test -s sub -E '!pr=1/noprompt' -l 8 -o nettimeout=8 -o ldif_wrap=no '(objectClass=inetOrgPerson)' uid`, with `LDAPTLS_REQCERT=hard` and the fixture CA in `LDAPTLS_CACERT`. The service bind and page size 1 returned `ldap-alice` and `ldap-bob` on separate pages and exited 0. A disabled member was omitted. A wrong token and the same token after agent revocation each exited 49 with `ldap_bind: Invalid credentials (49)`. The recipe records the certificate and the exact banner. Active Directory remains unverified.
