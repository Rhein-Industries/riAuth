# Platform CLI walkthrough

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D01
`a96a1977-3210-4284-8f7d-645793369301`.

This page records two disposable loopback runs of the
[Platform guide](../platform-guide.md). The first used an Essentials-catalog
binary in place and stopped after one client registration. The second copied
a Platform-catalog snapshot and continued through the server CLI group, claim,
and audit commands. Neither run used a browser, an external peer,
`cargo install`, `riauthctl`, or `riauth-maintenance`.

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
