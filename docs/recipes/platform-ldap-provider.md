# Platform LDAP-provider recipe

riAuth serves a read-only LDAPv3 directory from a Platform process. Applications bind and search riAuth. This page is the listener the LDAPS and STARTTLS test builds, the searches and denials that test asserts, and the administrator commands that create the same records. It is not a record that the test ran on this revision.

The client in that test is the `ldap3` 0.12.1 crate (`sync`, `tls-rustls-aws-lc-rs`) linked into [tests/identity/network.rs](../../tests/identity/network.rs). The server codec is `ldap3_proto` 0.8.1. Neither crate is Active Directory or OpenLDAP. [LDAP import](../ldap.md) is the opposite direction: riAuth is the client, and `scripts/test-ldap.sh` starts `slapd`. This recipe does not configure `[directories.*]` and does not import accounts.

Behavior and the operator example live in [ldap-provider.md](../ldap-provider.md). Routes and the listener exist only in a Platform build. An Essentials configuration with `ldap_listeners` fails validation with `ldap_listeners requires the Platform build` ([src/edition.rs](../../src/edition.rs)).

## Three different claims

| Claim | What it means here |
| --- | --- |
| Check-job script | [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) runs `cargo test --all-targets --features test-support,fuzzing`. This test is not `#[ignore]`, so that command includes it. The workflow text is not a result for this revision. |
| This revision | `ldap_provider_tls_scoped_search_paging_rebind_mfa_and_revocation` was not executed while this page was written. No Cargo command was run. |
| Deployment peer | The test dials loopback with a certificate it just generated. It does not speak to Active Directory, OpenLDAP, `ldapsearch`, or an application directory client. |

## Records the fixture creates

The test calls `Core` methods in process. It does not run `riauth client create` or `riauth agent create`.

| Record | Fixture value |
| --- | --- |
| Users | `ldap-alice` and `ldap-bob`. Alice enrolls TOTP through `mfa_begin` and `mfa_confirm` before any LDAP call. |
| Group | `directory`, with both users as durable members. |
| Client | `client_id` `ldap`, name `LDAP provider`, `confidential` false, `service` false, no redirect URIs, scopes `openid`, `profile`, `email`, `groups`, `allowed_groups` `directory`, `require_mfa` false. |
| LDAP settings | `base_dn` `dc=riauth,dc=test`, `search_groups` `directory`. |
| Agent | id `ldap-reader`, ttl 3600 seconds, no parent, one permission `ldap.search` on `client/ldap`. The simple-bind password is that credential's `token`. |

The example in [ldap-provider.md](../ldap-provider.md) is a different client: base `dc=example,dc=test`, group `staff`, and `require_mfa` true. Use that shape for an operator install. The result codes below are what the loopback fixture asserts.

`settings.ldap` requires a `dc=` base of at most 253 characters, one to 32 search groups, the `profile` scope, and an interactive client (`service` false) ([src/ldap_server.rs](../../src/ldap_server.rs) `Settings::validate`). Omitting `--scope` on `riauth client create` selects `openid`, `profile`, `email`, and `offline_access`, which does not match this fixture. Pass the four scopes explicitly.

## Listener configuration

Operator form, copied from [ldap-provider.md](../ldap-provider.md). `192.0.2.20` is a placeholder for the application server. The ldap3 test does not bind this address or use this client id:

```toml
[ldap_listeners.legacy]
listen = "0.0.0.0:1636"
client_id = "legacy-directory"
allowed_peers = ["192.0.2.20"]
ldaps = true
tls_cert_file = "ldap-fullchain.pem"
tls_key_file = "secrets/ldap-key.pem"
```

With `ldaps = false` the same fields are a STARTTLS listener: cleartext LDAP until STARTTLS, and binds or directory searches before that return confidentiality required. Certificate paths are relative to the config file and reload every 60 seconds; a failed reload keeps the active certificate. `allowed_peers` is one to 128 explicit addresses, never an unspecified or multicast address ([src/ldap_listener.rs](../../src/ldap_listener.rs)). At most 16 listeners ([src/config.rs](../../src/config.rs)). `local_unencrypted = true` is only a loopback listener with no certificate files. The fixture leaves it false.

The test inserts these two listeners and generates one certificate for both. `listen` port `0` asks the kernel for a port; ldap3 then dials the bound address. `LdapConn` order follows `BTreeMap` key order, so address 0 is `ldaps` and address 1 is `starttls`.

```toml
[ldap_listeners.ldaps]
listen = "127.0.0.1:0"
client_id = "ldap"
allowed_peers = ["127.0.0.1"]
ldaps = true
tls_cert_file = "{temp}/ldap.crt"
tls_key_file = "{temp}/ldap.key"

[ldap_listeners.starttls]
listen = "127.0.0.1:0"
client_id = "ldap"
allowed_peers = ["127.0.0.1"]
ldaps = false
tls_cert_file = "{temp}/ldap.crt"
tls_key_file = "{temp}/ldap.key"
```

The certificate is a one-day self-signed P-256 certificate, subject `CN=localhost`, with SAN DNS `localhost` and IP `127.0.0.1`. The ldap3 connection trusts that certificate and no other. It is not an enterprise CA.

A peer outside `allowed_peers` is dropped before an LDAP message ([src/ldap_server.rs](../../src/ldap_server.rs) `listen`). The ldap3 test connects only from `127.0.0.1` and does not assert that drop.

## Permissions

| Action | Who | Fixture |
| --- | --- | --- |
| Create the policy client | Administrator session. CLI client writes also require `--idempotency-key` and `--if-revision`. The mutation checks `client.write` on `client/ldap`. | `Core::create_client` as the initial administrator. |
| Create, rotate, or revoke the service agent | A live human administrator (`Core::admin`). Not a delegated agent. | `create_agent` and, later, `revoke_agent` for `ldap-reader`. |
| Service bind and every later search | The agent token must start with `ri_agent_` and must be allowed `ldap.search` on `client/{client_id}`. | The token from `create_agent`. |
| User bind | The user's password, plus the client policy (`allowed_groups`, and `require_mfa` when that flag is set). | Alice's password. `require_mfa` is false, and Alice still has TOTP enrolled. |
| Listener file | Whoever edits `riauth.toml` and runs `riauth serve`. Not an HTTP permission. | `ldap_server::start` inside the test. |

Group and user rows are ordinary riAuth records. Directory visibility uses durable membership. A temporary grant is not a `search_groups` membership ([ldap-provider.md](../ldap-provider.md)). The fixture grants durable membership only.

## Searches and paging the ldap3 test asserts

Every code below is an assertion in `ldap_provider_tls_scoped_search_paging_rebind_mfa_and_revocation`. Writing this page did not produce it. LDAPS is used for one successful service bind and `unbind`. The search, paging, MFA, and revocation rows run on the STARTTLS connection after `LdapConnSettings::set_starttls(true)`.

Base DN `dc=riauth,dc=test`. Service DN `cn=riauth-agent,dc=riauth,dc=test`. Alice's DN `uid=ldap-alice,ou=users,dc=riauth,dc=test`.

| Step | Result code |
| --- | --- |
| Simple bind of the service DN on the cleartext socket, before STARTTLS | 13 `confidentialityRequired` |
| Who Am I on that cleartext socket | Success. `authzid` is empty. |
| Base-scope search of `""` with `(objectClass=*)`, attributes `namingContexts` and `supportedExtension` | Success. `namingContexts` is `dc=riauth,dc=test`. The test does not assert `supportedExtension`. |
| LDAPS simple bind of the service DN with the agent token | Success, then `unbind`. |
| Subtree search `(uid=*)` on the STARTTLS connection before a successful bind | 50 `insufficientAccessRights` |
| Simple bind of the service DN with the agent token | Success |
| Who Am I | `authzid` is `dn:cn=riauth-agent,dc=riauth,dc=test` |
| Subtree search `(&(objectClass=inetOrgPerson)(uid=ldap-*))` with RFC 2696 control `1.2.840.113556.1.4.319`, page size 1, empty cookie, attributes `uid`, `mail`, `memberOf`, `userPassword`, `entryUUID` | One entry. `userPassword` is absent. `entryUUID` is present. The paging cookie is not empty. |
| Same cookie with a different filter, `(uid=*)` | 53 `unwillingToPerform` |
| Streaming search `(uid=*)`, page size 1, subtree | Two entries, then a success result. |
| Simple bind of the service DN with `wrong-service-token` | 49 `invalidCredentials` |
| Subtree search `(uid=*)` after that failed bind | 50 |
| Simple bind of Alice with the password and no one-time code | 49 |
| Simple bind of Alice with `password;` plus the current TOTP | Success |
| Who Am I | `authzid` is `dn:uid=ldap-alice,ou=users,dc=riauth,dc=test` |
| Subtree search `(uid=*)` as Alice | One entry, `uid` `ldap-alice` |
| Simple bind of Alice with the same TOTP again | 49 |
| Simple bind of the service DN with the original agent token | Success |
| `revoke_agent` for `ldap-reader`, then subtree search `(uid=*)` | 50 |
| Delete `uid=ldap-bob,ou=users,dc=riauth,dc=test` | 53 |

The delete assertion is the only write the ldap3 test sends. Add, modify, modify DN, and compare are not in this test. The listener returns 53 `unwillingToPerform` for all five ([src/ldap_server.rs](../../src/ldap_server.rs)).

Source accepts equality, presence, substring, and boolean `And`, `Or`, and `Not`. Other filter kinds fail validation with `This LDAP profile supports equality, presence, substring and boolean filters`, mapped from HTTP 400 to `inappropriateMatching`. The ldap3 test calls presence `(objectClass=*)` and `(uid=*)`, and one `And` of an equality and a substring. It does not call `Or`, `Not`, or an unsupported filter. Scopes in the test are base (root DSE) and subtree. One-level and children are implemented and are not called here.

A paging cookie is stored for one search fingerprint, the configuration revision, and that connection's authentication, and it expires after five minutes. Size must be 0 through 500 or the listener returns `adminLimitExceeded`. The ldap3 test asserts the transplanted-cookie failure and the two-entry stream. It does not wait five minutes, change the revision, or send a size outside 0 through 500.

## Administrative counterparts

There is no LDAP-provider subcommand. Listener changes are `riauth.toml` plus `riauth serve`.

| Fixture step | Supported equivalent |
| --- | --- |
| `Core::create_client` | `POST /api/clients`, or `riauth client create ldap --name 'LDAP provider' --scope openid,profile,email,groups --group directory --settings-file FILE`. The settings file for this fixture is `{"ldap":{"base_dn":"dc=riauth,dc=test","search_groups":["directory"]}}`. Client writes require `--idempotency-key` and `--if-revision` from `riauth revision`. |
| `create_group` / `group_member` | `POST /api/groups` and `PUT /api/groups/directory/members/{username}`, or `riauth group create directory` and `riauth group add-member directory USERNAME`, with the same revision flags. |
| `f.user` | `POST /api/users`, or `riauth user create USERNAME`. The fixture password is the shared test constant in `tests/common`, not an operator password. |
| `mfa_begin` / `mfa_confirm` | `POST /api/mfa/enroll` and `POST /api/mfa/confirm`, or `riauth mfa enroll` and `riauth mfa confirm`, on that user's session. The test does not run those commands. |
| `create_agent` | `POST /api/agents` with `{"id":"ldap-reader","ttl":3600,"permissions":[{"action":"ldap.search","resource":"client/ldap"}]}`, or `riauth agent create ldap-reader --permission ldap.search=client/ldap --ttl 3600 --out FILE`. The file receives `credential.token`. An administrator session creates it. |
| `revoke_agent` | `DELETE /api/agents/ldap-reader`, or `riauth agent revoke ldap-reader`. |
| ldap3 simple bind | The application's own LDAP client. riAuth does not ship a bind command for this listener. |

`GET /api/capabilities` reports `directory.ldap_provider` configured when `ldap_listeners` is non-empty. That flag does not mean a peer connected.

## What this fixture does not prove

The OpenLDAP harness in [ldap.md](../ldap.md) imports into riAuth. Passing it does not exercise this listener, and passing this listener does not exercise `slapd`.

In-process tests in [src/assembly/ldap_server.rs](../../src/assembly/ldap_server.rs) cover index paging, DN collisions, and the 2,000-user selection cap. They do not open a socket and were not run for this page. The ldap3 test's directory has two users.

Documented caps that this ldap3 test does not fill: 2,000 selected users, 4 MiB of search output, 128 connections per listener, eight per allowed peer, BER requests of 32 KiB, and 600 operations per minute per peer for category `ldap:{client_id}`. A full rate bucket ends the connection loop rather than returning a result code. Idle reads use a 60-second timeout. Filter depth is capped at 12 and filter nodes at 128.

POSIX and Active Directory schema are not advertised. `sn` is the display name. `entryUUID` is the local user id. The test checks that the attribute is present and does not check its value. No third-party application was pointed at the listener.

## Other D03 recipes

Still without a recipe page: SAML source, inbound SCIM, outbound SCIM, RADIUS, Workspace, Entra, Shared Signals, device trust, HTTPS client certificates, Vault Transit, Windows device login, the embedded reverse proxy, and shared-domain SSO. The [OIDC relying party](oidc-relying-party.md), the [SAML IdP](platform-saml-idp.md), the [LDAP import](ldap-import.md), and the [upstream OIDC recipe](upstream-oidc.md) are separate. The upstream issuer is the in-process loopback token endpoint, and Okta, Entra, and Google are not connected. The import page is the client of disposable loopback OpenLDAP, and a real Active Directory directory remains an open peer. That relying-party client is the in-tree axum fixture, not a named application. The SAML IdP signer is xmlsec1, and a named service provider remains an open peer. D04 emergency runbooks and D05 acceptance are separate work. The [capability matrix](../capability-matrix.md) records the protocol limits those recipes still have to cite.
