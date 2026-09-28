# Browser first-administrator setup

An operator prepares ownership privately; the first administrator chooses their
username and password in the browser. Anonymous visitors cannot claim a fresh
server. This is the shared server baseline, with the same password, identity,
storage and recovery semantics as `riauth init`.

## Prepare ownership

Choose the intended issuer before setup. Create a private configuration file
and proof directory on the server. For a local evaluation:

```sh
umask 077
mkdir -p deployment-private
cat > riauth.toml <<'CONFIG'
issuer = "http://localhost:9000"
listen = "127.0.0.1:9000"
data_dir = "data"
access_token_ttl = 300
refresh_token_ttl = 2592000
session_ttl = 28800
CONFIG
riauth-maintenance prepare-setup --proof-file deployment-private/setup-proof
riauth serve
```

Preparation is a local maintenance command. It requires operator control of the
configuration, data directory/backend and output file. It refuses an initialized
database and never overwrites the private proof file. It returns the setup URL
and expiry, without printing the proof. The file has mode 0600 on Unix; protect
its parent directory and apply equivalent owner-only ACLs on other platforms.
The proof is 256 random bits, valid for 15 minutes by default; `--expires-in`
accepts 1–3600 seconds. Do not send it through logs, URLs or public channels.
Give it to the intended administrator through an operator-controlled private
channel. The proof file is not web content.

The pending listener must bind to loopback. For an HTTPS deployment, configure
the intended HTTPS issuer and native TLS or the trusted TLS reverse proxy before
preparing ownership. Access the loopback backend through that proxy or a secure
operator tunnel that preserves the exact configured origin and Host authority.
The proxy must preserve Host and Origin, serve setup/assets under the issuer
path, avoid request-body logging, and not add credentialed CORS. Do not replace
the issuer with a temporary public name or expose a plaintext setup listener.
Other identity protocols and background delivery workers start after setup.

Configure `database_key_file` and/or `postgres` before preparation to use the
existing encrypted storage or PostgreSQL backend. The pending verifier uses
that same backend and encryption configuration; there is no plaintext fallback.

## Complete setup in the browser

1. Open the setup URL, such as `http://localhost:9000/setup`. Opening, reloading,
   scanning or fetching assets never consumes the proof.
2. Paste the privately supplied proof. Choose an administrator username, display
   name, optional email and password, then confirm the password.
3. Select **Create administrator**. The ordinary password policy applies:
   12–1024 UTF-8 bytes. Email is not automatically verified. The current shared
   first-admin policy creates a password account without a pre-enrolled factor;
   this flow has the same rule and offers no passkey-only bypass.
4. Select **Continue to sign in** and sign in normally. Ownership alone creates
   no session or bearer credential. Browser login uses the portal's protected
   HttpOnly cookie. No operator restart is needed to use the account.

The browser clears proof/password fields after submission and on navigation;
it never puts them in URLs, browser storage or generated HTML. Setup uses JSON,
an exact Origin, the portal custom header and same-origin Fetch Metadata when
present. A cross-site form cannot initialize the instance. Responses are
uncacheable and cannot be framed.

Invalid and expired proofs receive the same message. Failed validation does
not consume ownership or leave a user/signing key behind. If the response is
lost, try ordinary sign-in first; setup POSTs are not automatically retried.
After success, replay and new preparation fail. To replace an expired or lost
unfinished proof, stop the pending service, rerun `prepare-setup` with a new
private output filename, and restart. This rotates the verifier for the same
instance and invalidates the old proof. Remove private proof files when finished.
Revisiting `/setup` shows the completed setup page, including immediately after
the live transition without restarting the service.

## Ownership and transaction boundary

Pending storage contains an instance ID, exact issuer, creation/expiry times
and a domain-separated SHA-256 proof verifier. It contains no raw proof,
administrator or signing key. Verification compares fixed-length digests in
constant time and requires the same instance, issuer, uninitialized state and
unexpired bounded lifetime. The persisted instance ID survives restart/rotation.

Ownership is checked before expensive credential work and again inside the
shared initialization writer. That transaction creates schema/index metadata,
issuer, signing key, dummy password hash, the administrator, username index,
password history and success audit, and deletes the pending verifier. All
effects commit together. The redb writer/PostgreSQL advisory-locked writer
selects one winner. A losing HTTP/node cannot create another administrator.
Runtime activation belongs to the worker, so a disconnected HTTP caller cannot
strand a committed instance in setup mode. Other pending PostgreSQL nodes check
shared storage and activate the ordinary router after a winner commits.

`riauth init`, `Core::initialize`, `Core::open`, `api::router` and `api::serve`
retain their public signatures. The shared initialization writer consumes any
pending verifier in the same commit. The CLI retains its refusal to overwrite
an existing configuration/data file. Offline emergency `recover-admin` semantics
are unchanged.
Restoring an old unfinished database can restore its still-unexpired ownership
verifier; this follows the existing snapshot rollback boundary. Fence writers
and rotate unfinished ownership after such a restore. There is no external
anti-rollback claim.

## Verification and integration

Run HTTP and process contracts:

```sh
CARGO_BUILD_JOBS=2 cargo test --locked --test bootstrap --test bootstrap_cli
```

The HTTP tests cover plaintext/encrypted redb, protected operator output,
success and cookie sign-in, wrong/expired/instance/issuer proof, Origin/Host/
Fetch Metadata/JSON checks, replay, scanner GET, password-policy rollback,
concurrent winner and pending/initialized restart. An opt-in disposable
PostgreSQL test covers two independent HTTP nodes, one atomic winner and restart:

```sh
RIAUTH_TEST_BOOTSTRAP_PG_CONNECTION=/private/disposable-pg-connection \
CARGO_BUILD_JOBS=2 cargo test --locked --test bootstrap \
  postgres_http_nodes -- --ignored
```

The connection must target a dedicated empty disposable database. The test
initializes it and deliberately tries a mismatched encryption configuration.
Do not point it at an existing deployment. CI also runs the rendered setup
journey in Chromium, Firefox and WebKit from `tools/browser/setup.spec.js`. It
covers a private proof claim and ordinary sign-in, denial of a visitor without
the proof, expiry, and the closed page after a replay. The setup URL carries no
proof; the private proof must be entered in the form. Run the browser test with
the built `target/debug/riauth` binary:

```sh
CARGO_BUILD_JOBS=2 cargo build --locked --bin riauth
npm ci --prefix tools/browser
cd tools/browser
./node_modules/.bin/playwright install chromium firefox webkit
./node_modules/.bin/playwright test setup.spec.js --reporter=list
```

Network traces and failure screenshots are disabled for this test because they
can contain ownership data.
For manual desktop verification in RiWork, use Cua.ai Driver MCP exclusively;
report unavailable browser routes/permissions through RiWork's Cua setup.

Architecture integration must keep this pending runtime and ownership transition
in the shared server authority for both Essentials and Platform. Local
`prepare-setup` belongs with operator maintenance when `riauthctl` is separated;
it must not become an anonymous remote endpoint. Q02/Q05 should adapt these
contracts to the finalized builds/backend harness. Local HTTPS reverse-proxy
testing has covered the Chrome setup journey and verified-TLS guard, race and
expiry cases; the headless browser matrix has covered Chromium, Firefox and
WebKit over loopback HTTP. Native TLS, production proxy/trust configurations,
and the official Essentials/Platform build and platform matrix still require
separate acceptance evidence. A05/A06 must include the same bootstrap authority
in both server builds and verify parity; this baseline does not claim those
integrations.
