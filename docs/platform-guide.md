# Platform guide: first tasks for a new operator and user

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D01
`a96a1977-3210-4284-8f7d-645793369301`.

This is the Platform task guide through its third slice, plus the
invitation-acceptance page in section 14. It walks the same tasks as the
[Essentials guide](essentials-guide.md): one small loopback install, the
first administrator sign-in, one confidential OpenID Connect web application,
passkey self-service in the browser, and the backup and recovery commands
that open a restored store, then a local group, claims for that application,
an audit review, one LDAP directory import, and one outbound SCIM target.
Sections 11 through 13 add three Platform-only procedures: one configured
password workflow, one SAML service provider and one SAML source, and one
LDAP provider listener. Section 14 is the same browser invitation contract
as Essentials section 11. The install in section 1 selects the Platform
build. The walkthrough records nine loopback observations. The first
reported the Essentials catalog. The second was a copied Platform debug
server whose artifact catalog had `edition` `platform` and `build_features`
`["essentials", "platform"]`. The third used that same server snapshot with
separate `riauthctl` and `riauth-maintenance` snapshots supplied for a later
source revision. The fourth used only that server snapshot and ran the
section 5 backup, restore, and recovery-status entry points. The fifth used
only that server snapshot, opened an isolated browser at `/apps`, and stopped
the section 4 passkey ceremony before a credential was stored. The sixth used
only that server snapshot, captured one invitation on a loopback SMTP sink,
accepted the password in an isolated browser, and signed in only after that
acceptance. The seventh used only that server snapshot and ran the section 11
schema, validate, plan, apply, runtime `[workflows.local-password]` restart,
configured start, and export. It did not submit the password step. The eighth
used only that server snapshot and ran section 9 `directory list`,
`directory plan`, and `directory apply` against one disposable loopback
OpenLDAP listener. The ninth used only that server snapshot and ran section
10 `provision targets`, `provision plan`, `provision apply`, and `provision
jobs` against one disposable loopback SCIM fixture. The commands in sections
1 through 13 are the ones
implemented in this tree: `[features]` in
[Cargo.toml](../Cargo.toml), the [server CLI](../src/cli.rs),
[offline maintenance](../src/cli/local.rs), and the
[standalone client](../crates/riauthctl/src/main.rs).

The three slices were checked by reading the source and the current docs,
then by `python3 scripts/check-docs.py`. Section 14's password acceptance
was run once on loopback. Nine disposable loopback runs are recorded in
[Platform CLI walkthrough](roadmap/d01-platform-cli-walkthrough.md). No Cargo
build was run for any of them. The first stopped after one `local-demo`
client create on the Essentials catalog. The second repeated setup on the
Platform catalog and then ran the server CLI client, group, claim, and audit
commands that `riauth` implements. It skipped the printed `riauthctl` lines,
sections 4 and 5, and `group get` / `group has-member`. The third ran
`riauth-maintenance init` and the printed `riauthctl` client, group, and
claim commands, then the server CLI `explain` and audit commands. `riauthctl`
has no `doctor` subcommand. The fourth ran section 5 `keygen`, `backup`,
`restore`, and `recovery status` on the server snapshot. It left `recovery
complete`, `recover-admin`, and a second server unrun. The fifth signed in
at `/apps` with the password form, opened **Sign-in and security**, and
cancelled **Add a passkey** when Chrome required iCloud Keychain, the Chrome
profile, a USB security key, or Touch ID. The sixth appended a loopback
`[mail]` table, issued one invitation with `--idempotency-key` and
`--if-revision`, opened the captured link, accepted a password, left `/apps`
signed out, refused a replay of that link, and then signed in as the invited
person. The seventh ran section 11 `schema`, `validate`, `plan`, and `apply`,
restarted serve with `[workflows.local-password]`, started that configured
workflow, and ran `export`. The password step was not submitted. The
eighth created the local group `staff`, appended `[directories.staff]` with
`transport` `starttls`, and ran `directory list`, `directory plan`, and
`directory apply` against a disposable loopback OpenLDAP listener. The
ninth created local user `quinn`, added `admin` and `quinn` to `staff`, and
ran `provision targets`, `provision plan`, `provision apply`, and `provision
jobs` against a disposable loopback SCIM fixture. Rename,
remove, passkey sign-in, passkey invitation acceptance, hardware, peers, and
Essentials-guide execution remain unrun. The
[A01 coverage inventory](roadmap/coverage-inventory.md) still describes D01
against revision `96e23e2`, when editions were not in the tree. That row was
left as historical planning evidence.

The install below leaves PostgreSQL, Workspace, Entra, RADIUS, proxy, and
the other Platform protocols unset. Sections 9 and 10 add the shared LDAP
import and outbound SCIM steps on this Platform server. Sections 11 through
13 add the configured-workflow, SAML, and LDAP provider procedures. Section
11 was executed once on loopback. Section 9 was executed once on loopback
against a disposable OpenLDAP listener. Section 10 was executed once on
loopback against a disposable SCIM fixture. Sections 12 and 13 were read
from this tree and were not executed. Assembly and
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
([capability.rs](../src/capability.rs)). After the Platform install above,
`edition` is `platform`, `build_features` is `["essentials", "platform"]`,
and `scope` is `artifact`. Every `feature_states` entry has `usable` set to
`null`. That catalog describes the binary. It does not say that a peer, a
browser, or an authenticator is healthy, and it does not mean the extra
Platform features are configured.

The first loopback record ran this command on an Essentials debug binary.
That catalog reported `edition` `essentials`, `build_features`
`["essentials"]`, and `scope` `artifact`. All 87 `feature_states` entries
had `usable`, `enabled`, `configured`, and `runtime_ready` set to `null`.
Sixty had `compiled` true and 27 had `compiled` false. The later
Platform-catalog run copied a supplied debug snapshot and ran the same
command on the copy. That catalog reported `edition` `platform`,
`build_features` `["essentials", "platform"]`, and `scope` `artifact`. All
87 entries again had those four fields null, and all 87 had `compiled`
true. The remote-administration run copied that same server snapshot again.
Its `riauth capabilities` document matched the second run, including a
standard output of 20679 bytes. Its `riauthctl` and `riauth-maintenance`
copies printed version `0.1.1` and did not print an edition. Those two
snapshots were supplied for source revision `f430c2f`; the server snapshot
was supplied for `58357fd`. The Rust source that changed between those
revisions is provisioning and reconciliation, plus the reconciliation
completion test. Documentation commits sit in that range as well, and
`9c374be` is newer than both snapshots. This task did not run `cargo install`,
and these three files were not produced as one install. The Platform fields
above are the server copy's document, not an installed release binary.

The install command above is one set of Platform binaries plus the remote
client. The instance created in the next section is loopback redb, with the
issuer `http://localhost:9000` and the listener `127.0.0.1:9000`. `init` fills the
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
`data` beside that file, and records the store as the running binary's
edition. The Platform install above records Platform. Relative paths
are resolved from the configuration file's directory. The generated files are
under `deployment-private/`, which this repository ignores.
`reviewed_client_creation` defaults to false, so the application created
later does not wait for a creation review. The first two loopback records
used `riauth init` with these arguments plus `--password-stdin` and
`--non-interactive`. Each generated file had `reviewed_client_creation = false`.
Each `doctor` result listed `storage` `redb` and `healthy` true, with no
edition field. Those two runs did not launch `riauth-maintenance`. The
remote-administration run used the printed `riauth-maintenance init` with
the same issuer, listener, data directory, and admin name, plus
`--password-stdin` and `--non-interactive`. Its generated file had the same
`reviewed_client_creation = false`. Its `doctor` result again listed
`storage` `redb` and `healthy` true, with no edition field. The maintenance
binary printed `riauth-maintenance 0.1.1` and did not print an edition.

Leave the service running in this terminal:

```sh
riauth --config deployment-private/platform-lab/riauth.toml serve
```

In another terminal:

```sh
curl --fail http://127.0.0.1:9000/readyz
```

`/readyz` checks storage readiness. A passing probe is the entry check for
this slice. It is not a production cutover. The Essentials-catalog run
reported `duties.protocol_listeners` false. The Platform-catalog run and the
later remote-administration run each reported it true, with `authentication`
true, `background_jobs` true, `role` `integrated`, and `status` `ok`. Both
of those generated `proxy_listeners`, `radius_listeners`, and
`ldap_listeners` tables were empty.
[process_role.rs](../src/process_role.rs) sets that duty on an integrated
Platform process. The flag does not mean a listener stanza was added, and
it does not identify the maintenance binary's edition.

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
`doctor` uses that session. Keep the file private. All three loopback records
passed `--session-file` and `--password-stdin` for this server CLI login, and
the default session path stayed absent. The remote-administration run also
saved the riauthctl session with `--session-file`. `~/.config/riauthctl/session.json`
stayed absent. `riauthctl doctor` is not a command on that client. The copy
exited 2 with `unrecognized subcommand 'doctor'` and sent no request.

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

`riauth client create` is the server CLI, a separate command. Without both
`--idempotency-key` and `--if-revision` it stops locally with the message in
section 13, before it sends the request. A confidential client then needs
`--output-file` or `--show-secrets`. That command has no `--secret-file`.
With `--output-file`, standard output names `output_file` and `written`, and
the secret stays in the private file. The first two loopback records created
`local-demo` that way. The remote-administration run used the `riauthctl`
command printed above, with `--session-file` and `--non-interactive`, and
without `--idempotency-key`, `--if-revision`, or `--run-id`. It exited 0.
Standard output named `credential_file` and omitted the secret. The secret
file was mode `0600`, and the secret length was 53. The audit row for that
create has `run_id` null. The same run then executed `discovery` and
`whoami`. `whoami` ran before the group commands, so `groups` was empty.
`riauthctl` printed `password_available` as the string `[redacted]` because
the field name contains `password`
([riauthctl main.rs](../crates/riauthctl/src/main.rs)). The server CLI login
in that run printed the boolean true. No run started the application on port
3000. The Platform-catalog run continued at sections 6 through 8 with the
server CLI and skipped sections 4 and 5. The remote-administration run also
skipped sections 4 and 5, then continued with the printed `riauthctl` group
and claim commands.

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

### Loopback observation

The [walkthrough](roadmap/d01-platform-cli-walkthrough.md) records one
isolated-browser attempt on a copy of the Platform server snapshot supplied
for `58357fd`. Password sign-in at <http://localhost:9000/apps> opened the
applications page for the local `admin` account. **Sign in with a passkey**
was already in view on that first visit, before any passkey existed.
**Sign-in and security** reported **You have no passkeys yet.** The name
field was changed to `loopback-lab`, and **Add a passkey** opened the
browser prompt.

Chrome showed a sheet titled **Add a passkey?** The sheet said that
`localhost` supports passkeys and that a passkey for `admin` would be saved
in Passwords, and it named Touch ID as the save action. Escape on that sheet
opened **Choose where to save your passkey for localhost**. The choices were
iCloud Keychain, Your Chrome profile, and USB security key. Cancel on that
chooser returned the dialog to **Add a passkey**. The status text was
`Adding the passkey was cancelled or timed out. Select the add button to try
again, or Cancel change.` The name field still contained `loopback-lab`. The
dialog had no passkey row, so **Rename** and **Remove** had nothing to
change.

The same account's server CLI `passkey list` returned `[]`. iCloud Keychain,
the Chrome profile, Touch ID, and a USB security key stayed unused. `cargo
install` and the printed `riauthctl passkey` commands stayed unrun.

The printed commands, portal routes, and enrollment and removal rules in
this section match the tree. Browser enrollment asks for a resident
credential with user verification. The terminal USB registration path asks
with the non-resident flag, and that flag is advisory. Server `passkey enroll`
and `passkey login` stop before a ceremony because USB support is not compiled
into the server binary. The two-passkey sentence above covers this setup
administrator. The same conflict also covers a later administrator whose
password hash is empty. Step 6 describes **Sign in with a passkey** on a
later visit. This run also saw that button on the first unsigned visit. The
page shows it when `identity.passkeys` is usable and the browser can use
passkeys, including when the account has no passkey yet.

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

### Loopback observation

The [walkthrough](roadmap/d01-platform-cli-walkthrough.md) records one kept
run of these entry points on a copy of the Platform server snapshot supplied
for `58357fd`. `riauth-maintenance` was not in that run. The copy's help
lists `keygen --out`, `backup --key-file` and `--out`, and `restore
--backup`, `--key-file`, and `--out`. `recovery status` is read-only.
`restore` help also lists `--postgres-config` and `--database-key-file`.
The run omitted both.

`riauth keygen` wrote a new private backup key. `riauth backup` ran while
`riauth serve` was up, on the server CLI session from this lab. The result
was `api_version` `riauth.backup/v3`, `encrypted` true, and `verified` true.
The archive was mode `600` and its size matched the reported byte count,
12489. The transcript and stream id are omitted here. `verified` is the
archive check in [stream.rs](../src/operations/stream.rs): frames, the
trailer transcript of the bytes before the trailer, and the schema and
issuer checks. It does not mean a person signed in.

Serve was stopped before restore. `riauth restore` wrote a new directory and
returned `verified` true, `storage` `redb`, `encrypted_at_rest` false, and
`serving_allowed` false. `riauth recovery status` on that restored
configuration left service closed. The pending cause was `backup_restore`,
the policy was `riauth.recovery/v1`, `schema` was 3, history was empty, and
recorded and observed lineage were null. Reconcile listed `passwords`,
`enabled_accounts`, and `signing_keys` as counts. Those counts are the
restored credentials the policy names. They are not a rotation, and they are
not the `--persistent-credentials-reconciled` attestation.

`recovery complete`, `recovery invalidate`, `recover-admin`, a second
server, and PostgreSQL stayed unrun. The restored issuer was not started.
This run does not call the result a recovery drill.

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
`staff`. Both verbs are `riauthctl` commands.

The server CLI group commands are `review`, `list`, `create`, `add-member`,
and `remove-member` ([cli.rs](../src/cli.rs)). It has no `group get` or
`group has-member`. `riauth group create` and `riauth group add-member` stop
locally unless both `--idempotency-key` and `--if-revision` are present. The
message is:
`Group writes require --idempotency-key and --if-revision (from riauth revision)`.
The source string wraps `riauth revision` in backticks. The printed commands
in this section stay on `riauthctl`. The Platform-catalog run used the server
CLI instead. `riauth group --help` printed the five commands above.
`group create staff` without the two flags exited 1 with empty output and
that message. A later create at revision 1 and `group add-member staff admin`
at revision 2 both exited 0. The create response had no members. The
add-member response named `staff` and had one member. That server-CLI run
did not execute `group get` or `group has-member`.

The remote-administration run used the printed `riauthctl` commands, with
`--session-file` and `--non-interactive`. `riauthctl group --help` on that
copy listed `list`, `get`, `has-member`, `create`, `add-member`, and
`remove-member`. Create ran at revision 1 with key `staff-create`. Add-member
ran at revision 2 with key `staff-add-admin`. Both exited 0. The create
response named `staff` and had no members. The add-member response named
`staff` and had one member. `group get staff` exited 0 and returned `staff`
with one member. `group has-member staff admin` exited 0 and returned
`member` true.

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

The Platform-catalog run performed this update with `riauth client update`,
not the printed `riauthctl` line. The settings file was the object above.
The revision read was 3 and the key was `local-demo-claims`. The stored
scopes were `groups`, `openid`, and `profile`. `claim_mappings` kept the
literal `department` value `lab`, and `claims_in_access_token` stayed false.
The following `explain` exited 0 with `simulation` true, `token_issued`
false, `allowed` true, empty `reasons`, `userinfo.groups` containing
`staff`, `userinfo.department` `lab`, and both `userinfo.name` and
`userinfo.preferred_username` equal to `admin`. `userinfo` also contained
`sub`. `access_token_identity_claims` contained only `sub`.

The remote-administration run used the printed `riauthctl client update`.
The revision read was 3 and the key was `local-demo-claims`. The stored
scopes were `groups`, `openid`, and `profile`. `claim_mappings` kept the
literal `department` value `lab`. `riauthctl` printed
`claims_in_access_token` as the string `[redacted]` because the field name
ends in `_token`. The settings file did not set that field, and the boolean
was not visible in the riauthctl output. The following `explain` matched the
claim preview above: `simulation` true, `token_issued` false, `allowed`
true, empty `reasons`, `userinfo.groups` containing `staff`,
`userinfo.department` `lab`, and access-token identity claims containing
only `sub`.

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
`next_cursor`, `limit`, and `revision`. For the audit collection, `--filter` matches
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

The Platform-catalog run executed these three `riauth` commands with
`--session-file`. `audit --limit 100` returned six rows, including
`group.create` and `group.member.add` for `run_id` `staff-group`,
`client.create` for `local-demo`, and `client.update` for
`local-demo-claims`. `inventory audit --filter staff-group` returned those
two group rows, `next_cursor` null, `limit` 100, and `revision` 4.
`report audit` wrote two data rows under the header
`id,at,actor,action,target,run_id,request_id`, mode `0600`. The `--out`
path was a lab file, not `deployment-private/platform-lab/audit-staff-group.csv`,
and the file was removed with the lab. Row `details` were not copied into
the record.

The remote-administration run executed the same three commands on the server
CLI session. `audit --limit 100` returned seven rows. `client.create` for
`local-demo` had `run_id` null. The two group rows used `run_id`
`staff-group`, and `client.update` used `local-demo-claims`. Two
`login.succeeded` rows were present, one for each CLI login. `inventory
audit --filter staff-group` returned the two group rows, `next_cursor`
null, `limit` 100, and `revision` 4. `report audit` wrote two data rows
under the same header, mode `0600`, to a lab file that was removed with the
lab. Row `details` were not copied.

## 9. Import one LDAP directory

`init` writes an empty `[directories]` table and no directory id. LDAP import is a later edit of the
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

### Loopback observation

The [walkthrough](roadmap/d01-platform-cli-walkthrough.md) records one
section 9 run on a copy of the Platform server snapshot supplied for
`58357fd`. No Cargo build was run. The copy's `riauth capabilities` catalog
had `edition` `platform` and `build_features` `essentials` and `platform`.
`riauth --version` printed `riauth 0.1.1`. Those two commands are local.
The printed commands in this section do not include them.

`init` wrote empty `[directories]`, `[workspace_directories]`,
`[entra_directories]`, `[scim_targets]`, `[ldap_listeners]`,
`[proxy_listeners]`, `[radius_listeners]`, and `[signers]` tables. It wrote
no directory id. The local group `staff` was created with the server CLI
while `serve` was up, with `--idempotency-key` and `--if-revision`. Its
member count was 0. Section 6's `add-member` was left unrun, so `admin` was
not a member of `staff`.

Serve was stopped. The printed `[directories.staff]` block was appended with
`transport` `starttls`, the printed bases and filters, `password_file`
`ldap-password`, and `ca_file` `ca.crt`. The URL was
`ldap://127.0.0.1:60523`. The documented host `directory.example.test` stays
the shape in the example above. `/etc/hosts` was not edited. The certificate
was a one-day disposable CA and leaf, CN `localhost`, with SAN
`DNS:localhost` and `IP:127.0.0.1`. The same serve command was started again.

The directory was Homebrew OpenLDAP `slapd` 2.7.1 on that loopback URL.
`memberof` is compiled into that binary. Exported `memberof` symbols were 0,
and the only on-disk memberof file was the man page `slapo-memberof.5`.
The fixture configuration loaded `overlay memberof`. The built-in `memberOf`
attribute is operational and not user-modifiable, so the LDIF left it unset.
Adding `groupOfNames` `cn=staff` with member
`uid=alice,ou=people,dc=example,dc=test` made a plaintext search show
`memberOf` equal to `cn=staff,ou=groups,dc=example,dc=test`. `entryUUID` was
present and is not copied. The population client was plaintext `ldapadd` on
the same listener. The fixture root DN was the printed bind DN, with no
person entry for that DN. `scripts/test-ldap.sh` was not run.

`directory list` showed id `staff`, that URL, user base
`ou=people,dc=example,dc=test`, group `staff`, and reconciliation mode
`manual-review`. `directory plan` exited 0. Standard output had plan id
`3f9ffb68-406f-472a-8d78-69116899b0f0`, revision 1, and one `create` for
`alice` in group `staff`. It omitted `removal_impact`. The plan file's
`removal_impact.review_required` was false. Disabled users, missing users,
and removed memberships were 0. `directory apply` omitted
`--confirm-removals`. It printed `applied` true and the same change.
`user list` then showed `alice` enabled, with `admin` false,
`password_available` false, and `email_verified` false. `group list` showed
`staff` with one member, and that member was `alice`. The member value is a
user id and is not copied. No directory password login was attempted.

An OpenSSL STARTTLS probe of the same listener reported verification OK and
TLS 1.3. Apple `ldapsearch` 2.4.28 with `-ZZ` exited 1 and printed
`ldap_start_tls: Connect error (-11)`. That probe is not a printed command.
The plan and apply above are the riAuth client result for `transport`
`starttls`. A failed upgrade fails the plan before accounts change. The
commands came from the immutable snapshot. The walkthrough limits the source
comparison to the files it names.

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
cannot reach fails delivery.

### Loopback observation

One run used a copy of the Platform server snapshot supplied for `58357fd`.
`init` wrote an empty `[scim_targets]` table. The run created local group
`staff` and local user `quinn` (`enabled` true, `admin` false), then added
both `admin` and `quinn` to `staff`. It stopped `serve`, appended
`[scim_targets.payroll]` with `export_groups` true and no
`scim_reconciliation_modes` entry, and started `serve` again. The token file
was mode `0600`. The configuration did not contain the token. `provision
targets` showed reconciliation mode `manual-review`.

`provision plan` wrote plan `c0c69a62-6ffa-4477-90c9-ff95878a0269` at
revision 4. Standard output omitted `removal_impact`. The plan file had
`review_required` false and zero disabled users, missing users, and removed
memberships, so apply omitted `--confirm-removals`. The plan's only user was
`quinn`. Its `staff` member list was `quinn`, even though the local group
also contained `admin`.

The peer was a disposable loopback HTTP fixture at
`http://127.0.0.1:56148/scim/v2`, not a named directory and not a SCIM
conformance suite. It recorded `GET /scim/v2/Users` and `GET /scim/v2/Groups`
with `count=2`, each returning no resources, then `POST /scim/v2/Users`
status 201 for `quinn` and `POST /scim/v2/Groups` status 201 for `staff`.
The group member value was the fixture user id `fixture-user-1`. No delivered
user name was `admin`. `provision jobs` then showed `delivery_state`
`succeeded`, `completed` true, and processed 2 of 2. The walkthrough records
the requests and the cleanup. Sections 12 and 13 were not part of this run.

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
groups, clients, sources, source links, and delegated grants stay as they are. `issuer` binds
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
source links, delegated grants, and secrets, with `Workflow editor applies one workflow only`.
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

### Loopback observation

The [walkthrough](roadmap/d01-platform-cli-walkthrough.md) records one
`local-password` run on a copy of the Platform server snapshot supplied for
`58357fd`. No Cargo build was run. The copy's `riauth capabilities` catalog
had `edition` `platform` and `build_features` `essentials` and `platform`.
`riauth --version` printed `riauth 0.1.1`. `capabilities` is a local catalog
read. The printed commands in this section do not include it.

`schema workflow` and `schema manifest` exited 0. The schema titles were
`Definition` and `Manifest`. `validate` printed `valid` true, `validation`
`local_schema`, `resources` 0, and `secret_values_read` false. `plan` exited
0 with one create for `workflow/local-password`, `base_revision` 0,
`reconciliation_mode` `manual-review`, and `removal_impact.review_required`
false. Disabled users, missing users, and removed memberships were 0.
`credential_change` was false and the change had no secret references.
`apply` did not pass `--confirm-removals`. It printed `applied` true,
`changed` true, revision 0, and no `run_id`. The plan id was
`2cc4d8e3-5758-4094-ade0-8aaaf4c29447`. The plan `hash` failed a
64-lowercase-hexadecimal check, so it is not copied. Snapshot-equal
`crypto::digest` encodes SHA-256 as URL-safe unpadded base64.

Serve was stopped, the printed `[workflows.local-password]` block was
appended with `active = true`, and the same serve command was started again.
`POST /api/workflows/configured/local-password` used the administrator bearer
and an empty body. The response was HTTP 200, set no cookie, and returned a
view whose `binding.workflow` was `local-password`, `binding.revision` was
1, and `state` was active on step `password` at attempt 1. `attempts_used`
was 0 and `executions` was 0. The run id was
`00453fc4-2cb5-4db2-b2b9-010368c95596`. The binding fingerprint was
`0fbbab5fadf1019195b415112725a6fb62f794c0e2c41e8ea7a2fc22a8865f61`.
No source registration and no extension hash were present. `export` printed
`secrets_included` false and revision 0. The exported manifest had one
`local-password` workflow at revision 1, and its `issuer` was null.

The password continuation, the run lookup, and cancel were not called. No
audit command was run, so no audit row was recorded. The browser editor was
not opened. TOTP, recovery-code, and passkey configured shapes are described
in
[Local password reauthentication](workflows.md#local-password-reauthentication)
and were not added here. The names that fail adapter validation were not
used. The commands came from the immutable snapshot. The walkthrough limits
the source comparison to the files it names.

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

The server CLI stops when either flag is missing. The message is:
`Signing-key configuration requires --idempotency-key and --if-revision (from riauth revision)`.
The source string wraps `riauth revision` in backticks. The HTTP writer
returns `Signing-key configuration requires Idempotency-Key and If-Match`.
In this tree, the CLI gate in [src/cli.rs](../src/cli.rs) covers `keys import`,
`keys bind`, and `keys generate`, and `configure_key` in
[src/assembly/keyring.rs](../src/assembly/keyring.rs) checks the request
context before `mutation`. The imports below pass both flags. This procedure
prints the two imports. `keys bind` and `keys generate` stay unprinted. Neither
loopback record ran a signing-key command.

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

This task did not create keys or metadata, did not run the `revision` or
`keys import` printed in this section, and did not run `saml import-sp`,
`validate`, `plan`, `apply`, `saml metadata`, `source put`, `source list`, or
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

The server CLI stops when either flag is missing, before it chooses a secret
destination or sends the request. The message is:
`Client writes require --idempotency-key and --if-revision (from riauth revision)`.
The source string wraps `riauth revision` in backticks. The HTTP writer
returns `Client writes require Idempotency-Key and If-Match`. In this tree,
`run_client` in [src/cli.rs](../src/cli.rs) covers create, update, disable,
and enable, and `create_client` calls `require_client_retry_binding` in
[src/core.rs](../src/core.rs) before `mutation`. Both loopback records ran
`riauth client create local-demo` once without those flags and received that
message with exit code 1, then created `local-demo` with both flags. That
client is the section 3 application. The Platform-catalog run also updated
it in section 7 with `riauth client update`. It is separate from the
`legacy-directory` command printed below. Section 7's printed command passes
the same pair on `riauthctl client update`. This section prints `client create`
only.

`agent create` requires a human administrator and both a unique
`Idempotency-Key` and the current numeric revision in `If-Match`, including
for the administrator session from section 2. The CLI equivalents are
`--idempotency-key` and `--if-revision`. Read the revision again after
`client create`, since that write advances it.

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
riauth --server http://localhost:9000 revision
riauth --server http://localhost:9000 \
  --idempotency-key ldap-directory-agent-create \
  --if-revision '<current-revision>' \
  agent create ldap-directory \
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
fails with `Credential destination already exists`. The first committed
response is the only disclosure of that token. A server-side exact retry
with the original key, revision and body returns 409
`credential_already_issued` without another token or audit; a CLI retry
needs a new unused `--out` path if the first file exists. If the credential
file was never written, inspect the agent and rotate its credential separately.

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

This task did not write the settings file, did not run the `revision` or
`client create` printed in this section, did not create the agent, did not
add the listener, did not restart `serve`, and did not open an LDAP
connection. No `ldap3` client and no third-party directory client was run.
The automated network fixtures
described on the LDAP provider page are tests in this repository. They are
not a result from this lab. Section 9's directory import remains a separate,
also unrun, procedure.

## 14. Accept an invitation in the browser

This is the same invited person's page as Essentials section 11. It is
separate from the signed-in passkey task in section 4 and from the
configured password workflow in section 11. Issuing an invitation, the
`[mail]` table, and the password-only `riauth account accept` command stay
in [Account email and recovery](lifecycle.md). The enrollment routes and the
browser choice are in [Passkeys](passkeys.md). The headless journeys are
recorded in [Portal](PORTAL.md). This section states the acceptance contract
and does not copy that write-up.

The configuration from section 2 has no `[mail]` table. `identity.invitations`
is compiled on Essentials and on Platform. It becomes usable when `[mail]`
passes the local SMTP material check in `require_local_material`. One
loopback run added a `[mail]` table with `security` set to `loopback` and
host `127.0.0.1`, issued one invitation, and captured the message on that
host. `account invite` and `account revoke-invitation` stop locally unless both
`--idempotency-key` and `--if-revision` are present. The message is
`Invitation writes require --idempotency-key and --if-revision (from riauth revision)`.
The source string wraps `riauth revision` in backticks. This section does not
print an invite.

### Capability gating

On `/account/accept`, a fragment token that starts with `ri_mail_` and is at
most 128 characters shows the password form. Another token hides it. When
the capability refresh reports that `identity.invitations` is not compiled,
the page says the account action is not available and hides the form. A
missing `[mail]` table leaves that feature compiled and unusable, and the
password form still shows. Both editions compile it.

**Add a passkey** starts hidden. The script shows it when the password form
is visible, the fragment token is present, `PublicKeyCredential` exists in a
secure context, and `identity.passkeys` is usable. The form is also marked
`data-capability="identity.passkeys"`, and the portal stylesheet hides a
control with `data-capability-disabled`. Both editions compile
`identity.passkeys`. It has no mail or directory prerequisite. The lab issuer
`http://localhost:9000` is loopback HTTP, which a browser treats as a secure
context, so a browser with WebAuthn can show the choice. When the choice
stays hidden, password acceptance remains.

### One use and no session

Opening the link loads the page. The fragment is not sent with that GET, so
the proof is not spent. The page then removes the fragment from the address
bar and keeps the token only for the later POST.

**Accept invitation** posts the token and the new password to
`/api/portal/account/accept`. **Accept with a passkey** posts the token and
the passkey name to `/api/account/accept/passkey/start`, then the
authenticator response to `/finish`. Cancel posts `/cancel`. A blank passkey
name is refused, and the invitation stays unused. Starting a ceremony does
not spend the invitation. A cancelled or failed authenticator attempt leaves
the invitation available for another start.

A successful password acceptance or passkey finish returns `completed` true
and `login_required` true and retires the proof as used. Platform completes
that result through the shipped invitation workflow. That workflow is not
the configured `local-password` workflow in section 11, and the name
`platform-invitation-password-enrollment` still has no executable adapter.
The workflow request stores an empty session. The handlers return that JSON
and do not set a session cookie. The person signs in afterwards. The password
text says to sign in with the new password, and that an application may also
require a passkey or authenticator code. The passkey text says to sign in
with the new passkey and that this page did not sign them in. The same link
opened again reports that the invitation has already been accepted and hides
both forms. An expired invitation reports that it has expired, hides both
forms, and creates no session. A revoked or replaced link hides the forms
too. Password and passkey spend the same invitation. `riauth account accept`
is the password completion and does not call the passkey endpoints.

### Loopback observation

The [walkthrough](roadmap/d01-platform-cli-walkthrough.md) records one
password acceptance on a copy of the Platform server snapshot supplied for
`58357fd`. After `init`, the configuration gained a `[mail]` table with
host `127.0.0.1`, `security` `loopback`, and no SMTP username. A sink on
that host captured one message. The product mail job delivered it.
`account deliveries` listed one delivered row and no message body.

An invite without `--idempotency-key` and `--if-revision` exited 1 before
a request. Standard output was empty. Standard error matched the local bail
above, and the bytes included the backticks around `riauth revision`.
`riauth revision` printed revision 0. The bound invite exited 0. The new
account was disabled, was not an administrator, had no password, and had an
unverified `example.test` address. The captured link used
`http://localhost:9000/account/accept` and a fragment whose token starts
with `ri_mail_` and is 51 characters. The token is not recorded.

An isolated Chromium window opened that link. The address bar no longer
contained the fragment. The page offered a password and **Accept with a
passkey**. Only the password was submitted. The result was **Invitation
accepted**, **Your account is ready**, and the sentence that says to sign
in with the new password. Server CLI `user list` showed that account enabled,
with the delivered address verified and a password available. Chrome showed
**Save password?** and **Never** was pressed.

`/apps` then showed the sign-in panel, including **Sign in with a passkey**,
**Sign in**, and **Sign in with your terminal**. The signed-in welcome was
absent. Opening the same link again showed the password form. Submitting
the password again showed **This invitation has already been accepted.
Continue to sign in.** and hid both forms. `/apps` was still the sign-in
panel. A later password sign-in opened the catalogue for **Invitee Lab
(@invitee)** with **All applications (0)**. **Sign out** was visible and
was not used.

The success JSON, the session cookie, and `GET /api/portal` were not read.
The driver refused an in-page fetch, so this run does not record an HTTP
status for the portal. The signed-out panel is the evidence that acceptance
had not signed the browser in. The shipped invitation workflow was not
started as an operator workflow run. The Playwright invitation journeys,
Firefox, WebKit, an expired or revoked link, passkey acceptance, sign-out,
and a real mailbox were not run. The pages came from the immutable snapshot.
The walkthrough limits the source comparison to the invitation and mail
files it names.

### Browser test limits

The loopback observation above used one isolated desktop Chromium window.
It did not run these Playwright journeys, and it did not use Firefox or
WebKit.

`tools/browser/invitation-password.spec.js` is a headless keyboard journey
at 390 by 844 CSS pixels on Chromium, Firefox, and WebKit. It uses the
fixture's loopback SMTP capture. It installs no virtual authenticator and
does not post the invitation passkey endpoints, so a visible passkey choice
stays unused. Success sets no session cookie and leaves `GET /api/portal`
unauthorized. An expired link and a replay are rejected. The new password
then signs in, and signing out removes that session.

`tools/browser/invitation-passkey.spec.js` is the CDP passkey journey.
Chromium uses one CDP virtual authenticator. Firefox and WebKit skip it
because that authenticator exists only in Chromium. The virtual
authenticator is not a physical key, a synced passkey, a phone, or a
mobile operating system.

`tools/browser/invitation-passkey-shim.spec.js` is the simulated-credential
journey. Chromium, Firefox, and WebKit each install Playwright's
simulated WebAuthn credential before the page loads. The shim replaces
`navigator.credentials`, generates a P-256 key in the test process, and
sets the user-verified bit itself. It does not prompt, and it is not the
browser's authenticator. An expired invitation and a whitespace name create
no credential. A named passkey returns `completed` true and
`login_required` true, sets no session cookie, and leaves `GET /api/portal`
unauthorized. Replaying the link reports that it was already used. A later
passkey sign-in opens the account, and signing out removes the session. The
server does not prove discoverability. The CDP
ceremony stays in `invitation-passkey.spec.js`.

None of these journeys is an external mailbox, a spoken screen reader, a
phone hybrid, a mobile operating system, or a release. The CI integration
job is written to run the bounded headless allowlist in [Portal](PORTAL.md);
this guide slice did not run that job. The viewport is CSS pixels. The page is
`src/portal/account.js` and `src/portal/account.html`. The password handler
is `src/portal/http.rs`. The passkey routes are `src/api/invitation.rs` and
`src/lifecycle/invitation/passkey.rs`.

## Unverified architecture artifacts

These artifacts are in the tree. This slice did not execute them. The small
Platform install above does not configure the extra surfaces they draw.

| Artifact | What it is | What this slice can say |
| --- | --- | --- |
| [Architecture system map](architecture.md) | A combined diagram of browsers, the CLI, nginx and Traefik, HTTP, LDAP and RADIUS listeners, redb or PostgreSQL, and external peers. | It is a map of the codebase's surfaces. Sections 9 and 10 document a directory client and an outbound SCIM target. Section 13 documents one LDAP provider listener. Those commands were not run, and the listener was not started. RADIUS and proxy listeners stay unset. The diagram was not run against this install. |
| [Q08 exact edition matrix](roadmap/q08-exact-edition-bundles.md) | A local build observation at source revision `4ca7558`, including native binary hashes and a note that Linux release files were absent. | It is not an observation of this worktree's revision. This task did not rebuild the matrix. |
| Release archives named by [deployment examples](deployment-examples.md) and [release notes](release-notes.md) | Linux native archives, maintenance archives, `riauthctl` archives, container archives, `SHA256SUMS`, and `build-provenance` files. | They were not downloaded, loaded, or started here. `deploy/compose-small.yml` and `deploy/compose-distributed.yml` remain documented image layouts for a later deployment. |
| `riauth capabilities` | Artifact catalog for the binary on `PATH`. | The first loopback record's `edition` was `essentials`, with `usable` null on every entry. The Platform-catalog record's `edition` was `platform`, `build_features` was `["essentials", "platform"]`, and `usable` was null on every entry. A configured instance's runtime report is a different document. Compiled Platform features are not configured features. |

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

Those five are manual gates. This task did not run the Playwright spec or a
spoken screen reader. Section 4 records one isolated desktop browser on
loopback. That browser signed in at `/apps` and cancelled the passkey prompt
before a credential existed. Section 14 records a second isolated desktop
browser that accepted an invitation password and did not start a passkey
ceremony. Physical keys, synced passkeys, phones, and
spoken screen readers remain manual gates. [Passkeys](passkeys.md) also says
physical hardware and platform compatibility still need testing on the
intended devices. The cancelled prompt in section 4 is one loopback
observation. A synced passkey and a security key used from Safari or a phone
remain manual checks for each deployment. The Platform admin page and
workflow editor are not part of that spec and were not checked here.

## What remains for D01

This page is the Platform half of the task guide through the third slice.
It does not finish the Platform guide, and it does not finish D01.

Sections 6 through 8 have two loopback observations. The Platform-catalog
run used the server CLI and skipped the printed `riauthctl` commands,
`group get`, and `group has-member`. The remote-administration run executed
`riauth-maintenance init`, server startup, server CLI login and `doctor`,
`riauthctl` login, the printed client create, `discovery`, `whoami`, the
printed group commands including `group get` and `group has-member`, the
printed client update, and the server CLI `explain` and audit commands.
`riauthctl doctor` exited 2 locally. Both of those chains skipped sections
4 and 5. A later run executed the section 5 entry points on the server
snapshot: `keygen`, `backup`, `restore` into a new directory, and `recovery
status`. That run left `recovery complete`, `recover-admin`, and a second
server unrun. A fifth run opened section 4 in an isolated browser, signed
in at `/apps`, and cancelled passkey enrollment before a credential
existed. Rename, remove, and passkey sign-in stayed unrun. A sixth run
issued one invitation through loopback SMTP, accepted the password in an
isolated browser, confirmed `/apps` stayed on the sign-in panel, refused a
replay, and then signed in as the invited person. Passkey acceptance,
sign-out, and the headless invitation journeys stayed unrun. A seventh run
executed section 11: `schema`, `validate`, `plan`, `apply`, a runtime
`[workflows.local-password]` restart, one configured start, and `export`.
The password step was not submitted. An eighth run executed section 9:
server CLI creation of local group `staff`, then `directory list`,
`directory plan`, and `directory apply` against a disposable loopback
OpenLDAP listener with `transport` `starttls`.
`removal_impact.review_required` was false, so apply omitted
`--confirm-removals`. A ninth run executed section 10: `provision targets`,
`provision plan`, `provision apply`, and `provision jobs` for target
`payroll`. Local group `staff` contained `admin` and `quinn`. The plan and
the fixture received only `quinn`, and the job reached `delivery_state`
`succeeded`. `removal_impact.review_required` was false, so that apply also
omitted `--confirm-removals`. Sections 12 and 13 remain source-reviewed
procedures. The generated `init` file contains empty `[directories]`,
`[workspace_directories]`, `[entra_directories]`, `[scim_targets]`,
`[ldap_listeners]`, `[proxy_listeners]`, `[radius_listeners]`, and
`[signers]` tables, and no directory id until the operator appends one. It
has no workflow table and no SAML client. Inbound SCIM, SAML, and LDAP
provider commands were not executed here. No named service provider, SaaS
directory, or upstream identity provider was contacted. The section 9
directory was the disposable loopback `slapd`, not a customer directory.
The section 10 peer was a disposable loopback SCIM fixture, not a customer
directory and not a conformance suite.

Still outside this slice, as later tasks:

- Email verification, password change and reset, authenticator-app
  enrollment, recovery codes, session list, and consent withdrawal as
  operator or user tasks. Section 14 records one invitation password
  acceptance. Passkey acceptance of an invitation, an expired link, and a
  revoked link were not run. The portal markup includes password, authenticator-app, and
  sessions controls beside passkeys ([index.html](../src/portal/index.html)).
  [Account email and recovery](lifecycle.md) describes those browser pages.
  [Re-enrollment and user communication](reenrollment.md) cites the source
  and browser-management tests for authenticator-app enrollment and
  recovery-code creation. This guide slice did not run those pages.
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
  the [SAML source recipe](recipes/platform-saml-source.md),
  and the [outbound SCIM recipe](recipes/platform-outbound-scim.md)
  are the D03 recipes written so far. The SAML source recipe follows
  the in-process `Upstream` helper, and no named external IdP is
  connected. The outbound SCIM recipe follows a second riAuth
  router on loopback HTTP, and no named SaaS directory is connected.
  The LDAP import recipe, the upstream OIDC recipe, and the outbound
  SCIM recipe cover both editions. The relying-party recipe is the
  authorization-code fixture, and its client is the in-tree router rather
  than a named application. The SAML IdP recipe follows the xmlsec1 fixture.
  A named service provider remains an open peer. SAML source remains a
  separate profile. The LDAP provider recipe is separate from LDAP import.
  Other D03 recipes and acceptance against category targets (D05) remain
  open. The [acceptance evidence matrix](roadmap/d05-acceptance-evidence.md)
  classifies the evidence that exists and leaves every category unpassed.
  The first D04 decision page is
  [operational recovery](operational-recovery.md); further emergency runbooks
  remain open. The [connector dependency incidents](connector-incidents.md) page covers
  read, stop, and retry decisions for external dependencies. The
  [administrator lockout](admin-lockout.md) page covers a serving store when
  a second human administrator can still sign in. The
  [deactivation delivery](deactivation-delivery.md) page covers incomplete or
  ambiguous outbound disables. The
  [SSF delivery](ssf-delivery.md) page covers stopped, retrying, and
  cancelled outbound Shared Signals deliveries. The backup commands here are
  entry points.
- Any claim that a person completed `cargo install`, the OIDC redirect,
  passkey enrollment, passkey rename or removal, passkey sign-in,
  `recovery complete`, `recover-admin`, a SAML metadata exchange with a peer,
  an LDAP provider bind, or an invitation passkey acceptance. One section 9
  LDAP plan and apply was run against a disposable loopback OpenLDAP
  listener. One section 10 SCIM plan and apply was run against a disposable
  loopback fixture, and its job reached `delivery_state` `succeeded`. One
  section 11 workflow plan, apply, and configured start was run, and its
  password step was not submitted. The
  [loopback record](roadmap/d01-platform-cli-walkthrough.md) has nine runs.
  The first supplies `riauth capabilities`, local `init`, `serve`, `/readyz`,
  CLI `login`, `doctor`, and one `local-demo` client create on a binary whose
  catalog edition was `essentials`. The second supplies the same setup on a
  binary whose catalog edition was `platform`, then server-CLI `local-demo`
  create, `staff` create, `group add-member`, the claim update, `explain`,
  and the three audit commands. The third supplies the printed remote
  `riauthctl` client, group, and claim commands, then server CLI `explain`
  and audit. The fourth supplies section 5 `keygen`, `backup`, `restore`,
  and `recovery status` on the Platform server snapshot, with the restored
  service left closed. The fifth supplies password sign-in at `/apps` and a
  cancelled **Add a passkey** on that same server snapshot. The server then
  listed no passkeys. The sixth supplies one loopback invitation, password
  acceptance, a signed-out applications page, replay refusal, and a later
  password sign-in. The seventh supplies section 11 schema, validate, plan,
  apply, the runtime password workflow, one configured start, and export on
  that same server snapshot. The eighth supplies section 9 `directory list`,
  `directory plan`, and `directory apply` for directory `staff` on that same
  server snapshot, against one disposable loopback OpenLDAP listener. The
  imported account was `alice` in group `staff`. The ninth supplies section
  10 `provision targets`, `provision plan`, `provision apply`, and `provision
  jobs` for target `payroll` on that same server snapshot, against one
  disposable loopback SCIM fixture. The exported account was `quinn` in group
  `staff`. D01 remains incomplete.
