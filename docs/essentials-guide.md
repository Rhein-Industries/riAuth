# Essentials guide: first tasks for a new operator and user

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D01
`a96a1977-3210-4284-8f7d-645793369301`.

This is the first slice of the Essentials task guide. It walks one small
loopback install, the first administrator sign-in, one confidential OpenID
Connect web application, passkey self-service in the browser, and the backup
and recovery commands that open a restored store. The commands are the ones
implemented in this tree: `[features]` in [Cargo.toml](../Cargo.toml), the
[server CLI](../src/cli.rs), [offline maintenance](../src/cli/local.rs), and
the [standalone client](../crates/riauthctl/src/main.rs).

Reading this page does not mean those steps were executed here. This slice was
checked by reading the source and the current docs, then by
`python3 scripts/check-docs.py`. No Cargo build was run, no server was
started, and no browser, backup, or accessibility pass was recorded. The
[A01 coverage inventory](roadmap/coverage-inventory.md) still describes D01
against revision `96e23e2`, when editions were not in the tree. That row was
left as historical planning evidence.

A new Platform operator uses the [Platform guide](platform-guide.md) for the
same five tasks with the Platform binaries. Assembly limits that are outside
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

This page is the Essentials half of the first slice. It does not finish the
Essentials guide, and it does not finish D01.

Still outside this slice:

- Groups, claims, audit review, invitations, email verification, password
  change and reset, authenticator-app enrollment, recovery codes, session
  list, and consent withdrawal as operator or user tasks. The portal markup
  includes password, authenticator-app, and sessions controls beside passkeys
  ([index.html](../src/portal/index.html)). [Account email and recovery](lifecycle.md)
  both describes those browser pages and still says authenticator-app
  enrollment and recovery-code rotation need the terminal. The pages were
  not run here, so that conflict stays unresolved.
- LDAP import and outbound SCIM, which the Essentials contract includes and
  this small install does not configure.
- PostgreSQL, native HTTPS, a trusted proxy, and the released-image small
  layout in [deployment examples](deployment-examples.md).
- The Platform protocols and administration in the [Platform guide](platform-guide.md).
- The [capability and compatibility matrix](capability-matrix.md) records
  compiled inclusion, protocol direction, and the peers named by tests.
  Tested integration recipes (D03), emergency runbooks beyond the entry
  points above (D04), and acceptance against category targets (D05) are
  still open.
- Any claim that a person completed install, sign-in, the OIDC redirect,
  passkey enrollment, backup, or restore on this revision. Those claims need
  a run. This slice does not supply one.
