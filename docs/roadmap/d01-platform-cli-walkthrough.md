# Platform CLI walkthrough

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D01
`a96a1977-3210-4284-8f7d-645793369301`.

This record is one disposable loopback run of the contiguous
[Platform guide](../platform-guide.md) steps that can complete without a
browser or an external peer: artifact capabilities, initial instance setup,
operator login, and one client registration. The source tree at the start of
the run was `12165852700b23f54943d094f61c180819523a52`. That commit changes
documentation only, relative to its parent
`3e895e932d24c3d4160b9f7f21c2132b96a29a2f`. No Cargo build was run.
`CARGO_TARGET_DIR` was unset.

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
configured.

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
- Groups, claims, and audit were not run
- Backup, restore, and recovery were not run
- `riauthctl` and `riauth-maintenance` were not run
- Sections 11 through 14 of the Platform guide were not run, including
  signing-key import, the `legacy-directory` client, and invitation acceptance

The shared setup and `local-demo` registration above ran on a binary whose
catalog edition was `essentials`. D01 remains incomplete.
