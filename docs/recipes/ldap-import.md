# LDAP import recipe

This page is the directory client inside `openldap_plans_stable_ids_tls_login_mfa_and_fail_closed_sync` in [tests/ldap.rs](../../tests/ldap.rs). That test is `#[ignore = "requires the disposable OpenLDAP harness"]`. The file header says to run it with [scripts/test-ldap.sh](../../scripts/test-ldap.sh) against that script's disposable OpenLDAP server. The directory id is `staff`. riAuth searches that server and checks passwords with a simple bind. Local accounts are created or disabled in riAuth. The test was not executed while this page was written.

`directory.ldap_sync` and `identity.ldap_authentication` are in `FEATURES` and are absent from `PLATFORM_FEATURES` ([src/agent.rs](../../src/agent.rs)). `compiled_for` includes such a name on Essentials ([src/capability.rs](../../src/capability.rs)). `config_blockers` rejects `ldap_listeners`, `workspace_directories`, and `entra_directories` on Essentials. It does not reject `[directories]` ([src/edition.rs](../../src/edition.rs)). The client crate is `ldap3` 0.12.1 with `sync` and `tls-rustls-aws-lc-rs`, and that dependency is not optional ([Cargo.toml](../../Cargo.toml)). `ldap3_proto` 0.8.1 is the listener codec and is selected only with the `platform` feature. Cargo's default features select `platform`. [scripts/test-ldap.sh](../../scripts/test-ldap.sh) calls `cargo test` without `--no-default-features`, so the integration command builds that default. This page does not claim an Essentials run.

The [LDAP provider recipe](platform-ldap-provider.md) is the other direction: riAuth is the LDAPv3 server, and the client there is `ldap3` pointed at riAuth. This page does not configure `[ldap_listeners]`. The operator profile, including the `memberOf` example, stays in [ldap.md](../ldap.md).

## Three different claims

| Claim | What it means here |
| --- | --- |
| Integration-job script | [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) step "OpenLDAP synchronization and authentication" runs `scripts/test-ldap.sh` on `ubuntu-24.04` with Rust 1.98.1. The same job's "Install integration dependencies" step installs distro `slapd` and `ldap-utils` on the `postgresql` `nginx` `xmlsec1` line, with no version pin. The script then runs `cargo test --locked --test ldap -- --ignored --nocapture`. The workflow text is not a result for this revision. |
| This revision | No Cargo command was run while this page was written. The ignored test was not started. `scripts/test-ldap.sh` was not started. |
| Deployment peer | The directory process is the script's private loopback `slapd`, loaded with core, cosine, and inetOrgPerson. No Active Directory server is connected. No customer directory is connected. The LDAP provider listener is a different program. |

The check job runs `cargo test --all-targets --features test-support,fuzzing --locked` and does not pass `--ignored`, so this test is outside that command. This page did not run the check job.

[ldap.md](../ldap.md) describes the same script as the Linux and macOS operator entry. The workflow takes the script's Linux branch. The Darwin branch is script text on this page.

## Records the fixture creates

The test calls `LdapConn::new`, `Core::initialize`, `Core::login`, `Core::create_group`, and `Core::create_agent` in process. It does not run `riauth directory` or `riauth login`. The population client is cleartext `ldap://127.0.0.1:{port}`. riAuth's own directory client is separate and uses STARTTLS.

| Record | Fixture value |
| --- | --- |
| URL guard | `RIAUTH_TEST_LDAP_URL` must start with `ldap://127.0.0.1:`. |
| Service bind | DN `cn=fixture,dc=riauth,dc=test`. The password literal is `fixture-directory-service-password`. It exists only inside this harness. |
| Suffix and people | The test adds `dc=riauth,dc=test` (`top`, `domain`) and `ou=people,dc=riauth,dc=test` (`top`, `organizationalUnit`). |
| Alice | DN `uid=alice,ou=people,dc=riauth,dc=test`, object classes `top` and `inetOrgPerson`, `uid` `alice`, `cn` `Alice LDAP`, `sn` `LDAP`, `mail` `alice@example.test`, `description` `staff`, `userPassword` `fixture-user-password`. That user password exists only inside this harness. |
| Administrator | `NewUser` username `admin`, display name `Admin`, email absent, `admin` true. The password literal is `fixture-admin-password`. It is a local riAuth password for the temporary store. It is not written to `slapd`. |
| Group | Local group `staff`, created before the first plan. |
| Agent | id `syncer`, ttl 3600, parent absent. Permissions: `directory.read` and `directory.sync` on `directory/staff`, `user.write` on `*`, `group.members` on `group/staff`. The test keeps `credential.token`. |
| Store | `Config` with `data_dir` at `{temp}/db` and one `directories.staff` entry. The rest is `Default`. The test does not write a `riauth.toml`. |

The three password literals above are fixture strings in the test and the script. They are not operator secrets to copy into a deployment.

## OpenLDAP harness

[scripts/test-ldap.sh](../../scripts/test-ldap.sh) says it is a private loopback-only fixture and that it does not read or modify a system directory. It creates a `mktemp` directory, picks a free `127.0.0.1` port, and deletes that directory on exit.

On Linux it copies `${SLAPD:-/usr/sbin/slapd}` into the temp directory. The script comment says AppArmor confines `/usr/sbin/slapd` to system paths. Schema is `/etc/ldap/schema`. The config contains `modulepath /usr/lib/ldap` and `moduleload back_mdb`. On Darwin it uses `${LDAP_PREFIX:-$(brew --prefix openldap)}/libexec/slapd` and `${LDAP_SCHEMA:-$(brew --prefix)/etc/openldap/schema}`, with an empty module block. Both branches set backend `mdb`.

The script mints a one-day RSA 2048 CA, CN `localhost`, `CA:TRUE`, SAN `DNS:localhost` and `IP:127.0.0.1`, key usage `keyCertSign,cRLSign`. The leaf is RSA 2048, CN `localhost`, the same SAN, `CA:FALSE`, key usage `digitalSignature,keyEncipherment`, extended key usage `serverAuth`. `slapd.conf` includes `core.schema`, `cosine.schema`, and `inetorgperson.schema`, suffix `dc=riauth,dc=test`, root DN `cn=fixture,dc=riauth,dc=test`, and `rootpw` from `/usr/sbin/slappasswd -s fixture-directory-service-password`. The ACL is `access to attrs=userPassword by self write by anonymous auth by * none` and `access to * by * read`. `slapd.conf` and `tls.key` are mode `600`. `slapd` is started with `-f slapd.conf -h ldap://127.0.0.1:{port} -d 256`. Readiness is up to 50 tries, 0.1 seconds apart, of `ldapsearch -x -H {url} -s base -b '' '(objectClass=*)'`.

The script then writes the same service password to `{temp}/password` and sets that file to mode `600`. It exports `RIAUTH_TEST_LDAP_URL`, `RIAUTH_TEST_LDAP_PASSWORD` (the file path), and `RIAUTH_TEST_LDAP_CA` (the CA path). `${CARGO:-cargo}` is the binary.

The package version is whatever the runner installs. This page did not observe a `slapd` version.

## Directory configuration the test builds

[ldap.md](../ldap.md) shows an operator directory at `ldap://directory.example.test:389` with bind DN `cn=riauth,ou=services,dc=example,dc=test` and group filter `staff = "(memberOf=cn=staff,ou=groups,dc=example,dc=test)"`. The same guide names `objectGUID` and `sAMAccountName` as Active Directory mappings. The ignored test builds a different in-memory `Directory`:

| Field | Fixture value |
| --- | --- |
| `url` | `RIAUTH_TEST_LDAP_URL` |
| `transport` | `Transport::Starttls` |
| `bind_dn` | `cn=fixture,dc=riauth,dc=test` |
| `password_file` | `RIAUTH_TEST_LDAP_PASSWORD` (path) |
| `ca_file` | `RIAUTH_TEST_LDAP_CA` |
| `user_base` | `ou=people,dc=riauth,dc=test` |
| `user_filter` | `(objectClass=inetOrgPerson)` |
| `id_attribute` | `entryUUID` |
| `username_attribute` | `uid` |
| `display_attribute` | `cn` |
| `email_attribute` | `mail` |
| `username_prefix` | empty |
| `group_user_filters` | `staff` = `(description=staff)` |

`Transport::Starttls` makes `connection` call `set_starttls` ([src/directory.rs](../../src/directory.rs)). The population client's `LdapConn::new` does not. Removing `ca_file` makes `directory_plan` return code `directory_unavailable`. The test comment says an untrusted certificate must never downgrade to plaintext. The test restores `ca_file` after that call. It does not assert a handshake string.

`Directory::validate` requires `ldap://` for STARTTLS, `ldaps://` for LDAPS, and `ldap://` plus a literal loopback address for `Transport::Loopback`. It rejects a URL user, password, query, fragment, or non-empty path. Bind DN and user base must be nonempty, at most 2048 characters, and free of controls. Filters must be parenthesized, at most 4096 characters, and free of controls. At most 32 group filters are accepted. Attribute names are at most 64 ASCII letters, digits, or `-`. The test's only `validate` call is the loopback case below. These other checks are the source.

## Assertions the ignored test makes

| Step | Result the test requires |
| --- | --- |
| `directory_plan` as `syncer` | `changes` equals exactly `[{"username":"alice","action":"create","groups":["staff"]}]`. The plan JSON string does not contain `fixture-user-password`. |
| `directory_apply` of that id as `admin` | `is_err`. The test does not match a code. |
| `directory_apply` as `syncer` | Succeeds. A second apply of the same id returns `applied` true. |
| `login("alice", fixture-user-password, None)` | Succeeds. `email_verified` is false. `me` groups equal `["staff"]`. |
| `login` with an empty password, and with `bad-password` | Each error code is `invalid_credentials`. |
| `mfa_begin`, TOTP from the otpauth URL, `mfa_confirm` with the code at `now() - 30` | Login without a one-time code is `is_err`. The test does not match that code. Login with the current TOTP succeeds, and `me.mfa` is true. |
| `recovery_codes` | The first code succeeds once as the one-time code. The second use of that code is `is_err`. |
| Plan, then replace `cn` with `Changed Name`, then apply that plan id | Error code `conflict`. |
| `modifydn` to `uid=renamed` (delete the old RDN, no new superior), then plan and apply | The stored user id from the first login is unchanged. `username` is `renamed`. `totp_secret` is present. The previous session's `me` is `is_err`. |
| `modifydn` to `uid=admin` | `directory_plan` returns code `conflict`. The test then renames the entry back to `uid=renamed`. |
| `user_base` set to `ou=missing,dc=riauth,dc=test` | `directory_plan` is `is_err`. The stored user stays `enabled`. The base is restored. The test does not match a code. |
| Plan, `revoke_agent` of `syncer`, apply with the agent token | `is_err`. The test does not match a code. |
| Delete the renamed entry. Plan as `admin` | `changes[0].action` is `disable`. `directory_apply` as `admin` is `is_err`. `directory_apply_confirmed` with the plan id in the confirmation argument succeeds. The user `enabled` flag is false. Group `staff` members is empty. |
| Add `paged000` through `paged204` | Each entry is `inetOrgPerson` with `cn` `Paged User` and `sn` `User`, and the add omits `mail`, `description`, and `userPassword`. The admin plan's `changes` length is 205. Admin apply succeeds. |
| Configured group filters | Keys remain exactly `{"staff"}`. |
| Clone with URL `ldap://example.test` and `Transport::Loopback` | `validate()` is `is_err`. The test does not match the message. The live `staff` directory is left on STARTTLS. |
| End | `ldap.unbind()`. |

The test does not assert an HTTP status, a CLI flag, an idempotency header, `If-Match`, a page cookie, control OID `1.2.840.113556.1.4.319`, the 2,000-user cap, the 4 MiB snapshot cap, a `password_hash` field, an Active Directory attribute, or a `slapd` version.

## Credential handling

`service` reads `password_file` with `read_private_secret` at a 4,096-byte limit ([src/directory.rs](../../src/directory.rs), [src/config.rs](../../src/config.rs)). On Unix that function rejects a mode with any group or other bits, with the message `Credential file must have owner-only permissions (0600 or 0400)`. The script's password file, `slapd.conf`, and `tls.key` are mode `600`. `service` trims a trailing CR or LF and rejects an empty password or one longer than 4,096 characters (`LDAP bind password must be nonempty and bounded`). It then `simple_bind`s as `bind_dn`. A failed bind becomes `unavailable()`.

`GET /api/directories` returns `id`, `url`, `user_base`, the group-filter keys, and `reconciliation_mode` ([src/assembly/directory.rs](../../src/assembly/directory.rs)). That JSON has no password field. `riauth directory list` is that GET. The test does not call it.

On create, `reconcile` stores `password_hash: String::new()`, `admin: false`, and `email_verified: false`. The test asserts `email_verified` false and asserts that the plan text omits `fixture-user-password`. It does not read `password_hash`. An owned user who is an administrator, or whose `password_hash` is nonempty, makes `reconcile` return conflict `LDAP may only manage non-administrator directory accounts`. The test does not take that branch.

`directory_login` is the password path for an imported account. Eligibility requires the user to be enabled, not an administrator, a matching identity fingerprint, and a nonempty password of at most 4,096 characters. The function then binds as the service account, searches the stored DN at base scope, and requires one non-referral row whose stable id matches the binding. A fresh connection `simple_bind`s the submitted password: result code 0 is success, 49 is failure, and any other code is `directory_unavailable`. An ineligible account or a failed bind becomes code `invalid_credentials` and message `Invalid username, password, or one-time code`. The attempt counter resets when `start + 900 <= now`. When `count` is already 5, the next attempt returns code `rate_limited` and message `Too many login attempts`. The test covers the empty password, `bad-password`, a missing one-time code after enrollment, a current TOTP, and one recovery-code replay. It does not assert `rate_limited` or result code 49.

`LOGIN_BUDGET` is 800 milliseconds for the whole login exchange. `STEP_TIMEOUT` is 5 seconds for a sync step. Search options set a 5-second time limit and a size limit of 2,001. The test does not time these.

A rename that changes the account bumps `epoch` in `reconcile`. The test's evidence for that session effect is `me` returning `is_err` after the rename. It does not assert the epoch number. [ldap.md](../ldap.md) says normal login, request approval, and OIDC step-up send imported accounts to the directory, and that directory outages do not fall back to a local password. The test does not call request approval or an OIDC step-up.

## Partial results, referrals, and removals

`search_page` sends a critical paged-results control of size `LDAP_PAGE_SIZE` (200) and requires exactly one control with OID `1.2.840.113556.1.4.319`. A referral row (`entry.is_ref()`), a nonempty `result.refs`, elapsed time over 30 seconds, a page that already holds 200 rows, a missing or duplicate control, a control value over 4,096 bytes, or a cookie over 2,048 bytes returns `unavailable()`. That error is HTTP 503, code `directory_unavailable`, message `LDAP operation failed or did not return a complete result; verify bind credentials and paged-results support, then retry the complete snapshot`. The ignored test does not send a referral. The missing-base call is the test's fail-closed search: `directory_plan` is `is_err` and the stored user stays enabled. The test does not match `directory_unavailable` on that call. The source maps an incomplete search to that code.

Mapped attributes must be single-valued, nonempty, at most 2,048 bytes, and UTF-8 when read as text. The test does not send a multivalued mapped attribute.

Default `LdapReconciliationQuota` is `pages_per_call` 4, `max_pages_per_search` 20, `max_users` 2,000, `max_snapshot_bytes` 4,194,304, and `draft_ttl_seconds` 300 ([src/config.rs](../../src/config.rs)). `validate` rejects values outside `pages_per_call` 1..=4, `max_pages_per_search` 1..=20, `max_users` 1..=2000, `max_snapshot_bytes` 65536..=4194304, and `draft_ttl_seconds` 30..=300. A finished plan sets `expires_at` to `now() + 300`. The 205-entry assertion is a `changes` length. `LDAP_PAGE_SIZE` is 200, so that length is larger than one source page. The test does not read a cookie, the page size, or the control OID. It is not a measurement of the 2,000-user cap or the 4 MiB cap.

`directory_apply` calls `directory_apply_confirmed` with confirmation `None` ([src/assembly/directory.rs](../../src/assembly/directory.rs)). Before a plan is applied, apply re-crawls. An unfinished crawl returns progress and does not change accounts. When the re-crawl users differ from `plan.entries`, the source returns conflict `LDAP changed after planning; create a new plan`. The stale `cn` step asserts code `conflict` and does not assert that sentence. When the recomputed changes differ, the source returns conflict `LDAP plan no longer matches local state`. The test does not take that branch. `directory_apply_confirmed` loads the plan through `directory_plan_get`. That load returns `Error::forbidden()` when `actor.id` differs from `plan.actor`. `Error::forbidden()` is code `access_denied` and message `Access denied`. The admin apply of the agent's plan is the test's `is_err` on that boundary. It does not match a code. `directory_snapshot_actor` returns conflict `LDAP source, authority or local revision changed during snapshot` when the actor, local revision, fingerprint, or authority digest differs. The mutation also returns `Error::forbidden()` when `actor.id` differs from `plan.actor`. The test does not assert those sentences. When `plan.applied` is already true, a later apply by the same actor returns `id`, `applied: true`, and `changes` without another crawl. The test asserts `applied` true on that second call.

`modifydn` to `uid=admin` makes `directory_plan` return `conflict`. The owned row is still the directory user. The local name `admin` belongs to a different user id, so the source branch is conflict `LDAP username collides with an existing account; accounts are never automatically linked`. The test asserts the code only.

`ReviewBinding::confirm` returns conflict `Connector removals require explicit review; confirm this exact plan ID after inspecting removal_impact and changes` when `review_required`, `missing_users > 0`, or `removed_memberships > 0`, unless the reviewed value is that plan id ([src/connector_guard.rs](../../src/connector_guard.rs)). The delete step is the test's pair: unconfirmed `directory_apply` is `is_err`, and `directory_apply_confirmed` with `Some(plan id)` succeeds. After that success the user is disabled and group `staff` has no members. The test does not assert the conflict sentence.

`membership` changes only configured mapped groups, and it requires each of those groups to exist locally (`LDAP mappings require an existing local group`). The test creates `staff` first and, after the disable, observes that group empty. It does not add a second local group.

Disabling an owned administrator returns conflict `LDAP cannot disable an administrator`. The test's administrator is a separate local account. The collision above is the path the rename-to-`admin` plan takes.

A non-loopback host with `Transport::Loopback` fails `validate` with `LDAP requires LDAPS, mandatory STARTTLS, or explicit literal loopback transport; put DNs in configuration fields`. The test asserts `is_err` and does not assert that sentence.

An agent token builds a principal with `agent: true`, `delegated: false`, and id `agent:` plus the agent id ([src/agent.rs](../../src/agent.rs)). `allows` then matches that permission list. Plan and apply call `management` with `directory.sync` on `directory/{id}`. `directory_plan_get` calls `management` with `directory.read` on the same resource. `require_directory_user` on this principal calls `require("user.write", "user/{username}")`. `require_directory_group` calls `require("group.members", "group/{name}")`. The test grants `directory.read` and `directory.sync` on `directory/staff`, `user.write` on `*`, and `group.members` on `group/staff`. A human principal with `delegated: true` uses `directory.sync` for those user and group checks, and `require_directory_user` returns `Error::forbidden()` when the target user id is that principal's id. An administrator principal is neither an agent nor delegated, and `allows` returns true. The test's create apply uses the agent token. The disable apply uses the admin token.

## Closest API and CLI

The ignored test does not open these routes. They are the HTTP counterparts in [src/api.rs](../../src/api.rs):

| Operation | Route |
| --- | --- |
| List | `GET /api/directories` |
| Plan | `POST /api/directories/{id}/plan` |
| Read a plan | `GET /api/directory-plans/{id}` |
| Apply | `POST /api/directory-plans/{id}/apply` |

[docs/api.md](../api.md) describes the plan route as an LDAP import plan with no account writes. Workspace and Entra use `/api/workspace-directories` and `/api/entra-directories` and are Platform routes. [ldap.md](../ldap.md) says those imports do not enable LDAP password authentication.

The CLI counterparts in [src/cli.rs](../../src/cli.rs), also not executed by the test:

```sh
riauth directory list
riauth directory plan staff --out deployment-private/ldap-plan.json
riauth --if-revision "$(jq -r .revision deployment-private/ldap-plan.json)" directory apply --plan deployment-private/ldap-plan.json
riauth directory apply --plan deployment-private/ldap-plan.json --confirm-removals
```

`directory plan` POSTs up to 1,024 times while `decision` is `snapshot_in_progress`, then writes the plan with `write_private`. If the body has no `id`, it returns `LDAP snapshot did not complete within the request quota; retry directory plan to resume`. `directory apply` reads the file, GETs the stored plan, copies the file's `applied` field onto that copy, and rejects a mismatch with `LDAP plan was modified or belongs to another instance`. When `removal_impact.review_required` is true and `--confirm-removals` is absent, it returns `Inspect LDAP removal_impact and changes, then rerun with --confirm-removals`. The POST repeats up to 1,024 times while `decision` is `snapshot_in_progress`.

`--confirm-removals` sends both `x-riauth-confirm-removals` and `x-riauth-confirm-cloud-removals`, each set to the plan id ([src/cli/transport.rs](../../src/cli/transport.rs)). `removal_confirmation` accepts either header. It requires one distinct nonempty value, at most 128 ASCII graphic characters. The in-process confirmation argument is that same plan id.

[ldap.md](../ldap.md) includes `--if-revision` on the operator apply command. When that flag is set, the transport sends `If-Match` as one quoted revision. The protect middleware parses `If-Match` when the header is present: one quoted numeric revision on non-SCIM paths. The directory apply handler does not require the header. The ignored test does not send `If-Match` or an idempotency key. When an idempotency key is present, the middleware folds a removal-confirmation header and `If-Match` into the request fingerprint. The test does not exercise that path.

## What this fixture leaves open

The harness directory is disposable loopback OpenLDAP with core, cosine, and inetOrgPerson. No Active Directory server is in the test or the script. The names Keycloak, Okta, and Active Directory in the [capability matrix](../capability-matrix.md) peer table are migration-inventory fixtures. Distro `slapd` and Homebrew OpenLDAP are unpinned. The Darwin branch and the Linux copy of `slapd` are different script paths. This page did not start either one.

The ignored test does not fill the quota caps, does not send a referral, and does not continue an in-progress snapshot. Those behaviors are the source above. A named directory deployment remains an open peer. The [LDAP provider recipe](platform-ldap-provider.md) remains the listener fixture.

## Other D03 recipes

Still without a recipe page: SAML source, outbound SCIM, RADIUS, Workspace, Entra, Shared Signals, device trust, HTTPS client certificates, Vault Transit, Windows device login, the embedded reverse proxy, and shared-domain SSO. The [forward-auth](platform-forward-auth.md), [LDAP provider](platform-ldap-provider.md), [OIDC relying party](oidc-relying-party.md), [SAML IdP](platform-saml-idp.md), the [upstream OIDC recipe](upstream-oidc.md), and the [inbound SCIM recipe](platform-inbound-scim.md) are separate. The inbound SCIM client is in-process `oneshot`, and no named SCIM client is connected. The upstream issuer is the in-process loopback token endpoint, and Okta, Entra, and Google are not connected. A named external relying party is still an open peer. A named service provider is still an open peer. A real Active Directory directory is still an open peer. D04 emergency runbooks and D05 acceptance are separate work. The [capability matrix](../capability-matrix.md) records the protocol limits those recipes still have to cite.
