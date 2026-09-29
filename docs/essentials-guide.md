# Essentials guide: first tasks for a new operator and user

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D01
`a96a1977-3210-4284-8f7d-645793369301`.

This is the Essentials task guide through its second slice. The first five
tasks are one small loopback install, the first administrator sign-in, one
confidential OpenID Connect web application, passkey self-service in the
browser, and the backup and recovery commands that open a restored store.
The next tasks create a local group, publish claims for that application,
review the audit record, import one LDAP directory, and send one outbound
SCIM target. The commands are the ones implemented in this tree: `[features]`
in [Cargo.toml](../Cargo.toml), the [server CLI](../src/cli.rs),
[offline maintenance](../src/cli/local.rs), and the
[standalone client](../crates/riauthctl/src/main.rs).

Reading this page does not mean those steps were executed here. Both slices
were checked by reading the source and the current docs, then by
`python3 scripts/check-docs.py`. No Cargo build was run, no server was
started, and no browser, group change, claim preview, audit export, LDAP
plan, SCIM delivery, backup, or accessibility pass was recorded. The
[A01 coverage inventory](roadmap/coverage-inventory.md) still describes D01
against revision `96e23e2`, when editions were not in the tree. That row was
left as historical planning evidence.

A new Platform operator uses the [Platform guide](platform-guide.md) for the
same tasks with the Platform binaries. That guide's sections 11 through 13
add configured workflows, SAML, and the LDAP provider listener. Those three
procedures stay on the Platform page. Assembly limits that are outside
these tasks stay in [server editions](editions.md).

## Shared semantics

Essentials and Platform are two builds of the same source revision. They use
the same identity model, authorization, revocation, credential handling,
database format, and browser sign-in implementation. The `platform` Cargo
feature includes `essentials`. It adds compiled capabilities. It does not
grant a person or an agent any permission the server's authorization check
would refuse. [edition.rs](../src/edition.rs) states the same boundary: both
editions use the same Core, identity store, authorization checks, revocation
rules, and credential code.

Opening a store records the opening binary in `meta/edition_provenance`. An
Essentials binary refuses a store whose last activated edition is Platform,
or a store that still carries recorded Platform dependencies. Initialize,
restore, and `recover-admin` with the Essentials binaries so this small
instance stays Essentials. `keygen` does not open a store.

The [product contracts](roadmap/product-contracts.md) describe the desired
Essentials and Platform split. They are a target contract. They are not
evidence that every contract row is finished in this tree.

## Which program does which job

| Program | What it is in this tree | Use it for |
| --- | --- | --- |
| `riauth` | Server binary. This guide builds it with `--features essentials`. | `serve`, remote reads that still live on the server CLI (`login`, `doctor`, `backup`, `recovery`), and the legacy local commands that forward to maintenance. |
| `riauth-maintenance` | Offline binary from the same feature set. It has no server URL and no HTTP administration. | `init`, `prepare-setup`, `keygen`, `restore`, `recover-admin`. |
| `riauthctl` | Separate package. Its crate has no dependency on the server, on `riauth.toml`, or on storage. | Remote administration and end-user HTTP calls: `login`, `status`, `discovery`, `client create`. |
| `riauthctl` with `--features terminal-usb` | Optional rebuild of that client. The base client does not include it. | A CTAP2 USB authenticator on the operator's terminal. |

`riauthctl` always needs the exact issuer URL. For this slice that URL is
`http://localhost:9000`. The listener address `127.0.0.1:9000` is not a
substitute: the client checks discovery and requires the token endpoint to be
that server URL plus `/oauth/token`. HTTP is accepted for a loopback issuer.
The client rejects redirects. Its session file is
`$XDG_CONFIG_HOME/riauthctl/session.json` or `~/.config/riauthctl/session.json`.

The server CLI session is a different file,
`$XDG_CONFIG_HOME/riauth/session.json` or `~/.config/riauth/session.json`
([transport.rs](../src/cli/transport.rs)). `riauth backup` and `riauth doctor`
use that server-CLI session. A `riauthctl login` does not create it.

Signing in through the browser creates a browser session and an HttpOnly
cookie. It does not write either CLI session file.

Sections 6 and 7 sign the riauthctl session in again if it has expired, then
change groups and the `local-demo` client. `explain`, `audit`, `report`,
`directory`, and `provision` use the server CLI session from section 2.
Pass `--server http://localhost:9000` on those commands so the issuer is the
running lab server.

Terminal USB is a client feature. The base `riauthctl` fails `passkey login`
and `passkey enroll` locally, before any request, with: `USB passkeys are
unavailable in this build; rebuild riauthctl with --features terminal-usb`.
The server package has no USB transport. Its legacy passkey commands fail
locally with: `Terminal USB passkeys moved to riauthctl; install/build
riauthctl with --features terminal-usb. For other authenticator clients use
passkey start/finish` ([src/cli/usb.rs](../src/cli/usb.rs)). Passkey
self-service in this slice uses the browser, so the base client is enough.

Install one edition's `riauth` and `riauth-maintenance` on a deployment. A
later `cargo install` of the other feature set replaces those two executables
on the same Cargo bin path.

## 1. Install the small Essentials instance

Install Rust **1.98.1** (the channel in [rust-toolchain.toml](../rust-toolchain.toml)),
a C/C++ compiler, and CMake. On Linux the [getting-started guide](getting-started.md)
also lists `pkg-config` and OpenSSL development headers. This task did not
run the install.

From a clone of this repository:

```sh
cargo install --locked --path . --no-default-features --features essentials
cargo install --locked --path crates/riauthctl
riauth capabilities
```

The first command installs the Essentials `riauth` and `riauth-maintenance`
binaries. The server package's Cargo default feature is `platform`, so the
Essentials install names `--no-default-features --features essentials`
explicitly. The second command installs the base remote client. Its default
features are empty, so USB support is absent.

`riauth capabilities` prints the artifact catalog without opening a database
([capability.rs](../src/capability.rs)). In that document, `edition` is
`essentials`, `build_features` is `["essentials"]`, and `scope` is
`artifact`. Every `feature_states` entry has `usable` set to `null`. That
catalog describes the binary. It does not say that a peer, a browser, or an
authenticator is healthy.

This install is one set of Essentials binaries plus the remote client. The
instance created in the next section is loopback redb, with the issuer
`http://localhost:9000` and the listener `127.0.0.1:9000`. PostgreSQL,
released container archives, and Platform listeners are outside this slice.
[Linux image examples](deployment-examples.md) expect a released image archive
under `target/dist`. The [Q08 note](roadmap/q08-exact-edition-bundles.md)
records that those Linux archives were absent from the worktree it inspected,
at a different source revision. This task did not load an image.

## 2. Create the first administrator and sign in

Choose one of the two setups below. Both use the Essentials binaries and the
same loopback issuer. `init` refuses to replace an existing configuration
file, and `prepare-setup` refuses an initialized database, so they are
alternatives for a new directory.

### Operator: set the first password locally

```sh
mkdir -p deployment-private/essentials-lab
riauth-maintenance --config deployment-private/essentials-lab/riauth.toml init \
  --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 \
  --data-dir data \
  --admin admin
```

The command prompts for a password and does not echo it. The password must be
12 to 1024 bytes ([crypto.rs](../src/crypto.rs)). `init` writes
`deployment-private/essentials-lab/riauth.toml` and creates the redb directory
`data` beside that file. Relative paths are resolved from the configuration
file's directory. The generated files are under `deployment-private/`, which
this repository ignores. `reviewed_client_creation` defaults to false, so the
application created later does not wait for a creation review.

Leave the service running in this terminal:

```sh
riauth --config deployment-private/essentials-lab/riauth.toml serve
```

In another terminal:

```sh
curl --fail http://127.0.0.1:9000/readyz
```

`/readyz` checks storage readiness. A passing probe is the entry check for
this slice. It is not a production cutover.

### User: sign in

Open <http://localhost:9000/apps>. Sign in with username `admin` and the
password from `init`. The sign-in form asks for **Username** and
**Password**. Leave **Authenticator or recovery code** empty. This account
has no authenticator app yet.

That browser sign-in is the user session. It is separate from both CLI
session files.

### Operator: save the server CLI session used later for backup

Pass `--server` so the session binds to this issuer even when the current
directory contains a different `riauth.toml`. Without `--server`, the server
CLI uses `riauth.toml` in the current directory when that file exists, and
otherwise the built-in issuer `http://localhost:9000`.

```sh
riauth --server http://localhost:9000 login admin
riauth --server http://localhost:9000 doctor
```

`login` prompts for the same password and writes `~/.config/riauth/session.json`.
`doctor` uses that session. Keep the file private.

### Alternative: the administrator chooses the first credentials in the browser

Use this only for a directory that `init` has not already initialized.

```sh
umask 077
mkdir -p deployment-private/essentials-lab
cat > deployment-private/essentials-lab/riauth.toml <<'CONFIG'
issuer = "http://localhost:9000"
listen = "127.0.0.1:9000"
data_dir = "data"
access_token_ttl = 300
refresh_token_ttl = 2592000
session_ttl = 28800
CONFIG
riauth-maintenance --config deployment-private/essentials-lab/riauth.toml \
  prepare-setup --proof-file deployment-private/essentials-lab/setup-proof
riauth --config deployment-private/essentials-lab/riauth.toml serve
```

`prepare-setup` writes the proof file and prints `setup_url` and
`expires_at`. On Unix the file mode is 0600. The default lifetime is 15
minutes (`--expires-in` accepts 1 to 3600 seconds). The proof is not in the
URL. Give the proof to the intended administrator over a private channel.

The administrator opens <http://localhost:9000/setup>, pastes the proof, and
chooses a username. **Password** creates the account after the password is
confirmed. **Passkeys only** enrolls a primary passkey and a backup passkey
before the account exists. The server checks that the two credential IDs
differ. It cannot show that two synced passkeys live on two devices. Use a
second device or security key for the backup. Then choose **Continue to sign
in** and sign in. Setup does not create a session by itself.

The full ownership rules are in [browser first-administrator setup](browser-bootstrap.md).
After this path, the operator still runs `riauth login` with the new username
before `doctor` or `backup`. A passkey-only administrator has no password, so
that server-CLI password login is unavailable until recovery sets one. Backup
in the next sections assumes the password administrator from `init`.

## 3. Register one OIDC web application

The operator does this with `riauthctl` after the password administrator can
sign in. The server must still be running.

```sh
mkdir -p deployment-private
riauthctl --server http://localhost:9000 login admin
riauthctl --server http://localhost:9000 client create local-demo \
  --name 'Local demo' \
  --confidential \
  --redirect-uri http://localhost:3000/callback \
  --scope openid,profile \
  --secret-file deployment-private/local-demo-secret.json
```

`login` prompts for the administrator password and writes the riauthctl
session. Confidential creation requires a new `--secret-file` in a directory
that already exists. The client reserves that file before the request and
writes the one-time secret there. Standard output names `credential_file` and
omits the secret. Repeating the command needs the same `--idempotency-key`
and a new secret-file path; the details are in the
[riauthctl README](../crates/riauthctl/README.md).

If `--scope` is omitted, a non-service client asks for
`openid profile email offline_access`. This slice sets `openid,profile` so
the registered set matches the table below.

| Application setting | Value |
| --- | --- |
| Issuer | `http://localhost:9000` |
| Discovery | <http://localhost:9000/.well-known/openid-configuration> |
| Client ID | `local-demo` |
| Client secret | The secret in `deployment-private/local-demo-secret.json` |
| Redirect URI | `http://localhost:3000/callback`, exactly as registered |
| Flow | Authorization code, S256 PKCE, `state`, and `nonce` |

Check the issuer document and the saved session:

```sh
riauthctl --server http://localhost:9000 discovery
riauthctl --server http://localhost:9000 whoami
```

Start the application that serves `http://localhost:3000/callback` and use
its sign-in action. riAuth shows the browser sign-in and consent pages, then
redirects to that callback. Creating the client does not start an application.
Without a launch URL, the portal can show the client as **Setup pending**.
Adding a launcher is [Configure an application](PORTAL.md#configure-an-application).
The protocol details are in [OIDC profiles](oidc-profiles.md).

The same management service is available in the browser at
<http://localhost:9000/admin> under **Applications**. This slice uses the
remote command because its flags are the implemented client contract. The
admin page's extra fields, including Platform workflow authoring, are outside
this slice. Essentials hides the workflow section.

## 4. Add, rename, or remove your passkey

This is the signed-in user's task in the browser. The base `riauthctl` is not
required.

1. Open <http://localhost:9000/apps> and sign in.
2. Choose **Sign-in and security**.
3. Under **Passkeys**, enter a **Passkey name** and choose **Add a passkey**.
4. Complete the browser prompt. Enrollment asks for a resident
   (discoverable) credential with user verification. The relying party id is
   the issuer hostname, `localhost`, and the origin is the issuer origin.
5. Adding or removing a passkey signs that account out everywhere, including
   this browser. Sign in again to continue.
6. On a later visit, **Sign in with a passkey** is shown when the server
   reports `identity.passkeys` as usable and the browser can use passkeys
   ([signin.js](../src/portal/signin.js)).
7. To rename or remove a listed passkey, use **Rename** or **Remove**, then
   **Save name** or **Remove passkey**. A passkey-only account cannot remove
   its last passkey. An administrator who chose passkeys only at setup must
   keep two.

The first passkey on an account that has no passkey and no authenticator app
needs a recent sign-in. When the account already has a passkey or an
authenticator app, the change needs a passkey or authenticator sign-in from
the last five minutes. A browser that only received a terminal approval must
sign in itself before it can change passkeys. Those rules are in
[Passkeys](passkeys.md).

The dialog text says to pick this device, another device, or a security key
in the browser prompt. Synced passkeys, phones, and physical keys are part of
the manual accessibility and authenticator gates below. This slice does not
record a result for them.

### Optional terminal USB, separate from this self-service path

Rebuild the client only when a person will use a USB authenticator from the
terminal:

```sh
cargo install --locked --path crates/riauthctl --features terminal-usb
riauthctl --server http://localhost:9000 passkey enroll --name security-key
riauthctl --server http://localhost:9000 passkey login admin
```

Enrollment needs a recent riauthctl session. The USB build asks for touch and
PIN and fails with `--non-interactive`. It speaks CTAP2 over USB through
`webauthn-authenticator-rs`. Platform keychains, Bluetooth, and hybrid or
phone transports are not implemented in this client. A credential enrolled
here is often not discoverable, so it can be used for pinned
re-authentication and for the terminal, and it cannot start a browser
sign-in on its own.

The server binaries in this guide stay the Essentials build. USB support is
not compiled into them.

## 5. Backup and recovery entry points

Take the backup while `riauth serve` is running. `backup` is an authenticated
request on the server CLI session from section 2. `restore`, `recovery`, and
`recover-admin` are local. `riauthctl` has none of those commands.

Use a backup key that is different from any database key. This slice's `init`
does not set `database_key_file`, so the live store is plaintext and the
backup key is still required. Keep `backup.key` outside the host you are
willing to lose.

```sh
riauth-maintenance keygen --out deployment-private/essentials-lab/backup.key
riauth --server http://localhost:9000 backup \
  --key-file deployment-private/essentials-lab/backup.key \
  --out deployment-private/essentials-lab/backup.riauth
```

`keygen` refuses an existing output file. `backup` writes a private temporary
file and publishes `backup.riauth` only after the whole archive
authenticates. The current server command streams `riauth.backup/v3`.

Stop `riauth serve` before restore. A second opener of the same redb store
fails with `storage_owned`. Restore creates a new directory and does not
switch the running issuer.

```sh
riauth-maintenance restore \
  --backup deployment-private/essentials-lab/backup.riauth \
  --key-file deployment-private/essentials-lab/backup.key \
  --out deployment-private/essentials-lab/restored
```

Omit `--postgres-config`. The maintenance command then imports into new redb
([local.rs](../src/cli/local.rs)). On success the result includes
`"verified": true`, `"storage"`, `"serving_allowed": false`, and `"config"`
pointing at `deployment-private/essentials-lab/restored/riauth.toml`.
`verified` covers the local archive and store checks in
[operations.rs](../src/operations.rs). It does not mean a person signed in or
that an application completed login.

The restore applies the restored-state policy: sessions, grants, and pending
proofs are invalidated, and serving stays closed. Read the pending record:

```sh
riauth --config deployment-private/essentials-lab/restored/riauth.toml recovery status
```

`status` is read-only. Reopening service is a separate attestation:

```sh
riauth --config deployment-private/essentials-lab/restored/riauth.toml recovery complete \
  --recovery-id '<pending.id from status>' \
  --persistent-credentials-reconciled
```

Run `complete` only after the operator has reviewed the listed credentials.
The flag is the operator's statement. riAuth does not check it. This slice
stops at the entry points. It does not attest a restore, start a second
server under `http://localhost:9000`, or call this a recovery drill. The
order, the external files a backup omits, and the unrecoverable cases are in
[encrypted backup and restore](operations.md#encrypted-backup-and-restore),
[restored-state recovery](recovery.md), and
[disaster recovery](disaster-recovery.md).

### Break-glass administrator

Stop every process that has the store open, then:

```sh
riauth-maintenance --config deployment-private/essentials-lab/riauth.toml \
  recover-admin admin --password-stdin
```

Supply one new password of 12 to 1024 bytes on standard input. The command
sets that password, enables the account, makes it an administrator, revokes
its sessions, and records `admin.recover`. Existing factors stay enrolled.

A passkey-only account has an empty password hash and no authenticator-app
secret. Recovery then refuses the command unless the operator adds
`--reset-mfa`. The server error is `Passkey-only recovery requires explicit
--reset-mfa; enrolled factors will be removed`. With the flag, passkeys,
authenticator settings, and recovery codes are removed and the audit action
is `admin.recover.factors_reset`.

## 6. Create a group and add the administrator

The operator does this with `riauthctl` while `riauth serve` is still running.
Sign in again when the riauthctl session is missing:

```sh
riauthctl --server http://localhost:9000 login admin
```

Group names are 1–64 ASCII letters, digits, dots, hyphens, underscores, or
`@` ([validation.rs](../src/validation.rs)). `staff` fits. The server rejects
a group write that arrives without `Idempotency-Key` and `If-Match`
([core.rs](../src/core.rs), `Group writes require Idempotency-Key and If-Match`).
`riauthctl` can fill both in when the flags are omitted. The commands below
pass them so a retry repeats the same pair. Read `revision` immediately
before each write and substitute that number. A group write advances the
configuration revision, so the add-member command needs its own read and its
own key.

```sh
riauthctl --server http://localhost:9000 revision
riauthctl --server http://localhost:9000 \
  --run-id staff-group \
  --idempotency-key staff-create \
  --if-revision '<revision>' \
  group create staff
riauthctl --server http://localhost:9000 revision
riauthctl --server http://localhost:9000 \
  --run-id staff-group \
  --idempotency-key staff-add-admin \
  --if-revision '<revision>' \
  group add-member staff admin
riauthctl --server http://localhost:9000 group get staff
riauthctl --server http://localhost:9000 group has-member staff admin
```

`revision` returns `{"revision": <number>}`. `--if-revision` is that number.
`--idempotency-key` is 1–128 printable ASCII characters without spaces.
`--run-id` is the same shape and is stored on the audit row; it does not
replace the idempotency key. Repeat one write only by repeating its key and
its original `--if-revision`. The receipt is replayed before the revision
check. A different body or a different If-Match with that key returns
`Idempotency key was used for a different request`.

`group get` returns the group. `group has-member` reads the group and the
user and prints `member`. For this pair the value is true when `admin` is in
`staff`.

The server CLI has the same group verbs and uses the server CLI session.
`riauth group create` and `riauth group add-member` stop locally unless both
`--idempotency-key` and `--if-revision` are present. The message is:
`Group writes require --idempotency-key and --if-revision (from riauth revision)`.
The source string wraps `riauth revision` in backticks. This section uses
`riauthctl` because that is the session from section 3.

`--group staff` on `client update` replaces `allowed_groups` and limits who
may use the application. This section leaves `local-demo` unrestricted.
Membership and that restriction are different fields.

## 7. Publish the groups claim and one mapped claim

Two different settings put group and profile data in tokens for `local-demo`.

The `groups` scope adds a `groups` claim from the user's group names
([claims.rs](../src/claims.rs)). A claim mapping copies one declared source
into a named claim when the token request includes the mapping's scope. The
mapping below uses scope `profile`, claim `department`, and a literal
`lab`. Literal mappings are stored on the client and are public
configuration. The claim name must be absent from the protected set (`iss`,
`sub`, `aud`, `exp`, `iat`, `nbf`, `jti`, `nonce`, `auth_time`, `amr`,
`acr`, `at_hash`, `c_hash`, `sid`, `client_id`, `scope`, `cnf`, `act`,
`email`, `email_verified`). The mapping's scope must be one of the client's
scopes.

`--scope` replaces the whole scope set. Repeat `openid` and `profile` from
section 3, and add `groups`. `--settings-file` replaces the whole provider
settings object ([model.rs](../src/model.rs)). The client created in section
3 still has the default settings, so the file below is the complete object
for that client. Copy any setting already stored on the client into the file
before running the update.

Save this as `deployment-private/essentials-lab/local-demo-claims.json`:

```json
{
  "claim_mappings": [
    {
      "scope": "profile",
      "claim": "department",
      "source": { "type": "literal", "value": "lab" }
    }
  ]
}
```

`source.type` is the snake_case tag from [model/claims.rs](../src/model/claims.rs).
`literal` carries `value`. `groups` is the tag that copies group names, and
it is a different source from the built-in `groups` scope.

Read the revision, then update the client. A retry of this exact update
reuses the key and the same `--if-revision`.

```sh
riauthctl --server http://localhost:9000 revision
riauthctl --server http://localhost:9000 \
  --run-id local-demo-claims \
  --idempotency-key local-demo-claims \
  --if-revision '<revision>' \
  client update local-demo \
  --scope openid,profile,groups \
  --settings-file deployment-private/essentials-lab/local-demo-claims.json
```

There is no space after the commas. `--scope` on `client update` splits on
commas.

Preview the claims with the server CLI session. This posts to
`/api/policy/explain` and does not issue a token, create a grant, or sign
the user in:

```sh
riauth --server http://localhost:9000 explain local-demo admin \
  --scope 'openid profile groups'
```

`--scope` on `explain` splits on spaces. The response includes `simulation`,
`token_issued`, `allowed`, `reasons`, `userinfo`,
`id_token_identity_claims`, and `access_token_identity_claims`
([claims.rs](../src/claims.rs)). `token_issued` is false. After the writes above, these scopes put the group
list in `userinfo.groups` and `lab` in `userinfo.department`.
`userinfo.name` and `userinfo.preferred_username` come from the `profile`
scope. The access-token identity claims stay `sub` while
`claims_in_access_token` is left at its default, false. An administrator
satisfies the `group.read` check this preview makes for each projected
group.

## 8. Review the audit record

Audit commands use the server CLI session. An administrator passes
`audit.read` on `audit/events`. Rows are kept for 90 days
(`AUDIT_RETENTION_SECONDS` in [core.rs](../src/core.rs)). The group commands
in section 6 stored `run_id` `staff-group` on `group.create` and
`group.member.add`.

Recent events, a paged inventory row for that run, and a private CSV of the
same run:

```sh
riauth --server http://localhost:9000 audit --limit 100
riauth --server http://localhost:9000 inventory audit --limit 100 \
  --filter staff-group
riauth --server http://localhost:9000 report audit \
  --run-id staff-group \
  --out deployment-private/essentials-lab/audit-staff-group.csv
```

`audit --limit` reads `GET /api/audit`. The server clamps that limit to
1–1000. The default is 100. `inventory audit` returns `items`,
`next_cursor`, and `revision`. For the audit collection, `--filter` matches
an exact `run_id` and does not match an action prefix. Pass `next_cursor`
as `--after` until it is null. A cursor is bound to the caller, collection,
filter, and revision and expires after one hour.

`report audit` walks `GET /api/reports/audit.csv` and publishes a new file.
The command refuses an existing `--out`. On Unix the published file is mode
0600. `--run-id`, `--actor`, and `--target` are exact. `--action group`
also matches a dot-separated child such as `group.create`. `--from` and
`--to` are inclusive Unix seconds. `--limit` defaults to 1000 and the export
endpoint clamps it to 1–1000. The CSV columns are `id`, `at`, `actor`,
`action`, `target`, `run_id`, and `request_id`. The file has no details
column. The filter and column rules are in [audit review](enterprise/ENT-09.md)
and [CSV export](enterprise/ENT-15.md).

This review is the operational record. It is not a compliance certification.

## 9. Import one LDAP directory

`init` writes no `[directories]` table. LDAP import is a later edit of the
lab configuration, then `directory list`, `directory plan`, and
`directory apply` against the running server. Those three commands are
remote calls on the server CLI session. They are the shared directory
commands on both editions. On the Essentials build,
`workspace_directories` and `entra_directories` fail configuration
validation with `{field} requires the Platform build`
([edition.rs](../src/edition.rs)). The Workspace and Entra HTTP routes are
mounted only in the Platform build.

Stop the `riauth serve` process from section 2. Append this to
`deployment-private/essentials-lab/riauth.toml`, with the directory's real
URL, bind DN, bases, and filters. `directory.example.test` is only the shape
taken from [LDAP directories](ldap.md):

```toml
[directories.staff]
url = "ldap://directory.example.test:389"
transport = "starttls"
bind_dn = "cn=riauth,ou=services,dc=example,dc=test"
password_file = "ldap-password"
user_base = "ou=people,dc=example,dc=test"
user_filter = "(objectClass=inetOrgPerson)"
id_attribute = "entryUUID"
username_attribute = "uid"
display_attribute = "cn"
email_attribute = "mail"

[directories.staff.group_user_filters]
staff = "(memberOf=cn=staff,ou=groups,dc=example,dc=test)"
```

`starttls` requires an `ldap://` URL and a successful TLS upgrade before the
bind. `ldaps` requires an `ldaps://` URL. `loopback` is plaintext LDAP to a
literal loopback address, the transport the private test harness uses.
`starttls` and `ldaps` keep certificate and hostname checks on.
A private CA is `ca_file`, also relative to the configuration file.
`password_file` is resolved from the configuration file's directory
([config.rs](../src/config.rs)). Create
`deployment-private/essentials-lab/ldap-password` as a regular file with
owner-only permissions, mode 0600 or 0400, containing the service bind
password. A trailing CR or LF is stripped. An empty bind DN is rejected, and
an empty password is rejected before the bind. The key `staff` under
`group_user_filters` is the local group from
section 6. Apply returns `LDAP mappings require an existing local group`
when that group is missing ([assembly/directory.rs](../src/assembly/directory.rs)).

Start the same server again:

```sh
riauth --config deployment-private/essentials-lab/riauth.toml serve
```

`serve` loads the configuration once. The previous process must be stopped
first. A second opener of the same redb store fails with `storage_owned`.

In another terminal, with the server CLI session:

```sh
riauth --server http://localhost:9000 directory list
riauth --server http://localhost:9000 directory plan staff \
  --out deployment-private/essentials-lab/ldap-plan.json
```

`directory list` shows each configured directory's id, URL, user base,
mapped groups, and reconciliation mode. An omitted
`ldap_reconciliation_modes` entry stays manual review, so this directory
waits for `directory apply`. `directory plan` refuses an existing `--out`.
It posts until the snapshot finishes or the CLI page limit is reached, then
writes the plan. The plan's `revision` is the configuration revision it is
bound to. Planning contacts the directory. A directory the server cannot
reach fails the command before accounts change. `scripts/test-ldap.sh` is a
private OpenLDAP harness, not a step in this guide, and this task did not
run it.

Read `changes` and `removal_impact` in the plan file. Then:

```sh
riauth --server http://localhost:9000 directory apply \
  --plan deployment-private/essentials-lab/ldap-plan.json
```

When `removal_impact.review_required` is true, the CLI stops with
`Inspect LDAP removal_impact and changes, then rerun with --confirm-removals`.
The follow-up is the same command plus `--confirm-removals`. That sends the
plan id in `X-riAuth-Confirm-Removals`. The plan's `revision` must still
equal the current configuration revision. A later local change needs a new
plan. Adding `--if-revision` with the plan's `revision` sends If-Match for
the first apply. After a successful apply the configuration revision
advances, and the same `--if-revision` then returns
`Configuration revision changed`. A repeat without that flag returns the
stored applied result.

Page limits, the five-minute plan, and removal thresholds stay in
[LDAP directories](ldap.md) and
[connector removal safeguards](removal-safeguards.md). Imported accounts
have local passwords disabled. A later password login is checked against
the directory. This section stops at plan and apply.

## 10. Provision one outbound SCIM target

Outbound provisioning is `riauth provision`. It is mounted on both editions
at `/api/provisioning/`. Inbound SCIM is `riauth scim` and `/scim/v2`, and
the Platform build is the one that mounts those routes. This section does
not call `riauth scim`.

Stop `riauth serve` again. Append this to the same configuration. Replace
the URL with the target's SCIM base. The URL must be canonical HTTPS, or
HTTP on `localhost`, `127.0.0.1`, or `[::1]`, without credentials, a query,
or a fragment:

```toml
[scim_targets.payroll]
url = "https://payroll.example.com/scim/v2"
token_file = "payroll-scim-token"
groups = ["staff"]
export_groups = true
```

Leave `scim_reconciliation_modes` unset. The target then stays manual
review, and delivery waits for `provision apply`. `groups` lists one to 64
local group names. `staff` must already exist. `token_file` is resolved from
the configuration file's directory and is mutually exclusive with `oauth`.
Create `deployment-private/essentials-lab/payroll-scim-token` as a regular
owner-only file, mode 0600 or 0400. After trimming whitespace the token must
be nonempty ASCII graphic characters, at most 4096 bytes. OAuth client
credentials are the other mode, documented in
[outbound provisioning](scim.md#outbound-provisioning).

Start the server with the same `serve` command as section 9. Then:

```sh
riauth --server http://localhost:9000 provision targets
riauth --server http://localhost:9000 provision plan payroll \
  --out deployment-private/essentials-lab/payroll-plan.json
```

`provision targets` lists id, URL, groups, `export_groups`, and the
reconciliation mode. `provision plan` scans local users and writes the plan
after the snapshot completes. It refuses an existing `--out`. Enabled
non-administrator members of `staff` become Users. The `admin` account from
`init` is omitted. With `export_groups` true, the plan still carries the
`staff` group, and that group's members are the intersection with those
selected users. A plan whose only `staff` member is `admin` therefore has
an empty member list. Apply is the step that contacts the target.

Read `resources` and `removal_impact`. Then:

```sh
riauth --server http://localhost:9000 provision apply \
  --plan deployment-private/essentials-lab/payroll-plan.json
riauth --server http://localhost:9000 provision jobs
```

When removal review is required, the CLI stops with
`Inspect SCIM removal_impact and resources, then rerun with --confirm-removals`.
Add that flag on the same apply command after reading the plan. `provision
jobs` prints each job's `delivery_state`. The state names, the one-hour plan
lifetime, and the rule that delivery is at least once are in
[outbound provisioning](scim.md#outbound-provisioning). A target the server
cannot reach fails delivery. This task did not run a plan or an apply
against a live peer.

## Unverified architecture artifacts

These artifacts are in the tree. This slice did not execute them, and they
are not evidence that the small Essentials install serves every box they
draw.

| Artifact | What it is | What this slice can say |
| --- | --- | --- |
| [Architecture system map](architecture.md) | A combined diagram of browsers, the CLI, nginx and Traefik, HTTP, LDAP and RADIUS listeners, redb or PostgreSQL, and external peers. | It is a map of the codebase's surfaces. Essentials compiles out the SAML, RADIUS, proxy, and client-certificate adapters described in [server editions](editions.md). The diagram was not run against this install. |
| [Q08 exact edition matrix](roadmap/q08-exact-edition-bundles.md) | A local build observation at source revision `4ca7558`, including native binary hashes and a note that Linux release files were absent. | It is not an observation of this worktree's revision. This task did not rebuild the matrix. |
| Release archives named by [deployment examples](deployment-examples.md) and [release notes](release-notes.md) | Linux native archives, maintenance archives, `riauthctl` archives, container archives, `SHA256SUMS`, and `build-provenance` files. | They were not downloaded, loaded, or started here. `deploy/compose-small.yml` remains a documented image layout for a later deployment. |
| `riauth capabilities` | Artifact catalog for the binary on `PATH`. | `usable` is null until a configured instance reports runtime state. The catalog is not a peer or authenticator test. |

## Manual accessibility gates

[accessibility-journeys.spec.js](../tools/browser/accessibility-journeys.spec.js)
documents automated keyboard, axe WCAG 2.1 A/AA, 320/768/1440 reflow, and
200% text checks. It drives Chromium's virtual internal authenticator. The
spec itself says it does not claim:

- a physical security key
- a synced platform passkey
- a phone hybrid transport
- iOS or Android
- a spoken screen reader (VoiceOver, TalkBack, or NVDA)

Those five are manual gates. This task did not run the Playwright spec, a
desktop browser, or a screen reader. [Passkeys](passkeys.md) also says
physical hardware and platform compatibility still need testing on the
intended devices. A cancelled browser prompt, a synced passkey, and a
security key used from Safari or a phone remain manual checks for each
deployment.

## What remains for D01

This page is the Essentials half of the task guide through the second slice.
It does not finish the Essentials guide, and it does not finish D01.

Sections 6 through 10 are source-reviewed procedures. The generated `init`
file still has no directory and no SCIM target until the operator appends
them. Those commands were not executed here.

Still outside this slice:

- Invitations, email verification, password change and reset,
  authenticator-app enrollment, recovery codes, session list, and consent
  withdrawal as operator or user tasks. The portal markup includes password,
  authenticator-app, and sessions controls beside passkeys
  ([index.html](../src/portal/index.html)). [Account email and recovery](lifecycle.md)
  both describes those browser pages and still says authenticator-app
  enrollment and recovery-code rotation need the terminal. The pages were
  not run here, so that conflict stays unresolved.
- Inbound SCIM (`riauth scim` and `/scim/v2`). The Platform build mounts
  those routes. Section 10 is outbound `riauth provision` only.
- PostgreSQL, native HTTPS, a trusted proxy, and the released-image small
  layout in [deployment examples](deployment-examples.md).
- Configured workflows, SAML identity-provider and source administration,
  and the LDAP provider listener. Those procedures are sections 11 through
  13 of the [Platform guide](platform-guide.md). An Essentials configuration
  rejects a non-empty `workflows` table and a non-empty `ldap_listeners`
  table with `{field} requires the Platform build`. Client settings `saml`
  and `ldap` fail with `Client setting {field} requires the Platform build`.
  A SAML source fails with `SAML source requires the Platform build`.
- The other Platform protocols and administration in the
  [Platform guide](platform-guide.md), including Workspace and Entra
  directory import.
- The [capability and compatibility matrix](capability-matrix.md) records
  compiled inclusion, protocol direction, and the peers named by tests.
  The [Platform forward-auth recipe](recipes/platform-forward-auth.md),
  the [Platform LDAP-provider recipe](recipes/platform-ldap-provider.md),
  the [OIDC relying-party recipe](recipes/oidc-relying-party.md),
  and the [Platform SAML IdP recipe](recipes/platform-saml-idp.md)
  are the D03 recipes written so far. Forward auth, the LDAP provider,
  and the SAML IdP are Platform profiles. The relying-party recipe is the
  authorization-code fixture, and its client is the in-tree router rather
  than a named application. The SAML IdP recipe follows the xmlsec1 fixture.
  A named service provider remains an open peer. The LDAP provider recipe
  is not the LDAP import left unconfigured above. Other D03 recipes and
  acceptance against category targets (D05) remain open. The first D04
  decision page is [operational recovery](operational-recovery.md); further
  emergency runbooks remain open. The
  [connector dependency incidents](connector-incidents.md) page covers read,
  stop, and retry decisions for external dependencies. The backup commands
  here are entry points.
- Any claim that a person completed install, sign-in, the OIDC redirect,
  passkey enrollment, backup, restore, the group membership, the claim
  preview, the audit export, an LDAP plan or apply, or a SCIM plan or apply
  on this revision. Those claims need a run. This page does not supply one.
