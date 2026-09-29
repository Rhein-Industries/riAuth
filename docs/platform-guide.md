# Platform guide: first tasks for a new operator and user

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D01
`a96a1977-3210-4284-8f7d-645793369301`.

This is the Platform task guide through its third slice. It walks the same
tasks as the [Essentials guide](essentials-guide.md): one small loopback
install, the first administrator sign-in, one confidential OpenID Connect web
application, passkey self-service in the browser, and the backup and recovery
commands that open a restored store, then a local group, claims for that
application, an audit review, one LDAP directory import, and one outbound
SCIM target. Sections 11 through 13 add three Platform-only procedures: one
configured password workflow, one SAML service provider and one SAML source,
and one LDAP provider listener. The binaries are the Platform build. The
commands are the ones implemented in this tree: `[features]` in
[Cargo.toml](../Cargo.toml), the [server CLI](../src/cli.rs),
[offline maintenance](../src/cli/local.rs), and the
[standalone client](../crates/riauthctl/src/main.rs).

Reading this page does not mean those steps were executed here. The three
slices were checked by reading the source and the current docs, then by
`python3 scripts/check-docs.py`. No Cargo build was run, no server was
started, and no browser, group change, claim preview, audit export, LDAP
plan, SCIM delivery, workflow plan, SAML import, source registration, LDAP
listener, backup, or accessibility pass was recorded. The
[A01 coverage inventory](roadmap/coverage-inventory.md) still describes D01
against revision `96e23e2`, when editions were not in the tree. That row was
left as historical planning evidence.

The install below leaves PostgreSQL, Workspace, Entra, RADIUS, proxy, and
the other Platform protocols unset. Sections 9 and 10 add the shared LDAP
import and outbound SCIM steps on this Platform server. Sections 11 through
13 add the configured-workflow, SAML, and LDAP provider procedures. Those
procedures were read from this tree and were not executed. Assembly and
downgrade rules stay in [server editions](editions.md).

## Shared semantics

Essentials and Platform are two builds of the same source revision. They use
the same identity model, authorization, revocation, credential handling,
database format, and browser sign-in implementation. The `platform` Cargo
feature includes `essentials`. It adds compiled capabilities. It does not
grant a person or an agent any permission the server's authorization check
would refuse. [edition.rs](../src/edition.rs) states the same boundary: both
editions use the same Core, identity store, authorization checks, revocation
rules, and credential code.

Opening a store records the opening binary in `meta/edition_provenance`.
`init`, `restore`, and the reopen inside restore stamp **platform** when
these binaries run. An Essentials binary then refuses that store, including
when the configuration and rows no longer show a Platform-only feature.
Stay on the Platform binaries for every later open of this instance.
`keygen` does not open a store. Moving a Platform store to Essentials is the
explicit handoff in [server editions](editions.md), after a reviewed backup.
That handoff is not a step in this slice.

The [product contracts](roadmap/product-contracts.md) describe the desired
split. They are a target contract. They are not evidence that every Platform
row is finished in this tree.

## Which program does which job

| Program | What it is in this tree | Use it for |
| --- | --- | --- |
| `riauth` | Server binary. This guide builds it with `--features platform`. | `serve`, remote reads that still live on the server CLI (`login`, `doctor`, `backup`, `recovery`), and the legacy local commands that forward to maintenance. |
| `riauth-maintenance` | Offline binary from the same feature set. It has no server URL and no HTTP administration. | `init`, `prepare-setup`, `keygen`, `restore`, `recover-admin`. The Platform maintenance binary is also the one that can run edition transition preflight, plan, and activate. |
| `riauthctl` | Separate package. Its crate has no dependency on the server, on `riauth.toml`, or on storage. | Remote administration and end-user HTTP calls: `login`, `status`, `discovery`, `client create`. |
| `riauthctl` with `--features terminal-usb` | Optional rebuild of that client. The base client does not include it. | A CTAP2 USB authenticator on the operator's terminal. |

`riauthctl` talks to whichever edition is serving. It cannot turn on a
capability the server binary omitted, and it cannot open the database.
It always needs the exact issuer URL. For this slice that URL is
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

Sections 11 through 13 use that same server CLI session for `plan`, `apply`,
`keys`, `saml`, `source`, `client create`, and `agent create`. `schema`,
`validate`, and `saml import-sp` run locally and do not read the session.
`riauthctl` can create a client and can plan and apply a manifest on the same
`/api/state` routes, using `~/.config/riauthctl/session.json`. The commands
printed in those sections are the server CLI forms.

Terminal USB is a client feature on either edition. The base `riauthctl`
fails `passkey login` and `passkey enroll` locally, before any request, with:
`USB passkeys are unavailable in this build; rebuild riauthctl with
--features terminal-usb`. The server package has no USB transport. Its
legacy passkey commands fail locally with: `Terminal USB passkeys moved to
riauthctl; install/build riauthctl with --features terminal-usb. For other
authenticator clients use passkey start/finish`
([src/cli/usb.rs](../src/cli/usb.rs)). Passkey self-service in this slice
uses the browser, so the base client is enough.

Install one edition's `riauth` and `riauth-maintenance` on a deployment. The
Cargo default feature set is already `platform`. This guide still passes
`--no-default-features --features platform` so the selected build is visible
in the command. A later Essentials `cargo install` replaces those two
executables on the same Cargo bin path.

## 1. Install the small Platform instance

Install Rust **1.98.1** (the channel in [rust-toolchain.toml](../rust-toolchain.toml)),
a C/C++ compiler, and CMake. On Linux the [getting-started guide](getting-started.md)
also lists `pkg-config` and OpenSSL development headers. This task did not
run the install.

From a clone of this repository:

```sh
cargo install --locked --path . --no-default-features --features platform
cargo install --locked --path crates/riauthctl
riauth capabilities
```

The first command installs the Platform `riauth` and `riauth-maintenance`
binaries. The second installs the base remote client. Its default features
are empty, so USB support is absent.

`riauth capabilities` prints the artifact catalog without opening a database
([capability.rs](../src/capability.rs)). In that document, `edition` is
`platform`, `build_features` is `["essentials", "platform"]`, and `scope` is
`artifact`. Every `feature_states` entry has `usable` set to `null`. That
catalog describes the binary. It does not say that a peer, a browser, or an
authenticator is healthy, and it does not mean the extra Platform features
are configured.

This install is one set of Platform binaries plus the remote client. The
instance created in the next section is loopback redb, with the issuer
`http://localhost:9000` and the listener `127.0.0.1:9000`. `init` fills the
configuration from those flags and the configuration defaults: no PostgreSQL,
no SAML, no RADIUS, no proxy listener, no workflow table, no cloud directory,
and no external signer. Released container archives are outside this slice.
[Linux image examples](deployment-examples.md) expect a released image archive
under `target/dist`. The [Q08 note](roadmap/q08-exact-edition-bundles.md)
records that those Linux archives were absent from the worktree it inspected,
at a different source revision. This task did not load an image.

## 2. Create the first administrator and sign in

Choose one of the two setups below. Both use the Platform binaries and the
same loopback issuer. `init` refuses to replace an existing configuration
file, and `prepare-setup` refuses an initialized database, so they are
alternatives for a new directory.

### Operator: set the first password locally

```sh
mkdir -p deployment-private/platform-lab
riauth-maintenance --config deployment-private/platform-lab/riauth.toml init \
  --issuer http://localhost:9000 \
  --listen 127.0.0.1:9000 \
  --data-dir data \
  --admin admin
```

The command prompts for a password and does not echo it. The password must be
12 to 1024 bytes ([crypto.rs](../src/crypto.rs)). `init` writes
`deployment-private/platform-lab/riauth.toml`, creates the redb directory
`data` beside that file, and records the store as Platform. Relative paths
are resolved from the configuration file's directory. The generated files are
under `deployment-private/`, which this repository ignores.
`reviewed_client_creation` defaults to false, so the application created
later does not wait for a creation review.

Leave the service running in this terminal:

```sh
riauth --config deployment-private/platform-lab/riauth.toml serve
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
session files. The sign-in implementation is the shared one. This slice does
not enable a Platform-only factor.

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
mkdir -p deployment-private/platform-lab
cat > deployment-private/platform-lab/riauth.toml <<'CONFIG'
issuer = "http://localhost:9000"
listen = "127.0.0.1:9000"
data_dir = "data"
access_token_ttl = 300
refresh_token_ttl = 2592000
session_ttl = 28800
CONFIG
riauth-maintenance --config deployment-private/platform-lab/riauth.toml \
  prepare-setup --proof-file deployment-private/platform-lab/setup-proof
riauth --config deployment-private/platform-lab/riauth.toml serve
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
in** and sign in. Setup does not create a session by itself. Serving the
prepared configuration with the Platform binary records Platform provenance.

The full ownership rules are in [browser first-administrator setup](browser-bootstrap.md).
After this path, the operator still runs `riauth login` with the new username
before `doctor` or `backup`. A passkey-only administrator has no password, so
that server-CLI password login is unavailable until recovery sets one. Backup
in the next sections assumes the password administrator from `init`.

## 3. Register one OIDC web application

The operator does this with `riauthctl` after the password administrator can
sign in. The server must still be running. The client and the authorization
code flow are shared with Essentials. This slice does not set Platform-only
client policy.

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
admin page can also show workflow authoring on Platform. Section 11 names
that authoring path and the runtime configuration. The browser editor was
not opened here. The [workflow model](workflows.md) remains the reference.

## 4. Add, rename, or remove your passkey

This is the signed-in user's task in the browser. The base `riauthctl` is not
required. Passkey enrollment, sign-in, rename, and removal use the shared
WebAuthn implementation on both editions.

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

The server binaries stay the Platform build from section 1. USB support is
not compiled into them. "Platform" in the product name is the server feature
set. It is not the platform authenticator on a phone or laptop.

## 5. Backup and recovery entry points

Take the backup while `riauth serve` is running. `backup` is an authenticated
request on the server CLI session from section 2. `restore`, `recovery`, and
`recover-admin` are local. `riauthctl` has none of those commands. Use the
Platform binaries so the restored store stays Platform.

Use a backup key that is different from any database key. This slice's `init`
does not set `database_key_file`, so the live store is plaintext and the
backup key is still required. Keep `backup.key` outside the host you are
willing to lose.

```sh
riauth-maintenance keygen --out deployment-private/platform-lab/backup.key
riauth --server http://localhost:9000 backup \
  --key-file deployment-private/platform-lab/backup.key \
  --out deployment-private/platform-lab/backup.riauth
```

`keygen` refuses an existing output file. `backup` writes a private temporary
file and publishes `backup.riauth` only after the whole archive
authenticates. The current server command streams `riauth.backup/v3`.

Stop `riauth serve` before restore. A second opener of the same redb store
fails with `storage_owned`. Restore creates a new directory and does not
switch the running issuer.

```sh
riauth-maintenance restore \
  --backup deployment-private/platform-lab/backup.riauth \
  --key-file deployment-private/platform-lab/backup.key \
  --out deployment-private/platform-lab/restored
```

Omit `--postgres-config`. The maintenance command then imports into new redb
([local.rs](../src/cli/local.rs)). On success the result includes
`"verified": true`, `"storage"`, `"serving_allowed": false`, and `"config"`
pointing at `deployment-private/platform-lab/restored/riauth.toml`.
`verified` covers the local archive and store checks in
[operations.rs](../src/operations.rs). It does not mean a person signed in or
that an application completed login. The reopen records Platform provenance.

The configuration produced by this slice has no `device_trust`, RADIUS
listeners, client certificates, or external signers. `Config::validate` reads
those materials when a later configuration contains them
([config.rs](../src/config.rs)). Restore calls that validation before it
publishes `riauth.toml`. Provision those files before restoring a backup that
includes them. This slice's archive does not include them. The full order is
in [disaster recovery](disaster-recovery.md).

The restore applies the restored-state policy: sessions, grants, and pending
proofs are invalidated, and serving stays closed. Read the pending record:

```sh
riauth --config deployment-private/platform-lab/restored/riauth.toml recovery status
```

`status` is read-only. Reopening service is a separate attestation:

```sh
riauth --config deployment-private/platform-lab/restored/riauth.toml recovery complete \
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

`recovery` exists only on `riauth`. Database-native restore, including
PostgreSQL point-in-time recovery, is an operator duty described in
[restored-state recovery](recovery.md). It is not part of this small redb
entry point.

### Break-glass administrator

Stop every process that has the store open, then:

```sh
riauth-maintenance --config deployment-private/platform-lab/riauth.toml \
  recover-admin admin --password-stdin
```

Supply one new password of 12 to 1024 bytes on standard input. The command
sets that password, enables the account, makes it an administrator, revokes
its sessions, and records `admin.recover`. Existing factors stay enrolled.
On PostgreSQL the command also refuses to run while another session named
`riauth` is connected. This slice uses redb, so the practical requirement is
to stop the server first.

A passkey-only account has an empty password hash and no authenticator-app
secret. Recovery then refuses the command unless the operator adds
`--reset-mfa`. The server error is `Passkey-only recovery requires explicit
--reset-mfa; enrolled factors will be removed`. With the flag, passkeys,
authenticator settings, and recovery codes are removed and the audit action
is `admin.recover.factors_reset`. The Platform build also clears an HTTPS
client-certificate binding for that user in the same recovery
([assembly/mtls.rs](../src/assembly/mtls.rs)). This slice has no
client-certificate binding to clear.

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

Save this as `deployment-private/platform-lab/local-demo-claims.json`:

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
  --settings-file deployment-private/platform-lab/local-demo-claims.json
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
group. Conditional claim mappings are a Platform settings field. This file
does not set `policy.conditional`, so the preview uses the ordinary mapping
path.

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
  --out deployment-private/platform-lab/audit-staff-group.csv
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
remote calls on the server CLI session and are the same verbs on both
editions.

The Platform build also accepts `[workspace_directories]` and
`[entra_directories]`. `riauth directory workspace` and
`riauth directory entra` call `/api/workspace-directories` and
`/api/entra-directories`, which `platform_routes` mounts
([api.rs](../src/api.rs)). Each has `list`, `plan`, and `apply`. The
Essentials build rejects those configuration tables with
`{field} requires the Platform build`. This section does not add those
tables and does not run those commands. Their operator notes are
[Google Workspace](enterprise/ENT-03.md) and
[Microsoft Entra](enterprise/ENT-04.md).

Stop the `riauth serve` process from section 2. Append this to
`deployment-private/platform-lab/riauth.toml`, with the directory's real
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
`deployment-private/platform-lab/ldap-password` as a regular file with
owner-only permissions, mode 0600 or 0400, containing the service bind
password. A trailing CR or LF is stripped. An empty bind DN is rejected, and
an empty password is rejected before the bind. The key `staff` under
`group_user_filters` is the local group from
section 6. Apply returns `LDAP mappings require an existing local group`
when that group is missing ([assembly/directory.rs](../src/assembly/directory.rs)).

Start the same server again:

```sh
riauth --config deployment-private/platform-lab/riauth.toml serve
```

`serve` loads the configuration once. The previous process must be stopped
first. A second opener of the same redb store fails with `storage_owned`.

In another terminal, with the server CLI session:

```sh
riauth --server http://localhost:9000 directory list
riauth --server http://localhost:9000 directory plan staff \
  --out deployment-private/platform-lab/ldap-plan.json
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
  --plan deployment-private/platform-lab/ldap-plan.json
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
at `/api/provisioning/`. Inbound SCIM is `riauth scim` and `/scim/v2`. This
Platform build mounts those inbound routes in `platform_routes`. They stay a
later task. This section does not call `riauth scim`.

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
Create `deployment-private/platform-lab/payroll-scim-token` as a regular
owner-only file, mode 0600 or 0400. After trimming whitespace the token must
be nonempty ASCII graphic characters, at most 4096 bytes. OAuth client
credentials are the other mode, documented in
[outbound provisioning](scim.md#outbound-provisioning).

Start the server with the same `serve` command as section 9. Then:

```sh
riauth --server http://localhost:9000 provision targets
riauth --server http://localhost:9000 provision plan payroll \
  --out deployment-private/platform-lab/payroll-plan.json
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
  --plan deployment-private/platform-lab/payroll-plan.json
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

## 11. Select one configured password workflow

This procedure selects one Platform password-only reauthentication workflow
named `local-password`. The server CLI has no `workflow` subcommand. Two
stores are involved, and they do different jobs.

The runtime entry is `[workflows.local-password]` in `riauth.toml`. `serve`
loads that file once at startup. `active = true` is what
`workflow_configured_start` requires. A definition stored by desired state
does not set that table and does not start a run.

The authoring entry is one `workflows` item in a `riauth/v1` manifest.
`riauth plan` and `riauth apply` persist that definition in
`workflow_definitions`. The browser routes do the same for one definition.
The [workflow model](workflows.md#active-configured-local-verifier-paths)
prints this password-only shape, and
[Essentials and Platform](workflows.md#essentials-and-platform) separates
authoring from activation.

### Prerequisites

The Platform server from sections 1 and 2 is running, and the server CLI
session from section 2 exists. Remote commands pass
`--server http://localhost:9000`.

Editing `riauth.toml` means stopping `riauth serve` first. A second process
that opens the same redb file fails with code `storage_owned`: the store is
already owned by another riAuth process. After the edit, start the same
`riauth --config deployment-private/platform-lab/riauth.toml serve` command.
Relative paths in that file resolve from the configuration file's directory.

An Essentials configuration with a non-empty `workflows` table fails
validation with `workflows requires the Platform build`. A manifest with a
non-empty `workflows` array fails on an Essentials binary with
`Configured workflows require Platform`.

### Permissions

A human administrator session allows `workflow.write` on
`workflow/local-password` and `workflow.read` on the same resource. An agent
needs those permissions on that exact resource. `workflow.write` and
`workflow.read` are Platform actions.

The start route uses the caller's live session bearer. It is a separate
check from the authoring permission. The route takes the workflow id from
the path. It accepts no caller-selected actions.

Plan and apply bind the saved plan's `base_revision`. They refuse a
delegated human session. An agent with `workflow.write` on
`workflow/local-password` can plan and apply. This procedure does not pass
`--if-revision`. The signing-key imports in section 12 and the
`client create` in section 13 pass `--idempotency-key` and
`--if-revision` for every caller, including the administrator session from
section 2. This procedure uses that session.

### Commands

Print the local schemas, then write the authoring file. `riauth schema` is
local. It prints the JSON Schema for `workflow::Definition` and for
`state::Manifest`.

```sh
riauth schema workflow
riauth schema manifest
```

Write `deployment-private/platform-lab/local-password.json`. The `workflows`
value is the definition itself. It has no `active` field. Omitted users,
groups, clients, sources, and source links stay as they are. `issuer` binds
the file to this lab.

```json
{
  "api_version": "riauth/v1",
  "issuer": "http://localhost:9000",
  "workflows": [
    {
      "format": "riauth.workflow/v1",
      "id": "local-password",
      "revision": 1,
      "category": "authentication",
      "origin": "configured",
      "entry": "password",
      "limits": {"max_duration_seconds": 600, "max_executions": 3},
      "steps": [
        {
          "id": "password",
          "action": {"type": "verify_password"},
          "max_attempts": 3,
          "timeout_seconds": 120,
          "cancellable": true,
          "transitions": [
            {"on": "verified", "to": "success"},
            {"on": "failed", "to": "denied"}
          ]
        }
      ],
      "terminals": [
        {"id": "success", "outcome": "authenticated", "requires": [["password"]]},
        {"id": "denied", "outcome": "denied", "requires": []}
      ]
    }
  ]
}
```

```sh
riauth validate --file deployment-private/platform-lab/local-password.json
riauth --server http://localhost:9000 plan \
  --file deployment-private/platform-lab/local-password.json \
  --out deployment-private/platform-lab/local-password-plan.json
riauth --server http://localhost:9000 apply \
  --plan deployment-private/platform-lab/local-password-plan.json
```

`plan` refuses an existing `--out` with `Plan output already exists`. When
`removal_impact.review_required` is true, `apply` stops with
`Inspect desired-state removal_impact and changes, then rerun with --confirm-removals`.
Read the plan before adding that flag. A plan expires 15 minutes after it is
created. Apply refuses a stale base revision with
`Connector plan expired or source configuration or local revision changed; create a new plan`.

Stop `serve`, append the runtime entry to
`deployment-private/platform-lab/riauth.toml`, and start the same `serve`
command. The key must equal the definition id. At most 32 workflows are
accepted. The document must be at most 64 KiB.

```toml
[workflows.local-password]
active = true

[workflows.local-password.definition]
format = "riauth.workflow/v1"
id = "local-password"
revision = 1
category = "authentication"
origin = "configured"
entry = "password"

[workflows.local-password.definition.limits]
max_duration_seconds = 600
max_executions = 3

[[workflows.local-password.definition.steps]]
id = "password"
action = { type = "verify_password" }
max_attempts = 3
timeout_seconds = 120
cancellable = true
transitions = [
  { on = "verified", to = "success" },
  { on = "failed", to = "denied" },
]

[[workflows.local-password.definition.terminals]]
id = "success"
outcome = "authenticated"
requires = [["password"]]

[[workflows.local-password.definition.terminals]]
id = "denied"
outcome = "denied"
requires = []
```

`Config::validate` accepts this block because it matches the password-only
adapter. These names fail that check even when the shape matches:
`platform-password-totp-reauthentication`,
`platform-invitation-password-enrollment`,
`platform-source-reauthentication`, and
`platform-source-totp-reauthentication`. The error is
`Configured workflow {name} has no executable adapter`. An id that starts
with `essentials-` fails workflow validation because that prefix is reserved
for built-in workflows. Identifiers start with a lowercase letter and then
use lowercase letters, digits, `_`, or `-`.

There is no CLI wrapper for the start route. After the runtime entry is
active and `serve` has been restarted, a later caller sends
`POST /api/workflows/configured/local-password` with that caller's bearer
session. The body supplies no actions. Continuation routes that already
exist, and that this procedure does not call, are
`POST /api/workflows/{id}/password`, `GET /api/workflows/{id}`, and
`POST /api/workflows/{id}/cancel` for a cancellable step. The generic start
route does not start consent, password reset, passkey removal, or a
source-first passkey. Those have their own routes. The shapes left for later
work are listed in [Left to later work](workflows.md#left-to-later-work).

The browser authoring routes, mounted on Platform, are
`GET /api/admin/workflows`, `POST /api/admin/workflows/plan`, and
`POST /api/admin/workflows/apply`. The apply route rejects a body unless it
contains exactly one workflow and empty users, groups, clients, sources,
source links, and secrets, with `Workflow editor applies one workflow only`.
The list reads `workflow_definitions`. Saving there still leaves
`config.workflows` unchanged.

A stored definition can be read back with the server CLI:

```sh
riauth --server http://localhost:9000 export \
  --out deployment-private/platform-lab/state-export.json
```

`export` refuses an existing `--out`. The file is the visible manifest.
`secrets_included` is false. Workflows in it are the definitions the caller
may `workflow.read`.

### Expected observation

`riauth validate` prints `valid` true, `validation` `local_schema`, and
`secret_values_read` false. `resources` counts users, groups, and clients.
This file has none of those, so `resources` is 0. Local validate does not
decide that an adapter can start. A file that is not a definition fails
while the manifest is read, before that result.

A first plan, on a store with no `local-password` definition, records a
create for `workflow/local-password`. The command prints `plan_file`,
`plan_id`, `hash`, `base_revision`, `changes`, `removal_impact`,
`reconciliation_mode`, and `expires_at`. The plan file is Unix mode 0600.
An existing destination fails with `Refusing to overwrite` and the path.
An identical definition already stored produces no workflow change. A later
edit that keeps revision 1 fails apply with
`Workflow revision must increase when the definition changes`.

If `serve` starts after the runtime block is saved, `Config::validate` has
accepted the password-only adapter. A later successful
`POST /api/workflows/configured/local-password` returns a view whose
`binding.workflow` is `local-password`, `binding.revision` is 1, and `state`
is active on step `password`. The public view has no evidence reference.
The route issues neither a new session nor a downstream OpenID Connect code.
An account with enrolled TOTP is refused with
`This account needs a different verifier path`. The `admin` account from
`init` has no authenticator app, so that account matches this password-only
shape. An inactive or missing entry returns `Configured workflow is unavailable`.
A present entry whose id or shape matches no executable adapter returns
that same message as a conflict.

### Unrun and peer evidence

This task did not write the JSON file, did not edit `riauth.toml`, did not
stop or restart `serve`, did not run `validate`, `plan`, `apply`, or
`export`, and did not send the start request. The browser editor was not
opened. No run id, view, or audit row was recorded. TOTP, recovery-code,
and passkey configured shapes are described in
[Local password reauthentication](workflows.md#local-password-reauthentication)
and were not added here.

## 12. Prepare one SAML service provider and one SAML source

This procedure has two halves. riAuth is the identity provider for one
service provider, and riAuth is the service provider for one upstream SAML
identity provider. The command names are the current
[SamlCommand](../src/cli.rs) and [SourceCommand](../src/cli.rs). The profile
detail is [Connect your first SAML service provider](saml.md#connect-your-first-saml-service-provider)
and [Upstream SAML identity providers](saml.md#upstream-saml-identity-providers).
The desired-state contract is [Desired state](agent.md#desired-state).

This lab's issuer is `http://localhost:9000`. HTTP is the supported loopback
evaluation case. A public issuer is HTTPS. This task did not configure a
public HTTPS issuer, and it did not contact a service provider or an
upstream identity provider.

### Prerequisites

The Platform server from sections 1 and 2 is running, and the server CLI
session from section 2 exists. Remote commands pass
`--server http://localhost:9000`.

The operator supplies three local files before the identity-provider half:
the service provider's metadata XML, an RSA private key PEM, and the matching
public certificate PEM. `saml import-sp` reads at most 48 KiB plus one byte
of XML and 16 KiB plus one byte of the certificate. `keys import` reads the
private key through the private-file reader, limited to 16384 bytes. Keep
those files owner-readable under `deployment-private/platform-lab/`. This
task did not create them.

The upstream half needs the same kind of local RSA key, the service-provider
certificate that matches it, and one to four pinned identity-provider
certificates. Placeholder PEM text fails certificate parsing. Replace it
before `source put`.

An Essentials client with `settings.saml` fails with
`Client setting saml requires the Platform build`. An Essentials source with
`saml` set fails with `SAML source requires the Platform build`.

### Permissions

The administrator session allows `key.write` on `key/saml-signing` and on
`key/source-sp`, `client.write` on `client/legacy-sp`, and `source.write` on
`source/corporate-saml`. An agent needs those exact permissions. An agent
cannot set `allow_admin_login`. This example leaves that field at its
default, false, so the upstream source does not sign the local administrator
in.

`saml import-sp` is local. It returns before any HTTP call.
`keys import`, `saml metadata`, `source put`, `source list`, and
`source metadata` use the server CLI session. `source start` authenticates
only when `--link` is set. `source finish` sends no bearer.

`keys import`, `keys bind`, and `keys generate` are signing-key
configuration. Every caller, including the administrator session from
section 2, passes both `--idempotency-key` and `--if-revision`. Read
`riauth --server http://localhost:9000 revision` immediately before each
import and substitute that number. `saml-signing-import` and
`source-sp-import` are separate keys. The client plan and apply between
the two imports advance the configuration revision, so the second import
reads `revision` again. A retry of one import repeats that import's key
and the revision read for that attempt. The receipt is replayed before
the revision check. A different body or a different If-Match with that
key returns `Idempotency key was used for a different request`. A supplied
revision that is already stale fails with `Configuration revision changed`.

The accepted CLI stops when either flag is missing. The message is:
`Signing-key configuration requires --idempotency-key and --if-revision (from riauth revision)`.
The source string wraps `riauth revision` in backticks. The HTTP writer
returns `Signing-key configuration requires Idempotency-Key and If-Match`.
Both strings were read from accepted commit `03482b5`: the CLI gate covers
`keys import`, `keys bind`, and `keys generate`, and the HTTP gate is the
request-context check in `configure_key`. This checkout's `src/cli.rs` and
`src/assembly/keyring.rs` were compared with that commit and do not contain
those strings. The global flags exist in this checkout, and the imports
below pass them. This procedure prints the two imports. `keys bind` and
`keys generate` stay unprinted, and they were not run.

`source put` follows the scoped-mutation rule. An agent or a delegated
human sends `If-Match` with the current revision. The administrator
session from section 2 can call the printed `source put` without
`--if-revision`. Plan and apply bind the saved plan's `base_revision`.
A delegated human is refused by plan and apply. Plan and apply do not
take `--if-revision`.

### Commands

Import the identity-provider signing key, then import the service-provider
metadata offline. The revision read belongs to the key import.
`saml import-sp` stays local and takes neither retry flag. SAML XML signing
requires a local RS256 domain. The command also accepts `ES256` and
`EdDSA`; those algorithms fail the SAML check with
`SAML XML signing currently requires a local RS256 signing domain`.
Vault bind is a different command and stays outside this slice.

```sh
riauth --server http://localhost:9000 revision
riauth --server http://localhost:9000 \
  --idempotency-key saml-signing-import \
  --if-revision '<revision>' \
  keys import saml-signing \
  --file deployment-private/platform-lab/idp-private.pem \
  --algorithm RS256
riauth saml import-sp \
  --file deployment-private/platform-lab/sp-metadata.xml \
  --entity-id https://sp.example.com/metadata \
  --idp-certificate deployment-private/platform-lab/idp-certificate.pem \
  --out deployment-private/platform-lab/sp-import.json
```

`--entity-id` must be the entity ID printed in that metadata file. The
import selects one `EntityDescriptor` with that exact value and makes no
remote fetch. An existing `--out` fails with `Refusing to overwrite` and
the path.

Read `deployment-private/platform-lab/sp-import.json`. Copy its
`settings.saml` object into the client manifest below. Set
`settings.signing_key` to `saml-signing`. Set `scopes` from the report's
`required_scopes`: `openid` and `saml`, plus `email` when the selected
NameID format is email. Add scopes for any attribute mappings you review.
The report's `instruction` says to apply that object through the client
manifest. The private key must match `idp_certificate_pem`. A mismatch
fails the client write with
`SAML IdP certificate must match the selected signing domain`. Leave
`encryption_certificate_pem` unset unless assertion encryption is a separate
reviewed choice. The report may still show `sp_encryption_certificate_pem`.

`riauth schema provider` and `riauth schema manifest` print the local
schemas. The manifest below is the envelope. Replace the empty `saml`
object with the report's `settings.saml` before validate. An empty object
does not parse as SAML settings.

```sh
riauth schema provider
riauth schema manifest
```

```json
{
  "api_version": "riauth/v1",
  "issuer": "http://localhost:9000",
  "clients": [
    {
      "client_id": "legacy-sp",
      "name": "Legacy SAML application",
      "confidential": false,
      "service": false,
      "enabled": true,
      "redirect_uris": [],
      "scopes": ["openid", "saml"],
      "settings": {
        "signing_key": "saml-signing",
        "saml": {}
      }
    }
  ]
}
```

Save that file as `deployment-private/platform-lab/legacy-sp.json` only
after `settings.saml` is the reviewed report object. `service` stays false.
A service client fails SAML validation with
`Invalid SAML interactive policy client or profile limits`. The client can
have an empty redirect list.

```sh
riauth validate --file deployment-private/platform-lab/legacy-sp.json
riauth --server http://localhost:9000 plan \
  --file deployment-private/platform-lab/legacy-sp.json \
  --out deployment-private/platform-lab/legacy-sp-plan.json
riauth --server http://localhost:9000 apply \
  --plan deployment-private/platform-lab/legacy-sp-plan.json
riauth --server http://localhost:9000 saml metadata legacy-sp \
  --out deployment-private/platform-lab/idp-metadata.xml
```

`saml metadata` calls `GET /api/saml/legacy-sp/metadata` and writes the
`metadata_xml` string. There is no `client create --file` flag. The server
CLI create command accepts `--settings-file` as a `ProviderSettings`
document, which can carry `signing_key` and `saml` after the same review.
A direct `client create` for this client would also pass a fresh
`revision` read, its own `--idempotency-key`, and `--if-revision`, as
section 13 does for `legacy-directory`. This section uses the manifest
because that is the import report's apply path, and it does not print
that create. Attribute mapping fields are in the SAML page. This slice
adds none.

The public metadata URL, relative to this issuer, is
`/saml/legacy-sp/metadata`. The browser sign-in and logout behavior is the
SAML page's. Registering the exported metadata at a service provider, and
starting a login from that service provider, is peer work this task did not
do.

For the upstream source, import a second local RS256 domain and write a
`SourceInput` file. `riauth schema source-input` prints that schema. Omit
`client_secret`. A SAML source uses `token_endpoint_auth_method` `none`, an
empty `token_endpoint`, empty `scopes`, and no OAuth profile. A secret on
that source fails with `A public source cannot have a client secret`.

```sh
riauth schema source-input
riauth --server http://localhost:9000 revision
riauth --server http://localhost:9000 \
  --idempotency-key source-sp-import \
  --if-revision '<revision>' \
  keys import source-sp \
  --file deployment-private/platform-lab/sp-private.pem \
  --algorithm RS256
```

```json
{
  "source": {
    "id": "corporate-saml",
    "name": "Corporate SAML",
    "issuer": "urn:company:idp",
    "authorization_endpoint": "https://idp.example.com/sso",
    "client_id": "urn:company:riauth-sp",
    "token_endpoint_auth_method": "none",
    "token_endpoint": "",
    "scopes": [],
    "auto_provision": false,
    "saml": {
      "signing_key": "source-sp",
      "sp_certificate_pem": "PUBLIC SP CERTIFICATE PEM",
      "idp_certificates_pem": ["PINNED IDP CERTIFICATE PEM"],
      "name_id_format": "persistent"
    }
  }
}
```

Replace both PEM placeholders with the certificate text. `issuer` and
`client_id` are absolute entity IDs. `authorization_endpoint` is the
upstream redirect SSO URL, canonical HTTPS, or HTTP on loopback, with no
query or fragment. `name_id_format` `transient` is rejected. One to four
pinned certificates are required. `auto_provision` false creates no local
accounts. `https://idp.example.com/sso` and `urn:company:idp` are the shape
from the SAML page. They are not a peer that was contacted.

Save the reviewed file as
`deployment-private/platform-lab/corporate-saml.json`.

```sh
riauth --server http://localhost:9000 source put \
  --file deployment-private/platform-lab/corporate-saml.json
riauth --server http://localhost:9000 source list
riauth --server http://localhost:9000 source metadata corporate-saml \
  --out deployment-private/platform-lab/sp-metadata.xml
```

`source put` posts the file to `/api/sources`. The source signing domain
must already exist, must be local RS256, and the service-provider
certificate must match it. A missing domain fails with
`SAML source signing domain is missing`. A mismatched certificate fails with
`SAML source SP certificate does not match its signing domain`.

`source metadata` writes the metadata XML privately. The public copy is
`/saml/sources/corporate-saml/metadata`, and the assertion consumer is
`/saml/sources/corporate-saml/acs`, both relative to this issuer. Registering
that metadata at the upstream identity provider was not done.

`source start` and `source finish` are implemented. This task did not run
them. `source start` refuses an existing `--out` with
`Source transaction file already exists`, writes the credential privately,
and prints `authorization_url`, `transaction_file`, and `instruction`.
The instruction text is `Authenticate at the upstream provider, then inspect and finish this request in the CLI`.
The saved credential contains `issuer`, `source`, `token`, and `expires_at`.
The upstream response is delivered to the source ACS on the server.
`source finish` reads the private transaction file and posts its token.
`--yes` posts `approve` true and, when the status is `complete`, replaces
`~/.config/riauth/session.json` and deletes the transaction file. The same
command without `--yes` posts `approve` false.
`source start corporate-saml --link` is the linking form and needs the
server CLI session. A browser source-stage login remains acceptance work in
[ENT-11](enterprise/ENT-11.md).

### Expected observation

`saml import-sp` prints `report_file` and `unresolved`. The report file also
contains `settings.saml`, `sp_encryption_certificate_pem`, `metadata_trust`
with the value `explicit_local_operator_input`, `required_scopes`, and
`instruction`. On Unix the report, the plan, and both metadata files are
mode 0600.

`keys import` prints `id`, `active`, and `retained_verification_keys`.
The command posts the PEM it read to `/api/keys` as `private_key_pem`.
The server stores the imported local signing key in the riAuth key store,
at `key_domains/<id>` for `saml-signing` and `source-sp`. `active` is the
public JWK of that stored key.
Importing a key whose `kid` is already present fails with
`Signing key id is already in use`.

`saml metadata` prints `metadata_file` and `client_id`. `source list` prints
the stored sources. `source metadata` prints `source` and `metadata_file`.
`source put` prints the stored source object. A changed source revokes
sessions that already carry that source. This example has no such session.

`validate` on the client manifest, once `settings.saml` is the report
object, prints `valid` true and `secret_values_read` false. `resources`
counts that one client. Plan and apply follow the same review rules as
section 11. Apply of a new client records a create for `client/legacy-sp`.
The manifest does not put a SAML source. The source is the separate
`source put`.

### Unrun and peer evidence

This task did not create keys or metadata, did not run `revision` or
`keys import`, and did not run `saml import-sp`, `validate`, `plan`,
`apply`, `saml metadata`, `source put`, `source list`, or
`source metadata`. It did not run `source start` or `source finish`.
No service provider loaded the identity provider metadata. No upstream
identity provider received an AuthnRequest or posted a response. Logout,
assertion encryption, and a browser source-stage run were not exercised.
The `xmlsec1` cargo tests in the SAML page are test commands, not operator
steps, and they were not run.

## 13. Serve one LDAP provider listener

The LDAP provider is a read-only LDAPv3 listener in the same process as
`serve`. It is a different feature from section 9. Section 9's
`[directories]` table imports users from an upstream directory. This section
adds `[ldap_listeners]`, one policy client with `settings.ldap`, and one
service agent. User and group administration stays on the existing user and
group commands. The behavior page is [LDAP provider](ldap-provider.md).
That page's `client create --file` form is not a flag in this tree. The
create command accepts `--settings-file`, and the settings file is a
`ProviderSettings` document.

### Prerequisites

The Platform server from sections 1 and 2 is running for the client and
agent commands. The server CLI session from section 2 exists. Pass
`--server http://localhost:9000`.

`search_groups` and `allowed_groups` name `staff`. Section 6 is the
procedure that creates `staff` and adds `admin`. That procedure was not
executed in this task. Until the group exists, `ldap_profile` fails with
`LDAP search group does not exist`. A search then reports inappropriate
matching with an empty message. A bind reports invalid credentials. Until
`admin` is a member, a user bind for `admin` is refused by the client group
policy and the LDAP bind result is also invalid credentials.

The listener needs a certificate file and a key file. Place them next to
the configuration as `ldap-fullchain.pem` and `secrets/ldap-key.pem`, or
change the two paths. Relative paths are resolved from the configuration
file's directory when the file is loaded. This task did not create a
certificate.

Stop `serve` before editing `riauth.toml`, then start the same serve
command. A second opener of the redb store fails with `storage_owned`.

An Essentials configuration with a non-empty `ldap_listeners` table fails
with `ldap_listeners requires the Platform build`. An Essentials client
with `settings.ldap` fails with
`Client setting ldap requires the Platform build`.

### Permissions

The administrator session allows `client.write` on
`client/legacy-directory` and agent administration. The service credential
needs `ldap.search` on `client/legacy-directory`. `ldap.search` is a
Platform action. The listener id `legacy` and the client id
`legacy-directory` must be 1–64 ASCII letters, digits, dots, hyphens,
underscores, or `@`.

`client create` and `client update` require both `--idempotency-key` and
`--if-revision` for every caller, including the administrator session from
section 2. Read `riauth --server http://localhost:9000 revision` immediately
before this create and substitute that number. This create uses the key
`legacy-directory-create`. A retry repeats that key and the revision read
for that attempt. The receipt is replayed before the revision check. A
different body or a different If-Match with that key returns
`Idempotency key was used for a different request`. A supplied revision
that is already stale fails with `Configuration revision changed`.

The accepted CLI stops when either flag is missing. The message is:
`Client writes require --idempotency-key and --if-revision (from riauth revision)`.
The source string wraps `riauth revision` in backticks. The HTTP writer
returns `Client writes require Idempotency-Key and If-Match`. Both strings
were read from accepted commit `03482b5`: `run_client` covers create,
update, disable, and enable, and `create_client` calls
`require_client_retry_binding` before `mutation`. The client-write commit
`b60962c` is an ancestor of `03482b5`. This checkout's `src/cli.rs` and
`src/core.rs` were compared with that commit and do not contain those
client-write strings. The global flags exist in this checkout, and the
create below passes them. Section 7 already passes the same pair on
`riauthctl client update`. This section prints `client create` only.

`agent create` follows the scoped-mutation rule. An agent or a delegated
human sends `If-Match` with the current revision. The administrator
session from section 2 can call the printed `agent create` without
`--if-revision`.

### Commands

Write `deployment-private/platform-lab/ldap-settings.json`. The object is
`ProviderSettings`. `ldap` has `base_dn` and `search_groups` only.

```json
{
  "ldap": {
    "base_dn": "dc=lab,dc=test",
    "search_groups": ["staff"]
  }
}
```

`base_dn` is comma-separated `dc=` labels. Each label is 1–63 ASCII letters,
digits, or hyphens, and the whole base is at most 253 bytes. `search_groups`
has one to 32 names. The client must be interactive and its scopes must
include `profile`.

```sh
riauth schema provider
riauth --server http://localhost:9000 revision
riauth --server http://localhost:9000 \
  --idempotency-key legacy-directory-create \
  --if-revision '<revision>' \
  client create legacy-directory \
  --name "Legacy application directory" \
  --scope openid,profile,email,groups \
  --group staff \
  --settings-file deployment-private/platform-lab/ldap-settings.json
riauth --server http://localhost:9000 agent create ldap-directory \
  --permission ldap.search=client/legacy-directory \
  --out deployment-private/platform-lab/ldap-agent.json
```

`--service` is omitted. A service client fails LDAP settings validation
with `LDAP requires a simple dc=... base, one to 32 search groups and an interactive policy client`.
`--require-mfa` is omitted, so the flag stays false. The
[LDAP provider](ldap-provider.md) example sets `require_mfa` true, which
rejects a password bind until the account has the required factor.
`--group staff` sets `allowed_groups` to that one group. A user bind is
allowed when the account's durable groups intersect that set. The same
group name is the `search_groups` entry above.

`agent create` writes the credential file at Unix mode 0600 and prints
`agent` and `credential_file`. The token is inside that file. It is prefixed
`ri_agent_`. The default lifetime is 86400 seconds. An existing `--out`
fails with `Credential destination already exists`.

Stop `serve` and append the listener. `allowed_peers` is one to 128
explicit IP addresses. Unspecified and multicast addresses are rejected
with `LDAP listeners require one to 128 explicit peer IPs`. At most 16
listeners are accepted. `local_unencrypted` is a loopback test switch and
is not this example. With it left false, both TLS files are required, or
validation fails with
`LDAP requires certificate and key files for LDAPS or mandatory STARTTLS`.

```toml
[ldap_listeners.legacy]
listen = "127.0.0.1:1636"
client_id = "legacy-directory"
allowed_peers = ["127.0.0.1"]
ldaps = true
tls_cert_file = "ldap-fullchain.pem"
tls_key_file = "secrets/ldap-key.pem"
```

The [LDAP provider](ldap-provider.md) example listens on `0.0.0.0:1636` and
allows the documentation peer `192.0.2.20`. This lab uses the loopback
address for both the listen address and the only allowed peer. Replace
`127.0.0.1` in `allowed_peers` with the application host's address when the
client is not on this machine. The listener accepts the TCP peer address.
It does not read an HTTP forwarding header.

Start the same `serve` command. With `ldaps = true` the port speaks LDAPS.
With `ldaps` false and the same certificate files, the port speaks LDAP and
requires STARTTLS before binds and searches. Certificate files are reloaded
on a 60-second interval. A failed reload keeps the active certificate and logs
`LDAP certificate reload failed; retaining current certificate`.

The listener bind does not look up the policy client. A later bind or
search calls `ldap_profile`. A missing, disabled, or non-LDAP client makes
that lookup fail. The LDAP result message is empty. A bind in that state
reports invalid credentials. A search reports insufficient access rights.

A service bind is `cn=riauth-agent,dc=lab,dc=test`. The password is the
agent token. A user bind is `uid=<username>,ou=users,dc=lab,dc=test` with
that account's password. Group entries use `cn=<group>,ou=groups,<base>`.
The attribute and paging limits are on the LDAP provider page. This task
opened no LDAP connection.

### Expected observation

`client create` prints `client` and `client_secret`. This client is public,
so `client_secret` is null. The client is enabled. `agent create` prints
the agent view and the credential path. The token is only in the credential
file.

After `serve` restarts with the listener block and readable certificate
files, the process binds `127.0.0.1:1636`. A peer whose IP is not in
`allowed_peers` is dropped. A bind or search before the client exists, or
while it is disabled, fails closed. A bind reports invalid credentials. A
search reports insufficient access rights. The LDAP message is empty. A
search group that is not a stored group fails inside the server with
`LDAP search group does not exist`. That search reports inappropriate
matching, still with an empty message, and a bind reports invalid
credentials.

### Unrun and peer evidence

This task did not write the settings file, did not run `revision` or
`client create`, did not create the agent, did not add the listener, did
not restart `serve`, and did not open an LDAP connection. No `ldap3` client
and no third-party directory client was run. The automated network fixtures
described on the LDAP provider page are tests in this repository. They are
not a result from this lab. Section 9's directory import remains a separate,
also unrun, procedure.

## Unverified architecture artifacts

These artifacts are in the tree. This slice did not execute them. The small
Platform install above does not configure the extra surfaces they draw.

| Artifact | What it is | What this slice can say |
| --- | --- | --- |
| [Architecture system map](architecture.md) | A combined diagram of browsers, the CLI, nginx and Traefik, HTTP, LDAP and RADIUS listeners, redb or PostgreSQL, and external peers. | It is a map of the codebase's surfaces. Sections 9 and 10 document a directory client and an outbound SCIM target. Section 13 documents one LDAP provider listener. Those commands were not run, and the listener was not started. RADIUS and proxy listeners stay unset. The diagram was not run against this install. |
| [Q08 exact edition matrix](roadmap/q08-exact-edition-bundles.md) | A local build observation at source revision `4ca7558`, including native binary hashes and a note that Linux release files were absent. | It is not an observation of this worktree's revision. This task did not rebuild the matrix. |
| Release archives named by [deployment examples](deployment-examples.md) and [release notes](release-notes.md) | Linux native archives, maintenance archives, `riauthctl` archives, container archives, `SHA256SUMS`, and `build-provenance` files. | They were not downloaded, loaded, or started here. `deploy/compose-small.yml` and `deploy/compose-distributed.yml` remain documented image layouts for a later deployment. |
| `riauth capabilities` | Artifact catalog for the binary on `PATH`. | `usable` is null until a configured instance reports runtime state. Compiled Platform features are not configured features. |

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
deployment. The Platform admin page and workflow editor are not part of that
spec and were not checked here.

## What remains for D01

This page is the Platform half of the task guide through the third slice.
It does not finish the Platform guide, and it does not finish D01.

Sections 6 through 13 are source-reviewed procedures. The generated `init`
file still has no directory, no SCIM target, no workflow, no SAML client,
and no LDAP listener until the operator adds them. Those commands were not
executed here. Sections 11 through 13 name the configured workflow, the SAML
identity-provider and source commands, and the LDAP provider listener.
No workflow run, service provider, upstream identity provider, or LDAP
client was contacted.

Still outside this slice, as later tasks:

- Invitations, email verification, password change and reset,
  authenticator-app enrollment, recovery codes, session list, and consent
  withdrawal. The portal markup includes password, authenticator-app, and
  sessions controls beside passkeys
  ([index.html](../src/portal/index.html)). [Account email and recovery](lifecycle.md)
  describes those browser pages. This guide slice did not run the pages.
  [Re-enrollment and user communication](reenrollment.md) cites the source
  and the browser-management tests for authenticator-app enrollment and
  recovery-code creation.
- PostgreSQL and the two-host layout in [deployment examples](deployment-examples.md),
  native HTTPS, and a trusted proxy.
- Platform additions that this install does not configure: RADIUS
  ([RADIUS](radius.md)), inbound SCIM ([SCIM](scim.md)), proxy SSO
  ([proxy](proxy.md)), Workspace and Entra directory import
  ([Google Workspace](enterprise/ENT-03.md),
  [Microsoft Entra](enterprise/ENT-04.md)), device trust, Windows login,
  temporary access, delegated review, Shared Signals, client-certificate
  login, external signing, and the event map. The enterprise notes linked
  from the [documentation index](README.md) are the current references.
  They are not steps in this slice. Configured TOTP, recovery-code, passkey,
  consent, and enrollment workflows remain in
  [Left to later work](workflows.md#left-to-later-work).
- Edition transition preflight, plan, and activate. The commands exist on
  the Platform maintenance binary and are documented in
  [server editions](editions.md). This slice does not switch editions.
- The [capability and compatibility matrix](capability-matrix.md) records
  compiled inclusion, protocol direction, and the peers named by tests.
  The [Platform forward-auth recipe](recipes/platform-forward-auth.md),
  the [Platform LDAP-provider recipe](recipes/platform-ldap-provider.md),
  the [OIDC relying-party recipe](recipes/oidc-relying-party.md),
  the [Platform SAML IdP recipe](recipes/platform-saml-idp.md),
  the [LDAP import recipe](recipes/ldap-import.md),
  the [upstream OIDC recipe](recipes/upstream-oidc.md),
  the [inbound SCIM recipe](recipes/platform-inbound-scim.md),
  and the [SAML source recipe](recipes/platform-saml-source.md)
  are the D03 recipes written so far. The SAML source recipe follows
  the in-process `Upstream` helper, and no named external IdP is
  connected. The LDAP import recipe and the
  upstream OIDC recipe cover both editions. The relying-party recipe is the
  authorization-code fixture, and its client is the in-tree router rather
  than a named application. The SAML IdP recipe follows the xmlsec1 fixture.
  A named service provider remains an open peer. SAML source remains a
  separate profile. The LDAP provider recipe is separate from LDAP import.
  Other D03 recipes and acceptance against category targets (D05) remain
  open. The first D04 decision page is
  [operational recovery](operational-recovery.md); further emergency runbooks
  remain open. The [connector dependency incidents](connector-incidents.md) page covers
  read, stop, and retry decisions for external dependencies. The
  [administrator lockout](admin-lockout.md) page covers a serving store when
  a second human administrator can still sign in. The backup commands here
  are entry points.
- Any claim that a person completed install, sign-in, the OIDC redirect,
  passkey enrollment, backup, restore, the group membership, the claim
  preview, the audit export, an LDAP directory plan or apply, a SCIM plan
  or apply, a workflow plan or a configured-workflow run, a SAML metadata
  exchange with a peer, or an LDAP provider bind on this revision. Those
  claims need a run. This page does not supply one.
