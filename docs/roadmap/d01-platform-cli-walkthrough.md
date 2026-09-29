# Platform CLI walkthrough

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D01
`a96a1977-3210-4284-8f7d-645793369301`.

This page records eight disposable loopback runs of the
[Platform guide](../platform-guide.md). The first used an Essentials-catalog
binary in place and stopped after one client registration. The second copied
a Platform-catalog server snapshot and continued through the server CLI group,
claim, and audit commands. The third copied that same server snapshot plus
separate `riauthctl` and `riauth-maintenance` snapshots and ran the printed
maintenance init and remote client commands. The fourth copied only that
server snapshot and ran the section 5 backup, restore, and recovery-status
entry points. The fifth copied only that server snapshot and opened section 4
in an isolated browser. The sixth copied only that server snapshot, captured
one invitation on loopback SMTP, and accepted the password in an isolated
browser. The seventh copied only that server snapshot and ran the section 11
configured password workflow through export. The eighth copied only that
server snapshot and ran section 9 `directory list`, `directory plan`, and
`directory apply` against one disposable loopback OpenLDAP listener. No run
used an external peer or `cargo install`. The first two runs did not launch
`riauthctl` or `riauth-maintenance`. The fourth, fifth, sixth, seventh, and
eighth runs did not launch them either. The fifth and sixth runs opened a
browser. The fifth stored no passkey. The sixth did not start a passkey
ceremony. The seventh and eighth did not open a browser.

## Essentials catalog run

This run is one disposable loopback pass of the contiguous guide steps that
can complete without a browser or an external peer: artifact capabilities,
initial instance setup, operator login, and one client registration. The
source tree at the start of the run was
`12165852700b23f54943d094f61c180819523a52`. That commit changes documentation
only, relative to its parent `3e895e932d24c3d4160b9f7f21c2132b96a29a2f`. No
Cargo build was run. `CARGO_TARGET_DIR` was unset.

## Binary

The binary was the already built file
`/Users/dominik/orca/projects/riAuth-public-preview-roadmap-integration-accepted/target/debug/riauth`.
It was executed in place. Its mtime stayed `2026-09-29 16:28:16 +0200`.

| Field | Observed value |
| --- | --- |
| `riauth --version` | `riauth 0.1.1` |
| Size | 204953512 bytes |
| SHA-256 | `264ed196ec6366cd96aac6c2c058fa605af15541efd7160496e05f0a6d115e2f` |

`1216585` is timestamped `2026-09-29 16:29:25 +0200`, after this binary. This
record does not say the binary was compiled from that commit. The same
directory also contains `riauth-maintenance` (64590624 bytes, mtime
`2026-09-29 16:28:16 +0200`) and `riauthctl` (22432744 bytes, mtime
`2026-09-28 15:25:40 +0200`). Neither was executed.

`riauth capabilities` on this binary printed an artifact catalog with
`edition` `essentials` and `build_features` `["essentials"]`. The Platform
guide's install command is what selects the Platform catalog. That install
was not run, so this record does not observe `edition` `platform`.

## Lab

The configuration, redb directory, session, password file, and client output
lived in one `mktemp` directory under `/tmp`, mode `700`. This page calls that
directory `$LAB`. The real path is not recorded. The password was 32
characters, written to a mode `600` file, and passed with `--password-stdin`.
The password, session token, and client secret are not recorded.

The server process was PID 32800. After the client create it was still
running. It was stopped with that PID. `serve_sigkill` was not required. TCP
9000 had no listener afterward. `~/.config/riauth/session.json` was absent
before and after. The lab directory was removed. The serve log was 343 bytes
and was scanned for the password, the session token, and the client secret.
None of those values were present. The log text was deleted with the lab.

The run started at `2026-09-29T14:40:44Z`. Commands below use the binary path
above. `$BIN` is that path. Global `--json` was not passed, so each success
prints the data object rather than the `riauth.cli/v1` envelope.

## Commands and results

### Capabilities

```sh
$BIN capabilities
```

Exit 0. Standard error was empty. Standard output was 18743 bytes. The data
document contained:

| Field | Value |
| --- | --- |
| `schema_version` | `riauth.capabilities/v2` |
| `version` | `0.1.1` |
| `interface` | `server` |
| `edition` | `essentials` |
| `build_features` | `["essentials"]` |
| `scope` | `artifact` |
| `feature_states` | 87 entries |
| `usable`, `enabled`, `configured`, `runtime_ready` | `null` on every entry |
| `compiled` true | 60 |
| `compiled` false | 27 |

[capability.rs](../../src/capability.rs) keeps an artifact catalog on this
command and sets `edition` from the compiled edition. The feature-count test
in that file expects 87 features, 27 Platform-only features, and 60 others.
The names of the 27 `compiled: false` entries were not saved individually.

### Init

```sh
$BIN --config $LAB/riauth.toml init \
  --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 \
  --data-dir data \
  --admin admin \
  --password-stdin \
  --non-interactive
```

Exit 0. Standard error: `Creating instance and signing key…`

```json
{
  "config": "$LAB/riauth.toml",
  "initialized": true,
  "issuer": "http://localhost:9000"
}
```

The configuration file was mode `600`. `$LAB/data/riauth.redb` existed. The
generated configuration was:

```toml
browser_ui = true
state_reconciliation_mode = "manual-review"
issuer = "http://localhost:9000"
listen = "127.0.0.1:9000"
data_dir = "data"
access_token_ttl = 300
refresh_token_ttl = 2592000
session_ttl = 28800
password_history = 5
trusted_proxies = []
reviewed_client_creation = false

[proxy_listeners]

[radius_listeners]

[ldap_listeners]

[directories]

[workspace_directories]

[entra_directories]

[scim_targets]

[signers]
```

The printed guide command is `riauth-maintenance ... init` without
`--password-stdin`. This run used `riauth init` and did not launch
`riauth-maintenance`. `doctor` later reported `storage` `redb` and no edition
field. This run does not display a Platform store stamp.

### Serve and readiness

```sh
$BIN --config $LAB/riauth.toml serve
curl --fail --silent --show-error --max-time 2 http://127.0.0.1:9000/readyz
```

`/readyz` exited 0. The body was:

```json
{"duties":{"authentication":true,"background_jobs":true,"protocol_listeners":false},"issuer":"http://localhost:9000","role":"integrated","service":"riAuth","status":"ok","version":"0.1.1"}
```

`protocol_listeners` stayed false. No LDAP, RADIUS, or proxy listener was
configured. On an Essentials build that duty is false for every role
([process_role.rs](../../src/process_role.rs)). The Platform-catalog run
later on this page reports the same duty true without adding a listener
table.

### Login and doctor

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  --non-interactive \
  login admin --password-stdin
```

Exit 0. Standard error was empty. `$LAB/session.json` was mode `600`.

```json
{
  "expires_at": 1790721646,
  "session_file": "$LAB/session.json",
  "user": {
    "admin": true,
    "attributes": {},
    "created_at": 1790692845,
    "display_name": "admin",
    "email": null,
    "email_verified": false,
    "enabled": true,
    "id": "8fb288a9-6de4-457a-afa6-77498bc8435d",
    "mfa_enabled": false,
    "password_available": true,
    "subjects": {},
    "username": "admin"
  }
}
```

`password_available` is the boolean field returned for this administrator.

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  doctor
```

Exit 0. Standard error was empty.

```json
{
  "active_signing_key": "cca03803-8c93-495f-b774-512488e1fed3",
  "checked_at": 1790692846,
  "clients": 0,
  "enabled_administrators": 1,
  "encrypted_at_rest": false,
  "healthy": true,
  "issuer": "http://localhost:9000",
  "pending_logout_deliveries": 0,
  "revision": 0,
  "schema_version": 3,
  "storage": "redb",
  "tls": "reverse_proxy",
  "users": 1
}
```

The printed guide login omits `--session-file` and writes
`~/.config/riauth/session.json`. This run overrode the session path. The
default file stayed absent. Browser sign-in at `/apps` and `prepare-setup`
were not run.

### Client create

The one management step is section 3's `local-demo` client, using the server
CLI because that is the authorized binary. Group create and audit export were
not run. The printed `riauthctl` command, `--secret-file`, `discovery`, and
`whoami` were not run.

Without the retry flags:

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  --non-interactive \
  client create local-demo \
  --name 'Local demo' \
  --confidential \
  --redirect-uri http://localhost:3000/callback \
  --scope openid,profile
```

Exit 1. Standard output was empty. Standard error was:

```text
error: Client writes require --idempotency-key and --if-revision (from `riauth revision`)
```

The source string wraps `riauth revision` in backticks. No client file was
written by this attempt.

Revision, read immediately before the successful write:

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  revision
```

Exit 0.

```json
{
  "revision": 0
}
```

Successful create:

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  --idempotency-key local-demo-create \
  --if-revision 0 \
  --output-file $LAB/local-demo.json \
  --run-id local-demo \
  --non-interactive \
  client create local-demo \
  --name 'Local demo' \
  --confidential \
  --redirect-uri http://localhost:3000/callback \
  --scope openid,profile
```

Exit 0. Standard error was empty.

```json
{
  "output_file": "$LAB/local-demo.json",
  "written": true
}
```

`$LAB/local-demo.json` was mode `600`. Its `client_secret` was a non-empty
string of length 53 and is redacted below. The stored client fields were:

```json
{
  "client": {
    "allowed_groups": [],
    "client_id": "local-demo",
    "confidential": true,
    "enabled": true,
    "name": "Local demo",
    "redirect_uris": [
      "http://localhost:3000/callback"
    ],
    "require_mfa": false,
    "scopes": [
      "openid",
      "profile"
    ],
    "service": false,
    "settings": {
      "access_token_encryption": null,
      "access_token_ttl": null,
      "allowed_grants": [],
      "authorization_encryption": null,
      "backchannel_logout_uri": null,
      "claim_mappings": [],
      "claims_in_access_token": false,
      "code_ttl": null,
      "default_acr_values": [],
      "device_ttl": null,
      "dpop_bound_access_tokens": false,
      "exchange": null,
      "exchange_from": [],
      "frontchannel_logout_uri": null,
      "groups_in_profile": false,
      "id_token_encryption": null,
      "issuer": null,
      "jwks": null,
      "machine_trust": [],
      "native": false,
      "origins": [],
      "pairwise_sector": null,
      "policy": {
        "access": {
          "all_groups": [],
          "any_groups": [],
          "denied_groups": [],
          "denied_users": [],
          "require_mfa": false,
          "users": []
        },
        "scopes": {}
      },
      "post_logout_redirect_uris": [],
      "refresh_token_ttl": null,
      "require_device_trust": false,
      "require_pushed_authorization_requests": false,
      "require_signed_request": false,
      "resources": {},
      "signing_key": null,
      "token_endpoint_auth_method": null,
      "token_managers": [],
      "userinfo_encryption": null,
      "userinfo_only": false,
      "userinfo_signed_response": false
    }
  },
  "client_secret": {
    "redacted": true,
    "length": 53
  }
}
```

`doctor` was not repeated after the create. The output file is the create
result. No process was left listening, and the lab files were removed.

## Unrun steps

Browser:

- Sign-in at `http://localhost:9000/apps`
- `prepare-setup` and `http://localhost:9000/setup`
- The admin page at `http://localhost:9000/admin`
- An application on `http://localhost:3000/callback`, the authorization
  redirect, and consent
- Passkey enrollment, rename, and removal
- Invitation acceptance at `/account/accept`
- The Playwright journeys named by the guide, including the accessibility
  spec and the invitation password, CDP passkey, and simulated-credential
  specs
- A spoken screen reader (VoiceOver, TalkBack, or NVDA)

Hardware and devices:

- A physical security key
- A synced platform passkey
- A phone hybrid transport
- iOS, Android, or another mobile operating system

Peers and external systems:

- LDAP directory import, plan, or apply
- An outbound SCIM target or a named SaaS directory
- SAML metadata exchange, an upstream identity provider, or a named service
  provider
- An LDAP provider listener and any LDAP client
- A workflow plan, apply, or configured-workflow run
- RADIUS, proxy SSO, Google Workspace, and Microsoft Entra
- SMTP or an external mailbox
- PostgreSQL, native HTTPS, and a trusted proxy

Essentials guide and the remaining Platform guide:

- [Essentials guide](../essentials-guide.md) was not executed
- `cargo install` was not run for either edition
- Groups, claims, and audit were not run in this Essentials-catalog run
- Backup, restore, and recovery were not run
- `riauthctl` and `riauth-maintenance` were not run
- Sections 11 through 14 of the Platform guide were not run, including
  signing-key import, the `legacy-directory` client, and invitation acceptance

The shared setup and `local-demo` registration above ran on a binary whose
catalog edition was `essentials`. The next section is a separate
Platform-catalog run. D01 remains incomplete.

## Platform catalog run

The source tree during this run was
`58357fde77211e62dc51c14fb3fc216bdf143ceb`. No Cargo build was run.
`CARGO_TARGET_DIR` was unset. The binary was the supplied snapshot
`/tmp/riauth-platform-58357fd-immutable/riauth`, described as the Platform
debug binary built from that commit. This run checked the stated hash and
the artifact catalog. It did not compile the binary.

The snapshot was copied into a new `mktemp` directory under `/tmp`, mode
`700`. This page calls that directory `$LAB`. The real path is not recorded.
Every `riauth` command used `$LAB/riauth`. The snapshot was not executed.
Its mode stayed `-r-x------` and its mtime stayed `2026-09-29 17:00:59 +0200`.

| Field | Observed value |
| --- | --- |
| `riauth --version` | `riauth 0.1.1` |
| Size | 289661864 bytes |
| SHA-256 | `de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069` |
| Copy mode | `700` |

The copy's SHA-256 matched the snapshot before any `riauth` invocation.
`riauthctl` and `riauth-maintenance` were not on this path and were not
executed. `cargo install` was not run.

The password was 32 characters, written to a mode `600` file, and passed
with `--password-stdin`. The password, session token, and client secret are
not recorded. Login standard output kept `user`, `expires_at`, and
`session_file`. The session file stayed in `$LAB`. Client creation wrote
the full document to a private file; this page keeps the public client
fields and the secret length.

The server process was PID 10752. It was stopped with that PID.
`serve_sigkill` was not required. TCP 9000 had no listener afterward.
`~/.config/riauth/session.json` was absent before and after. The lab
directory was removed. The serve log was 343 bytes and was scanned for the
password, the session token, and the client secret. None of those values
were present. The log included a `background_overloaded` warning for the
`alerts` delivery lane before the listening line. `/readyz` was still
`status` `ok`. The log text was deleted with the lab.

The run started at `2026-09-29T15:09:54Z`. Global `--json` was not passed.

### Capabilities

```sh
$BIN capabilities
```

Exit 0. Standard error was empty. Standard output was 20679 bytes.

| Field | Value |
| --- | --- |
| `schema_version` | `riauth.capabilities/v2` |
| `version` | `0.1.1` |
| `interface` | `server` |
| `edition` | `platform` |
| `build_features` | `["essentials", "platform"]` |
| `scope` | `artifact` |
| `target` | `macos` / `aarch64` |
| `feature_states` | 87 entries |
| `compiled_features` | 87 |
| `usable`, `enabled`, `configured`, `runtime_ready` | `null` on every entry |
| `compiled` true | 87 |
| `compiled` false | 0 |

### Help verbs

`riauth group --help` listed `review`, `list`, `create`, `add-member`, and
`remove-member`. It did not list `get` or `has-member`. `riauth client
--help` listed `list`, `endpoint-review`, `status-review`,
`creation-review`, `review`, `create`, `update`, `disable`, `enable`, and
`rotate-secret`. It did not list `get`.

### Init

```sh
$BIN --config $LAB/riauth.toml init \
  --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 \
  --data-dir data \
  --admin admin \
  --password-stdin \
  --non-interactive
```

Exit 0. Standard error: `Creating instance and signing key…`

```json
{
  "config": "$LAB/riauth.toml",
  "initialized": true,
  "issuer": "http://localhost:9000"
}
```

The configuration file was mode `600`. `$LAB/data/riauth.redb` existed. The
generated scalars and empty tables matched the Essentials-catalog run:
`reviewed_client_creation = false`, `data_dir = "data"`, and empty
`proxy_listeners`, `radius_listeners`, `ldap_listeners`, `directories`,
`workspace_directories`, `entra_directories`, `scim_targets`, and `signers`.
No other tables were present. The printed guide command is
`riauth-maintenance ... init`. This run used `riauth init`.

### Serve and readiness

```sh
$BIN --config $LAB/riauth.toml serve
curl --fail --silent --show-error --max-time 2 http://127.0.0.1:9000/readyz
```

`/readyz` exited 0. The body was:

```json
{"duties":{"authentication":true,"background_jobs":true,"protocol_listeners":true},"issuer":"http://localhost:9000","role":"integrated","service":"riAuth","status":"ok","version":"0.1.1"}
```

`protocol_listeners` was true. The listener tables above were still empty.
That duty means this integrated Platform process owns listener startup
([process_role.rs](../../src/process_role.rs)). It does not mean an LDAP,
RADIUS, or proxy listener was configured.

### Login and doctor

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  --non-interactive \
  login admin --password-stdin
```

Exit 0. Standard error was empty. `$LAB/session.json` was mode `600`. The
session token is omitted. The public user fields were `id`
`127ca593-a85e-4244-a67c-c2d580ad86ab`, `username` `admin`, `display_name`
`admin`, `email` null, `enabled` true, `admin` true, `mfa_enabled` false,
`password_available` true, `email_verified` false, `created_at` 1790694596,
empty `attributes`, and empty `subjects`. `expires_at` was 1790723398.

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  doctor
```

Exit 0. Standard error was empty. `healthy` was true, `schema_version` 3,
`revision` 0, `storage` `redb`, `encrypted_at_rest` false,
`active_signing_key` `560689b5-d881-4cb3-99ef-ec9277bb9b36`, `users` 1,
`enabled_administrators` 1, `clients` 0, `pending_logout_deliveries` 0,
`tls` `reverse_proxy`, `checked_at` 1790694598, and `issuer`
`http://localhost:9000`. There was no `edition` field. `doctor` was not
repeated after the later writes. This run does not display a store stamp.

The printed guide login omits `--session-file`. The default session path
stayed absent. Browser sign-in at `/apps` and `prepare-setup` were not run.

### Client create

The printed section 3 command is `riauthctl ... client create` with
`--secret-file`. This run used the server CLI. Sections 4 and 5 were not
run between this client and the group commands. `discovery`, `whoami`, and
the application on port 3000 were not run.

Without the retry flags:

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  --non-interactive \
  client create local-demo \
  --name 'Local demo' \
  --confidential \
  --redirect-uri http://localhost:3000/callback \
  --scope openid,profile
```

Exit 1. Standard output was empty. Standard error was:

```text
error: Client writes require --idempotency-key and --if-revision (from `riauth revision`)
```

The source string wraps `riauth revision` in backticks. No client file was
written by this attempt.

`revision` immediately before the write returned `{"revision": 0}`.

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  --run-id local-demo \
  --idempotency-key local-demo-create \
  --if-revision 0 \
  --output-file $LAB/local-demo.json \
  --non-interactive \
  client create local-demo \
  --name 'Local demo' \
  --confidential \
  --redirect-uri http://localhost:3000/callback \
  --scope openid,profile
```

Exit 0. Standard error was empty. Standard output was:

```json
{
  "output_file": "$LAB/local-demo.json",
  "written": true
}
```

`$LAB/local-demo.json` was mode `600`. `client_secret` was a non-empty
string of length 53 and is omitted. The public client fields were
`client_id` `local-demo`, `name` `Local demo`, `confidential` true,
`enabled` true, `service` false, `require_mfa` false, `allowed_groups` `[]`,
`redirect_uris` `["http://localhost:3000/callback"]`, `scopes` `openid` and
`profile`, `claim_mappings` `[]`, and `claims_in_access_token` false. The
rest of the settings object is not copied here.

### Group create and membership

The printed section 6 commands are `riauthctl`, including `group get` and
`group has-member`. This run used `riauth`. `group --help` has no `get` or
`has-member`, so those two commands were not run.

Without the retry flags, `group create staff` exited 1 with empty standard
output and this standard error:

```text
error: Group writes require --idempotency-key and --if-revision (from `riauth revision`)
```

The source string wraps `riauth revision` in backticks.

`revision` immediately before create returned `{"revision": 1}`.

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  --run-id staff-group \
  --idempotency-key staff-create \
  --if-revision 1 \
  group create staff
```

Exit 0. The group name was `staff` and `member_count` was 0. Member ids are
not copied here.

`revision` immediately before add-member returned `{"revision": 2}`.

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  --run-id staff-group \
  --idempotency-key staff-add-admin \
  --if-revision 2 \
  group add-member staff admin
```

Exit 0. The group name was `staff` and `member_count` was 1.

### Claim update and explain

The printed section 7 update is `riauthctl client update` with the settings
file below. This run used `riauth client update` and the same JSON object.
`revision` immediately before the update returned `{"revision": 3}`.

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  --run-id local-demo-claims \
  --idempotency-key local-demo-claims \
  --if-revision 3 \
  client update local-demo \
  --scope openid,profile,groups \
  --settings-file $LAB/local-demo-claims.json
```

Exit 0. Standard output was the client view, not a secret wrapper. The
recorded public fields were the same client id, name, confidential,
enabled, service, redirect, `require_mfa`, and empty `allowed_groups`.
`scopes` were `groups`, `openid`, and `profile`. `claims_in_access_token`
stayed false. `claim_mappings` was one entry: scope `profile`, claim
`department`, source type `literal`, value `lab`. The rest of the client
view is not copied here.

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  explain local-demo admin \
  --scope 'openid profile groups'
```

Exit 0. `simulation` was true, `token_issued` was false, `allowed` was
true, and `reasons` was empty. `userinfo.groups` was `["staff"]`,
`userinfo.department` was `lab`, and `userinfo.name` and
`userinfo.preferred_username` were `admin`. `userinfo` also contained
`sub`. `access_token_identity_claims` contained only `sub`.
`id_token_identity_claims` contained `department`, `groups`, `name`,
`preferred_username`, and `sub`. No token was issued.

### Audit

These are the printed section 8 commands, with `--session-file` and a lab
`--out` instead of `deployment-private/platform-lab/audit-staff-group.csv`.

```sh
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  audit --limit 100
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  inventory audit --limit 100 --filter staff-group
$BIN --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  report audit --run-id staff-group --out $LAB/audit-staff-group.csv
```

`audit --limit 100` exited 0 and returned six rows. `details` is not
copied. In returned order:

| `action` | `target` | `run_id` |
| --- | --- | --- |
| `login.succeeded` | `3c9c6d1a-75fd-4c27-ab92-85096a679248` | null |
| `group.member.add` | `staff/127ca593-a85e-4244-a67c-c2d580ad86ab` | `staff-group` |
| `group.create` | `staff` | `staff-group` |
| `client.create` | `local-demo` | `local-demo` |
| `client.update` | `local-demo` | `local-demo-claims` |
| `instance.initialize` | `admin` | null |

CLI password login audits `login.succeeded` with the bearer session id
([core.rs](../../src/core.rs)). That id is the first target. It is not the
session token. The membership target is `staff/` plus the administrator id
from login.

`inventory audit` exited 0. `item_count` was 2, `limit` was 100,
`next_cursor` was null, and `revision` was 4. The items were `group.create`
then `group.member.add`, both with `run_id` `staff-group` and the targets
above. `details` is not copied. No `--after` page was requested.

`report audit` exited 0. Standard output was `rows` 2, `pages` 1, and
`output_file` `$LAB/audit-staff-group.csv`. The file was mode `600`. Its
header was `id,at,actor,action,target,run_id,request_id`. It had two data
rows, `group.member.add` and `group.create`, with the same targets and
`run_id` `staff-group`. The CSV was deleted with the lab and is not
committed.

### Unrun on this Platform-catalog run

The executed chain is server-CLI setup, `local-demo` create, `staff` create
and add-member, the claim update, `explain`, and the three audit commands.
It is not the printed `riauthctl` text, and it skips guide sections 4 and 5.

Still unrun:

- `cargo install` of `riauth`, `riauth-maintenance`, and `riauthctl`
- `riauth-maintenance init` and every `riauth-maintenance` command
- `riauthctl` login, client create, discovery, whoami, group commands, and
  client update
- `group get`, `group has-member`, and any server-CLI `client get`
- Browser sign-in, `/apps`, `/setup`, `/admin`, port 3000, consent, and
  passkey enrollment
- Invitation acceptance and the Playwright journeys named by the guide
- A spoken screen reader, a physical key, a synced passkey, a phone, or a
  mobile operating system
- Backup, restore, and recovery
- Sections 9 through 14: LDAP import, outbound SCIM, workflows, SAML,
  the LDAP provider, and invitation acceptance
- PostgreSQL, native HTTPS, a trusted proxy, RADIUS, proxy SSO, Workspace,
  Entra, and an external mailbox

The Essentials guide was not executed. D01 remains incomplete.

## Printed remote administration run

The docs worktree for this record is
`9c374beae00e99fbc7f922ef44c23243d6fbc28a`. No Cargo build was run.
`CARGO_TARGET_DIR` was unset. Three immutable snapshots were copied into a
new `mktemp` directory under `/tmp`, mode `700`. This page calls that
directory `$LAB`. The real path is not recorded. Every command used the
copies `$LAB/riauth`, `$LAB/riauthctl`, and `$LAB/riauth-maintenance`. The
snapshot paths were not executed. Their modes stayed `-r-x------`, and their
mtimes stayed at the values below.

The server snapshot is
`/tmp/riauth-platform-58357fd-immutable/riauth`, supplied as the Platform
debug binary for `58357fde77211e62dc51c14fb3fc216bdf143ceb`. The tool
snapshots are `/tmp/riauth-tools-f430c2f-immutable/riauthctl` and
`/tmp/riauth-tools-f430c2f-immutable/riauth-maintenance`, supplied for
`f430c2f01b63cbd53ffd0f56ebd277e03724d822`. This run checked the stated
hashes and the server copy's artifact catalog. It did not compile the
binaries.

`58357fd` is an ancestor of `f430c2f`. The Rust source that changed between
them is `4f5cde4` in [provisioning.rs](../../src/provisioning.rs) and
[reconciliation.rs](../../src/reconciliation.rs), with
`tests/reconciliation_completion.rs` and small edits in the reconciliation
tests. `395097c` and `f430c2f` change documentation. `crates/riauthctl` and
[cli/local.rs](../../src/cli/local.rs) have an empty diff across that range.
`9c374be` is the child of `f430c2f` and changes
[assembly.rs](../../src/assembly.rs),
[source_catalog.rs](../../src/assembly/source_catalog.rs),
[source.rs](../../src/source.rs), `tests/source_boundary.rs`, and
`scripts/check-module-boundaries.py`. That commit is in neither snapshot.
The three binaries were not produced by one `cargo install` in this task.

| Binary | Version | Size | SHA-256 | mtime |
| --- | --- | --- | --- | --- |
| `riauth` | `riauth 0.1.1` | 289661864 | `de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069` | `2026-09-29 17:00:59 +0200` |
| `riauthctl` | `riauthctl 0.1.1` | 22431144 | `edccd38a893972b1d369a3743a0e069c698fab99ee3508f066aa639b6f6602a3` | `2026-09-29 17:16:50 +0200` |
| `riauth-maintenance` | `riauth-maintenance 0.1.1` | 85114656 | `7b2293f9548f74d8e15a14e851bc0594c67c912fe4fbd2bf6e63e485e4cd71ab` | `2026-09-29 17:16:50 +0200` |

Copy mode was `700`. Each copy's SHA-256 matched its snapshot before the
first invocation of that copy. `riauthctl` and `riauth-maintenance` printed
a version and did not print an edition. The only edition in this run is the
server copy's artifact catalog below.

The password was 32 characters, written to a mode `600` file, and passed
with `--password-stdin`. The password, both session tokens, and the client
secret are not recorded. The server CLI session and the riauthctl session
were separate files in `$LAB`, each mode `600`. `~/.config/riauth/session.json`
and `~/.config/riauthctl/session.json` were absent before and after.

The server process was PID 64511. It was stopped with that PID.
`serve_sigkill` was not required. TCP 9000 had no listener afterward. The
lab directory was removed. The serve log was 141 bytes and was scanned for
the password, both session tokens, and the client secret. None of those
values were present. The log did not contain `background_overloaded`. The
log text was deleted with the lab.

The run started at `2026-09-29T15:30:04Z`. Global `--json` was not passed.
Progress notes were kept off standard output so a revision read stayed a
single JSON object.

### Capabilities and help

```sh
$LAB/riauth capabilities
```

Exit 0. Standard error was empty. Standard output was 20679 bytes.

| Field | Value |
| --- | --- |
| `schema_version` | `riauth.capabilities/v2` |
| `version` | `0.1.1` |
| `interface` | `server` |
| `edition` | `platform` |
| `build_features` | `["essentials", "platform"]` |
| `scope` | `artifact` |
| `target` | `macos` / `aarch64` |
| `feature_states` | 87 entries |
| `compiled_features` | 87 |
| `usable`, `enabled`, `configured`, `runtime_ready` | `null` on every entry |
| `compiled` true | 87 |
| `compiled` false | 0 |

Before any mutation, the copies' help was read. `$LAB/riauth group --help`
listed `review`, `list`, `create`, `add-member`, and `remove-member`.
`$LAB/riauthctl group --help` listed `list`, `get`, `has-member`, `create`,
`add-member`, and `remove-member`. `$LAB/riauthctl client --help` listed
`list`, `get`, `create`, `update`, `rotate-secret`, and `disable`.
`$LAB/riauthctl --help` listed `status`, `discovery`, `login`, `whoami`,
`logout`, `revision`, `inventory`, `user`, `group`, `client`, `session`,
`request`, `device`, `authorize`, `plan`, `apply`, and `passkey`. It did
not list `doctor`, `explain`, `audit`, or `report`.
`$LAB/riauth-maintenance --help` listed `init` and did not list `serve`.
`$LAB/riauth-maintenance init --help` included `--password-stdin`.
`$LAB/riauthctl client create --help` included `--secret-file`.
`$LAB/riauth report audit --help` included `--out` and `--run-id`.

`doctor` remains the server CLI command. The probe below was the unsupported
`riauthctl` spelling:

```sh
$LAB/riauthctl doctor
```

Exit 2. Standard output was empty. Standard error was:

```text
error: unrecognized subcommand 'doctor'

Usage: riauthctl [OPTIONS] <COMMAND>

For more information, try '--help'.
```

That rejection is local clap parsing. It sent no request. `riauthctl status`
was not run in its place.

### Maintenance init

```sh
$LAB/riauth-maintenance --config $LAB/riauth.toml init \
  --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 \
  --data-dir data \
  --admin admin \
  --password-stdin \
  --non-interactive
```

Exit 0. Standard error: `Creating instance and signing key…`

```json
{
  "config": "$LAB/riauth.toml",
  "initialized": true,
  "issuer": "http://localhost:9000"
}
```

The configuration file was mode `600`. `$LAB/data/riauth.redb` existed. The
generated configuration was:

```toml
browser_ui = true
state_reconciliation_mode = "manual-review"
issuer = "http://localhost:9000"
listen = "127.0.0.1:9000"
data_dir = "data"
access_token_ttl = 300
refresh_token_ttl = 2592000
session_ttl = 28800
password_history = 5
trusted_proxies = []
reviewed_client_creation = false

[proxy_listeners]

[radius_listeners]

[ldap_listeners]

[directories]

[workspace_directories]

[entra_directories]

[scim_targets]

[signers]
```

The maintenance command did not print an edition. This run does not display
a store stamp. The printed guide prompts for the password; this run passed
`--password-stdin` and `--non-interactive`.

### Serve and readiness

```sh
$LAB/riauth --config $LAB/riauth.toml serve
curl --fail --silent --show-error --max-time 2 http://127.0.0.1:9000/readyz
```

`/readyz` exited 0. The body was:

```json
{"duties":{"authentication":true,"background_jobs":true,"protocol_listeners":true},"issuer":"http://localhost:9000","role":"integrated","service":"riAuth","status":"ok","version":"0.1.1"}
```

`protocol_listeners` was true on this server copy. The listener tables above
were empty. The duty comes from the integrated Platform server process
([process_role.rs](../../src/process_role.rs)). It does not identify the
maintenance binary's edition.

### Server CLI login and doctor

```sh
$LAB/riauth --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  --non-interactive \
  login admin --password-stdin
```

Exit 0. Standard error was empty. The session token is omitted. The public
user fields were `id` `b2f7a829-d698-4b62-8e10-e2868d8f755a`, `username`
`admin`, `display_name` `admin`, `email` null, `enabled` true, `admin` true,
`mfa_enabled` false, `password_available` true, `email_verified` false,
`created_at` 1790695809, empty `attributes`, and empty `subjects`.
`expires_at` was 1790724610.

```sh
$LAB/riauth --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  doctor
```

Exit 0. Standard error was empty. `healthy` was true, `schema_version` 3,
`revision` 0, `storage` `redb`, `encrypted_at_rest` false,
`active_signing_key` `6018afc5-3d3e-436e-862e-9a3dcff16d10`, `users` 1,
`enabled_administrators` 1, `clients` 0, `pending_logout_deliveries` 0,
`tls` `reverse_proxy`, `checked_at` 1790695810, and `issuer`
`http://localhost:9000`. There was no `edition` field. `doctor` was not
repeated after the later writes. Browser sign-in at `/apps` and
`prepare-setup` were not run.

### riauthctl login, client create, discovery, and whoami

```sh
$LAB/riauthctl --server http://localhost:9000 \
  --session-file $LAB/riauthctl-session.json \
  --non-interactive \
  login admin --password-stdin
```

Exit 0. Standard output named `user`, `expires_at`, and `session_file`.
The same public user fields were present, with the same `id`, `created_at`,
and `expires_at` 1790724610. `password_available` was the string
`[redacted]`. [riauthctl main.rs](../../crates/riauthctl/src/main.rs)
replaces a field whose name contains `password` before it prints. The
server CLI login above is the boolean observation. The session token was
not in standard output.

The printed client create omits `--idempotency-key`, `--if-revision`, and
`--run-id`. This run did the same, and added `--session-file` and
`--non-interactive`. The secret file was `$LAB/local-demo-secret.json`
instead of `deployment-private/local-demo-secret.json`.

```sh
$LAB/riauthctl --server http://localhost:9000 \
  --session-file $LAB/riauthctl-session.json \
  --non-interactive \
  client create local-demo \
  --name 'Local demo' \
  --confidential \
  --redirect-uri http://localhost:3000/callback \
  --scope openid,profile \
  --secret-file $LAB/local-demo-secret.json
```

Exit 0. Standard output had `client` and `credential_file`. It had no
`client_secret` field. `credential_file` was `$LAB/local-demo-secret.json`.
The secret file was mode `600`. Its keys were `client` and `client_secret`.
`client_secret` was a non-empty string of length 53 and is omitted. The
public client fields were `client_id` `local-demo`, `name` `Local demo`,
`confidential` true, `enabled` true, `service` false, `require_mfa` false,
`allowed_groups` `[]`, `redirect_uris` `["http://localhost:3000/callback"]`,
and `scopes` `openid` and `profile`. `claim_mappings` was empty. The value
printed for `claims_in_access_token` was the string `[redacted]`, because
that field name ends in `_token` and the same riauthctl redaction replaces
it. The boolean stored for that field was not visible in this output. No
idempotency key was printed. The later audit row for this create has
`run_id` null.

```sh
$LAB/riauthctl --server http://localhost:9000 \
  --session-file $LAB/riauthctl-session.json \
  --non-interactive \
  discovery
$LAB/riauthctl --server http://localhost:9000 \
  --session-file $LAB/riauthctl-session.json \
  --non-interactive \
  whoami
```

`discovery` exited 0. Standard output was 3662 bytes. `issuer` was
`http://localhost:9000`. `authorization_endpoint` was
`http://localhost:9000/oauth/authorize`, `token_endpoint` was
`http://localhost:9000/oauth/token`, `userinfo_endpoint` was
`http://localhost:9000/oauth/userinfo`, and `jwks_uri` was
`http://localhost:9000/oauth/jwks`.

`whoami` exited 0. It returned the same user, `groups` `[]`, `mfa` false,
`expires_at` 1790724610, and `session_id`
`5c7a2180-0b29-4e37-9280-e0587f63a154`. `password_available` was again the
string `[redacted]`. This `whoami` ran before the group commands, so the
empty group list is the membership at that moment. The application on port
3000 was not started.

### Groups, claims, and audit

`revision` immediately before create returned `{"revision": 1}`.

```sh
$LAB/riauthctl --server http://localhost:9000 \
  --session-file $LAB/riauthctl-session.json \
  --non-interactive \
  --run-id staff-group \
  --idempotency-key staff-create \
  --if-revision 1 \
  group create staff
```

Exit 0. The group name was `staff` and the member list was empty.

`revision` immediately before add-member returned `{"revision": 2}`.

```sh
$LAB/riauthctl --server http://localhost:9000 \
  --session-file $LAB/riauthctl-session.json \
  --non-interactive \
  --run-id staff-group \
  --idempotency-key staff-add-admin \
  --if-revision 2 \
  group add-member staff admin
```

Exit 0. The group name was `staff` and the member list had one string.

```sh
$LAB/riauthctl --server http://localhost:9000 \
  --session-file $LAB/riauthctl-session.json \
  --non-interactive \
  group get staff
$LAB/riauthctl --server http://localhost:9000 \
  --session-file $LAB/riauthctl-session.json \
  --non-interactive \
  group has-member staff admin
```

`group get` exited 0 and returned `staff` with one member. `group
has-member` exited 0 and returned `group` `staff`, `username` `admin`, and
`member` true. Member ids are not copied here.

The settings file was the printed claim-mapping object, saved under `$LAB`
instead of `deployment-private/platform-lab/local-demo-claims.json`.
`revision` immediately before the update returned `{"revision": 3}`.

```sh
$LAB/riauthctl --server http://localhost:9000 \
  --session-file $LAB/riauthctl-session.json \
  --non-interactive \
  --run-id local-demo-claims \
  --idempotency-key local-demo-claims \
  --if-revision 3 \
  client update local-demo \
  --scope openid,profile,groups \
  --settings-file $LAB/local-demo-claims.json
```

Exit 0. Standard output was the client view. `scopes` were `groups`,
`openid`, and `profile`. `claim_mappings` was one entry: scope `profile`,
claim `department`, source type `literal`, value `lab`. The printed
`claims_in_access_token` value was again the string `[redacted]`. The
settings file did not set that field. The rest of the settings object is
not copied here.

```sh
$LAB/riauth --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  explain local-demo admin \
  --scope 'openid profile groups'
```

Exit 0. `simulation` was true, `token_issued` was false, `allowed` was
true, and `reasons` was empty. `userinfo.groups` was `["staff"]`,
`userinfo.department` was `lab`, and `userinfo.name` and
`userinfo.preferred_username` were `admin`. `userinfo` also contained
`sub`. `access_token_identity_claims` contained only `sub`.
`id_token_identity_claims` contained `department`, `groups`, `name`,
`preferred_username`, and `sub`. The response also included `mfa_assumed`
and `scope_decisions`; those values are not copied here. No token was
issued.

```sh
$LAB/riauth --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  audit --limit 100
$LAB/riauth --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  inventory audit --limit 100 --filter staff-group
$LAB/riauth --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/session.json \
  report audit --run-id staff-group --out $LAB/audit-staff-group.csv
```

`audit --limit 100` exited 0 and returned seven rows. `details` is not
copied. In returned order:

| `action` | `target` | `run_id` |
| --- | --- | --- |
| `login.succeeded` | `5c7a2180-0b29-4e37-9280-e0587f63a154` | null |
| `client.create` | `local-demo` | null |
| `group.create` | `staff` | `staff-group` |
| `client.update` | `local-demo` | `local-demo-claims` |
| `group.member.add` | `staff/b2f7a829-d698-4b62-8e10-e2868d8f755a` | `staff-group` |
| `instance.initialize` | `admin` | null |
| `login.succeeded` | `28a89b95-413b-4925-8fef-ced4ca4a40d3` | null |

CLI password login audits `login.succeeded` with the bearer session id
([core.rs](../../src/core.rs)). The first target equals the `whoami`
`session_id` from the riauthctl login. The last target is the other bearer
session, from the server CLI login. Neither target is a session token.
`client.create` has a null `run_id` because that command did not pass
`--run-id`. The membership target is `staff/` plus the administrator id.

`inventory audit` exited 0. `item_count` was 2, `limit` was 100,
`next_cursor` was null, and `revision` was 4. In returned order the items
were `group.member.add` and then `group.create`, both with `run_id`
`staff-group` and the targets above. `details` is not copied. No `--after`
page was requested.

`report audit` exited 0. Standard output was `rows` 2, `pages` 1, and
`output_file` `$LAB/audit-staff-group.csv`. The file was mode `600`. Its
header was `id,at,actor,action,target,run_id,request_id`. The data rows, in
file order, were `group.create` and then `group.member.add`, with the same
targets and `run_id` `staff-group`. The CSV was deleted with the lab and is
not committed.

### Unrun on this remote-administration run

The executed chain is maintenance init, Platform server startup and
`/readyz`, server CLI login and `doctor`, riauthctl login, the printed
`local-demo` create, `discovery`, `whoami`, the printed group commands
including `group get` and `group has-member`, the printed claim update,
`explain`, and the three audit commands. `riauthctl doctor` was attempted
and rejected locally.

Still unrun:

- `cargo install` of `riauth`, `riauth-maintenance`, and `riauthctl`
- Browser sign-in, `/apps`, `/setup`, `/admin`, port 3000, consent, and
  passkey enrollment
- Sections 4 and 5, including backup, restore, and recovery
- Sections 9 through 14: LDAP import, outbound SCIM, workflows, SAML,
  the LDAP provider, and invitation acceptance
- Signing-key commands, including `keys import`, `keys bind`, and
  `keys generate`
- `riauthctl status`, `plan`, `apply`, and `passkey`
- A second `riauthctl login`; the saved riauthctl session was still present
- A spoken screen reader, a physical key, a synced passkey, a phone, or a
  mobile operating system
- PostgreSQL, native HTTPS, a trusted proxy, RADIUS, proxy SSO, Workspace,
  Entra, and an external mailbox

The Essentials guide was not executed. D01 remains incomplete.

## Backup and restore entry points

The docs worktree for this record is
`f0d714c96652dcb2c6a9898b2e0c32cfea729b73`. No Cargo build was run.
`CARGO_TARGET_DIR` was unset. The authorized binary was the Platform server
snapshot
`/tmp/riauth-platform-58357fd-immutable/riauth`, the same file the catalog
run and the remote-administration run copied. Its mode stayed `-r-x------`,
its size stayed 289661864 bytes, its mtime stayed
`2026-09-29 17:00:59 +0200`, and its SHA-256 stayed
`de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069`. The
snapshot path was not executed. A copy lived in a new `mktemp` directory
under `/tmp`, mode `700`. The copy was chmod `700`, and its SHA-256 matched
the snapshot before any command. This page calls that directory `$LAB`. The
copy's real path is not recorded. `riauthctl` and `riauth-maintenance` were
not launched. `deployment-private/` was not written.

`riauth --version` printed `riauth 0.1.1`. This run did not print
`riauth capabilities` or `riauth doctor`. The earlier runs of this snapshot recorded
catalog `edition` `platform`. `/readyz` reported `protocol_listeners` true.
That duty does not identify a maintenance binary, and this run did not
launch one.

Help was read on the copy before any write:

| Command | Usage line |
| --- | --- |
| `keygen` | `Usage: riauth keygen [OPTIONS] --out <OUT>` |
| `backup` | `Usage: riauth backup [OPTIONS] --key-file <KEY_FILE> --out <OUT>` |
| `restore` | `Usage: riauth restore [OPTIONS] --backup <BACKUP> --key-file <KEY_FILE> --out <OUT>` |
| `recovery` | `Usage: riauth recovery [OPTIONS] <COMMAND>` |
| `recovery status` | `Usage: riauth recovery status [OPTIONS]` |
| `recovery invalidate` | `Usage: riauth recovery invalidate [OPTIONS] --database-restored` |
| `recovery complete` | `Usage: riauth recovery complete [OPTIONS] --recovery-id <RECOVERY_ID>` |
| `recover-admin` | `Usage: riauth recover-admin [OPTIONS] <USERNAME>` |

`complete` help also offers `--persistent-credentials-reconciled`.
`recover-admin` help also offers `--password-stdin` and `--reset-mfa`.
`restore` help also offers `--postgres-config` and `--database-key-file`.
The printed section 5 names `riauth-maintenance keygen` and
`riauth-maintenance restore`. This binary's help exposes those operations
on `riauth`. The run followed that help and omitted `--postgres-config` and
`--database-key-file`.

### Setup

The kept run started at `2026-09-29T15:50:16Z`. The prerequisite was a
section 2 loopback on this copy:

```sh
riauth --config $LAB/riauth.toml --non-interactive init \
  --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 \
  --data-dir data \
  --admin admin \
  --password-stdin
```

Exit 0. Standard error: `Creating instance and signing key…`. Standard
output had `initialized` true and issuer `http://localhost:9000`. The
configuration file was mode `600`. It had no `database_key_file` and no
PostgreSQL block. `reviewed_client_creation` was false, `browser_ui` was
true, `state_reconciliation_mode` was `manual-review`, `data_dir` was
`data`, the token lifetimes were 300, 2592000, and 28800, `password_history`
was 5, and `trusted_proxies` was empty. The listener tables, directory
tables, `scim_targets`, and `signers` were empty. `$LAB/data/riauth.redb`
existed, mode `600`, 114688 bytes.

`riauth --config $LAB/riauth.toml serve` stayed up through backup. SIGTERM
stopped it, and SIGKILL was not required. `/readyz` returned HTTP 200:

```json
{"duties":{"authentication":true,"background_jobs":true,"protocol_listeners":true},"issuer":"http://localhost:9000","role":"integrated","service":"riAuth","status":"ok","version":"0.1.1"}
```

The serve log was 986 bytes. A scan for the administrator password, the
backup key, and the session token found no hits. The log contained the
`background_overloaded` marker. The log text is not copied.

Login used the private session file:

```sh
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/session.json --non-interactive \
  login admin --password-stdin
```

Exit 0. Standard error was empty. The session file was mode `600`. The
token is not copied. The user id was `de1ad3a3-4339-4622-ba1d-282da2989a12`,
username `admin`, display name `admin`, email null, enabled true, admin
true, `mfa_enabled` false, `password_available` true, `email_verified`
false, `created_at` 1790697020, and `expires_at` 1790725821. Attributes and
subjects were empty.

The password file was mode `600` and held 32 alphanumeric characters plus a
newline. `riauth --non-interactive keygen --out $LAB/backup.key` exited 0
with `created` true. The key file was mode `600` and 43 bytes. It is a
separate file from the absent database key. Neither secret is copied.

### Backup

Backup ran while serve was still listening:

```sh
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/session.json --non-interactive \
  backup --key-file $LAB/backup.key --out $LAB/backup.riauth
```

Exit 0. Standard error was empty. The result was `api_version`
`riauth.backup/v3`, `encrypted` true, `verified` true, issuer
`http://localhost:9000`, `created_at` 1790697022, `frames` 3, `records` 20,
and `bytes` 12489. The published file was mode `600` and 12489 bytes. No
partial file remained. The stream id was non-empty and 22 characters. The
transcript was non-empty and 43 characters. Those two values are omitted.

`verified` is the backup command's archive check in
[stream.rs](../../src/operations/stream.rs): frame authentication, the
trailer transcript over the bytes before the trailer, end of file after the
trailer, and the schema and issuer checks. A hash of the whole file,
including the trailer, is a different input. This run did not reimplement
that split. It kept the command result, and restore later returned
`verified` true as well. Neither flag means a person signed in again or
that an application completed login.

### Restore and status

Serve was stopped first. Port 9000 was free, and no lab `riauth` process
remained.

```sh
riauth --non-interactive restore \
  --backup $LAB/backup.riauth \
  --key-file $LAB/backup.key \
  --out $LAB/restored
```

Exit 0. Standard error was empty. The result was `restored` true,
`verified` true, `encrypted_at_rest` false, `storage` `redb`,
`serving_allowed` false, and issuer `http://localhost:9000`. The published
configuration basename was `riauth.toml`, mode `600`, with the same public
scalars as the original file. `$LAB/restored/data/riauth.redb` was mode
`600` and 61440 bytes. The original redb file was still present. The `next`
field was the template that names `riauth recovery complete` and
`--persistent-credentials-reconciled`. That command was not run.

The recovery id was `e83d782d-3e10-43a9-bd0a-ec1e238137e5`. The policy was
`riauth.recovery/v1`. The cause was `backup_restore`. `invalidated_at` and
`snapshot_created_at` were 1790697022. `completed_at` was null.
`epoch_advanced_users` was 1. Invalidated counts were `sessions` 1 and
`session_tokens` 1. Reconcile counts were `enabled_accounts` 1, `passwords`
1, and `signing_keys` 1. `unclassified` was empty. The restore record had
no lineage object. The reconcile counts are the restored credentials the
policy lists. They are not a statement that those credentials were rotated.

```sh
riauth --config $LAB/restored/riauth.toml --non-interactive recovery status
```

Exit 0. Standard error was empty. Status is read-only
([recovery.rs](../../src/recovery.rs)). The result was policy
`riauth.recovery/v1`, backend `redb`, `initialized` true, `schema` 3,
`serving_allowed` false, and an empty history. Pending was the same recovery
id, cause, and counts, with `completed_at` null. Recorded lineage was null
and observed lineage was null. No system identifier was present.

`recovery complete`, `--persistent-credentials-reconciled`, `recovery
invalidate --database-restored`, and `recover-admin` were not run. A second
server was not started, and the restored store stayed closed. Free space on
the data volume stayed above 7 GiB. The lab, including the copy, the key,
the session, and the archive, was removed. After cleanup the snapshot hash
and mode were unchanged, port 9000 was free, and both default home session
files were absent.

### Unrun on this entry-point run

The executed chain is `riauth init`, serve, `/readyz`, server CLI login,
`keygen`, `backup`, stopping serve, `restore`, and `recovery status`.

Still unrun on this run:

- `cargo install`
- `riauth-maintenance` and `riauthctl`, including the printed maintenance
  names for init, keygen, restore, and recover-admin
- `riauth doctor`
- `recovery complete` and `--persistent-credentials-reconciled`
- `recovery invalidate`
- `recover-admin`
- a second `riauth serve`, including one on the restored configuration
- `--postgres-config` and any PostgreSQL target
- Section 4 and sections 6 through 14
- Signing-key operator steps, a browser, hardware, an external peer, and
  the Essentials guide

An earlier lab in the same hour completed this same command sequence. Its
evidence file was discarded before it was kept, and that directory was
removed. The numbers above are from the kept run. D01 remains incomplete.

## Section 4 passkey prompt

The docs worktree for this record is
`080452a45c97535fa858ddfd2d72148fbe4760ff`. No Cargo build was run.
`CARGO_TARGET_DIR` was unset. The authorized binary was the Platform server
snapshot
`/tmp/riauth-platform-58357fd-immutable/riauth`, the same file the catalog
run, the remote-administration run, and the section 5 run copied. Its mode
stayed `-r-x------`, its size stayed 289661864 bytes, its mtime stayed
`2026-09-29 17:00:59 +0200`, and its SHA-256 stayed
`de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069`. The
snapshot path was not executed. A copy lived in a new `mktemp` directory
under `/tmp`, mode `700`. The copy was chmod `700`, its SHA-256 matched the
snapshot before any command, and its inode differed. This page calls that
directory `$LAB`. The copy's real path is not recorded. `riauthctl` and
`riauth-maintenance` were not launched. `deployment-private/` was not written.

`riauth --version` printed `riauth 0.1.1`. Root help exited 0 with
`Usage: riauth [OPTIONS] <COMMAND>`. This run did not print
`riauth capabilities` or `riauth doctor`. The browser behavior below was
observed from that immutable snapshot. The passkey-specific functions
cited in the source review were compared with snapshot source `58357fd`
and were unchanged. The comparison is limited to those functions.
`src/portal/self_service/security.js` differs from that snapshot in its
session-revocation control. `src/cli.rs` also differs from that snapshot
outside the cited passkey functions.

Help was read on the copy:

| Command | Usage line |
| --- | --- |
| `passkey` | `Usage: riauth passkey [OPTIONS] <COMMAND>` |
| `passkey list` | `Usage: riauth passkey list [OPTIONS]` |
| `passkey remove` | `Usage: riauth passkey remove [OPTIONS] <ID>` |
| `passkey enroll` | `Usage: riauth passkey enroll [OPTIONS] --name <NAME>` |
| `passkey login` | `Usage: riauth passkey login [OPTIONS] <USERNAME>` |

`passkey --help` also lists `start` and `finish`. `enroll` is described as
enrollment with a connected USB FIDO2 authenticator. `login` is described as
authentication with a connected USB FIDO2 authenticator. `start` saves
public ceremony options for an external authenticator client. `finish`
completes a saved ceremony from a WebAuthn JSON response. `list` was the
only one of these verbs executed. The printed section 4 USB commands are
`riauthctl`, not these server verbs.

### Setup

Free space on the data volume stayed above 7 GiB. Port 9000 was free before
serve. Both default home session files were absent. The prerequisite was a
section 2 loopback on this copy:

```sh
riauth --config $LAB/riauth.toml --non-interactive init \
  --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 \
  --data-dir data \
  --admin admin \
  --password-stdin
```

Exit 0. Standard error was `Creating instance and signing key…`, 37 bytes.
Standard output had `initialized` true and issuer `http://localhost:9000`.
The configuration file was mode `600` and 430 bytes. It contained no
administrator password, no `database_key_file`, and no PostgreSQL block.
`$LAB/data/riauth.redb` was mode `600` and 1056768 bytes.

```sh
riauth --config $LAB/riauth.toml serve
```

`/readyz` returned HTTP 200:

```json
{"duties":{"authentication":true,"background_jobs":true,"protocol_listeners":true},"issuer":"http://localhost:9000","role":"integrated","service":"riAuth","status":"ok","version":"0.1.1"}
```

`protocol_listeners` true does not mean a listener stanza was configured.
At the last scan, after `passkey list`, the serve log was 3459 bytes. A
scan for the administrator password and for a PEM header found no hits. The
log contained the `background_overloaded` marker. The log text is not copied.

### Browser

Desktop interaction used the Cua driver 0.30.3. Accessibility and screen
recording were granted. The driver launched a new isolated Chromium profile
and copied no existing profile data. The page was `http://localhost:9000/apps`,
titled `Your applications · riAuth`.

Before sign-in, the page showed **Sign in with a passkey** in the viewport,
the password form, and **Sign in with your terminal**. The username field
read back as `admin`. The password value was not exposed in the page tree.
Submitting **Sign in** opened the catalogue. The account control was
**Signed in as admin (@admin)**. The notice was **Some applications need
extra verification. Add a passkey or an authenticator app under Sign-in and
security.** The catalogue heading was **All applications (0)**.

**Sign-in and security** showed **Sign in without a password. Adding or
removing a passkey signs you out everywhere.** The status was **You have no
passkeys yet.** The name field was prefilled `macOS` and then replaced with
`loopback-lab`. The visible hint was **Choose this device, another device
or a security key in your browser's passkey prompt.** A trusted browser
click on **Add a passkey** was refused before dispatch: that input route
will not foreground standalone Chromium on macOS. A page activation started
the ceremony, and the add controls became disabled.

Chrome then showed a sheet parented to the applications window. Its heading
was **Add a passkey?** Its text said that `localhost` supports passkeys and
that a passkey for `admin` would be saved in Passwords. A static label named
Touch ID as the save action. The sheet's own Cancel control was outside the
background click target. Escape on the sheet did not cancel the ceremony. It
opened a window titled **Choose where to save your passkey for localhost**,
with buttons **iCloud Keychain**, **Your Chrome profile**, **USB security
key**, and **Cancel**. Cancel on that chooser was pressed. The chooser
window disappeared.

The portal status then read `Adding the passkey was cancelled or timed out.
Select the add button to try again, or Cancel change.` The name field still
contained `loopback-lab`, the button was still **Add a passkey**, and no
passkey row appeared. **Rename** and **Remove** were therefore not
available. None of iCloud Keychain, the Chrome profile, Touch ID, or a USB
security key was selected.

Chrome also showed **Save password?** for the lab account. **Never** was
pressed. A following window list for that browser contained only the
applications window. The Cua session was ended, and the isolated browser
process was gone before the lab was removed.

### Empty passkey list

The browser session was not reused for the CLI. A separate server CLI login
wrote `$LAB/cli-session.json`, mode `600`. Standard error was empty. The
password was absent from standard output and standard error, and standard
output did not contain a session token. The account fields were `admin`
true, `email_verified` false, `enabled` true, `id`
`5c290300-cf0d-41b1-9f86-3f83691e1118`, `mfa_enabled` false, and `username`
`admin`.

```sh
riauth --config $LAB/riauth.toml \
  --server http://localhost:9000 \
  --session-file $LAB/cli-session.json \
  --non-interactive \
  passkey list
```

Exit 0. Standard error was empty. Standard output was `[]`. A repeat of
that list immediately before shutdown was empty as well. Both default home
session files stayed absent.

### Source review

Section 4's printed commands and the portal passkey API match this tree.
The comparison below was read from source. The USB bail strings were not
executed on the snapshot.

The portal page uses `GET /api/portal/passkeys`,
`POST /api/portal/passkeys/registration/start` with `name` and
`expected_user_id`, and finish and cancel routes under that registration
prefix. Rename is `POST /api/portal/passkeys/{id}/rename`. Remove is
`POST /api/portal/passkeys/{id}/remove`. Those handlers live in
[portal/http.rs](../../src/portal/http.rs). Browser start calls
`passkey_register_start_in` with the resident flag true
([portal.rs](../../src/portal.rs)). The resulting authenticator selection
is `residentKey` `required`, `requireResidentKey` true, and user
verification required ([assembly/passkey.rs](../../src/assembly/passkey.rs)).
The flag is advisory: the stored ceremony is unchanged, and a non-resident
key can still enroll. The relying party id and origin come from the issuer
URL ([passkey.rs](../../src/passkey.rs)). For `http://localhost:9000` that
is hostname `localhost` and origin `http://localhost:9000`. The Chrome
sheet named `localhost`. This run did not decode the ceremony JSON.

Server CLI `passkey list` is `GET /api/passkeys`. Remove is `DELETE` and
rename is `PATCH` on `/api/passkeys/{id}` ([api.rs](../../src/api.rs)).
CLI registration start posts `{name}` and passes the resident flag false.
`passkey enroll` and `passkey login` call `require_support` before a
ceremony ([cli.rs](../../src/cli.rs)). That function always stops with the
message that terminal USB passkeys moved to `riauthctl`
([cli/usb.rs](../../src/cli/usb.rs)). The base `riauthctl` stops with the
message that USB passkeys are unavailable until it is rebuilt with
`--features terminal-usb` ([riauthctl usb.rs](../../crates/riauthctl/src/usb.rs)).
With that feature, `--non-interactive` stops because touch and PIN input
are required ([riauthctl main.rs](../../crates/riauthctl/src/main.rs)).
The printed commands are `passkey enroll --name security-key` and
`passkey login admin`. Both match that clap shape. Neither command was run,
and `cargo install` was not run.

A factor change is allowed through 300 seconds after `auth_time`
([signin.rs](../../src/signin.rs)). Once the account has a passkey or an
authenticator app, the session also has to be an MFA session
([identity.rs](../../src/identity.rs)). Adding or removing a passkey sets
`sessions_revoked` true. Rename does not. Removing the last passkey of an
account with an empty password hash conflicts. An administrator in that
state must keep two passkeys. The guide sentence about the setup
administrator remains true. The same conflict covers any later
administrator whose password hash is empty. This lab account had a password
and no passkeys, so those conflicts were not reached.

**Sign in with a passkey** is shown when `identity.passkeys` is usable and
`PublicKeyCredential` is available in a secure context
([app.js](../../src/portal/app.js), [auth.js](../../src/portal/auth.js)).
The button was visible on the first unsigned visit. Step 6's later-visit
wording is the guide's sequence. The page condition does not require an
existing passkey. The observed cancellation text is the page's
`NotAllowedError` or `AbortError` status.

### Cleanup

SIGTERM stopped serve, and SIGKILL was not required. The lab, including the
copy, the password file, the session, and the redb file, was removed. After
cleanup the snapshot hash and mode were unchanged, port 9000 was free, and
both default home session files were absent. Free space on the data volume
stayed above 7 GiB.

### Unrun on this passkey attempt

The executed chain is `riauth init`, serve, `/readyz`, password sign-in at
`/apps`, opening **Sign-in and security**, starting **Add a passkey**,
cancelling the Chrome chooser, server CLI login, and `passkey list`.

Still unrun on this run:

- storing a passkey, **Rename**, **Remove**, and **Sign in with a passkey**
  after enrollment
- iCloud Keychain, the Chrome profile, Touch ID, and a USB security key
- `cargo install`
- `riauthctl` and `riauth-maintenance`, including the printed
  `passkey enroll` and `passkey login` lines
- server `passkey enroll`, `passkey login`, `passkey start`,
  `passkey finish`, and `passkey remove`
- `riauth doctor`, `recovery complete`, `recover-admin`, and a second server
- sections 6 through 14
- a physical key, a synced passkey, a phone, a spoken screen reader, an
  external peer, and the Essentials guide

D01 remains incomplete.

## Section 14 invitation password

The docs worktree for this record is
`0791deb4737326c1bc2157a48a636decc1117a69`. No Cargo build was run.
`CARGO_TARGET_DIR` was unset. The authorized binary was the Platform server
snapshot
`/tmp/riauth-platform-58357fd-immutable/riauth`, the same file the catalog
run, the remote-administration run, the section 5 run, and the section 4 run
copied. Its mode stayed `-r-x------`, its size stayed 289661864 bytes, its
mtime stayed `2026-09-29 17:00:59 +0200`, and its SHA-256 stayed
`de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069`. The
snapshot path was not executed. A copy lived in a new `mktemp` directory
under `/tmp`, mode `700`. The copy was chmod `700`, its SHA-256 matched the
snapshot before any command, and its inode differed. This page calls that
directory `$LAB`. The copy's real path is not recorded. `riauthctl` and
`riauth-maintenance` were not launched. `deployment-private/` was not written.

`riauth --version` printed `riauth 0.1.1`. Standard error was empty. This
run did not print `riauth capabilities` or `riauth doctor`. The browser
behavior below was observed from that immutable snapshot.

### Setup and mail

Free space on the data volume stayed above 7 GiB. Port 9000 was free before
serve. Both default home session files were absent. The administrator
password and the invited person's password were different 24-character
alphanumeric values in mode `600` files. Neither value is recorded.

```sh
riauth --config $LAB/riauth.toml --non-interactive init \
  --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 \
  --data-dir $LAB/data \
  --admin admin \
  --password-stdin
```

Exit 0. Standard error was `Creating instance and signing key…`, 37 bytes,
including the newline. The configuration file was mode `600`. It contained
no administrator password. After `init`, and before `serve`, the lab appended
this table. The port was the ephemeral port of a sink bound to `127.0.0.1`.
On this run that port was `65192`. The table had no `username` and no
`password_file`.

```toml
[mail]
host = "127.0.0.1"
port = 65192
from = "Identity <identity@example.test>"
security = "loopback"
```

```sh
riauth --config $LAB/riauth.toml serve
```

`/healthz` succeeded through `curl --fail`. The body was 187 bytes. This run
did not request `/readyz`. `$LAB/data` held one redb file of 118784 bytes.

```sh
riauth --config $LAB/riauth.toml --session-file $LAB/admin.session \
  login admin --password-stdin
```

Exit 0. The session file was mode `600`. Standard output named user `admin`
and did not contain a session token. The login object is not copied because
it contains the lab path.

The invitation file was mode `600`:

```json
{"username": "invitee", "email": "invitee@example.test", "display_name": "Invitee Lab", "groups": []}
```

`groups` was empty, so the invite did not require a group. The same file was
used for both invite attempts.

```sh
riauth --config $LAB/riauth.toml --session-file $LAB/admin.session \
  account invite --file $LAB/invite.json
```

Exit 1. Standard output was 0 bytes. Standard error matched this text,
including its trailing newline. The source string wraps the revision command
in backticks:

```text
error: Invitation writes require --idempotency-key and --if-revision (from `riauth revision`)
```

```sh
riauth --config $LAB/riauth.toml --session-file $LAB/admin.session revision
```

Exit 0. Standard error was 0 bytes. Standard output was a JSON object whose
only field was `revision` with value 0. `--json` was not passed.

`$KEY` below was one UUID in a mode `600` file. The value is not recorded.
The bound command is the one that issued the invitation:

```sh
riauth --config $LAB/riauth.toml --session-file $LAB/admin.session \
  --idempotency-key "$KEY" \
  --if-revision 0 \
  account invite --file $LAB/invite.json
```

Exit 0. `delivery_queued` was true. The returned account was `invitee`,
display name `Invitee Lab`, `enabled` false, `admin` false,
`password_available` false, `email_verified` false, and `mfa_enabled` false.
The address domain was `example.test`. The user id is not recorded. The
response contained no invitation token.

The product mail job delivered one message to the loopback sink during the
wait of up to 45 seconds. The fixture pump was not used. The captured
message was mode `600` and stayed in the lab. Its subject was `Your riAuth
account invitation`. The body contained `Account: invitee`. The link's
origin and path were `http://localhost:9000/account/accept`, the fragment
token started with `ri_mail_`, and that token was 51 characters. The link
and the message body are not recorded.

```sh
riauth --config $LAB/riauth.toml --session-file $LAB/admin.session \
  account deliveries
```

Exit 0. The result was one list row. `delivered_at` was set, `attempts` was
1, and `stopped` was false. The row had no body, recipient, or subject
field.

At that point the serve log was 433 bytes. A scan of the configuration, the
serve log, the health body, and the revision, invite, deliveries, login,
init, and unbound outputs found no password, no invitation token, and no
PEM header. The log contained the
`background_overloaded` marker once. The log text is not copied.

### Browser

Desktop interaction used the Cua driver 0.30.3. Accessibility and screen
recording were granted. The driver launched a new isolated Chromium profile
and copied no existing profile data. The page was the captured link. After
the page script ran, the address was `http://localhost:9000/account/accept`
with no fragment. The title was `Accept invitation · riAuth`.

The description was `Set a password, or add a passkey, to activate your
account. You will sign in after accepting the invitation.` The password
form, a **Passkey name** field, and **Accept with a passkey** were visible.
The password and confirmation each received 24 masked characters. The page
tree did not expose the password value. **Accept with a passkey** was not
used. Activating **Accept invitation** did not return a confirmed input
effect from the driver. The next snapshot was the proof.

The page showed **Invitation accepted**, **Your account is ready**, and
`Sign in with your new password to open your applications. An application
may also require a passkey or authenticator code.` The link was **Continue
to sign in**. The password form and the passkey form were not in the visible
tree.

```sh
riauth --config $LAB/riauth.toml --session-file $LAB/admin.session user list
```

Exit 0. Standard error was empty. The list had two accounts. `invitee` was
`admin` false, display name `Invitee Lab`, `email_verified` true, `enabled`
true, `mfa_enabled` false, and `password_available` true. The address domain
was `example.test`.

Chrome showed a window titled **Save password?**. **Never** was pressed.
That window was gone afterward. **Save** was not pressed. A later window
list did not show a second **Save password?** prompt. This record does not
say that **Never** suppresses a later prompt.

`http://localhost:9000/apps` then showed `Sign in to see the applications
available to you. Your workspace is ready when you are.` The buttons were
**Sign in with a passkey**, **Sign in**, and **Sign in with your terminal**.
The signed-in welcome was absent. **Sign in with a passkey** was not used.

The same captured link was opened again. The password form was visible
again. A second password submit was required before the page changed. The
alert was `This invitation has already been accepted. Continue to sign in.`
The password form and the passkey form were not in the visible tree. **Open
applications** was visible. The passkey-or-password description remained.
`/apps` still showed the same sign-in panel.

The username field then read back as `invitee`. The password field received
24 masked characters. Activating **Sign in** did not return a confirmed
input effect. The next snapshot showed the catalogue. The title was `Your
applications · riAuth`. The account control was **Signed in as Invitee Lab
(@invitee)**. The welcome text was **Welcome back, Invitee Lab. Find your
next starting point.** The page also showed `Some applications need extra
verification. Add a passkey or an authenticator app under Sign-in and
security.` The headings were **Your applications.** and **All applications
(0)**. The empty-catalogue text was `No applications are available for this
account yet. Contact your administrator if you’re expecting access.` The
status was `0 applications shown.` **Sign out**, **Sign-in and security**,
and **Sessions and consent** were visible and were not used.

The success JSON was not read. The session cookie was not read. An in-page
`GET /api/portal` was refused by the driver before it ran, so this record
has no portal HTTP status and no cookie-jar result. The signed-out panel
after acceptance, and again after the replay, is the evidence that the
browser had no session until the later password sign-in.

A final scan before shutdown, still on the running copy, printed `riauth
0.1.1` with empty standard error. The serve log was 1034 bytes. That scan
found no password, no invitation token, and no PEM header. The
`background_overloaded` marker occurred three times.
The redb file was still 118784 bytes. The copy was still mode `700`, its
SHA-256 still matched the snapshot, and its inode still differed.

### Source comparison

The comparison is `git diff 58357fd 0791deb`, limited to the files named
here. These files had no difference:

- `src/lifecycle.rs`
- `src/lifecycle/invitation.rs`
- `src/lifecycle/invitation/passkey.rs`
- `src/portal/account.js`
- `src/portal/account.html`
- `src/portal/http.rs`
- `src/api/invitation.rs`
- `src/api/server.rs`
- `src/config.rs`
- `src/background.rs`
- `src/workflow/executor/invitation.rs`
- `src/crypto.rs`
- `src/signin.rs`
- `src/portal/index.html`

`src/cli.rs` differs in the agent-creation and agent-revocation idempotency
checks. The invitation command was outside that diff. `src/management.rs`
differs in agent issuance receipt handling and the already-revoked check.
`invite_user` was outside that diff. This comparison does not cover the rest
of the tree. The password page was served by the immutable snapshot. The
name `platform-invitation-password-enrollment` was not started as an
operator workflow run.

### Cleanup

The Cua session was ended, and the isolated browser process was gone before
the lab was removed. The lab shell's cleanup stopped `serve` with SIGTERM.
SIGKILL was not required. It stopped the SMTP sink, re-hashed the snapshot,
found port 9000 free, found both default home session files absent, and
removed the lab directory. A following check found the directory gone, the
snapshot hash and mode unchanged, port 9000 free, and both home session
files absent. Free space on the data volume stayed above 7 GiB. After
cleanup, available space was 14006576 KiB.

### Unrun on this invitation run

The executed chain is `riauth init`, a loopback `[mail]` table, `serve`,
`/healthz`, server CLI login, an unbound `account invite`, `revision`, one
bound `account invite`, one captured message, `account deliveries`, password
acceptance in an isolated browser, `user list`, a signed-out `/apps` visit,
a replay of the same link, and a later password sign-in as `invitee`.

Still unrun on this run:

- `tools/browser/invitation-password.spec.js`,
  `invitation-passkey.spec.js`, and `invitation-passkey-shim.spec.js`
- Firefox, WebKit, and the CI browser job
- **Accept with a passkey**, **Sign in with a passkey**, and **Sign out**
- an expired, revoked, or replaced invitation link
- `account revoke-invitation` and `riauth account accept`
- a second invite, and any retry of the same idempotency key
- `GET /api/portal`, the session cookie, and the accept response JSON
- a real mailbox, and any SMTP host other than `127.0.0.1`
- `cargo install`
- `riauthctl` and `riauth-maintenance`
- `riauth doctor`, `riauth capabilities`, and `/readyz`
- `recovery complete`, `recover-admin`, and a second server
- sections 9 through 13
- a physical key, a synced passkey, a phone, a spoken screen reader, an
  external peer, and the Essentials guide

D01 remains incomplete.

## Section 11 configured password workflow

The docs worktree for this record is
`0add90f9febc5b00bbc37aed7099023ac1743737`. No Cargo build was run.
`CARGO_TARGET_DIR` was unset. The authorized binary was the Platform server
snapshot
`/tmp/riauth-platform-58357fd-immutable/riauth`, the same file the catalog
run, the remote-administration run, the section 5 run, the section 4 run,
and the section 14 run copied. Its mode stayed `-r-x------`, its size stayed
289661864 bytes, its mtime stayed `2026-09-29 17:00:59 +0200`, and its
SHA-256 stayed
`de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069`. The
snapshot path was not executed. A copy lived in a new `mktemp` directory
under `/tmp`, mode `700`. The copy was an APFS clone, then chmod `700`. Its
SHA-256 matched the snapshot before any command, and its inode differed.
This page calls that directory `$LAB`. The copy's real path is not recorded.
`riauthctl` and `riauth-maintenance` were not launched.
`deployment-private/` was not written. This run did not open a browser.

The printed guide lines use
`deployment-private/platform-lab/`. This run passed the same flags with
`$LAB` paths. Remote commands also passed `--config $LAB/riauth.toml` and
`--session-file $LAB/admin.session`. The default config path is `riauth.toml`
in the current directory, and the default server-CLI session is the home
session file. Neither home session file was written. No command passed
`--json`, `--idempotency-key`, `--if-revision`, or `--confirm-removals`.

### Setup

Free space on the data volume was 16633252 KiB before the copy and stayed
above 7 GiB. Port 9000 was free before serve. Both default home session
files were absent. The administrator password was 24 alphanumeric bytes in a
mode `600` file. The password is not recorded.

```sh
riauth --config $LAB/riauth.toml --non-interactive init \
  --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 \
  --data-dir $LAB/data \
  --admin admin \
  --password-stdin
```

Exit 0. Standard error was `Creating instance and signing key…`, 37 bytes,
including the newline. The configuration file was mode `600`. Standard
output said `initialized` true and issuer `http://localhost:9000`. That
object is not copied because it names the config path. The file contained
no administrator password and no `[mail]` table.

```sh
riauth --config $LAB/riauth.toml serve
```

`/readyz` on `127.0.0.1:9000` returned status `ok`, role `integrated`, and
version `0.1.1`. This readiness request is not one of the printed section 11
commands. `$LAB/data` held one redb file. After the later restart and
export it was 110592 bytes.

```sh
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/admin.session login admin --password-stdin
```

Exit 0. The session file was mode `600`. Standard output named user `admin`
and did not contain a session token. Standard error was empty. The login
object is not copied because it names the session path.

```sh
riauth --config $LAB/riauth.toml capabilities
riauth --version
```

`capabilities` exited 0. It is local, and the printed section 11 commands do
not include it. The catalog `schema_version` was `riauth.capabilities/v2`,
`edition` was `platform`, `interface` was `server`, `version` was `0.1.1`,
and `build_features` were `essentials` and `platform`. The target was
`macos` / `aarch64`. The schema list had 29 names, including `workflow` and
`manifest`. The catalog object was 20679 bytes and is not copied.
`riauth --version` printed `riauth 0.1.1`. Its standard error was empty.

### Schema, validate, plan, and apply

The authoring file was the printed `local-password` manifest, mode `600`,
with issuer `http://localhost:9000`. It had no `active` field.

```sh
riauth --config $LAB/riauth.toml schema workflow
riauth --config $LAB/riauth.toml schema manifest
riauth --config $LAB/riauth.toml validate --file $LAB/local-password.json
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/admin.session plan \
  --file $LAB/local-password.json \
  --out $LAB/local-password-plan.json
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/admin.session apply \
  --plan $LAB/local-password-plan.json
```

`schema workflow` exited 0. Standard error was empty. The document was
15785 bytes and its title was `Definition`. `schema manifest` exited 0.
Standard error was empty. The document was 59679 bytes and its title was
`Manifest`. Neither schema is copied here.

`validate` exited 0. Standard error was empty. It printed `valid` true,
`validation` `local_schema`, `resources` 0, and `secret_values_read` false.

`plan` exited 0. Standard error was empty. The plan file was mode `600`.
Standard output named the plan file, so that object is not copied. The
retained fields were:

| Field | Observed value |
| --- | --- |
| `plan_id` | `2cc4d8e3-5758-4094-ade0-8aaaf4c29447` |
| `base_revision` | 0 |
| `changes` | one `create` for `workflow/local-password` |
| `credential_change` | false |
| secret references | none |
| `removal_impact.review_required` | false |
| `disabled_users` | 0 |
| `missing_users` | 0 |
| `removed_memberships` | 0 |
| `reconciliation_mode` | `manual-review` |
| `expires_at` | `1790701860` |

`review_required` was false, so `--confirm-removals` was not added. The
extractor kept `hash` only when it was 64 lowercase hexadecimal characters.
The observed value failed that check, so this record does not contain it.
Snapshot-equal `src/cli.rs` includes `hash` in the plan stdout object, and
snapshot-equal `src/state.rs` assigns it with `digest`. Snapshot-equal
`src/crypto.rs` defines `digest` as URL-safe unpadded base64 of SHA-256.
That encoding is separate from the binding fingerprint below.

`apply` exited 0. Standard error was empty. It printed `applied` true,
`changed` true, `revision` 0, and the same plan id. `run_id` was absent.
The one change was again `create` for `workflow/local-password`.

### Runtime restart and configured start

SIGTERM stopped the first `serve`. SIGKILL was not required. The printed
`[workflows.local-password]` table was then appended, including
`active = true` and definition id `local-password`. The configuration stayed
mode `600` and still had no `[mail]` table.

```sh
riauth --config $LAB/riauth.toml serve
```

The second `/readyz` returned status `ok`. The body was 187 bytes.

The start route has no CLI wrapper. The request was
`POST http://localhost:9000/api/workflows/configured/local-password` with an
empty body and one `Authorization` header. The scheme was bearer. The token
was read from `$LAB/admin.session` and is not recorded.

HTTP status was 200. The response set no `Set-Cookie` header. The body did
not contain the session token. The public view fields were:

| Field | Observed value |
| --- | --- |
| `id` | `00453fc4-2cb5-4db2-b2b9-010368c95596` |
| `binding.workflow` | `local-password` |
| `binding.revision` | 1 |
| `binding.fingerprint` | `0fbbab5fadf1019195b415112725a6fb62f794c0e2c41e8ea7a2fc22a8865f61` |
| `binding.source_registration` | absent |
| `binding.extension_sha256` | absent |
| `state` | active, step `password`, attempt 1 |
| `started_at` | `1790700961` |
| `expires_at` | `1790701561` |
| `attempts_used` | 0 |
| `max_attempts` | 3 |
| `executions` | 0 |
| `reviewed_revision` | 1 |

`expires_at` is 600 seconds after `started_at`, which is the definition's
`max_duration_seconds`. The object also included `reviewed_policy` and
`step_started_at`. Those values are not copied. `authorization_response`,
`credential_epoch`, and `reviewed_failure` were absent. No evidence
reference was present.

```sh
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/admin.session export \
  --out $LAB/state-export.json
```

Exit 0. Standard error was empty. The export file was mode `600`. Standard
output said `secrets_included` false and `revision` 0, and it named the
manifest file, so that object is not copied. The manifest `api_version` was
`riauth/v1`. Its `issuer` was null. It contained one workflow,
`local-password` at revision 1, and one user record. The user record is not
copied. The workflow definition persisted; `meta.revision` stayed 0, the
same value `plan` printed as `base_revision`.

The serve log was 0 bytes. A scan of the configuration, the serve log, both
readiness bodies, the schema output, validate, plan, apply, export, the
start body, the capabilities output, login and init output, the export
file, the authoring file, and the plan file found no password, no session
token, and no PEM header. The session file and the password
file were outside that scan.

### Source comparison

The comparison is `git diff 58357fd 0add90f`, limited to the files named
here. These files had no difference:

- `src/workflow.rs`
- `src/workflow/validate.rs`
- `src/schema.rs`
- `src/api/workflow.rs`
- `src/crypto.rs`
- `src/core.rs`

`src/api.rs` adds `GET /api/operations/provisioning/deactivations`. This run
did not call it. `src/cli.rs` differs in the agent create, rotate, and
revoke idempotency checks. Schema, validate, plan, apply, export, and login
are outside that diff. `src/config.rs` differs in the browser-consent
adapter check, which also accepts a password-plus-TOTP consent workflow.
The password-only `local-password` block is outside that diff.
`src/state.rs` differs in client-description desired-state dependency scope.
The workflow reconcile that writes `workflow_definitions` is outside that
diff, and this manifest did not change a client description.
`src/workflow/executor.rs` differs in TOTP browser-consent imports.
`workflow_configured_start` is outside that diff. This comparison does not
cover the rest of the tree. The commands above were served by the immutable
snapshot. `platform-invitation-password-enrollment` was not configured.

In that same snapshot-equal `src/core.rs`, a revision bump follows audit
actions whose names start with a fixed set of prefixes. `workflow.` is not
one of those prefixes. The observed apply revision and export revision were
both 0 after the workflow create.

### Cleanup

The evidence script exited after the plan-hash shape check failed closed.
The product commands above had already exited 0. The script's cleanup then
stopped `serve` with SIGTERM. SIGKILL was not required. It re-hashed the
snapshot, found port 9000 free, found both default home session files
absent, and removed the lab directory. The remover's own exit code was 0.
A following check found the directory gone, the snapshot hash and mode
unchanged, port 9000 free, and both home session files absent. Free space
on the data volume stayed above 7 GiB. After cleanup, available space was
16601424 KiB.

### Unrun on this configured-workflow run

The executed chain is `riauth init`, `serve`, `/readyz`, server CLI login,
local `capabilities` and `--version`, `schema workflow`, `schema manifest`,
`validate`, `plan`, `apply`, a SIGTERM restart after appending
`[workflows.local-password]`, a second `/readyz`,
`POST /api/workflows/configured/local-password`, and `export`.

Still unrun on this run:

- `POST /api/workflows/{id}/password`, `GET /api/workflows/{id}`, and
  `POST /api/workflows/{id}/cancel`
- submitting the password for step `password`
- the browser authoring routes under `/api/admin/workflows`
- `--confirm-removals`
- an audit read of the apply or the workflow start
- TOTP, recovery-code, passkey, consent, and enrollment workflow shapes
- `platform-password-totp-reauthentication`,
  `platform-invitation-password-enrollment`,
  `platform-source-reauthentication`, and
  `platform-source-totp-reauthentication`
- sections 9, 10, 12, and 13
- `cargo install`
- `riauthctl` and `riauth-maintenance`
- `riauth doctor`
- `recovery complete`, `recover-admin`, and a second server
- a browser, a physical key, a synced passkey, a phone, a spoken screen
  reader, an external peer, and the Essentials guide

D01 remains incomplete.

## Section 9 LDAP directory import

The docs worktree for this record is
`11f1f8eaeaf23008b94767bcdd310bd46501c182`. No Cargo build was run.
`CARGO_TARGET_DIR` was unset. The authorized binary was the Platform server
snapshot
`/tmp/riauth-platform-58357fd-immutable/riauth`, the same file the catalog
run, the remote-administration run, the section 5 run, the section 4 run,
the section 14 run, and the section 11 run copied. Its mode stayed
`-r-x------`, its size stayed 289661864 bytes, its mtime stayed
`2026-09-29 17:00:59 +0200`, and its SHA-256 stayed
`de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069`. The
snapshot path was not executed. A copy lived in a new `mktemp` directory
under `/tmp`, mode `700`. The copy was an APFS clone, then chmod `700`. Its
SHA-256 matched the snapshot before any command, and its inode differed.
This page calls that directory `$LAB`. The copy's real path is not recorded.
`riauthctl` and `riauth-maintenance` were not launched.
`deployment-private/` was not written. This run did not open a browser.
`scripts/test-ldap.sh` was not run.

The printed guide lines use `deployment-private/platform-lab/`. This run
passed the same flags with `$LAB` paths. Remote commands also passed
`--config $LAB/riauth.toml` and `--session-file $LAB/admin.session`. The
default config path is `riauth.toml` in the current directory, and the
default server-CLI session is the home session file. Neither home session
file was written. No command passed `--json`.

### Setup

Free space on the data volume was 10329524 KiB before the copy and stayed
above 7 GiB. After the clone it was 10331176 KiB. Port 9000 was free before
serve. Both default home session files were absent. The administrator
password and the LDAP bind password were separate 24-character alphanumeric
files, mode `600`, with no trailing newline. Neither password is recorded.

```sh
riauth --config $LAB/riauth.toml --non-interactive init \
  --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 \
  --data-dir $LAB/data \
  --admin admin \
  --password-stdin
```

Exit 0. Standard error was `Creating instance and signing key…`, 37 bytes,
including the newline. The configuration file was mode `600`. Standard
output said `initialized` true and issuer `http://localhost:9000`. That
object is not copied because it names the config path. The file contained
no administrator password and no `[mail]` table. Its table headers were
`[proxy_listeners]`, `[radius_listeners]`, `[ldap_listeners]`,
`[directories]`, `[workspace_directories]`, `[entra_directories]`,
`[scim_targets]`, and `[signers]`. Each of those tables was empty.
`[directories.staff]` was absent. There was no `[workflows]` table.

```sh
riauth --version
riauth --config $LAB/riauth.toml capabilities
```

`riauth --version` printed `riauth 0.1.1`. Its standard error was empty.
`capabilities` exited 0. It is local, and the printed section 9 commands do
not include it. The catalog `schema_version` was `riauth.capabilities/v2`,
`edition` was `platform`, `interface` was `server`, `version` was `0.1.1`,
and `build_features` were `essentials` and `platform`. The target was
`macos` / `aarch64`. The schema list had 29 names. The catalog object was
20679 bytes and is not copied.

```sh
riauth --config $LAB/riauth.toml serve
```

`/readyz` on `127.0.0.1:9000` returned status `ok`, role `integrated`, and
version `0.1.1`. The body was 187 bytes. This readiness request is not one
of the printed section 9 commands.

```sh
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/admin.session login admin --password-stdin
```

Exit 0. The session file was mode `600`. Standard output named user `admin`,
with `admin` true. It did not contain a session token. Standard error was
empty. The login object is not copied because it names the session path.

`riauth revision` printed revision 0.

```sh
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/admin.session \
  --idempotency-key section9-staff \
  --if-revision 0 \
  group create staff
```

Exit 0. Standard error was empty. The group name was `staff` and its member
count was 0. The next `riauth revision` printed revision 1. Section 6's
`group add-member` was not run. `admin` was not added to `staff`.

### Directory fixture

SIGTERM stopped the first `serve`. SIGKILL was not required. Port 9000 was
free before the configuration edit. The printed `[directories.staff]` block
was then appended:

```toml
[directories.staff]
url = "ldap://127.0.0.1:60523"
transport = "starttls"
bind_dn = "cn=riauth,ou=services,dc=example,dc=test"
password_file = "ldap-password"
ca_file = "ca.crt"
user_base = "ou=people,dc=example,dc=test"
user_filter = "(objectClass=inetOrgPerson)"
id_attribute = "entryUUID"
username_attribute = "uid"
display_attribute = "cn"
email_attribute = "mail"

[directories.staff.group_user_filters]
staff = "(memberOf=cn=staff,ou=groups,dc=example,dc=test)"
```

`password_file` and `ca_file` are relative to the configuration file. The
bind password file was mode `600`. `ca.crt` was mode `600`. The
configuration stayed mode `600`.

The listener was Homebrew OpenLDAP `slapd` 2.7.1,
`@(#) $OpenLDAP: slapd 2.7.1 (Sep  8 2026 21:55:18) $`, at
`/opt/homebrew/opt/openldap/libexec/slapd`. `nm` reported 0 exported
`memberof` symbols. The Cellar contained the man page `slapo-memberof.5` and
no memberof module file. `strings` on that `slapd` includes `memberof.c`, and
the binary already defines operational attribute `memberOf` with
`NO-USER-MODIFICATION`. The fixture `slapd.conf` loaded `overlay memberof`
after the `mdb` database. Schema files were `core`, `cosine`, and
`inetorgperson`. The suffix was `dc=example,dc=test`. The root DN was the
printed bind DN. `rootpw` was a `slappasswd -n -T` hash of the bind password
file. The hash and the password are not recorded. `slapd.conf` and the TLS
key were mode `600`. There was no person entry for the bind DN. `ou=services`
was present.

The certificate was a one-day RSA 2048 CA, CN `localhost`, `CA:TRUE`, SAN
`DNS:localhost` and `IP:127.0.0.1`, key usage `keyCertSign,cRLSign`. The leaf
was RSA 2048, CN `localhost`, the same SAN, `CA:FALSE`, key usage
`digitalSignature,keyEncipherment`, extended key usage `serverAuth`. No PEM
from either file is copied.

`slapd` listened on `ldap://127.0.0.1:60523`. Readiness was a plaintext
anonymous base search. The population client was plaintext `ldapadd` to that
same URL, with the bind password read from the file. The LDIF added
`uid=alice,ou=people,dc=example,dc=test` as `inetOrgPerson` with `uid`
`alice`, `cn` `Alice Example`, `sn` `Example`, and `mail`
`alice@example.test`. It set no `userPassword` and no `memberOf`. It then
added `cn=staff,ou=groups,dc=example,dc=test` as `groupOfNames` with member
`uid=alice,ou=people,dc=example,dc=test`. `ldapadd` exited 0. A following
plaintext search showed `memberOf` equal to
`cn=staff,ou=groups,dc=example,dc=test` and showed `entryUUID`. The UUID
value is not copied.

An OpenSSL `s_client -starttls ldap` probe against that listener exited 0.
It reported `Verification: OK` and `Protocol version: TLSv1.3`. Apple
`/usr/bin/ldapsearch` 2.4.28, with `-ZZ` and the same CA, exited 1 and
printed `ldap_start_tls: Connect error (-11)`. That client did not complete
STARTTLS. The probe is not a printed section 9 command. The riAuth commands
below used the appended `transport` `starttls` configuration.

```sh
riauth --config $LAB/riauth.toml serve
```

The second `/readyz` returned status `ok`. The body was 187 bytes.

### List, plan, and apply

```sh
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/admin.session directory list
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/admin.session directory plan staff \
  --out $LAB/ldap-plan.json
```

`directory list` exited 0. Standard error was empty. The one row was:

| Field | Observed value |
| --- | --- |
| `id` | `staff` |
| `url` | `ldap://127.0.0.1:60523` |
| `user_base` | `ou=people,dc=example,dc=test` |
| `groups` | `staff` |
| `reconciliation_mode` | `manual-review` |

`directory plan` exited 0. Standard error was empty. The plan file was mode
`600`. Standard output named the plan file, so that object is not copied.
The retained stdout fields were id
`3f9ffb68-406f-472a-8d78-69116899b0f0`, revision 1, and one `create` for
`alice` with group `staff`. `removal_impact` was absent from that stdout
object. `riauth revision` after plan still printed revision 1.

The plan file held the same id, directory `staff`, revision 1, `expires_at`
`1790702694`, and `applied` false. It had one entry and the same change.
The entry's username was `alice`, display name `Alice Example`, email
`alice@example.test`, and group `staff`. `removal_impact` was:

| Field | Observed value |
| --- | --- |
| `disabled_users` | 0 |
| `missing_users` | 0 |
| `removed_memberships` | 0 |
| `review_required` | false |

`fingerprint` and `actor` were present. They are not copied. Distinguished
names and `entryUUID` values from the plan are not copied.
`review_required` was false, so apply omitted `--confirm-removals`.

Before apply, `user list` contained only `admin`. `group list` contained
`staff` with member count 0.

```sh
riauth --config $LAB/riauth.toml --server http://localhost:9000 \
  --session-file $LAB/admin.session directory apply \
  --plan $LAB/ldap-plan.json
```

Exit 0. Standard error was empty. The object had `applied` true, the same
plan id, and the same one `create` for `alice` in group `staff`. It had no
`decision` field. `riauth revision` then printed revision 4.

`user list` after apply contained `admin` and `alice`. `admin` stayed
`password_available` true. `alice` was:

| Field | Observed value |
| --- | --- |
| `username` | `alice` |
| `display_name` | `Alice Example` |
| `email` | `alice@example.test` |
| `enabled` | true |
| `admin` | false |
| `mfa_enabled` | false |
| `password_available` | false |
| `email_verified` | false |
| attributes | 0 |
| subjects | 0 |

User ids and `created_at` are not copied. `group list` showed `staff` with
member count 1. That member was the imported `alice` account. Group members
are stored as user ids, and the id is not copied.

`$LAB/data/riauth.redb` was 110592 bytes. The serve log was 683 bytes. The
serve log, the `slapd` log, and the configuration file did not contain the
administrator password or the bind password. The session file and both
password files were outside that scan.

### Source comparison

The comparison is `git diff 58357fd 11f1f8e`, limited to the files named
here. These files had no difference:

- `src/directory.rs`
- `src/assembly/directory.rs`
- `src/connector_guard.rs`
- `src/core.rs`
- `src/crypto.rs`
- `src/model.rs`

`src/api.rs` adds `GET /api/operations/provisioning/deactivations`. This run
did not call it. `src/cli.rs` differs in the agent create, rotate, and
revoke idempotency checks. `directory list`, `directory plan`,
`directory apply`, `group create`, `user list`, `group list`, `revision`,
and `login` are outside that diff. `src/config.rs` differs in the
browser-consent adapter check, which also accepts a password-plus-TOTP
consent workflow. Directory `password_file` and `ca_file` resolution is
outside that diff. This comparison does not cover the rest of the tree. The
commands above were served by the immutable snapshot.

In that same snapshot-equal `src/core.rs`, a revision bump follows audit
actions whose names start with `user.`, `group.`, or `directory.apply`,
among other prefixes. `directory.plan` is not one of those prefixes. The
observed revision stayed 1 across plan and was 4 after apply. No audit
command was run, so the audit rows were not read.

### Cleanup

The evidence script exited 0 after the product commands above exited 0. Its
cleanup then stopped `serve` and `slapd` with SIGTERM. SIGKILL was not
required. It re-hashed the snapshot, found port 9000 free, found both
default home session files absent, and removed the lab directory. The
remover's own record was `lab_removed` 1. A following check found no lab
directory, the snapshot hash and mode unchanged, port 9000 free, both home
session files absent, and no `slapd` process. Free space on the data volume
stayed above 7 GiB. After cleanup, available space was 10293580 KiB.

### Unrun on this LDAP import run

The executed chain is `riauth init`, `serve`, `/readyz`, server CLI login,
local `--version` and `capabilities`, `revision`, `group create staff`, a
SIGTERM restart after appending `[directories.staff]`, a second `/readyz`,
`directory list`, `directory plan`, `user list`, `group list`,
`directory apply`, and the same two reads again.

Still unrun on this run:

- `--confirm-removals`
- `--if-revision` on `directory apply`
- a directory password login
- `group add-member` for `admin`
- `directory workspace` and `directory entra`
- sections 10, 12, and 13
- an audit read of the plan or the apply
- `scripts/test-ldap.sh`
- `cargo install`
- `riauthctl` and `riauth-maintenance`
- `riauth doctor`
- `recovery complete`, `recover-admin`, and a second server
- a browser, a physical key, a synced passkey, a phone, a spoken screen
  reader, a customer directory, and the Essentials guide

D01 remains incomplete.
